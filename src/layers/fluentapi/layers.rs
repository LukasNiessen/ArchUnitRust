use std::path::PathBuf;

use crate::ProjectLocator;

use super::LayeredArchitecture;

/// Starts a named-layer architecture policy using automatic Cargo project discovery.
pub fn project_layers() -> LayeredArchitecture {
    LayeredArchitecture::new(ProjectLocator::auto_detect())
}

/// Alias for [`project_layers`].
pub fn layers() -> LayeredArchitecture {
    project_layers()
}

/// Starts a named-layer policy at an explicit directory or `Cargo.toml` manifest.
pub fn project_layers_in(path: impl Into<PathBuf>) -> LayeredArchitecture {
    LayeredArchitecture::new(ProjectLocator::from_path(path))
}

/// Alias for [`project_layers_in`].
pub fn layers_in(path: impl Into<PathBuf>) -> LayeredArchitecture {
    project_layers_in(path)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{layers, layers_in, project_layers, project_layers_in};

    #[test]
    fn default_entry_points_request_auto_detection() {
        assert!(project_layers().project_locator().path().is_none());
        assert!(layers().project_locator().path().is_none());
    }

    #[test]
    fn explicit_entry_points_own_the_starting_path() {
        assert_eq!(
            project_layers_in("examples/layered")
                .project_locator()
                .path(),
            Some(Path::new("examples/layered"))
        );
        assert_eq!(
            layers_in("examples/aliased").project_locator().path(),
            Some(Path::new("examples/aliased"))
        );
    }
}
