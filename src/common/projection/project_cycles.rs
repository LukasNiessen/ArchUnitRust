use std::{cmp::Ordering, collections::BTreeMap};

use crate::common::{Edge, Graph};

use super::{ProjectedEdge, cycles::johnson_cycles, per_internal_edge, project_edges};

/// Every elementary directed cycle found in a projected dependency graph.
pub type ProjectedCycles = Vec<Vec<ProjectedEdge>>;

/// Projects a raw graph to internal file dependencies and returns every elementary cycle.
#[must_use]
pub fn project_internal_cycles(graph: &Graph) -> ProjectedCycles {
    let projected = project_edges(graph, per_internal_edge());
    project_cycles(&projected)
}

/// Returns every elementary directed cycle in an evidence-retaining projected graph.
///
/// Projected self-edges are removed before detection. Duplicate projected endpoint pairs are
/// merged and their raw evidence is sorted and deduplicated. Each cycle starts at its
/// lexicographically smallest label, and the returned cycle list is deterministic.
#[must_use]
pub fn project_cycles(edges: &[ProjectedEdge]) -> ProjectedCycles {
    let projected = normalize_projected_edges(edges);
    let label_ids = label_ids_for(&projected);
    let edges_by_ids = index_edges(&projected, &label_ids);
    let adjacency = adjacency_for(&edges_by_ids);

    johnson_cycles(&adjacency)
        .into_iter()
        .filter_map(|path| edges_for_path(&path, &edges_by_ids))
        .collect()
}

fn normalize_projected_edges(edges: &[ProjectedEdge]) -> Vec<ProjectedEdge> {
    let mut groups = BTreeMap::<(String, String), Vec<Edge>>::new();

    for edge in edges {
        if edge.is_self_edge() {
            continue;
        }

        groups
            .entry((edge.source_label.clone(), edge.target_label.clone()))
            .or_default()
            .extend(edge.cumulated_edges.iter().cloned());
    }

    groups
        .into_iter()
        .map(|((source, target), mut raw_edges)| {
            raw_edges.sort_by(compare_raw_edges);
            raw_edges.dedup();
            ProjectedEdge::new(source, target, raw_edges)
        })
        .collect()
}

fn compare_raw_edges(left: &Edge, right: &Edge) -> Ordering {
    left.source
        .cmp(&right.source)
        .then_with(|| left.target.cmp(&right.target))
        .then_with(|| left.external.cmp(&right.external))
        .then_with(|| left.import_kinds.iter().cmp(right.import_kinds.iter()))
}

fn label_ids_for(edges: &[ProjectedEdge]) -> BTreeMap<String, usize> {
    edges
        .iter()
        .flat_map(|edge| [&edge.source_label, &edge.target_label])
        .cloned()
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .enumerate()
        .map(|(identifier, label)| (label, identifier))
        .collect()
}

fn index_edges(
    edges: &[ProjectedEdge],
    label_ids: &BTreeMap<String, usize>,
) -> BTreeMap<(usize, usize), ProjectedEdge> {
    edges
        .iter()
        .filter_map(|edge| {
            let source = label_ids.get(&edge.source_label).copied()?;
            let target = label_ids.get(&edge.target_label).copied()?;
            Some(((source, target), edge.clone()))
        })
        .collect()
}

fn adjacency_for(edges: &BTreeMap<(usize, usize), ProjectedEdge>) -> super::cycles::Adjacency {
    let mut adjacency = super::cycles::Adjacency::new();
    for (source, target) in edges.keys().copied() {
        adjacency.entry(source).or_default().insert(target);
        adjacency.entry(target).or_default();
    }
    adjacency
}

fn edges_for_path(
    path: &[usize],
    edges_by_ids: &BTreeMap<(usize, usize), ProjectedEdge>,
) -> Option<Vec<ProjectedEdge>> {
    if path.is_empty() {
        return None;
    }

    let mut edges = Vec::with_capacity(path.len());
    for (index, source) in path.iter().copied().enumerate() {
        let target = path[(index + 1) % path.len()];
        edges.push(edges_by_ids.get(&(source, target))?.clone());
    }
    Some(edges)
}

