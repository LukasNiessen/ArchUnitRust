use crate::checkable::execute_logged_check;
use crate::{
    checkable::{CheckResult, Checkable},
    common::{
        CheckOptions, ProjectLocator, extract_graph_with_options, gather_empty_test_violations,
        locate_project_from,
    },
    slices::{SliceProjection, gather_forbidden_slice_dependency_violations},
    violation::Violation,
};

use super::{SliceConfigurationError, SliceScopeBuilder};

/// Executable `slices should not contain dependency` rule.
#[derive(Debug, Clone)]
#[must_use = "an architecture rule has no effect until it is checked"]
pub struct ForbiddenSliceDependencyCondition {
    scope: SliceScopeBuilder,
    source_slice: String,
    target_slice: String,
    configuration_error: Option<SliceConfigurationError>,
}

impl ForbiddenSliceDependencyCondition {
    pub(super) fn new(
        scope: SliceScopeBuilder,
        source_slice: String,
        target_slice: String,
    ) -> Self {
        let configuration_error = if source_slice.trim().is_empty() {
            Some(SliceConfigurationError::EmptySourceSlice)
        } else if target_slice.trim().is_empty() {
            Some(SliceConfigurationError::EmptyTargetSlice)
        } else {
            None
        };
        Self {
            scope,
            source_slice,
            target_slice,
            configuration_error,
        }
    }

    /// Returns the directed source slice named by the rule.
    #[must_use]
    pub fn source_slice(&self) -> &str {
        &self.source_slice
    }

    /// Returns the directed target slice named by the rule.
    #[must_use]
    pub fn target_slice(&self) -> &str {
        &self.target_slice
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

    /// Returns whether the fluent mood is negated.
    #[must_use]
    pub const fn is_negated(&self) -> bool {
        true
    }

    fn first_configuration_error(&self) -> Option<&SliceConfigurationError> {
        self.scope
            .configuration_error()
            .or(self.configuration_error.as_ref())
    }
}

impl Checkable for ForbiddenSliceDependencyCondition {
    fn check_with(&self, options: &CheckOptions) -> CheckResult {
        execute_logged_check("slices.dependencies", options, |logger| {
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
                true,
                options.allows_empty_tests(),
            );
            if let Some(violation) = empty.into_iter().next() {
                return Ok(vec![Violation::from(violation)]);
            }

            let projected = self.projection().project(graph);
            logger.log_progress(format!("projected dependencies={}", projected.len()))?;
            Ok(gather_forbidden_slice_dependency_violations(
                &projected,
                self.source_slice(),
                self.target_slice(),
            )
            .into_iter()
            .map(Violation::from)
            .collect())
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{checkable::Checkable, common::ArchUnitError, slices::project_slices_in};

    #[test]
    fn first_projection_error_precedes_rule_names_and_project_discovery() {
        let rule = project_slices_in("definitely/missing")
            .defined_by("src/**")
            .should_not()
            .contain_dependency("", "");

        let error = rule
            .check()
            .expect_err("the invalid projection should prevent project discovery");

        assert!(matches!(error, ArchUnitError::User(_)));
        assert!(error.to_string().contains("exactly one"));
    }

    #[test]
    fn empty_slice_names_are_deferred_user_errors() {
        let cases = [
            project_slices_in("definitely/missing")
                .should_not()
                .contain_dependency("", "target"),
            project_slices_in("definitely/missing")
                .should_not()
                .contain_dependency("source", " "),
        ];

        for rule in cases {
            assert!(matches!(rule.check(), Err(ArchUnitError::User(_))));
        }
    }
}
