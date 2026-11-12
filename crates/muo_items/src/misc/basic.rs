//! `Static`, `Blood`, `Blocker`, `LOSBlocker`, `PowerCrystal`, `Scales`, `Guillotine`, `MorphItem` and
//! `WayPoint`: the small classes with a little code of their own.
//!

use muo_content::props::{Ctor, Kind, Param, Value, add_ctors};
use muo_content::utilities::types::{Numeric, PropType};
use muo_primitives::MessageType;
use muo_primitives::{DateTime, TimeSpan, utility};
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, base, deps, ops};
use muo_server::map::enumerate;
use muo_server::mobile::AccessLevel;
use muo_server::net_state::NetStateId;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::target::{TargetFlags, Targeted};
use muo_server::type_registry::{Phase, TypeRegistry};

use super::{item_id, random_list, read_version, write_version};

// ---- Static, LocalizedStatic ----

/// `Static()`: `base(0x80)`, `Movable = false`; the `[Constructible]` ones take an item id.
pub fn static_new(w: &mut World, ty: &'static ItemType, graphic: i32) -> ItemId {
    let id = ops::new_item_with_id(w, ty, graphic);
    ops::set_movable(w, id, false);
    id
}

/// `Static(itemID, count)`: `this(Utility.Random(itemID, count))`.
pub fn static_new_random(w: &mut World, ty: &'static ItemType, item_id: i32, count: i32) -> ItemId {
    let graphic = utility::random_from(&mut w.rng, item_id, count);
    static_new(w, ty, graphic)
}

/// `[SerializationGenerator(0, false)] Static`.
pub static STATIC: ItemType = ItemType {
    full_name: "Server.Items.Static",
    parent: Some(&ITEM_BASE),
    // `Static` has no parameterless constructor: `[add` uses `STATIC_CTORS`. This one is for `factory::make_item`
    // (the multi preview makes a Static and sets its graphic).
    construct: None,
    make: Some(|w| static_new(w, &STATIC, 0x80)),
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        read_version(r, false)?;
        Ok(())
    },
    ..ITEM_BASE
};

/// What `LocalizedStatic` adds.
#[derive(Clone, Debug, Default)]
pub struct LocalizedStaticData {
    pub number: i32,
}

/// `LocalizedStatic.Number`.
pub fn localized_number(w: &World, id: ItemId) -> i32 {
    w.item(id)
        .and_then(|i| i.ext.get::<LocalizedStaticData>())
        .map_or(0, |d| d.number)
}

/// `LocalizedStatic(itemID, number)`.
pub fn localized_static_new(
    w: &mut World,
    ty: &'static ItemType,
    item_id: i32,
    number: i32,
) -> ItemId {
    let id = static_new(w, ty, item_id);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(LocalizedStaticData { number });
    }
    id
}

/// `LocalizedStatic(itemID)`: `itemID < 0x4000 ? 1020000 + itemID : 1078872 + itemID`.
pub fn localized_static_new_default(w: &mut World, ty: &'static ItemType, item_id: i32) -> ItemId {
    let number = if item_id < 0x4000 {
        1020000 + item_id
    } else {
        1078872 + item_id
    };
    localized_static_new(w, ty, item_id, number)
}

/// `[SerializationGenerator(0)] LocalizedStatic`: `[EncodedInt] Number`.
pub static LOCALIZED_STATIC: ItemType = ItemType {
    full_name: "Server.Items.LocalizedStatic",
    parent: Some(&STATIC),
    construct: None, // `[add` uses `LOCALIZED_STATIC_CTORS`

    label_number: |w, id| localized_number(w, id),
    serialize: |w, id, out| {
        (STATIC.serialize)(w, id, out);
        out.write_encoded_int(0);
        out.write_encoded_int(localized_number(w, id));
    },
    deserialize: |w, id, r| {
        (STATIC.deserialize)(w, id, r)?;
        r.read_encoded_int()?;
        let number = r.read_encoded_int()?;
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(LocalizedStaticData { number });
        }
        Ok(())
    },
    ..STATIC
};

