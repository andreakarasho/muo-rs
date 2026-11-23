//! Bombing Run: carry the bomb to the other teams' goals. `BRController` (four teams),
//! `BRBomb` (thrown along a path that stops at walls, goals and players), `BRGoal` (the addon), `BRBoard`, and the
//! game's rules.

use std::cell::Cell;
use std::rc::Rc;

use muo_content::items::addon::{
    self, ADDON_COMPONENT, AddonClass, BASE_ADDON, BASE_ADDON_CLASS, Comp,
};
use muo_data::TileFlag;
use muo_io::{GenericReader, GenericWriter};
use muo_primitives::{MessageType, Point3D, Rectangle2D, TimeSpan};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, ops};
use muo_server::map::{MapId, enumerate, tiles};
use muo_server::mobile::part2::messages as mm;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::target::{self, Target, TargetFlags, Targeted};
use muo_server::timer::{self, TimerHandle};
use muo_server::type_registry::TypeRegistry;

use super::games::{
    self, EVENT_CONTROLLER, EventCtrl, GameKind, TeamInfo, alert, board_deserialize,
    board_double_click, board_serialize, color_of, ctrl, display_board, find_owner, new_board,
    new_controller, rt, rt_mut, team, team_mut, team_of,
};
use super::{ctx, flow};

/// What `BRController` adds to the shared controller data.
#[derive(Clone, Debug, Default)]
pub struct BrCtrl {
    pub bomb_home: Point3D,
}

/// What a running Bombing Run keeps: `m_Bomb`.
#[derive(Default)]
pub struct BrRt {
    pub bomb: Option<ItemId>,
}

// ---- BRBoard ----------------------------------------------------------------------------------------------------------

/// `BRBoard`.
pub static BR_BOARD: ItemType = ItemType {
    full_name: "Server.Engines.ConPVP.BRBoard",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| new_board(w, &BR_BOARD)),
    default_name: |w, id| crate::template_keys::item_name(w, id, ITEM_BASE.default_name),
    on_double_click: board_double_click,
    serialize: |w, id, out| board_serialize(w, id, out),
    deserialize: |w, id, r| board_deserialize(w, id, r),
    ..ITEM_BASE
};

// ---- BRBomb -----------------------------------------------------------------------------------------------------------

/// The state of a bomb (it is never saved).
#[derive(Default)]
pub struct BombData {
    /// `m_Game`: the context id.
    pub game: Option<u32>,
    pub helpers: Vec<MobileId>,
    pub path: Vec<Point3D>,
    pub path_idx: usize,
    pub flying: bool,
    pub thrower: Option<MobileId>,
    /// The `EffectTimer` and its count.
    pub timer: TimerHandle,
    pub count: i32,
}

fn bd(w: &World, b: ItemId) -> Option<&BombData> {
    w.item(b)?.ext.get::<BombData>()
}

fn bd_mut(w: &mut World, b: ItemId) -> Option<&mut BombData> {
    w.item_mut(b)?.ext.get_mut::<BombData>()
}

fn controller_of(w: &World, game: u32) -> Option<ItemId> {
    ctx(w, game)?.event_game.map(|g| g.controller)
}

fn bomb_home(w: &World, ec: ItemId) -> Point3D {
    ctrl(w, ec).map_or_else(Point3D::default, |c| c.br.bomb_home)
}

fn world_loc(w: &World, b: ItemId) -> Point3D {
    ops::get_world_location(w, b)
}

fn map_of(w: &World, b: ItemId) -> Option<MapId> {
    w.item(b).and_then(|i| i.map)
}

/// `new BRBomb(game)`.
pub fn new_bomb(w: &mut World, game: u32) -> ItemId {
    let id = ops::new_item_with_id(w, &BR_BOMB, 0x103C); // 0x103C = bread
    ops::set_movable(w, id, false);
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.set_hue)(w, id, 0x35);
    }
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(BombData {
            game: Some(game),
            ..BombData::default()
        });
    }
    // `m_Timer = new EffectTimer(this); m_Timer.Start()`
    let h = timer::start(w, TimeSpan::ZERO, TimeSpan::from_secs(1), 0, move |w| {
        effect_tick(w, id)
    });
    if let Some(d) = bd_mut(w, id) {
        d.timer = h;
    }
    id
}

/// `EffectTimer.OnTick`.
fn effect_tick(w: &mut World, b: ItemId) {
    let Some(d) = bd(w, b) else { return };
    let (game, flying) = (d.game, d.flying);
    let ec = game.and_then(|g| controller_of(w, g));
    let (Some(game), Some(ec)) = (game, ec) else {
        if let Some(d) = bd_mut(w, b) {
            d.count = 0;
        }
        return;
    };
    if w.item(b).is_none_or(|i| i.parent.is_some()) {
        if let Some(d) = bd_mut(w, b) {
            d.count = 0;
        }
        return;
    }
    let map = map_of(w, b);
    if !flying && map != Some(MapId::INTERNAL) {
        let hue = w.item(b).map_or(0, |i| i.hue);
        let loc = world_loc(w, b);
        muo_server::effects::send_location_effect(w, loc, map, 0x377A, 16, 10, hue, 0);
    }
    if w.item(b).map(|i| i.location) != Some(bomb_home(w, ec)) {
        let count = {
            let d = bd_mut(w, b).expect("bomb");
            d.count += 1;
            d.count
        };
        if count >= 30 {
            return_bomb(w, game);
            alert(
                w,
                game,
                "The bomb has been returned to it's starting point.",
            );
            if let Some(d) = bd_mut(w, b) {
                d.count = 0;
                d.helpers.clear();
            }
        }
    } else if let Some(d) = bd_mut(w, b) {
        d.count = 0;
    }
}

