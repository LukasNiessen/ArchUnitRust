# ADR 0019: Export deterministic offline metrics reports

- Status: Accepted
- Date: 2026-08-24
- Issue: #37

## Context

The mature ports can turn a metrics data map into a standalone HTML report and can export the
complete count, LCOM, and distance families from their fluent builders. Rust needs the same outcome
without coupling measurement types to one renderer or performing a separate project extraction for
each metric.

Report artifacts are commonly committed, archived by CI, or opened without a server. Their ordering
and optional metadata therefore affect reproducibility. User-controlled titles, metric labels,
values, and CSS also cross an HTML boundary and must not be able to create executable markup.

Rust has no optional arguments. Mixing an output path into presentation options would make rendering
an in-memory report awkward, while deriving identifiers only from a type name would collide when
different modules declare the same name.

## Decision

`MetricsReportData` is a `BTreeMap<String, String>`. This deliberately small, deterministic boundary
lets callers export arbitrary project metrics without constructing internal measurement types.
`MetricsExporter` can render that map in memory or write it to disk. `MetricsExportOptions` owns the
title, timestamp flag, and optional replacement CSS through consuming modifiers. The output path is
a separate argument to write operations.

The document is self-contained HTML with no scripts, remote assets, or network access. Titles, metric
names, and values are HTML-escaped. Replacement CSS is trusted as CSS but every case-insensitive
`</style` terminator is neutralized so it cannot break into document markup. Empty maps render an
explicit empty state.

Timestamps are enabled by default, formatted as UTC ISO 8601, and can be disabled for reproducible
artifacts. The formatter uses `SystemTime` and a local civil-date conversion rather than adding a
runtime dependency for one display value.

Write operations append `.html` case-insensitively when it is absent, create missing parent
directories, and return the resolved `PathBuf`. Invalid presentation options and paths are user
errors; directory and file-write failures are technical errors.

The consuming `count().export_as_html`, `lcom().export_as_html`, and
`distance().export_as_html` terminals export every metric in that family. Each validates fluent
configuration first, validates report configuration second, and performs project discovery only
after both succeed. A family analyzes or extracts the selected project once and calculates all of
its measurements from that snapshot. Type report identifiers use `source/path.rs:TypeName`; file and
distance identifiers use the normalized source path.

## Alternatives considered

### Serialize internal measurement structs directly

That would expose the closed subject representation as a report schema and make arbitrary
project-specific data hard to export. A display map keeps the rendering boundary stable and small.

### Put the output path inside the options object

An options object with an optional path conflates presentation with storage and gives in-memory
rendering irrelevant state. Rust methods can express the path explicitly while borrowing reusable
presentation options.

### Extract once per metric

This repeats parsing and graph construction and risks observing different project states within one
report. One snapshot per family is both faster and internally consistent.

### Add a date-time dependency

The report needs only a current UTC display string. A dependency would add compile time and supply-
chain surface without improving the public contract.

## Consequences

- Raw data and family reports have deterministic row order.
- Timestamp-free reports can be stable build artifacts.
- Family exports remain consistent because all values share one extraction snapshot.
- Reports are portable and do not execute scripts or load remote resources.
- Custom CSS can style the entire report but cannot terminate its style element.
- Type labels remain unambiguous across modules at the cost of being more verbose.