// ---- Blood ----

fn blood_start_timer(w: &mut World, id: ItemId) {
    muo_server::timer::start(
        w,
        TimeSpan::from_secs_f64(5.0),
        TimeSpan::ZERO,
        1,
        move |w| {
            if let Some(ty) = w.item(id).map(|i| i.ty) {
                (ty.delete)(w, id);
            }
        },
    );
}

/// `Blood(itemID)`: `Movable = false`, deletes itself after 5 seconds.
pub fn blood_new_with_id(w: &mut World, ty: &'static ItemType, graphic: i32) -> ItemId {
    let id = ops::new_item_with_id(w, ty, graphic);
    ops::set_movable(w, id, false);
    blood_start_timer(w, id);
    id
}

/// `Blood()`: `this(Utility.RandomList(0x1645, 0x122A, .. 0x122F))`.
pub fn blood_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let graphic = random_list(w, &[0x1645, 0x122A, 0x122B, 0x122C, 0x122D, 0x122E, 0x122F]);
    blood_new_with_id(w, ty, graphic)
}

/// `Server.Items.Blood`.
pub static BLOOD: ItemType = ItemType {
    full_name: "Server.Items.Blood",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| blood_new(w, &BLOOD)),
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        read_version(r, false)?;
        blood_start_timer(w, id); // [AfterDeserialization]
        Ok(())
    },
    ..ITEM_BASE
};

// ---- Blocker, LOSBlocker ----

/// `SendGMItem`: the world packet with the GM-visible graphic.
fn send_gm_item(w: &mut World, ns: NetStateId, id: ItemId, gm_item_id: u16) {
    let Some(c) = w.net_state(ns).map(|d| d.caps.clone()) else {
        return;
    };
    let bytes = if c.stygian_abyss() {
        let mut b =
            muo_server::packets::entity::create_world_entity(w, EntityId::Item(id), c.high_seas());
        if b.len() >= 10 {
            b[8..10].copy_from_slice(&gm_item_id.to_be_bytes());
        }
        b
    } else {
        let mut b = muo_server::packets::item::create_world_item(w, id).unwrap_or_default();
        if b.len() >= 9 {
            b[7..9].copy_from_slice(&gm_item_id.to_be_bytes());
        }
        b
    };
    w.send(ns, &bytes);
}

fn send_world_packet_gm(w: &mut World, id: ItemId, ns: NetStateId, world: &[u8], gm_item_id: u16) {
    let gm = w
        .net_state(ns)
        .and_then(|d| d.mobile)
        .and_then(|m| w.mobile(m))
        .is_some_and(|m| m.access_level >= AccessLevel::GameMaster);
    if gm {
        send_gm_item(w, ns, id, gm_item_id);
    } else {
        base::send_world_packet_to_base(w, id, ns, world); // base body: the helper is the overrides' body
    }
}

/// `Server.Items.Blocker`.
pub static BLOCKER: ItemType = ItemType {
    full_name: "Server.Items.Blocker",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let id = ops::new_item_with_id(w, &BLOCKER, 0x21A4);
        ops::set_movable(w, id, false);
        id
    }),
    label_number: |_, _| 503057, // Impassable!
    send_world_packet_to: |w, id, ns, world| send_world_packet_gm(w, id, ns, world, 0x1183),
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        read_version(r, false)?;
        Ok(())
    },
    ..ITEM_BASE
};

/// `Server.Items.LOSBlocker`.
pub static LOS_BLOCKER: ItemType = ItemType {
    full_name: "Server.Items.LOSBlocker",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let id = ops::new_item_with_id(w, &LOS_BLOCKER, 0x21A2);
        ops::set_movable(w, id, false);
        id
    }),
    default_name: |_, _| Some("no line of sight".to_string()),
    send_world_packet_to: |w, id, ns, world| send_world_packet_gm(w, id, ns, world, 0x36FF),
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        read_version(r, false)?;
        Ok(())
    },
    ..ITEM_BASE
};

