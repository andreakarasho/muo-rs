//! `BaseInstrument` and its seven instruments. State:
//! [`InstrumentData`] in `Item::ext`. [`configure`] sets `InstrumentHooks.ops`, the table Peacemaking, Discordance and
//! Provocation ask for the instrument a bard plays.

use std::collections::HashMap;

use muo_content::items::equip_hooks::CraftArgs;
use muo_content::items::slayer::{self, SlayerName};
use muo_content::skills::bard::{self, InstrumentHooks, InstrumentOps};
use muo_io::{GenericReader, GenericWriter, Result};
use muo_primitives::{DateTime, TimeSpan, utility};
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, LootType, base, deps, ops};
use muo_server::mobile::part2::effects;
use muo_server::packets::equipment::{EquipInfoAttribute, send_display_equipment_info};
use muo_server::property_list::PropertyList;
use muo_server::serialization_ext::ReadEntityExt;
use muo_server::target::{self, TargetFlags, Targeted};
use muo_server::type_registry::{Phase, TypeRegistry};

use super::{read_version, write_version};

/// `InstrumentQuality`.
pub const EXCEPTIONAL: i32 = 2;

/// `BaseInstrument`' fields.
#[derive(Clone, Debug)]
pub struct InstrumentData {
    pub replenishes_charges: bool,
    pub last_replenished: DateTime,
    pub crafter: Option<String>,
    pub quality: i32,
    pub slayer: i32,
    pub slayer2: i32,
    pub uses_remaining: i32,
    pub success_sound: i32,
    pub failure_sound: i32,
}

pub fn data(w: &World, id: ItemId) -> Option<&InstrumentData> {
    w.item(id)?.ext.get::<InstrumentData>()
}

pub fn data_mut(w: &mut World, id: ItemId) -> Option<&mut InstrumentData> {
    w.item_mut(id)?.ext.get_mut::<InstrumentData>()
}

/// `item is BaseInstrument`.
pub fn is_instrument(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.ty.is(&BASE_INSTRUMENT))
}

fn uor(w: &World) -> bool {
    muo_content::items::core_uor(w)
}

/// `InitMinUses` / `InitMaxUses`.
fn init_uses(w: &World) -> (i32, i32) {
    if uor(w) { (350, 450) } else { (100, 150) }
}

/// `InitMinUses` / `InitMaxUses` of the class of `id`: the classes that override both (`GwennosHarp`, `IolosLute`,
/// `FluteOfRenewal`) or the era's default.
fn init_uses_of(w: &World, id: ItemId) -> (i32, i32) {
    let mut t = w.item(id).map(|i| i.ty);
    while let Some(ty) = t {
        match ty.full_name {
            "Server.Items.GwennosHarp" | "Server.Items.IolosLute" => return (1600, 1600),
            "Server.Items.FluteOfRenewal" => return (300, 300),
            _ => {}
        }
        t = ty.parent;
    }
    init_uses(w)
}

/// `new BaseInstrument(itemID, wellSound, badlySound)` of class `ty`.
pub fn new_instrument(
    w: &mut World,
    ty: &'static ItemType,
    item_id: i32,
    well: i32,
    badly: i32,
) -> ItemId {
    let range = init_uses(w);
    new_instrument_uses(w, ty, item_id, well, badly, range)
}

/// `new BaseInstrument(..)` of a class that overrides `InitMinUses` / `InitMaxUses`.
pub fn new_instrument_uses(
    w: &mut World,
    ty: &'static ItemType,
    item_id: i32,
    well: i32,
    badly: i32,
    (lo, hi): (i32, i32),
) -> ItemId {
    let id = ops::new_item_with_id(w, ty, item_id);
    let uses = utility::random_min_max(&mut w.rng, lo, hi);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(InstrumentData {
            replenishes_charges: false,
            last_replenished: DateTime::MIN,
            crafter: None,
            quality: 0,
            slayer: 0,
            slayer2: 0,
            uses_remaining: uses,
            success_sound: well,
            failure_sound: badly,
        });
    }
    id
}

