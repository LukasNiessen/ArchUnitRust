/// Immutable presentation options for one metrics HTML report.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct MetricsExportOptions {
    title: String,
    include_timestamp: bool,
    custom_css: Option<String>,
}

impl MetricsExportOptions {
    /// Creates the default ArchUnitRust report options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the document title and visible heading.
    #[must_use]
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Includes or omits the generated-at UTC timestamp.
    #[must_use]
    pub const fn with_timestamp(mut self, include_timestamp: bool) -> Self {
        self.include_timestamp = include_timestamp;
        self
    }

    /// Replaces the built-in offline stylesheet with caller-supplied CSS.
    #[must_use]
    pub fn with_custom_css(mut self, custom_css: impl Into<String>) -> Self {
        self.custom_css = Some(custom_css.into());
        self
    }

    /// Returns the document title.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns whether the report includes a generated-at timestamp.
    #[must_use]
    pub const fn includes_timestamp(&self) -> bool {
        self.include_timestamp
    }

    /// Returns caller-supplied CSS, or `None` for the built-in stylesheet.
    #[must_use]
    pub fn custom_css(&self) -> Option<&str> {
        self.custom_css.as_deref()
    }
}

impl Default for MetricsExportOptions {
    fn default() -> Self {
        Self {
            title: "ArchUnitRust Metrics Report".to_owned(),
            include_timestamp: true,
            custom_css: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MetricsExportOptions;

    #[test]
    fn defaults_and_consuming_modifiers_are_branchable() {
        let base = MetricsExportOptions::new();
        let custom = base
            .clone()
            .with_title("Architecture Quality")
            .with_timestamp(false)
            .with_custom_css("body { color: navy; }");

        assert_eq!(base.title(), "ArchUnitRust Metrics Report");
        assert!(base.includes_timestamp());
        assert!(base.custom_css().is_none());
        assert_eq!(custom.title(), "Architecture Quality");
        assert!(!custom.includes_timestamp());
        assert_eq!(custom.custom_css(), Some("body { color: navy; }"));
    }
}
