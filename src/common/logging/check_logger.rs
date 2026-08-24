use crate::{ArchUnitError, LogEventKind, LogLevel, LogRecord, LoggingOptions};

/// Stateless per-call façade over an explicitly borrowed [`LoggingOptions`] value.
///
/// A logger with no options is disabled and every method becomes a no-op. This value never reads
/// global configuration and can be created independently for concurrent checks.
#[derive(Debug, Clone, Copy)]
pub struct CheckLogger<'a> {
    options: Option<&'a LoggingOptions>,
}

impl<'a> CheckLogger<'a> {
    /// Creates a logger from this check's optional configuration.
    #[must_use]
    pub const fn new(options: Option<&'a LoggingOptions>) -> Self {
        Self { options }
    }

    /// Returns whether this check opted into logging.
    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        self.options.is_some()
    }

    /// Validates the explicitly configured sinks without emitting a record.
    ///
    /// Built-in architecture checks call this before project discovery. Custom [`crate::Checkable`]
    /// implementations can do the same when they use this logger directly.
    pub fn validate(&self) -> Result<(), ArchUnitError> {
        self.options.map_or(Ok(()), LoggingOptions::validate)
    }

    /// Logs the beginning of one named architecture check.
    pub fn start_check(&self, rule_name: impl AsRef<str>) -> Result<(), ArchUnitError> {
        self.emit(LogLevel::Info, LogEventKind::StartCheck, rule_name)
    }

    /// Logs the completed verdict and its violation count.
    pub fn end_check(
        &self,
        rule_name: impl AsRef<str>,
        violation_count: usize,
    ) -> Result<(), ArchUnitError> {
        let level = if violation_count == 0 {
            LogLevel::Info
        } else {
            LogLevel::Warn
        };
        self.emit(
            level,
            LogEventKind::EndCheck,
            format!("{}; violations={violation_count}", rule_name.as_ref()),
        )
    }

    /// Logs detailed execution progress at debug level.
    pub fn log_progress(&self, message: impl AsRef<str>) -> Result<(), ArchUnitError> {
        self.emit(LogLevel::Debug, LogEventKind::Progress, message)
    }

    /// Logs one architecture violation at warning level.
    pub fn log_violation(&self, message: impl AsRef<str>) -> Result<(), ArchUnitError> {
        self.emit(LogLevel::Warn, LogEventKind::Violation, message)
    }

    /// Logs one calculated metric value at debug level.
    pub fn log_metric(
        &self,
        metric_name: impl AsRef<str>,
        subject: impl AsRef<str>,
        value: f64,
        threshold: Option<f64>,
    ) -> Result<(), ArchUnitError> {
        let threshold =
            threshold.map_or_else(String::new, |threshold| format!("; threshold={threshold}"));
        self.emit(
            LogLevel::Debug,
            LogEventKind::Metric,
            format!(
                "{} [{}]={value}{threshold}",
                metric_name.as_ref(),
                subject.as_ref()
            ),
        )
    }

    /// Logs an ordinary debug message.
    pub fn debug(&self, message: impl AsRef<str>) -> Result<(), ArchUnitError> {
        self.emit(LogLevel::Debug, LogEventKind::Debug, message)
    }

    /// Logs an ordinary informational message.
    pub fn info(&self, message: impl AsRef<str>) -> Result<(), ArchUnitError> {
        self.emit(LogLevel::Info, LogEventKind::Info, message)
    }

    /// Logs an ordinary warning.
    pub fn warn(&self, message: impl AsRef<str>) -> Result<(), ArchUnitError> {
        self.emit(LogLevel::Warn, LogEventKind::Warn, message)
    }

    /// Logs an error that prevented a verdict.
    pub fn error(&self, message: impl AsRef<str>) -> Result<(), ArchUnitError> {
        self.emit(LogLevel::Error, LogEventKind::Error, message)
    }

    fn emit(
        &self,
        level: LogLevel,
        event: LogEventKind,
        message: impl AsRef<str>,
    ) -> Result<(), ArchUnitError> {
        let Some(options) = self.options else {
            return Ok(());
        };
        if !options.accepts(level) {
            return Ok(());
        }
        options.write(&LogRecord::new(level, event, message))
    }
}

#[cfg(test)]
mod tests {
    use super::CheckLogger;

    #[test]
    fn absent_options_disable_every_vocabulary_method() {
        let logger = CheckLogger::new(None);

        assert!(!logger.is_enabled());
        assert!(logger.validate().is_ok());
        assert!(logger.start_check("rule").is_ok());
        assert!(logger.log_progress("progress").is_ok());
        assert!(logger.log_violation("violation").is_ok());
        assert!(
            logger
                .log_metric("count", "subject", 1.0, Some(2.0))
                .is_ok()
        );
        assert!(logger.debug("debug").is_ok());
        assert!(logger.info("info").is_ok());
        assert!(logger.warn("warn").is_ok());
        assert!(logger.error("error").is_ok());
        assert!(logger.end_check("rule", 0).is_ok());
    }
}
