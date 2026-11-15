//! Capture the flag: capture the cookies. `CTFController` (eight teams), `CTFFlag`, `CTFBoard`
//! and the game's rules. The scoreboard gump is `games::BoardGump`.

use muo_io::{GenericReader, GenericWriter};
use muo_primitives::{DateTime, MessageType, Point3D, TimeSpan};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, ops};
use muo_server::mobile::part2::messages as mm;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::target::{TargetFlags, Targeted};
use muo_server::timer::{self, TimerHandle};
use muo_server::type_registry::{Phase, TypeRegistry};

use super::games::{
    self, EVENT_CONTROLLER, EventCtrl, GameKind, TeamInfo, alert, board_deserialize,
    board_double_click, board_serialize, color_of, ctrl, ctrl_mut, display_board, facet,
    find_owner, new_board, new_controller, team, team_mut, team_of,
};
use super::{ctx, duel_context_of, flow};

const SOLID_FLAG_HUE: i32 = 0x4001;

// ---- CTFBoard ---------------------------------------------------------------------------------------------------------

/// `CTFBoard`.
pub static CTF_BOARD: ItemType = ItemType {
    full_name: "Server.Engines.ConPVP.CTFBoard",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| new_board(w, &CTF_BOARD)),
    default_name: |w, id| crate::template_keys::item_name(w, id, ITEM_BASE.default_name),
    on_double_click: board_double_click,
    serialize: |w, id, out| board_serialize(w, id, out),
    deserialize: |w, id, r| board_deserialize(w, id, r),
    ..ITEM_BASE
};

// ---- CTFFlag ----------------------------------------------------------------------------------------------------------

/// The runtime state of a `CTFFlag` (nothing of it is saved).
#[derive(Clone, Debug, Default)]
pub struct FlagData {
    /// `m_TeamInfo`: the controller and the team index.
    pub link: Option<(ItemId, usize)>,
    pub fragger: Option<MobileId>,
    pub frag_time: DateTime,
    pub return_count: i32,
    pub returner: Option<MobileId>,
    pub return_time: DateTime,
    pub timer: TimerHandle,
}

fn fd(w: &World, id: ItemId) -> Option<&FlagData> {
    w.item(id)?.ext.get::<FlagData>()
}

fn fd_mut(w: &mut World, id: ItemId) -> Option<&mut FlagData> {
    w.item_mut(id)?.ext.get_mut::<FlagData>()
}

/// `new CTFFlag()`.
pub fn new_flag(w: &mut World) -> ItemId {
    let id = ops::new_item_with_id(w, &CTF_FLAG, 5643);
    ops::set_movable(w, id, false);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(FlagData::default());
    }
    id
}

pub fn set_flag_link(w: &mut World, flag: ItemId, link: Option<(ItemId, usize)>) {
    if let Some(d) = fd_mut(w, flag) {
        d.link = link;
    }
}

/// The game a flag belongs to: (context id, controller, team).
fn game_of(w: &World, flag: ItemId) -> Option<(u32, ItemId, usize)> {
    let (ec, t) = fd(w, flag)?.link?;
    Some((team(w, ec, t)?.game?, ec, t))
}

/// `SendHome()`.
pub fn send_home(w: &mut World, flag: ItemId) {
    if let Some(d) = fd_mut(w, flag) {
        let mut t = std::mem::take(&mut d.timer);
        t.cancel(w);
    }
    let Some((ec, t)) = fd(w, flag).and_then(|d| d.link) else {
        return;
    };
    let Some(origin) = team(w, ec, t).map(|t| t.origin) else {
        return;
    };
    let Some(game) = team(w, ec, t).and_then(|t| t.game) else {
        return;
    };
    let map = facet(w, game);
    ops::move_to_world_map(w, flag, origin, map);
}

/// `DropTo(mob, killer)`.
pub fn drop_to(w: &mut World, flag: ItemId, mob: Option<MobileId>, killer: Option<MobileId>) {
    let now = w.now;
    if let Some(d) = fd_mut(w, flag) {
        d.fragger = killer;
        d.frag_time = now;
    }
    match mob {
        Some(m) => {
            let (loc, map) = w
                .mobile(m)
                .map_or((Point3D::default(), None), |x| (x.location, x.map));
            ops::move_to_world_map(w, flag, Point3D::new(loc.x, loc.y, loc.z + 2), map);
            if let Some(d) = fd_mut(w, flag) {
                d.return_count = d.return_count.min(10);
            }
        }
        None => send_home(w, flag),
    }
}

