# ADR 0024: Split CI by evidence and supported host

**Status:** accepted

## Context

ArchUnitRust needs continuous evidence for more than whether the library compiles on one machine.
Its public contract includes ordinary Rust tests, cross-platform path handling, a declared minimum
Rust version, warning-free documentation, a publishable crate archive, and architecture rules that
exercise the library against its own production sources.

One opaque job would make failures harder to classify. Running only the complete suite would also
leave the self-hosting promise hidden among hundreds of tests. The Go, .NET, and Ruby ports provide
direction for always-on repository checks, but Rust's toolchains, Clippy, rustdoc, Cargo packaging,
and host-specific behavior require Rust-native gates.

## Decision

Run `.github/workflows/ci.yml` for every push, every pull request, and manual dispatch with read-only
repository contents permission. Cancel an older run for the same workflow and ref when a newer
commit supersedes it.

Expose four independent jobs:

1. On stable Linux, check formatting, deny every Clippy warning across all targets and features,
   deny rustdoc warnings, and build the crate package.
2. On stable Linux, Windows, and macOS, run the complete workspace test suite with all features.
3. On stable Linux, run `tests/architecture.rs` separately as the visible dogfooding gate, even
   though the complete suite also includes it.
4. On Linux with Rust 1.85.0, check all workspace targets and features to enforce the declared MSRV.

Every Cargo command uses `Cargo.lock`. Toolchains are installed with rustup's minimal profile, and
the quality components are requested only by the job that needs them. The first workflow uses no
dependency cache or third-party Rust setup action; the official runner tool cache and Cargo's own
incremental behavior are sufficient until run data justifies more moving parts.

## Consequences

- A PR presents formatting, lint, documentation, packaging, host portability, self-architecture,
  and MSRV failures as distinct review signals.
- Path and process assumptions are exercised on all three primary GitHub-hosted operating systems.
- The architecture test intentionally runs twice on Linux. The small cost buys a named required
  signal that cannot disappear silently inside a general test command.
- Stable verifies current compiler behavior while Rust 1.85.0 protects the compatibility promise.
- Adding a cache later requires an explicit decision about action trust, cache keys, and observed
  workflow cost rather than becoming an unexamined default.

## Alternatives considered

### Use one Linux job

Rejected. It is quick but cannot validate Windows and macOS path semantics, and it mixes unrelated
failure modes into a long serial log.

### Run only the stable toolchain

Rejected. A stable build can start accepting syntax or dependency versions newer than the declared
MSRV without warning.

### Rely on the complete test suite for dogfooding

Rejected. The tests would execute, but the repository's defining self-architecture guarantee would
not be independently discoverable or enforceable in branch protection.
