# ADR 0008: Use the built-in harness without an adapter

- Status: Accepted
- Date: 2026-08-23
- Issue: #26

## Context

Sibling ArchUnit ports can require a framework adapter, matcher registration or import-time setup
to make architecture rules feel native in a test suite. Rust's stable built-in test harness exposes
none of those extension points. A native Rust test is an ordinary function annotated with `#[test]`,
and assertions report failures by panicking from that function.

ADR 0007 already places the panic boundary in `assert_passes!` and keeps rule checking and result
formatting in ordinary, non-panicking library code. Adding an adapter facade would therefore add
terminology and code without integrating with anything the built-in harness actually provides.

## Decision

`assert_passes!` is the native built-in-harness integration. Users import it and call it inside an
ordinary `#[test]` function. There is no adapter crate, feature flag, runtime framework detection or
registration side effect.

The macro continues to delegate to the shared evaluator and `ResultFactory`; no rule logic or
diagnostic formatting belongs at the harness boundary. A standalone Cargo consumer fixture verifies
both directions through a real test process:

- a passing architecture rule exits successfully under the built-in harness;
- an intentionally failing, normally ignored test exits unsuccessfully and exposes the shared
  numbered violation message in the harness output.

Rust's built-in assertions have no matcher-level negation protocol. Architecture negation remains
part of the fluent rule language through `should_not()`. `TestResultOptions` retains inverted
expectation semantics for lower-level custom integrations; the native assertion macro does not add a
second negation vocabulary.

## Alternatives considered

### A custom test attribute or custom test framework

A procedural attribute would only generate a normal `#[test]` wrapper, while Rust's custom test
framework support is unstable. Either approach would increase build and maintenance cost without
improving the stable harness integration.

### A trait with panicking assertion methods

This would move the assertion boundary into library code, require an extension-trait import and work
against the repository rule that reusable library paths return values rather than panic.

### A separate adapter crate or Cargo feature

There is no built-in-harness API for such an adapter to implement. A separate package or feature
would imply optional behavior and complicate setup while ultimately calling the same macro or
`assert!` expansion.

### A macro that generates the entire test function

Generating `#[test]` functions would reduce flexibility around fixture setup and test attributes,
and duplicate a language feature users already know. Keeping the macro at the assertion site composes
with ordinary Rust tests.

## Consequences

- Native usage is one import and one macro call inside `#[test]`.
- The assertion output cannot drift from other integrations because all paths use the shared result
  factory.
- The integration is verified from the perspective of a separate Cargo package, including failure
  exit status and diagnostic text.
- The fixture uses an isolated target directory and therefore adds a one-time cold-build cost to the
  integration suite.
- Framework-specific adapters can still be added later if an external Rust framework exposes a real
  extension contract, without changing the built-in path.
