//! The addons that have behavior: SpinningWheel/ElvenSpinningWheel (East, South),
//! LoomEast/South, FlourMillEast/South, WaterTroughEast/South, RejuvinationAnkhs, DartBoard, PickpocketDips,
//! TrainingDummies and ArcheryButte, with their deeds.
//!
//! State outside the serializer (timers, `IsWorking`, score tables) lives in `Ext` structs next to `AddonData` /
//! `ComponentData`; a loaded item has none until used, so every reader defaults a missing struct.
//!
//! `ISpinningWheel` / `ILoom` / `IFlourMill` are consumed by Cotton/Flax/Wool (`resources/cloth.rs`), the cloth
//! materials and the wheat sheaf (`food/cooking.rs`): [`spinning_wheel_begin_spin`] and friends are the interface.

use muo_content::items::addon::{
    self, ADDON_COMPONENT, ADDON_COMPONENT_CLASS, AddonClass, BASE_ADDON, BASE_ADDON_CLASS,
    BASE_ADDON_DEED, Comp, ComponentClass, DeedClass,
};
use muo_content::items::weapon;
use muo_content::mx;
use muo_io::{GenericReader, GenericWriter};
use muo_primitives::{Direction, Point3D, TimeSpan, utility};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ItemType, deps, messages, ops};
use muo_server::mobile::part1::{combat, props_b};
use muo_server::packets::message::MessageType;
use muo_server::skills::SkillName;
use muo_server::timer::{self, TimerHandle};
use muo_server::type_registry::TypeRegistry;

use crate::simple::{item_id, leaf};

// ---- shared pieces ------------------------------------------------------------------------------------------

/// `ISpinningWheel`'s `SpinCallback(sender, from, hue)`.
pub type SpinCallback = fn(&mut World, ItemId, MobileId, i32);

fn msg_to(w: &mut World, item: ItemId, to: MobileId, number: i32) {
    messages::send_localized_message_to(w, item, to, number, "");
}

fn overhead(w: &mut World, from: MobileId, number: i32) {
    deps::local_overhead_message(w, from, MessageType::REGULAR, 0x3B2, number);
}

fn public_msg(w: &mut World, item: ItemId, number: i32, args: &str) {
    messages::public_overhead_message_number(w, item, MessageType::REGULAR, 0x3B2, number, args);
}

fn cur_item_id(w: &World, id: ItemId) -> i32 {
    w.item(id).map_or(0, |i| i.item_id)
}

macro_rules! deed {
    ($ty:ident, $cls:ident, $full:literal, $ver:ident, $addon:path, $label:expr) => {
        static $cls: DeedClass = DeedClass {
            addon: |w, _| $addon(w),
            select_direction: None,
        };
        leaf! {
            #[doc = concat!("`", $full, "`.")]
            $ty = $full : BASE_ADDON_DEED, $ver;
            construct: Some(|w| addon::new_deed(w, &$ty)),
            family: Some(&$cls),
            label_number: |_, _| $label,
        }
    };
}

/// A save version that is an int (`[SerializationGenerator(0, false)]`) or an encoded int (`(0)`).
fn write_ver(out: &mut dyn GenericWriter, encoded: bool) {
    crate::simple::write_version(out, encoded);
}

fn read_ver(r: &mut dyn GenericReader, encoded: bool) -> muo_io::Result<i32> {
    crate::simple::read_version(r, encoded)
}

fn components_of(w: &World, a: ItemId) -> Vec<ItemId> {
    addon::components(w, a)
}

// ---- SpinningWheel ------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Default)]
struct SpinData {
    timer: Option<TimerHandle>,
}

/// The item ids a wheel steps through: `(spinning-on ids, step)`; a component with one of `idle` gets `step` added
/// when spinning starts and removed when it ends or loads.
struct WheelIds {
    idle: &'static [i32],
    spinning: &'static [i32],
    step: i32,
}

static WHEEL: WheelIds = WheelIds {
    idle: &[0x1015, 0x1019, 0x101C, 0x10A4],
    spinning: &[0x1016, 0x101A, 0x101D, 0x10A5],
    step: 1,
};
// The elven wheel counts the other way: 0x2E3D/0x2E3F turn into 0x2E3C/0x2E3E.
static ELVEN_WHEEL: WheelIds = WheelIds {
    idle: &[0x2E3D, 0x2E3F],
    spinning: &[0x2E3C, 0x2E3E],
    step: -1,
};

fn wheel_ids(w: &World, id: ItemId) -> &'static WheelIds {
    if w.item(id).is_some_and(|i| {
        i.ty.is(&ELVEN_SPINNING_WHEEL_EAST_ADDON) || i.ty.is(&ELVEN_SPINNING_WHEEL_SOUTH_ADDON)
    }) {
        &ELVEN_WHEEL
    } else {
        &WHEEL
    }
}

/// `ISpinningWheel.Spinning`.
pub fn spinning_wheel_spinning(w: &World, id: ItemId) -> bool {
    w.item(id)
        .and_then(|i| i.ext.get::<SpinData>())
        .is_some_and(|d| d.timer.is_some())
}

fn wheel_shift(w: &mut World, id: ItemId, from: &[i32], delta: i32) {
    for c in components_of(w, id) {
        if from.contains(&cur_item_id(w, c)) {
            item_id(w, c, cur_item_id(w, c) + delta);
        }
    }
}

/// `ISpinningWheel.BeginSpin(callback, from, hue)`.
pub fn spinning_wheel_begin_spin(
    w: &mut World,
    id: ItemId,
    callback: Option<SpinCallback>,
    from: MobileId,
    hue: i32,
) {
    let t = timer::start(
        w,
        TimeSpan::from_secs_f64(3.0),
        TimeSpan::ZERO,
        1,
        move |w| spinning_wheel_end_spin(w, id, callback, from, hue),
    );
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(SpinData { timer: Some(t) });
    }
    let ids = wheel_ids(w, id);
    wheel_shift(w, id, ids.idle, ids.step);
}

/// `EndSpin(callback, from, hue)`.
pub fn spinning_wheel_end_spin(
    w: &mut World,
    id: ItemId,
    callback: Option<SpinCallback>,
    from: MobileId,
    hue: i32,
) {
    if let Some(t) = w
        .item(id)
        .and_then(|i| i.ext.get::<SpinData>())
        .and_then(|d| d.timer)
    {
        timer::stop(w, t);
    }
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(SpinData { timer: None });
    }
    let ids = wheel_ids(w, id);
    wheel_shift(w, id, ids.spinning, -ids.step);
    if let Some(cb) = callback {
        cb(w, id, from, hue);
    }
}

/// `OnComponentLoaded(c)`: a component saved mid-spin is put back.
fn wheel_on_component_loaded(w: &mut World, a: ItemId, c: ItemId) {
    let ids = wheel_ids(w, a);
    let cur = cur_item_id(w, c);
    if ids.spinning.contains(&cur) {
        item_id(w, c, cur - ids.step);
    }
}

macro_rules! wheel {
    ($addon:ident, $acls:ident, $anew:ident, $deed:ident, $dcls:ident, $afull:literal, $dfull:literal, $ver:ident,
     $comp:expr, $alabel:expr, $dlabel:expr) => {
        pub fn $anew(w: &mut World) -> ItemId {
            let id = addon::new_addon(w, &$addon);
            addon::add_components(w, id, &[(Comp::Plain($comp), 0, 0, 0)]);
            id
        }
        static $acls: AddonClass = AddonClass {
            deed: Some(|w| addon::new_deed(w, &$deed)),
            on_component_loaded: wheel_on_component_loaded,
            ..BASE_ADDON_CLASS
        };
        leaf! {
            #[doc = concat!("`", $afull, "`.")]
            $addon = $afull : BASE_ADDON, $ver;
            construct: Some($anew),
            family: Some(&$acls),
            label_number: |_, _| $alabel,
        }
        deed!($deed, $dcls, $dfull, $ver, $anew, $dlabel);
    };
}

