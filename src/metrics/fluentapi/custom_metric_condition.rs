use crate::{
    CheckOptions, CheckResult, Checkable, TypeInfo, Violation, gather_custom_metric_violations,
    gather_empty_test_violations,
};

use super::CustomMetricSelection;

/// Executable user-defined metric predicate over selected Rust types.
#[derive(Debug, Clone)]
#[must_use = "an architecture rule has no effect until it is checked"]
pub struct CustomMetricCondition<Calculation, Predicate> {
    selection: CustomMetricSelection<Calculation>,
    predicate: Predicate,
}

impl<Calculation, Predicate> CustomMetricCondition<Calculation, Predicate>
where
    Calculation: Fn(&TypeInfo) -> f64,
{
    pub(super) const fn new(
        selection: CustomMetricSelection<Calculation>,
        predicate: Predicate,
    ) -> Self {
        Self {
            selection,
            predicate,
        }
    }

    /// Returns the custom metric name.
    #[must_use]
    pub fn metric_name(&self) -> &str {
        self.selection.name()
    }

    /// Returns the custom metric description.
    #[must_use]
    pub fn description(&self) -> &str {
        self.selection.description()
    }
}

impl<Calculation, Predicate> Checkable for CustomMetricCondition<Calculation, Predicate>
where
    Calculation: Fn(&TypeInfo) -> f64,
    Predicate: Fn(f64, &TypeInfo) -> bool,
{
    fn check_with(&self, options: &CheckOptions) -> CheckResult {
        let types = self.selection.selected_types_with(options)?;
        let empty = gather_empty_test_violations(
            &types,
            "metric types",
            self.selection.filters(),
            false,
            options.allows_empty_tests(),
        );
        if let Some(violation) = empty.into_iter().next() {
            return Ok(vec![Violation::from(violation)]);
        }

        Ok(gather_custom_metric_violations(
            &types,
            self.selection.name(),
            self.selection.description(),
            self.selection.calculation(),
            &self.predicate,
        )
        .into_iter()
        .map(Violation::from)
        .collect())
    }
}
