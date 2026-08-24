use std::{collections::BTreeSet, fs};

use syn::{
    Expr, Fields, FnArg, ImplItem, Item, Member, Stmt, TraitItem, Type,
    visit::{self, Visit},
};

use super::{
    FieldInfo, FileMetricsInfo, ImplInfo, MethodInfo, ProjectMetricsInfo, TypeInfo, TypeKind,
};
use crate::common::{
    ArchUnitError, CargoProject, SourceOptions, TechnicalError, enumerate_source_files,
};

/// A syntax failure while extracting metrics from one source input.
#[derive(Debug, thiserror::Error)]
#[error("could not parse Rust metrics source {identifier}: {source}")]
pub struct MetricsExtractionError {
    identifier: String,
    #[source]
    source: syn::Error,
}

impl MetricsExtractionError {
    /// Returns the source identifier supplied to extraction.
    #[must_use]
    pub fn identifier(&self) -> &str {
        &self.identifier
    }
}

/// Extracts count and type information from one Rust source string.
///
/// The identifier is retained as supplied and should normally be a normalized workspace-relative
/// source path.
pub fn extract_file_metrics(
    identifier: impl Into<String>,
    source: &str,
) -> Result<FileMetricsInfo, MetricsExtractionError> {
    let mut file = extract_file_metrics_unassociated(identifier.into(), source)?;
    associate_impls(std::slice::from_mut(&mut file));
    Ok(file)
}

fn extract_file_metrics_unassociated(
    identifier: String,
    source: &str,
) -> Result<FileMetricsInfo, MetricsExtractionError> {
    let syntax = syn::parse_file(source).map_err(|source| MetricsExtractionError {
        identifier: identifier.clone(),
        source,
    })?;

    let mut counts = CountVisitor::default();
    counts.visit_file(&syntax);
    let mut types = TypeCollector::new(&identifier);
    types.visit_file(&syntax);

    let associated_functions = types
        .impls
        .iter()
        .map(|impl_info| impl_info.associated_functions.len())
        .sum::<usize>()
        + types
            .types
            .iter()
            .filter(|type_info| type_info.kind == TypeKind::Trait)
            .map(|type_info| type_info.associated_functions.len())
            .sum::<usize>();

    Ok(FileMetricsInfo {
        path: identifier.clone(),
        lines_of_code: count_lines_of_code(source),
        statements: counts.statements,
        imports: counts.imports,
        concrete_types: counts.concrete_types,
        functions: counts.functions,
        traits: counts.traits,
        impl_blocks: counts.impl_blocks,
        macros: counts.macros,
        associated_functions,
        types: types.types,
        impls: types.impls,
    })
}

/// Extracts deterministic metrics information from every selected source in a Cargo project.
pub fn extract_project_metrics(
    project: &CargoProject,
    options: SourceOptions,
) -> Result<ProjectMetricsInfo, ArchUnitError> {
    let sources = enumerate_source_files(project, options)?;
    let mut files = Vec::with_capacity(sources.len());

    for source_file in sources {
        let source = fs::read_to_string(source_file.path()).map_err(|source| {
            ArchUnitError::from(TechnicalError::with_source(
                format!(
                    "could not read metrics source {}",
                    source_file.path().display()
                ),
                source,
            ))
        })?;
        let metrics =
            extract_file_metrics_unassociated(source_file.identifier().to_owned(), &source)
                .map_err(|source| {
                    ArchUnitError::from(TechnicalError::with_source(
                        format!(
                            "could not extract metrics from {}",
                            source_file.path().display()
                        ),
                        source,
                    ))
                })?;
        files.push(metrics);
    }

    associate_impls(&mut files);
    Ok(ProjectMetricsInfo::from_files(
        project.root().to_path_buf(),
        files,
    ))
}

#[derive(Debug, Default)]
struct CountVisitor {
    statements: usize,
    imports: usize,
    concrete_types: usize,
    functions: usize,
    traits: usize,
    impl_blocks: usize,
    macros: usize,
}

