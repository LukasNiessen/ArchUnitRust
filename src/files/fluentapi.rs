//! Sentence-like entry points and builders for file architecture rules.

mod file_condition_builder;
mod files;

pub use file_condition_builder::FileConditionBuilder;
pub use files::{files, files_in, project_files, project_files_in};
