# ADR 0016: Use source files as distance components

- Status: Accepted
- Date: 2026-08-24
- Issue: #34

## Context

Robert C. Martin's abstractness, instability, and main-sequence metrics assume a component boundary.
The boundary is commonly a package or assembly, but neither concept maps uniquely to Rust: a Cargo
package can expose several targets, a crate can span many modules, and inline and outlined modules
do not have a one-to-one relationship with files. ArchUnitRust's dependency extractor currently has
one lossless, evidence-carrying boundary: the normalized Rust source-file node.

Calculating coupling after applying a fluent selector would make results depend on the query rather
than the project. Selecting only one file could erase all incoming and outgoing edges, turning an
unstable component into an apparently isolated one. External crates and marker self-edges also
cannot be treated as component coupling without changing the denominator.

The sibling implementations agree on the abstractness, instability, distance, normalized-distance,
and zone formulas. The mature Ruby implementation defines coupling factor over both possible edge
directions; this keeps the result in `[0, 1]`. The older TypeScript expression does not use that
denominator consistently.

## Decision

The v0.1 distance component is one analyzed Rust source file. `DistanceInfo` combines its immutable
`FileMetricsInfo` with coupling derived from the dependency graph. Coupling counts distinct
internal, non-self source and target files. Parallel Rust references are already merged by the
graph, external crate edges are ignored, and endpoints outside the metrics snapshot cannot enter
the population.

The coupling universe is the complete project snapshot selected by `CheckOptions`, including dev
targets only when explicitly requested. Fluent file and type selectors choose which file components
are returned or checked but never alter their coupling evidence. A type selector retains a file
when at least one declaration matches; the distance calculations still use the complete file's type
population.

For one component, let:

- `T` be trait declarations;
- `C` be struct, enum, and union declarations;
- `Ca` be distinct incoming component coupling;
- `Ce` be distinct outgoing component coupling;
- `N` be the number of project components;
- `L` be physical non-comment lines of source.

The formulas are:

| Metric | Formula |
|---|---|
| Abstractness | `A = T / (T + C)` |
| Instability | `I = Ce / (Ca + Ce)` |
| Distance from main sequence | `D = abs(A + I - 1)` |
| Coupling factor | `(Ca + Ce) / (2 * (N - 1))` |
| Normalized distance | `D * (1 - min(L / 100, 1) * 0.5)` |

A zero denominator produces `0.0` for abstractness, instability, and coupling factor. Consequently,
a component with no types and no coupling has main-sequence distance `1.0`; this is a defined
measurement, not absence of data. The size discount reaches at most one half and does not hide a
large component's distance completely.

The zone of pain is strictly `A < 0.3 && I < 0.3`. The zone of uselessness is strictly
`A > 0.7 && I > 0.7`; points on either boundary are not inside the zone. Zone terminals implement
the shared `Checkable` contract, preserve typed `MetricZoneViolation` evidence, and apply the
universal strict empty-selection guard.

## Alternatives considered

### Treat each Cargo package as a component

This would collapse most single-package projects to one point and lose the internal design signal
users expect from an architecture test. Workspace packages remain a useful future projection, but
they should aggregate known file evidence rather than replace it.

### Treat inferred Rust modules as components

Outlined modules can be mapped to files, but inline modules share a file and `#[path]` can redirect
their source. Splitting syntax counts and dependency evidence across those cases would require a
module-span projection that the extractor does not yet provide. A later module-level metric can be
added as an explicit projection without changing file-level semantics.

### Recompute coupling inside the fluent selection

This makes a metric value query-dependent and rewards narrow selectors by deleting topology. The
selection must be a reporting boundary, not a new architectural universe.

### Include external crate dependencies

External names are not members of the analyzed file population, so including their edges while
keeping `N` file-based can produce invalid coupling factors. External dependency policy already has
its own typed rule family.

## Consequences

- Every numeric result retains the syntax and topology evidence that produced it.
- Distance results remain stable when the same file is selected through a narrower query.
- File-level components work uniformly across single crates and Cargo workspaces.
- Package, crate, or inferred-module rollups require a future explicit projection and new tests.
- Threshold checks in issue #36 can operate on the same measurements without redefining formulas.
