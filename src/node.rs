use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

use crate::edge::Edge;

#[derive(Clone, Copy)]
pub struct NodeId(pub usize);

impl Debug for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub struct TrieNode<Value, Segment: Eq + Hash> {
    pub edge: Edge<Segment>,
    pub children: HashMap<Segment, NodeId>,
    pub value: Option<Value>,
}

impl<'s, Value, Segment: Eq + Hash> TrieNode<Value, Segment> {
    pub fn new(edge: Edge<Segment>, value: Option<Value>) -> Self {
        TrieNode {
            edge,
            children: HashMap::new(),
            value,
        }
    }

    pub fn route(&self, segment: &Segment) -> Option<&NodeId> {
        self.children.get(segment)
    }

    pub fn insert_edge(&mut self, segment: Segment, node_id: NodeId) {
        self.children.insert(segment, node_id);
    }
}
