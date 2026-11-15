//! The mutation core items: `PlagueBeastInnard`, `PlagueBeastBackpack`, `PlagueBeastBlood`, `PlagueBeastHeart`,
//! `PlagueBeastMutationCore`, the organs and `PlagueBeastVein` (PlagueBeastGland is in `leaves`).
//!
//! All the innard classes keep their state in one [`Pb`] in `Item::ext`; the class tree is kept, and what a
//! subclass overrides is a `match` on the class in the function of the virtual. The plague beast lord itself
//! is reached through [`PlagueLordApi`](super::hooks::PlagueLordApi).
//!
//! `OnBandage` (called by the bandage's target, `skill_items/bandage.rs`) is [`on_bandage`]; `PlagueBeastLord.Carve` builds the backpack with [`backpack_new`] + [`backpack_initialize`].

use muo_content::items::interfaces::{register_carvable, register_scissorable};

use muo_primitives::{MessageType, Point3D, TimeSpan, utility};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, deps, ops};
use muo_server::layer::Layer;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::timer::{self, TimerHandle};
use muo_server::type_registry::TypeRegistry;

use super::hooks::hooks;
use super::util::{delete, deleted, msg};

/// The state of every class of the folder (what a class does not use stays at its default).
#[derive(Clone, Debug, Default)]
pub struct Pb {
    /// `PlagueBeastComponent._organ`.
    pub organ: Option<ItemId>,
    /// `PlagueBeastOrgan._components`.
    pub components: Vec<Option<ItemId>>,
    pub brain_hue: i32,
    pub opened: bool,
    pub opening: bool,
    /// `PlagueBeastRubbleOrgan._veins`.
    pub veins: i32,
    /// `PlagueBeastBackupOrgan._gland`.
    pub gland: Option<ItemId>,
    /// `PlagueBeastMainOrgan._brains`.
    pub brains: i32,
    /// `PlagueBeastVein._cut` / `PlagueBeastMutationCore._cut`.
    pub cut: bool,
    /// `PlagueBeastVein._cutting`.
    pub cutting: bool,
    /// `PlagueBeastHeart._timer` and the timer's `_delay`.
    pub heart_timer: Option<TimerHandle>,
    pub heart_delay: bool,
}

fn pb(w: &World, id: ItemId) -> Pb {
    w.item(id)
        .and_then(|i| i.ext.get::<Pb>())
        .cloned()
        .unwrap_or_default()
}

fn pb_mut(w: &mut World, id: ItemId) -> Option<&mut Pb> {
    let i = w.item_mut(id)?;
    if i.ext.get::<Pb>().is_none() {
        i.ext.insert(Pb::default());
    }
    i.ext.get_mut::<Pb>()
}

fn is(w: &World, id: ItemId, ty: &'static ItemType) -> bool {
    w.item(id).is_some_and(|i| i.ty.is(ty))
}

// ---- the lord ----

/// `RootParent as PlagueBeastLord`.
fn owner(w: &World, id: ItemId) -> Option<MobileId> {
    let api = hooks(w)?.plague_lord?;
    match ops::root_parent(w, id) {
        Some(EntityId::Mobile(m)) if (api.is_lord)(w, m) => Some(m),
        _ => None,
    }
}

fn unfreeze_and_kill(w: &mut World, lord: MobileId) {
    if let Some(api) = hooks(w).and_then(|h| h.plague_lord) {
        (api.unfreeze)(w, lord);
    }
    muo_server::mobile::base::kill(w, lord);
}

fn owner_play_sound(w: &mut World, id: ItemId, sound: i32) {
    if let Some(o) = owner(w, id) {
        muo_server::mobile::part2::effects::play_sound(w, o, sound);
    }
}

fn owner_alive(w: &World, id: ItemId) -> bool {
    owner(w, id).is_some_and(|o| muo_server::mobile::base::alive(w, o))
}

/// `PlagueBeastLord.IsBleeding`: an unpatched blood in its backpack.
pub fn is_bleeding(w: &mut World, lord: MobileId) -> bool {
    let Some(pack) = muo_server::mobile::part1::props_b::backpack(w, lord) else {
        return false;
    };
    let items = w.item(pack).map(|p| p.items.clone()).unwrap_or_default();
    items
        .iter()
        .any(|&i| is(w, i, &PLAGUE_BEAST_BLOOD) && !blood_patched(w, i))
}

/// `PlagueBeastInnard.IsAccessibleTo(check)`.
fn innard_is_accessible(w: &mut World, id: ItemId, check: MobileId) -> bool {
    if muo_content::mx::access_level(w, check) >= muo_server::mobile::AccessLevel::GameMaster {
        return true;
    }
    let Some(o) = owner(w, id) else { return false };
    let ns = w.mobile(check).and_then(|m| m.net_state);
    let opened_by = hooks(w)
        .and_then(|h| h.plague_lord)
        .and_then(|api| (api.opened_by)(w, o));
    let loc = w.mobile(check).map(|m| m.location);
    let in_range = loc.is_some_and(|l| muo_content::mx::in_range_point(w, o, l, 2));
    if !in_range {
        muo_server::mobile::part2::messages::private_overhead_message_number(
            w,
            o,
            MessageType::LABEL,
            0x3B2,
            500446,
            "",
            ns,
        ); // That is too far away.
    } else if opened_by.is_some_and(|b| b != check) {
        // That is being used by someone else
        muo_server::mobile::part2::messages::private_overhead_message_number(
            w,
            o,
            MessageType::LABEL,
            0x3B2,
            500365,
            "",
            ns,
        );
    } else if w.mobile(o).is_some_and(|m| m.frozen) {
        return true;
    }
    false
}

// ---- PlagueBeastInnard ----

/// `PlagueBeastInnard(itemID, hue)`.
fn innard_init(w: &mut World, id: ItemId, hue: i32) {
    super::hue(w, id, hue);
    ops::set_movable(w, id, false);
}

fn new_innard(w: &mut World, ty: &'static ItemType, item_id: i32, hue: i32) -> ItemId {
    let id = ops::new_item_with_id(w, ty, item_id);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(Pb::default());
    }
    innard_init(w, id, hue);
    id
}

/// `[AfterDeserialization(false)]` of the innard: one with no live lord goes away.
fn innard_after_deserialization(w: &mut World, id: ItemId) {
    if !owner_alive(w, id) && !deleted(w, id) {
        delete(w, id);
    }
}

fn innard_deserialize_tail(w: &mut World, id: ItemId) {
    timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
        innard_after_deserialization(w, id)
    });
}

