use crate::metrics::{MetricMeasurement, MetricSubject};

/// One built-in metric value that did not satisfy a user predicate.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct MetricPredicateViolation {
    /// The complete file, type, or distance subject evidence.
    pub subject: MetricSubject,
    /// The stable built-in metric name.
    pub metric_name: String,
    /// The value rejected by the predicate.
    pub value: f64,
}

impl MetricPredicateViolation {
    /// Creates structured predicate violation data.
    #[must_use]
    pub fn new(subject: MetricSubject, metric_name: impl Into<String>, value: f64) -> Self {
        Self {
            subject,
            metric_name: metric_name.into(),
            value,
        }
    }

    /// Returns the file path or type name identifying the rejected subject.
    #[must_use]
    pub fn identifier(&self) -> &str {
        self.subject.identifier()
    }
}

/// Returns structured violations for built-in measurements rejected by `predicate`.
///
/// The callback runs exactly once per measurement in input order. Its panics propagate normally.
#[must_use]
pub fn gather_metric_predicate_violations<Predicate>(
    measurements: &[MetricMeasurement],
    predicate: &Predicate,
) -> Vec<MetricPredicateViolation>
where
    Predicate: Fn(f64, &MetricSubject) -> bool,
{
    measurements
        .iter()
        .filter_map(|measurement| {
            if predicate(measurement.value(), measurement.subject()) {
                None
            } else {
                Some(MetricPredicateViolation::new(
                    measurement.subject().clone(),
                    measurement.metric_name(),
                    measurement.value(),
                ))
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use crate::metrics::{MetricMeasurement, MetricSubject, extract_file_metrics};

    use super::gather_metric_predicate_violations;

    #[test]
    fn invokes_the_predicate_once_with_value_and_complete_subject() {
        let type_info = extract_file_metrics("src/service.rs", "struct Service { port: usize }")
            .expect("fixture should parse")
            .types()
            .first()
            .expect("fixture should contain one type")
            .clone();
        let measurement = MetricMeasurement::from_parts(
            MetricSubject::Type(type_info),
            "field_count",
            "fields",
            1.0,
        );
        let calls = Cell::new(0);

        let violations = gather_metric_predicate_violations(&[measurement], &|value, subject| {
            calls.set(calls.get() + 1);
            value == 0.0 && subject.identifier() == "Service"
        });

        assert_eq!(calls.get(), 1);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].identifier(), "Service");
        assert_eq!(violations[0].metric_name, "field_count");
        assert_eq!(violations[0].value, 1.0);
    }
}
