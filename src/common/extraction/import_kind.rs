use std::collections::BTreeSet;
use std::fmt;
use std::iter::FromIterator;

/// The Rust syntax that produced a dependency edge.
///
/// Multiple forms can produce the same file-to-file edge. [`ImportKindSet`] retains all of them so
/// later rules and reports can distinguish, for example, a private import from a public re-export.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ImportKind {
    /// A private `use path;` declaration.
    Use,
    /// A public or restricted-visibility `pub use path;` re-export.
    PubUse,
    /// An `extern crate name;` declaration.
    ExternCrate,
    /// An outlined `mod child;` declaration.
    Mod,
    /// A qualified path used outside an import declaration.
    PathReference,
    /// A visible macro invocation, derive, or attribute-macro path.
    MacroReference,
}

impl ImportKind {
    /// Returns the stable report and serialization spelling of this kind.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Use => "use",
            Self::PubUse => "pub_use",
            Self::ExternCrate => "extern_crate",
            Self::Mod => "mod",
            Self::PathReference => "path_reference",
            Self::MacroReference => "macro_reference",
        }
    }
}

impl fmt::Display for ImportKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// A deterministically ordered set of [`ImportKind`] values.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct ImportKindSet {
    kinds: BTreeSet<ImportKind>,
}

impl ImportKindSet {
    /// Creates an empty set.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            kinds: BTreeSet::new(),
        }
    }

    /// Returns a set containing `kind` in addition to the existing values.
    #[must_use]
    pub fn with(mut self, kind: ImportKind) -> Self {
        self.kinds.insert(kind);
        self
    }

    /// Returns whether `kind` is present.
    #[must_use]
    pub fn contains(&self, kind: ImportKind) -> bool {
        self.kinds.contains(&kind)
    }

    /// Returns the number of different syntax kinds in the set.
    #[must_use]
    pub fn len(&self) -> usize {
        self.kinds.len()
    }

    /// Returns whether the set contains no syntax kinds.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.kinds.is_empty()
    }

    /// Iterates over kinds in their stable declaration order.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = ImportKind> + '_ {
        self.kinds.iter().copied()
    }
}

impl From<ImportKind> for ImportKindSet {
    fn from(kind: ImportKind) -> Self {
        Self::new().with(kind)
    }
}

impl FromIterator<ImportKind> for ImportKindSet {
    fn from_iter<T: IntoIterator<Item = ImportKind>>(kinds: T) -> Self {
        Self {
            kinds: kinds.into_iter().collect(),
        }
    }
}

impl<'a> IntoIterator for &'a ImportKindSet {
    type Item = ImportKind;
    type IntoIter = std::iter::Copied<std::collections::btree_set::Iter<'a, ImportKind>>;

    fn into_iter(self) -> Self::IntoIter {
        self.kinds.iter().copied()
    }
}

impl fmt::Display for ImportKindSet {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("[")?;
        for (index, kind) in self.iter().enumerate() {
            if index > 0 {
                formatter.write_str(", ")?;
            }
            kind.fmt(formatter)?;
        }
        formatter.write_str("]")
    }
}

#[cfg(test)]
mod tests {
    use super::{ImportKind, ImportKindSet};

    #[test]
    fn exposes_stable_kind_names() {
        let cases = [
            (ImportKind::Use, "use"),
            (ImportKind::PubUse, "pub_use"),
            (ImportKind::ExternCrate, "extern_crate"),
            (ImportKind::Mod, "mod"),
            (ImportKind::PathReference, "path_reference"),
            (ImportKind::MacroReference, "macro_reference"),
        ];

        for (kind, expected) in cases {
            assert_eq!(kind.as_str(), expected);
            assert_eq!(kind.to_string(), expected);
        }
    }

    #[test]
    fn deduplicates_and_orders_kinds() {
        let kinds: ImportKindSet = [
            ImportKind::MacroReference,
            ImportKind::Use,
            ImportKind::PubUse,
            ImportKind::Use,
        ]
        .into_iter()
        .collect();

        assert_eq!(
            kinds.iter().collect::<Vec<_>>(),
            vec![
                ImportKind::Use,
                ImportKind::PubUse,
                ImportKind::MacroReference
            ]
        );
        assert_eq!(kinds.to_string(), "[use, pub_use, macro_reference]");
    }

    #[test]
    fn supports_immutable_extension() {
        let base = ImportKindSet::from(ImportKind::Use);
        let extended = base.clone().with(ImportKind::Mod);

        assert_eq!(base.len(), 1);
        assert!(!base.contains(ImportKind::Mod));
        assert!(extended.contains(ImportKind::Use));
        assert!(extended.contains(ImportKind::Mod));
    }
}
