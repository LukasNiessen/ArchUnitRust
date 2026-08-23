use crate::Filter;

use super::EmptyTestViolation;

/// Produces the defensive violation for a terminal that selected no subjects.
///
/// This assertion deliberately judges the subject selection rather than derived evidence such as
/// dependency edges or cycles. A selected item with no relationships is still a non-empty test.
#[must_use]
pub fn gather_empty_test_violations<T>(
    selected_items: &[T],
    subject: impl Into<String>,
    selectors: &[Filter],
    is_negated: bool,
    allow_empty_tests: bool,
) -> Vec<EmptyTestViolation> {
    if !selected_items.is_empty() || allow_empty_tests {
        return Vec::new();
    }

    vec![EmptyTestViolation::new_with_mood(
        subject,
        selectors.iter().cloned(),
        is_negated,
    )]
}

#[cfg(test)]
mod tests {
    use super::gather_empty_test_violations;
    use crate::RegexFactory;

    #[test]
    fn empty_selection_produces_one_data_violation_with_selector_order_and_mood() {
        let factory = RegexFactory::default();
        let filters = [
            factory
                .folder_matcher("src/apis/**")
                .expect("fixture folder selector should compile"),
            factory
                .filename_matcher("*_handler.rs")
                .expect("fixture name selector should compile"),
        ];

        let violations =
            gather_empty_test_violations(&Vec::<String>::new(), "files", &filters, true, false);

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].subject, "files");
        assert_eq!(violations[0].selectors[0].pattern().source(), "src/apis/**");
        assert_eq!(
            violations[0].selectors[1].pattern().source(),
            "*_handler.rs"
        );
        assert!(violations[0].is_negated);
    }

    #[test]
    fn selected_subject_without_derived_evidence_is_not_empty() {
        let violations = gather_empty_test_violations(&[()], "files", &[], false, false);

        assert!(violations.is_empty());
    }

    #[test]
    fn explicit_option_allows_an_empty_selection() {
        let violations = gather_empty_test_violations(&Vec::<()>::new(), "files", &[], false, true);

        assert!(violations.is_empty());
    }
}
