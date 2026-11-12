//! Dueling arenas: the `ArenaController` item (it embeds the `Arena` in its save), the arena's
//! start points, finding a free arena for a duel, moving the players in and evicting them, and the two items a duel
//! makes at the arena (`DuelContext+ExitTeleporter`, `DuelContext+InternalWall`).
//!
//! The `[CommandProperty]` surface of `Arena`/`ArenaStartPoints`/`ArenaController` is in `props` (the nested `[props`
//! editor, `ForceEvict`) and the private-arena house test of `FindArena` (`BaseHouse.FindHouseAt(controller)`: only
//! the distance test is made); `AttachToTournament_Sandbox`.

use muo_io::{GenericReader, GenericWriter};
use muo_primitives::{DateTime, Point2D, Point3D, Rectangle2D, TimeSpan};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, ops};
use muo_server::map::MapId;
use muo_server::region::RegionId;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::timer;
use muo_server::type_registry::TypeRegistry;

use super::{flow, ladder, safezone};

pub const CONTROLLER_NAME: &str = "Server.Engines.ConPVP.ArenaController";
pub const INTERNAL_WALL_NAME: &str = "Server.Engines.ConPVP.DuelContext+InternalWall";
pub const EXIT_TELEPORTER_NAME: &str = "Server.Engines.ConPVP.DuelContext+ExitTeleporter";

/// `Arena`: the part of an `ArenaController` the duels read.
#[derive(Clone, Debug)]
pub struct ArenaData {
    pub is_private: bool,
    pub is_guarded: bool,
    pub ladder: Option<ItemId>,
    pub tournament: Option<ItemId>,
    pub announcer: Option<MobileId>,
    pub name: Option<String>,
    pub zone: Rectangle2D,
    pub gate_in: Point3D,
    pub gate_out: Point3D,
    pub teleporter: Option<ItemId>,
    pub players: Vec<MobileId>,
    pub facet: Option<MapId>,
    pub bounds: Rectangle2D,
    pub outside: Point3D,
    pub wall: Point3D,
    pub active: bool,
    /// `Points.Points` (8 of them: four edges, four corners).
    pub points: Vec<Point3D>,
    pub region: Option<RegionId>,
}

impl Default for ArenaData {
    fn default() -> Self {
        ArenaData {
            is_private: false,
            is_guarded: false,
            ladder: None,
            tournament: None,
            announcer: None,
            name: None,
            zone: Rectangle2D::EMPTY,
            gate_in: Point3D::default(),
            gate_out: Point3D::default(),
            teleporter: None,
            players: Vec::new(),
            facet: None,
            bounds: Rectangle2D::EMPTY,
            outside: Point3D::default(),
            wall: Point3D::default(),
            active: false,
            points: vec![Point3D::default(); 8],
            region: None,
        }
    }
}

/// `Arena.Arenas` (active, sorted by name) and `ArenaController.Instances`.
#[derive(Default)]
pub struct Arenas {
    pub arenas: Vec<ItemId>,
    pub controllers: Vec<ItemId>,
}

fn arenas_mut(w: &mut World) -> &mut Arenas {
    w.resources.get_mut::<Arenas>()
}

/// `Arena.Arenas`.
pub fn arenas(w: &World) -> Vec<ItemId> {
    w.resources
        .get::<Arenas>()
        .map(|a| a.arenas.clone())
        .unwrap_or_default()
}

/// The arena of a controller.
pub fn data(w: &World, controller: ItemId) -> Option<&ArenaData> {
    w.item(controller)?.ext.get::<ArenaData>()
}

pub fn data_mut(w: &mut World, controller: ItemId) -> Option<&mut ArenaData> {
    w.item_mut(controller)?.ext.get_mut::<ArenaData>()
}

/// `IsOccupied`.
pub fn is_occupied(w: &World, a: ItemId) -> bool {
    data(w, a).is_some_and(|d| !d.players.is_empty())
}

