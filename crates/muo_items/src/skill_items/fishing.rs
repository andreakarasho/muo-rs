//! `FishingPole`, `MessageInABottle`, `SOS`, `ShipwreckedItem`, `SpecialFishingNet` and `FabledFishingNet`.
//! `Sextant` is in runebook.rs.
//!
//! What the SOS side of fishing needs from these classes is [`sos_near`] and [`sos_catch`]
//! (muo_engines sets them on `HarvestHooks`). [`FishingHooks`] holds what lives in classes of other crates.

use muo_content::gumps::base::{StaticGumpLogic, StaticGumpObj};
use muo_content::gumps::builder::{GumpStrings, StaticGumpBuilder};
use muo_content::items::interfaces::register_dyable;
use muo_content::items::tool::{BASE_TOOL_CLASS, ToolClass};
use muo_content::props::{Ctor, Kind, Param, Value, add_ctors};
use muo_content::utilities::types::{Numeric, PropType};
use muo_io::{GenericReader, GenericWriter, Result};
use muo_primitives::{Point3D, TimeSpan, utility};
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, base, deps, ops};
use muo_server::layer::Layer;
use muo_server::map::MapId;
use muo_server::packets::message::MessageType;
use muo_server::property_list::PropertyList;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::target::{self, TargetFlags, Targeted};
use muo_server::timer;
use muo_server::type_registry::{Phase, TypeRegistry};

use super::runebook::sextant_format;
use super::{dyes, read_version, write_version};

type LocalizedFormatFn = fn(&mut World, i32, Option<&str>, &str) -> String;
/// What the SOS items ask of other crates. Unset: nothing happens.
#[derive(Default)]
pub struct FishingHooks {
    /// `Localization.Format(number, lang, $"{arg}")` (empty when the cliloc does not exist).
    pub localized_format: Option<LocalizedFormatFn>,
    /// `TreasureMapChest.Fill(chest, level)`.
    pub fill_treasure_chest: Option<fn(&mut World, ItemId, i32)>,
    /// `new Leviathan(fisher)`.
    pub make_leviathan: Option<fn(&mut World, MobileId) -> MobileId>,
}

fn hooks(w: &World) -> Option<&FishingHooks> {
    w.resources.get::<FishingHooks>()
}

// ---- FishingPole ----

fn pole_double_click(w: &mut World, id: ItemId, from: MobileId) {
    if !ops::is_child_of(w, id, Some(EntityId::Mobile(from))) {
        let loc = ops::get_world_location(w, id);
        if !muo_server::mobile::part1::combat::in_los_point(w, from, loc)
            || !deps::mobile_in_range(w, from, loc, 2)
        {
            deps::local_overhead_message(w, from, MessageType::REGULAR, 0x3E9, 1019045); // I can't reach that
            return;
        }
    }
    if let Some(f) = w
        .resources
        .get::<muo_content::items::weapon_hooks::WeaponHooks>()
        .and_then(|h| h.harvest_begin)
    {
        f(w, from, id);
    }
}

/// `Fishing.System` is the harvest system of the pole.
static POLE_CLASS: ToolClass = ToolClass {
    harvest_system: "Fishing",
    ..BASE_TOOL_CLASS
};

