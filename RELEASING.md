# Releasing ArchUnitRust

Crates.io versions are immutable. A release is complete only when the registry archive can be
installed by a separate project, the matching Git tag and GitHub release exist, and the public
installation instructions name that published version.

## One-time bootstrap for 0.0.1

Crates.io requires a crate's first version to be published manually before a trusted GitHub
publisher can be configured. From a clean, current `main` whose CI is green:

1. Sign in to crates.io, verify the account email address, and create a short-lived API token that
   is permitted to publish a new crate. Never put the token in Git, an issue, a workflow log, or a
   chat message.
2. Store it with `cargo login`, run `cargo publish --dry-run --locked`, inspect the reported archive
   contents, then run `cargo publish --locked` exactly once.
3. Copy `tests/fixtures/registry_consumer/` to a temporary directory, rename
   `Cargo.toml.template` to `Cargo.toml`, and run `cargo test --manifest-path <temp>/Cargo.toml`.
   Retry briefly if the crates.io index has not propagated yet. This fixture must resolve
   `archunit = "=0.0.1"`; do not add a path or Git override.
4. Only after that test passes, create the matching release boundary:

   ```console
   gh release create v0.0.1 --target main --title "archunit v0.0.1" --generate-notes
   ```

5. Remove the local token with `cargo logout` if it is not needed for another crate.
6. In the crate's crates.io Trusted Publishing settings, add the GitHub repository owner
   `LukasNiessen`, repository `ArchUnitRust`, workflow `release.yml`, and environment `release`.

Do not invoke `.github/workflows/release.yml` for 0.0.1: the trusted publisher cannot exist until
that first version has reserved the crate name.

## Later releases

Prepare each version in its own reviewed branch and PR:

1. Update `package.version` in `Cargo.toml` and refresh `Cargo.lock`.
2. Move the relevant changelog entries under a dated version heading and update comparison links.
3. Update the exact `archunit` version in
   `tests/fixtures/registry_consumer/Cargo.toml.template` and the constant in
   `tests/release_workflow.rs`.
4. Run the complete local gate set and merge only after CI is green.
5. From `main`, manually dispatch the Release workflow with the version number and no leading `v`.

The publish job rechecks the version, tag absence, formatting, Clippy, all tests, rustdoc, and the
crate archive before obtaining a short-lived crates.io token through OpenID Connect. The verify job
then installs the exact registry version in the standalone fixture. Only after that succeeds does it
create the Git tag and GitHub release.

If registry propagation delays the fixture, rerun only the failed verify job. The successful publish
job must not be rerun for an immutable version. If publication itself fails before crates.io accepts
the archive, correct the cause in a new PR; never reuse a version that the registry accepted.
