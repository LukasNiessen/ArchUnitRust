#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

mod checkable;
mod common;
mod files;
mod graph;
mod layers;
mod slices;
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
pub use graph::fluentapi::{
    ProjectGraphBuilder, dependency_graph, dependency_graph_in, project_graph, project_graph_in,
};
pub use graph::projection::{
    DEFAULT_GRAPH_TITLE, FolderDepthCollapse, GraphCollapse, GraphQueryError, GraphQueryOptions,
    GraphReportEdge, GraphReportNode, GraphReportSnapshot, GraphReportSummary,
    GraphSnapshotFactory, PatternCollapse, aggregate_graph_edges, collapse_graph_node,
    create_graph_snapshot,
};
pub use graph::rendering::{
    CsvRenderer, D2Renderer, DotRenderer, GraphRenderer, GraphReportFormat, HtmlRenderer,
    JsonRenderer, MermaidRenderer, export_graph_report,
};
pub use layers::assertion::{
    LayerDefinition, LayerDependencyRule, LayerDependencyViolation,
    gather_layer_dependency_violations,
};
pub use layers::fluentapi::{
    LayerDefinitionBuilder, LayerDependencyRuleBuilder, LayeredArchitecture, layers, layers_in,
    project_layers, project_layers_in,
};
pub use slices::assertion::{
    DiagramAdherenceOptions, SliceDependencyRule, SliceDependencyViolation,
    gather_diagram_adherence_violations, gather_forbidden_slice_dependency_violations,
};
pub use slices::fluentapi::{
    ForbiddenSliceDependencyCondition, NegativeSliceConditionBuilder, SliceScopeBuilder,
    project_slices, project_slices_in, slices, slices_in,
};
pub use slices::projection::{
    SliceProjection, SliceProjectionError, slice_by_file_suffix, slice_by_pattern, slice_by_regex,
    slice_identity,
};
pub use slices::uml::{
    PlantUmlDependency, PlantUmlDiagram, PlantUmlError, PlantUmlParser, PlantUmlRenderer,
    export_plantuml_report,
};
pub use testing::{
    ColorChoice, ColorUtils, ResultFactory, TestResult, TestResultOptions, TestViolation,
    ViolationFactory,
};
pub use violation::{Violation, ViolationKind};

#[doc(hidden)]
pub use testing::evaluate_assertion as __evaluate_assertion;

/// Asserts that an architecture rule produces no violations.
///
/// The optional second argument is a [`CheckOptions`] expression. The rule is evaluated exactly
/// once and borrowed, so completed terminals remain reusable. Both architecture violations and
/// check errors become one assertion failure formatted by the shared testing layer.
///
/// # Examples
///
/// ```no_run
/// use archunit::{CheckOptions, assert_passes, project_files};
///
/// let rule = project_files().in_folder("src/**").should().have_no_cycles();
/// assert_passes!(rule);
///
/// let options = CheckOptions::new().with_test_sources(true);
/// assert_passes!(rule, options);
/// ```
#[macro_export]
macro_rules! assert_passes {
    ($rule:expr $(,)?) => {{
        let __archunit_check_options = $crate::CheckOptions::default();
        let __archunit_result = $crate::__evaluate_assertion(&$rule, &__archunit_check_options);
        assert!(__archunit_result.passed, "{}", __archunit_result.message);
    }};
    ($rule:expr, $options:expr $(,)?) => {{
        let __archunit_result = $crate::__evaluate_assertion(&$rule, &$options);
        assert!(__archunit_result.passed, "{}", __archunit_result.message);
    }};
}
