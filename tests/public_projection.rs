use std::path::PathBuf;

use archunit::{
    Edge, Graph, ImportKind, MapFunction, MappedEdge, NodeProjectionOptions, ProjectLocator,
    SourceOptions, extract_graph, locate_project_from, project_edges, project_to_nodes,
    project_to_nodes_with_options,
};

#[test]
fn public_map_hook_filters_relabels_and_preserves_raw_evidence() {
    let first = Edge::new("src/api/a.rs", "src/domain/a.rs", false, [ImportKind::Use]);
    let second = Edge::new(
        "src/api/b.rs",
        "src/domain/b.rs",
        false,
        [ImportKind::PathReference],
    );
    let external = Edge::new("src/api/a.rs", "serde", true, [ImportKind::Use]);
    let graph = Graph::from_edges([first.clone(), second.clone(), external]);
    let mapper: &MapFunction<'_> =
        &|edge| (!edge.external).then(|| MappedEdge::new("API", "Domain"));

    let projected = project_edges(&graph, mapper);

    assert_eq!(projected.len(), 1);
    assert_eq!(projected[0].source_label, "API");
    assert_eq!(projected[0].target_label, "Domain");
    assert_eq!(projected[0].cumulated_edges, [first, second]);
}

#[test]
fn public_node_projection_retains_files_and_controls_external_targets() {
    let external = Edge::new("src/lib.rs", "serde", true, [ImportKind::Use]);
    let graph = Graph::from_edges([Edge::self_edge("src/isolated.rs"), external.clone()]);

    let internal_nodes = project_to_nodes(&graph);
    let all_nodes =
        project_to_nodes_with_options(&graph, NodeProjectionOptions::new().with_externals(true));

    assert_eq!(
        internal_nodes
            .iter()
            .map(|node| node.label.as_str())
            .collect::<Vec<_>>(),
        ["src/isolated.rs", "src/lib.rs"]
    );
    assert!(internal_nodes[0].incoming.is_empty());
    assert!(internal_nodes[0].outgoing.is_empty());
    assert_eq!(
        all_nodes
            .iter()
            .map(|node| node.label.as_str())
            .collect::<Vec<_>>(),
        ["serde", "src/isolated.rs", "src/lib.rs"]
    );
    assert_eq!(all_nodes[0].incoming, [external]);
}

#[test]
fn extracted_cargo_graph_flows_through_the_public_projection_layer() {
    let fixture =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/extraction_workspace");
    let project = locate_project_from(&ProjectLocator::from_path(fixture))
        .expect("fixture workspace should be discoverable");
    let extraction = extract_graph(&project, SourceOptions::default())
        .expect("fixture workspace should extract");

    let nodes = project_to_nodes(extraction.graph());
    let internal_labels = extraction
        .graph()
        .iter()
        .filter(|edge| edge.is_self_edge())
        .map(|edge| edge.source.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        nodes
            .iter()
            .map(|node| node.label.as_str())
            .collect::<Vec<_>>(),
        internal_labels
    );
    assert!(nodes.iter().all(|node| {
        node.incoming.iter().all(|edge| !edge.is_self_edge())
            && node.outgoing.iter().all(|edge| !edge.is_self_edge())
    }));

    let grouped = project_edges(extraction.graph(), |edge| {
        (edge.source == "crates/app/source/library.rs" && !edge.external && !edge.is_self_edge())
            .then(|| MappedEdge::new("library", "internal dependency"))
    });
    assert_eq!(grouped.len(), 1);
    assert!(grouped[0].cumulated_edges.len() > 1);
    assert!(grouped[0].cumulated_edges.iter().all(|edge| {
        edge.source == "crates/app/source/library.rs" && !edge.external && !edge.is_self_edge()
    }));
}
