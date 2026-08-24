use std::{
    fmt,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{ArchUnitError, LogLevel, LogRecord, TechnicalError, UserError};

/// Initialization policy for an existing per-options log file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum LogFileMode {
    /// Retain existing content and add this logging session after it.
    Append,
    /// Truncate existing content exactly once before this logging session's first record.
    Overwrite,
}

#[derive(Debug, Default)]
struct FileRuntime {
    initialized: bool,
}

#[derive(Debug, Clone)]
struct FileOutput {
    path: PathBuf,
    runtime: Arc<Mutex<FileRuntime>>,
}

/// Immutable per-check logging configuration.
///
/// Logging is enabled only when this value is supplied through
/// [`crate::CheckOptions::with_logging`]. Clones share only their file-initialization lock so the
/// same explicit configuration remains safe to use from concurrent checks; no global logger or
/// ambient configuration is read.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct LoggingOptions {
    level: LogLevel,
    console_output: bool,
    file_mode: LogFileMode,
    file_output: Option<FileOutput>,
}

impl LoggingOptions {
    /// Creates info-level console logging without file output.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            level: LogLevel::Info,
            console_output: true,
            file_mode: LogFileMode::Append,
            file_output: None,
        }
    }

    /// Sets the minimum emitted severity.
    #[must_use]
    pub const fn with_level(mut self, level: LogLevel) -> Self {
        self.level = level;
        self
    }

    /// Enables or disables console output for this configuration.
    #[must_use]
    pub const fn with_console_output(mut self, enabled: bool) -> Self {
        self.console_output = enabled;
        self
    }

    /// Enables file output under `directory` with an automatically timestamped `.log` filename.
    #[must_use]
    pub fn with_file_output(mut self, directory: impl AsRef<Path>) -> Self {
        let path = directory.as_ref().join(timestamped_log_filename());
        self.file_output = Some(FileOutput {
            path,
            runtime: Arc::new(Mutex::new(FileRuntime::default())),
        });
        self
    }

    /// Selects append or one-time overwrite initialization for file output.
    #[must_use]
    pub fn with_file_mode(mut self, mode: LogFileMode) -> Self {
        self.file_mode = mode;
        if let Some(output) = &mut self.file_output {
            output.runtime = Arc::new(Mutex::new(FileRuntime::default()));
        }
        self
    }

    /// Returns the minimum emitted severity.
    #[must_use]
    pub const fn level(&self) -> LogLevel {
        self.level
    }

    /// Returns whether records are written to the console.
    #[must_use]
    pub const fn logs_to_console(&self) -> bool {
        self.console_output
    }

    /// Returns the timestamped log path, or `None` when file output is disabled.
    #[must_use]
    pub fn file_path(&self) -> Option<&Path> {
        self.file_output
            .as_ref()
            .map(|output| output.path.as_path())
    }

    /// Returns the file initialization policy.
    #[must_use]
    pub const fn file_mode(&self) -> LogFileMode {
        self.file_mode
    }

    pub(super) fn validate(&self) -> Result<(), ArchUnitError> {
        if !self.console_output && self.file_output.is_none() {
            return Err(ArchUnitError::from(UserError::new(
                "logging must enable console output, file output, or both",
            )));
        }
        let Some(path) = self.file_path() else {
            return Ok(());
        };
        if path
            .parent()
            .is_none_or(|parent| parent.as_os_str().is_empty())
        {
            return Err(ArchUnitError::from(UserError::new(
                "the logging output directory must not be empty",
            )));
        }
        Ok(())
    }

    pub(super) fn accepts(&self, level: LogLevel) -> bool {
        level >= self.level
    }

    pub(super) fn write(&self, record: &LogRecord) -> Result<(), ArchUnitError> {
        self.validate()?;
        if self.console_output {
            write_console(record)?;
        }
        if let Some(output) = &self.file_output {
            write_file(output, self.file_mode, record)?;
        }
        Ok(())
    }
}

impl Default for LoggingOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for LoggingOptions {
    fn eq(&self, other: &Self) -> bool {
        self.level == other.level
            && self.console_output == other.console_output
            && self.file_mode == other.file_mode
            && self.file_path() == other.file_path()
    }
}

impl Eq for LoggingOptions {}

fn write_console(record: &LogRecord) -> Result<(), ArchUnitError> {
    let rendered = format!("{}\n", record.render());
    let result = match record.level() {
        LogLevel::Warn | LogLevel::Error => std::io::stderr().lock().write_all(rendered.as_bytes()),
        LogLevel::Debug | LogLevel::Info => std::io::stdout().lock().write_all(rendered.as_bytes()),
    };
    result.map_err(|source| {
        ArchUnitError::from(TechnicalError::with_source(
            "could not write an architecture log record to the console",
            source,
        ))
    })
}

