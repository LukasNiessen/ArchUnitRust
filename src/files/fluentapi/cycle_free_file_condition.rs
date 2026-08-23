use std::collections::BTreeSet;

use crate::{
    ArchUnitError, CheckOptions, CheckResult, Checkable, Filter, Graph,
    MatchPatternFileConditionBuilder, PatternError, ProjectLocator, UserError,
    extract_graph_with_options, gather_cycle_violations, locate_project_from, per_internal_edge,
    project_cycles, project_edges, project_to_nodes,
};

/// Executable positive rule requiring the selected file graph to be acyclic.
#[derive(Debug, Clone)]
#[must_use = "an architecture rule has no effect until it is checked"]
pub struct CycleFreeFileCondition {
    condition: MatchPatternFileConditionBuilder,
}

impl CycleFreeFileCondition {
    pub(super) const fn new(condition: MatchPatternFileConditionBuilder) -> Self {
        Self { condition }
    }

    /// Returns where Cargo project discovery begins.
    #[must_use]
    pub const fn project_locator(&self) -> &ProjectLocator {
        self.condition.project_locator()
    }

    /// Returns the selected file-scope filters in chain order.
    #[must_use]
    pub fn filters(&self) -> &[Filter] {
        self.condition.filters()
    }

    /// Returns the first invalid selector retained by the fluent scope.
    #[must_use]
    pub const fn selector_error(&self) -> Option<&PatternError> {
        self.condition.selector_error()
    }
}

impl Checkable for CycleFreeFileCondition {
    fn check_with(&self, options: &CheckOptions) -> CheckResult {
        if let Some(error) = self.selector_error() {
            return Err(ArchUnitError::from(UserError::with_source(
                "the file scope contains an invalid selector",
                error.clone(),
            )));
        }

        let project = locate_project_from(self.project_locator())?;
        let extraction = extract_graph_with_options(&project, options)?;
        let selected = selected_labels(extraction.graph(), self.filters());
        let cycles = cycles_within(extraction.graph(), &selected);
        Ok(gather_cycle_violations(cycles))
    }
}

fn selected_labels(graph: &Graph, filters: &[Filter]) -> BTreeSet<String> {
    project_to_nodes(graph)
        .into_iter()
        .filter(|node| filters.iter().all(|filter| filter.matches(&node.label)))
        .map(|node| node.label)
        .collect()
}

fn cycles_within(graph: &Graph, selected: &BTreeSet<String>) -> crate::ProjectedCycles {
    let edges = project_edges(graph, per_internal_edge())
        .into_iter()
        .filter(|edge| {
            selected.contains(&edge.source_label) && selected.contains(&edge.target_label)
        })
        .collect::<Vec<_>>();
    project_cycles(&edges)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::{Edge, Graph, ImportKind, RegexFactory};

    use super::{cycles_within, selected_labels};

    fn cyclic_graph() -> Graph {
        Graph::from_edges([
            Edge::self_edge("src/api.rs"),
            Edge::self_edge("src/domain.rs"),
            Edge::self_edge("src/isolated.rs"),
            Edge::new("src/api.rs", "src/domain.rs", false, [ImportKind::Use]),
            Edge::new("src/domain.rs", "src/api.rs", false, [ImportKind::Use]),
        ])
    }

    #[test]
    fn detects_only_cycles_wholly_inside_the_selected_scope() {
        let graph = cyclic_graph();
        let all = selected_labels(&graph, &[]);
        let isolated_filter = RegexFactory::default()
            .exact_file_matcher("src/isolated.rs")
            .expect("fixture selector should compile");
        let isolated = selected_labels(&graph, &[isolated_filter]);

        assert_eq!(cycles_within(&graph, &all).len(), 1);
        assert!(cycles_within(&graph, &isolated).is_empty());
    }

    #[test]
    fn selected_labels_use_and_semantics() {
        let graph = cyclic_graph();
        let filters = [
            RegexFactory::default()
                .path_matcher("src/**")
                .expect("fixture path selector should compile"),
            RegexFactory::default()
                .filename_matcher("api.rs")
                .expect("fixture filename selector should compile"),
        ];

        assert_eq!(
            selected_labels(&graph, &filters),
            BTreeSet::from(["src/api.rs".to_owned()])
        );
    }
}