/// `BRGame.ReturnBomb()`.
pub fn return_bomb(w: &mut World, game: u32) {
    let Some(bomb) = rt(w, game).and_then(|r| r.br.bomb) else {
        return;
    };
    let Some(ec) = controller_of(w, game) else {
        return;
    };
    ops::set_visible(w, bomb, false);
    let (home, map) = (bomb_home(w, ec), map_of(w, ec));
    ops::move_to_world_map(w, bomb, home, map);
    let secs = muo_primitives::utility::random_min_max(&mut w.rng, 5, 15);
    timer::start(
        w,
        TimeSpan::from_secs(i64::from(secs)),
        TimeSpan::ZERO,
        1,
        move |w| {
            // `UnhideBomb`: the game's bomb shows and takes whoever stands on it
            if let Some(b) = rt(w, game).and_then(|r| r.br.bomb)
                && !ops::is_deleted(w, b)
            {
                ops::set_visible(w, b, true);
                let loc = w.item(b).map_or_else(Point3D::default, |i| i.location);
                bomb_on_location_change(w, b, loc);
            }
        },
    );
}

fn drop_bomb_to(w: &mut World, b: ItemId, mob: MobileId, killer: Option<MobileId>) {
    let place = |w: &World, m: MobileId| {
        w.mobile(m)
            .filter(|x| !x.deleted)
            .map(|x| (x.location, x.map))
    };
    if let Some((loc, map)) = place(w, mob) {
        ops::move_to_world_map(w, b, loc, map);
    } else if let Some((loc, map)) = killer.and_then(|k| place(w, k)) {
        ops::move_to_world_map(w, b, loc, map);
    } else if let Some(g) = bd(w, b).and_then(|d| d.game) {
        return_bomb(w, g);
    }
}

fn bomb_team(w: &World, b: ItemId, m: MobileId) -> Option<usize> {
    team_of(w, bd(w, b)?.game?, m)
}

/// `TakeBomb(m, team, verb)`.
fn take_bomb(w: &mut World, b: ItemId, m: MobileId, tm: usize, verb: &str) -> bool {
    if !muo_content::mx::player(w, m)
        || !muo_content::mx::alive(w, m)
        || w.mobile(m).is_none_or(|x| x.net_state.is_none())
    {
        return false;
    }
    if !muo_server::mobile::part2::items::place_in_backpack(w, m, b) {
        return false;
    }
    muo_server::mobile::base::revealing_action(w, m);
    mm::local_overhead_message(w, m, MessageType::REGULAR, 0x59, false, "You got the bomb!");
    let Some(game) = bd(w, b).and_then(|d| d.game) else {
        return true;
    };
    let ec = controller_of(w, game);
    let tname = ec
        .and_then(|e| team(w, e, tm))
        .map(|t| t.text_name(GameKind::Br))
        .unwrap_or_default();
    let mname = flow::name(w, m);
    alert(w, game, &format!("{verb} ({mname}) {tname} the bomb!"));

    begin_bomb_target(w, b, m);

    if let Some(d) = bd_mut(w, b) {
        d.helpers.retain(|h| *h != m);
    }
    let last = bd(w, b).and_then(|d| d.helpers.first().copied());
    if let Some(last) = last
        && team_of(w, game, last) != Some(tm)
        && let Some(d) = bd_mut(w, b)
    {
        d.helpers.clear();
    }
    if let Some(d) = bd_mut(w, b) {
        d.helpers.push(m);
    }
    true
}

/// `CheckScore(goal, m, points)`.
fn check_score(w: &mut World, b: ItemId, goal: ItemId, m: MobileId, mut points: i32) -> bool {
    let Some(game) = bd(w, b).and_then(|d| d.game) else {
        return false;
    };
    let Some(ec) = controller_of(w, game) else {
        return false;
    };
    let tm = team_of(w, game, m);
    let goal_team = goal_data(w, goal).team;
    let (Some(tm), Some((gec, gt))) = (tm, goal_team) else {
        return false;
    };
    if gec == ec && gt == tm {
        return false;
    }
    let tname = team(w, ec, tm)
        .map(|t| t.text_name(GameKind::Br))
        .unwrap_or_default();
    let mname = flow::name(w, m);
    if points > 3 {
        alert(w, game, &format!("Touchdown {tname} ({mname})!"));
    } else {
        alert(w, game, &format!("Field goal {tname} ({mname})!"));
    }
    let helpers = bd(w, b).map(|d| d.helpers.clone()).unwrap_or_default();
    for mob in helpers.into_iter().rev() {
        if let Some(t) = team_mut(w, ec, tm) {
            if mob == m {
                t.add_captures(mob, points);
            }
            t.add_score(mob, points + 1);
        }
        points /= 2;
    }
    return_bomb(w, game);
    ops::set_visible(w, b, true);
    if let Some(d) = bd_mut(w, b) {
        d.flying = false;
        d.path.clear();
        d.path_idx = 0;
    }
    target::cancel_mobile_target(w, m);
    true
}

fn do_anim(w: &mut World, b: ItemId, start: Point3D, end: Point3D, map: Option<MapId>) {
    let (item_id, hue) = w.item(b).map_or((0, 0), |i| (i.item_id, i.hue));
    muo_server::effects::send_moving_effect_map(
        w, map, item_id, start, end, 15, 0, false, false, hue, 0,
    );
}

