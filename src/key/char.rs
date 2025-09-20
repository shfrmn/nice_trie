use std::borrow::Borrow;

use super::IntoEdge;

#[derive(Debug)]
pub struct CharSplitter {}

impl<'k, E: Borrow<str> + From<String>> IntoEdge<'k, E> for CharSplitter {
    type Segment = char;

    fn segment(&self, edge: &'k E) -> impl Iterator<Item = Self::Segment> {
        edge.borrow().chars()
    }

    // fn join(&self, segments: impl IntoIterator<Item = Self::Segment>) -> E {
    //     segments.into_iter().collect::<String>().into()
    // }
}
