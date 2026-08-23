use std::path::PathBuf;

use archunit::{
    Checkable, DependOnExternalModuleCondition, DependOnExternalModuleConditionBuilder, Violation,
    project_files_in,
};

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/extraction_workspace")
}

#[test]
fn positive_mood_is_an_allowlist_for_external_crates() {
    let rule: DependOnExternalModuleCondition = project_files_in(fixture_root())
        .in_file("crates/app/source/api.rs")
        .should()
        .depend_on_external_modules()
        .matching("std");

    let violations = rule
        .check()
        .expect("fixture external-dependency rule should execute");
    let data = violations
        .first()
        .and_then(Violation::as_external_module_dependency)
        .expect("the non-allowlisted core dependency should be reported");

    assert_eq!(violations.len(), 1);
    assert_eq!(data.dependency.source_label, "crates/app/source/api.rs");
    assert_eq!(data.dependency.target_label, "core");
    assert!(!data.is_negated);
    assert!(
        data.dependency
            .cumulated_edges
            .iter()
            .all(|edge| edge.external)
    );
}

#[test]
fn negated_mood_is_a_denylist_for_external_crates() {
    let rule = project_files_in(fixture_root())
        .in_file("crates/app/source/api.rs")
        .should_not()
        .depend_on_external_modules()
        .matching("std");

    let violations = rule
        .check()
        .expect("fixture external-dependency rule should execute");
    let data = violations
        .first()
        .and_then(Violation::as_external_module_dependency)
        .expect("the forbidden std dependency should be reported");

    assert_eq!(violations.len(), 1);
    assert_eq!(data.dependency.target_label, "std");
    assert!(data.is_negated);
}

#[test]
fn repeated_matching_selectors_are_branchable_or_alternatives() {
    let stage: DependOnExternalModuleConditionBuilder = project_files_in(fixture_root())
        .in_file("crates/app/source/api.rs")
        .should_not()
        .depend_on_external_modules();
    let std_only = stage.clone().matching("std");
    let std_or_core = std_only.clone().matching("core");

    let std_targets = std_only
        .check()
        .expect("single-module rule should execute")
        .into_iter()
        .filter_map(|violation| {
            violation
                .as_external_module_dependency()
                .map(|data| data.dependency.target_label.clone())
        })
        .collect::<Vec<_>>();
    let combined_targets = std_or_core
        .check()
        .expect("combined-module rule should execute")
        .into_iter()
        .filter_map(|violation| {
            violation
                .as_external_module_dependency()
                .map(|data| data.dependency.target_label.clone())
        })
        .collect::<Vec<_>>();

    assert_eq!(std_only.module_filters().len(), 1);
    assert_eq!(std_or_core.module_filters().len(), 2);
    assert_eq!(std_targets, ["std"]);
    assert_eq!(combined_targets, ["core", "std"]);
}

#[test]
fn invalid_module_selector_is_a_user_error_before_project_location() {
    let rule = project_files_in("definitely/missing/project")
        .should_not()
        .depend_on_external_modules()
        .matching("[module");

    let error = rule
        .check()
        .expect_err("invalid module selector should prevent project discovery");

    assert!(error.as_user().is_some());
    assert!(error.to_string().contains("external module target"));
    assert!(error.to_string().contains("[module"));
}

#[test]
fn invalid_subject_selector_precedes_an_invalid_module_selector() {
    let rule = project_files_in("definitely/missing/project")
        .in_path("src/[subject")
        .should()
        .depend_on_external_modules()
        .matching("[module");

    let retained = rule
        .selector_error()
        .expect("the first invalid selector should remain inspectable");
    let error = rule
        .check()
        .expect_err("invalid subject selector should prevent project discovery");

    assert_eq!(retained.pattern(), "src/[subject");
    assert!(error.to_string().contains("file scope"));
    assert!(error.to_string().contains("src/[subject"));
    assert!(!error.to_string().contains("[module"));
}
