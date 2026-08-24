use std::{
    fs,
    path::{Path, PathBuf},
    process,
    time::{SystemTime, UNIX_EPOCH},
};

use archunit::{
    ArchUnitError, CheckLogger, CheckOptions, Checkable, LogFileMode, LogLevel, LoggingOptions,
    metrics_in, project_files_in, project_layers_in, project_slices_in,
};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

struct TemporaryDirectory(PathBuf);

impl TemporaryDirectory {
    fn new(label: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("test clock should follow the Unix epoch")
            .as_nanos();
        Self(std::env::temp_dir().join(format!(
            "archunit-public-logging-{label}-{}-{nonce}",
            process::id()
        )))
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn built_in_families_emit_lifecycle_progress_violation_and_metric_records() {
    let output = TemporaryDirectory::new("families");
    let logging = LoggingOptions::new()
        .with_level(LogLevel::Debug)
        .with_console_output(false)
        .with_file_output(output.path().join("nested"))
        .with_file_mode(LogFileMode::Overwrite);
    let log_path = logging
        .file_path()
        .expect("file logging should expose its timestamped path")
        .to_path_buf();
    let options = CheckOptions::new().with_logging(logging);
    let layered = fixture("layered_project");

    let file_violations = project_files_in(layered.as_path())
        .in_file("src/api/mod.rs")
        .should()
        .have_name("wrong.rs")
        .check_with(&options)
        .expect("logged file rule should execute");
    let layer_violations = project_layers_in(layered.as_path())
        .layer("api")
        .defined_by_folder("src/api")
        .layer("application")
        .defined_by_folder("src/application")
        .layer("database")
        .defined_by_folder("src/database")
        .where_layer("api")
        .may_only_depend_on_layers(&["application", "database"])
        .check_with(&options)
        .expect("logged layer rule should execute");
    let slice_violations = project_slices_in(layered)
        .defined_by("src/(**)/")
        .should_not()
        .contain_dependency("database", "api")
        .check_with(&options)
        .expect("logged slice rule should execute");
    let metric_violations = metrics_in(fixture("metrics_project"))
        .with_name("domain.rs")
        .count()
        .lines_of_code()
        .should_be_below(1.0)
        .check_with(&options)
        .expect("logged metric rule should execute");

    assert_eq!(file_violations.len(), 1);
    assert!(layer_violations.is_empty());
    assert!(slice_violations.is_empty());
    assert_eq!(metric_violations.len(), 1);
    let content = fs::read_to_string(&log_path).expect("log artifact should be readable");
    for rule_name in [
        "files.pattern",
        "layers.dependencies",
        "slices.dependencies",
        "metrics.threshold",
    ] {
        assert!(content.contains(&format!("start check: {rule_name}")));
        assert!(content.contains(&format!("end check: {rule_name};")));
    }
    assert!(content.contains("[DEBUG] log progress: extracting project graph"));
    assert!(content.contains("[WARN] log violation: file-pattern"));
    assert!(content.contains("[WARN] log violation: metric-threshold"));
    assert!(content.contains("[DEBUG] log metric: lines_of_code [src/domain.rs]="));
    assert!(content.contains("; threshold=1"));
    assert!(
        content
            .lines()
            .all(|line| { line.starts_with('[') && line.contains("Z] [") && !line.contains('\r') })
    );

    let before_quiet_check = content;
    project_files_in(fixture("layered_project"))
        .in_file("src/api/mod.rs")
        .should()
        .have_name("api.rs")
        .check()
        .expect("quiet check should execute");
    assert_eq!(
        fs::read_to_string(&log_path).expect("log artifact should remain readable"),
        before_quiet_check
    );
}

#[test]
fn public_logger_exposes_the_complete_raw_and_specialized_vocabulary() {
    let output = TemporaryDirectory::new("vocabulary");
    let logging = LoggingOptions::new()
        .with_level(LogLevel::Debug)
        .with_console_output(false)
        .with_file_output(output.path());
    let path = logging
        .file_path()
        .expect("file logging should expose its path")
        .to_path_buf();
    let logger = CheckLogger::new(Some(&logging));

    logger.start_check("custom.rule").expect("start should log");
    logger.log_progress("step").expect("progress should log");
    logger
        .log_metric("score", "subject", 0.5, Some(0.75))
        .expect("metric should log");
    logger
        .log_violation("custom-kind")
        .expect("violation should log");
    logger.debug("debug record").expect("debug should log");
    logger.info("info record").expect("info should log");
    logger.warn("warn record").expect("warn should log");
    logger.error("error record").expect("error should log");
    logger.end_check("custom.rule", 1).expect("end should log");

    let content = fs::read_to_string(path).expect("log artifact should be readable");
    for event in [
        "start check",
        "log progress",
        "log metric",
        "log violation",
        "debug",
        "info",
        "warn",
        "error",
        "end check",
    ] {
        assert!(content.contains(event));
    }
}

#[test]
fn invalid_logging_configuration_precedes_rule_io_and_is_typed() {
    for level in [LogLevel::Debug, LogLevel::Error] {
        let no_sink = CheckOptions::new().with_logging(
            LoggingOptions::new()
                .with_level(level)
                .with_console_output(false),
        );
        let error = project_files_in("definitely/missing")
            .should()
            .have_no_cycles()
            .check_with(&no_sink)
            .expect_err("invalid logging should prevent rule execution");

        assert!(matches!(error, ArchUnitError::User(_)));
        assert!(error.to_string().contains("logging must enable"));
        assert!(!error.to_string().contains("Cargo"));
    }
}