/// `AcquireLadder()`: `Ladder?.Ladder ?? Ladder.Instance`.
pub fn acquire_ladder(w: &World, a: ItemId) -> Option<ItemId> {
    data(w, a)
        .and_then(|d| d.ladder)
        .or_else(|| ladder::instance(w))
}

/// `Arenas.Sort()`: by name, ordinal, a null name first.
fn sort_arenas(w: &mut World) {
    let mut list = arenas(w);
    muo_primitives::dotnet::net_sort(&mut list, |a, b| {
        let (x, y) = (
            data(w, *a).and_then(|d| d.name.clone()),
            data(w, *b).and_then(|d| d.name.clone()),
        );
        x.cmp(&y)
    });
    arenas_mut(w).arenas = list;
}

/// `Active = value`.
pub fn set_active(w: &mut World, a: ItemId, value: bool) {
    let Some(d) = data_mut(w, a) else { return };
    if d.active == value {
        return;
    }
    d.active = value;
    if value {
        arenas_mut(w).arenas.push(a);
        sort_arenas(w);
    } else {
        arenas_mut(w).arenas.retain(|x| *x != a);
    }
}

/// The `SafeZone` an arena with a zone and a facet has.
fn make_region(w: &mut World, a: ItemId) {
    let Some(d) = data(w, a) else { return };
    if d.zone.start != Point2D::ZERO
        && d.zone.end != Point2D::ZERO
        && let Some(facet) = d.facet
    {
        let (zone, outside, guarded) = (d.zone, d.outside, d.is_guarded);
        let r = safezone::create(w, zone, outside, facet, guarded);
        if let Some(d) = data_mut(w, a) {
            d.region = Some(r);
        }
    }
}

// ---- the properties of an `Arena` with side effects ----------------------------------------------------------------

/// `Name = value`: an active arena re-sorts the list.
pub fn set_name(w: &mut World, a: ItemId, value: Option<String>) {
    let Some(d) = data_mut(w, a) else { return };
    d.name = value;
    if d.active {
        sort_arenas(w);
    }
}

fn drop_region(w: &mut World, a: ItemId) {
    if let Some(r) = data_mut(w, a).and_then(|d| d.region.take()) {
        muo_server::region::unregister(w, r);
    }
}

/// `Facet = value`: the teleporter follows, the safe zone is made again.
pub fn set_facet(w: &mut World, a: ItemId, value: Option<MapId>) {
    let Some(d) = data_mut(w, a) else { return };
    d.facet = value;
    if let Some(t) = d.teleporter {
        ops::set_map(w, t, value);
    }
    drop_region(w, a);
    make_region(w, a);
}

/// `Zone = value`.
pub fn set_zone(w: &mut World, a: ItemId, value: Rectangle2D) {
    let Some(d) = data_mut(w, a) else { return };
    d.zone = value;
    drop_region(w, a);
    make_region(w, a);
}

/// `GateOut = value`: the teleporter moves there.
pub fn set_gate_out(w: &mut World, a: ItemId, value: Point3D) {
    let Some(d) = data_mut(w, a) else { return };
    d.gate_out = value;
    if let Some(t) = d.teleporter {
        ops::set_location(w, t, value);
    }
}

/// `IsGuarded = value`: the safe zone's guards follow.
pub fn set_is_guarded(w: &mut World, a: ItemId, value: bool) {
    let Some(d) = data_mut(w, a) else { return };
    d.is_guarded = value;
    if let Some(r) = d.region
        && let Some(rd) = muo_content::regions::base_region::data_mut(w, r)
    {
        rd.guards_disabled = !value;
    }
}

/// `Tournament = value`: the tournament's `Arenas` are the arenas that name its controller.
pub fn set_tournament(w: &mut World, a: ItemId, value: Option<ItemId>) {
    if let Some(d) = data_mut(w, a) {
        d.tournament = value;
    }
}

// ---- finding an arena -------------------------------------------------------------------------------------------------

