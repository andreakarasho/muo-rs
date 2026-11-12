//! Maps: `MapItem` and its crafted/preset subclasses, `MapItemPackets`
//! (the 0x56 map command and the 0x90/0xF5, 0x56 outgoing packets) and `PresetMap`. `TreasureMap` is in
//! `treasure.rs`.
//!
//! `ICraftable.OnCraft` is [`on_craft`], called through `CraftHooks::craftables`.
//! // shortcut: `[Flippable(0x14EB, 0x14EC)]` of `MapItem` is data in [`FLIPPABLE`] (the flip command is not implemented).

use muo_content::props::{Ctor, Kind, Param, Value, add_ctors};
use muo_content::utilities::types::{Numeric, PropType};
use muo_io::{GenericReader, GenericWriter, IoError, Result, SpanReader};
use muo_primitives::{Point2D, Rectangle2D, Serial, utility};
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, base, deps, messages, ops};
use muo_server::map::MapId;
use muo_server::mobile::AccessLevel;
use muo_server::net_state::NetStateId;
use muo_server::packets::Pk;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::skills::SkillName;
use muo_server::type_registry::{Phase, TypeRegistry};

use crate::misc::{hue, read_version, write_version};

pub mod treasure;

#[cfg(test)]
mod tests;

const MAX_USER_PINS: usize = 50;

/// What `MapItem` adds to an `Item`, and `PresetMap`'s `_labelNumber`.
#[derive(Clone, Debug)]
pub struct MapItemData {
    pub bounds: Rectangle2D,
    pub width: i32,
    pub height: i32,
    pub protected: bool,
    pub pins: Vec<Point2D>,
    pub facet: Option<MapId>,
    pub editable: bool,
    /// `PresetMap._labelNumber`.
    pub preset_label: i32,
    /// `TreasureMap`'s fields.
    pub treasure: Option<treasure::TreasureData>,
}

impl Default for MapItemData {
    fn default() -> Self {
        MapItemData {
            bounds: Rectangle2D::EMPTY,
            width: 0,
            height: 0,
            protected: false,
            pins: Vec::new(),
            facet: None,
            editable: false,
            preset_label: 0,
            treasure: None,
        }
    }
}

/// The virtuals of `MapItem` that a subclass overrides.
pub struct MapItemClass {
    /// `CraftInit(from)`.
    pub craft_init: fn(&mut World, ItemId, MobileId),
    /// `DisplayTo(from)`.
    pub display_to: fn(&mut World, ItemId, MobileId),
}

pub fn data(w: &World, id: ItemId) -> Option<&MapItemData> {
    w.item(id)?.ext.get::<MapItemData>()
}

pub fn data_mut(w: &mut World, id: ItemId) -> Option<&mut MapItemData> {
    w.item_mut(id)?.ext.get_mut::<MapItemData>()
}

/// `item is MapItem`.
pub fn is_map_item(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.ty.is(&MAP_ITEM))
}

fn class(w: &World, id: ItemId) -> &'static MapItemClass {
    w.item(id)
        .and_then(|i| i.ty.family::<MapItemClass>())
        .unwrap_or(&MAP_ITEM_CLASS)
}

/// `MapItem(facet)` of class `ty`.
pub fn map_item_new(w: &mut World, ty: &'static ItemType, facet: Option<MapId>) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0x14EC);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(MapItemData {
            width: 200,
            height: 200,
            facet,
            ..Default::default()
        });
    }
    id
}

/// `MapItem.SetDisplay(x1, y1, x2, y2, w, h)`.
#[allow(clippy::too_many_arguments)] // the display bounds and size
pub fn set_display(
    w: &mut World,
    id: ItemId,
    mut x1: i32,
    mut y1: i32,
    mut x2: i32,
    mut y2: i32,
    width: i32,
    height: i32,
) {
    if x1 < 0 {
        x1 = 0;
    }
    if y1 < 0 {
        y1 = 0;
    }
    if x2 > 5119 {
        x2 = 5119;
    }
    if y2 > 4095 {
        y2 = 4095;
    }
    if let Some(d) = data_mut(w, id) {
        d.width = width;
        d.height = height;
        d.bounds = Rectangle2D::new(x1, y1, x2 - x1, y2 - y1);
    }
}

