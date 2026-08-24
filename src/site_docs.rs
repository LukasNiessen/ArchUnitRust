//! Compile-only hosts for the Rust examples in the GitHub Pages guide.
//!
//! The module exists only while rustdoc collects doctests, so the guide cannot expand the runtime
//! library or its public API. Each site page is attached to a private item below; adding a page
//! without adding its host means its examples are not yet part of the documentation gate.

#![allow(dead_code)]

#[doc = include_str!("../docs/index.md")]
struct LandingPage;

#[doc = include_str!("../docs/grammar.md")]
struct GrammarPage;

#[doc = include_str!("../docs/patterns.md")]
struct PatternsPage;

#[doc = include_str!("../docs/files.md")]
struct FilesPage;

#[doc = include_str!("../docs/layers.md")]
struct LayersPage;

#[doc = include_str!("../docs/slices.md")]
struct SlicesPage;

#[doc = include_str!("../docs/metrics.md")]
struct MetricsPage;

#[doc = include_str!("../docs/graph.md")]
struct GraphPage;
