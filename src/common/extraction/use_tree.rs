use syn::UseTree;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FlattenedUse {
    pub segments: Vec<String>,
    pub binding: Option<String>,
}

pub(crate) fn flatten_use_tree(tree: &UseTree) -> Vec<FlattenedUse> {
    let mut flattened = Vec::new();
    flatten(tree, Vec::new(), &mut flattened);
    flattened
}

fn flatten(tree: &UseTree, prefix: Vec<String>, flattened: &mut Vec<FlattenedUse>) {
    match tree {
        UseTree::Path(path) => {
            let mut prefix = prefix;
            prefix.push(path.ident.to_string());
            flatten(&path.tree, prefix, flattened);
        }
        UseTree::Name(name) => {
            if name.ident == "self" {
                let binding = prefix.last().cloned();
                if !prefix.is_empty() {
                    flattened.push(FlattenedUse {
                        segments: prefix,
                        binding,
                    });
                }
            } else {
                let binding = name.ident.to_string();
                let mut segments = prefix;
                segments.push(binding.clone());
                flattened.push(FlattenedUse {
                    segments,
                    binding: Some(binding),
                });
            }
        }
        UseTree::Rename(rename) => {
            let mut segments = prefix;
            if rename.ident != "self" {
                segments.push(rename.ident.to_string());
            }
            if !segments.is_empty() {
                flattened.push(FlattenedUse {
                    segments,
                    binding: Some(rename.rename.to_string()),
                });
            }
        }
        UseTree::Glob(_) => {
            if !prefix.is_empty() {
                flattened.push(FlattenedUse {
                    segments: prefix,
                    binding: None,
                });
            }
        }
        UseTree::Group(group) => {
            for item in &group.items {
                flatten(item, prefix.clone(), flattened);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use syn::ItemUse;

    use super::{FlattenedUse, flatten_use_tree};

    fn flatten(source: &str) -> Vec<FlattenedUse> {
        let item = syn::parse_str::<ItemUse>(source).expect("fixture use should parse");
        flatten_use_tree(&item.tree)
    }

    #[test]
    fn flattens_groups_globs_self_and_aliases() {
        assert_eq!(
            flatten("use crate::api::{self as public_api, model::{User, *}};"),
            vec![
                FlattenedUse {
                    segments: vec!["crate".to_owned(), "api".to_owned()],
                    binding: Some("public_api".to_owned()),
                },
                FlattenedUse {
                    segments: vec![
                        "crate".to_owned(),
                        "api".to_owned(),
                        "model".to_owned(),
                        "User".to_owned(),
                    ],
                    binding: Some("User".to_owned()),
                },
                FlattenedUse {
                    segments: vec!["crate".to_owned(), "api".to_owned(), "model".to_owned(),],
                    binding: None,
                },
            ]
        );
    }

    #[test]
    fn plain_names_bind_the_final_segment() {
        assert_eq!(
            flatten("use std::collections::HashMap;"),
            vec![FlattenedUse {
                segments: vec![
                    "std".to_owned(),
                    "collections".to_owned(),
                    "HashMap".to_owned(),
                ],
                binding: Some("HashMap".to_owned()),
            }]
        );
    }
}
