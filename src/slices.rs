//! Architectural slice projections and rules.

pub mod assertion;
pub mod fluentapi;
pub mod projection;
pub mod uml;

pub use assertion::{
    DiagramAdherenceOptions, SliceDependencyRule, SliceDependencyViolation,
    gather_diagram_adherence_violations, gather_forbidden_slice_dependency_violations,
};
pub use fluentapi::{
    DiagramSliceCondition, DiagramSource, ForbiddenSliceDependencyCondition,
    NegativeSliceConditionBuilder, PositiveSliceConditionBuilder, SliceScopeBuilder,
    project_slices, project_slices_in, slices, slices_in,
};
pub use projection::{
    SliceProjection, SliceProjectionError, slice_by_file_suffix, slice_by_pattern, slice_by_regex,
    slice_identity,
};
pub use uml::{
    PlantUmlDependency, PlantUmlDiagram, PlantUmlError, PlantUmlParser, PlantUmlRenderer,
    export_plantuml_report,
};