wheel!(
    SPINNING_WHEEL_EAST_ADDON,
    SPINNING_WHEEL_EAST_ADDON_CLASS,
    spinning_wheel_east_addon_new,
    SPINNING_WHEEL_EAST_DEED,
    SPINNING_WHEEL_EAST_DEED_CLASS,
    "Server.Items.SpinningWheelEastAddon",
    "Server.Items.SpinningWheelEastDeed",
    v0,
    0x1019,
    1024117,
    1044341
);
wheel!(
    SPINNING_WHEEL_SOUTH_ADDON,
    SPINNING_WHEEL_SOUTH_ADDON_CLASS,
    spinning_wheel_south_addon_new,
    SPINNING_WHEEL_SOUTH_DEED,
    SPINNING_WHEEL_SOUTH_DEED_CLASS,
    "Server.Items.SpinningWheelSouthAddon",
    "Server.Items.SpinningWheelSouthDeed",
    v0,
    0x1015,
    1024117,
    1044342
);
wheel!(
    ELVEN_SPINNING_WHEEL_EAST_ADDON,
    ELVEN_SPINNING_WHEEL_EAST_ADDON_CLASS,
    elven_spinning_wheel_east_addon_new,
    ELVEN_SPINNING_WHEEL_EAST_DEED,
    ELVEN_SPINNING_WHEEL_EAST_DEED_CLASS,
    "Server.Items.ElvenSpinningWheelEastAddon",
    "Server.Items.ElvenSpinningWheelEastDeed",
    enc,
    0x2E3D,
    1031737,
    1073393
);
wheel!(
    ELVEN_SPINNING_WHEEL_SOUTH_ADDON,
    ELVEN_SPINNING_WHEEL_SOUTH_ADDON_CLASS,
    elven_spinning_wheel_south_addon_new,
    ELVEN_SPINNING_WHEEL_SOUTH_DEED,
    ELVEN_SPINNING_WHEEL_SOUTH_DEED_CLASS,
    "Server.Items.ElvenSpinningWheelSouthAddon",
    "Server.Items.ElvenSpinningWheelSouthDeed",
    enc,
    0x2E3F,
    1031737,
    1072878
);

// ---- Loom ---------------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Default)]
struct LoomData {
    phase: i32,
}

/// `ILoom.Phase`.
pub fn loom_phase(w: &World, id: ItemId) -> i32 {
    w.item(id)
        .and_then(|i| i.ext.get::<LoomData>())
        .map_or(0, |d| d.phase)
}

/// `ILoom.Phase = value`.
pub fn loom_set_phase(w: &mut World, id: ItemId, value: i32) {
    if loom_phase(w, id) == value {
        return;
    }
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(LoomData { phase: value });
    }
}

macro_rules! loom {
    ($addon:ident, $acls:ident, $anew:ident, $deed:ident, $dcls:ident, $afull:literal, $dfull:literal, $rows:expr, $dlabel:expr) => {
        pub fn $anew(w: &mut World) -> ItemId {
            let id = addon::new_addon(w, &$addon);
            if let Some(i) = w.item_mut(id) {
                i.ext.insert(LoomData::default());
            }
            addon::add_components(w, id, &$rows);
            id
        }
        static $acls: AddonClass = AddonClass {
            deed: Some(|w| addon::new_deed(w, &$deed)),
            ..BASE_ADDON_CLASS
        };
        #[doc = concat!("`", $afull, "`.")]
        pub static $addon: ItemType = ItemType {
            full_name: $afull,
            parent: Some(&BASE_ADDON),
            construct: Some($anew),
            family: Some(&$acls),
            serialize: |w, id, out| {
                (BASE_ADDON.serialize)(w, id, out);
                write_ver(out, false);
                out.write_i32(loom_phase(w, id));
            },
            deserialize: |w, id, r| {
                (BASE_ADDON.deserialize)(w, id, r)?;
                read_ver(r, false)?;
                let phase = r.read_i32()?;
                if let Some(i) = w.item_mut(id) {
                    i.ext.insert(LoomData { phase });
                }
                Ok(())
            },
            ..BASE_ADDON
        };
        deed!($deed, $dcls, $dfull, v0, $anew, $dlabel);
    };
}

loom!(
    LOOM_EAST_ADDON,
    LOOM_EAST_ADDON_CLASS,
    loom_east_addon_new,
    LOOM_EAST_DEED,
    LOOM_EAST_DEED_CLASS,
    "Server.Items.LoomEastAddon",
    "Server.Items.LoomEastDeed",
    [
        (Comp::Plain(0x1060), 0, 0, 0),
        (Comp::Plain(0x105F), 0, 1, 0)
    ],
    1044343
);
loom!(
    LOOM_SOUTH_ADDON,
    LOOM_SOUTH_ADDON_CLASS,
    loom_south_addon_new,
    LOOM_SOUTH_DEED,
    LOOM_SOUTH_DEED_CLASS,
    "Server.Items.LoomSouthAddon",
    "Server.Items.LoomSouthDeed",
    [
        (Comp::Plain(0x1061), 0, 0, 0),
        (Comp::Plain(0x1062), 1, 0, 0)
    ],
    1044344
);

// ---- FlourMill ----------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Default)]
pub struct MillData {
    pub cur_flour: i32,
    pub working: bool,
}

/// `IFlourMill.MaxFlour`.
pub const MILL_MAX_FLOUR: i32 = 2;

static MILL_EAST_TABLE: [[i32; 3]; 3] = [
    [0x1920, 0x1921, 0x1925],
    [0x1922, 0x1923, 0x1926],
    [0x1924, 0x1924, 0x1928],
];
static MILL_SOUTH_TABLE: [[i32; 3]; 3] = [
    [0x192C, 0x192D, 0x1931],
    [0x192E, 0x192F, 0x1932],
    [0x1930, 0x1930, 0x1934],
];

fn mill(w: &World, id: ItemId) -> MillData {
    w.item(id)
        .and_then(|i| i.ext.get::<MillData>())
        .copied()
        .unwrap_or_default()
}

fn mill_mut(w: &mut World, id: ItemId) -> Option<&mut MillData> {
    let i = w.item_mut(id)?;
    if i.ext.get::<MillData>().is_none() {
        i.ext.insert(MillData::default());
    }
    i.ext.get_mut::<MillData>()
}

fn mill_table(w: &World, id: ItemId) -> &'static [[i32; 3]; 3] {
    if w.item(id).is_some_and(|i| i.ty.is(&FLOUR_MILL_SOUTH_ADDON)) {
        &MILL_SOUTH_TABLE
    } else {
        &MILL_EAST_TABLE
    }
}

/// `item is IFlourMill`.
pub fn is_flour_mill(w: &World, id: ItemId) -> bool {
    w.item(id)
        .is_some_and(|i| i.ty.is(&FLOUR_MILL_EAST_ADDON) || i.ty.is(&FLOUR_MILL_SOUTH_ADDON))
}

/// `item is ISpinningWheel`.
pub fn is_spinning_wheel(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| {
        [
            &SPINNING_WHEEL_EAST_ADDON,
            &SPINNING_WHEEL_SOUTH_ADDON,
            &ELVEN_SPINNING_WHEEL_EAST_ADDON,
            &ELVEN_SPINNING_WHEEL_SOUTH_ADDON,
        ]
        .iter()
        .any(|t| i.ty.is(t))
    })
}

/// `item is ILoom`.
pub fn is_loom(w: &World, id: ItemId) -> bool {
    w.item(id)
        .is_some_and(|i| i.ty.is(&LOOM_EAST_ADDON) || i.ty.is(&LOOM_SOUTH_ADDON))
}

/// `IFlourMill.CurFlour`.
pub fn flour_mill_cur_flour(w: &World, id: ItemId) -> i32 {
    mill(w, id).cur_flour
}

