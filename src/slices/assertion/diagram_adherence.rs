use crate::{common::ProjectedEdge, slices::PlantUmlDiagram};

use super::{DiagramAdherenceOptions, SliceDependencyRule, SliceDependencyViolation};

/// Collects actual slice dependencies not permitted by the supplied diagram.
#[must_use]
pub fn gather_diagram_adherence_violations(
    edges: &[ProjectedEdge],
    diagram: &PlantUmlDiagram,
    options: &DiagramAdherenceOptions,
) -> Vec<SliceDependencyViolation> {
    edges
        .iter()
        .filter(|edge| !ignored(edge, diagram, options))
        .filter(|edge| !diagram.allows(&edge.source_label, &edge.target_label))
        .cloned()
        .map(|edge| {
            let source = edge.source_label.clone();
            let target = edge.target_label.clone();
            SliceDependencyViolation::new(
                edge,
                source,
                target,
                SliceDependencyRule::AdhereToDiagram,
                false,
            )
        })
        .collect()
}

fn ignored(
    edge: &ProjectedEdge,
    diagram: &PlantUmlDiagram,
    options: &DiagramAdherenceOptions,
) -> bool {
    if options.ignore_external_slices && edge.cumulated_edges.iter().any(|edge| edge.external) {
        return true;
    }
    options.ignore_orphan_slices
        && (!diagram.components.contains(&edge.source_label)
            || !diagram.components.contains(&edge.target_label))
}

#[cfg(test)]
mod tests {
    use crate::{
        common::{Edge, ImportKind, ProjectedEdge},
        slices::{PlantUmlDependency, PlantUmlDiagram},
    };

    use super::{DiagramAdherenceOptions, gather_diagram_adherence_violations};

    fn edge(source: &str, target: &str, external: bool) -> ProjectedEdge {
        ProjectedEdge::new(
            source,
            target,
            [Edge::new(source, target, external, [ImportKind::Use])],
        )
    }

    fn diagram() -> PlantUmlDiagram {
        PlantUmlDiagram::new(
            ["api", "services", "database"]
                .into_iter()
                .map(str::to_owned),
            [PlantUmlDependency::new("api", "services")
                .expect("fixture dependency should be valid")],
        )
        .expect("fixture diagram should be valid")
    }

    #[test]
    fn reports_every_actual_dependency_the_diagram_does_not_allow() {
        let edges = [
            edge("api", "services", false),
            edge("api", "database", false),
            edge("api", "serde", true),
        ];

        let violations = gather_diagram_adherence_violations(
            &edges,
            &diagram(),
            &DiagramAdherenceOptions::default(),
        );

        assert_eq!(
            violations
                .iter()
                .map(|violation| violation.target_slice.as_str())
                .collect::<Vec<_>>(),
            ["database", "serde"]
        );
        assert!(violations.iter().all(|violation| !violation.is_negated));
    }

    #[test]
    fn external_and_orphan_modifiers_are_independent_immutable_values() {
        let edges = [edge("api", "database", false), edge("api", "serde", true)];
        let base = DiagramAdherenceOptions::new();
        let external = base.with_external_slices_ignored(true);
        let orphans = base.with_orphan_slices_ignored(true);

        assert_eq!(
            gather_diagram_adherence_violations(&edges, &diagram(), &external).len(),
            1
        );
        assert_eq!(
            gather_diagram_adherence_violations(&edges, &diagram(), &orphans).len(),
            1
        );
        assert!(!base.ignore_external_slices);
        assert!(!base.ignore_orphan_slices);
    }
}
