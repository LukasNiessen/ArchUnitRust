use crate::LoggingOptions;

/// Options that control how one terminal architecture check runs.
///
/// The default is deliberately defensive and quiet: empty selections fail, logging is disabled,
/// an existing graph cache may be reused, and test-only source targets are excluded. Builders
/// consume and return the bag so a configured value can be cloned and branched without mutation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct CheckOptions {
    allow_empty_tests: bool,
    logging: Option<LoggingOptions>,
    clear_cache: bool,
    include_test_sources: bool,
}

impl CheckOptions {
    /// Creates the ordinary strict, quiet check configuration.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            allow_empty_tests: false,
            logging: None,
            clear_cache: false,
            include_test_sources: false,
        }
    }

    /// Returns whether a selector that matches nothing may pass.
    #[must_use]
    pub const fn allows_empty_tests(&self) -> bool {
        self.allow_empty_tests
    }

    /// Controls whether a selector that matches nothing may pass.
    #[must_use]
    pub const fn with_allow_empty_tests(mut self, allow: bool) -> Self {
        self.allow_empty_tests = allow;
        self
    }

    /// Returns this check's logging configuration, or `None` when it is quiet.
    #[must_use]
    pub const fn logging(&self) -> Option<&LoggingOptions> {
        self.logging.as_ref()
    }

    /// Enables per-check logging with the supplied configuration.
    #[must_use]
    pub fn with_logging(mut self, logging: LoggingOptions) -> Self {
        self.logging = Some(logging);
        self
    }

    /// Returns whether the shared extraction cache must be cleared before this check.
    #[must_use]
    pub const fn clears_cache(&self) -> bool {
        self.clear_cache
    }

    /// Controls whether extraction starts from an empty graph cache.
    #[must_use]
    pub const fn with_clear_cache(mut self, clear: bool) -> Self {
        self.clear_cache = clear;
        self
    }

    /// Returns whether Cargo test, example, and benchmark targets participate in analysis.
    #[must_use]
    pub const fn includes_test_sources(&self) -> bool {
        self.include_test_sources
    }

    /// Controls whether Cargo test, example, and benchmark targets participate in analysis.
    #[must_use]
    pub const fn with_test_sources(mut self, include: bool) -> Self {
        self.include_test_sources = include;
        self
    }
}

impl Default for CheckOptions {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::CheckOptions;
    use crate::LoggingOptions;

    #[test]
    fn defaults_are_strict_quiet_cached_and_production_only() {
        let options = CheckOptions::default();

        assert!(!options.allows_empty_tests());
        assert!(options.logging().is_none());
        assert!(!options.clears_cache());
        assert!(!options.includes_test_sources());
    }

    #[test]
    fn consuming_builders_compose_every_current_option() {
        let options = CheckOptions::new()
            .with_allow_empty_tests(true)
            .with_logging(LoggingOptions::new())
            .with_clear_cache(true)
            .with_test_sources(true);

        assert!(options.allows_empty_tests());
        assert!(options.logging().is_some());
        assert!(options.clears_cache());
        assert!(options.includes_test_sources());
    }

    #[test]
    fn configured_bags_can_be_branched_without_mutating_the_base() {
        let base = CheckOptions::new().with_clear_cache(true);
        let derived = base.clone().with_allow_empty_tests(true);

        assert!(!base.allows_empty_tests());
        assert!(base.clears_cache());
        assert!(derived.allows_empty_tests());
        assert!(derived.clears_cache());
    }
}
