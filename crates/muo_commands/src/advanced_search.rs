//! Advanced search: `AdvancedSearchCommand`, `AdvancedSearchGump` and its three confirm
//! gumps, `AdvancedSearchFilter`, `AdvancedSearchResult`, the result comparers, `AdvancedSearchUtilities`,
//! `AdvancedSearchConditions` (the leaf compiler is `generic::conditional::search_leaf`) and
//! `AdvancedSearchThreadWorker` (the filtering).
//!
//! The server is single threaded: [`do_search`] filters in one pass on the loop and the gump is resent at once, so
//! there is no `TryBeginSearch`, autosave toggle or worker thread.
//!
//! One search session per staff member (`Sessions`): the gump state is the session, `[AdvancedSearch` replaces
//! it, every response mutates and resends it.
//!
//! shortcut: a property test resolves names against the `[CommandProperty]` tables (`muo_content::props`) and the
//! `PUBLIC_ITEM` / `PUBLIC_MOBILE` lists of public properties (`generic::conditional`), not every public property of
//! the class; a name that is only a plain property outside those lists does not match.
//! Sorting is `slice::sort_by` over the comparers, which end in the serial, so the order is total (no ties).
//! The `Age` filter box is kept and drawn, but never applied.

use muo_primitives::dotnet::{IgnoreCase, InvariantCase};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use indexmap::IndexMap;
use muo_content::gumps::base::{LegacyGumpLogic, LegacyGumpObj, RelayInfo};
use muo_content::props::{self, ClassInfo};
use muo_primitives::{Point3D, TimeSpan, html};
use muo_server::commands::{CommandEventArgs, CommandSystem};
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::map::MapId;
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part2::messages::{send_localized_message, send_message};
use muo_server::net_state::NetStateId;
use muo_server::world::World;

use crate::generic::conditional::{Cond, search_leaf};
use crate::generic::{GenericCommands, Obj};
use crate::hooks::CommandHooks;
use crate::logging::CommandLogging;
use muo_content::gumps::base_grid::{mobile_of, send};

#[cfg(test)]
#[path = "advanced_search_behavior_tests.rs"]
mod behavior_tests;

const MAX_ENTRIES: usize = 18;

pub fn configure(w: &mut World) {
    CommandSystem::register(
        w,
        "AdvancedSearch",
        AccessLevel::Administrator,
        &["AdvSrch", "AS", "XmlFind"], // For those old school peeps!
        Rc::new(on_command),
    );
}

/// `[Usage("AdvancedSearch")]`
fn on_command(w: &mut World, e: &mut CommandEventArgs) {
    let from = e.mobile;
    w.resources
        .get_mut::<Sessions>()
        .0
        .insert(from, Session::default());
    let g = new_gump(w, from);
    muo_content::gumps::send_gump_to_mobile(w, from, Box::new(g), true);
}

// ---- AdvancedSearchFilter / AdvancedSearchResult ---------------------------------------------------------------

/// `AdvancedSearchFilter`.
#[derive(Clone, Default)]
pub struct Filter {
    pub filter_type: bool,
    pub filter_name: bool,
    pub filter_range: bool,
    pub filter_region: bool,
    pub filter_property_test: bool,
    pub filter_internal_map: bool,
    pub filter_null_map: bool,
    pub filter_age: bool,
    pub filter_age_direction: bool,
    pub hide_valid_internal_map: bool,
    pub filter_felucca: bool,
    pub filter_trammel: bool,
    pub filter_ilshenar: bool,
    pub filter_malas: bool,
    pub filter_tokuno: bool,
    pub filter_ter_mur: bool,
    pub age: Option<TimeSpan>,
    pub range: Option<i32>,
    pub region_name: Option<String>,
    pub property_test: Option<String>,
    /// The class, by full name.
    pub type_name: Option<&'static str>,
    pub name: Option<String>,
}

impl Filter {
    /// `AdvancedSearchGump.DefaultFilter`.
    pub fn default_filter() -> Filter {
        Filter {
            filter_type: true,
            filter_felucca: true,
            hide_valid_internal_map: true,
            type_name: Some("Server.Engines.Spawners.Spawner"),
            ..Filter::default()
        }
    }
}

/// `AdvancedSearchResult`.
#[derive(Clone)]
pub struct SearchResult {
    pub name: String,
    pub type_name: &'static str,
    pub location: Point3D,
    pub map: Option<MapId>,
    pub parent: Option<EntityId>,
    pub entity: Option<EntityId>,
    pub selected: bool,
}

impl SearchResult {
    fn live(w: &World, e: Option<EntityId>) -> Option<EntityId> {
        e.filter(|e| match e {
            EntityId::Item(i) => w.item(*i).is_some_and(|x| !x.deleted()),
            EntityId::Mobile(m) => w.mobile(*m).is_some_and(|x| !x.deleted),
        })
    }

    fn pos(w: &World, e: EntityId) -> (Point3D, Option<MapId>) {
        match e {
            EntityId::Item(i) => w.item(i).map(|x| (x.location, x.map)).unwrap_or_default(),
            EntityId::Mobile(m) => w.mobile(m).map(|x| (x.location, x.map)).unwrap_or_default(),
        }
    }

    /// `GetLocation()`: the live parent's, else the live entity's, else where it was found.
    pub fn get_location(&self, w: &World) -> Point3D {
        match Self::live(w, self.parent).or_else(|| Self::live(w, self.entity)) {
            Some(e) => Self::pos(w, e).0,
            None => self.location,
        }
    }

    /// `GetMap()`.
    pub fn get_map(&self, w: &World) -> Option<MapId> {
        match Self::live(w, self.parent).or_else(|| Self::live(w, self.entity)) {
            Some(e) => Self::pos(w, e).1,
            None => self.map,
        }
    }
}

// ---- the comparers ---------------------------------------------------------------------------------------------

