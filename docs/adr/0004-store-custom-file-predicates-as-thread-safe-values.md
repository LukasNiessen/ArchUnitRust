# ADR 0004: Store custom file predicates as thread-safe values

- **Status:** accepted
- **Date:** 2026-08-23
- **Decision owners:** ArchUnitRust maintainers
- **Tracking:** [#22](https://github.com/LukasNiessen/ArchUnitRust/issues/22)

## Context

The Files API needs an escape hatch for requirements that do not belong in the built-in fluent
vocabulary. Sibling implementations accept a function and message through `adhere to(fn,
message)`. In Rust, the rule sentence is lazy and its terminal is a concrete, cloneable value that
implements the object-safe `Checkable` contract. A closure may capture state, has an anonymous type
and is not necessarily a function pointer.

The predicate also needs stable file facts. Dependency extraction uses normalized,
workspace-relative identifiers, while source text must still be read from the host filesystem.
Exposing an absolute host path would make identical checks produce machine-specific data and would
confuse selector identity with I/O location.

## Decision

`adhere_to(predicate, message)` accepts a closure implementing
`Fn(&FileInfo) -> bool + Send + Sync + 'static`. The terminal erases its anonymous type behind an
`Arc`, allowing the completed rule to remain one concrete, cheaply cloneable `CustomFileCondition`.
The public `FilePredicate` alias documents this stored contract.

`FileInfo` contains:

- the normalized workspace-relative path;
- the filename without its final extension;
- the final extension including its leading dot;
- the normalized containing directory, or `.` at the workspace root;
- the complete UTF-8 source text exactly as read from disk;
- the number of non-blank source lines.

The graph identifier is used for all public path-derived fields. The absolute path is constructed
only inside extraction by joining that identifier to Cargo's workspace root and is not exposed.
Source text is read lazily after project discovery and subject selection. The predicate is invoked
exactly once per selected file. Positive rules report a violation when it returns `false`; negated
rules report one when it returns `true`.

The message is stored without eager validation so the fluent sentence remains infallible. A blank
message becomes a `UserError` during `check`, after an earlier malformed subject selector but before
project discovery.

## Alternatives considered

### Make the terminal generic over the closure type

Rejected for the public terminal. It would expose a different anonymous terminal type for every
closure, complicate explicit type annotations and make heterogeneous collections less ergonomic.
The one concrete terminal is consistent with the other rule families.

### Accept only a function pointer

Rejected. Function pointers cannot carry captured configuration, which is a common reason to use a
custom predicate.

### Store the closure in `Rc`

Rejected. `Rc` would prevent a completed rule from crossing thread boundaries. Architecture test
runners can execute tests concurrently, so the stored callable requires `Send + Sync` and uses
`Arc`.

### Read source files while building the sentence

Rejected. Eager I/O would break the library-wide lazy `Checkable` contract, perform work for rules
that are never checked and report project errors before terminal validation.

### Expose the absolute source path

Rejected. It is machine-specific, leaks workspace location details and disagrees with the
normalized identifiers used by selectors and violations.

## Consequences

### Positive

- custom rules retain the same lazy, cloneable and object-safe terminal shape as built-in rules;
- predicate results and violations are deterministic over normalized file identity;
- callers receive complete immutable source facts without learning extraction internals;
- one predicate call per file makes stateful instrumentation predictable;
- completed rules can be shared safely by concurrent test runners.

### Negative

- captured values must be owned for `'static` and thread-safe for `Send + Sync`;
- reading full UTF-8 source text adds I/O and allocation for selected files;
- non-UTF-8 source files produce a technical read error instead of reaching the predicate;
- `Arc` adds one allocation and atomic reference counting per completed custom rule.

### Mitigations

- encourage pure predicates and use `Arc<Mutex<_>>` or atomics only when captured state is needed;
- select graph nodes before reading source text, so unselected files incur no custom-rule I/O;
- retain the normalized path in read-error context for actionable diagnostics;
- keep built-in predicates for common cases where a closure and source read are unnecessary.