/// `CurFlour = value`: clamped to `0..=MaxFlour`; `OnCurFlourChanged` updates the stage.
pub fn flour_mill_set_cur_flour(w: &mut World, id: ItemId, value: i32) {
    let value = value.clamp(0, MILL_MAX_FLOUR);
    let changed = match mill_mut(w, id) {
        Some(d) if d.cur_flour != value => {
            d.cur_flour = value;
            true
        }
        _ => false,
    };
    if changed {
        flour_mill_update_stage(w, id);
    }
}

/// `UpdateStage()`.
pub fn flour_mill_update_stage(w: &mut World, id: ItemId) {
    let d = mill(w, id);
    let stage = if d.working {
        2
    } else if d.cur_flour > 0 {
        1
    } else {
        0
    };
    let table = mill_table(w, id);
    for c in components_of(w, id) {
        let cur = cur_item_id(w, c);
        if let Some(row) = table.iter().find(|row| row.contains(&cur)) {
            item_id(w, c, row[stage]);
        }
    }
}

fn flour_mill_start_working(w: &mut World, id: ItemId, from: MobileId) {
    if mill(w, id).working {
        return;
    }
    timer::start(
        w,
        TimeSpan::from_secs_f64(5.0),
        TimeSpan::ZERO,
        1,
        move |w| flour_mill_finish(w, id, from),
    );
    if let Some(d) = mill_mut(w, id) {
        d.working = true;
    }
    flour_mill_update_stage(w, id);
}

fn flour_mill_finish(w: &mut World, id: ItemId, from: MobileId) {
    if let Some(d) = mill_mut(w, id) {
        d.working = false;
    }
    let full = mill(w, id).cur_flour >= MILL_MAX_FLOUR;
    if !mx::deleted(w, from) && !ops::is_deleted(w, id) && full {
        // `new SackFlour { ItemID = 4153 or 4165 }`
        let flour = muo_content::factory::make_item(w, "Server.Items.SackFlour");
        let graphic = if utility::random_bool(&mut w.rng) {
            4153
        } else {
            4165
        };
        if let Some(f) = flour {
            item_id(w, f, graphic);
            if muo_server::mobile::part2::items::place_in_backpack(w, from, f) {
                if let Some(d) = mill_mut(w, id) {
                    d.cur_flour = 0;
                }
            } else {
                if let Some(ty) = w.item(f).map(|i| i.ty) {
                    (ty.delete)(w, f);
                }
                mx::msg(w, from, 500998); // There is not enough room in your backpack!  You stop grinding.
            }
        }
    }
    flour_mill_update_stage(w, id);
}

fn flour_mill_on_component_used(w: &mut World, a: ItemId, _c: ItemId, from: MobileId) {
    let loc = ops::get_world_location(w, a);
    if !mx::in_range_point(w, from, loc, 4) || !deps::in_los_item(w, from, a) {
        overhead(w, from, 1019045); // I can't reach that.
    } else if mill(w, a).cur_flour < MILL_MAX_FLOUR {
        mx::msg(w, from, 500997); // You need more wheat to make a sack of flour.
    } else {
        flour_mill_start_working(w, a, from);
    }
}

macro_rules! flour_mill {
    ($addon:ident, $acls:ident, $anew:ident, $deed:ident, $dcls:ident, $afull:literal, $dfull:literal, $rows:expr, $dlabel:expr) => {
        pub fn $anew(w: &mut World) -> ItemId {
            let id = addon::new_addon(w, &$addon);
            if let Some(i) = w.item_mut(id) {
                i.ext.insert(MillData::default());
            }
            addon::add_components(w, id, &$rows);
            id
        }
        static $acls: AddonClass = AddonClass {
            deed: Some(|w| addon::new_deed(w, &$deed)),
            on_component_used: flour_mill_on_component_used,
            ..BASE_ADDON_CLASS
        };
        #[doc = concat!("`", $afull, "`.")]
        pub static $addon: ItemType = ItemType {
            full_name: $afull,
            parent: Some(&BASE_ADDON),
            construct: Some($anew),
            family: Some(&$acls),
            serialize: |w, id, out| {
                (BASE_ADDON.serialize)(w, id, out);
                write_ver(out, false);
                out.write_i32(mill(w, id).cur_flour);
            },
            deserialize: |w, id, r| {
                (BASE_ADDON.deserialize)(w, id, r)?;
                read_ver(r, false)?;
                let cur_flour = r.read_i32()?;
                if let Some(i) = w.item_mut(id) {
                    i.ext.insert(MillData {
                        cur_flour,
                        working: false,
                    });
                }
                flour_mill_update_stage(w, id); // [AfterDeserialization]
                Ok(())
            },
            ..BASE_ADDON
        };
        deed!($deed, $dcls, $dfull, v0, $anew, $dlabel);
    };
}

flour_mill!(
    FLOUR_MILL_EAST_ADDON,
    FLOUR_MILL_EAST_ADDON_CLASS,
    flour_mill_east_addon_new,
    FLOUR_MILL_EAST_DEED,
    FLOUR_MILL_EAST_DEED_CLASS,
    "Server.Items.FlourMillEastAddon",
    "Server.Items.FlourMillEastDeed",
    [
        (Comp::Plain(0x1920), -1, 0, 0),
        (Comp::Plain(0x1922), 0, 0, 0),
        (Comp::Plain(0x1924), 1, 0, 0)
    ],
    1044347
);
flour_mill!(
    FLOUR_MILL_SOUTH_ADDON,
    FLOUR_MILL_SOUTH_ADDON_CLASS,
    flour_mill_south_addon_new,
    FLOUR_MILL_SOUTH_DEED,
    FLOUR_MILL_SOUTH_DEED_CLASS,
    "Server.Items.FlourMillSouthAddon",
    "Server.Items.FlourMillSouthDeed",
    [
        (Comp::Plain(0x192C), 0, -1, 0),
        (Comp::Plain(0x192E), 0, 0, 0),
        (Comp::Plain(0x1930), 0, 1, 0)
    ],
    1044348
);

// ---- WaterTrough --------------------------------------------------------------------------------------------

/// `IHasQuantity.Quantity` of a trough: always 500 (`set` does nothing).
pub const WATER_TROUGH_QUANTITY: i32 = 500;

macro_rules! trough {
    ($addon:ident, $acls:ident, $anew:ident, $deed:ident, $dcls:ident, $afull:literal, $dfull:literal, $rows:expr, $dlabel:expr) => {
        pub fn $anew(w: &mut World) -> ItemId {
            let id = addon::new_addon(w, &$addon);
            addon::add_components(w, id, &$rows);
            id
        }
        static $acls: AddonClass = AddonClass {
            deed: Some(|w| addon::new_deed(w, &$deed)),
            ..BASE_ADDON_CLASS
        };
        leaf! {
            #[doc = concat!("`", $afull, "`.")]
            $addon = $afull : BASE_ADDON, v0;
            construct: Some($anew),
            family: Some(&$acls),
        }
        deed!($deed, $dcls, $dfull, v0, $anew, $dlabel);
    };
}

trough!(
    WATER_TROUGH_EAST_ADDON,
    WATER_TROUGH_EAST_ADDON_CLASS,
    water_trough_east_addon_new,
    WATER_TROUGH_EAST_DEED,
    WATER_TROUGH_EAST_DEED_CLASS,
    "Server.Items.WaterTroughEastAddon",
    "Server.Items.WaterTroughEastDeed",
    [(Comp::Plain(0xB41), 0, 0, 0), (Comp::Plain(0xB42), 0, 1, 0)],
    1044349
);
trough!(
    WATER_TROUGH_SOUTH_ADDON,
    WATER_TROUGH_SOUTH_ADDON_CLASS,
    water_trough_south_addon_new,
    WATER_TROUGH_SOUTH_DEED,
    WATER_TROUGH_SOUTH_DEED_CLASS,
    "Server.Items.WaterTroughSouthAddon",
    "Server.Items.WaterTroughSouthDeed",
    [(Comp::Plain(0xB43), 0, 0, 0), (Comp::Plain(0xB44), 1, 0, 0)],
    1044350
);

