/// Normalizes a graph identifier without consulting the host filesystem.
///
/// Graph identifiers are workspace-relative logical paths. Using lexical normalization rather than
/// [`std::fs::canonicalize`] keeps them stable when a fixture is moved and makes Windows-authored
/// paths compare with Unix-authored rule patterns.
pub(crate) fn normalize_identifier(identifier: &str) -> String {
    let identifier = identifier.trim().replace('\\', "/");
    let absolute = identifier.starts_with('/');
    let mut segments: Vec<&str> = Vec::new();

    for segment in identifier.split('/') {
        match segment {
            "" | "." => {}
            ".." if segments.last().is_some_and(|last| *last != "..") => {
                segments.pop();
            }
            ".." if !absolute => segments.push(segment),
            ".." => {}
            _ => segments.push(segment),
        }
    }

    let normalized = segments.join("/");
    if absolute && !normalized.is_empty() {
        format!("/{normalized}")
    } else if absolute {
        "/".to_owned()
    } else {
        normalized
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_identifier;

    #[test]
    fn normalizes_separators_and_dot_segments() {
        assert_eq!(
            normalize_identifier(r".\crates\api\src\.\handler.rs"),
            "crates/api/src/handler.rs"
        );
    }

    #[test]
    fn resolves_parent_segments_lexically() {
        assert_eq!(
            normalize_identifier("crates/api/src/../tests/fixture.rs"),
            "crates/api/tests/fixture.rs"
        );
        assert_eq!(normalize_identifier("../../shared.rs"), "../../shared.rs");
        assert_eq!(
            normalize_identifier("/../workspace/src/lib.rs"),
            "/workspace/src/lib.rs"
        );
    }

    #[test]
    fn preserves_names_that_are_not_paths() {
        assert_eq!(normalize_identifier("serde_json"), "serde_json");
        assert_eq!(normalize_identifier("std::collections"), "std::collections");
    }

    #[test]
    fn trims_only_the_identifier_boundary() {
        assert_eq!(normalize_identifier("  src/my file.rs  "), "src/my file.rs");
        assert_eq!(normalize_identifier(" . "), "");
    }
}