/// `LOSBlocker.Initialize`: `ItemTable[0x21A2]` becomes a wall that blocks shots, 20 high.
fn los_blocker_initialize(w: &mut World) {
    use muo_data::tile_data::TileFlag;
    if let Some(d) = w.maps.tile_data.items.get_mut(0x21A2) {
        d.flags = TileFlag::WALL | TileFlag::NO_SHOOT;
        d.height = 20;
    }
}

// ---- PowerCrystal ----

/// `Server.Items.PowerCrystal`.
pub static POWER_CRYSTAL: ItemType = ItemType {
    full_name: "Server.Items.PowerCrystal",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| ops::new_item_with_id(w, &POWER_CRYSTAL, 0x1F1C)),
    default_weight: |_, _| 1.0,
    default_name: |_, _| Some("power crystal".to_string()),
    on_double_click: |w, id, from| {
        let loc = ops::get_world_location(w, id);
        if !deps::mobile_in_range(w, from, loc, 3) {
            // I can't reach that.
            muo_server::mobile::part2::messages::local_overhead_message_number(
                w,
                from,
                MessageType::REGULAR,
                0x3B2,
                1019045,
                "",
            );
        } else {
            muo_server::mobile::part2::messages::send_ascii_message(
                w,
                from,
                "This looks like part of a larger contraption.",
            );
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        read_version(r, false)?;
        Ok(())
    },
    ..ITEM_BASE
};

// ---- Scales ----

fn scales_target(w: &mut World, scales: ItemId, from: MobileId, targeted: Targeted) {
    use muo_server::mobile::part2::messages::{send_localized_message, send_message};
    if targeted == Targeted::Entity(EntityId::Item(scales)) {
        send_message(w, from, "It cannot weigh itself.");
        return;
    }
    if let Targeted::Entity(EntityId::Mobile(_)) = targeted {
        deps::send_localized_message(w, from, 502432); // That is too heavy for these scales!
    }
    let item = match targeted {
        Targeted::Entity(EntityId::Item(i)) if w.item(i).is_some_and(|i| i.movable()) => i,
        _ => {
            send_message(w, from, "You cannot weigh that.");
            return;
        }
    };
    let root = ops::root_parent(w, item);
    let parent = w.item(item).and_then(|i| i.parent);
    if root.is_some_and(|r| r != EntityId::Mobile(from)) || parent == Some(EntityId::Mobile(from)) {
        send_message(
            w,
            from,
            "You decide that item's current location is too awkward to get an accurate result.",
        );
        return;
    }
    let amount = if w.item(item).is_some_and(|i| i.amount > 1) {
        "one"
    } else {
        "that"
    };
    let weight = ops::weight(w, item);
    if weight <= 0.0 {
        send_message(
            w,
            from,
            &format!("You place {amount} item on the scale. It is lighter than a feather."),
        );
    } else {
        send_message(
            w,
            from,
            &format!(
                "You place {amount} item on the scale. It weighs {} stones.",
                weight
            ),
        );
    }
    let _ = send_localized_message;
}

/// `Server.Items.Scales`.
pub static SCALES: ItemType = ItemType {
    full_name: "Server.Items.Scales",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| ops::new_item_with_id(w, &SCALES, 0x1852)),
    default_weight: |_, _| 4.0,
    on_double_click: |w, id, from| {
        deps::send_localized_message(w, from, 502431); // What would you like to weigh?
        muo_server::target::begin_target(
            w,
            from,
            1,
            false,
            TargetFlags::NONE,
            move |w, from, t| scales_target(w, id, from, t),
        );
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        read_version(r, false)?;
        Ok(())
    },
    ..ITEM_BASE
};

// ---- Guillotine ----

#[derive(Clone, Debug, Default)]
struct GuillotineData {
    next_use: DateTime,
}

