use crate::common::ProjectedEdge;

/// One closed circular path through a projected dependency graph.
///
/// The edge data remains available for evidence-rich reports, while [`Self::path`] provides the
/// label sequence needed by human-readable formatters.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct CycleViolation {
    /// The projected dependencies in traversal order.
    pub cycle: Vec<ProjectedEdge>,
    /// The closed label path, including the starting label again at the end.
    pub path: Vec<String>,
}

impl CycleViolation {
    /// Creates violation data from one projected cycle.
    #[must_use]
    pub fn new(cycle: impl IntoIterator<Item = ProjectedEdge>) -> Self {
        let cycle = cycle.into_iter().collect::<Vec<_>>();
        let path = cycle.first().map_or_else(Vec::new, |first| {
            std::iter::once(first.source_label.clone())
                .chain(cycle.iter().map(|edge| edge.target_label.clone()))
                .collect()
        });
        Self { cycle, path }
    }
}

#[cfg(test)]
mod tests {
    use crate::common::{Edge, ImportKind, ProjectedEdge};

    use super::CycleViolation;

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
    fn retains_edges_and_builds_a_readable_closed_path() {
        let first = projected("api", "domain");
        let second = projected("domain", "api");

        let violation = CycleViolation::new([first.clone(), second.clone()]);

        assert_eq!(violation.cycle, [first, second]);
        assert_eq!(violation.path, ["api", "domain", "api"]);
        assert_eq!(violation.path.join(" -> "), "api -> domain -> api");
    }
}
