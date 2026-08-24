pub(crate) mod fluentapi;
pub(crate) mod projection;
pub(crate) mod rendering;

pub use fluentapi::{
    ProjectGraphBuilder, dependency_graph, dependency_graph_in, project_graph, project_graph_in,
};
pub use projection::{
    DEFAULT_GRAPH_TITLE, FolderDepthCollapse, GraphCollapse, GraphQueryError, GraphQueryOptions,
    GraphReportEdge, GraphReportNode, GraphReportSnapshot, GraphReportSummary,
    GraphSnapshotFactory, PatternCollapse, aggregate_graph_edges, collapse_graph_node,
    create_graph_snapshot,
};
pub use rendering::{
    CsvRenderer, D2Renderer, DotRenderer, GraphRenderer, GraphReportFormat, HtmlRenderer,
    JsonRenderer, MermaidRenderer, export_graph_report,
};
