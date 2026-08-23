# ADR 0009: Model layers as a lazy file-graph policy

- Status: Accepted
- Date: 2026-08-24
- Issue: #27

## Context

Named layers are a convenience vocabulary over file dependencies. Without them, an N-layer policy
requires many pairwise file rules and repeats the same selectors. The shared API uses variadic
target arguments, allows a zero-argument allowlist to seal a layer, and commonly reports invalid
names while the sentence is being built.

Stable Rust has neither variadic methods nor method overloading. This crate also deliberately keeps
fluent construction infallible and lazy: builders retain invalid selectors until `check()` so a
sentence does not alternate between values and `Result` at every stage. Layers need to preserve that
contract while remaining recognizable to users of sibling ports.

## Decision

Layers project only internal, non-self file dependencies from the shared extracted graph. They do
not introduce another extractor or graph representation. A `LayerDependencyViolation` retains the
projected file edge and its raw Rust reference evidence alongside the source layer, target layer and
rejected policy kind.

The fluent surface uses:

```rust,ignore
project_layers()
    .layer("api").defined_by("src/api/**")
    .layer("database").defined_by_folder("src/database")
    .where_layer("api").may_only_depend_on_layers(&["database"])
    .where_layer("database").may_only_depend_on_layers(&[])
```

Target names are `&[&str]`. This makes both ordinary literal lists and the sealed-layer `&[]`
spelling unambiguous without allocation at the call site. `may_not_depend_on_layers` rejects an empty
slice because an empty blocklist cannot express a constraint.

Builders compile selectors immediately but retain the first configuration failure. `check()` turns
it into `ArchUnitError::User` before locating the Cargo project. This applies to invalid patterns,
blank names, undefined source or target layers and empty blocklists.

Policy state uses ordered maps and sets. One allowlist replaces an earlier allowlist for the same
source because it states the complete allowed set; repeated blocklists accumulate. Blocklists are
evaluated before allowlists, producing at most one violation per file edge.

Layer assignment and empty safety follow these rules:

- selectors added by repeating one layer name use OR semantics;
- when different definitions overlap, the first declared layer owns the file;
- intra-layer edges are always allowed;
- an edge with either endpoint outside every declared layer is ignored;
- every layer used as a policy source is subject to the universal empty-test guard.

## Alternatives considered

### Variadic-style macros

A macro could imitate zero or more arguments, but would split one fluent sentence between methods
and macros, obscure ownership, and make IDE discovery worse. A borrowed slice is ordinary Rust and
keeps the builder type explicit.

### `IntoIterator<Item = String>` targets

This is flexible for dynamically built policies but leaves an empty array's item type ambiguous and
pushes allocation onto the common literal case. A future separately named dynamic-input helper can
be added without changing the core sentence if real usage justifies it.

### Return `Result` from definition and policy stages

Immediate errors are conventional for standalone parsing functions, but make long fluent sentences
awkward and diverge from every existing rule builder in this crate. Deferred typed errors preserve
the established check boundary without panicking.

### Project files to layer labels before assertion

That makes the high-level graph compact but risks losing the exact file pair behind a violation or
requiring a parallel evidence structure. Evaluating layer membership over file-projected edges keeps
the shared projection kernel and concrete evidence intact.

### Reject overlapping definitions

Overlap can be useful while incrementally defining legacy exceptions. Declaration-order precedence
is deterministic and matches the mature sibling implementation. It is documented because changing
definition order can change ownership.

## Consequences

- The layer API is a thin policy skin over the existing extraction and projection pipeline.
- Literal policies, including sealed layers, have concise and type-inferable Rust syntax.
- Invalid fluent input never panics and never triggers project I/O before becoming a user error.
- Deterministic collections and first-match assignment keep violation order reproducible.
- Unassigned files are deliberately outside the policy; users who require complete assignment need
  a future explicit coverage rule rather than an implicit behavior change.
- Dynamic target lists require converting their borrowed names into a slice before calling the
  current method.
