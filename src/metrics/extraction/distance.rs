use std::collections::{BTreeMap, BTreeSet};

use crate::common::{
    ArchUnitError, CargoProject, CheckOptions, Graph, SourceOptions, extract_graph_with_options,
};

use super::{FileMetricsInfo, ProjectMetricsInfo, extract_project_metrics};

/// Metrics and dependency evidence for one file-level architectural component.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DistanceInfo {
    file: FileMetricsInfo,
    afferent_coupling: usize,
    efferent_coupling: usize,
    project_file_count: usize,
}

impl DistanceInfo {
    fn new(
        file: FileMetricsInfo,
        afferent_coupling: usize,
        efferent_coupling: usize,
        project_file_count: usize,
    ) -> Self {
        Self {
            file,
            afferent_coupling,
            efferent_coupling,
            project_file_count,
        }
    }

    /// Returns the normalized source-file identifier for this component.
    #[must_use]
    pub fn identifier(&self) -> &str {
        self.file.path()
    }

    /// Returns the complete syntax metrics evidence for this component.
    #[must_use]
    pub const fn file(&self) -> &FileMetricsInfo {
        &self.file
    }

    /// Returns distinct analyzed files that depend on this component.
    #[must_use]
    pub const fn afferent_coupling(&self) -> usize {
        self.afferent_coupling
    }

    /// Returns distinct analyzed files on which this component depends.
    #[must_use]
    pub const fn efferent_coupling(&self) -> usize {
        self.efferent_coupling
    }

    /// Returns the number of files in the full analyzed coupling universe.
    #[must_use]
    pub const fn project_file_count(&self) -> usize {
        self.project_file_count
    }
}

/// Combines an already extracted syntax snapshot and dependency graph into distance inputs.
///
/// Only distinct internal, non-self file dependencies contribute to coupling. Graph endpoints
/// absent from the metrics snapshot and all external dependencies are ignored.
#[must_use]
pub fn build_distance_infos(metrics: &ProjectMetricsInfo, graph: &Graph) -> Vec<DistanceInfo> {
    let identifiers = metrics
        .files()
        .iter()
        .map(FileMetricsInfo::path)
        .collect::<BTreeSet<_>>();
    let mut incoming = BTreeMap::<&str, BTreeSet<&str>>::new();
    let mut outgoing = BTreeMap::<&str, BTreeSet<&str>>::new();

    for edge in graph.edges() {
        if edge.external
            || edge.is_self_edge()
            || !identifiers.contains(edge.source.as_str())
            || !identifiers.contains(edge.target.as_str())
        {
            continue;
        }
        outgoing
            .entry(edge.source.as_str())
            .or_default()
            .insert(edge.target.as_str());
        incoming
            .entry(edge.target.as_str())
            .or_default()
            .insert(edge.source.as_str());
    }

    let project_file_count = metrics.files().len();
    metrics
        .files()
        .iter()
        .cloned()
        .map(|file| {
            let afferent = incoming.get(file.path()).map_or(0, BTreeSet::len);
            let efferent = outgoing.get(file.path()).map_or(0, BTreeSet::len);
            DistanceInfo::new(file, afferent, efferent, project_file_count)
        })
        .collect()
}

/// Extracts file-level distance inputs from one Cargo project and options bag.
pub fn extract_distance_infos(
    project: &CargoProject,
    options: &CheckOptions,
) -> Result<Vec<DistanceInfo>, ArchUnitError> {
    let source_options = SourceOptions::new().with_dev_targets(options.includes_test_sources());
    let metrics = extract_project_metrics(project, source_options)?;
    let graph = extract_graph_with_options(project, options)?.into_graph();
    Ok(build_distance_infos(&metrics, &graph))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{
        common::{Edge, Graph, ImportKind},
        metrics::extract_file_metrics,
    };

    use super::{ProjectMetricsInfo, build_distance_infos};

    fn metrics() -> ProjectMetricsInfo {
        ProjectMetricsInfo::from_files(
            PathBuf::from("fixture"),
            ["src/a.rs", "src/b.rs", "src/c.rs"]
                .into_iter()
                .map(|path| extract_file_metrics(path, "struct Component;").expect("valid fixture"))
                .collect(),
        )
    }

    #[test]
    fn counts_distinct_internal_incoming_and_outgoing_components() {
        let graph = Graph::from_edges([
            Edge::new("src/a.rs", "src/b.rs", false, [ImportKind::Use]),
            Edge::new("src/a.rs", "src/b.rs", false, [ImportKind::PathReference]),
            Edge::new("src/c.rs", "src/b.rs", false, [ImportKind::Use]),
            Edge::new("src/b.rs", "src/a.rs", false, [ImportKind::Use]),
        ]);

        let infos = build_distance_infos(&metrics(), &graph);

        assert_eq!(
            infos
                .iter()
                .map(|info| (
                    info.identifier(),
                    info.afferent_coupling(),
                    info.efferent_coupling(),
                    info.project_file_count()
                ))
                .collect::<Vec<_>>(),
            [
                ("src/a.rs", 1, 1, 3),
                ("src/b.rs", 2, 1, 3),
                ("src/c.rs", 0, 1, 3),
            ]
        );
    }

    #[test]
    fn ignores_self_external_and_unknown_graph_endpoints() {
        let graph = Graph::from_edges([
            Edge::self_edge("src/a.rs"),
            Edge::new("src/a.rs", "serde", true, [ImportKind::Use]),
            Edge::new("src/a.rs", "src/missing.rs", false, [ImportKind::Use]),
            Edge::new("src/missing.rs", "src/b.rs", false, [ImportKind::Use]),
        ]);

        let infos = build_distance_infos(&metrics(), &graph);

        assert!(
            infos
                .iter()
                .all(|info| { info.afferent_coupling() == 0 && info.efferent_coupling() == 0 })
        );
    }
}