fn begin_countdown(w: &mut World, flag: ItemId, count: i32) {
    if let Some(d) = fd_mut(w, flag) {
        let mut t = std::mem::take(&mut d.timer);
        t.cancel(w);
    }
    let one = TimeSpan::from_secs(1);
    let h = timer::start(w, one, one, 0, move |w| countdown_tick(w, flag));
    if let Some(d) = fd_mut(w, flag) {
        d.timer = h;
        d.return_count = count;
    }
}

fn countdown_tick(w: &mut World, flag: ItemId) {
    let owner = match ops::root_parent(w, flag) {
        Some(EntityId::Mobile(m)) => Some(m),
        _ => None,
    };
    let count = fd(w, flag).map_or(0, |d| d.return_count);
    match count {
        60 | 30 | 15 | 10 | 5 | 4 | 3 | 2 | 1 => {
            if let Some(o) = owner {
                let text = if count == 1 {
                    format!("You have {count} second to capture the cookies!")
                } else {
                    format!("You have {count} seconds to capture the cookies!")
                };
                mm::send_message_hue(w, o, 0x26, &text);
            }
        }
        0 => {
            if let Some(o) = owner {
                mm::send_message_hue(
                    w,
                    o,
                    0x26,
                    "You have taken too long to capture the cookies!",
                );
                muo_server::mobile::base::kill(w, o);
            }
            send_home(w, flag);
            if let Some((game, ec, t)) = game_of(w, flag) {
                let name = team(w, ec, t)
                    .map(|t| t.text_name(GameKind::Ctf))
                    .unwrap_or_default();
                alert(w, game, &format!("The {name} cookies have been returned."));
            }
            return;
        }
        _ => {}
    }
    if let Some(d) = fd_mut(w, flag) {
        d.return_count -= 1;
    }
}

fn say(w: &mut World, to: MobileId, hue: i32, text: &str) {
    mm::local_overhead_message(w, to, MessageType::REGULAR, hue, false, text);
}

fn flag_double_click(w: &mut World, id: ItemId, from: MobileId) {
    let Some((game, ec, our)) = game_of(w, id) else {
        return;
    };
    let Some(use_team) = team_of(w, game, from) else {
        return;
    };
    let pack = w.mobile(from).and_then(|m| m.backpack);

    if ops::is_child_of(w, id, pack.map(EntityId::Item)) {
        target_flag(w, id, from);
    } else if !{
        let loc = ops::get_world_location(w, id);
        muo_server::item::deps::mobile_in_range(w, from, loc, 1)
    } || !muo_server::item::deps::in_los_item(w, from, id)
    {
        mm::local_overhead_message_number(w, from, MessageType::REGULAR, 0x26, 1019045, ""); // I can't reach that
    } else if our == use_team {
        let (loc, map) = w
            .item(id)
            .map_or((Point3D::default(), None), |i| (i.location, i.map));
        let origin = team(w, ec, our).map(|t| t.origin);
        if Some(loc) == origin && map == facet(w, game) {
            let (name, serial, graphic) = {
                let name = ops::name(w, id).unwrap_or_default();
                let i = w.item(id).expect("flag");
                (name, i.serial, i.item_id)
            };
            if let Some(ns) = w.mobile(from).and_then(|m| m.net_state) {
                muo_server::packets::message::send_message(
                    w,
                    ns,
                    serial,
                    graphic,
                    muo_server::packets::message::MessageType::REGULAR,
                    0x3B2,
                    3,
                    false,
                    Some("ENU"),
                    &name,
                    "Touch me not for I am chaste.",
                );
            }
        } else {
            if let Some(t) = team_mut(w, ec, use_team) {
                t.add_score(from, 4); // return
            }
            let now = w.now;
            if let Some(d) = fd_mut(w, id) {
                d.returner = Some(from);
                d.return_time = now;
            }
            send_home(w, id);
            say(w, from, 0x59, "You returned the cookies!");
            let (fname, tname) = (
                super::flow::name(w, from),
                team(w, ec, our)
                    .map(|t| t.text_name(GameKind::Ctf))
                    .unwrap_or_default(),
            );
            alert(
                w,
                game,
                &format!("The {fname} cookies have been returned by {tname}."),
            );
        }
    } else if !muo_server::mobile::part2::items::place_in_backpack(w, from, id) {
        say(w, from, 0x26, "I can't hold that.");
    } else {
        muo_server::mobile::base::revealing_action(w, from);
        say(w, from, 0x59, "You stole the cookies!");
        let our_name = team(w, ec, our)
            .map(|t| t.text_name(GameKind::Ctf))
            .unwrap_or_default();
        let use_name = team(w, ec, use_team)
            .map(|t| t.text_name(GameKind::Ctf))
            .unwrap_or_default();
        let from_name = super::flow::name(w, from);
        alert(
            w,
            game,
            &format!("The {our_name} cookies have been stolen by {from_name} ({use_name})."),
        );
        begin_countdown(w, id, 120);
    }
}

