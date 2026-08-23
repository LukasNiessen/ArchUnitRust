# ArchUnitRust

Architecture testing for Rust. Part of **ArchUnitEverything** — one architecture-testing library per language.

> Early development. Nothing to install yet.

A fluent file rule is an ordinary Rust test value. The zero-configuration assertion macro is the
universal path through the built-in harness and any Rust test framework that recognizes assertion
panics:

```rust,no_run
use archunit::{assert_passes, project_files};

#[test]
fn architecture_is_acyclic() {
    let rule = project_files()
        .in_folder("src/**")
        .should()
        .have_no_cycles();

    assert_passes!(rule);
}
```

Rules are lazy: building this sentence reads no files. `assert_passes!` calls `check()`, locates the
Cargo project, extracts its dependency graph and emits the shared numbered failure message when
violations are found. The macro borrows the rule, so named terminals remain reusable.

A scope that selects no files returns one typed `EmptyTestViolation` by default; it never silently
passes because a path was misspelled or became stale. This guard applies to every terminal and
checks selected files rather than dependency edges, so an existing isolated file is not considered
empty. Intentional empty scopes require an explicit per-check opt-out:

```rust,no_run
use archunit::{CheckOptions, assert_passes, project_files};

let optional_rule = project_files()
    .in_folder("generated/**")
    .should()
    .have_no_cycles();
let options = CheckOptions::new().with_allow_empty_tests(true);

assert_passes!(optional_rule, options);
```

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

## Named layer policies

Layers turn a set of file selectors into a compact dependency policy. The target list is a borrowed
slice in Rust; an empty allowlist seals a layer against every cross-layer dependency:

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

`layers()` aliases `project_layers()`; explicit project entry points are `layers_in(path)` and
`project_layers_in(path)`. Repeating `layer(name)` adds another OR selector to that layer. If layer
definitions overlap, the first declared layer wins. Dependencies within one layer and dependencies
with either endpoint outside every declared layer are ignored.

`may_not_depend_on_layers(&[...])` adds a blocklist. Blocklists are evaluated before allowlists, so
one file edge produces at most one layer violation even when both policies reject it. Source layers
used by a policy receive the same strict empty-selection guard as file rules.

## Dependency graph snapshots

Graph reports first build one renderer-neutral snapshot. Every query modifier is immutable and lazy;
only `snapshot()` or `summary()` locates and extracts the Cargo project:

```rust,no_run
use archunit::{ArchUnitError, CheckOptions, project_graph};

fn architecture_snapshot() -> Result<(), ArchUnitError> {
    let snapshot = project_graph()
        .include_external_dependencies()
        .focus_on("src/**", 1)
        .reachable_from("src/api/**")
        .collapse_to_folder_depth(2)
        .titled("Application Dependencies")
        .with_check_options(CheckOptions::new().with_clear_cache(true))
        .snapshot()?;

    println!("{} nodes", snapshot.summary.node_count);
    Ok(())
}
```

`dependency_graph()` is an alias; `project_graph_in(path)` and `dependency_graph_in(path)` start at
an explicit directory or manifest. Queries include undirected `focus_on(pattern, depth)`, transitive
outgoing `reachable_from(pattern)`, and transitive incoming `dependents_of(pattern)`. When several
are present, their selected nodes are combined as a union and the snapshot contains the induced
subgraph.

External dependencies and self dependencies are excluded by default. Collapse with
`collapse_to_folder_depth(depth)` or use a Rust regular expression whose first capture becomes the
label with `collapse_by_pattern(expression)`. The explicit replacement form is
`collapse_by_pattern_with_replacement(expression, replacement)` and uses Rust `regex` syntax such as
`$1` or `${component}`.

The snapshot owns stable sorted node IDs, aggregated edges, Rust import-kind unions, a title, and
summary counts. `raw_edge_count` counts selected merged file-to-file edges before collapsing;
`edge_count` counts the final aggregated edges. This snapshot is the single input contract for every
output format.

## Dependency graph renderers

The graph builder renders DOT, Mermaid, D2, CSV, JSON, and self-contained HTML. Each format has a
`to_*()` string terminal and an `export_as_*()` UTF-8 file terminal. Export creates missing parent
directories; the chosen method determines the format rather than the file extension.

```rust,no_run
use archunit::{ArchUnitError, GraphRenderer, project_graph};

fn export_architecture() -> Result<(), ArchUnitError> {
    let report = project_graph()
        .collapse_to_folder_depth(2)
        .titled("Application Dependencies");

    let mermaid = report.to_mermaid()?;
    println!("{mermaid}");
    report.export_as_html("target/architecture/dependencies.html")?;

    // Reuse one extraction explicitly when several formats are needed.
    let snapshot = report.snapshot()?;
    let dot = GraphRenderer::to_dot(&snapshot);
    let json = GraphRenderer::to_json(&snapshot);
    assert!(!dot.is_empty() && !json.is_empty());
    Ok(())
}
```

The six corresponding methods are `to_dot`, `to_mermaid`, `to_d2`, `to_csv`, `to_json`, and
`to_html`, plus `export_as_dot`, `export_as_mermaid`, `export_as_d2`, `export_as_csv`,
`export_as_json`, and `export_as_html`. `GraphRenderer::render` and `GraphRenderer::export` provide
typed dispatch through `GraphReportFormat`.

DOT, Mermaid, and D2 retain aggregated edge counts and visually distinguish external dependencies.
CSV contains one row per aggregated edge. JSON contains the complete snapshot contract. HTML embeds
its CSS and portable source views directly in the document: it has no scripts, remote assets, or
network dependency.

## Testing and framework-neutral results

`assert_passes!(rule)` and `assert_passes!(rule, check_options)` are the native integration for
ordinary `#[test]` functions. They preserve both formatted architecture violations and classified
check errors in the assertion message. Rust's built-in harness has no adapter registry, so importing
the macro is the complete setup.

Violations remain structured data until that testing layer formats them. `ViolationFactory` owns
the wording for every built-in violation; `ResultFactory` adds pass/fail semantics, numbering and
optional ANSI color. These values remain the low-level bridge for custom test or CI integration:

```rust,no_run
use archunit::{
    Checkable, ColorChoice, ResultFactory, TestResultOptions, project_files,
};

let rule = project_files()
    .in_folder("src/**")
    .should()
    .have_no_cycles();
let violations = rule.check().expect("the architecture check should run");
let display = TestResultOptions::new().with_color(ColorChoice::Never);
let result = ResultFactory::from_violations_with_options(&violations, &display);

assert_eq!(result.passed, violations.is_empty());
```

`ColorChoice::Auto` is the default and respects terminal capability, `NO_COLOR`, `TERM=dumb`, and
`CI=true`; `Always` and `Never` make output deterministic. The assertion macro uses these same
factories, so message formatting does not drift between integrations.

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
