use std::collections::BTreeSet;

use crate::{Edge, Graph};

use super::{
    GraphQueryError, GraphQueryOptions, GraphReportEdge, GraphReportNode, GraphReportSnapshot,
    GraphReportSummary, aggregate_graph_edges, collapse_graph_node,
    node_selection::select_graph_nodes,
};

/// The title used when a graph query does not provide one.
pub const DEFAULT_GRAPH_TITLE: &str = "ArchUnitRust Dependency Graph";

/// Builds the one renderer-neutral representation of a queried graph.
#[derive(Debug, Clone, Copy, Default)]
pub struct GraphSnapshotFactory;

impl GraphSnapshotFactory {
    /// Filters, selects, collapses, aggregates, and counts one graph.
    pub fn create(
        graph: &Graph,
        options: &GraphQueryOptions,
    ) -> Result<GraphReportSnapshot, GraphQueryError> {
        create_graph_snapshot(graph, options)
    }
}

/// Filters, selects, collapses, aggregates, and counts one graph.
pub fn create_graph_snapshot(
    graph: &Graph,
    options: &GraphQueryOptions,
) -> Result<GraphReportSnapshot, GraphQueryError> {
    let query_edges = graph
        .iter()
        .filter(|edge| options.includes_external_dependencies() || !edge.external)
        .cloned()
        .collect::<Vec<_>>();
    let selected_nodes = select_graph_nodes(&query_edges, options);
    let raw_edges = selected_edges(&query_edges, &selected_nodes, options);
    let report_edges = aggregate_graph_edges(
        &raw_edges,
        options.collapse(),
        options.includes_self_dependencies(),
    )?;
    let report_nodes = build_nodes(&selected_nodes, &report_edges, options)?;
    let summary = GraphReportSummary::new(
        report_nodes.len(),
        report_edges.len(),
        raw_edges.len(),
        raw_edges.iter().filter(|edge| edge.external).count(),
    );

    Ok(GraphReportSnapshot::new(
        options.title().unwrap_or(DEFAULT_GRAPH_TITLE),
        report_nodes,
        report_edges,
        summary,
    ))
}

fn selected_edges(
    edges: &[Edge],
    selected_nodes: &BTreeSet<String>,
    options: &GraphQueryOptions,
) -> Vec<Edge> {
    edges
        .iter()
        .filter(|edge| {
            (options.includes_self_dependencies() || !edge.is_self_edge())
                && selected_nodes.contains(&edge.source)
                && selected_nodes.contains(&edge.target)
        })
        .cloned()
        .collect()
}

fn build_nodes(
    selected_nodes: &BTreeSet<String>,
    report_edges: &[GraphReportEdge],
    options: &GraphQueryOptions,
) -> Result<Vec<GraphReportNode>, GraphQueryError> {
    let mut labels = BTreeSet::new();
    for node in selected_nodes {
        labels.insert(collapse_graph_node(node, options.collapse())?);
    }
    for edge in report_edges {
        labels.insert(edge.source.clone());
        labels.insert(edge.target.clone());
    }

    Ok(labels
        .into_iter()
        .enumerate()
        .map(|(index, label)| GraphReportNode::new(format!("n{index}"), label))
        .collect())
}

#[cfg(test)]
mod tests {
    use crate::{
        Edge, FolderDepthCollapse, Graph, GraphCollapse, GraphQueryOptions, GraphSnapshotFactory,
        ImportKind, PatternCollapse, RegexFactory,
    };

    fn sample_graph() -> Graph {
        Graph::from_edges([
            Edge::self_edge("src/app/controller.rs"),
            Edge::self_edge("src/app/helper.rs"),
            Edge::self_edge("src/domain/service.rs"),
            Edge::self_edge("src/infra/repository.rs"),
            Edge::self_edge("src/orphan/alone.rs"),
            Edge::new(
                "src/app/controller.rs",
                "src/domain/service.rs",
                false,
                [ImportKind::Use],
            ),
            Edge::new(
                "src/app/controller.rs",
                "src/domain/service.rs",
                false,
                [ImportKind::PubUse],
            ),
            Edge::new(
                "src/app/helper.rs",
                "src/domain/service.rs",
                false,
                [ImportKind::PathReference],
            ),
            Edge::new(
                "src/domain/service.rs",
                "src/infra/repository.rs",
                false,
                [ImportKind::Mod],
            ),
            Edge::new(
                "src/app/controller.rs",
                "src/infra/repository.rs",
                false,
                [ImportKind::MacroReference],
            ),
            Edge::new("src/app/controller.rs", "serde", true, [ImportKind::Use]),
        ])
    }

    fn filter(pattern: &str) -> crate::Filter {
        RegexFactory::default()
            .path_matcher(pattern)
            .expect("fixture selector should compile")
    }

    #[test]
    fn defaults_exclude_external_and_self_edges_but_keep_every_internal_node() {
        let snapshot = GraphSnapshotFactory::create(&sample_graph(), &GraphQueryOptions::new())
            .expect("default snapshot should succeed");

        assert_eq!(snapshot.title, "ArchUnitRust Dependency Graph");
        assert_eq!(snapshot.summary.node_count, 5);
        assert_eq!(snapshot.summary.edge_count, 4);
        assert_eq!(snapshot.summary.raw_edge_count, 4);
        assert_eq!(snapshot.summary.external_edge_count, 0);
        assert_eq!(
            snapshot
                .nodes
                .iter()
                .map(|node| node.label.as_str())
                .collect::<Vec<_>>(),
            [
                "src/app/controller.rs",
                "src/app/helper.rs",
                "src/domain/service.rs",
                "src/infra/repository.rs",
                "src/orphan/alone.rs"
            ]
        );
        let edge = snapshot
            .edges
            .iter()
            .find(|edge| {
                edge.target == "src/domain/service.rs" && edge.source.ends_with("controller.rs")
            })
            .expect("controller-domain edge should exist");
        assert_eq!(edge.count, 1);
        assert_eq!(
            edge.import_kinds.iter().collect::<Vec<_>>(),
            [ImportKind::Use, ImportKind::PubUse]
        );
    }

