use std::path::PathBuf;

use archunit::{
    ArchUnitError, CheckOptions, Checkable, Edge, Graph, ImportKind, MappedEdge,
    SliceDependencyRule, ViolationKind, assert_passes, project_slices_in, slice_by_file_suffix,
    slice_by_pattern, slice_by_regex, slice_identity,
};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

#[test]
fn public_slice_projections_are_reusable_without_project_io() {
    let pattern = slice_by_pattern("src/(**)/").expect("pattern projection should compile");
    let regex = slice_by_regex(r"\Asrc/([^/]+)/").expect("regex projection should compile");
    let suffix = slice_by_file_suffix([("_service", "services"), ("_controller", "controllers")])
        .expect("suffix projection should compile");
    let identity = slice_identity();
    let dependency = Edge::new(
        "src/api/order_controller.rs",
        "src/application/order_service.rs",
        false,
        [ImportKind::Use],
    );

    assert_eq!(
        pattern.map_edge(&dependency),
        Some(MappedEdge::new("api", "application"))
    );
    assert_eq!(
        regex.map_edge(&dependency),
        Some(MappedEdge::new("api", "application"))
    );
    assert_eq!(
        suffix.map_edge(&dependency),
        Some(MappedEdge::new("controllers", "services"))
    );
    assert_eq!(
        identity.map_edge(&dependency),
        Some(MappedEdge::new(
            "src/api/order_controller.rs",
            "src/application/order_service.rs"
        ))
    );

    let projected = pattern.project(&Graph::from_edges([dependency]));
    assert_eq!(projected.len(), 1);
    assert_eq!(projected[0].source_label, "api");
    assert_eq!(projected[0].target_label, "application");
}

#[test]
fn forbidden_internal_slice_dependency_retains_exact_rust_evidence() {
    let rule = project_slices_in(fixture("layered_project"))
        .defined_by("src/(**)/")
        .should_not()
        .contain_dependency("api", "database");
    let violations = rule
        .check()
        .expect("the layered Cargo fixture should be analyzable");

    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].kind(), ViolationKind::SliceDependency);
    let violation = violations[0]
        .as_slice_dependency()
        .expect("the forbidden pair should produce slice dependency data");
    assert_eq!(violation.source_slice, "api");
    assert_eq!(violation.target_slice, "database");
    assert_eq!(violation.rule, SliceDependencyRule::ContainDependency);
    assert!(violation.is_negated);
    assert_eq!(violation.dependency.cumulated_edges.len(), 1);
    assert_eq!(
        violation.dependency.cumulated_edges[0].source,
        "src/api/mod.rs"
    );
    assert_eq!(
        violation.dependency.cumulated_edges[0].target,
        "src/database/repository.rs"
    );
}

#[test]
fn absent_forbidden_pair_passes_through_the_native_harness() {
    let rule = project_slices_in(fixture("layered_project"))
        .defined_by_regex(r"\Asrc/([^/]+)/")
        .should_not()
        .contain_dependency("database", "api");

    assert_passes!(rule);
}

#[test]
fn slices_can_forbid_dependencies_on_external_cargo_modules() {
    let violations = project_slices_in(fixture("extraction_workspace"))
        .defined_by("crates/(**)/")
        .should_not()
        .contain_dependency("app", "tokio")
        .check()
        .expect("the extraction workspace should be analyzable");

    assert_eq!(violations.len(), 1);
    let violation = violations[0]
        .as_slice_dependency()
        .expect("the external pair should produce slice dependency data");
    assert_eq!(violation.target_slice, "tokio");
    assert!(
        violation
            .dependency
            .cumulated_edges
            .iter()
            .all(|edge| edge.external)
    );
}

#[test]
fn missing_slice_selection_uses_the_universal_empty_test_guard() {
    let rule = project_slices_in(fixture("layered_project"))
        .defined_by("missing/(**)/")
        .should_not()
        .contain_dependency("api", "database");

    let strict = rule
        .check()
        .expect("the empty slice definition should still reach a verdict");
    let allowed = rule
        .check_with(&CheckOptions::new().with_allow_empty_tests(true))
        .expect("an explicitly allowed empty slice definition should pass");

    assert_eq!(strict.len(), 1);
    assert_eq!(strict[0].kind(), ViolationKind::EmptyTest);
    assert!(
        strict[0]
            .as_empty_test()
            .expect("strict result should carry empty-test data")
            .is_negated
    );
    assert!(allowed.is_empty());
}

#[test]
fn invalid_projection_and_slice_names_are_user_errors_before_project_discovery() {
    let cases = [
        project_slices_in("definitely/missing")
            .defined_by("src/**")
            .should_not()
            .contain_dependency("api", "database"),
        project_slices_in("definitely/missing")
            .defined_by_regex(r"src/.*")
            .should_not()
            .contain_dependency("api", "database"),
        project_slices_in("definitely/missing")
            .should_not()
            .contain_dependency("", "database"),
    ];

    for rule in cases {
        assert!(matches!(rule.check(), Err(ArchUnitError::User(_))));
    }
}
