use crate::{
    ProjectLocator, SliceProjection, SliceProjectionError, slice_by_pattern, slice_by_regex,
    slice_identity,
};

use super::{NegativeSliceConditionBuilder, SliceConfigurationError};

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
    pub fn defined_by(mut self, pattern: impl AsRef<str>) -> Self {
        self.set_projection(slice_by_pattern(pattern), "pattern");
        self
    }

    /// Defines slice names through the first capture in a Rust regular expression.
    pub fn defined_by_regex(mut self, expression: impl AsRef<str>) -> Self {
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
