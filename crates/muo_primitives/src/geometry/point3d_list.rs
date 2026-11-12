//! A reusable list of 3D points.

use super::Point3D;

/// A reusable list of points.
#[derive(Default, Debug, Clone)]
pub struct Point3DList {
    list: Vec<Point3D>,
}

impl Point3DList {
    pub fn new() -> Point3DList {
        Point3DList::default()
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }

    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    pub fn last(&self) -> Option<Point3D> {
        self.list.last().copied()
    }

    pub fn get(&self, index: usize) -> Option<Point3D> {
        self.list.get(index).copied()
    }

    pub fn clear(&mut self) {
        self.list.clear();
    }

    pub fn add(&mut self, x: i32, y: i32, z: i32) {
        self.list.push(Point3D::new(x, y, z));
    }

    pub fn add_point(&mut self, p: Point3D) {
        self.list.push(p);
    }

    /// `ToArray()`: hands the points over and empties the list (`Count` goes to 0).
    pub fn to_array(&mut self) -> Vec<Point3D> {
        std::mem::take(&mut self.list)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_take() {
        let mut l = Point3DList::new();
        assert!(l.to_array().is_empty());
        l.add(1, 2, 3);
        l.add_point(Point3D::new(4, 5, 6));
        assert_eq!(l.len(), 2);
        assert_eq!(l.last(), Some(Point3D::new(4, 5, 6)));
        assert_eq!(l.get(0), Some(Point3D::new(1, 2, 3)));
        assert_eq!(l.to_array().len(), 2);
        assert!(l.is_empty());
    }
}
