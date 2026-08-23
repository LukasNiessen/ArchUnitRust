use std::path::PathBuf;

use archunit::{
    ArchUnitError, CheckOptions, Checkable, LayerDependencyRule, ViolationKind, assert_passes,
    layers_in, project_layers_in,
};

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/layered_project")
}

fn definitions() -> archunit::LayeredArchitecture {
    project_layers_in(fixture())
        .layer("api")
        .defined_by_folder("src/api")
        .layer("application")
        .defined_by("src/application/**")
        .layer("database")
        .defined_by_folder("src/database")
}

#[test]
fn complete_allowlist_policy_passes_as_a_native_architecture_test() {
    let rule = definitions()
        .where_layer("api")
        .may_only_depend_on_layers(&["application", "database"])
        .where_layer("application")
        .may_only_depend_on_layers(&["database"])
        .where_layer("database")
        .may_only_depend_on_layers(&[]);

    let _: &dyn Checkable = &rule;
    assert_passes!(rule);
}

#[test]
fn allowlist_violation_retains_layers_files_and_rust_evidence() {
    let violations = definitions()
        .where_layer("api")
        .may_only_depend_on_layers(&["application"])
        .check()
        .expect("the layered fixture should be analyzable");

    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].kind(), ViolationKind::LayerDependency);
    let violation = violations[0]
        .as_layer_dependency()
        .expect("the policy should produce layer data");
    assert_eq!(violation.source_layer, "api");
    assert_eq!(violation.target_layer, "database");
    assert_eq!(violation.rule, LayerDependencyRule::MayOnlyDependOnLayers);
    assert_eq!(violation.dependency.source_label, "src/api/mod.rs");
    assert_eq!(
        violation.dependency.target_label,
        "src/database/repository.rs"
    );
    assert!(!violation.dependency.cumulated_edges.is_empty());
}

#[test]
fn blocklist_has_priority_and_unassigned_support_edges_are_ignored() {
    let violations = definitions()
        .where_layer("api")
        .may_only_depend_on_layers(&["application", "database"])
        .where_layer("api")
        .may_not_depend_on_layers(&["database"])
        .check()
        .expect("the layered fixture should be analyzable");

    assert_eq!(violations.len(), 1);
    let violation = violations[0]
        .as_layer_dependency()
        .expect("the policy should produce layer data");
    assert_eq!(violation.rule, LayerDependencyRule::MayNotDependOnLayers);
    assert_eq!(violation.target_layer, "database");
}

#[test]
fn policy_source_layers_receive_the_universal_empty_test_guard() {
    let rule = layers_in(fixture())
        .layer("missing")
        .defined_by_folder("src/missing")
        .where_layer("missing")
        .may_only_depend_on_layers(&[]);

    let violations = rule
        .check()
        .expect("an empty layer is a rule verdict rather than a check error");
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].kind(), ViolationKind::EmptyTest);
    assert_eq!(
        violations[0]
            .as_empty_test()
            .expect("the guard should produce empty-test data")
            .subject,
        "layer 'missing'"
    );

    let options = CheckOptions::new().with_allow_empty_tests(true);
    assert!(
        rule.check_with(&options)
            .expect("an explicitly allowed empty layer should be checkable")
            .is_empty()
    );
}

#[test]
fn configuration_errors_are_classified_before_project_discovery() {
    let rule = project_layers_in("definitely/missing")
        .layer("api")
        .defined_by("src/api/**")
        .where_layer("api")
        .may_only_depend_on_layers(&["undefined"]);

    let error = rule
        .check()
        .expect_err("undefined target layer should prevent a verdict");
    assert!(matches!(error, ArchUnitError::User(_)));
    assert_eq!(
        error.to_string(),
        "archunit: undefined target layer: undefined"
    );
}
