//! Framework-neutral architecture-test result formatting.

mod color_utils;
mod result_factory;
mod test_result;
mod test_result_options;
mod test_violation;
mod violation_factory;

pub use color_utils::{ColorChoice, ColorUtils};
pub use result_factory::ResultFactory;
pub use test_result::TestResult;
pub use test_result_options::TestResultOptions;
pub use test_violation::TestViolation;
pub use violation_factory::ViolationFactory;
