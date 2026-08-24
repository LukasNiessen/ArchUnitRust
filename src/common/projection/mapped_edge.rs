use crate::common::Edge;

/// The labels produced when a raw dependency is mapped into a domain view.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MappedEdge {
    /// The projected label for the dependency source.
    pub source_label: String,
    /// The projected label for the dependency target.
    pub target_label: String,
}

impl MappedEdge {
    /// Creates a mapped edge without normalizing its domain-specific labels.
    #[must_use]
    pub fn new(source_label: impl Into<String>, target_label: impl Into<String>) -> Self {
        Self {
            source_label: source_label.into(),
            target_label: target_label.into(),
        }
    }

    /// Maps one raw edge to its unchanged endpoint labels.
    #[must_use]
    pub fn from_edge(edge: &Edge) -> Self {
        Self::new(&edge.source, &edge.target)
    }
}

#[cfg(test)]
mod tests {
    use crate::common::{Edge, ImportKind};

    use super::MappedEdge;

    #[test]
    fn owns_domain_labels_without_normalizing_them() {
        let mut source = String::from("API Layer");
        let mapped = MappedEdge::new(&source, "Domain Layer");
        source.clear();

        assert_eq!(mapped.source_label, "API Layer");
        assert_eq!(mapped.target_label, "Domain Layer");
    }

    #[test]
    fn maps_raw_endpoints_by_identity() {
        let edge = Edge::new("src/api.rs", "src/domain.rs", false, [ImportKind::Use]);

        assert_eq!(
            MappedEdge::from_edge(&edge),
            MappedEdge::new("src/api.rs", "src/domain.rs")
        );
    }
}
