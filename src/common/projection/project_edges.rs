use std::collections::BTreeMap;

use crate::common::{Edge, Graph};

use super::{MappedEdge, ProjectedEdge};

/// A graph relabeling hook. Returning `None` drops the raw edge from the projected view.
pub type MapFunction<'a> = dyn Fn(&Edge) -> Option<MappedEdge> + 'a;

/// A dependency graph after domain-specific relabeling and grouping.
pub type ProjectedGraph = Vec<ProjectedEdge>;

/// Relabels raw graph edges and cumulates equal projected endpoint pairs.
///
/// Each returned [`ProjectedEdge`] owns the raw edges it represents, preserving concrete evidence
/// for later violation messages. Results are ordered by projected `(source_label, target_label)`.
#[must_use]
pub fn project_edges<M>(graph: &Graph, mapper: M) -> ProjectedGraph
where
    M: Fn(&Edge) -> Option<MappedEdge>,
{
    let mut groups = BTreeMap::<(String, String), Vec<Edge>>::new();

    for edge in graph {
        let Some(mapped) = mapper(edge) else {
            continue;
        };

        groups
            .entry((mapped.source_label, mapped.target_label))
            .or_default()
            .push(edge.clone());
    }

    groups
        .into_iter()
        .map(|((source, target), raw_edges)| ProjectedEdge::new(source, target, raw_edges))
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::common::{Edge, Graph, ImportKind};

    use super::{MappedEdge, project_edges};

    #[test]
    fn maps_filters_and_cumulates_equal_label_pairs() {
        let first = Edge::new("src/api/a.rs", "src/domain/a.rs", false, [ImportKind::Use]);
        let second = Edge::new(
            "src/api/b.rs",
            "src/domain/b.rs",
            false,
            [ImportKind::PathReference],
        );
        let external = Edge::new("src/api/a.rs", "serde", true, [ImportKind::Use]);
        let graph = Graph::from_edges([first.clone(), second.clone(), external]);

        let projected = project_edges(&graph, |edge| {
            (!edge.external).then(|| MappedEdge::new("api", "domain"))
        });

        assert_eq!(projected.len(), 1);
        assert_eq!(projected[0].source_label, "api");
        assert_eq!(projected[0].target_label, "domain");
        assert_eq!(projected[0].cumulated_edges, [first, second]);
    }

    #[test]
    fn orders_projected_pairs_and_their_evidence_deterministically() {
        let first = Edge::new("src/a.rs", "src/c.rs", false, [ImportKind::Use]);
        let second = Edge::new("src/b.rs", "src/d.rs", false, [ImportKind::Use]);
        let graph = Graph::from_edges([second.clone(), first.clone()]);

        let projected = project_edges(&graph, |edge| {
            let pair = if edge.source.ends_with("a.rs") {
                ("z", "last")
            } else {
                ("a", "first")
            };
            Some(MappedEdge::new(pair.0, pair.1))
        });

        assert_eq!(projected[0].source_label, "a");
        assert_eq!(projected[0].cumulated_edges, [second]);
        assert_eq!(projected[1].source_label, "z");
        assert_eq!(projected[1].cumulated_edges, [first]);
    }

    #[test]
    fn does_not_call_the_mapper_for_an_empty_graph() {
        let projected = project_edges(&Graph::new(), |_| {
            unreachable!("the mapper must not be called")
        });

        assert!(projected.is_empty());
    }
}
