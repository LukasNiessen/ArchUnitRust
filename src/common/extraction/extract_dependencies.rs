use std::collections::{BTreeMap, BTreeSet};

use super::{
    CargoProject, DependencyExtraction, DependencyReference, DependencyTarget,
    ExtractionDiagnostic, ExtractionDiagnosticKind, ImportKind, SourceFile, SourceOptions,
    cargo_project::CargoDependencyTarget,
    dependency::{InternalResolution, LogicalModule, RawReference},
    enumerate_source_files,
    module_tree::extract_raw_dependencies,
};
use crate::ArchUnitError;

/// Parses selected Cargo targets and extracts Rust dependency references.
///
/// Internal module prefixes and renamed workspace dependencies resolve to workspace-relative files.
/// Sysroot and declared third-party dependencies resolve to their Cargo-visible crate names. Parse,
/// ambiguity, and unknown-name limitations are returned as diagnostics instead of aborting files.
pub fn extract_dependencies(
    project: &CargoProject,
    options: SourceOptions,
) -> Result<DependencyExtraction, ArchUnitError> {
    let sources = enumerate_source_files(project, options)?;
    Ok(extract_dependencies_from_sources(
        project, options, &sources,
    ))
}

pub(crate) fn extract_dependencies_from_sources(
    project: &CargoProject,
    options: SourceOptions,
    sources: &[SourceFile],
) -> DependencyExtraction {
    let raw = extract_raw_dependencies(project, options, sources);
    let internal_aliases = collect_internal_aliases(&raw.references, &raw.index);
    let cargo_aliases = collect_cargo_aliases(&raw.references, project);
    let mut diagnostics = raw.diagnostics;
    let mut references = Vec::new();

    for reference in &raw.references {
        if reference.ignored {
            continue;
        }
        let candidates = classification_candidates(
            reference,
            &raw.index,
            &internal_aliases,
            &cargo_aliases,
            project,
        );
        let target = match candidates.len() {
            0 => {
                if reference.kind != ImportKind::Mod {
                    diagnostics.push(ExtractionDiagnostic::new(
                        reference.source.clone(),
                        Some(reference.line),
                        ExtractionDiagnosticKind::UnknownReference,
                        Some(reference.rendered_path()),
                        Vec::new(),
                        reference.segments.first().map(|segment| {
                            format!(
                                "{segment} is not visible in Cargo package {}",
                                reference.module.package
                            )
                        }),
                    ));
                }
                None
            }
            1 => candidates.into_iter().next(),
            _ => {
                diagnostics.push(ExtractionDiagnostic::new(
                    reference.source.clone(),
                    Some(reference.line),
                    ExtractionDiagnosticKind::AmbiguousReference,
                    Some(reference.rendered_path()),
                    candidates
                        .into_iter()
                        .map(|candidate| candidate.as_str().to_owned())
                        .collect(),
                    None,
                ));
                None
            }
        };
        references.push(DependencyReference::new(
            reference.source.clone(),
            reference.rendered_path(),
            target,
            reference.kind,
            reference.line,
        ));
    }

    DependencyExtraction::new(references, diagnostics)
}

type AliasKey = (LogicalModule, String);
type AliasMap = BTreeMap<AliasKey, BTreeSet<InternalResolution>>;
type CargoAliasMap = BTreeMap<AliasKey, BTreeSet<DependencyTarget>>;

fn collect_cargo_aliases(references: &[RawReference], project: &CargoProject) -> CargoAliasMap {
    let mut aliases = CargoAliasMap::new();
    for reference in references {
        if !matches!(reference.kind, ImportKind::Use | ImportKind::PubUse) {
            continue;
        }
        let Some(binding) = &reference.binding else {
            continue;
        };
        let targets = direct_cargo_targets(reference, project);
        if !targets.is_empty() {
            aliases
                .entry((reference.module.clone(), binding.clone()))
                .or_default()
                .extend(targets);
        }
    }
    aliases
}

