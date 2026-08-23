use std::{fmt, sync::Arc};

use crate::{
    ArchUnitError, CheckOptions, CheckResult, Checkable, FileInfo, FilePredicate, Filter,
    MatchPatternFileConditionBuilder, PatternError, ProjectLocator, UserError,
    extract_graph_with_options, gather_custom_file_violations, locate_project_from,
    project_to_nodes,
};

use crate::files::extraction::extract_file_info;

/// Executable rule that judges selected source files with a user-defined predicate.
#[derive(Clone)]
#[must_use = "an architecture rule has no effect until it is checked"]
pub struct CustomFileCondition {
    condition: MatchPatternFileConditionBuilder,
    predicate: Arc<FilePredicate>,
    message: String,
}

impl CustomFileCondition {
    pub(super) fn new<F>(
        condition: MatchPatternFileConditionBuilder,
        predicate: F,
        message: impl Into<String>,
    ) -> Self
    where
        F: Fn(&FileInfo) -> bool + Send + Sync + 'static,
    {
        Self {
            condition,
            predicate: Arc::new(predicate),
            message: message.into(),
        }
    }

    /// Returns the selected scope and mood carried into this terminal.
    pub const fn condition(&self) -> &MatchPatternFileConditionBuilder {
        &self.condition
    }

    /// Returns where Cargo project discovery begins.
    #[must_use]
    pub const fn project_locator(&self) -> &ProjectLocator {
        self.condition.project_locator()
    }

    /// Returns the file-scope filters in chain order.
    #[must_use]
    pub fn filters(&self) -> &[Filter] {
        self.condition.filters()
    }

    /// Returns whether satisfying the predicate is forbidden rather than required.
    #[must_use]
    pub const fn is_negated(&self) -> bool {
        self.condition.is_negated()
    }

    /// Returns the first invalid selector retained by the fluent scope.
    #[must_use]
    pub const fn selector_error(&self) -> Option<&PatternError> {
        self.condition.selector_error()
    }

    /// Returns the user-facing predicate requirement.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the stored predicate.
    #[must_use]
    pub fn predicate(&self) -> &FilePredicate {
        self.predicate.as_ref()
    }
}

impl fmt::Debug for CustomFileCondition {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CustomFileCondition")
            .field("condition", &self.condition)
            .field("message", &self.message)
            .finish_non_exhaustive()
    }
}

impl Checkable for CustomFileCondition {
    fn check_with(&self, options: &CheckOptions) -> CheckResult {
        if let Some(error) = self.selector_error() {
            return Err(ArchUnitError::from(UserError::with_source(
                "the file scope contains an invalid selector",
                error.clone(),
            )));
        }
        if self.message.trim().is_empty() {
            return Err(ArchUnitError::from(UserError::new(
                "the custom file predicate message must not be blank",
            )));
        }

        let project = locate_project_from(self.project_locator())?;
        let extraction = extract_graph_with_options(&project, options)?;
        let file_infos = project_to_nodes(extraction.graph())
            .into_iter()
            .filter(|node| {
                self.filters()
                    .iter()
                    .all(|filter| filter.matches(&node.label))
            })
            .map(|node| extract_file_info(&project, &node.label))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(gather_custom_file_violations(
            &file_infos,
            self.predicate(),
            self.message(),
            self.is_negated(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::{FileInfo, project_files_in};

    #[test]
    fn retains_scope_mood_message_and_predicate_in_a_cloneable_terminal() {
        let rule = project_files_in("examples/layered")
            .in_path("src/**")
            .should_not()
            .adhere_to(
                |file: &FileInfo| file.name.ends_with("service"),
                "have a service name",
            );
        let clone = rule.clone();
        let info = FileInfo::new("src/order_service.rs", "");

        assert!(rule.is_negated());
        assert_eq!(rule.filters().len(), 1);
        assert_eq!(rule.message(), "have a service name");
        assert!((rule.predicate())(&info));
        assert!((clone.predicate())(&info));
        assert!(format!("{rule:?}").contains("have a service name"));
    }
}
