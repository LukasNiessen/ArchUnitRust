# ADR 0002: Defer fluent selector errors to terminal checks

- **Status:** accepted
- **Date:** 2026-08-23
- **Decision owners:** ArchUnitRust maintainers
- **Tracking:** [#16](https://github.com/LukasNiessen/ArchUnitRust/issues/16),
  [#23](https://github.com/LukasNiessen/ArchUnitRust/issues/23)

## Context

ArchUnit's public API is an English-like sentence. File selectors are intended to compose without
control-flow punctuation:

```rust
project_files()
    .in_folder("src/api/**")
    .with_name("*_handler.rs")
    .should_not()
    .depend_on_files()
    .in_folder("src/db/**");
```

The shared glob and regex substrate can reject malformed input. Sibling implementations raise an
exception at the selector call, but Rust has no exceptions. Returning `Result<Builder, Error>` from
every selector would require `?` or `expect` between most words of the fluent sentence and would
change the builder type at every step.

Panicking is not acceptable for invalid user input. Silently replacing an invalid selector with a
matcher that selects nothing is also unsafe: it would disguise a typo as an empty architecture
check and could later be mistaken for an ordinary `EmptyTestViolation`.

## Decision

Selector methods continue to consume and return the same builder type. They compile their pattern
immediately and append a successful `Filter`. On failure, the builder retains the first
`PatternError`; subsequent selector calls preserve that error and do not append more filters.

Every terminal must inspect the retained selector error before extracting or judging a graph. It
returns that error as `ArchUnitError::User`, with rule-specific context. A malformed rule is an API
error, not a rule violation and not an empty-test result.

Builders expose the retained error for diagnostics and tests. This state is cloned when a reusable
scope is branched, just like its locator and compiled filters.

## Alternatives considered

### Return `Result` from every selector

Rejected for the fluent surface. It is locally idiomatic but makes the primary product sentence
noisy and creates nested error handling before a rule can implement the shared `Checkable` terminal
contract. Lower-level pattern factories continue to return `Result` normally.

### Panic on an invalid pattern

Rejected. Invalid user input is recoverable and belongs to `UserError`; library code must not panic
for it.

### Compile patterns only inside `check`

Rejected. Builders would need to retain untyped selector descriptions, duplicate factory dispatch
at the terminal, and recompile patterns on repeated checks. Immediate compilation keeps each
selector's meaning local while deferring only error reporting.

### Treat an invalid selector as matching nothing

Rejected. That conflates malformed API input with a valid selector that happens to match no files,
undermining the empty-test guard.

## Consequences

### Positive

- fluent rule chains retain the shared sentence structure;
- builders stay one immutable, cloneable type throughout scope selection;
- malformed patterns cannot panic or silently pass;
- terminals have one consistent place to translate builder failures into `UserError`;
- successful patterns compile exactly once and can be reused across repeated checks.

### Negative

- a selector error is reported at `check`, not on the line that constructed the selector;
- every new terminal must perform the retained-error guard before extraction;
- tooling cannot use the return type alone to prove that a scope contains only valid selectors.

### Mitigations

- expose `selector_error()` for early inspection and focused tests;
- centralize terminal setup in shared file-rule support as soon as the first terminal lands;
- test every terminal with malformed selectors as well as zero-match selectors;
- keep the original pattern and compiler reason in `PatternError` so delayed diagnostics remain
  precise.
