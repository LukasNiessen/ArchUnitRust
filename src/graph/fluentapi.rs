//! Immutable entry points and query builders for graph report snapshots.

mod project_graph_builder;
mod project_graphs;

pub use project_graph_builder::ProjectGraphBuilder;
pub use project_graphs::{dependency_graph, dependency_graph_in, project_graph, project_graph_in};
