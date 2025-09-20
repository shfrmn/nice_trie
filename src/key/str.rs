use std::borrow::Borrow;

use super::IntoEdge;

#[derive(Debug)]
pub struct StrSplitter<'a> {
    delimiter: &'a str,
}

impl<'a> StrSplitter<'a> {
    pub fn new(delimiter: &'a impl Borrow<str>) -> Self {
        StrSplitter {
            delimiter: delimiter.borrow(),
        }
    }
}

impl<'a> Default for StrSplitter<'a> {
    fn default() -> Self {
        StrSplitter::new(&"/")
    }
}

impl<'p, E: Borrow<str> + From<String>> IntoEdge<'p, E> for StrSplitter<'p> {
    type Segment = &'p str;

    fn segment(&self, edge: &'p E) -> impl Iterator<Item = Self::Segment> {
        edge.borrow()
            .trim_start_matches(self.delimiter)
            .trim_end_matches(self.delimiter)
            .split(self.delimiter)
    }

    // fn join(&self, segments: impl IntoIterator<Item = Self::Segment>) -> E {
    //     segments
    //         .into_iter()
    //         .collect::<Box<[&str]>>()
    //         .join(self.delimiter)
    //         .into()
    // }
}