fn write_file(
    output: &FileOutput,
    mode: LogFileMode,
    record: &LogRecord,
) -> Result<(), ArchUnitError> {
    let mut runtime = output.runtime.lock().map_err(|source| {
        ArchUnitError::from(TechnicalError::new(format!(
            "could not lock logging output {}: {source}",
            output.path.display()
        )))
    })?;
    let parent = output.path.parent().ok_or_else(|| {
        ArchUnitError::from(UserError::new(
            "the logging output directory must not be empty",
        ))
    })?;
    fs::create_dir_all(parent).map_err(|source| {
        ArchUnitError::from(TechnicalError::with_source(
            format!("could not create logging directory {}", parent.display()),
            source,
        ))
    })?;

    if !runtime.initialized {
        initialize_file(&output.path, mode)?;
        runtime.initialized = true;
    }
    let timestamp = current_utc_timestamp()?;
    let line = format!("[{timestamp}] {}\n", record.render());
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(&output.path)
        .and_then(|mut file| file.write_all(line.as_bytes()))
        .map_err(|source| {
            ArchUnitError::from(TechnicalError::with_source(
                format!("could not write architecture log {}", output.path.display()),
                source,
            ))
        })
}

fn initialize_file(path: &Path, mode: LogFileMode) -> Result<(), ArchUnitError> {
    let mut options = OpenOptions::new();
    options.create(true).write(true);
    match mode {
        LogFileMode::Append => {
            options.append(true);
        }
        LogFileMode::Overwrite => {
            options.truncate(true);
        }
    }
    options.open(path).map(|_| ()).map_err(|source| {
        ArchUnitError::from(TechnicalError::with_source(
            format!("could not initialize architecture log {}", path.display()),
            source,
        ))
    })
}

fn timestamped_log_filename() -> String {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let timestamp = format_utc_timestamp(duration.as_secs());
    let compact = timestamp.replace(['-', ':'], "").replace('Z', "");
    format!(
        "archunit-{compact}-{:09}Z-p{}.log",
        duration.subsec_nanos(),
        process::id()
    )
}

fn current_utc_timestamp() -> Result<String, ArchUnitError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| format_utc_timestamp(duration.as_secs()))
        .map_err(|source| {
            ArchUnitError::from(TechnicalError::with_source(
                "could not obtain a UTC architecture log timestamp",
                source,
            ))
        })
}

fn format_utc_timestamp(seconds: u64) -> String {
    let days = (seconds / 86_400) as i64;
    let day_seconds = seconds % 86_400;
    let (year, month, day) = civil_date_from_unix_days(days);
    let hour = day_seconds / 3_600;
    let minute = day_seconds % 3_600 / 60;
    let second = day_seconds % 60;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

fn civil_date_from_unix_days(days: i64) -> (i64, i64, i64) {
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_part = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_part + 2) / 5 + 1;
    let month = month_part + if month_part < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    (year, month, day)
}

