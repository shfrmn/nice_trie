use std::rc::Rc;

use crate::edge::Edge;
use crate::node::{NodeId, TrieNode};
use crate::retrieval::Retrieval;
use crate::value::TrieValue;

#[derive(Debug)]
pub struct Trie<V> {
    nodes: Vec<TrieNode<V>>,
}

impl<V: TrieValue> Trie<V> {
    pub fn new(root_value: Option<Rc<V>>) -> Self {
        let edge_value = match &root_value {
            Some(value) => value.trie_path(),
            None => Rc::from(""),
        };
        let root = TrieNode::new(Edge::new(edge_value), root_value);
        Trie { nodes: vec![root] }
    }

    fn root(&self) -> &TrieNode<V> {
        &self.nodes[0]
    }

    fn get_node(&self, node_id: &NodeId) -> &TrieNode<V> {
        &self.nodes[node_id.0]
    }

    fn insert_node(&mut self, node: TrieNode<V>) -> NodeId {
        let node_id = NodeId(self.nodes.len());
        self.nodes.push(node);
        node_id
    }

    fn update_node(&mut self, node_id: &NodeId, mut f: impl FnMut(&mut TrieNode<V>)) {
        f(&mut self.nodes[node_id.0])
    }

    fn retrieve<'a>(&'a self, path: &'a str) -> Retrieval {
        let mut current_node_id = NodeId(0);
        let mut current_node = self.root();
        let mut leaf_edge_value = path.trim_matches('/');
        while !leaf_edge_value.is_empty() && leaf_edge_value.starts_with(current_node.edge.as_ref())
        {
            if current_node.edge.len() == leaf_edge_value.len() {
                return Retrieval::Exact {
                    node_id: current_node_id,
                };
            }
            leaf_edge_value = &leaf_edge_value[current_node.edge.len()..].trim_start_matches('/');
            let desc_id = match current_node.route(leaf_edge_value) {
                Some(desc_id) => *desc_id,
                None => break, // Terminating with TrieMatch::Ancestor
            };
            let desc_node = self.get_node(&desc_id);
            if leaf_edge_value.starts_with(desc_node.edge.as_ref()) {
                current_node_id = desc_id;
                current_node = desc_node;
                continue;
            }
            // Common prefix cannot possibly be empty because it's past the routing stage
            let common_prefix_len = desc_node
                .edge
                .chars()
                .zip(leaf_edge_value.chars())
                .take_while(|(a, b)| a == b)
                .map(|(c, _)| c.len_utf8())
                .sum::<usize>();
            let desc_edge_value = desc_node.edge[common_prefix_len..].trim_start_matches('/');
            if common_prefix_len == leaf_edge_value.len() {
                return Retrieval::Split {
                    ancestor_id: current_node_id,
                    desc_id,
                    leaf_edge: Edge::new(Rc::from(leaf_edge_value)),
                    desc_edge: Edge::new(Rc::from(desc_edge_value)),
                };
            } else {
                let branch_edge_value = leaf_edge_value[..common_prefix_len].trim_end_matches('/');
                leaf_edge_value = leaf_edge_value[common_prefix_len..].trim_start_matches('/');
                return Retrieval::Diverging {
                    ancestor_id: current_node_id,
                    desc_id,
                    branch_edge: Edge::new(Rc::from(branch_edge_value)),
                    leaf_edge: Edge::new(Rc::from(leaf_edge_value)),
                    desc_edge: Edge::new(Rc::from(desc_edge_value)),
                };
            }
        }
        if leaf_edge_value.is_empty() {
            Retrieval::Exact {
                node_id: current_node_id,
            }
        } else {
            Retrieval::Ancestor {
                ancestor_id: current_node_id,
                leaf_edge: Edge::new(Rc::from(leaf_edge_value)),
            }
        }
    }

    pub fn get(&self, path: &str) -> Option<Rc<V>> {
        if let Retrieval::Exact { node_id } = self.retrieve(path) {
            self.get_node(&node_id).value.clone()
        } else {
            None
        }
    }

    pub fn insert(&mut self, value: Rc<V>) {
        match self.retrieve(&value.trie_path()) {
            Retrieval::Exact { node_id } => {
                return self.update_node(&node_id, |node| {
                    node.value = Some(Rc::clone(&value));
                });
            }
            Retrieval::Ancestor {
                ancestor_id,
                leaf_edge,
            } => {
                let leaf_byte_prefix = leaf_edge.get_byte_prefix();
                let leaf = TrieNode::new(leaf_edge, Some(value));
                let leaf_id = self.insert_node(leaf);
                self.update_node(&ancestor_id, |ancestor| {
                    ancestor.insert_edge(leaf_byte_prefix, leaf_id)
                });
            }
            Retrieval::Split {
                ancestor_id,
                desc_id,
                leaf_edge,
                desc_edge,
            } => {
                let leaf_byte_prefix = leaf_edge.get_byte_prefix();
                let desc_byte_prefix = desc_edge.get_byte_prefix();
                let mut leaf = TrieNode::new(leaf_edge, Some(value));
                leaf.insert_edge(desc_byte_prefix, desc_id);
                let leaf_id = self.insert_node(leaf);
                self.update_node(&ancestor_id, |ancestor| {
                    // Replace descendant with the leaf node
                    ancestor.insert_edge(leaf_byte_prefix, leaf_id);
                });
                self.update_node(&desc_id, |desc_node| {
                    // Update desc edge value, reflecting its new position
                    desc_node.edge = Rc::clone(&desc_edge.0);
                });
            }
            Retrieval::Diverging {
                ancestor_id,
                branch_edge,
                leaf_edge,
                desc_id,
                desc_edge,
            } => {
                let branch_byte_prefix = branch_edge.get_byte_prefix();
                let leaf_byte_prefix = leaf_edge.get_byte_prefix();
                let desc_byte_prefix = desc_edge.get_byte_prefix();
                let mut branch = TrieNode::new(branch_edge, None);
                let leaf = TrieNode::new(leaf_edge, Some(value));
                branch.insert_edge(desc_byte_prefix, desc_id);
                let leaf_id = self.insert_node(leaf);
                branch.insert_edge(leaf_byte_prefix, leaf_id);
                let branch_id = self.insert_node(branch);
                self.update_node(&ancestor_id, |ancestor| {
                    // Replace descendant with the branch node
                    ancestor.insert_edge(branch_byte_prefix, branch_id);
                });
                self.update_node(&desc_id, |desc_node| {
                    // Update desc edge value, reflecting its new position
                    desc_node.edge = Rc::clone(&desc_edge.0);
                });
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;

    #[derive(Debug, PartialEq)]
    struct Val(Rc<str>);

    impl TrieValue for Val {
        fn trie_path(&self) -> Rc<str> {
            Rc::clone(&self.0)
        }
    }

    fn create_trie() -> Trie<Val> {
        let root = Val(Rc::from(""));
        Trie::new(Some(Rc::from(root)))
    }

    #[test]
    fn basic_insert() {
        let mut t = create_trie();
        let a = Rc::from(Val(Rc::from("/a")));
        t.insert(Rc::clone(&a));
        assert_eq!(t.get("/a"), Some(a));
    }

    #[test]
    fn nested_insert() {
        let mut t = create_trie();
        let a = Rc::from(Val(Rc::from("/a")));
        let b = Rc::from(Val(Rc::from("/a/b")));
        t.insert(Rc::clone(&a));
        t.insert(Rc::clone(&b));
        println!("{:#?}", t);
        assert_eq!(t.get("/a"), Some(a));
        assert_eq!(t.get("/a/b"), Some(b));
    }

    #[test]
    fn branched_insert() {
        let mut t = create_trie();
        let a = Rc::from(Val(Rc::from("/a")));
        let b = Rc::from(Val(Rc::from("/a/b")));
        let c = Rc::from(Val(Rc::from("/a/c")));
        t.insert(Rc::clone(&a));
        t.insert(Rc::clone(&b));
        t.insert(Rc::clone(&c));
        assert_eq!(t.get("/a"), Some(a));
        assert_eq!(t.get("/a/b"), Some(b));
        assert_eq!(t.get("/a/c"), Some(c));
    }

    #[test]
    fn diverging_insert() {
        let mut t = create_trie();
        let a = Rc::from(Val(Rc::from("/a")));
        let c = Rc::from(Val(Rc::from("/a/b/c")));
        let x = Rc::from(Val(Rc::from("/a/b/x")));
        t.insert(Rc::clone(&a));
        t.insert(Rc::clone(&c));
        t.insert(Rc::clone(&x));
        assert_eq!(t.get("/a"), Some(a));
        assert_eq!(t.get("/a/b/c"), Some(c));
        assert_eq!(t.get("/a/b/x"), Some(x));
    }

    #[test]
    fn split_insert() {
        let mut t = create_trie();
        let a = Rc::from(Val(Rc::from("/a")));
        let c = Rc::from(Val(Rc::from("/a/b/c")));
        let b = Rc::from(Val(Rc::from("/a/b")));
        t.insert(Rc::clone(&a));
        t.insert(Rc::clone(&c));
        t.insert(Rc::clone(&b));
        assert_eq!(t.get("/a"), Some(a));
        assert_eq!(t.get("/a/b/c"), Some(c));
        assert_eq!(t.get("/a/b"), Some(b));
    }
}
