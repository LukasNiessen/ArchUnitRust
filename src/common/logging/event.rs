use crate::common::LogLevel;

/// Stable vocabulary for records emitted during architecture checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum LogEventKind {
    /// A terminal check began.
    StartCheck,
    /// A terminal check completed with a verdict.
    EndCheck,
    /// A check reported detailed execution progress.
    Progress,
    /// A check produced one architecture violation.
    Violation,
    /// A metric value was calculated.
    Metric,
    /// A caller emitted an ordinary debug message.
    Debug,
    /// A caller emitted an ordinary informational message.
    Info,
    /// A caller emitted an ordinary warning.
    Warn,
    /// A caller emitted an error.
    Error,
}

impl LogEventKind {
    /// Returns the stable lowercase event spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::StartCheck => "start check",
            Self::EndCheck => "end check",
            Self::Progress => "log progress",
            Self::Violation => "log violation",
            Self::Metric => "log metric",
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
        }
    }
}

/// One immutable, single-line logging record before destination-specific timestamping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogRecord {
    level: LogLevel,
    event: LogEventKind,
    message: String,
}

impl LogRecord {
    pub(super) fn new(level: LogLevel, event: LogEventKind, message: impl AsRef<str>) -> Self {
        Self {
            level,
            event,
            message: sanitize_message(message.as_ref()),
        }
    }

    /// Returns this record's severity.
    #[must_use]
    pub const fn level(&self) -> LogLevel {
        self.level
    }

    /// Returns this record's stable event family.
    #[must_use]
    pub const fn event(&self) -> LogEventKind {
        self.event
    }

    /// Returns the sanitized single-line event details.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Renders the deterministic destination-neutral record.
    #[must_use]
    pub fn render(&self) -> String {
        format!("[{}] {}: {}", self.level, self.event.as_str(), self.message)
    }
}

fn sanitize_message(message: &str) -> String {
    message.replace('\r', "\\r").replace('\n', "\\n")
}

#[cfg(test)]
mod tests {
    use super::{LogEventKind, LogRecord};
    use crate::common::LogLevel;

    #[test]
    fn records_use_the_fixed_vocabulary_and_remain_one_line() {
        let record = LogRecord::new(
            LogLevel::Warn,
            LogEventKind::Violation,
            "first line\r\nsecond line",
        );

        assert_eq!(record.level(), LogLevel::Warn);
        assert_eq!(record.event(), LogEventKind::Violation);
        assert_eq!(record.message(), "first line\\r\\nsecond line");
        assert_eq!(
            record.render(),
            "[WARN] log violation: first line\\r\\nsecond line"
        );
    }
}