fn check_catch(w: &mut World, b: ItemId, m: MobileId, my_loc: Point3D) -> bool {
    if !muo_content::mx::alive(w, m)
        || !muo_content::mx::player(w, m)
        || bd(w, b).is_none_or(|d| d.game.is_none())
    {
        return false;
    }
    if bomb_team(w, b, m).is_none() {
        return false;
    }
    let zdiff = my_loc.z - w.mobile(m).map_or(0, |x| x.location.z);
    zdiff >= 0 && (zdiff < 12 || (zdiff < 16 && muo_primitives::utility::random_bool(&mut w.rng)))
}

fn do_catch(w: &mut World, b: ItemId, m: MobileId) {
    if let Some(d) = bd_mut(w, b) {
        d.flying = false;
    }
    ops::set_visible(w, b, true);
    if !muo_content::mx::alive(w, m) || !muo_content::mx::player(w, m) {
        return;
    }
    let Some(game) = bd(w, b).and_then(|d| d.game) else {
        return;
    };
    let Some(use_team) = team_of(w, game, m) else {
        return;
    };
    let (mloc, mmap) = w
        .mobile(m)
        .map_or((Point3D::default(), None), |x| (x.location, x.map));
    let from = world_loc(w, b);
    let map = map_of(w, b);
    do_anim(w, b, from, mloc, mmap);
    let _ = map;
    let mut verb = "caught";
    if let Some(t) = bd(w, b).and_then(|d| d.thrower)
        && team_of(w, game, t) != Some(use_team)
    {
        verb = "intercepted";
    }
    if !take_bomb(w, b, m, use_team, verb) {
        ops::move_to_world_map(w, b, mloc, mmap);
    }
}

fn hit_object(w: &mut World, b: ItemId, ball_loc: Point3D) {
    let (from, map) = (world_loc(w, b), map_of(w, b));
    do_anim(w, b, from, ball_loc, map);
    ops::move_to_world_map(w, b, ball_loc, map);
    if let Some(d) = bd_mut(w, b) {
        d.path.clear();
        d.path_idx = 0;
    }
    timer::start(w, TimeSpan::from_millis(50), TimeSpan::ZERO, 1, move |w| {
        continue_flight(w, b)
    });
}

/// `BeginFlight(dest)`: a path of points in a lob.
fn begin_flight(w: &mut World, b: ItemId, dest: Point3D) {
    let mut org = world_loc(w, b);
    org.z += 10; // always add 10 at the start cause we're coming from a mobile's eye level

    let mut list: Vec<Point3D> = Vec::new();
    let (xd, yd, zd) = (dest.x - org.x, dest.y - org.y, dest.z - org.z);
    let dist2d = f64::from(xd * xd + yd * yd).sqrt();
    let dist3d = if zd == 0 {
        dist2d
    } else {
        (dist2d * dist2d + f64::from(zd * zd)).sqrt()
    };
    let (rise, run, zslp) = (
        f64::from(yd) / dist3d,
        f64::from(xd) / dist3d,
        f64::from(zd) / dist3d,
    );

    let (mut x, mut y, mut z) = (f64::from(org.x), f64::from(org.y), f64::from(org.z));
    while muo_primitives::utility::number_between(x, dest.x, org.x, 0.5)
        && muo_primitives::utility::number_between(y, dest.y, org.y, 0.5)
        && muo_primitives::utility::number_between(z, dest.z, org.z, 0.5)
    {
        // `Math.Round`: halves go to the even number
        let (ix, iy, iz) = (
            x.round_ties_even() as i32,
            y.round_ties_even() as i32,
            z.round_ties_even() as i32,
        );
        if let Some(p) = list.last() {
            if p.x != ix || p.y != iy || p.z != iz {
                list.push(Point3D::new(ix, iy, iz));
            }
        } else {
            list.push(Point3D::new(ix, iy, iz));
        }
        x += run;
        y += rise;
        z += zslp;
    }
    if list.last().is_some_and(|l| *l != dest) {
        list.push(dest);
    }

    if dist2d > 1.0 {
        let count = list.len() as i32;
        let height = f64::from(count)
            * 2.0
            * (muo_primitives::utility::random_double(&mut w.rng) * 0.40 + 0.10); // 10 - 50%
        let coeff = -height / (f64::from(count * count) / 4.0);
        for (i, p) in list.iter_mut().enumerate() {
            let xp = i as i32 - count / 2;
            p.z += (coeff * f64::from(xp) * f64::from(xp) + height).ceil() as i32;
        }
    }

    if let Some(d) = bd_mut(w, b) {
        d.path = list;
        d.path_idx = 0;
    }
    continue_flight(w, b);
}

fn blocks(w: &World, item_id: i32) -> Option<i32> {
    let data = w
        .maps
        .tile_data
        .items
        .get((item_id as usize) & muo_data::TileData::MAX_ITEM_VALUE)?;
    data.flags
        .intersects(TileFlag::IMPASSABLE | TileFlag::WALL | TileFlag::NO_SHOOT)
        .then(|| data.calc_height())
}

