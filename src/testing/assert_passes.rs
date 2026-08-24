use crate::{
    checkable::Checkable,
    common::CheckOptions,
    testing::{ResultFactory, TestResult, TestResultOptions},
};

pub fn evaluate_assertion<R>(rule: &R, check_options: &CheckOptions) -> TestResult
where
    R: Checkable + ?Sized,
{
    evaluate_assertion_with_options(rule, check_options, &TestResultOptions::default())
}

fn evaluate_assertion_with_options<R>(
    rule: &R,
    check_options: &CheckOptions,
    result_options: &TestResultOptions,
) -> TestResult
where
    R: Checkable + ?Sized,
{
    match rule.check_with(check_options) {
        Ok(violations) => ResultFactory::from_violations_with_options(&violations, result_options),
        Err(error) => ResultFactory::from_error_with_options(&error, result_options),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        checkable::{CheckResult, Checkable},
        common::{ArchUnitError, CheckOptions, EmptyTestViolation, TechnicalError},
        testing::{ColorChoice, TestResultOptions},
        violation::Violation,
    };

    use super::evaluate_assertion_with_options;

    struct FixtureRule;

    impl Checkable for FixtureRule {
        fn check_with(&self, options: &CheckOptions) -> CheckResult {
            if options.clears_cache() {
                return Err(ArchUnitError::from(TechnicalError::new(
                    "fixture extraction failed",
                )));
            }
            if options.allows_empty_tests() {
                return Ok(Vec::new());
            }
            Ok(vec![Violation::from(EmptyTestViolation::new(
                "fixture files",
                [],
            ))])
        }
    }

    fn plain_results() -> TestResultOptions {
        TestResultOptions::new().with_color(ColorChoice::Never)
    }

    #[test]
    fn evaluates_violation_and_success_results_through_the_shared_factories() {
        let failure = evaluate_assertion_with_options(
            &FixtureRule,
            &CheckOptions::default(),
            &plain_results(),
        );
        let success = evaluate_assertion_with_options(
            &FixtureRule,
            &CheckOptions::new().with_allow_empty_tests(true),
            &plain_results(),
        );

        assert!(!failure.passed);
        assert!(
            failure
                .message
                .starts_with("Found 1 architecture violation:")
        );
        assert!(success.passed);
        assert_eq!(success.message, "No architecture violations found.");
    }

    #[test]
    fn evaluates_check_errors_as_assertion_failures_without_losing_context() {
        let result = evaluate_assertion_with_options(
            &FixtureRule,
            &CheckOptions::new().with_clear_cache(true),
            &plain_results(),
        );

        assert!(!result.passed);
        assert_eq!(
            result.message,
            "Architecture check could not run: archunit: fixture extraction failed"
        );
    }
}
