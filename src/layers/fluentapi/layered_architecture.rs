use std::collections::{BTreeMap, BTreeSet};

use crate::checkable::execute_logged_check;
use crate::{
    ArchUnitError, CheckOptions, CheckResult, Checkable, LayerDefinition, PatternError,
    ProjectLocator, UserError, Violation, extract_graph_with_options, gather_empty_test_violations,
    gather_layer_dependency_violations, locate_project_from, per_internal_edge, project_edges,
    project_to_nodes,
};

use super::{LayerDefinitionBuilder, LayerDependencyRuleBuilder};

#[derive(Debug, Clone)]
enum LayerConfigurationError {
    EmptyLayerName,
    InvalidSelector {
        layer_name: String,
        selector_kind: &'static str,
        source: PatternError,
    },
    UndefinedSourceLayer(String),
    EmptyTargetLayerName,
    UndefinedTargetLayers(Vec<String>),
    EmptyForbiddenTargets,
}

impl LayerConfigurationError {
    fn to_archunit_error(&self) -> ArchUnitError {
        let user_error = match self {
            Self::EmptyLayerName => UserError::new("layer names must not be empty"),
            Self::InvalidSelector {
                layer_name,
                selector_kind,
                source,
            } => UserError::with_source(
                format!("layer '{layer_name}' contains an invalid {selector_kind} selector"),
                source.clone(),
            ),
            Self::UndefinedSourceLayer(name) => UserError::new(format!(
                "layer '{name}' must be defined before it can have a dependency policy"
            )),
            Self::EmptyTargetLayerName => UserError::new("target layer names must not be empty"),
            Self::UndefinedTargetLayers(names) => UserError::new(format!(
                "undefined target layer{}: {}",
                if names.len() == 1 { "" } else { "s" },
                names.join(", ")
            )),
            Self::EmptyForbiddenTargets => {
                UserError::new("may_not_depend_on_layers requires at least one target layer")
            }
        };

        ArchUnitError::from(user_error)
    }
}

/// Immutable, executable named-layer dependency policy.
#[derive(Debug, Clone)]
#[must_use = "an architecture rule has no effect until it is checked"]
pub struct LayeredArchitecture {
    project_locator: ProjectLocator,
    layer_definitions: Vec<LayerDefinition>,
    allowed_dependencies: BTreeMap<String, BTreeSet<String>>,
    forbidden_dependencies: BTreeMap<String, BTreeSet<String>>,
    configuration_error: Option<LayerConfigurationError>,
}

impl LayeredArchitecture {
    pub(super) const fn new(project_locator: ProjectLocator) -> Self {
        Self {
            project_locator,
            layer_definitions: Vec::new(),
            allowed_dependencies: BTreeMap::new(),
            forbidden_dependencies: BTreeMap::new(),
            configuration_error: None,
        }
    }

    /// Enters the definition stage for `name`.
    ///
    /// Defining the same name again adds another OR selector without changing declaration order.
    pub fn layer(mut self, name: impl Into<String>) -> LayerDefinitionBuilder {
        let name = name.into();
        if name.trim().is_empty() {
            self.record_error(LayerConfigurationError::EmptyLayerName);
        }
        LayerDefinitionBuilder::new(self, name)
    }

    /// Enters the dependency-policy stage for a previously defined layer.
    pub fn where_layer(mut self, name: impl Into<String>) -> LayerDependencyRuleBuilder {
        let name = name.into();
        if name.trim().is_empty() {
            self.record_error(LayerConfigurationError::EmptyLayerName);
        } else if !self.has_layer(&name) {
            self.record_error(LayerConfigurationError::UndefinedSourceLayer(name.clone()));
        }
        LayerDependencyRuleBuilder::new(self, name)
    }

    /// Returns where Cargo project discovery begins.
    #[must_use]
    pub const fn project_locator(&self) -> &ProjectLocator {
        &self.project_locator
    }

    /// Returns named layer definitions in declaration order.
    #[must_use]
    pub fn layer_definitions(&self) -> &[LayerDefinition] {
        &self.layer_definitions
    }

