//! `Runebook`, `RunebookEntry`, `RecallRune` and
//! `Sextant` (the item and its `Format`/`ReverseLookup`). Also the `RunebookOps` the
//! `RunebookGump` reads the book through, and the `RecallHooks` the recall spells ask the book's charges of.
//!
//! // shortcut: `IsOpen`/`CloseGump<RunebookGump>` track the book's `Openers` (which `SendGumpTo` fills and the gump's
//! response empties) instead of searching the mobile's open gumps; a lifted book closes nothing on screen.

use std::rc::Rc;

use muo_content::items::equip_hooks::CraftArgs;
use muo_content::items::house_deps;
use muo_content::props::{Ctor, Kind, Param, add_ctors};
use muo_content::regions::house::house_ops;
use muo_content::utilities::types::{Numeric, PropType};
use muo_gumps::m_z::ops::{RuneSpell, RunebookEntryView, RunebookOps, RunebookView, SextantView};
use muo_io::{GenericReader, GenericWriter, Result};
use muo_primitives::{DateTime, Point3D, TimeSpan};
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, LootType, base, deps, ops};
use muo_server::layer::Layer;
use muo_server::map::MapId;
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part2::messages;
use muo_server::property_list::PropertyList;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::type_registry::{Phase, TypeRegistry};

use super::hue;

// ---- Sextant ----

/// `Sextant.ComputeMapDetails(map, x, y, ...)`: `(xCenter, yCenter, xWidth, yHeight)`.
pub fn compute_map_details(w: &World, map: MapId, x: i32, y: i32) -> Option<(i32, i32, i32, i32)> {
    let (xw, yh) = (5120, 4096);
    if map == MapId(0) || map == MapId(1) {
        // Felucca, Trammel
        if (0..5120).contains(&x) && (0..4096).contains(&y) {
            Some((1323, 1624, xw, yh))
        } else if (5120..6144).contains(&x) && (2304..4096).contains(&y) {
            Some((5936, 3112, xw, yh))
        } else {
            None
        }
    } else {
        let m = w.maps.get(map)?;
        (x >= 0 && y >= 0 && x < m.width && y < m.height).then_some((1323, 1624, xw, yh))
    }
}

/// `Sextant.Format(p, map, ...)`.
pub fn sextant_format(w: &World, p: Point3D, map: Option<MapId>) -> Option<SextantView> {
    let map = map.filter(|&m| m != MapId::INTERNAL)?;
    let (xc, yc, xw, yh) = compute_map_details(w, map, p.x, p.y)?;
    let mut abs_long = f64::from((p.x - xc) * 360) / f64::from(xw);
    let mut abs_lat = f64::from((p.y - yc) * 360) / f64::from(yh);
    if abs_long > 180.0 {
        abs_long = -180.0 + abs_long % 180.0;
    }
    if abs_lat > 180.0 {
        abs_lat = -180.0 + abs_lat % 180.0;
    }
    let (east, south) = (abs_long >= 0.0, abs_lat >= 0.0);
    abs_long = abs_long.abs();
    abs_lat = abs_lat.abs();
    Some(SextantView {
        x_long: abs_long as i32,
        y_lat: abs_lat as i32,
        x_mins: (abs_long % 1.0 * 60.0) as i32,
        y_mins: (abs_lat % 1.0 * 60.0) as i32,
        x_east: east,
        y_south: south,
    })
}

/// `Sextant.ReverseLookup(map, xLong, yLat, xMins, yMins, xEast, ySouth)`; `Point3D.Zero` on failure.
#[allow(clippy::too_many_arguments)]
pub fn sextant_reverse_lookup(
    w: &mut World,
    map: MapId,
    x_long: i32,
    y_lat: i32,
    x_mins: i32,
    y_mins: i32,
    x_east: bool,
    y_south: bool,
) -> Point3D {
    if map == MapId::INTERNAL {
        return Point3D::default();
    }
    let Some((xc, yc, xw, yh)) = compute_map_details(w, map, 0, 0) else {
        return Point3D::default();
    };
    let mut abs_long = f64::from(x_long) + f64::from(x_mins) / 60.0;
    let mut abs_lat = f64::from(y_lat) + f64::from(y_mins) / 60.0;
    if !x_east {
        abs_long = 360.0 - abs_long;
    }
    if !y_south {
        abs_lat = 360.0 - abs_lat;
    }
    let mut x = xc + (abs_long * f64::from(xw) / 360.0) as i32;
    let mut y = yc + (abs_lat * f64::from(yh) / 360.0) as i32;
    if x < 0 {
        x += xw;
    } else if x >= xw {
        x -= xw;
    }
    if y < 0 {
        y += yh;
    } else if y >= yh {
        y -= yh;
    }
    let z = w.maps.get_average_z(map, x, y);
    Point3D::new(x, y, z)
}

/// The house sign and house list format: both coordinates in degrees (the Sextant item and the runebook say `xLong'`).
fn house_sextant_text(s: &SextantView) -> String {
    format!(
        "{}° {}'{}, {}° {}'{}",
        s.y_lat,
        s.y_mins,
        if s.y_south { "S" } else { "N" },
        s.x_long,
        s.x_mins,
        if s.x_east { "E" } else { "W" }
    )
}

