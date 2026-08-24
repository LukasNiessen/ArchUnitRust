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
    exclusions: Vec<Filter>,
}

impl Filter {
    /// Creates a positive filter.
    #[must_use]
    pub const fn new(pattern: Pattern, target: PatternTarget) -> Self {
        Self {
            pattern,
            target,
            matching: true,
            exclusions: Vec::new(),
        }
    }

    /// Returns this filter with hard exclusions evaluated after the parent match.
    #[must_use]
    pub fn with_exclusions(mut self, exclusions: impl IntoIterator<Item = Filter>) -> Self {
        self.exclusions.extend(exclusions);
        self
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

    /// Returns hard exclusions in declaration order.
    #[must_use]
    pub fn exclusions(&self) -> &[Filter] {
        &self.exclusions
    }

    /// Returns whether this filter selects `identifier`.
    #[must_use]
    pub fn matches(&self, identifier: &str) -> bool {
        let Some(candidate) = self.target.extract(identifier) else {
            return false;
        };
        self.pattern.matches(&candidate) == self.matching
            && !self
                .exclusions
                .iter()
                .any(|exclusion| exclusion.matches(identifier))
    }
}

impl fmt::Display for Filter {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let verb = if self.matching {
            "matches"
        } else {
            "does not match"
        };
        write!(formatter, "{} {verb} {}", self.target, self.pattern)?;
        if !self.exclusions.is_empty() {
            write!(formatter, " except (")?;
            for (index, exclusion) in self.exclusions.iter().enumerate() {
                if index > 0 {
                    write!(formatter, ", ")?;
                }
                write!(formatter, "{exclusion}")?;
            }
            write!(formatter, ")")?;
        }
        Ok(())
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
        assert!(filter.exclusions().is_empty());
    }

    #[test]
    fn exclusions_are_hard_filters_and_may_use_other_targets() {
        let filter = Filter::new(
            Pattern::glob("src/**").expect("fixture glob should compile"),
            PatternTarget::Path,
        )
        .with_exclusions([
            Filter::new(
                Pattern::glob("src/generated/**").expect("fixture glob should compile"),
                PatternTarget::Path,
            ),
            Filter::new(
                Pattern::glob("*_generated.rs").expect("fixture glob should compile"),
                PatternTarget::Filename,
            ),
        ]);

        assert!(filter.matches("src/domain/service.rs"));
        assert!(!filter.matches("src/generated/model.rs"));
        assert!(!filter.matches("src/domain/model_generated.rs"));
        assert!(
            !filter
                .clone()
                .not_matching()
                .matches("src/generated/model.rs")
        );
        assert!(filter.to_string().contains("except ("));
    }
}
