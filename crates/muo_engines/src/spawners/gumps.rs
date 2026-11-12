//! `SpawnerGump`, `SpawnerControllerGump` (`[SpawnAdmin`) and `SpawnPropsGump`.
//!
//! `SpawnPropsGump` is a `PropertiesGump` with a `PropsVariant` (ShowAttribute filter, Apply button).
//! (was: shortcut, lists
//! the eleven attributes of `SpawnPropsGump.MobileAttributes` of the template with a text field each and the same
//! Apply button, and builds the same `name value` property string from the template.

use muo_primitives::dotnet::{IgnoreCase, InvariantCase};
use std::rc::Rc;

use muo_content::gumps::base::{
    DynamicGumpLogic, DynamicGumpObj, LegacyGumpLogic, LegacyGumpObj, RelayInfo, hues,
    text_colors as tc,
};
use muo_content::gumps::builder::HtmlStyle;
use muo_content::gumps::grid::{self, GridSizeSpec, ListViewLayout};
use muo_content::gumps::{DynamicGumpBuilder, send_gump_to_mobile};
use muo_content::mx;
use muo_content::props;
use muo_primitives::html::TextAlignment;
use muo_primitives::parse::to_i32;
use muo_primitives::{Point3D, TimeSpan};
use muo_server::commands::{CommandEventArgs, CommandSystem};
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::ops;
use muo_server::mobile::AccessLevel;
use muo_server::net_state::NetStateId;
use muo_server::world::World;

use super::{EntryFlags, EntryUid, SpawnerData, core, data, data_mut};
use muo_gumps::m_z::props::{
    PropObj,
    properties::{self, PropsVariant},
};
use muo_server::target::Targeted;

fn mobile_of(w: &World, ns: NetStateId) -> Option<MobileId> {
    w.net_states.get(&ns).and_then(|n| n.mobile)
}

fn send(w: &mut World, to: MobileId, g: impl muo_content::gumps::Gump + 'static) {
    send_gump_to_mobile(w, to, Box::new(g), false);
}

fn open_props(w: &mut World, from: MobileId, e: EntityId) {
    if let Some(f) = w
        .resources
        .get::<muo_commands::hooks::CommandHooks>()
        .and_then(|h| h.properties_gump)
    {
        f(w, from, e);
    }
}

// ---- SpawnerGump ---------------------------------------------------------------------------------------------------

pub struct SpawnerGump {
    spawner: ItemId,
    /// `_entry`: the entry object whose Params/Props rows are open (held by identity, not by index).
    entry: Option<EntryUid>,
    page: usize,
}

fn count_color(count: i32, max: i32) -> &'static str {
    match f64::from(count) / f64::from(max) {
        r if r <= 0.25 => "#DE3163", // red
        r if r <= 0.50 => "#FF7F50", // orange
        r if r <= 0.75 => "#DFFF00", // yellow
        r if r <= 1.0 => "#00FF00",  // green
        _ => "#F4F4F4",              // white
    }
}

/// `new SpawnerGump(spawner, focusEntry, page)` sent to `to`.
pub fn send_spawner_gump(
    w: &mut World,
    to: MobileId,
    spawner: ItemId,
    entry: Option<usize>,
    page: usize,
) {
    let g = spawner_gump(w, spawner, entry, page);
    send(w, to, g);
}

