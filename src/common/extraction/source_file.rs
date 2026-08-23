use std::path::{Path, PathBuf};

/// One Rust source file belonging to a Cargo workspace member.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub struct SourceFile {
    path: PathBuf,
    identifier: String,
}

impl SourceFile {
    pub(crate) fn new(path: PathBuf, identifier: String) -> Self {
        Self { path, identifier }
    }

    /// Returns the absolute filesystem path used for reading the source.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the normalized workspace-relative identifier used by graph nodes and patterns.
    #[must_use]
    pub fn identifier(&self) -> &str {
        &self.identifier
    }
}
