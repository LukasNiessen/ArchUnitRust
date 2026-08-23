use archunit::{Filter, Pattern, PatternOptions, PatternTarget};

#[test]
fn public_filters_match_normalized_project_identifiers() {
    let production_rust = Filter::new(
        Pattern::glob("crates/**/src/**/*.rs").expect("fixture glob should compile"),
        PatternTarget::Path,
    );
    let handlers = Filter::new(
        Pattern::regex_with(
            r"[a-z]+_handler\.rs",
            PatternOptions::new().case_insensitive(true),
        )
        .expect("fixture regular expression should compile"),
        PatternTarget::Filename,
    );

    assert!(production_rust.matches(r"crates\api\src\http\request_handler.rs"));
    assert!(!production_rust.matches("crates/api/tests/request_handler.rs"));
    assert!(handlers.matches("crates/api/src/HTTP_HANDLER.RS"));
    assert!(!handlers.matches("crates/api/src/http_router.rs"));
}
