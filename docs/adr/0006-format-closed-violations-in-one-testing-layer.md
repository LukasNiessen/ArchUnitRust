# ADR 0006: Format closed violations in one testing layer

- **Status:** accepted
- **Date:** 2026-08-23
- **Decision owners:** ArchUnitRust maintainers
- **Tracking:** [#24](https://github.com/LukasNiessen/ArchUnitRust/issues/24)

## Context

Architecture assertions deliberately return structured data. Putting human prose on each
violation would couple rule logic to terminal style, duplicate numbering and color decisions, and
make every future test integration produce subtly different output.

Sibling implementations use a `ViolationFactory` followed by a `ResultFactory`. Rust differs in
two useful ways: the built-in `Violation` abstraction is a closed enum, and a result message can be
an owned `String` rather than a callback required by JavaScript matcher APIs. Rust also needs
deterministic color control for snapshot tests, redirected output and CI.

## Decision

Create one top-level `testing` module above the rule domains.

`ViolationFactory::from_violation` exhaustively matches the closed `Violation` enum and converts
each variant into a plain `TestViolation { message, details }`. It owns all architecture-violation
wording, including Rust import evidence, but applies no numbering or ANSI styling. Adding a new
enum variant therefore makes this match incomplete until its formatter is added.

`ResultFactory` accepts the original violation slice, maps every value through `ViolationFactory`,
and returns an owned `TestResult { passed, message }`. It owns summary grammar, singular/plural
wording, stable numbering, indentation and color. `TestResultOptions` carries two independent
choices:

- `expected_to_pass` compares the observed empty/non-empty verdict with a normal or inverted
  expectation;
- `ColorChoice` is `Auto`, `Always` or `Never`.

`Auto` emits ANSI codes only for a terminal and disables them when `NO_COLOR` is present, `TERM` is
`dumb`, or `CI=true`. `Always` and `Never` are deterministic overrides. `ColorUtils` is public so
future built-in test integration and downstream adapters can use the same styling primitives.

The four expectation outcomes are explicit:

- no violations and expected pass: green success;
- violations and expected pass: red numbered failure;
- violations and expected failure: green summary with the same numbered evidence;
- no violations and expected failure: red explanation that expected violations were absent.

## Alternatives considered

### Implement `Display` on each violation type

Rejected. It moves prose into `common` and rule domains, violates the data-first boundary and gives
adapters no single place to control numbering, verbosity or color.

### Use an open formatter registry with an unknown fallback

Rejected. The crate already chose a closed violation sum. An exhaustive match turns a missing
formatter into a compile-time maintenance task instead of shipping an “unknown violation” message.

### Apply color inside `ViolationFactory`

Rejected. A formatted violation should be reusable in plain logs, snapshots and colored terminal
results. Result-level color keeps one policy decision for the complete message.

### Enable or disable color only through environment detection

Rejected. CI snapshots, IDE integrations and downstream tools need deterministic output
independent of the current process terminal. Explicit `Always` and `Never` choices provide that
control while `Auto` remains the ergonomic default.

### Store the result message as a closure

Rejected. Rust callers do not need JavaScript matcher's lazy callback shape. An owned string is
cloneable, inspectable, framework-neutral and cannot capture hidden state.

### Let each test adapter format its own result

Rejected. Adapters should translate one shared result into a host assertion only. Formatting in an
adapter would make message quality depend on how the user runs the same rule.

## Consequences

### Positive

- every current violation family has one evidence-aware human rendering;
- numbering, grammar and ANSI behavior are identical for every consumer;
- a new `Violation` enum variant cannot silently miss formatter coverage;
- plain and colored output are deterministic in unit and integration tests;
- normal and inverted expectations share one unambiguous truth table;
- future assert helpers remain thin consumers of `Checkable` and `ResultFactory`.

### Negative

- `testing` intentionally depends on the closed violation sum and therefore all current domains;
- adding a violation variant requires a corresponding factory branch and message tests;
- `Auto` color detection consults process environment and standard-output terminal state;
- detailed raw dependency evidence can make messages long for heavily aggregated edges.

### Mitigations

- keep `testing` at the top of the dependency graph and forbid rule modules from importing it;
- test every variant with exact plain prose and test ANSI wrappers separately;
- let callers select `ColorChoice::Never` for snapshots and machine-readable logs;
- retain structured violations beside formatted results so richer reporters can choose their own
  presentation without parsing prose.
