use std::collections::{BTreeMap, BTreeSet};

use crate::ProjectedEdge;

use super::{LayerDefinition, LayerDependencyRule, LayerDependencyViolation};

/// Collects rejected cross-layer dependencies from an already projected file graph.
///
/// Intra-layer edges and edges with an unassigned endpoint are ignored. Blocklists take precedence
/// over allowlists so one concrete dependency produces at most one violation.
#[must_use]
pub fn gather_layer_dependency_violations(
    edges: &[ProjectedEdge],
    layers: &[LayerDefinition],
    allowed_dependencies: &BTreeMap<String, BTreeSet<String>>,
    forbidden_dependencies: &BTreeMap<String, BTreeSet<String>>,
) -> Vec<LayerDependencyViolation> {
    edges
        .iter()
        .filter_map(|edge| {
            let source_layer = find_layer(&edge.source_label, layers)?;
            let target_layer = find_layer(&edge.target_label, layers)?;

            if source_layer.name == target_layer.name {
                return None;
            }

            let rule = violated_rule(
                &source_layer.name,
                &target_layer.name,
                allowed_dependencies,
                forbidden_dependencies,
            )?;

            Some(LayerDependencyViolation::new(
                edge.clone(),
                &source_layer.name,
                &target_layer.name,
                rule,
            ))
        })
        .collect()
}

fn find_layer<'a>(file_path: &str, layers: &'a [LayerDefinition]) -> Option<&'a LayerDefinition> {
    layers.iter().find(|layer| layer.matches(file_path))
}

fn violated_rule(
    source_layer: &str,
    target_layer: &str,
    allowed_dependencies: &BTreeMap<String, BTreeSet<String>>,
    forbidden_dependencies: &BTreeMap<String, BTreeSet<String>>,
) -> Option<LayerDependencyRule> {
    if forbidden_dependencies
        .get(source_layer)
        .is_some_and(|targets| targets.contains(target_layer))
    {
        return Some(LayerDependencyRule::MayNotDependOnLayers);
    }

    allowed_dependencies
        .get(source_layer)
        .filter(|targets| !targets.contains(target_layer))
        .map(|_| LayerDependencyRule::MayOnlyDependOnLayers)
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use crate::{Edge, ImportKind, ProjectedEdge, RegexFactory};

    use super::{LayerDefinition, LayerDependencyRule, gather_layer_dependency_violations};

    fn edge(source: &str, target: &str) -> ProjectedEdge {
        ProjectedEdge::new(
            source,
            target,
            [Edge::new(source, target, false, [ImportKind::Use])],
        )
    }

    fn layer(name: &str, folder: &str) -> LayerDefinition {
        LayerDefinition::new(
            name,
            [RegexFactory::default()
                .folder_matcher(folder)
                .expect("fixture folder should compile")],
        )
    }

    fn layers() -> Vec<LayerDefinition> {
        vec![
            layer("api", "src/api"),
            layer("services", "src/services"),
            layer("database", "src/database"),
        ]
    }

    fn policy(source: &str, targets: &[&str]) -> BTreeMap<String, BTreeSet<String>> {
        BTreeMap::from([(
            source.to_owned(),
            targets.iter().map(|target| (*target).to_owned()).collect(),
        )])
    }

    #[test]
    fn allowlists_reject_only_cross_layer_targets_outside_the_set() {
        let allowed = edge("src/api/handler.rs", "src/services/orders.rs");
        let rejected = edge("src/api/handler.rs", "src/database/store.rs");
        let intra_layer = edge("src/api/handler.rs", "src/api/model.rs");

        let violations = gather_layer_dependency_violations(
            &[allowed, rejected.clone(), intra_layer],
            &layers(),
            &policy("api", &["services"]),
            &BTreeMap::new(),
        );

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].dependency, rejected);
        assert_eq!(
            violations[0].rule,
            LayerDependencyRule::MayOnlyDependOnLayers
        );
    }

    #[test]
    fn empty_allowlist_seals_a_layer() {
        let violations = gather_layer_dependency_violations(
            &[edge("src/api/handler.rs", "src/services/orders.rs")],
            &layers(),
            &policy("api", &[]),
            &BTreeMap::new(),
        );

        assert_eq!(violations.len(), 1);
        assert_eq!(
            violations[0].rule,
            LayerDependencyRule::MayOnlyDependOnLayers
        );
    }

    #[test]
    fn blocklist_precedes_allowlist_and_emits_one_violation() {
        let violations = gather_layer_dependency_violations(
            &[edge("src/api/handler.rs", "src/services/orders.rs")],
            &layers(),
            &policy("api", &["database"]),
            &policy("api", &["services"]),
        );

        assert_eq!(violations.len(), 1);
        assert_eq!(
            violations[0].rule,
            LayerDependencyRule::MayNotDependOnLayers
        );
    }

    #[test]
    fn unassigned_endpoints_are_ignored() {
        let violations = gather_layer_dependency_violations(
            &[
                edge("src/jobs/sync.rs", "src/database/store.rs"),
                edge("src/api/handler.rs", "src/support/log.rs"),
            ],
            &layers(),
            &policy("api", &[]),
            &policy("database", &["api"]),
        );

        assert!(violations.is_empty());
    }

    #[test]
    fn first_declared_layer_wins_when_definitions_overlap() {
        let mut overlapping = vec![layer("application", "src/**")];
        overlapping.extend(layers());

        let violations = gather_layer_dependency_violations(
            &[edge("src/api/handler.rs", "src/database/store.rs")],
            &overlapping,
            &policy("application", &[]),
            &BTreeMap::new(),
        );

        assert!(violations.is_empty());
    }
}
