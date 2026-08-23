#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

mod common;

pub use common::extraction::{Edge, Graph, ImportKind, ImportKindSet};
pub use common::matching::{
    Filter, Pattern, PatternError, PatternOptions, PatternSyntax, PatternTarget,
};
