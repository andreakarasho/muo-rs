//! Double Domination: hold both waypoints for ten seconds. `DDController` (two teams),
//! `DDWayPoint` (the addon with its eight `DDStep`s), `DDBoard` and the game's rules.

use muo_content::items::addon::{self, ADDON_COMPONENT, AddonClass, BASE_ADDON, BASE_ADDON_CLASS};
use muo_io::{GenericReader, GenericWriter};
use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, ops};
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::timer;
use muo_server::type_registry::TypeRegistry;

use super::games::{
    self, EVENT_CONTROLLER, EventCtrl, GameKind, TeamInfo, alert, board_deserialize,
    board_double_click, board_serialize, ctrl, display_board, new_board, new_controller, rt_mut,
    team, team_mut, team_of,
};
use super::{ctx, flow};

pub const UNCAPTURABLE_HUE: i32 = 0x497;
pub const NON_CAPTURED_HUE: i32 = 0x38A;

/// `DDBoard`.
pub static DD_BOARD: ItemType = ItemType {
    full_name: "Server.Engines.ConPVP.DDBoard",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| new_board(w, &DD_BOARD)),
    default_name: |w, id| crate::template_keys::item_name(w, id, ITEM_BASE.default_name),
    on_double_click: board_double_click,
    serialize: |w, id, out| board_serialize(w, id, out),
    deserialize: |w, id, r| board_deserialize(w, id, r),
    ..ITEM_BASE
};

// ---- DDWayPoint -------------------------------------------------------------------------------------------------

/// The state of a waypoint that is never saved: `Game` (the context id) and `TeamOwner` (a team index).
#[derive(Clone, Copy, Debug, Default)]
pub struct WayPointData {
    pub game: Option<u32>,
    pub owner: Option<usize>,
}

fn wp(w: &World, id: ItemId) -> WayPointData {
    w.item(id)
        .and_then(|i| i.ext.get::<WayPointData>())
        .copied()
        .unwrap_or_default()
}

fn wp_mut(w: &mut World, id: ItemId) -> Option<&mut WayPointData> {
    w.item_mut(id)?.ext.get_mut::<WayPointData>()
}

fn set_comp_hues(w: &mut World, point: ItemId, hue: i32) {
    for c in addon::components(w, point) {
        if let Some(ty) = w.item(c).map(|i| i.ty) {
            (ty.set_hue)(w, c, hue);
        }
    }
}

fn set_hue(w: &mut World, point: ItemId, hue: i32) {
    if let Some(ty) = w.item(point).map(|i| i.ty) {
        (ty.set_hue)(w, point, hue);
    }
}

/// `SetUncapturableHue()`.
pub fn set_uncapturable_hue(w: &mut World, point: ItemId) {
    set_comp_hues(w, point, UNCAPTURABLE_HUE);
    set_hue(w, point, UNCAPTURABLE_HUE);
}

fn owner_color(w: &World, point: ItemId) -> Option<i32> {
    let d = wp(w, point);
    let ec = ctx(w, d.game?)?.event_game?.controller;
    team(w, ec, d.owner?).map(|t| t.color)
}

/// `SetNonCaptureHue()`.
pub fn set_non_capture_hue(w: &mut World, point: ItemId) {
    set_comp_hues(w, point, NON_CAPTURED_HUE);
    let hue = owner_color(w, point).unwrap_or(NON_CAPTURED_HUE);
    set_hue(w, point, hue);
}

/// `SetCaptureHue(stage)`.
pub fn set_capture_hue(w: &mut World, point: ItemId, stage: i32) {
    let Some(color) = owner_color(w, point) else {
        return;
    };
    set_hue(w, point, color);
    for (i, c) in addon::components(w, point).into_iter().enumerate() {
        if let Some(ty) = w.item(c).map(|x| x.ty) {
            (ty.set_hue)(
                w,
                c,
                if (i as i32) < stage {
                    color
                } else {
                    NON_CAPTURED_HUE
                },
            );
        }
    }
}

/// `Game = value`.
pub fn set_game(w: &mut World, point: ItemId, game: Option<u32>) {
    if let Some(d) = wp_mut(w, point) {
        d.game = game;
        d.owner = None;
    }
    if game.is_some() {
        set_non_capture_hue(w, point);
    } else {
        set_uncapturable_hue(w, point);
    }
}

/// `TeamOwner = value`.
pub fn set_owner(w: &mut World, point: ItemId, owner: Option<usize>) {
    if let Some(d) = wp_mut(w, point) {
        d.owner = owner;
    }
    set_non_capture_hue(w, point);
}

