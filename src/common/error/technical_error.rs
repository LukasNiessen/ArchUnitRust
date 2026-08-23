use std::error::Error;

type BoxedSource = Box<dyn Error + Send + Sync + 'static>;

/// A failure in ArchUnit or its execution environment.
///
/// Examples include an unreadable project, an unavailable Cargo command, and an extraction failure.
/// A rule violation is not a technical error.
#[derive(Debug, thiserror::Error)]
#[error("archunit: {message}{source_suffix}", source_suffix = source_suffix(.source.as_deref()))]
pub struct TechnicalError {
    message: String,
    #[source]
    source: Option<BoxedSource>,
}

impl TechnicalError {
    /// Creates a technical failure diagnosed entirely by ArchUnit.
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            source: None,
        }
    }

    /// Adds the lower-level failure that prevented the operation from completing.
    pub fn with_source(
        message: impl Into<String>,
        source: impl Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Returns ArchUnit's stable context without the lower-level source message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

fn source_suffix(source: Option<&(dyn Error + Send + Sync + 'static)>) -> String {
    source.map_or_else(String::new, |source| format!(": {source}"))
}

#[cfg(test)]
mod tests {
    use std::{error::Error, io};

    use super::TechnicalError;

    #[test]
    fn renders_archunit_context_without_a_source() {
        let error = TechnicalError::new("could not locate a Cargo project");

        assert_eq!(error.message(), "could not locate a Cargo project");
        assert_eq!(
            error.to_string(),
            "archunit: could not locate a Cargo project"
        );
        assert!(error.source().is_none());
    }

    #[test]
    fn exposes_the_original_technical_source() {
        let error = TechnicalError::with_source(
            "could not read Cargo.toml",
            io::Error::new(io::ErrorKind::PermissionDenied, "access denied"),
        );

        assert_eq!(
            error.to_string(),
            "archunit: could not read Cargo.toml: access denied"
        );
        assert_eq!(
            error.source().map(ToString::to_string),
            Some("access denied".to_owned())
        );
    }
}
