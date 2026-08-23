//! Deterministic text renderers over one completed graph report snapshot.

mod csv_renderer;
mod d2_renderer;
mod dot_renderer;
mod escaping;
mod graph_renderer;
mod html_renderer;
mod json_renderer;
mod mermaid_renderer;

pub use csv_renderer::CsvRenderer;
pub use d2_renderer::D2Renderer;
pub use dot_renderer::DotRenderer;
pub use graph_renderer::{GraphRenderer, GraphReportFormat};
pub use html_renderer::HtmlRenderer;
pub use json_renderer::JsonRenderer;
pub use mermaid_renderer::MermaidRenderer;
