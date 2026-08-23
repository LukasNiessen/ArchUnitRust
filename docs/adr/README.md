# Architecture decision records

Architecture decision records explain choices where an idiomatic Rust implementation cannot be a
mechanical port of a sibling. They are append-only: supersede a decision with a new ADR rather than
rewriting history after released behavior depends on it.

- [0001: Syntax and module-tree extraction](0001-syntax-and-module-tree-extraction.md)
- [0002: Deferred fluent selector errors](0002-defer-fluent-selector-errors-to-check.md)
- [0003: Aggregate closed violations above domains](0003-aggregate-closed-violations-above-domains.md)
- [0004: Store custom file predicates as thread-safe values](0004-store-custom-file-predicates-as-thread-safe-values.md)
- [0005: Guard selected subjects, not derived evidence](0005-guard-selected-subjects-not-derived-evidence.md)
- [0006: Format closed violations in one testing layer](0006-format-closed-violations-in-one-testing-layer.md)
