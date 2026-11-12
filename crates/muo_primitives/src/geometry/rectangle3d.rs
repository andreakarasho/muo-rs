//! A 3D rectangle.

use super::{IPoint2D, IPoint3D, Point2D, Point3D, split_plus, try_format};
use crate::parse::FormatError;
use std::fmt;
use std::str::FromStr;

/// `end` is exclusive. `start` and `end` aren't required to be ordered: see [`Rectangle3D::normalized`].
#[derive(Copy, Clone, Eq, PartialEq, Hash, Default, Debug)]
pub struct Rectangle3D {
    pub start: Point3D,
    pub end: Point3D,
}

impl Rectangle3D {
    pub const fn from_points(start: Point3D, end: Point3D) -> Rectangle3D {
        Rectangle3D { start, end }
    }

    pub const fn new(x: i32, y: i32, z: i32, width: i32, height: i32, depth: i32) -> Rectangle3D {
        Rectangle3D {
            start: Point3D::new(x, y, z),
            end: Point3D::new(
                x.wrapping_add(width),
                y.wrapping_add(height),
                z.wrapping_add(depth),
            ),
        }
    }

    pub const fn x(&self) -> i32 {
        self.start.x
    }

    pub const fn y(&self) -> i32 {
        self.start.y
    }

    pub const fn z(&self) -> i32 {
        self.start.z
    }

    pub const fn width(&self) -> i32 {
        self.end.x.wrapping_sub(self.start.x)
    }

    pub const fn height(&self) -> i32 {
        self.end.y.wrapping_sub(self.start.y)
    }

    pub const fn depth(&self) -> i32 {
        self.end.z.wrapping_sub(self.start.z)
    }

    /// Start holds the minimum coordinates and End the maximum in every dimension. Use it when the
    /// ordering matters (Contains checks, Z range calculations).
    pub fn normalized(&self) -> Rectangle3D {
        let (s, e) = (self.start, self.end);
        Rectangle3D {
            start: Point3D::new(s.x.min(e.x), s.y.min(e.y), s.z.min(e.z)),
            end: Point3D::new(s.x.max(e.x), s.y.max(e.y), s.z.max(e.z)),
        }
    }

    pub fn make_hold(&mut self, r: Rectangle3D) {
        self.start.x = self.start.x.min(r.start.x);
        self.start.y = self.start.y.min(r.start.y);
        self.start.z = self.start.z.min(r.start.z);
        self.end.x = self.end.x.max(r.end.x);
        self.end.y = self.end.y.max(r.end.y);
        // An old slip (`<` where `>` was meant): the end z only ever shrinks. Kept because results depend on it.
        self.end.z = self.end.z.min(r.end.z);
    }

    pub fn contains(&self, p: Point3D) -> bool {
        self.contains_xyz(p.x, p.y, p.z)
    }

    pub fn contains_xyz(&self, x: i32, y: i32, z: i32) -> bool {
        self.contains_xy(x, y) && z >= self.start.z && z < self.end.z
    }

    fn contains_xy(&self, x: i32, y: i32) -> bool {
        x >= self.start.x && x < self.end.x && y >= self.start.y && y < self.end.y
    }

    pub fn contains_2d(&self, p: Point2D) -> bool {
        self.contains_xy(p.x, p.y)
    }

    pub fn contains_ipoint_2d(&self, p: &impl IPoint2D) -> bool {
        self.contains_xy(p.x(), p.y())
    }

    pub fn contains_ipoint_3d(&self, p: &impl IPoint3D) -> bool {
        self.contains_xyz(p.x(), p.y(), p.z())
    }

    pub fn try_format(&self, buf: &mut [u8]) -> Option<usize> {
        try_format(buf, format_args!("{self}"))
    }

    pub fn try_parse(s: &str) -> Option<Rectangle3D> {
        let (a, b) = split_plus(s)?;
        Some(Rectangle3D {
            start: Point3D::try_parse(a)?,
            end: Point3D::try_parse(b)?,
        })
    }
}

/// `(x, y, z)+(width, height, depth)`
impl fmt::Display for Rectangle3D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({}, {}, {})+({}, {}, {})",
            self.x(),
            self.y(),
            self.z(),
            self.width(),
            self.height(),
            self.depth()
        )
    }
}

impl FromStr for Rectangle3D {
    type Err = FormatError;
    fn from_str(s: &str) -> Result<Rectangle3D, FormatError> {
        Rectangle3D::try_parse(s).ok_or(FormatError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_string() {
        let (max, min) = (i32::MAX, i32::MIN);
        assert_eq!(
            Rectangle3D::new(0, 0, 0, 0, 0, 0).to_string(),
            "(0, 0, 0)+(0, 0, 0)"
        );
        assert_eq!(
            Rectangle3D::new(1, 1, 1, 1, 1, 1).to_string(),
            "(1, 1, 1)+(1, 1, 1)"
        );
        assert_eq!(
            Rectangle3D::new(max, max, max, max, max, max).to_string(),
            format!("({max}, {max}, {max})+({max}, {max}, {max})")
        );
        assert_eq!(
            Rectangle3D::new(min, min, min, min, min, min).to_string(),
            format!("({min}, {min}, {min})+({min}, {min}, {min})")
        );
    }

    #[test]
    fn try_format_fails() {
        let mut buf = [0u8; 1];
        assert_eq!(
            Rectangle3D::new(1, 1, 1, 1, 1, 1).try_format(&mut buf),
            None
        );
    }

    #[test]
    fn try_parse() {
        let r = Rectangle3D::from_points(Point3D::new(101, 23, 10), Point3D::new(55, 89, 1));
        assert_eq!(Rectangle3D::try_parse("(101, 23, 10)+(55, 89, 1)"), Some(r));
        assert_eq!(" (101,23,10)+ (55, 89,1)  ".parse(), Ok(r));
        for bad in [
            "101, 23, 10)+( 55, 89, 1)",
            "(101, 23, 10)+(55, 89, 1",
            "()",
            "(,)",
            "(|)",
            "(101)",
            "(101,)",
            "(101,23)",
            "(,23)",
            "(,23)+(,55)",
            "(,23,)+(,89)",
            "(101,,10)+(55, 89 1)",
            "(,,)+(55,89, 1)",
        ] {
            assert_eq!(Rectangle3D::try_parse(bad), None, "{bad}");
            assert!(bad.parse::<Rectangle3D>().is_err());
        }
    }

    #[test]
    fn normalized_and_contains() {
        let r = Rectangle3D::from_points(Point3D::new(10, 10, 5), Point3D::new(0, 0, -5));
        let n = r.normalized();
        assert_eq!(n.start, Point3D::new(0, 0, -5));
        assert_eq!(n.end, Point3D::new(10, 10, 5));
        assert!(n.contains(Point3D::new(0, 0, -5)) && !n.contains(Point3D::new(0, 0, 5)));
        assert!(n.contains_2d(Point2D::new(9, 9)) && !n.contains_2d(Point2D::new(10, 9)));
    }
}
