//! Immutable metrics information extracted from Rust syntax.

mod distance;
mod extract;
mod model;

pub use distance::{DistanceInfo, build_distance_infos, extract_distance_infos};
pub use extract::{MetricsExtractionError, extract_file_metrics, extract_project_metrics};
pub use model::{
    FieldInfo, FileMetricsInfo, ImplInfo, MethodInfo, ProjectMetricsInfo, TypeInfo, TypeKind,
};
