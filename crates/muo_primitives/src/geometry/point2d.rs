//! A 2D point.

use super::{IPoint2D, Point3D, parse_tuple, try_format};
use crate::parse::FormatError;
use std::fmt;
use std::str::FromStr;

/// Orders by x then y, like `CompareTo`. The `<`/`>` comparisons of both axes at once are `lt_all` etc.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Default, Debug)]
pub struct Point2D {
    pub x: i32,
    pub y: i32,
}

impl Point2D {
    pub const ZERO: Point2D = Point2D { x: 0, y: 0 };

    pub const fn new(x: i32, y: i32) -> Point2D {
        Point2D { x, y }
    }

    pub fn gt_all(self, o: Point2D) -> bool {
        self.x > o.x && self.y > o.y
    }

    pub fn lt_all(self, o: Point2D) -> bool {
        self.x < o.x && self.y < o.y
    }

    pub fn ge_all(self, o: Point2D) -> bool {
        self.x >= o.x && self.y >= o.y
    }

    pub fn le_all(self, o: Point2D) -> bool {
        self.x <= o.x && self.y <= o.y
    }

    /// `TryFormat`: writes `(x, y)` as ASCII, `None` when `buf` is too small.
    pub fn try_format(&self, buf: &mut [u8]) -> Option<usize> {
        try_format(buf, format_args!("{self}"))
    }

    pub fn try_parse(s: &str) -> Option<Point2D> {
        let [x, y] = parse_tuple(s)?;
        Some(Point2D { x, y })
    }
}

impl IPoint2D for Point2D {
    fn x(&self) -> i32 {
        self.x
    }
    fn y(&self) -> i32 {
        self.y
    }
}

impl From<Point3D> for Point2D {
    fn from(p: Point3D) -> Point2D {
        Point2D { x: p.x, y: p.y }
    }
}

impl From<(i32, i32)> for Point2D {
    fn from((x, y): (i32, i32)) -> Point2D {
        Point2D { x, y }
    }
}

impl fmt::Display for Point2D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl FromStr for Point2D {
    type Err = FormatError;
    fn from_str(s: &str) -> Result<Point2D, FormatError> {
        Point2D::try_parse(s).ok_or(FormatError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_string() {
        assert_eq!(Point2D::new(0, 0).to_string(), "(0, 0)");
        assert_eq!(Point2D::new(1, 1).to_string(), "(1, 1)");
        let (max, min) = (i32::MAX, i32::MIN);
        assert_eq!(
            Point2D::new(max, max).to_string(),
            format!("({max}, {max})")
        );
        assert_eq!(
            Point2D::new(min, max).to_string(),
            format!("({min}, {max})")
        );
        assert_eq!(
            Point2D::new(min, min).to_string(),
            format!("({min}, {min})")
        );
        assert_eq!(
            Point2D::new(max, min).to_string(),
            format!("({max}, {min})")
        );
    }

    #[test]
    fn try_format_fails() {
        let mut buf = [0u8; 1];
        for p in [
            Point2D::new(0, 0),
            Point2D::new(1, 1),
            Point2D::new(i32::MAX, i32::MAX),
            Point2D::new(i32::MIN, i32::MIN),
        ] {
            assert_eq!(p.try_format(&mut buf), None);
        }
        let mut big = [0u8; 8];
        assert_eq!(Point2D::new(1, 2).try_format(&mut big), Some(6));
        assert_eq!(&big[..6], b"(1, 2)");
    }

    #[test]
    fn try_parse() {
        let p = Point2D::new(101, 23);
        assert_eq!(Point2D::try_parse("(101, 23)"), Some(p));
        assert_eq!("(101, 23)".parse(), Ok(p));
        assert_eq!(Point2D::try_parse(" (101,23)  "), Some(p));
        for bad in [
            "101, 23)", "(101, 23", "()", "(101)", "(101,)", "(,23)", "(1,2,3)",
        ] {
            assert_eq!(Point2D::try_parse(bad), None, "{bad}");
            assert!(bad.parse::<Point2D>().is_err());
        }
    }

    #[test]
    fn compare() {
        assert!(Point2D::new(1, 5) < Point2D::new(2, 0));
        assert!(Point2D::new(2, 1).gt_all(Point2D::new(1, 0)));
        assert!(!Point2D::new(2, 0).gt_all(Point2D::new(1, 0)));
        assert!(Point2D::new(2, 0).ge_all(Point2D::new(1, 0)));
    }
}
