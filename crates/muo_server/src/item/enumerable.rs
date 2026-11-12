//! Finding the items inside an item, breadth first.
//!
//! A live enumerator would throw `InvalidOperationException` when the container changed during the
//! walk. Rust cannot change the world while iterating
//! `&World`, so there is one snapshot function, [`find_items`], and the typed forms are filters over it. Order is
//! fixed: the container's own items first, then each child container's in the order they were met.

use std::collections::VecDeque;

use crate::ids::ItemId;
use crate::item::ItemType;
use crate::world::World;

/// `FindItems(recurse, predicate)`: every item inside `container` (an item with children) for which `pred`
/// holds, as `FindItemsByTypeEnumerator` yields them. The container itself is not part of the result.
pub fn find_items(
    w: &World,
    container: ItemId,
    recurse: bool,
    mut pred: impl FnMut(&World, ItemId) -> bool,
) -> Vec<ItemId> {
    let mut found = Vec::new();
    let mut containers: VecDeque<ItemId> = VecDeque::new();
    let mut current = Some(container);

    while let Some(cur) = current {
        let items: &[ItemId] = w.item(cur).map_or(&[], |i| i.items.as_slice());
        for &item in items {
            let Some(it) = w.item(item) else { continue };
            if recurse && !it.items.is_empty() {
                containers.push_back(item);
            }
            if pred(w, item) {
                found.push(item);
            }
        }
        current = if recurse {
            containers.pop_front()
        } else {
            None
        };
    }
    found
}

/// `FindItemsByType<T>(recurse, predicate)` and `FindItemsByType(Type, recurse)`: items of class `ty` (or a
/// subclass) for which `pred` holds.
pub fn find_items_by_type(
    w: &World,
    container: ItemId,
    recurse: bool,
    ty: &'static ItemType,
    mut pred: impl FnMut(&World, ItemId) -> bool,
) -> Vec<ItemId> {
    find_items(w, container, recurse, |w, id| {
        w.item(id).is_some_and(|i| i.ty.is(ty)) && pred(w, id)
    })
}

/// `FindItemsByType(ReadOnlySpan<Type>, recurse)`: items of any of `types` (`InTypeList`: assignable).
pub fn find_items_by_types(
    w: &World,
    container: ItemId,
    recurse: bool,
    types: &[&'static ItemType],
) -> Vec<ItemId> {
    find_items(w, container, recurse, |w, id| in_type_list(w, id, types))
}

/// `item.InTypeList(types)`.
pub fn in_type_list(w: &World, id: ItemId, types: &[&'static ItemType]) -> bool {
    w.item(id).is_some_and(|i| types.iter().any(|t| i.ty.is(t)))
}

/// `FindItemByType<T>`: the first match.
pub fn find_item_by_type(
    w: &World,
    container: ItemId,
    recurse: bool,
    ty: &'static ItemType,
) -> Option<ItemId> {
    find_items_by_type(w, container, recurse, ty, |_, _| true)
        .into_iter()
        .next()
}
