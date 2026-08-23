use crate::{Filter, PatternError, ProjectLocator, RegexFactory};

use super::{DependOnExternalModuleCondition, MatchPatternFileConditionBuilder};

/// Immutable object stage selecting external Cargo-visible crate names.
#[derive(Debug, Clone)]
#[must_use = "an external dependency stage has no effect until it is selected and checked"]
pub struct DependOnExternalModuleConditionBuilder {
    condition: MatchPatternFileConditionBuilder,
}

impl DependOnExternalModuleConditionBuilder {
    pub(super) const fn new(condition: MatchPatternFileConditionBuilder) -> Self {
        Self { condition }
    }

    /// Returns the subject scope and mood carried into this object stage.
    pub const fn condition(&self) -> &MatchPatternFileConditionBuilder {
        &self.condition
    }

    /// Returns where Cargo project discovery begins.
    #[must_use]
    pub const fn project_locator(&self) -> &ProjectLocator {
        self.condition.project_locator()
    }

    /// Returns the subject-file filters in chain order.
    #[must_use]
    pub fn subject_filters(&self) -> &[Filter] {
        self.condition.filters()
    }

    /// Returns whether matching crate dependencies are forbidden rather than allowed.
    #[must_use]
    pub const fn is_negated(&self) -> bool {
        self.condition.is_negated()
    }

    /// Returns the first invalid subject-scope selector.
    #[must_use]
    pub const fn selector_error(&self) -> Option<&PatternError> {
        self.condition.selector_error()
    }

    /// Selects external crate names matching `pattern`.
    pub fn matching(self, pattern: impl AsRef<str>) -> DependOnExternalModuleCondition {
        let filter = RegexFactory::default().path_matcher(pattern);
        DependOnExternalModuleCondition::new(self, filter)
    }
}

#[cfg(test)]
mod tests {
    use crate::{PatternTarget, project_files_in};

    #[test]
    fn enters_the_module_selector_with_the_subject_mood_intact() {
        let scope = project_files_in("examples/layered").in_folder("src/api");
        let positive = scope
            .clone()
            .should()
            .depend_on_external_modules()
            .matching("std");
        let negative = scope
            .should_not()
            .depend_on_external_modules()
            .matching("tokio");

        assert!(!positive.is_negated());
        assert!(negative.is_negated());
        assert_eq!(positive.module_filters().len(), 1);
        assert_eq!(positive.module_filters()[0].target(), PatternTarget::Path);
    }
}
