use std::path::PathBuf;

use archunit::{
    ArchUnitError, CheckOptions, Graph, GraphQueryOptions, GraphSnapshotFactory, ImportKind,
    dependency_graph_in, project_graph_in,
};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

#[test]
fn project_graph_extracts_one_stable_renderer_neutral_snapshot() {
    let report = project_graph_in(fixture("layered_project")).titled("Fixture Architecture");
    let snapshot = report
        .snapshot()
        .expect("the layered Cargo fixture should be analyzable");

    assert_eq!(snapshot.title, "Fixture Architecture");
    assert_eq!(snapshot.summary.node_count, 7);
    assert_eq!(snapshot.summary.edge_count, 10);
    assert_eq!(snapshot.summary.raw_edge_count, 10);
    assert_eq!(snapshot.summary.external_edge_count, 0);
    assert_eq!(
        report.summary().expect("summary should reuse the query"),
        snapshot.summary
    );
    assert_eq!(snapshot.nodes[0].id, "n0");
    assert!(snapshot.nodes.iter().any(|node| node.label == "src/lib.rs"));

    let dependency = snapshot
        .edges
        .iter()
        .find(|edge| edge.source == "src/api/mod.rs" && edge.target == "src/application/service.rs")
        .expect("the fixture API should depend on its application service");
    assert_eq!(dependency.count, 1);
    assert!(!dependency.external);
    assert!(dependency.import_kinds.contains(ImportKind::Use));
    assert_eq!(dependency.import_kinds.len(), 1);
}

#[test]
fn public_queries_select_directed_subgraphs_and_collapse_after_selection() {
    let root = fixture("layered_project");
    let focused = dependency_graph_in(&root)
        .focus_on("src/application/**", 0)
        .snapshot()
        .expect("focus query should succeed");
    let reachable = dependency_graph_in(&root)
        .reachable_from("src/api/**")
        .snapshot()
        .expect("outgoing traversal should succeed");
    let dependents = dependency_graph_in(&root)
        .dependents_of("src/database/repository.rs")
        .snapshot()
        .expect("incoming traversal should succeed");
    let collapsed = dependency_graph_in(&root)
        .collapse_to_folder_depth(2)
        .snapshot()
        .expect("folder collapse should succeed");

    assert_eq!(
        focused
            .nodes
            .iter()
            .map(|node| node.label.as_str())
            .collect::<Vec<_>>(),
        ["src/application/mod.rs", "src/application/service.rs"]
    );
    assert_eq!(focused.summary.edge_count, 1);
    assert_eq!(reachable.summary.node_count, 4);
    assert_eq!(reachable.summary.edge_count, 4);
    assert_eq!(dependents.summary.node_count, 6);
    assert_eq!(dependents.summary.edge_count, 8);
    assert_eq!(collapsed.summary.node_count, 5);
    assert_eq!(collapsed.summary.raw_edge_count, 10);
    assert_eq!(collapsed.summary.edge_count, 8);
    assert!(collapsed.nodes.iter().any(|node| node.label == "src/api"));
}

#[test]
fn external_self_and_source_target_options_reach_real_extraction() {
    let extraction_fixture = fixture("extraction_workspace");
    let default = project_graph_in(&extraction_fixture)
        .snapshot()
        .expect("default extraction snapshot should succeed");
    let inclusive = project_graph_in(&extraction_fixture)
        .include_external_dependencies()
        .include_self_dependencies()
        .with_check_options(CheckOptions::new().with_test_sources(true))
        .snapshot()
        .expect("inclusive extraction snapshot should succeed");

    assert!(!default.nodes.iter().any(|node| node.label == "tokio"));
    assert!(inclusive.nodes.iter().any(|node| node.label == "tokio"));
    assert!(inclusive.summary.external_edge_count > 0);
    assert!(
        inclusive
            .edges
            .iter()
            .any(|edge| edge.source == edge.target)
    );
    assert!(
        inclusive
            .nodes
            .iter()
            .any(|node| node.label == "crates/app/qa/architecture.rs")
    );
}

#[test]
fn fluent_query_errors_are_user_failures_before_project_discovery() {
    let cases = [
        project_graph_in("definitely/missing").focus_on("src/[api", 1),
        project_graph_in("definitely/missing").collapse_to_folder_depth(0),
        project_graph_in("definitely/missing").collapse_by_pattern("["),
        project_graph_in("definitely/missing").titled(""),
    ];

    for report in cases {
        assert!(matches!(report.snapshot(), Err(ArchUnitError::User(_))));
    }
}

#[test]
fn snapshot_factory_and_query_options_are_usable_without_project_io() {
    let graph = Graph::from_edges([]);
    let options = GraphQueryOptions::default();
    let snapshot = GraphSnapshotFactory::create(&graph, &options)
        .expect("an empty in-memory graph should still have a snapshot");

    assert!(snapshot.nodes.is_empty());
    assert!(snapshot.edges.is_empty());
    assert_eq!(snapshot.summary.node_count, 0);
}