#[derive(Copy, Clone, Eq, PartialEq)]
enum SortKey {
    Type,
    Name,
    Range,
    Map,
    Selected,
}

fn serial_of(r: &SearchResult) -> u32 {
    r.entity.map_or(0, |e| e.serial().value())
}

/// `.InsensitiveCompare`: ordinal, ignoring case.
fn insensitive_compare(a: &str, b: &str) -> Ordering {
    muo_primitives::dotnet::cmp_ignore_case(a, b)
}

fn compare(
    w: &World,
    from: MobileId,
    key: SortKey,
    reverse: bool,
    x: &SearchResult,
    y: &SearchResult,
) -> Ordering {
    let c = match key {
        SortKey::Type => {
            if reverse {
                insensitive_compare(y.type_name, x.type_name)
            } else {
                insensitive_compare(x.type_name, y.type_name)
            }
        }
        SortKey::Name => {
            if reverse {
                insensitive_compare(&y.name, &x.name)
            } else {
                insensitive_compare(&x.name, &y.name)
            }
        }
        SortKey::Map => {
            let id = |r: &SearchResult| r.map.and_then(|m| w.maps.get(m)).map_or(-1, |m| m.map_id);
            if reverse {
                id(y).cmp(&id(x))
            } else {
                id(x).cmp(&id(y))
            }
        }
        SortKey::Range => {
            let fm = w.mobile(from).and_then(|m| m.map);
            let floc = w.mobile(from).map(|m| m.location).unwrap_or_default();
            match (x.map == fm, y.map == fm) {
                (false, false) => Ordering::Equal,
                (true, false) => {
                    if reverse {
                        Ordering::Greater
                    } else {
                        Ordering::Less
                    }
                }
                (false, true) => {
                    if reverse {
                        Ordering::Less
                    } else {
                        Ordering::Greater
                    }
                }
                (true, true) => {
                    let d = |r: &SearchResult| {
                        let (dx, dy) = (
                            f64::from(floc.x - r.location.x),
                            f64::from(floc.y - r.location.y),
                        );
                        (dx * dx + dy * dy).sqrt()
                    };
                    let (xd, yd) = (d(x), d(y));
                    let o = if reverse {
                        yd.partial_cmp(&xd)
                    } else {
                        xd.partial_cmp(&yd)
                    };
                    o.unwrap_or(Ordering::Equal)
                }
            }
        }
        // True then false, which is 1 then 0, so the comparison is reverse of integers
        SortKey::Selected => {
            if reverse {
                x.selected.cmp(&y.selected)
            } else {
                y.selected.cmp(&x.selected)
            }
        }
    };
    c.then_with(|| serial_of(x).cmp(&serial_of(y)))
}

// ---- the session -----------------------------------------------------------------------------------------------

/// `AdvancedSearchGump`'s state.
#[derive(Clone)]
pub struct Session {
    pub filter: Filter,
    pub results: Option<Vec<SearchResult>>,
    pub display_from: usize,
    pub command_string: Option<String>,
    pub sort_descending: bool,
    pub sort_by_type: bool,
    pub sort_by_name: bool,
    pub sort_by_range: bool,
    pub sort_by_map: bool,
    pub sort_by_selected: bool,
    pub all_selected: bool,
}

impl Default for Session {
    fn default() -> Self {
        Session {
            filter: Filter::default_filter(),
            results: None,
            display_from: 0,
            command_string: None,
            sort_descending: false,
            sort_by_type: false,
            sort_by_name: false,
            sort_by_range: false,
            sort_by_map: false,
            sort_by_selected: false,
            all_selected: false,
        }
    }
}

#[derive(Default)]
pub struct Sessions(pub IndexMap<MobileId, Session>);

fn session(w: &mut World, m: MobileId) -> &mut Session {
    w.resources.get_mut::<Sessions>().0.entry(m).or_default()
}

/// `VisibleCount`: entries on the current page, bounded so a partial last page can't read past the end.
pub fn visible_count(total: usize, display_from: usize, max_entries: usize) -> usize {
    total.saturating_sub(display_from).min(max_entries)
}

// ---- the searching ---------------------------------------------------------------------------------------------

fn short(full: &str) -> &str {
    full.rsplit(['.', '+']).next().unwrap_or(full)
}

fn is_named(w: &World, e: EntityId, short_name: &str) -> bool {
    props::class_chain(w, e)
        .iter()
        .any(|c| short(c) == short_name)
}

fn entity_type_name(w: &World, e: EntityId) -> &'static str {
    match e {
        EntityId::Item(i) => w.item(i).map_or("", |x| x.ty.full_name),
        EntityId::Mobile(m) => w.mobile(m).map_or("", |x| x.ty.full_name),
    }
}

fn entity_name(w: &World, e: EntityId) -> Option<String> {
    match e {
        EntityId::Item(i) => Some(
            muo_server::item::ops::name(w, i)
                .or_else(|| {
                    w.item(i)
                        .map(|x| w.maps.tile_data.item(x.item_id as usize).name.clone())
                })
                .unwrap_or_default(),
        ),
        EntityId::Mobile(m) => w.mobile(m).and_then(|x| (x.ty.name)(w, m)),
    }
}

/// `OnASelectedMap`: with no map box ticked there is no constraint, else the entity's map must be a ticked one.
fn on_a_selected_map(f: &Filter, map: Option<MapId>) -> bool {
    let any = f.filter_felucca
        || f.filter_trammel
        || f.filter_ilshenar
        || f.filter_malas
        || f.filter_tokuno
        || f.filter_ter_mur
        || f.filter_internal_map
        || f.filter_null_map;
    if !any {
        return true;
    }
    match map {
        None => f.filter_null_map,
        Some(MapId::INTERNAL) => f.filter_internal_map,
        Some(m) => {
            (m == MapId::FELUCCA && f.filter_felucca)
                || (m == MapId::TRAMMEL && f.filter_trammel)
                || (m == MapId::ILSHENAR && f.filter_ilshenar)
                || (m == MapId::MALAS && f.filter_malas)
                || (m == MapId::TOKUNO && f.filter_tokuno)
                || (m == MapId::TER_MUR && f.filter_ter_mur)
        }
    }
}

