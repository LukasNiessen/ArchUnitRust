use super::PlantUmlError;

/// One allowed directed dependency in a PlantUML component diagram.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PlantUmlDependency {
    /// Source component name.
    pub source: String,
    /// Target component name.
    pub target: String,
}

impl PlantUmlDependency {
    /// Creates one dependency after validating both component names.
    pub fn new(
        source: impl Into<String>,
        target: impl Into<String>,
    ) -> Result<Self, PlantUmlError> {
        Ok(Self {
            source: validated_component_name(source.into())?,
            target: validated_component_name(target.into())?,
        })
    }
}

pub(super) fn validated_component_name(value: String) -> Result<String, PlantUmlError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(PlantUmlError::new("component names must not be empty"));
    }
    if value.contains([']', '\r', '\n']) {
        return Err(PlantUmlError::new(
            "component names must not contain ']' or a line break",
        ));
    }
    Ok(value.to_owned())
}