// ---- RejuvinationAnkhs --------------------------------------------------------------------------------------

#[derive(Clone, Copy, Default)]
struct AnkhData {
    next_message: Option<muo_primitives::DateTime>,
}

fn rejuvination_release(w: &mut World, comp: ItemId, from: MobileId, random: i32) {
    combat::end_action(w, from, "RejuvinationAddonComponent");
    if random == 4 {
        let (h, m, s) = (
            mx::hits_max(w, from),
            muo_server::mobile::base::mana_max(w, from),
            muo_server::mobile::base::stam_max(w, from),
        );
        props_b::set_hits(w, from, h);
        props_b::set_mana(w, from, m);
        props_b::set_stam(w, from, s);
        msg_to(w, comp, from, 500807); // You feel completely rejuvinated!
    }
}

fn rejuvination_on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    if !combat::begin_action(w, from, "RejuvinationAddonComponent") {
        return;
    }
    mx::fixed_effect(w, from, 0x373A, 1, 16);
    let random = utility::random_from(&mut w.rng, 1, 4);
    if random == 1 || random == 4 {
        let h = mx::hits_max(w, from);
        props_b::set_hits(w, from, h);
        msg_to(w, id, from, 500801); // A sense of warmth fills your body!
    } else if random == 2 || random == 4 {
        let m = muo_server::mobile::base::mana_max(w, from);
        props_b::set_mana(w, from, m);
        msg_to(w, id, from, 500802); // A feeling of power surges through your veins!
    } else if random == 3 || random == 4 {
        let s = muo_server::mobile::base::stam_max(w, from);
        props_b::set_stam(w, from, s);
        msg_to(w, id, from, 500803); // You feel as though you've slept for days!
    }
    timer::start(w, TimeSpan::from_hours(2), TimeSpan::ZERO, 1, move |w| {
        rejuvination_release(w, id, from, random)
    });
}

/// `RejuvinationAddonComponent(itemID)`.
pub fn rejuvination_addon_component_new(w: &mut World, item_id: i32) -> ItemId {
    addon::new_component(w, &REJUVINATION_ADDON_COMPONENT, item_id)
}

leaf! {
    /// `Server.Items.RejuvinationAddonComponent`.
    REJUVINATION_ADDON_COMPONENT = "Server.Items.RejuvinationAddonComponent" : ADDON_COMPONENT, v0;
    on_double_click: rejuvination_on_double_click,
}

fn ankh_on_movement(w: &mut World, id: ItemId, m: MobileId, old: Point3D) {
    let loc = w.item(id).map_or_else(Point3D::default, |i| i.location);
    let (player, mloc) = (mx::player(w, m), mx::location(w, m));
    let now = w.now;
    let next = w
        .item(id)
        .and_then(|i| i.ext.get::<AnkhData>())
        .and_then(|d| d.next_message);
    if player
        && utility::in_range(loc.x, loc.y, mloc.x, mloc.y, 3)
        && !utility::in_range(loc.x, loc.y, old.x, old.y, 3)
        && next.is_none_or(|n| now >= n)
    {
        if let Some(c) = components_of(w, id).first().copied() {
            msg_to(w, c, m, 1010061); // An overwhelming sense of peace fills you.
        }
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(AnkhData {
                next_message: Some(now + TimeSpan::from_secs_f64(25.0)),
            });
        }
    }
}

/// `Server.Items.BaseRejuvinationAnkh` (abstract).
pub static BASE_REJUVINATION_ANKH: ItemType = ItemType {
    full_name: "Server.Items.BaseRejuvinationAnkh",
    parent: Some(&BASE_ADDON),
    construct: None,
    handles_on_movement: |_, _| true,
    on_movement: ankh_on_movement,
    serialize: |w, id, out| {
        (BASE_ADDON.serialize)(w, id, out);
        write_ver(out, false);
    },
    deserialize: |w, id, r| {
        (BASE_ADDON.deserialize)(w, id, r)?;
        read_ver(r, false)?;
        Ok(())
    },
    ..BASE_ADDON
};

/// `RejuvinationAnkhWest()`.
pub fn rejuvination_ankh_west_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &REJUVINATION_ANKH_WEST);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Typed(rejuvination_addon_component_new, 0x3), 0, 0, 0),
            (Comp::Typed(rejuvination_addon_component_new, 0x2), 0, 1, 0),
        ],
    );
    id
}

/// `RejuvinationAnkhNorth()`.
pub fn rejuvination_ankh_north_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &REJUVINATION_ANKH_NORTH);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Typed(rejuvination_addon_component_new, 0x4), 0, 0, 0),
            (Comp::Typed(rejuvination_addon_component_new, 0x5), 1, 0, 0),
        ],
    );
    id
}

leaf! {
    /// `Server.Items.RejuvinationAnkhWest`.
    REJUVINATION_ANKH_WEST = "Server.Items.RejuvinationAnkhWest" : BASE_REJUVINATION_ANKH, v0;
    construct: Some(rejuvination_ankh_west_new),
}

leaf! {
    /// `Server.Items.RejuvinationAnkhNorth`.
    REJUVINATION_ANKH_NORTH = "Server.Items.RejuvinationAnkhNorth" : BASE_REJUVINATION_ANKH, v0;
    construct: Some(rejuvination_ankh_north_new),
}

// ---- DartBoard ----------------------------------------------------------------------------------------------

fn dart_board_east(w: &World, id: ItemId) -> bool {
    cur_item_id(w, id) == 0x1E2F
}

static DART_BOARD_CLASS: ComponentClass = ComponentClass {
    needs_wall: true,
    wall_position_of: Some(|w, id| {
        if dart_board_east(w, id) {
            Point3D::new(-1, 0, 0)
        } else {
            Point3D::new(0, -1, 0)
        }
    }),
    ..ADDON_COMPONENT_CLASS
};

fn dart_board_throw(w: &mut World, id: ItemId, from: MobileId) {
    let knife = muo_content::items::weapon_hooks::weapon_item(w, from)
        .filter(|k| w.item(*k).is_some_and(|i| i.ty.is(&weapon::BASE_KNIFE)));
    let Some(knife) = knife else {
        overhead(w, from, 500751); // Try holding a knife...
        return;
    };
    let action = if props_b::mounted(w, from) { 26 } else { 9 };
    mx::animate(w, from, action, 7, 1, true, false, 0);
    let graphic = cur_item_id(w, knife);
    muo_server::mobile::part2::effects::moving_effect_plain(
        w,
        from,
        EntityId::Item(id),
        graphic,
        7,
        1,
        false,
        false,
    );
    mx::play_sound(w, from, 0x238);
    let rand = utility::random_double(&mut w.rng);
    let message = if rand < 0.05 {
        500752
    } else if rand < 0.20 {
        500753
    } else if rand < 0.45 {
        500754
    } else if rand < 0.70 {
        500755
    } else if rand < 0.85 {
        500756
    } else {
        500757
    };
    public_msg(w, id, message, "");
}

fn dart_board_on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    let loc = w.item(id).map_or_else(Point3D::default, |i| i.location);
    let east = dart_board_east(w, id);
    let dir = if mx::location(w, from) != loc {
        muo_server::mobile::part2::effects::get_direction_to(w, from, loc.x, loc.y, false)
    } else if east {
        Direction::WEST
    } else {
        Direction::NORTH
    };
    mx::set_direction(w, from, dir);

    let can_throw = if !mx::in_range_point(w, from, loc, 4) || !deps::in_los_item(w, from, id) {
        false
    } else if east {
        dir == Direction::LEFT || dir == Direction::WEST || dir == Direction::UP
    } else {
        dir == Direction::UP || dir == Direction::NORTH || dir == Direction::RIGHT
    };
    if can_throw {
        dart_board_throw(w, id, from);
    } else {
        overhead(w, from, 1019045); // I can't reach that.
    }
}

