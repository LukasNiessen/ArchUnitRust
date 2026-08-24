use std::path::Path;

use crate::{common::ArchUnitError, graph::GraphReportSnapshot};

use super::{
    CsvRenderer, D2Renderer, DotRenderer, HtmlRenderer, JsonRenderer, MermaidRenderer,
    export_graph_report,
};

/// A supported graph report output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum GraphReportFormat {
    /// Graphviz DOT.
    Dot,
    /// Mermaid flowchart source.
    Mermaid,
    /// D2 diagram source.
    D2,
    /// Aggregated dependency CSV.
    Csv,
    /// Complete snapshot JSON.
    Json,
    /// Self-contained offline HTML.
    Html,
}

/// Dispatches every output format from the same completed snapshot.
#[derive(Debug, Clone, Copy, Default)]
pub struct GraphRenderer;

impl GraphRenderer {
    /// Renders `snapshot` as `format`.
    #[must_use]
    pub fn render(snapshot: &GraphReportSnapshot, format: GraphReportFormat) -> String {
        match format {
            GraphReportFormat::Dot => Self::to_dot(snapshot),
            GraphReportFormat::Mermaid => Self::to_mermaid(snapshot),
            GraphReportFormat::D2 => Self::to_d2(snapshot),
            GraphReportFormat::Csv => Self::to_csv(snapshot),
            GraphReportFormat::Json => Self::to_json(snapshot),
            GraphReportFormat::Html => Self::to_html(snapshot),
        }
    }

    /// Renders Graphviz DOT.
    #[must_use]
    pub fn to_dot(snapshot: &GraphReportSnapshot) -> String {
        DotRenderer::render(snapshot)
    }

    /// Renders Mermaid flowchart source.
    #[must_use]
    pub fn to_mermaid(snapshot: &GraphReportSnapshot) -> String {
        MermaidRenderer::render(snapshot)
    }

    /// Renders D2 diagram source.
    #[must_use]
    pub fn to_d2(snapshot: &GraphReportSnapshot) -> String {
        D2Renderer::render(snapshot)
    }

    /// Renders aggregated dependency CSV.
    #[must_use]
    pub fn to_csv(snapshot: &GraphReportSnapshot) -> String {
        CsvRenderer::render(snapshot)
    }

    /// Renders complete snapshot JSON.
    #[must_use]
    pub fn to_json(snapshot: &GraphReportSnapshot) -> String {
        JsonRenderer::render(snapshot)
    }

    /// Renders a self-contained offline HTML report.
    #[must_use]
    pub fn to_html(snapshot: &GraphReportSnapshot) -> String {
        HtmlRenderer::render(snapshot)
    }

    /// Renders `snapshot` as `format` and writes it as UTF-8.
    pub fn export(
        snapshot: &GraphReportSnapshot,
        format: GraphReportFormat,
        output_path: impl AsRef<Path>,
    ) -> Result<(), ArchUnitError> {
        export_graph_report(output_path, &Self::render(snapshot, format))
    }

    /// Exports Graphviz DOT as UTF-8.
    pub fn export_as_dot(
        snapshot: &GraphReportSnapshot,
        output_path: impl AsRef<Path>,
    ) -> Result<(), ArchUnitError> {
        export_graph_report(output_path, &Self::to_dot(snapshot))
    }

    /// Exports Mermaid flowchart source as UTF-8.
    pub fn export_as_mermaid(
        snapshot: &GraphReportSnapshot,
        output_path: impl AsRef<Path>,
    ) -> Result<(), ArchUnitError> {
        export_graph_report(output_path, &Self::to_mermaid(snapshot))
    }

    /// Exports D2 diagram source as UTF-8.
    pub fn export_as_d2(
        snapshot: &GraphReportSnapshot,
        output_path: impl AsRef<Path>,
    ) -> Result<(), ArchUnitError> {
        export_graph_report(output_path, &Self::to_d2(snapshot))
    }

    /// Exports aggregated dependency CSV as UTF-8.
    pub fn export_as_csv(
        snapshot: &GraphReportSnapshot,
        output_path: impl AsRef<Path>,
    ) -> Result<(), ArchUnitError> {
        export_graph_report(output_path, &Self::to_csv(snapshot))
    }

    /// Exports complete snapshot JSON as UTF-8.
    pub fn export_as_json(
        snapshot: &GraphReportSnapshot,
        output_path: impl AsRef<Path>,
    ) -> Result<(), ArchUnitError> {
        export_graph_report(output_path, &Self::to_json(snapshot))
    }

    /// Exports a self-contained offline HTML report as UTF-8.
    pub fn export_as_html(
        snapshot: &GraphReportSnapshot,
        output_path: impl AsRef<Path>,
    ) -> Result<(), ArchUnitError> {
        export_graph_report(output_path, &Self::to_html(snapshot))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        common::ImportKind,
        graph::{GraphReportEdge, GraphReportNode, GraphReportSnapshot, GraphReportSummary},
    };

    use super::{GraphRenderer, GraphReportFormat};

