use crate::FileInfo;

/// One selected file that disagrees with a user-defined predicate.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct CustomFileViolation {
    /// The immutable file facts supplied to the predicate.
    pub file_info: FileInfo,
    /// The user's description of the predicate requirement.
    pub message: String,
    /// Whether satisfying the predicate was forbidden rather than required.
    pub is_negated: bool,
}

impl CustomFileViolation {
    /// Creates data for one file that failed a custom rule.
    #[must_use]
    pub fn new(file_info: FileInfo, message: impl Into<String>, is_negated: bool) -> Self {
        Self {
            file_info,
            message: message.into(),
            is_negated,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CustomFileViolation;
    use crate::FileInfo;

    #[test]
    fn retains_file_requirement_and_mood_as_data() {
        let info = FileInfo::new("src/api.rs", "pub fn api() {}\n");

        let violation = CustomFileViolation::new(info, "contain no public functions", true);

        assert_eq!(violation.file_info.path, "src/api.rs");
        assert_eq!(violation.message, "contain no public functions");
        assert!(violation.is_negated);
    }
}