/// `DartBoard(east)`.
pub fn dart_board_new(w: &mut World, east: bool) -> ItemId {
    addon::new_component(w, &DART_BOARD, if east { 0x1E2F } else { 0x1E2E })
}

leaf! {
    /// `Server.Items.DartBoard`.
    DART_BOARD = "Server.Items.DartBoard" : ADDON_COMPONENT, enc;
    construct: Some(|w| dart_board_new(w, true)),
    family: Some(&DART_BOARD_CLASS),
    on_double_click: dart_board_on_double_click,
}

/// `DartBoardEastAddon()`.
pub fn dart_board_east_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &DART_BOARD_EAST_ADDON);
    let c = dart_board_new(w, true);
    addon::add_component(w, id, c, 0, 0, 0);
    id
}

/// `DartBoardSouthAddon()`.
pub fn dart_board_south_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &DART_BOARD_SOUTH_ADDON);
    let c = dart_board_new(w, false);
    addon::add_component(w, id, c, 0, 0, 0);
    id
}

static DART_BOARD_EAST_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(|w| addon::new_deed(w, &DART_BOARD_EAST_DEED)),
    ..BASE_ADDON_CLASS
};
static DART_BOARD_SOUTH_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(|w| addon::new_deed(w, &DART_BOARD_SOUTH_DEED)),
    ..BASE_ADDON_CLASS
};

// The addons have no [Constructible] constructor (`DartBoardEastAddon()` is not marked).
leaf! {
    /// `Server.Items.DartBoardEastAddon`.
    DART_BOARD_EAST_ADDON = "Server.Items.DartBoardEastAddon" : BASE_ADDON, enc;
    family: Some(&DART_BOARD_EAST_ADDON_CLASS),
}
leaf! {
    /// `Server.Items.DartBoardSouthAddon`.
    DART_BOARD_SOUTH_ADDON = "Server.Items.DartBoardSouthAddon" : BASE_ADDON, enc;
    family: Some(&DART_BOARD_SOUTH_ADDON_CLASS),
}
deed!(
    DART_BOARD_EAST_DEED,
    DART_BOARD_EAST_DEED_CLASS,
    "Server.Items.DartBoardEastDeed",
    enc,
    dart_board_east_addon_new,
    1044326
);
deed!(
    DART_BOARD_SOUTH_DEED,
    DART_BOARD_SOUTH_DEED_CLASS,
    "Server.Items.DartBoardSouthDeed",
    enc,
    dart_board_south_addon_new,
    1044325
);

// ---- PickpocketDip and TrainingDummy (the same shape: a swinging component with a skill window) ----------------

#[derive(Clone, Copy)]
pub struct PracticeData {
    pub min_skill: f64,
    pub max_skill: f64,
    pub timer: Option<TimerHandle>,
}

impl Default for PracticeData {
    fn default() -> Self {
        PracticeData {
            min_skill: -25.0,
            max_skill: 25.0,
            timer: None,
        }
    }
}

fn practice(w: &World, id: ItemId) -> PracticeData {
    w.item(id)
        .and_then(|i| i.ext.get::<PracticeData>())
        .copied()
        .unwrap_or_default()
}

fn practice_mut(w: &mut World, id: ItemId) -> Option<&mut PracticeData> {
    let i = w.item_mut(id)?;
    if i.ext.get::<PracticeData>().is_none() {
        i.ext.insert(PracticeData::default());
    }
    i.ext.get_mut::<PracticeData>()
}

fn practice_serialize(w: &World, id: ItemId, out: &mut dyn GenericWriter, parent: &ItemType) {
    (parent.serialize)(w, id, out);
    write_ver(out, false);
    let d = practice(w, id);
    out.write_f64(d.min_skill);
    out.write_f64(d.max_skill);
}

fn practice_deserialize(
    w: &mut World,
    id: ItemId,
    r: &mut dyn GenericReader,
    parent: &ItemType,
) -> muo_io::Result<()> {
    (parent.deserialize)(w, id, r)?;
    read_ver(r, false)?;
    let (min_skill, max_skill) = (r.read_f64()?, r.read_f64()?);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(PracticeData {
            min_skill,
            max_skill,
            timer: None,
        });
    }
    Ok(())
}

fn swinging(w: &World, id: ItemId) -> bool {
    practice(w, id).timer.is_some()
}

fn set_practice_timer(w: &mut World, id: ItemId, t: Option<TimerHandle>) {
    if let Some(d) = practice_mut(w, id) {
        d.timer = t;
    }
}

// PickpocketDip

fn dip_update_item_id(w: &mut World, id: ItemId) {
    let base = 0x1EC0 + (cur_item_id(w, id) - 0x1EC0) / 3 * 3;
    item_id(w, id, base + i32::from(swinging(w, id)));
}

fn dip_begin_swing(w: &mut World, id: ItemId) {
    if let Some(t) = practice(w, id).timer {
        timer::stop(w, t);
    }
    let t = timer::start(
        w,
        TimeSpan::from_secs_f64(3.0),
        TimeSpan::ZERO,
        1,
        move |w| dip_end_swing(w, id),
    );
    set_practice_timer(w, id, Some(t));
    dip_update_item_id(w, id);
}

fn dip_end_swing(w: &mut World, id: ItemId) {
    if let Some(t) = practice(w, id).timer {
        timer::stop(w, t);
    }
    set_practice_timer(w, id, None);
    dip_update_item_id(w, id);
}

fn dip_use(w: &mut World, id: ItemId, from: MobileId) {
    let loc = ops::get_world_location(w, id);
    let map = w.item(id).and_then(|i| i.map);
    let dir = muo_server::mobile::part2::effects::get_direction_to(w, from, loc.x, loc.y, false);
    mx::set_direction(w, from, dir);
    muo_server::effects::play_sound(w, loc, map, 0x4F);
    let d = practice(w, id);
    if mx::check_skill(w, from, SkillName::Stealing, d.min_skill, d.max_skill) {
        msg_to(w, id, from, 501834); // You successfully avoid disturbing the dip while searching it.
    } else {
        muo_server::effects::play_sound(w, loc, map, 0x390);
        dip_begin_swing(w, id);
        muo_server::item::base::process_delta(w, id);
        msg_to(w, id, from, 501831); // You carelessly bump the dip and start it swinging.
    }
}

fn dip_on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    let loc = ops::get_world_location(w, id);
    if !mx::in_range_point(w, from, loc, 1) {
        msg_to(w, id, from, 501816); // You are too far away to do that.
    } else if swinging(w, id) {
        msg_to(w, id, from, 501815); // You have to wait until it stops swinging.
    } else if muo_server::skills::base(w, from, SkillName::Stealing) >= practice(w, id).max_skill {
        msg_to(w, id, from, 501830); // Your ability to steal cannot improve any further by simply practicing on a dummy.
    } else if props_b::mounted(w, from) {
        msg_to(w, id, from, 501829); // You can't practice on this while on a mount.
    } else {
        dip_use(w, id, from);
    }
}

/// `PickpocketDip(itemID)`.
pub fn pickpocket_dip_new(w: &mut World, item_id: i32) -> ItemId {
    let id = addon::new_component(w, &PICKPOCKET_DIP, item_id);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(PracticeData::default());
    }
    id
}

/// `Server.Items.PickpocketDip` (`[Flippable(0x1EC0, 0x1EC3)]`).
pub static PICKPOCKET_DIP: ItemType = ItemType {
    full_name: "Server.Items.PickpocketDip",
    parent: Some(&ADDON_COMPONENT),
    construct: None,
    on_double_click: dip_on_double_click,
    serialize: |w, id, out| practice_serialize(w, id, out, &ADDON_COMPONENT),
    deserialize: |w, id, r| {
        practice_deserialize(w, id, r, &ADDON_COMPONENT)?;
        dip_update_item_id(w, id); // [AfterDeserialization]
        Ok(())
    },
    ..ADDON_COMPONENT
};

