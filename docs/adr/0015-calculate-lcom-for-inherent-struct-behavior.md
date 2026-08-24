# ADR 0015: Calculate LCOM only for inherent struct behavior

- Status: Accepted
- Date: 2026-08-24
- Issue: #33

## Context

The LCOM family was defined for object state and the methods implemented by one class. Rust spreads
behavior across inherent impl blocks, trait impl blocks, default trait methods, macro invocations,
and sometimes other files. Treating all associated functions as one class method set would let an
unrelated trait implementation change a struct's cohesion and would assign invented values to enums,
unions, and traits.

The LCOM names themselves have accumulated incompatible numbering and formula descriptions. The
TypeScript role model and the mature Ruby port agree on the issue's eight public names but disagree
on several implementations and edge cases. In particular, an unguarded one-method denominator can
produce infinities, the role-model LCOM5 expression can be negative, and its LCOM-star comment names
a pair-ratio formula instead of the normalized Henderson-Sellers form used by the Ruby port.

## Decision

`TypeInfo` retains `inherent_methods` separately from its complete `methods` collection. Count
metrics continue to count inherent and trait-impl methods, while LCOM uses only the inherent set.
An LCOM subject must be a struct with at least one unambiguously associated inherent method.

Therefore LCOM excludes:

- traits, enums, and unions;
- methods supplied by trait declarations or trait impls;
- macro-generated methods, because the source extractor does not expand macros;
- inherent impls whose target cannot be associated with exactly one declaration.

`LcomInput` is a small immutable formula boundary containing declared field names and one declared
field-access set per method. It can be constructed directly, allowing every formula to be tested
without parsing source, or derived from an eligible `TypeInfo`. Accesses to names outside the
declared field set are discarded.

Let `m` be the method count, `a` the declared field count, `S` the total number of distinct
method-to-field accesses, `P` the number of method pairs sharing no field, and `Q` the number sharing
at least one field. The issue's metrics are:

| Metric | Formula |
|---|---|
| LCOM96a | `(m - S/a) / (m - 1)` |
| LCOM96b | `1 - S/(m*a)` |
| LCOM1 | `max(P - Q, 0)` |
| LCOM2 | `1 - S/(m*a)` |
| LCOM3 | `(m - S/a) / (m - 1)` |
| LCOM4 | connected components in the method graph induced by shared fields |
| LCOM5 | `(m - S/a) / (m - 1)` |
| LCOM* | `(m - S/a) / (m - 1)` |

The repeated formulas are intentional aliases in the cross-language API. They retain their own
stable names and descriptions so reports and thresholds remain explicit. Values are not clamped:
when fields exist but none is used, the normalized distance can be greater than one, which preserves
the formula rather than hiding an extreme input.

Edge behavior is defined before division:

| Population | Normalized/density metrics | LCOM1 | LCOM4 |
|---|---:|---:|---:|
| zero methods | `0.0` | `0.0` | `0.0` |
| one method | `0.0` | `0.0` | `1.0` |
| zero fields, multiple methods | `0.0` | number of method pairs | method count |

LCOM4 initially uses only shared-field edges. Although some definitions additionally include direct
method calls, issue #33's accepted extraction evidence is the bidirectional method/field relation.
Adding call edges later would require a new decision and an explicit metric spelling or compatibility
review.

## Alternatives considered

### Calculate over every `TypeInfo::methods` entry

That would let adding a formatting, conversion, or framework trait impl change the inherent design
signal. Separating method origins keeps issue #32's broad method count while making cohesion about
behavior owned by the struct.

### Copy every TypeScript formula exactly

The role model lacks several denominator guards and its LCOM5 and LCOM-star implementations disagree
with the normalized metric family used by the mature Ruby port. Preserving those defects would make
Rust results non-finite or unexpectedly negative. The formula table is the compatibility contract.

### Return zero for ineligible Rust types

Zero means measured perfect cohesion. Traits, enums, unions, structs without inherent behavior, and
unresolved impl targets were not measured, so they are omitted from the measurement population
instead of receiving a misleading success value.

### Include trait impls selected by the user

Trait behavior can be cohesive in its own right, but one struct can implement multiple unrelated
traits and the same trait can be implemented across types. A future trait-implementation cohesion
metric should use `ImplInfo` as its subject rather than merging those methods into the struct metric.

## Consequences

- Pure formulas are independently testable and deterministic.
- Cross-file inherent impls participate after the same conservative association used by count
  metrics.
- Adding or removing a trait impl cannot change a struct's LCOM values.
- Ineligible types produce no measurement, preserving the distinction between unmeasured and zero.
- Threshold checks in issue #36 can reuse the existing numeric measurement boundary without changing
  formula semantics.
