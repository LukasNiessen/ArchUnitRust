use std::error::Error;
use std::fmt;

use regex::{Regex, RegexBuilder};

/// The user-facing syntax from which a [`Pattern`] was compiled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PatternSyntax {
    /// Portable path glob syntax.
    Glob,
    /// Rust [`regex`](https://docs.rs/regex) syntax.
    Regex,
}

/// Pattern compilation options.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct PatternOptions {
    /// Match ASCII and Unicode letters without regard to case.
    pub case_insensitive: bool,
}

impl PatternOptions {
    /// Creates the default case-sensitive options.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            case_insensitive: false,
        }
    }

    /// Returns options configured for case-sensitive or case-insensitive matching.
    #[must_use]
    pub const fn case_insensitive(mut self, enabled: bool) -> Self {
        self.case_insensitive = enabled;
        self
    }
}

/// A user pattern compiled to the regular-expression substrate used by every matcher.
///
/// Patterns match a complete candidate. Use `**` in a glob or `.*` in a regular expression when a
/// prefix or suffix is intentionally unconstrained.
#[derive(Debug, Clone)]
pub struct Pattern {
    source: String,
    syntax: PatternSyntax,
    regex: Regex,
}

impl Pattern {
    /// Compiles a portable, case-sensitive glob.
    ///
    /// The supported wildcards are `*`, `**`, `?`, character classes such as `[abc]` and `[a-z]`,
    /// and negated character classes such as `[!0-9]`. Backslashes are path separators rather than
    /// escapes so one glob behaves the same on every host OS.
    pub fn glob(glob: impl AsRef<str>) -> Result<Self, PatternError> {
        Self::glob_with(glob, PatternOptions::default())
    }

    /// Compiles a glob with explicit options.
    pub fn glob_with(glob: impl AsRef<str>, options: PatternOptions) -> Result<Self, PatternError> {
        let source = glob.as_ref();
        let normalized = normalize_separators(source);
        if normalized.is_empty() {
            return Err(PatternError::new(source, "glob is empty"));
        }

        let body =
            glob_to_regex(&normalized).map_err(|message| PatternError::new(source, message))?;
        Self::compile(source, PatternSyntax::Glob, &body, options)
    }

    /// Compiles a regular expression with complete-candidate matching.
    pub fn regex(expression: impl AsRef<str>) -> Result<Self, PatternError> {
        Self::regex_with(expression, PatternOptions::default())
    }

    /// Compiles a regular expression with explicit options.
    pub fn regex_with(
        expression: impl AsRef<str>,
        options: PatternOptions,
    ) -> Result<Self, PatternError> {
        let source = expression.as_ref();
        if source.trim().is_empty() {
            return Err(PatternError::new(source, "regular expression is empty"));
        }
        Self::compile(source, PatternSyntax::Regex, source, options)
    }

    /// Returns the pattern exactly as the user supplied it.
    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Returns whether this pattern originated as a glob or a regular expression.
    #[must_use]
    pub const fn syntax(&self) -> PatternSyntax {
        self.syntax
    }

    /// Returns whether the complete normalized candidate matches.
    #[must_use]
    pub fn matches(&self, candidate: &str) -> bool {
        self.regex.is_match(&normalize_separators(candidate))
    }

    fn compile(
        source: &str,
        syntax: PatternSyntax,
        body: &str,
        options: PatternOptions,
    ) -> Result<Self, PatternError> {
        let anchored = format!(r"\A(?:{body})\z");
        let regex = RegexBuilder::new(&anchored)
            .case_insensitive(options.case_insensitive)
            .build()
            .map_err(|error| PatternError::new(source, error.to_string()))?;

        Ok(Self {
            source: source.to_owned(),
            syntax,
            regex,
        })
    }
}

impl fmt::Display for Pattern {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "\"{}\"", self.source)
    }
}

/// An invalid glob or regular expression supplied by the user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatternError {
    pattern: String,
    message: String,
}

impl PatternError {
    fn new(pattern: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            pattern: pattern.into(),
            message: message.into(),
        }
    }

    /// Returns the pattern that could not be compiled.
    #[must_use]
    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    /// Returns the compiler's useful reason without the surrounding context.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for PatternError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "invalid pattern \"{}\": {}",
            self.pattern, self.message
        )
    }
}

impl Error for PatternError {}

fn normalize_separators(value: &str) -> String {
    let replaced = value.trim().replace('\\', "/");
    if replaced == "/" {
        return replaced;
    }

    let leading = replaced.starts_with('/');
    let normalized = replaced
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("/");

    if leading && !normalized.is_empty() {
        format!("/{normalized}")
    } else {
        normalized
    }
}

fn glob_to_regex(glob: &str) -> Result<String, &'static str> {
    let characters = glob.chars().collect::<Vec<_>>();
    let mut expression = String::with_capacity(glob.len() * 2);
    let mut index = 0;

    while index < characters.len() {
        let character = characters[index];

        if character == '/'
            && characters.get(index + 1) == Some(&'*')
            && characters.get(index + 2) == Some(&'*')
        {
            if index + 3 == characters.len() {
                expression.push_str("(?:/.*)?");
                index += 3;
                continue;
            }
            if characters.get(index + 3) == Some(&'/') {
                expression.push_str("/(?:.*/)?");
                index += 4;
                continue;
            }
        }

        match character {
            '*' if characters.get(index + 1) == Some(&'*') => {
                if characters.get(index + 2) == Some(&'/') {
                    expression.push_str("(?:.*/)?");
                    index += 3;
                } else {
                    expression.push_str(".*");
                    index += 2;
                }
            }
            '*' => {
                expression.push_str("[^/]*");
                index += 1;
            }
            '?' => {
                expression.push_str("[^/]");
                index += 1;
            }
            '[' => {
                let (class, next_index) = character_class(&characters, index)?;
                expression.push_str(&class);
                index = next_index;
            }
            value => {
                push_regex_literal(&mut expression, value);
                index += 1;
            }
        }
    }

    Ok(expression)
}

