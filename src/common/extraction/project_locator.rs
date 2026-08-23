use std::path::{Path, PathBuf};

/// Where Cargo project discovery begins.
///
/// The default starts at the process working directory. An explicit path may name a directory
/// inside a project or its `Cargo.toml` manifest.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub struct ProjectLocator {
    path: Option<PathBuf>,
}

impl ProjectLocator {
    /// Selects automatic discovery from the current working directory.
    #[must_use]
    pub const fn auto_detect() -> Self {
        Self { path: None }
    }

    /// Starts discovery at an explicit directory or `Cargo.toml` manifest.
    #[must_use]
    pub fn from_path(path: impl Into<PathBuf>) -> Self {
        Self {
            path: Some(path.into()),
        }
    }

    /// Returns the explicit starting path, or `None` for automatic discovery.
    #[must_use]
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }
}

impl From<PathBuf> for ProjectLocator {
    fn from(path: PathBuf) -> Self {
        Self::from_path(path)
    }
}

impl From<&Path> for ProjectLocator {
    fn from(path: &Path) -> Self {
        Self::from_path(path)
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::ProjectLocator;

    #[test]
    fn default_locator_requests_auto_detection() {
        assert!(ProjectLocator::default().path().is_none());
        assert!(ProjectLocator::auto_detect().path().is_none());
    }

    #[test]
    fn explicit_locator_owns_the_user_path() {
        let locator = ProjectLocator::from_path("crates/app");

        assert_eq!(locator.path(), Some(Path::new("crates/app")));
    }
}
