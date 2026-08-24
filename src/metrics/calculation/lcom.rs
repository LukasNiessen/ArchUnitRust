use std::{collections::BTreeSet, fmt};

use crate::metrics::{MetricMeasurement, MetricSubject, ProjectMetricsInfo, TypeInfo, TypeKind};

/// Immutable method/field incidence data used by every LCOM formula.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct LcomInput {
    fields: BTreeSet<String>,
    method_field_accesses: Vec<BTreeSet<String>>,
}

impl LcomInput {
    /// Creates a formula input from declared fields and one accessed-field list per method.
    ///
    /// Duplicate field names and duplicate accesses are removed. Accesses to undeclared fields are
    /// ignored so every formula uses one consistent method/field incidence relation.
    #[must_use]
    pub fn new(fields: Vec<String>, method_field_accesses: Vec<Vec<String>>) -> Self {
        let fields = fields.into_iter().collect::<BTreeSet<_>>();
        let method_field_accesses = method_field_accesses
            .into_iter()
            .map(|accesses| {
                accesses
                    .into_iter()
                    .filter(|field| fields.contains(field))
                    .collect()
            })
            .collect();
        Self {
            fields,
            method_field_accesses,
        }
    }

    /// Builds an input for an eligible Rust metrics type.
    ///
    /// Only structs with at least one unambiguously associated inherent method are eligible. Trait
    /// impl methods are intentionally absent from the resulting method population.
    #[must_use]
    pub fn from_type_info(type_info: &TypeInfo) -> Option<Self> {
        if type_info.kind() != TypeKind::Struct || type_info.inherent_methods().is_empty() {
            return None;
        }

        Some(Self::new(
            type_info
                .fields()
                .iter()
                .map(|field| field.name().to_owned())
                .collect(),
            type_info
                .inherent_methods()
                .iter()
                .map(|method| method.accessed_fields().to_vec())
                .collect(),
        ))
    }

    /// Returns the number of declared fields.
    #[must_use]
    pub fn field_count(&self) -> usize {
        self.fields.len()
    }

    /// Returns the number of inherent methods represented by the input.
    #[must_use]
    pub fn method_count(&self) -> usize {
        self.method_field_accesses.len()
    }

    /// Returns the number of distinct method-to-declared-field accesses.
    #[must_use]
    pub fn field_access_count(&self) -> usize {
        self.method_field_accesses.iter().map(BTreeSet::len).sum()
    }
}

/// One formula in the lack-of-cohesion-of-methods family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum LcomMetric {
    /// Henderson-Sellers normalized method/field distance, 1996a spelling.
    Lcom96a,
    /// Henderson-Sellers method/field density complement, 1996b spelling.
    Lcom96b,
    /// Difference between non-sharing and sharing method-pair counts.
    Lcom1,
    /// Method/field density complement.
    Lcom2,
    /// Normalized method/field distance.
    Lcom3,
    /// Connected components in the shared-field method graph.
    Lcom4,
    /// Henderson-Sellers normalized method/field distance.
    Lcom5,
    /// LCOM-star normalized method/field distance.
    LcomStar,
}

impl LcomMetric {
    /// Every LCOM metric in stable display order.
    pub const ALL: [Self; 8] = [
        Self::Lcom96a,
        Self::Lcom96b,
        Self::Lcom1,
        Self::Lcom2,
        Self::Lcom3,
        Self::Lcom4,
        Self::Lcom5,
        Self::LcomStar,
    ];