/// `ICraftable.OnCraft`: `CraftInit(from); return 1;`.
pub fn on_craft(w: &mut World, id: ItemId, from: MobileId) -> i32 {
    (class(w, id).craft_init)(w, id, from);
    1
}

/// `MapItem.Validate(from)`.
pub fn validate(w: &mut World, id: ItemId, from: MobileId) -> bool {
    if w.item(id).is_none() || w.mobile(from).is_none() {
        return false;
    }
    let seen = deps::can_see(w, from, id);
    let (Some(item), Some(mob)) = (w.item(id), w.mobile(from)) else {
        return false;
    };
    if !seen
        || mob.map != item.map
        || !muo_server::mobile::base::alive(w, from)
        || ops::in_secure_trade(w, id)
    {
        return false;
    }
    if mob.access_level >= AccessLevel::GameMaster {
        return true;
    }
    let protected = data(w, id).is_some_and(|d| d.protected);
    let loc = ops::get_world_location(w, id);
    if !item.movable() || protected || !deps::mobile_in_range(w, from, loc, 2) {
        return false;
    }
    // !(RootParent is Mobile && RootParent != from)
    !matches!(ops::root_parent(w, id), Some(EntityId::Mobile(m)) if m != from)
}

/// `MapItem.ValidateEdit(from)`.
pub fn validate_edit(w: &mut World, id: ItemId, from: MobileId) -> bool {
    data(w, id).is_some_and(|d| d.editable) && validate(w, id, from)
}

/// `MapItem.Validate(ref x, ref y)`.
pub fn validate_xy(w: &World, id: ItemId, x: i32, y: i32) -> (i32, i32) {
    let (width, height) = data(w, id).map_or((0, 0), |d| (d.width, d.height));
    // Math.Clamp throws when max < min; a map this small is never made.
    (
        x.clamp(0, (width - 1).max(0)),
        y.clamp(0, (height - 1).max(0)),
    )
}

/// `ConvertToWorld(x, y)`.
pub fn convert_to_world(w: &World, id: ItemId, x: i32, y: i32) -> (i32, i32) {
    let d = data(w, id).cloned().unwrap_or_default();
    (
        d.bounds
            .width()
            .wrapping_mul(x)
            .checked_div(d.width)
            .unwrap_or(0)
            + d.bounds.x(),
        d.bounds
            .height()
            .wrapping_mul(y)
            .checked_div(d.height)
            .unwrap_or(0)
            + d.bounds.y(),
    )
}

/// `ConvertToMap(x, y)`; the Y scale uses `Width` too.
pub fn convert_to_map(w: &World, id: ItemId, x: i32, y: i32) -> (i32, i32) {
    let d = data(w, id).cloned().unwrap_or_default();
    (
        (x - d.bounds.x())
            .wrapping_mul(d.width)
            .checked_div(d.bounds.width())
            .unwrap_or(0),
        (y - d.bounds.y())
            .wrapping_mul(d.width)
            .checked_div(d.bounds.height())
            .unwrap_or(0),
    )
}

/// `AddWorldPin(x, y)`.
pub fn add_world_pin(w: &mut World, id: ItemId, x: i32, y: i32) {
    let (mx, my) = convert_to_map(w, id, x, y);
    add_pin(w, id, mx, my);
}

/// `AddPin(x, y)`.
pub fn add_pin(w: &mut World, id: ItemId, x: i32, y: i32) {
    if let Some(d) = data_mut(w, id) {
        d.pins.push(Point2D::new(x, y));
    }
}

/// `RemovePin(index)`; index 0 is never removed (`index > 0`).
pub fn remove_pin(w: &mut World, id: ItemId, index: i32) {
    if let Some(d) = data_mut(w, id)
        && index > 0
        && (index as usize) < d.pins.len()
    {
        d.pins.remove(index as usize);
    }
}

/// `InsertPin(index, x, y)`.
pub fn insert_pin(w: &mut World, id: ItemId, index: i32, x: i32, y: i32) {
    if let Some(d) = data_mut(w, id) {
        if index < 0 || index as usize >= d.pins.len() {
            d.pins.push(Point2D::new(x, y));
        } else {
            d.pins.insert(index as usize, Point2D::new(x, y));
        }
    }
}

