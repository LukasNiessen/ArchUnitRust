use std::path::PathBuf;

use archunit::{
    ArchUnitError, CargoTargetKind, ProjectLocator, SourceOptions, enumerate_source_files,
    locate_project_from,
};

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/extraction_workspace")
}

fn fixture_project() -> archunit::CargoProject {
    locate_project_from(&ProjectLocator::from_path(
        fixture_root().join("crates/app/source"),
    ))
    .expect("fixture workspace should be discoverable from a member subdirectory")
}

#[test]
fn cargo_metadata_resolves_the_virtual_workspace_and_every_target_kind() {
    let project = fixture_project();
    let expected_root = fixture_root()
        .canonicalize()
        .expect("fixture workspace should have a canonical path");

    assert_eq!(project.root(), expected_root);
    assert_eq!(project.member_roots().count(), 2);
    assert_eq!(project.targets().len(), 7);

    let kinds = project
        .targets()
        .iter()
        .flat_map(|target| target.kinds())
        .map(CargoTargetKind::as_str)
        .collect::<Vec<_>>();
    for expected in [
        "lib",
        "bin",
        "custom-build",
        "proc-macro",
        "test",
        "example",
        "bench",
    ] {
        assert!(
            kinds.contains(&expected),
            "missing Cargo target kind {expected}"
        );
    }
}

#[test]
fn production_enumeration_is_sorted_deduplicated_and_workspace_scoped() {
    let project = fixture_project();
    let sources = enumerate_source_files(&project, SourceOptions::default())
        .expect("fixture source enumeration should succeed");
    let identifiers = sources
        .iter()
        .map(|source| source.identifier())
        .collect::<Vec<_>>();

    assert_eq!(
        identifiers,
        [
            "crates/app/alternate/storage.rs",
            "crates/app/build/custom.rs",
            "crates/app/cmd/server.rs",
            "crates/app/source/ambiguous.rs",
            "crates/app/source/ambiguous/mod.rs",
            "crates/app/source/api.rs",
            "crates/app/source/api/model.rs",
            "crates/app/source/broken.rs",
            "crates/app/source/inline/nested.rs",
            "crates/app/source/inline/redirected.rs",
            "crates/app/source/legacy/mod.rs",
            "crates/app/source/library.rs",
            "crates/app/source/platform.rs",
            "crates/app/source/shared.rs",
            "crates/macros/macro_src/entry.rs",
        ]
    );
    assert_eq!(
        identifiers
            .iter()
            .filter(|identifier| identifier.ends_with("shared.rs"))
            .count(),
        1
    );
}

#[test]
fn development_target_roots_are_included_only_when_requested() {
    let project = fixture_project();
    let options = SourceOptions::new().with_dev_targets(true);
    let sources = enumerate_source_files(&project, options)
        .expect("fixture source enumeration should succeed");
    let identifiers = sources
        .iter()
        .map(|source| source.identifier())
        .collect::<Vec<_>>();

    for expected in [
        "crates/app/qa/architecture.rs",
        "crates/app/samples/demo.rs",
        "crates/app/perf/speed.rs",
    ] {
        assert!(identifiers.contains(&expected));
    }
    assert_eq!(identifiers.len(), 18);
}

#[test]
fn an_unusable_explicit_locator_is_a_user_error() {
    let missing = fixture_root().join("missing/project");
    let error = locate_project_from(&ProjectLocator::from_path(missing))
        .expect_err("missing locator should be rejected");

    assert!(matches!(error, ArchUnitError::User(_)));
}
