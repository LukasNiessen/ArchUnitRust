---
layout: default
title: Running a rule
nav_order: 9
description: Execute ArchUnitRust rules through the built-in test harness or structured results, with explicit options, caching, logging, and ignored imports.
---

# Running a rule

A fluent sentence is lazy. Only a terminal locates the Cargo project, reads sources, extracts the
graph, and evaluates the condition.

## As a Rust test

`assert_passes!` is the native integration for an ordinary `#[test]`. It borrows the completed rule,
evaluates it once, and preserves formatted violations or a classified execution error in the panic
message.

```rust,no_run
use archunit::{assert_passes, project_files};

#[test]
fn source_is_acyclic() {
    let rule = project_files().in_path("src/**").should().have_no_cycles();
    assert_passes!(rule);
}
```

Any Rust test framework that recognizes assertion panics needs no additional adapter or setup.

## As structured data

Every terminal implements the object-safe `Checkable` contract:

```rust,no_run
use archunit::{ArchUnitError, Checkable, ViolationKind, project_files};

fn inspect() -> Result<(), ArchUnitError> {
    let rule = project_files()
        .in_path("src/api/**")
        .should_not()
        .depend_on_files()
        .in_path("src/database/**");
    let violations = rule.check()?;

    for violation in &violations {
        println!("{}", violation.kind());
        assert_eq!(violation.kind(), ViolationKind::FileDependency);
    }
    Ok(())
}
```

`Ok(Vec::new())` means the rule passed. `Ok` with violations is an architecture verdict.
`Err(ArchUnitError::User(_))` means invalid API input; `Err(ArchUnitError::Technical(_))` means the
library or environment could not complete the check.

The closed `Violation` enum retains typed data for empty selections, cycles, file patterns,
internal or external dependencies, custom file predicates, layers, slices, metric zones, custom
metrics, numeric thresholds, and metric predicates. `ViolationFactory` and `ResultFactory` turn
that data into consistent test output only at the reporting edge.

## Check options

`CheckOptions::new()` is strict, quiet, cache-friendly, and production-only. Its consuming modifiers
are:

| Modifier | Effect |
| --- | --- |
| `with_allow_empty_tests(bool)` | opt out of the strict empty-selection guard |
| `with_logging(LoggingOptions)` | enable this check's explicit logger |
| `with_clear_cache(bool)` | discard a matching extraction cache entry before this run |
| `with_test_sources(bool)` | include Cargo test, example, and benchmark targets |

Pass the value to `check_with(&options)` or as the macro's second argument:

```rust,no_run
use archunit::{CheckOptions, assert_passes, project_files};

let options = CheckOptions::new().with_test_sources(true);
let rule = project_files().in_path("tests/**").should().have_no_cycles();
assert_passes!(rule, options);
```

## Cache behavior

Extraction results are memoized by project identity and source options. Repeated rules in one test
process can share the same immutable graph. Use `with_clear_cache(true)` when the process changed
source files after a previous check, or call `clear_graph_cache()` for an explicit global reset.

## Per-check logging

Logging has no environment-variable or process-global switch. Put one `LoggingOptions` value into
the options for the check that needs it:

```rust,no_run
use archunit::{
    ArchUnitError, CheckOptions, Checkable, LogFileMode, LogLevel, LoggingOptions, project_files,
};

fn logged_check() -> Result<(), ArchUnitError> {
    let logging = LoggingOptions::new()
        .with_level(LogLevel::Debug)
        .with_console_output(false)
        .with_file_output("target/architecture-logs")
        .with_file_mode(LogFileMode::Overwrite);
    let options = CheckOptions::new().with_logging(logging);
    let rule = project_files()
        .in_path("src/api/**")
        .should_not()
        .depend_on_files()
        .in_path("src/database/**");

    assert!(rule.check_with(&options)?.is_empty());
    Ok(())
}
```

The levels are `Debug`, `Info`, `Warn`, and `Error`. The event vocabulary distinguishes check start
and end, progress, violations, metrics, and ordinary caller records. File output creates parent
directories and a collision-resistant timestamped `.log`; `file_path()` exposes the artifact path
before the check so CI can archive it.

## Keep one import out of the graph

Put `archunit: ignore` on the same line as an import or immediately above it:

```text
use legacy_client::Client; // archunit: ignore

// Only this member of the grouped import is ignored.
use crate::adapters::{legacy, current}; // archunit: ignore crate::adapters::legacy
```

An optional scope matches the written Rust path exactly or by `::` prefix. Ignored imports still
establish aliases for resolving later qualified paths, and the directive does not suppress a
separate expression or type path.

## When a rule selects nothing

The default `EmptyTestViolation` protects against misspelled and stale scopes. It checks selected
subjects, not derived edges, so an existing isolated file is not empty. Make a genuinely optional
scope explicit:

```rust,no_run
use archunit::{CheckOptions, assert_passes, project_files};

let rule = project_files()
    .in_path("generated/**")
    .should()
    .have_no_cycles();
let options = CheckOptions::new().with_allow_empty_tests(true);

assert_passes!(rule, options);
```

Continue with [how it works](internals.md) for the extraction pipeline and the boundaries the crate
enforces on itself.
