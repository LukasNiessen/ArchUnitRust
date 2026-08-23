/// Per-check logging configuration.
///
/// Supplying this value through [`crate::CheckOptions::with_logging`] opts one check into logging;
/// omitting it keeps the check silent. Levels and destinations are added with the logging feature
/// while this non-exhaustive value keeps the options seam stable.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[non_exhaustive]
pub struct LoggingOptions {
    _private: (),
}

impl LoggingOptions {
    /// Creates the default opt-in logging configuration.
    #[must_use]
    pub const fn new() -> Self {
        Self { _private: () }
    }
}

#[cfg(test)]
mod tests {
    use super::LoggingOptions;

    #[test]
    fn default_and_new_describe_the_same_logging_configuration() {
        assert_eq!(LoggingOptions::default(), LoggingOptions::new());
    }
}
