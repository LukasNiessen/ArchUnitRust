use std::collections::{BTreeMap, BTreeSet};

use proc_macro2::Span;
use syn::{
    File, ItemExternCrate, ItemMod, ItemUse,
    spanned::Spanned,
    visit::{self, Visit},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct DeclarationSpan {
    start_line: usize,
    start_column: usize,
    end_line: usize,
    end_column: usize,
}

impl DeclarationSpan {
    pub(crate) fn new(span: Span) -> Self {
        let start = span.start();
        let end = span.end();
        Self {
            start_line: start.line,
            start_column: start.column,
            end_line: end.line,
            end_column: end.column,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct IgnoredDeclarations {
    directives: BTreeMap<DeclarationSpan, IgnoreDirective>,
}

impl IgnoredDeclarations {
    pub(crate) fn from_source(source: &str, parsed: &File) -> Self {
        let mut collector = DeclarationCollector::default();
        collector.visit_file(parsed);
        collector.declarations.sort();
        collector.declarations.dedup();

        let mut directives = BTreeMap::<DeclarationSpan, IgnoreDirective>::new();
        for directive in located_directives(source) {
            let declaration = if directive.trailing {
                collector
                    .declarations
                    .iter()
                    .filter(|declaration| {
                        declaration.end_line == directive.line
                            && declaration.end_column <= directive.column
                    })
                    .max_by_key(|declaration| declaration.end_column)
            } else {
                collector
                    .declarations
                    .iter()
                    .filter(|declaration| declaration.start_line == directive.line + 1)
                    .min_by_key(|declaration| declaration.start_column)
            };
            if let Some(declaration) = declaration {
                directives
                    .entry(*declaration)
                    .or_default()
                    .merge(directive.scopes);
            }
        }

        Self { directives }
    }

    pub(crate) fn ignores(&self, declaration: Option<DeclarationSpan>, path: &str) -> bool {
        declaration
            .and_then(|declaration| self.directives.get(&declaration))
            .is_some_and(|directive| directive.matches(path))
    }
}

#[derive(Debug, Clone, Default)]
struct IgnoreDirective {
    scopes: BTreeSet<String>,
    unscoped: bool,
}

impl IgnoreDirective {
    fn merge(&mut self, scopes: BTreeSet<String>) {
        if scopes.is_empty() {
            self.unscoped = true;
            self.scopes.clear();
        } else if !self.unscoped {
            self.scopes.extend(scopes);
        }
    }

    fn matches(&self, path: &str) -> bool {
        self.unscoped
            || self.scopes.iter().any(|scope| {
                let path = normalized_path(path);
                let scope = normalized_path(scope);
                path == scope || path.starts_with(&format!("{scope}::"))
            })
    }
}

#[derive(Debug)]
struct LocatedDirective {
    line: usize,
    column: usize,
    trailing: bool,
    scopes: BTreeSet<String>,
}

fn located_directives(source: &str) -> Vec<LocatedDirective> {
    source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let column = line.find("//")?;
            let scopes = directive_scopes(&line[column + 2..])?;
            Some(LocatedDirective {
                line: index + 1,
                column,
                trailing: !line[..column].trim().is_empty(),
                scopes,
            })
        })
        .collect()
}

fn directive_scopes(comment: &str) -> Option<BTreeSet<String>> {
    let directive = comment.trim().strip_prefix("archunit:")?.trim_start();
    let scopes = directive.strip_prefix("ignore")?;
    if scopes
        .chars()
        .next()
        .is_some_and(|character| !character.is_whitespace())
    {
        return None;
    }
    Some(
        scopes
            .split(|character: char| character.is_whitespace() || character == ',')
            .filter(|scope| !scope.is_empty())
            .map(ToOwned::to_owned)
            .collect(),
    )
}

fn normalized_path(path: &str) -> &str {
    let mut path = path.trim_start_matches("::");
    loop {
        let stripped = ["crate::", "self::", "super::"]
            .into_iter()
            .find_map(|prefix| path.strip_prefix(prefix));
        match stripped {
            Some(stripped) => path = stripped,
            None => return path,
        }
    }
}

#[derive(Default)]
struct DeclarationCollector {
    declarations: Vec<DeclarationSpan>,
}

impl<'ast> Visit<'ast> for DeclarationCollector {
    fn visit_item_use(&mut self, item: &'ast ItemUse) {
        self.declarations.push(DeclarationSpan::new(item.span()));
    }

    fn visit_item_extern_crate(&mut self, item: &'ast ItemExternCrate) {
        self.declarations.push(DeclarationSpan::new(item.span()));
    }

    fn visit_item_mod(&mut self, item: &'ast ItemMod) {
        self.declarations.push(DeclarationSpan::new(item.span()));
        visit::visit_item_mod(self, item);
    }
}

#[cfg(test)]
mod tests {
    use syn::spanned::Spanned;

    use super::{DeclarationSpan, IgnoredDeclarations};

    fn ignored_paths(source: &str, paths: &[&str]) -> Vec<bool> {
        let parsed = syn::parse_file(source).expect("fixture source should parse");
        let directives = IgnoredDeclarations::from_source(source, &parsed);
        parsed
            .items
            .iter()
            .zip(paths)
            .map(|(item, path)| directives.ignores(Some(DeclarationSpan::new(item.span())), path))
            .collect()
    }

    #[test]
    fn attaches_inline_and_preceding_directives_with_lf_or_crlf() {
        for newline in ["\n", "\r\n"] {
            let source = [
                "use ignored_inline::Thing; // archunit: ignore",
                "// archunit: ignore",
                "extern crate ignored_preceding;",
                "use kept::Thing;",
            ]
            .join(newline);

            assert_eq!(
                ignored_paths(
                    &source,
                    &["ignored_inline::Thing", "ignored_preceding", "kept::Thing"]
                ),
                [true, true, false]
            );
        }
    }

    #[test]
    fn scopes_grouped_paths_and_rejects_lookalike_comments() {
        let source = concat!(
            "use grouped::{Ignored, Kept}; // archunit: ignore grouped::Ignored\n",
            "use mismatch::Thing; // archunit: ignore something_else\n",
            "use lookalike::Thing; // archunit ignore\n",
        );
        let parsed = syn::parse_file(source).expect("fixture source should parse");
        let directives = IgnoredDeclarations::from_source(source, &parsed);
        let grouped = DeclarationSpan::new(parsed.items[0].span());

        assert!(directives.ignores(Some(grouped), "grouped::Ignored"));
        assert!(directives.ignores(Some(grouped), "grouped::Ignored::Nested"));
        assert!(!directives.ignores(Some(grouped), "grouped::Kept"));
        assert!(!directives.ignores(
            Some(DeclarationSpan::new(parsed.items[1].span())),
            "mismatch::Thing"
        ));
        assert!(!directives.ignores(
            Some(DeclarationSpan::new(parsed.items[2].span())),
            "lookalike::Thing"
        ));

        let separated = "// archunit: ignore\n\nuse separated::Thing;\n";
        assert_eq!(ignored_paths(separated, &["separated::Thing"]), [false]);
    }

    #[test]
    fn one_trailing_directive_attaches_only_to_the_nearest_declaration() {
        let source = "use first::Thing; use second::Thing; // archunit: ignore\n";
        let parsed = syn::parse_file(source).expect("fixture source should parse");
        let directives = IgnoredDeclarations::from_source(source, &parsed);

        assert!(!directives.ignores(
            Some(DeclarationSpan::new(parsed.items[0].span())),
            "first::Thing"
        ));
        assert!(directives.ignores(
            Some(DeclarationSpan::new(parsed.items[1].span())),
            "second::Thing"
        ));
    }
}