fn sextant_text(s: &SextantView) -> String {
    format!(
        "{}° {}'{}, {}' {}'{}",
        s.y_lat,
        s.y_mins,
        if s.y_south { "S" } else { "N" },
        s.x_long,
        s.x_mins,
        if s.x_east { "E" } else { "W" }
    )
}

/// `Server.Items.Sextant`.
pub static SEXTANT: ItemType = ItemType {
    full_name: "Server.Items.Sextant",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| ops::new_item_with_id(w, &SEXTANT, 0x1058)),
    default_weight: crate::class_props::default_weight,
    on_double_click: |w, _id, from| {
        let Some((loc, map)) = w.mobile(from).map(|m| (m.location, m.map)) else {
            return;
        };
        if let Some(s) = sextant_format(w, loc, map) {
            let hue = w.mobile(from).map_or(0, |m| m.speech_hue);
            messages::local_overhead_message(
                w,
                from,
                muo_primitives::MessageType::REGULAR,
                hue,
                false,
                &sextant_text(&s),
            );
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..ITEM_BASE
};

// ---- RecallRune ----

/// `RecallRune`' fields.
#[derive(Clone, Debug, Default)]
pub struct RuneData {
    pub description: Option<String>,
    pub house: Option<ItemId>,
    pub marked: bool,
    pub target: Point3D,
    pub target_map: Option<MapId>,
}

pub fn rune(w: &World, id: ItemId) -> Option<&RuneData> {
    w.item(id)?.ext.get::<RuneData>()
}

fn rune_mut(w: &mut World, id: ItemId) -> Option<&mut RuneData> {
    w.item_mut(id)?.ext.get_mut::<RuneData>()
}

/// `House` getter: a deleted house reads as null (and is cleared).
fn rune_house(w: &mut World, id: ItemId) -> Option<ItemId> {
    let h = rune(w, id)?.house;
    if h.is_some_and(|h| w.item(h).is_none_or(|i| i.deleted())) {
        if let Some(d) = rune_mut(w, id) {
            d.house = None;
        }
        calculate_hue(w, id);
        ops::invalidate_properties(w, id);
        return None;
    }
    h
}

/// `CalculateHue()`.
fn calculate_hue(w: &mut World, id: ItemId) {
    let Some(d) = rune(w, id).cloned() else {
        return;
    };
    let house = d.house.is_some();
    let h = if !d.marked {
        Some(0)
    } else {
        match d.target_map.map(|m| m.0) {
            Some(1) => Some(if house { 0x47F } else { 50 }),
            Some(0) => Some(if house { 0x66D } else { 0 }),
            Some(2) | Some(3) => Some(if house { 0x55F } else { 1102 }),
            Some(4) => Some(if house { 0x47F } else { 1154 }),
            _ => None,
        }
    };
    if let Some(h) = h {
        hue(w, id, h);
    }
}

/// `new RecallRune()`.
pub fn new_rune(w: &mut World) -> ItemId {
    let id = crate::simple::from_template(w, &RECALL_RUNE, "Server.Items.RecallRune");
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(RuneData::default());
    }
    calculate_hue(w, id);
    id
}

/// `Description = value` (`[InvalidateProperties]`).
pub fn set_rune_description(w: &mut World, id: ItemId, text: Option<String>) {
    if let Some(d) = rune_mut(w, id)
        && d.description != text
    {
        d.description = text;
        ops::invalidate_properties(w, id);
    }
}

/// `Marked = value` (`OnMarkedChanged` recalculates the hue; `[InvalidateProperties]`).
pub fn set_marked(w: &mut World, id: ItemId, value: bool) {
    if rune(w, id).is_some_and(|d| d.marked != value) {
        if let Some(d) = rune_mut(w, id) {
            d.marked = value;
        }
        calculate_hue(w, id);
        ops::invalidate_properties(w, id);
    }
}

/// `TargetMap = value`.
pub fn set_target_map(w: &mut World, id: ItemId, value: Option<MapId>) {
    if rune(w, id).is_some_and(|d| d.target_map != value) {
        if let Some(d) = rune_mut(w, id) {
            d.target_map = value;
        }
        calculate_hue(w, id);
        ops::invalidate_properties(w, id);
    }
}

/// `House = value`: the setter recalculates the hue and invalidates properties.
pub fn set_house(w: &mut World, id: ItemId, value: Option<ItemId>) {
    if let Some(d) = rune_mut(w, id) {
        d.house = value;
    }
    calculate_hue(w, id);
    ops::invalidate_properties(w, id);
}

/// `Target = value`.
pub fn set_target(w: &mut World, id: ItemId, value: Point3D) {
    if let Some(d) = rune_mut(w, id) {
        d.target = value;
    }
}

