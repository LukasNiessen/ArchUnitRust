use crate::common::Filter;

use super::{GraphCollapse, GraphQueryError};

/// Immutable selection, collapse, and presentation options for one graph snapshot.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct GraphQueryOptions {
    include_external_dependencies: bool,
    include_self_dependencies: bool,
    focus: Option<Filter>,
    focus_depth: usize,
    reachable_from: Option<Filter>,
    dependents_of: Option<Filter>,
    collapse: Option<GraphCollapse>,
    title: Option<String>,
}

impl Default for GraphQueryOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphQueryOptions {
    /// Creates the complete, uncollapsed internal graph query.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            include_external_dependencies: false,
            include_self_dependencies: false,
            focus: None,
            focus_depth: 1,
            reachable_from: None,
            dependents_of: None,
            collapse: None,
            title: None,
        }
    }

    /// Controls whether external dependency targets and edges appear.
    #[must_use]
    pub const fn with_external_dependencies(mut self, include: bool) -> Self {
        self.include_external_dependencies = include;
        self
    }

    /// Controls whether extracted and collapse-produced self edges appear.
    #[must_use]
    pub const fn with_self_dependencies(mut self, include: bool) -> Self {
        self.include_self_dependencies = include;
        self
    }

    /// Adds an undirected focus query expanded to `depth` neighbors.
    #[must_use]
    pub fn with_focus(mut self, focus: Filter, depth: usize) -> Self {
        self.focus = Some(focus);
        self.focus_depth = depth;
        self
    }

    /// Adds a transitive outgoing-dependency query.
    #[must_use]
    pub fn with_reachable_from(mut self, reachable_from: Filter) -> Self {
        self.reachable_from = Some(reachable_from);
        self
    }

    /// Adds a transitive incoming-dependent query.
    #[must_use]
    pub fn with_dependents_of(mut self, dependents_of: Filter) -> Self {
        self.dependents_of = Some(dependents_of);
        self
    }

    /// Selects one node-collapse strategy.
    #[must_use]
    pub fn with_collapse(mut self, collapse: GraphCollapse) -> Self {
        self.collapse = Some(collapse);
        self
    }

    /// Sets a non-empty report title.
    pub fn with_title(mut self, title: impl Into<String>) -> Result<Self, GraphQueryError> {
        let title = title.into();
        if title.trim().is_empty() {
            return Err(GraphQueryError::EmptyTitle);
        }
        self.title = Some(title);
        Ok(self)
    }

    /// Returns whether external dependencies participate in this query.
    #[must_use]
    pub const fn includes_external_dependencies(&self) -> bool {
        self.include_external_dependencies
    }

    /// Returns whether self dependencies participate in the report.
    #[must_use]
    pub const fn includes_self_dependencies(&self) -> bool {
        self.include_self_dependencies
    }

    /// Returns the optional focus selector.
    #[must_use]
    pub const fn focus(&self) -> Option<&Filter> {
        self.focus.as_ref()
    }

    /// Returns the undirected focus expansion depth.
    #[must_use]
    pub const fn focus_depth(&self) -> usize {
        self.focus_depth
    }

    /// Returns the optional outgoing traversal selector.
    #[must_use]
    pub const fn reachable_from(&self) -> Option<&Filter> {
        self.reachable_from.as_ref()
    }

    /// Returns the optional incoming traversal selector.
    #[must_use]
    pub const fn dependents_of(&self) -> Option<&Filter> {
        self.dependents_of.as_ref()
    }

    /// Returns the optional node-collapse strategy.
    #[must_use]
    pub const fn collapse(&self) -> Option<&GraphCollapse> {
        self.collapse.as_ref()
    }

    /// Returns the custom report title, if configured.
    #[must_use]
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use crate::common::RegexFactory;

    use super::GraphQueryOptions;
    use crate::graph::GraphQueryError;

    #[test]
    fn defaults_exclude_external_and_self_edges_without_queries_or_collapse() {
        let options = GraphQueryOptions::new();

        assert!(!options.includes_external_dependencies());
        assert!(!options.includes_self_dependencies());
        assert!(options.focus().is_none());
        assert_eq!(options.focus_depth(), 1);
        assert!(options.reachable_from().is_none());
        assert!(options.dependents_of().is_none());
        assert!(options.collapse().is_none());
        assert!(options.title().is_none());
    }

    #[test]
    fn consuming_modifiers_are_branchable_values() {
        let filter = RegexFactory::default()
            .path_matcher("src/domain/**")
            .expect("fixture selector should compile");
        let base = GraphQueryOptions::new();
        let focused = base.clone().with_focus(filter, 2);
        let titled = focused
            .clone()
            .with_title("Domain")
            .expect("visible title should be valid");

        assert!(base.focus().is_none());
        assert_eq!(focused.focus_depth(), 2);
        assert_eq!(titled.title(), Some("Domain"));
        assert!(matches!(
            base.with_title(" "),
            Err(GraphQueryError::EmptyTitle)
        ));
    }
}
