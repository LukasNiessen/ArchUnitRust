use std::collections::BTreeSet;

use super::plantuml_dependency::validated_component_name;
use super::{PlantUmlDependency, PlantUmlError};

/// Immutable components and allowed directed dependencies parsed from PlantUML.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlantUmlDiagram {
    /// Declared and dependency-implied component names in first-seen order.
    pub components: Vec<String>,
    /// Unique allowed dependencies in first-seen order.
    pub dependencies: Vec<PlantUmlDependency>,
}

impl PlantUmlDiagram {
    /// Creates a diagram, adding dependency endpoints to the component collection.
    pub fn new<C, D>(components: C, dependencies: D) -> Result<Self, PlantUmlError>
    where
        C: IntoIterator<Item = String>,
        D: IntoIterator<Item = PlantUmlDependency>,
    {
        let mut unique_dependencies = BTreeSet::new();
        let dependencies = dependencies
            .into_iter()
            .filter(|dependency| unique_dependencies.insert(dependency.clone()))
            .collect::<Vec<_>>();
        let mut names = Vec::new();
        let mut unique_names = BTreeSet::new();
        for component in components.into_iter().chain(
            dependencies
                .iter()
                .flat_map(|dependency| [dependency.source.clone(), dependency.target.clone()]),
        ) {
            let component = validated_component_name(component)?;
            if unique_names.insert(component.clone()) {
                names.push(component);
            }
        }

        Ok(Self {
            components: names,
            dependencies,
        })
    }

    /// Returns whether the diagram allows this exact directed dependency.
    #[must_use]
    pub fn allows(&self, source: &str, target: &str) -> bool {
        self.dependencies
            .iter()
            .any(|dependency| dependency.source == source && dependency.target == target)
    }
}
