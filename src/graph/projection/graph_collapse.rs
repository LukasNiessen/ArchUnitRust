use super::{FolderDepthCollapse, GraphQueryError, PatternCollapse};

/// One strategy for relabeling report nodes before edge aggregation.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum GraphCollapse {
    /// Retain a fixed number of leading containing-folder components.
    FolderDepth(FolderDepthCollapse),
    /// Apply a regular-expression capture replacement.
    Pattern(PatternCollapse),
}

impl GraphCollapse {
    pub(crate) fn collapse(&self, node: &str) -> Result<String, GraphQueryError> {
        match self {
            Self::FolderDepth(strategy) => Ok(collapse_to_folder(node, strategy.depth())),
            Self::Pattern(strategy) => strategy.collapse(node),
        }
    }
}

fn collapse_to_folder(node: &str, depth: usize) -> String {
    let parts = node
        .split('/')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if parts.len() <= 1 {
        return node.to_owned();
    }

    let folders = &parts[..parts.len() - 1];
    folders
        .iter()
        .take(depth)
        .copied()
        .collect::<Vec<_>>()
        .join("/")
}

#[cfg(test)]
mod tests {
    use super::{FolderDepthCollapse, GraphCollapse, PatternCollapse};

    #[test]
    fn folder_depth_keeps_external_names_and_collapses_file_paths() {
        let collapse = GraphCollapse::FolderDepth(
            FolderDepthCollapse::new(2).expect("fixture depth should be valid"),
        );

        assert_eq!(
            collapse
                .collapse("crates/api/src/handler.rs")
                .expect("folder collapse should succeed"),
            "crates/api"
        );
        assert_eq!(
            collapse
                .collapse("serde")
                .expect("single-segment node should remain intact"),
            "serde"
        );
    }

    #[test]
    fn pattern_strategy_delegates_to_capture_replacement() {
        let collapse = GraphCollapse::Pattern(
            PatternCollapse::first_capture(r"src/([^/]+)/.*")
                .expect("fixture collapse should compile"),
        );

        assert_eq!(
            collapse
                .collapse("src/domain/model.rs")
                .expect("capture should produce a label"),
            "domain"
        );
    }
}
