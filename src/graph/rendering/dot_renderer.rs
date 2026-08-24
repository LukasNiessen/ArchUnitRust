use crate::graph::{GraphReportEdge, GraphReportSnapshot};

use super::escaping::quoted;

/// Renders a Graphviz DOT directed graph from a completed snapshot.
#[derive(Debug, Clone, Copy, Default)]
pub struct DotRenderer;

impl DotRenderer {
    /// Returns deterministic DOT source.
    #[must_use]
    pub fn render(snapshot: &GraphReportSnapshot) -> String {
        let mut lines = vec![
            "digraph dependencies {".to_owned(),
            "  rankdir=LR;".to_owned(),
            format!("  label={};", quoted(&snapshot.title)),
            "  labelloc=t;".to_owned(),
        ];
        lines.extend(
            snapshot
                .nodes
                .iter()
                .map(|node| format!("  {};", quoted(&node.label))),
        );
        lines.extend(snapshot.edges.iter().map(edge_line));
        lines.push("}".to_owned());
        lines.join("\n")
    }
}

fn edge_line(edge: &GraphReportEdge) -> String {
    let mut attributes = Vec::new();
    if edge.count > 1 {
        attributes.push(format!("label={}", quoted(&edge.count.to_string())));
    }
    if edge.external {
        attributes.push("style=dashed".to_owned());
    }
    if !edge.import_kinds.is_empty() {
        attributes.push(format!(
            "tooltip={}",
            quoted(
                &edge
                    .import_kinds
                    .iter()
                    .map(|kind| kind.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        ));
    }
    let suffix = if attributes.is_empty() {
        String::new()
    } else {
        format!(" [{}]", attributes.join(", "))
    };

    format!(
        "  {} -> {}{suffix};",
        quoted(&edge.source),
        quoted(&edge.target)
    )
}
