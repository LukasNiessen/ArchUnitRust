//! Closed aggregation of violation data owned by the individual rule domains.

use std::fmt;

use crate::common::assertion::EmptyTestViolation;
use crate::files::assertion::{
    CustomFileViolation, CycleViolation, ExternalModuleDependencyViolation,
    FileDependencyViolation, FilePatternViolation,
};
use crate::layers::assertion::LayerDependencyViolation;
use crate::metrics::assertion::{
    CustomMetricViolation, MetricPredicateViolation, MetricThresholdViolation, MetricZoneViolation,
};
use crate::slices::assertion::SliceDependencyViolation;

/// The machine-readable family of a [`Violation`].
///
/// Spellings are shared across ArchUnit ports and are stable report keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ViolationKind {
    /// A selector matched no subject, so the rule judged nothing.
    EmptyTest,
    /// The selected projected graph contains a circular dependency path.
    Cycle,
    /// A selected file disagrees with a filename, folder, or path pattern.
    FilePattern,
    /// An internal file dependency disagrees with an allowlist or denylist rule.
    FileDependency,
    /// An external crate dependency disagrees with an allowlist or denylist rule.
    ExternalModuleDependency,
    /// A selected file disagrees with a user-defined predicate.
    CustomFile,
    /// An internal dependency disagrees with a named-layer policy.
    LayerDependency,
    /// A projected dependency disagrees with a slice policy.
    SliceDependency,
    /// A file component lies in a discouraged abstractness/instability zone.
    MetricZone,
    /// A user-defined metric value did not satisfy its predicate.
    CustomMetric,
    /// A metric value did not meet an exact numeric threshold.
    MetricThreshold,
    /// A built-in metric value did not satisfy a user predicate.
    MetricPredicate,
}

impl ViolationKind {
    /// Returns the stable lowercase, hyphen-separated report key.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EmptyTest => "empty-test",
            Self::Cycle => "cycle",
            Self::FilePattern => "file-pattern",
            Self::FileDependency => "file-dependency",
            Self::ExternalModuleDependency => "external-module-dependency",
            Self::CustomFile => "custom-file",
            Self::LayerDependency => "layer-dependency",
            Self::SliceDependency => "slice-dependency",
            Self::MetricZone => "metric-zone",
            Self::CustomMetric => "custom-metric",
            Self::MetricThreshold => "metric-threshold",
            Self::MetricPredicate => "metric-predicate",
        }
    }
}

impl fmt::Display for ViolationKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// One data-carrying disagreement between a project and an architecture rule.
///
/// A complete rule result is `Vec<Violation>`: an empty vector means the rule passed. This enum
/// deliberately contains no user-facing prose; the testing layer formats each variant later.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Violation {
    /// The rule selected no subject and therefore could not judge its predicate.
    EmptyTest(EmptyTestViolation),
    /// A circular path exists in the selected projected graph.
    Cycle(CycleViolation),
    /// A file disagrees with a name or location predicate.
    FilePattern(FilePatternViolation),
    /// An internal dependency disagrees with a relational file rule.
    FileDependency(FileDependencyViolation),
    /// An external crate dependency disagrees with a relational file rule.
    ExternalModuleDependency(ExternalModuleDependencyViolation),
    /// A file disagrees with a user-defined predicate.
    CustomFile(CustomFileViolation),
    /// An internal dependency disagrees with a named-layer policy.
    LayerDependency(LayerDependencyViolation),
    /// A projected dependency disagrees with a slice policy.
    SliceDependency(SliceDependencyViolation),
    /// A file component lies inside a discouraged metrics zone.
    MetricZone(MetricZoneViolation),
    /// A user-defined type metric did not satisfy its predicate.
    CustomMetric(CustomMetricViolation),
    /// A metric value did not meet an exact numeric threshold.
    MetricThreshold(MetricThresholdViolation),
    /// A built-in metric value did not satisfy a user predicate.
    MetricPredicate(MetricPredicateViolation),
}

