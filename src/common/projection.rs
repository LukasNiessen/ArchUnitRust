//! Pure graph projections shared by architecture-rule families.

mod mapped_edge;
mod project_edges;
mod project_to_nodes;
mod projected_edge;
mod projected_node;

pub use mapped_edge::MappedEdge;
pub use project_edges::{MapFunction, ProjectedGraph, project_edges};
pub use project_to_nodes::{
    NodeProjectionOptions, project_to_nodes, project_to_nodes_with_options,
};
pub use projected_edge::ProjectedEdge;
pub use projected_node::ProjectedNode;
