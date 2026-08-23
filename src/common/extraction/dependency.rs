use std::slice;

use super::ImportKind;
use super::ignore_directive::DeclarationSpan;

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
    /// A qualified path matched more than one viable internal or Cargo-visible target.
    AmbiguousReference,
    /// A path's first segment matched neither an internal module nor Cargo's external prelude.
    UnknownReference,
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
            Self::UnknownReference => "unknown-reference",
        }
    }
}

/// The classified destination of one extracted Rust dependency reference.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum DependencyTarget {
    /// A normalized workspace-relative Rust source file.
    Internal(String),
    /// A Cargo-visible crate name, including dependency renames.
    External(String),
}

impl DependencyTarget {
    /// Returns the internal file or external Cargo-visible crate name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Internal(target) | Self::External(target) => target,
        }
    }

    /// Returns whether this target is outside the analyzed workspace.
    #[must_use]
    pub const fn is_external(&self) -> bool {
        matches!(self, Self::External(_))
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
    target: Option<DependencyTarget>,
    kind: ImportKind,
    line: usize,
}

impl DependencyReference {
    pub(crate) fn new(
        source: impl Into<String>,
        referenced_path: impl Into<String>,
        target: Option<DependencyTarget>,
        kind: ImportKind,
        line: usize,
    ) -> Self {
        Self {
            source: source.into(),
            referenced_path: referenced_path.into(),
            target,
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

    /// Returns the classified destination, or `None` when a diagnostic prevented classification.
    #[must_use]
    pub const fn target(&self) -> Option<&DependencyTarget> {
        self.target.as_ref()
    }

    /// Returns the resolved workspace file when the target is internal.
    #[must_use]
    pub fn internal_target(&self) -> Option<&str> {
        match &self.target {
            Some(DependencyTarget::Internal(target)) => Some(target),
            Some(DependencyTarget::External(_)) | None => None,
        }
    }

    /// Returns the Cargo-visible crate name when the target is external.
    #[must_use]
    pub fn external_target(&self) -> Option<&str> {
        match &self.target {
            Some(DependencyTarget::External(target)) => Some(target),
            Some(DependencyTarget::Internal(_)) | None => None,
        }
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

/// The deterministic result of Rust dependency extraction before graph-edge merging.
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
    pub package: String,
    pub dependency_scope: super::cargo_project::CargoDependencyScope,
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
    pub declaration: Option<DeclarationSpan>,
    pub ignored: bool,
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
        DependencyExtraction, DependencyReference, DependencyTarget, ExtractionDiagnostic,
        ExtractionDiagnosticKind,
    };
    use crate::ImportKind;

    #[test]
    fn extraction_results_sort_and_deduplicate_data() {
        let reference = DependencyReference::new(
            "src/lib.rs",
            "crate::api::Handler",
            Some(DependencyTarget::Internal("src/api.rs".to_owned())),
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
