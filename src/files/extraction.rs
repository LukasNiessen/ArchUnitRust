//! Source-file facts used by custom file predicates.

mod extract_file_info;
mod file_info;

pub(crate) use extract_file_info::extract_file_info;
pub use file_info::FileInfo;