fn is_valid_internal_item(w: &World, i: ItemId) -> bool {
    let Some(item) = w.item(i) else { return false };
    if item.parent.is_some() || item.held_by.is_some() {
        return true;
    }
    let e = EntityId::Item(i);
    ["Fists", "MountItem", "EffectItem", "MovingCrate", "BaseDockedBoat", "BaseBoat", "Plank", "TillerMan", "Hold"]
        .iter()
        .any(|n| is_named(w, e, n))
        // DisplayCache container
        || item.ty.full_name.contains("GenericBuyInfo+")
}

fn is_valid_internal_mobile(w: &mut World, m: MobileId) -> bool {
    // Logged out players
    if w.mobile(m).is_some_and(|x| x.account.is_some()) {
        return true;
    }
    // Stabled pets
    if muo_content::creature::data::cd(w, m).is_some_and(|d| d.is_stabled) {
        return true;
    }
    // Internalized vendors
    if is_named(w, EntityId::Mobile(m), "PlayerVendor")
        && w.resources
            .get_mut::<CommandHooks>()
            .vendor_has_house
            .is_some_and(|f| f(w, m))
    {
        return true;
    }
    // Currently mounted creatures
    muo_content::creature::mount::rider(w, m).is_some()
}

/// The per-search memo of compiled property tests, by runtime class.
type Predicates = HashMap<&'static str, Option<Vec<Vec<Cond>>>>;

/// `AdvancedSearchConditions.Parse`: OR of ANDs (`|` then `@`); a group with a dead leaf is dropped, no group left
/// is `None` (nothing can match).
fn build_predicate(w: &World, class: &ClassInfo, test: &str) -> Option<Vec<Vec<Cond>>> {
    let mut groups = Vec::new();
    for or_part in test.split('|') {
        let mut group: Vec<Cond> = Vec::new();
        let mut alive = true;
        for and_part in or_part.split('@') {
            match leaf(w, class, and_part) {
                Some(c) => group.push(c),
                None => {
                    alive = false;
                    break;
                }
            }
        }
        if alive {
            groups.push(group);
        }
    }
    (!groups.is_empty()).then_some(groups)
}

/// `AdvancedSearchConditions.Leaf`.
fn leaf(w: &World, class: &ClassInfo, expression: &str) -> Option<Cond> {
    let mut expr = expression.trim();
    if expr.is_empty() {
        return None;
    }
    let mut negate = false;
    if let Some(rest) = expr.strip_prefix('~') {
        negate = true;
        expr = rest;
    }
    let (index, op) = find_operator(expr)?;
    let name = expr[..index].trim();
    let value = expr[index + op.len()..].trim();
    if value.is_empty() {
        return None;
    }
    search_leaf(w, class, negate, op, name, value)
}

/// `AdvancedSearchUtilities.FindOperatorIndex`: the first of `= ! > < ~`, doubled when `<=`-style or `~x`.
pub fn find_operator(expression: &str) -> Option<(usize, &str)> {
    let index = expression.find(['=', '!', '>', '<', '~'])?;
    let bytes = expression.as_bytes();
    // We are at the end
    if index + 1 == expression.len() {
        return Some((index, &expression[index..=index]));
    }
    // <=, >=, ~<, ~>, ~~, ~=, ~!
    let (op, next) = (bytes[index], bytes[index + 1]);
    let double = (next == b'=' && matches!(op, b'=' | b'<' | b'>' | b'~' | b'!'))
        || (op == b'~' && matches!(next, b'<' | b'>' | b'~' | b'!'));
    Some((
        index,
        &expression[index..index + if double { 2 } else { 1 }],
    ))
}

fn passes_property_test(
    w: &mut World,
    predicates: &mut Predicates,
    filter: &Filter,
    e: EntityId,
) -> bool {
    let Some(test) = filter
        .property_test
        .as_deref()
        .filter(|t| !t.trim().is_empty())
    else {
        return false;
    };
    let full = entity_type_name(w, e);
    let preds = predicates.entry(full).or_insert_with(|| {
        let mut chain = props::class_chain(w, e);
        chain.reverse();
        let class = ClassInfo {
            full_name: full,
            chain,
            is_item: matches!(e, EntityId::Item(_)),
        };
        build_predicate(w, &class, test)
    });
    preds
        .as_ref()
        .is_some_and(|groups| groups.iter().any(|g| g.iter().all(|c| c.matches(w, e))))
}

