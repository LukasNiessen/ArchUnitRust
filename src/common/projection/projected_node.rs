use crate::common::Edge;

/// A graph node with its incoming and outgoing raw dependency evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectedNode {
    /// The raw graph identifier represented by this node.
    pub label: String,
    /// Non-self dependencies targeting this node.
    pub incoming: Vec<Edge>,
    /// Non-self dependencies originating from this node.
    pub outgoing: Vec<Edge>,
}

impl ProjectedNode {
    pub(crate) fn new(label: String, incoming: Vec<Edge>, outgoing: Vec<Edge>) -> Self {
        Self {
            label,
            incoming,
            outgoing,
        }
    }
}
