pub trait LendingIterator {
    type Item<'me>
    where
        Self: 'me;
    fn next(&mut self) -> Option<Self::Item<'_>>;
}
