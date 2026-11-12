//! The Character Statue Maker: `CharacterStatue`, `CharacterStatueMaker`, `CharacterStatuePlinth`, their packets and
//! `CharacterStatueGump`.
//!
//! shortcut: a `Restore` from a backup statue that was deleted with its deed copies what the deleted mobile still
//! holds (its body, pose and direction); nothing is copied for the clothes either (a deleted mobile's items are
//! gone). `OnRequestedAnimation` is not called from a packet handler (it belongs to the client's animation
//! request, which this server does not decode).

use std::rc::Rc;

use muo_content::gumps::base::{DynamicGumpLogic, DynamicGumpObj, RelayInfo};
use muo_content::gumps::builder::DynamicGumpBuilder;
use muo_content::gumps::send_gump_to_mobile;
use muo_content::items::addon::{self, AddonFitResult};
use muo_content::items::house_deps::{self, SimpleAddon};
use muo_items::misc::basic::STATIC;
use muo_primitives::{DateTime, Point3D, TimeSpan};
use muo_server::World;
use muo_server::context_menu_entry::ContextMenuEntry;
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, LootType, ops};
use muo_server::layer::Layer;
use muo_server::map::MapId;
use muo_server::mobile::{AccessLevel, MOBILE_BASE, MobileType};
use muo_server::net_state::NetStateId;
use muo_server::property_list::PropertyList;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::target::{self, Target, TargetFlags, Targeted};
use muo_server::timer;
use muo_server::type_registry::{DEFAULT_CALL_PRIORITY, Phase, TypeRegistry};

/// `StatueType`.
pub mod statue_type {
    pub const MARBLE: i32 = 0;
    pub const JADE: i32 = 1;
    pub const BRONZE: i32 = 2;
}

/// What `CharacterStatue` adds to a `Mobile`.
#[derive(Clone, Debug)]
pub struct StatueData {
    pub statue_type: i32,
    pub pose: i32,
    pub material: i32,
    pub sculpted_by: Option<String>,
    pub sculpted_on: DateTime,
    pub plinth: Option<ItemId>,
    pub is_reward_item: bool,
    /// `m_Animation`, `m_Frames`.
    pub animation: i32,
    pub frames: i32,
}

pub fn statue(w: &World, id: MobileId) -> Option<&StatueData> {
    w.mobile(id)?.ext.get::<StatueData>()
}

pub(crate) fn statue_mut(w: &mut World, id: MobileId) -> Option<&mut StatueData> {
    w.mobile_mut(id)?.ext.get_mut::<StatueData>()
}

fn is_statue(w: &World, id: MobileId) -> bool {
    w.mobile(id).is_some_and(|m| m.ty.is(&CHARACTER_STATUE))
}

fn mobile_alive_ref(w: &World, id: Option<MobileId>) -> Option<MobileId> {
    id.filter(|m| is_statue(w, *m) && !w.mobile(*m).is_some_and(|x| x.deleted))
}

fn delete_item(w: &mut World, id: ItemId) {
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.delete)(w, id);
    }
}

fn delete_mobile(w: &mut World, id: MobileId) {
    if let Some(ty) = w.mobile(id).map(|m| m.ty) {
        (ty.delete)(w, id);
    }
}

// ---- CharacterStatuePackets ----

pub const STATUE_ANIMATION_PACKET_LENGTH: usize = 17;

/// `CreateStatueAnimation(buffer, serial, status, anim, frame)`.
pub fn statue_animation_packet(
    serial: muo_primitives::Serial,
    status: i32,
    anim: i32,
    frame: i32,
) -> [u8; 17] {
    let mut p = [0u8; STATUE_ANIMATION_PACKET_LENGTH];
    p[0] = 0xBF;
    p[1..3].copy_from_slice(&17u16.to_be_bytes());
    p[3..5].copy_from_slice(&0x19i16.to_be_bytes());
    p[5] = 0x5;
    p[6..10].copy_from_slice(&serial.value().to_be_bytes());
    p[10] = 0;
    p[11] = 0xFF;
    p[12] = status as u8;
    p[13] = 0;
    p[14] = anim as u8;
    p[15] = 0;
    p[16] = frame as u8;
    p
}

// ---- CharacterStatue ----

/// `CharacterStatue(from, statueType)`.
pub fn statue_new(w: &mut World, from: MobileId, statue_type: i32) -> MobileId {
    let id = w.new_mobile(&CHARACTER_STATUE);
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(StatueData {
            statue_type,
            pose: 0,
            material: 0,
            sculpted_by: None,
            sculpted_on: DateTime::MIN,
            plinth: None,
            is_reward_item: false,
            animation: 0,
            frames: 0,
        });
    }
    muo_content::mx::set_direction(w, id, muo_primitives::Direction::SOUTH);
    muo_server::mobile::part1::props_a::set_access_level(w, id, AccessLevel::Counselor);
    let hits_max = muo_server::mobile::base::hits_max(w, id);
    muo_server::mobile::part1::props_b::set_hits(w, id, hits_max);
    muo_server::mobile::part1::props_a::set_blessed(w, id, true);
    muo_content::mx::set_frozen(w, id, true);
    clone_body(w, id, from);
    clone_clothes(w, id, from);
    invalidate_hues(w, id);
    id
}

/// `CloneBody(from)`.
pub fn clone_body(w: &mut World, id: MobileId, from: MobileId) {
    use muo_server::mobile::part1::props_b as b;
    let Some(src) = w.mobile(from) else { return };
    let (name, body, female, hair, facial) = (
        (src.ty.name)(w, from),
        b::body(w, from),
        src.female,
        src.hair_item_id,
        src.facial_hair_item_id,
    );
    muo_server::mobile::base::set_name(w, id, name);
    b::set_body(w, id, body);
    b::set_female(w, id, female);
    b::set_hair_item_id(w, id, hair);
    b::set_facial_hair_item_id(w, id, facial);
}

