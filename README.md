# ArchUnitRust

Architecture testing for Rust. Part of **ArchUnitEverything** — one architecture-testing library per language.

> Early development. Nothing to install yet.

The first fluent file rule is usable as an ordinary Rust test value:

```rust
use archunit::{Checkable, project_files};

let rule = project_files()
    .in_folder("src/**")
    .should()
    .have_no_cycles();

let _: &dyn Checkable = &rule;
```

Rules are lazy: building this sentence reads no files. Calling `check()` locates the Cargo project,
extracts its dependency graph, and returns one data-carrying cycle violation per circular path.
The graph model is also available directly:

```rust
use archunit::{Edge, Graph, ImportKind};

let dependency = Edge::new(
    "src/api.rs",
    "src/db.rs",
    false,
    [ImportKind::Use],
);
let graph = Graph::from_edges([dependency.clone()]);

assert_eq!(graph.edges(), &[dependency]);
```

## Rust dependency extraction

`extract_graph` discovers Cargo workspace sources, follows Rust module layout, classifies internal
and external dependencies, merges parallel edges, and returns non-fatal extraction diagnostics with
the graph. Results are memoized per project and extraction configuration; use `clear_graph_cache`
or `CheckOptions::with_clear_cache(true)` when source changes must be observed in the same process.

One `use`, `pub use`, `extern crate`, or `mod` declaration can be omitted from the graph with a Rust
comment on the same or immediately preceding line:

```text
use legacy_client::Client; // archunit: ignore

// Only the matching member of a grouped import is ignored.
use crate::adapters::{legacy, current}; // archunit: ignore crate::adapters::legacy
```

The optional scope matches the written Rust path exactly or by `::` prefix. The directive does not
suppress separate qualified-path expressions, and ignored imports still establish aliases for
resolving those expressions.

Siblings: [ArchUnitTS](https://github.com/LukasNiessen/ArchUnitTS) ·
[ArchUnitPython](https://github.com/LukasNiessen/ArchUnitPython)
