use crate::common::Edge;

/// A labeled dependency retaining every raw edge collapsed into it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectedEdge {
    /// The projected label for the dependency source.
    pub source_label: String,
    /// The projected label for the dependency target.
    pub target_label: String,
    /// The concrete extracted dependencies represented by this edge.
    pub cumulated_edges: Vec<Edge>,
}

impl ProjectedEdge {
    /// Creates a projected dependency with its raw evidence.
    #[must_use]
    pub fn new(
        source_label: impl Into<String>,
        target_label: impl Into<String>,
        cumulated_edges: impl IntoIterator<Item = Edge>,
    ) -> Self {
        Self {
            source_label: source_label.into(),
            target_label: target_label.into(),
            cumulated_edges: cumulated_edges.into_iter().collect(),
        }
    }

    /// Returns whether both projected endpoints have the same label.
    #[must_use]
    pub fn is_self_edge(&self) -> bool {
        self.source_label == self.target_label
    }
}

#[cfg(test)]
mod tests {
    use crate::common::{Edge, ImportKind};

    use super::ProjectedEdge;

    #[test]
    fn owns_all_raw_evidence() {
        let raw = Edge::new("src/api.rs", "src/db.rs", false, [ImportKind::Use]);
        let projected = ProjectedEdge::new("api", "database", [raw.clone()]);

        assert_eq!(projected.cumulated_edges, [raw]);
    }

    #[test]
    fn identifies_projected_self_edges_by_labels() {
        let raw = Edge::self_edge("src/lib.rs");

        assert!(ProjectedEdge::new("crate", "crate", [raw]).is_self_edge());
    }
}