impl Violation {
    /// Returns this violation's stable family.
    #[must_use]
    pub const fn kind(&self) -> ViolationKind {
        match self {
            Self::EmptyTest(_) => ViolationKind::EmptyTest,
            Self::Cycle(_) => ViolationKind::Cycle,
            Self::FilePattern(_) => ViolationKind::FilePattern,
            Self::FileDependency(_) => ViolationKind::FileDependency,
            Self::ExternalModuleDependency(_) => ViolationKind::ExternalModuleDependency,
            Self::CustomFile(_) => ViolationKind::CustomFile,
            Self::LayerDependency(_) => ViolationKind::LayerDependency,
            Self::SliceDependency(_) => ViolationKind::SliceDependency,
            Self::MetricZone(_) => ViolationKind::MetricZone,
            Self::CustomMetric(_) => ViolationKind::CustomMetric,
            Self::MetricThreshold(_) => ViolationKind::MetricThreshold,
            Self::MetricPredicate(_) => ViolationKind::MetricPredicate,
        }
    }

    /// Returns the empty-test data when this is an empty-test violation.
    #[must_use]
    pub const fn as_empty_test(&self) -> Option<&EmptyTestViolation> {
        match self {
            Self::EmptyTest(violation) => Some(violation),
            Self::Cycle(_)
            | Self::FilePattern(_)
            | Self::FileDependency(_)
            | Self::ExternalModuleDependency(_)
            | Self::CustomFile(_)
            | Self::LayerDependency(_)
            | Self::SliceDependency(_)
            | Self::MetricZone(_)
            | Self::CustomMetric(_)
            | Self::MetricThreshold(_)
            | Self::MetricPredicate(_) => None,
        }
    }

    /// Returns the cycle data when this is a cycle violation.
    #[must_use]
    pub const fn as_cycle(&self) -> Option<&CycleViolation> {
        match self {
            Self::Cycle(violation) => Some(violation),
            Self::EmptyTest(_)
            | Self::FilePattern(_)
            | Self::FileDependency(_)
            | Self::ExternalModuleDependency(_)
            | Self::CustomFile(_)
            | Self::LayerDependency(_)
            | Self::SliceDependency(_)
            | Self::MetricZone(_)
            | Self::CustomMetric(_)
            | Self::MetricThreshold(_)
            | Self::MetricPredicate(_) => None,
        }
    }

    /// Returns the file-pattern data when this is a file-pattern violation.
    #[must_use]
    pub const fn as_file_pattern(&self) -> Option<&FilePatternViolation> {
        match self {
            Self::FilePattern(violation) => Some(violation),
            Self::EmptyTest(_)
            | Self::Cycle(_)
            | Self::FileDependency(_)
            | Self::ExternalModuleDependency(_)
            | Self::CustomFile(_)
            | Self::LayerDependency(_)
            | Self::SliceDependency(_)
            | Self::MetricZone(_)
            | Self::CustomMetric(_)
            | Self::MetricThreshold(_)
            | Self::MetricPredicate(_) => None,
        }
    }

    /// Returns the file-dependency data when this is a file-dependency violation.
    #[must_use]
    pub const fn as_file_dependency(&self) -> Option<&FileDependencyViolation> {
        match self {
            Self::FileDependency(violation) => Some(violation),
            Self::EmptyTest(_)
            | Self::Cycle(_)
            | Self::FilePattern(_)
            | Self::ExternalModuleDependency(_)
            | Self::CustomFile(_)
            | Self::LayerDependency(_)
            | Self::SliceDependency(_)
            | Self::MetricZone(_)
            | Self::CustomMetric(_)
            | Self::MetricThreshold(_)
            | Self::MetricPredicate(_) => None,
        }
    }

    /// Returns the external-module data when this is an external-module dependency violation.
    #[must_use]
    pub const fn as_external_module_dependency(
        &self,
    ) -> Option<&ExternalModuleDependencyViolation> {
        match self {
            Self::ExternalModuleDependency(violation) => Some(violation),
            Self::EmptyTest(_)
            | Self::Cycle(_)
            | Self::FilePattern(_)
            | Self::FileDependency(_)
            | Self::CustomFile(_)
            | Self::LayerDependency(_)
            | Self::SliceDependency(_)
            | Self::MetricZone(_)
            | Self::CustomMetric(_)
            | Self::MetricThreshold(_)
            | Self::MetricPredicate(_) => None,
        }
    }

    /// Returns the custom-file data when this is a custom predicate violation.
    #[must_use]
    pub const fn as_custom_file(&self) -> Option<&CustomFileViolation> {
        match self {
            Self::CustomFile(violation) => Some(violation),
            Self::EmptyTest(_)
            | Self::Cycle(_)
            | Self::FilePattern(_)
            | Self::FileDependency(_)
            | Self::ExternalModuleDependency(_)
            | Self::LayerDependency(_)
            | Self::SliceDependency(_)
            | Self::MetricZone(_)
            | Self::CustomMetric(_)
            | Self::MetricThreshold(_)
            | Self::MetricPredicate(_) => None,
        }
    }

