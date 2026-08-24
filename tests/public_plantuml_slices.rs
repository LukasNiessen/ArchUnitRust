use std::{fs, path::PathBuf, time::SystemTime};

use archunit::{
    ArchUnitError, Checkable, PlantUmlDependency, PlantUmlDiagram, PlantUmlParser,
    PlantUmlRenderer, SliceDependencyRule, ViolationFactory, ViolationKind, project_slices_in,
};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

struct TemporaryDirectory(PathBuf);

impl TemporaryDirectory {
    fn new(test_name: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("system clock should follow the Unix epoch")
            .as_nanos();
        Self(std::env::temp_dir().join(format!(
            "archunit-public-plantuml-{}-{test_name}-{nonce}",
            std::process::id()
        )))
    }

    fn join(&self, path: &str) -> PathBuf {
        self.0.join(path)
    }
}

impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn complete_layered_diagram() -> &'static str {
    "@startuml\n\
       component [api]\n\
       component [application]\n\
       component [database]\n\
       component [support]\n\
       [api] --> [application]\n\
       [api] --> [database]\n\
       [api] --> [support]\n\
       [application] --> [database]\n\
     @enduml"
}

#[test]
fn parser_model_and_renderer_are_public_without_project_io() {
    let diagram =
        PlantUmlParser::parse("@startuml\ncomponent [api]\n[api] -> [application]\n@enduml")
            .expect("public parser should accept the supported subset");
    let constructed =
        PlantUmlDiagram::new(
            ["api".to_owned()],
            [PlantUmlDependency::new("api", "application")
                .expect("public dependency should be valid")],
        )
        .expect("public diagram should be valid");

    assert_eq!(diagram, constructed);
    assert!(diagram.allows("api", "application"));
    assert_eq!(
        PlantUmlRenderer::render(&[]).expect("empty edge set should render"),
        "@startuml\n@enduml\n"
    );
}

#[test]
fn strict_diagram_reports_disallowed_internal_dependency_with_rust_evidence() {
    let diagram = complete_layered_diagram().replace("[api] --> [database]\n", "");
    let violations = project_slices_in(fixture("layered_project"))
        .defined_by("src/(**)/")
        .should()
        .adhere_to_diagram(diagram)
        .check()
        .expect("the layered project and inline diagram should be analyzable");

    assert_eq!(violations.len(), 1);
    let violation = violations[0]
        .as_slice_dependency()
        .expect("diagram disagreement should be slice dependency data");
    assert_eq!(violation.source_slice, "api");
    assert_eq!(violation.target_slice, "database");
    assert_eq!(violation.rule, SliceDependencyRule::AdhereToDiagram);
    assert!(!violation.is_negated);
    assert_eq!(violation.dependency.cumulated_edges.len(), 1);
    assert_eq!(
        ViolationFactory::from_violation(&violations[0]).details,
        "Slice 'api' depends on slice 'database', which is not allowed by the architecture diagram. Evidence: src/api/mod.rs -> src/database/repository.rs [use]."
    );
}

#[test]
fn orphan_modifier_ignores_only_dependencies_outside_declared_components() {
    let narrow = "@startuml\n\
                  component [api]\n\
                  component [application]\n\
                  [api] --> [application]\n\
                  @enduml";
    let rule = project_slices_in(fixture("layered_project"))
        .defined_by("src/(**)/")
        .should()
        .ignoring_orphan_slices()
        .adhere_to_diagram(narrow);

    assert!(
        rule.check()
            .expect("orphan-ignoring diagram should be analyzable")
            .is_empty()
    );
}

#[test]
fn external_modifier_hides_cargo_modules_without_hiding_internal_edges() {
    let diagram = "@startuml\n\
                   component [app]\n\
                   component [macros]\n\
                   [app] --> [macros]\n\
                   @enduml";
    let scope = project_slices_in(fixture("extraction_workspace"))
        .defined_by("crates/(**)/")
        .should();
    let strict = scope.clone().adhere_to_diagram(diagram);
    let external = scope.ignoring_external_slices().adhere_to_diagram(diagram);

    let strict = strict
        .check()
        .expect("strict external diagram should be analyzable");
    let external = external
        .check()
        .expect("external-ignoring diagram should be analyzable");

    assert!(strict.iter().any(|violation| {
        violation
            .as_slice_dependency()
            .is_some_and(|data| data.target_slice == "tokio")
    }));
    assert!(external.is_empty());
}

#[test]
fn file_source_is_lazy_and_reverse_generation_exports_byte_identical_utf8() {
    let temporary = TemporaryDirectory::new("file-and-export");
    let diagram_path = temporary.join("input/architecture.puml");
    let output_path = temporary.join("output/actual.puml");
    let scope = project_slices_in(fixture("layered_project")).defined_by("src/(**)/");
    let rule = scope
        .clone()
        .should()
        .adhere_to_diagram_in_file(&diagram_path);

    assert!(matches!(rule.check(), Err(ArchUnitError::Technical(_))));
    fs::create_dir_all(
        diagram_path
            .parent()
            .expect("fixture input should have a parent"),
    )
    .expect("fixture input directory should be creatable");
    fs::write(&diagram_path, complete_layered_diagram())
        .expect("fixture diagram should be writable");
    assert!(
        rule.check()
            .expect("file-backed diagram should be analyzable after creation")
            .is_empty()
    );

    let generated = scope
        .to_plantuml()
        .expect("actual slice diagram generation should succeed");
    scope
        .export_as_plantuml(&output_path)
        .expect("actual slice diagram export should succeed");

    assert_eq!(
        generated,
        "@startuml\n  component [api]\n  component [application]\n  component [database]\n  component [support]\n  [api] --> [application]\n  [api] --> [database]\n  [api] --> [support]\n  [application] --> [database]\n@enduml\n"
    );
    assert_eq!(
        fs::read_to_string(output_path).expect("export should be readable as UTF-8"),
        generated
    );
}

#[test]
fn invalid_diagram_is_a_user_error_and_empty_scope_keeps_guard_priority() {
    let invalid = project_slices_in(fixture("layered_project"))
        .defined_by("src/(**)/")
        .should()
        .adhere_to_diagram("@startuml\n[api] -- [database]\n@enduml");
    let empty = project_slices_in(fixture("layered_project"))
        .defined_by("missing/(**)/")
        .should()
        .adhere_to_diagram("not parsed because selection is empty");

    assert!(matches!(invalid.check(), Err(ArchUnitError::User(_))));
    let empty = empty
        .check()
        .expect("empty selection should produce a verdict before diagram parsing");
    assert_eq!(empty.len(), 1);
    assert_eq!(empty[0].kind(), ViolationKind::EmptyTest);
    assert!(
        !empty[0]
            .as_empty_test()
            .expect("empty data should exist")
            .is_negated
    );
}
