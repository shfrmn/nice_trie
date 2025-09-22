use std::fmt::Debug;

use super::Radix;
use crate::node::NodeId;

pub struct VecRadix<Segment>(Vec<(Segment, NodeId)>);

impl<Segment: Debug> Debug for VecRadix<Segment> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "VecRadix ")?;
        let mut m = f.debug_map();
        for (segment, node_id) in self.0.iter() {
            m.entry(segment, node_id);
        }
        m.finish()
    }
}

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
