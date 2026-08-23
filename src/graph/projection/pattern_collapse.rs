use crate::Pattern;

use super::GraphQueryError;

/// Collapses node labels through a regular-expression capture replacement.
#[derive(Debug, Clone)]
pub struct PatternCollapse {
    pattern: Pattern,
    replacement: String,
}

impl PatternCollapse {
    /// The Rust `regex` spelling for the first capture group.
    pub const DEFAULT_REPLACEMENT: &'static str = "$1";

    /// Compiles a collapse expression with an explicit Rust `regex` replacement.
    pub fn new(
        expression: impl AsRef<str>,
        replacement: impl Into<String>,
    ) -> Result<Self, GraphQueryError> {
        let pattern = Pattern::regex(expression)
            .map_err(|source| GraphQueryError::invalid_pattern("collapse", source))?;
        let replacement = replacement.into();
        if replacement.trim().is_empty() {
            return Err(GraphQueryError::EmptyCollapseReplacement);
        }

        Ok(Self {
            pattern,
            replacement,
        })
    }

    /// Compiles a collapse expression that replaces a match with its first capture group.
    pub fn first_capture(expression: impl AsRef<str>) -> Result<Self, GraphQueryError> {
        Self::new(expression, Self::DEFAULT_REPLACEMENT)
    }

    /// Returns the compiled collapse pattern.
    #[must_use]
    pub const fn pattern(&self) -> &Pattern {
        &self.pattern
    }

    /// Returns the Rust `regex` replacement expression.
    #[must_use]
    pub fn replacement(&self) -> &str {
        &self.replacement
    }

    pub(crate) fn collapse(&self, node: &str) -> Result<String, GraphQueryError> {
        let collapsed = self.pattern.replace(node, &self.replacement);
        if collapsed.trim().is_empty() {
            return Err(GraphQueryError::EmptyCollapsedNode {
                node: node.to_owned(),
            });
        }
        Ok(collapsed)
    }
}

#[cfg(test)]
mod tests {
    use super::PatternCollapse;
    use crate::GraphQueryError;

    #[test]
    fn defaults_to_the_first_capture_with_rust_replacement_syntax() {
        let collapse = PatternCollapse::first_capture(r"src/([^/]+)/.*")
            .expect("fixture collapse should compile");

        assert_eq!(collapse.replacement(), "$1");
        assert_eq!(
            collapse
                .collapse("src/application/service.rs")
                .expect("capture should produce a label"),
            "application"
        );
        assert_eq!(
            collapse
                .collapse("tests/service.rs")
                .expect("unmatched node should remain unchanged"),
            "tests/service.rs"
        );
    }

    #[test]
    fn rejects_invalid_patterns_empty_replacements_and_empty_labels() {
        assert!(matches!(
            PatternCollapse::first_capture("["),
            Err(GraphQueryError::InvalidPattern { .. })
        ));
        assert!(matches!(
            PatternCollapse::new(".*", ""),
            Err(GraphQueryError::EmptyCollapseReplacement)
        ));

        let collapse =
            PatternCollapse::new(".*", "$1").expect("a missing capture is valid regex syntax");
        assert!(matches!(
            collapse.collapse("src/lib.rs"),
            Err(GraphQueryError::EmptyCollapsedNode { .. })
        ));
    }
}
