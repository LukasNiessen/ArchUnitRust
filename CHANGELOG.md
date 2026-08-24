# Changelog

All notable changes to ArchUnitRust are documented in this file. The project follows Semantic
Versioning; the `0.0.x` line deliberately signals that the public API is still experimental.

## [Unreleased]

## [0.0.1] - 2026-08-24

### Added

- Deterministic Cargo package and workspace discovery with Rust module-tree and dependency
  extraction.
- Fluent architecture rules for files, named layers, captured slices, and external Cargo modules.
- Public graph projections, cycle analysis, six offline report formats, and PlantUML policies.
- Rust-native count, cohesion, coupling, distance, custom-metric, and metrics-report APIs.
- Strict empty-test protection, reusable pattern exclusions, typed violations, deterministic
  diagnostics, `assert_passes!`, and opt-in per-check logging.
- Self-hosting architecture tests, a cross-platform CI matrix, a source-checked user guide, and
  generated API documentation.

### Known limitations

- Analysis is syntax-based and does not expand macros or build-script-generated source.
- Conditional compilation branches are modeled as a conservative union.
- Dependency nodes are source files rather than individual Rust items.

[Unreleased]: https://github.com/LukasNiessen/ArchUnitRust/compare/v0.0.1...HEAD
[0.0.1]: https://github.com/LukasNiessen/ArchUnitRust/releases/tag/v0.0.1
