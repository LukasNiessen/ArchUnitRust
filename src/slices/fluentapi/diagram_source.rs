use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{ArchUnitError, TechnicalError};

use super::SliceConfigurationError;

#[derive(Debug, Clone, PartialEq, Eq)]
enum DiagramSourceValue {
    Inline(String),
    File(PathBuf),
}

/// Immutable inline or file-backed PlantUML source, read only by a terminal check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagramSource {
    value: DiagramSourceValue,
}

impl DiagramSource {
    /// Stores inline diagram text without parsing it.
    #[must_use]
    pub fn inline(text: impl Into<String>) -> Self {
        Self {
            value: DiagramSourceValue::Inline(text.into()),
        }
    }

    /// Stores a path that will be read only when the terminal is checked.
    #[must_use]
    pub fn file(path: impl AsRef<Path>) -> Self {
        Self {
            value: DiagramSourceValue::File(path.as_ref().to_path_buf()),
        }
    }

    /// Returns whether this source carries inline text.
    #[must_use]
    pub const fn is_inline(&self) -> bool {
        matches!(self.value, DiagramSourceValue::Inline(_))
    }

    /// Returns the file path when this is a file-backed source.
    #[must_use]
    pub fn path(&self) -> Option<&Path> {
        match &self.value {
            DiagramSourceValue::Inline(_) => None,
            DiagramSourceValue::File(path) => Some(path),
        }
    }

    pub(super) fn configuration_error(&self) -> Option<SliceConfigurationError> {
        match &self.value {
            DiagramSourceValue::Inline(text) if text.trim().is_empty() => {
                Some(SliceConfigurationError::EmptyDiagramText)
            }
            DiagramSourceValue::File(path) if path.as_os_str().is_empty() => {
                Some(SliceConfigurationError::EmptyDiagramPath)
            }
            DiagramSourceValue::Inline(_) | DiagramSourceValue::File(_) => None,
        }
    }

    pub(super) fn read(&self) -> Result<String, ArchUnitError> {
        match &self.value {
            DiagramSourceValue::Inline(text) => Ok(text.clone()),
            DiagramSourceValue::File(path) => fs::read_to_string(path).map_err(|source| {
                TechnicalError::with_source(
                    format!("could not read PlantUML diagram '{}'", path.display()),
                    source,
                )
                .into()
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::DiagramSource;

    #[test]
    fn stores_inline_and_file_sources_without_reading_or_parsing() {
        let inline = DiagramSource::inline("not parsed yet");
        let file = DiagramSource::file("missing/architecture.puml");

        assert!(inline.is_inline());
        assert!(!file.is_inline());
        assert_eq!(file.path(), Some(Path::new("missing/architecture.puml")));
        assert!(inline.configuration_error().is_none());
        assert!(file.configuration_error().is_none());
    }
}
