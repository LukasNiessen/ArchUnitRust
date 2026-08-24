use std::fmt;

use crate::{MetricMeasurement, MetricSubject};

/// One exact numeric relationship used by the five threshold verbs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum MetricComparison {
    /// The value must be strictly smaller than the threshold.
    Below,
    /// The value must be strictly larger than the threshold.
    Above,
    /// The value must equal the threshold exactly.
    Equal,
    /// The value must be smaller than or equal to the threshold.
    BelowOrEqual,
    /// The value must be larger than or equal to the threshold.
    AboveOrEqual,
}

impl MetricComparison {
    /// All five numeric comparisons in stable fluent order.
    pub const ALL: [Self; 5] = [
        Self::Below,
        Self::Above,
        Self::Equal,
        Self::BelowOrEqual,
        Self::AboveOrEqual,
    ];

    /// Returns the stable comparison key.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Below => "below",
            Self::Above => "above",
            Self::Equal => "equal",
            Self::BelowOrEqual => "below-or-equal",
            Self::AboveOrEqual => "above-or-equal",
        }
    }

    /// Returns whether `value` satisfies this exact comparison against `threshold`.
    #[must_use]
    pub fn is_satisfied(self, value: f64, threshold: f64) -> bool {
        match self {
            Self::Below => value < threshold,
            Self::Above => value > threshold,
            Self::Equal => value == threshold,
            Self::BelowOrEqual => value <= threshold,
            Self::AboveOrEqual => value >= threshold,
        }
    }
}

impl fmt::Display for MetricComparison {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// A threshold that cannot define a meaningful ordered comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("metric threshold must be finite")]
pub struct MetricThresholdError;

/// One metric value that did not meet its numeric threshold.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct MetricThresholdViolation {
    /// The complete metric subject evidence.
    pub subject: MetricSubject,
    /// The stable built-in or user-defined metric name.
    pub metric_name: String,
    /// The measured value.
    pub value: f64,
    /// The required finite threshold.
    pub threshold: f64,
    /// The comparison that was not satisfied.
    pub comparison: MetricComparison,
}

impl MetricThresholdViolation {
    /// Creates structured threshold violation data.
    #[must_use]
    pub fn new(
        subject: MetricSubject,
        metric_name: impl Into<String>,
        value: f64,
        threshold: f64,
        comparison: MetricComparison,
    ) -> Self {
        Self {
            subject,
            metric_name: metric_name.into(),
            value,
            threshold,
            comparison,
        }
    }

    /// Returns the file path or type name identifying the rejected subject.
    #[must_use]
    pub fn identifier(&self) -> &str {
        self.subject.identifier()
    }
}

/// Validates the finite threshold required by every threshold terminal.
pub fn validate_metric_threshold(threshold: f64) -> Result<(), MetricThresholdError> {
    if threshold.is_finite() {
        Ok(())
    } else {
        Err(MetricThresholdError)
    }
}

/// Returns structured violations for measurements that fail an exact comparison.
pub fn gather_metric_threshold_violations(
    measurements: &[MetricMeasurement],
    comparison: MetricComparison,
    threshold: f64,
) -> Result<Vec<MetricThresholdViolation>, MetricThresholdError> {
    validate_metric_threshold(threshold)?;
    Ok(measurements
        .iter()
        .filter(|measurement| !comparison.is_satisfied(measurement.value(), threshold))
        .map(|measurement| {
            MetricThresholdViolation::new(
                measurement.subject().clone(),
                measurement.metric_name(),
                measurement.value(),
                threshold,
                comparison,
            )
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use crate::{MetricMeasurement, MetricSubject, extract_file_metrics};

    use super::{MetricComparison, gather_metric_threshold_violations, validate_metric_threshold};

    fn measurement(name: &str, value: f64) -> MetricMeasurement {
        let type_info = extract_file_metrics("src/types.rs", &format!("struct {name};"))
            .expect("fixture should parse")
            .types()
            .first()
            .expect("fixture should contain one type")
            .clone();
        MetricMeasurement::from_parts(MetricSubject::Type(type_info), "score", "score", value)
    }

    #[test]
    fn every_comparison_uses_exact_boundary_semantics() {
        let measurements = [
            measurement("Below", 1.0),
            measurement("Equal", 2.0),
            measurement("Above", 3.0),
        ];
        let rejected = |comparison| {
            gather_metric_threshold_violations(&measurements, comparison, 2.0)
                .expect("finite threshold should be valid")
                .into_iter()
                .map(|violation| violation.identifier().to_owned())
                .collect::<Vec<_>>()
        };

        assert_eq!(rejected(MetricComparison::Below), ["Equal", "Above"]);
        assert_eq!(rejected(MetricComparison::Above), ["Below", "Equal"]);
        assert_eq!(rejected(MetricComparison::Equal), ["Below", "Above"]);
        assert_eq!(rejected(MetricComparison::BelowOrEqual), ["Above"]);
        assert_eq!(rejected(MetricComparison::AboveOrEqual), ["Below"]);
    }

    #[test]
    fn finite_thresholds_include_negative_and_signed_zero() {
        for threshold in [f64::MIN, -1.0, -0.0, 0.0, f64::MAX] {
            assert!(validate_metric_threshold(threshold).is_ok());
        }
        for threshold in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(validate_metric_threshold(threshold).is_err());
        }
    }

    #[test]
    fn non_finite_custom_values_follow_standard_f64_comparisons() {
        let measurements = [
            measurement("NotANumber", f64::NAN),
            measurement("Infinity", f64::INFINITY),
        ];
        let violations =
            gather_metric_threshold_violations(&measurements, MetricComparison::Above, 0.0)
                .expect("finite threshold should be valid");

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].identifier(), "NotANumber");
        assert!(violations[0].value.is_nan());
    }

    #[test]
    fn names_and_display_cover_exactly_five_comparisons() {
        assert_eq!(
            MetricComparison::ALL.map(MetricComparison::as_str),
            [
                "below",
                "above",
                "equal",
                "below-or-equal",
                "above-or-equal"
            ]
        );
        for comparison in MetricComparison::ALL {
            assert_eq!(comparison.to_string(), comparison.as_str());
        }
    }
}
