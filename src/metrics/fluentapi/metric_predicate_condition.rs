use crate::{
    CheckOptions, CheckResult, Checkable, Filter, MetricMeasurement, MetricSubject, Violation,
    gather_empty_test_violations, gather_metric_predicate_violations,
};

use super::{DistanceMetricSelection, LcomMetricSelection, MetricSelection};

/// Executable arbitrary predicate over one selected built-in metric.
#[derive(Debug, Clone)]
#[must_use = "an architecture rule has no effect until it is checked"]
pub struct MetricPredicateCondition<Selection, Predicate> {
    selection: Selection,
    predicate: Predicate,
}

impl<Selection, Predicate> MetricPredicateCondition<Selection, Predicate> {
    pub(super) const fn new(selection: Selection, predicate: Predicate) -> Self {
        Self {
            selection,
            predicate,
        }
    }

    /// Returns the underlying metric selection.
    #[must_use]
    pub const fn selection(&self) -> &Selection {
        &self.selection
    }
}

macro_rules! impl_predicate_checkable {
    ($selection:ty) => {
        impl<Predicate> Checkable for MetricPredicateCondition<$selection, Predicate>
        where
            Predicate: Fn(f64, &MetricSubject) -> bool,
        {
            fn check_with(&self, options: &CheckOptions) -> CheckResult {
                self.selection.validate_configuration()?;
                finish_predicate_check(
                    self.selection.measure_with(options)?,
                    self.selection.filters(),
                    self.selection.subject_label(),
                    &self.predicate,
                    options,
                )
            }
        }
    };
}

impl_predicate_checkable!(MetricSelection);
impl_predicate_checkable!(LcomMetricSelection);
impl_predicate_checkable!(DistanceMetricSelection);

fn finish_predicate_check<Predicate>(
    measurements: Vec<MetricMeasurement>,
    filters: &[Filter],
    subject_label: &str,
    predicate: &Predicate,
    options: &CheckOptions,
) -> CheckResult
where
    Predicate: Fn(f64, &MetricSubject) -> bool,
{
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

    Ok(gather_metric_predicate_violations(&measurements, predicate)
        .into_iter()
        .map(Violation::from)
        .collect())
}
