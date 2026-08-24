---
layout: default
title: The layers family
nav_order: 5
description: Define named Rust source layers once, then express their allowed and forbidden dependency directions.
---

# The layers family

A layer policy names several file selections, then states which named boundaries may be crossed. It
is one immutable `LayeredArchitecture` value and one architecture check.

## Define, then constrain

Each `layer(name)` must be followed by `defined_by(pattern)` or
`defined_by_folder(pattern)`. After the definitions, `where_layer(name)` begins a policy clause:

```rust,no_run
use archunit::{assert_passes, project_layers};

#[test]
fn dependencies_follow_the_declared_layers() {
    let rule = project_layers()
        .layer("api")
        .defined_by("src/api/**")
        .layer("application")
        .defined_by_folder("src/application")
        .layer("database")
        .defined_by("src/database/**")
        .where_layer("api")
        .may_only_depend_on_layers(&["application"])
        .where_layer("application")
        .may_only_depend_on_layers(&["database"])
        .where_layer("database")
        .may_only_depend_on_layers(&[]);

    assert_passes!(rule);
}
```

`project_layers_in(path)` starts discovery explicitly. `layers()` and `layers_in(path)` are aliases.

## Allowlist and blocklist clauses

`may_only_depend_on_layers(&[...])` is an allowlist. An empty target slice seals that source layer
against every cross-layer dependency. `may_not_depend_on_layers(&[...])` is a blocklist and requires
at least one target.

Both clauses may be applied to one source. Blocklists are evaluated first, so one concrete file
edge produces at most one `LayerDependencyViolation` even if both policies reject it.

```rust
use archunit::{Checkable, project_layers};

let rule = project_layers()
    .layer("domain")
    .defined_by("src/domain/**")
    .layer("adapters")
    .defined_by("src/adapters/**")
    .layer("legacy")
    .defined_by("src/legacy/**")
    .where_layer("domain")
    .may_not_depend_on_layers(&["adapters", "legacy"]);

let _: &dyn Checkable = &rule;
```

## Assignment semantics

Three rules keep projection deterministic:

1. Repeating the same `layer(name)` adds another OR selector to that layer.
2. If different layer definitions overlap, the first declared layer wins.
3. Dependencies within one layer, and dependencies with either endpoint outside every declared
   layer, are ignored.

The last rule makes layers a policy over assigned components, not an implicit coverage check. A
source layer named by a policy must still select at least one file unless empty tests are explicitly
allowed.

## Exclusions and inspection

Every definition accepts the shared `pattern(...).except(...)` value. The completed rule exposes
`layer_definitions`, `allowed_dependencies`, and `forbidden_dependencies` as immutable data for
custom reporting:

```rust
use archunit::{pattern, project_layers};

let rule = project_layers()
    .layer("generated-aware")
    .defined_by(pattern("src/**").except("src/generated/**"));

assert_eq!(rule.layer_definitions().len(), 1);
assert!(rule.allowed_dependencies().is_empty());
assert!(rule.forbidden_dependencies().is_empty());
```

Next, derive component names rather than declaring them individually in
[the slices family](slices.md).

