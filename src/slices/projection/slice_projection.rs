use std::{collections::BTreeSet, error::Error, fmt};

use regex::Regex;

use crate::common::extraction::normalize_identifier;
use crate::{Edge, Graph, MappedEdge, ProjectedGraph, project_edges};

const SLICE_CAPTURE: &str = "(**)";

/// An invalid projection definition supplied by a caller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SliceProjectionError {
    input: String,
    message: String,
}

impl SliceProjectionError {
    fn new(input: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            input: input.into(),
            message: message.into(),
        }
    }

    /// Returns the projection input that was rejected.
    #[must_use]
    pub fn input(&self) -> &str {
        &self.input
    }

    /// Returns the reason without surrounding error context.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for SliceProjectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "invalid slice projection {:?}: {}",
            self.input, self.message
        )
    }
}

impl Error for SliceProjectionError {}

#[derive(Debug, Clone)]
enum SliceLabeler {
    Identity,
    Regex(Regex),
    FileSuffix(Vec<(String, String)>),
}

/// An immutable, reusable mapping from project-relative Rust files to slice names.
#[derive(Debug, Clone)]
pub struct SliceProjection {
    labeler: SliceLabeler,
}

impl SliceProjection {
    /// Returns the normalized file identifier as its own slice name.
    #[must_use]
    pub fn identity() -> Self {
        Self {
            labeler: SliceLabeler::Identity,
        }
    }

    /// Returns the slice name selected for one normalized file path.
    #[must_use]
    pub fn label_for(&self, path: &str) -> Option<String> {
        let path = normalize_identifier(path);
        if path.is_empty() {
            return None;
        }

        match &self.labeler {
            SliceLabeler::Identity => Some(path),
            SliceLabeler::Regex(regex) => regex
                .captures(&path)
                .and_then(|captures| captures.get(1))
                .map(|capture| capture.as_str().to_owned())
                .filter(|capture| !capture.is_empty()),
            SliceLabeler::FileSuffix(labels) => {
                let filename = path.rsplit('/').next().unwrap_or(path.as_str());
                let stem = filename
                    .rsplit_once('.')
                    .map_or(filename, |(stem, _extension)| stem);
                labels
                    .iter()
                    .find(|(suffix, _label)| stem.ends_with(suffix))
                    .map(|(_suffix, label)| label.clone())
            }
        }
    }

    /// Maps one raw dependency while retaining external targets and dropping intra-slice edges.
    #[must_use]
    pub fn map_edge(&self, edge: &Edge) -> Option<MappedEdge> {
        if edge.is_self_edge() {
            return None;
        }

        let source = self.label_for(&edge.source)?;
        let target = if edge.external {
            edge.target.clone()
        } else {
            self.label_for(&edge.target)?
        };
        if !edge.external && source == target {
            return None;
        }

        Some(MappedEdge::new(source, target))
    }

    /// Projects and cumulates a complete extracted graph through this slice definition.
    #[must_use]
    pub fn project(&self, graph: &Graph) -> ProjectedGraph {
        project_edges(graph, |edge| self.map_edge(edge))
    }

    /// Returns every selected internal slice, including isolated files represented by self-edges.
    #[must_use]
    pub fn slice_labels(&self, graph: &Graph) -> Vec<String> {
        let mut labels = BTreeSet::new();
        for edge in graph {
            if let Some(label) = self.label_for(&edge.source) {
                labels.insert(label);
            }
            if !edge.external
                && let Some(label) = self.label_for(&edge.target)
            {
                labels.insert(label);
            }
        }
        labels.into_iter().collect()
    }
}

/// Creates the identity slice projection.
///
/// The longer name avoids colliding with the raw-edge [`crate::identity`] mapper at the crate root.
#[must_use]
pub fn slice_identity() -> SliceProjection {
    SliceProjection::identity()
}

/// Captures a slice name through exactly one `(**)` placeholder in a portable path pattern.
pub fn slice_by_pattern(pattern: impl AsRef<str>) -> Result<SliceProjection, SliceProjectionError> {
    let original = pattern.as_ref();
    let pattern = original.trim().replace('\\', "/");
    let captures = pattern.match_indices(SLICE_CAPTURE).count();
    if captures != 1 {
        return Err(SliceProjectionError::new(
            original,
            format!("pattern must contain exactly one {SLICE_CAPTURE} slice capture"),
        ));
    }

    let Some((prefix, suffix)) = pattern.split_once(SLICE_CAPTURE) else {
        return Err(SliceProjectionError::new(
            original,
            format!("pattern must contain exactly one {SLICE_CAPTURE} slice capture"),
        ));
    };
    let expression = format!(
        r"\A{}([^/]+){}.*\z",
        glob_fragment(prefix),
        glob_fragment(suffix)
    );
    projection_from_regex(original, &expression)
}

/// Captures a slice name through the first group in a Rust regular expression.
pub fn slice_by_regex(
    expression: impl AsRef<str>,
) -> Result<SliceProjection, SliceProjectionError> {
    let expression = expression.as_ref();
    projection_from_regex(expression, expression)
}