/// `Mark(m)`.
pub fn mark(w: &mut World, id: ItemId, m: MobileId) {
    if let Some(d) = rune_mut(w, id) {
        d.marked = true;
    }
    let mut set_desc = false;
    let (mloc, mmap) = w
        .mobile(m)
        .map_or((Point3D::default(), None), |x| (x.location, x.map));
    if muo_content::core::aos(w) {
        let house = house_ops(w).and_then(|o| o.find_house_at(w, m));
        if let Some(d) = rune_mut(w, id) {
            d.house = house;
        }
        match house {
            None => {
                if let Some(d) = rune_mut(w, id) {
                    d.target = mloc;
                    d.target_map = mmap;
                }
            }
            Some(h) => {
                let o = house_ops(w).expect("a house came from it");
                let name = o
                    .sign(w, h)
                    .and_then(|s| ops::name(w, s))
                    .map(|n| n.trim().to_string())
                    .filter(|n| !n.is_empty())
                    .unwrap_or_else(|| "an unnamed house".to_string());
                set_rune_description(w, id, Some(name));
                set_desc = true;
                let ban = o.ban_location(w, h);
                let (x, mut y, mut z) = (ban.x, ban.y, ban.z);
                y += 2;
                let map = o.map(w, h);
                if let Some(mp) = map
                    && !muo_server::map::fit::can_fit(w, mp, x, y, z, 16, false, false, true)
                {
                    z = w.maps.get_average_z(mp, x, y);
                }
                let _ = &mut y;
                if let Some(d) = rune_mut(w, id) {
                    d.target = Point3D::new(x, y, z);
                    d.target_map = map;
                }
            }
        }
    } else if let Some(d) = rune_mut(w, id) {
        d.house = None;
        d.target = mloc;
        d.target_map = mmap;
    }
    if !set_desc {
        let (t, tm) = rune(w, id).map_or((Point3D::default(), None), |d| (d.target, d.target_map));
        let region = muo_server::region::find(w, t, tm);
        let name = muo_content::regions::base_region::get_rune_name_for(w, Some(region));
        set_rune_description(w, id, name);
    }
    calculate_hue(w, id);
    ops::invalidate_properties(w, id);
}

fn map_name(w: &World, m: Option<MapId>) -> String {
    m.and_then(|m| w.maps.get(m))
        .map_or_else(String::new, |m| m.name().to_string())
}

/// `(number, text)` the rune shows for properties and single click, or `Err(text)` for a plain line.
fn rune_label(w: &mut World, id: ItemId) -> Option<std::result::Result<(i32, String), String>> {
    let d = rune(w, id)?.clone();
    if !d.marked {
        return None;
    }
    let desc = d
        .description
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("an unknown location");
    let text = format!("a recall rune for {desc}");
    let house = rune_house(w, id).is_some();
    let h = |a: i32, b: i32| if house { a } else { b };
    Some(match d.target_map.map(|m| m.0) {
        Some(4) => Ok((h(1063260, 1063259), text)), // ~1_val~ (Tokuno Islands)[(House)]
        Some(3) => Ok((h(1062454, 1060804), text)), // ~1_val~ (Malas)[(House)]
        Some(0) => Ok((h(1062452, 1060805), text)), // ~1_val~ (Felucca)[(House)]
        Some(1) => Ok((h(1062453, 1060806), text)), // ~1_val~ (Trammel)[(House)]
        _ => {
            let name = map_name(w, d.target_map);
            Err(if house {
                format!("{text} ({name})(House)")
            } else {
                format!("{text} ({name})")
            })
        }
    })
}

fn rune_get_properties(w: &World, id: ItemId, list: &mut PropertyList) {
    base::get_properties_base(w, id, list);
    let Some(d) = rune(w, id) else { return };
    if !d.marked {
        return;
    }
    let desc = d
        .description
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("an unknown location");
    let text = format!("a recall rune for {desc}");
    let house = d
        .house
        .is_some_and(|h| w.item(h).is_some_and(|i| !i.deleted()));
    let h = |a: i32, b: i32| if house { a } else { b };
    match d.target_map.map(|m| m.0) {
        Some(4) => list.add_str(h(1063260, 1063259), &text),
        Some(3) => list.add_str(h(1062454, 1060804), &text),
        Some(0) => list.add_str(h(1062452, 1060805), &text),
        Some(1) => list.add_str(h(1062453, 1060806), &text),
        _ => {
            let name = map_name(w, d.target_map);
            list.add_text(&if house {
                format!("{text} ({name})(House)")
            } else {
                format!("{text} ({name})")
            });
        }
    }
}

fn rune_on_single_click(w: &mut World, id: ItemId, from: MobileId) {
    match rune_label(w, id) {
        None => ops::label_to(w, id, from, "an unmarked recall rune"),
        Some(Ok((n, text))) => ops::label_to_number(w, id, from, n, &text),
        Some(Err(text)) => ops::label_to(w, id, from, &text),
    }
}

fn rune_on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    let pack = w.mobile(from).and_then(|m| m.backpack);
    let number = if !pack.is_some_and(|p| ops::is_child_of(w, id, Some(EntityId::Item(p)))) {
        1042001 // That must be in your pack for you to use it.
    } else if rune_house(w, id).is_some() {
        1062399 // You cannot edit the description for this rune.
    } else if rune(w, id).is_some_and(|d| d.marked) {
        let prompt = muo_server::prompt::Prompt {
            on_response: Some(Rc::new(move |w: &mut World, from: MobileId, text: &str| {
                if rune_house(w, id).is_none() && rune(w, id).is_some_and(|d| d.marked) {
                    set_rune_description(w, id, Some(text.to_string()));
                    deps::send_localized_message(w, from, 1010474); // The etching on the rune has been changed.
                }
            })),
            on_cancel: None,
        };
        let pid = muo_server::prompt::add_prompt(w, prompt);
        muo_server::mobile::part1::props_b::set_prompt(w, from, Some(pid));
        501804 // Please enter a description for this marked object.
    } else {
        501805 // That rune is not yet marked.
    };
    deps::send_localized_message(w, from, number);
}

