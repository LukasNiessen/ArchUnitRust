use crate::{Filter, PatternError, ProjectLocator, RegexFactory};

use super::{FileConditionBuilder, MatchPatternFileCondition};

/// Shared immutable state for positive and negated file-predicate builders.
///
/// The mood is one boolean consumed later by a shared assertion path. Public positive and negated
/// wrappers make the fluent stages distinct without duplicating rule evaluation.
#[derive(Debug, Clone)]
#[must_use = "a file mood has no effect until a predicate is selected and checked"]
pub struct MatchPatternFileConditionBuilder {
    scope: FileConditionBuilder,
    negated: bool,
}

impl MatchPatternFileConditionBuilder {
    pub(super) const fn new(scope: FileConditionBuilder, negated: bool) -> Self {
        Self { scope, negated }
    }

    /// Returns the selected file scope carried into this mood.
    pub const fn scope(&self) -> &FileConditionBuilder {
        &self.scope
    }

    /// Returns whether the following predicate is negated.
    #[must_use]
    pub const fn is_negated(&self) -> bool {
        self.negated
    }

    /// Returns where Cargo project discovery will begin.
    #[must_use]
    pub const fn project_locator(&self) -> &ProjectLocator {
        self.scope.project_locator()
    }

    /// Returns the scope selectors in chain order.
    #[must_use]
    pub fn filters(&self) -> &[Filter] {
        self.scope.filters()
    }

    /// Returns the first invalid selector retained by the scope.
    #[must_use]
    pub const fn selector_error(&self) -> Option<&PatternError> {
        self.scope.selector_error()
    }

    /// Requires every selected file's final path segment to match `pattern`.
    pub fn have_name(self, pattern: impl AsRef<str>) -> MatchPatternFileCondition {
        let check_filter = RegexFactory::default().filename_matcher(pattern);
        self.matching(check_filter)
    }

    /// Requires every selected file's containing folder to match `pattern`.
    pub fn be_in_folder(self, pattern: impl AsRef<str>) -> MatchPatternFileCondition {
        let check_filter = RegexFactory::default().folder_matcher(pattern);
        self.matching(check_filter)
    }

    /// Requires every selected file's complete normalized path to match `pattern`.
    pub fn be_in_path(self, pattern: impl AsRef<str>) -> MatchPatternFileCondition {
        let check_filter = RegexFactory::default().path_matcher(pattern);
        self.matching(check_filter)
    }

    fn matching(self, check_filter: Result<Filter, PatternError>) -> MatchPatternFileCondition {
        MatchPatternFileCondition::new(self, check_filter)
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::project_files_in;

    use super::MatchPatternFileConditionBuilder;

    #[test]
    fn carries_one_owned_scope_and_one_mood_flag() {
        let scope = project_files_in("examples/layered")
            .in_folder("src/**")
            .with_name("*.rs");
        let mood = MatchPatternFileConditionBuilder::new(scope, true);

        assert!(mood.is_negated());
        assert_eq!(
            mood.project_locator().path(),
            Some(Path::new("examples/layered"))
        );
        assert_eq!(mood.filters().len(), 2);
        assert!(mood.selector_error().is_none());
    }

    #[test]
    fn preserves_invalid_selector_diagnostics() {
        let scope = project_files_in("examples/layered").in_path("src/[api");
        let mood = MatchPatternFileConditionBuilder::new(scope, false);

        assert!(!mood.is_negated());
        assert_eq!(
            mood.selector_error().map(|error| error.pattern()),
            Some("src/[api")
        );
    }

    #[test]
    fn creates_all_three_predicates_with_the_shared_mood() {
        let scope = project_files_in("examples/layered").in_path("src/**");
        let named =
            MatchPatternFileConditionBuilder::new(scope.clone(), false).have_name("*_service.rs");
        let folder =
            MatchPatternFileConditionBuilder::new(scope.clone(), true).be_in_folder("src/service");
        let path = MatchPatternFileConditionBuilder::new(scope, false).be_in_path("src/**");

        assert!(!named.is_negated());
        assert!(folder.is_negated());
        assert!(!path.is_negated());
        assert_eq!(
            named.check_filter().map(|filter| filter.target()),
            Some(crate::PatternTarget::Filename)
        );
        assert_eq!(
            folder.check_filter().map(|filter| filter.target()),
            Some(crate::PatternTarget::PathWithoutFilename)
        );
        assert_eq!(
            path.check_filter().map(|filter| filter.target()),
            Some(crate::PatternTarget::Path)
        );
    }
}
