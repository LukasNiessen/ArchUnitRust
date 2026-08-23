//! Pure assertions for named-layer architecture policies.

mod layer_definition;
mod layer_dependencies;
mod layer_dependency_rule;
mod layer_dependency_violation;

pub use layer_definition::LayerDefinition;
pub use layer_dependencies::gather_layer_dependency_violations;
pub use layer_dependency_rule::LayerDependencyRule;
pub use layer_dependency_violation::LayerDependencyViolation;
