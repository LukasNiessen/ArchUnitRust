use std::path::PathBuf;

use crate::ProjectLocator;

use super::FileConditionBuilder;

/// Starts a file architecture rule using automatic Cargo project discovery.
pub fn project_files() -> FileConditionBuilder {
    FileConditionBuilder::new(ProjectLocator::auto_detect())
}

/// Alias for [`project_files`].
pub fn files() -> FileConditionBuilder {
    project_files()
}

/// Starts a file architecture rule at an explicit directory or `Cargo.toml` manifest.
pub fn project_files_in(path: impl Into<PathBuf>) -> FileConditionBuilder {
    FileConditionBuilder::new(ProjectLocator::from_path(path))
}

/// Alias for [`project_files_in`].
pub fn files_in(path: impl Into<PathBuf>) -> FileConditionBuilder {
    project_files_in(path)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{files, files_in, project_files, project_files_in};

    #[test]
    fn default_entry_points_request_auto_detection() {
        assert!(project_files().project_locator().path().is_none());
        assert!(files().project_locator().path().is_none());
    }

    #[test]
    fn explicit_entry_points_own_the_starting_path() {
        assert_eq!(
            project_files_in("examples/layered")
                .project_locator()
                .path(),
            Some(Path::new("examples/layered"))
        );
        assert_eq!(
            files_in("examples/aliased").project_locator().path(),
            Some(Path::new("examples/aliased"))
        );
    }
}
