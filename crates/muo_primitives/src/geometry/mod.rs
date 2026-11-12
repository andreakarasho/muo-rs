//! Geometry: Point2D, Point3D, Point3DList, Rectangle2D, Rectangle3D.
//!
//! `WorldLocation` pairs a point with a `Map`, so it lives in muo_server.
//! `int` arithmetic wraps (unchecked).

mod point2d;
mod point3d;
mod point3d_list;
mod rectangle2d;
mod rectangle3d;

pub use point2d::Point2D;
pub use point3d::Point3D;
pub use point3d_list::Point3DList;
pub use rectangle2d::Rectangle2D;
pub use rectangle3d::Rectangle3D;

/// `IPoint2D`
pub trait IPoint2D {
    fn x(&self) -> i32;
    fn y(&self) -> i32;
}

/// `IPoint3D`
pub trait IPoint3D: IPoint2D {
    fn z(&self) -> i32;
}

/// `TryWrite`: the text, or `None` (and nothing written) when `buf` is too small.
pub fn try_format(buf: &mut [u8], args: std::fmt::Arguments<'_>) -> Option<usize> {
    let text = args.to_string();
    let dest = buf.get_mut(..text.len())?;
    dest.copy_from_slice(text.as_bytes());
    Some(text.len())
}

/// The shared body of the `Parse` methods: `(a, b, ..)` with `N` integers.
pub(crate) fn parse_tuple<const N: usize>(s: &str) -> Option<[i32; N]> {
    let s = s.trim().strip_prefix('(')?.strip_suffix(')')?;
    let mut out = [0; N];
    let mut parts = s.split(',');
    for slot in &mut out {
        *slot = crate::parse::try_to_i32(parts.next()?.trim())?;
    }
    // A longer list never parses: the last field keeps its comma and fails ToInt32.
    parts.next().is_none().then_some(out)
}

/// Splits `a+b` at the first `+`, as the rectangle parsers do.
pub(crate) fn split_plus(s: &str) -> Option<(&str, &str)> {
    s.trim().split_once('+')
}
