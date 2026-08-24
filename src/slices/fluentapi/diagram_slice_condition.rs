use crate::checkable::execute_logged_check;
use crate::{
    checkable::{CheckResult, Checkable},
    common::{
        ArchUnitError, CheckOptions, ProjectLocator, UserError, extract_graph_with_options,
        gather_empty_test_violations, locate_project_from,
    },
    slices::{
        DiagramAdherenceOptions, PlantUmlParser, SliceProjection,
        gather_diagram_adherence_violations,
    },
    violation::Violation,
};

use super::{DiagramSource, SliceConfigurationError, SliceScopeBuilder};

/// Executable `slices should adhere to diagram` rule.
#[derive(Debug, Clone)]
#[must_use = "an architecture rule has no effect until it is checked"]
pub struct DiagramSliceCondition {
    scope: SliceScopeBuilder,
    diagram_source: DiagramSource,
    options: DiagramAdherenceOptions,
    configuration_error: Option<SliceConfigurationError>,
}

impl DiagramSliceCondition {
    pub(super) fn new(
        scope: SliceScopeBuilder,
        diagram_source: DiagramSource,
        options: DiagramAdherenceOptions,
    ) -> Self {
        let configuration_error = diagram_source.configuration_error();
        Self {
            scope,
            diagram_source,
            options,
            configuration_error,
        }
    }

    /// Returns where Cargo project discovery begins.
    #[must_use]
    pub const fn project_locator(&self) -> &ProjectLocator {
        self.scope.project_locator()
    }

    /// Returns the immutable file-to-slice projection.
    #[must_use]
    pub const fn projection(&self) -> &SliceProjection {
        self.scope.projection()
    }

    /// Returns the lazy inline or file-backed diagram source.
    #[must_use]
    pub const fn diagram_source(&self) -> &DiagramSource {
        &self.diagram_source
    }

    /// Returns the immutable adherence modifiers.
    #[must_use]
    pub const fn options(&self) -> &DiagramAdherenceOptions {
        &self.options
    }

    /// Returns whether the fluent mood is negated.
    #[must_use]
    pub const fn is_negated(&self) -> bool {
        false
    }

    fn first_configuration_error(&self) -> Option<&SliceConfigurationError> {
        self.scope
            .configuration_error()
            .or(self.configuration_error.as_ref())
    }
}

impl Checkable for DiagramSliceCondition {
    fn check_with(&self, options: &CheckOptions) -> CheckResult {
        execute_logged_check("slices.diagram-adherence", options, |logger| {
            if let Some(error) = self.first_configuration_error() {
                return Err(error.to_archunit_error());
            }

            logger.log_progress("extracting project graph")?;
            let project = locate_project_from(self.project_locator())?;
            let extraction = extract_graph_with_options(&project, options)?;
            let graph = extraction.graph();
            let labels = self.projection().slice_labels(graph);
            logger.log_progress(format!("selected slices={}", labels.len()))?;
            let empty = gather_empty_test_violations(
                &labels,
                "slices",
                &[],
                false,
                options.allows_empty_tests(),
            );
            if let Some(violation) = empty.into_iter().next() {
                return Ok(vec![Violation::from(violation)]);
            }

            logger.log_progress("reading PlantUML diagram")?;
            let text = self.diagram_source.read()?;
            let diagram = PlantUmlParser::parse(&text).map_err(|source| {
                ArchUnitError::from(UserError::with_source(
                    "the PlantUML architecture diagram is invalid",
                    source,
                ))
            })?;
            let projected = self.projection().project(graph);
            logger.log_progress(format!("projected dependencies={}", projected.len()))?;
            Ok(
                gather_diagram_adherence_violations(&projected, &diagram, self.options())
                    .into_iter()
                    .map(Violation::from)
                    .collect(),
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{checkable::Checkable, common::ArchUnitError, slices::project_slices_in};

    #[test]
    fn scope_errors_precede_empty_diagram_input_and_project_discovery() {
        let rule = project_slices_in("definitely/missing")
            .defined_by("src/**")
            .should()
            .adhere_to_diagram("");

        let error = rule
            .check()
            .expect_err("the invalid slice projection should be reported first");

        assert!(matches!(error, ArchUnitError::User(_)));
        assert!(error.to_string().contains("exactly one"));
    }

    #[test]
    fn empty_diagram_sources_are_deferred_user_errors() {
        let inline = project_slices_in("definitely/missing")
            .should()
            .adhere_to_diagram(" ");
        let file = project_slices_in("definitely/missing")
            .should()
            .adhere_to_diagram_in_file("");

        assert!(matches!(inline.check(), Err(ArchUnitError::User(_))));
        assert!(matches!(file.check(), Err(ArchUnitError::User(_))));
    }
}
