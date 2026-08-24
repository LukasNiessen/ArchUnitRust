use crate::common::ProjectedEdge;

use super::LayerDependencyRule;

/// One cross-layer dependency rejected by a named-layer policy.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct LayerDependencyViolation {
    /// The concrete file dependency and every raw Rust reference behind it.
    pub dependency: ProjectedEdge,
    /// The declared layer containing the dependency source file.
    pub source_layer: String,
    /// The declared layer containing the dependency target file.
    pub target_layer: String,
    /// The policy kind that rejected the dependency.
    pub rule: LayerDependencyRule,
}

impl LayerDependencyViolation {
    /// Creates data for one rejected cross-layer dependency.
    #[must_use]
    pub fn new(
        dependency: ProjectedEdge,
        source_layer: impl Into<String>,
        target_layer: impl Into<String>,
        rule: LayerDependencyRule,
    ) -> Self {
        Self {
            dependency,
            source_layer: source_layer.into(),
            target_layer: target_layer.into(),
            rule,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::common::{Edge, ImportKind, ProjectedEdge};

    use super::{LayerDependencyRule, LayerDependencyViolation};

    #[test]
    fn retains_layers_rule_and_raw_dependency_evidence() {
        let raw = Edge::new("src/api.rs", "src/db.rs", false, [ImportKind::Use]);
        let dependency = ProjectedEdge::new("src/api.rs", "src/db.rs", [raw.clone()]);
        let violation = LayerDependencyViolation::new(
            dependency,
            "api",
            "database",
            LayerDependencyRule::MayNotDependOnLayers,
        );

        assert_eq!(violation.source_layer, "api");
        assert_eq!(violation.target_layer, "database");
        assert_eq!(violation.rule, LayerDependencyRule::MayNotDependOnLayers);
        assert_eq!(violation.dependency.cumulated_edges, [raw]);
    }
}
