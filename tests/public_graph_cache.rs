use std::{
    fs,
    path::{Path, PathBuf},
    process,
    sync::atomic::{AtomicU64, Ordering},
};

use archunit::{
    CheckOptions, ExtractionDiagnosticKind, ProjectLocator, SourceOptions, clear_graph_cache,
    extract_graph, extract_graph_with_options, locate_project_from,
};

static NEXT_TEMP_PROJECT: AtomicU64 = AtomicU64::new(0);

struct TempProject {
    root: PathBuf,
}

impl TempProject {
    fn new(name: &str, source: &str) -> Self {
        let unique = NEXT_TEMP_PROJECT.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "archunit-rust-cache-{name}-{}-{unique}",
            process::id()
        ));
        fs::create_dir_all(root.join("src")).expect("temporary source directory should be created");
        fs::write(
            root.join("Cargo.toml"),
            "[package]\nname = \"cache-fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
        )
        .expect("temporary manifest should be written");
        fs::write(root.join("src/lib.rs"), source)
            .expect("temporary Rust source should be written");
        Self { root }
    }

    fn root(&self) -> &Path {
        &self.root
    }

    fn write_source(&self, source: &str) {
        fs::write(self.root.join("src/lib.rs"), source)
            .expect("temporary Rust source should be replaced");
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn has_external(extraction: &archunit::GraphExtraction, target: &str) -> bool {
    extraction
        .graph()
        .iter()
        .any(|edge| edge.external && edge.target == target)
}

fn has_unknown(extraction: &archunit::GraphExtraction, subject: &str) -> bool {
    extraction.diagnostics().iter().any(|diagnostic| {
        diagnostic.kind() == ExtractionDiagnosticKind::UnknownReference
            && diagnostic.subject() == Some(subject)
    })
}

#[test]
fn cache_keys_results_and_both_clear_paths_are_observable_without_test_order() {
    clear_graph_cache().expect("cache should start empty");
    let first_project =
        TempProject::new("first", "pub fn value() -> ghost_dep::Thing { loop {} }\n");
    let project = locate_project_from(&ProjectLocator::from_path(first_project.root()))
        .expect("temporary project should be discoverable");

    let initial =
        extract_graph(&project, SourceOptions::default()).expect("initial graph should extract");
    assert!(has_unknown(&initial, "ghost_dep::Thing"));

    first_project.write_source("pub fn value() -> std::fmt::Result { Ok(()) }\n");
    let cached =
        extract_graph(&project, SourceOptions::default()).expect("cached graph should be reusable");
    assert_eq!(cached, initial);
    assert!(has_unknown(&cached, "ghost_dep::Thing"));
    assert!(!has_external(&cached, "std"));

    let refreshed =
        extract_graph_with_options(&project, &CheckOptions::new().with_clear_cache(true))
            .expect("per-check clearing should force re-extraction");
    assert!(has_external(&refreshed, "std"));
    assert!(!has_unknown(&refreshed, "ghost_dep::Thing"));

    first_project.write_source("pub fn value() -> core::fmt::Result { Ok(()) }\n");
    assert_eq!(
        extract_graph(&project, SourceOptions::default())
            .expect("refreshed graph should now be cached"),
        refreshed
    );

    clear_graph_cache().expect("public clearing should invalidate every entry");
    let globally_refreshed = extract_graph(&project, SourceOptions::default())
        .expect("global clearing should force re-extraction");
    assert!(has_external(&globally_refreshed, "core"));
    assert!(!has_external(&globally_refreshed, "std"));

    let second_project = TempProject::new(
        "second",
        "pub fn value() -> alloc::fmt::Error { alloc::fmt::Error }\n",
    );
    let other = locate_project_from(&ProjectLocator::from_path(second_project.root()))
        .expect("second temporary project should be discoverable");
    let other_extraction = extract_graph(&other, SourceOptions::default())
        .expect("workspace identity should select a distinct cache entry");
    assert!(has_external(&other_extraction, "alloc"));
    assert!(!has_external(&other_extraction, "core"));

    clear_graph_cache().expect("cache cleanup should succeed independently of test order");
}