pub fn spawner_gump(
    w: &mut World,
    spawner: ItemId,
    focus: Option<usize>,
    page: usize,
) -> LegacyGumpObj<SpawnerGump> {
    let entries = data(w, spawner)
        .map(|d| d.entries.clone())
        .unwrap_or_default();
    let focus_uid = focus.and_then(|i| entries.get(i)).map(|e| e.uid);
    let mut g = LegacyGumpObj::new(
        50,
        50,
        SpawnerGump {
            spawner,
            entry: focus_uid,
            page,
        },
    );
    let counts: Vec<i32> = (0..entries.len())
        .map(|i| core::count_spawns(w, spawner, i) as i32)
        .collect();
    let (running, spawner_count) = data(w, spawner).map_or((false, 0), |d| (d.running, d.count));
    let name = ops::name(w, spawner).unwrap_or_default();
    let height = 400 + if focus.is_some() { 44 } else { 0 };
    let l = &mut g.layout;
    l.add_page(0);
    l.add_background(0, 0, 369, height, 5054);
    l.add_alpha_region(0, 0, 369, height);
    l.add_html(
        263,
        1,
        250,
        20,
        "<BASEFONT COLOR=#F4F4F4>#</BASEFONT>",
        false,
        false,
    );
    l.add_html(
        294,
        1,
        250,
        20,
        "<BASEFONT COLOR=#F4F4F4>Max</BASEFONT>",
        false,
        false,
    );
    l.add_html(
        334,
        1,
        250,
        20,
        "<BASEFONT COLOR=#F4F4F4>Prb</BASEFONT>",
        false,
        false,
    );
    let button_id = |ty: i32, index: i32| 1 + index * 10 + ty;
    let mut offset = 0;
    for i in 0..13i32 {
        let text_index = i * 5;
        let entry_index = page * 13 + i as usize;
        let entry = entries.get(entry_index);
        let focused = entry.is_some() && focus == Some(entry_index);
        let y = 22 * i + 21 + offset;
        if !focused {
            l.add_button(
                28,
                y,
                if entry.is_some() { 0xFBA } else { 0xFA5 },
                if entry.is_some() { 0xFBC } else { 0xFA7 },
                button_id(2, i * 2),
            ); // Expand
        } else {
            l.add_button(28, y, 0xFBB, 0xFBC, button_id(2, i * 2)); // Unexpand
        }
        l.add_button(61, y, 0xFA2, 0xFA4, button_id(2, 1 + i * 2)); // Delete
        if let Some(e) = entry {
            if e.disabled {
                l.add_button(9, 22 * i + 24 + offset, 0x82C, 0x82C, button_id(3, i)); // Locked, press to unlock
            } else {
                l.add_button(7, 22 * i + 24 + offset, 0x2C88, 0x2C89, button_id(3, i)); // Unlocked, press to lock
            }
        }
        // Locked entries render on a grey tile as labels instead of text entries, so they can't be edited
        let locked = entry.is_some_and(|e| e.disabled);
        let field_tile = if locked { 0x23F4 } else { 0xBBC };
        l.add_image_tiled(94, 22 * i + 20 + offset, 161, 23, 0xA40);
        l.add_image_tiled(95, y, 159, 21, field_tile);
        l.add_image_tiled(258, 22 * i + 20 + offset, 35, 23, 0xA40);
        l.add_image_tiled(259, y, 33, 21, 0xE14);
        l.add_image_tiled(290, 22 * i + 20 + offset, 35, 23, 0xA40);
        l.add_image_tiled(291, y, 33, 21, field_tile);
        l.add_image_tiled(328, 22 * i + 20 + offset, 35, 23, 0xA40);
        l.add_image_tiled(329, y, 33, 21, field_tile);
        let (ename, probability, max_count, flags);
        if let Some(e) = entry {
            ename = e.name.clone().unwrap_or_default();
            probability = e.probability.to_string();
            max_count = e.max_count.to_string();
            flags = e.valid;
            let count = counts[entry_index];
            l.add_html(
                258,
                22 * i + 20 + offset + 1,
                35,
                15,
                &format!("<BASEFONT COLOR={}><div align=RIGHT>{count}<BASEFONT COLOR=#F4F4F4>/</BASEFONT></div></BASEFONT>", count_color(count, e.max_count)),
                false,
                false,
            );
        } else {
            (ename, probability, max_count, flags) = (
                String::new(),
                String::new(),
                String::new(),
                EntryFlags::empty(),
            );
        }
        let name_hue = if flags.contains(EntryFlags::INVALID_TYPE) {
            33
        } else {
            0
        };
        if locked {
            l.add_label_cropped(98, y, 156, 21, name_hue, &ename);
            l.add_label_cropped(293, y, 30, 21, 0, &max_count);
            l.add_label_cropped(331, y, 30, 21, 0, &probability);
        } else {
            l.add_text_entry(98, y, 156, 21, name_hue, text_index, &ename);
            l.add_text_entry(293, y, 30, 21, 0, text_index + 1, &max_count);
            l.add_text_entry(331, y, 30, 21, 0, text_index + 2, &probability);
        }
        if let (Some(e), true) = (entry, focused) {
            l.add_label(5, 22 * i + 42, 0x384, "Params");
            l.add_image_tiled(55, 22 * i + 42, 276, 23, 0xA40);
            l.add_image_tiled(56, 22 * i + 43, 274, 21, field_tile);
            l.add_label(5, 22 * i + 64, 0x384, "Props");
            l.add_image_tiled(55, 22 * i + 64, 276, 23, 0xA40);
            l.add_image_tiled(56, 22 * i + 65, 274, 21, field_tile);
            let params_hue = if flags.contains(EntryFlags::INVALID_PARAMS) {
                33
            } else {
                0
            };
            let props_hue = if flags.contains(EntryFlags::INVALID_PROPS) {
                33
            } else {
                0
            };
            let (p, q) = (
                e.parameters.clone().unwrap_or_default(),
                e.properties.clone().unwrap_or_default(),
            );
            if locked {
                l.add_label_cropped(59, 22 * i + 42, 271, 21, params_hue, &p);
                l.add_label_cropped(59, 22 * i + 62, 271, 21, props_hue, &q);
            } else {
                l.add_text_entry(59, 22 * i + 42, 271, 21, params_hue, text_index + 3, &p);
                l.add_text_entry(59, 22 * i + 62, 271, 21, props_hue, text_index + 4, &q);
            }
            offset += 44;
        }
    }
    if running {
        l.add_button(5, 312 + offset, 0x2A4E, 0x2A3A, button_id(1, 6));
        l.add_label(38, 317 + offset, 0x384, "On");
    } else {
        l.add_button(5, 312 + offset, 0x2A62, 0x2A3A, button_id(1, 7));
        l.add_label(38, 317 + offset, 0x384, "Off");
    }
    let (mut total_spawned, mut total_max, mut total_weight) = (0, 0, 0);
    for (i, e) in entries.iter().enumerate() {
        total_spawned += counts[i];
        if !e.disabled {
            total_max += e.max_count;
            total_weight += e.probability;
        }
    }
    let center = |t: i32| muo_primitives::html::center_rgb(&t.to_string(), 0xF4F4F4);
    l.add_html(
        258,
        308 + offset,
        35,
        20,
        &center(total_spawned),
        false,
        false,
    );
    l.add_html(290, 308 + offset, 35, 20, &center(total_max), false, false);
    l.add_html(
        328,
        308 + offset,
        35,
        20,
        &center(total_weight),
        false,
        false,
    );
    l.add_html(
        5,
        1,
        161,
        20,
        &format!("<BASEFONT COLOR=#FFEA00>{name}</BASEFONT><BASEFONT COLOR={}> ({total_spawned}/{spawner_count})</BASEFONT>", count_color(total_spawned, spawner_count)),
        false,
        false,
    );
    l.add_button(5, 347 + offset, 0xFAB, 0xFAD, button_id(1, 2));
    l.add_label(38, 347 + offset, 0x384, "Props");
    l.add_button(5, 369 + offset, 0xFAE, 0xFAF, button_id(1, 8));
    l.add_label(38, 369 + offset, 0x384, "Goto");
    l.add_button(90, 325 + offset, 0xFA2, 0xFA3, button_id(1, 9));
    l.add_label(123, 325 + offset, 0x384, "Reset");
    l.add_button(90, 347 + offset, 0xFB4, 0xFB6, button_id(1, 3));
    l.add_label(123, 347 + offset, 0x384, "Bring Home");
    l.add_button(90, 369 + offset, 0xFA8, 0xFAA, button_id(1, 4));
    l.add_label(123, 369 + offset, 0x384, "Total Respawn");
    l.add_button(283, 347 + offset, 0xFB7, 0xFB9, button_id(1, 5));
    l.add_label(316, 347 + offset, 0x384, "Save");
    l.add_button(283, 369 + offset, 0xFB1, 0xFB3, 0);
    l.add_label(316, 369 + offset, 0x384, "Cancel");
    if page > 0 {
        l.add_button(223, 308 + offset, 0x15E3, 0x15E7, button_id(1, 0));
    } else {
        l.add_image(223, 308 + offset, 0x25EA, 0);
    }
    if (page + 1) * 13 <= entries.len() {
        l.add_button(240, 308 + offset, 0x15E1, 0x15E5, button_id(1, 1));
    } else {
        l.add_image(240, 308 + offset, 0x25E6, 0);
    }
    g
}

