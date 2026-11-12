//! Ethereal mounts: `EtherealMount`, the statuette that summons a mount for its rider, and its
//! fourteen concrete classes. The rider's `Mobile.Mount` is the rider itself (as for a `VirtualMountItem`, whose
//! `IMount` is not a mobile): `ContentHooks::ethereal_mount_rider` / `ethereal_dismount` say so.

use muo_content::creature::mount as base_mount;
use muo_content::items::is_named;
use muo_content::mx;
use muo_content::spells::spell::{self, SPELL_BASE, SpellId, SpellType};
use muo_content::spells::{DisturbType, SpellInfo};
use muo_io::{GenericReader, GenericWriter};
use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{
    DeathMoveResult, ITEM_BASE, ItemType, LootType, base as ibase, container, ops,
};
use muo_server::layer::Layer;
use muo_server::mobile::part1::{props_a, props_b};
use muo_server::mobile::part2::{items, messages, speech};
use muo_server::property_list::PropertyList;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::type_registry::TypeRegistry;

/// `StepsMax`: "should be same as horse".
pub const STEPS_MAX: i32 = 3840;

/// What `EtherealMount` keeps: the two flags, the two item ids, the rider and the steps.
#[derive(Clone, Copy, Debug, Default)]
pub struct EtherealData {
    pub is_donation_item: bool,
    pub is_reward_item: bool,
    pub mounted_id: i32,
    pub regular_id: i32,
    pub rider: Option<MobileId>,
    pub steps: i32,
}

fn data(w: &World, id: ItemId) -> EtherealData {
    w.item(id)
        .and_then(|i| i.ext.get::<EtherealData>())
        .copied()
        .unwrap_or_default()
}
fn update(w: &mut World, id: ItemId, f: impl FnOnce(&mut EtherealData)) {
    let mut d = data(w, id);
    f(&mut d);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(d);
    }
}

pub fn is_ethereal(w: &World, id: ItemId) -> bool {
    w.item(id)
        .is_some_and(|i| is_named(i.ty, "Server.Mobiles.EtherealMount"))
}

/// `Rider`.
pub fn rider(w: &World, id: ItemId) -> Option<MobileId> {
    data(w, id).rider
}

/// `IsRewardItem = value` (the veteran reward system marks the statuettes it gives).
pub fn set_is_reward_item(w: &mut World, id: ItemId, value: bool) {
    update(w, id, |d| d.is_reward_item = value);
}

/// `EtherealHue`: `0x4001`, 0 for the polar bear and the charger.
fn ethereal_hue(w: &World, id: ItemId) -> i32 {
    match w.item(id).map(|i| i.ty.full_name) {
        Some("Server.Mobiles.RideablePolarBear" | "Server.Mobiles.ChargerOfTheFallen") => 0,
        _ => 0x4001,
    }
}

/// `EtherealMount(itemID, mountID)`.
fn ethereal_make(w: &mut World, ty: &'static ItemType, item_id: i32, mount_id: i32) -> ItemId {
    let id = ops::new_item_with_id(w, ty, item_id);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(EtherealData {
            mounted_id: mount_id,
            regular_id: item_id,
            ..EtherealData::default()
        });
    }
    ibase::set_layer(w, id, Layer::Invalid);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}

/// `new EtherealMount(itemID, mountID)` (the abstract base: a plain statue of the two graphics).
pub fn ethereal_mount_new(w: &mut World, item_id: i32, mount_id: i32) -> ItemId {
    ethereal_make(w, &ETHEREAL_MOUNT, item_id, mount_id)
}

/// `MountedID` set: a riding mount shows it at once.
pub fn set_mounted_id(w: &mut World, id: ItemId, value: i32) {
    update(w, id, |d| d.mounted_id = value);
    if data(w, id).rider.is_some() {
        ibase::set_item_id(w, id, value);
    }
}

/// `RegularID` set.
pub fn set_regular_id(w: &mut World, id: ItemId, value: i32) {
    update(w, id, |d| d.regular_id = value);
    if data(w, id).rider.is_none() {
        ibase::set_item_id(w, id, value);
    }
}

// ---- Rider ----

/// `RemoveFollowers()`.
fn remove_followers(w: &mut World, id: ItemId) {
    if let Some(r) = data(w, id).rider {
        let f = w.mobile(r).map_or(0, |m| m.followers);
        props_a::set_followers(w, r, f - f.min(1));
    }
}

