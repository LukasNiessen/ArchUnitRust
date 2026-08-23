use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use archunit::{Checkable, CustomFileCondition, FileInfo, ViolationKind, project_files_in};

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/extraction_workspace")
}

#[test]
fn custom_predicate_receives_complete_portable_file_info_once() {
    let observed = Arc::new(Mutex::new(Vec::<FileInfo>::new()));
    let observed_by_predicate = Arc::clone(&observed);
    let rule: CustomFileCondition = project_files_in(fixture_root())
        .in_file("crates/app/source/api/model.rs")
        .should()
        .adhere_to(
            move |file| {
                observed_by_predicate
                    .lock()
                    .expect("observation lock should remain available")
                    .push(file.clone());
                file.path == "crates/app/source/api/model.rs"
                    && file.name == "model"
                    && file.extension == ".rs"
                    && file.directory == "crates/app/source/api"
                    && file.content.contains("pub struct Model;")
                    && file.non_blank_line_count == 4
            },
            "expose stable source facts",
        );

    let violations = rule
        .check()
        .expect("fixture custom-predicate rule should execute");
    let observed = observed
        .lock()
        .expect("observation lock should remain available");

    assert!(violations.is_empty());
    assert_eq!(observed.len(), 1);
    assert!(observed[0].content.trim_end().ends_with('}'));
}

#[test]
fn positive_mood_reports_false_predicates_as_typed_data() {
    let rule = project_files_in(fixture_root())
        .in_file("crates/app/source/api.rs")
        .should()
        .adhere_to(|file| file.name == "model", "be named model");

    let violations = rule
        .check()
        .expect("fixture custom-predicate rule should execute");
    let data = violations[0]
        .as_custom_file()
        .expect("fixture should produce custom-file data");

    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].kind(), ViolationKind::CustomFile);
    assert_eq!(data.file_info.path, "crates/app/source/api.rs");
    assert_eq!(data.message, "be named model");
    assert!(!data.is_negated);
}

#[test]
fn negated_mood_reports_true_predicates_as_typed_data() {
    let rule = project_files_in(fixture_root())
        .in_file("crates/app/source/api.rs")
        .should_not()
        .adhere_to(
            |file| file.content.contains("pub struct Handler;"),
            "declare a handler",
        );

    let violations = rule
        .check()
        .expect("fixture custom-predicate rule should execute");
    let data = violations[0]
        .as_custom_file()
        .expect("fixture should produce custom-file data");

    assert_eq!(violations.len(), 1);
    assert_eq!(data.file_info.name, "api");
    assert_eq!(data.message, "declare a handler");
    assert!(data.is_negated);
}

#[test]
fn blank_message_is_a_user_error_before_project_location() {
    let rule = project_files_in("definitely/missing/project")
        .should()
        .adhere_to(|_| true, " \n ");

    let error = rule
        .check()
        .expect_err("blank message should prevent project discovery");

    assert!(error.as_user().is_some());
    assert_eq!(
        error.as_user().map(|error| error.message()),
        Some("the custom file predicate message must not be blank")
    );
}

#[test]
fn invalid_subject_selector_precedes_a_blank_message() {
    let rule = project_files_in("definitely/missing/project")
        .in_path("src/[scope")
        .should()
        .adhere_to(|_| true, "");

    let error = rule
        .check()
        .expect_err("invalid subject selector should be reported first");

    assert!(error.to_string().contains("file scope"));
    assert!(error.to_string().contains("src/[scope"));
    assert!(!error.to_string().contains("must not be blank"));
}
