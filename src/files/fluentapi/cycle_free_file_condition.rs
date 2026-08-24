use std::collections::BTreeSet;

use crate::checkable::execute_logged_check;
use crate::{
    checkable::{CheckResult, Checkable},
    common::{
        ArchUnitError, CheckOptions, Filter, Graph, PatternError, ProjectLocator, UserError,
        extract_graph_with_options, locate_project_from, per_internal_edge, project_cycles,
        project_edges,
    },
    files::{MatchPatternFileConditionBuilder, gather_cycle_violations},
};

use super::file_rule_support::{empty_selection_violation, selected_nodes};

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
        execute_logged_check("files.no-cycles", options, |logger| {
            if let Some(error) = self.selector_error() {
                return Err(ArchUnitError::from(UserError::with_source(
                    "the file scope contains an invalid selector",
                    error.clone(),
                )));
            }

            logger.log_progress("extracting project graph")?;
            let project = locate_project_from(self.project_locator())?;
            let extraction = extract_graph_with_options(&project, options)?;
            let selected = selected_nodes(extraction.graph(), self.filters());
            logger.log_progress(format!("selected files={}", selected.len()))?;
            if let Some(violation) = empty_selection_violation(
                &selected,
                self.filters(),
                self.condition.is_negated(),
                options,
            ) {
                return Ok(vec![violation]);
            }

            let labels = selected
                .into_iter()
                .map(|node| node.label)
                .collect::<BTreeSet<_>>();
            let cycles = cycles_within(extraction.graph(), &labels);
            logger.log_progress(format!("cycles={}", cycles.len()))?;
            Ok(gather_cycle_violations(cycles))
        })
    }
}

fn cycles_within(graph: &Graph, selected: &BTreeSet<String>) -> crate::common::ProjectedCycles {
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

    use crate::common::{Edge, Graph, ImportKind};

    use super::cycles_within;

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
        let all = BTreeSet::from([
            "src/api.rs".to_owned(),
            "src/domain.rs".to_owned(),
            "src/isolated.rs".to_owned(),
        ]);
        let isolated = BTreeSet::from(["src/isolated.rs".to_owned()]);

        assert_eq!(cycles_within(&graph, &all).len(), 1);
        assert!(cycles_within(&graph, &isolated).is_empty());
    }
}
