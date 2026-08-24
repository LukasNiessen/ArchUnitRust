use crate::{
    CustomFileViolation, CycleViolation, EmptyTestViolation, ExternalModuleDependencyViolation,
    FileDependencyViolation, FilePatternViolation, LayerDependencyRule, LayerDependencyViolation,
    ProjectedEdge, SliceDependencyRule, SliceDependencyViolation, TestViolation, Violation,
};

/// The sole mapping from structured violation data to human-readable prose.
#[derive(Debug, Clone, Copy, Default)]
pub struct ViolationFactory;

impl ViolationFactory {
    /// Formats one built-in violation without applying terminal color or numbering.
    #[must_use]
    pub fn from_violation(violation: &Violation) -> TestViolation {
        match violation {
            Violation::EmptyTest(violation) => format_empty_test(violation),
            Violation::Cycle(violation) => format_cycle(violation),
            Violation::FilePattern(violation) => format_file_pattern(violation),
            Violation::FileDependency(violation) => format_file_dependency(violation),
            Violation::ExternalModuleDependency(violation) => {
                format_external_module_dependency(violation)
            }
            Violation::CustomFile(violation) => format_custom_file(violation),
            Violation::LayerDependency(violation) => format_layer_dependency(violation),
            Violation::SliceDependency(violation) => format_slice_dependency(violation),
        }
    }
}

fn format_empty_test(violation: &EmptyTestViolation) -> TestViolation {
    let mood = if violation.is_negated {
        "negated"
    } else {
        "positive"
    };
    let scope = if violation.selectors.is_empty() {
        "without explicit selectors".to_owned()
    } else {
        format!(
            "with selectors: {}",
            violation
                .selectors
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(" AND ")
        )
    };
    let details = format!(
        "The {mood} {} rule selected no subjects {scope}. Verify the selectors or explicitly use CheckOptions::new().with_allow_empty_tests(true) for an intentional empty scope.",
        violation.subject
    );

    TestViolation::new("Empty test violation", details)
}

fn format_file_pattern(violation: &FilePatternViolation) -> TestViolation {
    let relationship = if violation.is_negated {
        "matches the forbidden"
    } else {
        "does not match the required"
    };
    let requirement = format!(
        "{} pattern \"{}\"",
        violation.check_filter.target(),
        violation.check_filter.pattern().source()
    );

    TestViolation::new(
        "File pattern violation",
        format!(
            "File '{}' {relationship} {requirement}.",
            violation.projected_node.label
        ),
    )
}

fn format_file_dependency(violation: &FileDependencyViolation) -> TestViolation {
    let edge = &violation.dependency;
    let relationship = if violation.is_negated {
        format!(
            "File '{}' depends on forbidden file '{}'.",
            edge.source_label, edge.target_label
        )
    } else {
        format!(
            "File '{}' depends on '{}', which is outside the allowed file target set.",
            edge.source_label, edge.target_label
        )
    };

    TestViolation::new(
        "File dependency violation",
        format!("{relationship}{}", evidence_suffix(edge)),
    )
}

fn format_external_module_dependency(
    violation: &ExternalModuleDependencyViolation,
) -> TestViolation {
    let edge = &violation.dependency;
    let relationship = if violation.is_negated {
        format!(
            "File '{}' depends on forbidden external module '{}'.",
            edge.source_label, edge.target_label
        )
    } else {
        format!(
            "File '{}' depends on external module '{}', which is outside the allowlist.",
            edge.source_label, edge.target_label
        )
    };

    TestViolation::new(
        "External module dependency violation",
        format!("{relationship}{}", evidence_suffix(edge)),
    )
}

fn format_cycle(violation: &CycleViolation) -> TestViolation {
    let evidence = violation
        .cycle
        .iter()
        .flat_map(|edge| edge.cumulated_edges.iter())
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    let evidence = if evidence.is_empty() {
        String::new()
    } else {
        format!(" Evidence: {}.", evidence.join("; "))
    };

    TestViolation::new(
        "Circular dependency detected",
        format!("Cycle: {}.{evidence}", violation.path.join(" -> ")),
    )
}

