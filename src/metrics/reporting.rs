//! Deterministic, self-contained metrics report rendering and export.

mod exporter;
mod options;

pub use exporter::{DEFAULT_METRICS_CSS, MetricsExporter, MetricsReportData};
pub use options::MetricsExportOptions;