/// `ChangePin(index, x, y)`.
pub fn change_pin(w: &mut World, id: ItemId, index: i32, x: i32, y: i32) {
    if let Some(d) = data_mut(w, id)
        && index >= 0
        && (index as usize) < d.pins.len()
    {
        d.pins[index as usize] = Point2D::new(x, y);
    }
}

/// `OnAddPin(from, x, y)`.
pub fn on_add_pin(w: &mut World, id: ItemId, from: MobileId, x: i32, y: i32) {
    if !validate_edit(w, id, from) || data(w, id).is_none_or(|d| d.pins.len() >= MAX_USER_PINS) {
        return;
    }
    let (x, y) = validate_xy(w, id, x, y);
    add_pin(w, id, x, y);
}

/// `OnRemovePin(from, number)`.
pub fn on_remove_pin(w: &mut World, id: ItemId, from: MobileId, number: i32) {
    if validate_edit(w, id, from) {
        remove_pin(w, id, number);
    }
}

/// `OnChangePin(from, number, x, y)`.
pub fn on_change_pin(w: &mut World, id: ItemId, from: MobileId, number: i32, x: i32, y: i32) {
    if !validate_edit(w, id, from) {
        return;
    }
    let (x, y) = validate_xy(w, id, x, y);
    change_pin(w, id, number, x, y);
}

/// `OnInsertPin(from, number, x, y)`.
pub fn on_insert_pin(w: &mut World, id: ItemId, from: MobileId, number: i32, x: i32, y: i32) {
    if !validate_edit(w, id, from) || data(w, id).is_none_or(|d| d.pins.len() >= MAX_USER_PINS) {
        return;
    }
    let (x, y) = validate_xy(w, id, x, y);
    insert_pin(w, id, number, x, y);
}

/// `OnClearPins(from)`.
pub fn on_clear_pins(w: &mut World, id: ItemId, from: MobileId) {
    if validate_edit(w, id, from)
        && let Some(d) = data_mut(w, id)
    {
        d.pins.clear();
    }
}

/// `OnToggleEditable(from)`.
pub fn on_toggle_editable(w: &mut World, id: ItemId, from: MobileId) {
    if validate(w, id, from)
        && let Some(d) = data_mut(w, id)
    {
        d.editable = !d.editable;
    }
    let editable = data(w, id).is_some_and(|d| d.editable) && validate(w, id, from);
    if let Some(ns) = deps::net_state_of(w, from) {
        send_map_set_editable(w, ns, id, editable);
    }
}

// ---- packets ----

/// `SendMapDetails`.
pub fn send_map_details(w: &mut World, ns: NetStateId, map: ItemId) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let new_char_list = w.net_state(ns).is_some_and(|d| d.caps.new_character_list());
    let (Some(serial), Some(d)) = (w.item(map).map(|i| i.serial), data(w, map).cloned()) else {
        return;
    };
    let mut p = Pk::new(if new_char_list { 21 } else { 19 });
    p.u8(if new_char_list { 0xF5 } else { 0x90 }); // Packet ID
    p.serial(serial);
    p.i16(0x139D);
    p.i16(d.bounds.start.x as i16);
    p.i16(d.bounds.start.y as i16);
    p.i16(d.bounds.end.x as i16);
    p.i16(d.bounds.end.y as i16);
    p.i16(d.width as i16);
    p.i16(d.height as i16);
    if new_char_list {
        let map_id = d.facet.and_then(|f| w.maps.get(f)).map_or(0, |m| m.map_id);
        p.i16(map_id as i16);
    }
    let bytes = p.finish();
    w.send(ns, &bytes);
}

/// `SendMapCommand`.
pub fn send_map_command(
    w: &mut World,
    ns: NetStateId,
    map: ItemId,
    command: u8,
    x: i32,
    y: i32,
    editable: bool,
) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let Some(serial) = w.item(map).map(|i| i.serial) else {
        return;
    };
    let mut p = Pk::new(11);
    p.u8(0x56); // Packet ID
    p.serial(serial);
    p.u8(command);
    p.bool(editable);
    p.i16(x as i16);
    p.i16(y as i16);
    let bytes = p.finish();
    w.send(ns, &bytes);
}

/// `SendMapDisplay`.
pub fn send_map_display(w: &mut World, ns: NetStateId, map: ItemId) {
    send_map_command(w, ns, map, 5, 0, 0, false);
}

