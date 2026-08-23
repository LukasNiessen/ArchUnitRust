use std::collections::{BTreeMap, BTreeSet};

use super::{
    CargoProject, DependencyExtraction, DependencyReference, ExtractionDiagnostic,
    ExtractionDiagnosticKind, ImportKind, SourceOptions,
    dependency::{InternalResolution, LogicalModule, RawReference},
    enumerate_source_files,
    module_tree::extract_raw_dependencies,
};
use crate::ArchUnitError;

/// Parses selected Cargo targets and extracts Rust dependency references.
///
/// Internal module prefixes are resolved to workspace-relative files. Paths not resolved internally
/// remain raw for Cargo-aware external/unknown classification. Parse and resolution limitations are
/// returned as diagnostics instead of aborting other files.
pub fn extract_dependencies(
    project: &CargoProject,
    options: SourceOptions,
) -> Result<DependencyExtraction, ArchUnitError> {
    let sources = enumerate_source_files(project, options)?;
    let raw = extract_raw_dependencies(project, options, &sources);
    let aliases = collect_internal_aliases(&raw.references, &raw.index);
    let mut diagnostics = raw.diagnostics;
    let mut references = Vec::new();

    for reference in &raw.references {
        let outcome = resolve_reference(reference, &raw.index, &aliases);
        let internal_target = match outcome {
            ResolutionOutcome::Found(resolution) => Some(resolution.source),
            ResolutionOutcome::Unresolved => None,
            ResolutionOutcome::Ambiguous(candidates) => {
                diagnostics.push(ExtractionDiagnostic::new(
                    reference.source.clone(),
                    Some(reference.line),
                    ExtractionDiagnosticKind::AmbiguousReference,
                    Some(reference.rendered_path()),
                    candidates
                        .into_iter()
                        .map(|candidate| candidate.source)
                        .collect(),
                    None,
                ));
                None
            }
        };
        references.push(DependencyReference::new(
            reference.source.clone(),
            reference.rendered_path(),
            internal_target,
            reference.kind,
            reference.line,
        ));
    }

    Ok(DependencyExtraction::new(references, diagnostics))
}

type AliasKey = (LogicalModule, String);
type AliasMap = BTreeMap<AliasKey, BTreeSet<InternalResolution>>;

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

fn resolve_reference(
    reference: &RawReference,
    index: &BTreeMap<LogicalModule, String>,
    aliases: &AliasMap,
) -> ResolutionOutcome {
    let mut candidates = BTreeSet::new();
    add_outcome(&mut candidates, resolve_direct(reference, index));

    if !reference.leading_colon && !starts_with_explicit_root(&reference.segments) {
        if let Some(binding) = reference.segments.first() {
            if let Some(resolutions) = aliases.get(&(reference.module.clone(), binding.clone())) {
                for resolution in resolutions {
                    candidates.insert(resolve_from_alias(reference, resolution, index));
                }
            }
        }
    }

    outcome_from_candidates(candidates)
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
                longest_prefix(index, &reference.module.target, &reference.segments[1..], 0)
            {
                candidates.insert(found);
            }
        }
        "self" => {
            let mut path = reference.module.segments.clone();
            path.extend_from_slice(&reference.segments[1..]);
            if let Some(found) = longest_prefix(
                index,
                &reference.module.target,
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
                if let Some(found) =
                    longest_prefix(index, &reference.module.target, &path, base_len)
                {
                    candidates.insert(found);
                }
            }
        }
        _ => {
            if let Some(found) =
                longest_prefix(index, &reference.module.target, &reference.segments, 1)
            {
                candidates.insert(found);
            }
            if !reference.module.segments.is_empty() {
                let mut local = reference.module.segments.clone();
                local.extend_from_slice(&reference.segments);
                if let Some(found) = longest_prefix(
                    index,
                    &reference.module.target,
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
    longest_prefix(
        index,
        &reference.module.target,
        &path,
        alias.module_segments.len(),
    )
    .unwrap_or_else(|| alias.clone())
}

fn longest_prefix(
    index: &BTreeMap<LogicalModule, String>,
    target: &str,
    path: &[String],
    minimum_length: usize,
) -> Option<InternalResolution> {
    for length in (minimum_length..=path.len()).rev() {
        let segments = path[..length].to_vec();
        let key = LogicalModule {
            target: target.to_owned(),
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

fn add_outcome(candidates: &mut BTreeSet<InternalResolution>, outcome: ResolutionOutcome) {
    match outcome {
        ResolutionOutcome::Found(resolution) => {
            candidates.insert(resolution);
        }
        ResolutionOutcome::Ambiguous(resolutions) => candidates.extend(resolutions),
        ResolutionOutcome::Unresolved => {}
    }
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
        dependency::{LogicalModule, RawReference},
    };

    fn module(target: &str, segments: &[&str]) -> LogicalModule {
        LogicalModule {
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
        };
        let aliases = collect_internal_aliases(&[import], &index());
        let reference = reference(current, &["public_api", "model", "User"]);

        let ResolutionOutcome::Found(found) = resolve_reference(&reference, &index(), &aliases)
        else {
            panic!("fixture alias should resolve uniquely");
        };

        assert_eq!(found.source, "src/api/model.rs");
    }
}