/// `CheckReplenishUses(invalidate)`.
pub fn check_replenish_uses(w: &mut World, id: ItemId, invalidate: bool) {
    let (_, max) = init_uses_of(w, id);
    let now = w.now;
    let Some(d) = data_mut(w, id) else { return };
    if !d.replenishes_charges || d.uses_remaining >= max {
        return;
    }
    let rate = TimeSpan::from_mins(5);
    if d.last_replenished + rate < now {
        let diff = now - d.last_replenished;
        d.uses_remaining = (d.uses_remaining + (diff.ticks() / rate.ticks()) as i32).min(max);
        d.last_replenished = now;
        if invalidate {
            ops::invalidate_properties(w, id);
        }
    }
}

/// `UsesRemaining` getter.
pub fn uses_remaining(w: &mut World, id: ItemId) -> i32 {
    check_replenish_uses(w, id, true);
    data(w, id).map_or(0, |d| d.uses_remaining)
}

/// `UsesRemaining = value`.
pub fn set_uses_remaining(w: &mut World, id: ItemId, v: i32) {
    if let Some(d) = data_mut(w, id) {
        d.uses_remaining = v;
    }
    ops::invalidate_properties(w, id);
}

fn uses_scalar(w: &World, id: ItemId) -> i32 {
    if data(w, id).is_some_and(|d| d.quality == EXCEPTIONAL) {
        200
    } else {
        100
    }
}

/// `Quality = value`: unscale, set, scale.
pub fn set_quality(w: &mut World, id: ItemId, v: i32) {
    let s = uses_scalar(w, id);
    let u = uses_remaining(w, id);
    set_uses_remaining(w, id, u * 100 / s);
    if let Some(d) = data_mut(w, id) {
        d.quality = v;
    }
    ops::invalidate_properties(w, id);
    let s = uses_scalar(w, id);
    let u = uses_remaining(w, id);
    set_uses_remaining(w, id, u * s / 100);
}

/// `OnCraft(..)`.
fn on_craft(w: &mut World, id: ItemId, a: &CraftArgs) -> i32 {
    set_quality(w, id, a.quality);
    if a.makers_mark {
        let name = muo_server::mobile::part1::props_b::raw_name(w, a.from);
        if let Some(d) = data_mut(w, id) {
            d.crafter = name;
        }
        ops::invalidate_properties(w, id);
    }
    let q = data(w, id).map_or(0, |d| d.quality);
    // Publish 16 change
    if muo_content::core::lbr(w) {
        if q == EXCEPTIONAL {
            let u = uses_remaining(w, id);
            set_uses_remaining(w, id, u + 100);
        }
    } else if q != 1 {
        let u = uses_remaining(w, id);
        set_uses_remaining(w, id, u + (f64::from(u) * f64::from(q - 1) * 0.1) as i32);
    }
    a.quality
}

fn craftable(w: &mut World, id: ItemId, a: &CraftArgs) -> Option<i32> {
    is_instrument(w, id).then(|| on_craft(w, id, a))
}

/// `ConsumeUse(from)`.
pub fn consume_use(w: &mut World, id: ItemId, from: Option<MobileId>) {
    let u = uses_remaining(w, id);
    if u > 1 {
        set_uses_remaining(w, id, u - 1);
    } else {
        if let Some(f) = from {
            deps::send_localized_message(w, f, 502079); // The instrument played its last tune.
        }
        base::delete(w, id);
    }
}

// ---- who plays what ----

#[derive(Default)]
struct Played(HashMap<MobileId, ItemId>);

/// `SetInstrument(from, item)`.
pub fn set_instrument(w: &mut World, from: MobileId, item: ItemId) {
    w.resources.get_mut::<Played>().0.insert(from, item);
}