/// `SpawnerGump.CreateArray(info, from, spawner)`.
fn create_array(w: &mut World, g: &mut SpawnerGump, info: &RelayInfo, from: MobileId) {
    let spawner = g.spawner;
    let ocount = data(w, spawner).map_or(0, |d| d.entries.len());
    let mut queue: Vec<usize> = Vec::new();
    for i in 0..13usize {
        let index = (i * 5) as u16;
        let entry_index = g.page * 13 + i;
        let t = |k: u16| info.get_text_entry(i32::from(k)).map(str::to_owned);
        let (type_entry, max_spawns, probability, custom_params, custom_props) = (
            t(index),
            t(index + 1),
            t(index + 2),
            t(index + 3),
            t(index + 4),
        );
        let Some(type_entry) = type_entry else {
            continue;
        };
        let s = type_entry.trim().lower_inv();
        if !s.is_empty() {
            if w.types.find_type_by_name(&s, false, true).is_none() {
                mx::send_message(
                    w,
                    from,
                    &format!("{s} is not a valid type name for entry #{i}."),
                );
                return;
            }
            let idx;
            if entry_index < ocount {
                idx = entry_index;
                if let Some(e) = data_mut(w, spawner).and_then(|d| d.entries.get_mut(idx)) {
                    e.name = Some(s.clone());
                    if let Some(m) = &max_spawns {
                        e.max_count = to_i32(m.trim());
                    }
                    if let Some(p) = &probability {
                        e.probability = to_i32(p.trim());
                    }
                }
            } else {
                let maxcount = max_spawns.as_deref().map_or(1, |m| to_i32(m.trim()));
                let probcount = probability.as_deref().map_or(100, |p| to_i32(p.trim()));
                idx = core::add_entry(w, spawner, Some(&s), probcount, maxcount, true, None, None);
            }
            if let Some(e) = data_mut(w, spawner).and_then(|d| d.entries.get_mut(idx)) {
                if let Some(p) = custom_params {
                    e.parameters = Some(p.trim().to_owned());
                }
                if let Some(p) = custom_props {
                    e.properties = Some(p.trim().to_owned());
                }
            }
        } else if entry_index < ocount {
            queue.push(entry_index);
        }
    }
    // removing by reference in queue order and highest index first gives the same result by index
    queue.sort_unstable_by(|a, b| b.cmp(a));
    for r in &queue {
        core::remove_entry(w, spawner, *r);
    }
    if ocount == 0 && data(w, spawner).is_some_and(|d| !d.entries.is_empty()) {
        core::start(w, spawner);
    }
}

impl LegacyGumpLogic for SpawnerGump {
    const TYPE_NAME: &'static str = "Server.Engines.Spawners.SpawnerGump";

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        let spawner = self.spawner;
        if w.item(spawner).is_none() || ops::is_deleted(w, spawner) {
            return;
        }
        let val = info.button_id - 1;
        if val < 0 {
            return;
        }
        let (ty, index) = (val % 10, val / 10);
        let n_entries = data(w, spawner).map_or(0, |d| d.entries.len());
        match ty {
            0 => return, // Cancel
            1 => match index {
                0 => {
                    if self.page > 0 {
                        self.page -= 1;
                        self.entry = None;
                    }
                }
                1 => {
                    if (self.page + 1) * 13 <= n_entries {
                        self.page += 1;
                        self.entry = None;
                    }
                }
                2 => open_props(w, from, EntityId::Item(spawner)), // Props
                3 => core::bring_to_home(w, spawner),
                4 => core::respawn(w, spawner), // Complete respawn
                5 => create_array(w, self, info, from), // Save
                6 => core::set_running(w, spawner, false), // On button
                7 => core::set_running(w, spawner, true), // Off button
                8 => {
                    // Goto
                    if let Some((loc, map)) =
                        w.item(spawner).and_then(|i| i.map.map(|m| (i.location, m)))
                    {
                        muo_server::mobile::base::move_to_world(w, from, loc, map);
                    }
                }
                9 => core::reset(w, spawner),
                _ => {}
            },
            2 => {
                let entry_index = (index / 2) as usize + self.page * 13;
                let button_type = index % 2;
                if let Some(uid) = data(w, spawner)
                    .and_then(|d| d.entries.get(entry_index))
                    .map(|e| e.uid)
                {
                    if button_type == 0 {
                        // Spawn creature
                        self.entry = if self.entry != Some(uid) {
                            Some(uid)
                        } else {
                            None
                        };
                    } else {
                        core::remove_spawn(w, spawner, entry_index); // Remove creatures
                    }
                }
                create_array(w, self, info, from);
            }
            3 => {
                // Enable/disable entry
                let entry_index = index as usize + self.page * 13;
                if let Some(e) = data_mut(w, spawner).and_then(|d| d.entries.get_mut(entry_index)) {
                    e.disabled = !e.disabled;
                }
                create_array(w, self, info, from);
            }
            _ => {}
        }
        // HasEntry: the held entry is still in the list (wherever it moved to)
        let focus = self
            .entry
            .and_then(|u| data(w, spawner).and_then(|d| d.entries.iter().position(|e| e.uid == u)));
        send_spawner_gump(w, from, spawner, focus, self.page);
    }
}

// ---- SpawnerControllerGump ---------------------------------------------------------------------------------------------

const GUMP_WIDTH: i32 = 1000;
const GUMP_HEIGHT: i32 = 800;
const ROW_HEIGHT: i32 = 45;
const HEADER_HEIGHT: i32 = 28;
const ENTRY_COUNT: i32 = 5;
const TYPE_COUNT: i32 = 15;
const COLUMN_SPEC: &str = "6* 8* 18* 6* 12* 6* 5* 5* 8* 8* 9* *";
const MAIN_ROW_SPEC: &str = "10* * 100";

/// `SpawnSearchType`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SpawnSearchType {
    Creature = 0,
    Coords,
    Props,
    Name,
}

/// `SpawnSearch`.
#[derive(Clone, Debug)]
pub struct SpawnSearch {
    pub ty: SpawnSearchType,
    pub pattern: String,
}

impl Default for SpawnSearch {
    fn default() -> Self {
        Self {
            ty: SpawnSearchType::Creature,
            pattern: String::new(),
        }
    }
}

struct Row {
    serial: u32,
    name: String,
    map: String,
    x: i32,
    y: i32,
    entries: usize,
    walking_range: i32,
    home_range_style: bool,
    home_range: i32,
    min_delay: TimeSpan,
    max_delay: TimeSpan,
    next_spawn: TimeSpan,
}

pub struct SpawnerControllerGump {
    page: i32,
    search: SpawnSearch,
    copy: Option<ItemId>,
    spawners: Vec<ItemId>,
    rows: Vec<Row>,
}

fn button(ty: i32, index: i32) -> i32 {
    1 + ty + index * TYPE_COUNT
}

/// `$@"{ts:hh\:mm\:ss}"`: the hours component only.
fn hms(t: TimeSpan) -> String {
    let secs = (t.ticks() / 10_000_000).abs();
    format!(
        "{:02}:{:02}:{:02}",
        (secs / 3600) % 24,
        (secs / 60) % 60,
        secs % 60
    )
}

