use crate::{FileInfo, Filter, PatternError, ProjectLocator};

use super::{
    CustomFileCondition, CycleFreeFileCondition, DependOnExternalModuleConditionBuilder,
    DependOnFileConditionBuilder, FileConditionBuilder, MatchPatternFileCondition,
    MatchPatternFileConditionBuilder,
};

/// The `should` mood for file predicates.
#[derive(Debug, Clone)]
#[must_use = "a file mood has no effect until a predicate is selected and checked"]
pub struct PositiveMatchPatternFileConditionBuilder {
    condition: MatchPatternFileConditionBuilder,
}

impl PositiveMatchPatternFileConditionBuilder {
    pub(super) const fn new(scope: FileConditionBuilder) -> Self {
        Self {
            condition: MatchPatternFileConditionBuilder::new(scope, false),
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

    /// Returns `false` for the positive mood.
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

    /// Requires the selected file dependency graph to contain no cycles.
    pub fn have_no_cycles(self) -> CycleFreeFileCondition {
        CycleFreeFileCondition::new(self.condition)
    }

    /// Requires every selected file's final path segment to match `pattern`.
    pub fn have_name(self, pattern: impl Into<crate::PatternSpec>) -> MatchPatternFileCondition {
        self.condition.have_name(pattern)
    }

    /// Requires every selected file's containing folder to match `pattern`.
    pub fn be_in_folder(self, pattern: impl Into<crate::PatternSpec>) -> MatchPatternFileCondition {
        self.condition.be_in_folder(pattern)
    }

    /// Requires every selected file's complete normalized path to match `pattern`.
    pub fn be_in_path(self, pattern: impl Into<crate::PatternSpec>) -> MatchPatternFileCondition {
        self.condition.be_in_path(pattern)
    }

    /// Starts an allowlist rule over dependencies from the selected files.
    pub fn depend_on_files(self) -> DependOnFileConditionBuilder {
        self.condition.depend_on_files()
    }

    /// Starts an allowlist rule over external crate dependencies from the selected files.
    pub fn depend_on_external_modules(self) -> DependOnExternalModuleConditionBuilder {
        self.condition.depend_on_external_modules()
    }

    /// Requires every selected file to satisfy `predicate`.
    pub fn adhere_to<F>(self, predicate: F, message: impl Into<String>) -> CustomFileCondition
    where
        F: Fn(&FileInfo) -> bool + Send + Sync + 'static,
    {
        self.condition.adhere_to(predicate, message)
    }
}
