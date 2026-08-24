# ADR 0021: Keep logging explicit and scoped to a check

- Status: Accepted
- Date: 2026-08-24
- Issue: #39

## Context

Architecture checks can be difficult to diagnose in CI when extraction, selection, and projection
happen before the final violation report. Issue #39 asks for opt-in logging with four levels, a
fixed event vocabulary, and optional timestamped file output. The sibling implementations also
show that useful progress and metric records belong near the domain operation rather than in the
final presentation layer.

Rust tests commonly run concurrently in one process. A process-global logger, environment switch,
or mutable singleton would let one test alter another test's output and would make file lifecycle
order-dependent. Conversely, adding logging arguments to every internal extraction and assertion
function would leak an observational concern through otherwise pure layers.

The library must also retain its result contract. An architecture disagreement is data in
`Ok(Vec<Violation>)`; a logging I/O failure means the requested execution could not be completed and
is therefore an `ArchUnitError`.

## Decision

Logging is represented by an immutable `LoggingOptions` value nested optionally in `CheckOptions`.
Its absence is the off switch. `check()` remains quiet and `check_with(&options)` is the only way a
built-in terminal receives logging configuration. No global logger, environment variable, or
ambient subscriber participates.

`CheckLogger` is a small public façade that borrows `Option<&LoggingOptions>`. With no configuration,
all methods are no-ops. It exposes the fixed specialized events `start check`, `end check`,
`log progress`, `log violation`, and `log metric`, plus ordinary `debug`, `info`, `warn`, and `error`
records. Newlines in record messages are escaped so every record remains one physical line.

Every built-in `Checkable` terminal executes through one internal lifecycle wrapper. It validates
the logging sinks before project discovery, writes a stable rule name at start and end, and logs the
closed `ViolationKind` for each disagreement. It does not render full violation prose because that
belongs to the testing/reporting boundary. Domain operations receive the borrowed logger only where
they have meaningful facts to add: extraction and selection counts are progress, and metric values
and thresholds are metric records. Metric callbacks are still evaluated exactly once.

The severity policy is fixed:

- progress and metric values are `Debug`;
- successful lifecycle events are `Info`;
- violations and an end event with violations are `Warn`;
- an execution error is logged as `Error` before the original error is returned.

Console output is enabled at `Info` by default once logging is opted into. Debug and info records go
to stdout; warning and error records go to stderr. File output is optional and creates missing
directories. Enabling it resolves one collision-resistant UTC-timestamped `.log` path immediately,
allowing callers to obtain `file_path()` before moving the options into a check and to archive the
artifact in CI.

`LogFileMode::Append` retains a pre-existing file. `Overwrite` truncates it exactly once before the
first record. Clones of one file-enabled `LoggingOptions` share an `Arc<Mutex<_>>` containing only
that path's initialization state and write critical section. They therefore produce complete lines
when explicitly shared by concurrent checks without creating global process state. Independently
constructed options stay independent.

An enabled configuration without either console or file output is a user error, regardless of its
minimum log level. Directory creation and write failures are technical errors. If the architecture
operation itself has already failed, a best-effort error record must not replace its original
`ArchUnitError`; on a successful operation, a requested logging failure becomes the check error.
Custom `Checkable` implementations may construct `CheckLogger` directly and call `validate()` to
adopt the same precedence.

## Alternatives considered

### Use the ecosystem `log` or `tracing` global subscriber

Those façades are valuable for applications, but their global dispatch and subscriber lifecycle do
not meet per-check isolation. They would also force consumers to install and coordinate a provider
to obtain the issue's simple console and artifact behavior.

### Read levels and paths from environment variables

Environment configuration is process-wide, hard to compose in parallel tests, and invisible at the
rule call site. Explicit values are more verbose but deterministic and reusable.

### Store a public trait-object sink

An injected sink would broaden the first release's ownership, equality, thread-safety, and error
semantics substantially. The public `LogRecord` and `CheckLogger` preserve room for a later explicit
extension after a concrete use case exists; v0.1 provides console and file sinks only.

### Swallow logging failures

Silent loss makes an explicitly requested CI artifact untrustworthy. Typed errors preserve the
existing distinction between user configuration mistakes and environmental failures.

## Consequences

- Default checks allocate no logger state and produce no logging output.
- Parallel tests cannot enable, disable, or redirect one another's logging through ambient state.
- All built-in rules share lifecycle and violation logging while domains retain meaningful progress
  and metric context.
- CI receives a discoverable timestamped artifact with deterministic line structure.
- Sharing a file is explicit by cloning one options value; unrelated configurations do not contend.
- Logging I/O is part of the requested check outcome and can fail that check.
- Custom rules can use the same public vocabulary, but automatic wrapping applies only to built-in
  terminals.
