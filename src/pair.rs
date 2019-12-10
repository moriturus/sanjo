#[derive(Debug, Clone, Copy)]
pub struct Pair<T>
where
    T: num_traits::int::PrimInt,
{
    pub x: T,
    pub y: T,
}

impl<T> From<(T, T)> for Pair<T>
where
    T: num_traits::int::PrimInt,
{
    fn from(tuple: (T, T)) -> Pair<T> {
        Pair::new(tuple.0, tuple.1)
    }
}

impl<T> Into<(T, T)> for Pair<T>
where
    T: num_traits::int::PrimInt,
{
    fn into(self) -> (T, T) {
        (self.x, self.y)
    }
}

impl From<&str> for Pair<u32> {
    fn from(s: &str) -> Pair<u32> {
        use arrayvec::ArrayVec;

        let p = s
            .split('x')
            .map(|s| s.parse().unwrap_or(0u32))
            .collect::<ArrayVec<[_; 2]>>();

        if p.is_full() {
            Pair::new(p[0], p[1])
        } else {
            Pair::new(p[0], p[0])
        }
    }
}

impl<T> Pair<T>
where
    T: num_traits::int::PrimInt,
{
    fn new(x: T, y: T) -> Pair<T> {
        Pair { x, y }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        let s = "32x64";
        let p = Pair::from(s);
        assert_eq!(p.x, 32);
        assert_eq!(p.y, 64);
    }

    #[test]
    fn test_from_str_non_num() {
        let s = "abxcd";
        let p = Pair::from(s);
        assert_eq!(p.x, 0);
        assert_eq!(p.y, 0);
    }

    #[test]
    fn test_from_str_non_x_format() {
        let s = "32";
        let p = Pair::from(s);
        assert_eq!(p.x, 32);
        assert_eq!(p.y, 32);
    }

    #[test]
    fn test_from_str_non_x_format_zero() {
        let s = "abcd";
        let p = Pair::from(s);
        assert_eq!(p.x, 0);
        assert_eq!(p.y, 0);
    }
}
