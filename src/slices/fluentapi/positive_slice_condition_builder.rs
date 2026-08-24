use std::path::Path;

use crate::slices::DiagramAdherenceOptions;

use super::{DiagramSliceCondition, DiagramSource, SliceScopeBuilder};

/// Immutable positive mood and optional modifiers for diagram adherence.
#[derive(Debug, Clone)]
#[must_use = "a positive slice-rule mood has no effect until it receives a condition"]
pub struct PositiveSliceConditionBuilder {
    scope: SliceScopeBuilder,
    options: DiagramAdherenceOptions,
}

impl PositiveSliceConditionBuilder {
    pub(super) const fn new(scope: SliceScopeBuilder) -> Self {
        Self {
            scope,
            options: DiagramAdherenceOptions::new(),
        }
    }

    /// Ignores actual dependencies when either endpoint is absent from the diagram.
    pub fn ignoring_orphan_slices(mut self) -> Self {
        self.options = self.options.with_orphan_slices_ignored(true);
        self
    }

    /// Ignores actual dependencies carrying external Cargo evidence.
    pub fn ignoring_external_slices(mut self) -> Self {
        self.options = self.options.with_external_slices_ignored(true);
        self
    }

    /// Requires actual slice dependencies to be allowed by inline PlantUML.
    pub fn adhere_to_diagram(self, text: impl Into<String>) -> DiagramSliceCondition {
        DiagramSliceCondition::new(self.scope, DiagramSource::inline(text), self.options)
    }

    /// Requires actual slice dependencies to be allowed by a lazily read PlantUML file.
    pub fn adhere_to_diagram_in_file(self, path: impl AsRef<Path>) -> DiagramSliceCondition {
        DiagramSliceCondition::new(self.scope, DiagramSource::file(path), self.options)
    }

    /// Returns the slice scope carried into this mood.
    pub const fn scope(&self) -> &SliceScopeBuilder {
        &self.scope
    }

    /// Returns the immutable adherence modifiers.
    #[must_use]
    pub const fn options(&self) -> &DiagramAdherenceOptions {
        &self.options
    }
}

#[cfg(test)]
mod tests {
    use crate::slices::project_slices;

    #[test]
    fn modifiers_are_consuming_branchable_values() {
        let base = project_slices().should();
        let orphan = base.clone().ignoring_orphan_slices();
        let external = base.clone().ignoring_external_slices();

        assert!(!base.options().ignore_orphan_slices);
        assert!(!base.options().ignore_external_slices);
        assert!(orphan.options().ignore_orphan_slices);
        assert!(!orphan.options().ignore_external_slices);
        assert!(external.options().ignore_external_slices);
        assert!(!external.options().ignore_orphan_slices);
    }
}
