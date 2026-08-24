use crate::common::ProjectedEdge;

use super::SliceDependencyRule;

/// One projected slice dependency rejected by a slice architecture rule.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct SliceDependencyViolation {
    /// The projected dependency and every concrete Rust file edge behind it.
    pub dependency: ProjectedEdge,
    /// The selected dependency source slice.
    pub source_slice: String,
    /// The selected dependency target slice or external crate.
    pub target_slice: String,
    /// The policy kind that rejected the dependency.
    pub rule: SliceDependencyRule,
    /// Whether the fluent rule used the negated mood.
    pub is_negated: bool,
}

impl SliceDependencyViolation {
    /// Creates structured data for one rejected slice dependency.
    #[must_use]
    pub fn new(
        dependency: ProjectedEdge,
        source_slice: impl Into<String>,
        target_slice: impl Into<String>,
        rule: SliceDependencyRule,
        is_negated: bool,
    ) -> Self {
        Self {
            dependency,
            source_slice: source_slice.into(),
            target_slice: target_slice.into(),
            rule,
            is_negated,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::common::{Edge, ImportKind, ProjectedEdge};

    use super::{SliceDependencyRule, SliceDependencyViolation};

    #[test]
    fn retains_slice_rule_mood_and_concrete_dependency_evidence() {
        let raw = Edge::new("src/api.rs", "serde", true, [ImportKind::Use]);
        let dependency = ProjectedEdge::new("api", "serde", [raw.clone()]);
        let violation = SliceDependencyViolation::new(
            dependency,
            "api",
            "serde",
            SliceDependencyRule::ContainDependency,
            true,
        );

        assert_eq!(violation.source_slice, "api");
        assert_eq!(violation.target_slice, "serde");
        assert_eq!(violation.rule, SliceDependencyRule::ContainDependency);
        assert!(violation.is_negated);
        assert_eq!(violation.dependency.cumulated_edges, [raw]);
    }
}
