//! Pure assertions for file architecture rules.

mod cycle_free;
mod cycle_violation;
mod depend_on_files;
mod file_dependency_violation;
mod file_pattern_violation;
mod matching_files;

pub use cycle_free::gather_cycle_violations;
pub use cycle_violation::CycleViolation;
pub use depend_on_files::gather_file_dependency_violations;
pub use file_dependency_violation::FileDependencyViolation;
pub use file_pattern_violation::FilePatternViolation;
pub use matching_files::gather_matching_file_violations;
