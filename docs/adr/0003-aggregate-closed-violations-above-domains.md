# ADR 0003: Aggregate closed violations above domains

- **Status:** accepted
- **Date:** 2026-08-23
- **Decision owners:** ArchUnitRust maintainers
- **Tracking:** [#65](https://github.com/LukasNiessen/ArchUnitRust/issues/65)

## Context

Rust models the library-wide `Violation` abstraction as a closed enum. This is more idiomatic and
more useful than trait objects here: callers can exhaustively inspect typed violation data without
allocation, downcasting or string keys.

The first cycle rule initially put both `Violation` and `CycleViolation` in `common/assertion`.
That stopped working once the Files API needed more file-specific violations. Domain violation data
belongs in `<domain>/assertion`; moving every future variant into `common` would make the kernel a
catalogue of domain concepts, while importing Files types into `common` would reverse the intended
dependency direction.

The same problem applies to `Checkable`: its result contains the library-wide `Violation` sum, so
it cannot honestly remain a common-only contract once the enum aggregates domain-owned data.

## Decision

Keep each concrete violation type with the assertion that produces it. In particular,
`CycleViolation` is owned by `files/assertion`, while domain-neutral `EmptyTestViolation` remains in
`common/assertion`.

Define two private top-level aggregation modules:

- `violation` owns the closed `Violation` and `ViolationKind` enums and imports concrete data from
  `common` and each domain;
- `checkable` owns the object-safe `Checkable` trait and `CheckResult`, joining `CheckOptions`,
  `ArchUnitError` and `Violation`.

`lib.rs` remains a re-export-only public surface. Existing public paths such as
`archunit::Violation`, `archunit::CycleViolation` and `archunit::Checkable` do not change.

Adding a violation variant therefore requires a deliberate edit to the top-level closed sum, but
does not make `common` depend on a domain or move domain data out of its natural module.

## Alternatives considered

### Keep all violation data in `common`

Rejected. It makes the shared kernel know every file, layer, slice, metric and graph-report concept.
That ownership becomes increasingly misleading as rule families grow.

### Let `common::Violation` import domain types

Rejected. It introduces the dependency inversion this repository's module boundaries are intended
to prevent and risks cycles between shared graph mechanics and rule families.

### Replace the enum with trait objects

Rejected. An open hierarchy matches some sibling languages, but gives up exhaustive matching,
straightforward typed accessors and value semantics in the Rust API. The set of built-in violation
families is known when the crate is compiled.

### Give every domain an unrelated result type

Rejected. Test helpers and report consumers need one object-safe terminal contract. Requiring them
to be generic over every rule family would leak implementation structure into the user-facing edge.

## Consequences

### Positive

- concrete violation data stays next to its pure assertion logic;
- `common` remains independent of rule domains;
- the public API and object-safe terminal seam remain uniform;
- enum variants, kinds and typed accessors stay synchronized in one small module;
- future reporting code has one explicit catalogue of supported violation families.

### Negative

- the aggregation modules know all domains and must change when a variant is added;
- domain terminals that return `CheckResult` necessarily meet the aggregation seam;
- source layout has two intentional top-level modules in addition to domain and common modules.

### Mitigations

- keep aggregation modules data- and contract-only, with no extraction or rule behavior;
- keep every concrete violation constructor and assertion function in its owning domain;
- preserve root re-exports so module placement does not leak into downstream code;
- test the public typed accessors whenever a new enum variant is introduced.
