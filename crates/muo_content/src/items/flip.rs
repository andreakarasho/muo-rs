//! `[Flippable(itemIDs..)]`, `[DynamicFlipping]` and `[Furniture]` as data
//! (`flip_table`), the `Flip` command and
//! `FlippableAttribute.Flip`.
//!
//! The table is keyed by the full class name, so a lookup is `GetCustomAttributes(typeof(FlippableAttribute),
//! false)`: a subclass of a flippable class is not flippable unless it declares the attribute itself ([`flip_ids`]).
//! `PlayerMobile`'s drag-and-drop rotation reads it with `inherit: true` ([`inherited_flip_ids`]).
//!
//! A bare `[Flippable]` (no ids) invokes the class's public `Flip()` method; those methods are `fn`s registered
//! with [`register_own_flip`] (clothing classes keep theirs in `ClothingClass::flip`).

use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::OnceLock;

use muo_primitives::Direction;
use muo_server::World;
use muo_server::commands::{CommandEventArgs, CommandSystem};
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ImplFlag, ItemType, deps, ops};

use super::{addon, addon_container};
use muo_server::mobile::AccessLevel;
use muo_server::target::{self, TargetFlags, Targeted};

use super::flip_table::{DYNAMIC_FLIPPING, FLIPPABLE, FLIPPABLE_OWN, FURNITURE};

struct Tables {
    flippable: HashMap<&'static str, &'static [i32]>,
    own: HashSet<&'static str>,
    dynamic: HashSet<&'static str>,
    furniture: HashSet<&'static str>,
}

fn tables() -> &'static Tables {
    static T: OnceLock<Tables> = OnceLock::new();
    T.get_or_init(|| {
        let mut flippable = HashMap::new();
        for (name, ids) in FLIPPABLE {
            flippable.entry(*name).or_insert(*ids);
        }
        Tables {
            flippable,
            own: FLIPPABLE_OWN.iter().copied().collect(),
            dynamic: DYNAMIC_FLIPPING.iter().copied().collect(),
            furniture: FURNITURE.iter().copied().collect(),
        }
    })
}

/// The `Flip()` methods of classes with a bare `[Flippable]`, by full class name.
#[derive(Default)]
pub struct OwnFlips(pub HashMap<&'static str, fn(&mut World, ItemId)>);

/// Registers `public void Flip()` of `full_name` (run it from a `Configure` phase).
pub fn register_own_flip(w: &mut World, full_name: &'static str, f: fn(&mut World, ItemId)) {
    w.resources.get_mut::<OwnFlips>().0.insert(full_name, f);
}

/// The `ItemIDs` of the `[Flippable(..)]` the class itself declares (`inherit: false`).
pub fn flip_ids(ty: &ItemType) -> Option<&'static [i32]> {
    tables().flippable.get(ty.full_name).copied()
}

/// Whether the class declares a bare `[Flippable]` (its `Flip()` method).
pub fn is_own_flip(ty: &ItemType) -> bool {
    tables().own.contains(ty.full_name)
}

/// `type.IsDefined(attribute, true)`: the class or any base has it.
fn defined_inherited(ty: &'static ItemType, set: &HashSet<&'static str>) -> bool {
    let mut t = Some(ty);
    while let Some(c) = t {
        if set.contains(c.full_name) {
            return true;
        }
        t = c.parent;
    }
    false
}