/// `ContinueFlight()`: five points of the path a step.
fn continue_flight(w: &mut World, b: ItemId) {
    if ops::is_deleted(w, b) {
        return;
    }
    let Some(d) = bd(w, b) else { return };
    let (idx, len) = (d.path_idx, d.path.len());
    let map = map_of(w, b);

    let Some(map) = map.filter(|m| idx < len && *m != MapId::INTERNAL) else {
        land(w, b);
        return;
    };

    let mut path_check_end = idx + 5;
    if len < path_check_end {
        path_check_end = len;
    }
    ops::set_visible(w, b, false);
    if idx > 0 {
        // move to the next location
        let p = bd(w, b).expect("bomb").path[idx - 1];
        ops::move_to_world_map(w, b, p, Some(map));
    }
    let path = bd(w, b).expect("bomb").path.clone();
    let mut p_top = world_loc(w, b);
    let mut p_bottom = path[path_check_end - 1];
    muo_primitives::utility::fix_points(&mut p_top, &mut p_bottom);

    for i in idx..path_check_end {
        let mut point = path[i];
        let land_tile = w.maps.land_tile(map, point.x, point.y);
        let (land_z, _, land_top) = w.maps.get_average_z_full(map, point.x, point.y);
        if land_z <= point.z && land_top >= point.z && !land_tile.ignored() {
            hit_object(w, b, point);
            return;
        }
        let mut found_statics = false;
        for t in tiles::static_and_multi_tiles(w, map, point.x, point.y) {
            found_statics = true;
            let data = w
                .maps
                .tile_data
                .item(usize::from(t.id) & muo_data::TileData::MAX_ITEM_VALUE);
            let height = data.calc_height();
            let tz = i32::from(t.z);
            if tz <= point.z
                && tz + height >= point.z
                && data
                    .flags
                    .intersects(TileFlag::IMPASSABLE | TileFlag::WALL | TileFlag::NO_SHOOT)
            {
                point = if i > idx {
                    path[i - 1]
                } else {
                    world_loc(w, b)
                };
                hit_object(w, b, point);
                return;
            }
        }
        if land_tile.id == 0x244 && found_statics {
            // 0x244 = invalid land tile
            let empty = !enumerate::items_at(w, Some(map), point.x, point.y).any(|it| it != b);
            if empty {
                hit_object(w, b, point);
                return;
            }
        }
    }

    let rect = Rectangle2D::new(
        p_top.x,
        p_top.y,
        p_bottom.x - p_top.x + 1,
        p_bottom.y - p_top.y + 1,
    );
    let items: Vec<ItemId> = enumerate::items_in_bounds(w, Some(map), rect, false).collect();
    for it in items {
        let Some(item) = w.item(it) else { continue };
        if it == b || item.item_id >= 0x4000 {
            continue;
        }
        let is_goal = item.ty.is(&BR_GOAL);
        let is_blocker = item.ty.is(&muo_items::misc::basic::BLOCKER);
        let (loc, item_id) = (item.location, item.item_id);
        let height = if is_goal {
            17
        } else if is_blocker {
            20
        } else {
            match blocks(w, item_id) {
                Some(h) => h,
                None => continue,
            }
        };
        for j in idx..path_check_end {
            let point = path[j];
            if loc.x == point.x
                && loc.y == point.y
                && (is_blocker || (loc.z <= point.z && loc.z + height >= point.z))
            {
                let hit = if j > idx {
                    path[j - 1]
                } else {
                    world_loc(w, b)
                };
                if is_goal {
                    let old = world_loc(w, b);
                    let thrower = bd(w, b).and_then(|d| d.thrower);
                    if thrower.is_some_and(|t| check_score(w, b, it, t, 3)) {
                        do_anim(w, b, old, hit, Some(map));
                    } else {
                        hit_object(w, b, hit);
                    }
                } else {
                    hit_object(w, b, hit);
                }
                return;
            }
        }
    }

    let thrower = bd(w, b).and_then(|d| d.thrower);
    let clients: Vec<MobileId> = enumerate::clients_in_bounds(w, Some(map), rect, false)
        .filter_map(|ns| w.net_states.get(&ns).and_then(|n| n.mobile))
        .collect();
    for m in clients {
        if Some(m) == thrower {
            continue;
        }
        let loc = w.mobile(m).map_or_else(Point3D::default, |x| x.location);
        let mut found = false;
        for point in &path[idx..path_check_end] {
            if found {
                break;
            }
            if loc.x == point.x && loc.y == point.y && loc.z <= point.z && loc.z + 16 >= point.z {
                found = check_catch(w, b, m, *point);
            }
        }
        if !found {
            continue;
        }
        do_catch(w, b, m);
        return;
    }

    if let Some(d) = bd_mut(w, b) {
        d.path_idx = path_check_end;
    }
    if path_check_end > 0 && path_check_end - 1 < path.len() {
        let from = world_loc(w, b);
        do_anim(w, b, from, path[path_check_end - 1], Some(map));
    }
    timer::start(w, TimeSpan::from_millis(100), TimeSpan::ZERO, 1, move |w| {
        continue_flight(w, b)
    });
}

/// The end of `ContinueFlight`: the bomb comes down on the highest thing under it.
fn land(w: &mut World, b: ItemId) {
    let (idx, path) = bd(w, b)
        .map(|d| (d.path_idx, d.path.clone()))
        .unwrap_or_default();
    let map = map_of(w, b);
    if idx > 0 && idx - 1 < path.len() {
        ops::move_to_world_map(w, b, path[idx - 1], map);
    } else if let Some(last) = path.last() {
        ops::move_to_world_map(w, b, *last, map);
    }
    let (x, y, z) = w
        .item(b)
        .map_or((0, 0, 0), |i| (i.location.x, i.location.y, i.location.z));
    let mut my_z = map.map_or(0, |m| w.maps.get_average_z(m, x, y));
    if let Some(m) = map {
        for t in tiles::static_and_multi_tiles(w, m, x, y) {
            let height = w
                .maps
                .tile_data
                .item(usize::from(t.id) & muo_data::TileData::MAX_ITEM_VALUE)
                .calc_height();
            let top = i32::from(t.z) + height;
            if top > my_z && top <= z {
                my_z = top;
            }
        }
    }
    let here: Vec<ItemId> = enumerate::items_at(w, map, x, y).collect();
    for it in here {
        let Some(i) = w.item(it) else { continue };
        if i.visible() && it != b {
            let height = w
                .maps
                .tile_data
                .item((i.item_id as usize) & muo_data::TileData::MAX_ITEM_VALUE)
                .calc_height();
            if i.location.z + height > my_z && i.location.z + height <= z {
                my_z = i.location.z + height;
            }
        }
    }
    ops::set_z(w, b, my_z);
    ops::set_visible(w, b, true);
    if let Some(d) = bd_mut(w, b) {
        d.flying = false;
        d.path.clear();
        d.path_idx = 0;
    }
}