/// `DoEntitySearchCore`; `ignored` collects what `HandleValidInternal` marks.
fn search_entity(
    w: &mut World,
    from_loc: (Point3D, Option<MapId>),
    filter: &Filter,
    predicates: &mut Predicates,
    ignored: &mut HashSet<EntityId>,
    e: EntityId,
) -> Option<SearchResult> {
    // `DoEntitySearch`: a deleted entity is skipped.
    SearchResult::live(w, Some(e))?;
    let (eloc, emap) = SearchResult::pos(w, e);
    if !on_a_selected_map(filter, emap) {
        return None;
    }
    let location = match e {
        EntityId::Item(i) => muo_server::item::ops::get_world_location(w, i),
        EntityId::Mobile(_) => eloc,
    };
    if filter.filter_range
        && (filter.range.is_none_or(|r| r < 0)
            || emap != from_loc.1
            || !muo_primitives::utility::in_range(
                from_loc.0.x,
                from_loc.0.y,
                location.x,
                location.y,
                filter.range.unwrap_or(0),
            ))
    {
        return None;
    }
    if filter.filter_region {
        let name = filter
            .region_name
            .as_deref()
            .filter(|n| !n.trim().is_empty())?;
        let mut r = Some(muo_server::region::find(w, location, emap));
        let mut part_of = false;
        while let Some(id) = r {
            let Some(reg) = w.region(id) else { break };
            if reg.name.as_deref().is_some_and(|n| n.eq_ic(name)) {
                part_of = true;
                break;
            }
            r = reg.parent;
        }
        if !part_of {
            return None;
        }
    }
    // After the cheap filters, so non-qualifying entities skip the house/keyring enumeration.
    if filter.hide_valid_internal_map
        && let Some(f) = w.resources.get_mut::<CommandHooks>().valid_internal_held
    {
        ignored.extend(f(w, e));
    }
    let name = entity_name(w, e);
    let name_filter = filter
        .filter_name
        .then_some(filter.name.as_deref())
        .flatten()
        .filter(|n| !n.trim().is_empty());
    if let Some(n) = name_filter
        && !name
            .as_deref()
            .unwrap_or("")
            .lower_inv()
            .contains(&n.lower_inv())
    {
        return None;
    }
    if filter.hide_valid_internal_map && emap == Some(MapId::INTERNAL) {
        let valid = match e {
            EntityId::Item(i) => is_valid_internal_item(w, i),
            EntityId::Mobile(m) => is_valid_internal_mobile(w, m),
        };
        if !valid {
            return None;
        }
    }
    if filter.filter_property_test && !passes_property_test(w, predicates, filter, e) {
        return None;
    }
    let parent = match e {
        EntityId::Item(i) => muo_server::item::ops::root_parent(w, i),
        EntityId::Mobile(_) => None,
    };
    Some(SearchResult {
        name: name.unwrap_or_default(),
        type_name: short(entity_type_name(w, e)),
        location: eloc,
        map: emap,
        parent,
        entity: Some(e),
        selected: false,
    })
}

/// `DoSearch`: filters every item, then every mobile of the filter's class.
pub fn do_search(w: &mut World, from: MobileId) {
    let s = session(w, from).clone();
    let filter = s.filter;
    let from_loc = w
        .mobile(from)
        .map(|m| (m.location, m.map))
        .unwrap_or_default();
    let class = filter.filter_type.then_some(filter.type_name).flatten();
    let mut entities: Vec<EntityId> = Vec::new();
    entities.extend(w.items.keys().map(|k| EntityId::Item(*k)));
    entities.extend(w.mobiles.keys().map(|k| EntityId::Mobile(*k)));
    let mut predicates = Predicates::new();
    let mut ignored: HashSet<EntityId> = HashSet::new();
    let mut results: Vec<SearchResult> = Vec::new();
    for e in entities {
        if class.is_some_and(|c| !props::is_class(w, e, c)) {
            continue;
        }
        if let Some(r) = search_entity(w, from_loc, &filter, &mut predicates, &mut ignored, e) {
            results.push(r);
        }
    }
    results.retain(|r| r.entity.is_none_or(|e| !ignored.contains(&e)));
    results.sort_by_key(serial_of);
    let s = session(w, from);
    s.results = Some(results);
    resend(w, from);
}

/// `Sort(from)`.
pub fn sort(w: &mut World, from: MobileId) {
    let s = session(w, from).clone();
    let Some(mut results) = s.results else { return };
    if results.is_empty() {
        return;
    }
    let key = if s.sort_by_type {
        SortKey::Type
    } else if s.sort_by_name {
        SortKey::Name
    } else if s.sort_by_range {
        SortKey::Range
    } else if s.sort_by_map {
        SortKey::Map
    } else if s.sort_by_selected {
        SortKey::Selected
    } else {
        return;
    };
    muo_primitives::dotnet::net_sort(&mut results, |a, b| {
        compare(w, from, key, s.sort_descending, a, b)
    }); // Array.Sort
    session(w, from).results = Some(results);
}

/// `ExecuteCommand(from, commandString)`.
fn execute_command(w: &mut World, from: MobileId, command_string: &str) {
    if command_string.trim().is_empty() {
        return;
    }
    let list: Vec<Obj> = session(w, from)
        .results
        .iter()
        .flatten()
        .filter(|r| r.selected)
        .filter_map(|r| r.entity)
        .map(muo_server::target::Targeted::Entity)
        .collect();
    if list.is_empty() {
        return;
    }
    let span = command_string.trim();
    let parts: Vec<&str> = span.split(' ').collect();
    let command = parts[0];
    let args: Vec<String> = parts[1..].iter().map(|s| (*s).to_string()).collect();
    // The last command that has the name wins: only the inner loop stops at a match.
    let cmd = w.resources.get::<GenericCommands>().and_then(|g| {
        g.all
            .iter()
            .rfind(|c| c.spec().commands.iter().any(|n| n.eq_ic(command)))
            .cloned()
    });
    let Some(cmd) = cmd else {
        send_message(w, from, &format!("Invalid command: {span}"));
        return;
    };
    let e = CommandEventArgs::new(from, command.to_string(), command_string.to_string(), args);
    let big = list.len() > 20;
    if big {
        w.resources.get_mut::<CommandLogging>().enabled = false;
    }
    let mut run = crate::generic::Run::default();
    cmd.execute_list(w, &mut run, &e, &list);
    if big {
        w.resources.get_mut::<CommandLogging>().enabled = true;
    }
    run.flush(w, from, big);
}

/// `Resend(m)`: `Reset(); Build(); m.SendGump(this)`.
pub fn resend(w: &mut World, m: MobileId) {
    let g = new_gump(w, m);
    send(w, m, g);
}

// ---- AdvancedSearchGump ----------------------------------------------------------------------------------------

pub struct AdvancedSearchGump {
    owner: MobileId,
}

pub fn new_gump(w: &mut World, owner: MobileId) -> LegacyGumpObj<AdvancedSearchGump> {
    let s = session(w, owner).clone();
    let mut g = LegacyGumpObj::new(50, 50, AdvancedSearchGump { owner });
    build(w, &mut g.layout, &s);
    g
}

