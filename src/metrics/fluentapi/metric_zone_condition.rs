use std::path::Path;

use crate::{
    ArchitecturalZone, CheckOptions, CheckResult, Checkable, ProjectLocator, Violation,
    gather_empty_test_violations, gather_metric_zone_violations,
};

use super::MetricsBuilder;

/// Executable distance rule that rejects one architectural zone.
#[derive(Debug, Clone)]
#[must_use = "an architecture rule has no effect until it is checked"]
pub struct MetricZoneCondition {
    query: MetricsBuilder,
    zone: ArchitecturalZone,
}

impl MetricZoneCondition {
    pub(super) const fn new(query: MetricsBuilder, zone: ArchitecturalZone) -> Self {
        Self { query, zone }
    }

    /// Returns the zone rejected by this rule.
    #[must_use]
    pub const fn zone(&self) -> ArchitecturalZone {
        self.zone
    }

    /// Returns the explicit discovery path, or `None` for automatic discovery.
    #[must_use]
    pub fn project_path(&self) -> Option<&Path> {
        self.query.project_path()
    }

    /// Returns where project discovery starts.
    #[must_use]
    pub fn project_locator(&self) -> ProjectLocator {
        self.query
            .project_path()
            .map_or_else(ProjectLocator::auto_detect, ProjectLocator::from_path)
    }
}

impl Checkable for MetricZoneCondition {
    fn check_with(&self, options: &CheckOptions) -> CheckResult {
        let infos = self.query.distance_infos_with(options)?;
        let empty = gather_empty_test_violations(
            &infos,
            "metric components",
            self.query.filters(),
            false,
            options.allows_empty_tests(),
        );
        if let Some(violation) = empty.into_iter().next() {
            return Ok(vec![Violation::from(violation)]);
        }

        Ok(gather_metric_zone_violations(&infos, self.zone)
            .into_iter()
            .map(Violation::from)
            .collect())
    }
}