/// `CloneItem(item)`.
fn clone_item(w: &mut World, item: ItemId) -> ItemId {
    let (item_id, layer, hue) = w
        .item(item)
        .map_or((0, Layer::Invalid, 0), |i| (i.item_id, i.layer, i.hue));
    let name = ops::name(w, item);
    let weight = ops::weight(w, item);
    let cloned = ops::new_item_with_id(w, &ITEM_BASE, item_id);
    if let Some(ty) = w.item(cloned).map(|i| i.ty) {
        (ty.set_layer)(w, cloned, layer);
        (ty.set_hue)(w, cloned, hue);
    }
    ops::set_name(w, cloned, name);
    ops::set_weight(w, cloned, weight);
    ops::set_movable(w, cloned, false);
    cloned
}

/// `CloneClothes(from)`.
pub fn clone_clothes(w: &mut World, id: MobileId, from: MobileId) {
    let mine: Vec<ItemId> = w.mobile(id).map(|m| m.items.clone()).unwrap_or_default();
    for item in mine.into_iter().rev() {
        delete_item(w, item);
    }
    let theirs: Vec<ItemId> = w.mobile(from).map(|m| m.items.clone()).unwrap_or_default();
    for item in theirs.into_iter().rev() {
        let layer = w.item(item).map(|i| i.layer);
        if !matches!(layer, Some(Layer::Backpack | Layer::Mount | Layer::Bank)) {
            let cloned = clone_item(w, item);
            muo_server::mobile::part2::items::add_item(w, id, cloned);
        }
    }
}

/// `InvalidateHues()`.
pub fn invalidate_hues(w: &mut World, id: MobileId) {
    let Some(d) = statue(w, id).cloned() else {
        return;
    };
    let hue = 0xB8F + d.statue_type * 4 + d.material;
    muo_server::mobile::base::set_hue(w, id, hue);
    muo_server::mobile::part1::props_b::set_hair_hue(w, id, hue);
    if w.mobile(id).is_some_and(|m| m.facial_hair_item_id > 0) {
        muo_server::mobile::part1::props_b::set_facial_hair_hue(w, id, hue);
    }
    let items: Vec<ItemId> = w.mobile(id).map(|m| m.items.clone()).unwrap_or_default();
    for item in items.into_iter().rev() {
        crate::util::hue(w, item, hue);
    }
    if let Some(p) = d.plinth {
        plinth_invalidate_hue(w, p);
    }
}

/// `InvalidatePose()`.
pub fn invalidate_pose(w: &mut World, id: MobileId) {
    let Some(pose) = statue(w, id).map(|d| d.pose) else {
        return;
    };
    let (animation, frames) = match pose {
        0 => (4, 0),  // Ready
        1 => (16, 2), // Casting
        2 => (33, 1), // Salute
        3 => (17, 4), // AllPraiseMe
        4 => (31, 5), // Fighting
        5 => (6, 1),  // HandsOnHips
        _ => {
            let d = statue(w, id).expect("checked");
            (d.animation, d.frames)
        }
    };
    if let Some(d) = statue_mut(w, id) {
        d.animation = animation;
        d.frames = frames;
    }
    let Some(m) = w.mobile(id) else { return };
    let (map, loc, serial) = (m.map, m.location, m.serial);
    if map.is_none() {
        return;
    }
    muo_server::mobile::base::process_delta(w, id);
    let clients: Vec<NetStateId> = muo_server::map::enumerate::clients_in_range(
        w,
        map,
        loc.x,
        loc.y,
        w.maps.global_max_update_range,
    )
    .collect();
    for ns in clients {
        if let Some(pm) = w.net_states.get(&ns).and_then(|n| n.mobile) {
            muo_server::mobile::base::process_delta(w, pm);
        }
        if w.cannot_send_packets(ns) {
            continue;
        }
        w.send(ns, &statue_animation_packet(serial, 1, animation, frames));
    }
}

/// `OnRequestedAnimation(from)`.
pub fn on_requested_animation(w: &mut World, id: MobileId, from: MobileId) {
    let Some(d) = statue(w, id) else { return };
    let (animation, frames) = (d.animation, d.frames);
    let serial = w.mobile(id).map(|m| m.serial);
    if let (Some(ns), Some(serial)) = (
        muo_server::mobile::part1::props_b::net_state(w, from),
        serial,
    ) {
        w.send(ns, &statue_animation_packet(serial, 1, animation, frames));
    }
}

pub(crate) fn set_pose(w: &mut World, id: MobileId, value: i32) {
    if let Some(d) = statue_mut(w, id)
        && d.pose != value
    {
        d.pose = value;
        invalidate_pose(w, id);
    }
}

pub(crate) fn set_material(w: &mut World, id: MobileId, value: i32) {
    if let Some(d) = statue_mut(w, id)
        && d.material != value
    {
        d.material = value;
        invalidate_hues(w, id);
        invalidate_pose(w, id);
    }
}

pub(crate) fn set_statue_type(w: &mut World, id: MobileId, value: i32) {
    if let Some(d) = statue_mut(w, id)
        && d.statue_type != value
    {
        d.statue_type = value;
        invalidate_hues(w, id);
        invalidate_pose(w, id);
    }
}

/// `Sculpt(by)`.
pub fn sculpt(w: &mut World, id: MobileId, by: MobileId) {
    if w.mobile(by).is_none() {
        return;
    }
    let show_fame = w
        .mobile(by)
        .map(|m| m.ty)
        .is_some_and(|t| (t.show_fame_title)(w, by));
    let human = muo_content::creature::misc::body_is_human(w, by);
    let m = w.mobile(by).expect("checked");
    let ty = m.ty;
    let raw = m.name.clone();
    let famous = show_fame && (m.player || human) && m.fame >= 10000;
    let female = m.female;
    let sculpted_by = if famous {
        format!(
            "{} {}",
            if female { "Lady" } else { "Lord" },
            (ty.name)(w, by).unwrap_or_default()
        )
    } else {
        raw.unwrap_or_default()
    };
    let now = w.now;
    if let Some(d) = statue_mut(w, id) {
        d.sculpted_by = Some(sculpted_by);
        d.sculpted_on = now;
    }
    muo_server::mobile::part2::properties::invalidate_properties(w, id);
}

