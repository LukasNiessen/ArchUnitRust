use crate::{Filter, PatternError, ProjectLocator, RegexFactory};

use super::{NegatedMatchPatternFileConditionBuilder, PositiveMatchPatternFileConditionBuilder};

/// Immutable scope builder selecting the files to which a rule applies.
///
/// Selector methods consume and return the builder, so a reusable scope can be cloned and branched.
/// Patterns compile immediately. The first invalid pattern is retained in [`Self::selector_error`]
/// and will become a user error when a terminal rule is checked; this keeps the fluent sentence
/// free of intermediate `Result` handling.
#[derive(Debug, Clone)]
#[must_use = "a file scope has no effect until it is completed and checked"]
pub struct FileConditionBuilder {
    project_locator: ProjectLocator,
    filters: Vec<Filter>,
    selector_error: Option<PatternError>,
}

impl FileConditionBuilder {
    pub(super) const fn new(project_locator: ProjectLocator) -> Self {
        Self {
            project_locator,
            filters: Vec::new(),
            selector_error: None,
        }
    }

    /// Selects files whose final path segment matches `pattern`.
    pub fn with_name(self, pattern: impl AsRef<str>) -> Self {
        let filter = RegexFactory::default().filename_matcher(pattern);
        self.with_filter(filter)
    }

    /// Selects files whose containing directory matches `pattern`.
    pub fn in_folder(self, pattern: impl AsRef<str>) -> Self {
        let filter = RegexFactory::default().folder_matcher(pattern);
        self.with_filter(filter)
    }

    /// Selects files whose complete normalized path matches `pattern`.
    pub fn in_path(self, pattern: impl AsRef<str>) -> Self {
        let filter = RegexFactory::default().path_matcher(pattern);
        self.with_filter(filter)
    }

    /// Selects exactly one normalized file path, treating metacharacters literally.
    pub fn in_file(self, path: impl AsRef<str>) -> Self {
        let filter = RegexFactory::default().exact_file_matcher(path);
        self.with_filter(filter)
    }

    /// Enters the positive mood for a file predicate.
    pub fn should(self) -> PositiveMatchPatternFileConditionBuilder {
        PositiveMatchPatternFileConditionBuilder::new(self)
    }

    /// Enters the negated mood for a file predicate.
    pub fn should_not(self) -> NegatedMatchPatternFileConditionBuilder {
        NegatedMatchPatternFileConditionBuilder::new(self)
    }

    /// Returns where Cargo project discovery will begin.
    #[must_use]
    pub const fn project_locator(&self) -> &ProjectLocator {
        &self.project_locator
    }

    /// Returns every successfully compiled selector in chain order.
    ///
    /// All filters combine with AND semantics.
    #[must_use]
    pub fn filters(&self) -> &[Filter] {
        &self.filters
    }

    /// Returns the first selector compilation error retained by this builder.
    #[must_use]
    pub const fn selector_error(&self) -> Option<&PatternError> {
        self.selector_error.as_ref()
    }

    fn with_filter(mut self, filter: Result<Filter, PatternError>) -> Self {
        if self.selector_error.is_some() {
            return self;
        }

        match filter {
            Ok(filter) => self.filters.push(filter),
            Err(error) => self.selector_error = Some(error),
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::{PatternTarget, files, project_files, project_files_in};

    fn matches(identifier: &str, builder: &super::FileConditionBuilder) -> bool {
        builder.selector_error().is_none()
            && builder
                .filters()
                .iter()
                .all(|filter| filter.matches(identifier))
    }

    #[test]
    fn starts_with_a_locator_and_no_selectors() {
        let automatic = project_files();
        let explicit = project_files_in("examples/layered/Cargo.toml");

        assert!(automatic.project_locator().path().is_none());
        assert!(automatic.filters().is_empty());
        assert!(automatic.selector_error().is_none());
        assert_eq!(
            explicit.project_locator().path(),
            Some(Path::new("examples/layered/Cargo.toml"))
        );
    }

    #[test]
    fn selects_by_filename_folder_path_and_exact_file() {
        let named = project_files().with_name("*_service.rs");
        let folder = project_files().in_folder("src/**/service");
        let path = project_files().in_path("src/**/*.rs");
        let file = project_files().in_file("src/order[legacy].rs");

        assert!(matches("src/orders/order_service.rs", &named));
        assert!(!matches("src/orders/order.rs", &named));
        assert!(matches("src/orders/service/order.rs", &folder));
        assert!(!matches("tests/orders/service/order.rs", &folder));
        assert!(matches("src/orders/order.rs", &path));
        assert!(!matches("tests/orders/order.rs", &path));
        assert!(matches("src/order[legacy].rs", &file));
        assert!(!matches("src/orderl.rs", &file));

        assert_eq!(named.filters()[0].target(), PatternTarget::Filename);
        assert_eq!(
            folder.filters()[0].target(),
            PatternTarget::PathWithoutFilename
        );
        assert_eq!(path.filters()[0].target(), PatternTarget::Path);
        assert_eq!(file.filters()[0].target(), PatternTarget::Path);
    }

    #[test]
    fn combines_chained_selectors_with_and_semantics() {
        let builder = project_files()
            .in_folder("src/**/service")
            .with_name("*_service.rs")
            .in_path("src/orders/**");

        assert!(matches("src/orders/service/order_service.rs", &builder));
        assert!(!matches("src/billing/service/billing_service.rs", &builder));
        assert!(!matches("src/orders/service/order_repository.rs", &builder));
    }

    #[test]
    fn reusable_scopes_can_be_cloned_and_branched() {
        let base = files().in_folder("src/**");
        let services = base.clone().with_name("*_service.rs");
        let repositories = base.clone().with_name("*_repository.rs");

        assert_eq!(base.filters().len(), 1);
        assert_eq!(services.filters().len(), 2);
        assert_eq!(repositories.filters().len(), 2);
        assert!(matches("src/orders/order_service.rs", &services));
        assert!(matches("src/orders/order_repository.rs", &repositories));
    }

    #[test]
    fn retains_the_first_invalid_selector_without_breaking_the_chain() {
        let builder = project_files()
            .in_path("src/**")
            .in_folder("src/[api")
            .with_name("*.rs");

        let error = builder
            .selector_error()
            .expect("invalid selector should be retained");
        assert_eq!(builder.filters().len(), 1);
        assert_eq!(error.pattern(), "src/[api");
        assert!(error.message().contains("not closed"));
    }

    #[test]
    fn enters_exactly_the_positive_or_negated_mood() {
        let scope = project_files().in_path("src/**");

        let positive = scope.clone().should();
        let negative = scope.should_not();

        assert!(!positive.is_negated());
        assert!(negative.is_negated());
    }
}