fn way_on_move_over(w: &mut World, id: ItemId, m: MobileId) -> bool {
    match wp(w, id).game {
        None => set_uncapturable_hue(w, id),
        Some(game) if muo_content::mx::alive(w, m) => {
            if let Some(t) = team_of(w, game, m)
                && Some(t) != wp(w, id).owner
            {
                dominate(w, game, id, m, t);
            }
        }
        Some(_) => {}
    }
    true
}

static WAYPOINT_CLASS: AddonClass = AddonClass {
    share_hue: false,
    ..BASE_ADDON_CLASS
};

/// `DDWayPoint.DDStep`.
pub static DD_STEP: ItemType = ItemType {
    full_name: "Server.Engines.ConPVP.DDWayPoint+DDStep",
    parent: Some(&ADDON_COMPONENT),
    construct: None,
    // `Addon.OnMoveOver(m)`
    on_move_over: |w, id, m| match addon::component_addon(w, id) {
        Some(a) => w
            .item(a)
            .map(|i| i.ty)
            .is_none_or(|ty| (ty.on_move_over)(w, a, m)),
        None => true,
    },
    serialize: |w, id, out| {
        (ADDON_COMPONENT.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (ADDON_COMPONENT.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..ADDON_COMPONENT
};

fn new_step(w: &mut World, item_id: i32) -> ItemId {
    let id = addon::new_component(w, &DD_STEP, item_id);
    ops::set_visible(w, id, true);
    id
}

/// `new DDWayPoint()`.
pub fn new_way_point(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &DD_WAY_POINT);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(WayPointData::default());
    }
    muo_content::templates::apply_item_as(w, id, DD_WAY_POINT.full_name);
    for (item, x, y) in [
        (0x7A8, -1, -1),
        (0x7A6, 0, -1),
        (0x7AA, 1, -1),
        (0x7A5, 1, 0),
        (0x7A9, 1, 1),
        (0x7A4, 0, 1),
        (0x7AB, -1, 1),
        (0x7A7, -1, 0),
    ] {
        let c = new_step(w, item);
        addon::add_component(w, id, c, x, y, -5);
    }
    set_uncapturable_hue(w, id);
    id
}

/// `DDWayPoint`.
pub static DD_WAY_POINT: ItemType = ItemType {
    full_name: "Server.Engines.ConPVP.DDWayPoint",
    parent: Some(&BASE_ADDON),
    construct: Some(new_way_point),
    family: Some(&WAYPOINT_CLASS),
    on_move_over: way_on_move_over,
    serialize: |w, id, out| {
        (BASE_ADDON.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (BASE_ADDON.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(WayPointData::default());
        }
        Ok(())
    },
    ..BASE_ADDON
};

// ---- DDController -----------------------------------------------------------------------------------------------

/// `new DDController()`.
pub fn new_controller_dd(w: &mut World) -> ItemId {
    new_controller(w, &DD_CONTROLLER, EventCtrl::new(GameKind::Dd, 2))
}

/// `DDTeamInfo.Serialize`.
fn write_team(w: &World, t: &TeamInfo, out: &mut dyn GenericWriter) {
    out.write_encoded_int(0);
    out.write_item(w, t.board);
    out.write_opt_string(t.name.as_deref());
    out.write_encoded_int(t.color);
    out.write_point3d(t.origin);
}

fn read_team(w: &World, id: usize, r: &mut dyn GenericReader) -> muo_io::Result<TeamInfo> {
    let mut t = TeamInfo::new(id);
    if r.read_encoded_int()? == 0 {
        t.board = r.read_item(w)?;
        t.name = r.read_string()?;
        t.color = r.read_encoded_int()?;
        t.origin = r.read_point3d()?;
    }
    Ok(t)
}

/// `DDController`.
pub static DD_CONTROLLER: ItemType = ItemType {
    full_name: "Server.Engines.ConPVP.DDController",
    parent: Some(&EVENT_CONTROLLER),
    construct: Some(new_controller_dd),
    default_name: |w, id| crate::template_keys::item_name(w, id, EVENT_CONTROLLER.default_name),
    serialize: |w, id, out| {
        (EVENT_CONTROLLER.serialize)(w, id, out);
        out.write_i32(0);
        let Some(c) = ctrl(w, id) else { return };
        out.write_time_span_ticks(c.duration.ticks());
        out.write_encoded_int(c.teams.len() as i32);
        for t in &c.teams {
            write_team(w, t, out);
        }
        out.write_item(w, c.point_a);
        out.write_item(w, c.point_b);
    },
    deserialize: |w, id, r| {
        (EVENT_CONTROLLER.deserialize)(w, id, r)?;
        let version = r.read_i32()?;
        let mut c = EventCtrl::new(GameKind::Dd, 2);
        if version == 0 {
            c.duration = TimeSpan::from_ticks(r.read_time_span_ticks()?);
            let n = r.read_encoded_int()?.max(0) as usize;
            c.teams = Vec::with_capacity(n);
            for i in 0..n {
                c.teams.push(read_team(w, i, r)?);
            }
            c.point_a = r.read_item(w)?;
            c.point_b = r.read_item(w)?;
        }
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(c);
        }
        Ok(())
    },
    ..EVENT_CONTROLLER
};

// ---- DDGame -----------------------------------------------------------------------------------------------------------

/// `DDGame.OnDeath`.
pub fn on_death(w: &mut World, id: u32, ec: ItemId, mob: MobileId, corpse: Option<ItemId>) {
    let killer = muo_server::mobile::part2::damage::find_most_recent_damager(w, mob, false);
    if let Some(killer) = killer.filter(|k| muo_content::mx::player(w, *k)) {
        let (ti, vict) = (team_of(w, id, killer), team_of(w, id, mob));
        if let Some(ti) = ti.filter(|t| Some(*t) != vict) {
            let (pa, pb) = ctrl(w, ec)
                .map(|c| (c.point_a, c.point_b))
                .unwrap_or_default();
            let on_point = |w: &World, p: Option<ItemId>| {
                p.is_some_and(|p| {
                    muo_content::mx::in_range_point(w, mob, ops::get_world_location(w, p), 2)
                })
            };
            let bonus = i32::from(on_point(w, pa)) + i32::from(on_point(w, pb));
            if let Some(t) = team_mut(w, ec, ti) {
                t.add_kills(killer, 1);
                t.add_score(killer, 1); // base frag
                t.add_score(killer, bonus); // extra points for killing someone on the waypoint
            }
            if let Some(v) = vict
                && let Some(t) = team_mut(w, ec, v)
            {
                t.add_score(mob, -1);
            }
        }
    }
    display_board(w, id, mob);
    flow::requip(w, id, mob, corpse);
    games::delay_bounce(w, id, TimeSpan::from_secs(30), mob, corpse, false);
}

/// `DDGame.OnStart`.
pub fn on_start(w: &mut World, id: u32, ec: ItemId) {
    {
        let rt = rt_mut(w, id);
        rt.capturable = true;
        let (mut c, mut u) = (
            std::mem::take(&mut rt.capture),
            std::mem::take(&mut rt.uncapture),
        );
        c.cancel(w);
        u.cancel(w);
    }
    let n = ctrl(w, ec).map_or(0, |c| c.teams.len());
    for i in 0..n {
        let board = team(w, ec, i).and_then(|t| t.board);
        if let Some(t) = team_mut(w, ec, i) {
            t.game = Some(id);
            t.reset_scores();
        }
        if let Some(b) = board {
            games::set_board_link(w, b, Some((ec, i)));
        }
    }
    let (pa, pb) = ctrl(w, ec)
        .map(|c| (c.point_a, c.point_b))
        .unwrap_or_default();
    for p in [pa, pb].into_iter().flatten() {
        set_game(w, p, Some(id));
    }
    let parts = ctx(w, id).map_or(0, |c| c.participants.len());
    for i in 0..parts {
        if let Some(color) = team(w, ec, i % n.max(1)).map(|t| t.color) {
            games::apply_hues(w, id, i, color);
        }
    }
    games::start_finish_timer(w, id, ec);
}

/// `DDGame.OnStop`.
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
    let (pa, pb) = ctrl(w, ec)
        .map(|c| (c.point_a, c.point_b))
        .unwrap_or_default();
    for p in [pa, pb].into_iter().flatten() {
        set_game(w, p, None);
    }
    let rt = rt_mut(w, id);
    rt.capturable = false;
    let (mut c, mut u) = (
        std::mem::take(&mut rt.capture),
        std::mem::take(&mut rt.uncapture),
    );
    c.cancel(w);
    u.cancel(w);
    let parts = ctx(w, id).map_or(0, |c| c.participants.len());
    for i in 0..parts {
        games::apply_hues(w, id, i, -1);
    }
}

