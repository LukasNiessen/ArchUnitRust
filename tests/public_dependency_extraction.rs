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

fn has_external_reference(
    extraction: &DependencyExtraction,
    source: &str,
    path: &str,
    target: &str,
    kind: ImportKind,
) -> bool {
    extraction.references().iter().any(|reference| {
        reference.source() == source
            && reference.referenced_path() == path
            && reference.external_target() == Some(target)
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
    assert!(has_internal_reference(
        &extraction,
        "crates/app/source/library.rs",
        "macro_tools",
        "crates/macros/macro_src/entry.rs",
        ImportKind::Use
    ));
    assert!(has_internal_reference(
        &extraction,
        "crates/app/source/library.rs",
        "macros_alias::fixture",
        "crates/macros/macro_src/entry.rs",
        ImportKind::MacroReference
    ));
}

#[test]
fn cargo_visible_names_classify_sysroot_and_registry_dependencies() {
    let extraction = extraction();

    for (source, path, target, kind) in [
        (
            "crates/app/source/api.rs",
            "std::collections::HashMap",
            "std",
            ImportKind::PathReference,
        ),
        (
            "crates/app/source/api.rs",
            "core::option::Option",
            "core",
            ImportKind::PathReference,
        ),
        (
            "crates/app/source/library.rs",
            "alloc::vec::Vec",
            "alloc",
            ImportKind::PathReference,
        ),
        (
            "crates/app/source/library.rs",
            "proc_macro",
            "proc_macro",
            ImportKind::ExternCrate,
        ),
        (
            "crates/app/source/library.rs",
            "tokio::join",
            "tokio",
            ImportKind::MacroReference,
        ),
        (
            "crates/app/source/library.rs",
            "::std::vec::Vec",
            "std",
            ImportKind::PathReference,
        ),
        (
            "crates/app/source/library.rs",
            "wire_format",
            "wire_format",
            ImportKind::Use,
        ),
        (
            "crates/app/source/library.rs",
            "serialization::Value",
            "wire_format",
            ImportKind::PathReference,
        ),
    ] {
        assert!(has_external_reference(
            &extraction,
            source,
            path,
            target,
            kind
        ));
    }
    assert!(!extraction.references().iter().any(|reference| {
        reference
            .referenced_path()
            .contains("macro_tokens_are_not_expanded")
    }));
}

#[test]
fn undeclared_first_segments_are_diagnostic_and_unclassified() {
    let extraction = extraction();
    let unknown = extraction
        .references()
        .iter()
        .find(|reference| reference.referenced_path() == "ghost_dependency::Thing")
        .expect("fixture should retain unknown syntax as diagnostic evidence");

    assert!(unknown.target().is_none());
    assert!(extraction.diagnostics().iter().any(|diagnostic| {
        diagnostic.kind() == ExtractionDiagnosticKind::UnknownReference
            && diagnostic.subject() == Some("ghost_dependency::Thing")
    }));
    assert_eq!(
        extraction
            .diagnostics()
            .iter()
            .filter(|diagnostic| diagnostic.kind() == ExtractionDiagnosticKind::UnknownReference)
            .filter_map(|diagnostic| diagnostic.subject())
            .collect::<Vec<_>>(),
        ["ghost_dependency::Thing"]
    );
}

#[test]
fn dependency_kinds_follow_the_cargo_target_context() {
    let fixture =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/extraction_workspace");
    let project = locate_project_from(&ProjectLocator::from_path(fixture))
        .expect("fixture workspace should be discoverable");
    let production = extract_dependencies(&project, SourceOptions::default())
        .expect("production extraction should complete");
    let with_dev = extract_dependencies(&project, SourceOptions::new().with_dev_targets(true))
        .expect("development extraction should complete");

    assert!(has_external_reference(
        &production,
        "crates/app/build/custom.rs",
        "build_only::compile",
        "build_only",
        ImportKind::PathReference
    ));
    assert!(
        !production
            .references()
            .iter()
            .any(|reference| reference.referenced_path().starts_with("dev_only"))
    );
    assert!(has_external_reference(
        &with_dev,
        "crates/app/qa/architecture.rs",
        "dev_only::assert_eq",
        "dev_only",
        ImportKind::MacroReference
    ));
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
