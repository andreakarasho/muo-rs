//! `MovementImpl`: the tile walk check behind `Mobile.Move`.

use muo_data::tile_matrix::StaticTile;
use muo_data::{ItemData, TileFlag};
use muo_primitives::dotnet::IgnoreCase;
use muo_primitives::{Direction, Point3D};
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::multi::BASE_MULTI;
use muo_server::map::{MapId, enumerate, tiles};
use muo_server::mobile::AccessLevel;
use muo_server::movement::{self, MovementImpl};
use muo_server::world::World;

use crate::ai::movement::MovementFlags;
use crate::creature::data::read;

const PERSON_HEIGHT: i32 = 16;
const STEP_HEIGHT: i32 = 2;

/// The statics of `MovementImpl` besides `IgnoreMovableImpassables` (that one is `ai::movement::MovementFlags`).
#[derive(Default)]
pub struct MovementStatics {
    pub always_ignore_doors: bool,
    pub goal: Point3D,
    /// While a path search runs: the mobiles of the search area by tile, read once. Nothing moves during a search,
    /// and looking each neighbour up here instead of walking the sector's whole mobile list per check is what keeps
    /// searching through a crowd (hundreds of mobiles per sector) affordable.
    pub mobs_at: Option<ByTile<MobileId>>,
    /// The same for ground items, only those whose tile data can block or carry (impassable, surface, wet): a melee
    /// leaves hundreds of blood splats around, which no movement check looks at but every sector walk pays for.
    pub items_at: Option<ByTile<ItemId>>,
}

pub type ByTile<T> = std::collections::HashMap<(i32, i32), Vec<T>, muo_server::world::EntityHasher>;

/// Whether a ground item can take part in a movement check (the search snapshot keeps only these).
pub fn may_block(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| {
        i.item_id > 0xFFFF
            || idata(w, (i.item_id & 0xFFFF) as usize)
                .has(TileFlag::IMPASSABLE | TileFlag::SURFACE | TileFlag::WET)
    })
}

/// The ground items on tile `(x, y)` in sector order: from the search snapshot when one is set (it holds only
/// [`may_block`] items), else from the sector.
pub fn items_at(w: &World, map: MapId, x: i32, y: i32) -> Vec<ItemId> {
    match w
        .resources
        .get::<MovementStatics>()
        .and_then(|s| s.items_at.as_ref())
    {
        Some(snap) => snap.get(&(x, y)).cloned().unwrap_or_default(),
        None => enumerate::items_at(w, Some(map), x, y).collect(),
    }
}

/// The mobiles on tile `(x, y)` in sector order: from the search snapshot when one is set, else from the sector.
pub fn mobiles_at(w: &World, map: MapId, x: i32, y: i32) -> Vec<MobileId> {
    match w
        .resources
        .get::<MovementStatics>()
        .and_then(|s| s.mobs_at.as_ref())
    {
        Some(snap) => snap.get(&(x, y)).cloned().unwrap_or_default(),
        None => enumerate::mobiles_at(w, Some(map), x, y).collect(),
    }
}

pub struct Impl;
pub static IMPL: Impl = Impl;

fn idata(w: &World, id: usize) -> &ItemData {
    w.maps.tile_data.item(id)
}

fn dead_bonded(w: &World, m: MobileId) -> bool {
    w.mobile(m)
        .is_some_and(|mm| (mm.ty.is_dead_bonded_pet)(w, m))
}

fn alive(w: &World, m: MobileId) -> bool {
    w.mobile(m).is_some_and(|mm| (mm.ty.alive)(w, m))
}

/// `CanMoveOver(m, t)`.
pub fn can_move_over(w: &World, m: MobileId, t: MobileId) -> bool {
    let Some(tm) = w.mobile(t) else { return true };
    !alive(w, t)
        || !alive(w, m)
        || dead_bonded(w, t)
        || dead_bonded(w, m)
        || (tm.hidden && tm.access_level > AccessLevel::Player)
}

/// The door graphics `MovementImpl` lets a door-opener, a ghost and a spectral body through.
pub fn is_door_id(data: &ItemData, item_id: usize) -> bool {
    data.door() || matches!(item_id, 0x692 | 0x846 | 0x873) || (0x6F5..=0x6F6).contains(&item_id)
}