/// `Demolish(by)`.
pub fn demolish(w: &mut World, id: MobileId, by: MobileId) -> bool {
    let deed = deed_new(w, None);
    if muo_server::mobile::part2::items::place_in_backpack(w, by, deed) {
        let (statue_type, reward, plinth) = statue(w, id).map_or((0, false, None), |d| {
            (d.statue_type, d.is_reward_item, d.plinth)
        });
        delete_mobile(w, id);
        set_deed_statue(w, deed, Some(id));
        set_deed_type(w, deed, statue_type);
        set_deed_reward(w, deed, reward);
        if let Some(p) = plinth {
            delete_item(w, p);
        }
        return true;
    }
    muo_content::mx::msg(w, by, 500720); // You don't have enough room in your backpack!
    delete_item(w, deed);
    false
}

/// `Restore(from)`.
pub fn restore(w: &mut World, id: MobileId, from: MobileId) {
    let Some(src) = statue(w, from).cloned() else {
        return;
    };
    if let Some(d) = statue_mut(w, id) {
        d.material = src.material;
        d.pose = src.pose;
    }
    let direction = w.mobile(from).map(|m| m.direction);
    if let Some(dir) = direction {
        muo_content::mx::set_direction(w, id, dir);
    }
    clone_body(w, id, from);
    clone_clothes(w, id, from);
    invalidate_hues(w, id);
    invalidate_pose(w, id);
}

fn after_deserialization(w: &mut World, id: MobileId) {
    invalidate_pose(w, id);
    muo_content::mx::set_frozen(w, id, true);
    let remove = statue(w, id).is_some_and(|d| d.sculpted_by.is_none())
        || w.mobile(id).is_some_and(|m| m.map == Some(MapId::INTERNAL));
    if remove {
        delete_mobile(w, id); // Remove preview statues
    }
}

fn demolish_entry() -> ContextMenuEntry {
    let mut e = ContextMenuEntry::new(6275, 2);
    e.on_click = Some(Rc::new(
        |w: &mut World, from: MobileId, target: EntityId| {
            let EntityId::Mobile(t) = target else { return };
            if muo_content::mx::alive(w, from)
                && is_statue(w, t)
                && !w.mobile(t).is_some_and(|m| m.deleted)
            {
                demolish(w, t, from);
            }
        },
    ));
    e
}

/// `[SerializationGenerator(1)] CharacterStatue`.
pub static CHARACTER_STATUE: MobileType = MobileType {
    full_name: "Server.Mobiles.CharacterStatue",
    parent: Some(&MOBILE_BASE),
    construct: None,
    on_double_click: |w, id, from| muo_server::mobile::base::display_paperdoll_to(w, id, from),
    get_properties: |w, id, list: &mut PropertyList| {
        (MOBILE_BASE.get_properties)(w, id, list);
        if let Some(by) = statue(w, id)
            .and_then(|d| d.sculpted_by.clone())
            .filter(|s| !s.is_empty())
        {
            list.add_str(1076202, &by); // Sculpted by ~1_Name~
        }
    },
    get_context_menu_entries: |w, id, from, list| {
        (MOBILE_BASE.get_context_menu_entries)(w, id, from, list);
        let alive = muo_content::mx::alive(w, from);
        if alive && statue(w, id).is_some_and(|d| d.sculpted_by.is_some()) {
            let co_owner = muo_multis::house::access::find_house_at_mobile(w, id)
                .is_some_and(|h| muo_multis::house::access::is_co_owner(w, h, from));
            let staff = w
                .mobile(from)
                .is_some_and(|m| m.access_level > AccessLevel::Counselor);
            if co_owner || staff {
                list.push(demolish_entry());
            }
        }
    },
    on_after_delete: |w, id| {
        (MOBILE_BASE.on_after_delete)(w, id);
        if let Some(p) = statue(w, id).and_then(|d| d.plinth)
            && w.item(p).is_some_and(|i| !i.deleted())
        {
            delete_item(w, p);
        }
    },
    on_map_change: |w, id, _old| {
        invalidate_pose(w, id);
        let (plinth, map) = (
            statue(w, id).and_then(|d| d.plinth),
            w.mobile(id).and_then(|m| m.map),
        );
        if let Some(p) = plinth {
            ops::set_map(w, p, map);
        }
    },
    on_location_change: |w, id, _old| {
        invalidate_pose(w, id);
        let (plinth, loc) = (
            statue(w, id).and_then(|d| d.plinth),
            w.mobile(id).map(|m| m.location),
        );
        if let (Some(p), Some(l)) = (plinth, loc) {
            ops::set_location(w, p, Point3D::new(l.x, l.y, l.z - 5));
        }
    },
    can_be_renamed_by: |_, _, _| false,
    can_be_damaged: |_, _| false,
    on_aos_single_click: |_, _, _| {},
    serialize: |w, id, out| {
        (MOBILE_BASE.serialize)(w, id, out);
        let Some(d) = statue(w, id).cloned() else {
            return;
        };
        out.write_encoded_int(1);
        out.write_enum(4, i64::from(d.statue_type));
        out.write_enum(4, i64::from(d.pose));
        out.write_enum(4, i64::from(d.material));
        out.write_opt_string(d.sculpted_by.as_deref());
        out.write_date_time_ticks(d.sculpted_on.ticks());
        out.write_item(w, d.plinth);
        out.write_bool(d.is_reward_item);
    },
    deserialize: |w, id, r| {
        (MOBILE_BASE.deserialize)(w, id, r)?;
        let version = r.read_encoded_int()?;
        let mut d = StatueData {
            statue_type: 0,
            pose: 0,
            material: 0,
            sculpted_by: None,
            sculpted_on: DateTime::MIN,
            plinth: None,
            is_reward_item: false,
            animation: 0,
            frames: 0,
        };
        let mut sculptor = None;
        if version < 1 {
            d.statue_type = r.read_i32()?;
            d.pose = r.read_i32()?;
            d.material = r.read_i32()?;
            sculptor = r.read_mobile(w)?;
            d.sculpted_on = DateTime::from_ticks(r.read_date_time_ticks()?);
            d.plinth = r.read_item(w)?;
            d.is_reward_item = r.read_bool()?;
        } else {
            d.statue_type = r.read_enum(4)? as i32;
            d.pose = r.read_enum(4)? as i32;
            d.material = r.read_enum(4)? as i32;
            d.sculpted_by = r.read_string()?;
            d.sculpted_on = DateTime::from_ticks(r.read_date_time_ticks()?);
            d.plinth = r.read_item(w)?;
            d.is_reward_item = r.read_bool()?;
        }
        if let Some(m) = w.mobile_mut(id) {
            m.ext.insert(d);
        }
        if version < 1 {
            // `Timer.DelayCall(sculpter => _sculptedBy = sculpter?.RawName, reader.ReadEntity<Mobile>())`
            let name = sculptor
                .and_then(|s| w.mobile(s))
                .and_then(|m| m.name.clone());
            timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
                if let Some(d) = statue_mut(w, id) {
                    d.sculpted_by = name.clone();
                }
            });
        }
        timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
            after_deserialization(w, id)
        });
        Ok(())
    },
    ..MOBILE_BASE
};

