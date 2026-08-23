use std::fmt;

/// The named-layer policy that rejected a dependency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum LayerDependencyRule {
    /// The target layer was absent from the source layer's allowlist.
    MayOnlyDependOnLayers,
    /// The target layer was present in the source layer's blocklist.
    MayNotDependOnLayers,
}

impl LayerDependencyRule {
    /// Returns the stable lowercase, hyphen-separated report key.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MayOnlyDependOnLayers => "may-only-depend-on-layers",
            Self::MayNotDependOnLayers => "may-not-depend-on-layers",
        }
    }
}

impl fmt::Display for LayerDependencyRule {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::LayerDependencyRule;

    #[test]
    fn exposes_stable_rule_names() {
        assert_eq!(
            LayerDependencyRule::MayOnlyDependOnLayers.as_str(),
            "may-only-depend-on-layers"
        );
        assert_eq!(
            LayerDependencyRule::MayNotDependOnLayers.to_string(),
            "may-not-depend-on-layers"
        );
    }
}
