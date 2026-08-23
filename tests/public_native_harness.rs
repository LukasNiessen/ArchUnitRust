use std::path::PathBuf;
use std::process::{Command, Output};

fn run_fixture_test(test_target: &str, harness_arguments: &[&str]) -> Output {
    let repository = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let manifest = repository.join("tests/fixtures/native_harness_project/Cargo.toml");
    let target = repository.join("target/native-harness-fixture");
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());

    Command::new(cargo)
        .arg("test")
        .arg("--offline")
        .arg("--manifest-path")
        .arg(manifest)
        .arg("--test")
        .arg(test_target)
        .arg("--quiet")
        .args(harness_arguments)
        .env("CARGO_TARGET_DIR", target)
        .output()
        .expect("the downstream Cargo test process should start")
}

fn process_output(output: &Output) -> String {
    format!(
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn assert_contains(output: &str, expected: &str) {
    assert!(
        output.contains(expected),
        "expected downstream test output to contain {expected:?}\n{output}"
    );
}

#[test]
fn assertion_macro_integrates_with_the_builtin_test_harness() {
    let passing = run_fixture_test("architecture_passes", &[]);
    assert!(passing.status.success(), "{}", process_output(&passing));

    let failing = run_fixture_test("architecture_fails", &["--", "--ignored"]);
    let failing_output = process_output(&failing);

    assert!(!failing.status.success(), "{failing_output}");
    assert_contains(&failing_output, "Found 1 architecture violation:");
    assert_contains(&failing_output, "1. File pattern violation");
    assert_contains(
        &failing_output,
        "File 'src/lib.rs' does not match the required filename pattern \"main.rs\".",
    );
}