fn target_flag(w: &mut World, id: ItemId, from: MobileId) {
    muo_server::target::begin_target(w, from, 1, false, TargetFlags::NONE, move |w, from, obj| {
        flag_on_target(w, id, from, obj)
    });
}

pub(super) fn flag_on_target(w: &mut World, id: ItemId, from: MobileId, obj: Targeted) {
    let Some((game, ec, our)) = game_of(w, id) else {
        return;
    };
    let pack = w.mobile(from).and_then(|m| m.backpack);
    if !ops::is_child_of(w, id, pack.map(EntityId::Item)) {
        return;
    }
    let Some(use_team) = team_of(w, game, from) else {
        return;
    };
    let Targeted::Entity(e) = obj else { return };

    match e {
        EntityId::Item(item) if w.item(item).is_some_and(|i| i.ty.is(&CTF_FLAG)) => {
            if team(w, ec, use_team).and_then(|t| t.flag) == Some(item) {
                say(w, from, 0x59, "You captured the cookies!");
                let (fname, oname) = (
                    super::flow::name(w, from),
                    team(w, ec, our)
                        .map(|t| t.text_name(GameKind::Ctf))
                        .unwrap_or_default(),
                );
                alert(w, game, &format!("{fname} captured the {oname} cookies!"));
                send_home(w, id);

                if let Some(t) = team_mut(w, ec, use_team) {
                    t.add_captures(from, 1);
                    t.add_score(from, 50); // capture
                }
                let team_flag = item;
                let now = w.now;
                let (fragger, frag_time, returner, return_time) = fd(w, team_flag)
                    .map(|d| (d.fragger, d.frag_time, d.returner, d.return_time))
                    .unwrap_or_default();
                let five = TimeSpan::from_secs(5);
                if let Some(f) = fragger
                    && now < frag_time + five
                    && team_of(w, game, f) == Some(use_team)
                    && let Some(t) = team_mut(w, ec, use_team)
                {
                    t.add_score(f, 6); // frag assist
                }
                if let Some(r) = returner
                    && now < return_time + five
                    && let Some(t) = team_mut(w, ec, use_team)
                {
                    t.add_score(r, 4); // return assist
                }
            } else {
                say(w, from, 0x26, "Those are not my cookies.");
            }
        }
        EntityId::Mobile(pass_to) => {
            let pass_team = team_of(w, game, pass_to);
            if pass_to == from {
                say(w, from, 0x26, "I can't pass to them.");
            } else if pass_team == Some(use_team)
                && muo_server::mobile::part2::items::place_in_backpack(w, pass_to, id)
            {
                let n = super::flow::name(w, from);
                say(
                    w,
                    pass_to,
                    0x59,
                    &format!("{n} has passed you the cookies!"),
                );
            } else {
                say(w, from, 0x26, "I can't pass to them.");
            }
        }
        _ => {}
    }
}

/// `CTFFlag.OnAdded`.
fn flag_on_added(w: &mut World, id: ItemId, parent: EntityId) {
    (ITEM_BASE.on_added)(w, id, parent);
    if let Some(m) = find_owner(w, parent) {
        muo_server::mobile::part1::props_b::set_solid_hue_override(w, m, SOLID_FLAG_HUE);
    }
}

/// `CTFFlag.OnRemoved`.
fn flag_on_removed(w: &mut World, id: ItemId, parent: EntityId) {
    (ITEM_BASE.on_removed)(w, id, parent);
    if let Some(m) = find_owner(w, parent) {
        let hue = match game_of(w, id) {
            Some((game, _, _)) => color_of(w, game, m),
            None => -1,
        };
        muo_server::mobile::part1::props_b::set_solid_hue_override(w, m, hue);
    }
}

/// `CTFFlag`.
pub static CTF_FLAG: ItemType = ItemType {
    full_name: "Server.Engines.ConPVP.CTFFlag",
    parent: Some(&ITEM_BASE),
    construct: Some(new_flag),
    default_name: |w, id| crate::template_keys::item_name(w, id, ITEM_BASE.default_name),
    on_double_click: flag_double_click,
    // `Delete()`: carried, it goes home instead
    delete: |w, id| {
        if w.item(id).is_some_and(|i| i.parent.is_some()) {
            send_home(w, id);
            return;
        }
        (ITEM_BASE.delete)(w, id);
    },
    on_added: flag_on_added,
    on_removed: flag_on_removed,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(FlagData::default());
        }
        Ok(())
    },
    ..ITEM_BASE
};

