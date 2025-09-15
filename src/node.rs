use std::fmt::Debug;
use std::{collections::HashMap, rc::Rc};

use crate::edge::Edge;
use crate::value::TrieValue;

#[derive(Clone, Copy, Debug)]
pub struct NodeId(pub usize);

pub struct TrieNode<V> {
    pub edge: Rc<str>,
    pub children: HashMap<u8, NodeId>,
    pub value: Option<Rc<V>>,
}

impl<V: Debug> Debug for TrieNode<V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut children = HashMap::new();
        for (first_byte, value) in &self.children {
            children.insert(*first_byte as char, value);
        }
        f.debug_struct("TrieNode")
            .field("edge", &self.edge)
            .field("children", &children)
            .field("value", &self.value)
            .finish()
    }
}

impl<V: TrieValue> TrieNode<V> {
    pub fn new(edge: Edge, value: Option<Rc<V>>) -> Self {
        TrieNode {
            edge: edge.0,
            children: HashMap::new(),
            value,
        }
    }

    pub fn route(&self, remainder: &str) -> Option<&NodeId> {
        self.children.get(&remainder.as_bytes()[0])
    }

    pub fn insert_edge(&mut self, byte_prefix: u8, node_id: NodeId) {
        self.children.insert(byte_prefix, node_id);
    }
}
