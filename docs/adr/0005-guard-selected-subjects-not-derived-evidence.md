# ADR 0005: Guard selected subjects, not derived evidence

- **Status:** accepted
- **Date:** 2026-08-23
- **Decision owners:** ArchUnitRust maintainers
- **Tracking:** [#23](https://github.com/LukasNiessen/ArchUnitRust/issues/23)

## Context

An architecture rule that selects no subjects can pass forever after a path typo or stale rename.
The absence of ordinary violations is not enough to detect this: a valid rule over existing files
also returns an empty list when every subject satisfies its requirement.

Relational and graph rules introduce a second ambiguity. One selected file can legitimately have
no outgoing dependency edges and no cycles. Treating derived evidence as the selection would call
that useful check empty even though its subject exists. The TypeScript role model documents this
most visibly for cycle rules, and the completed Ruby port applies one subject-selection guard to
all current file terminals.

## Decision

Every terminal determines its selected subjects before deriving rule-specific evidence. Under the
default `CheckOptions`, zero selected subjects returns exactly one `EmptyTestViolation`. That value
contains the domain subject name, subject selectors in fluent-chain order and the rule's positive
or negated mood. It is an ordinary typed violation, not a `UserError` or `TechnicalError`.

`CheckOptions::with_allow_empty_tests(true)` is the only opt-out and converts that result to an
empty violation list. Selector and other API validation still happen before project discovery, so
the option cannot hide malformed patterns.

The domain-neutral assertion `gather_empty_test_violations` judges any subject slice without
depending on extraction or a rule domain. The Files domain owns `file_rule_support`, which selects
projected nodes once and converts the pure empty-test data into the library-wide `Violation` sum.
Cycle, pattern, internal-dependency, external-dependency and custom-predicate terminals all call
that support before their specific assertion.

For file rules, emptiness therefore means zero selected projected file nodes. It never means zero
dependency edges, zero cycles, zero predicate failures or zero final violations.

## Alternatives considered

### Treat an empty final violation list as an empty test

Rejected. An empty final list is the successful result of every satisfied non-empty rule. The
terminal must retain the subject selection independently of its verdict.

### Guard dependency edges or cycle projections

Rejected. A selected isolated file has no such evidence but is still a real architecture subject.
This would create false empty-test failures for dependency-free modules and acyclic single-file
scopes.

### Make zero matches a user error

Rejected. A selector can be valid and intentionally match nothing in one project state. The rule
executed successfully and reached an architecture verdict, so the result belongs in the violation
channel and remains compatible with test reporting.

### Implement the guard inside `Checkable`

Rejected. The object-safe terminal contract receives only options and a final result; it cannot
know which intermediate collection represents the domain's subjects. Each domain must expose that
selection to the shared pure assertion at the correct stage.

### Let each terminal implement its own guard

Rejected. Repeated local conditions invite drift in option handling, selector evidence and mood.
A single domain support path also makes it testable that all current terminal families behave the
same way.

## Consequences

### Positive

- stale or misspelled scopes fail safely by default;
- isolated and dependency-free selected files are judged normally;
- all file terminals preserve identical selector, mood and option behavior;
- reporters receive structured empty-test data through the ordinary violation sum;
- future domains can reuse the pure assertion with their own subject selection.

### Negative

- every new terminal must identify its subject collection before its specific assertion;
- dependency terminals currently project nodes as well as edges during one check;
- the public `EmptyTestViolation` carries one additional mood field;
- callers that intentionally permit missing scopes must pass explicit options on every check.

### Mitigations

- keep domain selection in one small support module and test the complete terminal catalogue;
- derive both node and edge projections from the same cached raw graph;
- retain `EmptyTestViolation::new` as a positive-mood convenience constructor;
- document the opt-out next to the default behavior and keep it explicit rather than global.
