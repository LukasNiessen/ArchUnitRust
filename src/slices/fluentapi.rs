//! Sentence-like builders for slice architecture rules.

mod forbidden_slice_dependency_condition;
mod negative_slice_condition_builder;
mod slice_configuration_error;
mod slice_scope_builder;
mod slices;

pub use forbidden_slice_dependency_condition::ForbiddenSliceDependencyCondition;
pub use negative_slice_condition_builder::NegativeSliceConditionBuilder;
pub use slice_scope_builder::SliceScopeBuilder;
pub use slices::{project_slices, project_slices_in, slices, slices_in};

pub(crate) use slice_configuration_error::SliceConfigurationError;
