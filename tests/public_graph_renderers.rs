use std::{fs, path::PathBuf, time::SystemTime};

use archunit::{GraphRenderer, GraphReportFormat, dependency_graph_in};

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/layered_project")
}

struct TemporaryDirectory(PathBuf);

impl TemporaryDirectory {
    fn new(test_name: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("system clock should follow the Unix epoch")
            .as_nanos();
        Self(std::env::temp_dir().join(format!(
            "archunit-public-graph-renderers-{}-{test_name}-{nonce}",
            std::process::id()
        )))
    }

    fn join(&self, path: &str) -> PathBuf {
        self.0.join(path)
    }
}

impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn every_string_renderer_uses_the_same_public_snapshot_contract() {
    let report = dependency_graph_in(fixture()).titled("Layered λ Architecture");
    let snapshot = report
        .snapshot()
        .expect("the layered Cargo fixture should be analyzable");

    let rendered = [
        (GraphReportFormat::Dot, report.to_dot()),
        (GraphReportFormat::Mermaid, report.to_mermaid()),
        (GraphReportFormat::D2, report.to_d2()),
        (GraphReportFormat::Csv, report.to_csv()),
        (GraphReportFormat::Json, report.to_json()),
        (GraphReportFormat::Html, report.to_html()),
    ];

    for (format, actual) in rendered {
        let actual = actual.expect("the fluent renderer should analyze the fixture");
        assert_eq!(actual, GraphRenderer::render(&snapshot, format));
        assert!(actual.contains("src/api/mod.rs"));
    }

    let json = report.to_json().expect("JSON rendering should succeed");
    let value: serde_json::Value =
        serde_json::from_str(&json).expect("graph JSON should be standards-compliant");
    assert_eq!(value["title"], "Layered λ Architecture");
    assert_eq!(value["summary"]["node_count"], 7);

    let html = report.to_html().expect("HTML rendering should succeed");
    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(html.contains("Layered λ Architecture"));
    assert!(!html.contains("<script"));
    assert!(!html.contains("http://"));
    assert!(!html.contains("https://"));
}

#[test]
fn every_named_export_terminal_writes_exact_utf8_renderer_output() {
    let report = dependency_graph_in(fixture()).titled("Layered λ Architecture");
    let snapshot = report
        .snapshot()
        .expect("the layered Cargo fixture should be analyzable");
    let output = TemporaryDirectory::new("exports");
    let files = [
        (
            GraphReportFormat::Dot,
            output.join("nested/architecture.dot"),
        ),
        (
            GraphReportFormat::Mermaid,
            output.join("nested/architecture.mmd"),
        ),
        (GraphReportFormat::D2, output.join("nested/architecture.d2")),
        (
            GraphReportFormat::Csv,
            output.join("nested/architecture.csv"),
        ),
        (
            GraphReportFormat::Json,
            output.join("nested/architecture.json"),
        ),
        (
            GraphReportFormat::Html,
            output.join("nested/architecture.html"),
        ),
    ];

    report
        .export_as_dot(&files[0].1)
        .expect("DOT export should succeed");
    report
        .export_as_mermaid(&files[1].1)
        .expect("Mermaid export should succeed");
    report
        .export_as_d2(&files[2].1)
        .expect("D2 export should succeed");
    report
        .export_as_csv(&files[3].1)
        .expect("CSV export should succeed");
    report
        .export_as_json(&files[4].1)
        .expect("JSON export should succeed");
    report
        .export_as_html(&files[5].1)
        .expect("HTML export should succeed");

    for (format, path) in files {
        assert_eq!(
            fs::read_to_string(path).expect("export should be readable as UTF-8"),
            GraphRenderer::render(&snapshot, format)
        );
    }
}
