use crate::GraphReportSnapshot;

use super::escaping::csv_field;

/// Renders aggregated graph edges as standards-compliant CSV.
#[derive(Debug, Clone, Copy, Default)]
pub struct CsvRenderer;

impl CsvRenderer {
    /// Returns deterministic CSV with one row per aggregated edge.
    #[must_use]
    pub fn render(snapshot: &GraphReportSnapshot) -> String {
        let mut rows = vec!["source,target,count,external,import_kinds".to_owned()];
        rows.extend(snapshot.edges.iter().map(|edge| {
            let kinds = edge
                .import_kinds
                .iter()
                .map(|kind| kind.as_str())
                .collect::<Vec<_>>()
                .join("|");
            [
                csv_field(&edge.source),
                csv_field(&edge.target),
                edge.count.to_string(),
                edge.external.to_string(),
                csv_field(&kinds),
            ]
            .join(",")
        }));
        rows.join("\n")
    }
}
