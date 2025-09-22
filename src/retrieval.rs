use crate::node::NodeId;

#[derive(PartialEq, Eq, Debug)]
pub enum Retrieval {
    /// Example: Searching for /a/b/c, found /a/b/c
    Exact(NodeId),

    /// Example: Searching for /a/b/c, found /a/b
    Ancestor { ancestor_id: NodeId, depth: usize },

    /// Example: Searching for /a/b, descendant node is /a/b/c
    Descendant {
        ancestor_id: NodeId,
        desc_id: NodeId,
        depth: usize,
    },

    /// Example: Searching for /a/b/x, descendant node is /a/b/c
    Diverging {
        ancestor_id: NodeId,
        sibling_id: NodeId,
        depth: usize,
        common_len: usize,
    },
}