fn is_ok(
    w: &World,
    ignore_doors: bool,
    ignore_spell_fields: bool,
    our_z: i32,
    our_top: i32,
    tiles_here: &[StaticTile],
    items: &[ItemId],
) -> bool {
    for t in tiles_here {
        let d = idata(w, usize::from(t.id));
        if d.impassable_surface() {
            let check_z = i32::from(t.z);
            if check_z + d.calc_height() > our_z && our_top > check_z {
                return false;
            }
        }
    }
    for &id in items {
        let Some(item) = w.item(id) else { continue };
        let item_id = (item.item_id & 0xFFFF) as usize;
        let d = idata(w, item_id);
        if d.impassable_surface() {
            if ignore_doors && is_door_id(d, item_id) {
                continue;
            }
            if ignore_spell_fields && matches!(item_id, 0x82 | 0x3946 | 0x3956) {
                continue;
            }
            let check_z = item.location.z;
            if check_z + d.calc_height() > our_z && our_top > check_z {
                return false;
            }
        }
    }
    true
}

/// What `Check` reads from the mover, copied out once.
struct Mover {
    id: MobileId,
    z: i32,
    cant_walk: bool,
    can_swim: bool,
    flying: bool,
    body: i32,
    player_mobile: bool,
    alive: bool,
    dead_bonded: bool,
}

impl Mover {
    fn new(w: &World, id: MobileId) -> Option<Mover> {
        let m = w.mobile(id)?;
        Some(Mover {
            id,
            z: m.location.z,
            cant_walk: m.cant_walk,
            can_swim: m.can_swim,
            flying: m.flying,
            body: muo_server::mobile::part1::props_b::body(w, id).0,
            player_mobile: m.ty.is(&crate::player_mobile::PLAYER_MOBILE),
            alive: alive(w, id),
            dead_bonded: dead_bonded(w, id),
        })
    }
}

struct Cand {
    z: i32,
    data_id: usize,
    movable: bool,
}

#[allow(clippy::too_many_arguments)]
fn check(
    w: &mut World,
    m: &Mover,
    map: MapId,
    items: &[ItemId],
    mobiles: &[MobileId],
    x: i32,
    y: i32,
    start_top: i32,
    start_z: i32,
) -> (bool, i32) {
    let mut new_z = 0;
    let (cant_walk, can_swim) = (m.cant_walk, m.can_swim);
    let land_tile = w.maps.land_tile(map, x, y);
    let flags = w.maps.tile_data.land(land_tile.id as u16 as usize).flags;
    let impassable = flags.contains(TileFlag::IMPASSABLE);

    // Impassable + swim on water is ok, otherwise block if cannot walk or impassable
    let land_blocks =
        (cant_walk || impassable) && !(impassable && can_swim && flags.contains(TileFlag::WET));
    let consider_land = !land_tile.ignored();
    let (land_z, land_center, _) = w.maps.get_average_z_full(map, x, y);

    let mut move_is_ok = false;
    let step_top = start_top + STEP_HEIGHT;
    let check_top = start_z + PERSON_HEIGHT;

    let always_ignore = w
        .resources
        .get::<MovementStatics>()
        .is_some_and(|s| s.always_ignore_doors);
    let ignore_doors = always_ignore || !m.alive || m.body == 0x3DB || m.dead_bonded;
    let ignore_spell_fields = m.player_mobile && map != MapId::FELUCCA;

    let here = tiles::static_and_multi_tiles(w, map, x, y);

    // Statics first, then the ground items: the same body runs over both.
    let mut cands: Vec<Cand> = here
        .iter()
        .map(|t| Cand {
            z: i32::from(t.z),
            data_id: usize::from(t.id),
            movable: false,
        })
        .collect();
    let static_count = cands.len();
    for &id in items {
        if let Some(i) = w.item(id) {
            cands.push(Cand {
                z: i.location.z,
                data_id: (i.item_id & 0xFFFF) as usize,
                movable: i.movable(),
            });
        }
    }

    for (n, c) in cands.iter().enumerate() {
        let d = idata(w, c.data_id);
        if m.flying && d.name.eq_ic("hover over") {
            return (true, c.z);
        }
        if n < static_count {
            // Stygian Dragon
            if m.body == 826 && map == MapId::TER_MUR {
                let in_a = (307..=354).contains(&x) && (126..=192).contains(&y);
                let in_b = (42..=89).contains(&x)
                    && ((333..=399).contains(&y)
                        || (531..=597).contains(&y)
                        || (739..=805).contains(&y));
                if in_a || in_b {
                    if c.z > new_z {
                        new_z = c.z;
                    }
                    move_is_ok = true;
                }
            }
        } else if c.movable {
            continue; // only items that don't move are surfaces
        }
        let not_water = !d.wet();
        // To move: a passable surface and the mover can walk, or water and it can swim.
        if ((!d.surface() || d.impassable()) && (!can_swim || not_water))
            || (cant_walk && not_water)
        {
            continue;
        }
        let item_z = c.z;
        let mut item_top = item_z;
        let our_z = item_z + d.calc_height();
        let mut test_top = check_top;
        if move_is_ok {
            let cmp = (our_z - m.z).abs() - (new_z - m.z).abs();
            if cmp > 0 || (cmp == 0 && our_z > new_z) {
                continue;
            }
        }
        if our_z + PERSON_HEIGHT > test_top {
            test_top = our_z + PERSON_HEIGHT;
        }
        if !d.bridge() {
            item_top += i32::from(d.height);
        }
        if step_top < item_top {
            continue;
        }
        let land_check = item_z + i32::from(d.height).min(STEP_HEIGHT);
        if consider_land && land_check < land_center && land_center > our_z && test_top > land_z {
            continue;
        }
        if is_ok(
            w,
            ignore_doors,
            ignore_spell_fields,
            our_z,
            test_top,
            &here,
            items,
        ) {
            new_z = our_z;
            move_is_ok = true;
        }
    }

    if !consider_land || land_blocks || step_top < land_z {
        return (move_is_ok, new_z);
    }
    let mut test_top = check_top;
    if land_center + PERSON_HEIGHT > test_top {
        test_top = land_center + PERSON_HEIGHT;
    }
    let mut should_check = true;
    if move_is_ok {
        let cmp = (land_center - m.z).abs() - (new_z - m.z).abs();
        if cmp > 0 || (cmp == 0 && land_center > new_z) {
            should_check = false;
        }
    }
    if should_check
        && is_ok(
            w,
            ignore_doors,
            ignore_spell_fields,
            land_center,
            test_top,
            &here,
            items,
        )
    {
        new_z = land_center;
        move_is_ok = true;
    }
    if move_is_ok {
        for &mob in mobiles {
            if mob == m.id {
                continue;
            }
            let Some(mz) = w.mobile(mob).map(|o| o.location.z) else {
                continue;
            };
            if mz + 15 > new_z && new_z + 15 > mz && !can_move_over(w, m.id, mob) {
                move_is_ok = false;
                break;
            }
        }
    }
    (move_is_ok, new_z)
}

