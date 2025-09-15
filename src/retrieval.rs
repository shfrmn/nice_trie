use crate::edge::Edge;
use crate::node::NodeId;

#[derive(Debug)]
pub enum Retrieval {
    /// Example: Searching for /a/b/c, found /a/b/c
    Exact { node_id: NodeId },

    /// Example: Searching for /a/b/c, found /a/b
    Ancestor {
        ancestor_id: NodeId,
        leaf_edge: Edge,
    },

    /// Example: Searching for /a/b, descendant node is /a/b/c
    Split {
        ancestor_id: NodeId,
        desc_id: NodeId,
        leaf_edge: Edge,
        desc_edge: Edge,
    },

    /// Example: Searching for /a/b/x, descendant node is /a/b/c
    Diverging {
        ancestor_id: NodeId,
        desc_id: NodeId,
        branch_edge: Edge,
        leaf_edge: Edge,
        desc_edge: Edge,
    },
}