    fn snapshot() -> GraphReportSnapshot {
        GraphReportSnapshot::new(
            "Architecture <Main>",
            [
                GraphReportNode::new("n0", "app/\"api\".rs"),
                GraphReportNode::new("n1", "domain<&>.rs"),
                GraphReportNode::new("n2", "serde"),
            ],
            [
                GraphReportEdge::new(
                    "app/\"api\".rs",
                    "domain<&>.rs",
                    2,
                    false,
                    [ImportKind::Use, ImportKind::PubUse],
                ),
                GraphReportEdge::new(
                    "app/\"api\".rs",
                    "serde",
                    1,
                    true,
                    [ImportKind::PathReference],
                ),
            ],
            GraphReportSummary::new(3, 2, 3, 1),
        )
    }

    #[test]
    fn dot_escapes_labels_and_includes_counts_kinds_and_external_style() {
        let dot = GraphRenderer::to_dot(&snapshot());

        assert!(dot.starts_with("digraph dependencies {"));
        assert!(dot.contains("label=\"Architecture <Main>\";"));
        assert!(dot.contains(
            "\"app/\\\"api\\\".rs\" -> \"domain<&>.rs\" [label=\"2\", tooltip=\"use, pub_use\"]"
        ));
        assert!(dot.contains(
            "\"app/\\\"api\\\".rs\" -> \"serde\" [style=dashed, tooltip=\"path_reference\"]"
        ));
        assert!(dot.ends_with('}'));
    }

    #[test]
    fn mermaid_uses_stable_ids_safe_labels_counts_and_external_arrows() {
        let mermaid = GraphRenderer::to_mermaid(&snapshot());

        assert!(mermaid.contains("flowchart LR"));
        assert!(mermaid.contains("n0[\"app/&quot;api&quot;.rs\"]"));
        assert!(mermaid.contains("n1[\"domain&lt;&amp;&gt;.rs\"]"));
        assert!(mermaid.contains("n0 -->|2| n1"));
        assert!(mermaid.contains("n0 -.-> n2"));
    }

    #[test]
    fn d2_quotes_labels_and_styles_aggregated_external_edges() {
        let d2 = GraphRenderer::to_d2(&snapshot());

        assert!(d2.starts_with("# Architecture <Main>"));
        assert!(d2.contains("\"app/\\\"api\\\".rs\" -> \"domain<&>.rs\": \"2\""));
        assert!(d2.contains("\"app/\\\"api\\\".rs\" -> \"serde\" { style.stroke-dash: 4 }"));
    }

    #[test]
    fn csv_is_standards_compliant_and_contains_all_edge_evidence() {
        let csv = GraphRenderer::to_csv(&snapshot());
        let lines = csv.lines().collect::<Vec<_>>();

        assert_eq!(lines[0], "source,target,count,external,import_kinds");
        assert_eq!(
            lines[1],
            "\"app/\"\"api\"\".rs\",domain<&>.rs,2,false,use|pub_use"
        );
        assert_eq!(
            lines[2],
            "\"app/\"\"api\"\".rs\",serde,1,true,path_reference"
        );
    }

    #[test]
    fn json_contains_the_complete_parseable_snapshot() {
        let json = GraphRenderer::to_json(&snapshot());
        let parsed: serde_json::Value =
            serde_json::from_str(&json).expect("renderer should produce valid JSON");

        assert_eq!(parsed["title"], "Architecture <Main>");
        assert_eq!(parsed["nodes"].as_array().map(Vec::len), Some(3));
        assert_eq!(parsed["edges"][0]["count"], 2);
        assert_eq!(parsed["edges"][0]["import_kinds"][1], "pub_use");
        assert_eq!(parsed["summary"]["raw_edge_count"], 3);
    }

    #[test]
    fn html_is_one_escaped_offline_document_with_portable_sources() {
        let html = GraphRenderer::to_html(&snapshot());

        assert!(html.starts_with("<!DOCTYPE html>"));
        assert!(html.contains("<title>Architecture &lt;Main&gt;</title>"));
        assert!(html.contains("Generated by ArchUnitRust graph reporting"));
        assert!(html.contains("<strong>3</strong>Nodes"));
        assert!(html.contains("app/&quot;api&quot;.rs"));
        assert!(html.contains("domain&lt;&amp;&gt;.rs"));
        assert!(html.contains("<summary>Mermaid</summary>"));
        assert!(html.contains("<summary>JSON snapshot</summary>"));
        assert!(!html.contains("http://"));
        assert!(!html.contains("https://"));
        assert!(!html.contains("<script"));
        assert_eq!(html.matches("<html").count(), 1);
        assert_eq!(html.matches("</html>").count(), 1);
    }

    #[test]
    fn format_dispatch_is_exactly_the_six_specific_renderers() {
        let snapshot = snapshot();
        let cases = [
            (GraphReportFormat::Dot, GraphRenderer::to_dot(&snapshot)),
            (
                GraphReportFormat::Mermaid,
                GraphRenderer::to_mermaid(&snapshot),
            ),
            (GraphReportFormat::D2, GraphRenderer::to_d2(&snapshot)),
            (GraphReportFormat::Csv, GraphRenderer::to_csv(&snapshot)),
            (GraphReportFormat::Json, GraphRenderer::to_json(&snapshot)),
            (GraphReportFormat::Html, GraphRenderer::to_html(&snapshot)),
        ];

        for (format, expected) in cases {
            assert_eq!(GraphRenderer::render(&snapshot, format), expected);
        }
    }
}