// ---- CharacterStatueDeed ----

/// `CharacterStatueDeed`'s fields.
#[derive(Clone, Copy, Debug, Default)]
pub struct DeedData {
    pub statue_type: i32,
    pub statue: Option<MobileId>,
    pub is_reward_item: bool,
}

fn deed(w: &World, id: ItemId) -> DeedData {
    w.item(id)
        .and_then(|i| i.ext.get::<DeedData>())
        .copied()
        .unwrap_or_default()
}

fn deed_edit(w: &mut World, id: ItemId, f: impl FnOnce(&mut DeedData)) {
    if let Some(d) = w.item_mut(id).and_then(|i| i.ext.get_mut::<DeedData>()) {
        f(d);
    }
}

/// `CharacterStatueDeed(statue)`: `base(0x14F0)`.
pub fn deed_new(w: &mut World, statue_ref: Option<MobileId>) -> ItemId {
    let id = ops::new_item_with_id(w, &CHARACTER_STATUE_DEED, 0x14F0);
    let mut d = DeedData::default();
    if let Some(s) = statue_ref {
        d.statue = Some(s);
        d.statue_type = statue(w, s).map_or(0, |x| x.statue_type);
        d.is_reward_item = statue(w, s).is_some_and(|x| x.is_reward_item);
    }
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(d);
    }
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}

fn set_deed_statue(w: &mut World, id: ItemId, value: Option<MobileId>) {
    deed_edit(w, id, |d| d.statue = value);
}

fn set_deed_type(w: &mut World, id: ItemId, value: i32) {
    deed_edit(w, id, |d| d.statue_type = value);
}

fn set_deed_reward(w: &mut World, id: ItemId, value: bool) {
    deed_edit(w, id, |d| d.is_reward_item = value);
    ops::invalidate_properties(w, id);
}

/// `StatueType` of a deed: the backup statue's when there is one.
fn deed_statue_type(w: &World, id: ItemId) -> i32 {
    let d = deed(w, id);
    d.statue
        .and_then(|s| statue(w, s))
        .map_or(d.statue_type, |s| s.statue_type)
}

fn pack_has(w: &World, item: ItemId, from: MobileId) -> bool {
    let pack = w.mobile(from).and_then(|m| m.backpack).map(EntityId::Item);
    ops::is_child_of(w, item, pack)
}

fn is_body_mod(w: &World, m: MobileId) -> bool {
    muo_server::mobile::part1::props_b::is_body_mod(w, m)
}

/// `OnDoubleClick` of the deed and the maker: pick a place.
fn begin_placement(w: &mut World, maker: ItemId, from: MobileId, statue_type: i32) {
    if pack_has(w, maker, from) {
        if !is_body_mod(w, from) {
            muo_content::mx::msg(w, from, 1076194); // Select a place where you would like to put your statue.
            let mut t = Target::new(w, -1, true, TargetFlags::NONE);
            t.handlers.on_target = Some(Rc::new(move |w: &mut World, from, targeted| {
                placement_target(w, maker, from, targeted, statue_type)
            }));
            let tid = target::add_target(w, t);
            target::set_mobile_target(w, from, Some(tid));
        } else {
            muo_content::mx::msg(w, from, 1073648); // You may only proceed while in your original state...
        }
    } else {
        muo_content::mx::msg(w, from, 1042001); // That must be in your pack for you to use it.
    }
}

/// `CharacterStatueTarget.OnTarget`.
fn placement_target(
    w: &mut World,
    maker: ItemId,
    from: MobileId,
    targeted: Targeted,
    statue_type: i32,
) {
    let map = w.mobile(from).and_then(|m| m.map);
    let Some(map) = map else { return };
    if ops::is_deleted(w, maker) {
        return;
    }
    if !pack_has(w, maker, from) {
        muo_content::mx::msg(w, from, 1042001);
        return;
    }
    let loc = addon::surface_point(w, targeted);
    if let Targeted::Entity(EntityId::Item(item)) = targeted
        && !ops::is_locked_down(w, item)
        && !ops::is_secure(w, item)
        && !addon::is_component(w, item)
    {
        muo_content::mx::msg(w, from, 1076191); // Statues can only be placed in houses.
        return;
    }
    if is_body_mod(w, from) {
        muo_content::mx::msg(w, from, 1073648);
        return;
    }
    let mut house = None;
    let result = could_fit(w, loc, map, from, &mut house);
    match result {
        AddonFitResult::Valid => {
            let new_statue = statue_new(w, from, statue_type);
            let plinth = plinth_new(w, new_statue);
            if let Some(h) = house {
                addon::house_addons_add(w, h, plinth);
            }
            let reward = maker_reward(w, maker)
                .or_else(|| Some(deed(w, maker).is_reward_item))
                .unwrap_or(false);
            if let Some(d) = statue_mut(w, new_statue) {
                d.is_reward_item = reward;
                d.plinth = Some(plinth);
            }
            ops::move_to_world_map(w, plinth, loc, Some(map));
            invalidate_pose(w, new_statue);
            // TODO: the maker was not deleted before the customization, which led to redeeding issues.
            let backup = if is_deed(w, maker) {
                deed(w, maker).statue
            } else {
                None
            };
            let is_deed_maker = is_deed(w, maker);
            delete_item(w, maker);
            sculpt(w, new_statue, from);
            display_gump(w, from, is_deed_maker, backup, new_statue);
        }
        AddonFitResult::Blocked => muo_content::mx::msg(w, from, 500269), // You cannot build that there.
        AddonFitResult::NotInHouse => muo_content::mx::msg(w, from, 1076192), // ...where you are the owner or co-owner.
        AddonFitResult::DoorTooClose => muo_content::mx::msg(w, from, 500271), // You cannot build near the door.
        _ => {}
    }
}

