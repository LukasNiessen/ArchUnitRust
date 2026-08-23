use std::{
    env,
    io::{self, IsTerminal},
};

const RESET: &str = "\x1b[0m";

/// Controls ANSI styling in architecture-test messages.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ColorChoice {
    /// Detect terminal and environment support.
    #[default]
    Auto,
    /// Emit ANSI styling regardless of terminal detection.
    Always,
    /// Emit plain text.
    Never,
}

/// ANSI color helpers shared by result formatters and test integration.
#[derive(Debug, Clone, Copy, Default)]
pub struct ColorUtils;

impl ColorUtils {
    /// Returns whether automatic color detection currently permits ANSI styling.
    #[must_use]
    pub fn supports_color() -> bool {
        automatic_color_supported(
            io::stdout().is_terminal(),
            env::var_os("NO_COLOR").is_some(),
            env::var("TERM").ok().as_deref(),
            env::var("CI").ok().as_deref(),
        )
    }

    /// Resolves an explicit color choice to an enabled flag.
    #[must_use]
    pub fn is_enabled(choice: ColorChoice) -> bool {
        match choice {
            ColorChoice::Auto => Self::supports_color(),
            ColorChoice::Always => true,
            ColorChoice::Never => false,
        }
    }

    /// Colors text red.
    #[must_use]
    pub fn red(text: impl AsRef<str>, choice: ColorChoice) -> String {
        colorize(text.as_ref(), "31", choice)
    }

    /// Colors text green.
    #[must_use]
    pub fn green(text: impl AsRef<str>, choice: ColorChoice) -> String {
        colorize(text.as_ref(), "32", choice)
    }

    /// Colors text yellow.
    #[must_use]
    pub fn yellow(text: impl AsRef<str>, choice: ColorChoice) -> String {
        colorize(text.as_ref(), "33", choice)
    }

    /// Colors text blue.
    #[must_use]
    pub fn blue(text: impl AsRef<str>, choice: ColorChoice) -> String {
        colorize(text.as_ref(), "34", choice)
    }

    /// Colors text cyan.
    #[must_use]
    pub fn cyan(text: impl AsRef<str>, choice: ColorChoice) -> String {
        colorize(text.as_ref(), "36", choice)
    }

    /// Makes text bold.
    #[must_use]
    pub fn bold(text: impl AsRef<str>, choice: ColorChoice) -> String {
        colorize(text.as_ref(), "1", choice)
    }

    /// Makes text dim.
    #[must_use]
    pub fn dim(text: impl AsRef<str>, choice: ColorChoice) -> String {
        colorize(text.as_ref(), "2", choice)
    }

    /// Makes text bold and red without nested reset sequences.
    #[must_use]
    pub fn red_bold(text: impl AsRef<str>, choice: ColorChoice) -> String {
        colorize(text.as_ref(), "1;31", choice)
    }

    /// Makes text bold and green without nested reset sequences.
    #[must_use]
    pub fn green_bold(text: impl AsRef<str>, choice: ColorChoice) -> String {
        colorize(text.as_ref(), "1;32", choice)
    }
}

fn colorize(text: &str, code: &str, choice: ColorChoice) -> String {
    if ColorUtils::is_enabled(choice) {
        format!("\x1b[{code}m{text}{RESET}")
    } else {
        text.to_owned()
    }
}

fn automatic_color_supported(
    is_terminal: bool,
    no_color: bool,
    term: Option<&str>,
    ci: Option<&str>,
) -> bool {
    is_terminal
        && !no_color
        && !term.is_some_and(|value| value.eq_ignore_ascii_case("dumb"))
        && !ci.is_some_and(|value| value.eq_ignore_ascii_case("true"))
}

#[cfg(test)]
mod tests {
    use super::{ColorChoice, ColorUtils, automatic_color_supported};

    #[test]
    fn never_is_plain_and_always_emits_exact_ansi_sequences() {
        assert_eq!(ColorUtils::red("failure", ColorChoice::Never), "failure");
        assert_eq!(
            ColorUtils::red("failure", ColorChoice::Always),
            "\x1b[31mfailure\x1b[0m"
        );
        assert_eq!(
            ColorUtils::red_bold("failure", ColorChoice::Always),
            "\x1b[1;31mfailure\x1b[0m"
        );
        assert_eq!(
            ColorUtils::green_bold("success", ColorChoice::Always),
            "\x1b[1;32msuccess\x1b[0m"
        );
    }

    #[test]
    fn automatic_detection_requires_a_capable_terminal_and_respects_opt_outs() {
        assert!(automatic_color_supported(true, false, Some("xterm"), None));
        assert!(!automatic_color_supported(
            false,
            false,
            Some("xterm"),
            None
        ));
        assert!(!automatic_color_supported(true, true, Some("xterm"), None));
        assert!(!automatic_color_supported(true, false, Some("dumb"), None));
        assert!(!automatic_color_supported(
            true,
            false,
            Some("xterm"),
            Some("true")
        ));
    }

    #[test]
    fn explicit_choices_do_not_depend_on_automatic_detection() {
        assert!(ColorUtils::is_enabled(ColorChoice::Always));
        assert!(!ColorUtils::is_enabled(ColorChoice::Never));
    }
}