fn build(w: &World, l: &mut muo_content::gumps::legacy::LegacyLayout, s: &Session) {
    const HEIGHT: i32 = 500;
    let results = s.results.as_deref().unwrap_or(&[]);
    let have_results = !results.is_empty();
    let width = if have_results { 755 } else { 170 };
    l.add_background(0, 0, width, HEIGHT, 5054);
    l.add_alpha_region(0, 0, width, HEIGHT);

    // Sorting
    let mut y = 5;
    l.add_button(5, y, 0xFAB, 0xFAD, 700);
    l.add_label(38, y, 0x384, "Sort");
    if s.sort_descending {
        l.add_button(75, y + 3, 0x15E2, 0x15E6, 701);
        l.add_label(95, y, 0x384, "Desc");
    } else {
        l.add_button(75, y + 3, 0x15E0, 0x15E4, 701);
        l.add_label(95, y, 0x384, "Asc");
    }
    y += 22;
    l.add_radio(5, y, 0xD2, 0xD3, s.sort_by_type, 0);
    l.add_label(28, y, 0x384, "Type");
    l.add_radio(75, y, 0xD2, 0xD3, s.sort_by_name, 1);
    l.add_label(98, y, 0x384, "Name");
    y += 20;
    l.add_radio(5, y, 0xD2, 0xD3, s.sort_by_range, 2);
    l.add_label(28, y, 0x384, "Range");
    l.add_radio(75, y, 0xD2, 0xD3, s.sort_by_map, 3);
    l.add_label(98, y, 0x384, "Map");
    y += 20;
    l.add_radio(5, y, 0xD2, 0xD3, s.sort_by_selected, 4);
    l.add_label(28, y, 0x384, "Select");

    // Searching
    let f = &s.filter;
    y = 85;
    l.add_button(5, y, 0xFA8, 0xFAA, 3);
    l.add_label(38, y, 0x384, "Search");
    y += 20;
    l.add_check(5, y, 0xD2, 0xD3, f.filter_internal_map, 312);
    l.add_label(28, y, 0x384, "Internal");
    l.add_check(75, y, 0xD2, 0xD3, f.filter_null_map, 314);
    l.add_label(98, y, 0x384, "Null");
    y += 20;
    l.add_check(5, y, 0xD2, 0xD3, f.filter_felucca, 308);
    l.add_label(28, y, 0x384, "Fel");
    l.add_check(75, y, 0xD2, 0xD3, f.filter_trammel, 309);
    l.add_label(98, y, 0x384, "Tram");
    y += 20;
    l.add_check(5, y, 0xD2, 0xD3, f.filter_malas, 310);
    l.add_label(28, y, 0x384, "Mal");
    l.add_check(75, y, 0xD2, 0xD3, f.filter_ilshenar, 311);
    l.add_label(98, y, 0x384, "Ilsh");
    y += 20;
    l.add_check(5, y, 0xD2, 0xD3, f.filter_tokuno, 318);
    l.add_label(28, y, 0x384, "Tok");
    l.add_check(75, y, 0xD2, 0xD3, f.filter_ter_mur, 320);
    l.add_label(98, y, 0x384, "Ter");
    y += 20;
    l.add_check(5, y, 0xD2, 0xD3, f.hide_valid_internal_map, 316);
    l.add_label(28, y, 0x384, "Hide valid internal");

    // Filter
    y = HEIGHT - 295;
    l.add_label(28, y, 0x384, "Region");
    l.add_image_tiled(70, y, 68, 19, 0xBBC);
    l.add_text_entry(
        70,
        y,
        250,
        19,
        0,
        106,
        f.region_name.as_deref().unwrap_or(""),
    );
    l.add_check(5, y, 0xD2, 0xD3, f.filter_region, 319);
    y += 20;
    l.add_label(28, y, 0x384, "Age");
    l.add_image_tiled(70, y, 68, 19, 0xBBC);
    l.add_text_entry(
        70,
        y,
        250,
        19,
        0,
        105,
        &f.age.map_or(String::new(), |a| a.to_string()),
    );
    l.add_check(5, y, 0xD2, 0xD3, f.filter_age, 303);
    l.add_check(50, y + 2, 0x1467, 0x1468, f.filter_age_direction, 302);
    y += 20;
    l.add_label(28, y, 0x384, "Range");
    l.add_image_tiled(70, y, 45, 19, 0xBBC);
    l.add_text_entry(
        70,
        y,
        45,
        19,
        0,
        100,
        &f.range.map_or(String::new(), |r| r.to_string()),
    );
    l.add_check(5, y, 0xD2, 0xD3, f.filter_range, 304);
    y += 20;
    l.add_label(28, y, 0x384, "Type");
    l.add_check(5, y, 0xD2, 0xD3, f.filter_type, 305);
    l.add_image_tiled(6, y + 20, 132, 19, 0xBBC);
    l.add_text_entry(6, y + 20, 250, 19, 0, 101, f.type_name.map_or("", short));
    y += 41;
    l.add_label(28, y, 0x384, "Property Test");
    l.add_check(5, y, 0xD2, 0xD3, f.filter_property_test, 315);
    l.add_image_tiled(6, y + 20, 132, 19, 0xBBC);
    l.add_text_entry(
        6,
        y + 20,
        500,
        19,
        0,
        104,
        f.property_test.as_deref().unwrap_or(""),
    );
    y += 41;
    l.add_label(28, y, 0x384, "Name");
    l.add_check(5, y, 0xD2, 0xD3, f.filter_name, 306);
    l.add_image_tiled(6, y + 20, 132, 19, 0xBBC);
    l.add_text_entry(6, y + 20, 250, 19, 0, 102, f.name.as_deref().unwrap_or(""));

    if !have_results {
        return;
    }

    // Control Buttons
    y = HEIGHT - 25;
    l.add_button(150, y, 0xFB1, 0xFB3, 156);
    l.add_label(183, y, 0x384, "Delete");
    // For spawners in the list
    l.add_button(230, y, 0xFA2, 0xFA3, 157);
    l.add_label(263, y, 0x384, "Reset");
    l.add_button(310, y, 0xFA8, 0xFAA, 158);
    l.add_label(343, y, 0x384, "Respawn");
    l.add_button(5, y, 0xFAE, 0xFAF, 154);
    l.add_label(38, y, 0x384, "Bring");
    l.add_button(470, y - 25, 0xFA8, 0xFAA, 160);
    l.add_label(503, y - 25, 0x384, "Command:");
    l.add_image_tiled(560, y - 25, 180, 19, 0xBBC);
    l.add_text_entry(
        560,
        y - 25,
        180,
        19,
        0,
        301,
        s.command_string.as_deref().unwrap_or(""),
    );
    if s.display_from > 0 {
        l.add_button(395, HEIGHT - 25, 0x15E3, 0x15E7, 202); // backward
    }
    if s.display_from + MAX_ENTRIES < results.len() {
        l.add_button(415 + 25, HEIGHT - 25, 0x15E1, 0x15E5, 201); // forward
    }
    let max_entry_index = (s.display_from + MAX_ENTRIES).min(results.len() - 1);
    // Headers
    l.add_label(143, 5, 0x384, "Gump");
    l.add_label(178, 5, 0x384, "Prop");
    l.add_label(210, 5, 0x384, "Goto");
    l.add_label(250, 5, 0x384, "Name");
    l.add_label(365, 5, 0x384, "Type");
    l.add_label(460, 5, 0x384, "Location");
    l.add_label(578, 5, 0x384, "Map");
    l.add_label(650, 5, 0x384, "Owner");
    l.add_label(
        180,
        y - 50,
        68,
        &format!("Found {} items/mobiles", results.len()),
    );
    l.add_label(
        400,
        y - 50,
        68,
        &format!("Displaying {}-{}", s.display_from + 1, max_entry_index + 1),
    );
    // Count the number of selected objects
    let count = results.iter().filter(|e| e.selected).count();
    l.add_label(600, y - 50, 33, &format!("Selected {count}"));
    l.add_label(610, y, 0x384, "Select All");
    // display the select-all toggle
    let (n, p) = if s.all_selected {
        (0xD3, 0xD2)
    } else {
        (0xD2, 0xD3)
    };
    l.add_button(670, y, n, p, 1);

    let mut all_displayed_selected = true;
    // Bound to this page's real entries so a partial last page still renders in descending mode.
    let visible = visible_count(results.len(), s.display_from, MAX_ENTRIES);
    for i in 0..visible {
        let offset = if s.sort_descending {
            visible - 1 - i
        } else {
            i
        };
        let index = offset + s.display_from;
        let entry = &results[index];
        if !entry.selected {
            all_displayed_selected = false;
        }
        let row = 22 * i as i32;
        l.add_image_tiled(235, row + 30, 386, 23, 0x52);
        l.add_image_tiled(236, row + 31, 384, 21, 0xBBC);
        if entry.entity.is_some_and(|e| is_named(w, e, "BaseSpawner")) {
            l.add_button(145, row + 30, 0xFBD, 0xFBE, 2000 + offset as i32);
        }
        // Goto button
        l.add_button(205, row + 30, 0xFAE, 0xFAF, 1000 + offset as i32);
        // Interface button
        l.add_button(175, row + 30, 0xFAB, 0xFAD, 3000 + offset as i32);

        let mut text_hue = 0;
        let mut parent_name = String::new();
        let deleted = SearchResult::live(w, entry.entity).is_none();
        let loc = entry.get_location(w);
        let map = entry.get_map(w);
        match entry.parent {
            Some(EntityId::Mobile(pm)) => {
                text_hue = if w.mobile(pm).is_some_and(|m| m.player) {
                    44
                } else {
                    24
                };
                parent_name = w
                    .mobile(pm)
                    .and_then(|m| (m.ty.name)(w, pm))
                    .unwrap_or_default();
            }
            Some(EntityId::Item(pi)) => {
                text_hue = 5;
                parent_name = entity_name(w, EntityId::Item(pi)).unwrap_or_default();
            }
            None => {}
        }
        // Name
        l.add_label_cropped(
            248,
            row + 31,
            110,
            21,
            if deleted { 5 } else { 0 },
            &entry.name,
        );
        // Type
        l.add_image_tiled(360, row + 31, 90, 21, 0xBBC);
        l.add_label_cropped(360, row + 31, 90, 21, 0, entry.type_name);
        // Location
        l.add_image_tiled(450, row + 31, 137, 21, 0xBBC);
        l.add_label(450, row + 31, 0, &loc.to_string());
        // Map
        l.add_image_tiled(571, row + 31, 70, 21, 0xBBC);
        l.add_label(
            571,
            row + 31,
            0,
            &map.and_then(|m| w.maps.get(m))
                .map_or("(-null-)".to_string(), |m| m.name().to_string()),
        );
        // Parent
        l.add_image_tiled(640, row + 31, 90, 21, 0xBBC);
        l.add_label_cropped(640, row + 31, 90, 21, text_hue, &parent_name);
        // display the selection button
        let (n, p) = if entry.selected {
            (0xD3, 0xD2)
        } else {
            (0xD2, 0xD3)
        };
        l.add_button(730, row + 32, n, p, 4000 + offset as i32);
    }
    let (n, p) = if all_displayed_selected {
        (0xD3, 0xD2)
    } else {
        (0xD2, 0xD3)
    };
    l.add_button(730, 5, n, p, 2); // Select all displayed
}