/// `Server.Items.FishingPole`.
pub static FISHING_POLE: ItemType = ItemType {
    full_name: "Server.Items.FishingPole",
    parent: Some(&ITEM_BASE),
    construct: Some(new_fishing_pole),
    family: Some(&POLE_CLASS),
    default_weight: |_, _| 8.0,
    on_double_click: pole_double_click,
    get_context_menu_entries: |w, id, from, list| {
        base::get_context_menu_entries_base(w, id, from, list);
        if let Some(f) = w
            .resources
            .get::<muo_content::items::weapon_hooks::WeaponHooks>()
            .and_then(|h| h.harvest_context_entries)
        {
            f(w, from, id, list);
        }
    },
    check_conflicting_layer: |w, id, m, item, layer| {
        if base::check_conflicting_layer_base(w, id, m, item, layer) {
            return true;
        }
        if layer == Layer::OneHanded {
            deps::send_localized_message(w, m, 500214); // You already have something in both hands.
            return true;
        }
        false
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

/// `new FishingPole()`.
pub fn new_fishing_pole(w: &mut World) -> ItemId {
    new_pole_of(w, &FISHING_POLE)
}

/// `FishingPole()` of class `ty`.
pub fn new_pole_of(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0x0DC0);
    (FISHING_POLE.set_layer)(w, id, Layer::TwoHanded);
    id
}

// ---- MessageInABottle and SOS ----

/// `MessageInABottle._level` and `_targetMap`; the SOS adds its location and message.
#[derive(Clone, Copy, Debug)]
pub struct SosData {
    pub level: i32,
    pub target_map: Option<MapId>,
    pub target_location: Point3D,
    pub message_index: i32,
}

fn sos_data(w: &World, id: ItemId) -> Option<&SosData> {
    w.item(id)?.ext.get::<SosData>()
}

fn sos_data_mut(w: &mut World, id: ItemId) -> Option<&mut SosData> {
    w.item_mut(id)?.ext.get_mut::<SosData>()
}

/// `Math.Max(1, Math.Min(value, 4))`.
fn clamp_level(v: i32) -> i32 {
    v.clamp(1, 4)
}

/// `Level = value` of a bottle or SOS.
pub fn set_level(w: &mut World, id: ItemId, v: i32) {
    let v = clamp_level(v);
    if sos_data(w, id).is_some_and(|d| d.level == v) {
        return;
    }
    if let Some(d) = sos_data_mut(w, id) {
        d.level = v;
    }
    if w.item(id).is_some_and(|i| i.ty.is(&SOS)) {
        ops::invalidate_properties(w, id);
        update_hue(w, id);
    }
}

/// `MessageInABottle.GetRandomLevel()`.
pub fn get_random_level(w: &mut World) -> i32 {
    let n = if muo_content::core::se(w) { 25 } else { 40 };
    if muo_content::core::aos(w) && utility::random(&mut w.rng, n) < 1 {
        return 4; // ancient
    }
    utility::random_min_max(&mut w.rng, 1, 3)
}

/// `new MessageInABottle(map, level)`; `None` is no map.
pub fn new_message_in_a_bottle(w: &mut World, map: Option<MapId>, level: Option<i32>) -> ItemId {
    new_bottle_of(w, &MESSAGE_IN_A_BOTTLE, map, level)
}

/// `MessageInABottle(map, level)` of class `ty`.
pub fn new_bottle_of(
    w: &mut World,
    ty: &'static ItemType,
    map: Option<MapId>,
    level: Option<i32>,
) -> ItemId {
    let level = level.unwrap_or_else(|| get_random_level(w));
    let id = ops::new_item_with_id(w, ty, 0x099F);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(SosData {
            level,
            target_map: Some(map.unwrap_or(MapId::TRAMMEL)),
            target_location: Point3D::default(),
            message_index: 0,
        });
    }
    id
}

fn bottle_double_click(w: &mut World, id: ItemId, from: MobileId) {
    let pack = w.mobile(from).and_then(|m| m.backpack);
    if ops::is_child_of(w, id, pack.map(EntityId::Item)) {
        let d = sos_data(w, id).copied();
        if let Some(d) = d {
            let sos = new_sos(w, d.target_map, Some(d.level));
            base::replace_with(w, id, sos);
        }
        deps::local_overhead_message(w, from, MessageType::REGULAR, 0x3B2, 501891); // You extract the message from the bottle.
    } else {
        deps::send_localized_message(w, from, 1042001); // That must be in your pack for you to use it.
    }
}

fn sos_serialize(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    (ITEM_BASE.serialize)(w, id, out);
    write_version(out, false);
    let d = sos_data(w, id).copied().unwrap_or(SosData {
        level: 1,
        target_map: None,
        target_location: Point3D::default(),
        message_index: 0,
    });
    out.write_i32(d.level);
    out.write_map(d.target_map);
    if w.item(id).is_some_and(|i| i.ty.is(&SOS)) {
        out.write_point3d(d.target_location);
        out.write_i32(d.message_index);
    }
}

fn sos_deserialize(w: &mut World, id: ItemId, r: &mut dyn GenericReader) -> Result<()> {
    (ITEM_BASE.deserialize)(w, id, r)?;
    read_version(r, false)?;
    let mut d = SosData {
        level: r.read_i32()?,
        target_map: r.read_map()?,
        target_location: Point3D::default(),
        message_index: 0,
    };
    if w.item(id).is_some_and(|i| i.ty.is(&SOS)) {
        d.target_location = r.read_point3d()?;
        d.message_index = r.read_i32()?;
    }
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(d);
    }
    Ok(())
}

/// `Server.Items.MessageInABottle`.
pub static MESSAGE_IN_A_BOTTLE: ItemType = ItemType {
    full_name: "Server.Items.MessageInABottle",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| new_message_in_a_bottle(w, None, None)),
    default_weight: |_, _| 1.0,
    label_number: |_, _| 1041080, // a message in a bottle
    on_double_click: bottle_double_click,
    serialize: sos_serialize,
    deserialize: sos_deserialize,
    ..ITEM_BASE
};

const BRIT_REGIONS: [(i32, i32, i32, i32); 1] = [(0, 0, 5120, 4096)];
const ILSH_REGIONS: [(i32, i32, i32, i32); 2] = [(1472, 272, 304, 240), (1240, 1000, 312, 160)];
const MALAS_REGIONS: [(i32, i32, i32, i32); 1] = [(1376, 1520, 464, 280)];

/// `SOS.MessageEntries`: 1153537 to 1153550.
const MESSAGE_ENTRIES: i32 = 14;
const FIRST_MESSAGE: i32 = 1153537;

