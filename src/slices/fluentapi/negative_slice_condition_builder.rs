use super::{ForbiddenSliceDependencyCondition, SliceScopeBuilder};

/// Immutable negated mood for forbidden slice dependencies.
#[derive(Debug, Clone)]
#[must_use = "a slice-rule mood has no effect until it receives a condition"]
pub struct NegativeSliceConditionBuilder {
    scope: SliceScopeBuilder,
}

impl NegativeSliceConditionBuilder {
    pub(super) const fn new(scope: SliceScopeBuilder) -> Self {
        Self { scope }
    }

    /// Forbids the directed dependency from `source_slice` to `target_slice`.
    pub fn contain_dependency(
        self,
        source_slice: impl Into<String>,
        target_slice: impl Into<String>,
    ) -> ForbiddenSliceDependencyCondition {
        ForbiddenSliceDependencyCondition::new(self.scope, source_slice.into(), target_slice.into())
    }

    /// Returns the slice scope carried into this mood.
    pub const fn scope(&self) -> &SliceScopeBuilder {
        &self.scope
    }
}
