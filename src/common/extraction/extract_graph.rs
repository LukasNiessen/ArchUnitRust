use super::{
    CargoProject, Edge, ExtractionDiagnostic, Graph, SourceOptions, enumerate_source_files,
    extract_dependencies::extract_dependencies_from_sources,
};
use crate::ArchUnitError;

/// A deterministic dependency graph together with non-fatal extraction diagnostics.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct GraphExtraction {
    graph: Graph,
    diagnostics: Vec<ExtractionDiagnostic>,
}

impl GraphExtraction {
    fn new(graph: Graph, diagnostics: Vec<ExtractionDiagnostic>) -> Self {
        Self { graph, diagnostics }
    }

    /// Returns the fully classified, merged dependency graph.
    #[must_use]
    pub const fn graph(&self) -> &Graph {
        &self.graph
    }

    /// Returns non-fatal parse and resolution diagnostics in deterministic order.
    #[must_use]
    pub fn diagnostics(&self) -> &[ExtractionDiagnostic] {
        &self.diagnostics
    }

    /// Consumes the extraction result and returns its graph.
    #[must_use]
    pub fn into_graph(self) -> Graph {
        self.graph
    }
}

/// Extracts one deterministic graph from the selected Cargo project targets.
///
/// Every discovered Rust source receives a marker self-edge. Classified dependency references
/// become internal or external edges, while ambiguous and unknown references remain diagnostics.
/// Parallel endpoint pairs are merged by [`Graph`].
pub(crate) fn extract_graph_uncached(
    project: &CargoProject,
    options: SourceOptions,
) -> Result<GraphExtraction, ArchUnitError> {
    let sources = enumerate_source_files(project, options)?;
    let dependencies = extract_dependencies_from_sources(project, options, &sources);
    let mut edges = sources
        .iter()
        .map(|source| Edge::self_edge(source.identifier()))
        .collect::<Vec<_>>();

    edges.extend(dependencies.references().iter().filter_map(|reference| {
        reference.target().map(|target| {
            Edge::new(
                reference.source(),
                target.as_str(),
                target.is_external(),
                [reference.kind()],
            )
        })
    }));

    Ok(GraphExtraction::new(
        Graph::from_edges(edges),
        dependencies.diagnostics().to_vec(),
    ))
}
