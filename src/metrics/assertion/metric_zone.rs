use crate::metrics::{ArchitecturalZone, DistanceInfo, DistanceInput, DistanceMetric};

/// A file component whose abstractness/instability point lies in a discouraged zone.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct MetricZoneViolation {
    /// The complete file syntax and coupling evidence.
    pub distance_info: DistanceInfo,
    /// The zone that rejected the component.
    pub zone: ArchitecturalZone,
    /// Abstractness at the time the violation was gathered.
    pub abstractness: f64,
    /// Instability at the time the violation was gathered.
    pub instability: f64,
}

impl MetricZoneViolation {
    /// Creates violation data and derives its coordinates from the supplied evidence.
    #[must_use]
    pub fn new(distance_info: DistanceInfo, zone: ArchitecturalZone) -> Self {
        let input = DistanceInput::from_distance_info(&distance_info);
        Self {
            distance_info,
            zone,
            abstractness: DistanceMetric::Abstractness.calculate(&input),
            instability: DistanceMetric::Instability.calculate(&input),
        }
    }
}

/// Returns one structured violation for every component inside `zone`.
#[must_use]
pub fn gather_metric_zone_violations(
    distance_infos: &[DistanceInfo],
    zone: ArchitecturalZone,
) -> Vec<MetricZoneViolation> {
    distance_infos
        .iter()
        .filter(|info| zone.contains(&DistanceInput::from_distance_info(info)))
        .cloned()
        .map(|info| MetricZoneViolation::new(info, zone))
        .collect()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{
        common::{Edge, Graph, ImportKind},
        metrics::{
            ArchitecturalZone, ProjectMetricsInfo, build_distance_infos, extract_file_metrics,
        },
    };

    use super::gather_metric_zone_violations;

    #[test]
    fn gathers_only_components_inside_the_requested_zone() {
        let metrics = ProjectMetricsInfo::from_files(
            PathBuf::from("fixture"),
            vec![
                extract_file_metrics("src/stable.rs", "struct Stable;").expect("valid fixture"),
                extract_file_metrics("src/abstract.rs", "trait Abstract {}")
                    .expect("valid fixture"),
            ],
        );
        let graph = Graph::from_edges([
            Edge::self_edge("src/stable.rs"),
            Edge::new("src/abstract.rs", "src/stable.rs", false, [ImportKind::Use]),
        ]);
        let infos = build_distance_infos(&metrics, &graph);

        let pain = gather_metric_zone_violations(&infos, ArchitecturalZone::Pain);
        let uselessness = gather_metric_zone_violations(&infos, ArchitecturalZone::Uselessness);

        assert_eq!(pain.len(), 1);
        assert_eq!(pain[0].distance_info.identifier(), "src/stable.rs");
        assert_eq!(pain[0].abstractness, 0.0);
        assert_eq!(pain[0].instability, 0.0);
        assert_eq!(uselessness.len(), 1);
        assert_eq!(uselessness[0].distance_info.identifier(), "src/abstract.rs");
        assert_eq!(uselessness[0].abstractness, 1.0);
        assert_eq!(uselessness[0].instability, 1.0);
    }
}
