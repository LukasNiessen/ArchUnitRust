use crate::common::Filter;

/// A rule selected no subject and therefore judged nothing.
///
/// Zero matches is a violation by default because a stale or misspelled selector would otherwise
/// pass forever.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct EmptyTestViolation {
    /// The rule family's subject vocabulary, such as `files`, `layers`, or `slices`.
    pub subject: String,
    /// The selectors that, taken together, matched nothing, in fluent-chain order.
    pub selectors: Vec<Filter>,
    /// Whether the empty selection occurred in a negated rule.
    pub is_negated: bool,
}

impl EmptyTestViolation {
    /// Records the subject and selectors that matched nothing.
    #[must_use]
    pub fn new(subject: impl Into<String>, selectors: impl IntoIterator<Item = Filter>) -> Self {
        Self::new_with_mood(subject, selectors, false)
    }

    /// Records the subject, selectors and fluent mood that matched nothing.
    #[must_use]
    pub fn new_with_mood(
        subject: impl Into<String>,
        selectors: impl IntoIterator<Item = Filter>,
        is_negated: bool,
    ) -> Self {
        Self {
            subject: subject.into(),
            selectors: selectors.into_iter().collect(),
            is_negated,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::EmptyTestViolation;
    use crate::common::RegexFactory;

    #[test]
    fn carries_selector_data_in_chain_order() {
        let factory = RegexFactory::default();
        let folder = factory
            .folder_matcher("src/apis/**")
            .expect("fixture glob should compile");
        let filename = factory
            .filename_matcher("*_handler.rs")
            .expect("fixture glob should compile");

        let violation = EmptyTestViolation::new("files", [folder.clone(), filename.clone()]);

        assert_eq!(violation.subject, "files");
        assert_eq!(violation.selectors.len(), 2);
        assert!(!violation.is_negated);
        assert_eq!(
            violation.selectors[0].pattern().source(),
            folder.pattern().source()
        );
        assert_eq!(
            violation.selectors[1].pattern().source(),
            filename.pattern().source()
        );
    }

    #[test]
    fn owns_its_selector_collection() {
        let selector = RegexFactory::default()
            .path_matcher("src/**")
            .expect("fixture glob should compile");
        let mut selectors = vec![selector];

        let violation = EmptyTestViolation::new("files", selectors.clone());
        selectors.clear();

        assert_eq!(violation.selectors.len(), 1);
    }

    #[test]
    fn supports_a_subject_without_explicit_selectors() {
        let violation = EmptyTestViolation::new("project files", []);

        assert_eq!(violation.subject, "project files");
        assert!(violation.selectors.is_empty());
    }

    #[test]
    fn explicit_constructor_retains_the_negated_mood() {
        let violation = EmptyTestViolation::new_with_mood("files", [], true);

        assert!(violation.is_negated);
    }
}
