use archunit::{
    Checkable, ImportKind, assert_passes, pattern, project_files_in, project_layers_in,
};

const PROJECT_ROOT: &str = env!("CARGO_MANIFEST_DIR");
const DOMAIN_MODULES: [&str; 5] = ["files", "graph", "layers", "metrics", "slices"];

#[test]
fn common_depends_only_on_itself_the_standard_library_and_analysis_toolchain() {
    let internal_dependencies = project_files_in(PROJECT_ROOT)
        .in_path("src/common**")
        .should()
        .depend_on_files()
        .in_path("src/common**");
    let mut external_dependencies = project_files_in(PROJECT_ROOT)
        .in_path("src/common**")
        .should()
        .depend_on_external_modules()
        .matching("std");
    for allowed in [
        "core",
        "alloc",
        "cargo_metadata",
        "proc_macro2",
        "regex",
        "syn",
        "thiserror",
    ] {
        external_dependencies = external_dependencies.matching(allowed);
    }

    assert_passes!(internal_dependencies);
    assert_passes!(external_dependencies);
}

#[test]
fn domain_modules_do_not_depend_on_one_another() {
    let mut architecture = project_layers_in(PROJECT_ROOT);
    for domain in DOMAIN_MODULES {
        architecture = architecture
            .layer(domain)
            .defined_by(format!("src/{domain}**"));
    }
    for domain in DOMAIN_MODULES {
        let peers = DOMAIN_MODULES
            .iter()
            .copied()
            .filter(|candidate| *candidate != domain)
            .collect::<Vec<_>>();
        architecture = architecture
            .where_layer(domain)
            .may_not_depend_on_layers(&peers);
    }

    assert_passes!(architecture);
}

#[test]
fn implementation_files_do_not_depend_on_the_public_surface() {
    let rule = project_files_in(PROJECT_ROOT)
        .in_path(pattern("src/**").except("src/lib.rs"))
        .should_not()
        .depend_on_files()
        .in_path("src/lib.rs");

    assert_passes!(rule);
}

#[test]
fn every_architectural_unit_is_free_of_executable_dependency_cycles() {
    for scope in [
        "src/*.rs",
        "src/common**",
        "src/files**",
        "src/graph**",
        "src/layers**",
        "src/metrics**",
        "src/slices**",
        "src/testing**",
    ] {
        let rule = project_files_in(PROJECT_ROOT)
            .in_path(scope)
            .should()
            .have_no_cycles()
            .excluding_dependency_kinds([ImportKind::Mod, ImportKind::PubUse]);

        let _: &dyn Checkable = &rule;
        assert_passes!(rule);
    }
}
