use crate::{PatternError, RegexFactory};

use super::LayeredArchitecture;

/// Immutable stage defining which files belong to one named layer.
#[derive(Debug, Clone)]
#[must_use = "a layer definition has no effect until it receives a selector"]
pub struct LayerDefinitionBuilder {
    architecture: LayeredArchitecture,
    layer_name: String,
}

impl LayerDefinitionBuilder {
    pub(super) fn new(architecture: LayeredArchitecture, layer_name: String) -> Self {
        Self {
            architecture,
            layer_name,
        }
    }

    /// Assigns files whose complete normalized path matches `pattern` to this layer.
    pub fn defined_by(self, pattern: impl Into<crate::PatternSpec>) -> LayeredArchitecture {
        let filter = RegexFactory::default().path_matcher(pattern);
        self.add_filter(filter, "path")
    }

    /// Assigns files whose containing directory matches `pattern` to this layer.
    pub fn defined_by_folder(self, pattern: impl Into<crate::PatternSpec>) -> LayeredArchitecture {
        let filter = RegexFactory::default().folder_matcher(pattern);
        self.add_filter(filter, "folder")
    }

    /// Returns the architecture accumulated before this definition stage.
    pub const fn architecture(&self) -> &LayeredArchitecture {
        &self.architecture
    }

    /// Returns the name of the layer being defined.
    #[must_use]
    pub fn layer_name(&self) -> &str {
        &self.layer_name
    }

    fn add_filter(
        self,
        filter: Result<crate::Filter, PatternError>,
        selector_kind: &'static str,
    ) -> LayeredArchitecture {
        self.architecture
            .with_layer_filter(self.layer_name, filter, selector_kind)
    }
}