/// `GetInstrument(from)`.
pub fn get_instrument(w: &mut World, from: MobileId) -> Option<ItemId> {
    let found = w.resources.get_mut::<Played>().0.get(&from).copied();
    let pack = w.mobile(from).and_then(|m| m.backpack);
    if let Some(i) = found
        && w.item(i).is_some_and(|x| !x.deleted())
        && pack.is_some_and(|p| ops::is_child_of(w, i, Some(EntityId::Item(p))))
    {
        return Some(i);
    }
    w.resources.get_mut::<Played>().0.remove(&from);
    None
}

/// `PickInstrument(from, callback)`.
pub fn pick_instrument(w: &mut World, from: MobileId, callback: fn(&mut World, MobileId, ItemId)) {
    if let Some(i) = get_instrument(w, from) {
        callback(w, from, i);
        return;
    }
    deps::send_localized_message(w, from, 500617); // What instrument shall you play?
    target::begin_target(w, from, 1, false, TargetFlags::NONE, move |w, from, t| {
        match t {
            Targeted::Entity(EntityId::Item(i)) if is_instrument(w, i) => {
                set_instrument(w, from, i);
                callback(w, from, i);
            }
            _ => deps::send_localized_message(w, from, 500619), // That is not a musical instrument.
        }
    });
}

/// `PlayInstrumentWell(from)`.
pub fn play_well(w: &mut World, id: ItemId, from: MobileId) {
    let s = data(w, id).map_or(0, |d| d.success_sound);
    effects::play_sound(w, from, s);
}

/// `PlayInstrumentBadly(from)`.
pub fn play_badly(w: &mut World, id: ItemId, from: MobileId) {
    let s = data(w, id).map_or(0, |d| d.failure_sound);
    effects::play_sound(w, from, s);
}

/// `GetDifficultyFor(targ)`.
pub fn difficulty_for(w: &mut World, id: ItemId, targ: MobileId) -> f64 {
    let mut val = bard::get_base_difficulty(w, targ);
    let Some(d) = data(w, id).cloned() else {
        return val;
    };
    if d.quality == EXCEPTIONAL {
        val -= 5.0; // 10%
    }
    for s in [d.slayer, d.slayer2] {
        if let Some((entry, group)) =
            SlayerName::from_i32(s).and_then(|n| slayer::entry_by_name(w, n))
        {
            if slayer::slays(w, entry, targ) {
                val -= 10.0; // 20%
            } else if slayer::opposition_super_slays(w, group, targ) {
                val += 10.0; // -20%
            }
        }
    }
    val
}

struct Ops;

impl InstrumentOps for Ops {
    fn pick_instrument(
        &self,
        w: &mut World,
        from: MobileId,
        callback: fn(&mut World, MobileId, ItemId),
    ) {
        pick_instrument(w, from, callback);
    }
    fn play_well(&self, w: &mut World, instrument: ItemId, from: MobileId) {
        play_well(w, instrument, from);
    }
    fn play_badly(&self, w: &mut World, instrument: ItemId, from: MobileId) {
        play_badly(w, instrument, from);
    }
    fn consume_use(&self, w: &mut World, instrument: ItemId, from: MobileId) {
        consume_use(w, instrument, Some(from));
    }
    fn difficulty_for(&self, w: &mut World, instrument: ItemId, targ: MobileId) -> f64 {
        difficulty_for(w, instrument, targ)
    }
}

static OPS: Ops = Ops;

// ---- the item's virtuals ----

fn get_properties(w: &World, id: ItemId, list: &mut PropertyList) {
    base::get_properties_base(w, id, list);
    let Some(d) = data(w, id) else { return };
    if let Some(c) = &d.crafter {
        list.add_str(1050043, c); // crafted by ~1_NAME~
    }
    if d.quality == EXCEPTIONAL {
        list.add(1060636); // exceptional
    }
    list.add_int(1060584, d.uses_remaining); // uses remaining: ~1_val~
    if d.replenishes_charges {
        list.add(1070928); // Replenish Charges
    }
    for s in [d.slayer, d.slayer2] {
        if let Some((entry, _)) = SlayerName::from_i32(s).and_then(|n| slayer::entry_by_name(w, n))
        {
            list.add(slayer::title(w, entry));
        }
    }
}

