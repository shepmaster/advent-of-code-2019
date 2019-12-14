use std::iter::FromIterator;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Bounds<T> {
    pub min_x: T,
    pub max_x: T,
    pub min_y: T,
    pub max_y: T,
}

impl<T> Bounds<T>
where
    T: Ord + Copy,
{
    fn from_iter<I>(iter: I) -> Option<Self>
    where
        I: IntoIterator<Item = (T, T)>,
    {
        let mut min_x = None;
        let mut max_x = None;
        let mut min_y = None;
        let mut max_y = None;

        for (x, y) in iter {
            if min_x.map_or(true, |mx| x < mx) {
                min_x = Some(x);
            }
            if max_x.map_or(true, |mx| x > mx) {
                max_x = Some(x);
            }
            if min_y.map_or(true, |my| y < my) {
                min_y = Some(y);
            }
            if max_y.map_or(true, |my| y > my) {
                max_y = Some(y);
            }
        }

        Some(Bounds {
            min_x: min_x?,
            max_x: max_x?,
            min_y: min_y?,
            max_y: max_y?,
        })
    }
}

pub struct BoundsCollect<T>(pub Option<Bounds<T>>);

impl<T> FromIterator<(T, T)> for BoundsCollect<T>
where
    T: Ord + Copy,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (T, T)>,
    {
        BoundsCollect(Bounds::from_iter(iter))
    }
}
