# ADR 0018: Use exact measurement-backed thresholds

- Status: Accepted
- Date: 2026-08-24
- Issue: #36

## Context

Every numeric metric needs a consistent assertion vocabulary. The cross-language contract names
exactly six terminals: five numeric thresholds and one arbitrary predicate. ArchUnitTS accumulated
an extra equality synonym; issue #36 explicitly identifies that as a compatibility wart rather than
a precedent.

Rust's metric selections do not all have the same subject type. Count metrics can measure either a
`TypeInfo` or `FileMetricsInfo`, LCOM measures eligible types, distance metrics measure
`DistanceInfo`, and custom metrics have the statically precise `TypeInfo` callback introduced by
issue #35. The assertion layer needs one closed evidence model without erasing the subject facts
already retained by `MetricMeasurement`.

Floating-point equality and non-finite values also require policy. Adding an implicit epsilon to
`should_be` would make the public verb mean something other than equality and would require a hidden,
arbitrary tolerance. A non-finite threshold cannot establish a useful architecture boundary, while
issue #35 deliberately permits a custom calculation to produce non-finite values for user-defined
policy.

## Decision

Count, LCOM, distance, and custom selections expose exactly these threshold items:

- `should_be_below`;
- `should_be_above`;
- `should_be`;
- `should_be_below_or_equal`;
- `should_be_above_or_equal`;
- `should_satisfy`.

No aliases such as `should_equal`, `should_be_at_most`, or `should_be_less_than` are added.

`MetricComparison` is the closed data model for the five thresholds. Comparisons use Rust `f64`
operators directly: `<`, `>`, `==`, `<=`, and `>=`. `should_be` therefore means exact equality with
no epsilon. Callers that need tolerance can use `should_satisfy` and state that tolerance explicitly.

Thresholds must be finite. `NaN`, positive infinity, and negative infinity are deferred user errors
reported after any earlier fluent configuration error but before Cargo project discovery or metric
calculation. Negative thresholds and signed zero remain valid.

Measured values retain standard IEEE-754 behavior. In particular, a custom positive infinity can
satisfy `should_be_above` with a finite threshold, while `NaN` satisfies none of the five comparisons
and becomes a typed violation. This preserves issue #35's custom-value escape hatch while keeping
the configured boundary meaningful.

Threshold rules operate on the same `MetricMeasurement` population returned by `measure_with`.
Failures retain `MetricThresholdViolation` with the complete `MetricSubject`, metric name, value,
threshold, and comparison. Built-in `should_satisfy` predicates receive:

```text
Fn(f64, &MetricSubject) -> bool
```

This accommodates count metrics whose subject level is chosen by the metric enum while preserving
typed access through `as_file`, `as_type`, and `as_distance`. Failures retain
`MetricPredicateViolation`. Custom metrics keep their more precise
`Fn(f64, &TypeInfo) -> bool` terminal and `CustomMetricViolation` from issue #35.

Predicate callbacks run once per measurement and their panics propagate normally. Empty assertion
populations use the shared strict guard before invoking a predicate. LCOM's assertion population is
its measurable, eligible struct population; an ineligible-only selection is therefore guarded rather
than silently passing.

## Alternatives considered

### Add an epsilon to `should_be`

There is no domain-independent tolerance for counts, normalized ratios, coupling, and arbitrary
custom metrics. An implicit epsilon would surprise exact count assertions and still be wrong for some
floating-point policies. `should_satisfy` is the explicit tolerance escape hatch.

### Give every selection a subject-specific predicate terminal

Count selections choose file or type subjects at runtime from `CountMetric`, so a single static
callback subject cannot describe that API. `MetricSubject` keeps one stable built-in signature and
still offers typed accessors. Custom metrics remain statically type-level and keep `TypeInfo`.

### Recalculate values inside each assertion family

That would duplicate population rules and could make measurement and assertion disagree. Reusing
`MetricMeasurement` makes one calculation path authoritative and lets violations retain its evidence.

### Allow non-finite thresholds

`NaN` never satisfies an ordered comparison, and infinite boundaries make most rules vacuous.
Rejecting them as configuration errors catches mistakes without restricting custom calculated values.

## Consequences

- The fluent API has one intentional, cross-language assertion vocabulary without synonyms.
- Count, LCOM, distance, and custom thresholds share comparison and violation semantics.
- Exact equality is predictable and tolerance remains explicit in user code.
- Configuration failures precede filesystem work and callback execution.
- Metric assertion failures remain machine-readable through the closed violation enum.
