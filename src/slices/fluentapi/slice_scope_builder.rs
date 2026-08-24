use std::path::Path;

use crate::{
    ArchUnitError, CheckOptions, PlantUmlRenderer, ProjectLocator, SliceProjection,
    SliceProjectionError, export_plantuml_report, extract_graph_with_options, locate_project_from,
    slice_by_pattern, slice_by_regex, slice_identity,
};

use super::{
    NegativeSliceConditionBuilder, PositiveSliceConditionBuilder, SliceConfigurationError,
};

/// Immutable scope describing how project files become named slices.
#[derive(Debug, Clone)]
#[must_use = "a slice definition has no effect until it receives a rule terminal"]
pub struct SliceScopeBuilder {
    project_locator: ProjectLocator,
    projection: SliceProjection,
    configuration_error: Option<SliceConfigurationError>,
}

impl SliceScopeBuilder {
    pub(super) fn new(project_locator: ProjectLocator) -> Self {
        Self {
            project_locator,
            projection: slice_identity(),
            configuration_error: None,
        }
    }

    /// Defines slice names through exactly one `(**)` path capture.
    pub fn defined_by(mut self, pattern: impl Into<crate::PatternSpec>) -> Self {
        self.set_projection(slice_by_pattern(pattern), "pattern");
        self
    }

    /// Defines slice names through the first capture in a Rust regular expression.
    pub fn defined_by_regex(mut self, expression: impl Into<crate::PatternSpec>) -> Self {
        self.set_projection(slice_by_regex(expression), "regular-expression");
        self
    }

    /// Uses a pre-built slice projection, including suffix and identity projections.
    pub fn with_projection(mut self, projection: SliceProjection) -> Self {
        if self.configuration_error.is_none() {
            self.projection = projection;
        }
        self
    }

    /// Enters the negated mood for forbidden slice dependencies.
    pub fn should_not(self) -> NegativeSliceConditionBuilder {
        NegativeSliceConditionBuilder::new(self)
    }

    /// Enters the positive mood for PlantUML diagram adherence.
    pub fn should(self) -> PositiveSliceConditionBuilder {
        PositiveSliceConditionBuilder::new(self)
    }

    /// Extracts the project and renders its current slices as deterministic PlantUML.
    pub fn to_plantuml(&self) -> Result<String, ArchUnitError> {
        self.to_plantuml_with(&CheckOptions::default())
    }

    /// Renders PlantUML with explicit extraction options.
    pub fn to_plantuml_with(&self, options: &CheckOptions) -> Result<String, ArchUnitError> {
        self.validate_configuration()?;
        let project = locate_project_from(self.project_locator())?;
        let extraction = extract_graph_with_options(&project, options)?;
        let graph = extraction.graph();
        let edges = self.projection().project(graph);
        let components = self.projection().slice_labels(graph);
        PlantUmlRenderer::render_with_components(&edges, &components).map_err(|source| {
            crate::UserError::with_source("the generated PlantUML diagram is invalid", source)
                .into()
        })
    }

    /// Extracts once and exports the current slices as UTF-8 PlantUML.
    pub fn export_as_plantuml(&self, output_path: impl AsRef<Path>) -> Result<(), ArchUnitError> {
        self.export_as_plantuml_with(output_path, &CheckOptions::default())
    }

    /// Exports PlantUML with explicit extraction options.
    pub fn export_as_plantuml_with(
        &self,
        output_path: impl AsRef<Path>,
        options: &CheckOptions,
    ) -> Result<(), ArchUnitError> {
        let content = self.to_plantuml_with(options)?;
        export_plantuml_report(output_path, &content)
    }

    /// Returns where Cargo project discovery begins.
    #[must_use]
    pub const fn project_locator(&self) -> &ProjectLocator {
        &self.project_locator
    }

    /// Returns the immutable file-to-slice projection.
    #[must_use]
    pub const fn projection(&self) -> &SliceProjection {
        &self.projection
    }

    pub(super) const fn configuration_error(&self) -> Option<&SliceConfigurationError> {
        self.configuration_error.as_ref()
    }

    fn validate_configuration(&self) -> Result<(), ArchUnitError> {
        if let Some(error) = self.configuration_error() {
            Err(error.to_archunit_error())
        } else {
            Ok(())
        }
    }

    fn set_projection(
        &mut self,
        projection: Result<SliceProjection, SliceProjectionError>,
        context: &'static str,
    ) {
        if self.configuration_error.is_some() {
            return;
        }
        match projection {
            Ok(projection) => self.projection = projection,
            Err(source) => {
                self.configuration_error =
                    Some(SliceConfigurationError::InvalidProjection { context, source });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{project_slices, slice_by_file_suffix};

    #[test]
    fn definitions_are_consuming_branchable_values() {
        let base = project_slices();
        let pattern = base.clone().defined_by("src/(**)/");
        let regex = base.clone().defined_by_regex(r"\Asrc/([^/]+)/");
        let suffix = base.clone().with_projection(
            slice_by_file_suffix([("_service", "services")])
                .expect("fixture suffix should be valid"),
        );

        assert_eq!(
            base.projection().label_for("src/api/mod.rs"),
            Some("src/api/mod.rs".to_owned())
        );
        assert_eq!(
            pattern.projection().label_for("src/api/mod.rs"),
            Some("api".to_owned())
        );
        assert_eq!(
            regex.projection().label_for("src/domain/mod.rs"),
            Some("domain".to_owned())
        );
        assert_eq!(
            suffix.projection().label_for("src/order_service.rs"),
            Some("services".to_owned())
        );
        assert!(base.project_locator().path().is_none());
    }
}