    #[test]
    fn external_and_self_options_affect_nodes_edges_and_summary_together() {
        let options = GraphQueryOptions::new()
            .with_external_dependencies(true)
            .with_self_dependencies(true);
        let snapshot = GraphSnapshotFactory::create(&sample_graph(), &options)
            .expect("inclusive snapshot should succeed");

        assert_eq!(snapshot.summary.node_count, 6);
        assert_eq!(snapshot.summary.edge_count, 10);
        assert_eq!(snapshot.summary.raw_edge_count, 10);
        assert_eq!(snapshot.summary.external_edge_count, 1);
        assert!(snapshot.nodes.iter().any(|node| node.label == "serde"));
        assert!(snapshot.edges.iter().any(|edge| edge.external));
        assert!(snapshot.edges.iter().any(|edge| edge.source == edge.target));
    }

    #[test]
    fn focus_reachability_dependents_and_union_queries_select_induced_subgraphs() {
        let focused = GraphQueryOptions::new().with_focus(filter("src/domain/**"), 0);
        let reachable = GraphQueryOptions::new().with_reachable_from(filter("src/domain/**"));
        let dependents = GraphQueryOptions::new().with_dependents_of(filter("src/infra/**"));
        let union = focused
            .clone()
            .with_focus(filter("src/orphan/**"), 0)
            .with_reachable_from(filter("src/domain/**"));

        let focused = GraphSnapshotFactory::create(&sample_graph(), &focused)
            .expect("focus snapshot should succeed");
        let reachable = GraphSnapshotFactory::create(&sample_graph(), &reachable)
            .expect("reachable snapshot should succeed");
        let dependents = GraphSnapshotFactory::create(&sample_graph(), &dependents)
            .expect("dependents snapshot should succeed");
        let union = GraphSnapshotFactory::create(&sample_graph(), &union)
            .expect("union snapshot should succeed");

        assert_eq!(focused.nodes.len(), 1);
        assert!(focused.edges.is_empty());
        assert_eq!(
            reachable
                .nodes
                .iter()
                .map(|node| node.label.as_str())
                .collect::<Vec<_>>(),
            ["src/domain/service.rs", "src/infra/repository.rs"]
        );
        assert_eq!(dependents.nodes.len(), 4);
        assert_eq!(
            union
                .nodes
                .iter()
                .map(|node| node.label.as_str())
                .collect::<Vec<_>>(),
            [
                "src/domain/service.rs",
                "src/infra/repository.rs",
                "src/orphan/alone.rs"
            ]
        );
    }

    #[test]
    fn folder_collapse_aggregates_edges_and_assigns_stable_sorted_node_ids() {
        let collapse = GraphCollapse::FolderDepth(
            FolderDepthCollapse::new(2).expect("fixture depth should be valid"),
        );
        let options = GraphQueryOptions::new().with_collapse(collapse);
        let snapshot = GraphSnapshotFactory::create(&sample_graph(), &options)
            .expect("folder snapshot should succeed");

        assert_eq!(
            snapshot
                .nodes
                .iter()
                .map(|node| (node.id.as_str(), node.label.as_str()))
                .collect::<Vec<_>>(),
            [
                ("n0", "src/app"),
                ("n1", "src/domain"),
                ("n2", "src/infra"),
                ("n3", "src/orphan")
            ]
        );
        let edge = snapshot
            .edges
            .iter()
            .find(|edge| edge.source == "src/app" && edge.target == "src/domain")
            .expect("collapsed app-domain edge should exist");
        assert_eq!(edge.count, 2);
        assert_eq!(snapshot.summary.raw_edge_count, 4);
        assert_eq!(snapshot.summary.edge_count, 3);
    }

    #[test]
    fn pattern_collapse_removes_new_self_edges_and_preserves_custom_title() {
        let graph = Graph::from_edges(sample_graph().into_iter().chain([Edge::new(
            "src/app/controller.rs",
            "src/app/helper.rs",
            false,
            [ImportKind::Use],
        )]));
        let collapse = GraphCollapse::Pattern(
            PatternCollapse::first_capture(r"src/([^/]+)/.*")
                .expect("fixture collapse should compile"),
        );
        let options = GraphQueryOptions::new()
            .with_collapse(collapse)
            .with_title("Application Architecture")
            .expect("visible title should be valid");
        let snapshot = GraphSnapshotFactory::create(&graph, &options)
            .expect("pattern snapshot should succeed");

        assert_eq!(snapshot.title, "Application Architecture");
        assert_eq!(
            snapshot
                .nodes
                .iter()
                .map(|node| node.label.as_str())
                .collect::<Vec<_>>(),
            ["app", "domain", "infra", "orphan"]
        );
        assert!(!snapshot.edges.iter().any(|edge| edge.source == edge.target));
        assert_eq!(snapshot.summary.raw_edge_count, 5);
        assert_eq!(snapshot.summary.edge_count, 3);
    }
}
