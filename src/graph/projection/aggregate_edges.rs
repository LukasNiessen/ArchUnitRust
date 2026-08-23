use std::collections::{BTreeMap, BTreeSet};

use crate::{Edge, ImportKind};

use super::{GraphCollapse, GraphQueryError, GraphReportEdge, collapse_graph_node};

/// Collapses and aggregates selected graph edges in deterministic endpoint order.
pub fn aggregate_graph_edges(
    edges: &[Edge],
    collapse: Option<&GraphCollapse>,
    include_self_dependencies: bool,
) -> Result<Vec<GraphReportEdge>, GraphQueryError> {
    let mut groups = BTreeMap::<(String, String), (usize, bool, BTreeSet<ImportKind>)>::new();

    for edge in edges {
        let source = collapse_graph_node(&edge.source, collapse)?;
        let target = collapse_graph_node(&edge.target, collapse)?;
        if !include_self_dependencies && source == target {
            continue;
        }

        let group = groups
            .entry((source, target))
            .or_insert_with(|| (0, false, BTreeSet::new()));
        group.0 += 1;
        group.1 |= edge.external;
        group.2.extend(edge.import_kinds.iter());
    }

    Ok(groups
        .into_iter()
        .map(|((source, target), (count, external, kinds))| {
            GraphReportEdge::new(source, target, count, external, kinds)
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use crate::{
        Edge, FolderDepthCollapse, GraphCollapse, ImportKind, PatternCollapse,
        aggregate_graph_edges,
    };

    #[test]
    fn folder_collapse_aggregates_counts_external_flags_and_kind_unions() {
        let edges = vec![
            Edge::new("src/api/a.rs", "src/domain/a.rs", false, [ImportKind::Use]),
            Edge::new(
                "src/api/b.rs",
                "src/domain/b.rs",
                true,
                [ImportKind::PathReference],
            ),
        ];
        let collapse = GraphCollapse::FolderDepth(
            FolderDepthCollapse::new(2).expect("fixture depth should be valid"),
        );

        let aggregated = aggregate_graph_edges(&edges, Some(&collapse), false)
            .expect("folder collapse should succeed");

        assert_eq!(aggregated.len(), 1);
        assert_eq!(aggregated[0].source, "src/api");
        assert_eq!(aggregated[0].target, "src/domain");
        assert_eq!(aggregated[0].count, 2);
        assert!(aggregated[0].external);
        assert_eq!(
            aggregated[0].import_kinds.iter().collect::<Vec<_>>(),
            [ImportKind::Use, ImportKind::PathReference]
        );
    }

    #[test]
    fn collapse_produced_self_edges_follow_the_same_include_option() {
        let edges = vec![Edge::new(
            "src/api/a.rs",
            "src/api/b.rs",
            false,
            [ImportKind::Use],
        )];
        let collapse = GraphCollapse::Pattern(
            PatternCollapse::first_capture(r"src/([^/]+)/.*")
                .expect("fixture collapse should compile"),
        );

        assert!(
            aggregate_graph_edges(&edges, Some(&collapse), false)
                .expect("collapse should succeed")
                .is_empty()
        );
        assert_eq!(
            aggregate_graph_edges(&edges, Some(&collapse), true).expect("collapse should succeed")
                [0]
            .count,
            1
        );
    }
}
