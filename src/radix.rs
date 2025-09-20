pub mod hashmap;
pub mod vec;

use crate::node::NodeId;

pub trait Radix<Segment: Eq>
where
    Self: Default,
{
    fn get(&self, segment: &Segment) -> Option<&NodeId>;
    fn insert(&mut self, segment: Segment, node_id: NodeId);
}
