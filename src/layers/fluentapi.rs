//! Sentence-like entry points and builders for named-layer policies.

mod layer_definition_builder;
mod layer_dependency_rule_builder;
mod layered_architecture;
mod layers;

pub use layer_definition_builder::LayerDefinitionBuilder;
pub use layer_dependency_rule_builder::LayerDependencyRuleBuilder;
pub use layered_architecture::LayeredArchitecture;
pub use layers::{layers, layers_in, project_layers, project_layers_in};
