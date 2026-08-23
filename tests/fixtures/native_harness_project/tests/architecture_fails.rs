use archunit::{assert_passes, project_files_in};

#[test]
#[ignore = "invoked explicitly by the native harness integration test"]
fn shared_failure_message_reaches_builtin_harness() {
    assert_passes!(
        project_files_in(env!("CARGO_MANIFEST_DIR"))
            .in_file("src/lib.rs")
            .should()
            .have_name("main.rs")
    );
}
