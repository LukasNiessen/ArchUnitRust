use std::fmt;

use super::EmptyTestViolation;

/// The machine-readable family of a [`Violation`].
///
/// Spellings are shared across ArchUnit ports and are stable report keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ViolationKind {
    /// A selector matched no subject, so the rule judged nothing.
    EmptyTest,
}

impl ViolationKind {
    /// Returns the stable lowercase, hyphen-separated report key.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EmptyTest => "empty-test",
        }
    }
}

impl fmt::Display for ViolationKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// One data-carrying disagreement between a project and an architecture rule.
///
/// A complete rule result is `Vec<Violation>`: an empty vector means the rule passed. This enum
/// deliberately contains no user-facing prose; the testing layer formats each variant later.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Violation {
    /// The rule selected no subject and therefore could not judge its predicate.
    EmptyTest(EmptyTestViolation),
}

impl Violation {
    /// Returns this violation's stable family.
    #[must_use]
    pub const fn kind(&self) -> ViolationKind {
        match self {
            Self::EmptyTest(_) => ViolationKind::EmptyTest,
        }
    }

    /// Returns the empty-test data when this is an empty-test violation.
    #[must_use]
    pub const fn as_empty_test(&self) -> Option<&EmptyTestViolation> {
        match self {
            Self::EmptyTest(violation) => Some(violation),
        }
    }
}

impl From<EmptyTestViolation> for Violation {
    fn from(violation: EmptyTestViolation) -> Self {
        Self::EmptyTest(violation)
    }
}

#[cfg(test)]
mod tests {
    use super::{Violation, ViolationKind};
    use crate::EmptyTestViolation;

    #[test]
    fn empty_test_has_a_stable_kind() {
        let violation = Violation::from(EmptyTestViolation::new("files", []));

        assert_eq!(violation.kind(), ViolationKind::EmptyTest);
        assert_eq!(violation.kind().as_str(), "empty-test");
        assert_eq!(violation.kind().to_string(), "empty-test");
    }

    #[test]
    fn exposes_typed_data_without_formatting_it() {
        let violation = Violation::from(EmptyTestViolation::new("slices", []));

        let empty = violation
            .as_empty_test()
            .expect("fixture should be an empty-test violation");
        assert_eq!(empty.subject, "slices");
        assert!(empty.selectors.is_empty());
    }
}