/// `AddFollowers()`.
fn add_followers(w: &mut World, id: ItemId) {
    if let Some(r) = data(w, id).rider {
        let f = w.mobile(r).map_or(0, |m| m.followers);
        props_a::set_followers(w, r, f + 1);
    }
}

/// `Dismount(m)`: `m.Mount?.Rider = null` for a base mount or an ethereal one.
pub fn dismount(w: &mut World, m: MobileId) {
    base_mount::dismount(w, m);
}

/// `Rider` set.
pub fn set_rider(w: &mut World, id: ItemId, value: Option<MobileId>) {
    let current = data(w, id).rider;
    if value == current {
        return;
    }
    match value {
        None => {
            ops::internalize(w, id);
            unmount_me(w, id);
            remove_followers(w, id);
            update(w, id, |d| d.rider = None);
        }
        Some(v) => {
            if let Some(c) = current {
                dismount(w, c);
            }
            dismount(w, v);
            remove_followers(w, id);
            update(w, id, |d| d.rider = Some(v));
            add_followers(w, id);
            mount_me(w, id);
        }
    }
}

/// `UnmountMe()`: back to the statuette, into the rider's pack or onto the ground where it is.
pub fn unmount_me(w: &mut World, id: ItemId) {
    let Some(r) = data(w, id).rider else { return };
    let bp = props_b::backpack(w, r);
    let d = data(w, id);
    ibase::set_item_id(w, id, d.regular_id);
    ibase::set_layer(w, id, Layer::Invalid);
    ops::set_movable(w, id, true);
    if ibase::hue(w, id) == ethereal_hue(w, id) {
        ibase::set_hue(w, id, 0);
    }
    if let Some(bp) = bp {
        container::drop_item_default(w, bp, id);
    } else {
        let (mut loc, mut map) = (mx::location(w, r), mx::map(w, r));
        if map.is_none_or(|m| m == muo_server::map::MapId::INTERNAL)
            && let Some(m) = w.mobile(r)
        {
            loc = m.logout_location;
            map = m.logout_map;
        }
        ops::move_to_world_map(w, id, loc, map);
    }
}

/// `MountMe()`: the statuette becomes the mount the rider wears.
pub fn mount_me(w: &mut World, id: ItemId) {
    let Some(r) = data(w, id).rider else { return };
    let d = data(w, id);
    ibase::set_item_id(w, id, d.mounted_id);
    ibase::set_layer(w, id, Layer::Mount);
    ops::set_movable(w, id, false);
    if ibase::hue(w, id) == 0 {
        let h = ethereal_hue(w, id);
        ibase::set_hue(w, id, h);
    }
    ibase::process_delta(w, id);
    muo_server::mobile::base::process_delta(w, r);
    muo_server::mobile::base::equip_item(w, r, id);
    muo_server::mobile::base::process_delta(w, r);
    ibase::process_delta(w, id);
}

// ---- use ----

/// `Validate(from)`.
pub fn validate(w: &mut World, id: ItemId, from: MobileId) -> bool {
    if w.item(id).is_none_or(|i| i.parent.is_none()) {
        speech::say_to_number(w, from, from, 1010095, ""); // This must be on your person to use.
        return false;
    }
    let reward_blocked = data(w, id).is_reward_item && !reward_usable_by(w, from, id);
    if reward_blocked || !base_mount::check_mount_allowed(w, from) {
        return false;
    }
    if props_b::mounted(w, from) {
        mx::msg(w, from, 1005583); // Please dismount first.
        return false;
    }
    let body_mod = w.mobile(from).is_some_and(|m| m.body_mod.0 != 0);
    if body_mod && !muo_content::creature::misc::body_is_human(w, from) {
        mx::msg(w, from, 1061628); // You can't do that while polymorphed.
        return false;
    }
    if props_b::has_trade(w, from) {
        messages::send_localized_message(w, from, 1042317, "", 0x41); // You may not ride at this time
        return false;
    }
    let (followers, max) = w
        .mobile(from)
        .map_or((0, 0), |m| (m.followers, m.followers_max));
    if followers + 1 > max {
        mx::msg(w, from, 1049679); // You have too many followers to summon your mount.
        return false;
    }
    muo_content::player_mobile::hooks::design_check(w, from)
}

/// `RewardSystem.CheckIsUsableBy(from, this, Array.Empty<object>())`: usable when no reward system is installed.
fn reward_usable_by(w: &mut World, from: MobileId, id: ItemId) -> bool {
    match w
        .resources
        .get::<muo_items::clothing::special::RewardHooks>()
        .and_then(|h| h.check_is_usable_by)
    {
        Some(f) => f(w, from, id),
        None => true,
    }
}

