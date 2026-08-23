use std::path::PathBuf;

use crate::ProjectLocator;

use super::ProjectGraphBuilder;

/// Starts a dependency-graph report query using automatic Cargo project discovery.
pub fn project_graph() -> ProjectGraphBuilder {
    ProjectGraphBuilder::new(ProjectLocator::auto_detect())
}

/// Alias for [`project_graph`].
pub fn dependency_graph() -> ProjectGraphBuilder {
    project_graph()
}

/// Starts a dependency-graph report query at an explicit directory or `Cargo.toml` manifest.
pub fn project_graph_in(path: impl Into<PathBuf>) -> ProjectGraphBuilder {
    ProjectGraphBuilder::new(ProjectLocator::from_path(path))
}

/// Alias for [`project_graph_in`].
pub fn dependency_graph_in(path: impl Into<PathBuf>) -> ProjectGraphBuilder {
    project_graph_in(path)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{dependency_graph, dependency_graph_in, project_graph, project_graph_in};

    #[test]
    fn default_entry_points_request_auto_detection() {
        assert!(project_graph().project_locator().path().is_none());
        assert!(dependency_graph().project_locator().path().is_none());
    }

    #[test]
    fn explicit_entry_points_own_the_starting_path() {
        assert_eq!(
            project_graph_in("examples/layered")
                .project_locator()
                .path(),
            Some(Path::new("examples/layered"))
        );
        assert_eq!(
            dependency_graph_in("examples/aliased")
                .project_locator()
                .path(),
            Some(Path::new("examples/aliased"))
        );
    }
}