#[cfg(test)]
mod tests {
    use crate::common::{Edge, Graph, ImportKind};

    use super::{project_cycles, project_internal_cycles};
    use crate::common::ProjectedEdge;

    fn raw_edge(source: &str, target: &str, external: bool, kind: ImportKind) -> Edge {
        Edge::new(source, target, external, [kind])
    }

    fn projected_edge(source: &str, target: &str, raw: Edge) -> ProjectedEdge {
        ProjectedEdge::new(source, target, [raw])
    }

    #[test]
    fn returns_cycle_edges_in_order_with_raw_evidence_intact() {
        let first = projected_edge(
            "api",
            "domain",
            raw_edge("src/api.rs", "src/domain.rs", false, ImportKind::Use),
        );
        let second = projected_edge(
            "domain",
            "persistence",
            raw_edge(
                "src/domain.rs",
                "src/persistence.rs",
                false,
                ImportKind::Use,
            ),
        );
        let third = projected_edge(
            "persistence",
            "api",
            raw_edge(
                "src/persistence.rs",
                "src/api.rs",
                false,
                ImportKind::PathReference,
            ),
        );

        let cycles = project_cycles(&[third.clone(), first.clone(), second.clone()]);

        assert_eq!(cycles, [vec![first, second, third]]);
    }

    #[test]
    fn filters_projected_self_edges() {
        let self_edge = projected_edge(
            "api",
            "api",
            raw_edge("src/api.rs", "src/api.rs", false, ImportKind::Use),
        );

        assert!(project_cycles(&[self_edge]).is_empty());
    }

    #[test]
    fn cumulates_and_deduplicates_duplicate_projected_pairs() {
        let first_raw = raw_edge("src/a/first.rs", "src/b.rs", false, ImportKind::Use);
        let second_raw = raw_edge(
            "src/a/second.rs",
            "src/b.rs",
            false,
            ImportKind::PathReference,
        );
        let first = projected_edge("a", "b", first_raw.clone());
        let duplicate = ProjectedEdge::new("a", "b", [second_raw.clone(), first_raw.clone()]);
        let reverse = projected_edge(
            "b",
            "a",
            raw_edge("src/b.rs", "src/a/first.rs", false, ImportKind::Use),
        );

        let cycles = project_cycles(&[duplicate, reverse, first]);
        let forward = cycles[0]
            .iter()
            .find(|edge| edge.source_label == "a")
            .expect("the two-edge fixture cycle should contain its forward edge");

        assert_eq!(forward.cumulated_edges, [first_raw, second_raw]);
    }

    #[test]
    fn output_is_independent_of_projected_input_order() {
        let first = projected_edge(
            "a",
            "b",
            raw_edge("src/a.rs", "src/b.rs", false, ImportKind::Use),
        );
        let second = projected_edge(
            "b",
            "a",
            raw_edge("src/b.rs", "src/a.rs", false, ImportKind::Use),
        );

        assert_eq!(
            project_cycles(&[first.clone(), second.clone()]),
            project_cycles(&[second, first])
        );
    }

    #[test]
    fn raw_graph_entry_point_uses_only_internal_non_self_dependencies() {
        let forward = raw_edge("src/a.rs", "src/b.rs", false, ImportKind::Use);
        let reverse = raw_edge("src/b.rs", "src/a.rs", false, ImportKind::Use);
        let external = raw_edge("src/a.rs", "serde", true, ImportKind::Use);
        let graph = Graph::from_edges([forward.clone(), reverse.clone(), external]);

        let cycles = project_internal_cycles(&graph);

        assert_eq!(cycles.len(), 1);
        assert_eq!(cycles[0].len(), 2);
        assert_eq!(cycles[0][0].cumulated_edges, [forward]);
        assert_eq!(cycles[0][1].cumulated_edges, [reverse]);
    }
}