fn format_custom_file(violation: &CustomFileViolation) -> TestViolation {
    let relationship = if violation.is_negated {
        "matched the forbidden custom predicate"
    } else {
        "failed the required custom predicate"
    };
    let file = &violation.file_info;
    let line_word = if file.non_blank_line_count == 1 {
        "line"
    } else {
        "lines"
    };

    TestViolation::new(
        "Custom file condition violation",
        format!(
            "File '{}' {relationship} '{}'. Source facts: name '{}', extension '{}', directory '{}', {} non-blank {line_word}.",
            file.path,
            violation.message,
            file.name,
            file.extension,
            file.directory,
            file.non_blank_line_count
        ),
    )
}

fn format_layer_dependency(violation: &LayerDependencyViolation) -> TestViolation {
    let relationship = match violation.rule {
        LayerDependencyRule::MayOnlyDependOnLayers => format!(
            "Layer '{}' depends on layer '{}', which is outside its allowed layer set.",
            violation.source_layer, violation.target_layer
        ),
        LayerDependencyRule::MayNotDependOnLayers => format!(
            "Layer '{}' depends on forbidden layer '{}'.",
            violation.source_layer, violation.target_layer
        ),
    };
    let edge = &violation.dependency;

    TestViolation::new(
        "Layer dependency violation",
        format!(
            "{relationship} File dependency: '{}' -> '{}'.{}",
            edge.source_label,
            edge.target_label,
            evidence_suffix(edge)
        ),
    )
}

fn format_slice_dependency(violation: &SliceDependencyViolation) -> TestViolation {
    let relationship = match violation.rule {
        SliceDependencyRule::ContainDependency if violation.is_negated => format!(
            "Slice '{}' depends on forbidden slice '{}'.",
            violation.source_slice, violation.target_slice
        ),
        SliceDependencyRule::ContainDependency => format!(
            "Slice '{}' does not contain the required dependency on slice '{}'.",
            violation.source_slice, violation.target_slice
        ),
    };

    TestViolation::new(
        "Slice dependency violation",
        format!("{relationship}{}", evidence_suffix(&violation.dependency)),
    )
}