/// `Server.Items.PlagueBeastInnard` (abstract here: no `[Constructible]`).
pub static PLAGUE_BEAST_INNARD: ItemType = ItemType {
    full_name: "Server.Items.PlagueBeastInnard",
    parent: Some(&ITEM_BASE),
    construct: None,
    default_weight: crate::class_props::default_weight,
    default_name: crate::class_props::default_name,
    is_accessible_to: innard_is_accessible,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        innard_deserialize_tail(w, id);
        Ok(())
    },
    ..ITEM_BASE
};

/// `Scissor(from, scissors)` of the whole folder: false unless a class says otherwise.
fn scissor(w: &mut World, id: ItemId, from: MobileId, scissors: ItemId) -> bool {
    if is(w, id, &PLAGUE_BEAST_MUTATION_CORE) {
        return core_scissor(w, id, from);
    }
    if is(w, id, &PLAGUE_BEAST_VEIN) {
        return vein_scissor(w, id, from, scissors);
    }
    if is(w, id, &PLAGUE_BEAST_ORGAN) {
        return organ_scissor(w, id, from, scissors);
    }
    false
}

/// `Carve(from, with)` of the folder: the rock and backup organs refuse.
fn carve(w: &mut World, id: ItemId, from: MobileId, with: ItemId) {
    if (is(w, id, &PLAGUE_BEAST_ROCK_ORGAN) || is(w, id, &PLAGUE_BEAST_BACKUP_ORGAN))
        && innard_is_accessible(w, id, from)
    {
        // This is too crude an implement for such a procedure.
        muo_server::item::messages::public_overhead_message_number(
            w,
            with,
            muo_server::packets::message::MessageType::REGULAR,
            0x3B2,
            1071896,
            "",
        );
    }
}

/// `OnBandage(from)`: the blood is the one that patches.
pub fn on_bandage(w: &mut World, id: ItemId, from: MobileId) -> bool {
    if is(w, id, &PLAGUE_BEAST_BLOOD) {
        return blood_on_bandage(w, id, from);
    }
    false
}

// ---- PlagueBeastComponent ----

/// `PlagueBeastComponent(itemID, hue, movable = false)`.
pub fn plague_beast_component_new(w: &mut World, item_id: i32, hue: i32, movable: bool) -> ItemId {
    let id = new_innard(w, &PLAGUE_BEAST_COMPONENT, item_id, hue);
    ops::set_movable(w, id, movable);
    id
}

fn comp_is_brain(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.item_id == 0x1CF0)
}

fn comp_is_gland(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.item_id == 0x1CEF)
}

fn comp_is_receptacle(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.item_id == 0x9DF)
}

fn set_organ(w: &mut World, c: ItemId, organ: Option<ItemId>) {
    if let Some(d) = pb_mut(w, c) {
        d.organ = organ;
    }
}

/// `organ.AddToComponents(c)`.
fn add_to_components(w: &mut World, organ: ItemId, c: ItemId) {
    if let Some(d) = pb_mut(w, organ) {
        d.components.push(Some(c));
    }
}

/// `organ.RemoveFromComponents(c)`.
fn remove_from_components(w: &mut World, organ: ItemId, c: ItemId) {
    if let Some(d) = pb_mut(w, organ)
        && let Some(p) = d.components.iter().position(|x| *x == Some(c))
    {
        d.components.remove(p);
    }
}

/// `Server.Items.PlagueBeastComponent`.
pub static PLAGUE_BEAST_COMPONENT: ItemType = ItemType {
    full_name: "Server.Items.PlagueBeastComponent",
    parent: Some(&PLAGUE_BEAST_INNARD),
    construct: None,
    drop_to_item: |w, id, from, target, p: Point3D| {
        is(w, target, &PLAGUE_BEAST_BACKPACK)
            && (PLAGUE_BEAST_INNARD.drop_to_item)(w, id, from, target, p)
    },
    allow_secure_trade: |_, _, _, _, _, _| false,
    drop_to_mobile: |_, _, _, _, _| false,
    drop_to_world: |_, _, _, _| false,
    on_drag_drop: |w, id, from, dropped| {
        if let Some(organ) = pb(w, id).organ
            && organ_on_dropped(w, organ, from, dropped, id)
            && is(w, dropped, &PLAGUE_BEAST_COMPONENT)
        {
            add_to_components(w, organ, dropped);
        }
        true
    },
    on_drag_lift: |w, id, from| {
        if innard_is_accessible(w, id, from) {
            if let Some(organ) = pb(w, id).organ
                && organ_on_lifted(w, organ, from, id)
            {
                // * You rip the organ out of the plague beast's flesh *
                msg(
                    w,
                    from,
                    if comp_is_gland(w, id) {
                        1071895
                    } else {
                        1071914
                    },
                );
                remove_from_components(w, organ, id);
                set_organ(w, id, None);
                muo_server::mobile::part2::effects::play_sound(w, from, 0x1CA);
            }
            return true;
        }
        false
    },
    serialize: |w, id, out| {
        (PLAGUE_BEAST_INNARD.serialize)(w, id, out);
        out.write_encoded_int(0);
        out.write_item(w, pb(w, id).organ);
    },
    deserialize: |w, id, r| {
        (PLAGUE_BEAST_INNARD.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        let organ = r.read_item(w)?.filter(|&o| is(w, o, &PLAGUE_BEAST_ORGAN));
        set_organ(w, id, organ);
        Ok(())
    },
    ..PLAGUE_BEAST_INNARD
};

// ---- PlagueBeastBlood ----

/// `PlagueBeastBlood()`.
pub fn plague_beast_blood_new(w: &mut World) -> ItemId {
    let id = new_innard(w, &PLAGUE_BEAST_BLOOD, 0x122C, 0);
    ops::set_movable(w, id, false);
    timer::start(
        w,
        TimeSpan::from_millis(1500),
        TimeSpan::from_millis(1500),
        3,
        move |w| hemorrhage(w, id),
    );
    id
}

fn blood_patched(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.item_id == 0x1765)
}

fn blood_starting(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.item_id == 0x122C)
}

fn add_xy(w: &mut World, id: ItemId, dx: i32, dy: i32) {
    if let Some(l) = w.item(id).map(|i| i.location) {
        ops::set_location(w, id, Point3D::new(l.x + dx, l.y + dy, l.z));
    }
}