// ---- CTFController ----------------------------------------------------------------------------------------------------

/// `new CTFController()`.
pub fn new_controller_ctf(w: &mut World) -> ItemId {
    new_controller(w, &CTF_CONTROLLER, EventCtrl::new(GameKind::Ctf, 8))
}

/// `CTFTeamInfo.Serialize`.
fn write_team(w: &World, t: &TeamInfo, out: &mut dyn GenericWriter) {
    out.write_encoded_int(2);
    out.write_item(w, t.board);
    out.write_opt_string(t.name.as_deref());
    out.write_encoded_int(t.color);
    out.write_item(w, t.flag);
    out.write_point3d(t.origin);
}

/// `new CTFTeamInfo(teamID, reader)`.
fn read_team(w: &World, id: usize, r: &mut dyn GenericReader) -> muo_io::Result<TeamInfo> {
    let mut t = TeamInfo::new(id);
    let version = r.read_encoded_int()?;
    if version >= 2 {
        t.board = r.read_item(w)?;
    }
    if version >= 1 {
        t.name = r.read_string()?;
    }
    if version >= 0 {
        t.color = r.read_encoded_int()?;
        t.flag = r.read_item(w)?;
        t.origin = r.read_point3d()?;
    }
    Ok(t)
}

/// `CTFController`.
pub static CTF_CONTROLLER: ItemType = ItemType {
    full_name: "Server.Engines.ConPVP.CTFController",
    parent: Some(&EVENT_CONTROLLER),
    construct: Some(new_controller_ctf),
    serialize: |w, id, out| {
        (EVENT_CONTROLLER.serialize)(w, id, out);
        out.write_i32(2);
        let Some(c) = ctrl(w, id) else { return };
        out.write_time_span_ticks(c.duration.ticks());
        out.write_encoded_int(c.teams.len() as i32);
        for t in &c.teams {
            write_team(w, t, out);
        }
    },
    deserialize: |w, id, r| {
        (EVENT_CONTROLLER.deserialize)(w, id, r)?;
        let version = r.read_i32()?;
        let mut c = EventCtrl::new(GameKind::Ctf, 8);
        if version >= 2 {
            c.duration = TimeSpan::from_ticks(r.read_time_span_ticks()?);
        }
        if version >= 1 {
            let n = r.read_encoded_int()?.max(0) as usize;
            c.teams = Vec::with_capacity(n);
            for i in 0..n {
                c.teams.push(read_team(w, i, r)?);
            }
        }
        if version < 2 {
            c.duration = TimeSpan::from_mins(30);
        }
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(c);
        }
        Ok(())
    },
    ..EVENT_CONTROLLER
};

// ---- CTFGame ----------------------------------------------------------------------------------------------------------