fn search_spawners(w: &World, mobile: MobileId, search: &SpawnSearch) -> Vec<ItemId> {
    if search.pattern.is_empty() {
        return Vec::new();
    }
    let range = if search.ty == SpawnSearchType::Coords {
        search.pattern.trim().parse::<i32>().ok()
    } else {
        None
    };
    let pat = search.pattern.lower_inv();
    let mloc = w.mobile(mobile).map(|m| m.location).unwrap_or_default();
    let mut out = Vec::new();
    for (id, item) in &w.items {
        let Some(d) = item.ext.get::<SpawnerData>() else {
            continue;
        };
        let hit = match search.ty {
            SpawnSearchType::Creature => d.entries.iter().any(|e| {
                e.name
                    .as_deref()
                    .is_some_and(|n| n.lower_inv().contains(&pat))
            }),
            SpawnSearchType::Coords => {
                range.is_some_and(|r| {
                    muo_primitives::utility::in_range(
                        mloc.x,
                        mloc.y,
                        item.location.x,
                        item.location.y,
                        r,
                    )
                }) || range.is_none() && in_range_minus_one(mloc, item.location)
            }
            SpawnSearchType::Props => d.entries.iter().any(|e| {
                e.properties
                    .as_deref()
                    .is_some_and(|p| p.lower_inv().contains(&pat))
            }),
            SpawnSearchType::Name => {
                ops::name(w, *id).is_some_and(|n| n.lower_inv().contains(&pat))
            }
        };
        if hit {
            out.push(*id);
        }
    }
    out
}

/// `_mobile.InRange(location, -1)`: false for any distance.
fn in_range_minus_one(_a: Point3D, _b: Point3D) -> bool {
    false
}

/// `new SpawnerControllerGump(mobile, page, copy, search)` sent to the mobile.
pub fn send_controller(
    w: &mut World,
    mobile: MobileId,
    page: i32,
    copy: Option<ItemId>,
    search: SpawnSearch,
    re_search: bool,
    prior: Option<Vec<ItemId>>,
) {
    // `SearchSpawner()` returns before touching `_spawners` when the pattern is empty: the last list stays
    let spawners = if (re_search && !search.pattern.is_empty()) || prior.is_none() {
        search_spawners(w, mobile, &search)
    } else {
        prior.unwrap_or_default()
    };
    // (a spawner deleted since would stay listed; here it has no row, so it is dropped)
    let spawners: Vec<ItemId> = spawners
        .into_iter()
        .filter(|id| data(w, *id).is_some() && w.item(*id).is_some())
        .collect();
    let rows: Vec<Row> = spawners
        .iter()
        .filter_map(|id| {
            let d = data(w, *id)?;
            let i = w.item(*id)?;
            Some(Row {
                serial: i.serial.value(),
                name: ops::name(w, *id).unwrap_or_default(),
                map: i
                    .map
                    .and_then(|m| w.maps.get(m))
                    .map(|m| m.name().to_owned())
                    .unwrap_or_default(),
                x: i.location.x,
                y: i.location.y,
                entries: d.entries.len(),
                walking_range: core::walking_range(w, *id),
                home_range_style: core::is_home_range_style_at(w, *id, i.location),
                home_range: core::home_range(w, *id),
                min_delay: d.min_delay,
                max_delay: d.max_delay,
                next_spawn: if d.running && d.timer.is_some_and(|t| t.running(w)) {
                    d.end - w.now
                } else {
                    TimeSpan::ZERO
                },
            })
        })
        .collect();
    let g = SpawnerControllerGump {
        page,
        search,
        copy,
        spawners,
        rows,
    };
    send(w, mobile, DynamicGumpObj::new(20, 30, g));
}

impl SpawnerControllerGump {
    fn layout(&self) -> (ListViewLayout, [i32; 12], [i32; 12], [i32; 3], [i32; 3]) {
        let mut row_pos = [0; 3];
        let mut row_heights = [0; 3];
        let mut specs = [GridSizeSpec::star(); 3];
        GridSizeSpec::parse_all(MAIN_ROW_SPEC, &mut specs);
        grid::compute_track_sizes(&specs, GUMP_HEIGHT, 0, &mut row_pos, &mut row_heights);
        let mut col_pos = [0; 12];
        let mut col_widths = [0; 12];
        let lv = ListViewLayout::create(
            0,
            row_pos[1],
            GUMP_WIDTH,
            row_heights[1] - 7, // marginY adjustment
            self.rows.len() as i32,
            self.page,
            ROW_HEIGHT,
            HEADER_HEIGHT,
            COLUMN_SPEC,
            &mut col_pos,
            &mut col_widths,
        );
        (lv, col_pos, col_widths, row_pos, row_heights)
    }
}

fn st(color: &str, size: i32, align: TextAlignment) -> HtmlStyle<'_> {
    HtmlStyle {
        color,
        size,
        font_style: 0,
        align,
    }
}

impl DynamicGumpLogic for SpawnerControllerGump {
    const TYPE_NAME: &'static str = "Server.Gumps.SpawnerControllerGump";

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.add_page(0);
        b.add_background(0, 0, GUMP_WIDTH, GUMP_HEIGHT, 5054);
        b.add_image_tiled(0, 0, GUMP_WIDTH, GUMP_HEIGHT, 2624);
        b.add_alpha_region(0, 0, GUMP_WIDTH, GUMP_HEIGHT);
        let (lv, col_pos, col_widths, row_pos, row_heights) = self.layout();
        let (row1_y, row2_y) = (row_pos[1], row_pos[2]);
        let band = |b: &mut DynamicGumpBuilder, x: i32, y: i32, w: i32, h: i32, tile: i32| {
            b.add_image_tiled(x, y, w, h, tile);
            b.add_alpha_region(x, y, w, h);
        };
        band(b, 0, row1_y, GUMP_WIDTH, 3, 9357);
        band(b, 0, row1_y + 21, GUMP_WIDTH, 3, 9357);
        for i in 0..lv.column_count.saturating_sub(1) {
            band(
                b,
                col_pos[i] + col_widths[i] - 8,
                row_pos[1] + 3,
                3,
                20,
                9355,
            );
        }
        band(b, 0, row2_y - 11, GUMP_WIDTH, 6, 9357);
        band(b, 0, row2_y + 55, GUMP_WIDTH, 6, 9357);

        self.draw_search(b, row_pos[0], row_heights[0]);

        const HEADERS: [&str; 12] = [
            "Copy",
            "Paste",
            "Name/Serial",
            "Map",
            "Coords",
            "Entry",
            "Walk",
            "Home",
            "Min Delay",
            "Max Delay",
            "Next Spawn",
            "Actions",
        ];
        for (i, header) in HEADERS.iter().enumerate().take(lv.column_count) {
            let cell = lv.get_header_cell(i, &col_pos, &col_widths);
            let off = if i >= 4 { -4 } else { 0 };
            b.add_html_styled(
                cell.x + off,
                cell.y + 3,
                cell.width,
                30,
                header,
                st(tc::GOLD, 4, TextAlignment::Center),
                false,
                false,
            );
        }