// TrainingDummy

fn dummy_update_item_id(w: &mut World, id: ItemId) {
    let base = cur_item_id(w, id) / 2 * 2;
    item_id(w, id, base + i32::from(swinging(w, id)));
}

fn dummy_end_swing(w: &mut World, id: ItemId) {
    if let Some(t) = practice(w, id).timer {
        timer::stop(w, t);
    }
    set_practice_timer(w, id, None);
    dummy_update_item_id(w, id);
}

fn dummy_on_hit(w: &mut World, id: ItemId) {
    dummy_update_item_id(w, id);
    let loc = ops::get_world_location(w, id);
    let map = w.item(id).and_then(|i| i.map);
    let sounds = [0x3A4, 0x3A6, 0x3A9, 0x3AE, 0x3B4, 0x3B6];
    let s = utility::random_element(&mut w.rng, &sounds)
        .copied()
        .unwrap_or(0x3A4);
    muo_server::effects::play_sound(w, loc, map, s);
}

fn dummy_begin_swing(w: &mut World, id: ItemId) {
    if let Some(t) = practice(w, id).timer {
        timer::stop(w, t);
    }
    // InternalTimer: first tick after 0.25s hits, the next (2.75s later) ends the swing, alternating.
    let mut delay = true;
    let t = timer::start(
        w,
        TimeSpan::from_secs_f64(0.25),
        TimeSpan::from_secs_f64(2.75),
        0,
        move |w| {
            if delay {
                dummy_on_hit(w, id);
            } else {
                dummy_end_swing(w, id);
            }
            delay = !delay;
        },
    );
    set_practice_timer(w, id, Some(t));
}

fn dummy_use(w: &mut World, id: ItemId, from: MobileId, weapon_item: ItemId) {
    dummy_begin_swing(w, id);
    let loc = ops::get_world_location(w, id);
    let dir = muo_server::mobile::part2::effects::get_direction_to(w, from, loc.x, loc.y, false);
    mx::set_direction(w, from, dir);
    muo_content::items::weapon_combat::play_swing_animation(w, weapon_item, from);
    let d = practice(w, id);
    mx::check_skill(
        w,
        from,
        weapon::skill(w, weapon_item),
        d.min_skill,
        d.max_skill,
    );
}

fn dummy_on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    let weapon_item = muo_content::items::weapon_hooks::weapon_item(w, from);
    let ranged =
        weapon_item.is_some_and(|k| w.item(k).is_some_and(|i| i.ty.is(&weapon::BASE_RANGED)));
    let loc = ops::get_world_location(w, id);
    if ranged {
        msg_to(w, id, from, 501822); // You can't practice ranged weapons on this.
        return;
    }
    let Some(k) =
        weapon_item.filter(|k| mx::in_range_point(w, from, loc, weapon::max_range(w, *k)))
    else {
        msg_to(w, id, from, 501816); // You are too far away to do that.
        return;
    };
    if swinging(w, id) {
        msg_to(w, id, from, 501815); // You have to wait until it stops swinging.
    } else if muo_server::skills::base(w, from, weapon::skill(w, k)) >= practice(w, id).max_skill {
        msg_to(w, id, from, 501828); // Your skill cannot improve any further by simply practicing with a dummy.
    } else if props_b::mounted(w, from) {
        msg_to(w, id, from, 501829); // You can't practice on this while on a mount.
    } else {
        dummy_use(w, id, from, k);
    }
}

/// `TrainingDummy(itemID)`.
pub fn training_dummy_new(w: &mut World, item_id: i32) -> ItemId {
    let id = addon::new_component(w, &TRAINING_DUMMY, item_id);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(PracticeData::default());
    }
    id
}

/// `Server.Items.TrainingDummy` (`[Flippable(0x1070, 0x1074)]`).
pub static TRAINING_DUMMY: ItemType = ItemType {
    full_name: "Server.Items.TrainingDummy",
    parent: Some(&ADDON_COMPONENT),
    construct: Some(|w| training_dummy_new(w, 0x1074)),
    on_double_click: dummy_on_double_click,
    serialize: |w, id, out| practice_serialize(w, id, out, &ADDON_COMPONENT),
    deserialize: |w, id, r| {
        practice_deserialize(w, id, r, &ADDON_COMPONENT)?;
        dummy_update_item_id(w, id); // [AfterDeserialization]
        Ok(())
    },
    ..ADDON_COMPONENT
};

macro_rules! practice_addon {
    ($addon:ident, $acls:ident, $anew:ident, $deed:ident, $dcls:ident, $afull:literal, $dfull:literal, $comp:expr, $dlabel:expr) => {
        pub fn $anew(w: &mut World) -> ItemId {
            let id = addon::new_addon(w, &$addon);
            let c = $comp(w);
            addon::add_component(w, id, c, 0, 0, 0);
            id
        }
        static $acls: AddonClass = AddonClass {
            deed: Some(|w| addon::new_deed(w, &$deed)),
            ..BASE_ADDON_CLASS
        };
        leaf! {
            #[doc = concat!("`", $afull, "`.")]
            $addon = $afull : BASE_ADDON, v0;
            construct: Some($anew),
            family: Some(&$acls),
        }
        deed!($deed, $dcls, $dfull, v0, $anew, $dlabel);
    };
}

practice_addon!(
    PICKPOCKET_DIP_EAST_ADDON,
    PICKPOCKET_DIP_EAST_ADDON_CLASS,
    pickpocket_dip_east_addon_new,
    PICKPOCKET_DIP_EAST_DEED,
    PICKPOCKET_DIP_EAST_DEED_CLASS,
    "Server.Items.PickpocketDipEastAddon",
    "Server.Items.PickpocketDipEastDeed",
    |w| pickpocket_dip_new(w, 0x1EC3),
    1044337
);
practice_addon!(
    PICKPOCKET_DIP_SOUTH_ADDON,
    PICKPOCKET_DIP_SOUTH_ADDON_CLASS,
    pickpocket_dip_south_addon_new,
    PICKPOCKET_DIP_SOUTH_DEED,
    PICKPOCKET_DIP_SOUTH_DEED_CLASS,
    "Server.Items.PickpocketDipSouthAddon",
    "Server.Items.PickpocketDipSouthDeed",
    |w| pickpocket_dip_new(w, 0x1EC0),
    1044338
);
practice_addon!(
    TRAINING_DUMMY_EAST_ADDON,
    TRAINING_DUMMY_EAST_ADDON_CLASS,
    training_dummy_east_addon_new,
    TRAINING_DUMMY_EAST_DEED,
    TRAINING_DUMMY_EAST_DEED_CLASS,
    "Server.Items.TrainingDummyEastAddon",
    "Server.Items.TrainingDummyEastDeed",
    |w| training_dummy_new(w, 0x1074),
    1044335
);
practice_addon!(
    TRAINING_DUMMY_SOUTH_ADDON,
    TRAINING_DUMMY_SOUTH_ADDON_CLASS,
    training_dummy_south_addon_new,
    TRAINING_DUMMY_SOUTH_DEED,
    TRAINING_DUMMY_SOUTH_DEED_CLASS,
    "Server.Items.TrainingDummySouthAddon",
    "Server.Items.TrainingDummySouthDeed",
    |w| training_dummy_new(w, 0x1070),
    1044336
);

// ---- ArcheryButte -------------------------------------------------------------------------------------------

#[derive(Clone, Default)]
pub struct ButteData {
    pub min_skill: f64,
    pub max_skill: f64,
    pub arrows: i32,
    pub bolts: i32,
    pub last_use: Option<muo_primitives::DateTime>,
    /// `m_Entries`: `(shooter, total, count)` in insertion order.
    pub entries: Option<Vec<(MobileId, i32, i32)>>,
}

