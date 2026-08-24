use archunit::{assert_passes, project_files_in};

const PROJECT_ROOT: &str = env!("CARGO_MANIFEST_DIR");

#[test]
fn api_does_not_depend_on_persistence() {
    let rule = project_files_in(PROJECT_ROOT)
        .in_path("src/api.rs")
        .should_not()
        .depend_on_files()
        .in_path("src/persistence.rs");

    assert_passes!(rule);
}
