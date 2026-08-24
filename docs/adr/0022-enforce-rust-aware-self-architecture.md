# ADR 0022: Enforce a Rust-aware self-architecture

- Status: Accepted
- Date: 2026-08-24
- Issue: #40

## Context

The mature sibling libraries run their own architecture APIs over their implementation. Their core
rules keep the shared kernel isolated, prevent peer domains from coupling, keep implementation code
away from the public facade, and reject cycles. These tests are valuable both as regression gates
and as executable examples of realistic library usage.

A direct textual port exposed two Rust-specific facts. First, most implementation files imported
re-exported names from `crate::{...}`, which made `src/lib.rs` an inward dependency hub and hid the
actual owning module from a layer rule. Second, Rust source graphs contain structural `mod` and
`pub use` edges. A parent facade owns a child through those edges while the child may refer back to
types re-exported by the private parent. Treating that pair as an executable dependency cycle would
reject ordinary Rust module organization.

ArchUnitRust also deliberately uses the closed aggregation seam from ADR 0003. `violation` imports
domain-owned violation data, while domain terminals meet `checkable` and the cross-domain
`Violation` result. Flattening every production file into one DAG would either report this explicit
join as a cycle or force the library to abandon its closed typed result contract.

## Decision

Every top-level internal module owns an internal facade. `common.rs`, `files.rs`, `graph.rs`,
`layers.rs`, `metrics.rs`, and `slices.rs` re-export their module-owned vocabulary. `lib.rs` imports
those facades and remains the outward-only public surface. Production and in-file test code import
through `crate::common`, its own domain facade, `checkable`, `violation`, or `testing`; it does not
import public names directly from the crate root.

`tests/architecture.rs` uses only the public `archunit` API and the real Cargo project. It enforces
four policies:

1. `src/common.rs` and `src/common/**` may depend internally only on that same scope. Their external
   allowlist is `std`, `core`, `alloc`, `cargo_metadata`, `proc_macro2`, `regex`, `syn`, and
   `thiserror`.
2. `files`, `graph`, `layers`, `metrics`, and `slices`, including each domain's top-level facade,
   are named layers. Every layer blocklists every peer.
3. Every `src/**` implementation file except `src/lib.rs` is forbidden from depending on
   `src/lib.rs`.
4. The top-level aggregation files and each architectural unit are independently cycle-free after
   structural `ImportKind::Mod` and `ImportKind::PubUse` evidence is excluded.

Scopes use `src/common**` and `src/{domain}**` so a Rust 2018 module facade such as `src/files.rs`
and its `src/files/**` descendants form one unit. The top-level `src/*.rs` scope covers facade and
aggregation files without recursively selecting domain children.

`CycleFreeFileCondition::excluding_dependency_kinds` is the public Rust-specific projection hook.
It consumes and returns the rule, sorts and deduplicates excluded kinds, and exposes them for
inspection. Filtering happens on every raw edge before cycle projection. If an edge contains both
`Mod` and `Use`, excluding `Mod` retains a new evidence edge containing `Use`; the executable cycle
therefore remains detectable. An edge disappears only when all its syntax kinds are excluded.

The aggregation seam is governed by direction rather than flattened away:

- `common` knows no domain;
- peer domains know `common` but not one another;
- `checkable` and `violation` join the closed cross-domain contract and contain no rule behavior;
- `testing` is above the domains and aggregation contract;
- `lib.rs` depends outward on everything and nothing depends inward on it.

## Alternatives considered

### Run one unfiltered whole-crate cycle rule

This reports hundreds of paths composed from module ownership, re-exports, and the deliberate
closed aggregation seam. The failures are technically present in a raw syntax graph but do not
describe unwanted executable architecture cycles.

### Ignore structural declarations with source directives

Adding `archunit: ignore` to every `mod` and `pub use` declaration would pollute production code,
erase useful evidence for other reports, and turn a project-level policy into many local bypasses.

### Drop any edge containing an excluded kind

A merged source-target edge can contain `Mod` and `Use` simultaneously. Dropping the whole edge
would create a false negative. Kind filtering must retain non-excluded evidence.

### Move every violation type into the shared kernel

That would make `common` depend on all domain concepts and reverse the intended dependency
direction. ADR 0003 already rejected this layout in favor of domain-owned data and a small explicit
closed aggregation module.

### Replace `Violation` with trait objects to make the graph a DAG

An open hierarchy would avoid the closed sum's aggregation edge but lose exhaustive matching,
typed accessors, and allocation-free value semantics. Changing a public data contract solely to
simplify a self-test is the wrong trade-off.

### Assert over a hand-filtered private graph in the test

A private low-level assertion would not dogfood the fluent API and would be a poor adoption example.
The dependency-kind option belongs on the reusable public cycle rule and is unit-tested separately.

## Consequences

- The sibling architecture intent is executable in Rust without pretending Rust lacks module
  ownership edges.
- `lib.rs` is a genuine outward-only facade, and layer evidence points to owning modules.
- Peer-domain coupling fails an ordinary public architecture test.
- Structural exclusions cannot hide parallel executable dependency evidence.
- The closed violation/check aggregation seam remains explicit and documented rather than being
  weakened to satisfy a raw whole-file DAG.
- New domains, analysis crates, top-level modules, or dependency syntax kinds require a deliberate
  update to the self-hosting policy.