impl LegacyGumpLogic for AdvancedSearchGump {
    const TYPE_NAME: &'static str = "Server.Engines.AdvancedSearch.AdvancedSearchGump";

    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        let from = self.owner;
        // SetSortSwitches(info.Switches.Length > 0 ? info.Switches[0] : -1)
        let radio = info.switches.first().copied().unwrap_or(-1);
        let switched = |id: i32| info.is_switched(id);
        let text = |id: i32| info.get_text_entry(id).map(str::to_string);
        let type_name = text(101).and_then(|t| {
            // `AssemblyHandler.FindTypeByName(text)`: the class named exactly so.
            props::find_class(w, &t)
                .filter(|c| short(c.full_name) == t || c.full_name == t)
                .map(|c| c.full_name)
        });
        {
            let s = session(w, from);
            s.sort_by_type = radio == 0;
            s.sort_by_name = radio == 1;
            s.sort_by_range = radio == 2;
            s.sort_by_map = radio == 3;
            s.sort_by_selected = radio == 4;
            let f = &mut s.filter;
            f.filter_age_direction = switched(302);
            f.filter_age = switched(303);
            f.filter_range = switched(304);
            f.filter_type = switched(305);
            f.filter_name = switched(306);
            f.filter_felucca = switched(308);
            f.filter_trammel = switched(309);
            f.filter_malas = switched(310);
            f.filter_ilshenar = switched(311);
            f.filter_internal_map = switched(312);
            f.filter_null_map = switched(314);
            f.filter_property_test = switched(315);
            f.hide_valid_internal_map = switched(316);
            f.filter_tokuno = switched(318);
            f.filter_region = switched(319);
            f.filter_ter_mur = switched(320);
            f.range = text(100).map(|t| muo_primitives::parse::to_i32(&t));
            f.type_name = type_name;
            f.name = text(102);
            f.property_test = text(104);
            f.age = text(105).map(|t| TimeSpan::try_parse(&t).unwrap_or(TimeSpan::ZERO));
            f.region_name = text(106);
            s.command_string = text(301);
        }
        let button = info.button_id;
        let selected_count = |w: &mut World| {
            session(w, from)
                .results
                .iter()
                .flatten()
                .filter(|r| r.selected)
                .count()
        };
        match button {
            0 => return, // Close
            1 => {
                // Select all toggle
                let s = session(w, from);
                s.all_selected = !s.all_selected;
                let all = s.all_selected;
                for r in s.results.iter_mut().flatten() {
                    r.selected = all;
                }
            }
            2 => {
                // Select all displayed
                let s = session(w, from);
                let from_i = s.display_from;
                if let Some(results) = s.results.as_mut() {
                    let end = (from_i + MAX_ENTRIES).min(results.len());
                    let all = results[from_i.min(end)..end].iter().all(|r| r.selected);
                    for r in &mut results[from_i.min(end)..end] {
                        r.selected = !all;
                    }
                }
            }
            3 => {
                do_search(w, from);
                return; // the gump is resent by the search
            }
            154 => {
                // Bring
                let count = selected_count(w);
                if count > 0 {
                    let g = new_warning(
                        "Bring selected objects",
                        30720,
                        &format!("Bring {count} objects?"),
                        0xFFC000,
                        480,
                        360,
                        WarningKind::Bring,
                        from,
                    );
                    send(w, from, g);
                }
                return;
            }
            156 => {
                // Delete
                let count = selected_count(w);
                if count > 0 {
                    let g = new_warning(
                        "Delete selected objects",
                        30720,
                        &format!("Delete {count} objects?"),
                        0xFFC000,
                        480,
                        360,
                        WarningKind::Delete,
                        from,
                    );
                    send(w, from, g);
                }
                return;
            }
            157 => {
                // Reset
                let targets: Vec<EntityId> = session(w, from)
                    .results
                    .iter()
                    .flatten()
                    .filter(|r| r.selected)
                    .filter_map(|r| r.entity)
                    .collect();
                if let Some(f) = w.resources.get_mut::<CommandHooks>().spawner_reset {
                    for e in targets {
                        if let EntityId::Item(i) = e
                            && is_named(w, e, "Spawner")
                        {
                            f(w, i);
                        }
                    }
                }
            }
            158 => {
                // Respawn
                let targets: Vec<EntityId> = session(w, from)
                    .results
                    .iter()
                    .flatten()
                    .filter(|r| r.selected)
                    .filter_map(|r| r.entity)
                    .collect();
                if let Some(f) = w.resources.get_mut::<CommandHooks>().spawner_respawn {
                    for e in targets {
                        f(w, e);
                    }
                }
            }
            160 => {
                // Command
                let c = session(w, from).command_string.clone().unwrap_or_default();
                execute_command(w, from, &c);
            }
            201 => {
                // Forward
                let s = session(w, from);
                if let Some(n) = s.results.as_ref().map(Vec::len).filter(|n| *n > 0) {
                    let max_entry_index = (s.display_from + MAX_ENTRIES).min(n - 1);
                    if max_entry_index < n - 1 {
                        s.display_from = max_entry_index;
                    }
                }
            }
            202 => {
                // Backward
                let s = session(w, from);
                if let Some(n) = s.results.as_ref().map(Vec::len).filter(|n| *n > 0) {
                    s.display_from = s.display_from.saturating_sub(MAX_ENTRIES).min(n - 1);
                }
            }
            700 => sort(w, from),
            701 => {
                // Change sort order
                let s = session(w, from);
                s.sort_descending = !s.sort_descending;
                if (0..=4).contains(&radio) {
                    sort(w, from);
                }
            }
            1000..=1999 => {
                // Goto
                let index = (button - 1000) as usize + session(w, from).display_from;
                let entry = session(w, from)
                    .results
                    .as_ref()
                    .and_then(|r| r.get(index))
                    .cloned();
                if let Some(entry) = entry {
                    let (loc, map) = (entry.get_location(w), entry.get_map(w));
                    if let Some(map) = map.filter(|m| *m != MapId::INTERNAL) {
                        muo_server::mobile::base::move_to_world(w, from, loc, map);
                    }
                }
            }
            2000..=2999 => {
                // Open Spawner
                let index = (button - 2000) as usize + session(w, from).display_from;
                let entry = session(w, from)
                    .results
                    .as_ref()
                    .and_then(|r| r.get(index))
                    .cloned();
                let Some(e) = entry.and_then(|r| SearchResult::live(w, r.entity)) else {
                    resend(w, from);
                    return;
                };
                let map = SearchResult::pos(w, e).1;
                if map.is_none() || map == Some(MapId::INTERNAL) {
                    resend(w, from);
                    return;
                }
                resend(w, from);
                if let (EntityId::Item(i), Some(f)) =
                    (e, w.resources.get_mut::<CommandHooks>().spawner_open)
                    && is_named(w, e, "BaseSpawner")
                {
                    f(w, from, i);
                }
                return;
            }
            3000..=3999 => {
                // Props
                let index = (button - 3000) as usize + session(w, from).display_from;
                let entry = session(w, from)
                    .results
                    .as_ref()
                    .and_then(|r| r.get(index))
                    .cloned();
                let Some(e) = entry.and_then(|r| SearchResult::live(w, r.entity)) else {
                    resend(w, from);
                    return;
                };
                if !crate::generic::is_accessible(
                    w,
                    from,
                    Some(muo_server::target::Targeted::Entity(e)),
                ) {
                    send_localized_message(w, from, 500447, "", 0x3B2); // That is not accessible.
                    resend(w, from);
                    return;
                }
                resend(w, from);
                crate::properties::open_properties_gump(w, from, e);
                return;
            }
            4000..=4999 => {
                // Select
                let index = (button - 4000) as usize + session(w, from).display_from;
                if let Some(r) = session(w, from)
                    .results
                    .as_mut()
                    .and_then(|r| r.get_mut(index))
                {
                    r.selected = !r.selected;
                }
            }
            _ => {}
        }
        resend(w, from);
    }
}

