use super::{
    Filter, Pattern, PatternError, PatternOptions, PatternSpec, PatternSyntax, PatternTarget,
};

/// Options shared by every matcher produced by a [`RegexFactory`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct RegexFactoryOptions {
    syntax: PatternSyntax,
    case_insensitive: bool,
}

impl RegexFactoryOptions {
    /// Creates case-sensitive glob options.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            syntax: PatternSyntax::Glob,
            case_insensitive: false,
        }
    }

    /// Returns options configured to read user input as `syntax`.
    #[must_use]
    pub const fn syntax(mut self, syntax: PatternSyntax) -> Self {
        self.syntax = syntax;
        self
    }

    /// Returns options configured for case-sensitive or case-insensitive matching.
    #[must_use]
    pub const fn case_insensitive(mut self, enabled: bool) -> Self {
        self.case_insensitive = enabled;
        self
    }

    /// Returns the selected input syntax.
    #[must_use]
    pub const fn pattern_syntax(self) -> PatternSyntax {
        self.syntax
    }

    /// Returns whether generated matchers ignore letter case.
    #[must_use]
    pub const fn is_case_insensitive(self) -> bool {
        self.case_insensitive
    }
}

impl Default for RegexFactoryOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// Compiles user patterns consistently and binds them to selector targets.
///
/// The default factory reads case-sensitive globs. Construct one with [`RegexFactoryOptions`] when
/// a rule family accepts regular expressions or case-insensitive matching.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct RegexFactory {
    options: RegexFactoryOptions,
}

impl RegexFactory {
    /// Creates a factory with explicit compilation options.
    #[must_use]
    pub const fn new(options: RegexFactoryOptions) -> Self {
        Self { options }
    }

    /// Returns this factory's immutable options.
    #[must_use]
    pub const fn options(self) -> RegexFactoryOptions {
        self.options
    }

    /// Compiles one pattern according to this factory's syntax and case behavior.
    pub fn compile(&self, source: impl AsRef<str>) -> Result<Pattern, PatternError> {
        let pattern_options = PatternOptions::new().case_insensitive(self.options.case_insensitive);
        match self.options.syntax {
            PatternSyntax::Glob => Pattern::glob_with(source, pattern_options),
            PatternSyntax::Regex => Pattern::regex_with(source, pattern_options),
            PatternSyntax::Literal => Pattern::literal_with(source, pattern_options),
        }
    }

    /// Matches a pattern against the last path segment.
    pub fn filename_matcher(&self, source: impl Into<PatternSpec>) -> Result<Filter, PatternError> {
        self.matcher(source, PatternTarget::Filename)
    }

    /// Matches a pattern against a file's containing folder.
    pub fn folder_matcher(&self, source: impl Into<PatternSpec>) -> Result<Filter, PatternError> {
        self.matcher(source, PatternTarget::PathWithoutFilename)
    }

    /// Matches a pattern against the complete normalized path.
    pub fn path_matcher(&self, source: impl Into<PatternSpec>) -> Result<Filter, PatternError> {
        self.matcher(source, PatternTarget::Path)
    }

    /// Matches a pattern against an unqualified Rust type name.
    pub fn type_name_matcher(
        &self,
        source: impl Into<PatternSpec>,
    ) -> Result<Filter, PatternError> {
        self.matcher(source, PatternTarget::TypeName)
    }

    /// Matches exactly one normalized file path, treating every character literally.
    pub fn exact_file_matcher(&self, path: impl Into<PatternSpec>) -> Result<Filter, PatternError> {
        let specification = path.into();
        let options = PatternOptions::new().case_insensitive(self.options.case_insensitive);
        let pattern = Pattern::literal_with(specification.source(), options)?;
        self.bind_exclusions(
            Filter::new(pattern, PatternTarget::Path),
            &specification,
            PatternTarget::Path,
        )
    }

    fn matcher(
        &self,
        source: impl Into<PatternSpec>,
        target: PatternTarget,
    ) -> Result<Filter, PatternError> {
        let specification = source.into();
        let pattern = self.compile(specification.source())?;
        self.bind_exclusions(Filter::new(pattern, target), &specification, target)
    }

