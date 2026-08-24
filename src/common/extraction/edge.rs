use std::fmt;

use super::identifier::normalize_identifier;
use super::{ImportKind, ImportKindSet};

/// One directed dependency in an extracted Rust project.
///
/// Internal endpoints are normalized workspace-relative file identifiers. When [`Self::external`]
/// is true, `target` is instead the Cargo-visible crate name used in source.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct Edge {
    /// The internal file containing the dependency syntax.
    pub source: String,
    /// The internal target file or external Cargo-visible crate name.
    pub target: String,
    /// Whether the target is outside the analyzed workspace.
    pub external: bool,
    /// Every Rust syntax form that produced this source-target pair.
    pub import_kinds: ImportKindSet,
}

impl Edge {
    /// Builds an edge and lexically normalizes both identifiers.
    ///
    /// An edge whose normalized endpoints are equal is canonicalized to [`Self::self_edge`].
    #[must_use]
    pub fn new(
        source: impl AsRef<str>,
        target: impl AsRef<str>,
        external: bool,
        import_kinds: impl IntoIterator<Item = ImportKind>,
    ) -> Self {
        let source = normalize_identifier(source.as_ref());
        let target = normalize_identifier(target.as_ref());

        if source == target {
            return Self::self_edge(source);
        }

        Self {
            source,
            target,
            external,
            import_kinds: import_kinds.into_iter().collect(),
        }
    }

    /// Builds the marker edge that keeps a dependency-free file in the graph.
    #[must_use]
    pub fn self_edge(identifier: impl AsRef<str>) -> Self {
        let identifier = normalize_identifier(identifier.as_ref());
        Self {
            source: identifier.clone(),
            target: identifier,
            external: false,
            import_kinds: ImportKindSet::new(),
        }
    }

    /// Returns whether this edge is the marker from a file to itself.
    #[must_use]
    pub fn is_self_edge(&self) -> bool {
        self.source == self.target
    }
}

impl fmt::Display for Edge {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_self_edge() {
            return write!(formatter, "{} -> itself", self.source);
        }

        write!(formatter, "{} -> {}", self.source, self.target)?;
        if self.external {
            formatter.write_str(" (external)")?;
        }
        write!(formatter, " {}", self.import_kinds)
    }
}

#[cfg(test)]
mod tests {
    use super::Edge;
    use crate::common::ImportKind;

    #[test]
    fn normalizes_internal_file_identifiers() {
        let edge = Edge::new(
            r".\crates\api\src\handler.rs",
            "crates/api/src/../db/repository.rs",
            false,
            [ImportKind::Use],
        );

        assert_eq!(edge.source, "crates/api/src/handler.rs");
        assert_eq!(edge.target, "crates/api/db/repository.rs");
        assert!(!edge.external);
        assert!(edge.import_kinds.contains(ImportKind::Use));
    }

    #[test]
    fn keeps_external_crate_names() {
        let edge = Edge::new(
            "crates/api/src/lib.rs",
            "serde_json",
            true,
            [ImportKind::PathReference, ImportKind::Use],
        );

        assert_eq!(edge.target, "serde_json");
        assert!(edge.external);
        assert_eq!(edge.import_kinds.len(), 2);
    }

    #[test]
    fn creates_one_canonical_self_edge_shape() {
        let edge = Edge::new(
            "crates/api/src/lib.rs",
            r"crates\api\src\.\lib.rs",
            true,
            [ImportKind::Use],
        );

        assert!(edge.is_self_edge());
        assert!(!edge.external);
        assert!(edge.import_kinds.is_empty());
        assert_eq!(edge, Edge::self_edge("crates/api/src/lib.rs"));
    }

    #[test]
    fn renders_diagnostic_text_deterministically() {
        let edge = Edge::new(
            "src/api.rs",
            "src/db.rs",
            false,
            [ImportKind::PubUse, ImportKind::Use],
        );
        let external = Edge::new("src/api.rs", "tokio", true, [ImportKind::MacroReference]);

        assert_eq!(edge.to_string(), "src/api.rs -> src/db.rs [use, pub_use]");
        assert_eq!(
            external.to_string(),
            "src/api.rs -> tokio (external) [macro_reference]"
        );
        assert_eq!(
            Edge::self_edge("src/api.rs").to_string(),
            "src/api.rs -> itself"
        );
    }
}
