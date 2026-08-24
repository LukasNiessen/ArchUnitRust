use crate::metrics::TypeInfo;

/// A Rust type whose custom metric value did not satisfy a user predicate.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct CustomMetricViolation {
    /// The complete immutable Rust type evidence supplied to both callbacks.
    pub type_info: TypeInfo,
    /// The user-defined metric name.
    pub metric_name: String,
    /// The user-defined metric description.
    pub description: String,
    /// The calculated value rejected by the predicate.
    pub value: f64,
}

impl CustomMetricViolation {
    /// Creates structured data for one rejected custom metric value.
    #[must_use]
    pub fn new(
        type_info: TypeInfo,
        metric_name: impl Into<String>,
        description: impl Into<String>,
        value: f64,
    ) -> Self {
        Self {
            type_info,
            metric_name: metric_name.into(),
            description: description.into(),
            value,
        }
    }
}

/// Calculates and checks one custom metric for every supplied type.
///
/// Each callback is invoked exactly once per type in input order. Panics from either user callback
/// propagate normally and are intentionally not converted into architecture violations.
#[must_use]
pub fn gather_custom_metric_violations<Calculation, Predicate>(
    type_infos: &[TypeInfo],
    metric_name: &str,
    description: &str,
    calculation: &Calculation,
    predicate: &Predicate,
) -> Vec<CustomMetricViolation>
where
    Calculation: Fn(&TypeInfo) -> f64,
    Predicate: Fn(f64, &TypeInfo) -> bool,
{
    type_infos
        .iter()
        .filter_map(|type_info| {
            let value = calculation(type_info);
            if predicate(value, type_info) {
                None
            } else {
                Some(CustomMetricViolation::new(
                    type_info.clone(),
                    metric_name,
                    description,
                    value,
                ))
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use crate::metrics::extract_file_metrics;

    use super::gather_custom_metric_violations;

    fn types() -> Vec<crate::metrics::TypeInfo> {
        extract_file_metrics(
            "src/types.rs",
            "struct Small { one: usize } struct Large { one: usize, two: usize }",
        )
        .expect("fixture should parse")
        .types()
        .to_vec()
    }

    #[test]
    fn invokes_both_callbacks_once_and_retains_rejected_type_evidence() {
        let calculations = Cell::new(0);
        let predicates = Cell::new(0);
        let violations = gather_custom_metric_violations(
            &types(),
            "field_score",
            "declared field count",
            &|info| {
                calculations.set(calculations.get() + 1);
                info.fields().len() as f64
            },
            &|value, info| {
                predicates.set(predicates.get() + 1);
                value < 2.0 && info.name() == "Small"
            },
        );

        assert_eq!(calculations.get(), 2);
        assert_eq!(predicates.get(), 2);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].type_info.name(), "Large");
        assert_eq!(violations[0].metric_name, "field_score");
        assert_eq!(violations[0].description, "declared field count");
        assert_eq!(violations[0].value, 2.0);
    }

    #[test]
    fn preserves_non_finite_custom_values_for_the_user_predicate() {
        let violations = gather_custom_metric_violations(
            &types()[..1],
            "score",
            "arbitrary score",
            &|_| f64::NAN,
            &|value, _| value.is_finite(),
        );

        assert_eq!(violations.len(), 1);
        assert!(violations[0].value.is_nan());
    }
}