fn rune_serialize(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    (ITEM_BASE.serialize)(w, id, out);
    out.write_i32(2);
    let d = rune(w, id).cloned().unwrap_or_default();
    // The house getter clears a deleted house before it is written.
    let house = d.house.filter(|&h| w.item(h).is_some_and(|i| !i.deleted()));
    out.write_item(w, house);
    out.write_opt_string(d.description.as_deref());
    out.write_bool(d.marked);
    out.write_point3d(d.target);
    out.write_map(d.target_map);
}

fn rune_deserialize(w: &mut World, id: ItemId, r: &mut dyn GenericReader) -> Result<()> {
    (ITEM_BASE.deserialize)(w, id, r)?;
    let version = r.read_i32()?;
    let mut d = RuneData::default();
    if version >= 2 {
        d.house = r.read_item(w)?;
        d.description = r.read_string()?;
        d.marked = r.read_bool()?;
        d.target = r.read_point3d()?;
        d.target_map = r.read_map()?;
    } else {
        // Deserialize(reader, version): version 1 adds the house, then the version 0 fields.
        if version == 1 {
            d.house = r.read_item(w)?;
        }
        d.description = r.read_string()?;
        d.marked = r.read_bool()?;
        d.target = r.read_point3d()?;
        d.target_map = r.read_map()?;
    }
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(d);
    }
    if version < 2 {
        calculate_hue(w, id);
    }
    Ok(())
}

/// `Server.Items.RecallRune`.
pub static RECALL_RUNE: ItemType = ItemType {
    full_name: "Server.Items.RecallRune",
    parent: Some(&ITEM_BASE),
    construct: Some(new_rune),
    default_weight: crate::class_props::default_weight,
    get_properties: rune_get_properties,
    on_single_click: rune_on_single_click,
    on_double_click: rune_on_double_click,
    serialize: rune_serialize,
    deserialize: rune_deserialize,
    ..ITEM_BASE
};

// ---- Runebook ----

/// `RunebookEntry`.
#[derive(Clone, Debug, Default)]
pub struct Entry {
    pub house: Option<ItemId>,
    pub location: Point3D,
    pub map: Option<MapId>,
    pub description: Option<String>,
}

/// `Runebook`' fields.
#[derive(Clone, Debug)]
pub struct BookData {
    pub quality: i32,
    pub crafter: Option<String>,
    pub level: i32,
    pub entries: Vec<Entry>,
    pub description: Option<String>,
    pub cur_charges: i32,
    pub max_charges: i32,
    pub default_index: i32,
    pub next_use: DateTime,
    pub openers: Vec<MobileId>,
}

pub fn book(w: &World, id: ItemId) -> Option<&BookData> {
    w.item(id)?.ext.get::<BookData>()
}

fn book_mut(w: &mut World, id: ItemId) -> Option<&mut BookData> {
    w.item_mut(id)?.ext.get_mut::<BookData>()
}

/// `item is Runebook`.
pub fn is_runebook(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.ty.is(&RUNEBOOK))
}

/// `new Runebook(maxCharges)`.
pub fn new_runebook(w: &mut World, max_charges: i32) -> ItemId {
    let aos = muo_content::core::aos(w);
    let id = ops::new_item_with_id(w, &RUNEBOOK, if aos { 0x22C5 } else { 0xEFA });
    ops::set_loot_type(w, id, LootType::Blessed);
    hue(w, id, 0x461);
    (RUNEBOOK.set_layer)(
        w,
        id,
        if aos {
            Layer::Invalid
        } else {
            Layer::OneHanded
        },
    );
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(BookData {
            quality: 0,
            crafter: None,
            level: 1, // SecureLevel.CoOwners
            entries: Vec::new(),
            description: None,
            cur_charges: 0,
            max_charges,
            default_index: -1,
            next_use: DateTime::MIN,
            openers: Vec::new(),
        });
    }
    id
}

/// `new Runebook()`.
fn new_runebook_default(w: &mut World) -> ItemId {
    let n = if muo_content::core::se(w) { 12 } else { 6 };
    new_runebook(w, n)
}

/// `CurCharges`.
pub fn cur_charges(w: &World, id: ItemId) -> i32 {
    book(w, id).map_or(0, |b| b.cur_charges)
}

/// `CurCharges = value` (`[InvalidateProperties]`).
pub fn set_cur_charges(w: &mut World, id: ItemId, v: i32) {
    if let Some(b) = book_mut(w, id)
        && b.cur_charges != v
    {
        b.cur_charges = v;
        ops::invalidate_properties(w, id);
    }
}

