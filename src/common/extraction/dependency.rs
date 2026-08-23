use std::slice;

use super::ImportKind;

/// The category of a non-fatal source extraction diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ExtractionDiagnosticKind {
    /// A source file could not be read.
    ReadFile,
    /// `syn` could not parse a source file.
    ParseFile,
    /// An outlined module declaration had no matching source file.
    MissingModule,
    /// Both supported outlined module layouts matched one declaration.
    AmbiguousModule,
    /// Following module declarations would revisit a file in the same module ancestry.
    ModuleCycle,
    /// A `#[path]` attribute was not a literal string.
    InvalidPathAttribute,
    /// A qualified path matched more than one internal logical module.
    AmbiguousReference,
}

impl ExtractionDiagnosticKind {
    /// Returns the stable report spelling for this diagnostic category.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReadFile => "read-file",
            Self::ParseFile => "parse-file",
            Self::MissingModule => "missing-module",
            Self::AmbiguousModule => "ambiguous-module",
            Self::ModuleCycle => "module-cycle",
            Self::InvalidPathAttribute => "invalid-path-attribute",
            Self::AmbiguousReference => "ambiguous-reference",
        }
    }
}

/// One non-fatal limitation encountered while extracting Rust dependencies.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub struct ExtractionDiagnostic {
    source: String,
    line: Option<usize>,
    kind: ExtractionDiagnosticKind,
    subject: Option<String>,
    candidates: Vec<String>,
    detail: Option<String>,
}

impl ExtractionDiagnostic {
    pub(crate) fn new(
        source: impl Into<String>,
        line: Option<usize>,
        kind: ExtractionDiagnosticKind,
        subject: Option<String>,
        mut candidates: Vec<String>,
        detail: Option<String>,
    ) -> Self {
        candidates.sort();
        candidates.dedup();
        Self {
            source: source.into(),
            line,
            kind,
            subject,
            candidates,
            detail,
        }
    }

    /// Returns the normalized file identifier where extraction was limited.
    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Returns the one-based source line when syntax identified one.
    #[must_use]
    pub const fn line(&self) -> Option<usize> {
        self.line
    }

    /// Returns this diagnostic's stable category.
    #[must_use]
    pub const fn kind(&self) -> ExtractionDiagnosticKind {
        self.kind
    }

    /// Returns the module or path involved, when one exists.
    #[must_use]
    pub fn subject(&self) -> Option<&str> {
        self.subject.as_deref()
    }

    /// Returns every viable target for an ambiguity in deterministic order.
    #[must_use]
    pub fn candidates(&self) -> &[String] {
        &self.candidates
    }

    /// Returns parser or I/O detail intended for extraction reports.
    #[must_use]
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }
}

/// One dependency syntax occurrence extracted from a Rust source file.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub struct DependencyReference {
    source: String,
    referenced_path: String,
    internal_target: Option<String>,
    kind: ImportKind,
    line: usize,
}

impl DependencyReference {
    pub(crate) fn new(
        source: impl Into<String>,
        referenced_path: impl Into<String>,
        internal_target: Option<String>,
        kind: ImportKind,
        line: usize,
    ) -> Self {
        Self {
            source: source.into(),
            referenced_path: referenced_path.into(),
            internal_target,
            kind,
            line,
        }
    }

    /// Returns the normalized workspace-relative file containing the syntax.
    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Returns the Rust path before Cargo-aware external classification.
    #[must_use]
    pub fn referenced_path(&self) -> &str {
        &self.referenced_path
    }

    /// Returns the resolved workspace file when the longest internal module prefix was unique.
    #[must_use]
    pub fn internal_target(&self) -> Option<&str> {
        self.internal_target.as_deref()
    }

    /// Returns the Rust syntax category that produced the reference.
    #[must_use]
    pub const fn kind(&self) -> ImportKind {
        self.kind
    }

    /// Returns the one-based source line of the dependency syntax.
    #[must_use]
    pub const fn line(&self) -> usize {
        self.line
    }
}

/// The deterministic result of Rust dependency extraction before edge classification and merging.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct DependencyExtraction {
    references: Vec<DependencyReference>,
    diagnostics: Vec<ExtractionDiagnostic>,
}

impl DependencyExtraction {
    pub(crate) fn new(
        mut references: Vec<DependencyReference>,
        mut diagnostics: Vec<ExtractionDiagnostic>,
    ) -> Self {
        references.sort();
        references.dedup();
        diagnostics.sort();
        diagnostics.dedup();
        Self {
            references,
            diagnostics,
        }
    }

    /// Returns extracted dependency syntax in deterministic order.
    #[must_use]
    pub fn references(&self) -> &[DependencyReference] {
        &self.references
    }

    /// Returns non-fatal extraction diagnostics in deterministic order.
    #[must_use]
    pub fn diagnostics(&self) -> &[ExtractionDiagnostic] {
        &self.diagnostics
    }

    /// Iterates over extracted dependency references.
    pub fn iter(&self) -> slice::Iter<'_, DependencyReference> {
        self.references.iter()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct LogicalModule {
    pub target: String,
    pub segments: Vec<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct RawReference {
    pub source: String,
    pub module: LogicalModule,
    pub segments: Vec<String>,
    pub leading_colon: bool,
    pub kind: ImportKind,
    pub line: usize,
    pub binding: Option<String>,
}

impl RawReference {
    pub fn rendered_path(&self) -> String {
        let path = self.segments.join("::");
        if self.leading_colon {
            format!("::{path}")
        } else {
            path
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct InternalResolution {
    pub source: String,
    pub module_segments: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::{
        DependencyExtraction, DependencyReference, ExtractionDiagnostic, ExtractionDiagnosticKind,
    };
    use crate::ImportKind;

    #[test]
    fn extraction_results_sort_and_deduplicate_data() {
        let reference = DependencyReference::new(
            "src/lib.rs",
            "crate::api::Handler",
            Some("src/api.rs".to_owned()),
            ImportKind::PathReference,
            4,
        );
        let diagnostic = ExtractionDiagnostic::new(
            "src/lib.rs",
            Some(3),
            ExtractionDiagnosticKind::MissingModule,
            Some("missing".to_owned()),
            Vec::new(),
            None,
        );

        let result = DependencyExtraction::new(
            vec![reference.clone(), reference],
            vec![diagnostic.clone(), diagnostic],
        );

        assert_eq!(result.references().len(), 1);
        assert_eq!(result.diagnostics().len(), 1);
        assert_eq!(result.diagnostics()[0].kind().as_str(), "missing-module");
    }
}