fn on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    if validate(w, id, from) {
        let s = spell::new_spell(w, &ETHEREAL_SPELL, from, None);
        if let Some(sp) = spell::spells(w).get_mut(s) {
            sp.ext = muo_server::Ext::new(SpellMount(id));
        }
        spell::cast(w, s);
    }
}

fn on_single_click(w: &mut World, id: ItemId, from: MobileId) {
    (ITEM_BASE.on_single_click)(w, id, from);
    let text = if data(w, id).is_donation_item {
        "Donation Ethereal"
    } else {
        "Veteran Reward"
    };
    ops::label_to(w, id, from, text);
}

fn get_properties(w: &World, id: ItemId, list: &mut PropertyList) {
    (ITEM_BASE.get_properties)(w, id, list);
    let d = data(w, id);
    if d.is_donation_item {
        list.add_text("Donation Ethereal");
        list.add_text("7.5 sec slower cast time if not a 9mo. Veteran");
    }
    if w.core_ml() && d.is_reward_item {
        // X Year Veteran Reward
        if let Some(n) = muo_items::special::util::reward_year_label(w, id) {
            list.add(n);
        }
    }
}

fn on_parent_death(w: &mut World, id: ItemId, _parent: MobileId) -> DeathMoveResult {
    set_rider(w, id, None); // get off, move to pack
    DeathMoveResult::RemainEquipped
}

// ---- save: version 4 with flags ----

const FLAG_DONATION: i64 = 0x1;
const FLAG_REWARD: i64 = 0x2;
const FLAG_RIDER: i64 = 0x4;
const FLAG_STEPS: i64 = 0x8;

fn serialize(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    (ITEM_BASE.serialize)(w, id, out);
    out.write_i32(4);
    let d = data(w, id);
    let mut flags = 0;
    if d.is_donation_item {
        flags |= FLAG_DONATION;
    }
    if d.is_reward_item {
        flags |= FLAG_REWARD;
    }
    if d.rider.is_some() {
        flags |= FLAG_RIDER;
    }
    if d.steps != STEPS_MAX {
        flags |= FLAG_STEPS;
    }
    out.write_enum(4, flags);
    out.write_i32(d.mounted_id);
    out.write_i32(d.regular_id);
    if flags & FLAG_RIDER != 0 {
        out.write_mobile(w, d.rider);
    }
    if flags & FLAG_STEPS != 0 {
        out.write_i32(d.steps);
    }
}

fn deserialize(w: &mut World, id: ItemId, r: &mut dyn GenericReader) -> muo_io::Result<()> {
    (ITEM_BASE.deserialize)(w, id, r)?;
    let version = r.read_i32()?;
    let mut d = EtherealData::default();
    if version < 4 {
        d.is_donation_item = r.read_bool()?;
        d.is_reward_item = r.read_bool()?;
        d.mounted_id = r.read_i32()?;
        d.regular_id = r.read_i32()?;
        d.rider = r.read_mobile(w)?;
        d.steps = STEPS_MAX;
    } else {
        let flags = r.read_enum(4)?;
        d.is_donation_item = flags & FLAG_DONATION != 0;
        d.is_reward_item = flags & FLAG_REWARD != 0;
        d.mounted_id = r.read_i32()?;
        d.regular_id = r.read_i32()?;
        if flags & FLAG_RIDER != 0 {
            d.rider = r.read_mobile(w)?;
        }
        if flags & FLAG_STEPS != 0 {
            d.steps = r.read_i32()?;
        }
    }
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(d);
    }
    // [AfterDeserialize]
    add_followers(w, id);
    Ok(())
}

pub static ETHEREAL_MOUNT: ItemType = ItemType {
    full_name: "Server.Mobiles.EtherealMount",
    parent: Some(&ITEM_BASE),
    // `EtherealMount(itemID, mountID)` is for the concrete classes
    construct: None,
    default_weight: |_, _| 1.0,
    display_loot_type: |_, _| false,
    get_properties,
    on_double_click,
    on_single_click,
    on_parent_death,
    serialize,
    deserialize,
    ..ITEM_BASE
};

// ---- the concrete classes ----

