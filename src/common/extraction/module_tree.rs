use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use syn::{
    Attribute, Expr, Item, ItemExternCrate, ItemMod, ItemUse, Lit, Meta, Visibility,
    spanned::Spanned,
};

use super::{
    CargoProject, ExtractionDiagnostic, ExtractionDiagnosticKind, ImportKind, SourceFile,
    SourceOptions,
    dependency::{LogicalModule, RawReference},
    reference_visitor::references_in_attributes,
    reference_visitor::references_in_item,
    use_tree::flatten_use_tree,
};

pub(crate) struct RawDependencyExtraction {
    pub index: BTreeMap<LogicalModule, String>,
    pub references: Vec<RawReference>,
    pub diagnostics: Vec<ExtractionDiagnostic>,
}

pub(crate) fn extract_raw_dependencies(
    project: &CargoProject,
    options: SourceOptions,
    sources: &[SourceFile],
) -> RawDependencyExtraction {
    let available = sources
        .iter()
        .map(|source| (source.path().to_path_buf(), source.identifier().to_owned()))
        .collect();
    let mut extractor = ModuleExtractor {
        available,
        index: BTreeMap::new(),
        references: Vec::new(),
        diagnostics: Vec::new(),
        active_files: Vec::new(),
    };

    for target in project.source_targets(options) {
        let source = target.source();
        if !extractor.available.contains_key(source.path()) {
            continue;
        }
        let target_id = format!(
            "{}::{}@{}",
            target.package(),
            target.name(),
            source.identifier()
        );
        let module = LogicalModule {
            package: target.package().to_owned(),
            dependency_scope: target.dependency_scope(),
            target: target_id,
            segments: Vec::new(),
        };
        let module_directory = source
            .path()
            .parent()
            .map_or_else(PathBuf::new, Path::to_path_buf);
        extractor.walk_source(
            source.path(),
            source.identifier(),
            module,
            module_directory,
            None,
        );
    }

    RawDependencyExtraction {
        index: extractor.index,
        references: extractor.references,
        diagnostics: extractor.diagnostics,
    }
}

struct ModuleExtractor {
    available: BTreeMap<PathBuf, String>,
    index: BTreeMap<LogicalModule, String>,
    references: Vec<RawReference>,
    diagnostics: Vec<ExtractionDiagnostic>,
    active_files: Vec<PathBuf>,
}

impl ModuleExtractor {
    fn walk_source(
        &mut self,
        path: &Path,
        source: &str,
        module: LogicalModule,
        module_directory: PathBuf,
        declaration_line: Option<usize>,
    ) {
        if self.active_files.iter().any(|active| active == path) {
            self.diagnostics.push(ExtractionDiagnostic::new(
                source,
                declaration_line,
                ExtractionDiagnosticKind::ModuleCycle,
                Some(module.segments.join("::")),
                vec![source.to_owned()],
                None,
            ));
            return;
        }
        if !self.insert_module(&module, source, declaration_line) {
            return;
        }

        let contents = match fs::read_to_string(path) {
            Ok(contents) => contents,
            Err(error) => {
                self.diagnostics.push(ExtractionDiagnostic::new(
                    source,
                    declaration_line,
                    ExtractionDiagnosticKind::ReadFile,
                    None,
                    Vec::new(),
                    Some(error.to_string()),
                ));
                return;
            }
        };
        let parsed = match syn::parse_file(&contents) {
            Ok(parsed) => parsed,
            Err(error) => {
                self.diagnostics.push(ExtractionDiagnostic::new(
                    source,
                    Some(error.span().start().line.max(1)),
                    ExtractionDiagnosticKind::ParseFile,
                    None,
                    Vec::new(),
                    Some(error.to_string()),
                ));
                return;
            }
        };

        self.active_files.push(path.to_path_buf());
        self.walk_items(
            &parsed.items,
            path,
            source,
            &module,
            &module_directory,
            false,
        );
        self.active_files.pop();
    }

