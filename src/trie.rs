use std::borrow::Borrow;
use std::fmt::Debug;

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

impl<'k, K: Debug, V: Debug, E: IntoEdge<'k, K>, R: Radix<E::Segment>> Trie<'k, K, V, E, R> {
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

    fn get_node_mut(&mut self, node_id: &NodeId) -> &mut TrieNode<'k, K, V, E, R> {
        &mut self.nodes[node_id.0]
    }

    fn insert_node(&mut self, node: TrieNode<'k, K, V, E, R>) -> NodeId {
        let node_id = NodeId(self.nodes.len());
        self.nodes.push(node);
        node_id
    }

    /// Traverses the trie and determines the status of the key.
    ///
    /// # Arguments
    ///
    /// * `key_edge`   - A reference to pre-allocated key edge
    /// * `exact_only` - A flag that slightly improves performance by restricting return value
    ///                  to `Retrieval::Exact` or `Retrieval::Prefix` in all other cases.
    ///
    /// Important: By design, this function does not allocate any new edges,
    ///            because it is used both for reads and writes.
    fn retrieve(&self, key_edge: &Edge<E::Segment>, exact_only: bool) -> Retrieval {
        let mut current_node_id = NodeId(0);
        let mut current_node = self.root();
        let mut leaf_segments: &[_] = key_edge.borrow();
        let mut depth = current_node.edge.len();
        while !leaf_segments.is_empty() && current_node.edge.is_prefix_of(leaf_segments) {
            if current_node.edge.len() == leaf_segments.len() {
                return Retrieval::Exact(current_node_id);
            }
            let desc_id = match current_node.route(&mut leaf_segments) {
                Some(desc_id) => *desc_id,
                None => break,
            };
            let desc_node = self.get_node(&desc_id);
            if desc_node.edge.is_prefix_of(leaf_segments) {
                current_node_id = desc_id;
                current_node = desc_node;
                depth += desc_node.edge.len() + 1; // +1 for routing segment
                continue;
            } else if exact_only {
                break;
            }
            let common_prefix_len = desc_node.edge.common_prefix_len(leaf_segments);
            if common_prefix_len == leaf_segments.len() {
                return Retrieval::Split {
                    ancestor_id: current_node_id,
                    desc_id,
                    depth,
                };
            } else {
                return Retrieval::Divergence {
                    ancestor_id: current_node_id,
                    sibling_id: desc_id,
                    depth,
                    common_len: common_prefix_len + 1, // +1 for routing segment
                };
            }
        }
        if leaf_segments.is_empty() {
            Retrieval::Exact(current_node_id)
        } else {
            Retrieval::Prefix {
                ancestor_id: current_node_id,
                depth,
            }
        }
    }

    pub fn get(&self, key: &'k K) -> Option<&V> {
        let key_edge: Edge<E::Segment> = self.encoder.segment(key).collect();
        if let Retrieval::Exact(node_id) = self.retrieve(&key_edge, true) {
            self.get_node(&node_id).value.as_ref()
        } else {
            None
        }
    }

    pub fn insert(&mut self, key: &'k K, value: V) {
        let mut key_edge: Edge<E::Segment> = self.encoder.segment(key).collect();
        match self.retrieve(&key_edge, false) {
            Retrieval::Exact(node_id) => {
                let node = self.get_node_mut(&node_id);
                node.value = Some(value);
            }
            Retrieval::Prefix { ancestor_id, depth } => {
                key_edge.remove_prefix(depth);
                let route_segment = key_edge.remove_prefix(1).take_first();
                let leaf = TrieNode::new(key_edge, Some(value));
                let leaf_id = self.insert_node(leaf);
                let ancestor = self.get_node_mut(&ancestor_id);
                ancestor.insert_edge(route_segment, leaf_id)
            }
            Retrieval::Split {
                ancestor_id,
                desc_id,
                depth,
            } => {
                key_edge.remove_prefix(depth);
                let leaf_route_segment = key_edge.remove_prefix(1).take_first();
                let mut leaf = TrieNode::new(key_edge, Some(value));
                let desc = self.get_node_mut(&desc_id);
                let desc_route_segment = desc.edge.remove_prefix(1).take_first();
                leaf.insert_edge(desc_route_segment, desc_id);
                let leaf_id = self.insert_node(leaf);
                let ancestor = self.get_node_mut(&ancestor_id);
                // Replace descendant with the leaf node
                ancestor.insert_edge(leaf_route_segment, leaf_id);
            }
            Retrieval::Divergence {
                ancestor_id,
                sibling_id,
                depth,
                common_len,
            } => {
                key_edge.remove_prefix(depth);
                let mut branch_edge = key_edge.remove_prefix(common_len);
                let branch_route_segment = branch_edge.remove_prefix(1).take_first();
                let mut branch = TrieNode::new(branch_edge, None);
                let sibling = self.get_node_mut(&sibling_id);
                let sibling_route_segment = sibling.edge.remove_prefix(1).take_first();
                let leaf_route_segment = key_edge.remove_prefix(1).take_first();
                let leaf = TrieNode::new(key_edge, Some(value));
                branch.insert_edge(sibling_route_segment, sibling_id);
                let leaf_id = self.insert_node(leaf);
                branch.insert_edge(leaf_route_segment, leaf_id);
                let branch_id = self.insert_node(branch);
                let ancestor = self.get_node_mut(&ancestor_id);
                // Replace descendant with the branch node
                ancestor.insert_edge(branch_route_segment, branch_id);
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
        fn trie_key(&'v self) -> &'v Rc<str> {
            &self.0
        }
    }

    fn create_trie<'k>() -> Trie<'k, Rc<str>, Val, StrSplitter<'k>, VecRadix<&'k str>> {
        Trie::new(StrSplitter::default())
    }

    #[test]
    fn retrieve_exact() {
        let mut t = create_trie();
        let a = Val(Rc::from("/a/b"));
        let b = Val(Rc::from("/a/b/c"));
        let a_key = a.trie_key();
        let b_key = b.trie_key();
        t.insert(a_key, a.clone());
        t.insert(b_key, b.clone());
        let key_edge_a = t.encoder.segment(a_key).collect();
        assert_eq!(t.retrieve(&key_edge_a, true), Retrieval::Exact(NodeId(1)));
        assert_eq!(t.retrieve(&key_edge_a, false), Retrieval::Exact(NodeId(1)));
        let key_edge_b = t.encoder.segment(b_key).collect();
        assert_eq!(t.retrieve(&key_edge_b, true), Retrieval::Exact(NodeId(2)));
        assert_eq!(t.retrieve(&key_edge_b, false), Retrieval::Exact(NodeId(2)));
    }

    #[test]
    fn retrieve_ancestor() {
        let mut t = create_trie();
        let a = Val(Rc::from("/a/b"));
        let key = a.trie_key();
        t.insert(key, a.clone());
        let nested_key = Rc::from("/a/b/c");
        let edge = t.encoder.segment(&nested_key).collect();
        assert_eq!(
            t.retrieve(&edge, false),
            Retrieval::Prefix {
                ancestor_id: NodeId(1),
                depth: 2
            }
        );
    }

    #[test]
    fn retrieve_desc() {
        let mut t = create_trie();
        let a = Val(Rc::from("/a"));
        let c = Val(Rc::from("/a/b/c"));
        t.insert(a.trie_key(), a.clone());
        t.insert(c.trie_key(), c.clone());
        let nested_key = Rc::from("/a/b");
        let edge = t.encoder.segment(&nested_key).collect();
        assert_eq!(
            t.retrieve(&edge, false),
            Retrieval::Split {
                ancestor_id: NodeId(1),
                desc_id: NodeId(2),
                depth: 1
            }
        );
    }

    #[test]
    fn retrieve_diverging() {
        let mut t = create_trie();
        let a = Val(Rc::from("/a/b/x"));
        t.insert(a.trie_key(), a.clone());
        let nested_key = Rc::from("/a/b/y");
        let edge = t.encoder.segment(&nested_key).collect();
        assert_eq!(
            t.retrieve(&edge, false),
            Retrieval::Divergence {
                ancestor_id: NodeId(0),
                sibling_id: NodeId(1),
                depth: 0,
                common_len: 2
            }
        );
    }

    #[test]
    fn basic_insert() {
        let mut t = create_trie();
        let a = Val(Rc::from("/a"));
        let key = a.trie_key();
        t.insert(key, a.clone());
        assert_eq!(t.get(key), Some(&a));
    }

    #[test]
    fn nested_insert() {
        let mut t = create_trie();
        let a = Val(Rc::from("/a/b"));
        let a_key = a.trie_key();
        let b = Val(Rc::from("/a/b/c"));
        let b_key = b.trie_key();
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
        let a_key = a.trie_key();
        let c_key = c.trie_key();
        let x_key = x.trie_key();
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
        let a_key = a.trie_key();
        let c_key = c.trie_key();
        let b_key = b.trie_key();
        t.insert(a_key, a.clone());
        t.insert(c_key, c.clone());
        t.insert(b_key, b.clone());
        assert_eq!(t.get(a_key), Some(&a));
        assert_eq!(t.get(c_key), Some(&c));
        assert_eq!(t.get(b_key), Some(&b));
    }
}
