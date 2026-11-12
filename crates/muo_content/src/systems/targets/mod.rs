//! Targets: `BladedItemTarget`, `MoveTarget` and `PickMoveTarget`.
//!
//! A `Target` subclass is a [`Target`] whose overridden virtuals are set in its `handlers`; the constructor's
//! arguments are captured by the closures.

use std::rc::Rc;

use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::ops as item_ops;
use muo_server::mobile::AccessLevel;
use muo_server::mobile::base as mb;
use muo_server::mobile::part2::messages::send_localized_message;
use muo_server::target::{self, Target, TargetFlags, Targeted};

type UnholyBoneCarveFn = fn(&mut World, ItemId, MobileId, ItemId) -> bool;
type CarvableCarveFn = fn(&mut World, EntityId, MobileId, ItemId) -> bool;
type LumberjackingKindlingFn = fn(&mut World, MobileId, ItemId, Targeted) -> KindlingBank;
/// Systems a target calls into that other crates own. An unset hook answers "not that kind of thing".
#[derive(Default)]
pub struct TargetHooks {
    /// `CommandLogging.WriteLine(from, text)`.
    pub command_log: Option<fn(&mut World, MobileId, &str)>,
    /// `targeted is UnholyBone bone` then `bone.Carve(from, blade)`: false when `targeted` isn't one.
    pub unholy_bone_carve: Option<UnholyBoneCarveFn>,
    /// `targeted is ICarvable carvable` then `carvable.Carve(from, blade)`: false when it isn't one.
    pub carvable_carve: Option<CarvableCarveFn>,
    /// `targeted is SwampDragon pet && pet.HasBarding`: the whole branch (the ownership message, or
    /// `HasBarding = false`). False when it isn't a barded swamp dragon.
    pub swamp_dragon_remove_barding: Option<fn(&mut World, MobileId, MobileId) -> bool>,
    /// The Witch Apprentice quest's red mushroom objective (1055036, `obj.Complete()`): true when it consumed the
    /// click.
    pub witch_apprentice_red_mushroom: Option<fn(&mut World, MobileId) -> bool>,
    /// `Lumberjacking.System`: `GetHarvestDetails`, `def.Validate`, `def.GetBank`, `bank.Current < 5`,
    /// `bank.Consume(5, from)`.
    pub lumberjacking_kindling: Option<LumberjackingKindlingFn>,
}

/// What `BladedItemTarget` finds in the harvest bank under the target.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum KindlingBank {
    /// `!GetHarvestDetails(..) || !def.Validate(..)`.
    Invalid,
    /// `def.GetBank(..) == null`.
    NoBank,
    /// `bank.Current < 5`.
    NotEnough,
    /// 5 were consumed.
    Consumed,
}

fn hooks(w: &mut World) -> &TargetHooks {
    w.resources.get_mut::<TargetHooks>()
}

fn send(w: &mut World, to: MobileId, number: i32) {
    send_localized_message(w, to, number, "", 0x3B2);
}

/// `BaseCommand.IsAccessible(from, obj)`: staff of Administrator level and up reach anything; others can't touch a
/// mobile (or what one carries) of an equal or higher access level than their own.
pub fn is_accessible(w: &World, from: MobileId, obj: Option<EntityId>) -> bool {
    let Some(from_level) = w.mobile(from).map(|m| m.access_level) else {
        return true;
    };
    let Some(obj) = obj else { return true };
    if from_level >= AccessLevel::Administrator {
        return true;
    }
    let mob = match obj {
        EntityId::Mobile(m) => Some(m),
        EntityId::Item(i) => match item_ops::root_parent(w, i) {
            Some(EntityId::Mobile(m)) => Some(m),
            _ => None,
        },
    };
    match mob {
        None => true,
        Some(m) => m == from || w.mobile(m).is_none_or(|m| from_level > m.access_level),
    }
}

