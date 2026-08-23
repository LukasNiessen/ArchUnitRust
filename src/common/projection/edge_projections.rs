use crate::Edge;

use super::MappedEdge;

type EdgeProjection = fn(&Edge) -> Option<MappedEdge>;

/// Returns a mapper for every dependency except source-file self-edges.
#[must_use]
pub fn per_edge() -> EdgeProjection {
    map_non_self_edge
}

/// Returns a mapper for non-self dependencies within the extracted project.
#[must_use]
pub fn per_internal_edge() -> EdgeProjection {
    map_internal_edge
}

/// Returns a mapper for non-self dependencies targeting external crates.
#[must_use]
pub fn per_external_edge() -> EdgeProjection {
    map_external_edge
}

/// Returns a mapper for every raw edge, including source-file self-edges.
#[must_use]
pub fn identity() -> EdgeProjection {
    map_identity
}

fn map_non_self_edge(edge: &Edge) -> Option<MappedEdge> {
    (!edge.is_self_edge()).then(|| MappedEdge::from_edge(edge))
}

fn map_internal_edge(edge: &Edge) -> Option<MappedEdge> {
    (!edge.external && !edge.is_self_edge()).then(|| MappedEdge::from_edge(edge))
}

fn map_external_edge(edge: &Edge) -> Option<MappedEdge> {
    (edge.external && !edge.is_self_edge()).then(|| MappedEdge::from_edge(edge))
}

fn map_identity(edge: &Edge) -> Option<MappedEdge> {
    Some(MappedEdge::from_edge(edge))
}

#[cfg(test)]
mod tests {
    use crate::{Edge, ImportKind};

    use super::{identity, per_edge, per_external_edge, per_internal_edge};
    use crate::MappedEdge;

    fn internal_edge() -> Edge {
        Edge::new("src/source.rs", "src/target.rs", false, [ImportKind::Use])
    }

    fn external_edge() -> Edge {
        Edge::new("src/source.rs", "serde", true, [ImportKind::Use])
    }

    #[test]
    fn per_edge_maps_internal_and_external_dependencies_but_not_self_edges() {
        let mapper = per_edge();

        assert_eq!(
            mapper(&internal_edge()),
            Some(MappedEdge::new("src/source.rs", "src/target.rs"))
        );
        assert_eq!(
            mapper(&external_edge()),
            Some(MappedEdge::new("src/source.rs", "serde"))
        );
        assert_eq!(mapper(&Edge::self_edge("src/source.rs")), None);
    }

    #[test]
    fn per_internal_edge_keeps_only_internal_dependencies() {
        let mapper = per_internal_edge();

        assert_eq!(
            mapper(&internal_edge()),
            Some(MappedEdge::new("src/source.rs", "src/target.rs"))
        );
        assert_eq!(mapper(&external_edge()), None);
        assert_eq!(mapper(&Edge::self_edge("src/source.rs")), None);
    }

    #[test]
    fn per_external_edge_keeps_only_external_dependencies() {
        let mapper = per_external_edge();

        assert_eq!(mapper(&internal_edge()), None);
        assert_eq!(
            mapper(&external_edge()),
            Some(MappedEdge::new("src/source.rs", "serde"))
        );
        assert_eq!(mapper(&Edge::self_edge("src/source.rs")), None);
    }

    #[test]
    fn identity_keeps_source_file_self_edges() {
        let self_edge = Edge::self_edge("src/source.rs");

        assert_eq!(
            identity()(&self_edge),
            Some(MappedEdge::new("src/source.rs", "src/source.rs"))
        );
    }
}
