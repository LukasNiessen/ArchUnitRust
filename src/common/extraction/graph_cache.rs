use std::{
    collections::BTreeMap,
    sync::{OnceLock, RwLock},
};

use super::{
    CargoProject, DEFAULT_EXCLUDED_DIRECTORIES, GraphExtraction, SourceOptions,
    extract_graph::extract_graph_uncached,
};
use crate::{ArchUnitError, CheckOptions, TechnicalError};

const CFG_POLICY: &str = "conservative-union";
const FEATURE_POLICY: &str = "cargo-metadata-declarations";
const IGNORE_SCOPE_POLICY: &str = "declaration-comments-v1";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct GraphCacheKey {
    project: CargoProject,
    source_options: SourceOptions,
    excluded_directories: Vec<String>,
    cfg_policy: &'static str,
    feature_policy: &'static str,
    ignore_scope_policy: &'static str,
}

#[derive(Default)]
struct GraphCacheState {
    generation: u64,
    entries: BTreeMap<GraphCacheKey, GraphExtraction>,
}

static GRAPH_CACHE: OnceLock<RwLock<GraphCacheState>> = OnceLock::new();

fn graph_cache() -> &'static RwLock<GraphCacheState> {
    GRAPH_CACHE.get_or_init(|| RwLock::new(GraphCacheState::default()))
}

/// Builds the complete identity for inputs that can change graph extraction.
///
/// `CargoProject` includes canonical workspace and manifest paths, the target directory, workspace
/// members, target kinds/source roots, and Cargo dependency declarations. New extraction toggles
/// must be represented here before they are honored by the extractor.
fn build_graph_cache_key(project: &CargoProject, source_options: SourceOptions) -> GraphCacheKey {
    GraphCacheKey {
        project: project.clone(),
        source_options,
        excluded_directories: DEFAULT_EXCLUDED_DIRECTORIES
            .iter()
            .map(|directory| (*directory).to_owned())
            .collect(),
        cfg_policy: CFG_POLICY,
        feature_policy: FEATURE_POLICY,
        ignore_scope_policy: IGNORE_SCOPE_POLICY,
    }
}

/// Extracts or reuses the graph for one Cargo project and source configuration.
pub fn extract_graph(
    project: &CargoProject,
    source_options: SourceOptions,
) -> Result<GraphExtraction, ArchUnitError> {
    extract_graph_cached(project, source_options, false)
}

/// Extracts a graph using the extraction-related settings from one architecture check.
///
/// [`CheckOptions::clears_cache`] empties the shared cache before extraction. Test, example, and
/// benchmark target selection is mapped from [`CheckOptions::includes_test_sources`].
pub fn extract_graph_with_options(
    project: &CargoProject,
    options: &CheckOptions,
) -> Result<GraphExtraction, ArchUnitError> {
    let source_options = SourceOptions::new().with_dev_targets(options.includes_test_sources());
    extract_graph_cached(project, source_options, options.clears_cache())
}

/// Clears every memoized graph and its diagnostics in the current process.
pub fn clear_graph_cache() -> Result<(), ArchUnitError> {
    let mut cache = graph_cache()
        .write()
        .map_err(|_| cache_lock_error("clear"))?;
    cache.entries.clear();
    cache.generation = cache.generation.wrapping_add(1);
    Ok(())
}

fn extract_graph_cached(
    project: &CargoProject,
    source_options: SourceOptions,
    clear_before: bool,
) -> Result<GraphExtraction, ArchUnitError> {
    if clear_before {
        clear_graph_cache()?;
    }
    let key = build_graph_cache_key(project, source_options);
    let generation = {
        let cache = graph_cache().read().map_err(|_| cache_lock_error("read"))?;
        if let Some(extraction) = cache.entries.get(&key) {
            return Ok(extraction.clone());
        }
        cache.generation
    };

    let extracted = extract_graph_uncached(project, source_options)?;
    let mut cache = graph_cache()
        .write()
        .map_err(|_| cache_lock_error("write"))?;
    if cache.generation != generation {
        return Ok(extracted);
    }
    Ok(cache.entries.entry(key).or_insert(extracted).clone())
}

fn cache_lock_error(operation: &str) -> ArchUnitError {
    ArchUnitError::from(TechnicalError::new(format!(
        "could not {operation} the shared graph cache because its lock was poisoned"
    )))
}

#[cfg(test)]
mod tests {
    use super::{CFG_POLICY, build_graph_cache_key};
    use crate::{SourceOptions, locate_project};

    #[test]
    fn named_key_changes_with_source_options_and_captures_policy() {
        let project = locate_project().expect("this test runs inside a Cargo project");
        let production = build_graph_cache_key(&project, SourceOptions::new());
        let with_dev = build_graph_cache_key(&project, SourceOptions::new().with_dev_targets(true));

        assert_ne!(production, with_dev);
        assert_eq!(production.cfg_policy, CFG_POLICY);
        assert!(!production.excluded_directories.is_empty());
    }
}
