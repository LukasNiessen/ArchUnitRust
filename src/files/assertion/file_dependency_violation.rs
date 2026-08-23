use crate::ProjectedEdge;

/// One internal file dependency that disagrees with a relational rule.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct FileDependencyViolation {
    /// The projected dependency together with every raw Rust reference that produced it.
    pub dependency: ProjectedEdge,
    /// Whether this dependency matched a forbidden target rather than missing an allowed target.
    pub is_negated: bool,
}

impl FileDependencyViolation {
    /// Creates data for one dependency that failed a file rule.
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
    use crate::{Edge, ImportKind, ProjectedEdge};

    use super::FileDependencyViolation;

    #[test]
    fn retains_projected_and_raw_dependency_evidence_with_the_mood() {
        let raw = Edge::new("src/api.rs", "src/database.rs", false, [ImportKind::Use]);
        let dependency = ProjectedEdge::new("src/api.rs", "src/database.rs", [raw.clone()]);

        let violation = FileDependencyViolation::new(dependency, true);

        assert_eq!(violation.dependency.source_label, "src/api.rs");
        assert_eq!(violation.dependency.target_label, "src/database.rs");
        assert_eq!(violation.dependency.cumulated_edges, [raw]);
        assert!(violation.is_negated);
    }
}
