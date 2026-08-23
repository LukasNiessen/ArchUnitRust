use crate::{Filter, ProjectedEdge, Violation};

use super::ExternalModuleDependencyViolation;

/// Judges external dependencies from selected files against a crate allowlist or denylist.
///
/// Subject filters use AND semantics. Repeated external-module filters use OR semantics: matching
/// any configured crate pattern places the target in the allowlist or denylist.
#[must_use]
pub fn gather_external_module_dependency_violations(
    edges: &[ProjectedEdge],
    subject_filters: &[Filter],
    module_filters: &[Filter],
    is_negated: bool,
) -> Vec<Violation> {
    edges
        .iter()
        .filter(|edge| matches_all(&edge.source_label, subject_filters))
        .filter(|edge| matches_any(&edge.target_label, module_filters) == is_negated)
        .cloned()
        .map(|edge| ExternalModuleDependencyViolation::new(edge, is_negated))
        .map(Violation::from)
        .collect()
}

fn matches_all(identifier: &str, filters: &[Filter]) -> bool {
    filters.iter().all(|filter| filter.matches(identifier))
}

fn matches_any(identifier: &str, filters: &[Filter]) -> bool {
    filters.iter().any(|filter| filter.matches(identifier))
}

#[cfg(test)]
mod tests {
    use crate::{Edge, ImportKind, ProjectedEdge, RegexFactory, ViolationKind};

    use super::gather_external_module_dependency_violations;

    fn dependency(source: &str, target: &str) -> ProjectedEdge {
        ProjectedEdge::new(
            source,
            target,
            [Edge::new(source, target, true, [ImportKind::PathReference])],
        )
    }

    fn edges() -> Vec<ProjectedEdge> {
        vec![
            dependency("src/api.rs", "std"),
            dependency("src/api.rs", "tokio"),
            dependency("src/api.rs", "wire_format"),
            dependency("src/other.rs", "tokio"),
        ]
    }

    fn subject_filters() -> Vec<crate::Filter> {
        vec![
            RegexFactory::default()
                .filename_matcher("api.rs")
                .expect("fixture pattern should compile"),
        ]
    }

    fn allowed_modules() -> Vec<crate::Filter> {
        vec![
            RegexFactory::default()
                .path_matcher("std")
                .expect("fixture pattern should compile"),
            RegexFactory::default()
                .path_matcher("wire_*")
                .expect("fixture pattern should compile"),
        ]
    }

    #[test]
    fn positive_mood_reports_subject_dependencies_outside_the_allowlist() {
        let violations = gather_external_module_dependency_violations(
            &edges(),
            &subject_filters(),
            &allowed_modules(),
            false,
        );
        let data = violations
            .iter()
            .filter_map(crate::Violation::as_external_module_dependency)
            .collect::<Vec<_>>();

        assert_eq!(data.len(), 1);
        assert_eq!(data[0].dependency.source_label, "src/api.rs");
        assert_eq!(data[0].dependency.target_label, "tokio");
        assert!(!data[0].is_negated);
        assert_eq!(
            violations[0].kind(),
            ViolationKind::ExternalModuleDependency
        );
    }

    #[test]
    fn negated_mood_reports_every_subject_dependency_matching_any_denied_module() {
        let violations = gather_external_module_dependency_violations(
            &edges(),
            &subject_filters(),
            &allowed_modules(),
            true,
        );
        let targets = violations
            .iter()
            .filter_map(crate::Violation::as_external_module_dependency)
            .map(|violation| violation.dependency.target_label.as_str())
            .collect::<Vec<_>>();

        assert_eq!(targets, ["std", "wire_format"]);
        assert!(violations.iter().all(|violation| {
            violation
                .as_external_module_dependency()
                .is_some_and(|data| data.is_negated)
        }));
    }

    #[test]
    fn module_filters_are_combined_with_or_semantics() {
        let filters = allowed_modules();
        let std_only = gather_external_module_dependency_violations(
            &edges(),
            &subject_filters(),
            &filters[..1],
            true,
        );
        let std_or_wire = gather_external_module_dependency_violations(
            &edges(),
            &subject_filters(),
            &filters,
            true,
        );

        assert_eq!(std_only.len(), 1);
        assert_eq!(std_or_wire.len(), 2);
    }

    #[test]
    fn empty_subject_filters_select_all_sources_and_empty_modules_match_none() {
        let positive = gather_external_module_dependency_violations(&edges(), &[], &[], false);
        let negated = gather_external_module_dependency_violations(&edges(), &[], &[], true);

        assert_eq!(positive.len(), 4);
        assert!(negated.is_empty());
    }
}
