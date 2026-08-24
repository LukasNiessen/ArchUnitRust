//! Rust-native source metrics, calculations, and fluent queries.

pub mod assertion;
pub mod calculation;
pub mod extraction;
pub mod fluentapi;
pub mod reporting;

pub use assertion::{
    CustomMetricViolation, MetricComparison, MetricPredicateViolation, MetricThresholdError,
    MetricThresholdViolation, MetricZoneViolation, gather_custom_metric_violations,
    gather_metric_predicate_violations, gather_metric_threshold_violations,
    gather_metric_zone_violations, validate_metric_threshold,
};
pub use calculation::{
    ArchitecturalZone, CountMetric, DistanceInput, DistanceMetric, LcomInput, LcomMetric,
    MAXIMUM_SIZE_DISCOUNT, MetricMeasurement, MetricSubject, PAIN_LIMIT, SIZE_NORMALIZATION_LINES,
    USELESSNESS_LIMIT,
};
pub use extraction::{
    DistanceInfo, FieldInfo, FileMetricsInfo, ImplInfo, MethodInfo, MetricsExtractionError,
    ProjectMetricsInfo, TypeInfo, TypeKind, build_distance_infos, extract_distance_infos,
    extract_file_metrics, extract_project_metrics,
};
pub use fluentapi::{
    CountMetricsBuilder, CustomMetricCondition, CustomMetricSelection, DistanceMetricSelection,
    DistanceMetricsBuilder, LcomMetricSelection, LcomMetricsBuilder, MetricPredicateCondition,
    MetricSelection, MetricThresholdCondition, MetricZoneCondition, MetricsBuilder, metrics,
    metrics_in,
};
pub use reporting::{
    DEFAULT_METRICS_CSS, MetricsExportOptions, MetricsExporter, MetricsReportData,
};
