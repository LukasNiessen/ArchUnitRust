use std::fs;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[test]
fn manifest_and_archive_metadata_are_ready_for_crates_io() {
    let manifest = read("Cargo.toml");

    assert!(manifest.contains("name = \"archunit\""));
    assert!(manifest.contains(&format!("version = \"{VERSION}\"")));
    assert!(manifest.contains("rust-version = \"1.85\""));
    assert!(manifest.contains("license = \"MIT\""));
    assert!(manifest.contains("publish = [\"crates-io\"]"));
    for repository_only in [
        "\".github/**\"",
        "\"AGENTS.md\"",
        "\"RELEASING.md\"",
        "\"docs/adr/**\"",
        "\"docs/assets/**\"",
        "\"tests/**\"",
    ] {
        assert!(
            manifest.contains(repository_only),
            "publish exclusions are missing {repository_only}"
        );
    }
    assert!(manifest.contains("homepage = \"https://lukasniessen.github.io/ArchUnitRust/\""));
    assert!(
        manifest.contains(
            "documentation = \"https://lukasniessen.github.io/ArchUnitRust/api/archunit/\""
        )
    );

    let license = read("LICENSE");
    assert!(license.starts_with("Copyright 2026 Lukas Niessen"));
    assert!(license.contains("Permission is hereby granted, free of charge"));

    let changelog = read("CHANGELOG.md");
    assert!(changelog.contains(&format!("## [{VERSION}] - ")));
    assert!(changelog.contains("### Known limitations"));
}

#[test]
fn automatic_release_requires_main_and_revalidates_the_package() {
    let workflow = read(".github/workflows/release.yml");

    assert!(workflow.starts_with(
        "name: Release\n\non:\n  push:\n    branches: [main]\n  workflow_dispatch:\n"
    ));
    assert!(workflow.contains("  cancel-in-progress: false\n"));
    assert!(workflow.contains("if: github.ref == 'refs/heads/main'"));
    assert!(workflow.contains("environment: release"));
    assert!(workflow.contains("needs: checks"));
    assert!(workflow.contains("persist-credentials: false"));
    assert!(workflow.contains("cargo +stable fmt --all -- --check"));
    assert!(workflow.contains(
        "cargo +stable clippy --workspace --all-targets --all-features --locked -- -D warnings"
    ));
    assert!(workflow.contains("cargo +stable test --workspace --all-features --locked"));
    assert!(workflow.contains("cargo +stable publish --dry-run --locked"));
    assert!(workflow.contains("python3 .github/scripts/release.py"));
    assert!(workflow.contains("CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}"));
    assert!(workflow.contains("cargo +stable publish --locked"));
}

#[test]
fn release_is_tagged_only_after_the_registry_consumer_passes() {
    let workflow = read(".github/workflows/release.yml");
    let publish = position(&workflow, "          cargo +stable publish --locked");
    let verify = position(
        &workflow,
        "      - name: Install the exact registry version",
    );
    let consumer = position(&workflow, "cargo +stable test --manifest-path");
    let release = position(&workflow, "            gh release create");

    assert!(publish < verify);
    assert!(verify < consumer);
    assert!(consumer < release);
    assert!(workflow.contains("git push origin HEAD:main"));
    assert!(workflow.contains("for attempt in $(seq 1 18)"));
    assert!(workflow.contains("contents: write"));
}

#[test]
fn standalone_fixture_pins_and_exercises_the_exact_registry_version() {
    let fixture = read("tests/fixtures/registry_consumer/Cargo.toml.template");
    let architecture = read("tests/fixtures/registry_consumer/tests/architecture.rs");

    assert!(fixture.contains("publish = false"));
    assert!(fixture.contains(&format!("archunit = \"={VERSION}\"")));
    assert!(architecture.contains("use archunit::{assert_passes, project_files_in};"));
    assert!(architecture.contains(".should_not()"));
    assert!(architecture.contains(".depend_on_files()"));
    assert!(architecture.contains("assert_passes!(rule);"));
}

#[test]
fn public_installation_docs_name_the_verified_registry_version() {
    let readme = read("README.md");
    let guide = read("docs/index.md");
    let plan = read("docs/PORTING_PLAN.md");

    for public_entry_point in [&readme, &guide] {
        assert!(public_entry_point.contains(&format!("cargo add --dev archunit@{VERSION}")));
        assert!(public_entry_point.contains(&format!("archunit = \"{VERSION}\"")));
        assert!(!public_entry_point.contains("cargo add --dev --git"));
        assert!(!public_entry_point.contains("not on crates.io"));
        assert!(!public_entry_point.contains("not published on"));
    }
    assert!(readme.contains("https://crates.io/crates/archunit"));
    assert!(
        plan.contains("`archunit` 0.0.1 is published and its exact registry install is tested")
    );
}

fn read(path: &str) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
        .replace("\r\n", "\n")
}

fn position(content: &str, needle: &str) -> usize {
    content
        .find(needle)
        .unwrap_or_else(|| panic!("missing expected release workflow fragment: {needle}"))
}
