use std::fmt::Debug;

use crate::edge::Edge;
use crate::key::IntoEdge;
use crate::radix::Radix;

#[derive(Clone, Copy, PartialEq, Eq)]
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

    /// Matches the key slice to the next node id, removing used segments from the slice
    pub fn route(&self, key_segments: &mut &[E::Segment]) -> Option<&NodeId> {
        let result = (key_segments.len() >= self.edge.len())
            .then(|| self.radix.get(&key_segments[self.edge.len()]))
            .flatten();
        let route_len = self.edge.len() + result.is_some() as usize;
        *key_segments = &key_segments[route_len..];
        result
    }

    pub fn insert_edge(&mut self, segment: E::Segment, node_id: NodeId) {
        self.radix.insert(segment, node_id);
    }
}
