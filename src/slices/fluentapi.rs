//! Sentence-like builders for slice architecture rules.

mod diagram_slice_condition;
mod diagram_source;
mod forbidden_slice_dependency_condition;
mod negative_slice_condition_builder;
mod positive_slice_condition_builder;
mod slice_configuration_error;
mod slice_scope_builder;
mod slices;

pub use diagram_slice_condition::DiagramSliceCondition;
pub use diagram_source::DiagramSource;
pub use forbidden_slice_dependency_condition::ForbiddenSliceDependencyCondition;
pub use negative_slice_condition_builder::NegativeSliceConditionBuilder;
pub use positive_slice_condition_builder::PositiveSliceConditionBuilder;
pub use slice_scope_builder::SliceScopeBuilder;
pub use slices::{project_slices, project_slices_in, slices, slices_in};

pub(crate) use slice_configuration_error::SliceConfigurationError;
