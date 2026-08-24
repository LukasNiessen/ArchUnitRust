use std::path::PathBuf;

use archunit::{CheckOptions, LcomInput, MetricSubject, TypeKind, metrics_in};

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
    assert_eq!(
        project
            .files()
            .iter()
            .map(|file| file.path())
            .collect::<Vec<_>>(),
        ["src/domain.rs", "src/extensions.rs", "src/lib.rs"]
    );
    assert_eq!(
        project
            .types()
            .iter()
            .map(|type_info| type_info.name())
            .collect::<Vec<_>>(),
        ["Port", "Repository", "Service", "State", "Word"]
    );
    let service = project
        .types()
        .iter()
        .find(|type_info| type_info.name() == "Service")
        .expect("Service should be extracted");
    assert_eq!(service.kind(), TypeKind::Struct);
    assert_eq!(service.methods().len(), 4);
    assert_eq!(
        service
            .inherent_methods()
            .iter()
            .map(|method| method.name())
            .collect::<Vec<_>>(),
        ["execute", "increment", "reset"]
    );
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

#[test]
fn lcom_family_measures_inherent_struct_behavior_with_exact_formulas() {
    let root = fixture("metrics_project");
    let measure = |metric: &str| {
        let family = metrics_in(root.as_path())
            .for_types_matching("Service")
            .lcom();
        match metric {
            "lcom96a" => family.lcom96a().measure(),
            "lcom96b" => family.lcom96b().measure(),
            "lcom1" => family.lcom1().measure(),
            "lcom2" => family.lcom2().measure(),
            "lcom3" => family.lcom3().measure(),
            "lcom4" => family.lcom4().measure(),
            "lcom5" => family.lcom5().measure(),
            "lcom_star" => family.lcom_star().measure(),
            _ => unreachable!("the test supplies only known LCOM metrics"),
        }
        .expect("LCOM measurement should succeed")
    };
    let cases = [
        ("lcom96a", 0.5),
        ("lcom96b", 1.0 / 3.0),
        ("lcom1", 0.0),
        ("lcom2", 1.0 / 3.0),
        ("lcom3", 0.5),
        ("lcom4", 1.0),
        ("lcom5", 0.5),
        ("lcom_star", 0.5),
    ];

    for (name, expected) in cases {
        let measurements = measure(name);
        assert_eq!(measurements.len(), 1);
        assert_eq!(measurements[0].identifier(), "Service");
        assert_eq!(measurements[0].metric_name(), name);
        assert!(measurements[0].subject().as_type().is_some());
        assert!(
            (measurements[0].value() - expected).abs() < 0.000_001,
            "unexpected {name} value"
        );
    }
}

#[test]
fn lcom_population_excludes_non_struct_and_non_inherent_behavior() {
    let project = metrics_in(fixture("metrics_project"))
        .analyze()
        .expect("the metrics fixture should be analyzable");
    let eligibility = project
        .types()
        .iter()
        .map(|type_info| {
            (
                type_info.name(),
                LcomInput::from_type_info(type_info).is_some(),
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(
        eligibility,
        [
            ("Port", false),
            ("Repository", true),
            ("Service", true),
            ("State", false),
            ("Word", false),
        ]
    );
    let measurements = metrics_in(fixture("metrics_project"))
        .lcom()
        .lcom4()
        .measure()
        .expect("LCOM4 measurement should succeed");
    assert_eq!(
        measurements
            .iter()
            .map(|measurement| (measurement.identifier(), measurement.value()))
            .collect::<Vec<_>>(),
        [("Repository", 1.0), ("Service", 1.0)]
    );
}

#[test]
fn lcom_dev_sources_are_opt_in_and_empty_selections_remain_data() {
    let root = fixture("metrics_project");
    let selection = metrics_in(root.as_path())
        .with_name("architecture.rs")
        .lcom()
        .lcom4();

    assert!(
        selection
            .measure()
            .expect("production selection should succeed")
            .is_empty()
    );
    let inclusive = selection
        .measure_with(&CheckOptions::new().with_test_sources(true))
        .expect("dev-source LCOM should succeed");
    assert_eq!(inclusive.len(), 1);
    assert_eq!(inclusive[0].identifier(), "TestOnlyType");
    assert_eq!(inclusive[0].value(), 1.0);
}