fn guillotine_next_use(w: &World, id: ItemId) -> DateTime {
    w.item(id)
        .and_then(|i| i.ext.get::<GuillotineData>())
        .map_or(DateTime::default(), |d| d.next_use)
}

fn guillotine_item_id(w: &World, id: ItemId) -> i32 {
    w.item(id).map_or(0, |i| (i.ty.item_id)(w, id))
}

fn guillotine_down1(w: &mut World, id: ItemId) {
    let cur = guillotine_item_id(w, id);
    item_id(w, id, if cur == 4656 { 4678 } else { 4712 });
}

fn guillotine_down2(w: &mut World, id: ItemId) {
    let cur = guillotine_item_id(w, id);
    item_id(w, id, if cur == 4678 { 4679 } else { 4713 });
    let p = ops::get_world_location(w, id);
    let Some(map) = w.item(id).and_then(|i| i.map) else {
        return;
    };
    let blood = blood_new_with_id(w, &BLOOD, 4650);
    ops::move_to_world_map(w, blood, p, Some(map));
    for _ in 0..4 {
        let x = p.x - 2 + utility::random(&mut w.rng, 5);
        let y = p.y - 2 + utility::random(&mut w.rng, 5);
        let mut z = p.z;
        if !muo_server::map::fit::can_fit(w, map, x, y, z, 1, false, false, true) {
            z = w.maps.get_average_z(map, x, y);
            if !muo_server::map::fit::can_fit(w, map, x, y, z, 1, false, false, true) {
                continue;
            }
        }
        let loc =
            muo_server::map::fit::get_random_nearby_location(w, map, p, 2, -2, 4, 1, false, false);
        let b = blood_new(w, &BLOOD);
        ops::move_to_world_map(w, b, loc, Some(map));
    }
}

fn guillotine_back_up(w: &mut World, id: ItemId) {
    match guillotine_item_id(w, id) {
        4678 | 4679 => item_id(w, id, 4656),
        4712 | 4713 => item_id(w, id, 4702),
        _ => {}
    }
}

fn guillotine_use(w: &mut World, id: ItemId, from: MobileId) {
    let loc = ops::get_world_location(w, id);
    if !deps::mobile_in_range(w, from, loc, 2) || !deps::in_los_item(w, from, id) {
        // I can't reach that
        muo_server::mobile::part2::messages::local_overhead_message_number(
            w,
            from,
            MessageType::REGULAR,
            0x3B2,
            1019045,
            "",
        );
        return;
    }
    let item_graphic = guillotine_item_id(w, id);
    let visible = w.item(id).is_some_and(|i| i.visible());
    // `Visible && ItemID is 4656 or 4702 && Core.Now >= m_NextUse` (the `is .. or ..` binds only the ItemID test).
    if visible
        && (item_graphic == 4656 || item_graphic == 4702)
        && w.now >= guillotine_next_use(w, id)
    {
        let Some((fx, fy, floc, fmap, ty)) = w
            .mobile(from)
            .map(|m| (m.location.x, m.location.y, m.location, m.map, m.ty))
        else {
            return;
        };
        let far = (fx - loc.x).abs().max((fy - loc.y).abs());
        if utility::random(&mut w.rng, far) < 1 {
            let hurt = (ty.get_hurt_sound)(w, from);
            muo_server::effects::play_sound(w, floc, fmap, hurt);
            let hue = w.mobile(from).map_or(0, |m| m.speech_hue);
            muo_server::mobile::part2::messages::public_overhead_message(
                w,
                from,
                MessageType::REGULAR,
                hue,
                true,
                "Ouch!",
                true,
                AccessLevel::Player,
            );
            let dmg = utility::dice(&mut w.rng, 2, 10, 5);
            muo_content::spells::helper::damage_full(
                w,
                None,
                TimeSpan::from_secs_f64(0.5),
                from,
                None,
                f64::from(dmg),
            );
        }
        let map = w.item(id).and_then(|i| i.map);
        muo_server::effects::play_sound(w, loc, map, 0x387);
        muo_server::timer::start(
            w,
            TimeSpan::from_secs_f64(0.25),
            TimeSpan::ZERO,
            1,
            move |w| guillotine_down1(w, id),
        );
        muo_server::timer::start(
            w,
            TimeSpan::from_secs_f64(0.50),
            TimeSpan::ZERO,
            1,
            move |w| guillotine_down2(w, id),
        );
        muo_server::timer::start(
            w,
            TimeSpan::from_secs_f64(5.0),
            TimeSpan::ZERO,
            1,
            move |w| guillotine_back_up(w, id),
        );
        let next = w.now + TimeSpan::from_secs_f64(10.0);
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(GuillotineData { next_use: next });
        }
    }
}

