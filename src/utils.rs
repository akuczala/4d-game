pub fn partial_max<I, T: PartialOrd>(iter: I) -> Option<T>
where
    I: Iterator<Item = T>,
{
    iter.reduce(|acc, x| if x > acc { x } else { acc })
}

pub fn partial_min<I, T: PartialOrd>(iter: I) -> Option<T>
where
    I: Iterator<Item = T>,
{
    iter.reduce(|acc, x| if x < acc { x } else { acc })
}