/// Maps Rust filename stems to slices by their longest matching suffix.
pub fn slice_by_file_suffix<I, S, L>(labeling: I) -> Result<SliceProjection, SliceProjectionError>
where
    I: IntoIterator<Item = (S, L)>,
    S: Into<String>,
    L: Into<String>,
{
    let mut labels = labeling
        .into_iter()
        .map(|(suffix, label)| (suffix.into(), label.into()))
        .collect::<Vec<_>>();
    if labels.is_empty() {
        return Err(SliceProjectionError::new(
            "file suffixes",
            "at least one suffix-to-slice mapping is required",
        ));
    }
    if let Some((suffix, _label)) = labels.iter().find(|(suffix, _label)| suffix.is_empty()) {
        return Err(SliceProjectionError::new(
            suffix,
            "file suffix must not be empty",
        ));
    }
    if let Some((_suffix, label)) = labels
        .iter()
        .find(|(_suffix, label)| label.trim().is_empty())
    {
        return Err(SliceProjectionError::new(
            label,
            "slice name must not be empty",
        ));
    }

    labels.sort_by(|left, right| {
        right
            .0
            .len()
            .cmp(&left.0.len())
            .then_with(|| left.cmp(right))
    });
    Ok(SliceProjection {
        labeler: SliceLabeler::FileSuffix(labels),
    })
}

fn projection_from_regex(
    input: &str,
    expression: &str,
) -> Result<SliceProjection, SliceProjectionError> {
    if expression.trim().is_empty() {
        return Err(SliceProjectionError::new(
            input,
            "regular expression must not be empty",
        ));
    }
    let regex = Regex::new(expression)
        .map_err(|error| SliceProjectionError::new(input, error.to_string()))?;
    if regex.captures_len() < 2 {
        return Err(SliceProjectionError::new(
            input,
            "regular expression must contain a slice capture group",
        ));
    }
    Ok(SliceProjection {
        labeler: SliceLabeler::Regex(regex),
    })
}

fn glob_fragment(fragment: &str) -> String {
    let characters = fragment.chars().collect::<Vec<_>>();
    let mut expression = String::new();
    let mut index = 0;
    while index < characters.len() {
        match characters[index] {
            '*' if characters.get(index + 1) == Some(&'*') => {
                expression.push_str(".*");
                index += 2;
            }
            '*' => {
                expression.push_str("[^/]*");
                index += 1;
            }
            '?' => {
                expression.push_str("[^/]");
                index += 1;
            }
            character => {
                expression.push_str(&regex::escape(&character.to_string()));
                index += 1;
            }
        }
    }
    expression
}

#[cfg(test)]
mod tests {
    use crate::{Edge, Graph, ImportKind, MappedEdge};

    use super::{slice_by_file_suffix, slice_by_pattern, slice_by_regex, slice_identity};

    fn edge(source: &str, target: &str, external: bool) -> Edge {
        Edge::new(source, target, external, [ImportKind::Use])
    }

    #[test]
    fn pattern_capture_normalizes_paths_and_supports_surrounding_globs() {
        let projection =
            slice_by_pattern("crates/**/(**)/src/").expect("fixture slice pattern should compile");

        assert_eq!(
            projection.label_for(r"crates\workspace\billing\src\lib.rs"),
            Some("billing".to_owned())
        );
        assert_eq!(projection.label_for("crates/billing/tests/api.rs"), None);
    }

    #[test]
    fn pattern_requires_exactly_one_slice_capture() {
        for pattern in ["src/**", "src/(**)/(**)/"] {
            let error = slice_by_pattern(pattern).expect_err("pattern should be rejected");
            assert!(error.message().contains("exactly one"));
            assert_eq!(error.input(), pattern);
        }
    }

    #[test]
    fn regex_uses_its_first_capture_and_rejects_missing_captures() {
        let projection =
            slice_by_regex(r"\Asrc/([^/]+)/").expect("fixture regular expression should compile");

        assert_eq!(
            projection.label_for("src/application/service.rs"),
            Some("application".to_owned())
        );
        assert!(slice_by_regex(r"src/.*").is_err());
        assert!(slice_by_regex("[").is_err());
    }

    #[test]
    fn suffix_projection_uses_the_longest_matching_rust_stem_suffix() {
        let projection = slice_by_file_suffix([
            ("service", "generic"),
            ("_service", "services"),
            ("_controller", "controllers"),
        ])
        .expect("fixture suffixes should be valid");

        assert_eq!(
            projection.label_for("src/order_service.rs"),
            Some("services".to_owned())
        );
        assert_eq!(projection.label_for("src/helper.rs"), None);
        assert!(slice_by_file_suffix::<_, &str, &str>([]).is_err());
    }

    #[test]
    fn mapping_retains_external_targets_and_drops_self_and_intra_slice_edges() {
        let projection =
            slice_by_pattern("src/(**)/").expect("fixture slice pattern should compile");

        assert_eq!(
            projection.map_edge(&edge("src/api/a.rs", "src/domain/b.rs", false)),
            Some(MappedEdge::new("api", "domain"))
        );
        assert_eq!(
            projection.map_edge(&edge("src/api/a.rs", "serde", true)),
            Some(MappedEdge::new("api", "serde"))
        );
        assert_eq!(
            projection.map_edge(&edge("src/api/a.rs", "src/api/b.rs", false)),
            None
        );
        assert_eq!(projection.map_edge(&Edge::self_edge("src/api/a.rs")), None);
    }

    #[test]
    fn identity_and_slice_labels_retain_isolated_internal_files() {
        let graph =
            Graph::from_edges([Edge::self_edge("src/b.rs"), edge("src/a.rs", "serde", true)]);
        let projection = slice_identity();

        let external = graph
            .edges()
            .iter()
            .find(|edge| edge.external)
            .expect("fixture external edge should exist");
        assert_eq!(
            projection.map_edge(external),
            Some(MappedEdge::new("src/a.rs", "serde"))
        );
        assert_eq!(projection.slice_labels(&graph), ["src/a.rs", "src/b.rs"]);
    }
}
