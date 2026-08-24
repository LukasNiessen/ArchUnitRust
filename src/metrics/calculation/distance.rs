use std::fmt;

use crate::{DistanceInfo, MetricMeasurement, MetricSubject};

/// Strict upper boundary for abstractness and instability in the zone of pain.
pub const PAIN_LIMIT: f64 = 0.3;
/// Strict lower boundary for abstractness and instability in the zone of uselessness.
pub const USELESSNESS_LIMIT: f64 = 0.7;
/// File lines at which the normalized-distance size discount reaches its cap.
pub const SIZE_NORMALIZATION_LINES: f64 = 100.0;
/// Largest discount applied to distance by normalized distance.
pub const MAXIMUM_SIZE_DISCOUNT: f64 = 0.5;

/// Immutable numeric input for Robert C. Martin's component-distance formulas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct DistanceInput {
    concrete_types: usize,
    traits: usize,
    afferent_coupling: usize,
    efferent_coupling: usize,
    project_component_count: usize,
    lines_of_code: usize,
}

impl DistanceInput {
    /// Creates a complete formula input.
    #[must_use]
    pub const fn new(
        concrete_types: usize,
        traits: usize,
        afferent_coupling: usize,
        efferent_coupling: usize,
        project_component_count: usize,
        lines_of_code: usize,
    ) -> Self {
        Self {
            concrete_types,
            traits,
            afferent_coupling,
            efferent_coupling,
            project_component_count,
            lines_of_code,
        }
    }

    /// Builds a formula input from extracted file syntax and coupling evidence.
    #[must_use]
    pub const fn from_distance_info(info: &DistanceInfo) -> Self {
        Self::new(
            info.file().concrete_types(),
            info.file().traits(),
            info.afferent_coupling(),
            info.efferent_coupling(),
            info.project_file_count(),
            info.file().lines_of_code(),
        )
    }

    /// Returns concrete struct, enum, and union declarations.
    #[must_use]
    pub const fn concrete_types(self) -> usize {
        self.concrete_types
    }

    /// Returns trait declarations.
    #[must_use]
    pub const fn traits(self) -> usize {
        self.traits
    }

    /// Returns incoming coupling.
    #[must_use]
    pub const fn afferent_coupling(self) -> usize {
        self.afferent_coupling
    }

    /// Returns outgoing coupling.
    #[must_use]
    pub const fn efferent_coupling(self) -> usize {
        self.efferent_coupling
    }

    /// Returns the size of the full component universe.
    #[must_use]
    pub const fn project_component_count(self) -> usize {
        self.project_component_count
    }

    /// Returns physical non-comment source lines.
    #[must_use]
    pub const fn lines_of_code(self) -> usize {
        self.lines_of_code
    }
}

/// One metric in the abstractness/instability component-distance family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum DistanceMetric {
    /// Ratio of traits to all declared types.
    Abstractness,
    /// Ratio of outgoing coupling to total coupling.
    Instability,
    /// Absolute distance from `abstractness + instability = 1`.
    DistanceFromMainSequence,
    /// Observed bidirectional coupling relative to its project maximum.
    CouplingFactor,
    /// Main-sequence distance discounted by component source size.
    NormalizedDistance,
}

impl DistanceMetric {
    /// Every distance metric in stable display order.
    pub const ALL: [Self; 5] = [
        Self::Abstractness,
        Self::Instability,
        Self::DistanceFromMainSequence,
        Self::CouplingFactor,
        Self::NormalizedDistance,
    ];

    /// Returns the stable public metric name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Abstractness => "abstractness",
            Self::Instability => "instability",
            Self::DistanceFromMainSequence => "distance_from_main_sequence",
            Self::CouplingFactor => "coupling_factor",
            Self::NormalizedDistance => "normalized_distance",
        }
    }

    /// Returns the metric's concise semantic description.
    #[must_use]
    pub const fn description(self) -> &'static str {
        match self {
            Self::Abstractness => "trait declarations divided by all type declarations",
            Self::Instability => "efferent coupling divided by total coupling",
            Self::DistanceFromMainSequence => {
                "absolute abstractness-instability distance from the main sequence"
            }
            Self::CouplingFactor => "distinct bidirectional internal coupling density",
            Self::NormalizedDistance => "main-sequence distance with a capped source-size discount",
        }
    }

    /// Calculates this formula over prepared component facts.
    #[must_use]
    pub fn calculate(self, input: &DistanceInput) -> f64 {
        match self {
            Self::Abstractness => abstractness(input),
            Self::Instability => instability(input),
            Self::DistanceFromMainSequence => distance_from_main_sequence(input),
            Self::CouplingFactor => coupling_factor(input),
            Self::NormalizedDistance => normalized_distance(input),
        }
    }

    pub(crate) fn measurements(self, infos: &[DistanceInfo]) -> Vec<MetricMeasurement> {
        infos
            .iter()
            .cloned()
            .map(|info| {
                let input = DistanceInput::from_distance_info(&info);
                MetricMeasurement::from_parts(
                    MetricSubject::Distance(info),
                    self.name(),
                    self.description(),
                    self.calculate(&input),
                )
            })
            .collect()
    }
}

