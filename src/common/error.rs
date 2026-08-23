//! Failures that prevent an architecture rule from judging a project.

mod technical_error;
mod user_error;

pub use technical_error::TechnicalError;
pub use user_error::UserError;

/// A failure that prevented a rule from reaching an architecture verdict.
///
/// The variant identifies who can act on the failure. A rule that runs and finds a disagreement
/// returns [`crate::Violation`] values instead of this error.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ArchUnitError {
    /// The library or its execution environment failed.
    #[error(transparent)]
    Technical(#[from] TechnicalError),
    /// The architecture API was used incorrectly.
    #[error(transparent)]
    User(#[from] UserError),
}

impl ArchUnitError {
    /// Returns the technical failure data when the environment or library failed.
    #[must_use]
    pub const fn as_technical(&self) -> Option<&TechnicalError> {
        match self {
            Self::Technical(error) => Some(error),
            Self::User(_) => None,
        }
    }

    /// Returns the user failure data when the architecture API was used incorrectly.
    #[must_use]
    pub const fn as_user(&self) -> Option<&UserError> {
        match self {
            Self::Technical(_) => None,
            Self::User(error) => Some(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{error::Error, io};

    use super::{ArchUnitError, TechnicalError, UserError};

    #[test]
    fn classifies_technical_and_user_failures_without_message_matching() {
        let technical = ArchUnitError::from(TechnicalError::new("could not load the project"));
        let user = ArchUnitError::from(UserError::new("the folder pattern is invalid"));

        assert!(technical.as_technical().is_some());
        assert!(technical.as_user().is_none());
        assert!(user.as_technical().is_none());
        assert!(user.as_user().is_some());
    }

    #[test]
    fn transparent_wrapper_preserves_display_and_source_chain() {
        let error = ArchUnitError::from(TechnicalError::with_source(
            "could not read Cargo.toml",
            io::Error::new(io::ErrorKind::PermissionDenied, "access denied"),
        ));

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
