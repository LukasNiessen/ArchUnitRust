use std::fmt;

/// Severity threshold for one check's log records.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum LogLevel {
    /// Detailed progress and metric values.
    Debug,
    /// Check lifecycle and ordinary informational records.
    Info,
    /// Architecture violations and recoverable concerns.
    Warn,
    /// Technical or user errors that prevent a verdict.
    Error,
}

impl LogLevel {
    /// Returns the stable uppercase record spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
        }
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}
