use std::path::PathBuf;

use archunit::{
    SourceOptions, extract_graph, files, files_in, locate_project_from, project_files,
    project_files_in, project_to_nodes,
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
