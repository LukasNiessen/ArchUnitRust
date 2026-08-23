//! Framework-neutral architecture-test result formatting.

mod color_utils;
mod test_result;
mod test_result_options;
mod test_violation;

pub use color_utils::{ColorChoice, ColorUtils};
pub use test_result::TestResult;
pub use test_result_options::TestResultOptions;
pub use test_violation::TestViolation;
