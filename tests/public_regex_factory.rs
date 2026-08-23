use archunit::{PatternSyntax, RegexFactory, RegexFactoryOptions};

#[test]
fn factory_matchers_form_reusable_public_selectors() {
    let factory = RegexFactory::default();
    let rust_files = factory
        .filename_matcher("*.rs")
        .expect("fixture glob should compile");
    let production = factory
        .folder_matcher("crates/**/src/**")
        .expect("fixture glob should compile");

    let selected = [
        "crates/api/src/lib.rs",
        "crates/api/src/handler.rs",
        "crates/api/tests/handler_test.rs",
        "README.md",
    ]
    .into_iter()
    .filter(|identifier| rust_files.matches(identifier) && production.matches(identifier))
    .collect::<Vec<_>>();

    assert_eq!(
        selected,
        vec!["crates/api/src/lib.rs", "crates/api/src/handler.rs"]
    );
}

#[test]
fn regex_and_exact_factories_keep_distinct_semantics() {
    let factory = RegexFactory::new(RegexFactoryOptions::new().syntax(PatternSyntax::Regex));
    let versioned = factory
        .filename_matcher(r"handler_v[0-9]+\.rs")
        .expect("fixture regular expression should compile");
    let exact = factory
        .exact_file_matcher("src/handler_v[0-9]+.rs")
        .expect("fixture literal should compile");

    assert!(versioned.matches("src/handler_v12.rs"));
    assert!(!versioned.matches("src/handler_v[0-9]+.rs"));
    assert!(exact.matches("src/handler_v[0-9]+.rs"));
    assert!(!exact.matches("src/handler_v12.rs"));
}
