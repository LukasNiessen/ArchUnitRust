# ADR 0011: Render six formats from one offline-safe boundary

- Status: Accepted
- Date: 2026-08-24
- Issue: #29

## Context

ADR 0010 defines a renderer-neutral graph snapshot so query semantics cannot drift between output
formats. The first rendering release must serve diagram tools, automation, data analysis, and a
human-readable artifact through DOT, Mermaid, D2, CSV, JSON, and HTML.

Rust adds three boundary concerns. Each text syntax has different escaping rules, JSON must remain
standards-compliant for every Unicode string, and file I/O must preserve the library's distinction
between invalid API input and an environmental failure. An HTML report advertised as
self-contained must also work without JavaScript, a CDN, or network access.

## Decision

Every renderer is a pure function from `&GraphReportSnapshot` to `String`. `GraphRenderer` owns
typed dispatch through `GraphReportFormat` and exposes one named `to_*` function for each format.
It never discovers a project, queries a graph, or changes the snapshot.

The formats carry the same underlying evidence in forms appropriate to their consumers:

- DOT, Mermaid, and D2 use stable snapshot node identities, retain aggregate counts, and distinguish
  external dependency edges;
- CSV emits one RFC-style escaped row per aggregated edge, including count, external state, and
  import kinds;
- JSON serializes the complete title, nodes, edges, import kinds, and summary through `serde_json`;
- HTML renders escaped summary cards, nodes, dependencies, and embedded portable Mermaid, DOT, D2,
  and JSON source views.

Escaping is format-specific and centralized below the renderers. Quoted diagram syntaxes escape
control characters, slashes, and quotes. CSV doubles quotes and quotes fields only when required.
HTML escapes all user-controlled text. Mermaid labels escape markup before representing line breaks.
Deterministic snapshot ordering is preserved in every output.

HTML contains embedded CSS only. It deliberately does not render a live client-side diagram because
that would require a bundled runtime or network dependency. The report remains usable offline and
exposes portable source that dedicated diagram tools can render.

`ProjectGraphBuilder` exposes `to_dot`, `to_mermaid`, `to_d2`, `to_csv`, `to_json`, and `to_html`.
Each terminal extracts one snapshot and renders it. Callers producing several formats should invoke
`snapshot()` once and reuse the pure `GraphRenderer` functions.

Each named string form has a matching `export_as_*` terminal. `GraphRenderer::export` and the fluent
builder also support typed `GraphReportFormat` dispatch. The shared export boundary:

- rejects an empty path as `ArchUnitError::User` before filesystem access;
- creates missing parent directories;
- writes the complete string as UTF-8;
- wraps directory and write failures as `ArchUnitError::Technical` with the failing path retained in
  the stable context.

## Alternatives considered

### Let output extensions select the renderer

Extension inference makes a mistyped or extensionless path ambiguous and couples file I/O to format
policy. Named methods and `GraphReportFormat` make the choice explicit and support arbitrary paths.

### Hand-build JSON

Manual quoting appears small but is easy to get wrong for Unicode, control characters, and future
fields. A direct `serde_json` dependency provides a standards-tested boundary and keeps the JSON
schema reviewable.

### Use a template engine for HTML

One fixed document does not justify another runtime abstraction. Rust formatting plus one shared
HTML escape function keeps the dependency surface small while tests cover hostile labels and offline
constraints.

### Load Mermaid or another renderer from a CDN

That produces a richer initial visual but contradicts a self-contained report, fails in restricted CI
environments, and adds executable third-party content. Portable source sections preserve tool
interoperability without those costs.

### Accept a generic `Write` sink as the only export API

A writer abstraction is useful for advanced integrations but does not fulfill the ergonomic path
form shared across sibling libraries. The path terminal is the stable public workflow; callers can
already obtain a string and send it to any writer.

## Consequences

- All six formats are guaranteed to reflect one completed snapshot contract and deterministic order.
- Rendering can be unit-tested with in-memory snapshots, independently of Cargo extraction and disk
  access.
- Export errors remain classifiable without parsing messages.
- Unicode content is retained through every string and file boundary.
- The HTML artifact opens offline and has no script or remote-content security surface.
- Producing several fluent formats repeats extraction unless the caller explicitly reuses a snapshot;
  the API documents the reuse path rather than hiding mutable renderer state.
