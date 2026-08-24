use std::{error::Error, fmt};

/// Invalid input in the supported PlantUML component-diagram subset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlantUmlError {
    message: String,
    line: Option<usize>,
}

impl PlantUmlError {
    pub(super) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            line: None,
        }
    }

    pub(super) fn at_line(line: usize, message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            line: Some(line),
        }
    }

    /// Returns the stable diagnostic reason.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the one-based source line for a parser failure, when available.
    #[must_use]
    pub const fn line(&self) -> Option<usize> {
        self.line
    }
}

impl fmt::Display for PlantUmlError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(line) = self.line {
            write!(
                formatter,
                "invalid PlantUML at line {line}: {}",
                self.message
            )
        } else {
            write!(formatter, "invalid PlantUML: {}", self.message)
        }
    }
}

impl Error for PlantUmlError {}
