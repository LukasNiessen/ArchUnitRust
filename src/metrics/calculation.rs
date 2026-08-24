//! Pure metric calculations over extracted subjects.

mod distance;
mod lcom;

use std::fmt;

use super::extraction::{DistanceInfo, FileMetricsInfo, ProjectMetricsInfo, TypeInfo};

pub use distance::{
    ArchitecturalZone, DistanceInput, DistanceMetric, MAXIMUM_SIZE_DISCOUNT, PAIN_LIMIT,
    SIZE_NORMALIZATION_LINES, USELESSNESS_LIMIT,
};
pub use lcom::{LcomInput, LcomMetric};

/// A built-in count metric and its valid Rust subject population.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum CountMetric {
    /// Methods with a `self` receiver per type.
    MethodCount,
    /// Declared data fields per type.
    FieldCount,
    /// Physical non-comment source lines per file.
    LinesOfCode,
    /// Syntax-tree items and executable statements per file.
    Statements,
    /// `use` and `extern crate` items per file.
    Imports,
    /// Structs, enums, and unions per file.
    ConcreteTypes,
    /// Free functions per file.
    Functions,
    /// Trait declarations per file.
    Traits,
    /// Inherent and trait impl blocks per file.
    ImplBlocks,
    /// Macro invocations per file.
    Macros,
    /// Receiver-free functions in traits and impl blocks per file.
    AssociatedFunctions,
}

impl CountMetric {
    /// Every built-in count metric in stable report order.
    pub const ALL: [Self; 11] = [
        Self::MethodCount,
        Self::FieldCount,
        Self::LinesOfCode,
        Self::Statements,
        Self::Imports,
        Self::ConcreteTypes,
        Self::Functions,
        Self::Traits,
        Self::ImplBlocks,
        Self::Macros,
        Self::AssociatedFunctions,
    ];

    /// Returns the stable public metric name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::MethodCount => "method_count",
            Self::FieldCount => "field_count",
            Self::LinesOfCode => "lines_of_code",
            Self::Statements => "statements",
            Self::Imports => "imports",
            Self::ConcreteTypes => "concrete_types",
            Self::Functions => "functions",
            Self::Traits => "traits",
            Self::ImplBlocks => "impl_blocks",
            Self::Macros => "macros",
            Self::AssociatedFunctions => "associated_functions",
        }
    }

    /// Returns a concise description of the metric's Rust semantics.
    #[must_use]
    pub const fn description(self) -> &'static str {
        match self {
            Self::MethodCount => "associated functions with a self receiver",
            Self::FieldCount => "declared data fields",
            Self::LinesOfCode => "physical lines containing non-comment source text",
            Self::Statements => "syntax-tree items and executable block statements",
            Self::Imports => "use and extern crate items",
            Self::ConcreteTypes => "struct, enum, and union declarations",
            Self::Functions => "free function declarations",
            Self::Traits => "trait declarations",
            Self::ImplBlocks => "inherent and trait implementation blocks",
            Self::Macros => "macro invocations and definitions represented by syntax",
            Self::AssociatedFunctions => "receiver-free functions in traits and impl blocks",
        }
    }

    pub(crate) fn measurements(self, project: &ProjectMetricsInfo) -> Vec<MetricMeasurement> {
        if matches!(self, Self::MethodCount | Self::FieldCount) {
            return project
                .types()
                .iter()
                .cloned()
                .map(|type_info| {
                    let value = match self {
                        Self::MethodCount => type_info.methods().len(),
                        Self::FieldCount => type_info.fields().len(),
                        _ => 0,
                    };
                    MetricMeasurement::from_parts(
                        MetricSubject::Type(type_info),
                        self.name(),
                        self.description(),
                        value as f64,
                    )
                })
                .collect();
        }

        project
            .files()
            .iter()
            .cloned()
            .map(|file| {
                let value = match self {
                    Self::LinesOfCode => file.lines_of_code(),
                    Self::Statements => file.statements(),
                    Self::Imports => file.imports(),
                    Self::ConcreteTypes => file.concrete_types(),
                    Self::Functions => file.functions(),
                    Self::Traits => file.traits(),
                    Self::ImplBlocks => file.impl_blocks(),
                    Self::Macros => file.macros(),
                    Self::AssociatedFunctions => file.associated_functions(),
                    Self::MethodCount | Self::FieldCount => 0,
                };
                MetricMeasurement::from_parts(
                    MetricSubject::File(file),
                    self.name(),
                    self.description(),
                    value as f64,
                )
            })
            .collect()
    }
}

