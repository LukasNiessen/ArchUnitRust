use std::path::Path;

use crate::{
    ArchUnitError, CheckOptions, CountMetric, DistanceInfo, DistanceMetric, Filter, LcomMetric,
    MetricMeasurement, MetricSubject, PatternError, PatternTarget, ProjectLocator,
    ProjectMetricsInfo, RegexFactory, SourceOptions, TypeInfo, UserError, extract_distance_infos,
    extract_project_metrics, locate_project_from,
};

use super::{
    CustomMetricCondition, MetricPredicateCondition, MetricThresholdCondition, MetricZoneCondition,
};

macro_rules! metric_threshold_methods {
    () => {
        /// Requires every measured value to be strictly below `threshold`.
        pub fn should_be_below(self, threshold: f64) -> MetricThresholdCondition<Self> {
            MetricThresholdCondition::new(self, crate::MetricComparison::Below, threshold)
        }

        /// Requires every measured value to be strictly above `threshold`.
        pub fn should_be_above(self, threshold: f64) -> MetricThresholdCondition<Self> {
            MetricThresholdCondition::new(self, crate::MetricComparison::Above, threshold)
        }

        /// Requires every measured value to equal `threshold` exactly.
        pub fn should_be(self, threshold: f64) -> MetricThresholdCondition<Self> {
            MetricThresholdCondition::new(self, crate::MetricComparison::Equal, threshold)
        }

        /// Requires every measured value to be below or equal to `threshold`.
        pub fn should_be_below_or_equal(self, threshold: f64) -> MetricThresholdCondition<Self> {
            MetricThresholdCondition::new(self, crate::MetricComparison::BelowOrEqual, threshold)
        }

        /// Requires every measured value to be above or equal to `threshold`.
        pub fn should_be_above_or_equal(self, threshold: f64) -> MetricThresholdCondition<Self> {
            MetricThresholdCondition::new(self, crate::MetricComparison::AboveOrEqual, threshold)
        }
    };
}

/// Starts a metrics query with Cargo discovery at the working directory.
pub fn metrics() -> MetricsBuilder {
    MetricsBuilder::new(ProjectLocator::auto_detect())
}

/// Starts a metrics query with explicit Cargo project discovery.
pub fn metrics_in(locator: impl Into<ProjectLocator>) -> MetricsBuilder {
    MetricsBuilder::new(locator.into())
}

/// Immutable file/type selection for metrics extraction.
#[derive(Debug, Clone)]
#[must_use = "a metrics query has no effect until analyze or measure is called"]
pub struct MetricsBuilder {
    project_locator: ProjectLocator,
    filters: Vec<Filter>,
    configuration_error: Option<MetricsConfigurationError>,
}

impl MetricsBuilder {
    const fn new(project_locator: ProjectLocator) -> Self {
        Self {
            project_locator,
            filters: Vec::new(),
            configuration_error: None,
        }
    }

    /// Keeps source files whose final path segment matches `pattern`.
    pub fn with_name(mut self, pattern: impl AsRef<str>) -> Self {
        match RegexFactory::default().filename_matcher(pattern) {
            Ok(filter) => self.filters.push(filter),
            Err(source) => self.record_pattern_error("with_name", source),
        }
        self
    }

    /// Keeps source files whose containing folder matches `pattern`.
    pub fn in_folder(mut self, pattern: impl AsRef<str>) -> Self {
        match RegexFactory::default().folder_matcher(pattern) {
            Ok(filter) => self.filters.push(filter),
            Err(source) => self.record_pattern_error("in_folder", source),
        }
        self
    }

    /// Keeps source files whose normalized project path matches `pattern`.
    pub fn in_path(mut self, pattern: impl AsRef<str>) -> Self {
        match RegexFactory::default().path_matcher(pattern) {
            Ok(filter) => self.filters.push(filter),
            Err(source) => self.record_pattern_error("in_path", source),
        }
        self
    }

