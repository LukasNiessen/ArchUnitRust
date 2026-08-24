use std::path::{Path, PathBuf};

/// A Rust type declaration category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum TypeKind {
    /// A `struct` declaration.
    Struct,
    /// An `enum` declaration.
    Enum,
    /// A `union` declaration.
    Union,
    /// A `trait` declaration.
    Trait,
}

/// One associated function whose signature has a `self` receiver.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct MethodInfo {
    pub(crate) name: String,
    pub(crate) accessed_fields: Vec<String>,
    pub(crate) has_body: bool,
}

impl MethodInfo {
    /// Returns the declared method name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the sorted field names referenced as `self.field` in the method body.
    #[must_use]
    pub fn accessed_fields(&self) -> &[String] {
        &self.accessed_fields
    }

    /// Returns whether the declaration has a body.
    #[must_use]
    pub const fn has_body(&self) -> bool {
        self.has_body
    }
}

/// One declared data field and the methods that syntactically access it.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct FieldInfo {
    pub(crate) name: String,
    pub(crate) accessed_by: Vec<String>,
}

impl FieldInfo {
    /// Returns the field name. Tuple fields use their zero-based index.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns sorted method names containing a `self.<field>` access.
    #[must_use]
    pub fn accessed_by(&self) -> &[String] {
        &self.accessed_by
    }
}

/// One inherent or trait implementation block.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ImplInfo {
    pub(crate) target_type: String,
    pub(crate) trait_name: Option<String>,
    pub(crate) file_path: String,
    pub(crate) methods: Vec<MethodInfo>,
    pub(crate) associated_functions: Vec<String>,
}

impl ImplInfo {
    /// Returns the syntactic implementation target path.
    #[must_use]
    pub fn target_type(&self) -> &str {
        &self.target_type
    }

    /// Returns the implemented trait path, or `None` for an inherent impl.
    #[must_use]
    pub fn trait_name(&self) -> Option<&str> {
        self.trait_name.as_deref()
    }

    /// Returns the normalized source-file identifier.
    #[must_use]
    pub fn file_path(&self) -> &str {
        &self.file_path
    }

    /// Returns functions in this impl that have a `self` receiver.
    #[must_use]
    pub fn methods(&self) -> &[MethodInfo] {
        &self.methods
    }

    /// Returns receiver-free associated function names.
    #[must_use]
    pub fn associated_functions(&self) -> &[String] {
        &self.associated_functions
    }

    /// Returns whether this is a trait implementation.
    #[must_use]
    pub const fn is_trait_impl(&self) -> bool {
        self.trait_name.is_some()
    }
}

/// Metrics information for one Rust type declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct TypeInfo {
    pub(crate) name: String,
    pub(crate) file_path: String,
    pub(crate) kind: TypeKind,
    pub(crate) methods: Vec<MethodInfo>,
    pub(crate) fields: Vec<FieldInfo>,
    pub(crate) associated_functions: Vec<String>,
}

impl TypeInfo {
    /// Returns the module-qualified name available from source syntax.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the unqualified declared name.
    #[must_use]
    pub fn simple_name(&self) -> &str {
        self.name.rsplit("::").next().unwrap_or(self.name.as_str())
    }

    /// Returns the normalized source-file identifier.
    #[must_use]
    pub fn file_path(&self) -> &str {
        &self.file_path
    }

    /// Returns the declaration category.
    #[must_use]
    pub const fn kind(&self) -> TypeKind {
        self.kind
    }

    /// Returns functions with a `self` receiver declared by or associated with this type.
    #[must_use]
    pub fn methods(&self) -> &[MethodInfo] {
        &self.methods
    }

    /// Returns declared data fields.
    #[must_use]
    pub fn fields(&self) -> &[FieldInfo] {
        &self.fields
    }

    /// Returns receiver-free associated function names.
    #[must_use]
    pub fn associated_functions(&self) -> &[String] {
        &self.associated_functions
    }
}

