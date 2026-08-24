use std::fs;

const WORKFLOW_PATH: &str = ".github/workflows/ci.yml";

#[test]
fn workflow_runs_for_every_push_pull_request_and_manual_dispatch() {
    let workflow = workflow();

    assert!(workflow.starts_with("name: CI\n\non:\n"));
    assert!(workflow.contains("  push:\n  pull_request:\n  workflow_dispatch:\n"));
    assert!(workflow.contains("permissions:\n  contents: read\n"));
    assert!(workflow.contains("concurrency:\n"));
    assert!(workflow.contains("  cancel-in-progress: true\n"));
}

#[test]
fn stable_quality_job_enforces_every_static_gate() {
    let workflow = workflow();
    let quality = job(&workflow, "quality", "test");

    assert!(quality.contains("rustup toolchain install stable"));
    assert!(quality.contains("--component rustfmt,clippy"));
    assert!(quality.contains("cargo +stable fmt --all -- --check"));
    assert!(quality.contains(
        "cargo +stable clippy --workspace --all-targets --all-features --locked -- -D warnings"
    ));
    assert!(quality.contains("RUSTDOCFLAGS: -D warnings"));
    assert!(quality.contains("cargo +stable doc --workspace --all-features --no-deps --locked"));
    assert!(quality.contains("cargo +stable package --locked"));
}

#[test]
fn complete_suite_runs_on_all_three_supported_host_platforms() {
    let workflow = workflow();
    let tests = job(&workflow, "test", "architecture");

    for host in ["ubuntu-latest", "windows-latest", "macos-latest"] {
        assert!(tests.contains(host), "test matrix is missing {host}");
    }
    assert!(tests.contains("cargo +stable test --workspace --all-features --locked"));
}

#[test]
fn dogfooding_and_msrv_are_independent_visible_gates() {
    let workflow = workflow();
    let architecture = job(&workflow, "architecture", "msrv");
    let msrv = job(&workflow, "msrv", "");

    assert!(architecture.contains("Dogfood the public architecture API"));
    assert!(
        architecture.contains("cargo +stable test --test architecture --all-features --locked")
    );
    assert!(msrv.contains("rustup toolchain install 1.85.0 --profile minimal --no-self-update"));
    assert!(msrv.contains("cargo +1.85.0 check --workspace --all-targets --all-features --locked"));
}

fn workflow() -> String {
    fs::read_to_string(WORKFLOW_PATH)
        .expect("CI workflow should be readable")
        .replace("\r\n", "\n")
}

fn job<'a>(workflow: &'a str, name: &str, next: &str) -> &'a str {
    let start_marker = format!("  {name}:\n");
    let start = workflow
        .find(&start_marker)
        .unwrap_or_else(|| panic!("missing {name} job"));
    let rest = &workflow[start..];
    if next.is_empty() {
        return rest;
    }

    let end_marker = format!("\n  {next}:\n");
    let end = rest
        .find(&end_marker)
        .unwrap_or_else(|| panic!("missing {next} job after {name}"));
    &rest[..end]
}
