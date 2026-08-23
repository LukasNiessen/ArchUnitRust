use crate::{Filter, PatternError, ProjectLocator};

use super::{CycleFreeFileCondition, FileConditionBuilder, MatchPatternFileConditionBuilder};

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
}
