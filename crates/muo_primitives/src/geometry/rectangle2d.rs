//! A 2D rectangle.

use super::{IPoint2D, Point2D, Point3D, split_plus, try_format};
use crate::parse::FormatError;
use std::fmt;
use std::str::FromStr;

/// `end` is exclusive. All arithmetic wraps like unchecked `int`.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Default, Debug)]
pub struct Rectangle2D {
    pub start: Point2D,
    pub end: Point2D,
}

impl Rectangle2D {
    pub const EMPTY: Rectangle2D = Rectangle2D {
        start: Point2D::ZERO,
        end: Point2D::ZERO,
    };

    pub const fn from_points(start: Point2D, end: Point2D) -> Rectangle2D {
        Rectangle2D { start, end }
    }

    pub const fn new(x: i32, y: i32, width: i32, height: i32) -> Rectangle2D {
        Rectangle2D {
            start: Point2D::new(x, y),
            end: Point2D::new(x.wrapping_add(width), y.wrapping_add(height)),
        }
    }

    pub fn set(&mut self, x: i32, y: i32, width: i32, height: i32) {
        *self = Rectangle2D::new(x, y, width, height);
    }

    pub const fn x(&self) -> i32 {
        self.start.x
    }

    pub const fn y(&self) -> i32 {
        self.start.y
    }

    pub const fn width(&self) -> i32 {
        self.end.x.wrapping_sub(self.start.x)
    }

    pub const fn height(&self) -> i32 {
        self.end.y.wrapping_sub(self.start.y)
    }

    pub fn set_x(&mut self, x: i32) {
        self.start.x = x;
    }

    pub fn set_y(&mut self, y: i32) {
        self.start.y = y;
    }

    /// Moves `end` to `start + width`.
    pub fn set_width(&mut self, width: i32) {
        self.end.x = self.start.x.wrapping_add(width);
    }

    pub fn set_height(&mut self, height: i32) {
        self.end.y = self.start.y.wrapping_add(height);
    }

    /// Grows this rectangle to hold `r`.
    pub fn make_hold(&mut self, r: Rectangle2D) {
        self.start.x = self.start.x.min(r.start.x);
        self.start.y = self.start.y.min(r.start.y);
        self.end.x = self.end.x.max(r.end.x);
        self.end.y = self.end.y.max(r.end.y);
    }

    pub fn contains_xy(&self, x: i32, y: i32) -> bool {
        self.start.x <= x && self.start.y <= y && self.end.x > x && self.end.y > y
    }

    pub fn contains(&self, p: Point2D) -> bool {
        self.contains_xy(p.x, p.y)
    }

    pub fn contains_3d(&self, p: Point3D) -> bool {
        self.contains_xy(p.x, p.y)
    }

    pub fn contains_point(&self, p: &impl IPoint2D) -> bool {
        self.contains_xy(p.x(), p.y())
    }

    pub fn try_format(&self, buf: &mut [u8]) -> Option<usize> {
        try_format(buf, format_args!("{self}"))
    }

    pub fn try_parse(s: &str) -> Option<Rectangle2D> {
        let (a, b) = split_plus(s)?;
        Some(Rectangle2D {
            start: Point2D::try_parse(a)?,
            end: Point2D::try_parse(b)?,
        })
    }
}

/// `(x, y)+(width, height)`
impl fmt::Display for Rectangle2D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({}, {})+({}, {})",
            self.x(),
            self.y(),
            self.width(),
            self.height()
        )
    }
}

impl FromStr for Rectangle2D {
    type Err = FormatError;
    fn from_str(s: &str) -> Result<Rectangle2D, FormatError> {
        Rectangle2D::try_parse(s).ok_or(FormatError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_string() {
        let (max, min) = (i32::MAX, i32::MIN);
        assert_eq!(Rectangle2D::new(0, 0, 0, 0).to_string(), "(0, 0)+(0, 0)");
        assert_eq!(Rectangle2D::new(1, 1, 1, 1).to_string(), "(1, 1)+(1, 1)");
        assert_eq!(
            Rectangle2D::new(max, max, max, max).to_string(),
            format!("({max}, {max})+({max}, {max})")
        );
        assert_eq!(
            Rectangle2D::new(min, min, min, min).to_string(),
            format!("({min}, {min})+({min}, {min})")
        );
    }

    #[test]
    fn try_format_fails() {
        let mut buf = [0u8; 1];
        assert_eq!(Rectangle2D::new(1, 1, 1, 1).try_format(&mut buf), None);
        assert_eq!(
            Rectangle2D::new(i32::MAX, i32::MAX, i32::MAX, i32::MAX).try_format(&mut buf),
            None
        );
    }

    #[test]
    fn try_parse() {
        let r = Rectangle2D::from_points(Point2D::new(101, 23), Point2D::new(55, 89));
        assert_eq!(Rectangle2D::try_parse("(101, 23)+(55, 89)"), Some(r));
        assert_eq!(" (101,23)+ (55, 89)  ".parse(), Ok(r));
        for bad in [
            "101, 23)+( 55, 89)",
            "(101, 23)+(55, 89",
            "()",
            "(,)",
            "(|)",
            "(101)",
            "(101,)",
            "(,23)",
            "(,23)+(,55)",
            "(,23,)+(,89)",
            "(,,)+(55,89)",
        ] {
            assert_eq!(Rectangle2D::try_parse(bad), None, "{bad}");
            assert!(bad.parse::<Rectangle2D>().is_err());
        }
    }

    #[test]
    fn contains_is_half_open() {
        let r = Rectangle2D::new(10, 10, 5, 5);
        assert!(r.contains_xy(10, 10) && r.contains_xy(14, 14));
        assert!(!r.contains_xy(15, 10) && !r.contains(Point2D::new(9, 10)));
    }

    #[test]
    fn make_hold_and_setters() {
        let mut r = Rectangle2D::new(10, 10, 5, 5);
        r.make_hold(Rectangle2D::new(5, 12, 20, 1));
        assert_eq!(
            r,
            Rectangle2D::from_points(Point2D::new(5, 10), Point2D::new(25, 15))
        );
        r.set_width(3);
        assert_eq!(r.end.x, 8);
    }
}
