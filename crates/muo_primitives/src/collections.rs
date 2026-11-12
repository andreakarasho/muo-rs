//! `OrderedSet` and `SortedRangeIndex`.
//!
//! Skipped on purpose: `PooledRefList`, `PooledRefQueue`, `ArrayEnumerator`, `CollectionHelpers`,
//! `CollectionThrowStrings` and `HashHelpers` (pooling, enumerator structs and exception text; `Vec`,
//! `VecDeque` and std iterators replace them) and `ValueLinkList` (an intrusive list for pooled nodes).

/// `OrderedSet<T>`: insertion order is observable. `insert_full` gives the `Add` order index (the
/// existing index when the value was already there).
pub type OrderedSet<T> = indexmap::IndexSet<T>;

/// Immutable membership index over inclusive ranges, coalesced into disjoint runs so one binary search
/// decides membership.
#[derive(Clone, Debug, Default)]
pub struct SortedRangeIndex<T> {
    mins: Vec<T>,
    maxs: Vec<T>,
}

impl<T: Ord + Copy> SortedRangeIndex<T> {
    pub fn empty() -> SortedRangeIndex<T> {
        SortedRangeIndex {
            mins: Vec::new(),
            maxs: Vec::new(),
        }
    }

    /// Sorts by minimum, then coalesces (`Array.Sort(ranges, ByMin)` + `Build`).
    pub fn from_ranges(mut ranges: Vec<(T, T)>) -> SortedRangeIndex<T> {
        ranges.sort_by_key(|r| r.0);
        SortedRangeIndex::build(&ranges)
    }

    /// `Build`: ranges already sorted ascending by minimum; overlapping and nested ranges merge.
    pub fn build(sorted_by_min: &[(T, T)]) -> SortedRangeIndex<T> {
        let mut mins: Vec<T> = Vec::new();
        let mut maxs: Vec<T> = Vec::new();
        for &(min, max) in sorted_by_min {
            match maxs.last_mut() {
                Some(cur_max) if min <= *cur_max => {
                    if max > *cur_max {
                        *cur_max = max;
                    }
                }
                _ => {
                    mins.push(min);
                    maxs.push(max);
                }
            }
        }
        SortedRangeIndex { mins, maxs }
    }

    /// Number of disjoint runs.
    pub fn len(&self) -> usize {
        self.mins.len()
    }

    pub fn is_empty(&self) -> bool {
        self.mins.is_empty()
    }

    /// Finds the rightmost run whose minimum is <= `value` and tests the value against its maximum.
    pub fn contains(&self, value: T) -> bool {
        let found = self.mins.partition_point(|m| *m <= value);
        found > 0 && value <= self.maxs[found - 1]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordered_set_keeps_insertion_order() {
        // OrderedSetTests.TestOrderedSet
        let expected = [6, 2, 4, 23, 0];
        let mut set = OrderedSet::new();
        for (i, v) in expected.iter().enumerate() {
            assert_eq!(set.insert_full(*v), (i, true));
        }
        assert_eq!(set.insert_full(4), (2, false)); // the existing order index
        assert_eq!(set.len(), expected.len());
        assert!(set.contains(&4) && !set.contains(&8));
        assert_eq!(set.iter().copied().collect::<Vec<_>>(), expected);
    }

    #[test]
    fn ranges_coalesce() {
        let idx = SortedRangeIndex::from_ranges(vec![
            (50, 60),
            (10, 20),
            (15, 30),
            (12, 13),
            (31, 40),
            (100, 100),
        ]);
        // 10..=30 and 31..=40 do not touch (31 > 30), 50..=60 and 100 are separate
        assert_eq!(idx.len(), 4);
        for v in [10, 20, 30, 31, 40, 50, 60, 100] {
            assert!(idx.contains(v), "{v}");
        }
        for v in [9, 41, 49, 61, 99, 101] {
            assert!(!idx.contains(v), "{v}");
        }
        assert!(!SortedRangeIndex::<u8>::empty().contains(0));
        assert!(SortedRangeIndex::<i32>::from_ranges(vec![]).is_empty());
    }
}
