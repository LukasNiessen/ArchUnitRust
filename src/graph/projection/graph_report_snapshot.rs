use super::{GraphReportEdge, GraphReportNode, GraphReportSummary};

/// Immutable-by-default graph after filtering, collapsing, aggregation, and counting.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct GraphReportSnapshot {
    /// Human-readable report title.
    pub title: String,
    /// Deterministically sorted final nodes.
    pub nodes: Vec<GraphReportNode>,
    /// Deterministically sorted final edges.
    pub edges: Vec<GraphReportEdge>,
    /// Counts for the selected raw and final graph.
    pub summary: GraphReportSummary,
}

impl GraphReportSnapshot {
    /// Creates one complete renderer-neutral snapshot.
    #[must_use]
    pub fn new(
        title: impl Into<String>,
        nodes: impl IntoIterator<Item = GraphReportNode>,
        edges: impl IntoIterator<Item = GraphReportEdge>,
        summary: GraphReportSummary,
    ) -> Self {
        Self {
            title: title.into(),
            nodes: nodes.into_iter().collect(),
            edges: edges.into_iter().collect(),
            summary,
        }
    }
}
