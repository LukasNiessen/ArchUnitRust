use crate::common::ProjectedEdge;

use super::{SliceDependencyRule, SliceDependencyViolation};

/// Collects dependencies that match one forbidden source-to-target slice pair.
#[must_use]
pub fn gather_forbidden_slice_dependency_violations(
    edges: &[ProjectedEdge],
    source_slice: &str,
    target_slice: &str,
) -> Vec<SliceDependencyViolation> {
    edges
        .iter()
        .filter(|edge| edge.source_label == source_slice && edge.target_label == target_slice)
        .cloned()
        .map(|edge| {
            SliceDependencyViolation::new(
                edge,
                source_slice,
                target_slice,
                SliceDependencyRule::ContainDependency,
                true,
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::common::{Edge, ImportKind, ProjectedEdge};

    use super::gather_forbidden_slice_dependency_violations;

    fn projected(source: &str, target: &str) -> ProjectedEdge {
        ProjectedEdge::new(
            source,
            target,
            [Edge::new(source, target, false, [ImportKind::Use])],
        )
    }

    #[test]
    fn returns_only_the_exact_directed_forbidden_pair() {
        let rejected = projected("api", "database");
        let edges = [
            projected("api", "application"),
            rejected.clone(),
            projected("database", "api"),
        ];

        let violations = gather_forbidden_slice_dependency_violations(&edges, "api", "database");

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].dependency, rejected);
        assert!(violations[0].is_negated);
    }
}