macro_rules! ethereal {
    ($name:ident, $full:literal, $item:expr, $mount:expr, $label:expr, $encoded:literal) => {
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&ETHEREAL_MOUNT),
            construct: Some(|w| ethereal_make(w, &$name, $item, $mount)),
            label_number: |_, _| $label,
            serialize: |w, id, out| {
                (ETHEREAL_MOUNT.serialize)(w, id, out);
                if $encoded {
                    out.write_encoded_int(0);
                } else {
                    out.write_i32(0);
                }
            },
            deserialize: |w, id, r| {
                (ETHEREAL_MOUNT.deserialize)(w, id, r)?;
                let _version = if $encoded {
                    r.read_encoded_int()?
                } else {
                    r.read_i32()?
                };
                Ok(())
            },
            ..ETHEREAL_MOUNT
        };
    };
}

ethereal!(
    ETHEREAL_HORSE,
    "Server.Mobiles.EtherealHorse",
    0x20DD,
    0x3EAA,
    1041298,
    false
); // Ethereal Horse Statuette
ethereal!(
    ETHEREAL_LLAMA,
    "Server.Mobiles.EtherealLlama",
    0x20F6,
    0x3EAB,
    1041300,
    false
); // Ethereal Llama Statuette
ethereal!(
    ETHEREAL_OSTARD,
    "Server.Mobiles.EtherealOstard",
    0x2135,
    0x3EAC,
    1041299,
    false
); // Ethereal Ostard Statuette
ethereal!(
    ETHEREAL_RIDGEBACK,
    "Server.Mobiles.EtherealRidgeback",
    0x2615,
    0x3E9A,
    1049747,
    false
); // Ethereal Ridgeback Statuette
ethereal!(
    ETHEREAL_UNICORN,
    "Server.Mobiles.EtherealUnicorn",
    0x25CE,
    0x3E9B,
    1049745,
    false
); // Ethereal Unicorn Statuette
ethereal!(
    ETHEREAL_BEETLE,
    "Server.Mobiles.EtherealBeetle",
    0x260F,
    0x3E97,
    1049748,
    false
); // Ethereal Beetle Statuette
ethereal!(
    ETHEREAL_KIRIN,
    "Server.Mobiles.EtherealKirin",
    0x25A0,
    0x3E9C,
    1049746,
    false
); // Ethereal Ki-Rin Statuette
ethereal!(
    ETHEREAL_SWAMP_DRAGON,
    "Server.Mobiles.EtherealSwampDragon",
    0x2619,
    0x3E98,
    1049749,
    false
); // Ethereal Swamp Dragon Statuette
ethereal!(
    RIDEABLE_POLAR_BEAR,
    "Server.Mobiles.RideablePolarBear",
    0x20E1,
    0x3EC5,
    1076159,
    true
); // Rideable Polar Bear
ethereal!(
    ETHEREAL_CU_SIDHE,
    "Server.Mobiles.EtherealCuSidhe",
    0x2D96,
    0x3E91,
    1080386,
    true
); // Ethereal Cu Sidhe Statuette
ethereal!(
    ETHEREAL_HIRYU,
    "Server.Mobiles.EtherealHiryu",
    0x276A,
    0x3E94,
    1113813,
    true
); // Ethereal Hiryu Statuette
ethereal!(
    ETHEREAL_REPTALON,
    "Server.Mobiles.EtherealReptalon",
    0x2d95,
    0x3e90,
    1113812,
    true
); // Ethereal Reptalon Statuette
ethereal!(
    CHARGER_OF_THE_FALLEN,
    "Server.Mobiles.ChargerOfTheFallen",
    0x2D9C,
    0x3E92,
    1074816,
    false
); // Charger of the Fallen Statuette

// ---- EtherealSpell ----

/// `EtherealSpell.m_Mount`.
#[derive(Clone, Copy)]
struct SpellMount(ItemId);

/// `EtherealSpell.m_Stop`.
#[derive(Clone, Copy, Default)]
struct Stopped(bool);

fn spell_mount(w: &World, s: SpellId) -> Option<ItemId> {
    spell::spell(w, s)
        .and_then(|x| x.ext.get::<SpellMount>())
        .map(|m| m.0)
}
fn stopped(w: &World, s: SpellId) -> bool {
    spell::spell(w, s)
        .and_then(|x| x.ext.get::<Stopped>())
        .is_some_and(|m| m.0)
}

