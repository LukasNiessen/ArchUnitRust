---
layout: default
title: The grammar
nav_order: 2
description: The stages of an ArchUnitRust rule, from Cargo project entry point to assertion terminal.
---

# The grammar

The fluent API constrains which words can legally follow one another. A completed check rule has
this shape:

```text
entry point -> subject selectors -> mood -> condition -> optional object selectors -> terminal
```

Graph reports and raw measurements are queries rather than verdicts, so they omit the mood and end
in a rendering or measurement terminal.

## Entry points

The entry point chooses both a feature family and where Cargo discovery begins.

| Family | Auto-discover | Explicit directory or `Cargo.toml` | Alias |
| --- | --- | --- | --- |
| Files | `project_files()` | `project_files_in(path)` | `files()` / `files_in(path)` |
| Layers | `project_layers()` | `project_layers_in(path)` | `layers()` / `layers_in(path)` |
| Slices | `project_slices()` | `project_slices_in(path)` | `slices()` / `slices_in(path)` |
| Metrics | `metrics()` | `metrics_in(path)` | — |
| Graph reports | `project_graph()` | `project_graph_in(path)` | `dependency_graph()` / `dependency_graph_in(path)` |

Auto-discovery starts at the process working directory and walks to the relevant Cargo manifest.
The explicit forms are useful when a test runner starts outside the project being checked.

## Subject selectors

File selectors are `with_name`, `in_folder`, `in_path`, and literal `in_file`. Chaining selectors
narrows the subject with AND semantics:

```rust
use archunit::project_files;

let scope = project_files()
    .in_path("crates/**")
    .in_folder("**/services")
    .with_name("*_service.rs");

assert_eq!(scope.filters().len(), 3);
```

Metrics add `for_types_matching`. Layers name each selection with `layer(...).defined_by(...)`, and
slices derive component names with `defined_by`, `defined_by_regex`, or `with_projection`.

## Mood and condition

`should()` enters the positive mood; `should_not()` enters the negative mood. Naming and placement
conditions invert directly. Dependency conditions have the more useful architecture meaning:

- `should().depend_on_files()` is an allowlist for every outgoing dependency from each subject;
- `should_not().depend_on_files()` is a denylist of matched targets;
- the external-module equivalents apply the same semantics to Cargo-visible crate names.

Cycle freedom is positive only. Slice diagram adherence is positive only, while a forbidden pair of
slices uses `should_not().contain_dependency(source, target)`.

## Object selectors

Some conditions need an object before the sentence is complete. `depend_on_files()` accepts
`with_name`, `in_folder`, `in_path`, and `in_file`; repeated object selectors are OR alternatives.
`depend_on_external_modules()` accepts repeated `matching` crate-name globs.

```rust
use archunit::{Checkable, project_files};

let rule = project_files()
    .in_path("src/api/**")
    .should()
    .depend_on_files()
    .in_path("src/domain/**")
    .in_path("src/common/**");

let _: &dyn Checkable = &rule;
```

This permits API files to depend only on the two named internal areas. Dependencies outside the
analyzed workspace remain external and are governed by a separate external-module condition.

## Terminals

Every architecture condition implements `Checkable`. Use `check()` for typed data, `check_with` for
per-check options, or `assert_passes!` at the test boundary:

```rust,no_run
use archunit::{ArchUnitError, Checkable, project_files};

fn inspect_rule() -> Result<(), ArchUnitError> {
    let rule = project_files().in_path("src/**").should().have_no_cycles();
    let violations = rule.check()?;
    assert!(violations.is_empty());
    Ok(())
}
```

Metrics may instead end in `measure`, `analyze`, or `export_as_html`. Graph queries end in
`snapshot`, `summary`, a `to_*` renderer, or an `export_as_*` writer. Slice scopes can render the
actual component graph with `to_plantuml` or `export_as_plantuml`.

## Builders are values

Builder methods consume and return `Self`; they do not mutate shared global configuration. Clone a
partially built value to branch it:

```rust
use archunit::project_files;

let source = project_files().in_path("src/**");
let services = source.clone().with_name("*_service.rs");
let repositories = source.with_name("*_repository.rs");

assert_eq!(services.filters().len(), 2);
assert_eq!(repositories.filters().len(), 2);
```

Continue with [patterns and identifiers](patterns.md), then choose a family from the
[landing page](index.md#where-to-go-next).