/// `CommandLogging.Format(o)`.
pub fn format_entity(w: &World, e: EntityId) -> String {
    match e {
        EntityId::Mobile(m) => {
            let Some(mob) = w.mobile(m) else {
                return format!("0x{:X}", m.0.value());
            };
            let s = format!(
                "0x{:X} \"{}\"",
                mob.serial.value(),
                (mob.ty.name)(w, m).unwrap_or_default()
            );
            match mob.account {
                None => format!("{s} (no account)"),
                Some(acct) => {
                    let name = crate::accounting::account(w, acct)
                        .map(|a| a.username.clone())
                        .unwrap_or_default();
                    format!("{s} ('{name}')")
                }
            }
        }
        EntityId::Item(i) => {
            let name = w
                .item(i)
                .map(|it| it.ty.full_name.rsplit('.').next().unwrap_or(""))
                .unwrap_or("");
            format!("0x{:X} ({name})", i.0.value())
        }
    }
}

/// `BladedItemTarget(item)`: range 2, no ground.
pub fn bladed_item_target(w: &mut World, from: MobileId, blade: ItemId) {
    let mut t = Target::new(w, 2, false, TargetFlags::NONE);
    t.handlers.on_target_out_of_range = Some(Rc::new(
        move |w: &mut World, from: MobileId, targeted: Targeted| {
            if let Targeted::Entity(EntityId::Item(bone)) = targeted {
                let carve = hooks(w).unholy_bone_carve;
                let near = item_ops::get_world_location(w, bone);
                if let (Some(carve), true) = (carve, mb::in_range_point3d(w, from, near, 12))
                    && carve(w, bone, from, blade)
                {
                    return;
                }
            }
            send(w, from, 500446); // That is too far away.
        },
    ));
    t.handlers.on_target = Some(Rc::new(
        move |w: &mut World, from: MobileId, targeted: Targeted| {
            bladed_on_target(w, from, blade, targeted);
        },
    ));
    let id = target::add_target(w, t);
    target::set_mobile_target(w, from, Some(id));
}

fn bladed_on_target(w: &mut World, from: MobileId, blade: ItemId, targeted: Targeted) {
    if w.item(blade).is_none_or(|i| i.deleted()) {
        return;
    }

    if let Targeted::Entity(e) = targeted {
        if hooks(w)
            .carvable_carve
            .is_some_and(|f| f(w, e, from, blade))
        {
            return;
        }
        if let EntityId::Mobile(pet) = e
            && hooks(w)
                .swamp_dragon_remove_barding
                .is_some_and(|f| f(w, from, pet))
        {
            return;
        }
    }

    if let Targeted::Static(s) = targeted
        && matches!(s.item_id, 0xD15 | 0xD16) // red mushroom
            && hooks(w).witch_apprentice_red_mushroom.is_some_and(|f| f(w, from))
    {
        return;
    }

    let status = match hooks(w).lumberjacking_kindling {
        Some(f) => f(w, from, blade, targeted),
        None => KindlingBank::Invalid,
    };
    match status {
        KindlingBank::Invalid => send(w, from, 500494), // You can't use a bladed item on that!
        KindlingBank::NoBank => {}
        KindlingBank::NotEnough => send(w, from, 500493), // There's not enough wood here to harvest.
        KindlingBank::Consumed => {
            let Some(item) = make_item(w, "Server.Items.Kindling") else {
                return;
            };
            if muo_server::mobile::part2::items::place_in_backpack(w, from, item) {
                send(w, from, 500491); // You put some kindling into your backpack.
                send(w, from, 500492); // An axe would probably get you more wood.
            } else {
                send(w, from, 500490); // You can't place any kindling into your backpack!
                muo_server::item::base::delete(w, item);
            }
        }
    }
}

/// `MoveTarget(o)`: range -1, ground allowed.
pub fn move_target(w: &mut World, from: MobileId, object: EntityId) {
    let t = Target::simple(w, -1, TargetFlags::NONE, true, move |w, from, targeted| {
        move_on_target(w, from, object, targeted)
    });
    let id = target::add_target(w, t);
    target::set_mobile_target(w, from, Some(id));
}