    /// Keeps Rust type declarations whose unqualified name matches `pattern`.
    ///
    /// When present, files without a matching type are omitted. File-level type and trait counts
    /// describe only the retained declarations; source-wide counts such as lines and imports remain
    /// properties of the containing file.
    pub fn for_types_matching(mut self, pattern: impl AsRef<str>) -> Self {
        match RegexFactory::default().type_name_matcher(pattern) {
            Ok(filter) => self.filters.push(filter),
            Err(source) => self.record_pattern_error("for_types_matching", source),
        }
        self
    }

    /// Extracts the selected information model with production-source defaults.
    pub fn analyze(&self) -> Result<ProjectMetricsInfo, ArchUnitError> {
        self.analyze_with(&CheckOptions::default())
    }

    /// Extracts the selected information model with explicit source-target options.
    pub fn analyze_with(
        &self,
        check_options: &CheckOptions,
    ) -> Result<ProjectMetricsInfo, ArchUnitError> {
        self.validate_configuration()?;

        let project = locate_project_from(&self.project_locator)?;
        let options = SourceOptions::new().with_dev_targets(check_options.includes_test_sources());
        let extracted = extract_project_metrics(&project, options)?;
        Ok(self.select(extracted))
    }

    /// Enters the built-in count metric family.
    pub fn count(self) -> CountMetricsBuilder {
        CountMetricsBuilder { query: self }
    }

    /// Enters the lack-of-cohesion metric family for eligible Rust structs.
    #[must_use = "choose an LCOM formula before measuring"]
    pub fn lcom(self) -> LcomMetricsBuilder {
        LcomMetricsBuilder { query: self }
    }

    /// Enters the file-component distance metric family.
    #[must_use = "choose a distance formula or zone check before executing"]
    pub fn distance(self) -> DistanceMetricsBuilder {
        DistanceMetricsBuilder { query: self }
    }

    /// Defines a reusable numeric metric over every selected Rust type.
    ///
    /// The calculation receives an immutable [`TypeInfo`] and is invoked once per type on each
    /// execution. Panics from the user callback propagate normally.
    pub fn custom_metric<Calculation>(
        mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        calculation: Calculation,
    ) -> CustomMetricSelection<Calculation>
    where
        Calculation: Fn(&TypeInfo) -> f64,
    {
        let name = name.into();
        let description = description.into();
        if name.trim().is_empty() {
            self.record_custom_metric_error(MetricsConfigurationError::EmptyCustomMetricName);
        } else if description.trim().is_empty() {
            self.record_custom_metric_error(
                MetricsConfigurationError::EmptyCustomMetricDescription,
            );
        }
        CustomMetricSelection {
            query: self,
            name,
            description,
            calculation,
        }
    }

    /// Returns the explicit discovery path, or `None` for automatic discovery.
    #[must_use]
    pub fn project_path(&self) -> Option<&Path> {
        self.project_locator.path()
    }

    fn select(&self, project: ProjectMetricsInfo) -> ProjectMetricsInfo {
        let root = project.root().to_path_buf();
        let mut files = project
            .files()
            .iter()
            .filter(|file| self.file_matches(file.path()))
            .cloned()
            .collect::<Vec<_>>();

        if self.has_type_filters() {
            for file in &mut files {
                file.retain_types(|type_info| self.type_matches(type_info.name()));
            }
            files.retain(|file| !file.types().is_empty());
        }
        ProjectMetricsInfo::from_files(root, files)
    }

    pub(super) fn distance_infos_with(
        &self,
        check_options: &CheckOptions,
    ) -> Result<Vec<DistanceInfo>, ArchUnitError> {
        self.validate_configuration()?;

        let project = locate_project_from(&self.project_locator)?;
        let mut infos = extract_distance_infos(&project, check_options)?;
        infos.retain(|info| {
            self.file_matches(info.identifier())
                && (!self.has_type_filters()
                    || info
                        .file()
                        .types()
                        .iter()
                        .any(|type_info| self.type_matches(type_info.name())))
        });
        Ok(infos)
    }