/// `SOS.FindLocation(map)`.
pub fn find_location(w: &mut World, map: Option<MapId>) -> Point3D {
    let Some(map) = map.filter(|&m| m != MapId::INTERNAL) else {
        return Point3D::default();
    };
    let regions: Vec<(i32, i32, i32, i32)> = if map == MapId::FELUCCA || map == MapId::TRAMMEL {
        BRIT_REGIONS.to_vec()
    } else if map == MapId::ILSHENAR {
        ILSH_REGIONS.to_vec()
    } else if map == MapId::MALAS {
        MALAS_REGIONS.to_vec()
    } else {
        let (mw, mh) = w.maps.get(map).map_or((0, 0), |m| (m.width, m.height));
        vec![(0, 0, mw, mh)]
    };
    for _ in 0..50 {
        let Some(&(rx, ry, rw, rh)) = utility::random_element(&mut w.rng, &regions) else {
            return Point3D::default();
        };
        let x = utility::random_from(&mut w.rng, rx, rw);
        let y = utility::random_from(&mut w.rng, ry, rh);
        if full_validation(w, map, x, y) {
            return Point3D::new(x, y, 0);
        }
    }
    Point3D::default()
}

/// `new SOS(map, level)`; `None` for `level` is the overload that rolls one.
pub fn new_sos(w: &mut World, map: Option<MapId>, level: Option<i32>) -> ItemId {
    let level = level.unwrap_or_else(|| get_random_level(w));
    let id = ops::new_item_with_id(w, &SOS, 0x14EE);
    let message_index = utility::random(&mut w.rng, MESSAGE_ENTRIES);
    let target_map = Some(map.unwrap_or(MapId::TRAMMEL));
    let target_location = find_location(w, target_map);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(SosData {
            level,
            target_map,
            target_location,
            message_index,
        });
    }
    update_hue(w, id);
    id
}

/// `IsAncient`.
pub fn is_ancient(w: &World, id: ItemId) -> bool {
    sos_data(w, id).is_some_and(|d| d.level >= 4)
}

/// `UpdateHue()`.
fn update_hue(w: &mut World, id: ItemId) {
    let h = if is_ancient(w, id) { 0x481 } else { 0 };
    super::hue(w, id, h);
}

/// `Sextant.Format` as the two gumps print it, with `sep` between the latitude and the longitude parts.
fn sextant_text(w: &World, loc: Point3D, map: Option<MapId>, spaced: bool) -> String {
    match sextant_format(w, loc, map) {
        Some(s) => {
            let (ns, ew) = (
                if s.y_south { "S" } else { "N" },
                if s.x_east { "E" } else { "W" },
            );
            if spaced {
                format!(
                    "{}° {}'{}, {}°{}'{}",
                    s.y_lat, s.y_mins, ns, s.x_long, s.x_mins, ew
                )
            } else {
                format!(
                    "{}°{}'{},{}°{}'{}",
                    s.y_lat, s.y_mins, ns, s.x_long, s.x_mins, ew
                )
            }
        }
        None => "?????".to_owned(),
    }
}

/// `SOS.OldMessageGump`.
pub struct OldMessageGump {
    message: String,
}

const OLD_WIDTH: i32 = 350;
const OLD_HEIGHT: i32 = 300;

impl StaticGumpLogic for OldMessageGump {
    const TYPE_NAME: &'static str = "Server.Items.SOS+OldMessageGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.add_page(0);
        b.add_background(0, 0, OLD_WIDTH, OLD_HEIGHT, 2520);
        b.add_html_placeholder(
            38,
            38,
            OLD_WIDTH - 83,
            OLD_HEIGHT - 86,
            "message",
            false,
            false,
        );
    }

    fn build_strings(&self, s: &mut GumpStrings) {
        s.set_string_slot("message", &self.message);
    }
}

fn sos_double_click(w: &mut World, id: ItemId, from: MobileId) {
    let pack = w.mobile(from).and_then(|m| m.backpack);
    if !ops::is_child_of(w, id, pack.map(EntityId::Item)) {
        deps::send_localized_message(w, from, 1042001); // That must be in your pack for you to use it.
        return;
    }
    // UseNewMessages is false and every index below the entry count has an entry: the old gump.
    let Some(d) = sos_data(w, id).copied() else {
        return;
    };
    if w.mobile(from).and_then(|m| m.net_state).is_none() {
        return;
    }
    let fmt = sextant_text(w, d.target_location, d.target_map, false);
    if (0..MESSAGE_ENTRIES).contains(&d.message_index) {
        let number = FIRST_MESSAGE + d.message_index;
        let lang = w.mobile(from).and_then(|m| m.language.clone());
        let message = match hooks(w).and_then(|h| h.localized_format) {
            Some(f) => f(w, number, lang.as_deref(), &fmt),
            None => String::new(),
        };
        let g = StaticGumpObj::new(
            (640 - OLD_WIDTH) / 2,
            (480 - OLD_HEIGHT) / 2,
            OldMessageGump { message },
        );
        super::hue_pickers::send(w, from, g);
    } else {
        let fmt = sextant_text(w, d.target_location, d.target_map, true);
        super::hue_pickers::send(w, from, StaticGumpObj::new(150, 50, MessageGump { fmt }));
    }
}

/// `SOS.MessageGump`.
pub struct MessageGump {
    fmt: String,
}

