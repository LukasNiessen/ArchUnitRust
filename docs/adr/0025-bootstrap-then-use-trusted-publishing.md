# ADR 0025: Bootstrap once, then publish with short-lived credentials

**Status:** accepted

## Context

Publishing a crates.io version is irreversible: its archive cannot be overwritten or deleted. A
long-lived repository secret would make later automation possible, but it would also remain useful
to an attacker until rotated or expired. Crates.io Trusted Publishing exchanges GitHub's OpenID
Connect identity for a short-lived token, but the trusted publisher can only be configured by an
owner after the crate's first version exists.

A release also has two different success boundaries. Uploading the archive proves registry
acceptance; it does not prove that a clean consumer can resolve, compile, and exercise the exact
version. Combining upload and propagation-sensitive verification in one job makes recovery unsafe:
rerunning a failed job would attempt to publish the same immutable version again.

## Decision

Publish 0.0.1 manually from a clean, green `main` with a locally entered, short-lived crates.io token.
The token is never committed or transmitted through GitHub. Verify the accepted version through a
standalone fixture whose development dependency pins `archunit = "=0.0.1"`. Create the Git tag and
GitHub release only after that registry-backed test passes, then remove the local token.

After the crate exists, configure crates.io to trust `.github/workflows/release.yml` in the GitHub
`release` environment. Later releases use manual workflow dispatch from `main`. The workflow denies
version drift and existing tags, repeats all quality gates, obtains an ephemeral token through the
Rust project's authentication action, and publishes with the lockfile enforced.

Keep publication and verification in separate jobs. The second job polls registry installation,
runs a real fluent architecture rule, and creates the tag and GitHub release. A delayed registry
index can therefore rerun verification without rerunning a successful upload.

## Consequences

- The first release has one documented manual credential step; later releases store no crates.io
  token in GitHub.
- A version tag means both crates.io acceptance and successful clean-consumer installation.
- The registry fixture is version-coupled release data and must change with every version bump.
- The `release` environment becomes part of the crates.io trust identity and cannot be renamed
  casually.
- A publication accepted by crates.io remains released even if a later verification service is
  temporarily unavailable; the split job makes completing that boundary safely rerunnable.

## Alternatives considered

### Store a crates.io token as a GitHub Actions secret

Rejected. It solves the first upload but leaves a reusable credential in a second system for every
later release.

### Create the tag before publishing

Rejected. A rejected archive would leave a public release marker for a version users cannot install.

### Publish and verify in one job

Rejected. Registry propagation failure would make the failed job unsafe to rerun because its upload
step may already have succeeded.
