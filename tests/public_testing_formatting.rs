use std::path::PathBuf;

use archunit::{
    Checkable, ColorChoice, ColorUtils, ResultFactory, TestResultOptions, ViolationFactory,
    project_files_in,
};

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/extraction_workspace")
}

fn plain_options() -> TestResultOptions {
    TestResultOptions::new().with_color(ColorChoice::Never)
}

#[test]
fn real_rule_verdict_flows_through_both_shared_factories() {
    let rule = project_files_in(fixture_root())
        .in_file("crates/app/source/api.rs")
        .should()
        .have_name("model.rs");
    let violations = rule
        .check()
        .expect("fixture file-pattern rule should execute");

    let formatted = ViolationFactory::from_violation(&violations[0]);
    let result = ResultFactory::from_violations_with_options(&violations, &plain_options());

    assert_eq!(formatted.message, "File pattern violation");
    assert_eq!(
        formatted.details,
        "File 'crates/app/source/api.rs' does not match the required filename pattern \"model.rs\"."
    );
    assert!(!result.passed);
    assert_eq!(
        result.message,
        "Found 1 architecture violation:\n\n  1. File pattern violation\n     File 'crates/app/source/api.rs' does not match the required filename pattern \"model.rs\"."
    );
}

#[test]
fn passing_rule_and_inverted_expectation_have_unambiguous_flags() {
    let rule = project_files_in(fixture_root())
        .in_file("crates/app/source/api.rs")
        .should()
        .have_name("api.rs");
    let violations = rule
        .check()
        .expect("fixture file-pattern rule should execute");
    let ordinary = ResultFactory::from_violations_with_options(&violations, &plain_options());
    let inverted = ResultFactory::from_violations_with_options(
        &violations,
        &plain_options().with_expected_to_pass(false),
    );

    assert!(ordinary.passed);
    assert_eq!(ordinary.message, "No architecture violations found.");
    assert!(!inverted.passed);
    assert_eq!(
        inverted.message,
        "Expected architecture violations, but none were found."
    );
}

#[test]
fn explicit_color_choice_is_public_and_deterministic() {
    assert_eq!(
        ColorUtils::red_bold("failure", ColorChoice::Always),
        "\x1b[1;31mfailure\x1b[0m"
    );
    assert_eq!(
        ColorUtils::red_bold("failure", ColorChoice::Never),
        "failure"
    );
}
