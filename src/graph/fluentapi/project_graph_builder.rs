use std::path::Path;

use crate::{
    common::{
        ArchUnitError, CheckOptions, PatternError, ProjectLocator, RegexFactory, UserError,
        extract_graph_with_options, locate_project_from,
    },
    graph::{
        FolderDepthCollapse, GraphCollapse, GraphQueryError, GraphQueryOptions, GraphRenderer,
        GraphReportFormat, GraphReportSnapshot, GraphReportSummary, PatternCollapse,
        create_graph_snapshot,
    },
};

/// Immutable query builder for dependency-graph snapshots and reports.
#[derive(Debug, Clone)]
#[must_use = "a graph report query has no effect until a terminal is called"]
pub struct ProjectGraphBuilder {
    project_locator: ProjectLocator,
    options: GraphQueryOptions,
    check_options: CheckOptions,
    configuration_error: Option<GraphQueryError>,
}

impl ProjectGraphBuilder {
    pub(super) const fn new(project_locator: ProjectLocator) -> Self {
        Self {
            project_locator,
            options: GraphQueryOptions::new(),
            check_options: CheckOptions::new(),
            configuration_error: None,
        }
    }

    /// Includes Cargo-visible external dependency targets and edges.
    pub fn include_external_dependencies(mut self) -> Self {
        self.options = self.options.with_external_dependencies(true);
        self
    }

    /// Includes extracted marker self-edges and self-edges produced by collapsing.
    pub fn include_self_dependencies(mut self) -> Self {
        self.options = self.options.with_self_dependencies(true);
        self
    }

    /// Keeps matching nodes and their undirected neighbors up to `depth` hops away.
    pub fn focus_on(
        mut self,
        pattern: impl Into<crate::common::PatternSpec>,
        depth: usize,
    ) -> Self {
        match RegexFactory::default().path_matcher(pattern) {
            Ok(filter) => self.options = self.options.with_focus(filter, depth),
            Err(source) => self.record_pattern_error("focus", source),
        }
        self
    }

    /// Keeps matching nodes and every transitive outgoing dependency.
    pub fn reachable_from(mut self, pattern: impl Into<crate::common::PatternSpec>) -> Self {
        match RegexFactory::default().path_matcher(pattern) {
            Ok(filter) => self.options = self.options.with_reachable_from(filter),
            Err(source) => self.record_pattern_error("reachable-from", source),
        }
        self
    }

    /// Keeps matching nodes and every transitive incoming dependent.
    pub fn dependents_of(mut self, pattern: impl Into<crate::common::PatternSpec>) -> Self {
        match RegexFactory::default().path_matcher(pattern) {
            Ok(filter) => self.options = self.options.with_dependents_of(filter),
            Err(source) => self.record_pattern_error("dependents-of", source),
        }
        self
    }

    /// Collapses file nodes to their containing folder at a positive leading path depth.
    pub fn collapse_to_folder_depth(mut self, depth: usize) -> Self {
        match FolderDepthCollapse::new(depth) {
            Ok(collapse) => {
                self.options = self
                    .options
                    .with_collapse(GraphCollapse::FolderDepth(collapse));
            }
            Err(error) => self.record_error(error),
        }
        self
    }

    /// Collapses nodes with a regular expression and the first capture (`$1`) as their label.
    pub fn collapse_by_pattern(mut self, expression: impl AsRef<str>) -> Self {
        match PatternCollapse::first_capture(expression) {
            Ok(collapse) => {
                self.options = self.options.with_collapse(GraphCollapse::Pattern(collapse));
            }
            Err(error) => self.record_error(error),
        }
        self
    }

    /// Collapses nodes with explicit Rust `regex` capture-replacement syntax.
    pub fn collapse_by_pattern_with_replacement(
        mut self,
        expression: impl AsRef<str>,
        replacement: impl Into<String>,
    ) -> Self {
        match PatternCollapse::new(expression, replacement) {
            Ok(collapse) => {
                self.options = self.options.with_collapse(GraphCollapse::Pattern(collapse));
            }
            Err(error) => self.record_error(error),
        }
        self
    }

