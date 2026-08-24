use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

use cargo_metadata::MetadataCommand;

use super::{CargoProject, ProjectLocator};
use crate::common::{ArchUnitError, TechnicalError, UserError};

const MANIFEST_NAME: &str = "Cargo.toml";

/// Locates the Cargo project above the current working directory.
///
/// Cargo metadata is the authority for workspace membership, targets, and the final workspace root.
pub fn locate_project() -> Result<CargoProject, ArchUnitError> {
    locate_project_from(&ProjectLocator::auto_detect())
}

/// Locates the Cargo project selected by an explicit or automatic locator.
///
/// The nearest ancestor manifest is passed to `cargo metadata --no-deps`; this means a locator
/// inside a workspace member still resolves to the containing virtual or package workspace.
pub fn locate_project_from(locator: &ProjectLocator) -> Result<CargoProject, ArchUnitError> {
    let manifest = locate_manifest(locator)?;
    let mut command = MetadataCommand::new();
    command.manifest_path(&manifest).no_deps();
    if let Some(parent) = manifest.parent() {
        command.current_dir(parent);
    }
    let metadata = command.exec().map_err(|source| {
        ArchUnitError::from(TechnicalError::with_source(
            format!("could not read Cargo metadata for {}", manifest.display()),
            source,
        ))
    })?;

    CargoProject::from_metadata(metadata).map_err(ArchUnitError::from)
}

fn locate_manifest(locator: &ProjectLocator) -> Result<PathBuf, ArchUnitError> {
    let start = match locator.path() {
        Some(path) => explicit_start(path)?,
        None => automatic_start()?,
    };

    if start.is_file() {
        return Ok(start);
    }

    for directory in start.ancestors() {
        let candidate = directory.join(MANIFEST_NAME);
        match fs::metadata(&candidate) {
            Ok(metadata) if metadata.is_file() => return Ok(candidate),
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(source) => {
                return Err(ArchUnitError::from(TechnicalError::with_source(
                    format!("could not inspect {}", candidate.display()),
                    source,
                )));
            }
        }
    }

    Err(ArchUnitError::from(UserError::new(format!(
        "project locator {} has no Cargo.toml in it or any parent",
        start.display()
    ))))
}

fn automatic_start() -> Result<PathBuf, ArchUnitError> {
    let working_directory = env::current_dir().map_err(|source| {
        ArchUnitError::from(TechnicalError::with_source(
            "could not find the working directory for Cargo project discovery",
            source,
        ))
    })?;
    fs::canonicalize(&working_directory).map_err(|source| {
        ArchUnitError::from(TechnicalError::with_source(
            format!(
                "could not resolve the working directory {}",
                working_directory.display()
            ),
            source,
        ))
    })
}

fn explicit_start(path: &Path) -> Result<PathBuf, ArchUnitError> {
    let resolved = fs::canonicalize(path).map_err(|source| {
        ArchUnitError::from(UserError::with_source(
            format!("project locator {} is not usable", path.display()),
            source,
        ))
    })?;
    let metadata = fs::metadata(&resolved).map_err(|source| {
        ArchUnitError::from(UserError::with_source(
            format!("project locator {} is not usable", path.display()),
            source,
        ))
    })?;

    if metadata.is_dir() {
        return Ok(resolved);
    }
    if metadata.is_file()
        && resolved
            .file_name()
            .is_some_and(|name| name == MANIFEST_NAME)
    {
        return Ok(resolved);
    }

    Err(ArchUnitError::from(UserError::new(format!(
        "project locator {} must be a directory or Cargo.toml manifest",
        path.display()
    ))))
}

#[cfg(test)]
mod tests {
    use super::{locate_project, locate_project_from};
    use crate::common::ProjectLocator;

    #[test]
    fn auto_detection_finds_this_crate() {
        let project = locate_project().expect("this test runs inside a Cargo project");

        assert!(project.manifest_path().is_file());
        assert!(project.root().join("src/lib.rs").is_file());
    }

    #[test]
    fn explicit_manifest_finds_the_same_workspace() {
        let automatic = locate_project().expect("this test runs inside a Cargo project");
        let locator = ProjectLocator::from_path(automatic.manifest_path());
        let explicit =
            locate_project_from(&locator).expect("the discovered manifest should reload");

        assert_eq!(explicit.root(), automatic.root());
        assert_eq!(explicit.targets(), automatic.targets());
    }
}