    /// Returns the named-layer dependency data when this is a layer violation.
    #[must_use]
    pub const fn as_layer_dependency(&self) -> Option<&LayerDependencyViolation> {
        match self {
            Self::LayerDependency(violation) => Some(violation),
            Self::EmptyTest(_)
            | Self::Cycle(_)
            | Self::FilePattern(_)
            | Self::FileDependency(_)
            | Self::ExternalModuleDependency(_)
            | Self::CustomFile(_)
            | Self::SliceDependency(_)
            | Self::MetricZone(_)
            | Self::CustomMetric(_)
            | Self::MetricThreshold(_)
            | Self::MetricPredicate(_) => None,
        }
    }

    /// Returns the projected slice dependency data when this is a slice violation.
    #[must_use]
    pub const fn as_slice_dependency(&self) -> Option<&SliceDependencyViolation> {
        match self {
            Self::SliceDependency(violation) => Some(violation),
            Self::EmptyTest(_)
            | Self::Cycle(_)
            | Self::FilePattern(_)
            | Self::FileDependency(_)
            | Self::ExternalModuleDependency(_)
            | Self::CustomFile(_)
            | Self::LayerDependency(_)
            | Self::MetricZone(_)
            | Self::CustomMetric(_)
            | Self::MetricThreshold(_)
            | Self::MetricPredicate(_) => None,
        }
    }

    /// Returns metric-zone data when this is a distance-zone violation.
    #[must_use]
    pub const fn as_metric_zone(&self) -> Option<&MetricZoneViolation> {
        match self {
            Self::MetricZone(violation) => Some(violation),
            Self::EmptyTest(_)
            | Self::Cycle(_)
            | Self::FilePattern(_)
            | Self::FileDependency(_)
            | Self::ExternalModuleDependency(_)
            | Self::CustomFile(_)
            | Self::LayerDependency(_)
            | Self::SliceDependency(_)
            | Self::CustomMetric(_)
            | Self::MetricThreshold(_)
            | Self::MetricPredicate(_) => None,
        }
    }

    /// Returns custom-metric data when a user predicate rejected a type value.
    #[must_use]
    pub const fn as_custom_metric(&self) -> Option<&CustomMetricViolation> {
        match self {
            Self::CustomMetric(violation) => Some(violation),
            Self::EmptyTest(_)
            | Self::Cycle(_)
            | Self::FilePattern(_)
            | Self::FileDependency(_)
            | Self::ExternalModuleDependency(_)
            | Self::CustomFile(_)
            | Self::LayerDependency(_)
            | Self::SliceDependency(_)
            | Self::MetricZone(_)
            | Self::MetricThreshold(_)
            | Self::MetricPredicate(_) => None,
        }
    }

    /// Returns numeric threshold data when a metric value missed its boundary.
    #[must_use]
    pub const fn as_metric_threshold(&self) -> Option<&MetricThresholdViolation> {
        match self {
            Self::MetricThreshold(violation) => Some(violation),
            Self::EmptyTest(_)
            | Self::Cycle(_)
            | Self::FilePattern(_)
            | Self::FileDependency(_)
            | Self::ExternalModuleDependency(_)
            | Self::CustomFile(_)
            | Self::LayerDependency(_)
            | Self::SliceDependency(_)
            | Self::MetricZone(_)
            | Self::CustomMetric(_)
            | Self::MetricPredicate(_) => None,
        }
    }

    /// Returns built-in metric predicate data when a callback rejected a value.
    #[must_use]
    pub const fn as_metric_predicate(&self) -> Option<&MetricPredicateViolation> {
        match self {
            Self::MetricPredicate(violation) => Some(violation),
            Self::EmptyTest(_)
            | Self::Cycle(_)
            | Self::FilePattern(_)
            | Self::FileDependency(_)
            | Self::ExternalModuleDependency(_)
            | Self::CustomFile(_)
            | Self::LayerDependency(_)
            | Self::SliceDependency(_)
            | Self::MetricZone(_)
            | Self::CustomMetric(_)
            | Self::MetricThreshold(_) => None,
        }
    }
}

impl From<EmptyTestViolation> for Violation {
    fn from(violation: EmptyTestViolation) -> Self {
        Self::EmptyTest(violation)
    }
}

