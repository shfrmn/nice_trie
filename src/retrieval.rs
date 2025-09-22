use crate::node::NodeId;

#[derive(PartialEq, Eq, Debug)]
pub enum Retrieval {
    /// Example: Searching for /a/b/c: found /a/b/c
    Exact(NodeId),

    /// Example: Searching for /a/b/c: found /a/b
    Prefix { ancestor_id: NodeId, depth: usize },

    /// Example: Searching for /a/b: ancestor is /a, descendant is /a/b/c
    Split {
        ancestor_id: NodeId,
        desc_id: NodeId,
        depth: usize,
    },

    /// Example: Searching for /a/b/x: ancestor is /a, sibling is /a/b/c
    Divergence {
        ancestor_id: NodeId,
        sibling_id: NodeId,
        depth: usize,
        common_len: usize,
    },
}