    /// Returns the stable public metric name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Lcom96a => "lcom96a",
            Self::Lcom96b => "lcom96b",
            Self::Lcom1 => "lcom1",
            Self::Lcom2 => "lcom2",
            Self::Lcom3 => "lcom3",
            Self::Lcom4 => "lcom4",
            Self::Lcom5 => "lcom5",
            Self::LcomStar => "lcom_star",
        }
    }

    /// Returns the formula's stable description.
    #[must_use]
    pub const fn description(self) -> &'static str {
        match self {
            Self::Lcom96a => "normalized method-field distance (Henderson-Sellers 1996a)",
            Self::Lcom96b => "method-field density complement (Henderson-Sellers 1996b)",
            Self::Lcom1 => "non-sharing minus sharing method pairs",
            Self::Lcom2 => "method-field density complement",
            Self::Lcom3 => "normalized method-field distance",
            Self::Lcom4 => "shared-field method graph connected components",
            Self::Lcom5 => "normalized method-field distance (Henderson-Sellers)",
            Self::LcomStar => "LCOM-star normalized method-field distance",
        }
    }

    /// Calculates this formula over a prepared method/field incidence relation.
    ///
    /// Zero methods produce zero for every formula. One method produces one only for LCOM4 and zero
    /// otherwise. With multiple methods and no fields, normalized/density formulas produce zero,
    /// while LCOM1 and LCOM4 retain their method-pair and component meanings.
    #[must_use]
    pub fn calculate(self, input: &LcomInput) -> f64 {
        match self {
            Self::Lcom96a | Self::Lcom3 | Self::Lcom5 | Self::LcomStar => {
                normalized_method_field_distance(input)
            }
            Self::Lcom96b | Self::Lcom2 => method_field_density_complement(input),
            Self::Lcom1 => pair_difference(input),
            Self::Lcom4 => connected_components(input) as f64,
        }
    }

    pub(crate) fn measurements(self, project: &ProjectMetricsInfo) -> Vec<MetricMeasurement> {
        project
            .types()
            .iter()
            .filter_map(|type_info| {
                LcomInput::from_type_info(type_info).map(|input| {
                    MetricMeasurement::from_parts(
                        MetricSubject::Type(type_info.clone()),
                        self.name(),
                        self.description(),
                        self.calculate(&input),
                    )
                })
            })
            .collect()
    }
}

impl fmt::Display for LcomMetric {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name())
    }
}

fn normalized_method_field_distance(input: &LcomInput) -> f64 {
    let methods = input.method_count();
    let fields = input.field_count();
    if methods <= 1 || fields == 0 {
        return 0.0;
    }

    let average_accesses = input.field_access_count() as f64 / fields as f64;
    (methods as f64 - average_accesses) / (methods - 1) as f64
}

fn method_field_density_complement(input: &LcomInput) -> f64 {
    let methods = input.method_count();
    let fields = input.field_count();
    if methods <= 1 || fields == 0 {
        return 0.0;
    }

    1.0 - input.field_access_count() as f64 / (methods * fields) as f64
}

fn pair_difference(input: &LcomInput) -> f64 {
    let (sharing, non_sharing) = method_pair_counts(input);
    non_sharing.saturating_sub(sharing) as f64
}

fn method_pair_counts(input: &LcomInput) -> (usize, usize) {
    let mut sharing = 0;
    let mut non_sharing = 0;
    for left in 0..input.method_count() {
        for right in left + 1..input.method_count() {
            if fields_overlap(
                &input.method_field_accesses[left],
                &input.method_field_accesses[right],
            ) {
                sharing += 1;
            } else {
                non_sharing += 1;
            }
        }
    }
    (sharing, non_sharing)
}

fn connected_components(input: &LcomInput) -> usize {
    let method_count = input.method_count();
    let mut visited = vec![false; method_count];
    let mut components = 0;

    for start in 0..method_count {
        if visited[start] {
            continue;
        }
        components += 1;
        let mut pending = vec![start];
        while let Some(method) = pending.pop() {
            if visited[method] {
                continue;
            }
            visited[method] = true;
            for (candidate, candidate_visited) in visited.iter().enumerate() {
                if !candidate_visited
                    && fields_overlap(
                        &input.method_field_accesses[method],
                        &input.method_field_accesses[candidate],
                    )
                {
                    pending.push(candidate);
                }
            }
        }
    }
    components
}

fn fields_overlap(left: &BTreeSet<String>, right: &BTreeSet<String>) -> bool {
    left.iter().any(|field| right.contains(field))
}

#[cfg(test)]
mod tests {
    use super::{LcomInput, LcomMetric};
    use crate::metrics::extract_file_metrics;

    fn input(fields: &[&str], methods: &[&[&str]]) -> LcomInput {
        LcomInput::new(
            fields.iter().map(|field| (*field).to_owned()).collect(),
            methods
                .iter()
                .map(|accesses| accesses.iter().map(|field| (*field).to_owned()).collect())
                .collect(),
        )
    }

