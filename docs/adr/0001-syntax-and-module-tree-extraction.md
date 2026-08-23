# ADR 0001: Use Cargo metadata, `syn`, and an explicit module tree

- **Status:** accepted
- **Date:** 2026-08-23
- **Decision owners:** ArchUnitRust maintainers
- **Tracking:** [#45](https://github.com/LukasNiessen/ArchUnitRust/issues/45),
  [#7](https://github.com/LukasNiessen/ArchUnitRust/issues/7),
  [#8](https://github.com/LukasNiessen/ArchUnitRust/issues/8),
  [#9](https://github.com/LukasNiessen/ArchUnitRust/issues/9)

## Context

ArchUnit needs a repeatable file dependency graph without compiling the user's project. Rust source
resolution is not equivalent to scanning `use` lines:

- Cargo workspaces contain multiple packages and targets;
- a module may be inline, in `name.rs`, in `name/mod.rs`, or redirected by `#[path]`;
- a dependency may be written as `use`, `pub use`, `extern crate`, `mod`, or a fully qualified path;
- imports can be grouped, globbed, renamed, or relative through `crate`, `self`, and `super`;
- Cargo dependencies may be renamed;
- `cfg`, macros, build scripts, and generated source mean source syntax is not always the compiler's
  final program.

The extractor is the only language-specific stage. A bad boundary here would either leak Rust
details into every rule or create false-negative architecture checks.

## Decision

Use `cargo metadata --format-version 1 --no-deps` through the `cargo_metadata` crate to discover the
workspace, packages, dependencies, target roots, editions, and target directory.

Use `syn` with its full syntax tree and visit support to parse source. Build an explicit module index
by following module declarations according to the Rust Reference, including literal `#[path]`
attributes. Flatten `use` trees and visit fully qualified paths. Resolve them against the module
index, local bindings, crate root, and Cargo external prelude.

The public graph contains normalized workspace-relative file identifiers. Unresolvable or ambiguous
syntax yields diagnostics and no guessed edge. Parse failures skip one file; total absence of
analyzable source is a technical error.

Analyze `cfg` branches as a conservative union initially. Do not expand procedural or declarative
macros. Record macro-defining paths when visible, but do not claim dependencies generated inside
their expansion.

Keep this implementation behind the private extraction boundary so a future semantic backend can
replace it without changing fluent rules, projections, violations, or report formats.

## Alternatives considered

### rustc private APIs

Rejected for the initial release. They offer compiler truth but require nightly/internal crates,
track compiler internals closely, significantly raise build cost, and make ArchUnitRust's MSRV and
distribution story brittle.

### Embedding rust-analyzer crates

Rejected for the initial release. rust-analyzer provides excellent semantic resolution, but its
internal crate graph is large and intentionally evolves with the tool. Depending directly on those
crates would make a small architecture test library expensive to build and maintain.

### Requiring an external rust-analyzer process

Rejected. It adds an undeclared executable/configuration dependency, complicates editors and CI,
and makes test results depend on server state and protocol negotiation.

### Text or regex scanning

Rejected. It cannot correctly flatten use trees, distinguish comments and strings, follow module
layout, or find qualified paths. The apparent simplicity would produce silent false negatives.

### `cargo check` diagnostics or rustdoc JSON

Rejected as the primary graph source. Cargo diagnostics do not expose a stable file dependency
graph, while rustdoc JSON requires a compilation-oriented/nightly workflow and omits some private
implementation detail users want to police.

## Consequences

### Positive

- stable Rust and ordinary Cargo projects are enough;
- analysis remains fast, cacheable, deterministic, and independent of a successful build;
- workspace and dependency-renaming behavior comes from Cargo rather than duplicate TOML logic;
- most Rust syntax is represented structurally, with no comment/string false positives;
- the rest of ArchUnit remains language-agnostic and purely testable.

### Negative

- macro-generated dependencies and generated `OUT_DIR` modules are not visible;
- conservative `cfg` unioning may report dependencies absent on the current target;
- name resolution is intentionally less complete than rustc for complex re-export, macro, and
  scope interactions;
- syntax added by a future Rust edition may require a `syn` update before it can be analyzed.

### Mitigations

- document the boundary prominently and expose extraction diagnostics;
- include fully qualified paths, not only imports;
- never turn an ambiguity into a guessed internal edge;
- maintain fixture workspaces for every supported resolution form;
- keep graph/report ordering deterministic so extractor changes are reviewable;
- consider a future opt-in semantic backend only after real false-negative reports justify its cost.
