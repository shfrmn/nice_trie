use std::borrow::Borrow;

use crate::edge::Edge;
use crate::key::IntoEdge;
use crate::node::{NodeId, TrieNode};
use crate::radix::Radix;
use crate::retrieval::Retrieval;

#[derive(Debug)]
pub struct Trie<'k, K, V, E: IntoEdge<'k, K>, R: Radix<E::Segment>> {
    nodes: Vec<TrieNode<'k, K, V, E, R>>,
    encoder: E,
}

impl<'k, K, V, E: IntoEdge<'k, K>, R: Radix<E::Segment>> Trie<'k, K, V, E, R> {
    pub fn new(encoder: E) -> Self {
        let root = TrieNode::new(Edge::empty(), None);
        Trie {
            nodes: vec![root],
            encoder,
        }
    }

    fn root(&self) -> &TrieNode<'k, K, V, E, R> {
        &self.nodes[0]
    }

    fn get_node(&self, node_id: &NodeId) -> &TrieNode<'k, K, V, E, R> {
        &self.nodes[node_id.0]
    }

    fn insert_node(&mut self, node: TrieNode<'k, K, V, E, R>) -> NodeId {
        let node_id = NodeId(self.nodes.len());
        self.nodes.push(node);
        node_id
    }

    fn update_node(&mut self, node_id: &NodeId, f: impl FnOnce(&mut TrieNode<'k, K, V, E, R>)) {
        f(&mut self.nodes[node_id.0])
    }

    fn retrieve(&self, path_edge: &Edge<E::Segment>, exact_only: bool) -> Retrieval {
        let mut current_node_id = NodeId(0);
        let mut current_node = self.root();
        let mut leaf_segments: &[_] = path_edge.borrow();
        while !leaf_segments.is_empty() && current_node.edge.is_prefix_of(leaf_segments) {
            if current_node.edge.len() == leaf_segments.len() {
                return Retrieval::Exact {
                    node_id: current_node_id,
                };
            }
            leaf_segments = &leaf_segments[current_node.edge.len()..];
            let desc_id = match current_node.route(&leaf_segments[0]) {
                Some(desc_id) => *desc_id,
                None => break,
            };
            let desc_node = self.get_node(&desc_id);
            if desc_node.edge.is_prefix_of(leaf_segments) {
                current_node_id = desc_id;
                current_node = desc_node;
                continue;
            } else if exact_only {
                break;
            }
            let depth = path_edge.len() - leaf_segments.len();
            let common_prefix_len = desc_node.edge.common_prefix_len(leaf_segments);
            if common_prefix_len == leaf_segments.len() {
                return Retrieval::Descendant {
                    ancestor_id: current_node_id,
                    desc_id,
                    depth,
                };
            } else {
                return Retrieval::Diverging {
                    ancestor_id: current_node_id,
                    sibling_id: desc_id,
                    depth,
                    common_len: common_prefix_len,
                };
            }
        }
        if leaf_segments.is_empty() {
            Retrieval::Exact {
                node_id: current_node_id,
            }
        } else {
            Retrieval::Ancestor {
                ancestor_id: current_node_id,
                depth: path_edge.len() - leaf_segments.len(),
            }
        }
    }

    pub fn get(&self, path: &'k K) -> Option<&V> {
        let path_edge: Edge<E::Segment> = self.encoder.segment(path).collect();
        if let Retrieval::Exact { node_id } = self.retrieve(&path_edge, true) {
            self.get_node(&node_id).value.as_ref()
        } else {
            None
        }
    }

    pub fn insert(&mut self, path: &'k K, value: V) {
        let mut path_edge: Edge<E::Segment> = self.encoder.segment(path).collect();
        match self.retrieve(&path_edge, false) {
            Retrieval::Exact { node_id } => {
                return self.update_node(&node_id, |node| {
                    node.value = Some(value);
                });
            }
            Retrieval::Ancestor { ancestor_id, depth } => {
                path_edge.remove_prefix(depth);
                let route_segment = path_edge.first().clone();
                let leaf = TrieNode::new(path_edge, Some(value));
                let leaf_id = self.insert_node(leaf);
                self.update_node(&ancestor_id, |ancestor| {
                    ancestor.insert_edge(route_segment, leaf_id)
                });
            }
            Retrieval::Descendant {
                ancestor_id,
                desc_id,
                depth,
            } => {
                path_edge.remove_prefix(depth);
                self.update_node(&desc_id, |desc| {
                    desc.edge.remove_prefix(depth);
                });
                let route_segment = path_edge.first().clone();
                let mut leaf = TrieNode::new(path_edge, Some(value));
                let desc = self.get_node(&desc_id);
                leaf.insert_edge(desc.edge.first().clone(), desc_id);
                let leaf_id = self.insert_node(leaf);
                self.update_node(&ancestor_id, |ancestor| {
                    // Replace descendant with the leaf node
                    ancestor.insert_edge(route_segment, leaf_id);
                });
            }
            Retrieval::Diverging {
                ancestor_id,
                sibling_id,
                depth,
                common_len,
            } => {
                path_edge.remove_prefix(depth);
                let branch_edge = path_edge.remove_prefix(common_len);
                let branch_route_segment = branch_edge.first().clone();
                self.update_node(&sibling_id, |desc| {
                    desc.edge.remove_prefix(depth);
                });
                let mut branch = TrieNode::new(branch_edge, None);
                let sibling = self.get_node(&sibling_id);
                let sibling_route_segment = sibling.edge.first().clone();
                let leaf_route_segment = path_edge.first().clone();
                let leaf = TrieNode::new(path_edge, Some(value));
                branch.insert_edge(sibling_route_segment, sibling_id);
                let leaf_id = self.insert_node(leaf);
                branch.insert_edge(leaf_route_segment, leaf_id);
                let branch_id = self.insert_node(branch);
                self.update_node(&ancestor_id, |ancestor| {
                    // Replace descendant with the branch node
                    ancestor.insert_edge(branch_route_segment, branch_id);
                });
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use crate::key::str::StrSplitter;
    use crate::radix::vec::VecRadix;
    use crate::value::TrieValue;

    #[derive(Debug, Clone, PartialEq)]
    struct Val(Rc<str>);

    impl<'v> TrieValue<'v, &'v Rc<str>> for Val {
        fn trie_path(&'v self) -> &'v Rc<str> {
            &self.0
        }
    }

    fn create_trie<'k>() -> Trie<'k, Rc<str>, Val, StrSplitter<'k>, VecRadix<&'k str>> {
        Trie::new(StrSplitter::default())
    }

    #[test]
    fn basic_insert() {
        let mut t = create_trie();
        let a = Val(Rc::from("/a"));
        let key = a.trie_path();
        t.insert(key, a.clone());
        assert_eq!(t.get(key), Some(&a));
    }

    #[test]
    fn nested_insert() {
        let mut t = create_trie();
        let a = Val(Rc::from("/a/b"));
        let a_key = a.trie_path();
        let b = Val(Rc::from("/a/b/c"));
        let b_key = b.trie_path();
        t.insert(a_key, a.clone());
        t.insert(b_key, b.clone());
        assert_eq!(t.get(a_key), Some(&a));
        assert_eq!(t.get(b_key), Some(&b));
    }

    #[test]
    fn diverging_insert() {
        let mut t = create_trie();
        let a = Val(Rc::from("/a"));
        let c = Val(Rc::from("/a/b/c"));
        let x = Val(Rc::from("/a/b/x"));
        let a_key = a.trie_path();
        let c_key = c.trie_path();
        let x_key = x.trie_path();
        t.insert(a_key, a.clone());
        t.insert(c_key, c.clone());
        t.insert(x_key, x.clone());
        assert_eq!(t.get(a_key), Some(&a));
        assert_eq!(t.get(c_key), Some(&c));
        assert_eq!(t.get(x_key), Some(&x));
    }

    #[test]
    fn split_insert() {
        let mut t = create_trie();
        let a = Val(Rc::from("/a"));
        let c = Val(Rc::from("/a/b/c"));
        let b = Val(Rc::from("/a/b"));
        let a_key = a.trie_path();
        let c_key = c.trie_path();
        let b_key = b.trie_path();
        t.insert(a_key, a.clone());
        t.insert(c_key, c.clone());
        t.insert(b_key, b.clone());
        assert_eq!(t.get(a_key), Some(&a));
        assert_eq!(t.get(c_key), Some(&c));
        assert_eq!(t.get(b_key), Some(&b));
    }
}
