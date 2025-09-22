use std::borrow::Borrow;
use std::fmt::Debug;

pub struct Edge<Segment>(Vec<Segment>);

impl<Segment: Eq> Edge<Segment> {
    pub fn empty() -> Self {
        Edge(Vec::new())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn first(&self) -> &Segment {
        &self.0[0]
    }

    pub fn take_first(self) -> Segment {
        self.0.into_iter().next().unwrap()
    }

    pub fn common_prefix_len(&self, edge_like: &[Segment]) -> usize {
        let mut edge_iter = edge_like.into_iter();
        let mut prefix_len = 0;
        while edge_iter
            .next()
            .is_some_and(|s| prefix_len < self.len() && *s == self.0[prefix_len])
        {
            prefix_len += 1;
        }
        prefix_len
    }

    pub fn is_prefix_of(&self, edge_like: &[Segment]) -> bool {
        self.common_prefix_len(edge_like) == self.len()
    }

    /// Removes the prefix of the given length from the edge and returns it
    pub fn remove_prefix(&mut self, prefix_len: usize) -> Self {
        let mut result = self.0.split_off(prefix_len);
        std::mem::swap(&mut self.0, &mut result);
        Edge(result)
    }
}

impl<Segment> Borrow<[Segment]> for Edge<Segment> {
    fn borrow(&self) -> &[Segment] {
        self.0.borrow()
    }
}

impl<Segment> FromIterator<Segment> for Edge<Segment> {
    fn from_iter<E: IntoIterator<Item = Segment>>(edge_iter: E) -> Self {
        Edge(edge_iter.into_iter().collect())
    }
}

impl<Segment> IntoIterator for Edge<Segment> {
    type Item = Segment;

    type IntoIter = std::vec::IntoIter<Segment>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, Segment> IntoIterator for &'a Edge<Segment> {
    type Item = &'a Segment;

    type IntoIter = std::slice::Iter<'a, Segment>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<Segment: Debug> Debug for Edge<Segment> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