    fn assert_close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < 0.000_001,
            "expected {expected}, got {actual}"
        );
    }

    fn values(input: &LcomInput) -> Vec<f64> {
        LcomMetric::ALL
            .iter()
            .map(|metric| metric.calculate(input))
            .collect()
    }

    #[test]
    fn calculates_each_formula_for_a_partially_cohesive_input() {
        let subject = input(&["a", "b", "c"], &[&["a", "b"], &["a"], &["c"]]);
        let expected = [
            5.0 / 6.0,
            5.0 / 9.0,
            1.0,
            5.0 / 9.0,
            5.0 / 6.0,
            2.0,
            5.0 / 6.0,
            5.0 / 6.0,
        ];

        for (actual, expected) in values(&subject).into_iter().zip(expected) {
            assert_close(actual, expected);
        }
    }

    #[test]
    fn reports_perfect_cohesion_when_every_method_accesses_every_field() {
        let subject = input(&["a", "b"], &[&["a", "b"], &["a", "b"]]);

        assert_eq!(values(&subject), [0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
    }

    #[test]
    fn reports_disjoint_methods_and_transitive_components() {
        let disjoint = input(&["a", "b"], &[&["a"], &["b"]]);
        assert_eq!(values(&disjoint), [1.0, 0.5, 1.0, 0.5, 1.0, 2.0, 1.0, 1.0]);

        let transitive = input(&["a", "b"], &[&["a"], &["a", "b"], &["b"]]);
        assert_eq!(LcomMetric::Lcom4.calculate(&transitive), 1.0);
    }

    #[test]
    fn defines_zero_one_method_and_zero_field_edges_without_division_errors() {
        let empty = input(&[], &[]);
        let one_method = input(&["field"], &[&["field"]]);
        let no_fields = input(&[], &[&[], &[]]);

        assert_eq!(values(&empty), [0.0; 8]);
        assert_eq!(
            values(&one_method),
            [0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0]
        );
        assert_eq!(values(&no_fields), [0.0, 0.0, 1.0, 0.0, 0.0, 2.0, 0.0, 0.0]);
    }

    #[test]
    fn preserves_extreme_normalized_values_and_filters_invalid_accesses() {
        let unused = input(&["field"], &[&[], &[]]);
        assert_eq!(LcomMetric::Lcom96a.calculate(&unused), 2.0);
        assert_eq!(LcomMetric::Lcom96b.calculate(&unused), 1.0);

        let filtered = LcomInput::new(
            vec!["field".to_owned(), "field".to_owned()],
            vec![
                vec!["field".to_owned(), "field".to_owned(), "ghost".to_owned()],
                vec!["field".to_owned()],
            ],
        );
        assert_eq!(filtered.field_count(), 1);
        assert_eq!(filtered.field_access_count(), 2);
        assert_eq!(LcomMetric::Lcom96a.calculate(&filtered), 0.0);
    }

    #[test]
    fn exposes_stable_names_and_descriptions_for_the_complete_family() {
        assert_eq!(
            LcomMetric::ALL.map(LcomMetric::name),
            [
                "lcom96a",
                "lcom96b",
                "lcom1",
                "lcom2",
                "lcom3",
                "lcom4",
                "lcom5",
                "lcom_star"
            ]
        );
        for metric in LcomMetric::ALL {
            assert!(!metric.description().is_empty());
            assert_eq!(metric.to_string(), metric.name());
        }
    }

    #[test]
    fn rust_population_keeps_only_unambiguous_inherent_struct_methods() {
        let source = r#"
trait Port { fn send(&self); }
struct Eligible { field: usize }
impl Eligible { fn inherent(&self) { let _ = self.field; } }
impl Port for Eligible { fn send(&self) { let _ = self.field; } }

struct MacroOnly { field: usize }
macro_rules! generated { () => { fn generated(&self) { let _ = self.field; } }; }
impl MacroOnly { generated!(); }

mod first { pub struct Shared; }
mod second { pub struct Shared; }
impl Shared { fn unresolved(&self) {} }

enum Choice { One }
union Bits { integer: u32 }
"#;
        let file = extract_file_metrics("src/cohesion.rs", source).expect("fixture should parse");
        let eligible = file
            .types()
            .iter()
            .find(|type_info| type_info.name() == "Eligible")
            .expect("Eligible should be extracted");
        let input =
            LcomInput::from_type_info(eligible).expect("Eligible should have inherent data");

        assert_eq!(eligible.methods().len(), 2);
        assert_eq!(eligible.inherent_methods().len(), 1);
        assert_eq!(input.method_count(), 1);
        assert_eq!(input.field_access_count(), 1);
        for excluded in [
            "Port",
            "MacroOnly",
            "first::Shared",
            "second::Shared",
            "Choice",
            "Bits",
        ] {
            let type_info = file
                .types()
                .iter()
                .find(|type_info| type_info.name() == excluded)
                .expect("excluded type should still be extracted");
            assert!(
                LcomInput::from_type_info(type_info).is_none(),
                "{excluded} must not enter the LCOM population"
            );
        }
    }
}
