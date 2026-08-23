# ADR 0010: Query once and render one graph snapshot

- Status: Accepted
- Date: 2026-08-24
- Issue: #28

## Context

Graph reports support several output formats and several orthogonal query modifiers. If each
renderer filters, traverses, collapses or counts the extracted graph independently, formats drift:
a new query option must be implemented repeatedly and two reports can disagree about the same
project.

The shared direction therefore separates graph reporting into two steps: build one snapshot, then
render it. Rust additionally needs fallible project extraction and a non-panicking way to represent
invalid regular expressions or collapse results.

## Decision

`GraphSnapshotFactory::create` and `create_graph_snapshot` are pure functions over an extracted
`Graph` and immutable `GraphQueryOptions`. They return one `GraphReportSnapshot` containing:

- deterministically sorted nodes with stable `n0`, `n1`, ... IDs;
- deterministically sorted aggregated edges with count, external classification and a union of
  `ImportKind` values;
- a custom or default title;
- final node and edge counts plus selected raw and external edge counts.

The query pipeline has one fixed order:

1. exclude external edges unless explicitly included;
2. select nodes using focus, outgoing reachability and incoming dependents;
3. retain the induced edges whose endpoints are both selected, excluding extracted self edges by
   default;
4. collapse node labels;
5. aggregate equal collapsed endpoint pairs, unioning import kinds and external classification;
6. exclude collapse-produced self edges unless self dependencies were requested;
7. build sorted nodes and summary counts.

Self edges participate in node selection even when they are not displayed. This preserves isolated
source files in complete and exact-focus snapshots. Focus expands through both incoming and outgoing
neighbors to a finite depth. `reachable_from` walks outgoing edges transitively;
`dependents_of` walks incoming edges transitively. Multiple query modifiers contribute a union of
selected nodes before the induced subgraph is built.

`Graph` already merges a source-target file pair and unions its Rust syntax kinds. Consequently,
snapshot `raw_edge_count` is the number of selected merged endpoint pairs, not the number of source
syntax occurrences. Aggregated `count` is the number of those selected pairs represented after
collapse.

`ProjectGraphBuilder` owns a `ProjectLocator`, `GraphQueryOptions`, and `CheckOptions`. Every modifier
consumes and returns the builder. `snapshot()` and `summary()` are the only issue #28 terminals that
perform extraction. Output rendering and file export remain issue #29.

Rust has no optional function arguments, so automatic and explicit entry points are separate:

- `project_graph()` and `dependency_graph()`;
- `project_graph_in(path)` and `dependency_graph_in(path)`.

`focus_on` takes its depth explicitly. `collapse_by_pattern` uses the first capture with Rust
`regex` replacement spelling `$1`; `collapse_by_pattern_with_replacement` exposes custom capture
replacement.

Invalid query construction is retained on the fluent builder and becomes `ArchUnitError::User`
before project discovery. The pure factory returns `GraphQueryError`. This covers invalid selectors,
zero folder depth, invalid collapse expressions, empty replacements or titles, and a capture
replacement that produces an empty node label.

## Alternatives considered

### Let each renderer query the raw graph

This makes the first renderer superficially smaller but multiplies behavior and testing across six
formats. It also makes a renderer change capable of changing graph semantics. Renderers should be
pure serialization of one already-decided snapshot.

### Collapse before focus and traversal

Queries are written against stable project-relative file identifiers. Collapsing first would change
what patterns mean and make traversal lose file-level distinctions. Selection therefore precedes
collapse.

### Intersect multiple query modifiers

An intersection makes independent views surprisingly erase one another—for example, an orphan
focus plus a reachable dependency view would become empty. Mature siblings combine them as a union,
which also supports assembling a report from several architectural areas.

### Count source syntax occurrences

The extraction graph intentionally merges parallel endpoint pairs and retains syntax kinds as a
set. Reconstructing occurrence counts downstream is impossible and would contradict the graph
contract. Snapshot counts describe graph relationships before and after collapse.

### Panic on invalid capture output

A valid regular expression can still replace a matching node with an empty string, especially when
it references a missing capture. Returning `GraphQueryError::EmptyCollapsedNode` preserves the
library-wide no-panic rule and gives callers the original node label.

## Consequences

- Every output format will receive identical nodes, edges, counts, title, and ordering.
- A new query or collapse behavior is implemented and tested once.
- Isolated files remain visible without exposing marker self edges by default.
- Query results are deterministic across machines because identifiers, sets, maps, and final IDs
  use stable ordering.
- Snapshot values own their data and can be reused by several renderers without re-extraction.
- A custom pattern collapse can fail at snapshot time if its replacement erases a node; this is a
  classified user error rather than an assertion violation or panic.