fn butte(w: &World, id: ItemId) -> ButteData {
    w.item(id)
        .and_then(|i| i.ext.get::<ButteData>())
        .cloned()
        .unwrap_or(ButteData {
            min_skill: -25.0,
            max_skill: 25.0,
            ..Default::default()
        })
}

pub fn butte_mut(w: &mut World, id: ItemId) -> Option<&mut ButteData> {
    let fresh = butte(w, id);
    let i = w.item_mut(id)?;
    if i.ext.get::<ButteData>().is_none() {
        i.ext.insert(fresh);
    }
    i.ext.get_mut::<ButteData>()
}

fn butte_facing_east(w: &World, id: ItemId) -> bool {
    cur_item_id(w, id) == 0x100A
}

fn butte_gather(w: &mut World, id: ItemId, from: MobileId) {
    overhead(w, from, 500592); // You gather the arrows and bolts.
    let d = butte(w, id);
    if d.arrows > 0
        && let Some(a) = muo_content::factory::make_item_amount(w, "Server.Items.Arrow", d.arrows)
    {
        muo_server::mobile::part2::items::add_to_backpack(w, from, a);
    }
    if d.bolts > 0
        && let Some(b) = muo_content::factory::make_item_amount(w, "Server.Items.Bolt", d.bolts)
    {
        muo_server::mobile::part2::items::add_to_backpack(w, from, b);
    }
    if let Some(d) = butte_mut(w, id) {
        d.arrows = 0;
        d.bolts = 0;
        d.entries = None;
    }
}

/// `GetEntryFor(from)` then `Record(score)`: the shooter's `(Total, Count)` after the shot.
fn butte_record(w: &mut World, id: ItemId, from: MobileId, score: Option<i32>) -> (i32, i32) {
    let Some(d) = butte_mut(w, id) else {
        return (0, 0);
    };
    let entries = d.entries.get_or_insert_with(Vec::new);
    let pos = entries.iter().position(|e| e.0 == from).unwrap_or_else(|| {
        entries.push((from, 0, 0));
        entries.len() - 1
    });
    if let Some(s) = score {
        entries[pos].1 += s;
        entries[pos].2 += 1;
    }
    (entries[pos].1, entries[pos].2)
}

fn butte_report(w: &mut World, id: ItemId, total: i32, count: i32) {
    if count == 1 {
        public_msg(w, id, 1062719, &total.to_string());
    } else {
        public_msg(w, id, 1042683, &format!("{total}\t{count}"));
    }
}

fn ammo_type_of(w: &World, name: &str) -> Option<&'static ItemType> {
    match w.types.find_type_by_full_name(name, false)? {
        muo_server::type_registry::EntityType::Item(t) => Some(t),
        _ => None,
    }
}

fn butte_fire(w: &mut World, id: ItemId, from: MobileId) {
    let bow = muo_content::items::weapon_hooks::weapon_item(w, from)
        .filter(|k| w.item(*k).is_some_and(|i| i.ty.is(&weapon::BASE_RANGED)));
    let Some(bow) = bow else {
        msg_to(w, id, from, 500593); // You must practice with ranged weapons on this.
        return;
    };
    let now = w.now;
    if butte(w, id)
        .last_use
        .is_some_and(|l| now < l + TimeSpan::from_secs_f64(2.0))
    {
        return;
    }
    let world_loc = ops::get_world_location(w, id);
    let east = butte_facing_east(w, id);
    let floc = mx::location(w, from);
    if if east {
        floc.x <= world_loc.x
    } else {
        floc.y <= world_loc.y
    } {
        overhead(w, from, 500596); // You would do better to stand in front of the archery butte.
        return;
    }
    if if east {
        floc.y != world_loc.y
    } else {
        floc.x != world_loc.x
    } {
        overhead(w, from, 500597); // You aren't properly lined up with the archery butte to get an accurate shot.
        return;
    }
    if !mx::in_range_point(w, from, world_loc, 6) {
        overhead(w, from, 500598); // You are too far away from the archery butte to get an accurate shot.
        return;
    }
    if mx::in_range_point(w, from, world_loc, 4) {
        overhead(w, from, 500599); // You are too close to the target.
        return;
    }

    let pack = props_b::backpack(w, from);
    let ammo_name = weapon::class(w, bow).ammo_type;
    let is_arrow = ammo_name == Some("Server.Items.Arrow");
    let is_bolt = ammo_name == Some("Server.Items.Bolt");
    let is_known = is_arrow || is_bolt;

    let ammo = ammo_name.and_then(|n| ammo_type_of(w, n));
    let consumed = match (pack, ammo) {
        (Some(p), Some(t)) => muo_server::item::container::consume_total(w, p, t, 1, true, None),
        _ => false,
    };
    if !consumed {
        if is_arrow {
            overhead(w, from, 500594); // You do not have any arrows with which to practice.
        } else if is_bolt {
            overhead(w, from, 500595); // You do not have any crossbow bolts with which to practice.
        } else {
            msg_to(w, id, from, 500593); // You must practice with ranged weapons on this.
        }
        return;
    }

    if let Some(d) = butte_mut(w, id) {
        d.last_use = Some(now);
    }
    let dir = muo_server::mobile::part2::effects::get_direction_to(
        w,
        from,
        world_loc.x,
        world_loc.y,
        false,
    );
    mx::set_direction(w, from, dir);
    muo_content::items::weapon_combat::play_swing_animation(w, bow, from);
    let effect = weapon::effect_id(w, bow);
    muo_server::mobile::part2::effects::moving_effect_plain(
        w,
        from,
        EntityId::Item(id),
        effect,
        18,
        1,
        false,
        false,
    );

    let d = butte(w, id);
    if !mx::check_skill(w, from, weapon::skill(w, bow), d.min_skill, d.max_skill) {
        let miss = weapon::miss_sound(w, bow);
        mx::play_sound(w, from, miss);
        let name = w
            .mobile(from)
            .and_then(|m| (m.ty.name)(w, from))
            .unwrap_or_default();
        public_msg(w, id, 500604, &name); // You miss the target altogether.
        let (total, count) = butte_record(w, id, from, Some(0));
        butte_report(w, id, total, count);
        return;
    }

    let loc = w.item(id).map_or_else(Point3D::default, |i| i.location);
    let map = w.item(id).and_then(|i| i.map);
    muo_server::effects::play_sound(w, loc, map, 0x2B1);

    let rand = utility::random_double(&mut w.rng);
    let (area, score, split_score) = if rand < 0.10 {
        (0, 50, 100)
    } else if rand < 0.25 {
        (1, 10, 20)
    } else if rand < 0.50 {
        (2, 5, 15)
    } else {
        (3, 2, 5)
    };

    let d = butte(w, id);
    let split =
        is_known && f64::from(d.arrows + d.bolts) * 0.02 > utility::random_double(&mut w.rng);
    let name = w
        .mobile(from)
        .and_then(|m| (m.ty.name)(w, from))
        .unwrap_or_default();
    if split {
        public_msg(
            w,
            id,
            1010027 + area,
            &format!("{name}\t{}", if is_arrow { "arrow" } else { "bolt" }),
        );
    } else {
        public_msg(w, id, 1010035 + area, &name);
        if let Some(d) = butte_mut(w, id) {
            if is_arrow {
                d.arrows += 1;
            } else if is_bolt {
                d.bolts += 1;
            }
        }
        ops::invalidate_properties(w, id);
    }
    let (total, count) = butte_record(w, id, from, Some(if split { split_score } else { score }));
    butte_report(w, id, total, count);
}

