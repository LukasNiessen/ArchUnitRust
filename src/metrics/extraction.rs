//! Immutable metrics information extracted from Rust syntax.

mod extract;
mod model;

pub use extract::{MetricsExtractionError, extract_file_metrics, extract_project_metrics};
pub use model::{
    FieldInfo, FileMetricsInfo, ImplInfo, MethodInfo, ProjectMetricsInfo, TypeInfo, TypeKind,
};