impl fmt::Display for CountMetric {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name())
    }
}

/// The extracted subject measured by a metric.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum MetricSubject {
    /// One source file.
    File(FileMetricsInfo),
    /// One Rust type declaration.
    Type(TypeInfo),
    /// One file-level component with its project coupling evidence.
    Distance(DistanceInfo),
}

impl MetricSubject {
    /// Returns the stable file path or module-qualified type name.
    #[must_use]
    pub fn identifier(&self) -> &str {
        match self {
            Self::File(file) => file.path(),
            Self::Type(type_info) => type_info.name(),
            Self::Distance(info) => info.identifier(),
        }
    }

    /// Returns the file subject, if this measurement is file-level.
    #[must_use]
    pub const fn as_file(&self) -> Option<&FileMetricsInfo> {
        match self {
            Self::File(file) => Some(file),
            Self::Type(_) | Self::Distance(_) => None,
        }
    }

    /// Returns the type subject, if this measurement is type-level.
    #[must_use]
    pub const fn as_type(&self) -> Option<&TypeInfo> {
        match self {
            Self::Type(type_info) => Some(type_info),
            Self::File(_) | Self::Distance(_) => None,
        }
    }

    /// Returns the component distance subject, if this measurement uses project coupling.
    #[must_use]
    pub const fn as_distance(&self) -> Option<&DistanceInfo> {
        match self {
            Self::Distance(info) => Some(info),
            Self::File(_) | Self::Type(_) => None,
        }
    }
}

/// One numeric metric value and the complete subject that produced it.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct MetricMeasurement {
    subject: MetricSubject,
    metric_name: String,
    description: String,
    value: f64,
}

impl MetricMeasurement {
    pub(crate) fn from_parts(
        subject: MetricSubject,
        metric_name: impl Into<String>,
        description: impl Into<String>,
        value: f64,
    ) -> Self {
        Self {
            subject,
            metric_name: metric_name.into(),
            description: description.into(),
            value,
        }
    }

    /// Returns the measured subject.
    #[must_use]
    pub const fn subject(&self) -> &MetricSubject {
        &self.subject
    }

    /// Returns the file path or type name identifying the subject.
    #[must_use]
    pub fn identifier(&self) -> &str {
        self.subject.identifier()
    }

    /// Returns the stable metric name.
    #[must_use]
    pub fn metric_name(&self) -> &str {
        &self.metric_name
    }

    /// Returns the metric description.
    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Returns the numeric value. Exact count metrics are represented as whole-valued `f64`s.
    #[must_use]
    pub const fn value(&self) -> f64 {
        self.value
    }
}

#[cfg(test)]
mod tests {
    use super::CountMetric;

    #[test]
    fn names_and_descriptions_cover_every_count_metric() {
        let metrics = [
            CountMetric::MethodCount,
            CountMetric::FieldCount,
            CountMetric::LinesOfCode,
            CountMetric::Statements,
            CountMetric::Imports,
            CountMetric::ConcreteTypes,
            CountMetric::Functions,
            CountMetric::Traits,
            CountMetric::ImplBlocks,
            CountMetric::Macros,
            CountMetric::AssociatedFunctions,
        ];

        for metric in metrics {
            assert!(!metric.name().is_empty());
            assert!(!metric.description().is_empty());
            assert_eq!(metric.to_string(), metric.name());
        }
    }
}
