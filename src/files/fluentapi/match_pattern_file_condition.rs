use crate::{
    ArchUnitError, CheckOptions, CheckResult, Checkable, Filter, MatchPatternFileConditionBuilder,
    PatternError, ProjectLocator, UserError, extract_graph_with_options,
    gather_matching_file_violations, locate_project_from,
};

use super::file_rule_support::{empty_selection_violation, selected_nodes};

/// Executable filename, folder, or path predicate for selected files.
#[derive(Debug, Clone)]
#[must_use = "an architecture rule has no effect until it is checked"]
pub struct MatchPatternFileCondition {
    condition: MatchPatternFileConditionBuilder,
    check_filter: Result<Filter, PatternError>,
}

impl MatchPatternFileCondition {
    pub(super) const fn new(
        condition: MatchPatternFileConditionBuilder,
        check_filter: Result<Filter, PatternError>,
    ) -> Self {
        Self {
            condition,
            check_filter,
        }
    }

    /// Returns the selected scope and mood carried into this terminal.
    pub const fn condition(&self) -> &MatchPatternFileConditionBuilder {
        &self.condition
    }

    /// Returns the compiled predicate filter, if its pattern was valid.
    #[must_use]
    pub fn check_filter(&self) -> Option<&Filter> {
        self.check_filter.as_ref().ok()
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

    /// Returns whether matching the predicate is forbidden rather than required.
    #[must_use]
    pub const fn is_negated(&self) -> bool {
        self.condition.is_negated()
    }

    /// Returns the first invalid pattern in sentence order.
    #[must_use]
    pub fn selector_error(&self) -> Option<&PatternError> {
        self.condition
            .selector_error()
            .or_else(|| self.check_filter.as_ref().err())
    }
}

impl Checkable for MatchPatternFileCondition {
    fn check_with(&self, options: &CheckOptions) -> CheckResult {
        if let Some(error) = self.condition.selector_error() {
            return Err(ArchUnitError::from(UserError::with_source(
                "the file scope contains an invalid selector",
                error.clone(),
            )));
        }

        let check_filter = self.check_filter.as_ref().map_err(|error| {
            ArchUnitError::from(UserError::with_source(
                "the file predicate contains an invalid pattern",
                error.clone(),
            ))
        })?;
        let project = locate_project_from(self.project_locator())?;
        let extraction = extract_graph_with_options(&project, options)?;
        let selected = selected_nodes(extraction.graph(), self.filters());
        if let Some(violation) =
            empty_selection_violation(&selected, self.filters(), self.is_negated(), options)
        {
            return Ok(vec![violation]);
        }

        Ok(gather_matching_file_violations(
            &selected,
            check_filter,
            self.is_negated(),
        ))
    }
}
