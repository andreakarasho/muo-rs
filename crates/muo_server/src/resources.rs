//! Per-system statics: one value per type, held by the `World`.

use std::any::{Any, TypeId};
use std::collections::HashMap;

#[derive(Default)]
pub struct Resources(HashMap<TypeId, Box<dyn Any>>);

impl Resources {
    /// The value of `T`, made with `Default` on first use.
    pub fn get_mut<T: Any + Default>(&mut self) -> &mut T {
        self.0
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(T::default()))
            .downcast_mut()
            .expect("keyed by its own TypeId")
    }

    /// Stores a value, replacing any earlier one.
    pub fn insert<T: Any>(&mut self, v: T) {
        self.0.insert(TypeId::of::<T>(), Box::new(v));
    }

    pub fn get<T: Any>(&self) -> Option<&T> {
        self.0.get(&TypeId::of::<T>())?.downcast_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_then_keeps() {
        let mut r = Resources::default();
        assert_eq!(r.get::<u32>(), None);
        *r.get_mut::<u32>() += 2;
        assert_eq!(r.get::<u32>(), Some(&2));
    }
}
