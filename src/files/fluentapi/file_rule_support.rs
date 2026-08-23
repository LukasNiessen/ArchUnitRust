use crate::{
    CheckOptions, Filter, Graph, ProjectedNode, Violation, gather_empty_test_violations,
    project_to_nodes,
};

pub(super) fn selected_nodes(graph: &Graph, filters: &[Filter]) -> Vec<ProjectedNode> {
    project_to_nodes(graph)
        .into_iter()
        .filter(|node| filters.iter().all(|filter| filter.matches(&node.label)))
        .collect()
}

pub(super) fn empty_selection_violation(
    selected: &[ProjectedNode],
    filters: &[Filter],
    is_negated: bool,
    options: &CheckOptions,
) -> Option<Violation> {
    gather_empty_test_violations(
        selected,
        "files",
        filters,
        is_negated,
        options.allows_empty_tests(),
    )
    .into_iter()
    .next()
    .map(Violation::from)
}

#[cfg(test)]
mod tests {
    use crate::{CheckOptions, Edge, Graph, RegexFactory};

    use super::{empty_selection_violation, selected_nodes};

    fn graph() -> Graph {
        Graph::from_edges([
            Edge::self_edge("src/orders/order_service.rs"),
            Edge::self_edge("src/orders/order_repository.rs"),
            Edge::self_edge("tests/orders/order_service_test.rs"),
        ])
    }

    #[test]
    fn scope_filters_select_nodes_with_and_semantics() {
        let filters = [
            RegexFactory::default()
                .folder_matcher("src/**")
                .expect("fixture pattern should compile"),
            RegexFactory::default()
                .filename_matcher("*_service.rs")
                .expect("fixture pattern should compile"),
        ];

        let selected = selected_nodes(&graph(), &filters);

        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].label, "src/orders/order_service.rs");
    }

    #[test]
    fn strict_empty_selection_becomes_one_mood_aware_violation() {
        let filter = RegexFactory::default()
            .path_matcher("missing/**")
            .expect("fixture pattern should compile");
        let filters = [filter];
        let selected = selected_nodes(&graph(), &filters);

        let violation =
            empty_selection_violation(&selected, &filters, true, &CheckOptions::default())
                .expect("strict empty selection should produce a violation");
        let data = violation
            .as_empty_test()
            .expect("fixture should produce empty-test data");

        assert_eq!(data.subject, "files");
        assert_eq!(data.selectors[0].pattern().source(), "missing/**");
        assert!(data.is_negated);
    }

    #[test]
    fn option_allows_empty_but_never_reclassifies_a_selected_isolated_file() {
        let missing = RegexFactory::default()
            .path_matcher("missing/**")
            .expect("fixture pattern should compile");
        let selected = selected_nodes(&graph(), &[]);
        let allowed = CheckOptions::new().with_allow_empty_tests(true);

        assert!(empty_selection_violation(&[], &[missing], false, &allowed).is_none());
        assert!(
            empty_selection_violation(&selected, &[], false, &CheckOptions::default()).is_none()
        );
    }
}
