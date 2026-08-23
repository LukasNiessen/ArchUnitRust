use std::{collections::BTreeSet, path::PathBuf};

use archunit::{
    Edge, ExtractionDiagnosticKind, ImportKind, ProjectLocator, SourceOptions,
    enumerate_source_files, extract_graph, locate_project_from,
};

fn fixture_project() -> archunit::CargoProject {
    let fixture =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/extraction_workspace");
    locate_project_from(&ProjectLocator::from_path(fixture))
        .expect("fixture workspace should be discoverable")
}

#[test]
fn every_discovered_file_has_exactly_one_canonical_self_edge() {
    let project = fixture_project();
    let options = SourceOptions::default();
    let sources = enumerate_source_files(&project, options)
        .expect("fixture source enumeration should complete");
    let extraction = extract_graph(&project, options).expect("fixture graph should extract");
    let self_edges = extraction
        .graph()
        .iter()
        .filter(|edge| edge.is_self_edge())
        .collect::<Vec<_>>();

    assert_eq!(self_edges.len(), sources.len());
    for source in sources {
        assert_eq!(
            self_edges
                .iter()
                .filter(|edge| edge.source == source.identifier())
                .count(),
            1
        );
    }

    let library = extraction
        .graph()
        .iter()
        .find(|edge| edge.source == "crates/app/source/library.rs" && edge.is_self_edge())
        .expect("inline-module same-file references should retain the marker edge");
    assert!(!library.external);
    assert!(library.import_kinds.is_empty());
    assert_eq!(library, &Edge::self_edge("crates/app/source/library.rs"));
}

#[test]
fn parallel_internal_and_external_references_merge_their_kinds() {
    let project = fixture_project();
    let extraction =
        extract_graph(&project, SourceOptions::default()).expect("fixture graph should extract");
    let graph = extraction.graph();

    let shared = graph
        .iter()
        .find(|edge| {
            edge.source == "crates/app/source/library.rs"
                && edge.target == "crates/app/source/shared.rs"
        })
        .expect("fixture should merge every library-to-shared reference");
    assert!(!shared.external);
    assert_eq!(
        shared.import_kinds.iter().collect::<Vec<_>>(),
        [ImportKind::Use, ImportKind::Mod, ImportKind::PathReference]
    );

    let wire_format = graph
        .iter()
        .find(|edge| edge.source == "crates/app/source/library.rs" && edge.target == "wire_format")
        .expect("fixture should merge direct and aliased registry references");
    assert!(wire_format.external);
    assert_eq!(
        wire_format.import_kinds.iter().collect::<Vec<_>>(),
        [ImportKind::Use, ImportKind::PathReference]
    );
}

#[test]
fn graph_order_is_repeatable_and_unclassified_references_are_omitted() {
    let project = fixture_project();
    let first = extract_graph(&project, SourceOptions::default())
        .expect("first fixture graph should extract");
    let second = extract_graph(&project, SourceOptions::default())
        .expect("second fixture graph should extract");

    assert_eq!(first, second);
    assert_eq!(first.graph().to_string(), second.graph().to_string());
    assert!(
        !first
            .graph()
            .iter()
            .any(|edge| edge.target == "ghost_dependency")
    );
    assert!(first.diagnostics().iter().any(|diagnostic| {
        diagnostic.kind() == ExtractionDiagnosticKind::UnknownReference
            && diagnostic.subject() == Some("ghost_dependency::Thing")
    }));

    let endpoint_pairs = first
        .graph()
        .iter()
        .map(|edge| (&edge.source, &edge.target))
        .collect::<BTreeSet<_>>();
    assert_eq!(endpoint_pairs.len(), first.graph().len());
    assert!(
        first
            .graph()
            .edges()
            .windows(2)
            .all(|pair| (&pair[0].source, &pair[0].target) < (&pair[1].source, &pair[1].target))
    );
}

#[test]
fn development_sources_receive_self_edges_only_when_selected() {
    let project = fixture_project();
    let production =
        extract_graph(&project, SourceOptions::default()).expect("production graph should extract");
    let with_dev = extract_graph(&project, SourceOptions::new().with_dev_targets(true))
        .expect("development graph should extract");

    assert_eq!(
        with_dev
            .graph()
            .iter()
            .filter(|edge| edge.is_self_edge())
            .count(),
        production
            .graph()
            .iter()
            .filter(|edge| edge.is_self_edge())
            .count()
            + 3
    );
}
