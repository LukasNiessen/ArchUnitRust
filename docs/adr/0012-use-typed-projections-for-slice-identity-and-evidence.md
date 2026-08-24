# ADR 0012: Use typed projections for slice identity and evidence

- Status: Accepted
- Date: 2026-08-24
- Issue: #30

## Context

The shared slices vocabulary relabels file dependencies as named architectural components and then
forbids selected component-to-component edges. TypeScript represents this relabeling as a bare edge
callback. A Rust terminal also needs to enumerate every selected slice before projecting edges so an
isolated or dependency-free selection is not mistaken for an empty test.

The public projection family includes pattern, regular-expression, file-suffix, and identity forms.
Rust already exports an `identity()` raw-edge mapper from the common projection kernel, so exporting a
second unrelated root function with the same name is impossible. The extraction graph also retains
canonical file self-edges specifically so node-oriented rules can see isolated files.

## Decision

`SliceProjection` is an immutable, cloneable value that owns one file-to-slice labeler. It exposes:

- `label_for(path)` for one normalized project identifier;
- `map_edge(edge)` for one raw dependency;
- `project(graph)` for deterministic cumulation through the shared projection kernel;
- `slice_labels(graph)` for all selected internal slices, including files represented only by their
  canonical self-edge.

`map_edge` drops raw self-edges and dependencies whose internal endpoints map to the same slice.
Both internal endpoints must match the projection. An external dependency needs only a selected
source; its Cargo-visible crate name remains the target label. This lets slice policies explicitly
address external modules without pretending they are project files.

The projection constructors have these contracts:

- `slice_by_pattern` requires exactly one `(**)` token. The token captures one non-empty path
  segment. Surrounding `*`, `**`, and `?` keep portable glob meanings, separators normalize, and the
  remaining file path may follow the declared pattern prefix;
- `slice_by_regex` accepts Rust `regex` syntax and uses its first capture. Invalid expressions and
  expressions without a capture are rejected;
- `slice_by_file_suffix` strips the final filename extension and applies the longest matching suffix,
  making overlapping suffix maps deterministic;
- `slice_identity` and `SliceProjection::identity` use the normalized file identifier as the slice
  name. The prefixed free-function spelling avoids collision with the existing raw-edge `identity`.

Low-level constructor failures return `SliceProjectionError` with the rejected input and stable
reason. The fluent `defined_by` and `defined_by_regex` methods retain this error and return it as
`ArchUnitError::User` at the terminal before Cargo project discovery. `with_projection` accepts a
prepared pattern, regex, suffix, or identity value.

The issue #30 fluent sentence is:

```text
project_slices[_in](...).defined_by(...).should_not().contain_dependency(from, to)
```

All builders consume and return owned values so scopes remain cloneable and branchable. The terminal
projects through the shared `project_edges` function and returns one `SliceDependencyViolation` for
the exact directed forbidden pair. That violation retains the aggregated projected dependency and
all concrete Rust `Edge` evidence. External targets follow the same rule.

Before projection, the terminal passes `slice_labels` through the universal empty-test guard. A
selected isolated slice is therefore non-empty even though it produces no projected dependency.
`CheckOptions::with_allow_empty_tests(true)` remains the explicit opt-out.

PlantUML parsing, diagram adherence, orphan/external diagram modifiers, and diagram generation are
kept in issue #31. Issue #30 adds only the violation rule kind needed by `contain_dependency`.

## Alternatives considered

### Store only an edge callback

A callback is sufficient to create projected edges but cannot enumerate isolated selected slices.
The terminal would either silently pass a stale selector or need to reverse-engineer selection from
edge output. A typed projection owns both operations over one labeling policy.

### Reuse the common raw-edge `identity()` as the slice identity

That mapper intentionally retains raw self-edges and has no `slice_labels` operation. Changing its
return type would break the public projection kernel. `SliceProjection::identity` states the domain,
while `slice_identity` remains an ergonomic root function.

### Drop all external dependencies during slice projection

The earliest TypeScript implementation did so, but mature ports allow rules and diagrams to reason
about external modules explicitly. Preserving only external target names keeps raw evidence and lets
later diagram modifiers choose whether to ignore them.

### Accept a compiled `Regex` as the only regex API

That would expose the dependency type in every fluent call and make deferred builder errors
impossible. Accepting an expression string matches the rest of the Rust API, compiles once into the
projection value, and reports configuration errors through the common classification model.

## Consequences

- Direct projection users and fluent terminals share exactly one mapping implementation.
- Empty-test safety includes isolated files without leaking self-edges into slice dependency rules.
- Forbidden dependencies remain directed, aggregated, deterministic, and backed by concrete Rust
  evidence.
- Suffix and pattern ambiguity is resolved at construction rather than during checking.
- The root API has an intentional `slice_identity` spelling alongside `SliceProjection::identity`.
- Issue #31 can reuse projection labels and projected evidence without changing issue #30 semantics.
