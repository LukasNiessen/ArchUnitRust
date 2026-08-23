use crate::{
    ColorUtils, TestResult, TestResultOptions, TestViolation, Violation, ViolationFactory,
};

/// Shapes structured violations into a framework-neutral pass flag and complete message.
#[derive(Debug, Clone, Copy, Default)]
pub struct ResultFactory;

impl ResultFactory {
    /// Shapes violations using auto color detection and the expectation that the rule passes.
    pub fn from_violations(violations: &[Violation]) -> TestResult {
        Self::from_violations_with_options(violations, &TestResultOptions::default())
    }

    /// Shapes violations with explicit color and pass-expectation options.
    pub fn from_violations_with_options(
        violations: &[Violation],
        options: &TestResultOptions,
    ) -> TestResult {
        let observed_pass = violations.is_empty();
        let expected_pass = options.expects_to_pass();
        let passed = observed_pass == expected_pass;

        match (observed_pass, expected_pass) {
            (true, true) => TestResult::new(
                true,
                ColorUtils::green("No architecture violations found.", options.color()),
            ),
            (true, false) => TestResult::new(
                false,
                ColorUtils::red_bold(
                    "Expected architecture violations, but none were found.",
                    options.color(),
                ),
            ),
            (false, _) => {
                let formatted = violations
                    .iter()
                    .map(ViolationFactory::from_violation)
                    .collect::<Vec<_>>();
                TestResult::new(
                    passed,
                    format_violations(&formatted, expected_pass, options),
                )
            }
        }
    }
}

fn format_violations(
    violations: &[TestViolation],
    expected_pass: bool,
    options: &TestResultOptions,
) -> String {
    let count = violations.len();
    let noun = if count == 1 {
        "violation"
    } else {
        "violations"
    };
    let title = if expected_pass {
        ColorUtils::red_bold(
            format!("Found {count} architecture {noun}:"),
            options.color(),
        )
    } else {
        ColorUtils::green_bold(
            format!("Found {count} architecture {noun}, as expected:"),
            options.color(),
        )
    };
    let mut lines = vec![title, String::new()];

    for (index, violation) in violations.iter().enumerate() {
        let heading = format!("  {}. {}", index + 1, violation.message);
        lines.push(ColorUtils::yellow(heading, options.color()));
        lines.extend(violation.details.lines().map(|line| format!("     {line}")));
        if index + 1 < count {
            lines.push(String::new());
        }
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use crate::{ColorChoice, EmptyTestViolation, ResultFactory, TestResultOptions, Violation};

    fn empty_violation(subject: &str) -> Violation {
        Violation::from(EmptyTestViolation::new(subject, []))
    }

    fn plain_options() -> TestResultOptions {
        TestResultOptions::new().with_color(ColorChoice::Never)
    }

    #[test]
    fn empty_list_is_a_plain_success_by_default() {
        let result = ResultFactory::from_violations_with_options(&[], &plain_options());

        assert!(result.passed);
        assert_eq!(result.message, "No architecture violations found.");
    }

    #[test]
    fn one_violation_is_a_numbered_singular_failure() {
        let result = ResultFactory::from_violations_with_options(
            &[empty_violation("files")],
            &plain_options(),
        );

        assert!(!result.passed);
        assert_eq!(
            result.message,
            "Found 1 architecture violation:\n\n  1. Empty test violation\n     The positive files rule selected no subjects without explicit selectors. Verify the selectors or explicitly use CheckOptions::new().with_allow_empty_tests(true) for an intentional empty scope."
        );
    }

    #[test]
    fn multiple_violations_are_plural_numbered_and_separated() {
        let result = ResultFactory::from_violations_with_options(
            &[empty_violation("files"), empty_violation("slices")],
            &plain_options(),
        );

        assert!(!result.passed);
        assert!(
            result
                .message
                .starts_with("Found 2 architecture violations:")
        );
        assert!(result.message.contains("\n\n  1. Empty test violation\n"));
        assert!(result.message.contains("\n\n  2. Empty test violation\n"));
        assert!(!result.message.ends_with('\n'));
    }

    #[test]
    fn inverted_expectation_passes_on_violations_and_fails_on_none() {
        let options = plain_options().with_expected_to_pass(false);
        let expected_failure =
            ResultFactory::from_violations_with_options(&[empty_violation("files")], &options);
        let unexpected_pass = ResultFactory::from_violations_with_options(&[], &options);

        assert!(expected_failure.passed);
        assert!(
            expected_failure
                .message
                .starts_with("Found 1 architecture violation, as expected:")
        );
        assert!(!unexpected_pass.passed);
        assert_eq!(
            unexpected_pass.message,
            "Expected architecture violations, but none were found."
        );
    }

    #[test]
    fn explicit_color_is_deterministic_for_success_and_failure() {
        let options = TestResultOptions::new().with_color(ColorChoice::Always);
        let success = ResultFactory::from_violations_with_options(&[], &options);
        let failure =
            ResultFactory::from_violations_with_options(&[empty_violation("files")], &options);

        assert_eq!(
            success.message,
            "\x1b[32mNo architecture violations found.\x1b[0m"
        );
        assert!(
            failure
                .message
                .starts_with("\x1b[1;31mFound 1 architecture violation:\x1b[0m")
        );
        assert!(
            failure
                .message
                .contains("\x1b[33m  1. Empty test violation\x1b[0m")
        );
    }
}