/// `SendMapAddPin`.
pub fn send_map_add_pin(w: &mut World, ns: NetStateId, map: ItemId, p: Point2D) {
    send_map_command(w, ns, map, 1, p.x, p.y, false);
}

/// `SendMapSetEditable`: `true` is sent whatever `editable` is.
pub fn send_map_set_editable(w: &mut World, ns: NetStateId, map: ItemId, _editable: bool) {
    send_map_command(w, ns, map, 7, 0, 0, true);
}

/// `MapItemPackets.OnMapCommand` (0x56).
fn on_map_command(w: &mut World, ns: &mut muo_net::net_state::NetState, body: &[u8]) {
    let id = muo_server::net_glue::ensure_net_state(w, ns);
    let Some(from) = w.net_state(id).and_then(|d| d.mobile) else {
        return;
    };
    let mut r = SpanReader::new(body);
    let read = (|| {
        let serial = r.read_u32().ok()?;
        let command = r.read_byte().ok()?;
        let number = i32::from(r.read_byte().ok()?);
        let x = i32::from(r.read_i16().ok()?);
        let y = i32::from(r.read_i16().ok()?);
        Some((serial, command, number, x, y))
    })();
    let Some((serial, command, number, x, y)) = read else {
        return;
    };
    let Some(map) = w
        .find_item(Serial::new(serial))
        .filter(|m| is_map_item(w, *m))
    else {
        return;
    };
    match command {
        1 => on_add_pin(w, map, from, x, y),
        2 => on_insert_pin(w, map, from, number, x, y),
        3 => on_change_pin(w, map, from, number, x, y),
        4 => on_remove_pin(w, map, from, number),
        5 => on_clear_pins(w, map, from),
        6 => on_toggle_editable(w, map, from),
        _ => {}
    }
}

fn configure(w: &mut World) {
    muo_content::network::register_packet(w, 0x56, 11, true, false, on_map_command);
    // What a boat asks of a map: its pins, the pin to world conversion, and `BlankMap`
    let b = w.resources.get_mut::<muo_multis::boats::BoatHooks>();
    b.map_pins = Some(|w, id| {
        is_map_item(w, id).then(|| data(w, id).map(|d| d.pins.clone()).unwrap_or_default())
    });
    b.map_convert_to_world = Some(convert_to_world);
    b.is_blank_map = Some(|w, id| w.item(id).is_some_and(|i| i.ty.is(&BLANK_MAP)));
    // `item is ICraftable`: the craft system calls `OnCraft` of a map made by cartography
    muo_content::craft::hooks_mut(w)
        .craftables
        .push(|w, id, a| is_map_item(w, id).then(|| on_craft(w, id, a.from)));
}

// ---- display ----

pub(crate) fn display_to_base(w: &mut World, id: ItemId, from: MobileId) {
    let Some(ns) = deps::net_state_of(w, from) else {
        return;
    };
    let d = data(w, id).cloned().unwrap_or_default();
    let new_char_list = w.net_state(ns).is_some_and(|n| n.caps.new_character_list());
    if !new_char_list
        && d.facet
            .is_some_and(|f| f != MapId::FELUCCA && f != MapId::TRAMMEL)
    {
        muo_server::mobile::part2::messages::send_message(
            w,
            from,
            "You must have client 7.0.13.0 or higher to display this map.",
        );
        return;
    }
    send_map_details(w, ns, id);
    send_map_display(w, ns, id);
    for p in &d.pins {
        send_map_add_pin(w, ns, id, *p);
    }
    let editable = validate_edit(w, id, from);
    send_map_set_editable(w, ns, id, editable);
}

/// `MapItem.DisplayTo(from)` through the class.
pub fn display_to(w: &mut World, id: ItemId, from: MobileId) {
    (class(w, id).display_to)(w, id, from);
}

fn craft_init_base(_w: &mut World, _id: ItemId, _from: MobileId) {}

// ---- serialization ----

const MAP_ITEM_VERSION: i32 = 1;

fn serialize_map_item(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    (ITEM_BASE.serialize)(w, id, out);
    out.write_i32(MAP_ITEM_VERSION);
    let d = data(w, id).cloned().unwrap_or_default();
    out.write_rect2d(d.bounds);
    out.write_i32(d.width);
    out.write_i32(d.height);
    out.write_bool(d.protected);
    out.write_encoded_int(d.pins.len() as i32);
    for p in &d.pins {
        out.write_point2d(*p);
    }
    out.write_map(d.facet);
    out.write_bool(d.editable);
}