impl fmt::Display for LogFileMode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Append => "append",
            Self::Overwrite => "overwrite",
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, process, thread, time::SystemTime};

    use super::{LogFileMode, LoggingOptions};
    use crate::{CheckLogger, LogLevel};

    fn temporary_directory(label: &str) -> std::path::PathBuf {
        let nonce = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("test clock should follow the Unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "archunit-logging-{label}-{}-{nonce}",
            process::id()
        ))
    }

    #[test]
    fn defaults_and_consuming_modifiers_are_branchable() {
        let directory = temporary_directory("options");
        let base = LoggingOptions::new();
        let configured = base
            .clone()
            .with_level(LogLevel::Debug)
            .with_console_output(false)
            .with_file_output(&directory)
            .with_file_mode(LogFileMode::Overwrite);

        assert_eq!(base.level(), LogLevel::Info);
        assert!(base.logs_to_console());
        assert!(base.file_path().is_none());
        assert_eq!(base.file_mode(), LogFileMode::Append);
        assert_eq!(configured.level(), LogLevel::Debug);
        assert!(!configured.logs_to_console());
        assert_eq!(configured.file_mode(), LogFileMode::Overwrite);
        let path = configured
            .file_path()
            .expect("file output should have a path");
        assert_eq!(path.parent(), Some(directory.as_path()));
        assert_eq!(
            path.extension().and_then(|value| value.to_str()),
            Some("log")
        );
        assert!(path.file_name().is_some_and(|name| {
            let name = name.to_string_lossy();
            name.starts_with("archunit-") && !name.contains(':')
        }));
    }

    #[test]
    fn file_output_creates_parents_and_filters_levels() {
        let root = temporary_directory("file");
        let directory = root.join("nested");
        let options = LoggingOptions::new()
            .with_level(LogLevel::Warn)
            .with_console_output(false)
            .with_file_output(&directory);
        let path = options
            .file_path()
            .expect("file output should expose its path")
            .to_path_buf();
        let logger = CheckLogger::new(Some(&options));

        logger
            .start_check("files.pattern")
            .expect("start should filter");
        logger
            .log_progress("extracting")
            .expect("progress should filter");
        logger
            .log_violation("file-pattern")
            .expect("violation should write");
        logger
            .end_check("files.pattern", 1)
            .expect("warning end should write");

        let content = fs::read_to_string(&path).expect("log should be readable");
        assert!(!content.contains("start check"));
        assert!(!content.contains("log progress"));
        assert!(content.contains("[WARN] log violation: file-pattern"));
        assert!(content.contains("[WARN] end check: files.pattern; violations=1"));
        fs::remove_dir_all(root).expect("temporary logging tree should be removable");
    }

    #[test]
    fn append_and_overwrite_modes_initialize_existing_files_once() {
        for (mode, old_content_survives) in
            [(LogFileMode::Append, true), (LogFileMode::Overwrite, false)]
        {
            let directory = temporary_directory(&format!("mode-{mode}"));
            let options = LoggingOptions::new()
                .with_console_output(false)
                .with_file_output(&directory)
                .with_file_mode(mode);
            let path = options
                .file_path()
                .expect("file output should expose its path")
                .to_path_buf();
            fs::create_dir_all(&directory).expect("fixture directory should be creatable");
            fs::write(&path, "existing\n").expect("fixture log should be writable");
            let logger = CheckLogger::new(Some(&options));

            logger.info("first").expect("first record should write");
            logger.info("second").expect("second record should write");

            let content = fs::read_to_string(&path).expect("log should be readable");
            assert_eq!(content.contains("existing"), old_content_survives);
            assert_eq!(content.matches("[INFO] info:").count(), 2);
            fs::remove_dir_all(directory).expect("temporary logging tree should be removable");
        }
    }

    #[test]
    fn invalid_sink_configuration_is_user_input_and_io_failure_is_technical() {
        let no_sink = LoggingOptions::new().with_console_output(false);
        let empty_directory = LoggingOptions::new()
            .with_console_output(false)
            .with_file_output("");
        let root = temporary_directory("io-error");
        fs::create_dir_all(&root).expect("fixture root should be creatable");
        let occupied = root.join("occupied");
        fs::write(&occupied, "not a directory").expect("fixture file should be writable");
        let impossible = LoggingOptions::new()
            .with_console_output(false)
            .with_file_output(&occupied);

        let no_sink_error = CheckLogger::new(Some(&no_sink))
            .info("record")
            .expect_err("missing sinks should be rejected");
        let empty_error = CheckLogger::new(Some(&empty_directory))
            .info("record")
            .expect_err("empty directory should be rejected");
        let io_error = CheckLogger::new(Some(&impossible))
            .info("record")
            .expect_err("filesystem conflict should be technical");

        assert!(matches!(no_sink_error, crate::ArchUnitError::User(_)));
        assert!(matches!(empty_error, crate::ArchUnitError::User(_)));
        assert!(matches!(io_error, crate::ArchUnitError::Technical(_)));
        fs::remove_dir_all(root).expect("temporary logging tree should be removable");
    }

    #[test]
    fn shared_options_serialize_concurrent_append_records() {
        let directory = temporary_directory("threads");
        let options = LoggingOptions::new()
            .with_console_output(false)
            .with_file_output(&directory)
            .with_file_mode(LogFileMode::Overwrite);
        let path = options
            .file_path()
            .expect("file output should expose its path")
            .to_path_buf();
        let handles = (0..16)
            .map(|index| {
                let options = options.clone();
                thread::spawn(move || {
                    CheckLogger::new(Some(&options))
                        .info(format!("record-{index}"))
                        .expect("concurrent record should write");
                })
            })
            .collect::<Vec<_>>();

        for handle in handles {
            handle.join().expect("logging thread should finish");
        }
        let content = fs::read_to_string(&path).expect("log should be readable");
        assert_eq!(content.lines().count(), 16);
        for index in 0..16 {
            assert!(content.contains(&format!("[INFO] info: record-{index}")));
        }
        fs::remove_dir_all(directory).expect("temporary logging tree should be removable");
    }
}
