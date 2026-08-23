use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    iter::FromIterator,
    slice,
};

use super::{Edge, ImportKind};

/// The extracted dependency graph.
///
/// Edges are ordered by normalized `(source, target)`. Parallel edges are merged and their import
/// kinds are unioned, so every endpoint pair occurs at most once.
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

    /// Creates a deterministic graph and merges parallel endpoint pairs.
    ///
    /// If inconsistent callers describe one endpoint pair as both internal and external, the
    /// internal classification wins. Same-source pairs become canonical marker self-edges.
    #[must_use]
    pub fn from_edges(edges: impl IntoIterator<Item = Edge>) -> Self {
        let mut merged = BTreeMap::<(String, String), (bool, BTreeSet<ImportKind>)>::new();
        for edge in edges {
            let edge = Edge::new(
                &edge.source,
                &edge.target,
                edge.external,
                edge.import_kinds.iter(),
            );
            let key = (edge.source, edge.target);
            let entry = merged
                .entry(key)
                .or_insert_with(|| (edge.external, BTreeSet::new()));
            entry.0 &= edge.external;
            entry.1.extend(edge.import_kinds.iter());
        }

        let edges = merged
            .into_iter()
            .map(|((source, target), (external, kinds))| Edge::new(source, target, external, kinds))
            .collect();
        Self { edges }
    }

    /// Returns all merged edges in deterministic endpoint order.
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
    fn collects_and_iterates_in_endpoint_order() {
        let graph: Graph = fixture_edges().into_iter().rev().collect();

        let targets = graph
            .iter()
            .map(|edge| edge.target.as_str())
            .collect::<Vec<_>>();
        assert_eq!(targets, vec!["src/lib.rs", "std"]);

        let owned = graph.into_iter().collect::<Vec<_>>();
        assert_eq!(owned, fixture_edges());
    }

    #[test]
    fn merges_parallel_kinds_and_canonicalizes_same_file_references() {
        let graph = Graph::from_edges([
            Edge::new("src/a.rs", "src/b.rs", false, [ImportKind::PathReference]),
            Edge::new("src/a.rs", "src/b.rs", false, [ImportKind::Use]),
            Edge::new("src/a.rs", "src/a.rs", false, [ImportKind::PathReference]),
            Edge::self_edge("src/a.rs"),
        ]);

        assert_eq!(graph.len(), 2);
        assert_eq!(graph.edges()[0], Edge::self_edge("src/a.rs"));
        assert_eq!(
            graph.edges()[1].import_kinds.iter().collect::<Vec<_>>(),
            [ImportKind::Use, ImportKind::PathReference]
        );
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