pub fn set_max_charges(w: &mut World, id: ItemId, v: i32) {
    if let Some(b) = book_mut(w, id)
        && b.max_charges != v
    {
        b.max_charges = v;
        ops::invalidate_properties(w, id);
    }
}

/// `entry.ShouldSerializeHouse` view: the house of an entry when it is alive.
fn live_house(w: &World, e: &Entry) -> Option<ItemId> {
    e.house.filter(|&h| w.item(h).is_some_and(|i| !i.deleted()))
}

fn on_craft(w: &mut World, id: ItemId, a: &CraftArgs) -> i32 {
    let inscribe = muo_server::skills::value(w, a.from, muo_server::skills::SkillName::Inscribe);
    let charges = (5 + a.quality + (inscribe / 30.0) as i32).min(10);
    set_max_charges(
        w,
        id,
        if muo_content::core::se(w) {
            charges * 2
        } else {
            charges
        },
    );
    if a.makers_mark {
        let name = muo_server::mobile::part1::props_b::raw_name(w, a.from);
        if let Some(b) = book_mut(w, id) {
            b.crafter = name;
        }
        ops::invalidate_properties(w, id);
    }
    if let Some(b) = book_mut(w, id) {
        b.quality = a.quality - 1;
    }
    ops::invalidate_properties(w, id);
    a.quality
}

fn craftable(w: &mut World, id: ItemId, a: &CraftArgs) -> Option<i32> {
    is_runebook(w, id).then(|| on_craft(w, id, a))
}

fn get_properties(w: &World, id: ItemId, list: &mut PropertyList) {
    base::get_properties_base(w, id, list);
    let Some(b) = book(w, id) else { return };
    if b.quality == 1 {
        list.add(1063341); // exceptional
    }
    if let Some(c) = &b.crafter {
        list.add_str(1050043, c); // crafted by ~1_NAME~
    }
    if let Some(d) = b.description.as_deref().filter(|d| !d.is_empty()) {
        list.add_text(d);
    }
}

fn on_single_click(w: &mut World, id: ItemId, from: MobileId) {
    let (desc, crafter) =
        book(w, id).map_or((None, None), |b| (b.description.clone(), b.crafter.clone()));
    if let Some(d) = desc.filter(|d| !d.is_empty()) {
        ops::label_to(w, id, from, &d);
    }
    base::on_single_click_base(w, id, from);
    if let Some(c) = crafter {
        ops::label_to_number(w, id, from, 1050043, &c);
    }
}

/// `house.HasSecureAccess(m, (SecureLevel)level)`.
fn has_secure_access(w: &World, house: ItemId, m: MobileId, level: i32) -> bool {
    let level = muo_multis::house::SecureLevel::from_u8(level as u8);
    muo_multis::house::access::has_secure_access(w, house, m, level)
}

/// `CheckAccess(m)`.
pub fn check_access(w: &World, id: ItemId, m: MobileId) -> bool {
    if !ops::is_locked_down(w, id)
        || w.mobile(m)
            .is_some_and(|x| x.access_level >= AccessLevel::GameMaster)
    {
        return true;
    }
    let (loc, map) = (
        ops::get_world_location(w, id),
        w.item(id).and_then(|i| i.map),
    );
    let height = ops::item_data(w, id).height as i32;
    let Some(o) = house_ops(w) else { return false };
    let Some(house) = o.find_house_at_point(w, loc, map, height) else {
        return false;
    };
    let level = book(w, id).map_or(1, |b| b.level);
    (!o.is_aos_rules(w, house)
        || (o.public(w, house) && !o.is_banned(w, house, m))
        || o.has_access(w, house, m))
        && has_secure_access(w, house, m, level)
}

/// `toCheck.FindGump<RunebookGump>()?.Book == this`.
fn is_open(w: &World, id: ItemId, m: MobileId) -> bool {
    muo_gumps::m_z::runebook::open_book(w, m) == Some(id)
}

fn on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    let range = if muo_content::core::ml(w) { 3 } else { 1 };
    let loc = ops::get_world_location(w, id);
    if deps::mobile_in_range(w, from, loc, range) && check_access(w, id, from) {
        if matches!(ops::root_parent(w, id), Some(EntityId::Mobile(m)) if muo_content::creature::data::is_creature(w, m))
        {
            deps::send_localized_message(w, from, 502402); // That is inaccessible.
            return;
        }
        if w.now < book(w, id).map_or(DateTime::MIN, |b| b.next_use) {
            deps::send_localized_message(w, from, 502406); // This book needs time to recharge.
            return;
        }
        send_gump_to(w, id, from);
    }
}

/// `SendGumpTo(from)`.
pub fn send_gump_to(w: &mut World, id: ItemId, from: MobileId) {
    if let Some(v) = view(w, id) {
        muo_gumps::m_z::runebook::display_to(w, from, v);
    }
    if let Some(b) = book_mut(w, id)
        && !b.openers.contains(&from)
    {
        b.openers.push(from);
    }
}

