use std::collections::BTreeSet;

use crate::checkable::execute_logged_check;
use crate::{
    checkable::{CheckResult, Checkable},
    common::{
        ArchUnitError, CheckOptions, Edge, Filter, Graph, ImportKind, PatternError, ProjectLocator,
        ProjectedEdge, UserError, extract_graph_with_options, locate_project_from,
        per_internal_edge, project_cycles, project_edges,
    },
    files::{MatchPatternFileConditionBuilder, gather_cycle_violations},
};

use super::file_rule_support::{empty_selection_violation, selected_nodes};

/// Executable positive rule requiring the selected file graph to be acyclic.
#[derive(Debug, Clone)]
#[must_use = "an architecture rule has no effect until it is checked"]
pub struct CycleFreeFileCondition {
    condition: MatchPatternFileConditionBuilder,
    excluded_dependency_kinds: Vec<ImportKind>,
}

impl CycleFreeFileCondition {
    pub(super) const fn new(condition: MatchPatternFileConditionBuilder) -> Self {
        Self {
            condition,
            excluded_dependency_kinds: Vec::new(),
        }
    }

    /// Excludes Rust dependency syntax kinds from this cycle rule.
    ///
    /// When one source-target pair has both excluded and retained kinds, the dependency remains in
    /// the cycle graph with only its retained evidence. This is useful for separating structural
    /// `mod` and `pub use` ownership from executable `use` and path dependencies.
    pub fn excluding_dependency_kinds(
        mut self,
        kinds: impl IntoIterator<Item = ImportKind>,
    ) -> Self {
        self.excluded_dependency_kinds.extend(kinds);
        self.excluded_dependency_kinds.sort_unstable();
        self.excluded_dependency_kinds.dedup();
        self
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

    /// Returns excluded syntax kinds in stable declaration order.
    #[must_use]
    pub fn excluded_dependency_kinds(&self) -> &[ImportKind] {
        &self.excluded_dependency_kinds
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
            let cycles = cycles_within(
                extraction.graph(),
                &labels,
                self.excluded_dependency_kinds(),
            );
            logger.log_progress(format!("cycles={}", cycles.len()))?;
            Ok(gather_cycle_violations(cycles))
        })
    }
}

fn cycles_within(
    graph: &Graph,
    selected: &BTreeSet<String>,
    excluded_dependency_kinds: &[ImportKind],
) -> crate::common::ProjectedCycles {
    let edges = project_edges(graph, per_internal_edge())
        .into_iter()
        .filter(|edge| {
            selected.contains(&edge.source_label) && selected.contains(&edge.target_label)
        })
        .filter_map(|edge| without_dependency_kinds(edge, excluded_dependency_kinds))
        .collect::<Vec<_>>();
    project_cycles(&edges)
}

fn without_dependency_kinds(
    edge: ProjectedEdge,
    excluded_dependency_kinds: &[ImportKind],
) -> Option<ProjectedEdge> {
    let evidence = edge.cumulated_edges.into_iter().filter_map(|raw| {
        let retained = raw
            .import_kinds
            .iter()
            .filter(|kind| !excluded_dependency_kinds.contains(kind))
            .collect::<Vec<_>>();
        (!retained.is_empty()).then(|| Edge::new(raw.source, raw.target, raw.external, retained))
    });
    let filtered = ProjectedEdge::new(edge.source_label, edge.target_label, evidence);
    (!filtered.cumulated_edges.is_empty()).then_some(filtered)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::common::{Edge, Graph, ImportKind};
    use crate::files::project_files_in;

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

        assert_eq!(cycles_within(&graph, &all, &[]).len(), 1);
        assert!(cycles_within(&graph, &isolated, &[]).is_empty());
    }

    #[test]
    fn exclusions_remove_only_the_selected_syntax_evidence() {
        let graph = Graph::from_edges([
            Edge::new(
                "src/parent.rs",
                "src/child.rs",
                false,
                [ImportKind::Mod, ImportKind::PubUse],
            ),
            Edge::new("src/child.rs", "src/parent.rs", false, [ImportKind::Use]),
        ]);
        let selected = BTreeSet::from(["src/child.rs".to_owned(), "src/parent.rs".to_owned()]);

        assert_eq!(cycles_within(&graph, &selected, &[]).len(), 1);
        assert!(
            cycles_within(&graph, &selected, &[ImportKind::Mod, ImportKind::PubUse]).is_empty()
        );

        let graph_with_executable_evidence = Graph::from_edges([
            Edge::new(
                "src/parent.rs",
                "src/child.rs",
                false,
                [ImportKind::Mod, ImportKind::Use],
            ),
            Edge::new("src/child.rs", "src/parent.rs", false, [ImportKind::Use]),
        ]);
        let cycles = cycles_within(
            &graph_with_executable_evidence,
            &selected,
            &[ImportKind::Mod],
        );
        assert_eq!(cycles.len(), 1);
        assert!(cycles[0].iter().all(|edge| {
            edge.cumulated_edges.iter().all(|raw| {
                raw.import_kinds.contains(ImportKind::Use)
                    && !raw.import_kinds.contains(ImportKind::Mod)
            })
        }));
    }

    #[test]
    fn dependency_kind_exclusions_are_consuming_sorted_and_branchable() {
        let base = project_files_in("fixture").should().have_no_cycles();
        let executable_cycles = base.clone().excluding_dependency_kinds([
            ImportKind::PubUse,
            ImportKind::Mod,
            ImportKind::Mod,
        ]);

        assert!(base.excluded_dependency_kinds().is_empty());
        assert_eq!(
            executable_cycles.excluded_dependency_kinds(),
            &[ImportKind::PubUse, ImportKind::Mod]
        );
    }
}
