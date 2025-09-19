pub trait TrieValue<'v, E: 'v + ?Sized> {
    fn trie_path(&'v self) -> E;
}