fn character_class(characters: &[char], start: usize) -> Result<(String, usize), &'static str> {
    let mut end = start + 1;
    while end < characters.len() && characters[end] != ']' {
        end += 1;
    }
    if end == characters.len() {
        return Err("character class is not closed");
    }
    if end == start + 1 {
        return Err("character class is empty");
    }

    let contents = &characters[start + 1..end];
    let mut class = String::from("[");
    let mut content_index = 0;
    if contents.first() == Some(&'!') {
        class.push('^');
        content_index = 1;
    } else if contents.first() == Some(&'^') {
        class.push_str(r"\^");
        content_index = 1;
    }
    if content_index == contents.len() {
        return Err("character class has no members");
    }

    for character in &contents[content_index..] {
        match character {
            '\\' | ']' => {
                class.push('\\');
                class.push(*character);
            }
            _ => class.push(*character),
        }
    }
    class.push(']');
    Ok((class, end + 1))
}

fn push_regex_literal(expression: &mut String, character: char) {
    if matches!(
        character,
        '.' | '+' | '(' | ')' | '|' | '^' | '$' | '{' | '}' | '\\'
    ) {
        expression.push('\\');
    }
    expression.push(character);
}

#[cfg(test)]
mod tests {
    use super::{Pattern, PatternOptions, PatternSyntax};

    #[test]
    fn matches_glob_wildcards_without_crossing_segments() {
        let pattern = Pattern::glob("src/*/mod?.[rR][sS]").expect("fixture glob should compile");

        assert!(pattern.matches("src/api/mod1.rs"));
        assert!(pattern.matches(r"src\api\modx.RS"));
        assert!(!pattern.matches("src/api/internal/mod1.rs"));
        assert!(!pattern.matches("src/api/mod10.rs"));
    }

    #[test]
    fn double_star_crosses_zero_or_more_segments() {
        let between = Pattern::glob("src/**/handler.rs").expect("fixture glob should compile");
        let suffix = Pattern::glob("src/**").expect("fixture glob should compile");
        let prefix = Pattern::glob("**/handler.rs").expect("fixture glob should compile");

        for candidate in ["src/handler.rs", "src/api/handler.rs", "src/a/b/handler.rs"] {
            assert!(between.matches(candidate), "{candidate} should match");
        }
        for candidate in ["src", "src/api", "src/api/handler.rs"] {
            assert!(suffix.matches(candidate), "{candidate} should match");
        }
        assert!(prefix.matches("handler.rs"));
        assert!(prefix.matches("src/api/handler.rs"));
    }

    #[test]
    fn supports_negated_character_classes() {
        let pattern = Pattern::glob("src/[!0-9]*.rs").expect("fixture glob should compile");

        assert!(pattern.matches("src/handler.rs"));
        assert!(!pattern.matches("src/1handler.rs"));
    }

    #[test]
    fn treats_regex_metacharacters_as_glob_literals() {
        let pattern = Pattern::glob("src/file+(1).rs").expect("fixture glob should compile");

        assert!(pattern.matches("src/file+(1).rs"));
        assert!(!pattern.matches("src/file111.rs"));
    }

    #[test]
    fn anchors_globs_and_regular_expressions() {
        let glob = Pattern::glob("api").expect("fixture glob should compile");
        let regex = Pattern::regex("api").expect("fixture regex should compile");

        assert!(glob.matches("api"));
        assert!(regex.matches("api"));
        assert!(!glob.matches("src/api"));
        assert!(!regex.matches("src/api"));
    }

    #[test]
    fn regex_is_the_escape_hatch() {
        let pattern = Pattern::regex(r"src/(api|web)/[a-z_]+\.rs")
            .expect("fixture regular expression should compile");

        assert!(pattern.matches("src/api/handler.rs"));
        assert!(pattern.matches("src/web/router.rs"));
        assert!(!pattern.matches("src/db/repository.rs"));
        assert_eq!(pattern.syntax(), PatternSyntax::Regex);
    }

    #[test]
    fn matching_is_case_sensitive_unless_requested() {
        let strict = Pattern::glob("src/API/**").expect("fixture glob should compile");
        let insensitive =
            Pattern::glob_with("src/API/**", PatternOptions::new().case_insensitive(true))
                .expect("fixture glob should compile");

        assert!(!strict.matches("src/api/handler.rs"));
        assert!(insensitive.matches("src/api/handler.rs"));
    }

    #[test]
    fn retains_the_user_source_for_diagnostics() {
        let pattern = Pattern::glob(r"src\api\**").expect("fixture glob should compile");

        assert_eq!(pattern.source(), r"src\api\**");
        assert_eq!(pattern.to_string(), r#""src\api\**""#);
    }

    #[test]
    fn reports_invalid_patterns_with_context() {
        let empty = Pattern::glob(" ").expect_err("empty glob should fail");
        let unclosed = Pattern::glob("src/[api").expect_err("unclosed class should fail");
        let regex = Pattern::regex("(").expect_err("invalid regex should fail");

        assert_eq!(empty.pattern(), " ");
        assert!(empty.message().contains("empty"));
        assert!(unclosed.message().contains("not closed"));
        assert!(regex.to_string().contains("invalid pattern \"(\""));
    }
}