fn view(w: &World, id: ItemId) -> Option<RunebookView> {
    let b = book(w, id)?;
    let entries = b
        .entries
        .iter()
        .enumerate()
        .map(|(i, e)| RunebookEntryView {
            description: e.description.clone().unwrap_or_default(),
            map: e.map,
            map_index: e.map.map_or(-1, |m| m.0 as i32),
            location: e.location,
            sextant: sextant_format(w, e.location, e.map),
            is_default: b.default_index == i as i32,
        })
        .collect();
    Some(RunebookView {
        book: id,
        cur_charges: b.cur_charges,
        max_charges: b.max_charges,
        entries,
    })
}

fn on_travel(w: &mut World, id: ItemId) {
    if !muo_content::core::sa(w) {
        let n = w.now + TimeSpan::from_secs(7);
        if let Some(b) = book_mut(w, id) {
            b.next_use = n;
        }
    }
}

fn drop_rune(w: &mut World, id: ItemId, from: MobileId, index: usize) {
    let Some(b) = book_mut(w, id) else { return };
    if index >= b.entries.len() {
        return;
    }
    if b.default_index > index as i32 {
        b.default_index -= 1;
    } else if b.default_index == index as i32 {
        b.default_index = -1;
    }
    let e = b.entries.remove(index);
    ops::invalidate_properties(w, id);
    let r = new_rune(w);
    if let Some(d) = rune_mut(w, r) {
        d.target = e.location;
        d.target_map = e.map;
        d.description = e.description;
        d.house = e.house;
        d.marked = true;
    }
    calculate_hue(w, r);
    muo_server::mobile::part2::items::add_to_backpack(w, from, r);
    deps::send_localized_message(w, from, 502421); // You have removed the rune.
}

fn on_drag_lift(w: &mut World, id: ItemId, from: MobileId) -> bool {
    if muo_gumps::m_z::runebook::has_open_gump(w, from) {
        deps::send_localized_message(w, from, 500169); // You cannot pick that up.
        return false;
    }
    let openers = book(w, id).map(|b| b.openers.clone()).unwrap_or_default();
    for m in openers {
        if is_open(w, id, m) {
            muo_gumps::m_z::runebook::close_open_gump(w, m);
        }
    }
    if let Some(b) = book_mut(w, id) {
        b.openers.clear();
    }
    true
}

fn on_after_duped(w: &mut World, id: ItemId, new_item: ItemId) {
    let Some(entries) = book(w, id).map(|b| b.entries.clone()) else {
        return;
    };
    if let Some(b) = book_mut(w, new_item) {
        b.entries = entries;
    }
}

fn on_drag_drop(w: &mut World, id: ItemId, from: MobileId, dropped: ItemId) -> bool {
    let locked = ops::is_locked_down(w, id)
        && w.mobile(from)
            .is_some_and(|m| m.access_level < AccessLevel::GameMaster);
    if w.item(dropped).is_some_and(|i| i.ty.is(&RECALL_RUNE)) {
        if locked {
            messages::send_localized_message(w, from, 502413, "", 0x35); // That cannot be done while the book is locked down.
            return false;
        }
        if is_open(w, id, from) {
            deps::send_localized_message(w, from, 1005571); // You cannot place objects in the book while viewing the contents.
            return false;
        }
        if book(w, id).is_some_and(|b| b.entries.len() >= 16) {
            deps::send_localized_message(w, from, 502401); // This runebook is full.
            return false;
        }
        let rd = rune(w, dropped).cloned().unwrap_or_default();
        if rd.marked && rd.target_map.is_some() {
            if let Some(b) = book_mut(w, id) {
                b.entries.push(Entry {
                    house: rd.house,
                    location: rd.target,
                    map: rd.target_map,
                    description: rd.description.clone(),
                });
            }
            base::delete(w, dropped);
            let loc = ops::get_world_location(w, id);
            deps::send_sound(w, from, 0x42, loc);
            let text = rd
                .description
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .unwrap_or("(indescript)")
                .to_string();
            messages::send_message(w, from, &text);
            return true;
        }
        deps::send_localized_message(w, from, 502409); // This rune does not have a marked location.
        return false;
    }
    if w.item(dropped)
        .is_some_and(|i| i.ty.is(&super::scroll_list::RECALL_SCROLL))
    {
        let (cur, max) = book(w, id).map_or((0, 0), |b| (b.cur_charges, b.max_charges));
        if cur >= max {
            deps::send_localized_message(w, from, 502410); // This book already has the maximum amount of charges.
            return false;
        }
        let loc = ops::get_world_location(w, id);
        deps::send_sound(w, from, 0x249, loc);
        let amount = w.item(dropped).map_or(0, |i| i.amount);
        if amount > max - cur {
            base::consume(w, dropped, max - cur);
            set_cur_charges(w, id, max);
        } else {
            set_cur_charges(w, id, cur + amount);
            base::delete(w, dropped);
            return true;
        }
    }
    false
}

fn runebook_serialize(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    (ITEM_BASE.serialize)(w, id, out);
    out.write_i32(4);
    let Some(b) = book(w, id) else { return };
    out.write_enum(4, i64::from(b.quality));
    out.write_opt_string(b.crafter.as_deref());
    out.write_enum(4, i64::from(b.level));
    out.write_encoded_int(b.entries.len() as i32);
    for e in &b.entries {
        write_entry(w, e, out);
    }
    out.write_opt_string(b.description.as_deref());
    out.write_i32(b.cur_charges);
    out.write_i32(b.max_charges);
    out.write_i32(b.default_index);
}

