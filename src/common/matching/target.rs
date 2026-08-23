use std::fmt;

/// The part of an identifier against which a [`crate::Pattern`] is matched.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PatternTarget {
    /// The last path segment, such as `handler.rs`.
    Filename,
    /// The complete normalized path.
    Path,
    /// The path with its final filename removed.
    PathWithoutFilename,
    /// A Rust type name with module or path qualification removed.
    TypeName,
}

impl PatternTarget {
    pub(super) fn extract(self, identifier: &str) -> Option<String> {
        let identifier = normalize_identifier(identifier);
        if identifier.is_empty() {
            return None;
        }

        match self {
            Self::Path => Some(identifier),
            Self::Filename => identifier.rsplit('/').next().map(str::to_owned),
            Self::PathWithoutFilename => {
                let folder = identifier
                    .rsplit_once('/')
                    .map_or(".", |(folder, _)| folder);
                Some(folder.to_owned())
            }
            Self::TypeName => identifier
                .rsplit(['/', ':', '.'])
                .find(|part| !part.is_empty())
                .map(str::to_owned),
        }
    }

    /// Returns the stable diagnostic spelling of this target.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Filename => "filename",
            Self::Path => "path",
            Self::PathWithoutFilename => "path without filename",
            Self::TypeName => "type name",
        }
    }
}

impl fmt::Display for PatternTarget {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

fn normalize_identifier(identifier: &str) -> String {
    let replaced = identifier.trim().replace('\\', "/");
    replaced
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("/")
}

#[cfg(test)]
mod tests {
    use super::PatternTarget;

    #[test]
    fn extracts_each_target_from_an_identifier() {
        let identifier = r"crates\api\src\handler.rs";

        assert_eq!(
            PatternTarget::Path.extract(identifier).as_deref(),
            Some("crates/api/src/handler.rs")
        );
        assert_eq!(
            PatternTarget::Filename.extract(identifier).as_deref(),
            Some("handler.rs")
        );
        assert_eq!(
            PatternTarget::PathWithoutFilename
                .extract(identifier)
                .as_deref(),
            Some("crates/api/src")
        );
    }

    #[test]
    fn treats_the_project_root_as_a_folder() {
        assert_eq!(
            PatternTarget::PathWithoutFilename
                .extract("lib.rs")
                .as_deref(),
            Some(".")
        );
    }

    #[test]
    fn extracts_unqualified_rust_type_names() {
        for identifier in [
            "crate::api::RequestHandler",
            "crates/api.RequestHandler",
            "RequestHandler",
        ] {
            assert_eq!(
                PatternTarget::TypeName.extract(identifier).as_deref(),
                Some("RequestHandler")
            );
        }
    }

    #[test]
    fn empty_identifiers_have_no_target() {
        for target in [
            PatternTarget::Filename,
            PatternTarget::Path,
            PatternTarget::PathWithoutFilename,
            PatternTarget::TypeName,
        ] {
            assert_eq!(target.extract("  "), None);
        }
    }
}
