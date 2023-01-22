#[cfg(feature = "lending_iter")]
/// `LendingIterator` is a trait that uses the new GAT feature to iterate through
/// items owned by self. It is a simulation of
/// [std::iter::Iterator](https://doc.rust-lang.org/std/iter/trait.Iterator.html)
pub trait LendingIterator {
    /// `Item` specifies a type. It is like the
    /// [std::iterator::Iterator::Item](https://doc.rust-lang.org/std/iter/trait.Iterator.html#associatedtype.Item)
    type Item<'me>
    where
        Self: 'me;
    /// `next` generates and returns the next item. It is like the
    /// [std::iterator::Iterator::next](https://doc.rust-lang.org/std/iter/trait.Iterator.html#tymethod.next)
    fn next(&mut self) -> Option<Self::Item<'_>>;
}
