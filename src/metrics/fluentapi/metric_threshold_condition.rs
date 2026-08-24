use crate::checkable::execute_logged_check;
use crate::{
    ArchUnitError, CheckOptions, CheckResult, Checkable, Filter, MetricComparison,
    MetricMeasurement, UserError, Violation, gather_empty_test_violations,
    gather_metric_threshold_violations, validate_metric_threshold,
};

use super::{
    CustomMetricSelection, DistanceMetricSelection, LcomMetricSelection, MetricSelection,
    logging::log_measurements,
};

/// Executable exact numeric threshold over one selected metric.
#[derive(Debug, Clone)]
#[must_use = "an architecture rule has no effect until it is checked"]
pub struct MetricThresholdCondition<Selection> {
    selection: Selection,
    comparison: MetricComparison,
    threshold: f64,
}

impl<Selection> MetricThresholdCondition<Selection> {
    pub(super) const fn new(
        selection: Selection,
        comparison: MetricComparison,
        threshold: f64,
    ) -> Self {
        Self {
            selection,
            comparison,
            threshold,
        }
    }

    /// Returns the exact numeric comparison.
    #[must_use]
    pub const fn comparison(&self) -> MetricComparison {
        self.comparison
    }

    /// Returns the configured threshold.
    #[must_use]
    pub const fn threshold(&self) -> f64 {
        self.threshold
    }

    /// Returns the underlying metric selection.
    #[must_use]
    pub const fn selection(&self) -> &Selection {
        &self.selection
    }
}

macro_rules! impl_threshold_checkable {
    ($selection:ty) => {
        impl Checkable for MetricThresholdCondition<$selection> {
            fn check_with(&self, options: &CheckOptions) -> CheckResult {
                execute_logged_check("metrics.threshold", options, |logger| {
                    self.selection.validate_configuration()?;
                    validate_metric_threshold(self.threshold).map_err(threshold_error)?;
                    logger.log_progress("calculating metric values")?;
                    let measurements = self.selection.measure_with(options)?;
                    logger.log_progress(format!("measurements={}", measurements.len()))?;
                    log_measurements(logger, &measurements, Some(self.threshold))?;
                    finish_threshold_check(
                        measurements,
                        self.selection.filters(),
                        self.selection.subject_label(),
                        self.comparison,
                        self.threshold,
                        options,
                    )
                })
            }
        }
    };
}

impl_threshold_checkable!(MetricSelection);
impl_threshold_checkable!(LcomMetricSelection);
impl_threshold_checkable!(DistanceMetricSelection);

impl<Calculation> Checkable for MetricThresholdCondition<CustomMetricSelection<Calculation>>
where
    Calculation: Fn(&crate::TypeInfo) -> f64,
{
    fn check_with(&self, options: &CheckOptions) -> CheckResult {
        execute_logged_check("metrics.threshold", options, |logger| {
            self.selection.validate_configuration()?;
            validate_metric_threshold(self.threshold).map_err(threshold_error)?;
            logger.log_progress("calculating custom metric values")?;
            let measurements = self.selection.measure_with(options)?;
            logger.log_progress(format!("measurements={}", measurements.len()))?;
            log_measurements(logger, &measurements, Some(self.threshold))?;
            finish_threshold_check(
                measurements,
                self.selection.filters(),
                self.selection.subject_label(),
                self.comparison,
                self.threshold,
                options,
            )
        })
    }
}

fn finish_threshold_check(
    measurements: Vec<MetricMeasurement>,
    filters: &[Filter],
    subject_label: &str,
    comparison: MetricComparison,
    threshold: f64,
    options: &CheckOptions,
) -> CheckResult {
    let empty = gather_empty_test_violations(
        &measurements,
        subject_label,
        filters,
        false,
        options.allows_empty_tests(),
    );
    if let Some(violation) = empty.into_iter().next() {
        return Ok(vec![Violation::from(violation)]);
    }

    Ok(
        gather_metric_threshold_violations(&measurements, comparison, threshold)
            .map_err(threshold_error)?
            .into_iter()
            .map(Violation::from)
            .collect(),
    )
}

fn threshold_error(error: crate::MetricThresholdError) -> ArchUnitError {
    ArchUnitError::from(UserError::with_source(
        "the metric threshold is invalid",
        error,
    ))
}
