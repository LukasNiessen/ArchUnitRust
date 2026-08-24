use std::path::Path;

use crate::common::ProjectLocator;

use super::SliceScopeBuilder;

/// Starts a lazy slice architecture scope with automatic Cargo project discovery.
pub fn project_slices() -> SliceScopeBuilder {
    SliceScopeBuilder::new(ProjectLocator::auto_detect())
}

/// Starts a lazy slice architecture scope at an explicit directory or Cargo manifest.
pub fn project_slices_in(path: impl AsRef<Path>) -> SliceScopeBuilder {
    SliceScopeBuilder::new(ProjectLocator::from_path(path.as_ref()))
}

/// Alias for [`project_slices`].
pub fn slices() -> SliceScopeBuilder {
    project_slices()
}

/// Alias for [`project_slices_in`].
pub fn slices_in(path: impl AsRef<Path>) -> SliceScopeBuilder {
    project_slices_in(path)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{project_slices, project_slices_in, slices, slices_in};

    #[test]
    fn default_entry_points_request_auto_detection() {
        assert!(project_slices().project_locator().path().is_none());
        assert!(slices().project_locator().path().is_none());
    }

    #[test]
    fn explicit_entry_points_own_the_starting_path() {
        let project = project_slices_in("fixtures/project");
        let alias = slices_in("fixtures/alias");

        assert_eq!(
            project.project_locator().path(),
            Some(Path::new("fixtures/project"))
        );
        assert_eq!(
            alias.project_locator().path(),
            Some(Path::new("fixtures/alias"))
        );
    }
}