impl<'ast> Visit<'ast> for CountVisitor {
    fn visit_item(&mut self, item: &'ast Item) {
        self.statements += 1;
        visit::visit_item(self, item);
    }

    fn visit_impl_item(&mut self, item: &'ast ImplItem) {
        self.statements += 1;
        visit::visit_impl_item(self, item);
    }

    fn visit_trait_item(&mut self, item: &'ast TraitItem) {
        self.statements += 1;
        visit::visit_trait_item(self, item);
    }

    fn visit_stmt(&mut self, statement: &'ast Stmt) {
        if !matches!(statement, Stmt::Item(_)) {
            self.statements += 1;
        }
        visit::visit_stmt(self, statement);
    }

    fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
        self.imports += 1;
        visit::visit_item_use(self, item);
    }

    fn visit_item_extern_crate(&mut self, item: &'ast syn::ItemExternCrate) {
        self.imports += 1;
        visit::visit_item_extern_crate(self, item);
    }

    fn visit_item_struct(&mut self, item: &'ast syn::ItemStruct) {
        self.concrete_types += 1;
        visit::visit_item_struct(self, item);
    }

    fn visit_item_enum(&mut self, item: &'ast syn::ItemEnum) {
        self.concrete_types += 1;
        visit::visit_item_enum(self, item);
    }

    fn visit_item_union(&mut self, item: &'ast syn::ItemUnion) {
        self.concrete_types += 1;
        visit::visit_item_union(self, item);
    }

    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        self.functions += 1;
        visit::visit_item_fn(self, item);
    }

    fn visit_item_trait(&mut self, item: &'ast syn::ItemTrait) {
        self.traits += 1;
        visit::visit_item_trait(self, item);
    }

    fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
        self.impl_blocks += 1;
        visit::visit_item_impl(self, item);
    }

    fn visit_macro(&mut self, item: &'ast syn::Macro) {
        self.macros += 1;
        visit::visit_macro(self, item);
    }
}

struct TypeCollector<'a> {
    file_path: &'a str,
    modules: Vec<String>,
    types: Vec<TypeInfo>,
    impls: Vec<ImplInfo>,
}

impl<'a> TypeCollector<'a> {
    fn new(file_path: &'a str) -> Self {
        Self {
            file_path,
            modules: Vec::new(),
            types: Vec::new(),
            impls: Vec::new(),
        }
    }

    fn qualified_name(&self, name: &str) -> String {
        if self.modules.is_empty() {
            name.to_owned()
        } else {
            format!("{}::{name}", self.modules.join("::"))
        }
    }

    fn qualified_target(&self, target: String) -> String {
        if target.contains("::") || self.modules.is_empty() {
            target
        } else {
            self.qualified_name(&target)
        }
    }

    fn push_type(&mut self, name: &str, kind: TypeKind, fields: Vec<FieldInfo>) {
        self.types.push(TypeInfo {
            name: self.qualified_name(name),
            file_path: self.file_path.to_owned(),
            kind,
            methods: Vec::new(),
            inherent_methods: Vec::new(),
            fields,
            associated_functions: Vec::new(),
        });
    }
}