    pub(super) fn filters(&self) -> &[Filter] {
        &self.filters
    }

    pub(super) fn validate_configuration(&self) -> Result<(), ArchUnitError> {
        if let Some(error) = &self.configuration_error {
            Err(configuration_error(error.clone()))
        } else {
            Ok(())
        }
    }

    fn record_custom_metric_error(&mut self, error: MetricsConfigurationError) {
        if self.configuration_error.is_none() {
            self.configuration_error = Some(error);
        }
    }

    fn file_matches(&self, path: &str) -> bool {
        self.filters
            .iter()
            .filter(|filter| filter.target() != PatternTarget::TypeName)
            .all(|filter| filter.matches(path))
    }

    fn type_matches(&self, name: &str) -> bool {
        self.filters
            .iter()
            .filter(|filter| filter.target() == PatternTarget::TypeName)
            .all(|filter| filter.matches(name))
    }

    fn has_type_filters(&self) -> bool {
        self.filters
            .iter()
            .any(|filter| filter.target() == PatternTarget::TypeName)
    }

    fn record_pattern_error(&mut self, selector: &'static str, source: PatternError) {
        if self.configuration_error.is_none() {
            self.configuration_error =
                Some(MetricsConfigurationError::InvalidPattern { selector, source });
        }
    }
}

/// A user-defined type metric that can be measured or turned into a predicate rule.
#[derive(Debug, Clone)]
#[must_use = "a custom metric has no effect until measure or should_satisfy is called"]
pub struct CustomMetricSelection<Calculation> {
    query: MetricsBuilder,
    name: String,
    description: String,
    calculation: Calculation,
}

impl<Calculation> CustomMetricSelection<Calculation>
where
    Calculation: Fn(&TypeInfo) -> f64,
{
    metric_threshold_methods!();

    /// Extracts and measures selected production-source types.
    ///
    /// Panics from the custom calculation propagate normally.
    pub fn measure(&self) -> Result<Vec<MetricMeasurement>, ArchUnitError> {
        self.measure_with(&CheckOptions::default())
    }

    /// Extracts and measures selected types with explicit source-target options.
    ///
    /// Panics from the custom calculation propagate normally.
    pub fn measure_with(
        &self,
        check_options: &CheckOptions,
    ) -> Result<Vec<MetricMeasurement>, ArchUnitError> {
        let types = self.selected_types_with(check_options)?;
        Ok(types
            .into_iter()
            .map(|type_info| {
                let value = (self.calculation)(&type_info);
                MetricMeasurement::from_parts(
                    MetricSubject::Type(type_info),
                    &self.name,
                    &self.description,
                    value,
                )
            })
            .collect())
    }

    /// Creates a reusable rule whose predicate receives the value and the same type evidence.
    ///
    /// Panics from either callback propagate normally.
    pub fn should_satisfy<Predicate>(
        self,
        predicate: Predicate,
    ) -> CustomMetricCondition<Calculation, Predicate>
    where
        Predicate: Fn(f64, &TypeInfo) -> bool,
    {
        CustomMetricCondition::new(self, predicate)
    }

    /// Returns the user-defined metric name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the user-defined metric description.
    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }

    pub(super) fn selected_types_with(
        &self,
        check_options: &CheckOptions,
    ) -> Result<Vec<TypeInfo>, ArchUnitError> {
        Ok(self.query.analyze_with(check_options)?.types().to_vec())
    }

    pub(super) fn calculation(&self) -> &Calculation {
        &self.calculation
    }

    pub(super) fn filters(&self) -> &[Filter] {
        self.query.filters()
    }

    pub(super) fn subject_label(&self) -> &'static str {
        "metric types"
    }

    pub(super) fn validate_configuration(&self) -> Result<(), ArchUnitError> {
        self.query.validate_configuration()
    }
}

