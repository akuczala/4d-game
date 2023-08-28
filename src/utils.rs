use crate::vector::VecIndex;

pub fn partial_max<I, T: PartialOrd>(iter: I) -> Option<T>
where
    I: Iterator<Item = T>,
{
    iter.reduce(|acc, x| if x > acc { x } else { acc })
}

pub fn partial_fmax<I, F, S: Copy, T: PartialOrd>(iter: I, f: F) -> Option<(S, T)>
where
    I: Iterator<Item = S>,
    F: Fn(S) -> T,
{
    iter.map(|s| (s, f(s)))
        .reduce(|(s1, t1), (s2, t2)| match t1 > t2 {
            true => (s1, t1),
            false => (s2, t2),
        })
}

pub fn partial_argmax<I, T: PartialOrd + Copy>(iter: I) -> Option<usize>
where
    I: Iterator<Item = T>,
{
    partial_fmax(iter.enumerate(), |(_i, t)| t).map(|((imax, _), _)| imax)
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
            Self::Option1(a) => a.next(),
            Self::Option2(b) => b.next(),
            Self::Option3(c) => c.next(),
        }
    }
}

pub enum BranchIterator2<A, B> {
    Option1(A),
    Option2(B),
}
impl<A, B, T> Iterator for BranchIterator2<A, B>
where
    A: Iterator<Item = T>,
    B: Iterator<Item = T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Option1(a) => a.next(),
            Self::Option2(b) => b.next(),
        }
    }
}

#[derive(Copy, Clone)]
pub enum ValidDimension {
    Three,
    Four,
}

impl From<VecIndex> for ValidDimension {
    fn from(value: VecIndex) -> Self {
        match value {
            3 => Self::Three,
            4 => Self::Four,
            x => panic!("Invalid dimension {}", x),
        }
    }
}

impl ValidDimension {
    #[allow(dead_code)]
    pub fn to_index(self) -> VecIndex {
        match self {
            Self::Three => 3,
            Self::Four => 4,
        }
    }
}
