/// One human-readable rendering of structured architecture-violation data.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct TestViolation {
    /// The short violation category or custom requirement.
    pub message: String,
    /// The evidence and relationship that explain the failure.
    pub details: String,
}

impl TestViolation {
    /// Creates one framework-neutral violation rendering.
    #[must_use]
    pub fn new(message: impl Into<String>, details: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            details: details.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TestViolation;

    #[test]
    fn owns_message_and_details_as_plain_data() {
        let violation = TestViolation::new("File dependency violation", "api -> database");

        assert_eq!(violation.message, "File dependency violation");
        assert_eq!(violation.details, "api -> database");
    }
}
