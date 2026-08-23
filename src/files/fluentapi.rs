//! Sentence-like entry points and builders for file architecture rules.

mod cycle_free_file_condition;
mod file_condition_builder;
mod files;
mod match_pattern_file_condition;
mod match_pattern_file_condition_builder;
mod negated_match_pattern_file_condition_builder;
mod positive_match_pattern_file_condition_builder;

pub use cycle_free_file_condition::CycleFreeFileCondition;
pub use file_condition_builder::FileConditionBuilder;
pub use files::{files, files_in, project_files, project_files_in};
pub use match_pattern_file_condition::MatchPatternFileCondition;
pub use match_pattern_file_condition_builder::MatchPatternFileConditionBuilder;
pub use negated_match_pattern_file_condition_builder::NegatedMatchPatternFileConditionBuilder;
pub use positive_match_pattern_file_condition_builder::PositiveMatchPatternFileConditionBuilder;
