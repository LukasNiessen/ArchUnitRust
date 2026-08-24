---
layout: default
title: The slices family
nav_order: 6
description: Capture architectural components from Rust paths, forbid dependencies between them, check PlantUML, and render the actual graph.
---

# The slices family

Slices project many source files into architectural component names. A projection can forbid one
directed relationship, compare the whole projected graph with PlantUML, or render the graph that
the source actually contains.

## Capture slices

`defined_by` accepts a portable glob with exactly one `(**)` capture. The captured path becomes the
slice label:

```rust
use archunit::project_slices;

let scope = project_slices().defined_by("src/(**)/");
assert_eq!(
    scope.projection().label_for("src/api/handler.rs").as_deref(),
    Some("api"),
);
```

`defined_by_regex` uses the first capture of a Rust regular expression. For reusable projections,
use `slice_by_pattern`, `slice_by_regex`, `slice_by_file_suffix`, or `slice_identity`, then pass the
result to `with_projection`.

Suffix projections remove `.rs` and choose the longest configured suffix:

```rust,no_run
use std::error::Error;
use archunit::{Checkable, project_slices, slice_by_file_suffix};

fn suffix_rule() -> Result<(), Box<dyn Error>> {
    let projection = slice_by_file_suffix([
        ("_controller", "controllers"),
        ("_service", "services"),
    ])?;
    let rule = project_slices()
        .with_projection(projection)
        .should_not()
        .contain_dependency("controllers", "services");
    let _: &dyn Checkable = &rule;
    Ok(())
}
```

## Forbid one directed pair

The negative mood checks one exact source-target pair:

```rust,no_run
use archunit::{assert_passes, project_slices};

let rule = project_slices()
    .defined_by("src/(**)/")
    .should_not()
    .contain_dependency("api", "database");

assert_passes!(rule);
```

Dependencies inside one slice and projected self-edges are omitted. External Cargo dependencies
keep the crate name as the target slice, so a rule can explicitly forbid `("api", "tokio")`.

## Check the whole graph against PlantUML

The supported component-diagram subset is line based: `component [Name]`, `[A] -> [B]`,
`[A] --> [B]`, apostrophe or `//` comments, and `@startuml` / `@enduml`. Other styling lines are
ignored.

```rust,no_run
use archunit::{assert_passes, project_slices};

let diagram = r#"
    @startuml
    component [api]
    component [application]
    [api] --> [application]
    @enduml
"#;
let rule = project_slices()
    .defined_by("src/(**)/")
    .should()
    .ignoring_external_slices()
    .ignoring_orphan_slices()
    .adhere_to_diagram(diagram);

assert_passes!(rule);
```

Strict adherence reports every actual projected dependency missing from the diagram.
`ignoring_external_slices` omits Cargo targets. `ignoring_orphan_slices` omits dependencies whose
source or target component is undeclared. `adhere_to_diagram_in_file(path)` reads UTF-8 lazily when
the rule executes.

## Draw the project as it is

The reverse path includes isolated selected slices and emits stable sorted PlantUML:

```rust,no_run
use archunit::{ArchUnitError, project_slices};

fn export_actual_architecture() -> Result<(), ArchUnitError> {
    let slices = project_slices().defined_by("src/(**)/");
    let text = slices.to_plantuml()?;
    slices.export_as_plantuml("target/architecture/actual.puml")?;
    assert!(text.starts_with("@startuml"));
    Ok(())
}
```

Use `to_plantuml_with` and `export_as_plantuml_with` for explicit `CheckOptions`. The lower-level
`PlantUmlParser`, `PlantUmlDiagram`, `PlantUmlDependency`, and `PlantUmlRenderer` operate entirely
on in-memory values.

Next, quantify the source in [the metrics family](metrics.md).
