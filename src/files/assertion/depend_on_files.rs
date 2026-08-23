use crate::{Filter, ProjectedEdge, Violation};

use super::FileDependencyViolation;

/// Judges internal dependencies from selected subject files to an object-file allowlist or denylist.
///
/// A positive rule treats the object filters as an allowlist and reports subject dependencies whose
/// targets do not match every object filter. A negated rule treats them as a denylist and reports
/// dependencies whose targets do match every filter. Subject filters and object filters each use
/// AND semantics.
#[must_use]
pub fn gather_file_dependency_violations(
    edges: &[ProjectedEdge],
    subject_filters: &[Filter],
    object_filters: &[Filter],
    is_negated: bool,
) -> Vec<Violation> {
    edges
        .iter()
        .filter(|edge| matches_all(&edge.source_label, subject_filters))
        .filter(|edge| {
            let target_matches =
                !object_filters.is_empty() && matches_all(&edge.target_label, object_filters);
            target_matches == is_negated
        })
        .cloned()
        .map(|edge| FileDependencyViolation::new(edge, is_negated))
        .map(Violation::from)
        .collect()
}

fn matches_all(identifier: &str, filters: &[Filter]) -> bool {
    filters.iter().all(|filter| filter.matches(identifier))
}

#[cfg(test)]
mod tests {
    use crate::{Edge, ImportKind, PatternTarget, ProjectedEdge, RegexFactory, ViolationKind};

    use super::gather_file_dependency_violations;

    fn dependency(source: &str, target: &str) -> ProjectedEdge {
        ProjectedEdge::new(
            source,
            target,
            [Edge::new(source, target, false, [ImportKind::Use])],
        )
    }

    fn edges() -> Vec<ProjectedEdge> {
        vec![
            dependency("src/controller.rs", "src/service/order_service.rs"),
            dependency("src/controller.rs", "src/shared/logger.rs"),
            dependency("src/other.rs", "src/service/order_service.rs"),
        ]
    }

    fn subject_filters() -> Vec<crate::Filter> {
        vec![
            RegexFactory::default()
                .filename_matcher("controller.rs")
                .expect("fixture pattern should compile"),
        ]
    }

    fn service_filters() -> Vec<crate::Filter> {
        vec![
            RegexFactory::default()
                .folder_matcher("src/service")
                .expect("fixture pattern should compile"),
            RegexFactory::default()
                .filename_matcher("*_service.rs")
                .expect("fixture pattern should compile"),
        ]
    }

    #[test]
    fn positive_mood_reports_subject_dependencies_outside_the_allowlist() {
        let violations = gather_file_dependency_violations(
            &edges(),
            &subject_filters(),
            &service_filters(),
            false,
        );
        let data = violations
            .iter()
            .filter_map(crate::Violation::as_file_dependency)
            .collect::<Vec<_>>();

        assert_eq!(data.len(), 1);
        assert_eq!(data[0].dependency.source_label, "src/controller.rs");
        assert_eq!(data[0].dependency.target_label, "src/shared/logger.rs");
        assert!(!data[0].is_negated);
        assert_eq!(violations[0].kind(), ViolationKind::FileDependency);
    }

    #[test]
    fn negated_mood_reports_only_subject_dependencies_matching_the_denylist() {
        let violations = gather_file_dependency_violations(
            &edges(),
            &subject_filters(),
            &service_filters(),
            true,
        );
        let data = violations
            .iter()
            .filter_map(crate::Violation::as_file_dependency)
            .collect::<Vec<_>>();

        assert_eq!(data.len(), 1);
        assert_eq!(
            data[0].dependency.target_label,
            "src/service/order_service.rs"
        );
        assert!(data[0].is_negated);
    }

    #[test]
    fn object_filters_are_combined_with_and_semantics() {
        let filters = service_filters();
        let mut dependencies = edges();
        dependencies.push(dependency(
            "src/controller.rs",
            "src/service/order_repository.rs",
        ));
        let only_folder = gather_file_dependency_violations(
            &dependencies,
            &subject_filters(),
            &filters[..1],
            true,
        );
        let folder_and_name =
            gather_file_dependency_violations(&dependencies, &subject_filters(), &filters, true);

        assert_eq!(only_folder.len(), 2);
        assert_eq!(folder_and_name.len(), 1);
        let dependency = folder_and_name[0]
            .as_file_dependency()
            .expect("fixture should produce file-dependency data");
        assert_eq!(
            dependency.dependency.target_label,
            "src/service/order_service.rs"
        );
        assert_eq!(filters[0].target(), PatternTarget::PathWithoutFilename);
        assert_eq!(filters[1].target(), PatternTarget::Filename);
    }

    #[test]
    fn empty_subject_filters_select_all_sources_and_empty_objects_match_none() {
        let positive = gather_file_dependency_violations(&edges(), &[], &[], false);
        let negated = gather_file_dependency_violations(&edges(), &[], &[], true);

        assert_eq!(positive.len(), 3);
        assert!(negated.is_empty());
    }
}
