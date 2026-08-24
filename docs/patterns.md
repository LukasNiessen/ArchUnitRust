---
layout: default
title: Patterns and identifiers
nav_order: 3
description: The normalized Rust source identifiers, complete-match globs, regex escape hatches, and exclusions used by every feature family.
---

# Patterns and identifiers

A pattern only makes sense together with the identifier it is matched against. ArchUnitRust uses
the same matching substrate across files, dependency objects, layers, slices, metrics, and graph
queries.

## Identifiers

Internal source nodes use normalized paths relative to the Cargo workspace root, with `/` on every
operating system: `crates/api/src/handler.rs`. External dependency nodes use the Cargo-visible
crate name, including any rename in `Cargo.toml`: `serde_json`.

File selectors inspect one of four views:

| Selector | Candidate |
| --- | --- |
| `with_name("*.rs")` | final path segment, such as `handler.rs` |
| `in_folder("crates/api/**")` | containing path without the filename |
| `in_path("crates/api/**")` | complete normalized source path |
| `in_file("src/order[legacy].rs")` | one exact normalized path; metacharacters are literal |

`for_types_matching` inspects an unqualified Rust type name. External `matching` inspects a
Cargo-visible crate name.

## Globs

Plain string selectors are case-sensitive globs matched against the complete candidate:

| Syntax | Meaning |
| --- | --- |
| `*` | zero or more characters except `/` |
| `**` | zero or more characters including `/` |
| `?` | exactly one character except `/` |
| `[abc]`, `[a-z]` | one character from a class or range |
| `[!0-9]` | one character outside the class |

Backslashes in glob input are normalized as path separators. Because matching is anchored, use
`**` deliberately when a prefix or suffix is unconstrained.

```rust
use archunit::{Pattern, PatternSyntax};

let source = Pattern::glob("crates/**/src/*.rs").expect("valid documentation glob");
assert_eq!(source.syntax(), PatternSyntax::Glob);
assert!(source.matches("crates/api/src/handler.rs"));
assert!(!source.matches("crates/api/tests/handler.rs"));
```

## Exclusions

Wrap a selector string with `pattern` when that selector needs exceptions. `except` inherits the
parent selector target; the target-specific variants can exclude by another view of the same
candidate:

```rust
use archunit::{Checkable, pattern, project_files};

let rule = project_files()
    .in_path(
        pattern("src/**")
            .except("src/generated/**")
            .except_with_name("*_snapshot.rs"),
    )
    .should_not()
    .depend_on_files()
    .in_path(pattern("src/database/**").except("src/database/public.rs"));

let _: &dyn Checkable = &rule;
```

`except_all`, `except_in_path`, `except_in_folder`, `except_with_name`, and
`except_for_types_matching` complete this vocabulary. Exclusions on one selector use OR semantics:
matching any exclusion removes that candidate. They do not change the AND semantics between chained
subject selectors.

## When a glob is not enough

`Pattern::regex` and a `RegexFactory` configured with `PatternSyntax::Regex` are the low-level
escape hatch for complete-match Rust regular expressions. Fluent slice capture has the dedicated
`defined_by_regex` method; graph collapsing has `collapse_by_pattern` and
`collapse_by_pattern_with_replacement`.

```rust
use archunit::Pattern;

let module = Pattern::regex(r"src/(api|domain)/.*\.rs")
    .expect("valid documentation regex");
assert!(module.matches("src/api/handler.rs"));
```

An invalid pattern is retained by the fluent builder and becomes an `ArchUnitError::User` when the
terminal executes. A valid pattern that selects nothing is different: it produces the strict empty
selection violation described in [running a rule](running.md#when-a-rule-selects-nothing).

Next, use these selectors in [the files family](files.md).