/// `GetStartZ`: `(zLow, zTop)`.
fn get_start_z(
    w: &mut World,
    m: &Mover,
    map: MapId,
    loc: Point3D,
    item_list: &[ItemId],
) -> (i32, i32) {
    let land_tile = w.maps.land_tile(map, loc.x, loc.y);
    let flags = w.maps.tile_data.land(land_tile.id as u16 as usize).flags;
    let impassable = flags.contains(TileFlag::IMPASSABLE);
    let land_blocks =
        (m.cant_walk || impassable) && !(impassable && m.can_swim && flags.contains(TileFlag::WET));
    let (land_z, land_center, land_top) = w.maps.get_average_z_full(map, loc.x, loc.y);
    let consider_land = !land_tile.ignored();

    let (mut z_low, mut z_top, mut z_center) = (0, 0, 0);
    let mut is_set = false;
    if consider_land && !land_blocks && loc.z >= land_center {
        z_low = land_z;
        z_center = land_center;
        z_top = land_top;
        is_set = true;
    }

    let mut cands: Vec<(i32, usize)> = tiles::static_and_multi_tiles(w, map, loc.x, loc.y)
        .iter()
        .map(|t| (i32::from(t.z), usize::from(t.id)))
        .collect();
    for &id in item_list {
        if let Some(i) = w.item(id) {
            cands.push((i.location.z, (i.item_id & 0xFFFF) as usize));
        }
    }
    for (z, data_id) in cands {
        let d = idata(w, data_id);
        let calc_top = z + d.calc_height();
        if (is_set && calc_top < z_center)
            || loc.z < calc_top
            || (!d.surface() && !(m.can_swim && d.wet()))
        {
            continue;
        }
        if m.cant_walk && !d.wet() {
            continue;
        }
        z_low = z;
        z_center = calc_top;
        let top = z + i32::from(d.height);
        if !is_set || top > z_top {
            z_top = top;
        }
        is_set = true;
    }
    if !is_set {
        z_low = loc.z;
        z_top = loc.z;
    } else if loc.z > z_top {
        z_top = loc.z;
    }
    (z_low, z_top)
}

impl MovementImpl for Impl {
    fn check_movement(&self, w: &mut World, m: MobileId, d: Direction) -> (bool, i32) {
        let Some((map, loc)) = w.mobile(m).map(|mm| (mm.map, mm.location)) else {
            return (false, 0);
        };
        match map {
            Some(map) => self.check_movement_at(w, m, map, loc, d),
            None => (false, 0),
        }
    }

