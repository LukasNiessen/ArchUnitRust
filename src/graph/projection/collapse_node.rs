use super::{GraphCollapse, GraphQueryError};

/// Applies one optional collapse strategy to a graph node label.
pub fn collapse_graph_node(
    node: &str,
    collapse: Option<&GraphCollapse>,
) -> Result<String, GraphQueryError> {
    collapse.map_or_else(|| Ok(node.to_owned()), |strategy| strategy.collapse(node))
}
