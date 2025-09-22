use std::fmt::Debug;

use super::Radix;
use crate::node::NodeId;

#[derive(Debug)]
pub struct VecRadix<Segment>(Vec<(Segment, NodeId)>);

impl<Segment> Default for VecRadix<Segment> {
    fn default() -> Self {
        VecRadix(Vec::new())
    }
}

impl<Segment: Eq + Debug> Radix<Segment> for VecRadix<Segment> {
    fn get(&self, segment: &Segment) -> Option<&NodeId> {
        self.0
            .iter()
            .find(|(seg, _)| seg == segment)
            .map(|(_, node_id)| node_id)
    }

    fn insert(&mut self, segment: Segment, node_id: NodeId) {
        match self.0.iter_mut().find(|(seg, _)| *seg == segment) {
            Some((_, id)) => {
                *id = node_id;
            }
            None => self.0.push((segment, node_id)),
        };
    }
}