fn classification_candidates(
    reference: &RawReference,
    index: &BTreeMap<LogicalModule, String>,
    internal_aliases: &AliasMap,
    cargo_aliases: &CargoAliasMap,
    project: &CargoProject,
) -> BTreeSet<DependencyTarget> {
    let alias_candidates =
        classified_alias_candidates(reference, index, internal_aliases, cargo_aliases);
    if !alias_candidates.is_empty() {
        return alias_candidates;
    }

    let mut candidates = BTreeSet::new();
    match resolve_direct(reference, index) {
        ResolutionOutcome::Found(resolution) => {
            candidates.insert(DependencyTarget::Internal(resolution.source));
        }
        ResolutionOutcome::Ambiguous(resolutions) => {
            candidates.extend(
                resolutions
                    .into_iter()
                    .map(|resolution| DependencyTarget::Internal(resolution.source)),
            );
        }
        ResolutionOutcome::Unresolved => {}
    }
    candidates.extend(direct_cargo_targets(reference, project));

    candidates
}

fn classified_alias_candidates(
    reference: &RawReference,
    index: &BTreeMap<LogicalModule, String>,
    internal_aliases: &AliasMap,
    cargo_aliases: &CargoAliasMap,
) -> BTreeSet<DependencyTarget> {
    let mut candidates = BTreeSet::new();
    if !reference.leading_colon && !starts_with_explicit_root(&reference.segments) {
        if let Some(binding) = reference.segments.first() {
            if let Some(resolutions) =
                internal_aliases.get(&(reference.module.clone(), binding.clone()))
            {
                candidates.extend(resolutions.iter().map(|resolution| {
                    DependencyTarget::Internal(
                        resolve_from_alias(reference, resolution, index).source,
                    )
                }));
            }
            if let Some(targets) = cargo_aliases.get(&(reference.module.clone(), binding.clone())) {
                candidates.extend(targets.iter().cloned());
            }
        }
    }

    candidates
}

fn direct_cargo_targets(
    reference: &RawReference,
    project: &CargoProject,
) -> BTreeSet<DependencyTarget> {
    if starts_with_explicit_root(&reference.segments) {
        return BTreeSet::new();
    }
    let Some(first_segment) = reference.segments.first() else {
        return BTreeSet::new();
    };

    project
        .dependency_targets(
            &reference.module.package,
            reference.module.dependency_scope,
            first_segment,
        )
        .into_iter()
        .map(|target| match target {
            CargoDependencyTarget::Internal(target) => DependencyTarget::Internal(target),
            CargoDependencyTarget::External(target) => DependencyTarget::External(target),
        })
        .collect()
}

fn collect_internal_aliases(
    references: &[RawReference],
    index: &BTreeMap<LogicalModule, String>,
) -> AliasMap {
    let mut aliases = AliasMap::new();
    for reference in references {
        if !matches!(
            reference.kind,
            ImportKind::Use | ImportKind::PubUse | ImportKind::Mod
        ) {
            continue;
        }
        let Some(binding) = &reference.binding else {
            continue;
        };
        if let ResolutionOutcome::Found(resolution) = resolve_direct(reference, index) {
            aliases
                .entry((reference.module.clone(), binding.clone()))
                .or_default()
                .insert(resolution);
        }
    }
    aliases
}

#[cfg(test)]
fn resolve_reference(
    reference: &RawReference,
    index: &BTreeMap<LogicalModule, String>,
    aliases: &AliasMap,
) -> ResolutionOutcome {
    if !reference.leading_colon && !starts_with_explicit_root(&reference.segments) {
        if let Some(binding) = reference.segments.first() {
            if let Some(resolutions) = aliases.get(&(reference.module.clone(), binding.clone())) {
                let mut candidates = BTreeSet::new();
                for resolution in resolutions {
                    candidates.insert(resolve_from_alias(reference, resolution, index));
                }
                return outcome_from_candidates(candidates);
            }
        }
    }

    resolve_direct(reference, index)
}

