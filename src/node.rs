use std::fmt::Debug;

use crate::edge::Edge;
use crate::key::IntoEdge;
use crate::radix::Radix;

#[derive(Clone, Copy)]
pub struct NodeId(pub usize);

impl Debug for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub struct TrieNode<'k, K, V, E: IntoEdge<'k, K>, R: Radix<E::Segment>> {
    pub edge: Edge<E::Segment>,
    pub radix: R,
    pub value: Option<V>,
}

impl<'k, K, V, E: IntoEdge<'k, K>, R: Radix<E::Segment>> TrieNode<'k, K, V, E, R> {
    pub fn new(edge: Edge<E::Segment>, value: Option<V>) -> Self {
        TrieNode {
            edge,
            radix: R::default(),
            value,
        }
    }

    pub fn route(&self, segment: &E::Segment) -> Option<&NodeId> {
        self.radix.get(segment)
    }

    pub fn insert_edge(&mut self, segment: E::Segment, node_id: NodeId) {
        self.radix.insert(segment, node_id);
    }
}
