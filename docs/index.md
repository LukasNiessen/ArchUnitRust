---
layout: default
title: ArchUnitRust
nav_order: 1
description: Install ArchUnitRust, write a first boundary rule, and choose the next guide chapter.
---

# ArchUnitRust

Write the sentence your team already says — *the API must not reach the database* — as an ordinary
Rust test, and let `cargo test` show the dependency evidence when the code disagrees.

```rust,no_run
use archunit::{assert_passes, project_files};

#[test]
fn api_does_not_reach_database() {
    let rule = project_files()
        .in_path("src/api/**")
        .should_not()
        .depend_on_files()
        .in_path("src/database/**");

    assert_passes!(rule);
}
```

There is no configuration file, registry, or test adapter. A rule is an immutable value. Building
one reads nothing; `assert_passes!` or `Checkable::check` locates the Cargo project, extracts the
graph, and evaluates the rule.

## Install

ArchUnitRust 0.0.1 installs from crates.io as a development dependency and requires Rust 1.85 or
newer:

```console
cargo add --dev archunit@0.0.1
```

The equivalent manifest entry is `archunit = "0.0.1"` under `[dev-dependencies]`. Cargo records the
resolved registry version in `Cargo.lock`.

## Your first rule

Put architecture tests in `tests/architecture.rs`, change the paths in the opening example to
match the project, then run:

```console
cargo test --test architecture
```

Three details matter before the second rule:

- **Selectors match normalized, project-relative identifiers.** Plain strings are complete,
  case-sensitive globs. Read [patterns and identifiers](patterns.md) before translating filesystem
  assumptions into rules.
- **An empty scope fails.** A misspelled or stale selector returns an `EmptyTestViolation` rather
  than a false green result. Intentional empty scopes require `CheckOptions`.
- **Violations are data; execution failures are errors.** `Checkable::check` returns
  `Result<Vec<Violation>, ArchUnitError>`. The assertion macro turns either a non-empty violation
  list or an error into one useful test failure.

## Where to go next

- [The grammar](grammar.md) explains entry points, scopes, moods, conditions, and terminals.
- [Patterns and identifiers](patterns.md) defines glob behavior and selector targets.
- [The files family](files.md) covers dependencies, cycles, naming, placement, and callbacks.
- [The layers family](layers.md) turns named folders into one dependency policy.
- [The slices family](slices.md) captures components and checks or draws their relationships.
- [The metrics family](metrics.md) measures Rust code and turns values into thresholds or zones.
- [Dependency-graph reports](graph.md) renders the analyzed graph in six formats.
- [Running a rule](running.md) covers options, logging, ignored imports, results, and failures.
- [How it works](internals.md) describes the source-to-report pipeline and repository boundaries.

The generated [Rust API reference]({{ site.api_reference_path | relative_url }}) sits beside this
guide. The source code and its public doc comments remain the authority when a guide explanation
and a symbol signature differ.
