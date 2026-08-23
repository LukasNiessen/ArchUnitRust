use std::fmt;
use std::iter::FromIterator;
use std::slice;

use super::Edge;

/// The extracted dependency graph.
///
/// This first kernel slice intentionally stores only edges. Extraction issue #10 adds the canonical
/// merging and self-edge population invariants before any rule relies on them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Graph {
    edges: Vec<Edge>,
}

impl Graph {
    /// Creates an empty graph.
    #[must_use]
    pub const fn new() -> Self {
        Self { edges: Vec::new() }
    }

    /// Creates a graph from extracted edges.
    #[must_use]
    pub fn from_edges(edges: impl IntoIterator<Item = Edge>) -> Self {
        Self {
            edges: edges.into_iter().collect(),
        }
    }

    /// Returns all edges in extraction order.
    #[must_use]
    pub fn edges(&self) -> &[Edge] {
        &self.edges
    }

    /// Returns the number of edges.
    #[must_use]
    pub fn len(&self) -> usize {
        self.edges.len()
    }

    /// Returns whether the graph has no edges.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.edges.is_empty()
    }

    /// Iterates over graph edges without exposing mutable access.
    pub fn iter(&self) -> slice::Iter<'_, Edge> {
        self.edges.iter()
    }
}

impl FromIterator<Edge> for Graph {
    fn from_iter<T: IntoIterator<Item = Edge>>(edges: T) -> Self {
        Self::from_edges(edges)
    }
}

impl IntoIterator for Graph {
    type Item = Edge;
    type IntoIter = std::vec::IntoIter<Edge>;

    fn into_iter(self) -> Self::IntoIter {
        self.edges.into_iter()
    }
}

impl<'a> IntoIterator for &'a Graph {
    type Item = &'a Edge;
    type IntoIter = slice::Iter<'a, Edge>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl fmt::Display for Graph {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, edge) in self.edges.iter().enumerate() {
            if index > 0 {
                formatter.write_str("\n")?;
            }
            edge.fmt(formatter)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Graph;
    use crate::{Edge, ImportKind};

    fn fixture_edges() -> [Edge; 2] {
        [
            Edge::self_edge("src/lib.rs"),
            Edge::new("src/lib.rs", "std", true, [ImportKind::PathReference]),
        ]
    }

    #[test]
    fn stores_edges_as_an_immutable_value() {
        let graph = Graph::from_edges(fixture_edges());

        assert_eq!(graph.len(), 2);
        assert!(!graph.is_empty());
        assert_eq!(graph.edges()[0], Edge::self_edge("src/lib.rs"));
    }

    #[test]
    fn collects_and_iterates_in_extraction_order() {
        let graph: Graph = fixture_edges().into_iter().collect();

        let targets = graph
            .iter()
            .map(|edge| edge.target.as_str())
            .collect::<Vec<_>>();
        assert_eq!(targets, vec!["src/lib.rs", "std"]);

        let owned = graph.into_iter().collect::<Vec<_>>();
        assert_eq!(owned, fixture_edges());
    }

    #[test]
    fn renders_one_edge_per_line() {
        let graph = Graph::from_edges(fixture_edges());

        assert_eq!(
            graph.to_string(),
            "src/lib.rs -> itself\nsrc/lib.rs -> std (external) [path_reference]"
        );
    }
}