    fn bind_exclusions(
        &self,
        filter: Filter,
        specification: &PatternSpec,
        parent_target: PatternTarget,
    ) -> Result<Filter, PatternError> {
        let exclusions = specification
            .exclusions()
            .iter()
            .map(|exclusion| {
                self.compile(exclusion.source()).map(|pattern| {
                    Filter::new(pattern, exclusion.target().unwrap_or(parent_target))
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(filter.with_exclusions(exclusions))
    }
}

#[cfg(test)]
mod tests {
    use super::{RegexFactory, RegexFactoryOptions};
    use crate::{PatternSyntax, PatternTarget, pattern};

    #[test]
    fn defaults_to_case_sensitive_globs() {
        let factory = RegexFactory::default();
        let filter = factory
            .folder_matcher("crates/api/**")
            .expect("fixture glob should compile");

        assert_eq!(factory.options(), RegexFactoryOptions::new());
        assert_eq!(filter.pattern().syntax(), PatternSyntax::Glob);
        assert!(filter.matches("crates/api/src/handler.rs"));
        assert!(!filter.matches("crates/API/src/handler.rs"));
    }

    #[test]
    fn compiles_every_selector_with_the_expected_target() {
        let factory = RegexFactory::default();
        let cases = [
            (
                factory
                    .filename_matcher("*.rs")
                    .expect("fixture glob should compile"),
                PatternTarget::Filename,
            ),
            (
                factory
                    .folder_matcher("src/**")
                    .expect("fixture glob should compile"),
                PatternTarget::PathWithoutFilename,
            ),
            (
                factory
                    .path_matcher("src/**")
                    .expect("fixture glob should compile"),
                PatternTarget::Path,
            ),
            (
                factory
                    .type_name_matcher("*Handler")
                    .expect("fixture glob should compile"),
                PatternTarget::TypeName,
            ),
        ];

        for (filter, expected_target) in cases {
            assert_eq!(filter.target(), expected_target);
        }
    }

    #[test]
    fn reads_regular_expression_syntax_when_requested() {
        let factory = RegexFactory::new(RegexFactoryOptions::new().syntax(PatternSyntax::Regex));
        let filter = factory
            .filename_matcher(r"handler_v[0-9]+\.rs")
            .expect("fixture regular expression should compile");

        assert!(filter.matches("src/handler_v12.rs"));
        assert!(!filter.matches("src/handler_vX.rs"));
        assert_eq!(filter.pattern().syntax(), PatternSyntax::Regex);
    }

    #[test]
    fn exact_file_matcher_is_literal_in_every_factory_syntax() {
        for factory in [
            RegexFactory::default(),
            RegexFactory::new(RegexFactoryOptions::new().syntax(PatternSyntax::Regex)),
        ] {
            let filter = factory
                .exact_file_matcher(r"src\handler_v[1]+.rs")
                .expect("fixture literal should compile");

            assert!(filter.matches("src/handler_v[1]+.rs"));
            assert!(!filter.matches("src/handler_v1.rs"));
            assert_eq!(filter.pattern().syntax(), PatternSyntax::Literal);
        }
    }

    #[test]
    fn case_behavior_is_shared_by_pattern_and_literal_matchers() {
        let factory = RegexFactory::new(RegexFactoryOptions::new().case_insensitive(true));

        assert!(
            factory
                .filename_matcher("HANDLER.RS")
                .expect("fixture glob should compile")
                .matches("src/handler.rs")
        );
        assert!(
            factory
                .exact_file_matcher("SRC/HANDLER.RS")
                .expect("fixture literal should compile")
                .matches("src/handler.rs")
        );
    }

    #[test]
    fn reports_invalid_input_from_every_matcher() {
        let factory = RegexFactory::default();

        assert!(factory.filename_matcher("").is_err());
        assert!(factory.folder_matcher("src/[api").is_err());
        assert!(factory.path_matcher("").is_err());
        assert!(factory.type_name_matcher("[Type").is_err());
        assert!(factory.exact_file_matcher(" ").is_err());
    }

    #[test]
    fn compiles_plain_and_targeted_exclusions_with_factory_options() {
        let factory = RegexFactory::new(
            RegexFactoryOptions::new()
                .syntax(PatternSyntax::Regex)
                .case_insensitive(true),
        );
        let filter = factory
            .path_matcher(
                pattern(r"src/.*")
                    .except(r"src/generated/.*")
                    .except_with_name(r".*_generated\.rs"),
            )
            .expect("fixture expressions should compile");

        assert!(filter.matches("SRC/domain/service.rs"));
        assert!(!filter.matches("src/GENERATED/model.rs"));
        assert!(!filter.matches("src/domain/MODEL_GENERATED.RS"));
        assert_eq!(filter.exclusions()[0].target(), PatternTarget::Path);
        assert_eq!(filter.exclusions()[1].target(), PatternTarget::Filename);
        assert_eq!(
            filter.exclusions()[0].pattern().syntax(),
            PatternSyntax::Regex
        );
    }

    #[test]
    fn invalid_exclusions_are_reported_after_a_valid_parent() {
        let error = RegexFactory::default()
            .path_matcher(pattern("src/**").except("src/[generated"))
            .expect_err("invalid exclusion should fail the selector");

        assert_eq!(error.pattern(), "src/[generated");
    }
}
