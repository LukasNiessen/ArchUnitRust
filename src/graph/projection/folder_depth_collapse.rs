use super::GraphQueryError;

/// Collapses file nodes to their containing folder at a fixed path depth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FolderDepthCollapse {
    depth: usize,
}

impl FolderDepthCollapse {
    /// Creates a positive folder-depth strategy.
    pub fn new(depth: usize) -> Result<Self, GraphQueryError> {
        if depth == 0 {
            return Err(GraphQueryError::ZeroFolderDepth);
        }
        Ok(Self { depth })
    }

    /// Returns the number of leading folder components retained.
    #[must_use]
    pub const fn depth(self) -> usize {
        self.depth
    }
}

#[cfg(test)]
mod tests {
    use super::FolderDepthCollapse;
    use crate::GraphQueryError;

    #[test]
    fn requires_a_positive_depth() {
        assert_eq!(
            FolderDepthCollapse::new(2)
                .expect("positive depth should be valid")
                .depth(),
            2
        );
        assert!(matches!(
            FolderDepthCollapse::new(0),
            Err(GraphQueryError::ZeroFolderDepth)
        ));
    }
}
