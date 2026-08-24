use std::path::PathBuf;

use archunit::{CheckOptions, MetricSubject, TypeKind, metrics_in};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

#[test]
fn project_analysis_associates_impls_and_preserves_field_access_evidence() {
    let project = metrics_in(fixture("metrics_project"))
        .analyze()
        .expect("the metrics fixture should be analyzable");

    assert_eq!(project.files().len(), 3);
    assert_eq!(project.types().len(), 5);
    let service = project
        .types()
        .iter()
        .find(|type_info| type_info.name() == "Service")
        .expect("Service should be extracted");
    assert_eq!(service.kind(), TypeKind::Struct);
    assert_eq!(service.methods().len(), 4);
    assert_eq!(service.associated_functions(), &["make", "new"]);
    assert_eq!(service.fields()[0].name(), "repository");
    assert_eq!(service.fields()[0].accessed_by(), &["execute"]);
    assert_eq!(service.fields()[1].name(), "requests");
    assert_eq!(
        service.fields()[1].accessed_by(),
        &["execute", "increment", "reset", "send"]
    );

    let state = project
        .types()
        .iter()
        .find(|type_info| type_info.name() == "State")
        .expect("State should be extracted");
    assert_eq!(state.kind(), TypeKind::Enum);
    assert_eq!(state.fields().len(), 2);
    assert_eq!(state.methods().len(), 1);
}

#[test]
fn type_selectors_and_type_count_metrics_use_rust_vocabulary() {
    let methods = metrics_in(fixture("metrics_project"))
        .in_path("src/domain.rs")
        .for_types_matching("*Service")
        .count()
        .method_count()
        .measure()
        .expect("method count should succeed");
    let fields = metrics_in(fixture("metrics_project"))
        .in_folder("src")
        .with_name("domain.rs")
        .for_types_matching("Service")
        .count()
        .field_count()
        .measure()
        .expect("field count should succeed");

    assert_eq!(methods.len(), 1);
    assert_eq!(methods[0].identifier(), "Service");
    assert_eq!(methods[0].metric_name(), "method_count");
    assert_eq!(methods[0].value(), 4.0);
    assert!(matches!(methods[0].subject(), MetricSubject::Type(_)));
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].value(), 2.0);
}

#[test]
fn every_file_count_has_explicit_rust_semantics() {
    let root = fixture("metrics_project");
    let measure = |metric: &str| {
        let selection = metrics_in(root.as_path()).with_name("domain.rs").count();
        match metric {
            "lines" => selection.lines_of_code().measure(),
            "statements" => selection.statements().measure(),
            "imports" => selection.imports().measure(),
            "types" => selection.concrete_types().measure(),
            "functions" => selection.functions().measure(),
            "traits" => selection.traits().measure(),
            "impls" => selection.impl_blocks().measure(),
            "macros" => selection.macros().measure(),
            "associated" => selection.associated_functions().measure(),
            _ => unreachable!("the test supplies only known metrics"),
        }
        .expect("file count should succeed")[0]
            .value()
    };

    assert!(measure("lines") > 40.0);
    assert!(measure("statements") > 20.0);
    assert_eq!(measure("imports"), 1.0);
    assert_eq!(measure("types"), 4.0);
    assert_eq!(measure("functions"), 0.0);
    assert_eq!(measure("traits"), 1.0);
    assert_eq!(measure("impls"), 4.0);
    assert_eq!(measure("macros"), 3.0);
    assert_eq!(measure("associated"), 3.0);
}

#[test]
fn type_filtering_updates_type_populations_but_preserves_file_properties() {
    let root = fixture("metrics_project");
    let full = metrics_in(root.as_path())
        .with_name("domain.rs")
        .analyze()
        .expect("full analysis should succeed");
    let selected = metrics_in(root.as_path())
        .with_name("domain.rs")
        .for_types_matching("*Service")
        .analyze()
        .expect("selected analysis should succeed");

    assert_eq!(selected.files().len(), 1);
    assert_eq!(selected.types().len(), 1);
    assert_eq!(selected.files()[0].concrete_types(), 1);
    assert_eq!(selected.files()[0].traits(), 0);
    assert_eq!(
        selected.files()[0].lines_of_code(),
        full.files()[0].lines_of_code()
    );
    assert_eq!(selected.files()[0].imports(), full.files()[0].imports());
}

#[test]
fn dev_target_sources_are_opt_in_and_empty_measurements_are_data() {
    let root = fixture("metrics_project");
    let default = metrics_in(root.as_path())
        .with_name("architecture.rs")
        .count()
        .method_count()
        .measure()
        .expect("empty production selection should remain valid data");
    let inclusive = metrics_in(root.as_path())
        .with_name("architecture.rs")
        .count()
        .method_count()
        .measure_with(&CheckOptions::new().with_test_sources(true))
        .expect("dev-source analysis should succeed");

    assert!(default.is_empty());
    assert_eq!(inclusive.len(), 1);
    assert_eq!(inclusive[0].identifier(), "TestOnlyType");
    assert_eq!(inclusive[0].value(), 1.0);
}
