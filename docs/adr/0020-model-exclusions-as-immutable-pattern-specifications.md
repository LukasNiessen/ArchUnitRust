# ADR 0020: Model exclusions as immutable pattern specifications

- Status: Accepted
- Date: 2026-08-24
- Issue: #38

## Context

A selector often needs “everything matching this pattern except these generated or public files.”
Encoding that as a separate inverted architecture rule changes the selected population and can hide
an empty-test mistake. ArchUnitTS addresses this with an optional `except` field on every pattern
selector. Issue #38 identifies that behavior as the most valuable role-model feature to preserve.

Rust has neither optional arguments nor method overloading. Adding a second required argument to
every selector would make the common no-exclusion case noisy and break the fluent sentence. Adding
`except` directly to each returned builder would require mutable “last selector” state and becomes
ambiguous after graph query unions, repeated layer definitions, or a terminal predicate.

The matching kernel already binds a compiled pattern to `PatternTarget`. Exclusions need to retain
that invariant, preserve first-error ordering, and work for files, dependency objects, layers, graph
queries, metrics, and capture-based slices without duplicating matching branches.

## Decision

`PatternSpec` is an immutable owned input value containing one parent pattern and an ordered list of
`PatternExclusion` values. Plain `&str` and `String` inputs convert to an exclusion-free
specification, preserving ordinary selector calls. The public `pattern` helper enables this form:

```text
pattern("src/**").except_in_folder("src/generated/**")
```

`except` adds one exclusion that inherits the parent selector target; `except_all` adds a list.
Target-explicit methods are `except_in_path`, `except_in_folder`, `except_with_name`, and
`except_for_types_matching`, using Rust type vocabulary rather than the role model's class spelling.
Modifiers consume and return `PatternSpec`, so a base specification can be cloned and branched.

`RegexFactory` compiles the parent first and exclusions in declaration order with the same syntax
and case options. It binds each plain exclusion to the parent target and each targeted exclusion to
its explicit target. `Filter` owns the compiled exclusion filters. A candidate matches only when the
parent result (including a possible `not_matching` mood) is true and no exclusion filter matches the
complete identifier. Invalid exclusions therefore enter the same deferred configuration-error path
as invalid parent patterns and precede project discovery.

Factory-backed selectors automatically share this behavior across:

- file scopes, exact-file scopes, and filename/folder/path predicates;
- internal dependency objects and external module objects;
- layer definitions;
- graph focus, reachable-from, and dependents-of queries;
- metric filename, folder, path, and Rust type selections.

Slice pattern and regex projections compile captures outside `RegexFactory`, so they adapt the same
`PatternSpec` separately. A glob capture compiles exclusions as globs; a regex capture compiles them
as regex. Exclusions run on the normalized file path before slice labeling. Excluded files therefore
cannot produce slice names, projected edges, or isolated components.

Multiple exclusions on one parent use OR semantics: any match excludes the candidate. Multiple
parent selectors keep their existing AND semantics. Exclusions affect only their own parent
selector and never mutate extraction or the shared graph cache.

## Alternatives considered

### Add a second options argument to every selector

Rust cannot make that argument optional. Requiring `PatternOptions::default()` on the overwhelmingly
common path would make the fluent API cumbersome and would still need separate names for methods
whose existing second argument has another meaning, such as graph focus depth.

### Add `except` to every fluent builder stage

That API must remember which selector was added most recently. The state is ambiguous when a builder
contains several independent query slots, and terminal pattern predicates have no following builder
stage. Attaching exclusions to the pattern input makes ownership explicit at the call site.

### Store exclusion closures

Closures would erase pattern source and target data needed by diagnostics, make equality and caching
harder to reason about, and bypass the shared normalization and syntax policy.

### Remove excluded source files during extraction

Per-selector exclusions are query semantics, not source-discovery policy. Changing extraction would
make one rule's scope affect other rules and fragment graph-cache reuse.

## Consequences

- Existing string-literal selector calls stay concise.
- Every pattern selector can carry multiple plain or target-explicit exclusions.
- Diagnostics retain the rejected exclusion source and sentence-order precedence.
- One matching implementation governs all factory-backed families.
- Slice exclusions cannot leak through edge or isolated-component projection.
- Exclusions do not alter source enumeration, project topology, or cache keys.