fn on_single_click(w: &mut World, id: ItemId, from: MobileId) {
    let Some(d) = data(w, id).cloned() else {
        return;
    };
    let mut attrs = Vec::new();
    let ty = w.item(id).map(|i| i.ty);
    if ty.is_some_and(|t| (t.display_loot_type)(w, id)) {
        match w.item(id).map(|i| i.loot_type) {
            Some(LootType::Blessed) => attrs.push(EquipInfoAttribute::new(1038021, -1)), // blessed
            Some(LootType::Cursed) => attrs.push(EquipInfoAttribute::new(1049643, -1)),  // cursed
            _ => {}
        }
    }
    if d.quality == EXCEPTIONAL {
        attrs.push(EquipInfoAttribute::new(1018305 - d.quality, -1));
    }
    if d.replenishes_charges {
        attrs.push(EquipInfoAttribute::new(1070928, -1)); // Replenish Charges
    }
    for s in [d.slayer, d.slayer2] {
        if let Some((entry, _)) = SlayerName::from_i32(s).and_then(|n| slayer::entry_by_name(w, n))
        {
            attrs.push(EquipInfoAttribute::new(slayer::title(w, entry), -1));
        }
    }
    let name = ops::name(w, id);
    let number = match &name {
        None => ty.map_or(0, |t| (t.label_number)(w, id)),
        Some(n) => {
            ops::label_to(w, id, from, n);
            1041000
        }
    };
    if attrs.is_empty() && d.crafter.is_none() && name.is_some() {
        return;
    }
    let serial = w.item(id).map(|i| i.serial);
    if let (Some(ns), Some(serial)) = (deps::net_state_of(w, from), serial) {
        send_display_equipment_info(w, ns, serial, number, d.crafter.as_deref(), false, &attrs);
    }
}

fn on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    let loc = ops::get_world_location(w, id);
    if !deps::mobile_in_range(w, from, loc, 1) {
        deps::send_localized_message(w, from, 500446); // That is too far away.
    } else if muo_server::mobile::part1::combat::begin_action(
        w,
        from,
        "Server.Items.BaseInstrument",
    ) {
        set_instrument(w, from, id);
        // Delay of 6 second before being able to play another instrument again
        muo_server::timer::start(w, TimeSpan::from_secs(6), TimeSpan::ZERO, 1, move |w| {
            muo_server::mobile::part1::combat::end_action(w, from, "Server.Items.BaseInstrument");
        });
        if bard::check_musicianship(w, from) {
            play_well(w, id, from);
        } else {
            play_badly(w, id, from);
        }
    } else {
        deps::send_localized_message(w, from, 500119); // You must wait to perform another action
    }
}

fn serialize(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    (ITEM_BASE.serialize)(w, id, out);
    out.write_i32(4);
    let Some(d) = data(w, id) else { return };
    out.write_bool(d.replenishes_charges);
    out.write_date_time_ticks(d.last_replenished.ticks());
    out.write_opt_string(d.crafter.as_deref());
    out.write_enum(4, i64::from(d.quality));
    out.write_enum(4, i64::from(d.slayer));
    out.write_enum(4, i64::from(d.slayer2));
    out.write_i32(d.uses_remaining);
    out.write_i32(d.success_sound);
    out.write_i32(d.failure_sound);
}

