use std::path::PathBuf;

use archunit::{
    Checkable, CycleFreeFileCondition, MatchPatternFileCondition,
    NegatedMatchPatternFileConditionBuilder, PatternTarget,
    PositiveMatchPatternFileConditionBuilder, SourceOptions, ViolationKind, extract_graph, files,
    files_in, locate_project_from, project_files, project_files_in, project_to_nodes,
};

fn matches_all(identifier: &str, filters: &[archunit::Filter]) -> bool {
    filters.iter().all(|filter| filter.matches(identifier))
}

#[test]
fn file_entry_points_build_branchable_and_scopes() {
    let base = project_files().in_path("src/**");
    let services = base.clone().with_name("*_service.rs");
    let repositories = base.clone().with_name("*_repository.rs");

    assert!(base.project_locator().path().is_none());
    assert_eq!(base.filters().len(), 1);
    assert!(matches_all("src/order_service.rs", services.filters()));
    assert!(!matches_all("src/order_repository.rs", services.filters()));
    assert!(matches_all(
        "src/order_repository.rs",
        repositories.filters()
    ));
    assert!(files().filters().is_empty());
}

#[test]
fn explicit_file_scope_selects_identifiers_from_an_extracted_cargo_project() {
    let fixture =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/extraction_workspace");
    let scope = project_files_in(fixture.clone())
        .in_folder("crates/app/source/api/**")
        .with_name("model.rs");
    let project = locate_project_from(scope.project_locator())
        .expect("builder locator should discover the fixture workspace");
    let extraction = extract_graph(&project, SourceOptions::default())
        .expect("fixture workspace should extract");

    let selected = project_to_nodes(extraction.graph())
        .into_iter()
        .filter(|node| matches_all(&node.label, scope.filters()))
        .map(|node| node.label)
        .collect::<Vec<_>>();

    assert_eq!(selected, ["crates/app/source/api/model.rs"]);
    assert_eq!(
        files_in(fixture).project_locator().path(),
        scope.project_locator().path()
    );
}

#[test]
fn invalid_selectors_remain_diagnostic_without_interrupting_the_sentence() {
    let scope = project_files().in_path("crates/[app").with_name("*.rs");

    let error = scope
        .selector_error()
        .expect("invalid selector should be retained for the future terminal");
    assert_eq!(error.pattern(), "crates/[app");
    assert_eq!(scope.filters().len(), 0);
}

#[test]
fn should_and_should_not_are_distinct_thin_moods_over_shared_state() {
    let base = project_files_in("examples/layered")
        .in_folder("src/**")
        .with_name("*.rs");
    let positive: PositiveMatchPatternFileConditionBuilder = base.clone().should();
    let negative: NegatedMatchPatternFileConditionBuilder = base.should_not();

    assert!(!positive.is_negated());
    assert!(negative.is_negated());
    assert_eq!(positive.filters().len(), 2);
    assert_eq!(negative.filters().len(), 2);
    assert_eq!(
        positive.project_locator().path(),
        negative.project_locator().path()
    );
    assert!(positive.selector_error().is_none());
    assert!(negative.selector_error().is_none());
}

#[test]
fn have_no_cycles_reports_a_readable_path_from_the_rust_fixture() {
    let fixture =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/extraction_workspace");
    let rule: CycleFreeFileCondition = project_files_in(fixture).should().have_no_cycles();

    let violations = rule.check().expect("fixture cycle rule should execute");
    let cycle = violations
        .iter()
        .filter(|violation| violation.kind() == ViolationKind::Cycle)
        .filter_map(archunit::Violation::as_cycle)
        .find(|violation| {
            violation.path
                == [
                    "crates/app/source/api.rs",
                    "crates/app/source/api/model.rs",
                    "crates/app/source/api.rs",
                ]
        })
        .expect("the fixture's parent/model module cycle should be reported");

    assert_eq!(
        cycle.path.join(" -> "),
        concat!(
            "crates/app/source/api.rs -> ",
            "crates/app/source/api/model.rs -> ",
            "crates/app/source/api.rs"
        )
    );
    assert!(
        cycle
            .cycle
            .iter()
            .all(|edge| !edge.cumulated_edges.is_empty())
    );
}

#[test]
fn have_no_cycles_checks_only_the_selected_files() {
    let fixture =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/extraction_workspace");
    let rule = project_files_in(fixture)
        .in_file("crates/app/source/shared.rs")
        .should()
        .have_no_cycles();

    assert!(
        rule.check()
            .expect("selected acyclic file should be checked")
            .is_empty()
    );
}