fn deserialize_map_item(w: &mut World, id: ItemId, r: &mut dyn GenericReader) -> Result<()> {
    (ITEM_BASE.deserialize)(w, id, r)?;
    let version = r.read_i32()?;
    let mut d = MapItemData {
        bounds: r.read_rect2d()?,
        width: r.read_i32()?,
        height: r.read_i32()?,
        ..Default::default()
    };
    d.protected = r.read_bool()?;
    // Version 0 doesn't serialize Facet/Editable, and the count is not encoded.
    let count = if version < MAP_ITEM_VERSION {
        r.read_i32()?
    } else {
        r.read_encoded_int()?
    };
    if count < 0 {
        return Err(IoError::InvalidData(format!("MapItem pins: {count}")));
    }
    for _ in 0..count {
        d.pins.push(r.read_point2d()?);
    }
    if version >= MAP_ITEM_VERSION {
        d.facet = r.read_map()?;
        d.editable = r.read_bool()?;
    }
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(d);
    }
    Ok(())
}

// ---- classes ----

pub static MAP_ITEM_CLASS: MapItemClass = MapItemClass {
    craft_init: craft_init_base,
    display_to: display_to_base,
};

/// `Server.Items.MapItem`; `[Flippable(0x14EB, 0x14EC)]`.
pub static MAP_ITEM: ItemType = ItemType {
    full_name: "Server.Items.MapItem",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| map_item_new(w, &MAP_ITEM, None)),
    family: Some(&MAP_ITEM_CLASS),
    default_weight: |_, _| 1.0,
    on_double_click: |w, id, from| {
        let loc = ops::get_world_location(w, id);
        if deps::mobile_in_range(w, from, loc, 2) {
            display_to(w, id, from);
        } else {
            deps::send_localized_message(w, from, 500446); // That is too far away.
        }
    },
    serialize: serialize_map_item,
    deserialize: deserialize_map_item,
    ..ITEM_BASE
};

/// `[Flippable]` data of this folder: the classes and their item ids.
pub static FLIPPABLE: &[(&ItemType, &[i32])] = &[(&MAP_ITEM, &[0x14EB, 0x14EC])];

/// `Hue`-only helper kept here so the classes below stay short.
fn skill(w: &World, from: MobileId) -> f64 {
    muo_server::skills::value_ref(w, from, SkillName::Cartography)
}

fn from_xy(w: &World, from: MobileId) -> (i32, i32) {
    w.mobile(from)
        .map_or((0, 0), |m| (m.location.x, m.location.y))
}

macro_rules! map_leaf {
    (
        $(#[$m:meta])*
        $name:ident, $class:ident = $full:literal, encoded $enc:literal;
        $(@init |$w:ident, $id:ident| $init:block;)?
        $(label $label:literal;)?
        $(craft |$cw:ident, $cid:ident, $cfrom:ident| $craft:block;)?
        $(@ty { $($tf:ident : $tv:expr,)* };)?
    ) => {
        pub static $class: MapItemClass = MapItemClass {
            $(craft_init: |$cw, $cid, $cfrom| $craft,)?
            ..MAP_ITEM_CLASS
        };

        $(#[$m])*
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&MAP_ITEM),
            construct: Some(|w| {
                let id = map_item_new(w, &$name, None);
                $( let init = |$w: &mut World, $id: ItemId| $init; init(w, id); )?
                id
            }),
            family: Some(&$class),
            $(label_number: |_, _| $label,)?
            $($($tf: $tv,)*)?
            serialize: |w, id, out| {
                serialize_map_item(w, id, out);
                write_version(out, $enc);
            },
            deserialize: |w, id, r| {
                deserialize_map_item(w, id, r)?;
                read_version(r, $enc)?;
                Ok(())
            },
            ..MAP_ITEM
        };
    };
}

map_leaf! {
    /// `Server.Items.BlankMap`.
    BLANK_MAP, BLANK_MAP_CLASS = "Server.Items.BlankMap", encoded false;
    @ty {
        on_double_click: |w, id, from| messages::send_localized_message_to(w, id, from, 500208, ""), // It appears to be blank.
    };
}