fn owners(w: &World, ec: ItemId) -> (Option<usize>, Option<usize>) {
    let (pa, pb) = ctrl(w, ec)
        .map(|c| (c.point_a, c.point_b))
        .unwrap_or_default();
    (
        pa.and_then(|p| wp(w, p).owner),
        pb.and_then(|p| wp(w, p).owner),
    )
}

/// `Dominate(point, from, team)`.
pub fn dominate(w: &mut World, id: u32, point: ItemId, _from: MobileId, t: usize) {
    let Some(ec) = ctx(w, id).and_then(|c| c.event_game).map(|g| g.controller) else {
        return;
    };
    if !games::rt(w, id).is_some_and(|r| r.capturable) {
        return;
    }
    let dom = |(a, b): (Option<usize>, Option<usize>)| a == b && a.is_some();
    let was_dom = dom(owners(w, ec));

    set_owner(w, point, Some(t));
    let tname = team(w, ec, t)
        .map(|t| t.text_name(GameKind::Dd))
        .unwrap_or_default();
    let pname = ops::name(w, point).unwrap_or_default();
    alert(w, id, &format!("{tname} has captured {pname}!"));

    let is_dom = dom(owners(w, ec));
    let (pa, pb) = ctrl(w, ec)
        .map(|c| (c.point_a, c.point_b))
        .unwrap_or_default();
    if was_dom && !is_dom {
        alert(w, id, "Domination averted!");
        for p in [pa, pb].into_iter().flatten() {
            set_non_capture_hue(w, p);
        }
        let mut h = std::mem::take(&mut rt_mut(w, id).capture);
        h.cancel(w);
    }
    if !was_dom && is_dom {
        rt_mut(w, id).cap_stage = 0;
        let h = timer::start(w, TimeSpan::ZERO, TimeSpan::from_secs(1), 0, move |w| {
            capture_tick(w, id, ec)
        });
        rt_mut(w, id).capture = h;
    }
}

