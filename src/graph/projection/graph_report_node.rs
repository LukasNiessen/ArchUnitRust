/// Stable node identity and display label in a graph report snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct GraphReportNode {
    /// Deterministic renderer-safe identifier assigned after label sorting.
    pub id: String,
    /// Collapsed or original graph node label.
    pub label: String,
}

impl GraphReportNode {
    /// Creates one report node.
    #[must_use]
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
        }
    }
}