        const V_CENTER: i32 = 10;
        for i in 0..lv.visible_count {
            let data_index = lv.get_data_index(i);
            let r = &self.rows[data_index as usize];
            let row_y = lv.get_row_y(i);
            let (c0x, c0w) = (col_pos[0], col_widths[0]);
            b.add_button(c0x + 15, row_y, 4011, 4012, button(2, data_index));
            b.add_html_styled(
                c0x,
                row_y + 21,
                c0w,
                30,
                "copy",
                st(tc::WHITE, 3, TextAlignment::Center),
                false,
                false,
            );
            let (c1x, c1w) = (col_pos[1], col_widths[1]);
            let c1h = c1x + c1w / 2;
            b.add_button(c1x + 2, row_y, 4029, 4031, button(3, data_index));
            b.add_html_styled(
                c1x + 3,
                row_y + 21,
                c1w,
                30,
                "props",
                st(tc::WHITE, 3, TextAlignment::Left),
                false,
                false,
            );
            b.add_button(c1h - 2, row_y, 4029, 4031, button(4, data_index));
            b.add_html_styled(
                c1h - 3,
                row_y + 21,
                55,
                30,
                "entry",
                st(tc::WHITE, 3, TextAlignment::Left),
                false,
                false,
            );
            let (c2x, c2w) = (col_pos[2], col_widths[2]);
            let entry_index = i * ENTRY_COUNT;
            b.add_image_tiled(c2x + 6, row_y + 13, c2w - 8, 1, 9357);
            b.add_text_entry(
                c2x + 8,
                row_y - 3,
                c2w - 8,
                80,
                hues::WHITE,
                entry_index,
                &r.name,
            );
            b.add_button(c2x + 8, row_y + 19, 5837, 5838, button(6, data_index));
            b.add_html_styled(
                c2x + 54,
                row_y + 19,
                c2w,
                30,
                &format!("0x{:08X}", r.serial),
                st(tc::WHITE, 4, TextAlignment::Left),
                false,
                false,
            );
            let (c3x, c3w) = (col_pos[3], col_widths[3]);
            b.add_html_styled(
                c3x,
                row_y + V_CENTER,
                c3w,
                30,
                &r.map,
                st(tc::WHITE, 4, TextAlignment::Center),
                false,
                false,
            );
            let (c4x, c4w) = (col_pos[4], col_widths[4]);
            b.add_button(c4x + 3, row_y + V_CENTER, 2062, 2062, button(5, data_index));
            b.add_image(c4x + 3, row_y + V_CENTER, 2062, 936);
            b.add_html_styled(
                c4x + 12,
                row_y + V_CENTER,
                c4w,
                30,
                &format!("X {} Y {}", r.x, r.y),
                st(tc::WHITE, 4, TextAlignment::Left),
                false,
                false,
            );
            let (c5x, c5w) = (col_pos[5], col_widths[5]);
            b.add_button(
                c5x + 6,
                row_y + V_CENTER - 4,
                0x2635,
                0x2635,
                button(7, data_index),
            );
            b.add_image(c5x + 6, row_y + V_CENTER - 7, 0x2635, 827);
            b.add_html_styled(
                c5x,
                row_y + V_CENTER,
                c5w,
                30,
                &r.entries.to_string(),
                st(tc::WHITE, 4, TextAlignment::Center),
                false,
                false,
            );
            let (c6x, c6w) = (col_pos[6], col_widths[6]);
            b.add_image_tiled(c6x + 8, row_y + 24, c6w / 2, 1, 9357);
            b.add_text_entry(
                c6x + 12,
                row_y + 8,
                c6w,
                80,
                hues::WHITE,
                entry_index + 1,
                &r.walking_range.to_string(),
            );
            let (c7x, c7w) = (col_pos[7], col_widths[7]);
            if r.home_range_style {
                b.add_image_tiled(c7x + 8, row_y + 24, c7w / 2, 1, 9357);
                b.add_text_entry(
                    c7x + 12,
                    row_y + 8,
                    c7w,
                    80,
                    hues::WHITE,
                    entry_index + 2,
                    &r.home_range.to_string(),
                );
            } else {
                b.add_html_styled(
                    c7x - 6,
                    row_y + 8,
                    c7w,
                    30,
                    "Custom",
                    st(tc::YELLOW, 4, TextAlignment::Center),
                    false,
                    false,
                );
            }
            let (c8x, c8w) = (col_pos[8], col_widths[8]);
            b.add_image_tiled(c8x + 6, row_y + 24, c8w - 20, 1, 9357);
            b.add_text_entry(
                c8x + 8,
                row_y + 8,
                c8w,
                80,
                hues::WHITE,
                entry_index + 3,
                &r.min_delay.to_string(),
            );
            let (c9x, c9w) = (col_pos[9], col_widths[9]);
            b.add_image_tiled(c9x + 6, row_y + 24, c9w - 20, 1, 9357);
            b.add_text_entry(
                c9x + 8,
                row_y + 8,
                c9w,
                80,
                hues::WHITE,
                entry_index + 4,
                &r.max_delay.to_string(),
            );
            let (c10x, c10w) = (col_pos[10], col_widths[10]);
            b.add_html_styled(
                c10x,
                row_y + V_CENTER,
                c10w,
                30,
                &hms(r.next_spawn),
                st(tc::WHITE, 4, TextAlignment::Center),
                false,
                false,
            );
            let c11x = col_pos[11];
            b.add_button(c11x, row_y, 4023, 4025, button(8, data_index));
            b.add_html_styled(
                c11x + 4,
                row_y + 21,
                55,
                30,
                "save",
                st(tc::WHITE, 3, TextAlignment::Left),
                false,
                false,
            );
            b.add_button(c11x + ROW_HEIGHT, row_y, 4017, 4018, button(9, data_index));
            b.add_html_styled(
                c11x + ROW_HEIGHT,
                row_y + 21,
                55,
                30,
                "delete",
                st(tc::WHITE, 3, TextAlignment::Left),
                false,
                false,
            );
            band(b, 0, row_y + ROW_HEIGHT, GUMP_WIDTH, 1, 9357);
        }

