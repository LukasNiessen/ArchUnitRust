use std::path::Path;

use crate::{
    ArchUnitError, CheckOptions, CountMetric, Filter, MetricMeasurement, PatternError,
    ProjectLocator, ProjectMetricsInfo, RegexFactory, SourceOptions, UserError,
    extract_project_metrics, locate_project_from,
};

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
    file_filters: Vec<Filter>,
    type_filters: Vec<Filter>,
    configuration_error: Option<MetricsConfigurationError>,
}

impl MetricsBuilder {
    const fn new(project_locator: ProjectLocator) -> Self {
        Self {
            project_locator,
            file_filters: Vec::new(),
            type_filters: Vec::new(),
            configuration_error: None,
        }
    }

    /// Keeps source files whose final path segment matches `pattern`.
    pub fn with_name(mut self, pattern: impl AsRef<str>) -> Self {
        match RegexFactory::default().filename_matcher(pattern) {
            Ok(filter) => self.file_filters.push(filter),
            Err(source) => self.record_pattern_error("with_name", source),
        }
        self
    }

    /// Keeps source files whose containing folder matches `pattern`.
    pub fn in_folder(mut self, pattern: impl AsRef<str>) -> Self {
        match RegexFactory::default().folder_matcher(pattern) {
            Ok(filter) => self.file_filters.push(filter),
            Err(source) => self.record_pattern_error("in_folder", source),
        }
        self
    }

    /// Keeps source files whose normalized project path matches `pattern`.
    pub fn in_path(mut self, pattern: impl AsRef<str>) -> Self {
        match RegexFactory::default().path_matcher(pattern) {
            Ok(filter) => self.file_filters.push(filter),
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
            Ok(filter) => self.type_filters.push(filter),
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
        if let Some(error) = &self.configuration_error {
            return Err(configuration_error(error.clone()));
        }

        let project = locate_project_from(&self.project_locator)?;
        let options = SourceOptions::new().with_dev_targets(check_options.includes_test_sources());
        let extracted = extract_project_metrics(&project, options)?;
        Ok(self.select(extracted))
    }

    /// Enters the built-in count metric family.
    pub fn count(self) -> CountMetricsBuilder {
        CountMetricsBuilder { query: self }
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
            .filter(|file| {
                self.file_filters
                    .iter()
                    .all(|filter| filter.matches(file.path()))
            })
            .cloned()
            .collect::<Vec<_>>();

        if !self.type_filters.is_empty() {
            for file in &mut files {
                file.retain_types(|type_info| {
                    self.type_filters
                        .iter()
                        .all(|filter| filter.matches(type_info.name()))
                });
            }
            files.retain(|file| !file.types().is_empty());
        }
        ProjectMetricsInfo::from_files(root, files)
    }

    fn record_pattern_error(&mut self, selector: &'static str, source: PatternError) {
        if self.configuration_error.is_none() {
            self.configuration_error =
                Some(MetricsConfigurationError::InvalidPattern { selector, source });
        }
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
}

#[derive(Debug, Clone, thiserror::Error)]
enum MetricsConfigurationError {
    #[error("invalid {selector} pattern: {source}")]
    InvalidPattern {
        selector: &'static str,
        #[source]
        source: PatternError,
    },
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
    use crate::{ArchUnitError, CountMetric, ProjectLocator};

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
}
