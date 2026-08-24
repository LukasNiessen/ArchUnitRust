use crate::common::ProjectedEdge;

/// One external crate dependency that disagrees with a file rule.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ExternalModuleDependencyViolation {
    /// The projected dependency together with every raw Rust reference that produced it.
    pub dependency: ProjectedEdge,
    /// Whether this dependency matched a forbidden crate rather than missing an allowed crate.
    pub is_negated: bool,
}

impl ExternalModuleDependencyViolation {
    /// Creates data for one external dependency that failed a rule.
    #[must_use]
    pub const fn new(dependency: ProjectedEdge, is_negated: bool) -> Self {
        Self {
            dependency,
            is_negated,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::common::{Edge, ImportKind, ProjectedEdge};

    use super::ExternalModuleDependencyViolation;

    #[test]
    fn retains_external_dependency_evidence_with_the_mood() {
        let raw = Edge::new("src/api.rs", "tokio", true, [ImportKind::MacroReference]);
        let dependency = ProjectedEdge::new("src/api.rs", "tokio", [raw.clone()]);

        let violation = ExternalModuleDependencyViolation::new(dependency, true);

        assert_eq!(violation.dependency.source_label, "src/api.rs");
        assert_eq!(violation.dependency.target_label, "tokio");
        assert_eq!(violation.dependency.cumulated_edges, [raw]);
        assert!(violation.is_negated);
    }
}