impl<'ast> Visit<'ast> for TypeCollector<'_> {
    fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
        let Some((_, items)) = &item.content else {
            return;
        };
        self.modules.push(item.ident.to_string());
        for nested in items {
            self.visit_item(nested);
        }
        self.modules.pop();
    }

    fn visit_item_struct(&mut self, item: &'ast syn::ItemStruct) {
        self.push_type(
            &item.ident.to_string(),
            TypeKind::Struct,
            fields_from(&item.fields, None),
        );
    }

    fn visit_item_enum(&mut self, item: &'ast syn::ItemEnum) {
        let fields = item
            .variants
            .iter()
            .flat_map(|variant| fields_from(&variant.fields, Some(&variant.ident.to_string())))
            .collect();
        self.push_type(&item.ident.to_string(), TypeKind::Enum, fields);
    }

    fn visit_item_union(&mut self, item: &'ast syn::ItemUnion) {
        let fields = item
            .fields
            .named
            .iter()
            .map(|field| FieldInfo {
                name: field
                    .ident
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_default(),
                accessed_by: Vec::new(),
            })
            .collect();
        self.push_type(&item.ident.to_string(), TypeKind::Union, fields);
    }

    fn visit_item_trait(&mut self, item: &'ast syn::ItemTrait) {
        let mut methods = Vec::new();
        let mut associated_functions = Vec::new();
        for trait_item in &item.items {
            if let TraitItem::Fn(function) = trait_item {
                if has_receiver(&function.sig) {
                    methods.push(method_info(&function.sig, function.default.as_ref()));
                } else {
                    associated_functions.push(function.sig.ident.to_string());
                }
            }
        }
        methods.sort_by(|left, right| left.name.cmp(&right.name));
        associated_functions.sort();
        self.types.push(TypeInfo {
            name: self.qualified_name(&item.ident.to_string()),
            file_path: self.file_path.to_owned(),
            kind: TypeKind::Trait,
            methods,
            inherent_methods: Vec::new(),
            fields: Vec::new(),
            associated_functions,
        });
    }

    fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
        let Some(target_type) = type_name(&item.self_ty) else {
            return;
        };
        let mut methods = Vec::new();
        let mut associated_functions = Vec::new();
        for impl_item in &item.items {
            if let ImplItem::Fn(function) = impl_item {
                if has_receiver(&function.sig) {
                    methods.push(method_info(&function.sig, Some(&function.block)));
                } else {
                    associated_functions.push(function.sig.ident.to_string());
                }
            }
        }
        methods.sort_by(|left, right| left.name.cmp(&right.name));
        associated_functions.sort();
        self.impls.push(ImplInfo {
            target_type: self.qualified_target(target_type),
            trait_name: item.trait_.as_ref().map(|(_, path, _)| path_name(path)),
            file_path: self.file_path.to_owned(),
            methods,
            associated_functions,
        });
    }
}

fn fields_from(fields: &Fields, variant: Option<&str>) -> Vec<FieldInfo> {
    fields
        .iter()
        .enumerate()
        .map(|(index, field)| {
            let name = field
                .ident
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_else(|| index.to_string());
            FieldInfo {
                name: variant.map_or(name.clone(), |variant| format!("{variant}.{name}")),
                accessed_by: Vec::new(),
            }
        })
        .collect()
}

fn has_receiver(signature: &syn::Signature) -> bool {
    signature
        .inputs
        .iter()
        .any(|argument| matches!(argument, FnArg::Receiver(_)))
}

fn method_info(signature: &syn::Signature, body: Option<&syn::Block>) -> MethodInfo {
    let mut visitor = FieldAccessVisitor::default();
    if let Some(body) = body {
        visitor.visit_block(body);
    }
    MethodInfo {
        name: signature.ident.to_string(),
        accessed_fields: visitor.fields.into_iter().collect(),
        has_body: body.is_some(),
    }
}

#[derive(Default)]
struct FieldAccessVisitor {
    fields: BTreeSet<String>,
}

impl<'ast> Visit<'ast> for FieldAccessVisitor {
    fn visit_expr_field(&mut self, expression: &'ast syn::ExprField) {
        if is_self_expression(&expression.base) {
            let name = match &expression.member {
                Member::Named(identifier) => identifier.to_string(),
                Member::Unnamed(index) => index.index.to_string(),
            };
            self.fields.insert(name);
        }
        visit::visit_expr_field(self, expression);
    }
}

fn is_self_expression(expression: &Expr) -> bool {
    matches!(
        expression,
        Expr::Path(path)
            if path.qself.is_none()
                && path.path.segments.len() == 1
                && path.path.segments[0].ident == "self"
    )
}

