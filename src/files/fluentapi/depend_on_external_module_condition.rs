use crate::checkable::execute_logged_check;
use crate::{
    ArchUnitError, CheckOptions, CheckResult, Checkable, Filter, PatternError, ProjectLocator,
    RegexFactory, UserError, extract_graph_with_options,
    gather_external_module_dependency_violations, locate_project_from, per_external_edge,
    project_edges,
};

use super::{
    DependOnExternalModuleConditionBuilder,
    file_rule_support::{empty_selection_violation, selected_nodes},
};

/// Executable rule over dependencies from project files to external crates.
#[derive(Debug, Clone)]
#[must_use = "an architecture rule has no effect until it is checked"]
pub struct DependOnExternalModuleCondition {
    builder: DependOnExternalModuleConditionBuilder,
    module_filters: Vec<Filter>,
    module_selector_error: Option<PatternError>,
}

impl DependOnExternalModuleCondition {
    pub(super) fn new(
        builder: DependOnExternalModuleConditionBuilder,
        module_filter: Result<Filter, PatternError>,
    ) -> Self {
        let (module_filters, module_selector_error) = match module_filter {
            Ok(filter) => (vec![filter], None),
            Err(error) => (Vec::new(), Some(error)),
        };
        Self {
            builder,
            module_filters,
            module_selector_error,
        }
    }

    /// Returns the object-stage builder that owns the subject scope and mood.
    pub const fn builder(&self) -> &DependOnExternalModuleConditionBuilder {
        &self.builder
    }

    /// Returns where Cargo project discovery begins.
    #[must_use]
    pub const fn project_locator(&self) -> &ProjectLocator {
        self.builder.project_locator()
    }

    /// Returns the subject-file filters in chain order.
    #[must_use]
    pub fn subject_filters(&self) -> &[Filter] {
        self.builder.subject_filters()
    }

    /// Returns the external-crate filters in chain order.
    ///
    /// These filters combine with OR semantics.
    #[must_use]
    pub fn module_filters(&self) -> &[Filter] {
        &self.module_filters
    }

    /// Returns whether matching crate dependencies are forbidden rather than allowed.
    #[must_use]
    pub const fn is_negated(&self) -> bool {
        self.builder.is_negated()
    }

    /// Returns the first invalid subject or module selector in sentence order.
    #[must_use]
    pub fn selector_error(&self) -> Option<&PatternError> {
        self.builder
            .selector_error()
            .or(self.module_selector_error.as_ref())
    }

    /// Adds another external crate pattern using OR semantics.
    pub fn matching(self, pattern: impl Into<crate::PatternSpec>) -> Self {
        let filter = RegexFactory::default().path_matcher(pattern);
        self.with_filter(filter)
    }

    fn with_filter(mut self, filter: Result<Filter, PatternError>) -> Self {
        if self.module_selector_error.is_some() {
            return self;
        }

        match filter {
            Ok(filter) => self.module_filters.push(filter),
            Err(error) => self.module_selector_error = Some(error),
        }
        self
    }
}

impl Checkable for DependOnExternalModuleCondition {
    fn check_with(&self, options: &CheckOptions) -> CheckResult {
        execute_logged_check("files.external-dependencies", options, |logger| {
            if let Some(error) = self.builder.selector_error() {
                return Err(ArchUnitError::from(UserError::with_source(
                    "the file scope contains an invalid selector",
                    error.clone(),
                )));
            }
            if let Some(error) = &self.module_selector_error {
                return Err(ArchUnitError::from(UserError::with_source(
                    "the external module target contains an invalid selector",
                    error.clone(),
                )));
            }

            logger.log_progress("extracting project graph")?;
            let project = locate_project_from(self.project_locator())?;
            let extraction = extract_graph_with_options(&project, options)?;
            let selected = selected_nodes(extraction.graph(), self.subject_filters());
            logger.log_progress(format!("selected files={}", selected.len()))?;
            if let Some(violation) = empty_selection_violation(
                &selected,
                self.subject_filters(),
                self.is_negated(),
                options,
            ) {
                return Ok(vec![violation]);
            }

            let edges = project_edges(extraction.graph(), per_external_edge());
            logger.log_progress(format!("external dependencies={}", edges.len()))?;

            Ok(gather_external_module_dependency_violations(
                &edges,
                self.subject_filters(),
                self.module_filters(),
                self.is_negated(),
            ))
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::project_files_in;

    #[test]
    fn matching_selectors_chain_immutably_with_or_semantics() {
        let std_only = project_files_in("examples/layered")
            .should_not()
            .depend_on_external_modules()
            .matching("std");
        let std_or_core = std_only.clone().matching("core");

        assert_eq!(std_only.module_filters().len(), 1);
        assert_eq!(std_or_core.module_filters().len(), 2);
        assert_eq!(std_or_core.module_filters()[0].pattern().source(), "std");
        assert_eq!(std_or_core.module_filters()[1].pattern().source(), "core");
        assert!(std_or_core.is_negated());
    }

    #[test]
    fn retains_the_first_invalid_module_selector_without_breaking_the_chain() {
        let rule = project_files_in("examples/layered")
            .should()
            .depend_on_external_modules()
            .matching("[external")
            .matching("tokio");

        assert!(rule.module_filters().is_empty());
        assert_eq!(
            rule.selector_error().map(|error| error.pattern()),
            Some("[external")
        );
    }
}