// ---- the warning gumps -----------------------------------------------------------------------------------------

#[derive(Copy, Clone)]
pub enum WarningKind {
    Bring,
    Delete,
}

pub struct AdvancedSearchWarningGump {
    kind: WarningKind,
    owner: MobileId,
}

/// `AdvancedSearchWarningGump(header, headerColor, content, contentColor, width, height)` for the bring and delete
/// confirmations.
#[allow(clippy::too_many_arguments)]
pub fn new_warning(
    header: &str,
    header_color: i32,
    content: &str,
    content_color: i32,
    width: i32,
    height: i32,
    kind: WarningKind,
    owner: MobileId,
) -> LegacyGumpObj<AdvancedSearchWarningGump> {
    let mut g = LegacyGumpObj::new(
        (640 - width) / 2,
        (480 - height) / 2,
        AdvancedSearchWarningGump { kind, owner },
    );
    // the gump is the AdvancedSearchConfirmBringGump / AdvancedSearchConfirmDeleteGump subclass
    g.core.type_id = muo_content::gumps::base::get_type_id(g.logic.gump_type_name());
    let l = &mut g.layout;
    l.closable = false;
    l.add_page(0);
    l.add_background(0, 0, width, height, 5054);
    l.add_image_tiled(10, 10, width - 20, 20, 2624);
    l.add_alpha_region(10, 10, width - 20, 20);
    l.add_html(
        10,
        10,
        width - 20,
        20,
        &html::color_rgb(header, header_color),
        false,
        false,
    );
    l.add_image_tiled(10, 40, width - 20, height - 80, 2624);
    l.add_alpha_region(10, 40, width - 20, height - 80);
    if !content.trim().is_empty() {
        l.add_html(
            10,
            40,
            width - 20,
            height - 80,
            &html::color_rgb(content, content_color),
            false,
            true,
        );
    }
    l.add_image_tiled(10, height - 30, width - 20, 20, 2624);
    l.add_alpha_region(10, height - 30, width - 20, 20);
    l.add_button(10, height - 30, 4005, 4007, 1);
    l.add_html_localized_color(40, height - 30, 170, 20, 1011036, 32767, false, false); // OKAY
    l.add_button(10 + (width - 20) / 2, height - 30, 4005, 4007, 0);
    l.add_html_localized_color(
        40 + (width - 20) / 2,
        height - 30,
        170,
        20,
        1011012,
        32767,
        false,
        false,
    ); // CANCEL
    g
}

