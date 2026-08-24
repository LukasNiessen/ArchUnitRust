use crate::{ArchUnitError, SliceProjectionError, UserError};

#[derive(Debug, Clone)]
pub(crate) enum SliceConfigurationError {
    InvalidProjection {
        context: &'static str,
        source: SliceProjectionError,
    },
    EmptySourceSlice,
    EmptyTargetSlice,
}

impl SliceConfigurationError {
    pub(super) fn to_archunit_error(&self) -> ArchUnitError {
        let error = match self {
            Self::InvalidProjection { context, source } => UserError::with_source(
                format!("the {context} slice definition is invalid"),
                source.clone(),
            ),
            Self::EmptySourceSlice => UserError::new("source slice name must not be empty"),
            Self::EmptyTargetSlice => UserError::new("target slice name must not be empty"),
        };
        error.into()
    }
}
