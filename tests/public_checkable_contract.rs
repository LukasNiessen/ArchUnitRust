use archunit::{
    ArchUnitError, CheckOptions, CheckResult, Checkable, EmptyTestViolation, LoggingOptions,
    TechnicalError, Violation,
};

struct FixtureRule;

impl Checkable for FixtureRule {
    fn check_with(&self, options: &CheckOptions) -> CheckResult {
        if options.clears_cache() {
            return Err(ArchUnitError::from(TechnicalError::new(
                "fixture cache clear failed",
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

fn run_rule(rule: &dyn Checkable) -> CheckResult {
    rule.check()
}

#[test]
fn consumers_program_only_against_the_checkable_trait() {
    let result = run_rule(&FixtureRule).expect("fixture rule should reach a verdict");

    assert_eq!(result.len(), 1);
}

#[test]
fn check_with_threads_one_immutable_options_bag_to_the_terminal() {
    let options = CheckOptions::new()
        .with_allow_empty_tests(true)
        .with_logging(LoggingOptions::new())
        .with_test_sources(true);

    let result = FixtureRule
        .check_with(&options)
        .expect("allowing the empty selection should pass");

    assert!(result.is_empty());
    assert!(options.logging().is_some());
    assert!(options.includes_test_sources());
}

#[test]
fn technical_failures_are_distinct_from_rule_violations() {
    let options = CheckOptions::new().with_clear_cache(true);
    let result = FixtureRule.check_with(&options);

    assert!(matches!(result, Err(ArchUnitError::Technical(_))));
}
