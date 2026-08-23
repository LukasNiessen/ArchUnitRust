#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

mod common;

pub use common::assertion::{EmptyTestViolation, Violation, ViolationKind};
pub use common::error::{ArchUnitError, TechnicalError, UserError};
pub use common::extraction::{
    CargoProject, CargoTarget, CargoTargetKind, DEFAULT_EXCLUDED_DIRECTORIES, DependencyExtraction,
    DependencyReference, Edge, ExtractionDiagnostic, ExtractionDiagnosticKind, Graph, ImportKind,
    ImportKindSet, ProjectLocator, SourceFile, SourceOptions, enumerate_source_files,
    extract_dependencies, locate_project, locate_project_from,
};
pub use common::fluentapi::{CheckOptions, CheckResult, Checkable};
pub use common::logging::LoggingOptions;
pub use common::matching::{
    Filter, Pattern, PatternError, PatternOptions, PatternSyntax, PatternTarget, RegexFactory,
    RegexFactoryOptions,
};