    /// Returns the complete allowlist policy keyed by source layer.
    #[must_use]
    pub const fn allowed_dependencies(&self) -> &BTreeMap<String, BTreeSet<String>> {
        &self.allowed_dependencies
    }

    /// Returns the complete blocklist policy keyed by source layer.
    #[must_use]
    pub const fn forbidden_dependencies(&self) -> &BTreeMap<String, BTreeSet<String>> {
        &self.forbidden_dependencies
    }

    pub(super) fn with_layer_filter(
        mut self,
        layer_name: String,
        filter: Result<crate::Filter, PatternError>,
        selector_kind: &'static str,
    ) -> Self {
        if self.configuration_error.is_some() {
            return self;
        }

        let filter = match filter {
            Ok(filter) => filter,
            Err(source) => {
                self.record_error(LayerConfigurationError::InvalidSelector {
                    layer_name,
                    selector_kind,
                    source,
                });
                return self;
            }
        };

        if let Some(definition) = self
            .layer_definitions
            .iter_mut()
            .find(|definition| definition.name == layer_name)
        {
            definition.filters.push(filter);
        } else {
            self.layer_definitions
                .push(LayerDefinition::new(layer_name, [filter]));
        }
        self
    }

    pub(super) fn with_allowed_dependencies(
        mut self,
        source_layer: String,
        target_layers: &[&str],
    ) -> Self {
        if let Some(targets) = self.validated_targets(target_layers) {
            self.allowed_dependencies.insert(source_layer, targets);
        }
        self
    }

    pub(super) fn with_forbidden_dependencies(
        mut self,
        source_layer: String,
        target_layers: &[&str],
    ) -> Self {
        if target_layers.is_empty() {
            self.record_error(LayerConfigurationError::EmptyForbiddenTargets);
            return self;
        }

        if let Some(targets) = self.validated_targets(target_layers) {
            self.forbidden_dependencies
                .entry(source_layer)
                .or_default()
                .extend(targets);
        }
        self
    }

    fn validated_targets(&mut self, target_layers: &[&str]) -> Option<BTreeSet<String>> {
        if self.configuration_error.is_some() {
            return None;
        }
        if target_layers.iter().any(|name| name.trim().is_empty()) {
            self.record_error(LayerConfigurationError::EmptyTargetLayerName);
            return None;
        }

        let targets = target_layers
            .iter()
            .map(|name| (*name).to_owned())
            .collect::<BTreeSet<_>>();
        let undefined = targets
            .iter()
            .filter(|name| !self.has_layer(name))
            .cloned()
            .collect::<Vec<_>>();
        if !undefined.is_empty() {
            self.record_error(LayerConfigurationError::UndefinedTargetLayers(undefined));
            return None;
        }

        Some(targets)
    }

    fn has_layer(&self, name: &str) -> bool {
        self.layer_definitions
            .iter()
            .any(|definition| definition.name == name)
    }

    fn record_error(&mut self, error: LayerConfigurationError) {
        if self.configuration_error.is_none() {
            self.configuration_error = Some(error);
        }
    }

    fn policy_source_layers(&self) -> BTreeSet<&str> {
        self.allowed_dependencies
            .keys()
            .chain(self.forbidden_dependencies.keys())
            .map(String::as_str)
            .collect()
    }
}