fn blood_on_bandage(w: &mut World, id: ItemId, from: MobileId) -> bool {
    if !innard_is_accessible(w, id, from) || blood_patched(w, id) {
        return false;
    }
    if blood_starting(w, id) {
        add_xy(w, id, 2, -9);
        match pb(w, id).organ {
            Some(o) if is(w, o, &PLAGUE_BEAST_RUBBLE_ORGAN) => add_xy(w, id, 0, -5),
            Some(o) if is(w, o, &PLAGUE_BEAST_BACKUP_ORGAN) => add_xy(w, id, 7, 0),
            _ => {}
        }
    } else {
        add_xy(w, id, -4, -2);
    }
    super::item_id_of(w, id, 0x1765);
    if let Some(o) = owner(w, id)
        && let Some(pack) = muo_server::mobile::part1::props_b::backpack(w, o)
    {
        let items = w.item(pack).map(|p| p.items.clone()).unwrap_or_default();
        for i in items {
            if is(w, i, &PLAGUE_BEAST_MAIN_ORGAN) && pb(w, i).brains >= 4 {
                organ_finish_opening(w, i, from);
            }
        }
    }
    muo_server::item::messages::public_overhead_message_number(
        w,
        id,
        muo_server::packets::message::MessageType::REGULAR,
        0x3B2,
        1071916,
        "",
    ); // * You patch up the wound with a bandage *
    true
}

fn hemorrhage(w: &mut World, id: ItemId) {
    if deleted(w, id) || blood_patched(w, id) {
        return;
    }
    owner_play_sound(w, id, 0x25);
    if w.item(id).is_some_and(|i| i.item_id == 0x122A) {
        if let Some(o) = owner(w, id) {
            unfreeze_and_kill(w, o);
        }
    } else {
        if blood_starting(w, id) {
            add_xy(w, id, 8, -10);
        }
        let g = w.item(id).map_or(0, |i| i.item_id);
        super::item_id_of(w, id, g - 1);
    }
}