        let col0_hcenter = GUMP_WIDTH / 2;
        let row2_vcenter = row2_y + row_heights[2] / 2;
        if lv.can_go_next {
            b.add_button(
                col0_hcenter + 60,
                row2_vcenter + 16,
                4005,
                4006,
                button(10, 0),
            );
        }
        if lv.can_go_back {
            b.add_button(
                col0_hcenter - 60,
                row2_vcenter + 16,
                4014,
                4015,
                button(11, 0),
            );
        }
        let n = self.spawners.len();
        b.add_button(15, row2_y, 4029, 4031, button(1, 5));
        b.add_html_styled(
            55,
            row2_y + 4,
            GUMP_WIDTH,
            20,
            &format!("Paste props for {n} spawners"),
            st(tc::ORANGE, 4, TextAlignment::Left),
            false,
            false,
        );
        b.add_button(15, row2_y + 28, 4029, 4031, button(1, 6));
        b.add_html_styled(
            55,
            row2_y + 32,
            GUMP_WIDTH,
            20,
            &format!("Paste entry for {n} spawners"),
            st(tc::ORANGE, 4, TextAlignment::Left),
            false,
            false,
        );
        b.add_button(275, row2_y, 4029, 4031, button(1, 7));
        b.add_html_styled(
            315,
            row2_y + 4,
            GUMP_WIDTH,
            20,
            "Paste copy to the ground",
            st(tc::YELLOW, 4, TextAlignment::Left),
            false,
            false,
        );
        b.add_html_styled(
            16,
            row2_vcenter + 17,
            GUMP_WIDTH,
            20,
            &format!("Pages {}/{}", lv.current_page + 1, lv.total_pages),
            st(tc::WHITE, 4, TextAlignment::Left),
            false,
            false,
        );
        b.add_html_styled(
            120,
            row2_vcenter + 17,
            GUMP_WIDTH,
            20,
            &format!("Total Spawners {n}"),
            st(tc::WHITE, 4, TextAlignment::Left),
            false,
            false,
        );
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(mobile) = mobile_of(w, sender) else {
            return;
        };
        if info.button_id == 0 {
            return;
        }
        let button_id = info.button_id - 1;
        let (ty, index) = (button_id % TYPE_COUNT, button_id / TYPE_COUNT);
        let line_count = self.layout().0.items_per_page;
        let len = self.spawners.len() as i32;
        let mut re_search = true;
        match ty {
            1 => {
                if index < 4 {
                    self.search.ty = [
                        SpawnSearchType::Creature,
                        SpawnSearchType::Coords,
                        SpawnSearchType::Props,
                        SpawnSearchType::Name,
                    ][index as usize];
                    let numeric = self.search.pattern.trim().parse::<i32>().is_ok();
                    if index == 1 && !numeric || index != 1 && numeric {
                        self.search.pattern.clear();
                    }
                } else if index == 4 {
                    if let Some(e) = info.get_text_entry(0xFFFF).filter(|e| !e.is_empty()) {
                        self.search.pattern = e.to_owned();
                    }
                } else if index == 5
                    && len > 0
                    && let Some(copy) = self.copy
                {
                    for s in self.spawners.clone() {
                        copy_property(w, copy, s);
                    }
                } else if index == 6
                    && len > 0
                    && let Some(copy) = self.copy
                {
                    for s in self.spawners.clone() {
                        copy_entry(w, copy, s);
                    }
                } else if index == 7
                    && let Some(copy) = self.copy
                {
                    let id = core::create(
                        w,
                        &core::SPAWNER,
                        super::Kind::Spawner,
                        1,
                        super::DEFAULT_MIN_DELAY,
                        super::DEFAULT_MAX_DELAY,
                        0,
                        Default::default(),
                        &[],
                    );
                    copy_property(w, copy, id);
                    copy_entry(w, copy, id);
                    if let Some(m) = w.mobile(mobile).map(|m| (m.map, m.location)) {
                        ops::move_to_world_map(w, id, m.1, m.0);
                    }
                    core::stop(w, id);
                    core::start(w, id);
                    core::respawn(w, id);
                }
                self.page = 0;
            }
            2 => {
                if index < len {
                    self.copy = Some(self.spawners[index as usize]);
                }
            }
            3 => {
                if index < len
                    && let Some(c) = self.copy
                {
                    copy_property(w, c, self.spawners[index as usize]);
                }
            }
            4 => {
                if index < len
                    && let Some(c) = self.copy
                {
                    copy_entry(w, c, self.spawners[index as usize]);
                }
            }
            5 => {
                if index < len {
                    let s = self.spawners[index as usize];
                    if let Some((loc, map)) = w.item(s).map(|i| (i.location, i.map)) {
                        muo_server::mobile::part1::lifecycle::set_map(w, mobile, map);
                        muo_server::mobile::part1::lifecycle::set_location(w, mobile, loc);
                    }
                }
                self.refresh(w, mobile, false);
                return;
            }
            6 | 7 => {
                if index < len {
                    let s = self.spawners[index as usize];
                    self.refresh(w, mobile, true);
                    if ty == 7 {
                        send_spawner_gump(w, mobile, s, None, 0);
                    } else {
                        open_props(w, mobile, EntityId::Item(s));
                    }
                    return;
                }
            }
            8 => {
                if index < len {
                    let s = self.spawners[index as usize];
                    let entry_index = if index >= line_count {
                        (index - line_count * self.page) * ENTRY_COUNT
                    } else {
                        index * ENTRY_COUNT
                    };
                    let t = |k: i32| info.get_text_entry(entry_index + k).map(str::to_owned);
                    if let Some(n) = t(0).filter(|n| !n.is_empty()) {
                        ops::set_name(w, s, Some(n));
                    }
                    if let Some(v) = t(1).and_then(|x| x.trim().parse::<i32>().ok()) {
                        core::set_walking_range(w, s, v);
                    }
                    if let Some(v) = t(2).and_then(|x| x.trim().parse::<i32>().ok()) {
                        core::set_home_range(w, s, v);
                    }
                    if let Some(v) = t(3).and_then(|x| TimeSpan::try_parse(&x))
                        && let Some(d) = data_mut(w, s)
                    {
                        d.min_delay = v;
                        ops::invalidate_properties(w, s);
                    }
                    if let Some(v) = t(4).and_then(|x| TimeSpan::try_parse(&x))
                        && let Some(d) = data_mut(w, s)
                    {
                        d.max_delay = v;
                        ops::invalidate_properties(w, s);
                    }
                }
            }
            9 => {
                if index < len {
                    let s = self.spawners[index as usize];
                    if let Some(t) = w.item(s).map(|i| i.ty) {
                        (t.delete)(w, s);
                    }
                }
            }
            10 => {
                self.page += 1;
                self.refresh(w, mobile, false);
                return;
            }
            11 => {
                if self.page > 0 {
                    self.page -= 1;
                }
                self.refresh(w, mobile, false);
                return;
            }
            _ => {}
        }
        let _ = &mut re_search;
        self.refresh(w, mobile, true);
    }
}

impl SpawnerControllerGump {
    fn refresh(&self, w: &mut World, mobile: MobileId, re_search: bool) {
        send_controller(
            w,
            mobile,
            self.page,
            self.copy,
            self.search.clone(),
            re_search,
            Some(self.spawners.clone()),
        );
    }

