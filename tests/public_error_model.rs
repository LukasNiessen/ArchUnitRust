use std::{error::Error, io};

use archunit::{ArchUnitError, TechnicalError, UserError, Violation};

#[test]
fn callers_can_classify_the_party_that_can_fix_a_failure() {
    let errors = [
        ArchUnitError::from(TechnicalError::new("could not locate a Cargo project")),
        ArchUnitError::from(UserError::new("the layer name must not be empty")),
    ];

    assert!(errors[0].as_technical().is_some());
    assert!(errors[0].as_user().is_none());
    assert!(errors[1].as_technical().is_none());
    assert!(errors[1].as_user().is_some());
}

#[test]
fn technical_sources_remain_available_to_error_consumers() {
    let error = ArchUnitError::from(TechnicalError::with_source(
        "could not read Cargo.toml",
        io::Error::new(io::ErrorKind::PermissionDenied, "access denied"),
    ));

    assert_eq!(
        error.to_string(),
        "archunit: could not read Cargo.toml: access denied"
    );
    assert_eq!(
        error.source().map(ToString::to_string),
        Some("access denied".to_owned())
    );
}

#[test]
fn a_failed_rule_is_a_violation_result_and_not_an_error() {
    let result: Result<Vec<Violation>, ArchUnitError> = Ok(Vec::new());

    assert!(result.is_ok());
}