fn evidence_suffix(edge: &ProjectedEdge) -> String {
    if edge.cumulated_edges.is_empty() {
        String::new()
    } else {
        format!(
            " Evidence: {}.",
            edge.cumulated_edges
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("; ")
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        CustomFileViolation, CycleViolation, Edge, EmptyTestViolation,
        ExternalModuleDependencyViolation, FileDependencyViolation, FileInfo, FilePatternViolation,
        Graph, ImportKind, LayerDependencyRule, LayerDependencyViolation, ProjectedEdge,
        RegexFactory, SliceDependencyRule, SliceDependencyViolation, Violation, project_to_nodes,
    };

    use super::ViolationFactory;

    fn projected(source: &str, target: &str, external: bool) -> ProjectedEdge {
        ProjectedEdge::new(
            source,
            target,
            [Edge::new(
                source,
                target,
                external,
                [ImportKind::Use, ImportKind::PathReference],
            )],
        )
    }

    #[test]
    fn formats_empty_scope_with_mood_selectors_and_actionable_opt_out() {
        let selector = RegexFactory::default()
            .path_matcher("missing/**")
            .expect("fixture selector should compile");
        let violation =
            Violation::from(EmptyTestViolation::new_with_mood("files", [selector], true));

        let formatted = ViolationFactory::from_violation(&violation);

        assert_eq!(formatted.message, "Empty test violation");
        assert_eq!(
            formatted.details,
            "The negated files rule selected no subjects with selectors: path matches \"missing/**\". Verify the selectors or explicitly use CheckOptions::new().with_allow_empty_tests(true) for an intentional empty scope."
        );
    }

    #[test]
    fn formats_both_file_pattern_moods_from_typed_filter_data() {
        let filter = RegexFactory::default()
            .filename_matcher("*_service.rs")
            .expect("fixture selector should compile");
        let node = project_to_nodes(&Graph::from_edges([Edge::self_edge("src/api.rs")]))
            .into_iter()
            .next()
            .expect("fixture graph should project one node");
        let positive = Violation::from(FilePatternViolation::new(
            filter.clone(),
            node.clone(),
            false,
        ));
        let negated = Violation::from(FilePatternViolation::new(filter, node, true));

        assert_eq!(
            ViolationFactory::from_violation(&positive).details,
            "File 'src/api.rs' does not match the required filename pattern \"*_service.rs\"."
        );
        assert_eq!(
            ViolationFactory::from_violation(&negated).details,
            "File 'src/api.rs' matches the forbidden filename pattern \"*_service.rs\"."
        );
    }

    #[test]
    fn formats_internal_and_external_dependencies_with_raw_rust_evidence() {
        let internal = Violation::from(FileDependencyViolation::new(
            projected("src/api.rs", "src/db.rs", false),
            true,
        ));
        let external = Violation::from(ExternalModuleDependencyViolation::new(
            projected("src/api.rs", "tokio", true),
            false,
        ));

        assert_eq!(
            ViolationFactory::from_violation(&internal).details,
            "File 'src/api.rs' depends on forbidden file 'src/db.rs'. Evidence: src/api.rs -> src/db.rs [use, path_reference]."
        );
        assert_eq!(
            ViolationFactory::from_violation(&external).details,
            "File 'src/api.rs' depends on external module 'tokio', which is outside the allowlist. Evidence: src/api.rs -> tokio (external) [use, path_reference]."
        );
    }

    #[test]
    fn formats_cycle_path_and_every_raw_dependency() {
        let first = projected("src/api.rs", "src/domain.rs", false);
        let second = projected("src/domain.rs", "src/api.rs", false);
        let violation = Violation::from(CycleViolation::new([first, second]));

        let formatted = ViolationFactory::from_violation(&violation);

        assert_eq!(formatted.message, "Circular dependency detected");
        assert_eq!(
            formatted.details,
            "Cycle: src/api.rs -> src/domain.rs -> src/api.rs. Evidence: src/api.rs -> src/domain.rs [use, path_reference]; src/domain.rs -> src/api.rs [use, path_reference]."
        );
    }

    #[test]
    fn formats_custom_requirement_mood_and_portable_source_facts() {
        let info = FileInfo::new("src/api.rs", "pub fn api() {}\n\n");
        let violation = Violation::from(CustomFileViolation::new(
            info,
            "contain no public functions",
            true,
        ));

        let formatted = ViolationFactory::from_violation(&violation);

        assert_eq!(formatted.message, "Custom file condition violation");
        assert_eq!(
            formatted.details,
            "File 'src/api.rs' matched the forbidden custom predicate 'contain no public functions'. Source facts: name 'api', extension '.rs', directory 'src', 1 non-blank line."
        );
    }

    #[test]
    fn formats_layer_policy_and_concrete_file_evidence() {
        let dependency = projected("src/api.rs", "src/db.rs", false);
        let allowed = Violation::from(LayerDependencyViolation::new(
            dependency.clone(),
            "api",
            "database",
            LayerDependencyRule::MayOnlyDependOnLayers,
        ));
        let forbidden = Violation::from(LayerDependencyViolation::new(
            dependency,
            "api",
            "database",
            LayerDependencyRule::MayNotDependOnLayers,
        ));

        assert_eq!(
            ViolationFactory::from_violation(&allowed).details,
            "Layer 'api' depends on layer 'database', which is outside its allowed layer set. File dependency: 'src/api.rs' -> 'src/db.rs'. Evidence: src/api.rs -> src/db.rs [use, path_reference]."
        );
        assert_eq!(
            ViolationFactory::from_violation(&forbidden).details,
            "Layer 'api' depends on forbidden layer 'database'. File dependency: 'src/api.rs' -> 'src/db.rs'. Evidence: src/api.rs -> src/db.rs [use, path_reference]."
        );
    }

    #[test]
    fn formats_forbidden_slice_dependency_with_concrete_rust_evidence() {
        let dependency = projected("src/api.rs", "src/db.rs", false);
        let violation = Violation::from(SliceDependencyViolation::new(
            dependency,
            "api",
            "database",
            SliceDependencyRule::ContainDependency,
            true,
        ));

        assert_eq!(
            ViolationFactory::from_violation(&violation).details,
            "Slice 'api' depends on forbidden slice 'database'. Evidence: src/api.rs -> src/db.rs [use, path_reference]."
        );
    }
}
