pub(crate) mod assertion;
pub(crate) mod extraction;
pub(crate) mod fluentapi;

pub use assertion::{
    CustomFileViolation, CycleViolation, ExternalModuleDependencyViolation,
    FileDependencyViolation, FilePatternViolation, FilePredicate, gather_custom_file_violations,
    gather_cycle_violations, gather_external_module_dependency_violations,
    gather_file_dependency_violations, gather_matching_file_violations,
};
pub use extraction::FileInfo;
pub use fluentapi::{
    CustomFileCondition, CycleFreeFileCondition, DependOnExternalModuleCondition,
    DependOnExternalModuleConditionBuilder, DependOnFileCondition, DependOnFileConditionBuilder,
    FileConditionBuilder, MatchPatternFileCondition, MatchPatternFileConditionBuilder,
    NegatedMatchPatternFileConditionBuilder, PositiveMatchPatternFileConditionBuilder, files,
    files_in, project_files, project_files_in,
};
