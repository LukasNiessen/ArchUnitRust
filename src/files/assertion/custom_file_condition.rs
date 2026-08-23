use crate::{FileInfo, Violation};

use super::CustomFileViolation;

/// A reusable, thread-safe question about one source file.
pub type FilePredicate = dyn Fn(&FileInfo) -> bool + Send + Sync + 'static;

/// Judges immutable file facts with one user-defined predicate.
///
/// The predicate is called exactly once per file. Positive rules report `false`; negated rules
/// report `true`. Empty selections remain the terminal empty-test guard's responsibility.
#[must_use]
pub fn gather_custom_file_violations<P>(
    file_infos: &[FileInfo],
    predicate: &P,
    message: &str,
    is_negated: bool,
) -> Vec<Violation>
where
    P: Fn(&FileInfo) -> bool + ?Sized,
{
    file_infos
        .iter()
        .filter(|file_info| predicate(file_info) == is_negated)
        .cloned()
        .map(|file_info| CustomFileViolation::new(file_info, message, is_negated))
        .map(Violation::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use crate::{FileInfo, ViolationKind};

    use super::gather_custom_file_violations;

    fn files() -> Vec<FileInfo> {
        vec![
            FileInfo::new("src/short.rs", "fn short() {}\n"),
            FileInfo::new(
                "src/long.rs",
                "fn first() {}\n\nfn second() {}\nfn third() {}\n",
            ),
        ]
    }

    #[test]
    fn positive_mood_reports_files_for_which_the_predicate_is_false() {
        let predicate = |file: &FileInfo| file.non_blank_line_count <= 2;

        let violations = gather_custom_file_violations(
            &files(),
            &predicate,
            "contain at most two non-blank lines",
            false,
        );
        let data = violations[0]
            .as_custom_file()
            .expect("fixture should produce custom-file data");

        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].kind(), ViolationKind::CustomFile);
        assert_eq!(data.file_info.path, "src/long.rs");
        assert_eq!(data.message, "contain at most two non-blank lines");
        assert!(!data.is_negated);
    }

    #[test]
    fn negated_mood_reports_files_for_which_the_predicate_is_true() {
        let predicate = |file: &FileInfo| file.content.contains("short");

        let violations = gather_custom_file_violations(&files(), &predicate, "contain short", true);
        let data = violations[0]
            .as_custom_file()
            .expect("fixture should produce custom-file data");

        assert_eq!(violations.len(), 1);
        assert_eq!(data.file_info.path, "src/short.rs");
        assert!(data.is_negated);
    }

    #[test]
    fn calls_the_predicate_exactly_once_per_file() {
        let calls = AtomicUsize::new(0);
        let predicate = |_: &FileInfo| {
            calls.fetch_add(1, Ordering::Relaxed);
            true
        };

        let violations = gather_custom_file_violations(&files(), &predicate, "always hold", false);

        assert!(violations.is_empty());
        assert_eq!(calls.load(Ordering::Relaxed), 2);
    }
}
