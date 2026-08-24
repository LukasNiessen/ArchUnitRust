//! Pure assertions and structured violations for metrics rules.

mod custom_metric;
mod metric_predicate;
mod metric_threshold;
mod metric_zone;

pub use custom_metric::{CustomMetricViolation, gather_custom_metric_violations};
pub use metric_predicate::{MetricPredicateViolation, gather_metric_predicate_violations};
pub use metric_threshold::{
    MetricComparison, MetricThresholdError, MetricThresholdViolation,
    gather_metric_threshold_violations, validate_metric_threshold,
};
pub use metric_zone::{MetricZoneViolation, gather_metric_zone_violations};
