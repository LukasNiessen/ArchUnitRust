use crate::graph::{GraphReportEdge, GraphReportSnapshot};

use super::escaping::{quoted, single_line};

/// Renders D2 diagram source from a completed snapshot.
#[derive(Debug, Clone, Copy, Default)]
pub struct D2Renderer;

impl D2Renderer {
    /// Returns deterministic D2 source.
    #[must_use]
    pub fn render(snapshot: &GraphReportSnapshot) -> String {
        let mut lines = vec![format!("# {}", single_line(&snapshot.title))];
        lines.extend(snapshot.nodes.iter().map(|node| quoted(&node.label)));
        lines.extend(snapshot.edges.iter().map(edge_line));
        lines.join("\n")
    }
}

fn edge_line(edge: &GraphReportEdge) -> String {
    let label = if edge.count > 1 {
        format!(": {}", quoted(&edge.count.to_string()))
    } else {
        String::new()
    };
    let style = if edge.external {
        " { style.stroke-dash: 4 }"
    } else {
        ""
    };
    format!(
        "{} -> {}{label}{style}",
        quoted(&edge.source),
        quoted(&edge.target)
    )
}
