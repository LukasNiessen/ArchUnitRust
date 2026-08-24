---
layout: default
title: The files family
nav_order: 4
description: File rules for dependency boundaries, external crates, cycles, naming, placement, and custom Rust predicates.
---

# The files family

File rules are the direct view of the extracted Cargo source graph: select source files, choose a
mood, and state the condition each selected file must satisfy.

## Scope

Start with `project_files()` or its `project_files_in(path)` form. Subject selectors can be chained
in any order and use AND semantics:

- `with_name(pattern)` matches the final filename;
- `in_folder(pattern)` matches the containing path;
- `in_path(pattern)` matches the complete project-relative path;
- `in_file(path)` selects one exact normalized source file.

The aliases are `files()` and `files_in(path)`. See [patterns and identifiers](patterns.md) for glob
and exclusion behavior.

## The conditions

| Condition | Positive mood | Negative mood |
| --- | --- | --- |
| cycles | `should().have_no_cycles()` | — |
| filename | `should().have_name(pattern)` | `should_not().have_name(pattern)` |
| folder | `should().be_in_folder(pattern)` | `should_not().be_in_folder(pattern)` |
| path | `should().be_in_path(pattern)` | `should_not().be_in_path(pattern)` |
| internal dependency | `should().depend_on_files()` | `should_not().depend_on_files()` |
| external Cargo crate | `should().depend_on_external_modules()` | `should_not().depend_on_external_modules()` |
| callback | `should().adhere_to(...)` | `should_not().adhere_to(...)` |

Each completed condition implements `Checkable` and returns typed `Violation` values.

## A boundary between two folders

A negative dependency rule is a denylist. Every matching internal edge becomes a
`FileDependencyViolation` carrying the source, target, and raw Rust import evidence.

```rust
use archunit::{Checkable, project_files};

let rule = project_files()
    .in_path("src/api/**")
    .should_not()
    .depend_on_files()
    .in_path("src/database/**");

let _: &dyn Checkable = &rule;
```

A positive dependency rule is an allowlist: every outgoing internal dependency from each selected
source must match at least one target selector. Dependencies within the selected source set are not
implicitly exempt; list every permitted area deliberately.

## External crates

External rules inspect crate names as Rust code sees them. Repeated `matching` calls are OR
alternatives:

```rust
use archunit::{Checkable, project_files};

let rule = project_files()
    .in_path("src/common/**")
    .should()
    .depend_on_external_modules()
    .matching("std")
    .matching("core")
    .matching("syn");

let _: &dyn Checkable = &rule;
```

Use `should_not()` for a denylist instead. Cargo renames matter: a dependency renamed in
`Cargo.toml` is matched by the name available to source code.

## Cycles

`have_no_cycles` checks cycles wholly inside the selected file set. Rust module ownership can add
structural `Mod` and `PubUse` edges that are not executable coupling. Exclude those syntax kinds
when that distinction is part of the policy:

```rust
use archunit::{Checkable, ImportKind, project_files};

let rule = project_files()
    .in_path("src/files**")
    .should()
    .have_no_cycles()
    .excluding_dependency_kinds([ImportKind::Mod, ImportKind::PubUse]);

let _: &dyn Checkable = &rule;
```

Filtering is evidence-aware. If a merged source-target edge also contains `Use`,
`PathReference`, or another retained kind, the edge remains in the cycle graph with that evidence.

## Naming and placement

Naming and placement conditions use the same matcher targets as the scope:

```rust
use archunit::{Checkable, project_files};

let services = project_files()
    .in_folder("src/services")
    .should()
    .have_name("*_service.rs");
let tests = project_files()
    .with_name("*_test.rs")
    .should_not()
    .be_in_path("src/**");

let _: [&dyn Checkable; 2] = [&services, &tests];
```

## A project-specific predicate

`adhere_to` receives immutable `FileInfo` for every selected source. It exposes the normalized path,
filename without extension, extension, containing directory, full source, and non-blank line count.

```rust
use archunit::{Checkable, FileInfo, project_files};

let rule = project_files().in_path("src/**").should().adhere_to(
    |file: &FileInfo| file.non_blank_line_count <= 200,
    "contain at most 200 non-blank lines",
);

let _: &dyn Checkable = &rule;
```

The stored callback must be `Send + Sync + 'static`. Panics from user callbacks propagate normally;
returning `false` produces a `CustomFileViolation` with the supplied message and file facts.

Next, give several file scopes names in [the layers family](layers.md).