/// `OnBombTarget(from, obj)`: true when the target is done with (false resends it).
fn on_bomb_target(w: &mut World, b: ItemId, from: MobileId, obj: Targeted) -> bool {
    if bd(w, b).is_none_or(|d| d.game.is_none()) {
        return true;
    }
    let pack = w.mobile(from).and_then(|m| m.backpack);
    if !ops::is_child_of(w, b, pack.map(EntityId::Item)) {
        return true;
    }
    // don't let them throw it to themselves
    if obj == Targeted::Entity(EntityId::Mobile(from)) {
        return false;
    }
    let pt = match obj {
        Targeted::Entity(EntityId::Mobile(m)) => {
            let mut p = w.mobile(m).map_or_else(Point3D::default, |x| x.location);
            p.z += 10;
            p
        }
        Targeted::Entity(EntityId::Item(it)) => {
            let Some(i) = w.item(it) else { return false };
            let mut p = i.location;
            p.z += w
                .maps
                .tile_data
                .item((i.item_id as usize) & muo_data::TileData::MAX_ITEM_VALUE)
                .calc_height()
                + 1;
            p
        }
        Targeted::Land(l) => l.location,
        Targeted::Static(s) => s.location,
    };
    if let Some(d) = bd_mut(w, b) {
        d.flying = true;
        d.thrower = Some(from);
    }
    ops::set_visible(w, b, false);
    let (loc, map) = (world_loc(w, b), w.mobile(from).and_then(|m| m.map));
    ops::move_to_world_map(w, b, loc, map);
    begin_flight(w, b, pt);
    true
}

type TargetCb = Rc<dyn Fn(&mut World, MobileId, Targeted)>;

/// `m.Target = new BombTarget(bomb, m)`.
fn begin_bomb_target(w: &mut World, b: ItemId, m: MobileId) {
    mm::send_message_hue(w, m, 0x26, "Where do you want to throw it?");
    let mut t = Target::new(w, 10, true, TargetFlags::NONE);
    t.check_los = false;
    let resend = Rc::new(Cell::new(true));
    let r1 = resend.clone();
    let on: TargetCb = Rc::new(move |w, from, obj| {
        r1.set(!on_bomb_target(w, b, from, obj));
    });
    t.handlers.on_target = Some(on.clone());
    t.handlers.on_target_untargetable = Some(on);
    t.handlers.on_target_finish = Some(Rc::new(move |w, _from| {
        // has to be delayed in case some other target canceled us...
        if resend.get() {
            timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
                // Make sure they still have the bomb, then give them the target back
                if !ops::is_deleted(w, b)
                    && !muo_content::mx::deleted(w, m)
                    && muo_content::mx::alive(w, m)
                    && ops::is_child_of(w, b, Some(EntityId::Mobile(m)))
                {
                    begin_bomb_target(w, b, m);
                }
            });
        }
    }));
    let id = target::add_target(w, t);
    target::set_mobile_target(w, m, Some(id));
}

fn bomb_on_location_change(w: &mut World, b: ItemId, old: Point3D) {
    (ITEM_BASE.on_location_change)(w, b, old);
    let Some(d) = bd(w, b) else { return };
    if d.flying || !w.item(b).is_some_and(|i| i.visible() && i.parent.is_none()) || d.game.is_none()
    {
        return;
    }
    let (map, loc) = (
        map_of(w, b),
        w.item(b).map_or_else(Point3D::default, |i| i.location),
    );
    let clients: Vec<MobileId> = enumerate::clients_at(w, map, loc.x, loc.y)
        .filter_map(|ns| w.net_states.get(&ns).and_then(|n| n.mobile))
        .collect();
    for m in clients {
        if !muo_content::mx::player(w, m) || !muo_content::mx::alive(w, m) {
            continue;
        }
        if let Some(t) = bomb_team(w, b, m) {
            take_bomb(w, b, m, t, "got");
        }
    }
}

/// `BRBomb`.
pub static BR_BOMB: ItemType = ItemType {
    full_name: "Server.Engines.ConPVP.BRBomb",
    parent: Some(&ITEM_BASE),
    construct: None,
    default_name: |w, id| crate::template_keys::item_name(w, id, ITEM_BASE.default_name),
    skip_serialization: |_, _| true,
    on_added: |w, id, parent| {
        (ITEM_BASE.on_added)(w, id, parent);
        if let Some(m) = find_owner(w, parent) {
            muo_server::mobile::part1::props_b::set_solid_hue_override(w, m, 0x0499);
        }
    },
    on_removed: |w, id, parent| {
        (ITEM_BASE.on_removed)(w, id, parent);
        if let Some(m) = find_owner(w, parent)
            && let Some(g) = bd(w, id).and_then(|d| d.game)
        {
            let hue = color_of(w, g, m);
            muo_server::mobile::part1::props_b::set_solid_hue_override(w, m, hue);
        }
    },
    on_move_over: |w, id, m| {
        let Some(d) = bd(w, id) else { return true };
        if d.flying
            || !w.item(id).is_some_and(|i| i.visible())
            || d.game.is_none()
            || !muo_content::mx::alive(w, m)
        {
            return true;
        }
        match bomb_team(w, id, m) {
            None => true,
            Some(t) => take_bomb(w, id, m, t, "picked up"),
        }
    },
    on_location_change: bomb_on_location_change,
    on_after_delete: |w, id| {
        (ITEM_BASE.on_after_delete)(w, id);
        if let Some(d) = bd_mut(w, id) {
            let mut t = std::mem::take(&mut d.timer);
            t.cancel(w);
        }
    },
    on_double_click: |w, id, m| {
        let Some(d) = bd(w, id) else { return };
        if d.game.is_none()
            || !w.item(id).is_some_and(|i| i.visible())
            || !muo_content::mx::alive(w, m)
        {
            return;
        }
        let pack = w.mobile(m).and_then(|x| x.backpack);
        if !d.flying && ops::is_child_of(w, id, pack.map(EntityId::Item)) {
            begin_bomb_target(w, id, m);
        } else if w.item(id).is_some_and(|i| i.parent.is_none()) {
            let (loc, z) = w
                .item(id)
                .map_or((Point3D::default(), 0), |i| (i.location, i.location.z));
            if muo_content::mx::in_range_point(w, m, loc, 1)
                && w.mobile(m).is_some_and(|x| x.location.z != z)
                && let Some(t) = bomb_team(w, id, m)
            {
                take_bomb(w, id, m, t, "grabbed");
            }
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(BombData::default());
        }
        Ok(())
    },
    ..ITEM_BASE
};

