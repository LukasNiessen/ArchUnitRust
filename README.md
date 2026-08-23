# ArchUnitRust

Architecture testing for Rust. Part of **ArchUnitEverything** — one architecture-testing library per language.

> Early development. Nothing to install yet.

A fluent file rule is usable as an ordinary Rust test value:

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

The same scope and mood grammar applies to file naming and placement. All three predicates support
both `should()` and `should_not()`:

```rust
use archunit::{Checkable, project_files};

let naming = project_files()
    .in_folder("src/services")
    .should()
    .have_name("*_service.rs");
let placement = project_files()
    .with_name("*_test.rs")
    .should_not()
    .be_in_path("src/**");

let _: [&dyn Checkable; 2] = [&naming, &placement];
```

Relational rules use the same sentence grammar. Positive dependency rules are allowlists for every
outgoing dependency from the selected files; negated rules are denylists:

```rust
use archunit::{Checkable, project_files};

let boundary = project_files()
    .in_folder("src/api")
    .should_not()
    .depend_on_files()
    .in_folder("src/database");

let _: &dyn Checkable = &boundary;
```

External dependency rules match Cargo-visible crate names. Repeated `matching` selectors are OR
alternatives, making allowlists and denylists straightforward:

```rust
use archunit::{Checkable, project_files};

let approved_crates = project_files()
    .should()
    .depend_on_external_modules()
    .matching("std")
    .matching("serde");

let _: &dyn Checkable = &approved_crates;
```

Custom predicates cover project-specific facts without expanding the built-in vocabulary. They
receive immutable, normalized `FileInfo` data and run once for each selected file:

```rust
use archunit::{Checkable, FileInfo, project_files};

let manageable_modules = project_files()
    .in_folder("src/**")
    .should()
    .adhere_to(
        |file: &FileInfo| file.non_blank_line_count <= 200,
        "contain at most 200 non-blank lines",
    );

let _: &dyn Checkable = &manageable_modules;
```

Alongside the line count and full source text, `FileInfo` exposes the normalized
workspace-relative path, filename without extension, extension and containing directory. Stored
predicates are `Send + Sync + 'static`; captured configuration therefore needs owned,
thread-safe values.

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