fn resolve_direct(
    reference: &RawReference,
    index: &BTreeMap<LogicalModule, String>,
) -> ResolutionOutcome {
    if reference.leading_colon || reference.segments.is_empty() {
        return ResolutionOutcome::Unresolved;
    }

    let mut candidates = BTreeSet::new();
    match reference.segments[0].as_str() {
        "crate" => {
            if let Some(found) =
                longest_prefix(index, &reference.module, &reference.segments[1..], 0)
            {
                candidates.insert(found);
            }
        }
        "self" => {
            let mut path = reference.module.segments.clone();
            path.extend_from_slice(&reference.segments[1..]);
            if let Some(found) = longest_prefix(
                index,
                &reference.module,
                &path,
                reference.module.segments.len(),
            ) {
                candidates.insert(found);
            }
        }
        "super" => {
            let super_count = reference
                .segments
                .iter()
                .take_while(|segment| segment.as_str() == "super")
                .count();
            if super_count <= reference.module.segments.len() {
                let base_len = reference.module.segments.len() - super_count;
                let mut path = reference.module.segments[..base_len].to_vec();
                path.extend_from_slice(&reference.segments[super_count..]);
                if let Some(found) = longest_prefix(index, &reference.module, &path, base_len) {
                    candidates.insert(found);
                }
            }
        }
        _ => {
            if let Some(found) = longest_prefix(index, &reference.module, &reference.segments, 1) {
                candidates.insert(found);
            }
            if !reference.module.segments.is_empty() {
                let mut local = reference.module.segments.clone();
                local.extend_from_slice(&reference.segments);
                if let Some(found) = longest_prefix(
                    index,
                    &reference.module,
                    &local,
                    reference.module.segments.len() + 1,
                ) {
                    candidates.insert(found);
                }
            }
        }
    }

    outcome_from_candidates(candidates)
}

fn resolve_from_alias(
    reference: &RawReference,
    alias: &InternalResolution,
    index: &BTreeMap<LogicalModule, String>,
) -> InternalResolution {
    let mut path = alias.module_segments.clone();
    path.extend(reference.segments.iter().skip(1).cloned());
    longest_prefix(index, &reference.module, &path, alias.module_segments.len())
        .unwrap_or_else(|| alias.clone())
}

fn longest_prefix(
    index: &BTreeMap<LogicalModule, String>,
    context: &LogicalModule,
    path: &[String],
    minimum_length: usize,
) -> Option<InternalResolution> {
    for length in (minimum_length..=path.len()).rev() {
        let segments = path[..length].to_vec();
        let key = LogicalModule {
            package: context.package.clone(),
            dependency_scope: context.dependency_scope,
            target: context.target.clone(),
            segments: segments.clone(),
        };
        if let Some(source) = index.get(&key) {
            return Some(InternalResolution {
                source: source.clone(),
                module_segments: segments,
            });
        }
    }
    None
}

fn starts_with_explicit_root(segments: &[String]) -> bool {
    segments
        .first()
        .is_some_and(|segment| matches!(segment.as_str(), "crate" | "self" | "super"))
}

fn outcome_from_candidates(candidates: BTreeSet<InternalResolution>) -> ResolutionOutcome {
    match candidates.len() {
        0 => ResolutionOutcome::Unresolved,
        1 => match candidates.into_iter().next() {
            Some(candidate) => ResolutionOutcome::Found(candidate),
            None => ResolutionOutcome::Unresolved,
        },
        _ => ResolutionOutcome::Ambiguous(candidates.into_iter().collect()),
    }
}

enum ResolutionOutcome {
    Found(InternalResolution),
    Ambiguous(Vec<InternalResolution>),
    Unresolved,
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{AliasMap, ResolutionOutcome, collect_internal_aliases, resolve_reference};
    use crate::common::extraction::{
        ImportKind,
        cargo_project::CargoDependencyScope,
        dependency::{LogicalModule, RawReference},
    };

