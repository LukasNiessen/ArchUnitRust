use syn::{
    Attribute, Item, ItemExternCrate, ItemUse, Macro, Path, Token, Visibility,
    punctuated::Punctuated,
    spanned::Spanned,
    visit::{self, Visit},
};

use super::{ImportKind, ignore_directive::DeclarationSpan, use_tree::flatten_use_tree};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct VisitedReference {
    pub segments: Vec<String>,
    pub leading_colon: bool,
    pub kind: ImportKind,
    pub line: usize,
    pub declaration: Option<DeclarationSpan>,
}

pub(crate) fn references_in_item(item: &Item) -> Vec<VisitedReference> {
    let mut visitor = ReferenceVisitor::default();
    visitor.visit_item(item);
    visitor.references
}

pub(crate) fn references_in_attributes(attributes: &[Attribute]) -> Vec<VisitedReference> {
    let mut visitor = ReferenceVisitor::default();
    for attribute in attributes {
        visitor.visit_attribute(attribute);
    }
    visitor.references
}

#[derive(Default)]
struct ReferenceVisitor {
    references: Vec<VisitedReference>,
}

impl ReferenceVisitor {
    fn record_path(&mut self, path: &Path, kind: ImportKind) {
        let segments = path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>();
        if segments.len() < 2 {
            return;
        }
        self.references.push(VisitedReference {
            segments,
            leading_colon: path.leading_colon.is_some(),
            kind,
            line: path.span().start().line.max(1),
            declaration: None,
        });
    }

    fn record_macro(&mut self, node: &Macro) {
        self.record_path(&node.path, ImportKind::MacroReference);
    }
}

impl<'ast> Visit<'ast> for ReferenceVisitor {
    fn visit_item_use(&mut self, item: &'ast ItemUse) {
        for attribute in &item.attrs {
            self.visit_attribute(attribute);
        }
        let kind = if matches!(item.vis, Visibility::Inherited) {
            ImportKind::Use
        } else {
            ImportKind::PubUse
        };
        let line = item.span().start().line.max(1);
        let declaration = Some(DeclarationSpan::new(item.span()));
        for flattened in flatten_use_tree(&item.tree) {
            self.references.push(VisitedReference {
                segments: flattened.segments,
                leading_colon: item.leading_colon.is_some(),
                kind,
                line,
                declaration,
            });
        }
    }

    fn visit_item_extern_crate(&mut self, item: &'ast ItemExternCrate) {
        for attribute in &item.attrs {
            self.visit_attribute(attribute);
        }
        self.references.push(VisitedReference {
            segments: vec![item.ident.to_string()],
            leading_colon: false,
            kind: ImportKind::ExternCrate,
            line: item.span().start().line.max(1),
            declaration: Some(DeclarationSpan::new(item.span())),
        });
    }

    fn visit_path(&mut self, path: &'ast Path) {
        self.record_path(path, ImportKind::PathReference);
        visit::visit_path(self, path);
    }

    fn visit_macro(&mut self, node: &'ast Macro) {
        self.record_macro(node);
    }

    fn visit_attribute(&mut self, attribute: &'ast Attribute) {
        if attribute.path().is_ident("derive") {
            if let Ok(paths) =
                attribute.parse_args_with(Punctuated::<Path, Token![,]>::parse_terminated)
            {
                for path in paths {
                    self.record_path(&path, ImportKind::MacroReference);
                }
            }
            return;
        }

        self.record_path(attribute.path(), ImportKind::MacroReference);
    }
}

#[cfg(test)]
mod tests {
    use syn::Item;

    use super::references_in_item;
    use crate::common::ImportKind;

    #[test]
    fn visits_qualified_paths_in_signatures_bodies_patterns_impls_and_attributes() {
        let source = r#"
            #[serde::instrument]
            impl crate::domain::Port for crate::api::Handler {
                fn call(
                    input: std::collections::HashMap<String, crate::domain::Value>,
                ) -> core::option::Option<crate::domain::Value> {
                    let crate::domain::Marker::Active = crate::domain::marker();
                    tokio::join!(crate::api::work());
                    None
                }
            }
        "#;
        let item = syn::parse_str::<Item>(source).expect("fixture item should parse");
        let references = references_in_item(&item);
        let paths = references
            .iter()
            .map(|reference| reference.segments.join("::"))
            .collect::<Vec<_>>();

        for expected in [
            "serde::instrument",
            "crate::domain::Port",
            "crate::api::Handler",
            "std::collections::HashMap",
            "crate::domain::Value",
            "core::option::Option",
            "crate::domain::Marker::Active",
            "crate::domain::marker",
            "tokio::join",
        ] {
            assert!(
                paths.contains(&expected.to_owned()),
                "missing path {expected}"
            );
        }
        assert!(references.iter().any(|reference| {
            reference.segments == ["tokio", "join"] && reference.kind == ImportKind::MacroReference
        }));
    }

    #[test]
    fn records_qualified_derive_paths_as_macro_references() {
        let item = syn::parse_str::<Item>(
            "#[derive(serde::Serialize, crate::macros::Local)] struct Value;",
        )
        .expect("fixture item should parse");
        let references = references_in_item(&item);

        assert!(references.iter().any(|reference| {
            reference.segments == ["serde", "Serialize"]
                && reference.kind == ImportKind::MacroReference
        }));
        assert!(references.iter().any(|reference| {
            reference.segments == ["crate", "macros", "Local"]
                && reference.kind == ImportKind::MacroReference
        }));
    }
}
