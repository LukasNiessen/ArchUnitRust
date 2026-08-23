use archunit::{EmptyTestViolation, RegexFactory, Violation, ViolationKind};

#[test]
fn violation_results_are_data_accessible_from_the_public_surface() {
    let selector = RegexFactory::default()
        .folder_matcher("src/apis/**")
        .expect("fixture glob should compile");
    let violations = [Violation::from(EmptyTestViolation::new(
        "files",
        [selector],
    ))];

    assert_eq!(violations[0].kind(), ViolationKind::EmptyTest);
    let empty = violations[0]
        .as_empty_test()
        .expect("fixture should be an empty-test violation");
    assert_eq!(empty.subject, "files");
    assert_eq!(empty.selectors[0].pattern().source(), "src/apis/**");
}

#[test]
fn an_empty_violation_list_is_the_complete_pass_result() {
    let result: Vec<Violation> = Vec::new();

    assert!(result.is_empty());
}