impl LegacyGumpLogic for AdvancedSearchWarningGump {
    const TYPE_NAME: &'static str = "Server.Engines.AdvancedSearch.AdvancedSearchConfirmBringGump";

    fn gump_type_name(&self) -> &'static str {
        match self.kind {
            WarningKind::Bring => "Server.Engines.AdvancedSearch.AdvancedSearchConfirmBringGump",
            WarningKind::Delete => "Server.Engines.AdvancedSearch.AdvancedSearchConfirmDeleteGump",
        }
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        let okay = info.button_id == 1;
        let _ = self.owner;
        let selected: Vec<EntityId> = session(w, from)
            .results
            .iter()
            .flatten()
            .filter(|r| r.selected)
            .filter_map(|r| r.entity)
            .collect();
        if okay {
            match self.kind {
                WarningKind::Bring => {
                    let (loc, map) = w
                        .mobile(from)
                        .map(|m| (m.location, m.map))
                        .unwrap_or_default();
                    for e in selected {
                        match e {
                            EntityId::Item(i) => {
                                muo_server::item::ops::move_to_world_map(w, i, loc, map)
                            }
                            EntityId::Mobile(m) => {
                                if let Some(map) = map {
                                    muo_server::mobile::base::move_to_world(w, m, loc, map);
                                }
                            }
                        }
                    }
                }
                WarningKind::Delete => {
                    for e in selected {
                        match e {
                            EntityId::Item(i) => muo_server::item::base::delete(w, i),
                            EntityId::Mobile(m) => muo_server::mobile::base::delete(w, m),
                        }
                    }
                }
            }
        }
        resend(w, from);
    }
}