fn deserialize(w: &mut World, id: ItemId, r: &mut dyn GenericReader) -> Result<()> {
    (ITEM_BASE.deserialize)(w, id, r)?;
    let version = r.read_i32()?;
    let d = if version >= 4 {
        let replenishes_charges = r.read_bool()?;
        let last_replenished = DateTime::from_ticks(r.read_date_time_ticks()?);
        InstrumentData {
            replenishes_charges,
            last_replenished,
            crafter: r.read_string()?,
            quality: r.read_enum(4)? as i32,
            slayer: r.read_enum(4)? as i32,
            slayer2: r.read_enum(4)? as i32,
            uses_remaining: r.read_i32()?,
            success_sound: r.read_i32()?,
            failure_sound: r.read_i32()?,
        }
    } else {
        // Deserialize(reader, version): encoded ints, and the crafter was a Mobile.
        let replenishes_charges = r.read_bool()?;
        let last_replenished = if replenishes_charges {
            DateTime::from_ticks(r.read_date_time_ticks()?)
        } else {
            DateTime::MIN
        };
        let crafter = r
            .read_mobile(w)?
            .and_then(|m| muo_server::mobile::part1::props_b::raw_name(w, m));
        InstrumentData {
            replenishes_charges,
            last_replenished,
            crafter,
            quality: r.read_encoded_int()?,
            slayer: r.read_encoded_int()?,
            slayer2: r.read_encoded_int()?,
            uses_remaining: r.read_encoded_int()?,
            success_sound: r.read_encoded_int()?,
            failure_sound: r.read_encoded_int()?,
        }
    };
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(d);
    }
    check_replenish_uses(w, id, false);
    Ok(())
}

/// `Server.Items.BaseInstrument` (abstract).
pub static BASE_INSTRUMENT: ItemType = ItemType {
    full_name: "Server.Items.BaseInstrument",
    parent: Some(&ITEM_BASE),
    construct: None,
    get_properties,
    on_single_click,
    on_double_click,
    serialize,
    deserialize,
    ..ITEM_BASE
};

macro_rules! instruments {
    ($($name:ident, $new:ident = $full:literal, gid $gid:expr, $well:expr, $badly:expr, weight $weight:expr;)*) => {$(
        #[doc = concat!("`new ", $full, "()`.")]
        pub fn $new(w: &mut World) -> ItemId {
            new_instrument(w, &$name, $gid, $well, $badly)
        }
        #[doc = concat!("`", $full, "`.")]
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&BASE_INSTRUMENT),
            construct: Some($new),
            default_weight: |_, _| $weight,
            serialize: |w, id, out| {
                (BASE_INSTRUMENT.serialize)(w, id, out);
                write_version(out, false);
            },
            deserialize: |w, id, r| {
                (BASE_INSTRUMENT.deserialize)(w, id, r)?;
                read_version(r, false)?;
                Ok(())
            },
            ..BASE_INSTRUMENT
        };
    )*};
}

instruments! {
    HARP, new_harp = "Server.Items.Harp", gid 0xEB1, 0x43, 0x44, weight 35.0;
    DRUMS, new_drums = "Server.Items.Drums", gid 0xE9C, 0x38, 0x39, weight 4.0;
    BAMBOO_FLUTE, new_bamboo_flute = "Server.Items.BambooFlute", gid 0x2805, 0x504, 0x503, weight 2.0;
    LUTE, new_lute = "Server.Items.Lute", gid 0xEB3, 0x4C, 0x4D, weight 5.0;
    LAP_HARP, new_lap_harp = "Server.Items.LapHarp", gid 0xEB2, 0x45, 0x46, weight 10.0;
    TAMBOURINE, new_tambourine = "Server.Items.Tambourine", gid 0xE9D, 0x52, 0x53, weight 1.0;
    TAMBOURINE_TASSEL, new_tambourine_tassel = "Server.Items.TambourineTassel", gid 0xE9E, 0x52, 0x53, weight 1.0;
}

/// Every instrument class.
pub static ALL: &[&ItemType] = &[
    &HARP,
    &DRUMS,
    &BAMBOO_FLUTE,
    &LUTE,
    &LAP_HARP,
    &TAMBOURINE,
    &TAMBOURINE_TASSEL,
];

fn configure(w: &mut World) {
    w.resources.get_mut::<InstrumentHooks>().ops = Some(&OPS);
    muo_content::craft::hooks_mut(w).craftables.push(craftable);
}

/// Registers the instruments.
pub fn register(types: &mut TypeRegistry) {
    types.register_item(&BASE_INSTRUMENT, &[]);
    for t in ALL {
        types.register_item(t, &[]);
    }
    types.register_phase_default(
        Phase::Configure,
        "Server.Items.BaseInstrument.Configure",
        configure,
    );
}
