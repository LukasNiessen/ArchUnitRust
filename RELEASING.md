# Releasing ArchUnitRust

The crates.io package and Rust import are both named **archunit**. Every push or merge to main
starts the Release workflow. A manual workflow dispatch on main retries or completes a release.

## Credentials

The repository secret CARGO_REGISTRY_TOKEN contains a crates.io API token permitted to publish
archunit. The first publication also needs permission to create this crate, and the account must
have a verified email address. The token is exposed only to the cargo publish step; it is not
stored in Git or passed as a command-line argument.

Maintainers can replace the secret with GitHub's repository settings or
gh secret set CARGO_REGISTRY_TOKEN --repo LukasNiessen/ArchUnitRust, supplying the token on stdin.

## Automatic releases

1. The reusable CI workflow runs formatting, Clippy, documentation, archive validation, the full
   Linux/Windows/macOS test matrix, architecture tests, MSRV checks, and release-script tests.
2. The release script checks crates.io. It uses Cargo.toml's version for the first publication.
   For a new source commit whose manifest version is already published, it increments the latest
   stable patch version. An explicit higher minor or major version in Cargo.toml is honored.
3. The script updates the root manifest and lockfile, current installation examples, changelog,
   and exact-version registry-consumer fixture. It commits only those release metadata files.
4. The workflow reruns the quality gates and cargo publish --dry-run on that exact clean commit,
   then pushes the metadata commit to main without force. If main advanced in the meantime, the
   push fails before uploading; the newer main run will release the combined changes.
5. cargo publish --locked uploads the crate. A separate temporary Cargo project then installs the
   exact published version from crates.io and runs an architecture test against its own source.
6. Only after that consumer passes does the workflow create the matching tag and GitHub release.

Ordinary product changes still follow the issue/branch/PR workflow. The release bot is the explicit
exception for version metadata on main. Its GITHUB_TOKEN push does not trigger another workflow,
so automatic version bumps do not cause a release loop. Release attempts are serialized; GitHub
may replace an older pending run with a newer one, which includes the intervening main changes.

## Retrying failures

Use the GitHub Actions rerun controls or dispatch Release on main. Each metadata commit records
the original source SHA in a Release-Source trailer. A rerun recovers that commit, and compares
the archive's .cargo_vcs_info.json commit with the checkout. If that exact version and commit are
already on crates.io, publishing is skipped and registry verification/release creation resume.
A registry or network error fails the job; only an HTTP 404 means the crate does not yet exist.

Crates.io versions cannot be overwritten, including yanked versions. Never force a release tag
to point at different source. If a published version has a product defect, fix it through a PR;
the next main run publishes a new patch.

## Local checks

Run the Cargo gates in CONTRIBUTING.md, plus:

    python -m unittest discover -s .github/scripts -p "test_*.py"
    cargo publish --dry-run --locked

The release script itself is intended for GitHub Actions: it requires GITHUB_SHA and GITHUB_OUTPUT,
reads crates.io, and may create a local metadata commit. Do not run it in a dirty development tree.
