//! A value with a weight, for weighted random picks.

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct WeightedValue<T> {
    pub value: T,
    pub weight: i32,
}

impl<T> WeightedValue<T> {
    pub const fn new(weight: i32, value: T) -> WeightedValue<T> {
        WeightedValue { value, weight }
    }

    pub const fn is_valid(&self) -> bool {
        self.weight > 0
    }
}

/// `(weight, value)` tuples convert into one.
impl<T> From<(i32, T)> for WeightedValue<T> {
    fn from((weight, value): (i32, T)) -> WeightedValue<T> {
        WeightedValue { value, weight }
    }
}
