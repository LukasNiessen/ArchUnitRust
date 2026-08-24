# ADR 0023: Publish a source-checked guide beside rustdoc

**Status:** accepted

## Context

The README is a useful entry point but is too long to be the only user documentation. The sibling
ports organize their sites around the same journey: start, grammar and patterns, files, layers,
slices, metrics, graph reports, execution, and internals. Rust also needs an API reference generated
from public source comments rather than a hand-maintained catalogue.

A documentation generator can itself become a second package ecosystem, lockfile, update stream,
and source of broken builds. At the same time, plain Markdown without validation can describe APIs
that do not exist. ArchUnitTS's historical slice documentation is the concrete failure mode this
site must prevent.

## Decision

Keep the user guide as ten Markdown pages under `docs/`, using GitHub Pages' supported Jekyll build,
one Liquid layout, and one stylesheet. Preserve the siblings' information architecture while
writing every Rust statement from this repository's public source, tests, and accepted ADRs.

Generate the API reference with `cargo doc --all-features --no-deps` from the same commit and place
it at `/api/`. The guide links to `/api/archunit/`; no API signature is copied into a second
hand-written reference.

Compile every Rust example by attaching each page through `include_str!` to a private module enabled
only under `cfg(doctest)`. Add an integration test that fixes the expected chapter set and order,
validates front matter, local page links and fragments, layout accessibility hooks, doctest hosts,
and the inputs of the Pages workflow.

Publish only `main`. The build runs even when Pages is disabled; deployment is skipped with a notice
until the repository setting uses GitHub Actions as its Pages source. Guide and API artifacts are
then uploaded and deployed together.

## Consequences

- A code example that stops compiling fails before publication.
- A renamed or missing chapter, orphaned page, broken fragment, missing navigation field, or removed
  deployment input fails a normal Rust integration test.
- The guide has no Node, Ruby, or mdBook dependency in this repository. GitHub's maintained Pages
  action owns the Jekyll environment.
- The shipped crate has no documentation-only public module because the page hosts exist only while
  rustdoc collects doctests.
- Architecture records remain beside the guide but are excluded from the published Jekyll source.
- Pages availability remains an administrator-controlled repository setting; the workflow reports
  that state honestly instead of turning a missing site into a false deployment failure.

## Alternatives considered

### Put everything in the README

Rejected. A landing document cannot provide the navigable depth of a real guide without becoming a
poor entry point, and it offers no clean place for generated API documentation.

### Use mdBook

Rejected for the first site. mdBook is Rust-native, but this guide does not need its search runtime
or another installed binary. The sibling chapter structure maps directly to Markdown and Jekyll.
This can be revisited if the guide needs book-specific features.

### Hand-write an API reference

Rejected. Public doc comments and signatures already have a canonical renderer in rustdoc. Copying
them creates the exact drift risk this issue is intended to remove.