    fn check_movement_at(
        &self,
        w: &mut World,
        m: MobileId,
        map: MapId,
        loc: Point3D,
        d: Direction,
    ) -> (bool, i32) {
        if map == MapId::INTERNAL {
            return (false, 0);
        }
        let Some(mover) = Mover::new(w, m) else {
            return (false, 0);
        };
        let (x_start, y_start) = (loc.x, loc.y);
        let dir = |n: i32| Direction::from_bits_retain((n & 0x7) as u8);
        let bits = i32::from((d & Direction::MASK).bits());
        let check_diagonals = bits & 1 == 1;
        let (mut xf, mut yf) = (x_start, y_start);
        let (mut xl, mut yl) = (x_start, y_start);
        let (mut xr, mut yr) = (x_start, y_start);
        movement::offset_xy(d, &mut xf, &mut yf, 1);
        movement::offset_xy(dir(bits - 1), &mut xl, &mut yl, 1);
        movement::offset_xy(dir(bits + 1), &mut xr, &mut yr, 1);

        let (width, height) = w.maps.get(map).map_or((0, 0), |mm| (mm.width, mm.height));
        if xf < 0 || yf < 0 || xf >= width || yf >= height {
            return (false, 0);
        }

        let ignore_movable_impassables = w
            .resources
            .get::<MovementFlags>()
            .is_some_and(|f| f.ignore_movable_impassables);
        let mut req_flags = TileFlag::IMPASSABLE | TileFlag::SURFACE;
        if mover.can_swim {
            req_flags |= TileFlag::WET;
        }
        let goal = w
            .resources
            .get::<MovementStatics>()
            .map_or(Point3D::ZERO, |s| s.goal);
        // `(m as BaseCreature)?.Controlled == false`: a mobile that isn't a creature reads false.
        let check_mobs = crate::creature::is_creature(w, m)
            && read(w, m, false, |c| !c.controlled)
            && (xf != goal.x || yf != goal.y);

        let (mut mobs_f, mut mobs_l, mut mobs_r) = (Vec::new(), Vec::new(), Vec::new());
        let searching = w
            .resources
            .get::<MovementStatics>()
            .is_some_and(|s| s.mobs_at.is_some());
        if check_mobs && searching {
            mobs_f = mobiles_at(w, map, xf, yf);
            if check_diagonals {
                mobs_l = mobiles_at(w, map, xl, yl);
                mobs_r = mobiles_at(w, map, xr, yr);
            }
        } else if check_mobs {
            let near: Vec<MobileId> =
                enumerate::mobiles_in_range(w, Some(map), loc.x, loc.y, 1).collect();
            for mob in near {
                let Some(p) = w.mobile(mob).map(|o| o.location) else {
                    continue;
                };
                if p.x == xf && p.y == yf {
                    mobs_f.push(mob);
                } else if check_diagonals && p.x == xl && p.y == yl {
                    mobs_l.push(mob);
                } else if check_diagonals && p.x == xr && p.y == yr {
                    mobs_r.push(mob);
                }
            }
        }

        let (mut i_start, mut i_f, mut i_l, mut i_r) =
            (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        let near: Vec<ItemId> = if searching {
            [(x_start, y_start), (xf, yf), (xl, yl), (xr, yr)]
                .into_iter()
                .flat_map(|(x, y)| items_at(w, map, x, y))
                .collect()
        } else {
            enumerate::items_in_range(w, Some(map), loc.x, loc.y, 1).collect()
        };
        for id in near {
            let Some(item) = w.item(id) else { continue };
            let data = idata(w, (item.item_id & 0xFFFF) as usize);
            if ignore_movable_impassables && item.movable() && data.impassable_surface() {
                continue;
            }
            if !data.has(req_flags) || item.item_id > 0xFFFF || item.parent.is_some() {
                continue;
            }
            if item.ty.is(&BASE_MULTI) {
                continue;
            }
            let p = item.location;
            if p.x == x_start && p.y == y_start {
                i_start.push(id);
            } else if p.x == xf && p.y == yf {
                i_f.push(id);
            } else if check_diagonals && p.x == xl && p.y == yl {
                i_l.push(id);
            } else if check_diagonals && p.x == xr && p.y == yr {
                i_r.push(id);
            }
        }

        let (start_z, start_top) = get_start_z(w, &mover, map, loc, &i_start);
        let (mut ok, new_z) = check(w, &mover, map, &i_f, &mobs_f, xf, yf, start_top, start_z);
        if ok && check_diagonals {
            let strict = w
                .mobile(m)
                .is_some_and(|mm| mm.player && mm.access_level < AccessLevel::GameMaster);
            let left = check(w, &mover, map, &i_l, &mobs_l, xl, yl, start_top, start_z).0;
            if strict {
                if !left || !check(w, &mover, map, &i_r, &mobs_r, xr, yr, start_top, start_z).0 {
                    ok = false;
                }
            } else if !left && !check(w, &mover, map, &i_r, &mobs_r, xr, yr, start_top, start_z).0 {
                ok = false;
            }
        }
        if ok { (true, new_z) } else { (false, start_z) }
    }
}
