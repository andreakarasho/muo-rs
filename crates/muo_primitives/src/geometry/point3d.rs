//! A 3D point.

use super::{IPoint2D, IPoint3D, Point2D, parse_tuple, try_format};
use crate::parse::FormatError;
use std::fmt;
use std::str::FromStr;

/// Orders by x, y, then z, like `CompareTo`.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Default, Debug)]
pub struct Point3D {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Point3D {
    pub const ZERO: Point3D = Point3D { x: 0, y: 0, z: 0 };

    pub const fn new(x: i32, y: i32, z: i32) -> Point3D {
        Point3D { x, y, z }
    }

    pub const fn from_2d(p: Point2D, z: i32) -> Point3D {
        Point3D { x: p.x, y: p.y, z }
    }

    // These compare every axis at once. `<`, `>=` and `<=` compare z with `>`
    // (an old copy-paste slip); that is kept because callers' results depend on it.
    pub fn gt_all(self, o: Point3D) -> bool {
        self.x > o.x && self.y > o.y && self.z > o.z
    }

    pub fn lt_all(self, o: Point3D) -> bool {
        self.x < o.x && self.y < o.y && self.z > o.z
    }

    pub fn ge_all(self, o: Point3D) -> bool {
        self.x >= o.x && self.y >= o.y && self.z > o.z
    }

    pub fn le_all(self, o: Point3D) -> bool {
        self.x <= o.x && self.y <= o.y && self.z > o.z
    }

    /// `TryFormat`: writes `(x, y, z)` as ASCII, `None` when `buf` is too small.
    pub fn try_format(&self, buf: &mut [u8]) -> Option<usize> {
        try_format(buf, format_args!("{self}"))
    }

    pub fn try_parse(s: &str) -> Option<Point3D> {
        let [x, y, z] = parse_tuple(s)?;
        Some(Point3D { x, y, z })
    }
}

impl IPoint2D for Point3D {
    fn x(&self) -> i32 {
        self.x
    }
    fn y(&self) -> i32 {
        self.y
    }
}

impl IPoint3D for Point3D {
    fn z(&self) -> i32 {
        self.z
    }
}

impl From<(i32, i32, i32)> for Point3D {
    fn from((x, y, z): (i32, i32, i32)) -> Point3D {
        Point3D { x, y, z }
    }
}

impl fmt::Display for Point3D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

impl FromStr for Point3D {
    type Err = FormatError;
    fn from_str(s: &str) -> Result<Point3D, FormatError> {
        Point3D::try_parse(s).ok_or(FormatError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regression_parse() {
        let p = Point3D::new(709, 2236, -2);
        assert_eq!(Point3D::try_parse("(709, 2236, -2)"), Some(p));
        assert_eq!("(709, 2236, -2)".parse(), Ok(p));
    }

    #[test]
    fn to_string() {
        let (max, min) = (i32::MAX, i32::MIN);
        assert_eq!(Point3D::new(0, 0, 0).to_string(), "(0, 0, 0)");
        assert_eq!(Point3D::new(1, 1, 1).to_string(), "(1, 1, 1)");
        assert_eq!(
            Point3D::new(max, max, max).to_string(),
            format!("({max}, {max}, {max})")
        );
        assert_eq!(
            Point3D::new(min, min, min).to_string(),
            format!("({min}, {min}, {min})")
        );
    }

    #[test]
    fn try_format_fails() {
        let mut buf = [0u8; 1];
        assert_eq!(Point3D::new(0, 0, 0).try_format(&mut buf), None);
        assert_eq!(
            Point3D::new(i32::MIN, i32::MIN, i32::MIN).try_format(&mut buf),
            None
        );
        let mut big = [0u8; 9];
        assert_eq!(Point3D::new(1, 2, 3).try_format(&mut big), Some(9));
    }

    #[test]
    fn try_parse() {
        let p = Point3D::new(101, 23, 10);
        assert_eq!(Point3D::try_parse("(101, 23, 10)"), Some(p));
        assert_eq!(Point3D::try_parse(" (101,23,10)  "), Some(p));
        for bad in [
            "101, 23, 10)",
            "(101, 23, 10",
            "()",
            "(,)",
            "(|)",
            "(101)",
            "(101,)",
            "(101,23)",
            "(,23)",
            "(,23,)",
            "(101,,10)",
            "(,,)",
            "(101, 23 1)",
        ] {
            assert_eq!(Point3D::try_parse(bad), None, "{bad}");
            assert!(bad.parse::<Point3D>().is_err());
        }
    }

    #[test]
    fn operator_quirks() {
        let a = Point3D::new(1, 1, 5);
        let b = Point3D::new(2, 2, 1);
        assert!(a.lt_all(b)); // z compared with `>`
        assert!(a.le_all(b));
        assert!(!b.gt_all(a));
    }
}
