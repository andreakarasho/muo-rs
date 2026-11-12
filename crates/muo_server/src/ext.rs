//! State that subclasses add to `Item` or `Mobile`: a family struct such as `WeaponData`, and any struct of a
//! class below it (`BaseCreature`'s `CreatureData` and `BaseVendor`'s own). At most one value per type.

use std::any::Any;

#[derive(Default)]
pub struct Ext(Vec<Box<dyn Any>>);

impl Ext {
    pub const NONE: Self = Self(Vec::new());

    pub fn new<T: Any>(state: T) -> Self {
        Self(vec![Box::new(state)])
    }

    /// Adds a subclass's state, replacing any earlier value of the same type.
    pub fn insert<T: Any>(&mut self, state: T) {
        match self.get_mut::<T>() {
            Some(old) => *old = state,
            None => self.0.push(Box::new(state)),
        }
    }

    pub fn get<T: Any>(&self) -> Option<&T> {
        self.0.iter().find_map(|s| s.downcast_ref())
    }

    pub fn get_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.0.iter_mut().find_map(|s| s.downcast_mut())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_access() {
        let mut e = Ext::new(5_u32);
        assert_eq!(e.get::<u32>(), Some(&5));
        assert_eq!(e.get::<u8>(), None);
        *e.get_mut::<u32>().unwrap() = 6;
        assert_eq!(e.get::<u32>(), Some(&6));
        assert!(Ext::NONE.get::<u32>().is_none());
    }

    #[test]
    fn a_family_and_a_subclass_coexist() {
        let mut e = Ext::new(5_u32);
        e.insert("vendor");
        e.insert(7_u32);
        assert_eq!(e.get::<u32>(), Some(&7));
        assert_eq!(e.get::<&str>(), Some(&"vendor"));
    }
}