impl StaticGumpLogic for MessageGump {
    const TYPE_NAME: &'static str = "Server.Items.SOS+MessageGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.add_page(0);
        b.add_background(0, 40, 350, 300, 2520);
        /* This is a message hastily scribbled by a passenger aboard a sinking ship.
         * While it is probably too late to save the passengers and crew,
         * perhaps some treasure went down with the ship!
         * The message gives the ship's last known sextant co-ordinates.
         */
        b.add_html_localized(30, 80, 285, 160, 1018326, true, true);
        b.add_html_placeholder(35, 240, 230, 20, "fmt", false, false);
        b.add_button(35, 265, 4005, 4007, 0);
        b.add_html_localized(70, 265, 100, 20, 1011036, false, false); // OKAY
    }

    fn build_strings(&self, s: &mut GumpStrings) {
        s.set_string_slot("fmt", &self.fmt);
    }
}

/// `Server.Items.SOS`.
pub static SOS: ItemType = ItemType {
    full_name: "Server.Items.SOS",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| new_sos(w, None, None)),
    default_weight: |_, _| 1.0,
    label_number: |w, id| if is_ancient(w, id) { 1063450 } else { 1041081 }, // an ancient SOS / a waterstained SOS
    on_double_click: sos_double_click,
    serialize: sos_serialize,
    deserialize: sos_deserialize,
    ..ITEM_BASE
};

/// The first `SOS` in the pack whose target map and location are within 60 tiles (`from.Map == sos.TargetMap &&
/// from.InRange(sos.TargetLocation, 60)`).
pub fn sos_near(w: &World, from: MobileId) -> Option<ItemId> {
    let pack = w.mobile(from)?.backpack?;
    let (map, loc) = w.mobile(from).map(|m| (m.map, m.location))?;
    muo_server::item::enumerable::find_items_by_type(w, pack, true, &SOS, |w, id| {
        sos_data(w, id)
            .is_some_and(|d| d.target_map == map && deps::in_range_xy(loc, d.target_location, 60))
    })
    .into_iter()
    .next()
}

// ---- ShipwreckedItem ----

fn shipwrecked_dye(w: &mut World, id: ItemId, from: MobileId, tub: ItemId) -> bool {
    if w.item(id).is_none_or(|i| i.deleted()) {
        return false;
    }
    let item_id = w.item(id).map_or(0, |i| i.item_id);
    if (0x13A4..=0x13AE).contains(&item_id) {
        let h = dyes::data(w, tub).map_or(0, |d| d.dyed_hue);
        super::hue(w, id, h);
        return true;
    }
    deps::send_localized_message(w, from, dyes::class(w, tub).fail_message);
    false
}

/// `new ShipwreckedItem(itemID)`.
pub fn new_shipwrecked_item(w: &mut World, item_id: i32) -> ItemId {
    ops::new_item_with_id(w, &SHIPWRECKED_ITEM, item_id)
}

/// `Server.Items.ShipwreckedItem`: no `[Constructible]` constructor.
pub static SHIPWRECKED_ITEM: ItemType = ItemType {
    full_name: "Server.Items.ShipwreckedItem",
    parent: Some(&ITEM_BASE),
    construct: None,
    default_weight: |w, id| {
        let weight = f64::from(ops::item_data(w, id).weight);
        if weight >= 255.0 { 1.0 } else { weight }
    },
    on_single_click: |w, id, from| {
        let label = base::label_number(w, id);
        ops::label_to_number(w, id, from, 1050039, &format!("{label}\t{}", 1041645));
    },
    add_name_properties: |w, id, list: &mut PropertyList| {
        base::add_name_properties_base(w, id, list);
        list.add(1041645); // recovered from a shipwreck
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

// ---- SpecialFishingNet ----

const NET_HUES: [i32; 14] = [
    0x09B, 0x0CD, 0x0D3, 0x14D, 0x1DD, 0x1E9, 0x1F4, 0x373, 0x451, 0x47F, 0x489, 0x492, 0x4B5,
    0x8AA,
];

/// `SpecialFishingNet._inUse`.
#[derive(Clone, Copy, Debug, Default)]
pub struct NetData {
    pub in_use: bool,
}

fn in_use(w: &World, id: ItemId) -> bool {
    w.item(id)
        .and_then(|i| i.ext.get::<NetData>())
        .is_some_and(|d| d.in_use)
}

pub fn set_in_use_pub(w: &mut World, id: ItemId, v: bool) {
    set_in_use(w, id, v);
}

fn set_in_use(w: &mut World, id: ItemId, v: bool) {
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(NetData { in_use: v });
    }
}

/// The virtuals of a net class: `RequireDeepWater` and `FinishEffect(p, map, from)`.
pub struct NetClass {
    pub require_deep_water: bool,
    pub finish_effect: fn(&mut World, ItemId, Point3D, Option<MapId>, MobileId),
}

pub static NET_CLASS: NetClass = NetClass {
    require_deep_water: true,
    finish_effect,
};

fn net_class(w: &World, id: ItemId) -> &'static NetClass {
    w.item(id)
        .and_then(|i| i.ty.family::<NetClass>())
        .unwrap_or(&NET_CLASS)
}

