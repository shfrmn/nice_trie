use std::rc::Rc;

#[derive(Debug)]
pub struct Edge(pub Rc<str>);

impl Edge {
    pub fn new(edge_value: Rc<str>) -> Self {
        Edge(edge_value)
    }

    pub fn get_byte_prefix(&self) -> u8 {
        self.0.as_bytes()[0]
    }
}
