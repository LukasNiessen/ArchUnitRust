use super::LayeredArchitecture;

/// Immutable stage adding an allowlist or blocklist policy for one source layer.
#[derive(Debug, Clone)]
#[must_use = "a layer policy has no effect until an allowlist or blocklist is selected"]
pub struct LayerDependencyRuleBuilder {
    architecture: LayeredArchitecture,
    layer_name: String,
}

impl LayerDependencyRuleBuilder {
    pub(super) fn new(architecture: LayeredArchitecture, layer_name: String) -> Self {
        Self {
            architecture,
            layer_name,
        }
    }

    /// Allows dependencies only to the named layers.
    ///
    /// Passing an empty slice seals the source layer against every cross-layer dependency.
    pub fn may_only_depend_on_layers(self, layer_names: &[&str]) -> LayeredArchitecture {
        self.architecture
            .with_allowed_dependencies(self.layer_name, layer_names)
    }

    /// Forbids dependencies to the named layers.
    ///
    /// At least one target is required; an empty blocklist is a user error reported by `check()`.
    pub fn may_not_depend_on_layers(self, layer_names: &[&str]) -> LayeredArchitecture {
        self.architecture
            .with_forbidden_dependencies(self.layer_name, layer_names)
    }

    /// Returns the architecture accumulated before this policy stage.
    pub const fn architecture(&self) -> &LayeredArchitecture {
        &self.architecture
    }

    /// Returns the source layer to which the next policy applies.
    #[must_use]
    pub fn layer_name(&self) -> &str {
        &self.layer_name
    }
}
