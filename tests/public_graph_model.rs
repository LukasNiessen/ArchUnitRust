use archunit::{Edge, Graph, ImportKind};

#[test]
fn graph_model_is_usable_from_the_public_crate_surface() {
    let self_edge = Edge::self_edge(r"crates\api\src\lib.rs");
    let dependency = Edge::new(
        "crates/api/src/lib.rs",
        "crates/domain/src/lib.rs",
        false,
        [ImportKind::Use, ImportKind::PathReference],
    );
    let graph = Graph::from_edges([self_edge.clone(), dependency.clone()]);

    assert_eq!(graph.edges(), &[self_edge, dependency]);
    assert_eq!(
        graph.to_string(),
        concat!(
            "crates/api/src/lib.rs -> itself\n",
            "crates/api/src/lib.rs -> crates/domain/src/lib.rs [use, path_reference]"
        )
    );
}