    fn insert_module(&mut self, module: &LogicalModule, source: &str, line: Option<usize>) -> bool {
        if let Some(existing) = self.index.get(module) {
            if existing == source {
                return false;
            }
            self.diagnostics.push(ExtractionDiagnostic::new(
                source,
                line,
                ExtractionDiagnosticKind::AmbiguousModule,
                Some(module.segments.join("::")),
                vec![existing.clone(), source.to_owned()],
                None,
            ));
            return false;
        }
        self.index.insert(module.clone(), source.to_owned());
        true
    }

    fn walk_items(
        &mut self,
        items: &[Item],
        source_path: &Path,
        source: &str,
        module: &LogicalModule,
        module_directory: &Path,
        inside_inline_module: bool,
    ) {
        for item in items {
            match item {
                Item::Mod(item_mod) => self.walk_module_item(
                    item_mod,
                    source_path,
                    source,
                    module,
                    module_directory,
                    inside_inline_module,
                ),
                Item::Use(item_use) => {
                    self.collect_attributes(&item_use.attrs, source, module);
                    self.collect_use(item_use, source, module);
                }
                Item::ExternCrate(item_extern) => {
                    self.collect_attributes(&item_extern.attrs, source, module);
                    self.collect_extern_crate(item_extern, source, module);
                }
                item => {
                    for reference in references_in_item(item) {
                        self.references.push(RawReference {
                            source: source.to_owned(),
                            module: module.clone(),
                            segments: reference.segments,
                            leading_colon: reference.leading_colon,
                            kind: reference.kind,
                            line: reference.line,
                            binding: None,
                        });
                    }
                }
            }
        }
    }

    fn walk_module_item(
        &mut self,
        item: &ItemMod,
        source_path: &Path,
        source: &str,
        module: &LogicalModule,
        module_directory: &Path,
        inside_inline_module: bool,
    ) {
        self.collect_attributes(&item.attrs, source, module);
        let line = item.span().start().line.max(1);
        let name = item.ident.to_string();
        let mut child_segments = module.segments.clone();
        child_segments.push(name.clone());
        let child_module = LogicalModule {
            package: module.package.clone(),
            dependency_scope: module.dependency_scope,
            target: module.target.clone(),
            segments: child_segments,
        };
        let mut absolute_path = vec!["crate".to_owned()];
        absolute_path.extend(child_module.segments.iter().cloned());
        self.references.push(RawReference {
            source: source.to_owned(),
            module: module.clone(),
            segments: absolute_path,
            leading_colon: false,
            kind: ImportKind::Mod,
            line,
            binding: Some(name.clone()),
        });

        let path_attribute = self.path_attribute(&item.attrs, source, line);
        if let Some((_, inline_items)) = &item.content {
            if !self.insert_module(&child_module, source, Some(line)) {
                return;
            }
            let child_directory = match path_attribute {
                PathAttribute::Literal(path) => source_path
                    .parent()
                    .map_or_else(PathBuf::new, |parent| parent.join(path)),
                PathAttribute::Absent | PathAttribute::Invalid => module_directory.join(&name),
            };
            self.walk_items(
                inline_items,
                source_path,
                source,
                &child_module,
                &child_directory,
                true,
            );
            return;
        }
        if matches!(path_attribute, PathAttribute::Invalid) {
            return;
        }

        let candidates = match path_attribute {
            PathAttribute::Literal(relative) => {
                let candidate = if inside_inline_module {
                    module_directory.join(&relative)
                } else {
                    source_path
                        .parent()
                        .map_or_else(|| relative.clone(), |parent| parent.join(&relative))
                };
                vec![canonical_if_existing(candidate)]
            }
            PathAttribute::Absent => vec![
                module_directory.join(format!("{name}.rs")),
                module_directory.join(&name).join("mod.rs"),
            ],
            PathAttribute::Invalid => Vec::new(),
        };
        let matches = candidates
            .iter()
            .filter_map(|candidate| {
                self.available
                    .get(candidate)
                    .map(|identifier| (candidate.clone(), identifier.clone()))
            })
            .collect::<Vec<_>>();

        match matches.as_slice() {
            [(path, identifier)] => {
                self.walk_source(
                    path,
                    identifier,
                    child_module,
                    module_directory_for_file(path),
                    Some(line),
                );
            }
            [] => self.diagnostics.push(ExtractionDiagnostic::new(
                source,
                Some(line),
                ExtractionDiagnosticKind::MissingModule,
                Some(name),
                candidates.iter().map(|path| display_path(path)).collect(),
                None,
            )),
            _ => self.diagnostics.push(ExtractionDiagnostic::new(
                source,
                Some(line),
                ExtractionDiagnosticKind::AmbiguousModule,
                Some(name),
                matches
                    .iter()
                    .map(|(_, identifier)| identifier.clone())
                    .collect(),
                None,
            )),
        }
    }