fn is_fabled(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.ty.is(&FABLED_FISHING_NET))
}

/// `new SpecialFishingNet()`.
pub fn new_special_fishing_net(w: &mut World) -> ItemId {
    new_net_of(w, &SPECIAL_FISHING_NET)
}

/// `SpecialFishingNet()` of class `ty`.
pub fn new_net_of(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0x0DCA);
    set_in_use(w, id, false);
    let hue = if muo_content::core::lbr(w) && utility::random_double(&mut w.rng) < 0.1 {
        utility::random_element(&mut w.rng, &NET_HUES)
            .copied()
            .unwrap_or(0x8A0)
    } else {
        0x8A0
    };
    super::hue(w, id, hue);
    id
}

/// `new FabledFishingNet()`.
pub fn new_fabled_fishing_net(w: &mut World) -> ItemId {
    let id = ops::new_item_with_id(w, &FABLED_FISHING_NET, 0x0DCA);
    set_in_use(w, id, false);
    let hue = if muo_content::core::lbr(w) && utility::random_double(&mut w.rng) < 0.1 {
        utility::random_element(&mut w.rng, &NET_HUES)
            .copied()
            .unwrap_or(0x8A0)
    } else {
        0x8A0
    };
    super::hue(w, id, hue);
    super::hue(w, id, 0x481);
    id
}

/// `ValidateDeepWater(map, x, y)`: the land tile is one of `_waterTiles` (0xA8-0xAB, 0x136-0x137).
fn validate_deep_water(w: &mut World, map: MapId, x: i32, y: i32) -> bool {
    let id = i32::from(w.maps.land_tile(map, x, y).id);
    (0xA8..=0xAB).contains(&id) || (0x136..=0x137).contains(&id)
}

/// `SpecialFishingNet.FullValidation(map, x, y)`: every tile closer than 14 must be water.
pub fn full_validation(w: &mut World, map: MapId, x: i32, y: i32) -> bool {
    for ty in (y - 13)..=(y + 13) {
        for tx in (x - 13)..=(x + 13) {
            if !validate_deep_water(w, map, tx, ty) {
                return false;
            }
        }
    }
    true
}

/// `ValidateUndeepWater(map, obj, ref z)`: a static of the shallow water tiles (0x1797-0x179C) outside a house.
fn validate_undeep_water(w: &mut World, map: MapId, t: &Targeted, z: &mut i32) -> bool {
    let Targeted::Static(s) = t else { return false };
    let house = w
        .resources
        .get::<muo_content::items::addon::AddonHooks>()
        .and_then(|h| h.find_house_at_point);
    if let Some(f) = house
        && f(w, s.location, map, 0).is_some()
    {
        return false;
    }
    if (0x1797..=0x179C).contains(&s.item_id) {
        *z = s.location.z;
        return true;
    }
    false
}

fn net_double_click(w: &mut World, id: ItemId, from: MobileId) {
    let pack = w.mobile(from).and_then(|m| m.backpack);
    if in_use(w, id) {
        deps::send_localized_message(w, from, 1010483); // Someone is already using that net!
    } else if ops::is_child_of(w, id, pack.map(EntityId::Item)) {
        deps::send_localized_message(w, from, 1010484); // Where do you wish to use the net?
        target::begin_target(w, from, -1, true, TargetFlags::NONE, move |w, from, t| {
            net_on_target(w, id, from, t)
        });
    } else {
        deps::send_localized_message(w, from, 1042001); // That must be in your pack for you to use it.
    }
}

/// `IPoint3D` of what was targeted.
fn point_of(w: &World, t: &Targeted) -> Option<Point3D> {
    match t {
        Targeted::Entity(EntityId::Item(i)) => w.item(*i).map(|x| x.location),
        Targeted::Entity(EntityId::Mobile(m)) => w.mobile(*m).map(|x| x.location),
        Targeted::Land(l) => Some(l.location),
        Targeted::Static(s) => Some(s.location),
    }
}