    fn module(target: &str, segments: &[&str]) -> LogicalModule {
        LogicalModule {
            package: "fixture".to_owned(),
            dependency_scope: CargoDependencyScope::Normal,
            target: target.to_owned(),
            segments: segments.iter().map(ToString::to_string).collect(),
        }
    }

    fn reference(current: LogicalModule, segments: &[&str]) -> RawReference {
        RawReference {
            source: "src/current.rs".to_owned(),
            module: current,
            segments: segments.iter().map(ToString::to_string).collect(),
            leading_colon: false,
            kind: ImportKind::PathReference,
            line: 1,
            binding: None,
            declaration: None,
            ignored: false,
        }
    }

    fn index() -> BTreeMap<LogicalModule, String> {
        [
            (module("lib", &[]), "src/lib.rs"),
            (module("lib", &["api"]), "src/api.rs"),
            (module("lib", &["api", "model"]), "src/api/model.rs"),
            (module("lib", &["consumer"]), "src/consumer.rs"),
            (
                module("lib", &["consumer", "local"]),
                "src/consumer/local.rs",
            ),
        ]
        .into_iter()
        .map(|(module, source)| (module, source.to_owned()))
        .collect()
    }

    #[test]
    fn resolves_explicit_roots_and_longest_internal_prefix() {
        let reference = reference(
            module("lib", &["consumer"]),
            &["crate", "api", "model", "User"],
        );

        let ResolutionOutcome::Found(found) =
            resolve_reference(&reference, &index(), &AliasMap::new())
        else {
            panic!("fixture path should resolve uniquely");
        };

        assert_eq!(found.source, "src/api/model.rs");
        assert_eq!(found.module_segments, ["api", "model"]);
    }

    #[test]
    fn reports_bare_paths_that_match_root_and_local_modules_as_ambiguous() {
        let reference = reference(module("lib", &["consumer"]), &["local", "Thing"]);
        let mut index = index();
        index.insert(module("lib", &["local"]), "src/local.rs".to_owned());

        let ResolutionOutcome::Ambiguous(found) =
            resolve_reference(&reference, &index, &AliasMap::new())
        else {
            panic!("fixture path should be ambiguous");
        };

        assert_eq!(found.len(), 2);
    }

    #[test]
    fn resolves_paths_through_internal_use_aliases() {
        let current = module("lib", &["consumer"]);
        let import = RawReference {
            source: "src/consumer.rs".to_owned(),
            module: current.clone(),
            segments: vec!["crate".to_owned(), "api".to_owned()],
            leading_colon: false,
            kind: ImportKind::Use,
            line: 1,
            binding: Some("public_api".to_owned()),
            declaration: None,
            ignored: false,
        };
        let aliases = collect_internal_aliases(&[import], &index());
        let reference = reference(current, &["public_api", "model", "User"]);

        let ResolutionOutcome::Found(found) = resolve_reference(&reference, &index(), &aliases)
        else {
            panic!("fixture alias should resolve uniquely");
        };

        assert_eq!(found.source, "src/api/model.rs");
    }

    #[test]
    fn explicit_bindings_take_precedence_over_bare_module_candidates() {
        let current = module("lib", &["consumer"]);
        let import = RawReference {
            source: "src/consumer.rs".to_owned(),
            module: current.clone(),
            segments: vec!["crate".to_owned(), "api".to_owned()],
            leading_colon: false,
            kind: ImportKind::Use,
            line: 1,
            binding: Some("local".to_owned()),
            declaration: None,
            ignored: false,
        };
        let aliases = collect_internal_aliases(&[import], &index());
        let reference = reference(current, &["local", "Thing"]);

        let ResolutionOutcome::Found(found) = resolve_reference(&reference, &index(), &aliases)
        else {
            panic!("the explicit local binding should win");
        };

        assert_eq!(found.source, "src/api.rs");
    }
}