/// `Server.Items.Guillotine`.
pub static GUILLOTINE: ItemType = ItemType {
    full_name: "Server.Items.Guillotine",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let id = ops::new_item_with_id(w, &GUILLOTINE, 4656);
        ops::set_movable(w, id, false);
        id
    }),
    on_double_click: guillotine_use,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, true);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        read_version(r, true)?;
        guillotine_back_up(w, id); // [AfterDeserialization]
        Ok(())
    },
    ..ITEM_BASE
};

// ---- MorphItem ----

/// What `MorphItem` adds.
#[derive(Clone, Debug, Default)]
pub struct MorphData {
    pub outside_range: i32,
    pub inactive_item_id: i32,
    pub active_item_id: i32,
    pub inside_range: i32,
}

fn morph(w: &World, id: ItemId) -> MorphData {
    w.item(id)
        .and_then(|i| i.ext.get::<MorphData>())
        .cloned()
        .unwrap_or_default()
}

/// `MorphItem.CurrentRange`.
pub fn morph_current_range(w: &World, id: ItemId) -> i32 {
    let d = morph(w, id);
    if guillotine_item_id(w, id) == d.inactive_item_id {
        d.inside_range
    } else {
        d.outside_range
    }
}

/// `MorphItem(inactiveItemID, activeItemID, inRange, outRange)`; the 3-argument form passes `range` twice.
pub fn morph_item_new(
    w: &mut World,
    ty: &'static ItemType,
    inactive_item_id: i32,
    active_item_id: i32,
    in_range: i32,
    out_range: i32,
) -> ItemId {
    let id = ops::new_item_with_id(w, ty, inactive_item_id);
    ops::set_movable(w, id, false);
    if let Some(i) = w.item_mut(id) {
        // the field assignments bypass the clamping setters
        i.ext = Ext::new(MorphData {
            outside_range: out_range,
            inactive_item_id,
            active_item_id,
            inside_range: in_range,
        });
    }
    id
}

/// `MorphItem.Refresh()`.
pub fn morph_refresh(w: &mut World, id: ItemId) {
    let d = morph(w, id);
    let range = morph_current_range(w, id);
    let loc = ops::get_world_location(w, id);
    let map = w.item(id).and_then(|i| i.map);
    let found = enumerate::mobiles_in_range(w, map, loc.x, loc.y, range).any(|m| {
        w.mobile(m)
            .is_some_and(|m| !m.hidden || m.access_level <= AccessLevel::Player)
    });
    item_id(
        w,
        id,
        if found {
            d.active_item_id
        } else {
            d.inactive_item_id
        },
    );
    let visible = guillotine_item_id(w, id) != 0x1;
    ops::set_visible(w, id, visible);
}

