/// A framework-neutral pass flag and its complete display message.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
#[must_use = "a shaped test result must be inspected or passed to a test integration"]
pub struct TestResult {
    /// Whether the observed architecture verdict matched the expectation.
    pub passed: bool,
    /// The complete success or failure message.
    pub message: String,
}

impl TestResult {
    /// Creates one framework-neutral test result.
    pub fn new(passed: bool, message: impl Into<String>) -> Self {
        Self {
            passed,
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TestResult;

    #[test]
    fn owns_the_pass_flag_and_complete_message() {
        let result = TestResult::new(false, "Found one violation");

        assert!(!result.passed);
        assert_eq!(result.message, "Found one violation");
    }
}
