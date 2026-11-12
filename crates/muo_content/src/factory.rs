//! `Activator.CreateInstance` over the type registry: classes made by type name from content that can't depend on
//! the leaf crates.

use muo_server::ids::{ItemId, MobileId};
use muo_server::type_registry::EntityType;
use muo_server::world::World;

/// `new T()` for the item class `full_name` (a `Server.Items.Gold`); `None` if it is unknown or has no constructor.
pub fn make_item(w: &mut World, full_name: &str) -> Option<ItemId> {
    match w.types.find_type_by_full_name(full_name, false)? {
        EntityType::Item(t) => (t.construct.or(t.make)?)(w).into(),
        EntityType::Mobile(_) => None,
    }
}

/// `new T()` for the mobile class `full_name`.
pub fn make_mobile(w: &mut World, full_name: &str) -> Option<MobileId> {
    match w.types.find_type_by_full_name(full_name, false)? {
        EntityType::Mobile(t) => (t.construct.or(t.make)?)(w).into(),
        EntityType::Item(_) => None,
    }
}

/// `item.GetType()` is `full_name` or derives from it (`IsAssignableFrom`/`InTypeList`).
pub fn item_is(w: &World, id: ItemId, full_name: &str) -> bool {
    let Some(it) = w.item(id) else { return false };
    let mut t = Some(it.ty);
    while let Some(ty) = t {
        if ty.full_name == full_name {
            return true;
        }
        t = ty.parent;
    }
    false
}

/// As [`item_is`] for a mobile.
pub fn mobile_is(w: &World, id: MobileId, full_name: &str) -> bool {
    let Some(m) = w.mobile(id) else { return false };
    let mut t = Some(m.ty);
    while let Some(ty) = t {
        if ty.full_name == full_name {
            return true;
        }
        t = ty.parent;
    }
    false
}

/// `container.FindItemByType<T>()`: the first item of class `full_name` (or a subclass) in the container or in a
/// container inside it, breadth first.
pub fn find_item_by_type(w: &World, container: ItemId, full_name: &str) -> Option<ItemId> {
    find_item_by_types(w, container, &[full_name])
}

/// `container.FindItemByType(Type[] types)`: the first item (breadth first) that is any of the classes.
pub fn find_item_by_types(w: &World, container: ItemId, full_names: &[&str]) -> Option<ItemId> {
    let mut queue = std::collections::VecDeque::from([container]);
    while let Some(c) = queue.pop_front() {
        let Some(it) = w.item(c) else { continue };
        for &child in &it.items {
            if full_names.iter().any(|n| item_is(w, child, n)) {
                return Some(child);
            }
            queue.push_back(child);
        }
    }
    None
}

/// `new T(args..)` for the item class `full_name`: the first `[Constructible]` overload of its
/// [`crate::props::add_ctors`] table that takes `args.len()` arguments (omitted trailing ones take their defaults;
/// a missing table or no fitting overload falls back to the parameterless `construct` when `args` is empty).
/// For constructors a class calls on other classes (`DropItem(new BlackPearl(amount))`).
pub fn make_item_args(
    w: &mut World,
    full_name: &str,
    args: &[crate::props::Value],
) -> Option<ItemId> {
    use crate::props::{Value, ctors_of};
    use muo_server::ids::EntityId;
    let Some(ctors) = ctors_of(w, full_name) else {
        return if args.is_empty() {
            make_item(w, full_name)
        } else {
            None
        };
    };
    for c in ctors {
        let required = c.params.iter().take_while(|p| p.default.is_none()).count();
        if args.len() < required || args.len() > c.params.len() {
            continue;
        }
        let values: Vec<Value> = c
            .params
            .iter()
            .enumerate()
            .map(|(i, p)| {
                args.get(i)
                    .cloned()
                    .or_else(|| p.default.clone())
                    .unwrap_or(Value::Null)
            })
            .collect();
        if let EntityId::Item(id) = (c.make)(w, &values) {
            return Some(id);
        }
    }
    None
}

/// `new T(amount)`: [`make_item_args`] with one int.
pub fn make_item_amount(w: &mut World, full_name: &str, amount: i32) -> Option<ItemId> {
    make_item_args(w, full_name, &[crate::props::Value::Int(i64::from(amount))])
}