/// `Arena.FindArena()`: a random start, the first free arena from there.
pub fn find_arena_any(w: &mut World) -> Option<ItemId> {
    let list = arenas(w);
    if list.is_empty() {
        return None;
    }
    let offset = muo_content::hooks::rnd::below(w, list.len() as i32) as usize;
    for i in 0..list.len() {
        let a = list[(i + offset) % list.len()];
        if !is_occupied(w, a) {
            return Some(a);
        }
    }
    Some(list[offset])
}

/// `Arena.FindArena(players)`.
pub fn find_arena(w: &mut World, players: &[MobileId]) -> Option<ItemId> {
    let Some(prefs) = ladder::preferences_instance(w) else {
        return find_arena_any(w);
    };
    let list = arenas(w);
    if list.is_empty() {
        return None;
    }

    if let Some(&first) = players.first() {
        let controllers = w
            .resources
            .get::<Arenas>()
            .map(|a| a.controllers.clone())
            .unwrap_or_default();
        for controller in controllers {
            let Some(it) = w.item(controller) else {
                continue;
            };
            if it.deleted() || !data(w, controller).is_some_and(|d| d.is_private) {
                continue;
            }
            let loc = ops::get_world_location(w, controller);
            let cmap = w.item(controller).and_then(|i| i.map);
            let near = |w: &World, m: MobileId| {
                w.mobile(m)
                    .is_some_and(|x| x.map == cmap && (x.ty.in_range_point3d)(w, m, loc, 24))
            };
            if !near(w, first) {
                continue;
            }
            let house = muo_multis::house::access::find_house_at_item(w, controller);
            let all_near = players.iter().all(|&p| match house {
                None => near(w, p),
                Some(h) => muo_multis::house::access::find_house_at_mobile(w, p) == Some(h),
            });
            if all_near {
                return Some(controller);
            }
        }
    }

    let free: Vec<ItemId> = list
        .iter()
        .copied()
        .filter(|a| !is_occupied(w, *a))
        .collect();
    if free.is_empty() {
        return Some(list[0]);
    }

    let mut votes = vec![0i32; free.len()];
    let mut tc = 0;
    for (i, a) in free.iter().enumerate() {
        let name = data(w, *a).and_then(|d| d.name.clone());
        for p in players {
            let disliked = ladder::disliked(w, prefs, *p);
            if !disliked.contains(&name) {
                votes[i] += 1;
            }
        }
        tc += votes[i];
    }

    let mut rn = muo_content::hooks::rnd::below(w, tc);
    for (i, a) in free.iter().enumerate() {
        if rn < votes[i] {
            return Some(*a);
        }
        rn -= votes[i];
    }
    let i = muo_content::hooks::rnd::below(w, free.len() as i32) as usize;
    Some(free[i])
}

// ---- moving in and out ------------------------------------------------------------------------------------------------

const EDGE_OFFSETS: [(i32, i32); 10] = [
    (0, 0),
    (0, -1),
    (0, 1),
    (1, 0),
    (1, -1),
    (1, 1),
    (2, 0),
    (2, -1),
    (2, 1),
    (3, 0),
];
const CORNER_OFFSETS: [(i32, i32); 10] = [
    (0, 0),
    (0, 1),
    (1, 0),
    (1, 1),
    (0, 2),
    (2, 0),
    (2, 1),
    (1, 2),
    (0, 3),
    (3, 0),
];
/// `m_Rotate`: west, east, north, south, nw, se, sw, ne.
const ROTATE: [[[i32; 2]; 2]; 8] = [
    [[1, 0], [0, 1]],
    [[-1, 0], [0, -1]],
    [[0, 1], [1, 0]],
    [[0, -1], [-1, 0]],
    [[1, 0], [0, 1]],
    [[-1, 0], [0, -1]],
    [[1, 0], [0, -1]],
    [[-1, 0], [0, 1]],
];