    fn collect_use(&mut self, item: &ItemUse, source: &str, module: &LogicalModule) {
        let kind = if matches!(item.vis, Visibility::Inherited) {
            ImportKind::Use
        } else {
            ImportKind::PubUse
        };
        let line = item.span().start().line.max(1);
        for flattened in flatten_use_tree(&item.tree) {
            self.references.push(RawReference {
                source: source.to_owned(),
                module: module.clone(),
                segments: flattened.segments,
                leading_colon: item.leading_colon.is_some(),
                kind,
                line,
                binding: flattened.binding,
            });
        }
    }

    fn collect_extern_crate(
        &mut self,
        item: &ItemExternCrate,
        source: &str,
        module: &LogicalModule,
    ) {
        self.references.push(RawReference {
            source: source.to_owned(),
            module: module.clone(),
            segments: vec![item.ident.to_string()],
            leading_colon: false,
            kind: ImportKind::ExternCrate,
            line: item.span().start().line.max(1),
            binding: Some(
                item.rename
                    .as_ref()
                    .map_or_else(|| item.ident.to_string(), |(_, rename)| rename.to_string()),
            ),
        });
    }

    fn collect_attributes(
        &mut self,
        attributes: &[Attribute],
        source: &str,
        module: &LogicalModule,
    ) {
        for reference in references_in_attributes(attributes) {
            self.references.push(RawReference {
                source: source.to_owned(),
                module: module.clone(),
                segments: reference.segments,
                leading_colon: reference.leading_colon,
                kind: reference.kind,
                line: reference.line,
                binding: None,
            });
        }
    }

    fn path_attribute(
        &mut self,
        attributes: &[Attribute],
        source: &str,
        line: usize,
    ) -> PathAttribute {
        let Some(attribute) = attributes
            .iter()
            .find(|attribute| attribute.path().is_ident("path"))
        else {
            return PathAttribute::Absent;
        };
        if let Meta::NameValue(name_value) = &attribute.meta {
            if let Expr::Lit(expression) = &name_value.value {
                if let Lit::Str(path) = &expression.lit {
                    return PathAttribute::Literal(PathBuf::from(path.value()));
                }
            }
        }

        self.diagnostics.push(ExtractionDiagnostic::new(
            source,
            Some(line),
            ExtractionDiagnosticKind::InvalidPathAttribute,
            Some("path".to_owned()),
            Vec::new(),
            None,
        ));
        PathAttribute::Invalid
    }
}

enum PathAttribute {
    Absent,
    Literal(PathBuf),
    Invalid,
}

fn canonical_if_existing(path: PathBuf) -> PathBuf {
    match fs::canonicalize(&path) {
        Ok(canonical) => canonical,
        Err(_) => path,
    }
}

fn module_directory_for_file(path: &Path) -> PathBuf {
    let parent = path.parent().map_or_else(PathBuf::new, Path::to_path_buf);
    if path.file_name().is_some_and(|name| name == "mod.rs") {
        return parent;
    }
    path.file_stem()
        .map_or(parent.clone(), |stem| parent.join(stem))
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
