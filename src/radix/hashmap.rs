use std::collections::HashMap;
use std::hash::Hash;

use crate::node::NodeId;

use super::Radix;

#[derive(Debug)]
pub struct HashMapRadix<Segment>(HashMap<Segment, NodeId>);

impl<Segment> Default for HashMapRadix<Segment> {
    fn default() -> Self {
        HashMapRadix(HashMap::new())
    }
}

impl<Segment: Eq + Hash> Radix<Segment> for HashMapRadix<Segment> {
    fn get(&self, segment: &Segment) -> Option<&NodeId> {
        self.0.get(segment)
    }

    fn insert(&mut self, segment: Segment, node_id: NodeId) {
        self.0.insert(segment, node_id);
    }
}