/// `MoveInside(players, index)`.
pub fn move_inside(w: &mut World, a: ItemId, players: &[Option<MobileId>], index: usize) {
    let Some(d) = data(w, a) else { return };
    let index = index % d.points.len();
    let (start, facet, wall) = (d.points[index], d.facet, d.wall);
    let offsets = if index < 4 {
        &EDGE_OFFSETS
    } else {
        &CORNER_OFFSETS
    };
    let m = ROTATE[index];
    let mut offset = 0;
    for mob in players.iter().flatten() {
        let p = if offset < offsets.len() {
            offset += 1;
            offsets[offset - 1]
        } else {
            offsets[offsets.len() - 1]
        };
        let x = p.0 * m[0][0] + p.1 * m[0][1];
        let y = p.0 * m[1][0] + p.1 * m[1][1];
        if let (Some(ty), Some(map)) = (w.mobile(*mob).map(|x| x.ty), facet) {
            (ty.move_to_world)(
                w,
                *mob,
                Point3D::new(start.x + x, start.y + y, start.z),
                map,
            );
        }
        let dir =
            muo_server::mobile::part2::effects::get_direction_to(w, *mob, wall.x, wall.y, false);
        muo_server::mobile::part2::effects::set_direction(w, *mob, dir);
        if let Some(d) = data_mut(w, a) {
            d.players.push(*mob);
        }
    }
}

/// `Evict()`.
pub fn evict(w: &mut World, a: ItemId) {
    let Some(d) = data(w, a) else { return };
    let (loc, facet) = match d.facet {
        None => (Point3D::new(2715, 2165, 0), Some(MapId::FELUCCA)),
        Some(f) => (d.outside, Some(f)),
    };
    let (arena_facet, bounds, players) = (d.facet, d.bounds, d.players.clone());
    let has_bounds = bounds.start != Point2D::ZERO && bounds.end != Point2D::ZERO;

    for mob in &players {
        let Some(m) = w.mobile(*mob) else { continue };
        let (map, mloc, lmap, lloc) = (m.map, m.location, m.logout_map, m.logout_location);
        if map.is_none() {
            if (arena_facet.is_none() || lmap == arena_facet)
                && (!has_bounds || bounds.contains_point(&lloc))
                && let Some(m) = w.mobile_mut(*mob)
            {
                m.logout_location = loc;
            }
        } else if (arena_facet.is_none() || map == arena_facet)
            && (!has_bounds || bounds.contains_point(&mloc))
            && let (Some(ty), Some(f)) = (w.mobile(*mob).map(|x| x.ty), facet)
        {
            (ty.move_to_world)(w, *mob, loc, f);
        }
        muo_content::mx::set_combatant(w, *mob, None);
        muo_server::mobile::part1::props_a::set_frozen(w, *mob, false);
        flow::debuff(w, *mob);
        flow::cancel_spell(w, *mob);
    }

    if has_bounds && let Some(f) = facet {
        // Pets of the players in the arena go out with them.
        let pets: Vec<MobileId> =
            muo_server::map::enumerate::mobiles_in_bounds(w, Some(f), bounds, false)
                .collect::<Vec<_>>()
                .into_iter()
                .filter(|p| {
                    muo_content::creature::control::control_master(w, *p)
                        .is_some_and(|c| players.contains(&c))
                })
                .collect();
        for pet in pets {
            muo_content::mx::set_combatant(w, pet, None);
            muo_server::mobile::part1::props_a::set_frozen(w, pet, false);
            if let Some(ty) = w.mobile(pet).map(|x| x.ty) {
                (ty.move_to_world)(w, pet, loc, f);
            }
        }
    }

    if let Some(d) = data_mut(w, a) {
        d.players.clear();
    }
}