map_leaf! {
    /// `Server.Items.CityMap`.
    CITY_MAP, CITY_MAP_CLASS = "Server.Items.CityMap", encoded false;
    @init |w, id| { set_display(w, id, 0, 0, 5119, 4095, 400, 400); };
    label 1015231; // city map
    craft |w, id, from| {
        let skill_value = skill(w, from);
        let dist = (64 + (skill_value * 4.0) as i32).max(200);
        let size = (32 + (skill_value * 2.0) as i32).clamp(200, 400);
        let (x, y) = from_xy(w, from);
        set_display(w, id, x - dist, y - dist, x + dist, y + dist, size, size);
    };
}

map_leaf! {
    /// `Server.Items.LocalMap`.
    LOCAL_MAP, LOCAL_MAP_CLASS = "Server.Items.LocalMap", encoded false;
    @init |w, id| { set_display(w, id, 0, 0, 5119, 4095, 400, 400); };
    label 1015230; // local map
    craft |w, id, from| {
        let dist = 64 + (skill(w, from) * 2.0) as i32;
        let (x, y) = from_xy(w, from);
        set_display(w, id, x - dist, y - dist, x + dist, y + dist, 200, 200);
    };
}

map_leaf! {
    /// `Server.Items.SeaChart`.
    SEA_CHART, SEA_CHART_CLASS = "Server.Items.SeaChart", encoded false;
    @init |w, id| { set_display(w, id, 0, 0, 5119, 4095, 400, 400); };
    label 1015232; // sea chart
    craft |w, id, from| {
        let skill_value = skill(w, from);
        let dist = (64 + (skill_value * 10.0) as i32).max(200);
        let size = (24 + (skill_value * 3.3) as i32).clamp(200, 400);
        let (x, y) = from_xy(w, from);
        set_display(w, id, x - dist, y - dist, x + dist, y + dist, size, size);
    };
}

map_leaf! {
    /// `Server.Items.WorldMap`.
    WORLD_MAP, WORLD_MAP_CLASS = "Server.Items.WorldMap", encoded true;
    @init |w, id| { set_display(w, id, 0, 0, 5119, 4095, 400, 400); };
    label 1015233; // world map
    craft |w, id, from| {
        // Unlike the others, world map is not based on crafted location
        let skill_value = skill(w, from);
        let x20 = (skill_value * 20.0) as i32;
        let size = (25 + (skill_value * 6.6) as i32).clamp(200, 400);
        set_display(w, id, 1344 - x20, 1600 - x20, 1472 + x20, 1728 + x20, size, size);
    };
}

map_leaf! {
    /// `Server.Items.IndecipherableMap`.
    INDECIPHERABLE_MAP, INDECIPHERABLE_MAP_CLASS = "Server.Items.IndecipherableMap", encoded true;
    @init |w, id| {
        let ruined = utility::random_double(&mut w.rng) < 0.2;
        hue(w, id, if ruined { 0x965 } else { 0x961 });
    };
    label 1070799; // indecipherable map
    @ty {
        on_double_click: |w, _, from| deps::send_localized_message(w, from, 1070801), // You cannot decipher this ruined map.
    };
}

// ---- WeatherMap ----

/// `WeatherMap.OnDoubleClick`: a pin at the middle of every weather area of the facet, then `MapItem`'s.
fn weather_map_double_click(w: &mut World, id: ItemId, from: MobileId) {
    let Some(facet) = w.mobile(from).and_then(|m| m.map) else {
        return;
    };
    if let Some(d) = data_mut(w, id) {
        d.pins.clear();
    }
    for (x, y) in muo_content::misc::weather::pin_positions(w, facet) {
        add_world_pin(w, id, x, y);
    }
    (MAP_ITEM.on_double_click)(w, id, from);
}

map_leaf! {
    /// `Server.Misc.WeatherMap` (`[SerializationGenerator(0)]`).
    WEATHER_MAP, WEATHER_MAP_CLASS = "Server.Misc.WeatherMap", encoded true;
    @init |w, id| { set_display(w, id, 0, 0, 5119, 4095, 400, 400); };
    @ty {
        default_name: |_, _| Some("weather map".to_string()),
        on_double_click: weather_map_double_click,
    };
}

// ---- PresetMap ----

/// `PresetMapEntry`.
pub struct PresetMapEntry {
    pub name: i32,
    pub width: i32,
    pub height: i32,
    pub bounds: Rectangle2D,
}