/// `EtherealSpell`.
pub static ETHEREAL_SPELL: SpellType = SpellType {
    full_name: "Server.Mobiles.EtherealMount+EtherealSpell",
    parent: Some(&SPELL_BASE),
    info: SpellInfo::new("Ethereal Mount", "", &[]).action(230),
    // RevealOnCast => false, ClearHandsOnCast => false
    reveal_on_cast: false,
    clear_hands_on_cast: false,
    cast_delay_fast_scalar: 0.0,
    cast_delay_base: |w, s| {
        let aos = w.core_aos();
        let base = if aos { 3.0 } else { 2.0 };
        let donation = spell_mount(w, s).is_some_and(|m| data(w, m).is_donation_item);
        let level = spell::caster(w, s)
            .and_then(|c| {
                w.resources
                    .get::<muo_items::clothing::special::RewardHooks>()
                    .and_then(|h| h.reward_level)
                    .map(|f| f(w, c))
            })
            .unwrap_or(0);
        TimeSpan::from_secs_f64(if donation && level < 3 {
            7.5 + base
        } else {
            base
        })
    },
    get_cast_recovery: |_, _| TimeSpan::ZERO,
    get_mana: |_, _| 0,
    consume_reagents: |_, _| true,
    check_fizzle: |_, _| true,
    check_disturb: |_, _, t: DisturbType, _, _| {
        t != DisturbType::EquipRequest && t != DisturbType::UseRequest
    },
    do_hurt_fizzle: |w, s| {
        if !stopped(w, s) {
            (SPELL_BASE.do_hurt_fizzle)(w, s);
        }
    },
    do_fizzle: |w, s| {
        if !stopped(w, s) {
            (SPELL_BASE.do_fizzle)(w, s);
        }
    },
    on_disturb: |w, s, _t, message| {
        if message
            && !stopped(w, s)
            && let Some(c) = spell::caster(w, s)
        {
            mx::msg(w, c, 1049455); // You have been disrupted while attempting to summon your ethereal mount!
        }
    },
    on_cast: |w, s| {
        if let (Some(m), Some(c)) = (spell_mount(w, s), spell::caster(w, s))
            && !ops::is_deleted(w, m)
            && data(w, m).rider.is_none()
            && validate(w, m, c)
        {
            set_rider(w, m, Some(c));
        }
        spell::finish_sequence(w, s);
    },
    ..SPELL_BASE
};

/// `EtherealMount.StopMounting(mob)`: `(mob.Spell as EtherealSpell)?.Stop()`.
pub fn stop_mounting(w: &mut World, mob: MobileId) {
    let Some(s) = w.mobile(mob).and_then(|m| m.spell) else {
        return;
    };
    if spell::spell_type(w, s).is_some_and(|t| std::ptr::eq(t, &ETHEREAL_SPELL)) {
        if let Some(sp) = spell::spells(w).get_mut(s) {
            sp.ext.insert(Stopped(true));
        }
        spell::disturb(w, s, DisturbType::Hurt, false, false);
    }
}

// ---- wiring ----

pub fn register(types: &mut TypeRegistry) {
    for t in [
        &ETHEREAL_MOUNT,
        &ETHEREAL_HORSE,
        &ETHEREAL_LLAMA,
        &ETHEREAL_OSTARD,
        &ETHEREAL_RIDGEBACK,
        &ETHEREAL_UNICORN,
        &ETHEREAL_BEETLE,
        &ETHEREAL_KIRIN,
        &ETHEREAL_SWAMP_DRAGON,
        &RIDEABLE_POLAR_BEAR,
        &ETHEREAL_CU_SIDHE,
        &ETHEREAL_HIRYU,
        &ETHEREAL_REPTALON,
        &CHARGER_OF_THE_FALLEN,
    ] {
        types.register_item(t, &[]);
    }
}

pub fn configure(w: &mut World) {
    let h = muo_content::hooks::hooks_mut(w);
    h.ethereal_mount_rider = Some(rider);
    h.ethereal_dismount = Some(|w, r| {
        // the rider's mount item on the Mount layer, if it is an ethereal one
        if let Some(i) =
            items::find_item_on_layer(w, r, Layer::Mount).filter(|&i| is_ethereal(w, i))
        {
            set_rider(w, i, None);
        }
    });
    h.ethereal_stop_mounting = Some(stop_mounting);
    // `IsEthereal(m, out pm)`: the player riding an EtherealMount or wearing a VirtualMountItem
    w.resources
        .get_mut::<muo_content::misc::stamina_system::StaminaHooks>()
        .ethereal_rider = Some(|w, e| {
        let EntityId::Item(i) = e else { return None };
        let r = if is_ethereal(w, i) {
            rider(w, i)
        } else {
            muo_content::creature::virtual_mount::rider(w, i)
        };
        r.filter(|&m| muo_content::player_mobile::data(w, m).is_some())
    });
}
