#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

mod checkable;
mod common;
mod files;
mod graph;
mod layers;
mod metrics;
#[cfg(doctest)]
mod site_docs;
mod slices;
mod testing;
mod violation;

pub use checkable::{CheckResult, Checkable};
pub use common::{
    ArchUnitError, CargoProject, CargoTarget, CargoTargetKind, CheckLogger, CheckOptions,
    DEFAULT_EXCLUDED_DIRECTORIES, DependencyExtraction, DependencyReference, DependencyTarget,
    Edge, EmptyTestViolation, ExtractionDiagnostic, ExtractionDiagnosticKind, Filter, Graph,
    GraphExtraction, ImportKind, ImportKindSet, LogEventKind, LogFileMode, LogLevel, LogRecord,
    LoggingOptions, MapFunction, MappedEdge, NodeProjectionOptions, Pattern, PatternError,
    PatternExclusion, PatternOptions, PatternSpec, PatternSyntax, PatternTarget, ProjectLocator,
    ProjectedCycles, ProjectedEdge, ProjectedGraph, ProjectedNode, RegexFactory,
    RegexFactoryOptions, SourceFile, SourceOptions, TechnicalError, UserError, clear_graph_cache,
    enumerate_source_files, extract_dependencies, extract_graph, extract_graph_with_options,
    gather_empty_test_violations, identity, locate_project, locate_project_from, pattern, per_edge,
    per_external_edge, per_internal_edge, project_cycles, project_edges, project_internal_cycles,
    project_to_nodes, project_to_nodes_with_options,
};
pub use files::{
    CustomFileCondition, CustomFileViolation, CycleFreeFileCondition, CycleViolation,
    DependOnExternalModuleCondition, DependOnExternalModuleConditionBuilder, DependOnFileCondition,
    DependOnFileConditionBuilder, ExternalModuleDependencyViolation, FileConditionBuilder,
    FileDependencyViolation, FileInfo, FilePatternViolation, FilePredicate,
    MatchPatternFileCondition, MatchPatternFileConditionBuilder,
    NegatedMatchPatternFileConditionBuilder, PositiveMatchPatternFileConditionBuilder, files,
    files_in, gather_custom_file_violations, gather_cycle_violations,
    gather_external_module_dependency_violations, gather_file_dependency_violations,
    gather_matching_file_violations, project_files, project_files_in,
};
pub use graph::{
    CsvRenderer, D2Renderer, DEFAULT_GRAPH_TITLE, DotRenderer, FolderDepthCollapse, GraphCollapse,
    GraphQueryError, GraphQueryOptions, GraphRenderer, GraphReportEdge, GraphReportFormat,
    GraphReportNode, GraphReportSnapshot, GraphReportSummary, GraphSnapshotFactory, HtmlRenderer,
    JsonRenderer, MermaidRenderer, PatternCollapse, ProjectGraphBuilder, aggregate_graph_edges,
    collapse_graph_node, create_graph_snapshot, dependency_graph, dependency_graph_in,
    export_graph_report, project_graph, project_graph_in,
};
pub use layers::{
    LayerDefinition, LayerDefinitionBuilder, LayerDependencyRule, LayerDependencyRuleBuilder,
    LayerDependencyViolation, LayeredArchitecture, gather_layer_dependency_violations, layers,
    layers_in, project_layers, project_layers_in,
};
pub use metrics::{
    ArchitecturalZone, CountMetric, CountMetricsBuilder, CustomMetricCondition,
    CustomMetricSelection, CustomMetricViolation, DEFAULT_METRICS_CSS, DistanceInfo, DistanceInput,
    DistanceMetric, DistanceMetricSelection, DistanceMetricsBuilder, FieldInfo, FileMetricsInfo,
    ImplInfo, LcomInput, LcomMetric, LcomMetricSelection, LcomMetricsBuilder,
    MAXIMUM_SIZE_DISCOUNT, MethodInfo, MetricComparison, MetricMeasurement,
    MetricPredicateCondition, MetricPredicateViolation, MetricSelection, MetricSubject,
    MetricThresholdCondition, MetricThresholdError, MetricThresholdViolation, MetricZoneCondition,
    MetricZoneViolation, MetricsBuilder, MetricsExportOptions, MetricsExporter,
    MetricsExtractionError, MetricsReportData, PAIN_LIMIT, ProjectMetricsInfo,
    SIZE_NORMALIZATION_LINES, TypeInfo, TypeKind, USELESSNESS_LIMIT, build_distance_infos,
    extract_distance_infos, extract_file_metrics, extract_project_metrics,
    gather_custom_metric_violations, gather_metric_predicate_violations,
    gather_metric_threshold_violations, gather_metric_zone_violations, metrics, metrics_in,
    validate_metric_threshold,
};
pub use slices::{
    DiagramAdherenceOptions, DiagramSliceCondition, DiagramSource,
    ForbiddenSliceDependencyCondition, NegativeSliceConditionBuilder, PlantUmlDependency,
    PlantUmlDiagram, PlantUmlError, PlantUmlParser, PlantUmlRenderer,
    PositiveSliceConditionBuilder, SliceDependencyRule, SliceDependencyViolation, SliceProjection,
    SliceProjectionError, SliceScopeBuilder, export_plantuml_report,
    gather_diagram_adherence_violations, gather_forbidden_slice_dependency_violations,
    project_slices, project_slices_in, slice_by_file_suffix, slice_by_pattern, slice_by_regex,
    slice_identity, slices, slices_in,
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
