pub trait TrieValue<'v, E: 'v + ?Sized> {
    fn trie_key(&'v self) -> E;
}