/// The `ExitTeleporter` an arena has, made at its gate-out (or outside) when it has none.
pub fn ensure_teleporter(w: &mut World, a: ItemId) -> Option<ItemId> {
    let d = data(w, a)?;
    if let Some(t) = d.teleporter.filter(|t| {
        w.item(*t)
            .is_some_and(|i| i.ty.full_name == EXIT_TELEPORTER_NAME)
    }) {
        return Some(t);
    }
    let (at, facet) = (
        if d.gate_out == Point3D::default() {
            d.outside
        } else {
            d.gate_out
        },
        d.facet,
    );
    let tp = muo_content::factory::make_item(w, EXIT_TELEPORTER_NAME)?;
    ops::move_to_world_map(w, tp, at, facet);
    if let Some(d) = data_mut(w, a) {
        d.teleporter = Some(tp);
    }
    Some(tp)
}

// ---- ExitTeleporter -----------------------------------------------------------------------------------------------------

/// `ReturnEntry`.
#[derive(Clone, Debug)]
pub struct ReturnEntry {
    pub mobile: MobileId,
    pub location: Point3D,
    pub facet: Option<MapId>,
    pub expire: DateTime,
}

const RETURN_EXPIRE: TimeSpan = TimeSpan::from_millis(30 * 60_000);

#[derive(Default)]
struct ExitEntries(Vec<ReturnEntry>);

impl ReturnEntry {
    /// `ReturnEntry(mob)` / `Update()`: where the mobile is now (the logout spot when logged out).
    fn snapshot(w: &World, mob: MobileId) -> ReturnEntry {
        let (facet, location) = match w.mobile(mob) {
            Some(m) if m.map.is_none() => (m.logout_map, m.logout_location),
            Some(m) => (m.map, m.location),
            None => (None, Point3D::default()),
        };
        ReturnEntry {
            mobile: mob,
            location,
            facet,
            expire: w.now + RETURN_EXPIRE,
        }
    }
}

/// `Register(mob)`.
pub fn teleporter_register(w: &mut World, tp: ItemId, mob: MobileId) {
    let snap = ReturnEntry::snapshot(w, mob);
    let now = w.now;
    let Some(e) = w.item_mut(tp).and_then(|i| i.ext.get_mut::<ExitEntries>()) else {
        return;
    };
    // `Find(mob)` drops the expired entries it passes before the match.
    let mut i = 0;
    while i < e.0.len() {
        if e.0[i].mobile == mob {
            e.0[i] = snap;
            return;
        }
        if now >= e.0[i].expire {
            e.0.remove(i);
        } else {
            i += 1;
        }
    }
    e.0.push(snap);
}

/// `ExitTeleporter.OnMoveOver(m)`.
fn teleporter_on_move_over(w: &mut World, id: ItemId, m: MobileId) -> bool {
    if !(ITEM_BASE.on_move_over)(w, id, m) {
        return false;
    }
    let now = w.now;
    let entry = {
        let Some(e) = w.item_mut(id).and_then(|i| i.ext.get_mut::<ExitEntries>()) else {
            return true;
        };
        // `Find(mob)` drops the expired entries it passes.
        let mut found = None;
        let mut i = 0;
        while i < e.0.len() {
            if e.0[i].mobile == m {
                found = Some(e.0[i].clone());
                break;
            }
            if now >= e.0[i].expire {
                e.0.remove(i);
            } else {
                i += 1;
            }
        }
        found
    };
    match entry {
        Some(entry) => {
            // `entry.Return()`
            if let Some(facet) = entry.facet {
                if w.mobile(m).is_some_and(|x| x.map.is_none()) {
                    if let Some(x) = w.mobile_mut(m) {
                        x.logout_location = entry.location;
                        x.logout_map = Some(facet);
                    }
                } else {
                    // `Mobile.Location = Location; Mobile.Map = Facet;` (two moves, not `MoveToWorld`)
                    muo_server::mobile::part1::lifecycle::set_location(w, m, entry.location);
                    muo_server::mobile::part1::lifecycle::set_map(w, m, Some(facet));
                }
            }
            let (loc, map) = (
                ops::get_world_location(w, id),
                w.item(id).and_then(|i| i.map),
            );
            muo_server::effects::play_sound(w, loc, map, 0x1FE);
            let (mloc, mmap) = w
                .mobile(m)
                .map_or((Point3D::default(), None), |x| (x.location, x.map));
            muo_server::effects::play_sound(w, mloc, mmap, 0x1FE);
            if let Some(e) = w.item_mut(id).and_then(|i| i.ext.get_mut::<ExitEntries>()) {
                e.0.retain(|x| x.mobile != m);
            }
            false
        }
        None => {
            muo_server::mobile::part2::messages::send_localized_message(w, m, 1049383, "", 0x3B2); // The teleporter doesn't seem to work for you.
            true
        }
    }
}

