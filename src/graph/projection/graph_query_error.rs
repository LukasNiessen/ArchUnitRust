use crate::PatternError;

/// Invalid graph query input or a collapse that cannot produce a report node.
#[derive(Debug, Clone, thiserror::Error)]
#[non_exhaustive]
pub enum GraphQueryError {
    /// A focus, traversal, or collapse pattern could not be compiled.
    #[error("invalid {context} pattern: {source}")]
    InvalidPattern {
        /// The query modifier that owns the pattern.
        context: &'static str,
        /// The shared pattern compiler's diagnostic.
        #[source]
        source: PatternError,
    },
    /// Folder collapsing requires at least one path component.
    #[error("folder collapse depth must be greater than zero")]
    ZeroFolderDepth,
    /// Pattern collapsing needs a non-empty replacement expression.
    #[error("collapse replacement must not be empty")]
    EmptyCollapseReplacement,
    /// Snapshot titles must contain visible text.
    #[error("graph title must not be empty")]
    EmptyTitle,
    /// A capture replacement erased a selected node label.
    #[error("collapse pattern produced an empty label for node '{node}'")]
    EmptyCollapsedNode {
        /// The original normalized graph identifier.
        node: String,
    },
}

impl GraphQueryError {
    pub(crate) const fn invalid_pattern(context: &'static str, source: PatternError) -> Self {
        Self::InvalidPattern { context, source }
    }
}
