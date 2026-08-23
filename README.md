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
