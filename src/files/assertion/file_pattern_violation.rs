use crate::{Filter, ProjectedNode};

/// One selected file that disagrees with a name or location predicate.
///
/// The compiled filter records both the original pattern and the part of the file identifier that
/// was judged. The mood remains data so reporting can describe the rule without reconstructing it.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct FilePatternViolation {
    /// The filename, folder, or path requirement that the file disagreed with.
    pub check_filter: Filter,
    /// The offending file and its dependency evidence.
    pub projected_node: ProjectedNode,
    /// Whether matching the filter was forbidden rather than required.
    pub is_negated: bool,
}

impl FilePatternViolation {
    /// Creates data for one file that failed a pattern predicate.
    #[must_use]
    pub const fn new(
        check_filter: Filter,
        projected_node: ProjectedNode,
        is_negated: bool,
    ) -> Self {
        Self {
            check_filter,
            projected_node,
            is_negated,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Graph, PatternTarget, RegexFactory, project_to_nodes};

    use super::FilePatternViolation;

    #[test]
    fn retains_the_requirement_file_evidence_and_mood() {
        let node = project_to_nodes(&Graph::from_edges([crate::Edge::self_edge(
            "src/order_service.rs",
        )]))
        .into_iter()
        .next()
        .expect("fixture graph should project one node");
        let filter = RegexFactory::default()
            .filename_matcher("*_service.rs")
            .expect("fixture pattern should compile");

        let violation = FilePatternViolation::new(filter, node, true);

        assert_eq!(violation.check_filter.target(), PatternTarget::Filename);
        assert_eq!(violation.check_filter.pattern().source(), "*_service.rs");
        assert_eq!(violation.projected_node.label, "src/order_service.rs");
        assert!(violation.is_negated);
    }
}