/// `CouldFit(p, map, from, ref house)`.
fn could_fit(
    w: &mut World,
    p: Point3D,
    map: MapId,
    from: MobileId,
    house: &mut Option<ItemId>,
) -> AddonFitResult {
    if !muo_server::map::fit::can_fit(w, map, p.x, p.y, p.z, 20, true, true, true) {
        return AddonFitResult::Blocked;
    }
    if !addon::check_house(w, Some(from), p, map, 20, house) {
        return AddonFitResult::NotInHouse;
    }
    match *house {
        Some(h) => check_doors(w, p, 20, h),
        None => AddonFitResult::Valid,
    }
}

/// `CheckDoors(p, height, house)`.
pub fn check_doors(w: &World, p: Point3D, height: i32, house: ItemId) -> AddonFitResult {
    for door in addon::house_doors(w, house) {
        if addon::door_blocks(w, door, p, height) {
            return AddonFitResult::DoorTooClose;
        }
    }
    AddonFitResult::Valid
}

fn is_deed(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.ty.is(&CHARACTER_STATUE_DEED))
}

/// `CharacterStatueDeed` of the deed class.
pub static CHARACTER_STATUE_DEED: ItemType = ItemType {
    full_name: "Server.Mobiles.CharacterStatueDeed",
    parent: Some(&ITEM_BASE),
    construct: None,
    default_weight: |_, _| 1.0,
    label_number: |w, id| {
        // `t = _statueType; if (Statue != null) t = Statue.StatueType`
        match deed_statue_type(w, id) {
            0 => 1076189,
            1 => 1076188,
            2 => 1076190,
            _ => 1076173,
        }
    },
    get_properties: |w, id, list: &mut PropertyList| {
        (ITEM_BASE.get_properties)(w, id, list);
        let d = deed(w, id);
        if d.is_reward_item {
            list.add(1076222); // 6th Year Veteran Reward
        }
        if let Some(name) = d
            .statue
            .and_then(|s| w.mobile(s).and_then(|m| (m.ty.name)(w, s)))
        {
            list.add_str(1076231, &name); // Statue of ~1_Name~
        }
    },
    on_double_click: |w, id, from| {
        let player = w
            .mobile(from)
            .is_some_and(|m| m.access_level == AccessLevel::Player);
        if player && let Some(acct) = w.mobile(from).and_then(|m| m.account) {
            let age = muo_content::accounting::account_age(w, acct);
            let interval = TimeSpan::from_days_f64(super::state_interval_days(w) * 6.0);
            let time = interval - age;
            if time > TimeSpan::ZERO {
                let months = (time.total_days() / super::state_interval_days(w)).ceil();
                // Your account is not old enough to use this item. Months until you can use this item :
                muo_server::mobile::part2::messages::send_localized_message_affix(
                    w,
                    from,
                    1008126,
                    true,
                    &format!("{months}"),
                    "",
                    0x3B2,
                );
                return;
            }
        }
        let t = deed_statue_type(w, id);
        begin_placement(w, id, from, t);
    },
    on_delete: |w, id| {
        (ITEM_BASE.on_delete)(w, id);
        if let Some(s) = deed(w, id).statue {
            delete_mobile(w, s);
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        let d = deed(w, id);
        out.write_encoded_int(1);
        out.write_enum(4, i64::from(d.statue_type));
        out.write_mobile(w, d.statue);
        out.write_bool(d.is_reward_item);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let version = r.read_encoded_int()?;
        let d = if version < 1 {
            DeedData {
                statue_type: r.read_i32()?,
                statue: mobile_alive_ref(w, r.read_mobile(w)?),
                is_reward_item: r.read_bool()?,
            }
        } else {
            DeedData {
                statue_type: r.read_enum(4)? as i32,
                statue: mobile_alive_ref(w, r.read_mobile(w)?),
                is_reward_item: r.read_bool()?,
            }
        };
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(d);
        }
        Ok(())
    },
    ..ITEM_BASE
};

// ---- CharacterStatueMaker ----

/// `CharacterStatueMaker`'s fields.
#[derive(Clone, Copy, Debug, Default)]
pub struct MakerData {
    pub is_reward_item: bool,
    pub statue_type: i32,
}

fn maker(w: &World, id: ItemId) -> Option<MakerData> {
    w.item(id)?.ext.get::<MakerData>().copied()
}

pub fn maker_reward(w: &World, id: ItemId) -> Option<bool> {
    maker(w, id).map(|d| d.is_reward_item)
}

/// `IsRewardItem = value` of a maker.
pub fn set_maker_reward(w: &mut World, id: ItemId, value: bool) {
    if let Some(d) = w.item_mut(id).and_then(|i| i.ext.get_mut::<MakerData>()) {
        d.is_reward_item = value;
    }
    ops::invalidate_properties(w, id);
}

