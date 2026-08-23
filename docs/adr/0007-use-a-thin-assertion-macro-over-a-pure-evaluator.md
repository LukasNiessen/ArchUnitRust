# ADR 0007: Use a thin assertion macro over a pure evaluator

- **Status:** accepted
- **Date:** 2026-08-23
- **Decision owners:** ArchUnitRust maintainers
- **Tracking:** [#25](https://github.com/LukasNiessen/ArchUnitRust/issues/25)

## Context

The universal testing path must run any `Checkable`, apply optional `CheckOptions`, format the
outcome through the shared factories and raise the host language's assertion failure. In Rust,
frameworks and the built-in `#[test]` harness recognize panics as assertion failures, but ordinary
library logic must not panic. Rust also has no optional function arguments, and callers should be
able to pass an inline terminal without giving up reuse for named rules.

An assertion function could panic internally with `#[track_caller]`, but that would put a panic in
ordinary compiled library code and make the optional-options call less natural. A macro can emit
Rust's own `assert!` at the caller while delegating every fallible or presentational decision to a
normal function.

## Decision

Expose `assert_passes!(rule)` and `assert_passes!(rule, check_options)` as the universal assertion
surface.

The macro borrows each expression, evaluates it once, calls a hidden public evaluator required for
downstream macro expansion, and passes the resulting `TestResult` to Rust's built-in `assert!`.
Because the assertion expands at the invocation, the panic location and test-harness behavior are
the caller's. A blanket `Checkable` implementation for `&T` allows concrete rules, borrowed rules
and `&dyn Checkable` values to use the same macro forms.

The evaluator itself never panics. It calls `check_with` exactly once:

- successful checks flow to `ResultFactory::from_violations_with_options`;
- `UserError` and `TechnicalError` flow to `ResultFactory::from_error_with_options`;
- the default display options use automatic color and expect the rule to pass.

A check error always fails the assertion, including when an architecture expectation is inverted,
because no architecture verdict was reached. The original classified error text remains in the
formatted assertion message.

## Alternatives considered

### Provide only a function returning `Result<(), Error>`

Rejected. It would require every test to add `?`, `unwrap` or a separate assertion and would not
fulfil the promise that the helper raises the language's assertion failure with shared formatting.

### Provide a panicking `assert_passes` function

Rejected. `#[track_caller]` could improve its location, but the panic would still live in ordinary
library code and optional options would require a second function name or an `Option` argument.

### Put rule checking and message construction inside the macro

Rejected. Macro logic is hard to unit-test and would duplicate the testing pipeline. The macro is
limited to borrowing, calling the evaluator and invoking `assert!`.

### Return the violation list from the macro

Rejected. An assertion helper should return unit on success. Structured results remain available
through `Checkable`, `ViolationFactory` and `ResultFactory` for custom integrations.

### Require owned rules

Rejected. Architecture rules are immutable and cloneable, but assertions do not need ownership.
Borrowing preserves reusable named terminals and permits trait-object collections.

## Consequences

### Positive

- one zero-configuration assertion works in the built-in harness and panic-aware Rust frameworks;
- inline, named, borrowed and type-erased rules share the same syntax;
- optional `CheckOptions` read naturally without an `Option` wrapper;
- assertion failures reuse identical violation numbering, prose, error context and color policy;
- evaluator behavior is unit-testable without catching panics;
- the assertion failure originates at the macro call site.

### Negative

- exported macros require one `#[doc(hidden)]` root evaluator symbol for downstream expansion;
- macro callers see a panic-based failure because that is Rust's test assertion mechanism;
- the two macro arms must remain synchronized;
- automatic color is the only presentation policy exposed by the convenience macro.

### Mitigations

- keep the hidden evaluator narrow and route all results through public factories;
- cover macro expansion from integration tests outside the library module;
- borrow and evaluate each macro expression once in both arms;
- direct callers who need deterministic color or inverted expectations to `ResultFactory`.
