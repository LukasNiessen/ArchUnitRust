pub(crate) mod assertion;
pub(crate) mod fluentapi;

pub use assertion::{
    LayerDefinition, LayerDependencyRule, LayerDependencyViolation,
    gather_layer_dependency_violations,
};
pub use fluentapi::{
    LayerDefinitionBuilder, LayerDependencyRuleBuilder, LayeredArchitecture, layers, layers_in,
    project_layers, project_layers_in,
};