/// `CharacterStatueMaker(type)`: `base(0x32F0)`.
pub fn maker_new(w: &mut World, ty: &'static ItemType, statue_type: i32) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0x32F0);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(MakerData {
            is_reward_item: false,
            statue_type,
        });
    }
    crate::util::hue(w, id, 0xB8F + statue_type * 4); // InvalidateHue
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}

macro_rules! maker_class {
    ($(#[$m:meta])* $name:ident = $full:literal, $ty:expr) => {
        $(#[$m])*
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&CHARACTER_STATUE_MAKER),
            construct: Some(|w| maker_new(w, &$name, $ty)),
            serialize: |w, id, out| {
                (CHARACTER_STATUE_MAKER.serialize)(w, id, out);
                out.write_encoded_int(0);
            },
            deserialize: |w, id, r| {
                (CHARACTER_STATUE_MAKER.deserialize)(w, id, r)?;
                let _version = r.read_encoded_int()?;
                Ok(())
            },
            ..CHARACTER_STATUE_MAKER
        };
    };
}

/// `Server.Items.CharacterStatueMaker` (abstract), `[SerializationGenerator(1)]`.
pub static CHARACTER_STATUE_MAKER: ItemType = ItemType {
    full_name: "Server.Items.CharacterStatueMaker",
    parent: Some(&ITEM_BASE),
    construct: None,
    default_weight: |_, _| 5.0,
    label_number: |_, _| 1076173, // Character Statue Maker
    get_properties: |w, id, list: &mut PropertyList| {
        (ITEM_BASE.get_properties)(w, id, list);
        if maker(w, id).is_some_and(|d| d.is_reward_item) {
            list.add(1076222); // 6th Year Veteran Reward
        }
    },
    on_double_click: |w, id, from| {
        let Some(d) = maker(w, id) else { return };
        if d.is_reward_item
            && !super::check_is_usable_by(
                w,
                from,
                "Server.Items.CharacterStatueMaker",
                false,
                Some(&[i64::from(d.statue_type)]),
            )
        {
            return;
        }
        begin_placement(w, id, from, d.statue_type);
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        let d = maker(w, id).unwrap_or_default();
        out.write_encoded_int(1);
        out.write_bool(d.is_reward_item);
        out.write_enum(4, i64::from(d.statue_type));
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let version = r.read_encoded_int()?;
        let d = if version < 1 {
            MakerData {
                is_reward_item: r.read_bool()?,
                statue_type: r.read_i32()?,
            }
        } else {
            MakerData {
                is_reward_item: r.read_bool()?,
                statue_type: r.read_enum(4)? as i32,
            }
        };
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(d);
        }
        Ok(())
    },
    ..ITEM_BASE
};

maker_class!(
    /// `Server.Items.MarbleStatueMaker`.
    MARBLE_STATUE_MAKER = "Server.Items.MarbleStatueMaker", statue_type::MARBLE
);
maker_class!(
    /// `Server.Items.JadeStatueMaker`.
    JADE_STATUE_MAKER = "Server.Items.JadeStatueMaker", statue_type::JADE
);
maker_class!(
    /// `Server.Items.BronzeStatueMaker`.
    BRONZE_STATUE_MAKER = "Server.Items.BronzeStatueMaker", statue_type::BRONZE
);

// ---- CharacterStatuePlinth ----

fn plinth_statue(w: &World, id: ItemId) -> Option<MobileId> {
    w.item(id)?.ext.get::<PlinthData>()?.0
}

/// `CharacterStatuePlinth._statue`.
#[derive(Clone, Copy, Debug, Default)]
pub struct PlinthData(pub Option<MobileId>);

/// `CharacterStatuePlinth(statue)`: `base(0x32F2)`.
pub fn plinth_new(w: &mut World, statue_ref: MobileId) -> ItemId {
    let id = ops::new_item_with_id(w, &CHARACTER_STATUE_PLINTH, 0x32F2);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(PlinthData(Some(statue_ref)));
    }
    plinth_invalidate_hue(w, id);
    id
}

/// `InvalidateHue()`.
pub fn plinth_invalidate_hue(w: &mut World, id: ItemId) {
    if let Some(d) = plinth_statue(w, id).and_then(|s| statue(w, s)).cloned() {
        crate::util::hue(w, id, 0xB8F + d.statue_type * 4 + d.material);
    }
}

fn plinth_after_deserialization(w: &mut World, id: ItemId) {
    let empty = plinth_statue(w, id)
        .and_then(|s| statue(w, s))
        .is_none_or(|d| d.sculpted_by.as_deref().is_none_or(str::is_empty));
    if empty || w.item(id).is_some_and(|i| i.map == Some(MapId::INTERNAL)) {
        delete_item(w, id);
    }
}

fn plinth_could_fit(w: &mut World, _item: ItemId, p: Point3D, map: MapId) -> bool {
    if !muo_server::map::fit::can_fit(w, map, p.x, p.y, p.z, 20, false, true, true) {
        return false;
    }
    let Some(house) = muo_multis::house::access::find_house_at_point(w, p, Some(map), 20) else {
        return false;
    };
    check_doors(w, p, 20, house) == AddonFitResult::Valid
}

