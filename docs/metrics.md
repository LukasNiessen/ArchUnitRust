---
layout: default
title: The metrics family
nav_order: 7
description: Measure Rust counts, cohesion, component distance, architecture zones, custom metrics, thresholds, and offline reports.
---

# The metrics family

Metrics expose measurements first and architecture verdicts second. The source model uses Rust
vocabulary—files, structs, enums, unions, traits, impl blocks, methods, fields, functions, and
macros—rather than translating class metrics literally.

## Scope

Start with `metrics()` or `metrics_in(path)`. `with_name`, `in_folder`, and `in_path` select source
files; `for_types_matching` narrows unqualified Rust type names. Chained selectors use AND
semantics and the shared [pattern exclusions](patterns.md#exclusions).

`analyze()` returns the full immutable `ProjectMetricsInfo`. Most callers choose a family and one
metric instead:

```rust,no_run
use archunit::{ArchUnitError, metrics};

fn inspect_instability() -> Result<(), ArchUnitError> {
    let measurements = metrics()
        .in_path("src/**")
        .distance()
        .instability()
        .measure()?;

    for measurement in measurements {
        println!("{}: {:.3}", measurement.identifier(), measurement.value());
    }
    Ok(())
}
```

Use `analyze_with` or `measure_with` when an explicit `CheckOptions` value should include test,
example, and benchmark targets.

## Counts

`count()` offers `method_count`, `field_count`, `lines_of_code`, `statements`, `imports`,
`concrete_types`, `functions`, `traits`, `impl_blocks`, `macros`, and `associated_functions`.
Method and field counts produce one measurement per selected type; the remaining counts produce one
per selected file.

```rust,no_run
use archunit::{ArchUnitError, metrics};

fn service_sizes() -> Result<(), ArchUnitError> {
    let methods = metrics()
        .for_types_matching("*Service")
        .count()
        .method_count()
        .measure()?;
    assert!(methods.iter().all(|value| value.value() >= 0.0));
    Ok(())
}
```

## Cohesion

`lcom()` measures eligible Rust structs from their inherent methods and `self.field` access
evidence. Trait declarations, enums, unions, trait impls, generated methods, and macro-expanded
accesses do not pretend to be class behavior.

The available formulas are `lcom96a`, `lcom96b`, `lcom1`, `lcom2`, `lcom3`, `lcom4`, `lcom5`, and
`lcom_star`. Each selection can be measured, constrained by a threshold, or passed to
`should_satisfy`.

## Component distance

In v0.1 one component is one analyzed Rust source file. `distance()` offers:

- `abstractness`: traits divided by traits plus concrete types;
- `instability`: distinct outgoing dependencies divided by total distinct coupling;
- `distance_from_main_sequence`: absolute distance from `A + I = 1`;
- `coupling_factor`: density of bidirectional internal coupling;
- `normalized_distance`: main-sequence distance discounted by component size.

File and type selectors choose reported components but do not shrink the coupling universe used by
distance calculations.

## Thresholds and predicates

Every selected count, LCOM, or distance metric has exactly five threshold verbs:
`should_be_below`, `should_be_above`, `should_be`, `should_be_below_or_equal`, and
`should_be_above_or_equal`.

```rust,no_run
use archunit::{MetricSubject, assert_passes, metrics};

let threshold = metrics()
    .for_types_matching("*Service")
    .count()
    .method_count()
    .should_be_below_or_equal(20.0);
assert_passes!(threshold);

let predicate = metrics().distance().instability().should_satisfy(
    |value, subject: &MetricSubject| subject.as_distance().is_some() && value <= 0.8,
);
assert_passes!(predicate);
```

Thresholds must be finite. `should_be` uses exact `f64` equality; use `should_satisfy` when a policy
needs an explicit tolerance or facts from the typed `MetricSubject`.

## Architecture zones

Two distance terminals are already architecture rules:

```rust,no_run
use archunit::{assert_passes, metrics};

let pain = metrics().in_path("src/**").distance().not_in_zone_of_pain();
let useless = metrics()
    .in_path("src/**")
    .distance()
    .not_in_zone_of_uselessness();

assert_passes!(pain);
assert_passes!(useless);
```

The zone boundaries are the public `PAIN_LIMIT` and `USELESSNESS_LIMIT` constants. Violations retain
the component, measured abstractness and instability, and the rejected `ArchitecturalZone`.

## A metric of your own

`custom_metric` stores a generic calculation over immutable `TypeInfo`, then either measures it or
turns it into a predicate or threshold rule:

```rust,no_run
use archunit::{TypeInfo, assert_passes, metrics};

let member_count = metrics().custom_metric(
    "member_count",
    "methods plus fields must remain manageable",
    |info: &TypeInfo| (info.methods().len() + info.fields().len()) as f64,
);
let rule = member_count.should_satisfy(|value, _info| value <= 20.0);

assert_passes!(rule);
```

The calculation and predicate run once per selected type on every execution. Non-finite custom
values are preserved for the predicate to interpret; user callback panics propagate with their Rust
backtrace.

## Export an offline report

Each built-in family exports every metric in that family as one self-contained HTML document:

```rust,no_run
use archunit::{ArchUnitError, MetricsExportOptions, metrics};

fn export_cohesion() -> Result<(), ArchUnitError> {
    let options = MetricsExportOptions::new()
        .with_title("Service cohesion")
        .with_timestamp(false);
    metrics()
        .for_types_matching("*Service")
        .lcom()
        .export_as_html_with("target/architecture/cohesion", &options)?;
    Ok(())
}
```

The exporter adds `.html`, creates parent directories, escapes content, and embeds all CSS. Disable
the UTC timestamp for byte-stable artifacts; custom CSS replaces the default stylesheet.

Next, visualize dependencies without making a verdict in
[dependency-graph reports](graph.md).