fn butte_on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    let loc = ops::get_world_location(w, id);
    let in_range = mx::in_range_point(w, from, loc, 1);
    let d = butte(w, id);
    if (d.arrows > 0 || d.bolts > 0) && in_range {
        butte_gather(w, id, from);
    } else {
        let thrown = muo_content::items::weapon_hooks::weapon_item(w, from)
            .is_some_and(|k| w.item(k).is_some_and(|i| i.ty.is(&weapon::BASE_THROWN)));
        if !thrown || in_range {
            butte_fire(w, id, from);
        }
    }
}

/// `ArcheryButte(itemID)`.
pub fn archery_butte_new(w: &mut World, item_id: i32) -> ItemId {
    let id = addon::new_component(w, &ARCHERY_BUTTE, item_id);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(ButteData {
            min_skill: -25.0,
            max_skill: 25.0,
            ..Default::default()
        });
    }
    id
}

/// `Server.Items.ArcheryButte` (`[Flippable(0x100A, 0x100B)]`).
pub static ARCHERY_BUTTE: ItemType = ItemType {
    full_name: "Server.Items.ArcheryButte",
    parent: Some(&ADDON_COMPONENT),
    construct: Some(|w| archery_butte_new(w, 0x100A)),
    on_double_click: butte_on_double_click,
    serialize: |w, id, out| {
        (ADDON_COMPONENT.serialize)(w, id, out);
        write_ver(out, false);
        let d = butte(w, id);
        out.write_f64(d.min_skill);
        out.write_f64(d.max_skill);
        out.write_i32(d.arrows);
        out.write_i32(d.bolts);
    },
    deserialize: |w, id, r| {
        (ADDON_COMPONENT.deserialize)(w, id, r)?;
        read_ver(r, false)?;
        let mut d = ButteData {
            min_skill: r.read_f64()?,
            max_skill: r.read_f64()?,
            arrows: r.read_i32()?,
            bolts: r.read_i32()?,
            ..Default::default()
        };
        // [AfterDeserialization]: the old defaults.
        if d.min_skill == 0.0 && d.max_skill == 30.0 {
            d.min_skill = -25.0;
            d.max_skill = 25.0;
        }
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(d);
        }
        Ok(())
    },
    ..ADDON_COMPONENT
};

/// `ArcheryButteAddon()`.
pub fn archery_butte_addon_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &ARCHERY_BUTTE_ADDON);
    let c = archery_butte_new(w, 0x100A);
    addon::add_component(w, id, c, 0, 0, 0);
    id
}

static ARCHERY_BUTTE_ADDON_CLASS: AddonClass = AddonClass {
    deed: Some(|w| addon::new_deed(w, &ARCHERY_BUTTE_DEED)),
    ..BASE_ADDON_CLASS
};

leaf! {
    /// `Server.Items.ArcheryButteAddon`.
    ARCHERY_BUTTE_ADDON = "Server.Items.ArcheryButteAddon" : BASE_ADDON, v0;
    construct: Some(archery_butte_addon_new),
    family: Some(&ARCHERY_BUTTE_ADDON_CLASS),
}
deed!(
    ARCHERY_BUTTE_DEED,
    ARCHERY_BUTTE_DEED_CLASS,
    "Server.Items.ArcheryButteDeed",
    v0,
    archery_butte_addon_new,
    1024106
);

// ---- registry -----------------------------------------------------------------------------------------------

pub static ALL: &[(&ItemType, &[&str])] = &[
    (
        &SPINNING_WHEEL_EAST_ADDON,
        &["Server.Items.SpinningwheelEastAddon"],
    ),
    (
        &SPINNING_WHEEL_EAST_DEED,
        &["Server.Items.SpinningwheelEastDeed"],
    ),
    (
        &SPINNING_WHEEL_SOUTH_ADDON,
        &["Server.Items.SpinningwheelSouthAddon"],
    ),
    (
        &SPINNING_WHEEL_SOUTH_DEED,
        &["Server.Items.SpinningwheelSouthDeed"],
    ),
    (
        &ELVEN_SPINNING_WHEEL_EAST_ADDON,
        &["Server.Items.ElvenSpinningwheelEastAddon"],
    ),
    (
        &ELVEN_SPINNING_WHEEL_EAST_DEED,
        &["Server.Items.ElvenSpinningwheelEastDeed"],
    ),
    (
        &ELVEN_SPINNING_WHEEL_SOUTH_ADDON,
        &["Server.Items.ElvenSpinningwheelSouthAddon"],
    ),
    (
        &ELVEN_SPINNING_WHEEL_SOUTH_DEED,
        &["Server.Items.ElvenSpinningwheelSouthDeed"],
    ),
    (&LOOM_EAST_ADDON, &[]),
    (&LOOM_EAST_DEED, &[]),
    (&LOOM_SOUTH_ADDON, &[]),
    (&LOOM_SOUTH_DEED, &[]),
    (&FLOUR_MILL_EAST_ADDON, &[]),
    (&FLOUR_MILL_EAST_DEED, &[]),
    (&FLOUR_MILL_SOUTH_ADDON, &[]),
    (&FLOUR_MILL_SOUTH_DEED, &[]),
    (&WATER_TROUGH_EAST_ADDON, &[]),
    (&WATER_TROUGH_EAST_DEED, &[]),
    (&WATER_TROUGH_SOUTH_ADDON, &[]),
    (&WATER_TROUGH_SOUTH_DEED, &[]),
    (&REJUVINATION_ADDON_COMPONENT, &[]),
    (&BASE_REJUVINATION_ANKH, &[]),
    (&REJUVINATION_ANKH_WEST, &[]),
    (&REJUVINATION_ANKH_NORTH, &[]),
    (&DART_BOARD, &[]),
    (&DART_BOARD_EAST_ADDON, &[]),
    (&DART_BOARD_EAST_DEED, &[]),
    (&DART_BOARD_SOUTH_ADDON, &[]),
    (&DART_BOARD_SOUTH_DEED, &[]),
    (&PICKPOCKET_DIP, &[]),
    (&PICKPOCKET_DIP_EAST_ADDON, &[]),
    (&PICKPOCKET_DIP_EAST_DEED, &[]),
    (&PICKPOCKET_DIP_SOUTH_ADDON, &[]),
    (&PICKPOCKET_DIP_SOUTH_DEED, &[]),
    (&TRAINING_DUMMY, &[]),
    (&TRAINING_DUMMY_EAST_ADDON, &[]),
    (&TRAINING_DUMMY_EAST_DEED, &[]),
    (&TRAINING_DUMMY_SOUTH_ADDON, &[]),
    (&TRAINING_DUMMY_SOUTH_DEED, &[]),
    (&ARCHERY_BUTTE, &[]),
    (&ARCHERY_BUTTE_ADDON, &[]),
    (&ARCHERY_BUTTE_DEED, &[]),
];

pub fn register(types: &mut TypeRegistry) {
    for (t, aliases) in ALL {
        types.register_item(t, aliases);
    }
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "Server.Items.WaterTrough.Configure",
        configure_water,
    );
}

/// `item as IWaterSource` (or the addon of an `AddonComponent`): a water trough's `Quantity`, always 500.
fn water_source(w: &World, item: ItemId) -> Option<i32> {
    let trough = |id: ItemId| {
        w.item(id).is_some_and(|i| {
            i.ty.is(&WATER_TROUGH_EAST_ADDON) || i.ty.is(&WATER_TROUGH_SOUTH_ADDON)
        })
    };
    if trough(item) || addon::component_addon(w, item).is_some_and(trough) {
        Some(WATER_TROUGH_QUANTITY)
    } else {
        None
    }
}

/// Sets the hooks of the beverages that ask for a water source.
pub fn configure_water(w: &mut World) {
    let b = w
        .resources
        .get_mut::<crate::food::beverage::BeverageHooks>();
    b.water_source = Some(water_source);
    // `IHasQuantity.Quantity` of a trough: `set { }` does nothing.
    b.water_source_set = Some(|_, _, _| {});
}
