use std::path::PathBuf;

use archunit::{
    ArchUnitError, Checkable, PatternTarget, RegexFactory, pattern, project_files,
    project_files_in, project_graph_in, project_layers_in, project_slices_in,
};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

#[test]
fn public_pattern_specs_support_ordered_parent_and_targeted_exclusions() {
    let base = pattern("src/**");
    let configured = base
        .clone()
        .except_all(["src/generated/**", "src/vendor/**"])
        .except_with_name("*_generated.rs")
        .except_in_folder("src/fixtures/**")
        .except_in_path("src/private/**")
        .except_for_types_matching("Generated*");
    let filter = RegexFactory::default()
        .path_matcher(configured.clone())
        .expect("public exclusion specification should compile");

    assert!(base.exclusions().is_empty());
    assert_eq!(configured.exclusions().len(), 6);
    assert_eq!(filter.exclusions().len(), 6);
    assert_eq!(filter.exclusions()[0].target(), PatternTarget::Path);
    assert_eq!(filter.exclusions()[2].target(), PatternTarget::Filename);
    assert_eq!(filter.exclusions()[5].target(), PatternTarget::TypeName);
    assert!(filter.matches("src/domain/service.rs"));
    assert!(!filter.matches("src/generated/model.rs"));
    assert!(!filter.matches("src/domain/model_generated.rs"));
}

#[test]
fn file_scope_predicate_and_exact_selectors_all_accept_exclusions() {
    let scope = project_files()
        .in_path(pattern("src/**").except_in_folder("src/generated/**"))
        .in_folder(pattern("src/**").except_with_name("mod.rs"))
        .with_name(pattern("*.rs").except("lib.rs"))
        .in_file(pattern("src/domain.rs").except_in_path("src/private/**"));
    let predicates = [
        scope
            .clone()
            .should()
            .have_name(pattern("*.rs").except("generated.rs")),
        scope
            .clone()
            .should()
            .be_in_folder(pattern("src/**").except_with_name("mod.rs")),
        scope
            .should_not()
            .be_in_path(pattern("src/**").except_in_folder("src/generated/**")),
    ];

    for predicate in predicates {
        let filter = predicate
            .check_filter()
            .expect("predicate exclusion should compile");
        assert_eq!(filter.exclusions().len(), 1);
    }
}

#[test]
fn file_dependency_source_object_and_external_exclusions_reach_real_rules() {
    let layered = fixture("layered_project");
    let source_excluded = project_files_in(layered.as_path())
        .in_path(pattern("src/**").except_in_folder("src/api"))
        .should_not()
        .depend_on_files()
        .in_path("src/database/**")
        .check()
        .expect("source-excluded dependency rule should execute");
    let object_excluded = project_files_in(layered)
        .in_file("src/api/mod.rs")
        .should_not()
        .depend_on_files()
        .in_path(pattern("src/database/**").except_with_name("repository.rs"))
        .check()
        .expect("object-excluded dependency rule should execute");
    let external_excluded = project_files_in(fixture("extraction_workspace"))
        .in_file("crates/app/source/api.rs")
        .should_not()
        .depend_on_external_modules()
        .matching(pattern("*").except("std"))
        .check()
        .expect("external-module exclusion should execute");

    assert!(source_excluded.iter().any(|violation| {
        violation
            .as_file_dependency()
            .is_some_and(|data| data.dependency.source_label == "src/application/service.rs")
    }));
    assert!(source_excluded.iter().all(|violation| {
        violation
            .as_file_dependency()
            .is_none_or(|data| data.dependency.source_label != "src/api/mod.rs")
    }));
    assert!(object_excluded.is_empty());
    assert_eq!(external_excluded.len(), 1);
    assert_eq!(
        external_excluded[0]
            .as_external_module_dependency()
            .expect("the remaining dependency should retain external evidence")
            .dependency
            .target_label,
        "core"
    );
}

#[test]
fn layer_graph_metric_and_slice_selectors_apply_exclusions() {
    let layered = fixture("layered_project");
    let layers = project_layers_in(layered.as_path())
        .layer("application")
        .defined_by(pattern("src/application/**").except_with_name("service.rs"));
    let layer = &layers.layer_definitions()[0];
    let focused = project_graph_in(layered.as_path())
        .focus_on(pattern("src/application/**").except_with_name("mod.rs"), 0)
        .snapshot()
        .expect("excluded graph focus should execute");
    let graph_queries = project_graph_in(layered.as_path())
        .reachable_from(pattern("src/api/**").except_in_path("src/api/mod.rs"))
        .dependents_of(pattern("src/database/**").except_with_name("repository.rs"));
    let metrics = archunit::metrics_in(fixture("metrics_project"))
        .in_path(pattern("src/**").except_with_name("extensions.rs"))
        .for_types_matching(
            pattern("*")
                .except("Service")
                .except_for_types_matching("State"),
        )
        .analyze()
        .expect("excluded metrics selection should execute");
    let plantuml = project_slices_in(layered)
        .defined_by(pattern("src/(**)/").except_in_folder("src/support"))
        .to_plantuml()
        .expect("excluded slice projection should execute");

    assert!(layer.matches("src/application/mod.rs"));
    assert!(!layer.matches("src/application/service.rs"));
    assert_eq!(
        focused
            .nodes
            .iter()
            .map(|node| node.label.as_str())
            .collect::<Vec<_>>(),
        ["src/application/service.rs"]
    );
    assert!(
        !graph_queries
            .options()
            .reachable_from()
            .expect("query should retain reachable selector")
            .matches("src/api/mod.rs")
    );
    assert!(
        !graph_queries
            .options()
            .dependents_of()
            .expect("query should retain dependents selector")
            .matches("src/database/repository.rs")
    );
    assert_eq!(
        metrics
            .files()
            .iter()
            .map(|file| file.path())
            .collect::<Vec<_>>(),
        ["src/domain.rs"]
    );
    assert_eq!(
        metrics
            .types()
            .iter()
            .map(|type_info| type_info.name())
            .collect::<Vec<_>>(),
        ["Port", "Repository", "Word"]
    );
    assert!(!plantuml.contains("support"));
}

#[test]
fn invalid_exclusions_are_user_errors_before_discovery_in_each_family() {
    let invalid = pattern("src/**").except("src/[generated");
    let file = project_files_in("definitely/missing")
        .in_path(invalid.clone())
        .should()
        .have_no_cycles()
        .check()
        .expect_err("file exclusion should fail before discovery");
    let graph = project_graph_in("definitely/missing")
        .focus_on(invalid.clone(), 0)
        .snapshot()
        .expect_err("graph exclusion should fail before discovery");
    let layer = project_layers_in("definitely/missing")
        .layer("api")
        .defined_by(invalid.clone())
        .where_layer("api")
        .may_only_depend_on_layers(&[])
        .check()
        .expect_err("layer exclusion should fail before discovery");
    let metric = archunit::metrics_in(PathBuf::from("definitely/missing"))
        .in_path(invalid.clone())
        .analyze()
        .expect_err("metric exclusion should fail before discovery");
    let slice = project_slices_in("definitely/missing")
        .defined_by(pattern("src/(**)/").except("src/[generated"))
        .to_plantuml()
        .expect_err("slice exclusion should fail before discovery");

    for error in [file, graph, layer, metric, slice] {
        assert!(matches!(error, ArchUnitError::User(_)));
        assert!(error.to_string().contains("src/[generated"));
    }
}
