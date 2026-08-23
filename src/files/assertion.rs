//! Pure assertions for file architecture rules.

mod custom_file_condition;
mod custom_file_violation;
mod cycle_free;
mod cycle_violation;
mod depend_on_external_modules;
mod depend_on_files;
mod external_module_dependency_violation;
mod file_dependency_violation;
mod file_pattern_violation;
mod matching_files;

pub use custom_file_condition::{FilePredicate, gather_custom_file_violations};
pub use custom_file_violation::CustomFileViolation;
pub use cycle_free::gather_cycle_violations;
pub use cycle_violation::CycleViolation;
pub use depend_on_external_modules::gather_external_module_dependency_violations;
pub use depend_on_files::gather_file_dependency_violations;
pub use external_module_dependency_violation::ExternalModuleDependencyViolation;
pub use file_dependency_violation::FileDependencyViolation;
pub use file_pattern_violation::FilePatternViolation;
pub use matching_files::gather_matching_file_violations;
