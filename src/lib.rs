#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

mod common;

pub use common::assertion::{EmptyTestViolation, Violation, ViolationKind};
pub use common::error::{ArchUnitError, TechnicalError, UserError};
pub use common::extraction::{
    CargoProject, CargoTarget, CargoTargetKind, DEFAULT_EXCLUDED_DIRECTORIES, DependencyExtraction,
    DependencyReference, DependencyTarget, Edge, ExtractionDiagnostic, ExtractionDiagnosticKind,
    Graph, GraphExtraction, ImportKind, ImportKindSet, ProjectLocator, SourceFile, SourceOptions,
    clear_graph_cache, enumerate_source_files, extract_dependencies, extract_graph,
    extract_graph_with_options, locate_project, locate_project_from,
};
pub use common::fluentapi::{CheckOptions, CheckResult, Checkable};
pub use common::logging::LoggingOptions;
pub use common::matching::{
    Filter, Pattern, PatternError, PatternOptions, PatternSyntax, PatternTarget, RegexFactory,
    RegexFactoryOptions,
};
pub use common::projection::{
    MapFunction, MappedEdge, NodeProjectionOptions, ProjectedEdge, ProjectedGraph, ProjectedNode,
    project_edges, project_to_nodes, project_to_nodes_with_options,
};