/// Distance formula and architectural-zone choices over file components.
#[derive(Debug, Clone)]
#[must_use = "choose a distance formula or zone check before executing"]
pub struct DistanceMetricsBuilder {
    query: MetricsBuilder,
}

impl DistanceMetricsBuilder {
    /// Selects trait-declaration abstractness.
    pub fn abstractness(self) -> DistanceMetricSelection {
        self.select(DistanceMetric::Abstractness)
    }

    /// Selects incoming/outgoing dependency instability.
    pub fn instability(self) -> DistanceMetricSelection {
        self.select(DistanceMetric::Instability)
    }

    /// Selects absolute distance from the main sequence.
    pub fn distance_from_main_sequence(self) -> DistanceMetricSelection {
        self.select(DistanceMetric::DistanceFromMainSequence)
    }

    /// Selects bidirectional internal coupling density.
    pub fn coupling_factor(self) -> DistanceMetricSelection {
        self.select(DistanceMetric::CouplingFactor)
    }

    /// Selects size-discounted distance from the main sequence.
    pub fn normalized_distance(self) -> DistanceMetricSelection {
        self.select(DistanceMetric::NormalizedDistance)
    }

    /// Rejects components with low abstractness and low instability.
    pub fn not_in_zone_of_pain(self) -> MetricZoneCondition {
        MetricZoneCondition::new(self.query, crate::ArchitecturalZone::Pain)
    }

    /// Rejects components with high abstractness and high instability.
    pub fn not_in_zone_of_uselessness(self) -> MetricZoneCondition {
        MetricZoneCondition::new(self.query, crate::ArchitecturalZone::Uselessness)
    }

    fn select(self, metric: DistanceMetric) -> DistanceMetricSelection {
        DistanceMetricSelection {
            query: self.query,
            metric,
        }
    }
}

/// One selected component-distance formula ready for extraction and measurement.
#[derive(Debug, Clone)]
#[must_use = "a distance selection has no effect until measure is called"]
pub struct DistanceMetricSelection {
    query: MetricsBuilder,
    metric: DistanceMetric,
}

impl DistanceMetricSelection {
    metric_threshold_methods!();

    /// Extracts and measures production-source components.
    pub fn measure(&self) -> Result<Vec<MetricMeasurement>, ArchUnitError> {
        self.measure_with(&CheckOptions::default())
    }

    /// Extracts and measures components with explicit source-target options.
    pub fn measure_with(
        &self,
        check_options: &CheckOptions,
    ) -> Result<Vec<MetricMeasurement>, ArchUnitError> {
        let infos = self.query.distance_infos_with(check_options)?;
        Ok(self.metric.measurements(&infos))
    }

    /// Returns the selected distance formula.
    #[must_use]
    pub const fn metric(&self) -> DistanceMetric {
        self.metric
    }

    /// Creates a reusable predicate over each value and complete distance subject.
    pub fn should_satisfy<Predicate>(
        self,
        predicate: Predicate,
    ) -> MetricPredicateCondition<Self, Predicate>
    where
        Predicate: Fn(f64, &MetricSubject) -> bool,
    {
        MetricPredicateCondition::new(self, predicate)
    }

    pub(super) fn filters(&self) -> &[Filter] {
        self.query.filters()
    }

    pub(super) fn subject_label(&self) -> &'static str {
        "metric components"
    }

    pub(super) fn validate_configuration(&self) -> Result<(), ArchUnitError> {
        self.query.validate_configuration()
    }
}

/// LCOM formula choices over eligible structs in one immutable selection.
#[derive(Debug, Clone)]
#[must_use = "choose an LCOM formula before measuring"]
pub struct LcomMetricsBuilder {
    query: MetricsBuilder,
}

impl LcomMetricsBuilder {
    /// Selects normalized method/field distance under the LCOM96a name.
    pub fn lcom96a(self) -> LcomMetricSelection {
        self.select(LcomMetric::Lcom96a)
    }

