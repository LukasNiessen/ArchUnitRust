use std::path::PathBuf;

use archunit::{
    DependencyExtraction, ExtractionDiagnosticKind, ImportKind, ProjectLocator, SourceOptions,
    extract_dependencies, locate_project_from,
};

fn extraction() -> DependencyExtraction {
    let fixture =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/extraction_workspace");
    let project = locate_project_from(&ProjectLocator::from_path(fixture))
        .expect("fixture workspace should be discoverable");
    extract_dependencies(&project, SourceOptions::default())
        .expect("fixture dependency extraction should complete")
}

fn has_internal_reference(
    extraction: &DependencyExtraction,
    source: &str,
    path: &str,
    target: &str,
    kind: ImportKind,
) -> bool {
    extraction.references().iter().any(|reference| {
        reference.source() == source
            && reference.referenced_path() == path
            && reference.internal_target() == Some(target)
            && reference.kind() == kind
            && reference.line() > 0
    })
}

#[test]
fn module_tree_resolves_outlined_inline_path_redirected_and_cfg_modules() {
    let extraction = extraction();
    let library = "crates/app/source/library.rs";

    for (path, target) in [
        ("crate::shared", "crates/app/source/shared.rs"),
        ("crate::api", "crates/app/source/api.rs"),
        ("crate::storage", "crates/app/alternate/storage.rs"),
        ("crate::platform", "crates/app/source/platform.rs"),
        ("crate::legacy", "crates/app/source/legacy/mod.rs"),
    ] {
        assert!(has_internal_reference(
            &extraction,
            library,
            path,
            target,
            ImportKind::Mod
        ));
    }
    assert!(has_internal_reference(
        &extraction,
        library,
        "crate::inline",
        library,
        ImportKind::Mod
    ));
    assert!(has_internal_reference(
        &extraction,
        "crates/app/source/library.rs",
        "crate::inline::nested",
        "crates/app/source/inline/nested.rs",
        ImportKind::Mod
    ));
    assert!(has_internal_reference(
        &extraction,
        "crates/app/source/library.rs",
        "crate::inline::redirected",
        "crates/app/source/inline/redirected.rs",
        ImportKind::Mod
    ));
}

#[test]
fn uses_aliases_and_qualified_paths_resolve_to_the_longest_module_prefix() {
    let extraction = extraction();

    assert!(has_internal_reference(
        &extraction,
        "crates/app/source/library.rs",
        "public_api::model::Model",
        "crates/app/source/api/model.rs",
        ImportKind::PathReference
    ));
    assert!(has_internal_reference(
        &extraction,
        "crates/app/source/api.rs",
        "crate::shared::Marker",
        "crates/app/source/shared.rs",
        ImportKind::PathReference
    ));
    assert!(has_internal_reference(
        &extraction,
        "crates/app/source/api/model.rs",
        "super::Handler",
        "crates/app/source/api.rs",
        ImportKind::PathReference
    ));
    assert!(has_internal_reference(
        &extraction,
        "crates/app/source/api/model.rs",
        "self::Model",
        "crates/app/source/api/model.rs",
        ImportKind::PathReference
    ));
    assert!(has_internal_reference(
        &extraction,
        "crates/app/source/library.rs",
        "crate::api::Handler",
        "crates/app/source/api.rs",
        ImportKind::PubUse
    ));
    assert!(has_internal_reference(
        &extraction,
        "crates/app/source/library.rs",
        "crate::api::model::Model",
        "crates/app/source/api/model.rs",
        ImportKind::Use
    ));
}

#[test]
fn external_syntax_remains_raw_for_cargo_aware_classification() {
    let extraction = extraction();

    for (path, kind) in [
        ("std::collections::HashMap", ImportKind::PathReference),
        ("core::option::Option", ImportKind::PathReference),
        ("proc_macro", ImportKind::ExternCrate),
        ("tokio::join", ImportKind::MacroReference),
        ("::std::vec::Vec", ImportKind::PathReference),
    ] {
        assert!(extraction.references().iter().any(|reference| {
            reference.referenced_path() == path
                && reference.internal_target().is_none()
                && reference.kind() == kind
        }));
    }
    assert!(!extraction.references().iter().any(|reference| {
        reference
            .referenced_path()
            .contains("macro_tokens_are_not_expanded")
    }));
}

#[test]
fn parse_missing_and_ambiguous_module_failures_are_non_fatal_diagnostics() {
    let extraction = extraction();
    let kinds = extraction
        .diagnostics()
        .iter()
        .map(|diagnostic| diagnostic.kind())
        .collect::<Vec<_>>();

    assert!(kinds.contains(&ExtractionDiagnosticKind::ParseFile));
    assert!(kinds.contains(&ExtractionDiagnosticKind::MissingModule));
    assert!(kinds.contains(&ExtractionDiagnosticKind::AmbiguousModule));
    assert!(kinds.contains(&ExtractionDiagnosticKind::InvalidPathAttribute));
    assert!(kinds.contains(&ExtractionDiagnosticKind::ModuleCycle));

    let ambiguous = extraction
        .diagnostics()
        .iter()
        .find(|diagnostic| {
            diagnostic.kind() == ExtractionDiagnosticKind::AmbiguousModule
                && diagnostic.subject() == Some("ambiguous")
        })
        .expect("fixture should report the two supported ambiguous module layouts");
    assert_eq!(
        ambiguous.candidates(),
        [
            "crates/app/source/ambiguous.rs",
            "crates/app/source/ambiguous/mod.rs"
        ]
    );
}