impl From<CycleViolation> for Violation {
    fn from(violation: CycleViolation) -> Self {
        Self::Cycle(violation)
    }
}

impl From<FilePatternViolation> for Violation {
    fn from(violation: FilePatternViolation) -> Self {
        Self::FilePattern(violation)
    }
}

impl From<FileDependencyViolation> for Violation {
    fn from(violation: FileDependencyViolation) -> Self {
        Self::FileDependency(violation)
    }
}

impl From<ExternalModuleDependencyViolation> for Violation {
    fn from(violation: ExternalModuleDependencyViolation) -> Self {
        Self::ExternalModuleDependency(violation)
    }
}

impl From<CustomFileViolation> for Violation {
    fn from(violation: CustomFileViolation) -> Self {
        Self::CustomFile(violation)
    }
}

impl From<LayerDependencyViolation> for Violation {
    fn from(violation: LayerDependencyViolation) -> Self {
        Self::LayerDependency(violation)
    }
}

impl From<SliceDependencyViolation> for Violation {
    fn from(violation: SliceDependencyViolation) -> Self {
        Self::SliceDependency(violation)
    }
}

impl From<MetricZoneViolation> for Violation {
    fn from(violation: MetricZoneViolation) -> Self {
        Self::MetricZone(violation)
    }
}

impl From<CustomMetricViolation> for Violation {
    fn from(violation: CustomMetricViolation) -> Self {
        Self::CustomMetric(violation)
    }
}

impl From<MetricThresholdViolation> for Violation {
    fn from(violation: MetricThresholdViolation) -> Self {
        Self::MetricThreshold(violation)
    }
}

impl From<MetricPredicateViolation> for Violation {
    fn from(violation: MetricPredicateViolation) -> Self {
        Self::MetricPredicate(violation)
    }
}

#[cfg(test)]
mod tests {
    use super::{Violation, ViolationKind};
    use crate::{
        CustomFileViolation, CycleViolation, Edge, EmptyTestViolation,
        ExternalModuleDependencyViolation, FileDependencyViolation, FileInfo, FilePatternViolation,
        Graph, ImportKind, LayerDependencyRule, LayerDependencyViolation, ProjectedEdge,
        RegexFactory, SliceDependencyRule, SliceDependencyViolation, project_to_nodes,
    };

    #[test]
    fn empty_test_has_a_stable_kind() {
        let violation = Violation::from(EmptyTestViolation::new("files", []));

        assert_eq!(violation.kind(), ViolationKind::EmptyTest);
        assert_eq!(violation.kind().as_str(), "empty-test");
        assert_eq!(violation.kind().to_string(), "empty-test");
    }

    #[test]
    fn exposes_typed_data_without_formatting_it() {
        let violation = Violation::from(EmptyTestViolation::new("slices", []));

        let empty = violation
            .as_empty_test()
            .expect("fixture should be an empty-test violation");
        assert_eq!(empty.subject, "slices");
        assert!(empty.selectors.is_empty());
    }

    #[test]
    fn cycle_has_a_stable_kind_and_typed_accessor() {
        let violation = Violation::from(CycleViolation::new(Vec::<ProjectedEdge>::new()));

        assert_eq!(violation.kind(), ViolationKind::Cycle);
        assert_eq!(violation.kind().as_str(), "cycle");
        assert!(violation.as_cycle().is_some());
        assert!(violation.as_empty_test().is_none());
        assert!(violation.as_file_pattern().is_none());
        assert!(violation.as_file_dependency().is_none());
        assert!(violation.as_external_module_dependency().is_none());
        assert!(violation.as_custom_file().is_none());
    }

    #[test]
    fn file_pattern_has_a_stable_kind_and_typed_accessor() {
        let filter = RegexFactory::default()
            .filename_matcher("*.rs")
            .expect("fixture pattern should compile");
        let node = project_to_nodes(&Graph::from_edges([Edge::self_edge("src/lib.rs")]))
            .into_iter()
            .next()
            .expect("fixture graph should project one node");
        let violation = Violation::from(FilePatternViolation::new(filter, node, false));

        assert_eq!(violation.kind(), ViolationKind::FilePattern);
        assert_eq!(violation.kind().as_str(), "file-pattern");
        assert!(violation.as_file_pattern().is_some());
        assert!(violation.as_cycle().is_none());
        assert!(violation.as_empty_test().is_none());
        assert!(violation.as_file_dependency().is_none());
        assert!(violation.as_external_module_dependency().is_none());
        assert!(violation.as_custom_file().is_none());
    }

