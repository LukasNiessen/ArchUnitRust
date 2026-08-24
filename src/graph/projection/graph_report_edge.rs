use crate::common::{ImportKind, ImportKindSet};

/// Aggregated dependency evidence in a graph report snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct GraphReportEdge {
    /// Collapsed or original source-node label.
    pub source: String,
    /// Collapsed or original target-node label.
    pub target: String,
    /// Number of selected raw graph edges represented by this edge.
    pub count: usize,
    /// Whether at least one represented edge targets an external dependency.
    pub external: bool,
    /// Union of every Rust import kind represented by this edge.
    pub import_kinds: ImportKindSet,
}

impl GraphReportEdge {
    /// Creates one aggregated report edge.
    #[must_use]
    pub fn new(
        source: impl Into<String>,
        target: impl Into<String>,
        count: usize,
        external: bool,
        import_kinds: impl IntoIterator<Item = ImportKind>,
    ) -> Self {
        Self {
            source: source.into(),
            target: target.into(),
            count,
            external,
            import_kinds: import_kinds.into_iter().collect(),
        }
    }
}
