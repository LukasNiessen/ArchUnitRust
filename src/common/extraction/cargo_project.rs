use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use cargo_metadata::{DependencyKind, Metadata, TargetKind};

use super::{SourceFile, SourceOptions};
use crate::TechnicalError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum CargoDependencyScope {
    Normal,
    Development,
    Build,
}

impl CargoDependencyScope {
    const fn accepts(self, kind: CargoDependencyKind) -> bool {
        match self {
            Self::Normal => matches!(
                kind,
                CargoDependencyKind::Normal | CargoDependencyKind::Unknown
            ),
            Self::Development => !matches!(kind, CargoDependencyKind::Build),
            Self::Build => matches!(
                kind,
                CargoDependencyKind::Build | CargoDependencyKind::Unknown
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum CargoDependencyKind {
    Normal,
    Development,
    Build,
    Unknown,
}

impl From<DependencyKind> for CargoDependencyKind {
    fn from(kind: DependencyKind) -> Self {
        match kind {
            DependencyKind::Normal => Self::Normal,
            DependencyKind::Development => Self::Development,
            DependencyKind::Build => Self::Build,
            DependencyKind::Unknown => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum CargoDependencyTarget {
    Internal(String),
    External(String),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct CargoDependency {
    visible_name: String,
    kind: CargoDependencyKind,
    target: CargoDependencyTarget,
}

/// A Cargo target category relevant to source analysis.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum CargoTargetKind {
    /// A benchmark target.
    Bench,
    /// An executable target.
    Bin,
    /// A package build script.
    CustomBuild,
    /// A C-compatible dynamic library target.
    CDyLib,
    /// A Rust dynamic library target.
    DyLib,
    /// An example target.
    Example,
    /// A Rust library target.
    Lib,
    /// A procedural macro target.
    ProcMacro,
    /// A Rust intermediate library target.
    RLib,
    /// A static system library target.
    StaticLib,
    /// An integration test target.
    Test,
    /// A target category introduced by a newer Cargo version.
    Unknown(String),
}

impl CargoTargetKind {
    /// Returns Cargo's stable spelling for this target category.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Bench => "bench",
            Self::Bin => "bin",
            Self::CustomBuild => "custom-build",
            Self::CDyLib => "cdylib",
            Self::DyLib => "dylib",
            Self::Example => "example",
            Self::Lib => "lib",
            Self::ProcMacro => "proc-macro",
            Self::RLib => "rlib",
            Self::StaticLib => "staticlib",
            Self::Test => "test",
            Self::Unknown(kind) => kind,
        }
    }

    fn from_metadata(kind: &TargetKind) -> Self {
        match kind {
            TargetKind::Bench => Self::Bench,
            TargetKind::Bin => Self::Bin,
            TargetKind::CustomBuild => Self::CustomBuild,
            TargetKind::CDyLib => Self::CDyLib,
            TargetKind::DyLib => Self::DyLib,
            TargetKind::Example => Self::Example,
            TargetKind::Lib => Self::Lib,
            TargetKind::ProcMacro => Self::ProcMacro,
            TargetKind::RLib => Self::RLib,
            TargetKind::StaticLib => Self::StaticLib,
            TargetKind::Test => Self::Test,
            TargetKind::Unknown(kind) => Self::Unknown(kind.clone()),
            kind => Self::Unknown(kind.to_string()),
        }
    }

    const fn is_dev_only(&self) -> bool {
        matches!(self, Self::Bench | Self::Example | Self::Test)
    }
}

/// One crate target reported by Cargo metadata.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub struct CargoTarget {
    package: String,
    name: String,
    kinds: Vec<CargoTargetKind>,
    source: SourceFile,
}

impl CargoTarget {
    fn new(
        package: String,
        name: String,
        mut kinds: Vec<CargoTargetKind>,
        source: SourceFile,
    ) -> Self {
        kinds.sort_unstable();
        kinds.dedup();
        Self {
            package,
            name,
            kinds,
            source,
        }
    }

    /// Returns the workspace package that declares this target.
    #[must_use]
    pub fn package(&self) -> &str {
        &self.package
    }

    /// Returns the target name from Cargo metadata.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns every Cargo category associated with this target.
    #[must_use]
    pub fn kinds(&self) -> &[CargoTargetKind] {
        &self.kinds
    }

    /// Returns the crate-root source file for this target.
    #[must_use]
    pub const fn source(&self) -> &SourceFile {
        &self.source
    }

    /// Returns whether this target is a test, example, or benchmark.
    #[must_use]
    pub fn is_dev_only(&self) -> bool {
        self.kinds.iter().any(CargoTargetKind::is_dev_only)
    }

    pub(crate) fn dependency_scope(&self) -> CargoDependencyScope {
        if self
            .kinds
            .iter()
            .any(|kind| matches!(kind, CargoTargetKind::CustomBuild))
        {
            CargoDependencyScope::Build
        } else if self.is_dev_only() {
            CargoDependencyScope::Development
        } else {
            CargoDependencyScope::Normal
        }
    }

    fn is_dependency_target(&self) -> bool {
        self.kinds.iter().any(|kind| {
            matches!(
                kind,
                CargoTargetKind::CDyLib
                    | CargoTargetKind::DyLib
                    | CargoTargetKind::Lib
                    | CargoTargetKind::ProcMacro
                    | CargoTargetKind::RLib
                    | CargoTargetKind::StaticLib
            )
        })
    }
}

/// Cargo's authoritative description of the selected workspace.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub struct CargoProject {
    root: PathBuf,
    manifest_path: PathBuf,
    target_directory: PathBuf,
    member_roots: Vec<PathBuf>,
    targets: Vec<CargoTarget>,
    dependencies: BTreeMap<String, Vec<CargoDependency>>,
}

impl CargoProject {
    pub(crate) fn from_metadata(metadata: Metadata) -> Result<Self, TechnicalError> {
        let root = PathBuf::from(metadata.workspace_root.as_std_path());
        let manifest_path = root.join("Cargo.toml");
        let target_directory = PathBuf::from(metadata.target_directory.as_std_path());
        let mut member_roots = BTreeSet::new();
        let mut package_roots = BTreeMap::new();
        let mut targets = Vec::new();
        let workspace_packages = metadata.workspace_packages();

        for package in &workspace_packages {
            let package_manifest = PathBuf::from(package.manifest_path.as_std_path());
            let Some(package_root) = package_manifest.parent() else {
                return Err(TechnicalError::new(format!(
                    "Cargo returned a manifest without a parent: {}",
                    package_manifest.display()
                )));
            };
            let package_root = package_root.to_path_buf();
            member_roots.insert(package_root.clone());
            package_roots.insert(comparable_path(&package_root), package.name.to_string());

            for target in &package.targets {
                let source_path = PathBuf::from(target.src_path.as_std_path());
                if !source_path.starts_with(&root) || !source_path.starts_with(&package_root) {
                    continue;
                }
                let Some(identifier) = workspace_identifier(&root, &source_path) else {
                    continue;
                };
                let kinds = target
                    .kind
                    .iter()
                    .map(CargoTargetKind::from_metadata)
                    .collect();
                targets.push(CargoTarget::new(
                    package.name.to_string(),
                    target.name.clone(),
                    kinds,
                    SourceFile::new(source_path, identifier),
                ));
            }
        }

        targets.sort_by(|left, right| {
            left.source
                .identifier()
                .cmp(right.source.identifier())
                .then_with(|| left.package.cmp(&right.package))
                .then_with(|| left.name.cmp(&right.name))
        });
        targets.dedup_by(|left, right| {
            left.package == right.package
                && left.name == right.name
                && left.source == right.source
                && left.kinds == right.kinds
        });

        let mut dependency_targets = BTreeMap::<String, BTreeSet<String>>::new();
        for target in &targets {
            if target.is_dependency_target() {
                dependency_targets
                    .entry(target.package.clone())
                    .or_default()
                    .insert(target.source.identifier().to_owned());
            }
        }

        let mut dependencies = BTreeMap::<String, Vec<CargoDependency>>::new();
        for package in workspace_packages {
            let package_name = package.name.to_string();
            let package_dependencies = dependencies.entry(package_name).or_default();
            for dependency in &package.dependencies {
                let visible_name = normalize_crate_name(
                    dependency
                        .rename
                        .as_deref()
                        .unwrap_or(dependency.name.as_str()),
                );
                let kind = CargoDependencyKind::from(dependency.kind);
                let workspace_package = dependency
                    .path
                    .as_ref()
                    .and_then(|path| package_roots.get(&comparable_path(path.as_std_path())));

                if let Some(workspace_package) = workspace_package {
                    if let Some(targets) = dependency_targets.get(workspace_package) {
                        for target in targets {
                            package_dependencies.push(CargoDependency {
                                visible_name: visible_name.clone(),
                                kind,
                                target: CargoDependencyTarget::Internal(target.clone()),
                            });
                        }
                    }
                } else {
                    package_dependencies.push(CargoDependency {
                        visible_name: visible_name.clone(),
                        kind,
                        target: CargoDependencyTarget::External(visible_name),
                    });
                }
            }
            package_dependencies.sort();
            package_dependencies.dedup();
        }

        Ok(Self {
            root,
            manifest_path,
            target_directory,
            member_roots: member_roots.into_iter().collect(),
            targets,
            dependencies,
        })
    }

    /// Returns Cargo's absolute workspace root.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Returns the workspace manifest, including for a virtual workspace.
    #[must_use]
    pub fn manifest_path(&self) -> &Path {
        &self.manifest_path
    }

    /// Returns Cargo's configured build-output directory.
    #[must_use]
    pub fn target_directory(&self) -> &Path {
        &self.target_directory
    }

    /// Returns the absolute roots of workspace member packages in deterministic order.
    pub fn member_roots(&self) -> impl ExactSizeIterator<Item = &Path> {
        self.member_roots.iter().map(PathBuf::as_path)
    }

    /// Returns all target roots Cargo reported, including development-only targets.
    #[must_use]
    pub fn targets(&self) -> &[CargoTarget] {
        &self.targets
    }

    /// Returns target roots selected by the source analysis options.
    pub fn source_targets(&self, options: SourceOptions) -> impl Iterator<Item = &CargoTarget> {
        self.targets
            .iter()
            .filter(move |target| options.includes_dev_targets() || !target.is_dev_only())
    }

    pub(crate) fn dependency_targets(
        &self,
        package: &str,
        scope: CargoDependencyScope,
        visible_name: &str,
    ) -> BTreeSet<CargoDependencyTarget> {
        if matches!(visible_name, "std" | "core" | "alloc" | "proc_macro") {
            return [CargoDependencyTarget::External(visible_name.to_owned())]
                .into_iter()
                .collect();
        }

        self.dependencies
            .get(package)
            .into_iter()
            .flatten()
            .filter(|dependency| {
                dependency.visible_name == visible_name && scope.accepts(dependency.kind)
            })
            .map(|dependency| dependency.target.clone())
            .collect()
    }
}

fn comparable_path(path: &Path) -> PathBuf {
    match fs::canonicalize(path) {
        Ok(path) => path,
        Err(_) => path.to_path_buf(),
    }
}

fn normalize_crate_name(name: &str) -> String {
    name.replace('-', "_")
}

pub(crate) fn workspace_identifier(root: &Path, path: &Path) -> Option<String> {
    let relative = path.strip_prefix(root).ok()?;
    let identifier = relative
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/");
    (!identifier.is_empty()).then_some(identifier)
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, path::PathBuf};

    use super::{
        CargoDependencyKind, CargoDependencyScope, CargoProject, CargoTarget, CargoTargetKind,
        normalize_crate_name, workspace_identifier,
    };
    use crate::{SourceFile, SourceOptions};

    fn target(kind: CargoTargetKind, identifier: &str) -> CargoTarget {
        CargoTarget::new(
            "fixture".to_owned(),
            "fixture".to_owned(),
            vec![kind],
            SourceFile::new(PathBuf::from(identifier), identifier.to_owned()),
        )
    }

    #[test]
    fn target_kinds_have_stable_cargo_spellings() {
        assert_eq!(CargoTargetKind::CustomBuild.as_str(), "custom-build");
        assert_eq!(CargoTargetKind::ProcMacro.as_str(), "proc-macro");
        assert_eq!(
            CargoTargetKind::Unknown("future-kind".to_owned()).as_str(),
            "future-kind"
        );
    }

    #[test]
    fn source_target_selection_keeps_dev_targets_opt_in() {
        let project = CargoProject {
            root: PathBuf::from("workspace"),
            manifest_path: PathBuf::from("workspace/Cargo.toml"),
            target_directory: PathBuf::from("workspace/target"),
            member_roots: vec![PathBuf::from("workspace/crate")],
            targets: vec![
                target(CargoTargetKind::Lib, "src/lib.rs"),
                target(CargoTargetKind::Test, "tests/architecture.rs"),
            ],
            dependencies: BTreeMap::new(),
        };

        assert_eq!(project.source_targets(SourceOptions::new()).count(), 1);
        assert_eq!(
            project
                .source_targets(SourceOptions::new().with_dev_targets(true))
                .count(),
            2
        );
    }

    #[test]
    fn workspace_identifiers_are_separator_normalized() {
        let root = PathBuf::from("workspace");
        let path = root.join("crates").join("app").join("src").join("lib.rs");

        assert_eq!(
            workspace_identifier(&root, &path).as_deref(),
            Some("crates/app/src/lib.rs")
        );
    }

    #[test]
    fn cargo_names_and_dependency_scopes_match_rust_visibility() {
        assert_eq!(normalize_crate_name("wire-format"), "wire_format");
        assert!(CargoDependencyScope::Normal.accepts(CargoDependencyKind::Normal));
        assert!(!CargoDependencyScope::Normal.accepts(CargoDependencyKind::Development));
        assert!(CargoDependencyScope::Development.accepts(CargoDependencyKind::Normal));
        assert!(CargoDependencyScope::Development.accepts(CargoDependencyKind::Development));
        assert!(!CargoDependencyScope::Development.accepts(CargoDependencyKind::Build));
        assert!(CargoDependencyScope::Build.accepts(CargoDependencyKind::Build));
        assert!(!CargoDependencyScope::Build.accepts(CargoDependencyKind::Normal));
    }
}
