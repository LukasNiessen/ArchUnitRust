pub(super) fn quoted(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len() + 2);
    escaped.push('"');
    for character in value.chars() {
        match character {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            character => escaped.push(character),
        }
    }
    escaped.push('"');
    escaped
}

pub(super) fn html(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            character => escaped.push(character),
        }
    }
    escaped
}

pub(super) fn mermaid_label(value: &str) -> String {
    html(value)
        .replace("\r\n", "<br/>")
        .replace(['\r', '\n'], "<br/>")
}

pub(super) fn single_line(value: &str) -> String {
    value.replace(['\r', '\n'], " ")
}

pub(super) fn csv_field(value: &str) -> String {
    if value.contains([',', '"', '\r', '\n']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::{csv_field, html, mermaid_label, quoted, single_line};

    #[test]
    fn quoted_syntax_escapes_controls_backslashes_and_quotes() {
        assert_eq!(quoted("a\\\"b\n"), "\"a\\\\\\\"b\\n\"");
    }

    #[test]
    fn html_and_mermaid_escape_markup_before_preserving_line_breaks() {
        assert_eq!(html("<&\"'>"), "&lt;&amp;&quot;&#39;&gt;");
        assert_eq!(mermaid_label("a<&\nb"), "a&lt;&amp;<br/>b");
        assert_eq!(single_line("a\r\nb"), "a  b");
    }

    #[test]
    fn csv_quotes_only_fields_that_require_it_and_doubles_quotes() {
        assert_eq!(csv_field("plain"), "plain");
        assert_eq!(csv_field("a,\"b\""), "\"a,\"\"b\"\"\"");
    }
}
