use std::collections::BTreeMap;

use crate::graph::{GraphReportEdge, GraphReportSnapshot};

use super::escaping::{mermaid_label, single_line};

/// Renders a Mermaid flowchart from a completed snapshot.
#[derive(Debug, Clone, Copy, Default)]
pub struct MermaidRenderer;

impl MermaidRenderer {
    /// Returns deterministic Mermaid source using the snapshot's stable node IDs.
    #[must_use]
    pub fn render(snapshot: &GraphReportSnapshot) -> String {
        let node_ids = snapshot
            .nodes
            .iter()
            .map(|node| (node.label.as_str(), node.id.as_str()))
            .collect::<BTreeMap<_, _>>();
        let mut lines = vec![
            format!("%% {}", single_line(&snapshot.title)),
            "flowchart LR".to_owned(),
        ];
        lines.extend(
            snapshot
                .nodes
                .iter()
                .map(|node| format!("  {}[\"{}\"]", node.id, mermaid_label(&node.label))),
        );
        lines.extend(
            snapshot
                .edges
                .iter()
                .filter_map(|edge| edge_line(edge, &node_ids)),
        );
        lines.join("\n")
    }
}

fn edge_line(edge: &GraphReportEdge, node_ids: &BTreeMap<&str, &str>) -> Option<String> {
    let source = node_ids.get(edge.source.as_str())?;
    let target = node_ids.get(edge.target.as_str())?;
    let arrow = if edge.external { "-.->" } else { "-->" };
    let label = if edge.count > 1 {
        format!("|{}|", edge.count)
    } else {
        String::new()
    };
    Some(format!("  {source} {arrow}{label} {target}"))
}