#[test]
fn cycle_terminal_reports_selector_errors_before_project_location() {
    let rule = project_files_in("definitely/missing/project")
        .in_path("src/[api")
        .should()
        .have_no_cycles();

    let error = rule
        .check()
        .expect_err("invalid selector should prevent the rule from running");

    assert!(error.as_user().is_some());
    assert!(error.to_string().contains("invalid selector"));
    assert!(error.to_string().contains("src/[api"));
}

#[test]
fn name_and_location_predicates_report_both_moods_on_a_real_rust_project() {
    let fixture =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/extraction_workspace");
    let api_file = "crates/app/source/api.rs";
    let rules: [(MatchPatternFileCondition, PatternTarget, bool); 6] = [
        (
            project_files_in(fixture.clone())
                .in_file(api_file)
                .should()
                .have_name("model.rs"),
            PatternTarget::Filename,
            false,
        ),
        (
            project_files_in(fixture.clone())
                .in_file(api_file)
                .should_not()
                .have_name("api.rs"),
            PatternTarget::Filename,
            true,
        ),
        (
            project_files_in(fixture.clone())
                .in_file(api_file)
                .should()
                .be_in_folder("crates/app/source/api"),
            PatternTarget::PathWithoutFilename,
            false,
        ),
        (
            project_files_in(fixture.clone())
                .in_file(api_file)
                .should_not()
                .be_in_folder("crates/app/source"),
            PatternTarget::PathWithoutFilename,
            true,
        ),
        (
            project_files_in(fixture.clone())
                .in_file(api_file)
                .should()
                .be_in_path("crates/app/source/api/model.rs"),
            PatternTarget::Path,
            false,
        ),
        (
            project_files_in(fixture)
                .in_file(api_file)
                .should_not()
                .be_in_path(api_file),
            PatternTarget::Path,
            true,
        ),
    ];

    for (rule, expected_target, expected_mood) in rules {
        let violations = rule
            .check()
            .expect("fixture file-pattern rule should execute");
        let violation = violations
            .first()
            .and_then(archunit::Violation::as_file_pattern)
            .expect("the selected file should disagree with the predicate");

        assert_eq!(violations.len(), 1);
        assert_eq!(violation.projected_node.label, api_file);
        assert_eq!(violation.check_filter.target(), expected_target);
        assert_eq!(violation.is_negated, expected_mood);
    }
}

#[test]
fn name_and_location_predicates_pass_when_every_selected_file_agrees() {
    let fixture =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/extraction_workspace");
    let api_file = "crates/app/source/api.rs";
    let rules = [
        project_files_in(fixture.clone())
            .in_file(api_file)
            .should()
            .have_name("api.rs"),
        project_files_in(fixture.clone())
            .in_file(api_file)
            .should()
            .be_in_folder("crates/app/source"),
        project_files_in(fixture.clone())
            .in_file(api_file)
            .should()
            .be_in_path(api_file),
        project_files_in(fixture)
            .in_file(api_file)
            .should_not()
            .have_name("model.rs"),
    ];

    for rule in rules {
        assert!(
            rule.check()
                .expect("fixture file-pattern rule should execute")
                .is_empty()
        );
    }
}

#[test]
fn predicate_errors_are_user_errors_before_project_location() {
    let rule = project_files_in("definitely/missing/project")
        .should()
        .have_name("[broken");

    let error = rule
        .check()
        .expect_err("invalid predicate should prevent project discovery");

    assert!(error.as_user().is_some());
    assert!(error.to_string().contains("file predicate"));
    assert!(error.to_string().contains("[broken"));
}

#[test]
fn the_first_invalid_pattern_follows_sentence_order() {
    let rule = project_files_in("definitely/missing/project")
        .in_path("src/[scope")
        .should()
        .be_in_path("src/[predicate");

    let retained = rule
        .selector_error()
        .expect("the first invalid pattern should remain inspectable");
    let error = rule
        .check()
        .expect_err("invalid scope should prevent project discovery");

    assert_eq!(retained.pattern(), "src/[scope");
    assert!(error.to_string().contains("file scope"));
    assert!(error.to_string().contains("src/[scope"));
    assert!(!error.to_string().contains("src/[predicate"));
}