// ---- BRGoal -----------------------------------------------------------------------------------------------------------

/// `BRGoal`: `North` is saved; `Team` (controller, team index) is not.
#[derive(Clone, Copy, Debug, Default)]
pub struct GoalData {
    pub north: bool,
    pub team: Option<(ItemId, usize)>,
}

fn goal_data(w: &World, g: ItemId) -> GoalData {
    w.item(g)
        .and_then(|i| i.ext.get::<GoalData>())
        .copied()
        .unwrap_or_default()
}

fn set_goal_hue(w: &mut World, g: ItemId, hue: i32) {
    if let Some(ty) = w.item(g).map(|i| i.ty) {
        (ty.set_hue)(w, g, hue);
    }
}

/// `Team = value`.
pub fn set_goal_team(w: &mut World, g: ItemId, tm: Option<(ItemId, usize)>) {
    if let Some(d) = w.item_mut(g).and_then(|i| i.ext.get_mut::<GoalData>()) {
        d.team = tm;
    }
    let color = tm.and_then(|(ec, t)| team(w, ec, t)).map_or(0, |t| t.color);
    set_goal_hue(w, g, if color != 0 { color } else { 0x84C });
}

fn hued(w: &mut World, item_id: i32, hue: i32) -> ItemId {
    let c = addon::new_component(w, &ADDON_COMPONENT, item_id);
    if let Some(ty) = w.item(c).map(|i| i.ty) {
        (ty.set_hue)(w, c, hue);
    }
    c
}

/// `Remake()`.
fn remake(w: &mut World, g: ItemId) {
    for c in addon::components(w, g) {
        if let Some(d) = addon::component_data_mut(w, c) {
            d.addon = None;
        }
        if let Some(ty) = w.item(c).map(|i| i.ty) {
            (ty.delete)(w, c);
        }
    }
    if let Some(d) = addon::addon_data_mut(w, g) {
        d.components.clear();
    }
    let plain = |w: &mut World, g: ItemId, id: i32, x: i32, y: i32, z: i32| {
        let c = addon::new_component(w, &ADDON_COMPONENT, id);
        addon::add_component(w, g, c, x, y, z);
    };
    // stairs
    for (id, x, y) in [
        (0x74D, -1, 1),
        (0x71F, 0, 1),
        (0x74B, 1, 1),
        (0x736, 1, 0),
        (0x74C, 1, -1),
        (0x737, 0, -1),
        (0x74A, -1, -1),
        (0x749, -1, 0),
    ] {
        plain(w, g, id, x, y, -5);
    }
    // Center Sparkle
    plain(w, g, 0x375A, 0, 0, -1);

    let north = goal_data(w, g).north;
    let at = |a: i32| if north { (a, 0) } else { (0, a) };
    // Pillars
    let pillars: [(i32, (i32, i32)); 3] = if north {
        [(0x0CF, (1, 0)), (0x0CC, (-1, 0)), (0x0D1, (0, 0))]
    } else {
        [(0x0CE, (0, 1)), (0x0CC, (0, -1)), (0x0D0, (0, 0))]
    };
    for (id, (x, y)) in pillars {
        plain(w, g, id, x, y, -2);
    }
    // Yellow parts
    for (a, z) in [(1, 7), (0, 16), (-1, 7)] {
        let (x, y) = at(a);
        let c = hued(w, 0x0DF, 0x499);
        addon::add_component(w, g, c, x, y, z);
    }
    // Blue Sparkles
    for (a, z) in [(1, 12), (1, -1), (-1, 12), (-1, -1)] {
        let (x, y) = at(a);
        let c = hued(w, 0x377A, 0x84C);
        addon::add_component(w, g, c, x, y, z);
    }
}

/// `North = value` (`OnNorthChanged`).
pub fn set_goal_north(w: &mut World, g: ItemId, north: bool) {
    let changed = goal_data(w, g).north != north;
    if let Some(d) = w.item_mut(g).and_then(|i| i.ext.get_mut::<GoalData>()) {
        d.north = north;
    }
    if changed {
        remake(w, g);
    }
}

fn new_goal(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &BR_GOAL);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(GoalData::default());
    }
    (ITEM_BASE.set_item_id)(w, id, 0x51D);
    set_goal_hue(w, id, 0x84C);
    ops::set_visible(w, id, true);
    remake(w, id);
    id
}

static GOAL_CLASS: AddonClass = AddonClass {
    share_hue: false,
    ..BASE_ADDON_CLASS
};