fn net_on_target(w: &mut World, id: ItemId, from: MobileId, t: Targeted) {
    use muo_server::map::los::LosObject;
    if w.item(id).is_none_or(|i| i.deleted()) || in_use(w, id) {
        return;
    }
    let Some(p3d) = point_of(w, &t) else { return };
    let Some(map) = w
        .mobile(from)
        .and_then(|m| m.map)
        .filter(|&m| m != MapId::INTERNAL)
    else {
        return;
    };
    let (x, y) = (p3d.x, p3d.y);
    let mut z = w.maps.get_average_z(map, x, y); // OSI just takes the targeted Z
    let require_deep_water = net_class(w, id).require_deep_water;
    let obj = match &t {
        Targeted::Entity(EntityId::Item(i)) => LosObject::Item(*i),
        Targeted::Entity(EntityId::Mobile(m)) => LosObject::Mobile(*m),
        Targeted::Land(l) => LosObject::Land(l.location),
        Targeted::Static(s) => LosObject::Static {
            x: s.location.x,
            y: s.location.y,
            z: s.location.z,
            item_id: s.item_id,
        },
    };
    if !deps::mobile_in_range(w, from, p3d, 6) {
        deps::send_localized_message(w, from, 500976); // You need to be closer to the water to fish!
    } else if !muo_server::mobile::part1::combat::in_los_object(w, from, obj) {
        deps::send_localized_message(w, from, 500979); // You cannot see that location.
    } else if if require_deep_water {
        full_validation(w, map, x, y)
    } else {
        validate_deep_water(w, map, x, y) || validate_undeep_water(w, map, &t, &mut z)
    } {
        let p = Point3D::new(x, y, z);
        if w.item(id)
            .is_some_and(|i| std::ptr::eq(i.ty, &SPECIAL_FISHING_NET))
        {
            let amount = w.item(id).map_or(1, |i| i.amount);
            for _ in 1..amount {
                // these were stackable before, doh
                let n = new_special_fishing_net(w);
                muo_server::mobile::part2::items::add_to_backpack(w, from, n);
            }
        }
        set_in_use(w, id, true);
        ops::set_movable(w, id, false);
        ops::move_to_world_map(w, id, p, Some(map));

        muo_content::spells::helper::turn(
            w,
            from,
            Some(muo_content::spells::helper::Point3dRef::Point(p)),
        );
        muo_server::mobile::base::animate(w, from, 12, 5, 1, true, false, 0);

        muo_server::effects::send_location_effect(w, p, Some(map), 0x352D, 16, 4, 0, 0);
        muo_server::effects::play_sound(w, p, Some(map), 0x364);

        let mut index = 0;
        timer::start(
            w,
            TimeSpan::from_secs(1),
            TimeSpan::from_millis(1250),
            14,
            move |w| {
                do_effect(w, id, from, p, index);
                index += 1;
            },
        );
        // You plunge the net into the sea... / You plunge the net into the water...
        deps::send_localized_message(w, from, if require_deep_water { 1010487 } else { 1074492 });
    } else {
        // You can only use this net in deep water! / You can only use this net in water!
        deps::send_localized_message(w, from, if require_deep_water { 1010485 } else { 1074491 });
    }
}

fn do_effect(w: &mut World, id: ItemId, from: MobileId, p: Point3D, index: i32) {
    let Some(map) = w.item(id).filter(|i| !i.deleted()).map(|i| i.map) else {
        return;
    };
    if index == 0 {
        muo_server::effects::send_location_effect(w, p, map, 0x352D, 16, 4, 0, 0);
        muo_server::effects::play_sound(w, p, map, 0x364);
    } else if index <= 6 || index == 13 {
        for _ in 0..if net_class(w, id).require_deep_water {
            3
        } else {
            0
        } {
            let (x, y) = match utility::random(&mut w.rng, 8) {
                1 => (-1, 0),
                2 => (-1, 1),
                3 => (0, -1),
                4 => (0, 1),
                5 => (1, -1),
                6 => (1, 0),
                7 => (1, 1),
                _ => (-1, -1),
            };
            muo_server::effects::send_location_effect(
                w,
                Point3D::new(p.x + x, p.y + y, p.z),
                map,
                0x352D,
                16,
                4,
                0,
                0,
            );
        }
        if !net_class(w, id).require_deep_water {
            muo_server::effects::send_location_effect(w, p, map, 0x352D, 16, 4, 0, 0);
        }
        if utility::random_bool(&mut w.rng) {
            muo_server::effects::play_sound(w, p, map, 0x364);
        }
        if index == 13 {
            (net_class(w, id).finish_effect)(w, id, p, map, from);
        } else if let Some(loc) = w.item(id).map(|i| i.location) {
            ops::set_location(w, id, Point3D::new(loc.x, loc.y, loc.z - 1));
        }
    }
}

/// `GetSpawnCount()`.
fn spawn_count(w: &mut World, id: ItemId) -> i32 {
    let hue = w.item(id).map_or(0, |i| i.hue);
    let base_count = if hue != 0x8A0 {
        utility::random_min_max(&mut w.rng, 2, 5)
    } else {
        utility::random_min_max(&mut w.rng, 1, 3)
    };
    if is_fabled(w, id) {
        base_count + 4
    } else {
        base_count
    }
}

/// `Spawn(p, map, spawn)`.
fn spawn(w: &mut World, p: Point3D, map: Option<MapId>, creature: MobileId, kraken: bool) {
    let Some(map) = map else {
        muo_server::mobile::base::delete(w, creature);
        return;
    };
    let (mut x, mut y) = (p.x, p.y);
    for _ in 0..20 {
        let tx = p.x - 2 + utility::random(&mut w.rng, 5);
        let ty = p.y - 2 + utility::random(&mut w.rng, 5);
        let t = w.maps.land_tile(map, tx, ty);
        let id = i32::from(t.id);
        if i32::from(t.z) == p.z
            && ((0xA8..=0xAB).contains(&id) || (0x136..=0x137).contains(&id))
            && !muo_content::spells::helper::check_multi(
                w,
                Point3D::new(tx, ty, p.z),
                Some(map),
                true,
                0,
            )
        {
            x = tx;
            y = ty;
            break;
        }
    }
    muo_server::mobile::base::move_to_world(w, creature, Point3D::new(x, y, p.z), map);
    if kraken && utility::random_double(&mut w.rng) < 0.2 {
        let m = if map == MapId::FELUCCA {
            MapId::FELUCCA
        } else {
            MapId::TRAMMEL
        };
        let bottle = new_message_in_a_bottle(w, Some(m), None);
        muo_content::creature::setters::pack_item(w, creature, Some(bottle));
    }
}