/// `Server.Items.MorphItem`.
pub static MORPH_ITEM: ItemType = ItemType {
    full_name: "Server.Items.MorphItem",
    parent: Some(&ITEM_BASE),
    construct: None, // `[add` uses `MORPH_ITEM_CTORS`
    handles_on_movement: |_, _| true,
    on_movement: |w, id, m, old| {
        let range = morph_current_range(w, id);
        let loc = w.item(id).map_or_else(Default::default, |i| i.location);
        let ml = w.mobile(m).map(|m| m.location);
        if ml.is_some_and(|l| utility::in_range_3d(l, loc, range))
            || utility::in_range_3d(old, loc, range)
        {
            morph_refresh(w, id);
        }
    },
    on_map_change: |w, id| {
        if !ops::is_deleted(w, id) {
            morph_refresh(w, id);
        }
    },
    on_location_change: |w, id, _| {
        if !ops::is_deleted(w, id) {
            morph_refresh(w, id);
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, false);
        let d = morph(w, id);
        out.write_i32(d.outside_range);
        out.write_i32(d.inactive_item_id);
        out.write_i32(d.active_item_id);
        out.write_i32(d.inside_range);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        read_version(r, false)?;
        let d = MorphData {
            outside_range: r.read_i32()?,
            inactive_item_id: r.read_i32()?,
            active_item_id: r.read_i32()?,
            inside_range: r.read_i32()?,
        };
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(d);
        }
        morph_refresh(w, id); // [AfterDeserialization]
        Ok(())
    },
    ..ITEM_BASE
};

// ---- WayPoint ----

#[derive(Clone, Debug, Default)]
struct WayPointData {
    next_point: Option<ItemId>,
}

pub fn way_point_next(w: &World, id: ItemId) -> Option<ItemId> {
    w.item(id)
        .and_then(|i| i.ext.get::<WayPointData>())
        .and_then(|d| d.next_point)
}

/// `WayPoint.NextPoint = value`.
pub fn way_point_set_next(w: &mut World, id: ItemId, next: Option<ItemId>) {
    if let Some(d) = w.item_mut(id).and_then(|i| i.ext.get_mut::<WayPointData>()) {
        d.next_point = next;
    }
}

/// `WayPoint(prev)`.
pub fn way_point_new(w: &mut World, ty: &'static ItemType, prev: Option<ItemId>) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0x1f14);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(WayPointData::default());
    }
    super::hue(w, id, 0x498);
    ops::set_visible(w, id, false);
    if let Some(p) = prev {
        way_point_set_next(w, p, Some(id));
    }
    id
}

fn is_way_point(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.ty.is(&WAY_POINT))
}

/// `Server.Items.WayPoint`.
pub static WAY_POINT: ItemType = ItemType {
    full_name: "Server.Items.WayPoint",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| way_point_new(w, &WAY_POINT, None)),
    default_name: |_, _| Some("AI Way Point".to_string()),
    on_double_click: |w, id, from| {
        if w.mobile(from)
            .is_some_and(|m| m.access_level >= AccessLevel::GameMaster)
        {
            muo_server::mobile::part2::messages::send_message(
                w,
                from,
                "Target the next way point in the sequence.",
            );
            muo_server::target::begin_target(
                w,
                from,
                -1,
                false,
                TargetFlags::NONE,
                move |w, from, t| {
                    // NextPointTarget
                    match t {
                        Targeted::Entity(EntityId::Item(p)) if is_way_point(w, p) => {
                            way_point_set_next(w, id, Some(p))
                        }
                        _ => muo_server::mobile::part2::messages::send_message(
                            w,
                            from,
                            "Target a way point.",
                        ),
                    }
                },
            );
        }
    },
    on_single_click: |w, id, from| {
        base::on_single_click_base(w, id, from);
        match way_point_next(w, id) {
            None => ops::label_to(w, id, from, "(Unlinked)"),
            Some(n) => {
                let l = w.item(n).map_or_else(Default::default, |i| i.location);
                ops::label_to(
                    w,
                    id,
                    from,
                    &format!("(Linked: ({}, {}, {}))", l.x, l.y, l.z),
                );
            }
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, false);
        out.write_item(w, way_point_next(w, id));
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        read_version(r, false)?;
        let next = r.read_item(w)?;
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(WayPointData { next_point: next });
        }
        Ok(())
    },
    ..ITEM_BASE
};

// ---- `[add` constructor tables ----

