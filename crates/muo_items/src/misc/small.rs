//! `TrashChest`, `TrashBarrel`, `Acid`, `OrigamiPaper` and `EmptyOilFlask`.
//!
//! `TrashBarrel.OnChop` (`IChoppable`) is [`trash_barrel_chop`], found through `CommonItemHooks::choppables`;
//! `OilFlask`/`FishOilFlask` are in `gadgets.rs`.

use muo_primitives::{Point3D, TimeSpan, utility};
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::container::{self, CONTAINER};
use muo_server::item::{ITEM_BASE, ItemType, deps, messages, ops};
use muo_server::mobile::AccessLevel;
use muo_server::packets::message::MessageType;
use muo_server::timer::{self, TimerHandle};
use muo_server::type_registry::TypeRegistry;

use super::{hue, item_id, read_version, write_version};

// ---- TrashChest ----

fn trash_message(w: &mut World, id: ItemId) {
    let number = utility::random_from(&mut w.rng, 1042891, 8);
    messages::public_overhead_message_number(w, id, MessageType::REGULAR, 0x3B2, number, "");
}

fn delete_item(w: &mut World, item: ItemId) {
    if let Some(ty) = w.item(item).map(|i| i.ty) {
        (ty.delete)(w, item);
    }
}

/// `Server.Items.TrashChest`; `[Flippable(0xE41, 0xE40)]`.
pub static TRASH_CHEST: ItemType = ItemType {
    full_name: "Server.Items.TrashChest",
    parent: Some(&CONTAINER),
    construct: Some(|w| {
        let id = container::new_container(w, &TRASH_CHEST, 0xE41);
        ops::set_movable(w, id, false);
        id
    }),
    default_max_weight: |_, _| 0, // A value of 0 signals unlimited weight
    is_deco_container: |_, _| false,
    on_drag_drop: |w, id, from, dropped| {
        if (CONTAINER.on_drag_drop)(w, id, from, dropped) {
            trash_message(w, id);
            delete_item(w, dropped);
            return true;
        }
        false
    },
    on_drag_drop_into: |w, id, from, item, p: Point3D| {
        if container::on_drag_drop_into_default(w, id, from, item, p) {
            trash_message(w, id);
            delete_item(w, item);
            return true;
        }
        false
    },
    serialize: |w, id, out| {
        (CONTAINER.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (CONTAINER.deserialize)(w, id, r)?;
        read_version(r, false)?;
        Ok(())
    },
    ..CONTAINER
};

// ---- TrashBarrel ----

/// `TrashBarrel._timer`.
#[derive(Clone, Debug, Default)]
struct BarrelData {
    timer: Option<TimerHandle>,
}

fn barrel_timer(w: &World, id: ItemId) -> Option<TimerHandle> {
    w.item(id)?.ext.get::<BarrelData>()?.timer
}

fn set_barrel_timer(w: &mut World, id: ItemId, t: Option<TimerHandle>) {
    if let Some(i) = w.item_mut(id) {
        if let Some(d) = i.ext.get_mut::<BarrelData>() {
            d.timer = t;
        } else {
            i.ext = Ext::new(BarrelData { timer: t });
        }
    }
}

/// `_timer = new EmptyTimer(this); _timer.Start();`
fn start_empty_timer(w: &mut World, id: ItemId) {
    let t = timer::start(
        w,
        TimeSpan::from_secs_f64(180.0),
        TimeSpan::ZERO,
        1,
        move |w| {
            barrel_empty(w, id, 501479); // Emptying the trashcan!
        },
    );
    set_barrel_timer(w, id, Some(t));
}

/// `TrashBarrel.Empty(message)`.
pub fn barrel_empty(w: &mut World, id: ItemId, message: i32) {
    let items = w.item(id).map(|i| i.items.clone()).unwrap_or_default();
    if !items.is_empty() {
        messages::public_overhead_message_number(w, id, MessageType::REGULAR, 0x3B2, message, "");
        for item in items.into_iter().rev() {
            delete_item(w, item);
        }
    }
    if let Some(mut t) = barrel_timer(w, id) {
        t.cancel(w);
    }
    set_barrel_timer(w, id, None);
}

fn invalidate_contents(w: &mut World, id: ItemId, from: MobileId) {
    if ops::get_total(w, id, muo_server::item::TotalType::Items) >= 50 {
        barrel_empty(w, id, 501478); // The trash is full!  Emptying!
    } else {
        messages::send_localized_message_to(w, id, from, 1010442, ""); // The item will be deleted in three minutes
        if let Some(mut t) = barrel_timer(w, id) {
            // `_timer.Stop(); _timer.Start()`: restarts the three minutes
            t.cancel(w);
        }
        start_empty_timer(w, id);
    }
}

/// `TrashBarrel.OnChop(from)` (`IChoppable`): true when `id` is a trash barrel.
pub fn trash_barrel_chop(w: &mut World, id: ItemId, from: MobileId) -> bool {
    if !w.item(id).is_some_and(|i| i.ty.is(&TRASH_BARREL)) {
        return false;
    }
    let house = muo_multis::house::access::find_house_at_mobile(w, from);
    if house.is_some_and(|h| muo_multis::house::access::is_co_owner(w, h, from)) {
        let loc = w.item(id).map_or_else(Point3D::default, |i| i.location);
        let map = w.item(id).and_then(|i| i.map);
        muo_server::effects::play_sound(w, loc, map, 0x3B3);
        deps::send_localized_message(w, from, 500461); // You destroy the item.
        if let Some(ty) = w.item(id).map(|i| i.ty) {
            (ty.destroy)(w, id);
        }
    }
    true
}

/// `Server.Items.TrashBarrel`.
pub static TRASH_BARREL: ItemType = ItemType {
    full_name: "Server.Items.TrashBarrel",
    parent: Some(&CONTAINER),
    construct: Some(|w| {
        let id = container::new_container(w, &TRASH_BARREL, 0xE77);
        hue(w, id, 0x3B2);
        ops::set_movable(w, id, false);
        id
    }),
    label_number: |_, _| 1041064, // a trash barrel
    default_max_weight: |_, _| 0, // A value of 0 signals unlimited weight
    is_deco_container: |_, _| false,
    on_drag_drop: |w, id, from, dropped| {
        if (CONTAINER.on_drag_drop)(w, id, from, dropped) {
            invalidate_contents(w, id, from);
            return true;
        }
        false
    },
    on_drag_drop_into: |w, id, from, item, p: Point3D| {
        if container::on_drag_drop_into_default(w, id, from, item, p) {
            invalidate_contents(w, id, from);
            return true;
        }
        false
    },
    serialize: |w, id, out| {
        (CONTAINER.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (CONTAINER.deserialize)(w, id, r)?;
        read_version(r, false)?;
        // [AfterDeserialization(false)] -> Timer.DelayCall(AfterDeserialization)
        timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
            if w.item(id).is_some_and(|i| !i.items.is_empty()) {
                start_empty_timer(w, id);
            }
        });
        Ok(())
    },
    ..CONTAINER
};