/// `FinishEffect(p, map, from)`.
pub fn finish_effect(w: &mut World, id: ItemId, p: Point3D, map: Option<MapId>, from: MobileId) {
    if is_fabled(w, id)
        && let Some(f) = hooks(w).and_then(|h| h.make_leviathan)
    {
        let l = f(w, from);
        spawn(w, p, map, l, false);
    }
    muo_server::mobile::base::revealing_action(w, from);
    let count = spawn_count(w, id);
    for _ in 0..count {
        if map.is_none() {
            break;
        }
        let (name, kraken) = match utility::random(&mut w.rng, 4) {
            1 => ("DeepSeaSerpent", false),
            2 => ("WaterElemental", false),
            3 => ("Kraken", true),
            _ => ("SeaSerpent", false),
        };
        let Some(creature) =
            muo_content::factory::make_mobile(w, &format!("Server.Mobiles.{name}"))
        else {
            continue;
        };
        spawn(w, p, map, creature, kraken);
        muo_content::mx::set_combatant(w, creature, Some(from));
    }
    base::delete(w, id);
}

fn net_serialize(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    (ITEM_BASE.serialize)(w, id, out);
    write_version(out, false);
    out.write_bool(in_use(w, id));
}

fn net_deserialize(w: &mut World, id: ItemId, r: &mut dyn GenericReader) -> Result<()> {
    (ITEM_BASE.deserialize)(w, id, r)?;
    read_version(r, false)?;
    let used = r.read_bool()?;
    set_in_use(w, id, used);
    if used {
        // [AfterDeserialization]
        deps::delay_delete(w, id);
    }
    Ok(())
}

/// `Server.Items.SpecialFishingNet`.
pub static SPECIAL_FISHING_NET: ItemType = ItemType {
    full_name: "Server.Items.SpecialFishingNet",
    parent: Some(&ITEM_BASE),
    construct: Some(new_special_fishing_net),
    family: Some(&NET_CLASS),
    default_weight: |_, _| 1.0,
    label_number: |_, _| 1041079, // a special fishing net
    get_properties: |w, id, list| {
        base::get_properties_base(w, id, list);
        // AddNetProperties: as if the name wasn't enough..
        if !is_fabled(w, id) {
            list.add(1017410); // Special Fishing Net
        }
    },
    on_double_click: net_double_click,
    serialize: net_serialize,
    deserialize: net_deserialize,
    ..ITEM_BASE
};

