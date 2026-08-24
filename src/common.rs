pub(crate) mod assertion;
pub(crate) mod error;
pub(crate) mod extraction;
pub(crate) mod fluentapi;
pub(crate) mod logging;
pub(crate) mod matching;
pub(crate) mod projection;

pub use assertion::{EmptyTestViolation, gather_empty_test_violations};
pub use error::{ArchUnitError, TechnicalError, UserError};
pub use extraction::{
    CargoProject, CargoTarget, CargoTargetKind, DEFAULT_EXCLUDED_DIRECTORIES, DependencyExtraction,
    DependencyReference, DependencyTarget, Edge, ExtractionDiagnostic, ExtractionDiagnosticKind,
    Graph, GraphExtraction, ImportKind, ImportKindSet, ProjectLocator, SourceFile, SourceOptions,
    clear_graph_cache, enumerate_source_files, extract_dependencies, extract_graph,
    extract_graph_with_options, locate_project, locate_project_from,
};
pub use fluentapi::CheckOptions;
pub use logging::{CheckLogger, LogEventKind, LogFileMode, LogLevel, LogRecord, LoggingOptions};
pub use matching::{
    Filter, Pattern, PatternError, PatternExclusion, PatternOptions, PatternSpec, PatternSyntax,
    PatternTarget, RegexFactory, RegexFactoryOptions, pattern,
};
pub use projection::{
    MapFunction, MappedEdge, NodeProjectionOptions, ProjectedCycles, ProjectedEdge, ProjectedGraph,
    ProjectedNode, identity, per_edge, per_external_edge, per_internal_edge, project_cycles,
    project_edges, project_internal_cycles, project_to_nodes, project_to_nodes_with_options,
};
