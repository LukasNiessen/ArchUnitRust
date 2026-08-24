/// Immutable modifiers controlling PlantUML diagram adherence.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct DiagramAdherenceOptions {
    /// Ignore dependencies when either endpoint is absent from the diagram component set.
    pub ignore_orphan_slices: bool,
    /// Ignore projected dependencies carrying external Cargo edge evidence.
    pub ignore_external_slices: bool,
}

impl DiagramAdherenceOptions {
    /// Creates strict diagram adherence options.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            ignore_orphan_slices: false,
            ignore_external_slices: false,
        }
    }

    /// Returns a value configured to ignore or enforce undeclared slice endpoints.
    #[must_use]
    pub const fn with_orphan_slices_ignored(mut self, ignored: bool) -> Self {
        self.ignore_orphan_slices = ignored;
        self
    }

    /// Returns a value configured to ignore or enforce external Cargo dependencies.
    #[must_use]
    pub const fn with_external_slices_ignored(mut self, ignored: bool) -> Self {
        self.ignore_external_slices = ignored;
        self
    }
}
