# ArchUnitRust porting plan

**Status:** accepted direction for the initial implementation
**Last reviewed:** 2026-08-23
**Tracking issue:** [#45](https://github.com/LukasNiessen/ArchUnitRust/issues/45)

This document turns the shared ArchUnitEverything architecture into an executable Rust plan. It is
not a second product specification. The shared architecture and the issue backlog define the common
product; this document records where Rust requires a deliberate interpretation.

## Evidence used

The plan was formed from the current source, tests, issue history, and documentation of:

- [ArchUnitTS](https://github.com/LukasNiessen/ArchUnitTS), the longest-running public API and the
  broadest reference implementation;
- [ArchUnitGo](https://github.com/LukasNiessen/ArchUnitGo), the clearest current example of a
  toolchain-backed extractor, strict module boundaries, file-by-file unit tests, fixture tests, and
  self-hosted architecture checks;
- [ArchUnitNET](https://github.com/LukasNiessen/ArchUnitNET), especially its Roslyn-backed semantic
  extraction and real-project fixtures;
- [ArchUnitRuby](https://github.com/LukasNiessen/ArchUnitRuby), especially its complete 1-44 issue
  path, AST extraction, report modules, profiling, packaging, and CI;
- the shared `ARCHITECTURE.md` and `FEATURES.md` kickoff artifacts.

The sibling source is evidence, not something to transliterate. TypeScript documentation is known to
contain APIs absent from its implementation, while .NET status documents and open issues show places
where "implemented" is not yet the same as release-ready. ArchUnitRust therefore treats executable
tests and public source as truth and keeps a candid "not implemented" list.

## Product promise

ArchUnitRust turns a Cargo project into a directed dependency graph and lets users assert architecture
rules as ordinary Rust tests. A user who knows another ArchUnit library should recognize the fluent
sentence, but Rust naming, ownership, error handling, and test integration remain idiomatic.

The invariant pipeline is:

```text
SOURCE -> EXTRACT -> PROJECT -> ASSERT -> REPORT
```

Only source discovery and extraction know Rust. Projection and assertion stay pure and are tested
against hand-built graphs. Fluent builders are lazy values; only a terminal reads the project.

## Initial crate contract

- The crates.io package and library crate are both named `archunit`. The name was unclaimed when
  checked on 2026-08-23.
- Edition 2024 is the baseline. The initial minimum supported Rust version is 1.85, the first stable
  release supporting that edition. CI must test the MSRV as well as stable before this becomes a
  compatibility promise in a release.
- `#![forbid(unsafe_code)]` applies to the library.
- Builders consume `self` and return a new value. Terminals borrow the completed rule.
- `check()` uses defaults. `check_with(&CheckOptions)` supplies options without spreading
  `Option<CheckOptions>` through the API.
- A technical failure is `Err(ArchUnitError)`. A rule failure is `Ok(Vec<Violation>)` with at least
  one data-carrying violation.
- `Violation` is a non-exhaustive enum, not a trait-object hierarchy. This is an intentional Rust
  divergence that preserves exhaustive internal matching while allowing new variants in minor
  releases.

## Source and project model

### Project location

With no locator, walk from the current directory to the nearest `Cargo.toml`, then ask
`cargo metadata --format-version 1` for the workspace. An explicit locator may name a manifest or a
directory. Cargo metadata, rather than ad hoc TOML parsing, is authoritative for:

- workspace membership, including virtual workspaces;
- package names and dependency renames;
- library, binary, proc-macro, build-script, example, benchmark, and test targets;
- target source roots and the workspace target directory.

Production targets (`lib`, `bin`, `proc-macro`, and `custom-build`) are included by default. An
analysis option can include test, example, and benchmark targets. `target/`, VCS metadata, vendored
dependencies, and files outside workspace members are excluded by default.

### Stable identifiers

Every internal identifier is a normalized, forward-slash-separated path relative to the workspace
root. This is stable across machines and unique across workspace members. An external target is the
Cargo-visible crate name (including a dependency rename), not a registry URL or checkout path.

Every discovered source file receives a self-edge. Parallel `(source, target)` edges are merged and
their import kinds are unioned. Deterministic ordering is part of the contract so violations, JSON,
and diagrams produce reviewable diffs.

### Crates, modules, and files

Each Cargo target source is a crate root. The extractor builds a module index by following inline and
outlined `mod` items, including `#[path = "..."]` where the path is a literal. It implements Rust's
different lookup bases for `lib.rs`/`main.rs`/`mod.rs` and ordinary module files. The index maps a
logical module path and target context to the owning file.

A single file can participate in more than one target. Its filesystem identifier remains one node;
edges found in different valid target contexts merge at the graph boundary.

### Dependency syntax

Rust can depend on another module without a `use` item, so copying an import-only extractor would
create dangerous false negatives. The Rust extractor records:

- `mod child;` declarations as `Mod` edges to outlined module files;
- `use` trees, including nesting, globs, aliases, `self`, `super`, and `crate`, as `Use` or `PubUse`;
- `extern crate`, including aliases, as `ExternCrate`;
- fully qualified paths in expressions, types, patterns, signatures, impls, and attributes as
  `PathReference`;
- the defining path of a macro invocation or derive/attribute macro as `MacroReference` when that
  path can be identified without expansion.

For an internal path, the target is the file owning the longest resolvable module prefix. An item
declared later in that file does not create a separate node. For an external path, the target is the
first segment after Cargo rename normalization. Same-file references merge into the self-edge.

Resolution order is explicit prefixes first (`crate`, `self`, repeated `super`), then bindings in the
containing module, then crate-root modules and Cargo's external prelude according to the package
edition. Ambiguous unqualified paths are not guessed: they produce an extraction diagnostic and are
omitted from the dependency graph.

One declaration can be suppressed with `// archunit: ignore` on the declaration line or the
immediately preceding line. An optional Rust path scopes the directive, for example
`// archunit: ignore crate::legacy`; scoped paths match exactly or by `::` prefix within a grouped
import. The initial contract applies only to `use`, `pub use`, `extern crate`, and `mod`
declarations. It deliberately does not suppress separate qualified-path syntax, whose comment/span
attachment would be ambiguous. Ignored imports still participate in alias resolution so later,
non-ignored qualified paths retain Rust semantics.

### Conservative boundary

The first extractor uses `cargo metadata` plus `syn`; it does not embed rustc or rust-analyzer. The
detailed decision and alternatives are in
[ADR 0001](adr/0001-syntax-and-module-tree-extraction.md).

Consequences that must be visible in the README and API docs:

- macro bodies are not expanded, so dependencies generated entirely by a macro are invisible;
- a literal `include!("file.rs")` may be supported later, but generated `OUT_DIR` sources are outside
  the initial source graph;
- `cfg`-guarded source is analyzed conservatively as a union by default. This avoids cross-platform
  false negatives at the cost of possible false positives; active-target evaluation is a later
  analysis option;
- parse failures are collected as diagnostics and the file is skipped. A project with no analyzable
  source is a technical error rather than a passing rule;
- no network access or dependency build is required to analyze a project once Cargo metadata can be
  produced with `--no-deps`.

These limitations are not reasons to weaken the rest of the graph contract. They are test cases and
documented compatibility boundaries.

## Public vocabulary

Keep the shared grammar and translate casing only:

```rust
project_files()
    .in_folder("src/api/**")
    .should_not()
    .depend_on_files()
    .in_folder("src/db/**")
```

`project_files`, `files`, `project_layers`, `layers`, `project_slices`, `project_graph`,
`dependency_graph`, and `metrics` are free functions. A locator is optional through a separate
`*_in(locator)` entry point or a builder modifier; Rust does not have optional function arguments.
The exact spelling will be locked by compile-pass API tests before the files module is declared
stable.

Patterns remain the shared substrate: glob strings compile once to regex, separators normalize,
selectors combine with AND, alternatives inside one selector combine with OR, and `except` is
available from the beginning rather than retrofitted at issue #38.

## Metrics mean Rust concepts

Rust does not have classes. The shared metrics goals are preserved with a truthful Rust model:

- `TypeInfo` describes structs, enums, unions, and traits;
- `ImplInfo` associates inherent and trait impl blocks with their target type;
- methods are associated functions with a `self` receiver; free associated functions are counted
  separately;
- fields exist for structs, unions, and enum variants; traits have required/provided items instead;
- file counts cover physical lines, logical statements/items, `use` declarations, free functions,
  concrete types, traits, impl blocks, and macros;
- the selector is `for_types_matching`, not `for_classes_matching`. An alias is not supplied merely
  for cosmetic cross-language parity.

LCOM is meaningful initially for structs with inherent methods. Field access is derived from
`self.field` expressions. Traits, enums, unions, trait impls, generated methods, and macro-expanded
field access are excluded from LCOM rather than assigned invented values.

Distance metrics initially use analyzed Rust source files as components. Package, crate, and inferred
module aggregation remain future explicit projections because files are currently the dependency
graph's only lossless evidence-carrying boundary. The shared formulas adapt as follows:

- abstractness is traits divided by traits plus concrete types for the selected component;
- instability is efferent coupling divided by total afferent and efferent coupling;
- main-sequence distance, normalized distance, coupling factor, and zone checks use those values.

Coupling is always calculated across the full project snapshot; fluent selectors narrow reported
components without changing their topology. See ADR 0016 for formulas, denominator behavior, and
strict zone boundaries.

Custom metrics receive immutable `TypeInfo` through generic owned callbacks. Custom predicate panics
propagate as ordinary Rust panics rather than being reclassified as violations. All threshold verbs
remain the shared six. Every metric documents its population and zero-denominator behavior with
table-driven tests.

## Test and quality strategy

Every issue lands both kinds of tests where applicable:

1. unit tests beside pure implementation modules under `#[cfg(test)]`;
2. black-box integration tests under `tests/` through the public fluent API.

`tests/fixtures/` contains complete Cargo projects, not loose source snippets. Fixtures will cover:

- a single crate with inline, modern outlined, and legacy `mod.rs` modules;
- a workspace with renamed internal and external dependencies;
- library and binary targets sharing modules;
- aliases, grouped imports, globs, `crate`/`self`/`super`, `pub use`, and fully qualified paths;
- `#[path]`, `cfg`, ignored edges, syntax errors, cycles, empty selections, and isolated files;
- a deliberately layered project used by file, layer, slice, graph, and metrics integration tests.

Required local and CI gates grow with the project and ultimately include:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo test --doc
cargo doc --workspace --all-features --no-deps
cargo package --allow-dirty
```

The test matrix covers stable Rust on Linux, Windows, and macOS plus the declared MSRV on Linux.
Extraction tests use temporary directories and never depend on test ordering. Golden render outputs
are deterministic and reviewed as text.

## Sequential delivery plan

Only one implementation branch and one pull request may be open at a time. Each PR closes one issue
unless two tickets are inseparable; any grouping is explained before code is written. Merge commits
preserve the small conventional commits made on the branch.

| Milestone | Issues | Exit condition |
|---|---:|---|
| Strategy | #45 | Decisions and workflow are versioned |
| Kernel | #1-#6, with #38 semantics introduced alongside matching | Public graph, matcher, violation, check, and error contracts compile |
| Extraction | #7-#12 | Fixture workspaces produce deterministic, correctly classified graphs |
| Projection | #13-#15 | Pure edge/node/cycle projections are exhaustive and deterministic |
| Files MVP | #16-#23 | The primary fluent sentence works end to end with empty-test safety |
| Testing | #24-#26 | Violations render consistently and `assert_passes!` works in `#[test]` |
| Policies and reports | #27-#31 | Layers, snapshots/renderers, slices, and PlantUML work on fixtures |
| Metrics | #32-#37 | Rust-defined counts, cohesion, coupling, custom metrics, and HTML export work |
| Cross-cutting | #39 | Opt-in deterministic logging is covered |
| Self-hosting | #40 | ArchUnitRust enforces its own module rules |
| Adoption | #41-#43 | README, rustdoc/GitHub Pages, and the full CI matrix are live |
| Release | #44 | A verified package is published and install instructions are tested |

Issue #38 stays open until every selector family has exclusion coverage, even though the matching
model is designed for exclusions in the kernel. Release notes state exactly which selectors have it.

## Definition of done for each issue

- Acceptance criteria are reconciled with Rust before implementation.
- A conventional branch is created from current `main`.
- Commits are small, buildable where possible, and reference the issue.
- Public behavior has unit and integration coverage; failure messages are asserted where relevant.
- Formatting, Clippy, tests, and documentation checks pass locally.
- The PR explains deliberate sibling divergence and closes the issue.
- CI is green, the PR is merged, the remote feature branch is deleted, and local `main` is refreshed
  before the next issue begins.

## v0.1 boundary

The first publishable release includes deterministic Cargo workspace extraction, files, layers,
slices, graph reports, Rust-relevant metrics, testing helpers, empty-test protection, exclusions,
logging, self-hosting tests, documentation, and CI.

It does not promise compiler-exact macro expansion, build-script-generated source analysis,
active-target `cfg` evaluation, rust-analyzer semantic parity, or item-level architecture nodes.
Those are roadmap features. File nodes remain the compatibility surface even though the internal
module index is Rust-native.
