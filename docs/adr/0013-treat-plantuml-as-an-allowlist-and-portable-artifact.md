# ADR 0013: Treat PlantUML as an allowlist and portable artifact

- Status: Accepted
- Date: 2026-08-24
- Issue: #31

## Context

Architects need to compare the dependency graph extracted from Rust with a small component diagram,
and they need the reverse operation to capture the current graph as a reviewable artifact. A full
PlantUML grammar includes aliases, packages, directions, labels, styling, macros, includes, and many
diagram families. Issue #31 requires only named components and directed dependencies.

The diagram boundary combines four failure classes that must remain distinct: invalid fluent input,
invalid recognized PlantUML, unreadable diagram files, and ordinary architecture disagreements. It
must also compose with issue #30's isolated-slice awareness and external Cargo targets.

## Decision

### Supported language

`PlantUmlParser` is a deterministic line-based parser for:

- case-insensitive `component [Name]` declarations;
- `[Source] -> [Target]` and `[Source] --> [Target]` dependencies;
- whole-line apostrophe and `//` comments plus apostrophe inline comments;
- `@startuml` and `@enduml` directives.

Unknown non-component lines, including styling directives, are ignored. A line that begins as a
supported component declaration or bracketed dependency but is malformed returns `PlantUmlError`
with its one-based line number. Empty text and unsafe component names are also rejected. Component
declarations preserve first-seen order; dependency endpoints become implicit components; duplicate
dependencies retain only their first occurrence.

`PlantUmlDiagram` owns components and directed `PlantUmlDependency` values. `allows(source, target)`
is exact and directional.

### Adherence semantics

A diagram is an allowlist for actual projected dependencies. The assertion walks the issue #30
`ProjectedEdge` values and emits a positive `SliceDependencyViolation` for every actual edge the
diagram does not allow. The violation uses the `AdhereToDiagram` rule kind and retains all raw Rust
edge evidence.

Diagram dependencies are permissions, not required runtime relationships. A drawn edge that is
currently absent does not fail the rule. This matches architecture-boundary enforcement: the diagram
constrains what may exist without demanding incidental coupling.

Strict mode includes external Cargo targets and endpoints absent from component declarations. Two
immutable options narrow that policy independently:

- `ignoring_external_slices` drops projected groups carrying external edge evidence;
- `ignoring_orphan_slices` drops an edge when either endpoint is absent from the diagram component
  set.

The fluent order is
`project_slices().defined_by(...).should().<modifiers>.adhere_to_diagram(...)`. The file form is
`adhere_to_diagram_in_file(path)`.

Inline text is stored without parsing and file paths are stored without reading. Only `check` parses
or performs diagram file I/O. Empty inline text and empty file paths are retained as configuration
errors and returned before Cargo project discovery. For a valid source, the terminal extracts the
project and applies the universal empty-test guard before reading or parsing the diagram. This keeps
a stale empty slice definition visible even if its diagram is also temporarily unavailable.

Malformed PlantUML is `ArchUnitError::User`; diagram file read failures are
`ArchUnitError::Technical`; disallowed dependencies are normal `Ok(Vec<Violation>)` verdicts.

### Reverse generation

`PlantUmlRenderer` generates:

1. `@startuml`;
2. sorted unique component declarations;
3. sorted unique directed dependencies;
4. `@enduml` and one final newline.

Explicit component input is combined with dependency endpoints so issue #30's isolated selected
slices remain present. Names containing `]` or line breaks are rejected rather than escaped into an
ambiguous syntax.

`SliceScopeBuilder::to_plantuml` extracts once and renders the current projection;
`export_as_plantuml` writes that string as UTF-8 and creates missing parent directories. `_with`
forms accept explicit `CheckOptions`. Low-level `PlantUmlRenderer` render/export operations remain
available over in-memory projected edges.

## Alternatives considered

### Embed a complete PlantUML parser

The additional grammar and dependency surface would not improve the required component allowlist.
A documented subset is easier to keep deterministic and portable across ArchUnit languages.

### Treat a diagram as exact graph equality

Requiring every drawn permission to have a current dependency rewards unnecessary coupling and makes
diagrams fragile during refactoring. The useful invariant is that actual dependencies stay inside the
allowed architecture.

### Parse and read sources while building the fluent sentence

That would make builders perform I/O, prevent reusable file-backed rules, and violate the library's
lazy terminal contract. It would also make construction order determine whether project or diagram
errors are visible.

### Omit isolated components during generation

Deriving components only from dependency endpoints makes an intentionally isolated slice disappear.
The typed `SliceProjection::slice_labels` collection exists specifically to retain that subject data.

### Escape arbitrary component names

PlantUML bracket names have several ambiguous edge cases. Rejecting `]` and line breaks produces a
clear user failure and stable output without inventing an incompatible quoting convention.

## Consequences

- Parsed and generated diagrams are deterministic, diffable text artifacts.
- Diagram rules reuse the exact same projection and evidence as forbidden slice dependencies.
- File-backed rules observe diagram edits between checks and remain lazy.
- External and undeclared endpoints are strict by default and opt-out independently.
- Empty slice definitions cannot silently pass because diagram processing follows the universal
  subject-selection guard.
- Rich PlantUML constructs outside the documented subset have no architectural meaning; unknown
  styling lines are tolerated, while malformed recognized lines fail clearly.