const INT: Kind = Kind::Plain(PropType::Numeric(Numeric::Int));
const fn int(name: &'static str) -> Param {
    Param {
        name,
        kind: INT,
        default: None,
        ty: "",
    }
}

static STATIC_CTORS: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[int("itemID")],
        make: |w, v| EntityId::Item(static_new(w, &STATIC, v[0].i32())),
    },
    Ctor {
        access: None,
        params: &[int("itemID"), int("count")],
        make: |w, v| EntityId::Item(static_new_random(w, &STATIC, v[0].i32(), v[1].i32())),
    },
];

static LOCALIZED_STATIC_CTORS: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[int("itemID")],
        make: |w, v| {
            EntityId::Item(localized_static_new_default(
                w,
                &LOCALIZED_STATIC,
                v[0].i32(),
            ))
        },
    },
    Ctor {
        access: None,
        params: &[int("itemID"), int("number")],
        make: |w, v| {
            EntityId::Item(localized_static_new(
                w,
                &LOCALIZED_STATIC,
                v[0].i32(),
                v[1].i32(),
            ))
        },
    },
];

static MORPH_ITEM_CTORS: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[int("inactiveItemID"), int("activeItemID"), int("range")],
        make: |w, v| {
            let (a, b, r) = (v[0].i32(), v[1].i32(), v[2].i32());
            EntityId::Item(morph_item_new(w, &MORPH_ITEM, a, b, r, r))
        },
    },
    Ctor {
        access: None,
        params: &[
            int("inactiveItemID"),
            int("activeItemID"),
            int("inRange"),
            int("outRange"),
        ],
        make: |w, v| {
            EntityId::Item(morph_item_new(
                w,
                &MORPH_ITEM,
                v[0].i32(),
                v[1].i32(),
                v[2].i32(),
                v[3].i32(),
            ))
        },
    },
];

static WAY_POINT_CTORS: [Ctor; 1] = [Ctor {
    access: None,
    params: &[Param {
        name: "prev",
        kind: Kind::Plain(PropType::Entity),
        default: Some(Value::Entity(None)),
        ty: "WayPoint",
    }],
    make: |w, v| {
        let prev = match v[0].entity() {
            Some(EntityId::Item(p)) if is_way_point(w, p) => Some(p),
            _ => None,
        };
        EntityId::Item(way_point_new(w, &WAY_POINT, prev))
    },
}];

// ---- WayPointSeq ----

/// `WayPointSeqTarget.OnTarget`.
fn way_point_seq_target(w: &mut World, from: MobileId, last: Option<ItemId>, targeted: Targeted) {
    let point = match targeted {
        Targeted::Entity(EntityId::Item(p)) if is_way_point(w, p) => {
            if let Some(l) = last {
                way_point_set_next(w, l, Some(p));
            }
            return;
        }
        Targeted::Entity(EntityId::Item(i)) => w.item(i).map(|i| i.location),
        Targeted::Entity(EntityId::Mobile(m)) => w.mobile(m).map(|m| m.location),
        Targeted::Land(l) => Some(l.location),
        Targeted::Static(s) => Some(s.location),
    };
    let Some(p) = point else {
        muo_server::mobile::part2::messages::send_message(
            w,
            from,
            "Target a position, or another way point.",
        );
        return;
    };
    let point = way_point_new(w, &WAY_POINT, last);
    let map = w.mobile(from).and_then(|m| m.map);
    ops::move_to_world_map(w, point, p, map);
    begin_way_point_seq(w, from, Some(point));
    muo_server::mobile::part2::messages::send_message(
        w,
        from,
        "Target the position of the next way point in the sequence, or target a way point link the newest way point to.",
    );
}

fn begin_way_point_seq(w: &mut World, from: MobileId, last: Option<ItemId>) {
    muo_server::target::begin_target(w, from, -1, true, TargetFlags::NONE, move |w, from, t| {
        way_point_seq_target(w, from, last, t)
    });
}