    /// Selects the method/field density complement under the LCOM96b name.
    pub fn lcom96b(self) -> LcomMetricSelection {
        self.select(LcomMetric::Lcom96b)
    }

    /// Selects non-sharing minus sharing method pairs.
    pub fn lcom1(self) -> LcomMetricSelection {
        self.select(LcomMetric::Lcom1)
    }

    /// Selects the method/field density complement.
    pub fn lcom2(self) -> LcomMetricSelection {
        self.select(LcomMetric::Lcom2)
    }

    /// Selects normalized method/field distance.
    pub fn lcom3(self) -> LcomMetricSelection {
        self.select(LcomMetric::Lcom3)
    }

    /// Selects shared-field method graph connected components.
    pub fn lcom4(self) -> LcomMetricSelection {
        self.select(LcomMetric::Lcom4)
    }

    /// Selects normalized Henderson-Sellers method/field distance.
    pub fn lcom5(self) -> LcomMetricSelection {
        self.select(LcomMetric::Lcom5)
    }

    /// Selects normalized LCOM-star method/field distance.
    pub fn lcom_star(self) -> LcomMetricSelection {
        self.select(LcomMetric::LcomStar)
    }

    fn select(self, metric: LcomMetric) -> LcomMetricSelection {
        LcomMetricSelection {
            query: self.query,
            metric,
        }
    }
}

/// One selected LCOM formula ready for extraction and measurement.
#[derive(Debug, Clone)]
#[must_use = "an LCOM selection has no effect until measure is called"]
pub struct LcomMetricSelection {
    query: MetricsBuilder,
    metric: LcomMetric,
}

impl LcomMetricSelection {
    metric_threshold_methods!();

    /// Extracts and measures eligible production-source structs.
    pub fn measure(&self) -> Result<Vec<MetricMeasurement>, ArchUnitError> {
        self.measure_with(&CheckOptions::default())
    }

    /// Extracts and measures eligible structs with explicit source-target options.
    pub fn measure_with(
        &self,
        check_options: &CheckOptions,
    ) -> Result<Vec<MetricMeasurement>, ArchUnitError> {
        let project = self.query.analyze_with(check_options)?;
        Ok(self.metric.measurements(&project))
    }

    /// Returns the selected LCOM formula.
    #[must_use]
    pub const fn metric(&self) -> LcomMetric {
        self.metric
    }

    /// Creates a reusable predicate over each value and complete type subject.
    pub fn should_satisfy<Predicate>(
        self,
        predicate: Predicate,
    ) -> MetricPredicateCondition<Self, Predicate>
    where
        Predicate: Fn(f64, &MetricSubject) -> bool,
    {
        MetricPredicateCondition::new(self, predicate)
    }

    pub(super) fn filters(&self) -> &[Filter] {
        self.query.filters()
    }

    pub(super) fn subject_label(&self) -> &'static str {
        "metric types"
    }

    pub(super) fn validate_configuration(&self) -> Result<(), ArchUnitError> {
        self.query.validate_configuration()
    }
}

/// Built-in count choices over one immutable selection.
#[derive(Debug, Clone)]
#[must_use = "choose a count metric before measuring"]
pub struct CountMetricsBuilder {
    query: MetricsBuilder,
}

impl CountMetricsBuilder {
    /// Counts methods with a `self` receiver for every selected type.
    pub fn method_count(self) -> MetricSelection {
        self.select(CountMetric::MethodCount)
    }

    /// Counts declared data fields for every selected type.
    pub fn field_count(self) -> MetricSelection {
        self.select(CountMetric::FieldCount)
    }

    /// Counts physical non-comment source lines for every selected file.
    pub fn lines_of_code(self) -> MetricSelection {
        self.select(CountMetric::LinesOfCode)
    }

    /// Counts syntax-tree items and executable statements for every selected file.
    pub fn statements(self) -> MetricSelection {
        self.select(CountMetric::Statements)
    }

