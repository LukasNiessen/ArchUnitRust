use std::path::PathBuf;

use archunit::{
    Checkable, DependOnFileCondition, DependOnFileConditionBuilder, PatternTarget, Violation,
    project_files_in,
};

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/extraction_workspace")
}

#[test]
fn positive_mood_is_an_allowlist_for_subject_outgoing_dependencies() {
    let rule: DependOnFileCondition = project_files_in(fixture_root())
        .in_file("crates/app/source/api.rs")
        .should()
        .depend_on_files()
        .in_path("crates/app/source/api/model.rs");

    let violations = rule
        .check()
        .expect("fixture file-dependency rule should execute");
    let targets = violations
        .iter()
        .filter_map(Violation::as_file_dependency)
        .map(|violation| violation.dependency.target_label.as_str())
        .collect::<Vec<_>>();

    assert_eq!(targets, ["crates/app/source/shared.rs"]);
    let violation = violations[0]
        .as_file_dependency()
        .expect("fixture should produce file-dependency data");
    assert_eq!(
        violation.dependency.source_label,
        "crates/app/source/api.rs"
    );
    assert!(!violation.is_negated);
    assert!(!violation.dependency.cumulated_edges.is_empty());
}

#[test]
fn negated_mood_is_a_denylist_and_chained_object_selectors_use_and_semantics() {
    let rule = project_files_in(fixture_root())
        .in_file("crates/app/source/api.rs")
        .should_not()
        .depend_on_files()
        .in_folder("crates/app/source/api")
        .with_name("model.rs");

    let violations = rule
        .check()
        .expect("fixture file-dependency rule should execute");
    let data = violations
        .first()
        .and_then(Violation::as_file_dependency)
        .expect("the forbidden model dependency should be reported");

    assert_eq!(violations.len(), 1);
    assert_eq!(
        data.dependency.target_label,
        "crates/app/source/api/model.rs"
    );
    assert!(data.is_negated);
    assert_eq!(rule.object_filters().len(), 2);
    assert_eq!(
        rule.object_filters()[0].target(),
        PatternTarget::PathWithoutFilename
    );
    assert_eq!(rule.object_filters()[1].target(), PatternTarget::Filename);
}

#[test]
fn object_stage_and_terminal_are_branchable_values_in_both_moods() {
    let positive: DependOnFileConditionBuilder =
        project_files_in(fixture_root()).should().depend_on_files();
    let negative: DependOnFileConditionBuilder = project_files_in(fixture_root())
        .should_not()
        .depend_on_files();
    let folder = negative.clone().in_folder("crates/app/source/api");
    let named = folder.clone().with_name("model.rs");
    let path = negative.in_path("crates/app/source/api/**");

    assert!(!positive.is_negated());
    assert!(folder.is_negated());
    assert_eq!(folder.object_filters().len(), 1);
    assert_eq!(named.object_filters().len(), 2);
    assert_eq!(path.object_filters().len(), 1);
}

#[test]
fn invalid_object_selector_is_a_user_error_before_project_location() {
    let rule = project_files_in("definitely/missing/project")
        .should_not()
        .depend_on_files()
        .in_path("src/[object");

    let error = rule
        .check()
        .expect_err("invalid object selector should prevent project discovery");

    assert!(error.as_user().is_some());
    assert!(error.to_string().contains("dependency target"));
    assert!(error.to_string().contains("src/[object"));
}

#[test]
fn invalid_subject_selector_precedes_an_invalid_object_selector() {
    let rule = project_files_in("definitely/missing/project")
        .in_path("src/[subject")
        .should()
        .depend_on_files()
        .in_path("src/[object");

    let retained = rule
        .selector_error()
        .expect("the first invalid selector should remain inspectable");
    let error = rule
        .check()
        .expect_err("invalid subject selector should prevent project discovery");

    assert_eq!(retained.pattern(), "src/[subject");
    assert!(error.to_string().contains("file scope"));
    assert!(error.to_string().contains("src/[subject"));
    assert!(!error.to_string().contains("src/[object"));
}
