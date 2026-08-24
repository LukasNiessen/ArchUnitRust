/// The slice policy that rejected a projected dependency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SliceDependencyRule {
    /// A negated rule forbids one explicit source-to-target slice dependency.
    ContainDependency,
}

impl SliceDependencyRule {
    /// Returns the stable snake-case report key.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ContainDependency => "contain_dependency",
        }
    }
}
