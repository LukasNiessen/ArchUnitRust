use std::path::Path;

use crate::common::extraction::normalize_identifier;

/// Immutable source-file facts supplied to a user-defined predicate.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct FileInfo {
    /// The normalized workspace-relative identifier used by the graph and selectors.
    pub path: String,
    /// The final filename with its last extension removed.
    pub name: String,
    /// The last filename extension, including its leading dot, or an empty string.
    pub extension: String,
    /// The normalized containing directory, or `.` for a workspace-root file.
    pub directory: String,
    /// The complete UTF-8 source text exactly as read from disk.
    pub content: String,
    /// The number of lines containing at least one non-whitespace character.
    pub non_blank_line_count: usize,
}

impl FileInfo {
    /// Derives user-facing file facts from a graph identifier and its source text.
    #[must_use]
    pub fn new(identifier: impl AsRef<str>, content: impl Into<String>) -> Self {
        let path = normalize_identifier(identifier.as_ref());
        let content = content.into();
        let non_blank_line_count = content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .count();

        if path.is_empty() {
            return Self {
                path,
                name: String::new(),
                extension: String::new(),
                directory: String::new(),
                content,
                non_blank_line_count,
            };
        }

        let file_path = Path::new(&path);
        let name = file_path
            .file_stem()
            .map_or_else(String::new, |value| value.to_string_lossy().into_owned());
        let extension = file_path
            .extension()
            .map_or_else(String::new, |value| format!(".{}", value.to_string_lossy()));
        let directory = file_path.parent().map_or_else(
            || ".".to_owned(),
            |parent| {
                let parent = parent.to_string_lossy();
                if parent.is_empty() {
                    ".".to_owned()
                } else {
                    parent.into_owned()
                }
            },
        );

        Self {
            path,
            name,
            extension,
            directory,
            content,
            non_blank_line_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::FileInfo;

    #[test]
    fn derives_portable_path_name_extension_directory_and_non_blank_lines() {
        let info = FileInfo::new(
            r".\crates\api\src\order.service.rs",
            "use crate::domain;\r\n\r\n   \r\npub fn run() {}\r\n",
        );

        assert_eq!(info.path, "crates/api/src/order.service.rs");
        assert_eq!(info.name, "order.service");
        assert_eq!(info.extension, ".rs");
        assert_eq!(info.directory, "crates/api/src");
        assert_eq!(info.non_blank_line_count, 2);
        assert!(info.content.ends_with("\r\n"));
    }

    #[test]
    fn describes_workspace_root_and_extensionless_files() {
        let info = FileInfo::new("build", "command\n");

        assert_eq!(info.name, "build");
        assert!(info.extension.is_empty());
        assert_eq!(info.directory, ".");
        assert_eq!(info.non_blank_line_count, 1);
    }

    #[test]
    fn empty_identifier_has_no_derived_path_components() {
        let info = FileInfo::new(" . ", "\n");

        assert!(info.path.is_empty());
        assert!(info.name.is_empty());
        assert!(info.extension.is_empty());
        assert!(info.directory.is_empty());
        assert_eq!(info.non_blank_line_count, 0);
    }
}
