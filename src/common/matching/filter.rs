use std::fmt;

use super::{Pattern, PatternTarget};

/// A compiled pattern bound to the part of an identifier it describes.
///
/// The target travels with the filter so filename, path, folder, and type-name selectors all use
/// the same [`Self::matches`] operation.
#[derive(Debug, Clone)]
pub struct Filter {
    pattern: Pattern,
    target: PatternTarget,
    matching: bool,
}

impl Filter {
    /// Creates a positive filter.
    #[must_use]
    pub const fn new(pattern: Pattern, target: PatternTarget) -> Self {
        Self {
            pattern,
            target,
            matching: true,
        }
    }

    /// Returns a filter with the pattern's meaning inverted.
    #[must_use]
    pub fn not_matching(mut self) -> Self {
        self.matching = !self.matching;
        self
    }

    /// Returns the compiled pattern carried by this filter.
    #[must_use]
    pub const fn pattern(&self) -> &Pattern {
        &self.pattern
    }

    /// Returns the candidate part carried by this filter.
    #[must_use]
    pub const fn target(&self) -> PatternTarget {
        self.target
    }

    /// Returns whether this filter selects `identifier`.
    #[must_use]
    pub fn matches(&self, identifier: &str) -> bool {
        let Some(candidate) = self.target.extract(identifier) else {
            return false;
        };
        self.pattern.matches(&candidate) == self.matching
    }
}

impl fmt::Display for Filter {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let verb = if self.matching {
            "matches"
        } else {
            "does not match"
        };
        write!(formatter, "{} {verb} {}", self.target, self.pattern)
    }
}

#[cfg(test)]
mod tests {
    use super::Filter;
    use crate::{Pattern, PatternTarget};

    #[test]
    fn binds_patterns_to_each_target() {
        let cases = [
            (
                PatternTarget::Filename,
                "*_test.rs",
                "crates/api/tests/handler_test.rs",
            ),
            (
                PatternTarget::Path,
                "crates/api/**",
                "crates/api/src/handler.rs",
            ),
            (
                PatternTarget::PathWithoutFilename,
                "crates/api/**",
                "crates/api/src/handler.rs",
            ),
            (
                PatternTarget::TypeName,
                "*Handler",
                "crate::api::RequestHandler",
            ),
        ];

        for (target, glob, identifier) in cases {
            let filter = Filter::new(
                Pattern::glob(glob).expect("fixture glob should compile"),
                target,
            );
            assert!(
                filter.matches(identifier),
                "{filter} should select {identifier}"
            );
        }
    }

    #[test]
    fn non_matching_filters_invert_only_valid_candidates() {
        let filter = Filter::new(
            Pattern::glob("*_test.rs").expect("fixture glob should compile"),
            PatternTarget::Filename,
        )
        .not_matching();

        assert!(filter.matches("src/lib.rs"));
        assert!(!filter.matches("src/lib_test.rs"));
        assert!(!filter.matches(""));
        assert_eq!(filter.to_string(), "filename does not match \"*_test.rs\"");
    }

    #[test]
    fn accessors_preserve_filter_data() {
        let filter = Filter::new(
            Pattern::regex(r"[A-Z][A-Za-z]+Handler")
                .expect("fixture regular expression should compile"),
            PatternTarget::TypeName,
        );

        assert_eq!(filter.pattern().source(), r"[A-Z][A-Za-z]+Handler");
        assert_eq!(filter.target(), PatternTarget::TypeName);
    }
}