    fn draw_search(&self, b: &mut DynamicGumpBuilder, row0_y: i32, search_height: i32) {
        let mut pos = [0; 2];
        let mut widths = [0; 2];
        grid::compute_uniform_track_sizes(2, GUMP_WIDTH, 0, &mut pos, &mut widths);
        let (c0x, c1x) = (pos[0], pos[1]);
        let c1h = c1x + widths[1] / 2;
        let row0_vcenter = row0_y + search_height / 2;
        const DELTA_X: i32 = 140;
        b.add_html_styled(
            c0x,
            row0_y + 8,
            widths[0],
            30,
            "Search type",
            st(tc::GOLD, 4, TextAlignment::Center),
            false,
            false,
        );
        b.add_html_styled(
            c1x,
            row0_y + 8,
            widths[1],
            30,
            "Search Match",
            st(tc::GOLD, 4, TextAlignment::Center),
            false,
            false,
        );
        b.add_image_tiled(0, row0_y + 30, GUMP_WIDTH, 3, 9357);
        b.add_alpha_region(0, row0_y + 30, GUMP_WIDTH, 3);
        b.add_image_tiled(c1x, row0_y, 3, search_height, 9355);
        b.add_alpha_region(c1x, row0_y, 3, search_height);
        let sel = |t: SpawnSearchType| if self.search.ty == t { 9021 } else { 9020 };
        let (creature, coords, props, name) = (
            sel(SpawnSearchType::Creature),
            sel(SpawnSearchType::Coords),
            sel(SpawnSearchType::Props),
            sel(SpawnSearchType::Name),
        );
        b.add_button(c0x + 10, row0_y + 45, creature, creature, button(1, 0));
        b.add_html_styled(
            c0x + 15,
            row0_y + 48,
            100,
            30,
            "Creature",
            st(tc::WHITE, 4, TextAlignment::Center),
            false,
            false,
        );
        b.add_button(
            c0x + 10 + DELTA_X - 40,
            row0_y + 45,
            coords,
            coords,
            button(1, 1),
        );
        b.add_html_styled(
            c0x + 40 + DELTA_X - 40,
            row0_y + 48,
            150,
            30,
            "Range",
            st(tc::WHITE, 4, TextAlignment::Left),
            false,
            false,
        );
        b.add_button(
            c0x + 10 + DELTA_X * 2 - 100,
            row0_y + 45,
            props,
            props,
            button(1, 2),
        );
        b.add_html_styled(
            c0x + 40 + DELTA_X * 2 - 100,
            row0_y + 48,
            150,
            30,
            "Creature Props",
            st(tc::WHITE, 4, TextAlignment::Left),
            false,
            false,
        );
        b.add_button(
            c0x + 10 + DELTA_X * 3 - 100,
            row0_y + 45,
            name,
            name,
            button(1, 3),
        );
        b.add_html_styled(
            c0x + 40 + DELTA_X * 3 - 100,
            row0_y + 48,
            150,
            30,
            "Spawner Name",
            st(tc::WHITE, 4, TextAlignment::Left),
            false,
            false,
        );
        b.add_image_tiled(c1h - 100, row0_vcenter + 16, 200, 1, 9357);
        b.add_text_entry(
            c1h - 100,
            row0_vcenter,
            200,
            20,
            hues::WHITE,
            0xFFFF,
            &self.search.pattern,
        );
        b.add_button(c1h + 120, row0_vcenter, 4023, 4024, button(1, 4));
        let description = match self.search.ty {
            SpawnSearchType::Creature => "Search by creature name",
            SpawnSearchType::Coords => "Search by range (number)",
            SpawnSearchType::Props => "Search by entry property field",
            SpawnSearchType::Name => "Search by spawner name",
        };
        b.add_html_styled(
            c1x,
            row0_vcenter + 20,
            widths[1],
            30,
            description,
            st(tc::WHITE, 4, TextAlignment::Center),
            false,
            false,
        );
    }
}

/// `SpawnerControllerGump.CopyProperty(spawner, target)`.
pub fn copy_property(w: &mut World, spawner: ItemId, target: ItemId) {
    let Some(d) = data(w, spawner) else { return };
    let (min, max, bounds, home, group, count) = (
        d.min_delay,
        d.max_delay,
        d.spawn_bounds,
        d.spawn_location_is_home,
        d.group,
        d.count,
    );
    let walking = core::walking_range(w, spawner);
    let name = ops::name(w, spawner);
    ops::set_name(w, target, name);
    if let Some(t) = data_mut(w, target) {
        t.min_delay = min;
        t.max_delay = max;
    }
    core::set_walking_range(w, target, walking);
    core::set_spawn_bounds(w, target, bounds);
    if let Some(t) = data_mut(w, target) {
        t.spawn_location_is_home = home;
        t.group = group;
    }
    core::set_count(w, target, count);
}

/// `SpawnerControllerGump.CopyEntry(spawner, target)`.
pub fn copy_entry(w: &mut World, spawner: ItemId, target: ItemId) {
    if data(w, spawner).is_some_and(|d| !d.entries.is_empty()) {
        core::copy_entries_to(w, spawner, target);
    }
}

/// `[SpawnAdmin`.
pub fn register(w: &mut World) {
    CommandSystem::register(
        w,
        "SpawnAdmin",
        AccessLevel::GameMaster,
        &["Spawn"],
        Rc::new(|w: &mut World, e: &mut CommandEventArgs| {
            send_controller(w, e.mobile, 0, None, SpawnSearch::default(), true, None)
        }),
    );
}

// ---- SpawnPropsGump ------------------------------------------------------------------------------------------------------

/// `SpawnPropsGump.MobileAttributes`.
pub const MOBILE_ATTRIBUTES: [&str; 11] = [
    "Hue",
    "Str",
    "Dex",
    "Int",
    "HitsMaxSeed",
    "Hits",
    "DamageMin",
    "DamageMax",
    "ActiveSpeed",
    "PassiveSpeed",
    "VirtualArmor",
];

/// `SpawnPropsGump`: a `PropertiesGump` that shows only [`MOBILE_ATTRIBUTES`] and has an Apply button.
struct SpawnPropsGump {
    spawners: Vec<ItemId>,
}

/// `object.GetType().Name`.
fn type_name_of(w: &World, o: &PropObj) -> String {
    match o {
        PropObj::Entity(EntityId::Item(i)) => w.item(*i).map(|x| x.ty.full_name),
        PropObj::Entity(EntityId::Mobile(m)) => w.mobile(*m).map(|x| x.ty.full_name),
        PropObj::Other(_) => None,
    }
    .map(|f| f.rsplit('.').next().unwrap_or(f).to_owned())
    .unwrap_or_default()
}

/// `GetPropValue(src, name)` as `$"{attr} {prop}"` prints it: `None` for a missing property or a null.
fn get_prop_value(w: &mut World, o: &PropObj, attr: &str) -> Option<String> {
    match o {
        PropObj::Entity(e) => {
            let def = props::find_prop(w, *e, attr)?;
            let v = (def.get)(w, *e);
            (v != props::Value::Null).then(|| props::display(w, &def.kind, &v))
        }
        // StaticTarget.Hue is the only one of the attributes a non-entity target has
        PropObj::Other(t) => match t.downcast_ref::<Targeted>() {
            Some(Targeted::Static(s)) if attr == "Hue" => Some(s.hue.to_string()),
            _ => None,
        },
    }
}

