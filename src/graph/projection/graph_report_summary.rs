/// Counts describing the selected and aggregated graph snapshot.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct GraphReportSummary {
    /// Number of final report nodes.
    pub node_count: usize,
    /// Number of final aggregated report edges.
    pub edge_count: usize,
    /// Number of selected graph edges before collapse aggregation.
    pub raw_edge_count: usize,
    /// Number of selected raw edges targeting external dependencies.
    pub external_edge_count: usize,
}

impl GraphReportSummary {
    /// Creates complete snapshot counts.
    #[must_use]
    pub const fn new(
        node_count: usize,
        edge_count: usize,
        raw_edge_count: usize,
        external_edge_count: usize,
    ) -> Self {
        Self {
            node_count,
            edge_count,
            raw_edge_count,
            external_edge_count,
        }
    }
}