impl Checkable for LayeredArchitecture {
    fn check_with(&self, options: &CheckOptions) -> CheckResult {
        execute_logged_check("layers.dependencies", options, |logger| {
            if let Some(error) = &self.configuration_error {
                return Err(error.to_archunit_error());
            }

            logger.log_progress("extracting project graph")?;
            let project = locate_project_from(self.project_locator())?;
            let extraction = extract_graph_with_options(&project, options)?;
            let nodes = project_to_nodes(extraction.graph());
            logger.log_progress(format!("project files={}", nodes.len()))?;
            let mut violations = Vec::new();

            for source_layer in self.policy_source_layers() {
                if let Some(definition) = self
                    .layer_definitions
                    .iter()
                    .find(|definition| definition.name == source_layer)
                {
                    let selected = nodes
                        .iter()
                        .filter(|node| definition.matches(&node.label))
                        .collect::<Vec<_>>();
                    logger.log_progress(format!(
                        "layer {source_layer}; selected files={}",
                        selected.len()
                    ))?;
                    violations.extend(
                        gather_empty_test_violations(
                            &selected,
                            format!("layer '{source_layer}'"),
                            &definition.filters,
                            false,
                            options.allows_empty_tests(),
                        )
                        .into_iter()
                        .map(Violation::from),
                    );
                }
            }

            let edges = project_edges(extraction.graph(), per_internal_edge());
            logger.log_progress(format!("internal dependencies={}", edges.len()))?;
            violations.extend(
                gather_layer_dependency_violations(
                    &edges,
                    &self.layer_definitions,
                    &self.allowed_dependencies,
                    &self.forbidden_dependencies,
                )
                .into_iter()
                .map(Violation::from),
            );

            Ok(violations)
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{ArchUnitError, Checkable, PatternTarget, layers, layers_in};

    #[test]
    fn definitions_are_branchable_and_repeated_names_add_or_selectors() {
        let base = layers();
        let api = base.clone().layer("api").defined_by_folder("src/api");
        let expanded = api.clone().layer("api").defined_by("src/legacy/api/**");

        assert!(base.layer_definitions().is_empty());
        assert_eq!(api.layer_definitions().len(), 1);
        assert_eq!(api.layer_definitions()[0].filters.len(), 1);
        assert_eq!(expanded.layer_definitions()[0].filters.len(), 2);
        assert_eq!(
            expanded.layer_definitions()[0].filters[0].target(),
            PatternTarget::PathWithoutFilename
        );
        assert_eq!(
            expanded.layer_definitions()[0].filters[1].target(),
            PatternTarget::Path
        );
    }

    #[test]
    fn allowlists_replace_and_blocklists_accumulate_deterministically() {
        let definitions = layers()
            .layer("api")
            .defined_by("src/api/**")
            .layer("services")
            .defined_by("src/services/**")
            .layer("database")
            .defined_by("src/database/**");
        let policy = definitions
            .where_layer("api")
            .may_only_depend_on_layers(&["services"])
            .where_layer("api")
            .may_only_depend_on_layers(&["database"])
            .where_layer("api")
            .may_not_depend_on_layers(&["services"])
            .where_layer("api")
            .may_not_depend_on_layers(&["database", "services"]);

        assert_eq!(
            policy.allowed_dependencies()["api"]
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            ["database"]
        );
        assert_eq!(
            policy.forbidden_dependencies()["api"]
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            ["database", "services"]
        );
    }

    #[test]
    fn sealed_layer_is_represented_by_an_empty_allowlist() {
        let policy = layers()
            .layer("database")
            .defined_by("src/database/**")
            .where_layer("database")
            .may_only_depend_on_layers(&[]);

        assert!(policy.allowed_dependencies()["database"].is_empty());
    }

    #[test]
    fn first_configuration_error_precedes_project_location() {
        let invalid_selector = layers_in("definitely/missing")
            .layer("api")
            .defined_by("src/[api")
            .where_layer("missing")
            .may_only_depend_on_layers(&[])
            .check();
        let error = invalid_selector.expect_err("invalid selector should fail before extraction");

        assert!(matches!(error, ArchUnitError::User(_)));
        assert!(error.to_string().contains("invalid path selector"));
        assert!(error.to_string().contains("src/[api"));
    }

    #[test]
    fn undefined_and_empty_policy_references_are_user_errors() {
        let definitions = layers_in("definitely/missing")
            .layer("api")
            .defined_by("src/api/**");
        let cases = [
            definitions
                .clone()
                .where_layer("missing")
                .may_only_depend_on_layers(&[]),
            definitions
                .clone()
                .where_layer("api")
                .may_only_depend_on_layers(&["missing"]),
            definitions.where_layer("api").may_not_depend_on_layers(&[]),
        ];

        for rule in cases {
            assert!(matches!(rule.check(), Err(ArchUnitError::User(_))));
        }
    }
}
