use crate::{Filter, PatternError, ProjectLocator};

use super::{FileConditionBuilder, MatchPatternFileCondition, MatchPatternFileConditionBuilder};

/// The `should_not` mood for file predicates.
#[derive(Debug, Clone)]
#[must_use = "a file mood has no effect until a predicate is selected and checked"]
pub struct NegatedMatchPatternFileConditionBuilder {
    condition: MatchPatternFileConditionBuilder,
}

impl NegatedMatchPatternFileConditionBuilder {
    pub(super) const fn new(scope: FileConditionBuilder) -> Self {
        Self {
            condition: MatchPatternFileConditionBuilder::new(scope, true),
        }
    }

    /// Returns the shared mood state.
    pub const fn condition(&self) -> &MatchPatternFileConditionBuilder {
        &self.condition
    }

    /// Returns the selected file scope.
    pub const fn scope(&self) -> &FileConditionBuilder {
        self.condition.scope()
    }

    /// Returns `true` for the negated mood.
    #[must_use]
    pub const fn is_negated(&self) -> bool {
        self.condition.is_negated()
    }

    /// Returns where Cargo project discovery will begin.
    #[must_use]
    pub const fn project_locator(&self) -> &ProjectLocator {
        self.condition.project_locator()
    }

    /// Returns the scope selectors in chain order.
    #[must_use]
    pub fn filters(&self) -> &[Filter] {
        self.condition.filters()
    }

    /// Returns the first invalid selector retained by the scope.
    #[must_use]
    pub const fn selector_error(&self) -> Option<&PatternError> {
        self.condition.selector_error()
    }

    /// Forbids selected files whose final path segment matches `pattern`.
    pub fn have_name(self, pattern: impl AsRef<str>) -> MatchPatternFileCondition {
        self.condition.have_name(pattern)
    }

    /// Forbids selected files whose containing folder matches `pattern`.
    pub fn be_in_folder(self, pattern: impl AsRef<str>) -> MatchPatternFileCondition {
        self.condition.be_in_folder(pattern)
    }

    /// Forbids selected files whose complete normalized path matches `pattern`.
    pub fn be_in_path(self, pattern: impl AsRef<str>) -> MatchPatternFileCondition {
        self.condition.be_in_path(pattern)
    }
}