/// `CharacterStatuePlinth` `[SerializationGenerator(0)]`, a `Static` and `IAddon`.
pub static CHARACTER_STATUE_PLINTH: ItemType = ItemType {
    full_name: "Server.Items.CharacterStatuePlinth",
    parent: Some(&STATIC),
    construct: None,
    label_number: |_, _| 1076201, // Character Statue
    on_after_delete: |w, id| {
        (STATIC.on_after_delete)(w, id);
        if let Some(s) = plinth_statue(w, id)
            && w.mobile(s).is_some_and(|m| !m.deleted)
        {
            delete_mobile(w, s);
        }
    },
    on_map_change: |w, id| {
        let (s, map) = (plinth_statue(w, id), w.item(id).and_then(|i| i.map));
        if let Some(s) = s {
            // `_statue.Map = Map`
            if let (Some(map), Some(loc)) = (map, w.mobile(s).map(|m| m.location)) {
                muo_server::mobile::base::move_to_world(w, s, loc, map);
            }
        }
    },
    on_location_change: |w, id, _old| {
        let (s, loc) = (plinth_statue(w, id), ops::get_world_location(w, id));
        if let Some(s) = s {
            muo_server::mobile::part1::lifecycle::set_location(
                w,
                s,
                Point3D::new(loc.x, loc.y, loc.z + 5),
            );
        }
    },
    on_double_click: |w, id, from| {
        if let Some(s) = plinth_statue(w, id) {
            display_plinth_gump(w, from, s);
        }
    },
    serialize: |w, id, out| {
        (STATIC.serialize)(w, id, out);
        out.write_encoded_int(0);
        out.write_mobile(w, plinth_statue(w, id));
    },
    deserialize: |w, id, r| {
        (STATIC.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        let s = r.read_mobile(w)?;
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(PlinthData(s));
        }
        timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
            plinth_after_deserialization(w, id)
        });
        Ok(())
    },
    ..STATIC
};

fn plinth_deed(w: &mut World, id: ItemId) -> ItemId {
    deed_new(w, plinth_statue(w, id))
}

// ---- CharacterPlinthGump ----

struct CharacterPlinthGump {
    name: String,
    sculpted_on: String,
    type_number: i32,
}

fn display_plinth_gump(w: &mut World, from: MobileId, s: MobileId) {
    if w.mobile(from).and_then(|m| m.net_state).is_none() || w.mobile(s).is_none_or(|m| m.deleted) {
        return;
    }
    let Some(d) = statue(w, s).cloned() else {
        return;
    };
    let g = CharacterPlinthGump {
        name: w
            .mobile(s)
            .and_then(|m| (m.ty.name)(w, s))
            .unwrap_or_default(),
        sculpted_on: format!("{}", d.sculpted_on),
        type_number: match d.statue_type {
            0 => 1076181,
            1 => 1076180,
            2 => 1076230,
            _ => 1076181,
        },
    };
    send_gump_to_mobile(w, from, Box::new(DynamicGumpObj::new(60, 30, g)), true);
}

impl DynamicGumpLogic for CharacterPlinthGump {
    const TYPE_NAME: &'static str = "Server.Items.CharacterStatuePlinth+CharacterPlinthGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.add_page(0);
        b.add_image(0, 0, 0x24F4, 0);
        b.add_html(55, 50, 150, 20, &self.name, false, false);
        b.add_html(55, 75, 150, 20, &self.sculpted_on, false, false);
        b.add_html_localized(55, 100, 150, 20, self.type_number, false, false);
    }
}

// ---- CharacterStatueGump ----

struct CharacterStatueGump {
    /// `_maker is CharacterStatueDeed` and the deed's backup statue, as they were when the gump was made.
    maker_is_deed: bool,
    backup: Option<MobileId>,
    statue: MobileId,
    pose: i32,
    direction: muo_primitives::Direction,
    statue_type: i32,
    material: i32,
}

fn display_gump(
    w: &mut World,
    from: MobileId,
    maker_is_deed: bool,
    backup: Option<MobileId>,
    s: MobileId,
) {
    if w.mobile(from).and_then(|m| m.net_state).is_none() || w.mobile(s).is_none_or(|m| m.deleted) {
        return;
    }
    let (Some(d), Some(m)) = (statue(w, s).cloned(), w.mobile(s)) else {
        return;
    };
    let g = CharacterStatueGump {
        maker_is_deed,
        backup,
        statue: s,
        pose: d.pose,
        direction: m.direction,
        statue_type: d.statue_type,
        material: d.material,
    };
    send_gump_to_mobile(w, from, Box::new(DynamicGumpObj::new(60, 36, g)), true);
}

fn material_number(statue_type: i32, material: i32) -> i32 {
    match material {
        0 => match statue_type {
            2 => 1076187,
            1 => 1076186,
            0 => 1076182,
            _ => 1076187,
        },
        1 => {
            if statue_type == 0 {
                1076183
            } else {
                1076182
            }
        }
        2 => 1076184,
        3 => 1076185,
        _ => 1076187,
    }
}

fn direction_number(d: muo_primitives::Direction) -> i32 {
    match d.bits() & 7 {
        0 => 1075389, // North
        1 => 1075388, // Right
        2 => 1075387, // East
        3 => 1076204, // Down
        4 => 1075386, // South
        5 => 1075391, // Left
        6 => 1075390, // West
        7 => 1076205, // Up
        _ => 1075386,
    }
}

mod button {
    pub const CLOSE: i32 = 0;
    pub const SCULPT: i32 = 1;
    pub const POSE_PREV: i32 = 2;
    pub const POSE_NEXT: i32 = 3;
    pub const DIR_PREV: i32 = 4;
    pub const DIR_NEXT: i32 = 5;
    pub const MAT_PREV: i32 = 6;
    pub const MAT_NEXT: i32 = 7;
    pub const RESTORE: i32 = 8;
}

impl DynamicGumpLogic for CharacterStatueGump {
    const TYPE_NAME: &'static str = "Server.Gumps.CharacterStatueGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.add_page(0);
        b.add_background(0, 0, 327, 324, 0x13BE);
        b.add_image_tiled(10, 10, 307, 20, 0xA40);
        b.add_image_tiled(10, 40, 307, 244, 0xA40);
        b.add_image_tiled(10, 294, 307, 20, 0xA40);
        b.add_alpha_region(10, 10, 307, 304);
        b.add_html_localized_color(14, 12, 327, 20, 1076156, 0x7FFF, false, false); // Character Statue Maker

        // pose
        b.add_html_localized_color(133, 41, 120, 20, 1076168, 0x7FFF, false, false); // Choose Pose
        b.add_html_localized_color(133, 61, 120, 20, 1076208 + self.pose, 0x77E, false, false);
        b.add_button(163, 81, 0xFA5, 0xFA7, button::POSE_NEXT);
        b.add_button(133, 81, 0xFAE, 0xFB0, button::POSE_PREV);

