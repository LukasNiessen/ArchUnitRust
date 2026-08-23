use super::ColorChoice;

/// Presentation and expectation options for [`crate::ResultFactory`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct TestResultOptions {
    color: ColorChoice,
    expected_to_pass: bool,
}

impl TestResultOptions {
    /// Creates auto-colored output that expects the architecture rule to pass.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            color: ColorChoice::Auto,
            expected_to_pass: true,
        }
    }

    /// Returns the configured ANSI color policy.
    #[must_use]
    pub const fn color(self) -> ColorChoice {
        self.color
    }

    /// Returns whether an empty violation list is expected.
    #[must_use]
    pub const fn expects_to_pass(self) -> bool {
        self.expected_to_pass
    }

    /// Selects the ANSI color policy.
    #[must_use]
    pub const fn with_color(mut self, color: ColorChoice) -> Self {
        self.color = color;
        self
    }

    /// Controls whether the architecture rule is expected to have no violations.
    #[must_use]
    pub const fn with_expected_to_pass(mut self, expected: bool) -> Self {
        self.expected_to_pass = expected;
        self
    }
}

impl Default for TestResultOptions {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::TestResultOptions;
    use crate::ColorChoice;

    #[test]
    fn defaults_to_auto_color_and_a_passing_expectation() {
        let options = TestResultOptions::default();

        assert_eq!(options.color(), ColorChoice::Auto);
        assert!(options.expects_to_pass());
    }

    #[test]
    fn consuming_builders_are_branchable_values() {
        let base = TestResultOptions::new().with_color(ColorChoice::Never);
        let inverted = base.with_expected_to_pass(false);

        assert!(base.expects_to_pass());
        assert!(!inverted.expects_to_pass());
        assert_eq!(inverted.color(), ColorChoice::Never);
    }
}