// ---- Acid ----

/// `Acid`'s fields (none are serialized: `SkipSerialization` is true).
#[derive(Clone, Debug)]
struct AcidData {
    duration: TimeSpan,
    max_damage: i32,
    min_damage: i32,
    timer: TimerHandle,
    drying: bool,
}

fn acid(w: &World, id: ItemId) -> Option<AcidData> {
    w.item(id)?.ext.get::<AcidData>().cloned()
}

/// `Acid(duration, tickRate, minDamage, maxDamage)`.
pub fn acid_new(
    w: &mut World,
    ty: &'static ItemType,
    duration: TimeSpan,
    tick_rate: TimeSpan,
    min: i32,
    max: i32,
) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0x122A);
    hue(w, id, 0x3F);
    ops::set_movable(w, id, false);
    let timer = timer::start(w, TimeSpan::ZERO, tick_rate, 0, move |w| acid_tick(w, id));
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(AcidData {
            duration,
            max_damage: max,
            min_damage: min,
            timer,
            drying: false,
        });
    }
    id
}

/// `Acid.Damage(m)`.
pub fn acid_damage(w: &mut World, m: MobileId, min: i32, max: i32) {
    let amount = utility::random_min_max(&mut w.rng, min, max);
    if let Some(ty) = w.mobile(m).map(|m| m.ty) {
        (ty.damage)(w, m, amount, None, true, false);
    }
}

fn acid_tick(w: &mut World, id: ItemId) {
    let Some(a) = acid(w, id) else { return };
    let age = w.now - w.item(id).map_or(w.now, |i| i.created);
    if age > a.duration {
        delete_item(w, id);
        return;
    }
    // Dries half way through duration
    if !a.drying && age > a.duration - age {
        if let Some(d) = w.item_mut(id).and_then(|i| i.ext.get_mut::<AcidData>()) {
            d.drying = true;
        }
        item_id(w, id, 0x122B);
    }
    let loc = ops::get_world_location(w, id);
    let map = w.item(id).and_then(|i| i.map);
    let victims: Vec<MobileId> =
        muo_server::map::enumerate::mobiles_in_range(w, map, loc.x, loc.y, 0)
            .filter(|&m| {
                let Some(mob) = w.mobile(m) else { return false };
                mob.access_level == AccessLevel::Player
                    && muo_server::mobile::base::alive(w, m)
                    && !(mob.ty.is_dead_bonded_pet)(w, m)
                    && muo_content::creature::read(w, m, true, |c| c.controlled || c.summoned)
            })
            .collect();
    for m in victims {
        acid_damage(w, m, a.min_damage, a.max_damage);
    }
}