const fn entry(
    name: i32,
    width: i32,
    height: i32,
    x_left: i32,
    y_top: i32,
    x_right: i32,
    y_bottom: i32,
) -> PresetMapEntry {
    PresetMapEntry {
        name,
        width,
        height,
        bounds: Rectangle2D::new(x_left, y_top, x_right - x_left, y_bottom - y_top),
    }
}

/// `PresetMapEntry.Table`, indexed by `PresetMapType`.
pub static PRESET_MAP_TABLE: [PresetMapEntry; 28] = [
    entry(1041189, 200, 200, 1092, 1396, 1736, 1924), // map of Britain
    entry(1041203, 200, 200, 256, 1792, 1736, 2560),  // map of Britain to Skara Brae
    entry(1041192, 200, 200, 1024, 1280, 2304, 3072), // map of Britain to Trinsic
    entry(1041183, 200, 200, 2500, 1900, 3000, 2400), // map of Buccaneer's Den
    entry(1041198, 200, 200, 2560, 1792, 3840, 2560), // map of Buccaneer's Den to Magincia
    entry(1041194, 200, 200, 2560, 1792, 3840, 3072), // map of Buccaneer's Den to Ocllo
    entry(1041181, 200, 200, 1088, 3572, 1528, 4056), // map of Jhelom
    entry(1041186, 200, 200, 3530, 2022, 3818, 2298), // map of Magincia
    entry(1041199, 200, 200, 3328, 1792, 3840, 2304), // map of Magincia to Ocllo
    entry(1041182, 200, 200, 2360, 356, 2706, 702),   // map of Minoc
    entry(1041190, 200, 200, 0, 256, 2304, 3072),     // map of Minoc to Yew
    entry(1041191, 200, 200, 2467, 572, 2878, 746),   // map of Minoc to Vesper
    entry(1041188, 200, 200, 4156, 808, 4732, 1528),  // map of Moonglow
    entry(1041201, 200, 200, 3328, 768, 4864, 1536),  // map of Moonglow to Nujelm
    entry(1041185, 200, 200, 3446, 1030, 3832, 1424), // map of Nujelm
    entry(1041197, 200, 200, 3328, 1024, 3840, 2304), // map of Nujelm to Magincia
    entry(1041187, 200, 200, 3582, 2456, 3770, 2742), // map of Ocllo
    entry(1041184, 200, 200, 2714, 3329, 3100, 3639), // map of Serpent's Hold
    entry(1041200, 200, 200, 2560, 2560, 3840, 3840), // map of Serpent's Hold to Ocllo
    entry(1041180, 200, 200, 524, 2064, 960, 2452),   // map of Skara Brae
    entry(1041204, 200, 200, 0, 0, 5199, 4095),       // map of The World
    entry(1041177, 200, 200, 1792, 2630, 2118, 2952), // map of Trinsic
    entry(1041193, 200, 200, 1792, 1792, 3072, 3072), // map of Trinsic to Buccaneer's Den
    entry(1041195, 200, 200, 256, 1792, 2304, 4095),  // map of Trinsic to Jhelom
    entry(1041178, 200, 200, 2636, 592, 3064, 1012),  // map of Vesper
    entry(1041196, 200, 200, 2636, 592, 3840, 1536),  // map of Vesper to Nujelm
    entry(1041179, 200, 200, 236, 741, 766, 1269),    // map of Yew
    entry(1041202, 200, 200, 0, 512, 1792, 2048),     // map of Yew to Britain
];

/// `PresetMap.InitEntry(entry)`.
pub fn preset_map_init_entry(w: &mut World, id: ItemId, entry: &PresetMapEntry) {
    if let Some(d) = data_mut(w, id) {
        d.preset_label = entry.name;
        d.width = entry.width;
        d.height = entry.height;
        d.bounds = entry.bounds;
    }
}

/// `PresetMap(PresetMapType type)`: a type outside the table builds a blank preset map.
pub fn preset_map_new(w: &mut World, ty: &'static ItemType, kind: i32) -> ItemId {
    let id = map_item_new(w, ty, None);
    if kind >= 0 && (kind as usize) < PRESET_MAP_TABLE.len() {
        preset_map_init_entry(w, id, &PRESET_MAP_TABLE[kind as usize]);
    }
    id
}