/// `BRGoal`.
pub static BR_GOAL: ItemType = ItemType {
    full_name: "Server.Engines.ConPVP.BRGoal",
    parent: Some(&BASE_ADDON),
    construct: Some(new_goal),
    family: Some(&GOAL_CLASS),
    default_name: |w, id| crate::template_keys::item_name(w, id, BASE_ADDON.default_name),
    on_move_over: |w, id, m| {
        if !w.item(id).is_some_and(|i| i.visible()) {
            return true;
        }
        let pack = w.mobile(m).and_then(|x| x.backpack);
        let team_game = goal_data(w, id)
            .team
            .and_then(|(ec, t)| team(w, ec, t))
            .and_then(|t| t.game);
        if !muo_content::mx::player(w, m)
            || !muo_content::mx::alive(w, m)
            || pack.is_none()
            || team_game.is_none()
        {
            return true;
        }
        if !(BASE_ADDON.on_move_over)(w, id, m) {
            return false;
        }
        let color = goal_data(w, id)
            .team
            .and_then(|(ec, t)| team(w, ec, t))
            .map_or(0, |t| t.color);
        set_goal_hue(w, id, if color != 0 { color } else { 0x84C });
        if let Some(bomb) =
            pack.and_then(|p| muo_server::item::enumerable::find_item_by_type(w, p, true, &BR_BOMB))
        {
            check_score(w, bomb, id, m, 7);
        }
        true
    },
    serialize: |w, id, out| {
        (BASE_ADDON.serialize)(w, id, out);
        out.write_i32(0);
        out.write_bool(goal_data(w, id).north);
    },
    deserialize: |w, id, r| {
        (BASE_ADDON.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let north = r.read_bool()?;
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(GoalData { north, team: None });
        }
        Ok(())
    },
    ..BASE_ADDON
};

// ---- BRController -----------------------------------------------------------------------------------------------------

/// `new BRController()`.
pub fn new_controller_br(w: &mut World) -> ItemId {
    new_controller(w, &BR_CONTROLLER, EventCtrl::new(GameKind::Br, 4))
}

/// `BRTeamInfo.Serialize`.
fn write_team(w: &World, t: &TeamInfo, out: &mut dyn GenericWriter) {
    out.write_encoded_int(0);
    out.write_item(w, t.board);
    out.write_opt_string(t.name.as_deref());
    out.write_encoded_int(t.color);
    out.write_item(w, t.goal);
}

fn read_team(w: &World, id: usize, r: &mut dyn GenericReader) -> muo_io::Result<TeamInfo> {
    let mut t = TeamInfo::new(id);
    if r.read_encoded_int()? == 0 {
        t.board = r.read_item(w)?;
        t.name = r.read_string()?;
        t.color = r.read_encoded_int()?;
        t.goal = r.read_item(w)?;
    }
    Ok(t)
}

/// `BRController`.
pub static BR_CONTROLLER: ItemType = ItemType {
    full_name: "Server.Engines.ConPVP.BRController",
    parent: Some(&EVENT_CONTROLLER),
    construct: Some(new_controller_br),
    default_name: |w, id| crate::template_keys::item_name(w, id, EVENT_CONTROLLER.default_name),
    serialize: |w, id, out| {
        (EVENT_CONTROLLER.serialize)(w, id, out);
        out.write_i32(0);
        let Some(c) = ctrl(w, id) else { return };
        out.write_point3d(c.br.bomb_home);
        out.write_time_span_ticks(c.duration.ticks());
        out.write_encoded_int(c.teams.len() as i32);
        for t in &c.teams {
            write_team(w, t, out);
        }
    },
    deserialize: |w, id, r| {
        (EVENT_CONTROLLER.deserialize)(w, id, r)?;
        let version = r.read_i32()?;
        let mut c = EventCtrl::new(GameKind::Br, 4);
        if version == 0 {
            c.br.bomb_home = r.read_point3d()?;
            c.duration = TimeSpan::from_ticks(r.read_time_span_ticks()?);
            let n = r.read_encoded_int()?.max(0) as usize;
            c.teams = Vec::with_capacity(n);
            for i in 0..n {
                c.teams.push(read_team(w, i, r)?);
            }
        }
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(c);
        }
        Ok(())
    },
    ..EVENT_CONTROLLER
};

// ---- BRGame -----------------------------------------------------------------------------------------------------------

/// `BRGame.CantDoAnything(mob)`: carrying the bomb.
pub fn cant_do_anything(w: &World, id: u32, mob: MobileId) -> bool {
    let carrying = w
        .mobile(mob)
        .and_then(|m| m.backpack)
        .and_then(|p| muo_server::item::enumerable::find_item_by_type(w, p, true, &BR_BOMB))
        .is_some();
    carrying && team_of(w, id, mob).is_some()
}

/// `BRGame.OnDeath`.
pub fn on_death(w: &mut World, id: u32, ec: ItemId, mob: MobileId, corpse: Option<ItemId>) {
    let killer = muo_server::mobile::part2::damage::find_most_recent_damager(w, mob, false);
    let mut had_bomb = false;
    let mut bombs = Vec::new();
    if let Some(c) = corpse {
        bombs.extend(muo_server::item::enumerable::find_items_by_type(
            w,
            c,
            false,
            &BR_BOMB,
            |_, _| true,
        ));
    }
    if let Some(p) = w.mobile(mob).and_then(|m| m.backpack) {
        bombs.extend(muo_server::item::enumerable::find_items_by_type(
            w,
            p,
            false,
            &BR_BOMB,
            |_, _| true,
        ));
    }
    for b in bombs {
        had_bomb = true;
        drop_bomb_to(w, b, mob, killer);
    }
    if let Some(killer) = killer.filter(|k| muo_content::mx::player(w, *k)) {
        let (ti, vict) = (team_of(w, id, killer), team_of(w, id, mob));
        if let Some(ti) = ti.filter(|t| Some(*t) != vict)
            && let Some(t) = team_mut(w, ec, ti)
        {
            t.add_kills(killer, 1);
            t.add_score(killer, 1); // base frag
            if had_bomb {
                t.add_score(killer, 4); // fragged bomb carrier
            }
        }
    }
    display_board(w, id, mob);
    flow::requip(w, id, mob, corpse);
    games::delay_bounce(w, id, TimeSpan::from_secs(30), mob, corpse, false);
}

