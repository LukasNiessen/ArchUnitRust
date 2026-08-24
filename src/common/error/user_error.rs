use std::error::Error;

type BoxedSource = Box<dyn Error + Send + Sync + 'static>;

/// An invalid use of the ArchUnit API.
///
/// Examples include a malformed selector, an undefined layer, and contradictory options. The
/// library has not judged the project when it returns this error.
#[derive(Debug, thiserror::Error)]
#[error("archunit: {message}{source_suffix}", source_suffix = source_suffix(.source.as_deref()))]
pub struct UserError {
    message: String,
    #[source]
    source: Option<BoxedSource>,
}

impl UserError {
    /// Creates an API-usage failure diagnosed entirely by ArchUnit.
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            source: None,
        }
    }

    /// Adds the lower-level reason that the API input was rejected.
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
    use std::error::Error;

    use crate::common::Pattern;

    use super::UserError;

    #[test]
    fn renders_archunit_context_without_a_source() {
        let error = UserError::new("the layer name must not be empty");

        assert_eq!(error.message(), "the layer name must not be empty");
        assert_eq!(
            error.to_string(),
            "archunit: the layer name must not be empty"
        );
        assert!(error.source().is_none());
    }

    #[test]
    fn exposes_the_original_api_source() {
        let pattern_error = Pattern::glob("src/[").expect_err("fixture glob should be invalid");
        let error = UserError::with_source("the folder pattern is invalid", pattern_error);

        assert!(
            error
                .to_string()
                .starts_with("archunit: the folder pattern is invalid: invalid pattern")
        );
        assert!(error.source().is_some());
    }
}
