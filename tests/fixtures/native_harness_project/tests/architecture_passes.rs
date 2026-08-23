use archunit::{assert_passes, project_files_in};

#[test]
fn source_tree_has_expected_library_name() {
    assert_passes!(
        project_files_in(env!("CARGO_MANIFEST_DIR"))
            .in_file("src/lib.rs")
            .should()
            .have_name("lib.rs")
    );
}
