use std::{collections::BTreeSet, fs, path::Path};

use crate::{ArchUnitError, ProjectedEdge, TechnicalError, UserError};

use super::PlantUmlError;
use super::plantuml_dependency::validated_component_name;

/// Deterministic PlantUML generation from projected slice dependencies.
#[derive(Debug, Clone, Copy, Default)]
pub struct PlantUmlRenderer;

impl PlantUmlRenderer {
    /// Renders components derived only from dependency endpoints.
    pub fn render(edges: &[ProjectedEdge]) -> Result<String, PlantUmlError> {
        Self::render_with_components(edges, &Vec::<String>::new())
    }

    /// Renders dependencies plus explicit components, including isolated slices.
    pub fn render_with_components<S>(
        edges: &[ProjectedEdge],
        components: &[S],
    ) -> Result<String, PlantUmlError>
    where
        S: AsRef<str>,
    {
        let components = component_names(edges, components)?;
        let dependencies = dependency_pairs(edges)?;
        let mut lines = vec!["@startuml".to_owned()];
        lines.extend(
            components
                .into_iter()
                .map(|component| format!("  component [{component}]")),
        );
        lines.extend(
            dependencies
                .into_iter()
                .map(|(source, target)| format!("  [{source}] --> [{target}]")),
        );
        lines.push("@enduml".to_owned());
        Ok(format!("{}\n", lines.join("\n")))
    }

    /// Renders and exports dependencies derived only from endpoints.
    pub fn export(
        edges: &[ProjectedEdge],
        output_path: impl AsRef<Path>,
    ) -> Result<(), ArchUnitError> {
        let content = Self::render(edges).map_err(invalid_diagram)?;
        export_plantuml_report(output_path, &content)
    }

    /// Renders and exports dependencies plus explicit isolated components.
    pub fn export_with_components<S>(
        edges: &[ProjectedEdge],
        components: &[S],
        output_path: impl AsRef<Path>,
    ) -> Result<(), ArchUnitError>
    where
        S: AsRef<str>,
    {
        let content = Self::render_with_components(edges, components).map_err(invalid_diagram)?;
        export_plantuml_report(output_path, &content)
    }
}

/// Writes an already-rendered PlantUML report as UTF-8, creating missing parents.
pub fn export_plantuml_report(
    output_path: impl AsRef<Path>,
    content: &str,
) -> Result<(), ArchUnitError> {
    let output_path = output_path.as_ref();
    if output_path.as_os_str().is_empty() {
        return Err(UserError::new("PlantUML output path must not be empty").into());
    }
    if let Some(parent) = output_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).map_err(|source| {
            TechnicalError::with_source(
                format!(
                    "could not create PlantUML report directory '{}'",
                    parent.display()
                ),
                source,
            )
        })?;
    }
    fs::write(output_path, content.as_bytes()).map_err(|source| {
        TechnicalError::with_source(
            format!(
                "could not write PlantUML report '{}'",
                output_path.display()
            ),
            source,
        )
        .into()
    })
}

fn component_names<S>(
    edges: &[ProjectedEdge],
    values: &[S],
) -> Result<BTreeSet<String>, PlantUmlError>
where
    S: AsRef<str>,
{
    values
        .iter()
        .map(|value| value.as_ref().to_owned())
        .chain(
            edges
                .iter()
                .flat_map(|edge| [edge.source_label.clone(), edge.target_label.clone()]),
        )
        .map(validated_component_name)
        .collect()
}

fn dependency_pairs(edges: &[ProjectedEdge]) -> Result<BTreeSet<(String, String)>, PlantUmlError> {
    edges
        .iter()
        .map(|edge| {
            Ok((
                validated_component_name(edge.source_label.clone())?,
                validated_component_name(edge.target_label.clone())?,
            ))
        })
        .collect()
}

fn invalid_diagram(error: PlantUmlError) -> ArchUnitError {
    UserError::with_source("the generated PlantUML diagram is invalid", error).into()
}

#[cfg(test)]
mod tests {
    use std::{fs, time::SystemTime};

    use crate::{Edge, ImportKind, ProjectedEdge};

    use super::PlantUmlRenderer;

    fn edge(source: &str, target: &str) -> ProjectedEdge {
        ProjectedEdge::new(
            source,
            target,
            [Edge::new(source, target, false, [ImportKind::Use])],
        )
    }

    #[test]
    fn renders_isolated_components_and_edges_in_stable_sorted_order() {
        let rendered = PlantUmlRenderer::render_with_components(
            &[edge("services", "models"), edge("api", "services")],
            &["orphan"],
        )
        .expect("fixture diagram should render");

        assert_eq!(
            rendered,
            "@startuml\n  component [api]\n  component [models]\n  component [orphan]\n  component [services]\n  [api] --> [services]\n  [services] --> [models]\n@enduml\n"
        );
    }

    #[test]
    fn exports_byte_identical_utf8_and_rejects_invalid_names() {
        let nonce = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("clock should follow the Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "archunit-plantuml-renderer-{}-{nonce}",
            std::process::id()
        ));
        let path = root.join("nested/architecture.puml");
        let edges = [edge("api", "services")];
        let rendered = PlantUmlRenderer::render(&edges).expect("fixture should render");

        PlantUmlRenderer::export(&edges, &path).expect("fixture should export");

        assert_eq!(
            fs::read_to_string(&path).expect("export should be readable as UTF-8"),
            rendered
        );
        assert!(PlantUmlRenderer::render_with_components(&[], &["bad]name"]).is_err());
        fs::remove_dir_all(root).expect("temporary export should be removable");
    }
}
