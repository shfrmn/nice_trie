use std::rc::Rc;

pub trait TrieValue {
    fn trie_path(&self) -> Rc<str>;
}