fn capture_tick(w: &mut World, id: u32, ec: ItemId) {
    let (pa, pb) = ctrl(w, ec)
        .map(|c| (c.point_a, c.point_b))
        .unwrap_or_default();
    let (oa, ob) = owners(w, ec);
    let Some(t) = oa.or(ob) else {
        let rt = rt_mut(w, id);
        rt.capturable = true;
        let mut h = std::mem::take(&mut rt.capture);
        h.cancel(w);
        return;
    };
    let stage = {
        let rt = rt_mut(w, id);
        rt.cap_stage += 1;
        rt.cap_stage
    };
    let tname = team(w, ec, t)
        .map(|t| t.text_name(GameKind::Dd))
        .unwrap_or_default();
    if stage < 10 {
        alert(w, id, &format!("{tname} is dominating... {}", 10 - stage));
        for p in [pa, pb].into_iter().flatten() {
            set_capture_hue(w, p, stage);
        }
    } else {
        alert(w, id, &format!("{tname} has scored!"));
        if let Some(tm) = team_mut(w, ec, t) {
            tm.score += 100;
            tm.captures += 1;
        }
        {
            let rt = rt_mut(w, id);
            rt.capturable = false;
            rt.cap_stage = 0;
            let mut h = std::mem::take(&mut rt.capture);
            h.cancel(w);
        }
        for p in [pa, pb].into_iter().flatten() {
            set_owner(w, p, None);
            set_uncapturable_hue(w, p);
        }
        let h = timer::start(w, TimeSpan::from_secs(30), TimeSpan::ZERO, 1, move |w| {
            uncapture_tick(w, id, ec)
        });
        rt_mut(w, id).uncapture = h;
    }
}

fn uncapture_tick(w: &mut World, id: u32, ec: ItemId) {
    {
        let rt = rt_mut(w, id);
        rt.capturable = true;
        let (mut c, mut u) = (
            std::mem::take(&mut rt.capture),
            std::mem::take(&mut rt.uncapture),
        );
        c.cancel(w);
        u.cancel(w);
    }
    let (pa, pb) = ctrl(w, ec)
        .map(|c| (c.point_a, c.point_b))
        .unwrap_or_default();
    for p in [pa, pb].into_iter().flatten() {
        set_owner(w, p, None);
        set_non_capture_hue(w, p);
    }
}

/// Registers the classes.
pub fn register(types: &mut TypeRegistry) {
    types.register_item(&DD_BOARD, &[]);
    types.register_item(&DD_STEP, &[]);
    types.register_item(&DD_WAY_POINT, &[]);
    types.register_item(&DD_CONTROLLER, &[]);
}
