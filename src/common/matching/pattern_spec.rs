use std::borrow::Cow;

use super::PatternTarget;

/// One exclusion attached to a [`PatternSpec`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatternExclusion {
    source: String,
    target: Option<PatternTarget>,
}

impl PatternExclusion {
    fn new(source: impl Into<String>, target: Option<PatternTarget>) -> Self {
        Self {
            source: source.into(),
            target,
        }
    }

    /// Returns the uncompiled exclusion pattern.
    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Returns an explicit target, or `None` when the parent selector target is inherited.
    #[must_use]
    pub const fn target(&self) -> Option<PatternTarget> {
        self.target
    }
}

/// One selector pattern together with its ordered per-selector exclusions.
///
/// Plain strings convert to this type automatically. Use [`pattern`] only when a selector needs an
/// `except` companion or a target-explicit exclusion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatternSpec {
    source: String,
    exclusions: Vec<PatternExclusion>,
}

impl PatternSpec {
    /// Creates a selector without exclusions.
    #[must_use]
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            exclusions: Vec::new(),
        }
    }

    /// Excludes a pattern interpreted against the parent selector's target.
    #[must_use]
    pub fn except(self, exclusion: impl Into<String>) -> Self {
        self.with_exclusion(exclusion, None)
    }

    /// Excludes multiple patterns interpreted against the parent selector's target.
    #[must_use]
    pub fn except_all<I, S>(mut self, exclusions: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.exclusions.extend(
            exclusions
                .into_iter()
                .map(|source| PatternExclusion::new(source, None)),
        );
        self
    }

    /// Excludes a pattern matched against the complete normalized path.
    #[must_use]
    pub fn except_in_path(self, exclusion: impl Into<String>) -> Self {
        self.with_exclusion(exclusion, Some(PatternTarget::Path))
    }

    /// Excludes a pattern matched against the containing folder.
    #[must_use]
    pub fn except_in_folder(self, exclusion: impl Into<String>) -> Self {
        self.with_exclusion(exclusion, Some(PatternTarget::PathWithoutFilename))
    }

    /// Excludes a pattern matched against the filename.
    #[must_use]
    pub fn except_with_name(self, exclusion: impl Into<String>) -> Self {
        self.with_exclusion(exclusion, Some(PatternTarget::Filename))
    }

    /// Excludes a pattern matched against the unqualified Rust type name.
    #[must_use]
    pub fn except_for_types_matching(self, exclusion: impl Into<String>) -> Self {
        self.with_exclusion(exclusion, Some(PatternTarget::TypeName))
    }

    /// Returns the parent selector pattern.
    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Returns exclusions in declaration order.
    #[must_use]
    pub fn exclusions(&self) -> &[PatternExclusion] {
        &self.exclusions
    }

    fn with_exclusion(mut self, source: impl Into<String>, target: Option<PatternTarget>) -> Self {
        self.exclusions.push(PatternExclusion::new(source, target));
        self
    }
}

impl From<&str> for PatternSpec {
    fn from(source: &str) -> Self {
        Self::new(source)
    }
}

impl From<String> for PatternSpec {
    fn from(source: String) -> Self {
        Self::new(source)
    }
}

impl From<&String> for PatternSpec {
    fn from(source: &String) -> Self {
        Self::new(source.clone())
    }
}

impl From<Cow<'_, str>> for PatternSpec {
    fn from(source: Cow<'_, str>) -> Self {
        Self::new(source.into_owned())
    }
}

/// Creates a selector pattern that can receive one or more `except` companions.
#[must_use]
pub fn pattern(source: impl Into<String>) -> PatternSpec {
    PatternSpec::new(source)
}

#[cfg(test)]
mod tests {
    use super::pattern;
    use crate::PatternTarget;

    #[test]
    fn exclusions_are_consuming_ordered_and_optionally_targeted() {
        let base = pattern("src/**");
        let configured = base
            .clone()
            .except_all(["src/generated/**", "src/vendor/**"])
            .except_with_name("*_generated.rs")
            .except_in_folder("src/fixtures/**")
            .except_in_path("src/private/**")
            .except_for_types_matching("Generated*");

        assert!(base.exclusions().is_empty());
        assert_eq!(configured.source(), "src/**");
        assert_eq!(configured.exclusions().len(), 6);
        assert_eq!(configured.exclusions()[0].target(), None);
        assert_eq!(
            configured.exclusions()[2].target(),
            Some(PatternTarget::Filename)
        );
        assert_eq!(
            configured.exclusions()[5].target(),
            Some(PatternTarget::TypeName)
        );
    }
}