/// Extracted counts and type information for one Rust source file.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct FileMetricsInfo {
    pub(crate) path: String,
    pub(crate) lines_of_code: usize,
    pub(crate) statements: usize,
    pub(crate) imports: usize,
    pub(crate) concrete_types: usize,
    pub(crate) functions: usize,
    pub(crate) traits: usize,
    pub(crate) impl_blocks: usize,
    pub(crate) macros: usize,
    pub(crate) associated_functions: usize,
    pub(crate) types: Vec<TypeInfo>,
    pub(crate) impls: Vec<ImplInfo>,
}

impl FileMetricsInfo {
    /// Returns the normalized source-file identifier.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Returns physical lines containing non-comment source text.
    #[must_use]
    pub const fn lines_of_code(&self) -> usize {
        self.lines_of_code
    }

    /// Returns syntax-tree items plus executable block statements.
    #[must_use]
    pub const fn statements(&self) -> usize {
        self.statements
    }

    /// Returns `use` and `extern crate` item count.
    #[must_use]
    pub const fn imports(&self) -> usize {
        self.imports
    }

    /// Returns struct, enum, and union declaration count.
    #[must_use]
    pub const fn concrete_types(&self) -> usize {
        self.concrete_types
    }

    /// Returns free-function count.
    #[must_use]
    pub const fn functions(&self) -> usize {
        self.functions
    }

    /// Returns trait declaration count.
    #[must_use]
    pub const fn traits(&self) -> usize {
        self.traits
    }

    /// Returns inherent and trait impl-block count.
    #[must_use]
    pub const fn impl_blocks(&self) -> usize {
        self.impl_blocks
    }

    /// Returns macro invocation count, including macro definitions represented by `syn`.
    #[must_use]
    pub const fn macros(&self) -> usize {
        self.macros
    }

    /// Returns receiver-free functions declared in traits and impl blocks.
    #[must_use]
    pub const fn associated_functions(&self) -> usize {
        self.associated_functions
    }

    /// Returns type declarations in the file.
    #[must_use]
    pub fn types(&self) -> &[TypeInfo] {
        &self.types
    }

    /// Returns impl blocks in the file.
    #[must_use]
    pub fn impls(&self) -> &[ImplInfo] {
        &self.impls
    }

    pub(crate) fn retain_types(&mut self, predicate: impl Fn(&TypeInfo) -> bool) {
        self.types.retain(predicate);
        self.concrete_types = self
            .types
            .iter()
            .filter(|type_info| type_info.kind != TypeKind::Trait)
            .count();
        self.traits = self
            .types
            .iter()
            .filter(|type_info| type_info.kind == TypeKind::Trait)
            .count();
        let names = self
            .types
            .iter()
            .map(|type_info| type_info.name.as_str())
            .collect::<Vec<_>>();
        self.impls.retain(|impl_info| {
            names.iter().any(|name| {
                **name == impl_info.target_type
                    || name.rsplit("::").next() == impl_info.target_type.rsplit("::").next()
            })
        });
    }
}

/// A deterministic metrics snapshot for one Cargo project selection.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ProjectMetricsInfo {
    pub(crate) root: PathBuf,
    pub(crate) files: Vec<FileMetricsInfo>,
    pub(crate) types: Vec<TypeInfo>,
}

impl ProjectMetricsInfo {
    /// Returns the Cargo workspace root.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Returns selected source files sorted by identifier.
    #[must_use]
    pub fn files(&self) -> &[FileMetricsInfo] {
        &self.files
    }

    /// Returns selected types sorted by file and qualified name.
    #[must_use]
    pub fn types(&self) -> &[TypeInfo] {
        &self.types
    }

    pub(crate) fn from_files(root: PathBuf, mut files: Vec<FileMetricsInfo>) -> Self {
        files.sort_by(|left, right| left.path.cmp(&right.path));
        let mut types = files
            .iter()
            .flat_map(|file| file.types.iter().cloned())
            .collect::<Vec<_>>();
        types.sort_by(|left, right| {
            left.file_path
                .cmp(&right.file_path)
                .then_with(|| left.name.cmp(&right.name))
        });
        Self { root, files, types }
    }
}
