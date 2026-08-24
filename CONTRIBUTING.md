# Contributing to ArchUnitRust

ArchUnitRust follows the same issue-driven product plan as the other ArchUnitEverything ports, with
an intentionally sequential delivery workflow while the initial public surface is being formed.

Read [AGENTS.md](AGENTS.md), the [porting plan](docs/PORTING_PLAN.md), and the relevant issue before
changing code. Release maintainers must also follow [RELEASING.md](RELEASING.md).

## Workflow

1. Start from an up-to-date, clean `main`.
2. Work on one issue at a time. Do not leave another implementation pull request open.
3. Use a conventional branch name:
   - `feature/issue-<number>-<topic>` for product behavior;
   - `fix/issue-<number>-<topic>` for defects;
   - `chore/issue-<number>-<topic>` for tooling or maintenance;
   - `docs/issue-<number>-<topic>` for documentation.
4. Make small conventional commits such as `feat: add normalized graph edges` or
   `test: cover grouped use resolution`. Do not use agent- or tool-branded branch names.
5. Push the branch and open a pull request that links and closes the issue.
6. Wait for required checks, review the diff and test output, merge with a merge commit, and delete
   the remote branch.
7. Refresh local `main` before starting the next issue.

The branch and pull request are part of the engineering record even for a one-maintainer project.
Do not commit product changes directly to `main`.

## Quality bar

Add focused unit tests beside pure modules and at least one public-API integration test for new rule
behavior. Complete Cargo fixture projects belong under `tests/fixtures/`; avoid fixtures made from
isolated source strings when project or module behavior matters.

Run the gates available at the current stage. The final gate set is:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo test --doc
cargo doc --workspace --all-features --no-deps
cargo package --locked
```

Tests must be deterministic and independent of execution order. Public failures should assert the
useful diagnostic, not merely that some error occurred. Never use `unwrap` or `panic!` in library
code, and keep the crate free of unsafe code.

## Deliberate divergence

The sibling libraries are strong references, not specifications for unidiomatic Rust. If a change
diverges because Rust's module system, type system, ownership model, or testing conventions demand
it, explain the trade-off in the pull request. Add or supersede an ADR when the choice affects future
public behavior.
