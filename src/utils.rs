pub fn partial_max<I, T: PartialOrd>(iter: I) -> Option<T>
where
    I: Iterator<Item = T>,
{
    iter.reduce(|acc, x| if x > acc { x } else { acc })
}
pub enum BranchIterator<A, B, C> {
    Option1(A),
    Option2(B),
    Option3(C),
}
impl<A, B, C, T> Iterator for BranchIterator<A, B, C>
where
    A: Iterator<Item = T>,
    B: Iterator<Item = T>,
    C: Iterator<Item = T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            BranchIterator::Option1(a) => a.next(),
            BranchIterator::Option2(b) => b.next(),
            BranchIterator::Option3(c) => c.next(),
        }
    }
}