fn write_entry(w: &World, e: &Entry, out: &mut dyn GenericWriter) {
    out.write_encoded_int(2);
    let house = live_house(w, e);
    let others = house.is_none();
    let flags = i32::from(house.is_some())
        | (i32::from(others) << 1)
        | (i32::from(others) << 2)
        | (i32::from(others) << 3);
    out.write_enum(4, i64::from(flags));
    if house.is_some() {
        out.write_item(w, house);
    }
    if others {
        out.write_point3d(e.location);
        out.write_map(e.map);
        out.write_opt_string(e.description.as_deref());
    }
}

fn read_entry(w: &World, r: &mut dyn GenericReader) -> Result<Entry> {
    let version = r.read_encoded_int()?;
    let mut e = Entry::default();
    if version >= 2 {
        let flags = r.read_enum(4)? as i32;
        if flags & 1 != 0 {
            e.house = r.read_item(w)?;
        }
        if flags & 2 != 0 {
            e.location = r.read_point3d()?;
        }
        if flags & 4 != 0 {
            e.map = r.read_map()?;
        }
        if flags & 8 != 0 {
            e.description = r.read_string()?;
        }
    } else {
        if version == 1 {
            e.house = r.read_item(w)?;
        }
        e.location = r.read_point3d()?;
        e.map = r.read_map()?;
        e.description = r.read_string()?;
    }
    Ok(e)
}

fn runebook_deserialize(w: &mut World, id: ItemId, r: &mut dyn GenericReader) -> Result<()> {
    (ITEM_BASE.deserialize)(w, id, r)?;
    let version = r.read_i32()?;
    let mut b = BookData {
        quality: 0,
        crafter: None,
        level: 1,
        entries: Vec::new(),
        description: None,
        cur_charges: 0,
        max_charges: 0,
        default_index: -1,
        next_use: DateTime::MIN,
        openers: Vec::new(),
    };
    if version >= 4 {
        b.quality = r.read_enum(4)? as i32;
        b.crafter = r.read_string()?;
        b.level = r.read_enum(4)? as i32;
        let n = r.read_encoded_int()?;
        for _ in 0..n.max(0) {
            b.entries.push(read_entry(w, r)?);
        }
    } else {
        // Deserialize(reader, version): the crafter was a Mobile whose name is copied on a zero-delay timer.
        b.quality = i32::from(r.read_u8()?);
        b.crafter = r
            .read_mobile(w)?
            .and_then(|m| muo_server::mobile::part1::props_b::raw_name(w, m));
        b.level = r.read_i32()?;
        let n = r.read_i32()?;
        for _ in 0..n.max(0) {
            b.entries.push(read_entry(w, r)?);
        }
    }
    b.description = r.read_string()?;
    b.cur_charges = r.read_i32()?;
    b.max_charges = r.read_i32()?;
    b.default_index = r.read_i32()?;
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(b);
    }
    Ok(())
}

/// `Server.Items.Runebook`.
pub static RUNEBOOK: ItemType = ItemType {
    full_name: "Server.Items.Runebook",
    parent: Some(&ITEM_BASE),
    construct: Some(new_runebook_default),
    default_weight: |w, _| if muo_content::core::se(w) { 1.0 } else { 3.0 },
    label_number: crate::class_props::label_number, // runebook
    display_loot_type: |w, _| muo_content::core::aos(w),
    allow_equipped_cast: |_, _, _| true,
    get_context_menu_entries: |w, id, from, list| {
        base::get_context_menu_entries_base(w, id, from, list);
        if let Some(e) = muo_multis::secure_entry::add_to(w, from, id) {
            list.push(e);
        }
    },
    get_properties,
    on_single_click,
    on_double_click,
    on_drag_lift,
    on_drag_drop,
    on_after_duped,
    serialize: runebook_serialize,
    deserialize: runebook_deserialize,
    ..ITEM_BASE
};

const INT: Kind = Kind::Plain(PropType::Numeric(Numeric::Int));
static RUNEBOOK_CTORS: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| EntityId::Item(new_runebook_default(w)),
    },
    Ctor {
        access: None,
        params: &[Param {
            name: "maxCharges",
            kind: INT,
            default: None,
            ty: "",
        }],
        make: |w, v| EntityId::Item(new_runebook(w, v[0].i32())),
    },
];
static RUNE_CTORS: [Ctor; 1] = [Ctor {
    access: None,
    params: &[],
    make: |w, _| EntityId::Item(new_rune(w)),
}];

// ---- RunebookOps ----

struct Ops;

fn entry_of(w: &World, book_id: ItemId, index: usize) -> Option<Entry> {
    book(w, book_id)?.entries.get(index).cloned()
}

