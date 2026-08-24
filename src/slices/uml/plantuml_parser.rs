use super::{PlantUmlDependency, PlantUmlDiagram, PlantUmlError};

/// Line-based parser for the supported PlantUML component-diagram subset.
#[derive(Debug, Clone, Copy, Default)]
pub struct PlantUmlParser;

impl PlantUmlParser {
    /// Parses component declarations, directed arrows, comments, and PlantUML directives.
    pub fn parse(text: &str) -> Result<PlantUmlDiagram, PlantUmlError> {
        if text.trim().is_empty() {
            return Err(PlantUmlError::new("diagram text must not be empty"));
        }

        let mut components = Vec::new();
        let mut dependencies = Vec::new();
        for (index, original_line) in text.lines().enumerate() {
            let line_number = index + 1;
            let line = uncommented(original_line);
            if line.is_empty() || line.starts_with('@') {
                continue;
            }

            if starts_with_keyword(line, "component") {
                components.push(bracketed_component(line, line_number)?);
            } else if line.starts_with('[') {
                dependencies.push(parse_dependency(line, line_number)?);
            }
        }

        PlantUmlDiagram::new(components, dependencies)
    }
}

fn uncommented(line: &str) -> &str {
    let line = line.trim();
    if line.starts_with('\'') || line.starts_with("//") {
        return "";
    }
    line.split_once('\'')
        .map_or(line, |(before, _comment)| before.trim())
}

fn starts_with_keyword(line: &str, keyword: &str) -> bool {
    line.get(..keyword.len())
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(keyword))
        && line
            .get(keyword.len()..)
            .is_some_and(|rest| rest.starts_with(char::is_whitespace))
}

fn bracketed_component(line: &str, line_number: usize) -> Result<String, PlantUmlError> {
    let Some(open) = line.find('[') else {
        return Err(PlantUmlError::at_line(
            line_number,
            "component declaration must contain '[name]'",
        ));
    };
    let Some(relative_close) = line[open + 1..].find(']') else {
        return Err(PlantUmlError::at_line(
            line_number,
            "component declaration must close its name with ']'",
        ));
    };
    let name = line[open + 1..open + 1 + relative_close].trim();
    if name.is_empty() {
        return Err(PlantUmlError::at_line(
            line_number,
            "component name must not be empty",
        ));
    }
    Ok(name.to_owned())
}

fn parse_dependency(line: &str, line_number: usize) -> Result<PlantUmlDependency, PlantUmlError> {
    let (source, rest) = take_bracketed(line, line_number, "source")?;
    let rest = rest.trim_start();
    let rest = rest
        .strip_prefix("-->")
        .or_else(|| rest.strip_prefix("->"))
        .ok_or_else(|| PlantUmlError::at_line(line_number, "dependency must use '->' or '-->'"))?;
    let (target, _rest) = take_bracketed(rest.trim_start(), line_number, "target")?;
    PlantUmlDependency::new(source, target)
        .map_err(|error| PlantUmlError::at_line(line_number, error.message()))
}

fn take_bracketed<'a>(
    value: &'a str,
    line_number: usize,
    endpoint: &str,
) -> Result<(&'a str, &'a str), PlantUmlError> {
    let Some(value) = value.strip_prefix('[') else {
        return Err(PlantUmlError::at_line(
            line_number,
            format!("dependency {endpoint} must start with '['"),
        ));
    };
    let Some(close) = value.find(']') else {
        return Err(PlantUmlError::at_line(
            line_number,
            format!("dependency {endpoint} must end with ']'"),
        ));
    };
    let name = value[..close].trim();
    if name.is_empty() {
        return Err(PlantUmlError::at_line(
            line_number,
            format!("dependency {endpoint} must not be empty"),
        ));
    }
    Ok((name, &value[close + 1..]))
}

#[cfg(test)]
mod tests {
    use super::PlantUmlParser;

    #[test]
    fn parses_declarations_both_arrows_comments_directives_and_implicit_components() {
        let diagram = PlantUmlParser::parse(
            "@startuml Architecture\n\
             ' comment\n\
             // comment\n\
             component [api] #Green\n\
             component [services]\n\
             [api] -> [services]\n\
             [services] --> [models] ' inline\n\
             [api] -> [services]\n\
             skinparam componentStyle rectangle\n\
             @enduml",
        )
        .expect("fixture diagram should parse");

        assert_eq!(diagram.components, ["api", "services", "models"]);
        assert_eq!(diagram.dependencies.len(), 2);
        assert!(diagram.allows("api", "services"));
        assert!(diagram.allows("services", "models"));
        assert!(!diagram.allows("models", "api"));
    }

    #[test]
    fn rejects_empty_diagrams_and_malformed_recognized_lines_with_location() {
        assert!(PlantUmlParser::parse("").is_err());
        for (text, expected_line) in [
            ("@startuml\ncomponent api\n@enduml", 2),
            ("@startuml\n[api] -- [database]\n@enduml", 2),
            ("@startuml\n[api] -> []\n@enduml", 2),
        ] {
            let error = PlantUmlParser::parse(text).expect_err("diagram should be rejected");
            assert_eq!(error.line(), Some(expected_line));
        }
    }
}
