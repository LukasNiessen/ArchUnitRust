use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

use super::{CargoProject, SourceFile, SourceOptions, cargo_project::workspace_identifier};
use crate::common::{ArchUnitError, TechnicalError};

/// Directory names omitted from source discovery wherever they occur.
///
/// Cargo's actual target directory is excluded separately, including when configured outside the
/// conventional `target` path.
pub const DEFAULT_EXCLUDED_DIRECTORIES: &[&str] = &[
    ".cache",
    ".git",
    ".hg",
    ".svn",
    "node_modules",
    "target",
    "vendor",
];

const CONVENTIONAL_DEV_DIRECTORIES: &[&str] = &["benches", "examples", "tests"];

/// Enumerates Rust source files under Cargo workspace members.
///
/// Results are deduplicated and sorted by normalized workspace-relative identifier. Symlinked
/// directories are not followed, which prevents escaping a member or traversing a cycle.
pub fn enumerate_source_files(
    project: &CargoProject,
    options: SourceOptions,
) -> Result<Vec<SourceFile>, ArchUnitError> {
    let member_roots = project
        .member_roots()
        .map(Path::to_path_buf)
        .collect::<Vec<_>>();
    let dev_target_roots = project
        .targets()
        .iter()
        .filter(|target| target.is_dev_only())
        .map(|target| target.source().path().to_path_buf())
        .collect::<BTreeSet<_>>();
    let mut sources = BTreeMap::new();

    for member_root in &member_roots {
        visit_directory(
            project,
            member_root,
            member_root,
            &member_roots,
            &dev_target_roots,
            options,
            &mut sources,
        )?;
    }

    Ok(sources.into_values().collect())
}

#[allow(clippy::too_many_arguments)]
fn visit_directory(
    project: &CargoProject,
    member_root: &Path,
    directory: &Path,
    member_roots: &[PathBuf],
    dev_target_roots: &BTreeSet<PathBuf>,
    options: SourceOptions,
    sources: &mut BTreeMap<String, SourceFile>,
) -> Result<(), ArchUnitError> {
    let entries = fs::read_dir(directory).map_err(|source| {
        ArchUnitError::from(TechnicalError::with_source(
            format!(
                "could not enumerate Rust sources under {}",
                directory.display()
            ),
            source,
        ))
    })?;
    let mut entries = entries.collect::<Result<Vec<_>, _>>().map_err(|source| {
        ArchUnitError::from(TechnicalError::with_source(
            format!(
                "could not enumerate Rust sources under {}",
                directory.display()
            ),
            source,
        ))
    })?;
    entries.sort_by_key(fs::DirEntry::path);

    for entry in entries {
        let path = entry.path();
        let file_type = entry.file_type().map_err(|source| {
            ArchUnitError::from(TechnicalError::with_source(
                format!("could not inspect source path {}", path.display()),
                source,
            ))
        })?;

        if file_type.is_dir() {
            if should_prune_directory(project, member_root, &path, member_roots) {
                continue;
            }
            visit_directory(
                project,
                member_root,
                &path,
                member_roots,
                dev_target_roots,
                options,
                sources,
            )?;
            continue;
        }
        if !file_type.is_file() || path.extension() != Some(OsStr::new("rs")) {
            continue;
        }
        if !options.includes_dev_targets()
            && (dev_target_roots.contains(&path) || is_conventional_dev_source(member_root, &path))
        {
            continue;
        }
        let Some(identifier) = workspace_identifier(project.root(), &path) else {
            continue;
        };
        sources.insert(identifier.clone(), SourceFile::new(path, identifier));
    }

    Ok(())
}

fn should_prune_directory(
    project: &CargoProject,
    current_member_root: &Path,
    directory: &Path,
    member_roots: &[PathBuf],
) -> bool {
    if directory.starts_with(project.target_directory()) {
        return true;
    }
    if directory
        .file_name()
        .is_some_and(is_default_excluded_directory)
    {
        return true;
    }
    if directory != current_member_root && member_roots.iter().any(|root| root == directory) {
        return true;
    }

    directory != current_member_root
        && directory.join("Cargo.toml").is_file()
        && !member_roots.iter().any(|root| root == directory)
}

fn is_default_excluded_directory(name: &OsStr) -> bool {
    let name = name.to_string_lossy();
    DEFAULT_EXCLUDED_DIRECTORIES
        .iter()
        .any(|excluded| name.eq_ignore_ascii_case(excluded))
}

fn is_conventional_dev_source(member_root: &Path, path: &Path) -> bool {
    let Ok(relative) = path.strip_prefix(member_root) else {
        return false;
    };
    let Some(first) = relative.components().next() else {
        return false;
    };
    let first = first.as_os_str().to_string_lossy();
    CONVENTIONAL_DEV_DIRECTORIES
        .iter()
        .any(|directory| first.eq_ignore_ascii_case(directory))
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;

    use super::is_default_excluded_directory;

    #[test]
    fn default_exclusions_are_ascii_case_insensitive_on_every_host() {
        assert!(is_default_excluded_directory(OsStr::new("target")));
        assert!(is_default_excluded_directory(OsStr::new("VENDOR")));
        assert!(is_default_excluded_directory(OsStr::new(".Git")));
        assert!(!is_default_excluded_directory(OsStr::new("src")));
    }
}
