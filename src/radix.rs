use std::fmt::Debug;
use std::hash::Hash;

pub trait Radix<'p, Path>
where
    Self::Segment: Debug + Eq + Hash + Clone,
{
    type Segment;

    fn segment(&self, path: &'p Path) -> impl Iterator<Item = Self::Segment>;
    fn join(&self, segments: impl IntoIterator<Item = Self::Segment>) -> Path;
}

//
//
//

#[derive(Debug)]
pub struct StrRadix<'a> {
    delimiter: &'a str,
}

impl<'a> StrRadix<'a> {
    pub fn new(delimiter: &'a impl AsRef<str>) -> Self {
        StrRadix {
            delimiter: delimiter.as_ref(),
        }
    }
}

impl<'a> Default for StrRadix<'a> {
    fn default() -> Self {
        StrRadix::new(&"/")
    }
}

impl<'a, E: AsRef<str> + From<String>> Radix<'a, E> for StrRadix<'a> {
    type Segment = &'a str;

    fn segment(&self, edge: &'a E) -> impl Iterator<Item = Self::Segment> {
        edge.as_ref()
            .trim_start_matches(self.delimiter)
            .trim_end_matches(self.delimiter)
            .split(self.delimiter)
    }

    fn join(&self, segments: impl IntoIterator<Item = Self::Segment>) -> E {
        segments
            .into_iter()
            .collect::<Box<[&str]>>()
            .join(self.delimiter)
            .into()
    }
}

//
//
//
#[derive(Debug)]
pub struct CharRadix {}

impl<'a, E: AsRef<str> + From<String>> Radix<'a, E> for CharRadix {
    type Segment = char;

    fn segment(&self, edge: &'a E) -> impl Iterator<Item = Self::Segment> {
        edge.as_ref().chars()
    }

    fn join(&self, segments: impl IntoIterator<Item = Self::Segment>) -> E {
        segments.into_iter().collect::<String>().into()
    }
}