    /// Counts `use` and `extern crate` items for every selected file.
    pub fn imports(self) -> MetricSelection {
        self.select(CountMetric::Imports)
    }

    /// Counts structs, enums, and unions for every selected file.
    pub fn concrete_types(self) -> MetricSelection {
        self.select(CountMetric::ConcreteTypes)
    }

    /// Counts free functions for every selected file.
    pub fn functions(self) -> MetricSelection {
        self.select(CountMetric::Functions)
    }

    /// Counts trait declarations for every selected file.
    pub fn traits(self) -> MetricSelection {
        self.select(CountMetric::Traits)
    }

    /// Counts inherent and trait impl blocks for every selected file.
    pub fn impl_blocks(self) -> MetricSelection {
        self.select(CountMetric::ImplBlocks)
    }

    /// Counts macro invocations and definitions represented by syntax for every selected file.
    pub fn macros(self) -> MetricSelection {
        self.select(CountMetric::Macros)
    }

    /// Counts receiver-free functions in traits and impl blocks for every selected file.
    pub fn associated_functions(self) -> MetricSelection {
        self.select(CountMetric::AssociatedFunctions)
    }

    fn select(self, metric: CountMetric) -> MetricSelection {
        MetricSelection {
            query: self.query,
            metric,
        }
    }
}

/// One selected count metric ready for extraction and measurement.
#[derive(Debug, Clone)]
#[must_use = "a metric selection has no effect until measure is called"]
pub struct MetricSelection {
    query: MetricsBuilder,
    metric: CountMetric,
}

impl MetricSelection {
    metric_threshold_methods!();

    /// Extracts and measures with production-source defaults.
    pub fn measure(&self) -> Result<Vec<MetricMeasurement>, ArchUnitError> {
        self.measure_with(&CheckOptions::default())
    }

    /// Extracts and measures with explicit source-target options.
    pub fn measure_with(
        &self,
        check_options: &CheckOptions,
    ) -> Result<Vec<MetricMeasurement>, ArchUnitError> {
        let project = self.query.analyze_with(check_options)?;
        Ok(self.metric.measurements(&project))
    }

    /// Returns the selected built-in metric.
    #[must_use]
    pub const fn metric(&self) -> CountMetric {
        self.metric
    }

    /// Creates a reusable predicate over each value and complete file or type subject.
    pub fn should_satisfy<Predicate>(
        self,
        predicate: Predicate,
    ) -> MetricPredicateCondition<Self, Predicate>
    where
        Predicate: Fn(f64, &MetricSubject) -> bool,
    {
        MetricPredicateCondition::new(self, predicate)
    }

    pub(super) fn filters(&self) -> &[Filter] {
        self.query.filters()
    }

    pub(super) fn subject_label(&self) -> &'static str {
        if matches!(
            self.metric,
            CountMetric::MethodCount | CountMetric::FieldCount
        ) {
            "metric types"
        } else {
            "metric files"
        }
    }

    pub(super) fn validate_configuration(&self) -> Result<(), ArchUnitError> {
        self.query.validate_configuration()
    }
}

#[derive(Debug, Clone, thiserror::Error)]
enum MetricsConfigurationError {
    #[error("invalid {selector} pattern: {source}")]
    InvalidPattern {
        selector: &'static str,
        #[source]
        source: PatternError,
    },
    #[error("custom metric name must not be empty")]
    EmptyCustomMetricName,
    #[error("custom metric description must not be empty")]
    EmptyCustomMetricDescription,
}