/// `Server.Items.PresetMap`.
pub static PRESET_MAP: ItemType = ItemType {
    full_name: "Server.Items.PresetMap",
    parent: Some(&MAP_ITEM),
    construct: None, // `[add` uses `PRESET_MAP_CTORS`
    label_number: |w, id| match data(w, id).map_or(0, |d| d.preset_label) {
        0 => base::label_number_base(w, id),
        n => n,
    },
    serialize: |w, id, out| {
        serialize_map_item(w, id, out);
        write_version(out, false);
        out.write_i32(data(w, id).map_or(0, |d| d.preset_label));
    },
    deserialize: |w, id, r| {
        deserialize_map_item(w, id, r)?;
        read_version(r, false)?;
        let label = r.read_i32()?;
        if let Some(d) = data_mut(w, id) {
            d.preset_label = label;
        }
        Ok(())
    },
    ..MAP_ITEM
};

/// The classes of this file (`TreasureMap` is in `treasure.rs`).
pub static ALL: &[&ItemType] = &[
    &MAP_ITEM,
    &BLANK_MAP,
    &CITY_MAP,
    &LOCAL_MAP,
    &SEA_CHART,
    &WORLD_MAP,
    &INDECIPHERABLE_MAP,
    &WEATHER_MAP,
    &PRESET_MAP,
];

const PRESET_MAP_NAMES: &[(&str, i64)] = &[
    ("Britain", 0),
    ("BritainToSkaraBrae", 1),
    ("BritainToTrinsic", 2),
    ("BucsDen", 3),
    ("BucsDenToMagincia", 4),
    ("BucsDenToOcllo", 5),
    ("Jhelom", 6),
    ("Magincia", 7),
    ("MaginciaToOcllo", 8),
    ("Minoc", 9),
    ("MinocToYew", 10),
    ("MinocToVesper", 11),
    ("Moonglow", 12),
    ("MoonglowToNujelm", 13),
    ("Nujelm", 14),
    ("NujelmToMagincia", 15),
    ("Ocllo", 16),
    ("SerpentsHold", 17),
    ("SerpentsHoldToOcllo", 18),
    ("SkaraBrae", 19),
    ("TheWorld", 20),
    ("Trinsic", 21),
    ("TrinsicToBucsDen", 22),
    ("TrinsicToJhelom", 23),
    ("Vesper", 24),
    ("VesperToNujelm", 25),
    ("Yew", 26),
    ("YewToBritain", 27),
];

/// `[Constructible] PresetMap(PresetMapType type)`.
static PRESET_MAP_CTORS: [Ctor; 1] = [Ctor {
    access: None,
    params: &[Param {
        name: "type",
        kind: Kind::Plain(PropType::Enum(PRESET_MAP_NAMES)),
        default: None,
        ty: "PresetMapType",
    }],
    make: |w, v| EntityId::Item(preset_map_new(w, &PRESET_MAP, v[0].i32())),
}];

/// `[Constructible] TreasureMap(int level, Map map)`.
static TREASURE_MAP_CTORS: [Ctor; 1] = [Ctor {
    access: None,
    params: &[
        Param {
            name: "level",
            kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
            default: None,
            ty: "",
        },
        Param {
            name: "map",
            kind: Kind::Map,
            default: None,
            ty: "",
        },
    ],
    make: |w, v| {
        let map = match &v[1] {
            Value::Map(m) => *m,
            _ => None,
        };
        EntityId::Item(treasure::treasure_map_new(
            w,
            &treasure::TREASURE_MAP,
            v[0].i32(),
            map,
        ))
    },
}];

pub(crate) fn ctors_configure(w: &mut World) {
    add_ctors(w, "Server.Items.PresetMap", &PRESET_MAP_CTORS);
    add_ctors(w, "Server.Items.TreasureMap", &TREASURE_MAP_CTORS);
}

/// Registers the classes of this folder.
pub fn register(types: &mut TypeRegistry) {
    types.register_phase_default(
        Phase::Configure,
        "Server.Items.MapItem.Constructors",
        ctors_configure,
    );
    for t in ALL {
        types.register_item(t, &[]);
    }
    treasure::register(types);
    types.register_phase_default(
        Phase::Configure,
        "Server.Network.MapItemPackets.Configure",
        configure,
    );
}