fn move_on_target(w: &mut World, from: MobileId, object: EntityId, targeted: Targeted) {
    // `o is not IPoint3D ip`: every Targeted is one.
    if !is_accessible(w, from, Some(object)) {
        send(w, from, 500447); // That is not accessible.
        return;
    }

    let p = match targeted {
        Targeted::Entity(EntityId::Item(i)) => item_ops::get_world_top(w, i),
        Targeted::Entity(EntityId::Mobile(m)) => {
            w.mobile(m).map(|m| m.location).unwrap_or_default()
        }
        Targeted::Land(l) => l.location,
        Targeted::Static(s) => s.location,
    };

    if let Some(log) = hooks(w).command_log {
        let level = w.mobile(from).map(|m| m.access_level);
        let f = format_entity(w, EntityId::Mobile(from));
        let o = format_entity(w, object);
        log(w, from, &format!("{level:?} {f} moving {o} to {p}"));
    }

    let map = w.mobile(from).and_then(|m| m.map);
    match object {
        EntityId::Item(item) => {
            if w.item(item).is_some_and(|i| !i.deleted()) {
                item_ops::move_to_world_map(w, item, p, map);
            }
        }
        EntityId::Mobile(m) => {
            if let (true, Some(map)) = (w.mobile(m).is_some_and(|m| !m.deleted), map) {
                mb::move_to_world(w, m, p, map);
            }
        }
    }
}

/// `PickMoveTarget()`: range -1, no ground; picks what to move, then hands out a [`move_target`].
pub fn pick_move_target(w: &mut World, from: MobileId) {
    let t = Target::simple(w, -1, TargetFlags::NONE, false, |w, from, targeted| {
        let o = match targeted {
            Targeted::Entity(e) => Some(e),
            _ => None,
        };
        if !is_accessible(w, from, o) {
            send(w, from, 500447); // That is not accessible.
            return;
        }
        if let Some(o) = o {
            move_target(w, from, o);
        }
    });
    let id = target::add_target(w, t);
    target::set_mobile_target(w, from, Some(id));
}

/// `new T()` for an item class by full name (the content-crate factory, `muo_content::factory`, is the same).
fn make_item(w: &mut World, full_name: &str) -> Option<ItemId> {
    match w.types.find_type_by_full_name(full_name, false)? {
        muo_server::type_registry::EntityType::Item(t) => Some((t.construct?)(w)),
        muo_server::type_registry::EntityType::Mobile(_) => None,
    }
}

/// `CommandLogging.WriteLine(from, text)`: through the command system's hook, else the trace log.
pub fn command_log(w: &mut World, from: MobileId, text: &str) {
    match hooks(w).command_log {
        Some(f) => f(w, from, text),
        None => tracing::info!(target: "commands", "{text}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use muo_primitives::Serial;

    #[test]
    fn staff_cannot_touch_equal_or_higher_staff_unless_admin() {
        let mut w = World::new();
        let a = w.new_mobile(&muo_server::mobile::MOBILE_BASE);
        let b = w.new_mobile(&muo_server::mobile::MOBILE_BASE);
        w.mobile_mut(a).unwrap().access_level = AccessLevel::GameMaster;
        w.mobile_mut(b).unwrap().access_level = AccessLevel::GameMaster;
        assert!(is_accessible(&w, a, Some(EntityId::Mobile(a)))); // self
        assert!(!is_accessible(&w, a, Some(EntityId::Mobile(b)))); // equal level
        w.mobile_mut(b).unwrap().access_level = AccessLevel::Player;
        assert!(is_accessible(&w, a, Some(EntityId::Mobile(b)))); // lower
        w.mobile_mut(b).unwrap().access_level = AccessLevel::Seer;
        w.mobile_mut(a).unwrap().access_level = AccessLevel::Administrator;
        assert!(is_accessible(&w, a, Some(EntityId::Mobile(b)))); // admin
        assert!(is_accessible(&w, a, None));
        let _ = Serial::ZERO;
    }

    #[test]
    fn move_target_takes_a_target_id_and_pick_move_target_is_ground_less() {
        let mut w = World::new();
        let a = w.new_mobile(&muo_server::mobile::MOBILE_BASE);
        pick_move_target(&mut w, a);
        let id = w.mobile(a).unwrap().target.unwrap();
        let t = &w.targets[&id];
        assert_eq!((t.range, t.allow_ground), (-1, false));
        move_target(&mut w, a, EntityId::Mobile(a));
        let id = w.mobile(a).unwrap().target.unwrap();
        let t = &w.targets[&id];
        assert_eq!((t.range, t.allow_ground), (-1, true));
    }
}