/// `Server.Items.FabledFishingNet`.
pub static FABLED_FISHING_NET: ItemType = ItemType {
    full_name: "Server.Items.FabledFishingNet",
    parent: Some(&SPECIAL_FISHING_NET),
    construct: Some(new_fabled_fishing_net),
    label_number: |_, _| 1063451, // a fabled fishing net
    serialize: |w, id, out| {
        net_serialize(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        net_deserialize(w, id, r)?;
        read_version(r, false)?;
        Ok(())
    },
    ..SPECIAL_FISHING_NET
};

/// `Fishing.Construct` for an SOS in the pack within range: the catch the SOS brings up and the SOS consumed, or
/// `None` when no SOS is near. `fill` is `TreasureMapChest.Fill` through [`FishingHooks`].
pub fn sos_catch(w: &mut World, from: MobileId) -> Option<ItemId> {
    let pack = w.mobile(from)?.backpack?;
    let (map, loc) = w.mobile(from).map(|m| (m.map, m.location))?;
    let sos = muo_server::item::enumerable::find_items_by_type(w, pack, true, &SOS, |w, id| {
        sos_data(w, id)
            .is_some_and(|d| d.target_map == map && deps::in_range_xy(loc, d.target_location, 60))
    })
    .into_iter()
    .next()?;

    if utility::random(&mut w.rng, 4) != 0 {
        const BODY_PARTS: [i32; 8] = [
            0x1CDD, 0x1CE5, 0x1CE0, 0x1CE8, 0x1CE1, 0x1CE9, 0x1CE2, 0x1CEC,
        ];
        const BONE_PARTS: [i32; 15] = [
            0x1AE0, 0x1AE1, 0x1AE2, 0x1AE3, 0x1AE4, 0x1B09, 0x1B0A, 0x1B0B, 0x1B0C, 0x1B0D, 0x1B0E,
            0x1B0F, 0x1B10, 0x1B15, 0x1B16,
        ];
        const MISC_ITEMS: [i32; 9] = [
            0x1EB5, 0xA2A, 0xC1F, 0x1047, 0x1048, 0x1EB1, 0x1EB2, 0x1EB3, 0x1EB4,
        ];
        return Some(match utility::random(&mut w.rng, 7) {
            0 => {
                let g = *utility::random_element(&mut w.rng, &BODY_PARTS)?;
                new_shipwrecked_item(w, g)
            }
            1 => {
                let g = *utility::random_element(&mut w.rng, &BONE_PARTS)?;
                new_shipwrecked_item(w, g)
            }
            2 => {
                let g = utility::random_from(&mut w.rng, 0xE9F, 10);
                new_shipwrecked_item(w, g)
            }
            3 => {
                let g = utility::random_from(&mut w.rng, 0x13A4, 11);
                new_shipwrecked_item(w, g)
            }
            4 => {
                let g = utility::random_from(&mut w.rng, 0xFC4, 9);
                new_shipwrecked_item(w, g)
            }
            5 => {
                let name = if utility::random_bool(&mut w.rng) {
                    "Server.Items.SkullCap"
                } else {
                    "Server.Items.TricorneHat"
                };
                let hat = muo_content::factory::make_item(w, name)?;
                if let Some(d) = muo_content::items::clothing::data_mut(w, hat) {
                    d.is_shipwrecked = true;
                }
                hat
            }
            _ => {
                let r = utility::random(&mut w.rng, MISC_ITEMS.len() as i32 + 1) as usize;
                if r < MISC_ITEMS.len() {
                    new_shipwrecked_item(w, MISC_ITEMS[r])
                } else {
                    let c = muo_content::factory::make_item(w, "Server.Items.Candelabra")?;
                    crate::last::lights::set_shipwrecked(w, c, true);
                    c
                }
            }
        });
    }

    let name = if utility::random_bool(&mut w.rng) {
        "Server.Items.MetalGoldenChest"
    } else {
        "Server.Items.WoodenChest"
    };
    let chest = muo_content::factory::make_item(w, name)?;
    let (ancient, level) = (is_ancient(w, sos), sos_data(w, sos).map_or(1, |d| d.level));
    if ancient && muo_content::core::se(w) {
        super::hue(w, chest, 0x481);
    }
    if let Some(f) = hooks(w).and_then(|h| h.fill_treasure_chest) {
        f(w, chest, level.clamp(1, 4));
    }
    if muo_content::core::uor(w) {
        let net = if ancient {
            new_fabled_fishing_net(w)
        } else {
            new_special_fishing_net(w)
        };
        if let Some(ty) = w.item(chest).map(|i| i.ty) {
            (ty.drop_item)(w, chest, net);
        }
    }
    ops::set_movable(w, chest, true);
    muo_content::items::cont::set_locked(w, chest, false);
    if let Some(d) = muo_content::items::cont::ext_mut(w, chest) {
        d.trap_type = muo_content::items::cont::TrapType::None;
        d.trap_power = 0;
        d.trap_level = 0;
    }
    base::delete(w, sos);
    Some(chest)
}

const INT: Kind = Kind::Plain(PropType::Numeric(Numeric::Int));
static BOTTLE_CTORS: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[Param {
            name: "map",
            kind: Kind::Map,
            default: Some(Value::Null),
            ty: "",
        }],
        make: |w, v| EntityId::Item(new_message_in_a_bottle(w, v[0].map(), None)),
    },
    Ctor {
        access: None,
        params: &[
            Param {
                name: "map",
                kind: Kind::Map,
                default: None,
                ty: "",
            },
            Param {
                name: "level",
                kind: INT,
                default: None,
                ty: "",
            },
        ],
        make: |w, v| EntityId::Item(new_message_in_a_bottle(w, v[0].map(), Some(v[1].i32()))),
    },
];
static SOS_CTORS: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[Param {
            name: "map",
            kind: Kind::Map,
            default: Some(Value::Null),
            ty: "",
        }],
        make: |w, v| EntityId::Item(new_sos(w, v[0].map(), None)),
    },
    Ctor {
        access: None,
        params: &[
            Param {
                name: "map",
                kind: Kind::Map,
                default: None,
                ty: "",
            },
            Param {
                name: "level",
                kind: INT,
                default: None,
                ty: "",
            },
        ],
        make: |w, v| EntityId::Item(new_sos(w, v[0].map(), Some(v[1].i32()))),
    },
];

fn configure(w: &mut World) {
    add_ctors(w, "Server.Items.MessageInABottle", &BOTTLE_CTORS);
    add_ctors(w, "Server.Items.SOS", &SOS_CTORS);
    register_dyable(w, &SHIPWRECKED_ITEM, shipwrecked_dye);
}

/// Registers the fishing classes.
pub fn register(types: &mut TypeRegistry) {
    for t in [
        &FISHING_POLE,
        &MESSAGE_IN_A_BOTTLE,
        &SOS,
        &SHIPWRECKED_ITEM,
        &SPECIAL_FISHING_NET,
        &FABLED_FISHING_NET,
    ] {
        types.register_item(t, &[]);
    }
    types.register_phase_default(Phase::Configure, "Server.Items.SOS.Configure", configure);
}