fn type_name(type_: &Type) -> Option<String> {
    match type_ {
        Type::Path(path) if path.qself.is_none() => Some(path_name(&path.path)),
        Type::Reference(reference) => type_name(&reference.elem),
        Type::Group(group) => type_name(&group.elem),
        Type::Paren(parenthesized) => type_name(&parenthesized.elem),
        _ => None,
    }
}

fn path_name(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

fn associate_impls(files: &mut [FileMetricsInfo]) {
    let implementations = files
        .iter()
        .enumerate()
        .flat_map(|(file_index, file)| {
            file.impls
                .iter()
                .cloned()
                .map(move |impl_info| (file_index, impl_info))
        })
        .collect::<Vec<_>>();

    for (impl_file, impl_info) in implementations {
        let candidates = matching_types(files, &impl_info.target_type);
        let exact = candidates
            .iter()
            .copied()
            .filter(|&(file, index)| {
                names_match_exactly(&files[file].types[index].name, &impl_info.target_type)
            })
            .collect::<Vec<_>>();
        let same_file = candidates
            .iter()
            .copied()
            .filter(|(file, _)| *file == impl_file)
            .collect::<Vec<_>>();
        let selected = unique(&exact)
            .or_else(|| unique(&same_file))
            .or_else(|| unique(&candidates));
        let Some((file, type_index)) = selected else {
            continue;
        };
        let type_info = &mut files[file].types[type_index];
        if !impl_info.is_trait_impl() {
            type_info
                .inherent_methods
                .extend(impl_info.methods.iter().cloned());
            type_info
                .inherent_methods
                .sort_by(|left, right| left.name.cmp(&right.name));
        }
        type_info.methods.extend(impl_info.methods);
        type_info
            .associated_functions
            .extend(impl_info.associated_functions);
        type_info
            .methods
            .sort_by(|left, right| left.name.cmp(&right.name));
        type_info.associated_functions.sort();
    }

    for file in files {
        for type_info in &mut file.types {
            for field in &mut type_info.fields {
                field.accessed_by = type_info
                    .methods
                    .iter()
                    .filter(|method| method.accessed_fields.contains(&field.name))
                    .map(|method| method.name.clone())
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect();
            }
        }
    }
}

fn matching_types(files: &[FileMetricsInfo], target: &str) -> Vec<(usize, usize)> {
    let target_simple = target.rsplit("::").next();
    files
        .iter()
        .enumerate()
        .flat_map(|(file_index, file)| {
            file.types
                .iter()
                .enumerate()
                .filter(move |(_, type_info)| Some(type_info.simple_name()) == target_simple)
                .map(move |(type_index, _)| (file_index, type_index))
        })
        .collect()
}

fn names_match_exactly(declaration: &str, target: &str) -> bool {
    let target = target
        .strip_prefix("crate::")
        .or_else(|| target.strip_prefix("self::"))
        .unwrap_or(target);
    declaration == target
        || declaration
            .strip_suffix(target)
            .is_some_and(|prefix| prefix.ends_with("::"))
        || target
            .strip_suffix(declaration)
            .is_some_and(|prefix| prefix.ends_with("::"))
}

fn unique(values: &[(usize, usize)]) -> Option<(usize, usize)> {
    (values.len() == 1).then(|| values[0])
}

#[derive(Debug, Clone, Copy)]
enum LiteralState {
    String { escaped: bool },
    RawString { hashes: usize },
}

fn count_lines_of_code(source: &str) -> usize {
    let mut block_depth = 0_usize;
    let mut literal = None;
    source
        .split('\n')
        .filter(|line| {
            let characters = line.trim_end_matches('\r').chars().collect::<Vec<_>>();
            line_has_code(&characters, &mut block_depth, &mut literal)
        })
        .count()
}

fn line_has_code(
    characters: &[char],
    block_depth: &mut usize,
    literal: &mut Option<LiteralState>,
) -> bool {
    let mut has_code = literal.is_some();
    let mut index = 0;
    while index < characters.len() {
        if let Some(state) = *literal {
            match state {
                LiteralState::String { mut escaped } => {
                    let character = characters[index];
                    if escaped {
                        escaped = false;
                    } else if character == '\\' {
                        escaped = true;
                    } else if character == '"' {
                        *literal = None;
                    }
                    if literal.is_some() {
                        *literal = Some(LiteralState::String { escaped });
                    }
                    index += 1;
                    continue;
                }
                LiteralState::RawString { hashes } => {
                    if characters[index] == '"'
                        && characters
                            .get(index + 1..index + 1 + hashes)
                            .is_some_and(|suffix| suffix.iter().all(|character| *character == '#'))
                    {
                        *literal = None;
                        index += hashes + 1;
                    } else {
                        index += 1;
                    }
                    continue;
                }
            }
        }

        if *block_depth > 0 {
            if characters.get(index..index + 2) == Some(&['/', '*']) {
                *block_depth += 1;
                index += 2;
            } else if characters.get(index..index + 2) == Some(&['*', '/']) {
                *block_depth -= 1;
                index += 2;
            } else {
                index += 1;
            }
            continue;
        }

        if characters[index].is_whitespace() {
            index += 1;
        } else if characters.get(index..index + 2) == Some(&['/', '/']) {
            break;
        } else if characters.get(index..index + 2) == Some(&['/', '*']) {
            *block_depth = 1;
            index += 2;
        } else if let Some((hashes, opening_length)) = raw_string_opening(characters, index) {
            has_code = true;
            *literal = Some(LiteralState::RawString { hashes });
            index += opening_length;
        } else if characters[index] == '"' {
            has_code = true;
            *literal = Some(LiteralState::String { escaped: false });
            index += 1;
        } else {
            has_code = true;
            index += 1;
        }
    }
    has_code
}

fn raw_string_opening(characters: &[char], index: usize) -> Option<(usize, usize)> {
    if characters.get(index) != Some(&'r') {
        return None;
    }
    let mut cursor = index + 1;
    while characters.get(cursor) == Some(&'#') {
        cursor += 1;
    }
    (characters.get(cursor) == Some(&'"')).then(|| (cursor - index - 1, cursor - index + 1))
}

#[cfg(test)]
mod tests {
    use super::{count_lines_of_code, extract_file_metrics};
    use crate::metrics::TypeKind;

    const SOURCE: &str = r#"
use std::fmt::Debug;
extern crate core;

pub trait Port {
    fn required(&self);
    fn provided(&self) { let _ = 1; }
    fn create() -> Self where Self: Sized;
}

pub struct Service {
    repo: String,
    count: usize,
}

impl Service {
    pub fn new(repo: String) -> Self { Self { repo, count: 0 } }
    pub fn execute(&self) { let _ = &self.repo; }
    pub fn increment(&mut self) { self.count += 1; }
}

impl Port for Service {
    fn required(&self) { let _ = self.count; }
}

pub enum State { Ready { code: u8 }, Failed(String) }
pub union Bits { integer: u32, bytes: [u8; 4] }
pub fn bootstrap() {}
macro_rules! local { () => {}; }
"#;

    #[test]
    fn extracts_rust_types_impls_and_field_access_evidence() {
        let metrics = extract_file_metrics("src/lib.rs", SOURCE).expect("fixture should parse");

        assert_eq!(metrics.imports(), 2);
        assert_eq!(metrics.concrete_types(), 3);
        assert_eq!(metrics.traits(), 1);
        assert_eq!(metrics.impl_blocks(), 2);
        assert_eq!(metrics.functions(), 1);
        assert_eq!(metrics.associated_functions(), 2);
        assert_eq!(metrics.macros(), 1);
        assert!(metrics.statements() > 10);

        let service = metrics
            .types()
            .iter()
            .find(|type_info| type_info.name() == "Service")
            .expect("Service should be extracted");
        assert_eq!(service.kind(), TypeKind::Struct);
        assert_eq!(service.methods().len(), 3);
        assert_eq!(
            service
                .inherent_methods()
                .iter()
                .map(|method| method.name())
                .collect::<Vec<_>>(),
            ["execute", "increment"]
        );
        assert_eq!(service.fields().len(), 2);
        assert_eq!(metrics.impls()[0].associated_functions(), &["new"]);
        assert_eq!(service.fields()[0].accessed_by(), &["execute"]);
        assert_eq!(
            service.fields()[1].accessed_by(),
            &["increment", "required"]
        );

        let port = metrics
            .types()
            .iter()
            .find(|type_info| type_info.name() == "Port")
            .expect("Port should be extracted");
        assert_eq!(port.methods().len(), 2);
        assert!(port.inherent_methods().is_empty());
        assert!(
            !port
                .methods()
                .iter()
                .find(|method| method.name() == "required")
                .expect("required method should exist")
                .has_body()
        );
        assert!(
            port.methods()
                .iter()
                .find(|method| method.name() == "provided")
                .expect("provided method should exist")
                .has_body()
        );
        assert_eq!(port.associated_functions(), &["create"]);
    }

    #[test]
    fn counts_non_comment_physical_lines_without_being_fooled_by_literals() {
        let source = r##"
// comment
let url = "https://example.test/*path*/";
/* outer
   /* nested */
*/
let raw = r#"// still source
and /* still source */"#;

let value = 1; /* trailing comment */
"##;

        assert_eq!(count_lines_of_code(source), 4);
    }

    #[test]
    fn reports_the_source_identifier_on_parse_failure() {
        let error = extract_file_metrics("src/broken.rs", "fn {")
            .expect_err("invalid Rust should fail extraction");

        assert_eq!(error.identifier(), "src/broken.rs");
        assert!(error.to_string().contains("src/broken.rs"));
    }

    #[test]
    fn leaves_ambiguous_impl_targets_unattached_but_observable() {
        let source = r#"
mod first { pub struct Shared; }
mod second { pub struct Shared; }
impl Shared { fn ambiguous(&self) {} }
"#;
        let metrics = extract_file_metrics("src/lib.rs", source).expect("fixture should parse");

        assert_eq!(metrics.types().len(), 2);
        assert!(
            metrics
                .types()
                .iter()
                .all(|type_info| type_info.methods().is_empty())
        );
        assert_eq!(metrics.impls().len(), 1);
        assert_eq!(metrics.impls()[0].target_type(), "Shared");
        assert_eq!(metrics.impls()[0].methods()[0].name(), "ambiguous");
    }

    #[test]
    fn applies_explicit_field_semantics_to_every_rust_type_kind() {
        let source = r#"
struct Named { left: u8, right: u8 }
struct Tuple(u8, u8);
struct Unit;
enum Choice { Named { code: u8 }, Tuple(u8, u8), Unit }
union Bits { integer: u32, bytes: [u8; 4] }
trait Contract { fn apply(&self); }
"#;
        let metrics = extract_file_metrics("src/types.rs", source).expect("fixture should parse");
        let cases = [
            ("Named", TypeKind::Struct, vec!["left", "right"]),
            ("Tuple", TypeKind::Struct, vec!["0", "1"]),
            ("Unit", TypeKind::Struct, vec![]),
            (
                "Choice",
                TypeKind::Enum,
                vec!["Named.code", "Tuple.0", "Tuple.1"],
            ),
            ("Bits", TypeKind::Union, vec!["integer", "bytes"]),
            ("Contract", TypeKind::Trait, vec![]),
        ];

        for (name, kind, expected_fields) in cases {
            let type_info = metrics
                .types()
                .iter()
                .find(|type_info| type_info.name() == name)
                .expect("table type should be extracted");
            assert_eq!(type_info.kind(), kind, "unexpected kind for {name}");
            assert_eq!(
                type_info
                    .fields()
                    .iter()
                    .map(|field| field.name())
                    .collect::<Vec<_>>(),
                expected_fields,
                "unexpected fields for {name}"
            );
        }
    }
}