/// `BRGame.OnStart`.
pub fn on_start(w: &mut World, id: u32, ec: ItemId) {
    let n = ctrl(w, ec).map_or(0, |c| c.teams.len());
    for i in 0..n {
        let (board, goal) = team(w, ec, i)
            .map(|t| (t.board, t.goal))
            .unwrap_or_default();
        if let Some(t) = team_mut(w, ec, i) {
            t.game = Some(id);
            t.reset_scores();
        }
        if let Some(b) = board {
            games::set_board_link(w, b, Some((ec, i)));
        }
        if let Some(g) = goal {
            set_goal_team(w, g, Some((ec, i)));
        }
    }
    let parts = ctx(w, id).map_or(0, |c| c.participants.len());
    for i in 0..parts {
        if let Some(color) = team(w, ec, i % n.max(1)).map(|t| t.color) {
            games::apply_hues(w, id, i, color);
        }
    }
    let mut old = std::mem::take(&mut rt_mut(w, id).finish);
    old.cancel(w);
    let bomb = new_bomb(w, id);
    rt_mut(w, id).br.bomb = Some(bomb);
    return_bomb(w, id);
    games::start_finish_timer(w, id, ec);
}

/// `BRGame.OnStop`.
pub fn on_stop(w: &mut World, id: u32, ec: ItemId) {
    let n = ctrl(w, ec).map_or(0, |c| c.teams.len());
    for i in 0..n {
        if let Some(b) = team(w, ec, i).and_then(|t| t.board) {
            games::set_board_link(w, b, None);
        }
        if let Some(t) = team_mut(w, ec, i) {
            t.game = None;
        }
    }
    return_bomb(w, id);
    if let Some(b) = rt(w, id).and_then(|r| r.br.bomb)
        && let Some(ty) = w.item(b).map(|i| i.ty)
    {
        (ty.delete)(w, b);
    }
    let parts = ctx(w, id).map_or(0, |c| c.participants.len());
    for i in 0..parts {
        games::apply_hues(w, id, i, -1);
    }
}

/// Registers the classes.
pub fn register(types: &mut TypeRegistry) {
    types.register_item(&BR_BOARD, &[]);
    types.register_item(&BR_BOMB, &[]);
    types.register_item(&BR_GOAL, &[]);
    types.register_item(&BR_CONTROLLER, &[]);
    let _ = Comp::Plain(0);
}

#[cfg(test)]
mod tests {
    use super::*;
    use muo_server::mobile::MOBILE_BASE;
    use muo_server::target::LandTarget;

    #[test]
    fn pvp_a_thrown_bomb_flies_the_path_and_lands_on_the_target() {
        let mut w = World::new();
        muo_content::register(&mut w.types);
        muo_items::register(&mut w.types);
        let a = w.new_mobile(&MOBILE_BASE);
        w.mobile_mut(a).unwrap().player = true;
        muo_server::mobile::base::move_to_world(
            &mut w,
            a,
            Point3D::new(100, 100, 0),
            MapId::FELUCCA,
        );
        let pack = muo_content::factory::make_item(&mut w, "Server.Items.Backpack").unwrap();
        muo_server::mobile::part2::items::add_item(&mut w, a, pack);
        muo_server::mobile::part1::props_b::backpack(&mut w, a);
        let bomb = new_bomb(&mut w, 1);
        assert!(muo_server::mobile::part2::items::place_in_backpack(
            &mut w, a, bomb
        ));
        assert!(ops::is_child_of(
            &w,
            bomb,
            Some(EntityId::Item(w.mobile(a).unwrap().backpack.unwrap()))
        ));

        let target = Targeted::Land(LandTarget {
            location: Point3D::new(108, 100, 0),
            tile_id: 0,
        });
        assert!(
            on_bomb_target(&mut w, bomb, a, target),
            "a throw is handled"
        );
        assert!(bd(&w, bomb).unwrap().flying);
        assert!(!w.item(bomb).unwrap().visible());
        // the path is a lob from the thrower to the target
        let path = bd(&w, bomb).unwrap().path.clone();
        assert!(!path.is_empty() || bd(&w, bomb).unwrap().path_idx > 0);
        for _ in 0..60 {
            w.tick_count += 100;
            w.now += TimeSpan::from_millis(100);
            muo_server::timer::slice(&mut w);
        }
        let d = bd(&w, bomb).unwrap();
        assert!(!d.flying, "it came down");
        assert!(w.item(bomb).unwrap().visible());
        let at = w.item(bomb).unwrap().location;
        assert!(
            (at.x - 108).abs() <= 1 && at.y == 100,
            "it lands at the target: {at:?}"
        );
        // a thrower cannot throw it at himself (once it is in his pack again)
        assert!(muo_server::mobile::part2::items::place_in_backpack(
            &mut w, a, bomb
        ));
        assert!(!on_bomb_target(
            &mut w,
            bomb,
            a,
            Targeted::Entity(EntityId::Mobile(a))
        ));
    }
}
