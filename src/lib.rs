#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

mod common;

pub use common::assertion::{EmptyTestViolation, Violation, ViolationKind};
pub use common::error::{ArchUnitError, TechnicalError, UserError};
pub use common::extraction::{Edge, Graph, ImportKind, ImportKindSet};
pub use common::matching::{
    Filter, Pattern, PatternError, PatternOptions, PatternSyntax, PatternTarget, RegexFactory,
    RegexFactoryOptions,
};