/// `Server.Items.PlagueBeastBlood`.
pub static PLAGUE_BEAST_BLOOD: ItemType = ItemType {
    full_name: "Server.Items.PlagueBeastBlood",
    parent: Some(&PLAGUE_BEAST_COMPONENT),
    construct: None,
    serialize: |w, id, out| {
        (PLAGUE_BEAST_COMPONENT.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (PLAGUE_BEAST_COMPONENT.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        Ok(())
    },
    ..PLAGUE_BEAST_COMPONENT
};

// ---- PlagueBeastVein ----

/// `PlagueBeastVein(itemID, hue)`.
pub fn plague_beast_vein_new(w: &mut World, item_id: i32, hue: i32) -> ItemId {
    new_innard(w, &PLAGUE_BEAST_VEIN, item_id, hue)
}

fn vein_scissor(w: &mut World, id: ItemId, from: MobileId, scissors: ItemId) -> bool {
    if innard_is_accessible(w, id, from) {
        let d = pb(w, id);
        if !d.cut && !d.cutting {
            if let Some(d) = pb_mut(w, id) {
                d.cutting = true;
            }
            timer::start(w, TimeSpan::from_secs(3), TimeSpan::ZERO, 1, move |w| {
                cutting_done(w, id, from)
            });
            // You begin cutting through the vein.
            scissors_say(w, scissors, 1071899);
            return true;
        }
        // This vein has already been cut.
        scissors_say(w, scissors, 1071900);
    }
    false
}

/// `scissors.PublicOverheadMessage(Regular, 0x3B2, number)`.
fn scissors_say(w: &mut World, scissors: ItemId, number: i32) {
    muo_server::item::messages::public_overhead_message_number(
        w,
        scissors,
        muo_server::packets::message::MessageType::REGULAR,
        0x3B2,
        number,
        "",
    );
}

fn cutting_done(w: &mut World, id: ItemId, from: MobileId) {
    if let Some(d) = pb_mut(w, id) {
        d.cutting = false;
        d.cut = true;
    }
    let g = w.item(id).map_or(0, |i| i.item_id);
    super::item_id_of(w, id, if g == 0x1B1C { 0x1B1B } else { 0x1B1C });
    owner_play_sound(w, id, 0x199);
    if let Some(o) = pb(w, id)
        .organ
        .filter(|&o| is(w, o, &PLAGUE_BEAST_RUBBLE_ORGAN))
    {
        rubble_on_vein_cut(w, o, from, id);
    }
}

/// `Server.Items.PlagueBeastVein`.
pub static PLAGUE_BEAST_VEIN: ItemType = ItemType {
    full_name: "Server.Items.PlagueBeastVein",
    parent: Some(&PLAGUE_BEAST_COMPONENT),
    construct: None,
    serialize: |w, id, out| {
        (PLAGUE_BEAST_COMPONENT.serialize)(w, id, out);
        out.write_encoded_int(0);
        out.write_bool(pb(w, id).cut);
    },
    deserialize: |w, id, r| {
        (PLAGUE_BEAST_COMPONENT.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        let cut = r.read_bool()?;
        if let Some(d) = pb_mut(w, id) {
            d.cut = cut;
        }
        Ok(())
    },
    ..PLAGUE_BEAST_COMPONENT
};

// ---- PlagueBeastHeart ----

/// `PlagueBeastHeart()`.
pub fn plague_beast_heart_new(w: &mut World) -> ItemId {
    let id = new_innard(w, &PLAGUE_BEAST_HEART, 0x1363, 0x21);
    start_heart(w, id);
    id
}

fn start_heart(w: &mut World, id: ItemId) {
    let h = timer::start(
        w,
        TimeSpan::from_millis(500),
        TimeSpan::from_millis(500),
        0,
        move |w| heart_tick(w, id),
    );
    if let Some(d) = pb_mut(w, id) {
        d.heart_timer = Some(h);
        d.heart_delay = false;
    }
}

fn heart_tick(w: &mut World, id: ItemId) {
    let stop = |w: &mut World| {
        if let Some(mut h) = pb(w, id).heart_timer {
            h.cancel(w);
        }
    };
    if deleted(w, id) || !owner_alive(w, id) {
        stop(w);
        return;
    }
    let g = w.item(id).map_or(0, |i| i.item_id);
    if g == 0x1363 {
        if pb(w, id).heart_delay {
            super::item_id_of(w, id, 0x1367);
            owner_play_sound(w, id, 0x11F);
        }
        if let Some(d) = pb_mut(w, id) {
            d.heart_delay = !d.heart_delay;
        }
    } else {
        super::item_id_of(w, id, 0x1363);
        owner_play_sound(w, id, 0x120);
        if let Some(d) = pb_mut(w, id) {
            d.heart_delay = false;
        }
    }
}

/// `Server.Items.PlagueBeastHeart`.
pub static PLAGUE_BEAST_HEART: ItemType = ItemType {
    full_name: "Server.Items.PlagueBeastHeart",
    parent: Some(&PLAGUE_BEAST_INNARD),
    construct: None,
    on_after_delete: |w, id| {
        if let Some(mut h) = pb(w, id).heart_timer.filter(|h| h.running(w)) {
            h.cancel(w);
        }
    },
    serialize: |w, id, out| {
        (PLAGUE_BEAST_INNARD.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (PLAGUE_BEAST_INNARD.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        start_heart(w, id); // [AfterDeserialization]
        Ok(())
    },
    ..PLAGUE_BEAST_INNARD
};

// ---- PlagueBeastOrgan ----

/// `PlagueBeastOrgan(itemID, hue)` of class `ty`: `Initialize()` runs on the next tick.
fn new_organ(w: &mut World, ty: &'static ItemType, item_id: i32, hue: i32) -> ItemId {
    let id = new_innard(w, ty, item_id, hue);
    ops::set_movable(w, id, false);
    ops::set_visible(w, id, item_id <= 1);
    timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
        organ_initialize(w, id)
    });
    id
}

fn organ_is_cuttable(w: &World, id: ItemId) -> bool {
    is(w, id, &PLAGUE_BEAST_ROCK_ORGAN) || is(w, id, &PLAGUE_BEAST_BACKUP_ORGAN)
}

fn organ_initialize(w: &mut World, id: ItemId) {
    if is(w, id, &PLAGUE_BEAST_RUBBLE_ORGAN) {
        rubble_initialize(w, id);
    } else {
        add_prop_components(w, id);
    }
}

/// `Initialize()` of an organ with a fixed layout: its `"organComponents"` prop, `[[itemID, hue, x, y], ..]` in
/// `AddComponent` order.
fn add_prop_components(w: &mut World, id: ItemId) {
    use muo_content::templates::props;
    let Some(list) = props::item(w, id, "organComponents").and_then(|v| v.as_array()) else {
        return;
    };
    let rows: Vec<[i32; 4]> = list
        .iter()
        .filter_map(|r| {
            let n = |i| props::as_i32(r.get(i));
            Some([n(0)?, n(1)?, n(2)?, n(3)?])
        })
        .collect();
    for [g, h, x, y] in rows {
        add_comp(w, id, g, h, x, y);
    }
}

/// `AddComponent(c, x, y)`.
fn add_component(w: &mut World, organ: ItemId, c: ItemId, x: i32, y: i32) {
    let parent = w.item(organ).and_then(|i| i.parent);
    if let Some(EntityId::Item(pack)) = parent
        && w.item(pack)
            .is_some_and(|p| p.ty.is(&muo_server::item::container::CONTAINER))
        && let Some(ty) = w.item(pack).map(|p| p.ty)
    {
        (ty.drop_item)(w, pack, c);
    }
    set_organ(w, c, Some(organ));
    let (loc, map) = w
        .item(organ)
        .map_or((Point3D::default(), None), |i| (i.location, i.map));
    ops::set_location(w, c, Point3D::new(loc.x + x, loc.y + y, loc.z));
    ops::set_map(w, c, map);
    add_to_components(w, organ, c);
}

fn add_comp(w: &mut World, organ: ItemId, item_id: i32, hue: i32, x: i32, y: i32) -> ItemId {
    let c = plague_beast_component_new(w, item_id, hue, false);
    add_component(w, organ, c, x, y);
    c
}

fn add_brain(w: &mut World, organ: ItemId, x: i32, y: i32) {
    let hue = pb(w, organ).brain_hue;
    let c = plague_beast_component_new(w, 0x1CF0, hue, true);
    add_component(w, organ, c, x, y);
}

fn add_blood(w: &mut World, organ: ItemId, x: i32, y: i32) {
    let c = plague_beast_blood_new(w);
    add_component(w, organ, c, x, y);
}

fn organ_scissor(w: &mut World, id: ItemId, from: MobileId, scissors: ItemId) -> bool {
    if organ_is_cuttable(w, id) && innard_is_accessible(w, id, from) {
        let d = pb(w, id);
        if !d.opened && !d.opening {
            if let Some(d) = pb_mut(w, id) {
                d.opening = true;
            }
            timer::start(w, TimeSpan::from_secs(3), TimeSpan::ZERO, 1, move |w| {
                if let Some(d) = pb_mut(w, id) {
                    d.opening = false;
                }
                if !deleted(w, id) {
                    organ_finish_opening(w, id, from);
                }
            });
            scissors_say(w, scissors, 1071897); // You carefully cut into the organ.
            return true;
        }
        scissors_say(w, scissors, 1071898); // You have already cut this organ open.
    }
    false
}

/// `OnLifted(from, c)`.
fn organ_on_lifted(w: &mut World, organ: ItemId, _from: MobileId, c: ItemId) -> bool {
    let default = comp_is_gland(w, c) || comp_is_brain(w, c);
    if is(w, organ, &PLAGUE_BEAST_ROCK_ORGAN) {
        if comp_is_brain(w, c) {
            add_blood(w, organ, -7, 24);
            return true;
        }
        return false;
    }
    if is(w, organ, &PLAGUE_BEAST_RUBBLE_ORGAN) {
        if comp_is_brain(w, c) {
            add_blood(w, organ, -13, 25);
            return true;
        }
        return false;
    }
    if is(w, organ, &PLAGUE_BEAST_BACKUP_ORGAN) {
        if comp_is_brain(w, c) {
            add_blood(w, organ, 47, 72);
            return true;
        }
        if comp_is_gland(w, c) {
            if let Some(d) = pb_mut(w, organ) {
                d.gland = None;
            }
            return true;
        }
        return false;
    }
    if is(w, organ, &PLAGUE_BEAST_MAIN_ORGAN) {
        if comp_is_brain(w, c)
            && let Some(d) = pb_mut(w, organ)
        {
            d.brains -= 1;
        }
        return true;
    }
    default
}

/// `OnDropped(from, item, to)`.
fn organ_on_dropped(
    w: &mut World,
    organ: ItemId,
    from: MobileId,
    item: ItemId,
    to: ItemId,
) -> bool {
    if is(w, organ, &PLAGUE_BEAST_BACKUP_ORGAN) {
        let hue = w.item(to).map_or(0, |i| i.hue);
        if hue == 0x1
            && pb(w, organ).gland.is_none()
            && is(w, item, &super::leaves::PLAGUE_BEAST_GLAND)
        {
            if let Some(d) = pb_mut(w, organ) {
                d.gland = Some(item);
            }
            timer::start(w, TimeSpan::from_secs(3), TimeSpan::ZERO, 1, move |w| {
                backup_finish_healing(w, organ)
            });
            muo_server::mobile::part2::messages::send_ascii_message_hue(
                w,
                from,
                0x3B2,
                "* You place the healthy gland inside the organ sac *",
            );
            ops::set_movable(w, item, false);
            owner_play_sound(w, organ, 0x20);
            return true;
        }
        return false;
    }
    if is(w, organ, &PLAGUE_BEAST_MAIN_ORGAN) {
        let item_hue = w.item(item).map_or(0, |i| i.hue);
        let to_hue = w.item(to).map_or(0, |i| i.hue);
        if !pb(w, organ).opened && comp_is_receptacle(w, to) && item_hue == to_hue {
            set_organ(w, to, Some(organ));
            if let Some(d) = pb_mut(w, organ) {
                d.brains += 1;
            }
            // You place the organ in the fleshy receptacle near the core.
            muo_server::mobile::part2::messages::local_overhead_message_number(
                w,
                from,
                MessageType::REGULAR,
                0x34,
                1071913,
                "",
            );
            if let Some(o) = owner(w, organ) {
                muo_server::mobile::part2::effects::play_sound(w, o, 0x1BA);
                if is_bleeding(w, o) {
                    // The plague beast is still bleeding from open wounds.  You must seal any bleeding wounds before the core will open!
                    muo_server::mobile::part2::messages::local_overhead_message_number(
                        w,
                        from,
                        MessageType::REGULAR,
                        0x34,
                        1071922,
                        "",
                    );
                    return true;
                }
            }
            if pb(w, organ).brains == 4 {
                organ_finish_opening(w, organ, from);
            }
            return true;
        }
        return false;
    }
    false
}

/// `FinishOpening(from)`.
fn organ_finish_opening(w: &mut World, id: ItemId, from: MobileId) {
    if is(w, id, &PLAGUE_BEAST_MAIN_ORGAN) {
        main_finish_opening(w, id);
    } else if is(w, id, &PLAGUE_BEAST_RUBBLE_ORGAN) {
        rubble_finish_opening(w, id);
    } else if is(w, id, &PLAGUE_BEAST_BACKUP_ORGAN) {
        base_finish_opening(w, id);
        add_comp(w, id, 0x1363, 0xF, -3, 3);
        add_comp(w, id, 0x1365, 0x1, -3, 10);
        let g = plague_beast_component_new(w, 0x1CEF, 0x3F, true);
        if let Some(d) = pb_mut(w, id) {
            d.gland = Some(g);
        }
        add_component(w, id, g, -4, 16);
    } else if is(w, id, &PLAGUE_BEAST_ROCK_ORGAN) {
        base_finish_opening(w, id);
        add_comp(w, id, 0x1775, 0x60, 3, 5);
        add_comp(w, id, 0x1777, 0x1, 10, 14);
        if pb(w, id).brain_hue > 0 {
            add_brain(w, id, 1, 24); // 22, 29
        } else {
            add_blood(w, id, -7, 24);
        }
    } else if is(w, id, &PLAGUE_BEAST_MAIDEN_ORGAN) {
        super::item_id_of(w, id, 0x1249);
        owner_play_sound(w, id, 0x187);
        add_comp(w, id, 0x1D0D, 0x0, 22, 3);
        add_comp(w, id, 0x1D12, 0x0, 15, 18);
        add_comp(w, id, 0x1DA3, 0x21, 26, 46);
        if pb(w, id).brain_hue > 0 {
            add_brain(w, id, 22, 29);
        }
        if let Some(d) = pb_mut(w, id) {
            d.opened = true;
        }
    } else {
        base_finish_opening(w, id);
    }
    let _ = from;
}

/// `PlagueBeastOrgan.FinishOpening`.
fn base_finish_opening(w: &mut World, id: ItemId) {
    if let Some(d) = pb_mut(w, id) {
        d.opened = true;
    }
    owner_play_sound(w, id, 0x50);
}

fn organ_on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    if is(w, id, &PLAGUE_BEAST_MAIN_ORGAN) || !is(w, id, &PLAGUE_BEAST_MAIDEN_ORGAN) {
        return;
    }
    if !pb(w, id).opened {
        organ_finish_opening(w, id, from);
    }
}

// Rubble

const RUBBLE_HUES: [i32; 6] = [0xD, 0x17, 0x2B, 0x42, 0x54, 0x5D];

fn random_hue(w: &mut World, exclude: i32) -> i32 {
    for _ in 0..20 {
        let hue = RUBBLE_HUES[utility::random(&mut w.rng, RUBBLE_HUES.len() as i32) as usize];
        if hue != exclude {
            return hue;
        }
    }
    RUBBLE_HUES[0]
}

fn rubble_initialize(w: &mut World, id: ItemId) {
    let hue = RUBBLE_HUES[utility::random(&mut w.rng, RUBBLE_HUES.len() as i32) as usize];
    super::hue(w, id, hue);
    add_comp(w, id, 0x3BB, hue, 0, 0);
    add_comp(w, id, 0x3BA, hue, 4, 6);
    add_comp(w, id, 0x3BA, hue, -6, 17);
    let v = utility::random(&mut w.rng, 4);
    for (n, (g, x, y)) in [
        (0x1B1B, -23, -3),
        (0x1B1C, 19, 4),
        (0x1B1B, 21, 27),
        (0x1B1B, 10, 40),
    ]
    .into_iter()
    .enumerate()
    {
        let h = if v == n as i32 {
            hue
        } else {
            random_hue(w, hue)
        };
        let c = plague_beast_vein_new(w, g, h);
        add_component(w, id, c, x, y);
    }
}

fn rubble_finish_opening(w: &mut World, id: ItemId) {
    add_comp(w, id, 0x1777, 0x1, 5, 14);
    if pb(w, id).brain_hue > 0 {
        add_brain(w, id, -5, 22);
    } else {
        add_blood(w, id, -13, 25);
    }
    if let Some(d) = pb_mut(w, id) {
        d.opened = true;
    }
}

fn rubble_on_vein_cut(w: &mut World, id: ItemId, from: MobileId, vein: ItemId) {
    let (vein_hue, hue) = (
        w.item(vein).map_or(0, |i| i.hue),
        w.item(id).map_or(0, |i| i.hue),
    );
    if vein_hue != hue {
        let d = pb(w, id);
        if !d.opened && d.veins > 0 {
            let left = d.veins - 1;
            if let Some(d) = pb_mut(w, id) {
                d.veins = left;
            }
            if left == 0 {
                organ_finish_opening(w, id, from);
            }
        }
    } else {
        // * As you cut the vein, a cloud of poison is expelled from the plague beast's organ, and the plague beast dissolves into a puddle of goo *
        muo_server::mobile::part2::messages::local_overhead_message_number(
            w,
            from,
            MessageType::REGULAR,
            0x3B2,
            1071901,
            "",
        );
        let greater = w
            .resources
            .get::<muo_server::poison::Poisons>()
            .and_then(|p| p.get_poison("Greater"))
            .map(|p| p.id());
        if let Some(p) = greater {
            muo_server::mobile::base::apply_poison(w, from, Some(from), p);
        }
        muo_server::mobile::part2::effects::play_sound(w, from, 0x22F);
        if let Some(o) = owner(w, id) {
            unfreeze_and_kill(w, o);
        }
    }
}

// Backup

fn backup_finish_healing(w: &mut World, id: ItemId) {
    let comps = pb(w, id).components;
    for c in comps.iter().take(7).flatten() {
        super::hue(w, *c, 0x6);
    }
    timer::start(w, TimeSpan::from_secs(2), TimeSpan::ZERO, 1, move |w| {
        backup_open_organ(w, id)
    });
}

fn backup_open_organ(w: &mut World, id: ItemId) {
    add_comp(w, id, 0x1367, 0xF, 55, 61);
    add_comp(w, id, 0x1366, 0x1, 57, 66);
    if pb(w, id).brain_hue > 0 {
        add_brain(w, id, 55, 69);
    }
}

// Main

fn main_finish_opening(w: &mut World, id: ItemId) {
    add_comp(w, id, 0x1363, 0x1, 0, 22);
    add_comp(w, id, 0x1D04, 0xD, 0, 22);
    if let Some(o) = owner(w, id)
        && let Some(pack) = muo_server::mobile::part1::props_b::backpack(w, o)
    {
        let core = plague_beast_mutation_core_new(w);
        if let Some(ty) = w.item(pack).map(|p| p.ty) {
            (ty.add_item)(w, pack, core);
        }
        ops::set_movable(w, core, false);
        if let Some(d) = pb_mut(w, core) {
            d.cut = false;
        }
        let loc = w.item(id).map_or_else(Point3D::default, |i| i.location);
        if let Some(cl) = w.item(core).map(|i| i.location) {
            ops::set_location(w, core, Point3D::new(loc.x, loc.y + 34, cl.z));
        }
        muo_server::mobile::part2::effects::play_sound(w, o, 0x21);
        muo_server::mobile::part2::effects::play_sound(w, o, 0x166);
    }
    if let Some(d) = pb_mut(w, id) {
        d.opened = true;
    }
}

macro_rules! organ_class {
    ($(#[$m:meta])* $name:ident = $full:literal, $parent:ident, |$w:ident, $id:ident, $out:ident| $ser:block, |$w2:ident, $id2:ident, $r:ident| $de:block) => {
        $(#[$m])*
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&$parent),
            construct: None,
            serialize: |$w, $id, $out| {
                ($parent.serialize)($w, $id, $out);
                $out.write_encoded_int(0);
                $ser
            },
            deserialize: |$w2, $id2, $r| {
                ($parent.deserialize)($w2, $id2, $r)?;
                let _version = $r.read_encoded_int()?;
                $de
                Ok(())
            },
            ..$parent
        };
    };
}

/// `Server.Items.PlagueBeastOrgan`.
pub static PLAGUE_BEAST_ORGAN: ItemType = ItemType {
    full_name: "Server.Items.PlagueBeastOrgan",
    parent: Some(&PLAGUE_BEAST_INNARD),
    construct: None,
    on_double_click: organ_on_double_click,
    serialize: |w, id, out| {
        (PLAGUE_BEAST_INNARD.serialize)(w, id, out);
        out.write_encoded_int(0);
        let d = pb(w, id);
        out.write_encoded_int(d.components.len() as i32);
        for c in &d.components {
            out.write_item(w, *c);
        }
        out.write_i32(d.brain_hue);
        out.write_bool(d.opened);
    },
    deserialize: |w, id, r| {
        (PLAGUE_BEAST_INNARD.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        let count = r.read_encoded_int()?;
        let mut comps = Vec::new();
        for _ in 0..count {
            comps.push(
                r.read_item(w)?
                    .filter(|&c| is(w, c, &PLAGUE_BEAST_COMPONENT)),
            );
        }
        let brain_hue = r.read_i32()?;
        let opened = r.read_bool()?;
        if let Some(d) = pb_mut(w, id) {
            d.components = comps;
            d.brain_hue = brain_hue;
            d.opened = opened;
        }
        Ok(())
    },
    ..PLAGUE_BEAST_INNARD
};

organ_class!(
    /// `Server.Items.PlagueBeastMaidenOrgan`.
    PLAGUE_BEAST_MAIDEN_ORGAN = "Server.Items.PlagueBeastMaidenOrgan", PLAGUE_BEAST_ORGAN, |w, id, out| {}, |w, id, r| {}
);
organ_class!(
    /// `Server.Items.PlagueBeastRockOrgan`.
    PLAGUE_BEAST_ROCK_ORGAN = "Server.Items.PlagueBeastRockOrgan", PLAGUE_BEAST_ORGAN, |w, id, out| {}, |w, id, r| {}
);
organ_class!(
    /// `Server.Items.PlagueBeastRubbleOrgan`.
    PLAGUE_BEAST_RUBBLE_ORGAN = "Server.Items.PlagueBeastRubbleOrgan", PLAGUE_BEAST_ORGAN,
    |w, id, out| { out.write_i32(pb(w, id).veins); },
    |w, id, r| {
        let v = r.read_i32()?;
        if let Some(d) = pb_mut(w, id) {
            d.veins = v;
        }
    }
);
organ_class!(
    /// `Server.Items.PlagueBeastBackupOrgan`.
    PLAGUE_BEAST_BACKUP_ORGAN = "Server.Items.PlagueBeastBackupOrgan", PLAGUE_BEAST_ORGAN,
    |w, id, out| { out.write_item(w, pb(w, id).gland); },
    |w, id, r| {
        let g = r.read_item(w)?;
        if let Some(d) = pb_mut(w, id) {
            d.gland = g;
        }
    }
);
organ_class!(
    /// `Server.Items.PlagueBeastMainOrgan`.
    PLAGUE_BEAST_MAIN_ORGAN = "Server.Items.PlagueBeastMainOrgan", PLAGUE_BEAST_ORGAN,
    |w, id, out| { out.write_i32(pb(w, id).brains); },
    |w, id, r| {
        let b = r.read_i32()?;
        if let Some(d) = pb_mut(w, id) {
            d.brains = b;
        }
    }
);

/// `PlagueBeastMaidenOrgan()`.
pub fn plague_beast_maiden_organ_new(w: &mut World) -> ItemId {
    new_organ(w, &PLAGUE_BEAST_MAIDEN_ORGAN, 0x124D, 0)
}

/// `PlagueBeastRockOrgan()`.
pub fn plague_beast_rock_organ_new(w: &mut World) -> ItemId {
    new_organ(w, &PLAGUE_BEAST_ROCK_ORGAN, 0x177A, 0x60)
}

/// `PlagueBeastRubbleOrgan()`.
pub fn plague_beast_rubble_organ_new(w: &mut World) -> ItemId {
    let id = new_organ(w, &PLAGUE_BEAST_RUBBLE_ORGAN, 1, 0);
    if let Some(d) = pb_mut(w, id) {
        d.veins = 3;
    }
    id
}

/// `PlagueBeastBackupOrgan()`.
pub fn plague_beast_backup_organ_new(w: &mut World) -> ItemId {
    new_organ(w, &PLAGUE_BEAST_BACKUP_ORGAN, 0x1362, 0x6)
}

/// `PlagueBeastMainOrgan()`.
pub fn plague_beast_main_organ_new(w: &mut World) -> ItemId {
    new_organ(w, &PLAGUE_BEAST_MAIN_ORGAN, 1, 0)
}

// ---- PlagueBeastBackpack ----

const POSITIONS: [[(i32, i32); 6]; 3] = [
    [
        (275, 85),
        (360, 111),
        (375, 184),
        (332, 228),
        (141, 105),
        (189, 75),
    ],
    [
        (274, 34),
        (327, 89),
        (354, 168),
        (304, 225),
        (113, 86),
        (189, 75),
    ],
    [
        (276, 79),
        (369, 117),
        (372, 192),
        (336, 230),
        (141, 116),
        (189, 75),
    ],
];

const BRAIN_HUES: [i32; 4] = [0x2B, 0x42, 0x54, 0x60];

/// `PlagueBeastBackpack()`.
pub fn backpack_new(w: &mut World) -> ItemId {
    let id = muo_content::items::cont::new_base_container(w, &PLAGUE_BEAST_BACKPACK, 0x261B);
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.set_layer)(w, id, Layer::Backpack);
    }
    id
}

/// `AddInnard(innard, pos)`.
fn add_innard(w: &mut World, pack: ItemId, innard: ItemId, x: i32, y: i32) {
    if let Some(ty) = w.item(pack).map(|p| p.ty) {
        (ty.add_item)(w, pack, innard);
    }
    ops::set_location(w, innard, Point3D::new(x, y, 0));
    let map = w.item(pack).and_then(|p| p.map);
    ops::set_map(w, innard, map);
}

fn add_plain_innard(w: &mut World, pack: ItemId, item_id: i32, hue: i32, x: i32, y: i32) {
    let i = new_innard(w, &PLAGUE_BEAST_INNARD, item_id, hue);
    add_innard(w, pack, i, x, y);
}

/// `PlagueBeastBackpack.Initialize()`.
pub fn backpack_initialize(w: &mut World, pack: ItemId) {
    const PLAIN: [(i32, i32, i32, i32); 36] = [
        (0x1CF6, 0x0, 227, 128),
        (0x1D10, 0x0, 251, 128),
        (0x1FBE, 0x21, 240, 83),
        (0, 0, 0, 0), // the heart
        (0x1D06, 0x0, 283, 91),
        (0x1FAF, 0x21, 315, 107),
        (0x1FB9, 0x21, 289, 87),
        (0x9E7, 0x21, 304, 96),
        (0x1B1A, 0x66D, 335, 102),
        (0x1D10, 0x0, 338, 146),
        (0x1FB3, 0x21, 358, 167),
        (0x1D0B, 0x0, 357, 155),
        (0x9E7, 0x21, 339, 184),
        (0x1B1A, 0x66D, 157, 172),
        (0x1D11, 0x0, 147, 157),
        (0x1FB9, 0x21, 121, 131),
        (0x9E7, 0x21, 166, 176),
        (0x1D0B, 0x0, 122, 138),
        (0x1D0D, 0x0, 118, 150),
        (0x1FB3, 0x21, 97, 123),
        (0x1D08, 0x0, 115, 113),
        (0x9E7, 0x21, 109, 109),
        (0x9E7, 0x21, 91, 122),
        (0x9E7, 0x21, 94, 160),
        (0x1B19, 0x66D, 170, 121),
        (0x1FAF, 0x21, 161, 111),
        (0x1D0B, 0x0, 158, 112),
        (0x9E7, 0x21, 159, 101),
        (0x1D10, 0x0, 132, 177),
        (0x1D0E, 0x0, 110, 178),
        (0x1FB3, 0x21, 95, 194),
        (0x1FAF, 0x21, 154, 203),
        (0x1B1A, 0x66D, 110, 237),
        (0x9E7, 0x21, 111, 171),
        (0x9E7, 0x21, 90, 197),
        (0x9E7, 0x21, 166, 205),
    ];
    const TAIL: [(i32, i32, i32, i32); 3] = [
        (0x9E7, 0x21, 96, 242),
        (0x1D10, 0x0, 334, 196),
        (0x1D0B, 0x0, 322, 270),
    ];
    for (item_id, hue, x, y) in PLAIN {
        if item_id == 0 {
            let heart = plague_beast_heart_new(w);
            add_innard(w, pack, heart, 229, 104);
        } else {
            add_plain_innard(w, pack, item_id, hue, x, y);
        }
    }
    for (item_id, hue, x, y) in TAIL {
        add_plain_innard(w, pack, item_id, hue, x, y);
    }

    let mut organs: Vec<ItemId> = Vec::new();
    #[allow(clippy::needless_range_loop)] // `i` is also the slot: the last organ is always the rock
    for i in 0..6 {
        let random = if i == 5 {
            0
        } else {
            utility::random(&mut w.rng, 3)
        };
        let organ = match random {
            1 => plague_beast_maiden_organ_new(w),
            2 => plague_beast_rubble_organ_new(w),
            _ => plague_beast_rock_organ_new(w),
        };
        organs.push(organ);
        let (x, y) = POSITIONS[random as usize][i];
        add_innard(w, pack, organ, x, y);
    }
    let organ = plague_beast_backup_organ_new(w);
    organs.push(organ);
    add_innard(w, pack, organ, 129, 214);

    for hue in BRAIN_HUES {
        let n = utility::random(&mut w.rng, organs.len() as i32) as usize;
        let organ = organs.remove(n);
        if let Some(d) = pb_mut(w, organ) {
            d.brain_hue = hue;
        }
    }

    let main = plague_beast_main_organ_new(w);
    add_innard(w, pack, main, 240, 161);
}

fn is_innard_or_gland(w: &World, id: ItemId) -> bool {
    is(w, id, &PLAGUE_BEAST_INNARD) || is(w, id, &super::leaves::PLAGUE_BEAST_GLAND)
}

/// `Server.Items.PlagueBeastBackpack`.
pub static PLAGUE_BEAST_BACKPACK: ItemType = ItemType {
    full_name: "Server.Items.PlagueBeastBackpack",
    parent: Some(&muo_content::items::cont::BASE_CONTAINER),
    construct: None,
    default_max_weight: crate::class_props::default_max_weight,
    default_max_items: crate::class_props::default_max_items,
    default_gump_id: crate::class_props::default_gump_id,
    default_drop_sound: crate::class_props::default_drop_sound,
    try_drop_item: |w, id, from, dropped, send_full, _| {
        is_innard_or_gland(w, dropped)
            && (muo_content::items::cont::BASE_CONTAINER.try_drop_item)(
                w, id, from, dropped, send_full, false,
            )
    },
    on_drag_drop_into: |w, id, from, item, p: Point3D| {
        if !w
            .item(id)
            .map(|i| i.ty)
            .is_some_and(|t| (t.is_accessible_to)(w, id, from))
            || !is_innard_or_gland(w, item)
        {
            return false;
        }
        let ir = ops::get_graphic_bounds(w, item);
        let cx = p.x + ir.x() + ir.width() / 2;
        let cy = p.y + ir.y() + ir.height() / 2;
        let items = w.item(id).map(|c| c.items.clone()).unwrap_or_default();
        for &innard in items.iter().rev() {
            if is(w, innard, &PLAGUE_BEAST_COMPONENT) {
                let r = ops::get_graphic_bounds(w, innard);
                let loc = w.item(innard).map_or_else(Point3D::default, |i| i.location);
                let (x, y) = (loc.x + r.x(), loc.y + r.y());
                if cx >= x && cx <= x + r.width() && cy >= y && cy <= y + r.height() {
                    if let Some(ty) = w.item(innard).map(|i| i.ty) {
                        (ty.on_drag_drop)(w, innard, from, item);
                    }
                    break;
                }
            }
        }
        (muo_content::items::cont::BASE_CONTAINER.on_drag_drop_into)(w, id, from, item, p)
    },
    serialize: |w, id, out| {
        (muo_content::items::cont::BASE_CONTAINER.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (muo_content::items::cont::BASE_CONTAINER.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        Ok(())
    },
    ..muo_content::items::cont::BASE_CONTAINER
};

// ---- PlagueBeastMutationCore ----

/// `PlagueBeastMutationCore()`.
pub fn plague_beast_mutation_core_new(w: &mut World) -> ItemId {
    let id = crate::simple::from_template(
        w,
        &PLAGUE_BEAST_MUTATION_CORE,
        "Server.Items.PlagueBeastMutationCore",
    );
    if let Some(d) = pb_mut(w, id) {
        d.cut = true;
    }
    super::hue(w, id, 0x480);
    id
}

fn core_scissor(w: &mut World, id: ItemId, from: MobileId) -> bool {
    if pb(w, id).cut {
        return false;
    }
    let lord = owner(w, id);
    if let Some(d) = pb_mut(w, id) {
        d.cut = true;
    }
    ops::set_movable(w, id, true);
    deps::add_to_backpack(w, from, id);
    // * You remove the plague mutation core from the plague beast, causing it to dissolve into a pile of goo *
    muo_server::mobile::part2::messages::local_overhead_message_number(
        w,
        from,
        MessageType::REGULAR,
        0x34,
        1071906,
        "",
    );
    if let Some(lord) = lord {
        timer::start(w, TimeSpan::from_secs(1), TimeSpan::ZERO, 1, move |w| {
            unfreeze_and_kill(w, lord)
        });
    }
    true
}

/// `Server.Items.PlagueBeastMutationCore` (`IScissorable`).
pub static PLAGUE_BEAST_MUTATION_CORE: ItemType = ItemType {
    full_name: "Server.Items.PlagueBeastMutationCore",
    parent: Some(&ITEM_BASE),
    construct: Some(plague_beast_mutation_core_new),
    default_weight: crate::class_props::default_weight,
    label_number: crate::class_props::label_number, // a plague beast mutation core
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_encoded_int(0);
        out.write_bool(pb(w, id).cut);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        let cut = r.read_bool()?;
        if let Some(d) = pb_mut(w, id) {
            d.cut = cut;
        }
        Ok(())
    },
    ..ITEM_BASE
};

pub fn register(types: &mut TypeRegistry) {
    for t in [
        &PLAGUE_BEAST_INNARD,
        &PLAGUE_BEAST_COMPONENT,
        &PLAGUE_BEAST_BLOOD,
        &PLAGUE_BEAST_VEIN,
        &PLAGUE_BEAST_HEART,
        &PLAGUE_BEAST_ORGAN,
        &PLAGUE_BEAST_MAIDEN_ORGAN,
        &PLAGUE_BEAST_ROCK_ORGAN,
        &PLAGUE_BEAST_RUBBLE_ORGAN,
        &PLAGUE_BEAST_BACKUP_ORGAN,
        &PLAGUE_BEAST_MAIN_ORGAN,
        &PLAGUE_BEAST_BACKPACK,
        &PLAGUE_BEAST_MUTATION_CORE,
    ] {
        types.register_item(t, &[]);
    }
}

pub(super) fn configure(w: &mut World) {
    register_scissorable(w, &PLAGUE_BEAST_INNARD, scissor);
    register_scissorable(w, &PLAGUE_BEAST_MUTATION_CORE, scissor);
    register_carvable(w, &PLAGUE_BEAST_INNARD, carve);
}
