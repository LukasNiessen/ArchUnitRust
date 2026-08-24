use crate::{Filter, PatternError, ProjectLocator, RegexFactory};

use super::{DependOnFileCondition, MatchPatternFileConditionBuilder};

/// Immutable object stage selecting internal dependency targets.
#[derive(Debug, Clone)]
#[must_use = "a dependency object stage has no effect until it is selected and checked"]
pub struct DependOnFileConditionBuilder {
    condition: MatchPatternFileConditionBuilder,
}

impl DependOnFileConditionBuilder {
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

    /// Returns whether matching object dependencies are forbidden rather than allowed.
    #[must_use]
    pub const fn is_negated(&self) -> bool {
        self.condition.is_negated()
    }

    /// Returns the first invalid subject-scope selector.
    #[must_use]
    pub const fn selector_error(&self) -> Option<&PatternError> {
        self.condition.selector_error()
    }

    /// Selects dependency targets whose final path segment matches `pattern`.
    pub fn with_name(self, pattern: impl Into<crate::PatternSpec>) -> DependOnFileCondition {
        let filter = RegexFactory::default().filename_matcher(pattern);
        DependOnFileCondition::new(self, filter)
    }

    /// Selects dependency targets whose containing folder matches `pattern`.
    pub fn in_folder(self, pattern: impl Into<crate::PatternSpec>) -> DependOnFileCondition {
        let filter = RegexFactory::default().folder_matcher(pattern);
        DependOnFileCondition::new(self, filter)
    }

    /// Selects dependency targets whose complete normalized path matches `pattern`.
    pub fn in_path(self, pattern: impl Into<crate::PatternSpec>) -> DependOnFileCondition {
        let filter = RegexFactory::default().path_matcher(pattern);
        DependOnFileCondition::new(self, filter)
    }
}

#[cfg(test)]
mod tests {
    use crate::{PatternTarget, project_files_in};

    #[test]
    fn enters_each_object_selector_with_the_subject_mood_intact() {
        let scope = project_files_in("examples/layered").in_folder("src/api");
        let named = scope
            .clone()
            .should()
            .depend_on_files()
            .with_name("*_service.rs");
        let folder = scope
            .clone()
            .should_not()
            .depend_on_files()
            .in_folder("src/service");
        let path = scope
            .should_not()
            .depend_on_files()
            .in_path("src/service/**");

        assert!(!named.is_negated());
        assert!(folder.is_negated());
        assert!(path.is_negated());
        assert_eq!(named.object_filters()[0].target(), PatternTarget::Filename);
        assert_eq!(
            folder.object_filters()[0].target(),
            PatternTarget::PathWithoutFilename
        );
        assert_eq!(path.object_filters()[0].target(), PatternTarget::Path);
    }
}