    /// Sets the snapshot title.
    pub fn titled(mut self, title: impl Into<String>) -> Self {
        match self.options.clone().with_title(title) {
            Ok(options) => self.options = options,
            Err(error) => self.record_error(error),
        }
        self
    }

    /// Replaces the extraction options used by snapshot terminals.
    pub fn with_check_options(mut self, options: CheckOptions) -> Self {
        self.check_options = options;
        self
    }

    /// Extracts the project and returns the renderer-neutral queried snapshot.
    pub fn snapshot(&self) -> Result<GraphReportSnapshot, ArchUnitError> {
        if let Some(error) = &self.configuration_error {
            return Err(configuration_error(error.clone()));
        }

        let project = locate_project_from(self.project_locator())?;
        let extraction = extract_graph_with_options(&project, self.check_options())?;
        create_graph_snapshot(extraction.graph(), self.options()).map_err(configuration_error)
    }

    /// Extracts the project and returns only the queried snapshot counts.
    pub fn summary(&self) -> Result<GraphReportSummary, ArchUnitError> {
        self.snapshot().map(|snapshot| snapshot.summary)
    }

    /// Extracts once and renders the queried snapshot as `format`.
    pub fn render(&self, format: GraphReportFormat) -> Result<String, ArchUnitError> {
        self.snapshot()
            .map(|snapshot| GraphRenderer::render(&snapshot, format))
    }

    /// Extracts once and renders Graphviz DOT.
    pub fn to_dot(&self) -> Result<String, ArchUnitError> {
        self.render(GraphReportFormat::Dot)
    }

    /// Extracts once and renders Mermaid flowchart source.
    pub fn to_mermaid(&self) -> Result<String, ArchUnitError> {
        self.render(GraphReportFormat::Mermaid)
    }

    /// Extracts once and renders D2 diagram source.
    pub fn to_d2(&self) -> Result<String, ArchUnitError> {
        self.render(GraphReportFormat::D2)
    }

    /// Extracts once and renders aggregated dependency CSV.
    pub fn to_csv(&self) -> Result<String, ArchUnitError> {
        self.render(GraphReportFormat::Csv)
    }

    /// Extracts once and renders complete snapshot JSON.
    pub fn to_json(&self) -> Result<String, ArchUnitError> {
        self.render(GraphReportFormat::Json)
    }

    /// Extracts once and renders a self-contained offline HTML report.
    pub fn to_html(&self) -> Result<String, ArchUnitError> {
        self.render(GraphReportFormat::Html)
    }

    /// Extracts once, renders `format`, and writes it as UTF-8.
    pub fn export(
        &self,
        format: GraphReportFormat,
        output_path: impl AsRef<Path>,
    ) -> Result<(), ArchUnitError> {
        let snapshot = self.snapshot()?;
        GraphRenderer::export(&snapshot, format, output_path)
    }

    /// Extracts once and exports Graphviz DOT as UTF-8.
    pub fn export_as_dot(&self, output_path: impl AsRef<Path>) -> Result<(), ArchUnitError> {
        self.export(GraphReportFormat::Dot, output_path)
    }

    /// Extracts once and exports Mermaid flowchart source as UTF-8.
    pub fn export_as_mermaid(&self, output_path: impl AsRef<Path>) -> Result<(), ArchUnitError> {
        self.export(GraphReportFormat::Mermaid, output_path)
    }

    /// Extracts once and exports D2 diagram source as UTF-8.
    pub fn export_as_d2(&self, output_path: impl AsRef<Path>) -> Result<(), ArchUnitError> {
        self.export(GraphReportFormat::D2, output_path)
    }

    /// Extracts once and exports aggregated dependency CSV as UTF-8.
    pub fn export_as_csv(&self, output_path: impl AsRef<Path>) -> Result<(), ArchUnitError> {
        self.export(GraphReportFormat::Csv, output_path)
    }

    /// Extracts once and exports complete snapshot JSON as UTF-8.
    pub fn export_as_json(&self, output_path: impl AsRef<Path>) -> Result<(), ArchUnitError> {
        self.export(GraphReportFormat::Json, output_path)
    }

