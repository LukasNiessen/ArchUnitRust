use crate::common::Filter;

/// A named architectural layer and the file selectors that define it.
///
/// Selectors within one definition use OR semantics. When definitions overlap, policy evaluation
/// assigns a file to the first matching layer in declaration order.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct LayerDefinition {
    /// The stable name used by dependency policies.
    pub name: String,
    /// File selectors that independently assign files to this layer.
    pub filters: Vec<Filter>,
}

impl LayerDefinition {
    /// Creates a named layer from one or more file selectors.
    #[must_use]
    pub fn new(name: impl Into<String>, filters: impl IntoIterator<Item = Filter>) -> Self {
        Self {
            name: name.into(),
            filters: filters.into_iter().collect(),
        }
    }

    /// Returns whether this layer contains `file_path`.
    #[must_use]
    pub fn matches(&self, file_path: &str) -> bool {
        self.filters.iter().any(|filter| filter.matches(file_path))
    }
}

#[cfg(test)]
mod tests {
    use crate::common::RegexFactory;

    use super::LayerDefinition;

    #[test]
    fn selectors_define_one_layer_with_or_semantics() {
        let factory = RegexFactory::default();
        let layer = LayerDefinition::new(
            "application",
            [
                factory
                    .folder_matcher("src/api")
                    .expect("fixture folder should compile"),
                factory
                    .path_matcher("src/legacy/**")
                    .expect("fixture path should compile"),
            ],
        );

        assert!(layer.matches("src/api/handler.rs"));
        assert!(layer.matches("src/legacy/handler.rs"));
        assert!(!layer.matches("src/database/store.rs"));
    }
}
