//! Comparers: `DescendingComparer`, `LocationComparer`.

use std::cmp::Ordering;

use crate::geometry::IPoint3D;

/// `DescendingComparer<T>`: largest first (`y.CompareTo(x)`). Use with `sort_by(descending)`.
pub fn descending<T: Ord>(x: &T, y: &T) -> Ordering {
    y.cmp(x)
}

/// `LocationComparer`: orders points by weighted distance to a fixed point, nearest first. The x and y
/// deltas count eleven times a z delta. (There is no shared `GetInstance` singleton: make one.)
#[derive(Copy, Clone, Debug)]
pub struct LocationComparer<P: IPoint3D> {
    pub relative_to: P,
}

impl<P: IPoint3D> LocationComparer<P> {
    pub fn new(relative_to: P) -> LocationComparer<P> {
        LocationComparer { relative_to }
    }

    pub fn distance(&self, p: &impl IPoint3D) -> i32 {
        let x = self.relative_to.x().wrapping_sub(p.x()).wrapping_mul(11);
        let y = self.relative_to.y().wrapping_sub(p.y()).wrapping_mul(11);
        let z = self.relative_to.z().wrapping_sub(p.z());
        x.wrapping_mul(x)
            .wrapping_add(y.wrapping_mul(y))
            .wrapping_add(z.wrapping_mul(z))
    }

    /// `Compare(x, y)`: the sign of `distance(x) - distance(y)`.
    pub fn compare(&self, x: &impl IPoint3D, y: &impl IPoint3D) -> Ordering {
        self.distance(x).wrapping_sub(self.distance(y)).cmp(&0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Point3D;

    #[test]
    fn descending_sorts_largest_first() {
        let mut v = vec![3, 9, 1, 5];
        v.sort_by(descending);
        assert_eq!(v, [9, 5, 3, 1]);
    }

    #[test]
    fn nearest_first() {
        let c = LocationComparer::new(Point3D::new(100, 100, 0));
        let mut pts = [
            Point3D::new(110, 100, 0),
            Point3D::new(101, 100, 0),
            Point3D::new(100, 100, 50),
        ];
        pts.sort_by(|a, b| c.compare(a, b));
        assert_eq!(pts[0], Point3D::new(101, 100, 0));
        assert_eq!(c.distance(&Point3D::new(101, 100, 3)), 11 * 11 + 9);
        assert_eq!(c.compare(&pts[0], &pts[0]), Ordering::Equal);
    }
}
