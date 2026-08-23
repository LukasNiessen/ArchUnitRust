use crate::{CycleViolation, ProjectedCycles, Violation};

/// Converts every projected cycle into machine-readable violation data.
#[must_use]
pub fn gather_cycle_violations(cycles: ProjectedCycles) -> Vec<Violation> {
    cycles
        .into_iter()
        .map(CycleViolation::new)
        .map(Violation::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::{Edge, ImportKind, ProjectedEdge, ViolationKind};

    use super::gather_cycle_violations;

    fn projected(source: &str, target: &str) -> ProjectedEdge {
        ProjectedEdge::new(
            source,
            target,
            [Edge::new(
                format!("src/{source}.rs"),
                format!("src/{target}.rs"),
                false,
                [ImportKind::Use],
            )],
        )
    }

    #[test]
    fn returns_one_data_violation_per_cycle() {
        let first = vec![projected("a", "b"), projected("b", "a")];
        let second = vec![projected("c", "d"), projected("d", "c")];

        let violations = gather_cycle_violations(vec![first, second]);

        assert_eq!(violations.len(), 2);
        assert!(
            violations
                .iter()
                .all(|violation| violation.kind() == ViolationKind::Cycle)
        );
        assert_eq!(
            violations[0]
                .as_cycle()
                .map(|violation| violation.path.clone()),
            Some(vec!["a".to_owned(), "b".to_owned(), "a".to_owned()])
        );
    }
}
