//! Immutable projections from extracted files to named architectural slices.

mod slice_projection;

pub use slice_projection::{
    SliceProjection, SliceProjectionError, slice_by_file_suffix, slice_by_pattern, slice_by_regex,
    slice_identity,
};
