use crate::{
    common::{Filter, ProjectedNode},
    violation::Violation,
};

use super::FilePatternViolation;

/// Judges selected files against one filename, folder, or path requirement.
///
/// Positive rules report non-matches and negated rules report matches. An empty selection remains
/// empty here; the terminal layer, rather than this pure predicate, is responsible for reporting
/// an empty test.
#[must_use]
pub fn gather_matching_file_violations(
    nodes: &[ProjectedNode],
    check_filter: &Filter,
    is_negated: bool,
) -> Vec<Violation> {
    nodes
        .iter()
        .filter(|node| check_filter.matches(&node.label) == is_negated)
        .cloned()
        .map(|node| FilePatternViolation::new(check_filter.clone(), node, is_negated))
        .map(Violation::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::{
        common::{Edge, Graph, PatternTarget, RegexFactory, project_to_nodes},
        violation::ViolationKind,
    };

    use super::gather_matching_file_violations;

    fn nodes() -> Vec<crate::common::ProjectedNode> {
        project_to_nodes(&Graph::from_edges([
            Edge::self_edge("src/orders/order_service.rs"),
            Edge::self_edge("src/orders/order_repository.rs"),
            Edge::self_edge("tests/orders/order_service_test.rs"),
        ]))
    }

    #[test]
    fn positive_mood_reports_each_selected_non_match_in_node_order() {
        let filter = RegexFactory::default()
            .filename_matcher("*_service.rs")
            .expect("fixture pattern should compile");

        let violations = gather_matching_file_violations(&nodes(), &filter, false);
        let labels = violations
            .iter()
            .filter_map(crate::violation::Violation::as_file_pattern)
            .map(|violation| violation.projected_node.label.as_str())
            .collect::<Vec<_>>();

        assert_eq!(
            labels,
            [
                "src/orders/order_repository.rs",
                "tests/orders/order_service_test.rs"
            ]
        );
        assert!(
            violations
                .iter()
                .all(|violation| violation.kind() == ViolationKind::FilePattern)
        );
    }

    #[test]
    fn negated_mood_reports_matches_without_inverting_the_filter() {
        let filter = RegexFactory::default()
            .folder_matcher("src/**")
            .expect("fixture pattern should compile");

        let violations = gather_matching_file_violations(&nodes(), &filter, true);
        let data = violations
            .iter()
            .filter_map(crate::violation::Violation::as_file_pattern)
            .collect::<Vec<_>>();

        assert_eq!(data.len(), 2);
        assert!(data.iter().all(|violation| violation.is_negated));
        assert!(data.iter().all(|violation| {
            violation.check_filter.target() == PatternTarget::PathWithoutFilename
        }));
        assert_eq!(
            data.iter()
                .map(|violation| violation.projected_node.label.as_str())
                .collect::<Vec<_>>(),
            [
                "src/orders/order_repository.rs",
                "src/orders/order_service.rs"
            ]
        );
    }

    #[test]
    fn empty_input_is_left_for_the_terminal_empty_test_guard() {
        let filter = RegexFactory::default()
            .path_matcher("src/**")
            .expect("fixture pattern should compile");

        assert!(gather_matching_file_violations(&[], &filter, false).is_empty());
        assert!(gather_matching_file_violations(&[], &filter, true).is_empty());
    }
}