/// `MovementPath.Path_OnTarget` and `Path`: finds the route to the targeted point and drops a `WayPoint` on every step.
fn path_target(w: &mut World, from: MobileId, targeted: Targeted) {
    use muo_server::mobile::part2::messages::send_message;
    let goal = muo_content::items::addon::surface_point(w, targeted);
    let start = std::time::Instant::now();
    let path = muo_content::pathing::astar::find_path(w, from, goal);
    let ms = start.elapsed().as_millis();
    if path.is_empty() {
        send_message(w, from, &format!("Bitmap path failed: {ms}ms"));
        return;
    }
    send_message(w, from, &format!("Bitmap path success: {ms}ms"));
    let Some((mut p, map)) = w.mobile(from).map(|m| (m.location, m.map)) else {
        return;
    };
    let mut prev = None;
    for d in path {
        muo_server::movement::offset(d, &mut p, 1);
        let point = way_point_new(w, &WAY_POINT, prev);
        ops::move_to_world_map(w, point, p, map);
        prev = Some(point);
    }
}

pub(crate) fn basic_configure(w: &mut World) {
    use muo_server::commands::CommandSystem;
    for (name, t) in [
        ("Server.Items.Static", &STATIC_CTORS[..]),
        ("Server.Items.LocalizedStatic", &LOCALIZED_STATIC_CTORS[..]),
        ("Server.Items.MorphItem", &MORPH_ITEM_CTORS[..]),
        ("Server.Items.WayPoint", &WAY_POINT_CTORS[..]),
    ] {
        add_ctors(w, name, t);
    }
    // Usage: Path; Description: Draws a path from your current location to a targeted location.
    CommandSystem::register(
        w,
        "Path",
        AccessLevel::GameMaster,
        &[],
        std::rc::Rc::new(
            |w: &mut World, e: &mut muo_server::commands::CommandEventArgs| {
                muo_server::target::begin_target(
                    w,
                    e.mobile,
                    -1,
                    true,
                    TargetFlags::NONE,
                    path_target,
                );
                muo_server::mobile::part2::messages::send_message(
                    w,
                    e.mobile,
                    "Target a location and a path will be drawn there.",
                );
            },
        ),
    );
    // Usage: WayPointSeq; Description: Creates a sequence of way points.
    CommandSystem::register(
        w,
        "WayPointSeq",
        AccessLevel::GameMaster,
        &[],
        std::rc::Rc::new(
            |w: &mut World, e: &mut muo_server::commands::CommandEventArgs| {
                muo_server::mobile::part2::messages::send_message(
                    w,
                    e.mobile,
                    "Target the position of the first way point.",
                );
                begin_way_point_seq(w, e.mobile, None);
            },
        ),
    );
}

/// The classes of this file.
pub static ALL: &[(&ItemType, &[&str])] = &[
    (&STATIC, &[]),
    (&LOCALIZED_STATIC, &[]),
    (&BLOOD, &[]),
    (&BLOCKER, &[]),
    (&LOS_BLOCKER, &[]),
    (&POWER_CRYSTAL, &[]),
    (&SCALES, &[]),
    (&GUILLOTINE, &[]),
    (&MORPH_ITEM, &[]),
    (&WAY_POINT, &[]),
];

/// `[Flippable(0x1f14, 0x1f15, 0x1f16, 0x1f17)]` of `WayPoint`: data for the flip command.
pub static FLIPPABLE: &[(&ItemType, &[i32])] = &[(&WAY_POINT, &[0x1f14, 0x1f15, 0x1f16, 0x1f17])];

/// Registers the classes of this file.
pub fn register(types: &mut TypeRegistry) {
    for (t, aliases) in ALL {
        types.register_item(t, aliases);
    }
    types.register_phase_default(
        Phase::Configure,
        "Server.Items.WayPoint.Configure",
        basic_configure,
    );
    types.register_phase_default(
        Phase::Initialize,
        "Server.Items.LOSBlocker.Initialize",
        los_blocker_initialize,
    );
}
