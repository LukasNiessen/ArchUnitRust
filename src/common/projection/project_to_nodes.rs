use std::collections::BTreeMap;

use crate::{Edge, Graph};

use super::ProjectedNode;

/// Options controlling projection of a raw graph to file nodes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct NodeProjectionOptions {
    include_externals: bool,
}

impl NodeProjectionOptions {
    /// Creates the internal-node-only projection configuration.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            include_externals: false,
        }
    }

    /// Returns whether external dependency targets become projected nodes.
    #[must_use]
    pub const fn includes_externals(self) -> bool {
        self.include_externals
    }

    /// Controls whether external dependency targets become projected nodes.
    #[must_use]
    pub const fn with_externals(mut self, include: bool) -> Self {
        self.include_externals = include;
        self
    }
}

/// Projects a graph to its internal nodes using default options.
///
/// Self-edges retain dependency-free files as nodes but are not exposed as incoming or outgoing
/// dependencies.
#[must_use]
pub fn project_to_nodes(graph: &Graph) -> Vec<ProjectedNode> {
    project_to_nodes_with_options(graph, NodeProjectionOptions::default())
}

/// Projects a graph to nodes using explicit options.
#[must_use]
pub fn project_to_nodes_with_options(
    graph: &Graph,
    options: NodeProjectionOptions,
) -> Vec<ProjectedNode> {
    let mut nodes = BTreeMap::<String, (Vec<Edge>, Vec<Edge>)>::new();

    for edge in graph {
        nodes.entry(edge.source.clone()).or_default();
        if edge.is_self_edge() {
            continue;
        }

        if let Some((_, outgoing)) = nodes.get_mut(&edge.source) {
            outgoing.push(edge.clone());
        }

        if !edge.external || options.includes_externals() {
            let (incoming, _) = nodes.entry(edge.target.clone()).or_default();
            incoming.push(edge.clone());
        }
    }

    nodes
        .into_iter()
        .map(|(label, (incoming, outgoing))| ProjectedNode::new(label, incoming, outgoing))
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::{Edge, Graph, ImportKind};

    use super::{NodeProjectionOptions, project_to_nodes, project_to_nodes_with_options};

    #[test]
    fn retains_isolated_nodes_without_reporting_self_dependencies() {
        let graph = Graph::from_edges([Edge::self_edge("src/isolated.rs")]);

        let nodes = project_to_nodes(&graph);

        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].label, "src/isolated.rs");
        assert!(nodes[0].incoming.is_empty());
        assert!(nodes[0].outgoing.is_empty());
    }

    #[test]
    fn groups_internal_edges_into_sorted_nodes() {
        let dependency = Edge::new("src/a.rs", "src/b.rs", false, [ImportKind::Use]);
        let graph = Graph::from_edges([
            Edge::self_edge("src/b.rs"),
            dependency.clone(),
            Edge::self_edge("src/a.rs"),
        ]);

        let nodes = project_to_nodes(&graph);

        assert_eq!(
            nodes
                .iter()
                .map(|node| node.label.as_str())
                .collect::<Vec<_>>(),
            ["src/a.rs", "src/b.rs"]
        );
        assert_eq!(
            nodes[0].outgoing.as_slice(),
            std::slice::from_ref(&dependency)
        );
        assert_eq!(nodes[1].incoming, [dependency]);
    }

    #[test]
    fn omits_external_target_nodes_but_keeps_source_evidence_by_default() {
        let dependency = Edge::new("src/lib.rs", "serde", true, [ImportKind::Use]);
        let graph = Graph::from_edges([dependency.clone()]);

        let nodes = project_to_nodes(&graph);

        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].label, "src/lib.rs");
        assert_eq!(nodes[0].outgoing, [dependency]);
    }

    #[test]
    fn includes_external_target_nodes_when_requested() {
        let dependency = Edge::new("src/lib.rs", "serde", true, [ImportKind::Use]);
        let graph = Graph::from_edges([dependency.clone()]);

        let nodes = project_to_nodes_with_options(
            &graph,
            NodeProjectionOptions::new().with_externals(true),
        );

        assert_eq!(
            nodes
                .iter()
                .map(|node| node.label.as_str())
                .collect::<Vec<_>>(),
            ["serde", "src/lib.rs"]
        );
        assert_eq!(
            nodes[0].incoming.as_slice(),
            std::slice::from_ref(&dependency)
        );
        assert_eq!(nodes[1].outgoing, [dependency]);
    }
}
