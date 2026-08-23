/// Options that change Cargo source discovery.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub struct SourceOptions {
    include_dev_targets: bool,
}

impl SourceOptions {
    /// Creates the production-target-only source configuration.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            include_dev_targets: false,
        }
    }

    /// Returns whether Cargo test, example, and benchmark targets are included.
    #[must_use]
    pub const fn includes_dev_targets(self) -> bool {
        self.include_dev_targets
    }

    /// Controls whether Cargo test, example, and benchmark targets are included.
    #[must_use]
    pub const fn with_dev_targets(mut self, include: bool) -> Self {
        self.include_dev_targets = include;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::SourceOptions;

    #[test]
    fn development_targets_are_opt_in() {
        assert!(!SourceOptions::default().includes_dev_targets());
        assert!(
            SourceOptions::new()
                .with_dev_targets(true)
                .includes_dev_targets()
        );
    }
}
