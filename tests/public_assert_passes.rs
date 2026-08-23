use std::{any::Any, panic};

use archunit::{CheckOptions, Checkable, assert_passes, project_files_in};

fn fixture_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/extraction_workspace")
}

fn panic_message(payload: &(dyn Any + Send)) -> &str {
    payload.downcast_ref::<String>().map_or_else(
        || {
            payload
                .downcast_ref::<&'static str>()
                .copied()
                .unwrap_or("non-string panic payload")
        },
        String::as_str,
    )
}

#[test]
fn default_macro_accepts_an_inline_rule_and_a_trait_object() {
    assert_passes!(
        project_files_in(fixture_root())
            .in_file("crates/app/source/api.rs")
            .should()
            .have_name("api.rs")
    );

    let reusable = project_files_in(fixture_root())
        .in_file("crates/app/source/api/model.rs")
        .should()
        .have_name("model.rs");
    let erased: &dyn Checkable = &reusable;

    assert_passes!(erased);
    assert_passes!(reusable);
}

#[test]
fn options_form_forwards_check_options_and_borrows_both_values() {
    let rule = project_files_in(fixture_root())
        .in_folder("intentionally/missing/**")
        .should()
        .have_no_cycles();
    let options = CheckOptions::new().with_allow_empty_tests(true);

    assert_passes!(&rule, &options);
    assert!(options.allows_empty_tests());
    assert!(rule.filters().len() == 1);
}

#[test]
fn architecture_violations_raise_one_shared_numbered_assertion_message() {
    let rule = project_files_in(fixture_root())
        .in_file("crates/app/source/api.rs")
        .should()
        .have_name("model.rs");

    let panic = panic::catch_unwind(|| assert_passes!(rule))
        .expect_err("violating architecture rule should fail the assertion");
    let message = panic_message(panic.as_ref());

    assert!(message.contains("Found 1 architecture violation:"));
    assert!(message.contains("1. File pattern violation"));
    assert!(message.contains(
        "File 'crates/app/source/api.rs' does not match the required filename pattern \"model.rs\"."
    ));
}

#[test]
fn check_errors_raise_an_assertion_failure_with_the_original_context() {
    let rule = project_files_in("definitely/missing/project")
        .should()
        .have_name("*.rs");

    let panic = panic::catch_unwind(|| assert_passes!(rule))
        .expect_err("project discovery error should fail the assertion");
    let message = panic_message(panic.as_ref());

    assert!(message.contains("Architecture rule is invalid: archunit:"));
    assert!(message.contains("definitely/missing/project"));
}
