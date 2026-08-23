use crate::GraphReportSnapshot;

/// Renders the complete graph snapshot as stable JSON.
#[derive(Debug, Clone, Copy, Default)]
pub struct JsonRenderer;

impl JsonRenderer {
    /// Returns pretty, valid JSON containing every snapshot field.
    #[must_use]
    pub fn render(snapshot: &GraphReportSnapshot) -> String {
        let nodes = snapshot
            .nodes
            .iter()
            .map(|node| serde_json::json!({ "id": node.id, "label": node.label }))
            .collect::<Vec<_>>();
        let edges = snapshot
            .edges
            .iter()
            .map(|edge| {
                serde_json::json!({
                    "source": edge.source,
                    "target": edge.target,
                    "count": edge.count,
                    "external": edge.external,
                    "import_kinds": edge.import_kinds.iter().map(|kind| kind.as_str()).collect::<Vec<_>>()
                })
            })
            .collect::<Vec<_>>();
        let summary = &snapshot.summary;
        let value = serde_json::json!({
            "title": snapshot.title,
            "nodes": nodes,
            "edges": edges,
            "summary": {
                "node_count": summary.node_count,
                "edge_count": summary.edge_count,
                "raw_edge_count": summary.raw_edge_count,
                "external_edge_count": summary.external_edge_count
            }
        });

        format!("{value:#}")
    }
}
