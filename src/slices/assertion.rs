//! Pure assertions over already-projected slice dependencies.

mod forbidden_slice_dependencies;
mod slice_dependency_rule;
mod slice_dependency_violation;

pub use forbidden_slice_dependencies::gather_forbidden_slice_dependency_violations;
pub use slice_dependency_rule::SliceDependencyRule;
pub use slice_dependency_violation::SliceDependencyViolation;
