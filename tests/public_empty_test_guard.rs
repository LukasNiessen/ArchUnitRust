use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use archunit::{CheckOptions, Checkable, FileConditionBuilder, ViolationKind, project_files_in};

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/extraction_workspace")
}

fn missing_scope() -> FileConditionBuilder {
    project_files_in(fixture_root())
        .in_folder("missing/**")
        .with_name("*_missing.rs")
}

fn current_terminals(predicate_calls: Arc<AtomicUsize>) -> Vec<Box<dyn Checkable>> {
    let custom_calls = Arc::clone(&predicate_calls);

    vec![
        Box::new(missing_scope().should().have_no_cycles()),
        Box::new(missing_scope().should_not().have_name("*.rs")),
        Box::new(
            missing_scope()
                .should()
                .depend_on_files()
                .in_folder("crates/app/source"),
        ),
        Box::new(
            missing_scope()
                .should_not()
                .depend_on_external_modules()
                .matching("std"),
        ),
        Box::new(missing_scope().should().adhere_to(
            move |_| {
                custom_calls.fetch_add(1, Ordering::Relaxed);
                true
            },
            "satisfy the fixture predicate",
        )),
    ]
}

#[test]
fn every_current_terminal_reports_one_mood_aware_empty_test_by_default() {
    let predicate_calls = Arc::new(AtomicUsize::new(0));
    let expected_moods = [false, true, false, true, false];

    for (rule, expected_negated) in current_terminals(Arc::clone(&predicate_calls))
        .iter()
        .zip(expected_moods)
    {
        let violations = rule
            .check()
            .expect("fixture empty-scope rule should execute");
        let data = violations[0]
            .as_empty_test()
            .expect("empty scope should produce typed empty-test data");

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].kind(), ViolationKind::EmptyTest);
        assert_eq!(data.subject, "files");
        assert_eq!(data.selectors.len(), 2);
        assert_eq!(data.selectors[0].pattern().source(), "missing/**");
        assert_eq!(data.selectors[1].pattern().source(), "*_missing.rs");
        assert_eq!(data.is_negated, expected_negated);
    }

    assert_eq!(predicate_calls.load(Ordering::Relaxed), 0);
}

#[test]
fn explicit_check_option_allows_empty_scopes_for_every_current_terminal() {
    let predicate_calls = Arc::new(AtomicUsize::new(0));
    let options = CheckOptions::new().with_allow_empty_tests(true);

    for rule in current_terminals(Arc::clone(&predicate_calls)) {
        assert!(
            rule.check_with(&options)
                .expect("allowed empty-scope rule should execute")
                .is_empty()
        );
    }

    assert_eq!(predicate_calls.load(Ordering::Relaxed), 0);
}

#[test]
fn selected_file_without_dependency_edges_is_not_an_empty_test() {
    let rule = project_files_in(fixture_root())
        .in_file("crates/app/cmd/server.rs")
        .should()
        .have_no_cycles();

    let violations = rule
        .check()
        .expect("isolated-file cycle rule should execute");

    assert!(violations.is_empty());
}