    /// Extracts once and exports a self-contained offline HTML report as UTF-8.
    pub fn export_as_html(&self, output_path: impl AsRef<Path>) -> Result<(), ArchUnitError> {
        self.export(GraphReportFormat::Html, output_path)
    }

    /// Returns where Cargo project discovery begins.
    #[must_use]
    pub const fn project_locator(&self) -> &ProjectLocator {
        &self.project_locator
    }

    /// Returns the immutable graph query options.
    #[must_use]
    pub const fn options(&self) -> &GraphQueryOptions {
        &self.options
    }

    /// Returns the immutable extraction options.
    #[must_use]
    pub const fn check_options(&self) -> &CheckOptions {
        &self.check_options
    }

    fn record_pattern_error(&mut self, context: &'static str, source: PatternError) {
        self.record_error(GraphQueryError::invalid_pattern(context, source));
    }

    fn record_error(&mut self, error: GraphQueryError) {
        if self.configuration_error.is_none() {
            self.configuration_error = Some(error);
        }
    }
}

fn configuration_error(error: GraphQueryError) -> ArchUnitError {
    ArchUnitError::from(UserError::with_source(
        "the graph report query is invalid",
        error,
    ))
}

#[cfg(test)]
mod tests {
    use crate::{
        common::{ArchUnitError, CheckOptions},
        graph::{GraphCollapse, dependency_graph, project_graph_in},
    };

    #[test]
    fn modifiers_are_consuming_branchable_values() {
        let base = dependency_graph();
        let external = base.clone().include_external_dependencies();
        let self_edges = external.clone().include_self_dependencies();
        let focused = self_edges.clone().focus_on("src/domain/**", 2);
        let reachable = focused.clone().reachable_from("src/api.rs");
        let dependents = reachable.clone().dependents_of("src/database.rs");
        let collapsed = dependents.clone().collapse_by_pattern(r"src/([^/]+)/.*");
        let titled = collapsed.clone().titled("Focused Architecture");

        assert!(!base.options().includes_external_dependencies());
        assert!(external.options().includes_external_dependencies());
        assert!(self_edges.options().includes_self_dependencies());
        assert_eq!(focused.options().focus_depth(), 2);
        assert!(reachable.options().reachable_from().is_some());
        assert!(dependents.options().dependents_of().is_some());
        assert!(matches!(
            collapsed.options().collapse(),
            Some(GraphCollapse::Pattern(_))
        ));
        assert_eq!(titled.options().title(), Some("Focused Architecture"));
    }

    #[test]
    fn folder_and_pattern_collapse_replace_the_previous_strategy() {
        let folder = dependency_graph().collapse_to_folder_depth(2);
        let pattern = folder
            .clone()
            .collapse_by_pattern_with_replacement(r"src/([^/]+)/.*", "component-$1");

        assert!(matches!(
            folder.options().collapse(),
            Some(GraphCollapse::FolderDepth(_))
        ));
        assert!(matches!(
            pattern.options().collapse(),
            Some(GraphCollapse::Pattern(_))
        ));
    }

    #[test]
    fn check_options_are_owned_and_branchable() {
        let options = CheckOptions::new()
            .with_clear_cache(true)
            .with_test_sources(true);
        let base = dependency_graph();
        let configured = base.clone().with_check_options(options);

        assert!(!base.check_options().clears_cache());
        assert!(configured.check_options().clears_cache());
        assert!(configured.check_options().includes_test_sources());
    }

    #[test]
    fn first_query_error_is_reported_before_project_location() {
        let rule = project_graph_in("definitely/missing")
            .focus_on("src/[domain", 1)
            .collapse_to_folder_depth(0)
            .titled("");
        let error = rule
            .snapshot()
            .expect_err("invalid focus should prevent project discovery");

        assert!(matches!(error, ArchUnitError::User(_)));
        assert!(error.to_string().contains("invalid focus pattern"));
        assert!(error.to_string().contains("src/[domain"));
    }
}