/// `[Furniture]` or `[DynamicFlipping]` (inherited) and then the nearest `[Flippable(..)]` (inherited): the
/// `ItemIDs` `PlayerMobile.OnMoveItem` rotates by.
pub fn inherited_flip_ids(ty: &'static ItemType) -> Option<&'static [i32]> {
    let t = tables();
    if !(defined_inherited(ty, &t.furniture) || defined_inherited(ty, &t.dynamic)) {
        return None;
    }
    let mut c = Some(ty);
    while let Some(cur) = c {
        if let Some(ids) = t.flippable.get(cur.full_name) {
            return Some(ids);
        }
        c = cur.parent;
    }
    None
}

/// `FlippableAttribute.Flip(item)` for the attribute `ty` declares; false when it declares none.
pub fn flip(w: &mut World, id: ItemId) -> bool {
    let Some(ty) = w.item(id).map(|i| i.ty) else {
        return false;
    };
    if let Some(ids) = flip_ids(ty) {
        let current = (ty.item_id)(w, id);
        let mut index = 0;
        for (i, v) in ids.iter().enumerate() {
            if current == *v {
                index = i + 1;
                break;
            }
        }
        if index > ids.len() - 1 {
            index = 0;
        }
        (ty.set_item_id)(w, id, ids[index]);
        return true;
    }
    if tables().own.contains(ty.full_name) {
        // `GetMethod("Flip", Type.EmptyTypes)?.Invoke(..)`; a class without the method does nothing.
        let f = w
            .resources
            .get::<OwnFlips>()
            .and_then(|o| o.0.get(ty.full_name).copied());
        if let Some(f) = f {
            f(w, id);
        } else if super::clothing::is_clothing(w, id)
            && let Some(f) = super::clothing::class(w, id).flip
        {
            f(w, id);
        }
        return true;
    }
    false
}

type AddonFlipFn = fn(&mut World, MobileId, ItemId, Direction);

/// The `[FlippableAddon(directions)]` of a class and its `public virtual void Flip(Mobile, Direction)`.
#[derive(Default)]
pub struct AddonFlips(pub HashMap<&'static str, (&'static [Direction], AddonFlipFn)>);

/// Registers the attribute and `Flip(from, direction)` of the addon class `full_name`.
pub fn register_addon_flip(
    w: &mut World,
    full_name: &'static str,
    directions: &'static [Direction],
    f: fn(&mut World, MobileId, ItemId, Direction),
) {
    w.resources
        .get_mut::<AddonFlips>()
        .0
        .insert(full_name, (directions, f));
}

/// `ClearComponents(item)` of `FlippableAddonAttribute`: every component is unlinked and deleted, then the list cleared.
fn clear_components(w: &mut World, item: ItemId) {
    let (comps, container) = if addon::is_addon(w, item) {
        (addon::components(w, item), false)
    } else if addon_container::is_container_addon(w, item) {
        (addon_container::components(w, item), true)
    } else {
        return;
    };
    for c in comps {
        if container {
            if let Some(d) = w
                .item_mut(c)
                .and_then(|i| i.ext.get_mut::<addon_container::ContainerComponentData>())
            {
                d.addon = None;
            }
        } else if let Some(d) = addon::component_data_mut(w, c) {
            d.addon = None;
        }
        if let Some(ty) = w.item(c).map(|i| i.ty) {
            (ty.delete)(w, c);
        }
    }
    if container {
        if let Some(d) = addon_container::data_mut(w, item) {
            d.components.clear();
        }
    } else if let Some(d) = addon::addon_data_mut(w, item) {
        d.components.clear();
    }
}

