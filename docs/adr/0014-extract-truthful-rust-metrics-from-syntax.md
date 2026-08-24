# ADR 0014: Extract truthful Rust metrics from syntax

- Status: Accepted
- Date: 2026-08-24
- Issue: #32

## Context

The sibling ArchUnit libraries expose metrics over classes, interfaces, methods, fields, and files.
Rust has structs, enums, unions, traits, inherent implementations, trait implementations, free
functions, and associated functions. Calling those concepts classes or interfaces would make the API
look familiar while giving its values ambiguous or false meanings.

Metrics also need a stable information model for issue #33 cohesion calculations and issue #35
custom metrics. That model must retain field-access evidence without requiring a second parse, but it
cannot promise compiler expansion or type resolution when the extraction boundary is the source
syntax tree.

## Decision

The metrics extraction boundary parses each discovered Rust source file with `syn` and produces
immutable public values:

- `MethodInfo` represents an associated function whose signature has a `self` receiver and records
  syntactic `self.field` accesses;
- `FieldInfo` represents a struct or union field, or an enum-variant field, and records the methods
  that access it;
- `TypeInfo` represents a struct, enum, union, or trait and retains its methods, fields, and
  receiver-free associated functions;
- `ImplInfo` represents one inherent or trait implementation and its target path;
- `FileMetricsInfo` contains the source identifier, extracted type and impl information, and file
  counts;
- `ProjectMetricsInfo` contains the selected project snapshot.

Methods from inherent and trait impl blocks are associated with a declared type when the syntax gives
one unambiguous target. Matching first prefers a unique declaration in the same file and then a
unique project-wide unqualified name. Ambiguous targets remain visible through `ImplInfo` but are not
silently attached to a `TypeInfo`. This is deliberately syntax-aware rather than compiler-exact.

The initial count metrics are:

| Subject | Metric | Meaning |
|---|---|---|
| Type | method count | Associated functions with a `self` receiver |
| Type | field count | Declared struct, union, or enum-variant fields |
| File | lines of code | Physical lines containing non-comment source text |
| File | statements | Syntax-tree items plus block statements that are not item statements |
| File | imports | `use` and `extern crate` items |
| File | concrete types | Structs, enums, and unions |
| File | functions | Free functions only |
| File | traits | Trait declarations |
| File | impl blocks | Inherent and trait impl blocks |
| File | macros | Macro invocations, including macro definitions represented by `syn` |
| File | associated functions | Impl or trait functions without a `self` receiver |

Comment-only and blank lines do not count as lines of code. A small lexical scanner handles line and
nested block comments while preserving strings and character literals well enough to decide whether
a physical line contains source text. The number remains a physical-line measure, not a token count.

The fluent entry points are `metrics()` and `metrics_in(locator)`. File selectors are `with_name`,
`in_folder`, and `in_path`; type selection is `for_types_matching`. Selectors combine with AND and
compile eagerly into retained configuration errors, which terminals return before project discovery.
No `for_classes_matching` or `interfaces` alias is provided.

`analyze` returns the selected information model. `count` selects a named count metric and `measure`
returns deterministic `MetricMeasurement` values. Counts are stored as `f64` at the measurement
boundary so threshold and custom metrics can share one representation in later issues, while the
extracted counts remain exact `usize` values.

## Alternatives considered

### Preserve class and interface names for sibling parity

This would obscure whether an enum, trait, or impl block participates in a metric and would require
invented class-level behavior. Rust-native names make divergences reviewable and prevent consumers
from inferring semantics the implementation cannot provide.

### Use rust-analyzer or compiler internals

Those systems could resolve impl targets and macro expansion more precisely, but would substantially
increase toolchain coupling, runtime, and MSRV risk. The project explicitly defines source-syntax
analysis as its initial boundary. The public evidence model can be enriched later without changing
the meaning of current counts.

### Count every physical line

Raw file length is deterministic but makes blank formatting and comment blocks architectural
complexity. Counting physical lines with source text preserves a familiar lines-of-code metric while
remaining independent of statement formatting.

### Fold associated functions into free functions or methods

Receiver-free functions inside impls are neither free functions nor methods in Rust terminology.
Keeping a separate count avoids inflating either metric and supplies a useful Rust-specific signal.

## Consequences

- Metrics use Rust vocabulary and document every population explicitly.
- Extraction is deterministic, safe, and independent of compiler-private APIs.
- Macro-generated declarations and field accesses are excluded until an expansion-aware extractor is
  introduced.
- Ambiguous impl associations remain observable rather than guessed.
- Issues #33 through #37 can reuse one snapshot, subject model, and numeric measurement boundary.
