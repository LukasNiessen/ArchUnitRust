#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

mod checkable;
mod common;
mod files;
mod testing;
mod violation;

pub use checkable::{CheckResult, Checkable};
pub use common::assertion::{EmptyTestViolation, gather_empty_test_violations};
pub use common::error::{ArchUnitError, TechnicalError, UserError};
pub use common::extraction::{
    CargoProject, CargoTarget, CargoTargetKind, DEFAULT_EXCLUDED_DIRECTORIES, DependencyExtraction,
    DependencyReference, DependencyTarget, Edge, ExtractionDiagnostic, ExtractionDiagnosticKind,
    Graph, GraphExtraction, ImportKind, ImportKindSet, ProjectLocator, SourceFile, SourceOptions,
    clear_graph_cache, enumerate_source_files, extract_dependencies, extract_graph,
    extract_graph_with_options, locate_project, locate_project_from,
};
pub use common::fluentapi::CheckOptions;
pub use common::logging::LoggingOptions;
pub use common::matching::{
    Filter, Pattern, PatternError, PatternOptions, PatternSyntax, PatternTarget, RegexFactory,
    RegexFactoryOptions,
};
pub use common::projection::{
    MapFunction, MappedEdge, NodeProjectionOptions, ProjectedCycles, ProjectedEdge, ProjectedGraph,
    ProjectedNode, identity, per_edge, per_external_edge, per_internal_edge, project_cycles,
    project_edges, project_internal_cycles, project_to_nodes, project_to_nodes_with_options,
};
pub use files::assertion::{
    CustomFileViolation, CycleViolation, ExternalModuleDependencyViolation,
    FileDependencyViolation, FilePatternViolation, FilePredicate, gather_custom_file_violations,
    gather_cycle_violations, gather_external_module_dependency_violations,
    gather_file_dependency_violations, gather_matching_file_violations,
};
pub use files::extraction::FileInfo;
pub use files::fluentapi::{
    CustomFileCondition, CycleFreeFileCondition, DependOnExternalModuleCondition,
    DependOnExternalModuleConditionBuilder, DependOnFileCondition, DependOnFileConditionBuilder,
    FileConditionBuilder, MatchPatternFileCondition, MatchPatternFileConditionBuilder,
    NegatedMatchPatternFileConditionBuilder, PositiveMatchPatternFileConditionBuilder, files,
    files_in, project_files, project_files_in,
};
pub use testing::{ColorChoice, ColorUtils, TestResult, TestResultOptions, TestViolation};
pub use violation::{Violation, ViolationKind};
