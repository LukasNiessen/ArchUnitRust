//! Closed aggregation of violation data owned by the individual rule domains.

use std::fmt;

use crate::common::assertion::EmptyTestViolation;
use crate::files::assertion::{CycleViolation, FilePatternViolation};

/// The machine-readable family of a [`Violation`].
///
/// Spellings are shared across ArchUnit ports and are stable report keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ViolationKind {
    /// A selector matched no subject, so the rule judged nothing.
    EmptyTest,
    /// The selected projected graph contains a circular dependency path.
    Cycle,
    /// A selected file disagrees with a filename, folder, or path pattern.
    FilePattern,
}

impl ViolationKind {
    /// Returns the stable lowercase, hyphen-separated report key.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EmptyTest => "empty-test",
            Self::Cycle => "cycle",
            Self::FilePattern => "file-pattern",
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
    /// A circular path exists in the selected projected graph.
    Cycle(CycleViolation),
    /// A file disagrees with a name or location predicate.
    FilePattern(FilePatternViolation),
}

impl Violation {
    /// Returns this violation's stable family.
    #[must_use]
    pub const fn kind(&self) -> ViolationKind {
        match self {
            Self::EmptyTest(_) => ViolationKind::EmptyTest,
            Self::Cycle(_) => ViolationKind::Cycle,
            Self::FilePattern(_) => ViolationKind::FilePattern,
        }
    }

    /// Returns the empty-test data when this is an empty-test violation.
    #[must_use]
    pub const fn as_empty_test(&self) -> Option<&EmptyTestViolation> {
        match self {
            Self::EmptyTest(violation) => Some(violation),
            Self::Cycle(_) | Self::FilePattern(_) => None,
        }
    }

    /// Returns the cycle data when this is a cycle violation.
    #[must_use]
    pub const fn as_cycle(&self) -> Option<&CycleViolation> {
        match self {
            Self::Cycle(violation) => Some(violation),
            Self::EmptyTest(_) | Self::FilePattern(_) => None,
        }
    }

    /// Returns the file-pattern data when this is a file-pattern violation.
    #[must_use]
    pub const fn as_file_pattern(&self) -> Option<&FilePatternViolation> {
        match self {
            Self::FilePattern(violation) => Some(violation),
            Self::EmptyTest(_) | Self::Cycle(_) => None,
        }
    }
}

impl From<EmptyTestViolation> for Violation {
    fn from(violation: EmptyTestViolation) -> Self {
        Self::EmptyTest(violation)
    }
}

impl From<CycleViolation> for Violation {
    fn from(violation: CycleViolation) -> Self {
        Self::Cycle(violation)
    }
}

impl From<FilePatternViolation> for Violation {
    fn from(violation: FilePatternViolation) -> Self {
        Self::FilePattern(violation)
    }
}

#[cfg(test)]
mod tests {
    use super::{Violation, ViolationKind};
    use crate::{
        CycleViolation, Edge, EmptyTestViolation, FilePatternViolation, Graph, ProjectedEdge,
        RegexFactory, project_to_nodes,
    };

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

    #[test]
    fn cycle_has_a_stable_kind_and_typed_accessor() {
        let violation = Violation::from(CycleViolation::new(Vec::<ProjectedEdge>::new()));

        assert_eq!(violation.kind(), ViolationKind::Cycle);
        assert_eq!(violation.kind().as_str(), "cycle");
        assert!(violation.as_cycle().is_some());
        assert!(violation.as_empty_test().is_none());
        assert!(violation.as_file_pattern().is_none());
    }

    #[test]
    fn file_pattern_has_a_stable_kind_and_typed_accessor() {
        let filter = RegexFactory::default()
            .filename_matcher("*.rs")
            .expect("fixture pattern should compile");
        let node = project_to_nodes(&Graph::from_edges([Edge::self_edge("src/lib.rs")]))
            .into_iter()
            .next()
            .expect("fixture graph should project one node");
        let violation = Violation::from(FilePatternViolation::new(filter, node, false));

        assert_eq!(violation.kind(), ViolationKind::FilePattern);
        assert_eq!(violation.kind().as_str(), "file-pattern");
        assert!(violation.as_file_pattern().is_some());
        assert!(violation.as_cycle().is_none());
        assert!(violation.as_empty_test().is_none());
    }
}