/// `DuelContext+ExitTeleporter`.
pub static EXIT_TELEPORTER: ItemType = ItemType {
    full_name: EXIT_TELEPORTER_NAME,
    parent: Some(&ITEM_BASE),
    construct: None,
    make: Some(|w| {
        let id = ops::new_item_with_id(w, &EXIT_TELEPORTER, 0x1822);
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(ExitEntries::default());
        }
        let ty = &EXIT_TELEPORTER;
        (ty.set_hue)(w, id, 0x482);
        ops::set_movable(w, id, false);
        id
    }),
    default_name: |_, _| Some("return teleporter".to_owned()),
    on_move_over: teleporter_on_move_over,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
        let entries = w.item(id).and_then(|i| i.ext.get::<ExitEntries>());
        out.write_encoded_int(entries.map_or(0, |e| e.0.len() as i32));
        for e in entries.iter().flat_map(|e| e.0.iter()) {
            out.write_mobile(w, Some(e.mobile));
            out.write_point3d(e.location);
            out.write_map(e.facet);
        }
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let version = r.read_i32()?;
        let mut list = Vec::new();
        if version == 0 {
            let count = r.read_encoded_int()?;
            for _ in 0..count {
                let mob = r.read_mobile(w)?;
                let location = r.read_point3d()?;
                let facet = r.read_map()?;
                if let Some(mobile) = mob {
                    list.push(ReturnEntry {
                        mobile,
                        location,
                        facet,
                        expire: w.now + RETURN_EXPIRE,
                    });
                }
            }
        }
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(ExitEntries(list));
        }
        Ok(())
    },
    ..ITEM_BASE
};

