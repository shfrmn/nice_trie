pub mod char;
pub mod str;

use std::fmt::Debug;

pub trait IntoEdge<'k, K>
where
    Self: Debug,
    Self::Segment: Eq + Clone + Debug,
{
    type Segment;
    fn segment(&self, path: &'k K) -> impl Iterator<Item = Self::Segment>;
}
