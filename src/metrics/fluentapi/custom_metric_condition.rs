use crate::checkable::execute_logged_check;
use crate::{
    checkable::{CheckResult, Checkable},
    common::{CheckOptions, gather_empty_test_violations},
    metrics::{CustomMetricViolation, TypeInfo},
    violation::Violation,
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
        execute_logged_check("metrics.custom-predicate", options, |logger| {
            logger.log_progress("selecting custom metric types")?;
            let types = self.selection.selected_types_with(options)?;
            logger.log_progress(format!("metric types={}", types.len()))?;
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

            let mut violations = Vec::new();
            for type_info in &types {
                let value = (self.selection.calculation())(type_info);
                logger.log_metric(self.selection.name(), type_info.name(), value, None)?;
                if !(self.predicate)(value, type_info) {
                    violations.push(Violation::from(CustomMetricViolation::new(
                        type_info.clone(),
                        self.selection.name(),
                        self.selection.description(),
                        value,
                    )));
                }
            }
            Ok(violations)
        })
    }
}
