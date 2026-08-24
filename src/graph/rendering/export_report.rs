use std::{fs, path::Path};

use crate::common::{ArchUnitError, TechnicalError, UserError};

/// Writes a rendered graph report as UTF-8, creating missing parent directories.
pub fn export_graph_report(
    output_path: impl AsRef<Path>,
    content: &str,
) -> Result<(), ArchUnitError> {
    let output_path = output_path.as_ref();
    if output_path.as_os_str().is_empty() {
        return Err(UserError::new("graph report output path must not be empty").into());
    }

    if let Some(parent) = output_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).map_err(|source| {
            TechnicalError::with_source(
                format!(
                    "could not create graph report directory '{}'",
                    parent.display()
                ),
                source,
            )
        })?;
    }

    fs::write(output_path, content.as_bytes()).map_err(|source| {
        TechnicalError::with_source(
            format!("could not write graph report '{}'", output_path.display()),
            source,
        )
        .into()
    })
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path, time::SystemTime};

    use crate::common::ArchUnitError;

    use super::export_graph_report;

    fn temporary_directory(test_name: &str) -> std::path::PathBuf {
        let nonce = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("system clock should follow the Unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "archunit-graph-export-{}-{test_name}-{nonce}",
            std::process::id()
        ))
    }

    #[test]
    fn writes_utf8_and_creates_missing_parent_directories() {
        let root = temporary_directory("utf8");
        let output = root.join("nested/report.txt");

        export_graph_report(&output, "Rust architecture: λ").expect("report export should succeed");

        assert_eq!(
            fs::read_to_string(&output).expect("export should be readable as UTF-8"),
            "Rust architecture: λ"
        );
        fs::remove_dir_all(root).expect("temporary export should be removable");
    }

    #[test]
    fn rejects_an_empty_output_path_as_user_input() {
        let error = export_graph_report(Path::new(""), "report")
            .expect_err("empty output path should be rejected");

        assert!(matches!(error, ArchUnitError::User(_)));
    }

    #[test]
    fn classifies_filesystem_failures_as_technical_errors() {
        let root = temporary_directory("technical");
        fs::create_dir_all(&root).expect("temporary directory should be created");
        let parent_file = root.join("parent-file");
        fs::write(&parent_file, "not a directory").expect("fixture file should be written");

        let error = export_graph_report(parent_file.join("report.txt"), "report")
            .expect_err("a file cannot be used as a parent directory");

        assert!(matches!(error, ArchUnitError::Technical(_)));
        fs::remove_dir_all(root).expect("temporary export should be removable");
    }
}
