use crate::{
    ArchUnitError, CheckOptions, CheckResult, Checkable, Filter, PatternError, ProjectLocator,
    RegexFactory, UserError, extract_graph_with_options, gather_file_dependency_violations,
    locate_project_from, per_internal_edge, project_edges,
};

use super::DependOnFileConditionBuilder;

/// Executable rule over dependencies between project files.
#[derive(Debug, Clone)]
#[must_use = "an architecture rule has no effect until it is checked"]
pub struct DependOnFileCondition {
    builder: DependOnFileConditionBuilder,
    object_filters: Vec<Filter>,
    object_selector_error: Option<PatternError>,
}

impl DependOnFileCondition {
    pub(super) fn new(
        builder: DependOnFileConditionBuilder,
        object_filter: Result<Filter, PatternError>,
    ) -> Self {
        let (object_filters, object_selector_error) = match object_filter {
            Ok(filter) => (vec![filter], None),
            Err(error) => (Vec::new(), Some(error)),
        };
        Self {
            builder,
            object_filters,
            object_selector_error,
        }
    }

    /// Returns the object-stage builder that owns the subject scope and mood.
    pub const fn builder(&self) -> &DependOnFileConditionBuilder {
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

    /// Returns the dependency-target filters in chain order.
    #[must_use]
    pub fn object_filters(&self) -> &[Filter] {
        &self.object_filters
    }

    /// Returns whether matching object dependencies are forbidden rather than allowed.
    #[must_use]
    pub const fn is_negated(&self) -> bool {
        self.builder.is_negated()
    }

    /// Returns the first invalid subject or object selector in sentence order.
    #[must_use]
    pub fn selector_error(&self) -> Option<&PatternError> {
        self.builder
            .selector_error()
            .or(self.object_selector_error.as_ref())
    }

    /// Further restricts dependency targets by filename using AND semantics.
    pub fn with_name(self, pattern: impl AsRef<str>) -> Self {
        let filter = RegexFactory::default().filename_matcher(pattern);
        self.with_filter(filter)
    }

    /// Further restricts dependency targets by containing folder using AND semantics.
    pub fn in_folder(self, pattern: impl AsRef<str>) -> Self {
        let filter = RegexFactory::default().folder_matcher(pattern);
        self.with_filter(filter)
    }

    /// Further restricts dependency targets by complete path using AND semantics.
    pub fn in_path(self, pattern: impl AsRef<str>) -> Self {
        let filter = RegexFactory::default().path_matcher(pattern);
        self.with_filter(filter)
    }

    fn with_filter(mut self, filter: Result<Filter, PatternError>) -> Self {
        if self.object_selector_error.is_some() {
            return self;
        }

        match filter {
            Ok(filter) => self.object_filters.push(filter),
            Err(error) => self.object_selector_error = Some(error),
        }
        self
    }
}

impl Checkable for DependOnFileCondition {
    fn check_with(&self, options: &CheckOptions) -> CheckResult {
        if let Some(error) = self.builder.selector_error() {
            return Err(ArchUnitError::from(UserError::with_source(
                "the file scope contains an invalid selector",
                error.clone(),
            )));
        }
        if let Some(error) = &self.object_selector_error {
            return Err(ArchUnitError::from(UserError::with_source(
                "the dependency target contains an invalid selector",
                error.clone(),
            )));
        }

        let project = locate_project_from(self.project_locator())?;
        let extraction = extract_graph_with_options(&project, options)?;
        let edges = project_edges(extraction.graph(), per_internal_edge());

        Ok(gather_file_dependency_violations(
            &edges,
            self.subject_filters(),
            self.object_filters(),
            self.is_negated(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::{PatternTarget, project_files_in};

    #[test]
    fn object_selectors_chain_immutably_with_and_semantics() {
        let base = project_files_in("examples/layered")
            .should_not()
            .depend_on_files()
            .in_folder("src/service");
        let named = base.clone().with_name("*_service.rs");
        let path = base.clone().in_path("src/**");

        assert_eq!(base.object_filters().len(), 1);
        assert_eq!(named.object_filters().len(), 2);
        assert_eq!(path.object_filters().len(), 2);
        assert_eq!(
            named.object_filters()[0].target(),
            PatternTarget::PathWithoutFilename
        );
        assert_eq!(named.object_filters()[1].target(), PatternTarget::Filename);
        assert_eq!(path.object_filters()[1].target(), PatternTarget::Path);
        assert!(named.is_negated());
    }

    #[test]
    fn retains_the_first_invalid_object_selector_without_breaking_the_chain() {
        let rule = project_files_in("examples/layered")
            .should()
            .depend_on_files()
            .in_folder("src/[service")
            .with_name("*.rs");

        assert!(rule.object_filters().is_empty());
        assert_eq!(
            rule.selector_error().map(|error| error.pattern()),
            Some("src/[service")
        );
    }
}
