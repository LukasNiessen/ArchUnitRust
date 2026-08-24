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
- [0007: Use a thin assertion macro over a pure evaluator](0007-use-a-thin-assertion-macro-over-a-pure-evaluator.md)
- [0008: Use the built-in harness without an adapter](0008-use-the-built-in-harness-without-an-adapter.md)
- [0009: Model layers as a lazy file-graph policy](0009-model-layers-as-a-lazy-file-graph-policy.md)
- [0010: Query once and render one graph snapshot](0010-query-once-and-render-one-graph-snapshot.md)
- [0011: Render six formats from one offline-safe boundary](0011-render-six-formats-from-one-offline-safe-boundary.md)
- [0012: Use typed projections for slice identity and evidence](0012-use-typed-projections-for-slice-identity-and-evidence.md)
