# ADR 0017: Own generic custom metric callbacks

- Status: Accepted
- Date: 2026-08-24
- Issue: #35

## Context

The metrics module needs an escape hatch for project-specific numeric facts. The cross-language API
defines a named calculation over complete class information and a predicate over the resulting value
and that same subject. Rust has no `ClassInfo`; `TypeInfo` is the truthful immutable model for
structs, enums, unions, and traits.

Callback storage creates a Rust-specific trade-off. Boxing callbacks behind `Arc<dyn Fn + Send +
Sync + 'static>` would make one concrete terminal type easy to name, but would impose thread-safety
and lifetime constraints unrelated to synchronous architecture checks. Borrowing callbacks would
make fluent terminals self-referential or short-lived. Catching callback panics could keep `check()`
inside its result type, but it would misclassify user code defects as architecture disagreements or
technical extraction failures.

The numeric boundary also needs explicit behavior for `NaN` and infinity. A custom calculation is
intentionally outside the built-in metric vocabulary, so the library cannot infer whether a
non-finite result is accidental or meaningful to its predicate.

## Decision

`MetricsBuilder::custom_metric` owns a generic calculation closure with this contract:

```text
Fn(&TypeInfo) -> f64
```

`CustomMetricSelection::should_satisfy` consumes the selection and owns a second generic closure:

```text
Fn(f64, &TypeInfo) -> bool
```

Both receive shared references to the same immutable type evidence. The generic structs avoid
dynamic dispatch and accept captured values without requiring `Send`, `Sync`, or `'static`.
Selections and conditions derive `Clone`; they are cloneable whenever their captured callbacks are
cloneable. Execution methods borrow the terminal, so cloning is not required merely to measure or
check it repeatedly.

The selected `TypeInfo` population is deterministic and follows the existing file and type filters.
A calculation is invoked exactly once per type per execution. For a predicate rule, its predicate is
then invoked exactly once with that value and type. Measurements retain `MetricSubject::Type`; failed
predicates retain a closed `CustomMetricViolation` containing the complete type, name, description,
and value.

Metric names and descriptions must contain non-whitespace text. Validation is deferred in the
builder's existing first-error slot, so an earlier invalid selector still wins and all configuration
errors are reported before Cargo project discovery.

An empty measurement is valid data. An empty predicate rule receives the universal strict
empty-selection guard under the `metric types` subject, with the normal explicit
`CheckOptions::with_allow_empty_tests(true)` opt-out.

Panic and numeric behavior is deliberate:

- panics from calculation or predicate callbacks propagate unchanged as ordinary Rust panics;
- the library does not use `catch_unwind` around user callbacks;
- finite and non-finite `f64` values are passed to the predicate and retained in violations exactly
  as calculated.

## Alternatives considered

### Store `Arc<dyn Fn>` callbacks

This would erase the callback type but require a `'static` lifetime and, for broadly reusable
terminals, likely `Send + Sync`. Custom metrics execute synchronously and need neither restriction.
Generic ownership is zero-cost and lets the compiler describe precisely which terminal values are
cloneable.

### Return `Result<f64, ArchUnitError>` from calculations

This would force every custom metric to manufacture library error semantics and would blur the line
between project extraction failures and user calculation policy. A project can encode fallibility in
its chosen value or handle it before constructing the metric.

### Catch panics and create violations

A panic does not mean the architecture failed the predicate; it means the callback did not produce a
verdict. Propagating preserves Rust backtraces and the test harness's normal failure behavior.

### Reject non-finite values

Built-in metrics define their denominator behavior and remain finite, but the custom-metric escape
hatch must not invent policy. Predicates can explicitly require `value.is_finite()` when desired.

## Consequences

- Custom metrics can inspect all extracted Rust type, method, field, and impl association evidence.
- Terminals remain lazy, reusable, and statically dispatched.
- Some callback captures make the resulting terminal non-`Clone`; this is visible at compile time.
- Callback side effects and determinism remain the caller's responsibility.
- Issue #36 can add shared threshold terminals over numeric selections without changing callback
  semantics.