/// `Server.Items.Acid`.
pub static ACID: ItemType = ItemType {
    full_name: "Server.Items.Acid",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        acid_new(
            w,
            &ACID,
            TimeSpan::from_secs_f64(10.0),
            TimeSpan::from_secs_f64(1.0),
            2,
            5,
        )
    }),
    default_name: |_, _| Some("a pool of acid".to_string()),
    skip_serialization: |_, _| true,
    on_delete: |w, id| {
        if let Some(mut a) = acid(w, id) {
            a.timer.cancel(w);
        }
    },
    on_move_over: |w, id, m| {
        if let Some(a) = acid(w, id) {
            acid_damage(w, m, a.min_damage, a.max_damage);
        }
        true
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, true);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        read_version(r, true)?;
        Ok(())
    },
    ..ITEM_BASE
};

// ---- OrigamiPaper ----

/// `Server.Items.OrigamiPaper`.
pub static ORIGAMI_PAPER: ItemType = ItemType {
    full_name: "Server.Items.OrigamiPaper",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| ops::new_item_with_id(w, &ORIGAMI_PAPER, 0x2830)),
    label_number: |_, _| 1030288, // origami paper
    on_double_click: |w, id, from| {
        let pack = w.mobile(from).and_then(|m| m.backpack);
        let in_pack = match pack {
            Some(p) => ops::is_child_of(w, id, Some(muo_server::ids::EntityId::Item(p))),
            None => ops::is_child_of(w, id, None),
        };
        if !in_pack {
            deps::send_localized_message(w, from, 1042001); // That must be in your pack for you to use it.
            return;
        }
        delete_item(w, id);
        let bac = w.mobile(from).map_or(0, |m| m.bac);
        let kinds: [&'static ItemType; 6] = [
            &super::leaves::ORIGAMI_BUTTERFLY,
            &super::leaves::ORIGAMI_SWAN,
            &super::leaves::ORIGAMI_FROG,
            &super::leaves::ORIGAMI_SHAPE,
            &super::leaves::ORIGAMI_SONGBIRD,
            &super::leaves::ORIGAMI_FISH,
        ];
        let pick = utility::random(&mut w.rng, if bac >= 5 { 6 } else { 5 }) as usize;
        if let Some(construct) = kinds.get(pick).and_then(|t| t.construct) {
            let item = construct(w);
            deps::add_to_backpack(w, from, item);
        }
        deps::send_localized_message(w, from, 1070822); // You fold the paper into an interesting shape.
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, true);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        read_version(r, true)?;
        Ok(())
    },
    ..ITEM_BASE
};

// ---- EmptyOilFlask ----

/// `EmptyOilFlask(amount)`.
pub fn empty_oil_flask_new(w: &mut World, ty: &'static ItemType, amount: i32) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0x1C18);
    ops::set_stackable(w, id, true);
    ops::set_amount(w, id, amount);
    id
}

/// `Server.Items.EmptyOilFlask`.
pub static EMPTY_OIL_FLASK: ItemType = ItemType {
    full_name: "Server.Items.EmptyOilFlask",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| empty_oil_flask_new(w, &EMPTY_OIL_FLASK, 1)),
    label_number: |_, _| 1150866, // empty oil flask
    on_double_click: |w, _, from| deps::send_localized_message(w, from, 500318), // This flask is empty.
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, true);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        read_version(r, true)?;
        Ok(())
    },
    ..ITEM_BASE
};

/// The classes of this file, and `[Flippable]` data of `TrashChest`.
pub static ALL: &[&ItemType] = &[
    &TRASH_CHEST,
    &TRASH_BARREL,
    &ACID,
    &ORIGAMI_PAPER,
    &EMPTY_OIL_FLASK,
];

/// `[Flippable(0xE41, 0xE40)]` of `TrashChest`.
pub static FLIPPABLE: &[(&ItemType, &[i32])] = &[(&TRASH_CHEST, &[0xE41, 0xE40])];

/// Registers the classes of this file.
pub fn register(types: &mut TypeRegistry) {
    for t in ALL {
        types.register_item(t, &[]);
    }
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "Server.Items.TrashBarrel.Configure",
        |w| {
            muo_content::items::common::hooks_mut(w)
                .choppables
                .push(trash_barrel_chop)
        },
    );
}
