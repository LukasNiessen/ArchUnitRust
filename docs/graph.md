---
layout: default
title: Dependency-graph reports
nav_order: 8
description: Query one Rust dependency snapshot and render stable DOT, Mermaid, D2, CSV, JSON, or self-contained HTML.
---

# Dependency-graph reports

Graph reports show the architecture rather than declaring it valid. Every renderer consumes the
same immutable `GraphReportSnapshot`, so filtering, collapsing, counts, and Rust import evidence do
not drift between formats.

## Shape the query

Start with `project_graph()` or `project_graph_in(path)`. `dependency_graph()` and
`dependency_graph_in(path)` are aliases.

```rust,no_run
use archunit::{ArchUnitError, CheckOptions, project_graph};

fn snapshot() -> Result<(), ArchUnitError> {
    let graph = project_graph()
        .include_external_dependencies()
        .focus_on("src/api/**", 1)
        .reachable_from("src/application/**")
        .collapse_to_folder_depth(2)
        .titled("Application dependencies")
        .with_check_options(CheckOptions::new().with_clear_cache(true))
        .snapshot()?;

    println!("{} nodes", graph.summary.node_count);
    Ok(())
}
```

External dependencies and marker/collapsed self-edges are excluded by default. Enable them with
`include_external_dependencies` and `include_self_dependencies`.

## Select nodes

Three graph queries can be combined:

- `focus_on(pattern, depth)` selects matches and undirected neighbors up to the requested hops;
- `reachable_from(pattern)` selects matches and transitive outgoing dependencies;
- `dependents_of(pattern)` selects matches and transitive incoming dependents.

When several are present, their selected nodes form a union and the snapshot contains the induced
subgraph. A query selector that matches no node is a typed `GraphQueryError`, not an empty report.

## Collapse nodes

`collapse_to_folder_depth(depth)` groups file paths by a positive leading folder depth.
`collapse_by_pattern(expression)` uses the first regex capture as the label. The explicit
`collapse_by_pattern_with_replacement(expression, replacement)` form accepts Rust `regex`
replacement syntax such as `$1` and `${component}`.

Collapsed edges aggregate their raw edge count, external flag, and union of `ImportKind` evidence.
Self-edges created by collapse remain omitted unless explicitly enabled.

## Read the snapshot

`snapshot()` returns:

- a report `title`;
- stable sorted `GraphReportNode` values with renderer-safe IDs and labels;
- stable sorted `GraphReportEdge` values with source, target, count, external flag, and import kinds;
- `GraphReportSummary` counts for final nodes, final edges, selected raw edges, and external edges.

`summary()` is the shorter terminal when only those counts are needed. `raw_edge_count` counts
selected merged file-to-file edges before collapse; `edge_count` counts final aggregated edges.

## Render or export

There are six formats and two ergonomic terminal styles for each:

| Format | In-memory | UTF-8 file |
| --- | --- | --- |
| Graphviz DOT | `to_dot()` | `export_as_dot(path)` |
| Mermaid | `to_mermaid()` | `export_as_mermaid(path)` |
| D2 | `to_d2()` | `export_as_d2(path)` |
| CSV | `to_csv()` | `export_as_csv(path)` |
| JSON | `to_json()` | `export_as_json(path)` |
| HTML | `to_html()` | `export_as_html(path)` |

```rust,no_run
use archunit::{ArchUnitError, GraphRenderer, project_graph};

fn render_reports() -> Result<(), ArchUnitError> {
    let report = project_graph()
        .collapse_to_folder_depth(2)
        .titled("Application dependencies");

    let mermaid = report.to_mermaid()?;
    report.export_as_html("target/architecture/dependencies.html")?;

    let snapshot = report.snapshot()?;
    let dot = GraphRenderer::to_dot(&snapshot);
    let json = GraphRenderer::to_json(&snapshot);
    assert!(!mermaid.is_empty() && !dot.is_empty() && !json.is_empty());
    Ok(())
}
```

`render(GraphReportFormat)` and `export(GraphReportFormat, path)` provide typed dispatch. DOT,
Mermaid, and D2 preserve edge counts and distinguish external nodes. CSV contains one row per
aggregated edge. JSON contains the full snapshot. HTML is self-contained, offline-safe, and embeds
portable source views without scripts or remote assets.

Continue with [running a rule](running.md) for extraction options, caching, logging, and structured
failure handling.