/// `FlippableAddonAttribute.Flip(from, addon)`: turns the addon to its next direction, rebuilding its components;
/// when the new place doesn't fit, it goes back and `from` is told why.
pub fn flip_addon(w: &mut World, from: MobileId, item: ItemId) {
    let Some(ty) = w.item(item).map(|i| i.ty) else {
        return;
    };
    let Some(&(directions, flip)) = w
        .resources
        .get::<AddonFlips>()
        .and_then(|f| f.0.get(ty.full_name))
    else {
        return;
    };
    if directions.len() <= 1 {
        return;
    }
    let current = w.item(item).map_or(Direction::NORTH, |i| i.direction);
    let mut index = 0;
    for (i, d) in directions.iter().enumerate() {
        if current == *d {
            index = i + 1;
            break;
        }
    }
    if index >= directions.len() {
        index = 0;
    }

    clear_components(w, item);
    flip(w, from, item, directions[index]);

    let mut house = None;
    let from_map = w.mobile(from).and_then(|m| m.map);
    let loc = w.item(item).map_or_else(Default::default, |i| i.location);
    ops::set_map(w, item, Some(muo_server::map::MapId::INTERNAL));
    let result = match from_map {
        Some(map) if addon::is_addon(w, item) => {
            addon::could_fit(w, item, loc, map, Some(from), &mut house)
        }
        Some(map) if addon_container::is_container_addon(w, item) => {
            addon_container::could_fit(w, item, loc, map, Some(from), &mut house)
        }
        _ => addon::AddonFitResult::Valid,
    };
    ops::set_map(w, item, from_map);

    if result != addon::AddonFitResult::Valid {
        index = if index == 0 {
            directions.len() - 1
        } else {
            index - 1
        };
        clear_components(w, item);
        flip(w, from, item, directions[index]);
        match result {
            addon::AddonFitResult::Blocked => deps::send_localized_message(w, from, 500269), // You cannot build that there.
            addon::AddonFitResult::NotInHouse => deps::send_localized_message(w, from, 500274), // ..a house that you own!
            addon::AddonFitResult::DoorsNotClosed => {
                deps::send_message(
                    w,
                    from,
                    "You must close all house doors before placing this.",
                );
            }
            addon::AddonFitResult::DoorTooClose => deps::send_localized_message(w, from, 500271), // ..near the door.
            addon::AddonFitResult::NoWall => deps::send_localized_message(w, from, 500268), // ..mounted on something.
            addon::AddonFitResult::Valid => {}
        }
    }
    ops::set_direction(w, item, directions[index]);
}

/// `FlipTarget.OnTarget`.
pub fn on_flip_target(w: &mut World, from: MobileId, targeted: Targeted) {
    let Targeted::Entity(EntityId::Item(item)) = targeted else {
        return;
    };
    let Some(i) = w.item(item) else { return };
    let movable = i.flags.contains(ImplFlag::MOVABLE);
    let player = w
        .mobile(from)
        .is_none_or(|m| m.access_level == AccessLevel::Player);
    if !movable && player {
        return;
    }
    flip(w, item);
}

/// `FlipCommandHandlers.Configure`, and the furniture rotation hook of `PlayerMobile`.
pub fn configure(w: &mut World) {
    CommandSystem::register(
        w,
        "Flip",
        AccessLevel::GameMaster,
        &[],
        Rc::new(|w: &mut World, e: &mut CommandEventArgs| {
            // Usage: Flip; Description: Turns an item.
            let mut t = target::Target::new(w, -1, false, TargetFlags::NONE);
            t.handlers.on_target = Some(Rc::new(on_flip_target));
            let tid = target::add_target(w, t);
            target::set_mobile_target(w, e.mobile, Some(tid));
        }),
    );
    w.resources
        .get_mut::<crate::player_mobile::hooks::SystemHooks>()
        .furniture_flip_ids = Some(|w, item| {
        let ty = w.item(item)?.ty;
        inherited_flip_ids(ty).map(<[i32]>::to_vec)
    });
}

pub fn register(types: &mut muo_server::type_registry::TypeRegistry) {
    use muo_server::type_registry::Phase;
    types.register_phase_default(
        Phase::Configure,
        "Server.Items.FlipCommandHandlers.Configure",
        configure,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::items::cont::WOODEN_BOX;

    #[test]
    fn table_has_no_duplicate_classes() {
        let mut seen = HashSet::new();
        for (n, _) in FLIPPABLE {
            assert!(seen.insert(*n), "{n} declared twice");
        }
    }

    #[test]
    fn flip_cycles_and_does_not_inherit() {
        let mut w = World::new();
        let id = (WOODEN_BOX.construct.unwrap())(&mut w);
        assert_eq!(flip_ids(&WOODEN_BOX), Some(&[0x9AA, 0xE7D][..]));
        assert!(flip(&mut w, id));
        assert_eq!(w.item(id).unwrap().item_id, 0xE7D);
        assert!(flip(&mut w, id));
        assert_eq!(w.item(id).unwrap().item_id, 0x9AA);
    }
}