/// `DuelContext+InternalWall`: `SkipSerialization`.
pub static INTERNAL_WALL: ItemType = ItemType {
    full_name: INTERNAL_WALL_NAME,
    parent: Some(&ITEM_BASE),
    construct: None,
    make: Some(|w| {
        let id = ops::new_item_with_id(w, &INTERNAL_WALL, 0x80);
        ops::set_movable(w, id, false);
        id
    }),
    skip_serialization: |_, _| true,
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

// ---- ArenaController ------------------------------------------------------------------------------------------------------

fn write_arena(w: &World, d: &ArenaData, out: &mut dyn GenericWriter) {
    out.write_encoded_int(7);
    out.write_bool(d.is_guarded);
    out.write_item(w, d.ladder);
    out.write_item(w, d.tournament);
    out.write_mobile(w, d.announcer);
    out.write_opt_string(d.name.as_deref());
    out.write_rect2d(d.zone);
    out.write_point3d(d.gate_in);
    out.write_point3d(d.gate_out);
    out.write_item(w, d.teleporter);
    out.write_mobile_list(w, &d.players);
    out.write_map(d.facet);
    out.write_rect2d(d.bounds);
    out.write_point3d(d.outside);
    out.write_point3d(d.wall);
    out.write_bool(d.active);
    out.write_encoded_int(d.points.len() as i32);
    for p in &d.points {
        out.write_point3d(*p);
    }
}

fn read_arena(w: &World, r: &mut dyn GenericReader) -> muo_io::Result<ArenaData> {
    let version = r.read_encoded_int()?;
    let mut d = ArenaData::default();
    if !(0..=7).contains(&version) {
        return Ok(d);
    }
    if version >= 7 {
        d.is_guarded = r.read_bool()?;
    }
    if version >= 6 {
        d.ladder = r.read_item(w)?;
    }
    if version >= 5 {
        d.tournament = r.read_item(w)?;
        d.announcer = r.read_mobile(w)?;
    }
    if version >= 4 {
        d.name = r.read_string()?;
    }
    if version >= 3 {
        d.zone = r.read_rect2d()?;
    }
    if version >= 2 {
        d.gate_in = r.read_point3d()?;
        d.gate_out = r.read_point3d()?;
        d.teleporter = r.read_item(w)?;
    }
    if version >= 1 {
        d.players = r.read_mobile_list(w)?;
    }
    d.facet = r.read_map()?;
    d.bounds = r.read_rect2d()?;
    d.outside = r.read_point3d()?;
    d.wall = r.read_point3d()?;
    if version == 0 {
        r.read_bool()?;
        d.players = Vec::new();
    }
    d.active = r.read_bool()?;
    let n = r.read_encoded_int()?;
    d.points = (0..n.max(0))
        .map(|_| r.read_point3d())
        .collect::<muo_io::Result<Vec<_>>>()?;
    if d.points.is_empty() {
        d.points = vec![Point3D::default(); 8];
    }
    Ok(d)
}

/// `ArenaController`.
pub static ARENA_CONTROLLER: ItemType = ItemType {
    full_name: CONTROLLER_NAME,
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let id = ops::new_item_with_id(w, &ARENA_CONTROLLER, 0x1B7A);
        ops::set_visible(w, id, false);
        ops::set_movable(w, id, false);
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(ArenaData::default());
        }
        arenas_mut(w).controllers.push(id);
        id
    }),
    default_name: |_, _| Some("arena controller".to_owned()),
    on_double_click: |w, id, from| {
        // `new PropertiesGump(from, Arena)`: the arena's properties are the controller's (see `props`)
        if muo_content::mx::access_level(w, from) >= muo_server::mobile::AccessLevel::GameMaster {
            super::games::open_properties(w, from, id);
        }
    },
    on_delete: |w, id| {
        (ITEM_BASE.on_delete)(w, id);
        arenas_mut(w).controllers.retain(|x| *x != id);
        // `Arena.Delete()`
        set_active(w, id, false);
        if let Some(r) = data_mut(w, id).and_then(|d| d.region.take()) {
            muo_server::region::unregister(w, r);
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(1);
        let d = data(w, id).cloned().unwrap_or_default();
        out.write_bool(d.is_private);
        write_arena(w, &d, out);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let version = r.read_i32()?;
        let mut d = ArenaData::default();
        if version == 0 || version == 1 {
            if version >= 1 {
                d.is_private = r.read_bool()?;
            }
            d = ArenaData {
                is_private: d.is_private,
                ..read_arena(w, r)?
            };
        }
        let active = d.active;
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(d);
        }
        if active {
            // `Arenas.Add(this); Arenas.Sort()`: the flag is set by `set_active`.
            if let Some(d) = data_mut(w, id) {
                d.active = false;
            }
            set_active(w, id, true);
        }
        make_region(w, id);
        if is_occupied(w, id) {
            timer::start(
                w,
                TimeSpan::from_millis(2000),
                TimeSpan::ZERO,
                1,
                move |w| evict(w, id),
            );
        }
        // `AttachToTournament_Sandbox` adds the arena to the tournament's `Arenas`: here `Tournament.Arenas` is every
        // controller whose `Tournament` is the controller (`tournament::arenas_of`), so a loaded arena is attached.
        arenas_mut(w).controllers.push(id);
        Ok(())
    },
    ..ITEM_BASE
};

/// Registers the item classes.
pub fn register(types: &mut TypeRegistry) {
    types.register_item(&ARENA_CONTROLLER, &[]);
    types.register_item(&EXIT_TELEPORTER, &[]);
    types.register_item(&INTERNAL_WALL, &[]);
}

/// `Configure`-time setup: the region class `GetRegion<SafeZone>()` asks for.
pub fn configure(w: &mut World) {
    safezone::configure(w);
}