        // direction
        b.add_html_localized_color(133, 126, 120, 20, 1076170, 0x7FFF, false, false); // Choose Direction
        b.add_html_localized_color(
            133,
            146,
            120,
            20,
            direction_number(self.direction),
            0x77E,
            false,
            false,
        );
        b.add_button(163, 167, 0xFA5, 0xFA7, button::DIR_NEXT);
        b.add_button(133, 167, 0xFAE, 0xFB0, button::DIR_PREV);

        // material
        b.add_html_localized_color(133, 211, 120, 20, 1076171, 0x7FFF, false, false); // Choose Material
        b.add_html_localized_color(
            133,
            231,
            120,
            20,
            material_number(self.statue_type, self.material),
            0x77E,
            false,
            false,
        );
        b.add_button(163, 253, 0xFA5, 0xFA7, button::MAT_NEXT);
        b.add_button(133, 253, 0xFAE, 0xFB0, button::MAT_PREV);

        // cancel
        b.add_button(10, 294, 0xFB1, 0xFB2, button::CLOSE);
        b.add_html_localized_color(45, 294, 80, 20, 1006045, 0x7FFF, false, false); // Cancel

        // sculpt
        b.add_button(234, 294, 0xFB7, 0xFB9, button::SCULPT);
        b.add_html_localized_color(269, 294, 80, 20, 1076174, 0x7FFF, false, false); // Sculpt

        // restore
        if self.maker_is_deed {
            b.add_button(107, 294, 0xFAB, 0xFAD, button::RESTORE);
            b.add_html_localized_color(142, 294, 80, 20, 1076193, 0x7FFF, false, false); // Restore
        }
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(from) = w.net_states.get(&sender).and_then(|n| n.mobile) else {
            return;
        };
        let s = self.statue;
        if w.mobile(s).is_none_or(|m| m.deleted) || !is_statue(w, s) {
            return;
        }
        let mut send_gump = false;
        let (pose, mat) = statue(w, s).map_or((0, 0), |d| (d.pose, d.material));
        let dir = w.mobile(s).map_or(0, |m| i32::from(m.direction.bits() & 7));
        match info.button_id {
            button::SCULPT => {
                if self.maker_is_deed
                    && let Some(backup) = self.backup
                {
                    delete_mobile(w, backup);
                }
                // `_maker?.Delete()`: the maker was already deleted when the statue was placed.
                sculpt(w, s, from);
            }
            button::POSE_PREV => {
                set_pose(w, s, (pose + 5) % 6);
                send_gump = true;
            }
            button::POSE_NEXT => {
                set_pose(w, s, (pose + 1) % 6);
                send_gump = true;
            }
            button::DIR_PREV => {
                set_direction(w, s, (dir + 7) % 8);
                invalidate_pose(w, s);
                send_gump = true;
            }
            button::DIR_NEXT => {
                set_direction(w, s, (dir + 1) % 8);
                invalidate_pose(w, s);
                send_gump = true;
            }
            button::MAT_PREV => {
                set_material(w, s, (mat + 3) % 4);
                send_gump = true;
            }
            button::MAT_NEXT => {
                set_material(w, s, (mat + 1) % 4);
                send_gump = true;
            }
            button::RESTORE => {
                if self.maker_is_deed
                    && let Some(backup) = self.backup.filter(|b| w.mobile(*b).is_some())
                {
                    restore(w, s, backup);
                }
                send_gump = true;
            }
            _ => send_gump = !demolish(w, s, from), // Close
        }
        if send_gump {
            display_gump(w, from, self.maker_is_deed, self.backup, s);
        }
    }
}

fn set_direction(w: &mut World, id: MobileId, value: i32) {
    muo_content::mx::set_direction(
        w,
        id,
        muo_primitives::Direction::from_bits_retain(value as u8),
    );
}

// ---- registration ----

pub fn register(types: &mut TypeRegistry) {
    types.register_mobile(&CHARACTER_STATUE, &[]);
    // The class is in `Server.Mobiles`; `Server.Items` is an older name some saves still carry.
    types.register_item(
        &CHARACTER_STATUE_DEED,
        &["Server.Items.CharacterStatueDeed"],
    );
    for t in [
        &CHARACTER_STATUE_MAKER,
        &MARBLE_STATUE_MAKER,
        &JADE_STATUE_MAKER,
        &BRONZE_STATUE_MAKER,
        &CHARACTER_STATUE_PLINTH,
    ] {
        types.register_item(t, &[]);
    }
    types.register_phase(
        Phase::Configure,
        "CharacterStatue.Configure",
        DEFAULT_CALL_PRIORITY,
        |w| {
            house_deps::deps_mut(w).simple_addons.push(SimpleAddon {
                is: |w, id| {
                    w.item(id)
                        .is_some_and(|i| i.ty.is(&CHARACTER_STATUE_PLINTH))
                },
                deed: plinth_deed,
                could_fit: plinth_could_fit,
            });
        },
    );
}

// ---- accessors of the property tables ----

/// `StatueType` of a maker.
pub fn maker_type(w: &World, id: ItemId) -> i64 {
    maker(w, id).map_or(0, |d| i64::from(d.statue_type))
}

/// `StatueType = value` of a maker (`OnStatueTypeChanged`: `InvalidateHue()`).
pub fn set_maker_type(w: &mut World, id: ItemId, value: i32) {
    let changed = w
        .item_mut(id)
        .and_then(|i| i.ext.get_mut::<MakerData>())
        .is_some_and(|d| {
            let c = d.statue_type != value;
            d.statue_type = value;
            c
        });
    if changed {
        crate::util::hue(w, id, 0xB8F + value * 4);
    }
}

/// `StatueType` of a deed (the backup statue's when there is one).
pub fn deed_type(w: &World, id: ItemId) -> i32 {
    deed_statue_type(w, id)
}

/// `StatueType = value` of a deed.
pub fn set_deed_type_pub(w: &mut World, id: ItemId, value: i32) {
    set_deed_type(w, id, value);
}
