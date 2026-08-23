//! The execution contract shared by every terminal architecture rule.

mod check_options;
mod checkable;

pub use check_options::CheckOptions;
pub use checkable::{CheckResult, Checkable};