impl PropsVariant for SpawnPropsGump {
    fn type_name(&self) -> &'static str {
        "Server.Gumps.SpawnPropsGump"
    }

    fn show_attribute(&self, name: &str) -> bool {
        MOBILE_ATTRIBUTES.iter().any(|a| a.eq_ic(name))
    }

    fn on_apply(&self, w: &mut World, from: MobileId, object: &PropObj) {
        let parts: Vec<String> = MOBILE_ATTRIBUTES
            .iter()
            .filter_map(|n| get_prop_value(w, object, n).map(|v| format!("{n} {v}")))
            .collect();
        let name = match object {
            PropObj::Other(t) => match t.downcast_ref::<Targeted>() {
                Some(Targeted::Land(_)) => "LandTarget".to_owned(),
                Some(Targeted::Static(_)) => "StaticTarget".to_owned(),
                _ => String::new(),
            },
            o => type_name_of(w, o),
        };
        let props_text = parts.join(" ");
        mx::send_message(w, from, "Updating spawners...");
        for s in self.spawners.clone() {
            super::commands::update_spawner(w, s, &name, None, Some(&props_text), None);
        }
        mx::send_message(w, from, "Update completed.");
    }
}

/// `from.SendGump(new SpawnPropsGump(from, targeted, spawners))`.
pub fn send_spawn_props(w: &mut World, from: MobileId, template: PropObj, spawners: Vec<ItemId>) {
    let g = properties::new_variant(w, from, template, Rc::new(SpawnPropsGump { spawners }));
    muo_gumps::m_z::common::send(w, from, g);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spawners::core::SPAWNER;
    use muo_content::gumps::base::Gump;
    use muo_server::mobile::MOBILE_BASE;
    use muo_server::net_state::NetStateData;

    /// `SearchSpawner()` returns early on an empty pattern: a refresh keeps the last list; the serial shows as 0x%08X.
    #[test]
    fn controller_keeps_its_list_on_an_empty_pattern() {
        let mut w = World::new();
        let sp = (SPAWNER.construct.unwrap())(&mut w);
        let gm = w.new_mobile(&MOBILE_BASE);
        let ns = NetStateId(1);
        w.net_states.insert(
            ns,
            NetStateData {
                mobile: Some(gm),
                ..Default::default()
            },
        );
        w.mobile_mut(gm).unwrap().net_state = Some(ns);
        send_controller(
            &mut w,
            gm,
            0,
            None,
            SpawnSearch::default(),
            true,
            Some(vec![sp]),
        );
        let g =
            muo_content::gumps::system::find_gump::<SpawnerControllerGump>(&w, ns).expect("sent");
        assert_eq!(g.spawners, vec![sp]);
        assert_eq!(g.rows.len(), 1);
    }

    /// SpawnerGump holds its `_entry` object: a list that changed between send and response moves no focus.
    #[test]
    fn spawner_gump_holds_its_entry_by_reference() {
        let mut w = World::new();
        let sp = (SPAWNER.construct.unwrap())(&mut w);
        let gm = w.new_mobile(&MOBILE_BASE);
        let ns = NetStateId(1);
        w.net_states.insert(
            ns,
            NetStateData {
                mobile: Some(gm),
                ..Default::default()
            },
        );
        for n in ["a", "b", "c"] {
            core::add_entry(&mut w, sp, Some(n), 100, 1, false, None, None);
        }
        let name_at = |w: &World, i: usize| data(w, sp).unwrap().entries[i].name.clone().unwrap();
        let expand = |row: i32| 1 + (row * 2) * 10 + 2;
        // "b" is open; "a" is deleted behind the gump's back, so "b" is now row 0
        let mut g = spawner_gump(&mut w, sp, Some(1), 0);
        core::remove_entry(&mut w, sp, 0);
        assert_eq!(name_at(&w, 0), "b");
        // row 0 is "b" again: Unexpand closes it (an index-held focus would have opened row 0 as a new focus)
        g.logic
            .on_response(&mut w, ns, &RelayInfo::new(expand(0), vec![], vec![]));
        assert_eq!(g.logic.entry, None);
        // open "b" again, then row 1 ("c") is a different entry: the focus moves to it
        g.logic
            .on_response(&mut w, ns, &RelayInfo::new(expand(0), vec![], vec![]));
        assert_eq!(g.logic.entry, data(&w, sp).map(|d| d.entries[0].uid));
        g.logic
            .on_response(&mut w, ns, &RelayInfo::new(expand(1), vec![], vec![]));
        assert_eq!(g.logic.entry, data(&w, sp).map(|d| d.entries[1].uid));
    }

    /// `SpawnPropsGump` on a non-entity target: the gump opens, Apply writes `Hue` of a StaticTarget into the
    /// entries named "StaticTarget", and a LandTarget has no attribute to write.
    #[test]
    fn spawn_props_apply_on_a_static_target() {
        use muo_gumps::m_z::props::PropObj;
        use muo_server::target::{LandTarget, StaticTarget};
        let mut w = World::new();
        muo_gumps::m_z::hooks::hooks_mut(&mut w).props =
            Some(Rc::new(muo_gumps::m_z::props::registry::RegistryHost));
        let sp = (SPAWNER.construct.unwrap())(&mut w);
        let gm = w.new_mobile(&MOBILE_BASE);
        let ns = NetStateId(1);
        w.net_states.insert(
            ns,
            NetStateData {
                mobile: Some(gm),
                ..Default::default()
            },
        );
        core::add_entry(&mut w, sp, Some("StaticTarget"), 100, 1, false, None, None);
        core::add_entry(&mut w, sp, Some("LandTarget"), 100, 1, false, None, None);
        core::add_entry(&mut w, sp, Some("Orc"), 100, 1, false, None, None);
        let st = StaticTarget {
            location: Point3D::new(1, 2, 3),
            item_id: 5,
            hue: 33,
        };
        let mut g = properties::new_variant(
            &mut w,
            gm,
            PropObj::Other(Rc::new(Targeted::Static(st))),
            Rc::new(SpawnPropsGump { spawners: vec![sp] }),
        );
        assert_eq!(
            g.core.type_id,
            muo_content::gumps::base::get_type_id("Server.Gumps.SpawnPropsGump")
        );
        g.on_response(&mut w, ns, &RelayInfo::new(3, vec![], vec![]));
        let props = |w: &World, i: usize| data(w, sp).unwrap().entries[i].properties.clone();
        assert_eq!(props(&w, 0).as_deref(), Some("Hue 33"));
        assert_eq!(props(&w, 1), None);
        let land = LandTarget {
            location: Point3D::new(1, 2, 3),
            tile_id: 3,
        };
        let mut g = properties::new_variant(
            &mut w,
            gm,
            PropObj::Other(Rc::new(Targeted::Land(land))),
            Rc::new(SpawnPropsGump { spawners: vec![sp] }),
        );
        g.on_response(&mut w, ns, &RelayInfo::new(3, vec![], vec![]));
        assert_eq!(props(&w, 1).as_deref(), Some(""));
    }
}