impl fmt::Display for DistanceMetric {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name())
    }
}

/// One rejected region in the abstractness/instability plane.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ArchitecturalZone {
    /// Low abstractness and low instability.
    Pain,
    /// High abstractness and high instability.
    Uselessness,
}

impl ArchitecturalZone {
    /// Returns the stable public zone name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Pain => "pain",
            Self::Uselessness => "uselessness",
        }
    }

    /// Returns whether the input lies strictly inside this zone.
    #[must_use]
    pub fn contains(self, input: &DistanceInput) -> bool {
        let abstractness = abstractness(input);
        let instability = instability(input);
        match self {
            Self::Pain => abstractness < PAIN_LIMIT && instability < PAIN_LIMIT,
            Self::Uselessness => {
                abstractness > USELESSNESS_LIMIT && instability > USELESSNESS_LIMIT
            }
        }
    }
}

impl fmt::Display for ArchitecturalZone {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name())
    }
}

fn abstractness(input: &DistanceInput) -> f64 {
    let type_count = input.concrete_types + input.traits;
    if type_count == 0 {
        0.0
    } else {
        input.traits as f64 / type_count as f64
    }
}

fn instability(input: &DistanceInput) -> f64 {
    let total = input.afferent_coupling + input.efferent_coupling;
    if total == 0 {
        0.0
    } else {
        input.efferent_coupling as f64 / total as f64
    }
}

fn distance_from_main_sequence(input: &DistanceInput) -> f64 {
    (abstractness(input) + instability(input) - 1.0).abs()
}

fn coupling_factor(input: &DistanceInput) -> f64 {
    let possible = 2 * input.project_component_count.saturating_sub(1);
    if possible == 0 {
        0.0
    } else {
        (input.afferent_coupling + input.efferent_coupling) as f64 / possible as f64
    }
}

fn normalized_distance(input: &DistanceInput) -> f64 {
    let size_ratio = (input.lines_of_code as f64 / SIZE_NORMALIZATION_LINES).min(1.0);
    distance_from_main_sequence(input) * (1.0 - size_ratio * MAXIMUM_SIZE_DISCOUNT)
}

#[cfg(test)]
mod tests {
    use super::{ArchitecturalZone, DistanceInput, DistanceMetric};

    fn assert_close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < 0.000_001,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn calculates_the_complete_family_for_a_balanced_component() {
        let input = DistanceInput::new(2, 2, 1, 3, 5, 50);
        let expected = [0.5, 0.75, 0.25, 0.5, 0.1875];

        for (metric, expected) in DistanceMetric::ALL.into_iter().zip(expected) {
            assert_close(metric.calculate(&input), expected);
        }
    }

    #[test]
    fn defines_zero_denominators_without_non_finite_values() {
        let input = DistanceInput::new(0, 0, 0, 0, 1, 0);

        assert_eq!(DistanceMetric::Abstractness.calculate(&input), 0.0);
        assert_eq!(DistanceMetric::Instability.calculate(&input), 0.0);
        assert_eq!(DistanceMetric::CouplingFactor.calculate(&input), 0.0);
        assert_eq!(
            DistanceMetric::DistanceFromMainSequence.calculate(&input),
            1.0
        );
        assert_eq!(DistanceMetric::NormalizedDistance.calculate(&input), 1.0);
    }

    #[test]
    fn caps_the_normalized_distance_discount_at_half() {
        let medium = DistanceInput::new(1, 0, 0, 0, 2, 50);
        let large = DistanceInput::new(1, 0, 0, 0, 2, 500);

        assert_eq!(DistanceMetric::NormalizedDistance.calculate(&medium), 0.75);
        assert_eq!(DistanceMetric::NormalizedDistance.calculate(&large), 0.5);
    }

    #[test]
    fn zones_use_strict_boundaries() {
        let pain = DistanceInput::new(3, 1, 3, 1, 5, 1);
        let pain_boundary = DistanceInput::new(7, 3, 7, 3, 11, 1);
        let uselessness = DistanceInput::new(1, 3, 1, 3, 5, 1);
        let uselessness_boundary = DistanceInput::new(3, 7, 3, 7, 11, 1);

        assert!(ArchitecturalZone::Pain.contains(&pain));
        assert!(!ArchitecturalZone::Pain.contains(&pain_boundary));
        assert!(ArchitecturalZone::Uselessness.contains(&uselessness));
        assert!(!ArchitecturalZone::Uselessness.contains(&uselessness_boundary));
    }

    #[test]
    fn names_descriptions_and_display_are_stable() {
        assert_eq!(
            DistanceMetric::ALL.map(DistanceMetric::name),
            [
                "abstractness",
                "instability",
                "distance_from_main_sequence",
                "coupling_factor",
                "normalized_distance"
            ]
        );
        for metric in DistanceMetric::ALL {
            assert!(!metric.description().is_empty());
            assert_eq!(metric.to_string(), metric.name());
        }
        assert_eq!(ArchitecturalZone::Pain.to_string(), "pain");
        assert_eq!(ArchitecturalZone::Uselessness.to_string(), "uselessness");
    }
}