impl RunebookOps for Ops {
    fn deleted(&self, w: &World, b: ItemId) -> bool {
        w.item(b).is_none_or(|i| i.deleted())
    }
    fn world_location(&self, w: &World, b: ItemId) -> Point3D {
        ops::get_world_location(w, b)
    }
    fn remove_opener(&self, w: &mut World, b: ItemId, from: MobileId) {
        if let Some(d) = book_mut(w, b) {
            d.openers.retain(|&m| m != from);
        }
    }
    fn design_context_check(&self, w: &mut World, from: MobileId) -> bool {
        muo_content::player_mobile::hooks::design_check(w, from)
    }
    fn is_locked_down(&self, w: &World, b: ItemId) -> bool {
        ops::is_locked_down(w, b)
    }
    fn check_access(&self, w: &mut World, b: ItemId, from: MobileId) -> bool {
        check_access(w, b, from)
    }
    fn cur_charges(&self, w: &World, b: ItemId) -> i32 {
        cur_charges(w, b)
    }
    fn entry_count(&self, w: &World, b: ItemId) -> usize {
        book(w, b).map_or(0, |d| d.entries.len())
    }
    fn on_travel(&self, w: &mut World, b: ItemId) {
        on_travel(w, b);
    }
    fn drop_rune(&self, w: &mut World, b: ItemId, from: MobileId, index: usize) {
        drop_rune(w, b, from, index);
    }
    fn set_default(&self, w: &mut World, b: ItemId, index: usize) {
        if let Some(d) = book_mut(w, b) {
            d.default_index = if index < d.entries.len() {
                index as i32
            } else {
                -1
            };
        }
    }
    fn send_gump_to(&self, w: &mut World, b: ItemId, from: MobileId) {
        send_gump_to(w, b, from);
    }
    fn set_description(&self, w: &mut World, b: ItemId, text: &str) {
        if let Some(d) = book_mut(w, b) {
            d.description = Some(text.to_string());
        }
        ops::invalidate_properties(w, b);
    }
    fn entry_location(
        &self,
        w: &World,
        b: ItemId,
        index: usize,
    ) -> Option<(Point3D, Option<MapId>, Option<SextantView>)> {
        let e = entry_of(w, b, index)?;
        Some((e.location, e.map, sextant_format(w, e.location, e.map)))
    }
    fn has_spell(&self, w: &World, from: MobileId, spell_id: i32) -> bool {
        // `Spellbook.Find` keeps a per-mobile cache it refreshes; a read-only search finds the same book.
        super::spellbook::find_all_spellbooks(w, from)
            .into_iter()
            .any(|bk| {
                super::spellbook::validate(
                    w,
                    bk,
                    spell_id,
                    super::spellbook::type_for_spell(spell_id),
                )
            })
    }
    fn cast(&self, w: &mut World, b: ItemId, from: MobileId, index: usize, spell: RuneSpell) {
        let Some(e) = entry_of(w, b, index) else {
            return;
        };
        let entry = Some((e.location, e.map));
        let sp = match spell {
            RuneSpell::RecallCharge => {
                muo_spells::fourth::recall::new_recall_spell(w, from, entry, Some(b), Some(b))
            }
            RuneSpell::Recall => {
                muo_spells::fourth::recall::new_recall_spell(w, from, entry, None, None)
            }
            RuneSpell::Gate => {
                muo_spells::seventh::gate_travel::new_gate_travel_spell(w, from, entry, None)
            }
            RuneSpell::SacredJourney => {
                muo_spells::chivalry::sacred_journey::new_sacred_journey_spell(
                    w, from, entry, None, None,
                )
            }
        };
        muo_content::spells::spell::cast(w, sp);
    }
}

fn configure(w: &mut World) {
    muo_gumps::m_z::hooks::hooks_mut(w).ops.runebooks = Some(Rc::new(Ops));
    {
        let h = w
            .resources
            .get_mut::<muo_spells::fourth::recall::RecallHooks>();
        h.charges = Some(cur_charges);
        h.add_charges = Some(|w, b, d| {
            let c = cur_charges(w, b);
            set_cur_charges(w, b, c + d);
        });
    }
    muo_content::craft::hooks_mut(w).craftables.push(craftable);
    let f = &mut w.resources.get_mut::<muo_content::skills::SkillHooks>().fns;
    f.is_recall_rune = Some(|w, item| w.item(item).is_some_and(|i| i.ty.is(&RECALL_RUNE)));
    let hd = house_deps::deps_mut(w);
    hd.sextant_format = Some(|w, p, map| sextant_format(w, p, map).map(|s| house_sextant_text(&s)));
    hd.securables
        .push(|w, id| is_runebook(w, id).then(|| book(w, id).map_or(1, |b| b.level) as u8));
    hd.securable_setters.push(|w, id, level| {
        if !is_runebook(w, id) {
            return false;
        }
        if let Some(b) = book_mut(w, id) {
            b.level = i32::from(level);
        }
        true
    });
    add_ctors(w, "Server.Items.Runebook", &RUNEBOOK_CTORS);
    add_ctors(w, "Server.Items.RecallRune", &RUNE_CTORS);
}

/// Registers the runebook, recall rune and sextant.
pub fn register(types: &mut TypeRegistry) {
    types.register_item(&RUNEBOOK, &[]);
    types.register_item(&RECALL_RUNE, &[]);
    types.register_item(&SEXTANT, &[]);
    types.register_phase_default(
        Phase::Configure,
        "Server.Items.Runebook.Configure",
        configure,
    );
}