fn configuration_error(error: MetricsConfigurationError) -> ArchUnitError {
    ArchUnitError::from(UserError::with_source(
        "the metrics query is invalid",
        error,
    ))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{metrics, metrics_in};
    use crate::{
        ArchUnitError, ArchitecturalZone, CountMetric, DistanceMetric, LcomMetric, ProjectLocator,
    };

    #[test]
    fn builders_are_consuming_cloneable_and_branchable() {
        let base = metrics_in(ProjectLocator::from_path("fixture"));
        let files = base.clone().in_path("src/**").with_name("*.rs");
        let types = files.clone().for_types_matching("*Service");

        assert_eq!(base.project_path(), Some(Path::new("fixture")));
        assert_eq!(types.project_path(), Some(Path::new("fixture")));
        assert_eq!(
            types.clone().count().method_count().metric(),
            CountMetric::MethodCount
        );
        assert_eq!(
            types.count().field_count().metric(),
            CountMetric::FieldCount
        );
    }

    #[test]
    fn invalid_patterns_fail_before_automatic_project_discovery() {
        let error = metrics()
            .in_folder("src/[")
            .for_types_matching("*")
            .analyze()
            .expect_err("invalid configuration should fail");

        assert!(matches!(error, ArchUnitError::User(_)));
        assert!(error.to_string().contains("invalid in_folder pattern"));
    }

    #[test]
    fn every_count_terminal_retains_the_expected_metric() {
        let query = metrics();
        let cases = [
            (
                query.clone().count().method_count(),
                CountMetric::MethodCount,
            ),
            (query.clone().count().field_count(), CountMetric::FieldCount),
            (
                query.clone().count().lines_of_code(),
                CountMetric::LinesOfCode,
            ),
            (query.clone().count().statements(), CountMetric::Statements),
            (query.clone().count().imports(), CountMetric::Imports),
            (
                query.clone().count().concrete_types(),
                CountMetric::ConcreteTypes,
            ),
            (query.clone().count().functions(), CountMetric::Functions),
            (query.clone().count().traits(), CountMetric::Traits),
            (query.clone().count().impl_blocks(), CountMetric::ImplBlocks),
            (query.clone().count().macros(), CountMetric::Macros),
            (
                query.count().associated_functions(),
                CountMetric::AssociatedFunctions,
            ),
        ];

        for (selection, expected) in cases {
            assert_eq!(selection.metric(), expected);
        }
    }

    #[test]
    fn every_lcom_terminal_retains_the_expected_metric() {
        let query = metrics();
        let cases = [
            (query.clone().lcom().lcom96a(), LcomMetric::Lcom96a),
            (query.clone().lcom().lcom96b(), LcomMetric::Lcom96b),
            (query.clone().lcom().lcom1(), LcomMetric::Lcom1),
            (query.clone().lcom().lcom2(), LcomMetric::Lcom2),
            (query.clone().lcom().lcom3(), LcomMetric::Lcom3),
            (query.clone().lcom().lcom4(), LcomMetric::Lcom4),
            (query.clone().lcom().lcom5(), LcomMetric::Lcom5),
            (query.lcom().lcom_star(), LcomMetric::LcomStar),
        ];

        for (selection, expected) in cases {
            assert_eq!(selection.metric(), expected);
        }
    }

    #[test]
    fn every_distance_terminal_retains_the_expected_formula_or_zone() {
        let query = metrics();
        let metrics = [
            (
                query.clone().distance().abstractness().metric(),
                DistanceMetric::Abstractness,
            ),
            (
                query.clone().distance().instability().metric(),
                DistanceMetric::Instability,
            ),
            (
                query
                    .clone()
                    .distance()
                    .distance_from_main_sequence()
                    .metric(),
                DistanceMetric::DistanceFromMainSequence,
            ),
            (
                query.clone().distance().coupling_factor().metric(),
                DistanceMetric::CouplingFactor,
            ),
            (
                query.clone().distance().normalized_distance().metric(),
                DistanceMetric::NormalizedDistance,
            ),
        ];

        for (actual, expected) in metrics {
            assert_eq!(actual, expected);
        }
        assert_eq!(
            query.clone().distance().not_in_zone_of_pain().zone(),
            ArchitecturalZone::Pain
        );
        assert_eq!(
            query.distance().not_in_zone_of_uselessness().zone(),
            ArchitecturalZone::Uselessness
        );
    }
}
