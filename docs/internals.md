---
layout: default
title: How it works
nav_order: 10
description: The ArchUnitRust source-to-report pipeline, Rust syntax model, repository layout, self-enforced boundaries, and documentation checks.
---

# How it works

ArchUnitRust turns a Cargo workspace into a directed dependency graph, reshapes that graph into the
vocabulary of a rule or report, and keeps the verdict separate from its presentation.

## The pipeline

```text
SOURCE -> EXTRACT -> PROJECT -> ASSERT -> REPORT
```

| Stage | Responsibility | Rust-specific? |
| --- | --- | --- |
| Source | Locate a package or workspace and enumerate selected Cargo targets | Cargo-specific |
| Extract | Parse modules and references into normalized `Edge` values | Yes |
| Project | Relabel or select edges as files, layers, slices, metrics, or report nodes | Mostly no |
| Assert | Compare immutable projected data with one condition and emit violations | No |
| Report | Format test output or render an artifact | Only at integration edges |

Almost all language-specific work belongs in extraction. Projection and assertion operate on the
same graph shapes used across the ArchUnitEverything family.

## Rust source extraction

Project discovery uses `cargo_metadata`; syntax extraction uses `syn` with `proc_macro2` source
locations. The extractor understands Rust module layout, inline and outlined modules, literal
`#[path = "..."]`, `use`, `pub use`, `extern crate`, `mod`, qualified expression/type/pattern paths,
impls, and qualified attribute or derive paths.

Every internal source file receives a self-edge so isolated files remain visible. Parallel edges
with the same source and target are merged while unioning their `ImportKind` values. Internal
identifiers are stable workspace-relative paths with normalized separators.

The analysis is deliberately syntax-based. It does not expand declarative or procedural macros,
inspect build-script-generated source, evaluate the active target's `cfg` expressions, promise
rust-analyzer semantic parity, or expose item-level architecture nodes. `cfg` branches are analyzed
as a conservative union; non-fatal ambiguities remain available as `ExtractionDiagnostic` values.

## Repository layout

```text
src/common/      shared extraction, projection, matching, errors, options, logging
src/files/       file rules and FileInfo extraction
src/layers/      named-layer policies
src/slices/      slice projections, pair rules, PlantUML checking and rendering
src/metrics/     Rust source metrics, conditions, and HTML reports
src/graph/       report queries, snapshots, and six renderers
src/testing/     assertion evaluation and stable violation formatting
src/checkable.rs cross-domain terminal contract
src/violation.rs closed cross-domain violation sum
src/lib.rs       outward-only public re-export surface
```

Each domain owns its `fluentapi` and `assertion` folders, adding `projection`, `extraction`,
`calculation`, or `reporting` only when needed. Pure assertion, projection, and calculation code is
tested with hand-built in-memory values before integration fixtures exercise the Cargo boundary.

## Self-enforced architecture

`tests/architecture.rs` dogfoods the public API and keeps five directions executable:

1. `common` depends only on itself, the standard library, and the explicit analysis toolchain.
2. `files`, `graph`, `layers`, `metrics`, and `slices` do not depend on one another.
3. `checkable` and `violation` are the intentional closed aggregation seam above domain data.
4. `testing` consumes those shared and domain contracts; product domains do not consume testing.
5. `lib.rs` depends outward as the public facade, while implementation files never import through it.

Each architectural unit also rejects executable dependency cycles. The rule excludes structural
`Mod` and `PubUse` evidence while retaining any parallel `Use`, `PathReference`, or other executable
evidence on the same edge.

## Adding a rule

A new check normally follows one path:

1. write the fluent sentence and choose its owning domain;
2. define a data-only domain violation;
3. implement the pure gather/assertion function over projected data;
4. add consuming fluent stages and a terminal implementing `Checkable`;
5. apply the strict empty-subject guard;
6. add centralized formatting in `testing`;
7. test the pure behavior, public builder, fixture project, error path, and failure message;
8. update the family page and API doc comments in the same change.

The closed `Violation` enum is an intentional Rust adaptation: callers get typed accessors and one
stable catalogue, while each domain continues to own its violation data.

## Why the documents are tested

The README is included as crate-level documentation. Every guide page is attached to a private
`cfg(doctest)` host in `src/site_docs.rs`, so every Rust fence compiles against the current public
crate without becoming shipped API. A separate site-integrity test checks front matter, navigation
order, local links and fragments, the expected chapter set, layout wiring, and deployment inputs.

The Pages workflow builds these Markdown chapters with GitHub's Jekyll action and builds the
generated rustdoc from the same commit. There is no JavaScript application, documentation package
manager, or second hand-written API reference to drift.

Read the generated [Rust API reference]({{ site.api_reference_path | relative_url }}) for signatures
and type-level contracts, or return to [the landing page](index.md).

