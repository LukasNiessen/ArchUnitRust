use std::fs;

use crate::common::{ArchUnitError, CargoProject, TechnicalError};

use super::FileInfo;

pub(crate) fn extract_file_info(
    project: &CargoProject,
    identifier: &str,
) -> Result<FileInfo, ArchUnitError> {
    let source_path = project.root().join(identifier);
    let content = fs::read_to_string(source_path).map_err(|error| {
        ArchUnitError::from(TechnicalError::with_source(
            format!("could not read project source file {identifier}"),
            error,
        ))
    })?;

    Ok(FileInfo::new(identifier, content))
}