/// `CTFGame.OnDeath`.
pub fn on_death(w: &mut World, id: u32, ec: ItemId, mob: MobileId, corpse: Option<ItemId>) {
    let killer = muo_server::mobile::part2::damage::find_most_recent_damager(w, mob, false);
    let mut had_flag = false;

    let mut flags = Vec::new();
    if let Some(c) = corpse {
        flags.extend(muo_server::item::enumerable::find_items_by_type(
            w,
            c,
            false,
            &CTF_FLAG,
            |_, _| true,
        ));
    }
    if let Some(p) = w.mobile(mob).and_then(|m| m.backpack) {
        flags.extend(muo_server::item::enumerable::find_items_by_type(
            w,
            p,
            false,
            &CTF_FLAG,
            |_, _| true,
        ));
    }
    for f in flags {
        had_flag = true;
        drop_to(w, f, Some(mob), killer);
    }

    if let Some(killer) = killer.filter(|k| muo_content::mx::player(w, *k)) {
        let team_idx = team_of(w, id, killer);
        let vict = team_of(w, id, mob);
        if let Some(ti) = team_idx.filter(|t| Some(*t) != vict) {
            let origin = team(w, ec, ti).map_or_else(Point3D::default, |t| t.origin);
            let in_base = muo_content::mx::in_range_point(w, mob, origin, 24)
                && muo_content::mx::map(w, mob) == facet(w, id);
            if let Some(t) = team_mut(w, ec, ti) {
                t.add_kills(killer, 1);
                t.add_score(killer, 1); // base frag
                if had_flag {
                    t.add_score(killer, 4); // fragged flag carrier
                }
                if in_base {
                    t.add_score(killer, 1); // fragged in base -- guarding
                }
            }
            let n = ctrl(w, ec).map_or(0, |c| c.teams.len());
            for i in 0..n {
                if i == ti {
                    continue;
                }
                let carrier = team(w, ec, i).and_then(|t| t.flag).and_then(|f| {
                    match ops::root_parent(w, f) {
                        Some(EntityId::Mobile(m)) => Some(m),
                        _ => None,
                    }
                });
                let Some(carrier) = carrier.filter(|c| team_of(w, id, *c) == Some(ti)) else {
                    continue;
                };
                let defended = w.mobile(carrier).is_some_and(|c| {
                    c.aggressors
                        .iter()
                        .any(|a| a.defender == carrier && a.attacker == mob)
                });
                if defended && let Some(t) = team_mut(w, ec, ti) {
                    t.add_score(killer, 2); // helped defend guy capturing enemy flag
                }
                let near = muo_content::mx::map(w, mob) == muo_content::mx::map(w, carrier)
                    && muo_content::mx::in_range(w, carrier, mob, 12);
                if near && let Some(t) = team_mut(w, ec, ti) {
                    t.add_score(killer, 1); // helped defend guy capturing enemy flag
                }
            }
        }
    }

    display_board(w, id, mob);
    flow::requip(w, id, mob, corpse);
    games::delay_bounce(w, id, TimeSpan::from_secs(30), mob, corpse, false);
}

/// `CTFGame.OnStart`.
pub fn on_start(w: &mut World, id: u32, ec: ItemId) {
    let n = ctrl(w, ec).map_or(0, |c| c.teams.len());
    for i in 0..n {
        if let Some(t) = team_mut(w, ec, i) {
            t.game = Some(id);
            t.reset_scores();
        }
        // `Reset()`: the flag goes home in the team's colour, the board shows the team
        let (flag, board, color) = team(w, ec, i)
            .map(|t| (t.flag, t.board, t.color))
            .unwrap_or_default();
        if let Some(f) = flag {
            set_flag_link(w, f, Some((ec, i)));
            if let Some(ty) = w.item(f).map(|x| x.ty) {
                (ty.set_hue)(w, f, color);
            }
            send_home(w, f);
        }
        if let Some(b) = board {
            games::set_board_link(w, b, Some((ec, i)));
        }
    }
    let parts = ctx(w, id).map_or(0, |c| c.participants.len());
    for i in 0..parts {
        if let Some(color) = team(w, ec, i % 8).map(|t| t.color) {
            games::apply_hues(w, id, i, color);
        }
    }
    games::start_finish_timer(w, id, ec);
}

/// `CTFGame.OnStop`.
pub fn on_stop(w: &mut World, id: u32, ec: ItemId) {
    let n = ctrl(w, ec).map_or(0, |c| c.teams.len());
    for i in 0..n {
        let (flag, board) = team(w, ec, i)
            .map(|t| (t.flag, t.board))
            .unwrap_or_default();
        if let Some(f) = flag {
            send_home(w, f);
            set_flag_link(w, f, None);
        }
        if let Some(b) = board {
            games::set_board_link(w, b, None);
        }
        if let Some(t) = team_mut(w, ec, i) {
            t.game = None;
        }
    }
    let parts = ctx(w, id).map_or(0, |c| c.participants.len());
    for i in 0..parts {
        games::apply_hues(w, id, i, -1);
    }
}

/// `CTFGame.Initialize()`: the cookie stands' tiles (0x7C9..=0x7D0) block shots.
fn initialize(w: &mut World) {
    let items = &mut w.maps.tile_data.items;
    for i in 0x7C9..=0x7D0 {
        if let Some(t) = items.get_mut(i) {
            t.flags |= muo_data::TileFlag::NO_SHOOT;
        }
    }
}

/// Registers the classes.
pub fn register(types: &mut TypeRegistry) {
    types.register_item(&CTF_BOARD, &[]);
    types.register_item(&CTF_FLAG, &[]);
    types.register_item(&CTF_CONTROLLER, &[]);
    types.register_phase_default(
        Phase::Initialize,
        "Server.Engines.ConPVP.CTFGame.Initialize",
        initialize,
    );
}

#[allow(dead_code)]
fn _unused() {
    let _ = (duel_context_of, ctrl_mut);
}