    #[test]
    fn file_dependency_has_a_stable_kind_and_typed_accessor() {
        let raw = Edge::new("src/api.rs", "src/db.rs", false, [ImportKind::Use]);
        let edge = ProjectedEdge::new("src/api.rs", "src/db.rs", [raw]);
        let violation = Violation::from(FileDependencyViolation::new(edge, true));

        assert_eq!(violation.kind(), ViolationKind::FileDependency);
        assert_eq!(violation.kind().as_str(), "file-dependency");
        assert!(violation.as_file_dependency().is_some());
        assert!(violation.as_file_pattern().is_none());
        assert!(violation.as_cycle().is_none());
        assert!(violation.as_empty_test().is_none());
        assert!(violation.as_external_module_dependency().is_none());
        assert!(violation.as_custom_file().is_none());
    }

    #[test]
    fn external_module_dependency_has_a_stable_kind_and_typed_accessor() {
        let raw = Edge::new("src/api.rs", "tokio", true, [ImportKind::MacroReference]);
        let edge = ProjectedEdge::new("src/api.rs", "tokio", [raw]);
        let violation = Violation::from(ExternalModuleDependencyViolation::new(edge, false));

        assert_eq!(violation.kind(), ViolationKind::ExternalModuleDependency);
        assert_eq!(violation.kind().as_str(), "external-module-dependency");
        assert!(violation.as_external_module_dependency().is_some());
        assert!(violation.as_file_dependency().is_none());
        assert!(violation.as_file_pattern().is_none());
        assert!(violation.as_cycle().is_none());
        assert!(violation.as_empty_test().is_none());
        assert!(violation.as_custom_file().is_none());
    }

    #[test]
    fn custom_file_has_a_stable_kind_and_typed_accessor() {
        let info = FileInfo::new("src/api.rs", "pub fn api() {}\n");
        let violation = Violation::from(CustomFileViolation::new(
            info,
            "contain no public functions",
            true,
        ));

        assert_eq!(violation.kind(), ViolationKind::CustomFile);
        assert_eq!(violation.kind().as_str(), "custom-file");
        assert!(violation.as_custom_file().is_some());
        assert!(violation.as_external_module_dependency().is_none());
        assert!(violation.as_file_dependency().is_none());
        assert!(violation.as_file_pattern().is_none());
        assert!(violation.as_cycle().is_none());
        assert!(violation.as_empty_test().is_none());
        assert!(violation.as_layer_dependency().is_none());
    }

    #[test]
    fn layer_dependency_has_a_stable_kind_and_typed_accessor() {
        let raw = Edge::new("src/api.rs", "src/db.rs", false, [ImportKind::Use]);
        let dependency = ProjectedEdge::new("src/api.rs", "src/db.rs", [raw]);
        let violation = Violation::from(LayerDependencyViolation::new(
            dependency,
            "api",
            "database",
            LayerDependencyRule::MayOnlyDependOnLayers,
        ));

        assert_eq!(violation.kind(), ViolationKind::LayerDependency);
        assert_eq!(violation.kind().as_str(), "layer-dependency");
        assert!(violation.as_layer_dependency().is_some());
        assert!(violation.as_custom_file().is_none());
        assert!(violation.as_external_module_dependency().is_none());
        assert!(violation.as_file_dependency().is_none());
        assert!(violation.as_file_pattern().is_none());
        assert!(violation.as_cycle().is_none());
        assert!(violation.as_empty_test().is_none());
    }

    #[test]
    fn slice_dependency_has_a_stable_kind_and_typed_accessor() {
        let raw = Edge::new("src/api.rs", "src/db.rs", false, [ImportKind::Use]);
        let dependency = ProjectedEdge::new("api", "database", [raw]);
        let violation = Violation::from(SliceDependencyViolation::new(
            dependency,
            "api",
            "database",
            SliceDependencyRule::ContainDependency,
            true,
        ));

        assert_eq!(violation.kind(), ViolationKind::SliceDependency);
        assert_eq!(violation.kind().as_str(), "slice-dependency");
        assert!(violation.as_slice_dependency().is_some());
        assert!(violation.as_layer_dependency().is_none());
        assert!(violation.as_empty_test().is_none());
    }
}
