//! The rest of `DuelContext`: creating a duel, the countdowns, `Finish`, `Requip`,
//! `CheckCompletion`, sudden death, the automatic tie, `Unregister`, rematches, and what the players do around them
//! (`Debuff`, `CancelSpell`, `CreateWall`, `BuildParties`, `ClearIllegalItems`).
//!
//! `StartCountdown(count, cb)` runs with `Timer.StartTimer(1s, count, ..)`: the first callback runs at once and
//! reads `RemainingCount`, which is `count` for the first tick down to 1 for the last. A callback with 0 is never
//! delivered, so the 10-second begin countdown and the 3-second ready countdown never reach their
//! `count == 0` branch. `countdown_tick` keeps that exactly.

use muo_primitives::{MessageType, Point3D, TimeSpan};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::layer::Layer;
use muo_server::mobile::MobileDelta;
use muo_server::mobile::part2::messages as mm;
use muo_server::timer::{self, TimerHandle};

use super::{
    CountdownKind, DuelContext, DuelPlayer, Participant, arena, ctx, ctx_mut, duel_context_of,
    gumps, ladder, release_mobile, set_eliminated, st_mut, tournament,
};

/// The countdown never delivers a 0 (see the module comment), so a duel never gets past its ready countdown. Set this to run
/// one more tick, which delivers 0 and lets duels start; the standard behavior is `false`.
pub const COUNTDOWN_REACHES_ZERO: bool = false;

const COMBAT_DELAY: TimeSpan = TimeSpan::from_millis(30_000);
const AUTO_TIE_DELAY: TimeSpan = TimeSpan::from_millis(15 * 60_000);

/// `mob.Name`.
pub fn name(w: &World, m: MobileId) -> String {
    w.mobile(m)
        .and_then(|x| (x.ty.name)(w, m))
        .unwrap_or_default()
}

fn msg(w: &mut World, m: MobileId, text: &str) {
    mm::send_message(w, m, text);
}

fn msg_hue(w: &mut World, m: MobileId, hue: i32, text: &str) {
    mm::send_message_hue(w, m, hue, text);
}

/// `new DuelContext(initiator, layout)`: two participants of one, the initiator in the first. Returns the id.
pub fn new_context(w: &mut World, initiator: MobileId) -> u32 {
    let layout = super::ruleset::root(w);
    let mut ruleset = super::Ruleset::new(layout);
    ruleset.apply_default(0);
    let mut c = DuelContext::empty(initiator, ruleset);
    c.participants = vec![st_mut(w).new_participant(1), st_mut(w).new_participant(1)];
    let id = st_mut(w).add(c);
    super::set_slot(w, id, 0, 0, initiator);
    id
}

// ---- combat check ----------------------------------------------------------------------------------------------

/// `CheckCombat(m)`: a player fought them in the last 30 seconds.
pub fn check_combat(w: &World, m: MobileId) -> bool {
    let Some(mob) = w.mobile(m) else { return false };
    let is_player = |x: MobileId| w.mobile(x).is_some_and(|p| p.player);
    mob.aggressed
        .iter()
        .any(|i| is_player(i.defender) && w.now - i.last_combat_time < COMBAT_DELAY)
        || mob
            .aggressors
            .iter()
            .any(|i| is_player(i.attacker) && w.now - i.last_combat_time < COMBAT_DELAY)
}

// ---- death, requip, refresh -------------------------------------------------------------------------------------

/// `[OnEvent(PlayerDeathEvent)] OnPlayerDeath(m) => m.DuelContext?.OnDeath(m, m.Corpse)`.
pub fn on_player_death(w: &mut World, m: MobileId) {
    let Some(id) = duel_context_of(w, m) else {
        return;
    };
    let corpse = w.mobile(m).and_then(|x| x.corpse);
    on_death(w, id, m, corpse);
}

/// `OnDeath(mob, corpse)`.
pub fn on_death(w: &mut World, id: u32, mob: MobileId, corpse: Option<ItemId>) {
    let Some(c) = ctx(w, id) else { return };
    if !c.registered || !c.started {
        return;
    }
    if c.find(mob).is_none_or(|p| p.eliminated) {
        return;
    }
    if c.event_game.is_some() && !super::games::on_death(w, id, mob, corpse) {
        return;
    }
    set_eliminated(w, id, mob);

    if w.mobile(mob).is_some_and(|m| m.poison.is_some()) {
        muo_server::mobile::part1::props_b::set_poison(w, mob, None);
    }

    requip(w, id, mob, corpse);
    delay_bounce(w, id, TimeSpan::from_millis(4000), mob, corpse);

    if let Some(winner) = check_completion(w, id) {
        finish(w, id, winner);
    } else if !ctx(w, id).is_some_and(|c| c.yielding) {
        mm::local_overhead_message(
            w,
            mob,
            MessageType::REGULAR,
            0x22,
            false,
            "You have been defeated.",
        );
        let n = name(w, mob);
        mm::nonlocal_overhead_message(
            w,
            mob,
            MessageType::REGULAR,
            0x22,
            false,
            &format!("{n} has been defeated."),
        );
    }
}

/// `DelayBounce(ts, mob, corpse)`.
pub fn delay_bounce(w: &mut World, id: u32, ts: TimeSpan, mob: MobileId, corpse: Option<ItemId>) {
    timer::start(w, ts, TimeSpan::ZERO, 1, move |w| {
        delay_bounce_callback(w, id, mob, corpse)
    });
}

fn delay_bounce_callback(w: &mut World, id: u32, mob: MobileId, corpse: Option<ItemId>) {
    remove_aggressions(w, id, mob);
    send_outside(w, id, mob);
    refresh(w, mob, corpse);
    debuff(w, mob);
    cancel_spell(w, mob);
    set_frozen(w, mob, false);
}

fn set_frozen(w: &mut World, m: MobileId, v: bool) {
    muo_server::mobile::part1::props_a::set_frozen(w, m, v);
}

/// `CheckFull()`.
pub fn check_full(w: &World, id: u32) -> bool {
    ctx(w, id).is_some_and(|c| !c.participants.iter().any(|p| p.has_open_slot()))
}

/// `Requip(from, cont)`: the player takes what they can from the corpse.
pub fn requip(w: &mut World, id: u32, from: MobileId, cont: Option<ItemId>) {
    use muo_content::items::corpse;
    let Some(corpse_id) = cont.filter(|c| corpse::is_corpse(w, *c)) else {
        return;
    };
    let items = w
        .item(corpse_id)
        .map(|c| c.items.clone())
        .unwrap_or_default();
    let mut didnt_fit = false;
    let pack = w.mobile(from).and_then(|m| m.backpack);

    let mut i = 0;
    while !didnt_fit && i < items.len() {
        let item = items[i];
        i += 1;
        let Some(it) = w.item(item) else { continue };
        if matches!(it.layer, Layer::Hair | Layer::FacialHair) || !it.movable() {
            continue;
        }
        match pack {
            Some(p) => {
                let ty = w.item(p).map(|x| x.ty);
                if let Some(ty) = ty {
                    (ty.drop_item)(w, p, item);
                }
            }
            None => didnt_fit = true,
        }
    }

    corpse::set_flag(w, corpse_id, corpse::flag::CARVED, true);

    if w.item(corpse_id).is_some_and(|c| c.item_id == 0x2006) {
        if let Some(ty) = w.item(corpse_id).map(|c| c.ty) {
            (ty.process_delta)(w, corpse_id);
            muo_server::item::ops::send_remove_packet(w, corpse_id);
            let g = muo_primitives::utility::random_from(&mut w.rng, 0xECA, 9); // bone graphic
            (ty.set_item_id)(w, corpse_id, g);
            (ty.set_hue)(w, corpse_id, 0);
            (ty.process_delta)(w, corpse_id);
        }
        let killer = muo_server::mobile::part2::damage::find_most_recent_damager(w, from, false);
        if killer.is_some_and(|k| w.mobile(k).is_some_and(|m| m.player)) {
            let head_type = if ctx(w, id).is_some_and(|c| c.tournament.is_none()) {
                1
            } else {
                2
            };
            let head = muo_items::body_parts::parts::new_head(w, head_type, Some(name(w, from)));
            if let Some(k) = killer {
                muo_server::mobile::part2::items::add_to_backpack(w, k, head);
            }
        }
    }

    muo_server::mobile::part2::effects::send_sound(w, from, 0x3E3);
    if didnt_fit {
        mm::send_localized_message(w, from, 1062472, "", 0x3B2); // You gather some of your belongings. The rest remain on the corpse.
    } else {
        mm::send_localized_message(w, from, 1062471, "", 0x3B2); // You quickly gather all of your belongings.
    }
}

/// `Refresh(mob, cont)`.
pub fn refresh(w: &mut World, mob: MobileId, cont: Option<ItemId>) {
    use muo_content::items::corpse;
    if !muo_content::mx::alive(w, mob) {
        if let Some(ty) = w.mobile(mob).map(|m| m.ty) {
            (ty.resurrect)(w, mob);
        }
        if let Some(robe) =
            muo_server::mobile::part2::items::find_item_on_layer(w, mob, Layer::OuterTorso)
            && muo_content::factory::item_is(w, robe, "Server.Items.DeathRobe")
            && let Some(ty) = w.item(robe).map(|i| i.ty)
        {
            (ty.delete)(w, robe);
        }
        if let Some(c) = cont.filter(|c| corpse::is_corpse(w, *c)) {
            let pack = w.mobile(mob).and_then(|m| m.backpack);
            for item in corpse::equip_items(w, c) {
                let Some(it) = w.item(item) else { continue };
                if it.movable()
                    && it.layer != Layer::Hair
                    && it.layer != Layer::FacialHair
                    && pack.is_some_and(|p| is_child_of(w, item, p))
                    && let Some(ty) = w.mobile(mob).map(|m| m.ty)
                {
                    (ty.equip_item)(w, mob, item);
                }
            }
        }
    }
    restore_vitals(w, mob);
}

/// `item.IsChildOf(parent)`.
fn is_child_of(w: &World, item: ItemId, parent: ItemId) -> bool {
    muo_server::item::ops::is_child_of(w, item, Some(muo_server::ids::EntityId::from(parent)))
}

fn restore_vitals(w: &mut World, mob: MobileId) {
    use muo_server::mobile::part1::props_b;
    let Some(ty) = w.mobile(mob).map(|m| m.ty) else {
        return;
    };
    let (h, s, m) = (
        (ty.hits_max)(w, mob),
        (ty.stam_max)(w, mob),
        (ty.mana_max)(w, mob),
    );
    props_b::set_hits(w, mob, h);
    props_b::set_stam(w, mob, s);
    props_b::set_mana(w, mob, m);
    props_b::set_poison(w, mob, None);
}

/// `SendOutside(mob)`.
pub fn send_outside(w: &mut World, id: u32, mob: MobileId) {
    let Some(a) = ctx(w, id)
        .and_then(|c| c.arena)
        .and_then(|a| arena::data(w, a))
    else {
        return;
    };
    let (facet, outside) = (a.facet, a.outside);
    muo_content::mx::set_combatant(w, mob, None);
    if let (Some(ty), Some(map)) = (w.mobile(mob).map(|m| m.ty), facet) {
        (ty.move_to_world)(w, mob, outside, map);
    }
}

// ---- finish -----------------------------------------------------------------------------------------------------

/// `Broadcast` of a `Participant`: `{0}` is the player's name, `{1}` his/her.
fn broadcast(w: &mut World, p: &Participant, hue: i32, non_local: Option<&str>, local: &str) {
    for pl in p.players.iter().flatten() {
        let m = pl.mobile;
        if let Some(t) = non_local {
            let n = name(w, m);
            let text = t.replace("{0}", &n).replace(
                "{1}",
                if w.mobile(m).is_some_and(|x| x.female) {
                    "her"
                } else {
                    "his"
                },
            );
            mm::nonlocal_overhead_message(w, m, MessageType::REGULAR, hue, false, &text);
        }
        mm::local_overhead_message(w, m, MessageType::REGULAR, hue, false, local);
    }
}

/// `Participant.Broadcast(hue, message, nonLocalOverhead, localOverhead)` for the duel messages: the non-local text
/// formats the *first* two arguments as the player's name and `his`/`her`, as `string.Format` does per player.
fn broadcast_duel(w: &mut World, p: &Participant, hue: i32, non_local: &str, local: &str) {
    broadcast(w, p, hue, Some(non_local), local);
}

/// `Finish(winner)`.
pub fn finish(w: &mut World, id: u32, winner: usize) {
    let Some(c) = ctx(w, id) else { return };
    if c.finished {
        return;
    }
    cancel_timer(w, id, |c| &mut c.auto_tie_timer);
    stop_sd_timers(w, id);
    if let Some(c) = ctx_mut(w, id) {
        c.finished = true;
    }
    let c = ctx(w, id).expect("context").clone();
    let Some(win) = c.participants.get(winner) else {
        return;
    };

    for pl in win.players.iter().flatten() {
        if !pl.eliminated {
            delay_bounce(w, id, TimeSpan::from_millis(8000), pl.mobile, None);
        }
    }

    let (nl, l) = if win.count() == 1 {
        ("{0} has won the duel.", "You have won the duel.")
    } else {
        (
            "{0} and {1} team have won the duel.",
            "Your team has won the duel.",
        )
    };
    broadcast_duel(w, win, 0x59, nl, l);

    if let (Some(ctrl), Some(m), Some(tp)) = (c.tournament_ctrl, c.tmatch, win.tourney_part) {
        tournament::set_winner(w, ctrl, m, tp);
        tournament::won_match(w, ctrl, tp);
        tournament::handle_won(w, ctrl, c.arena, m, tp);
    }

    for (i, loser) in c.participants.iter().enumerate() {
        if i != winner {
            let (nl, l) = if loser.count() == 1 {
                ("{0} has lost the duel.", "You have lost the duel.")
            } else {
                (
                    "{0} and {1} team have lost the duel.",
                    "Your team has lost the duel.",
                )
            };
            broadcast_duel(w, loser, 0x22, nl, l);
            if let (Some(ctrl), Some(tp)) = (c.tournament_ctrl, loser.tourney_part) {
                tournament::lost_match(w, ctrl, tp);
            }
        }
        for pl in loser.players.iter().flatten() {
            let m = pl.mobile;
            remove_aggressions(w, id, m);
            if let Some(ty) = w.mobile(m).map(|x| x.ty) {
                (ty.delta)(w, m, MobileDelta::NOTO);
            }
            gumps::close_begin(w, m);
            if c.tournament.is_some() {
                muo_server::mobile::part2::effects::send_everything(w, m);
            }
        }
    }

    if c.is_one_vs_one()
        && let (Some(dp1), Some(dp2)) = (c.participants[0].players[0], c.participants[1].players[0])
    {
        award(w, id, dp1.mobile, dp2.mobile, winner == 0);
        award(w, id, dp2.mobile, dp1.mobile, winner == 1);
    }

    super::games::on_stop(w, id);

    timer::start(
        w,
        TimeSpan::from_millis(9000),
        TimeSpan::ZERO,
        1,
        move |w| unregister(w, id, true),
    );
}

/// `Award(us, them, won)`.
pub fn award(w: &mut World, id: u32, us: MobileId, them: MobileId, won: bool) {
    let Some(c) = ctx(w, id) else { return };
    let tournament = c.tournament.is_some();
    let ladder_item = match c.arena {
        None => ladder::instance(w),
        Some(a) => arena::acquire_ladder(w, a),
    };
    let Some(lad) = ladder_item else { return };

    let (Some(ours), Some(theirs)) = (ladder::find(w, lad, us), ladder::find(w, lad, them)) else {
        return;
    };
    let mut xp_gain = ladder::get_experience_gain(Some(&ours), Some(&theirs), won);
    if xp_gain == 0 {
        return;
    }
    if tournament {
        xp_gain *= if xp_gain > 0 { 5 } else { 2 };
    }

    let old_level = ladder::get_level(ours.experience);
    let new_xp = {
        let Some(e) = ladder::entry_mut(w, lad, us) else {
            return;
        };
        if won {
            e.wins += 1;
        } else {
            e.losses += 1;
        }
        e.experience += xp_gain;
        if e.experience < 0 {
            e.experience = 0;
        }
        e.experience
    };
    ladder::update_entry(w, lad, us);

    let new_level = ladder::get_level(new_xp);
    if new_level > old_level {
        msg_hue(
            w,
            us,
            0x59,
            &format!("You have achieved level {new_level}!"),
        );
    } else if new_level < old_level {
        msg_hue(
            w,
            us,
            0x22,
            &format!("You have lost a level. You are now at {new_level}."),
        );
    }
}

// ---- unregister, rematch ----------------------------------------------------------------------------------------

/// `Unregister(queryRematch)`.
pub fn unregister(w: &mut World, id: u32, query_rematch: bool) {
    destroy_wall(w, id);
    let Some(c) = ctx(w, id) else { return };
    if !c.registered {
        return;
    }
    let arena_id = c.arena;
    if let Some(c) = ctx_mut(w, id) {
        c.registered = false;
    }
    if let Some(a) = arena_id {
        arena::evict(w, a);
    }
    stop_sd_timers(w, id);

    let players = ctx(w, id).map(|c| c.all_players()).unwrap_or_default();
    for m in players {
        release_mobile(w, m);
        gumps::close_all_gumps_of(w, m);
    }

    if query_rematch && ctx(w, id).is_some_and(|c| c.tournament.is_none()) {
        query_rematch_for(w, id);
    }
}

/// `QueryRematch()`.
pub fn query_rematch_for(w: &mut World, id: u32) {
    let Some(old) = ctx(w, id).cloned() else {
        return;
    };
    let mut dc = DuelContext::empty(old.initiator, old.ruleset.clone());
    dc.rematch = true;
    for old_part in &old.participants {
        let mut np = st_mut(w).new_participant(old_part.count());
        for (j, p) in old_part.players.iter().enumerate() {
            if let Some(p) = p {
                np.players[j] = Some(DuelPlayer::new(p.mobile));
            }
        }
        dc.participants.push(np);
    }
    let nid = st_mut(w).add(dc);
    close_all_gumps(w, nid);
    send_ready_up_gump(w, nid);
}

/// `CheckCompletion()`: the participant that won, if the others are eliminated.
pub fn check_completion(w: &World, id: u32) -> Option<usize> {
    let c = ctx(w, id)?;
    let mut winner = None;
    let mut has_winner = false;
    let mut eliminated = 0;
    for (i, p) in c.participants.iter().enumerate() {
        if p.eliminated() {
            eliminated += 1;
            if eliminated == c.participants.len() - 1 {
                has_winner = true;
            }
        } else {
            winner = Some(i);
        }
    }
    has_winner.then(|| winner.unwrap_or(0))
}

// ---- countdowns -------------------------------------------------------------------------------------------------

fn cancel_timer(w: &mut World, id: u32, pick: fn(&mut DuelContext) -> &mut TimerHandle) {
    let mut h = match ctx_mut(w, id) {
        Some(c) => std::mem::take(pick(c)),
        None => return,
    };
    h.cancel(w);
}

/// `StartCountdown(count, cb)`.
pub fn start_countdown(w: &mut World, id: u32, count: i32, kind: CountdownKind) {
    let ticks = if COUNTDOWN_REACHES_ZERO {
        count + 1
    } else {
        count
    };
    let h = timer::start(
        w,
        TimeSpan::ZERO,
        TimeSpan::from_millis(1000),
        ticks,
        move |w| countdown_tick(w, id, kind),
    );
    if let Some(c) = ctx_mut(w, id) {
        c.countdown_timer = h;
    }
}

/// `StopCountdown()`.
pub fn stop_countdown(w: &mut World, id: u32) {
    cancel_timer(w, id, |c| &mut c.countdown_timer);
}

/// `Countdown_Callback(cb)`: `RemainingCount` is read from the timer, so a callback never sees 0.
fn countdown_tick(w: &mut World, id: u32, kind: CountdownKind) {
    let Some(h) = ctx(w, id).map(|c| c.countdown_timer) else {
        return;
    };
    let count = h.remaining_count(w) - i32::from(COUNTDOWN_REACHES_ZERO);
    if count == 0 {
        stop_countdown(w, id);
    }
    match kind {
        CountdownKind::Ready => send_ready_gump_count(w, id, count),
        CountdownKind::Begin => send_begin_gump(w, id, count),
    }
}

/// `StopSDTimers()`.
pub fn stop_sd_timers(w: &mut World, id: u32) {
    cancel_timer(w, id, |c| &mut c.sd_warn_timer);
    cancel_timer(w, id, |c| &mut c.sd_activate_timer);
}

/// `StartSuddenDeath(timeUntilActive)`.
pub fn start_sudden_death(w: &mut World, id: u32, until_active: TimeSpan) {
    cancel_timer(w, id, |c| &mut c.sd_warn_timer);
    let warn = timer::start(
        w,
        TimeSpan::from_millis((until_active.total_milliseconds() * 0.9) as i64),
        TimeSpan::ZERO,
        1,
        move |w| warn_sudden_death(w, id),
    );
    if let Some(c) = ctx_mut(w, id) {
        c.sd_warn_timer = warn;
    }
    cancel_timer(w, id, |c| &mut c.sd_activate_timer);
    let act = timer::start(w, until_active, TimeSpan::ZERO, 1, move |w| {
        activate_sudden_death(w, id)
    });
    if let Some(c) = ctx_mut(w, id) {
        c.sd_activate_timer = act;
    }
}

fn live_players(w: &World, id: u32) -> Vec<MobileId> {
    ctx(w, id).map_or_else(Vec::new, |c| {
        c.participants
            .iter()
            .flat_map(|p| p.players.iter().flatten())
            .filter(|p| !p.eliminated)
            .map(|p| p.mobile)
            .collect()
    })
}

/// `WarnSuddenDeath()`.
pub fn warn_sudden_death(w: &mut World, id: u32) {
    for m in live_players(w, id) {
        muo_server::mobile::part2::effects::send_sound(w, m, 0x1E1);
        msg_hue(w, m, 0x22, "Warning! Warning! Warning!");
        msg_hue(w, m, 0x22, "Sudden death will be active soon!");
    }
    if let Some(c) = ctx(w, id)
        && let Some(ctrl) = c.tournament_ctrl
    {
        let a = c.arena;
        tournament::alert_arena(w, a, &["Sudden death will be active soon!"]);
        let _ = ctrl;
    }
    cancel_timer(w, id, |c| &mut c.sd_warn_timer);
}

/// `ActivateSuddenDeath()`.
pub fn activate_sudden_death(w: &mut World, id: u32) {
    for m in live_players(w, id) {
        muo_server::mobile::part2::effects::send_sound(w, m, 0x1E1);
        msg_hue(w, m, 0x22, "Warning! Warning! Warning!");
        msg_hue(
            w,
            m,
            0x22,
            "Sudden death has ACTIVATED. You are now unable to perform any beneficial actions.",
        );
    }
    if let Some(c) = ctx(w, id)
        && c.tournament_ctrl.is_some()
    {
        let a = c.arena;
        tournament::alert_arena(w, a, &["Sudden death has been activated!"]);
    }
    if let Some(c) = ctx_mut(w, id) {
        c.is_sudden_death = true;
    }
    cancel_timer(w, id, |c| &mut c.sd_activate_timer);
}

/// `BeginAutoTie()`.
pub fn begin_auto_tie(w: &mut World, id: u32) {
    let standard = ctx(w, id)
        .and_then(|c| c.tournament_ctrl)
        .and_then(|t| tournament::get(w, t))
        .is_none_or(|t| t.tourney_type == tournament::TourneyType::Standard);
    let ts = if standard {
        AUTO_TIE_DELAY
    } else {
        TimeSpan::from_mins(90)
    };
    cancel_timer(w, id, |c| &mut c.auto_tie_timer);
    let h = timer::start(w, ts, TimeSpan::ZERO, 1, move |w| invoke_auto_tie(w, id));
    if let Some(c) = ctx_mut(w, id) {
        c.auto_tie_timer = h;
    }
}

/// `InvokeAutoTie()`.
pub fn invoke_auto_tie(w: &mut World, id: u32) {
    cancel_timer(w, id, |c| &mut c.auto_tie_timer);
    let Some(c) = ctx(w, id) else { return };
    if !c.started || c.finished {
        return;
    }
    if let Some(c) = ctx_mut(w, id) {
        c.tied = true;
        c.finished = true;
    }
    stop_sd_timers(w, id);

    let c = ctx(w, id).expect("context").clone();
    let mut remaining: Vec<usize> = Vec::new();
    for p in &c.participants {
        if p.eliminated() {
            let (nl, l) = if p.count() == 1 {
                ("{0} has lost the duel.", "You have lost the duel.")
            } else {
                (
                    "{0} and {1} team have lost the duel.",
                    "Your team has lost the duel.",
                )
            };
            broadcast_duel(w, p, 0x22, nl, l);
        } else {
            let (nl, l) = if p.count() == 1 {
                (
                    "{0} has tied the duel due to time expiration.",
                    "You have tied the duel due to time expiration.",
                )
            } else {
                (
                    "{0} and {1} team have tied the duel due to time expiration.",
                    "Your team has tied the duel due to time expiration.",
                )
            };
            broadcast_duel(w, p, 0x59, nl, l);
            for pl in p.players.iter().flatten() {
                if !pl.eliminated {
                    delay_bounce(w, id, TimeSpan::from_millis(8000), pl.mobile, None);
                }
            }
            if let Some(tp) = p.tourney_part {
                remaining.push(tp);
            }
        }
        for pl in p.players.iter().flatten() {
            if let Some(ty) = w.mobile(pl.mobile).map(|x| x.ty) {
                (ty.delta)(w, pl.mobile, MobileDelta::NOTO);
            }
            muo_server::mobile::part2::effects::send_everything(w, pl.mobile);
        }
    }
    if let (Some(ctrl), Some(m)) = (c.tournament_ctrl, c.tmatch) {
        tournament::handle_tie(w, ctrl, c.arena, m, &remaining);
    }

    timer::start(
        w,
        TimeSpan::from_millis(10_000),
        TimeSpan::ZERO,
        1,
        move |w| unregister(w, id, false),
    );
}

// ---- gumps around the flow ----------------------------------------------------------------------------------------

/// `CloseAllGumps()`.
pub fn close_all_gumps(w: &mut World, id: u32) {
    for m in ctx(w, id).map(|c| c.all_players()).unwrap_or_default() {
        gumps::close_all_gumps_of(w, m);
    }
}

/// `RejectReady(rejector, page)`: `page` `None` is a yield.
pub fn reject_ready(w: &mut World, id: u32, rejector: MobileId, page: Option<&str>) {
    let Some(c) = ctx(w, id) else { return };
    if c.started_ready_countdown {
        return; // sanity
    }
    let rematch = c.rematch;
    let players = c.all_players();
    if let Some(c) = ctx_mut(w, id) {
        for p in c
            .participants
            .iter_mut()
            .flat_map(|p| p.players.iter_mut())
            .flatten()
        {
            p.ready = false;
        }
    }
    let rname = name(w, rejector);
    for mob in players {
        match page {
            None => {
                if mob != rejector {
                    msg_hue(w, mob, 0x22, &format!("{rname} has yielded."));
                }
            }
            Some(page) => {
                let text = if mob == rejector {
                    if rematch {
                        "You have rejected the rematch.".to_owned()
                    } else {
                        format!("You have rejected the {page}.")
                    }
                } else if rematch {
                    format!("{rname} has rejected the rematch.")
                } else {
                    format!("{rname} has rejected the {page}.")
                };
                msg_hue(w, mob, 0x22, &text);
            }
        }
        gumps::close_reject_gumps(w, mob);
    }

    let (rematch, yielding, initiator) = {
        let c = ctx(w, id).expect("context");
        (c.rematch, c.yielding, c.initiator)
    };
    if rematch {
        unregister(w, id, false);
    } else if !yielding {
        gumps::display_duel_context(w, initiator, id);
    }
    if let Some(c) = ctx_mut(w, id) {
        c.ready_wait = false;
        c.ready_count = 0;
    }
}

// ---- Debuff / CancelSpell -----------------------------------------------------------------------------------------

/// `Debuff(mob)`.
pub fn debuff(w: &mut World, mob: MobileId) {
    use muo_server::mobile::base as mb;
    use muo_server::mobile::part1::{combat, props_a, props_b};

    for n in [
        "[Magic] Str Buff",
        "[Magic] Dex Buff",
        "[Magic] Int Buff",
        "[Magic] Str Curse",
        "[Magic] Dex Curse",
        "[Magic] Int Curse",
        "Concussion",
        "blood-rose",
        "clarity-potion",
        "RoseOfTrinsicPetal",
        "Holy Bless",
        "Holy Curse",
    ] {
        mb::remove_stat_mod_by_name(w, mob, n);
    }

    crate::plants::misc::orange_petals_remove_effect(w, mob);

    mb::set_paralyzed(w, mob, false);
    props_b::set_hidden(w, mob, false);

    if !muo_content::core::aos(w) {
        if !muo_content::core::uor(w) {
            muo_spells::first::reactive_armor::remove_effect(w, mob);
            muo_spells::second::protection::remove_t2a_protection(w, mob);
        }
        if let Some(m) = w.mobile_mut(mob) {
            m.magic_damage_absorb = 0;
            m.melee_damage_absorb = 0;
        }
        muo_spells::second::protection::registry_remove(w, mob);
        muo_spells::fourth::arch_protection::remove_entry(w, mob);
        combat::end_action(w, mob, "DefensiveSpell");
    }

    muo_content::spells::helper::remove_context(w, mob, true);
    muo_spells::ninjitsu::animal_form::remove_context(w, mob);
    if let Some(f) = muo_content::spells::hooks::hooks(w).and_then(|h| h.disguise_stop_timer) {
        f(w, mob);
    }

    if !combat::can_begin_action(w, mob, muo_spells::seventh::polymorph::KEY) {
        props_b::set_body_mod(w, mob, muo_primitives::Body::default());
        props_b::set_hue_mod(w, mob, -1);
        combat::end_action(w, mob, muo_spells::seventh::polymorph::KEY);
    }

    muo_content::items::armor::validate_mobile(w, mob);
    muo_content::items::clothing::validate_mobile(w, mob);
    let _ = props_a::set_frozen;
    restore_vitals(w, mob);
}

/// `CancelSpell(mob)`.
pub fn cancel_spell(w: &mut World, mob: MobileId) {
    use muo_content::spells::{spell, types::DisturbType};
    if let Some(s) = w.mobile(mob).and_then(|m| m.spell) {
        spell::disturb(w, s, DisturbType::Kill, true, false);
    }
    muo_server::target::cancel_mobile_target(w, mob);
}

// ---- walls, parties, illegal items ------------------------------------------------------------------------------------

/// `DestroyWall()`.
pub fn destroy_wall(w: &mut World, id: u32) {
    let walls = ctx_mut(w, id)
        .map(|c| std::mem::take(&mut c.walls))
        .unwrap_or_default();
    for item in walls {
        if let Some(ty) = w.item(item).map(|i| i.ty) {
            (ty.delete)(w, item);
        }
    }
}

/// `CreateWall()`.
pub fn create_wall(w: &mut World, id: u32) {
    let Some(a) = ctx(w, id)
        .and_then(|c| c.arena)
        .and_then(|a| arena::data(w, a))
    else {
        return;
    };
    let (start, wall, facet) = (a.points[0], a.wall, a.facet);
    let (dx, dy) = (start.x - wall.x, start.y - wall.y);
    let (rx, ry) = (dx - dy, dx + dy);
    let east_to_west = rx == 0 && ry >= 0 || rx >= 0 && ry == 0;

    muo_server::effects::play_sound(w, wall, facet, 0x1F6);

    for i in -1..=1 {
        let loc = Point3D::new(
            if east_to_west { wall.x + i } else { wall.x },
            if east_to_west { wall.y } else { wall.y + i },
            wall.z,
        );
        let Some(created) = muo_content::factory::make_item(w, arena::INTERNAL_WALL_NAME) else {
            continue;
        };
        // `Appear`: `MoveToWorld(loc, map); Effects.SendLocationParticles(this, 0x376A, 9, 10, 5025)`.
        muo_server::item::ops::move_to_world_map(w, created, loc, facet);
        muo_server::effects::send_location_particles(
            w,
            muo_server::ids::EntityId::from(created),
            0x376A,
            9,
            10,
            5025,
        );
        if let Some(c) = ctx_mut(w, id) {
            c.walls.push(created);
        }
    }
}

/// `BuildParties()`: a team of players share a party.
pub fn build_parties(w: &mut World, id: u32) {
    use crate::party;
    let Some(c) = ctx(w, id) else { return };
    let teams: Vec<Vec<MobileId>> = c
        .participants
        .iter()
        .filter(|p| p.count() > 1)
        .map(|p| p.players.iter().flatten().map(|d| d.mobile).collect())
        .collect();
    for players in teams {
        if players.len() <= 1 {
            continue;
        }
        let mut leader_index = 0;
        while leader_index + 1 < players.len() {
            let leader = players[leader_index];
            let party = match party::get(w, leader) {
                None => {
                    let p = party::new_party(w, leader);
                    set_party(w, leader, Some(p));
                    p
                }
                Some(p) if party::state(w, p).is_some_and(|s| s.leader != leader) => {
                    party::send_public_message(
                        w,
                        p,
                        leader,
                        "I leave this party to fight in a duel.",
                    );
                    party::remove(w, p, leader);
                    let np = party::new_party(w, leader);
                    set_party(w, leader, Some(np));
                    np
                }
                Some(p) => p,
            };
            let mut j = leader_index + 1;
            while j < players.len() && j < leader_index + party::CAPACITY {
                let player = players[j];
                j += 1;
                let existing = party::get(w, player);
                if existing == Some(party) {
                    continue;
                }
                let full = party::state(w, party)
                    .is_some_and(|s| s.members.len() + s.candidates.len() >= party::CAPACITY);
                if full {
                    msg(
                        w,
                        player,
                        "You could not be added to the team party because it is at full capacity.",
                    );
                    // "{0}" is deliberately left unformatted in this message.
                    msg(
                        w,
                        leader,
                        "{0} could not be added to the team party because it is at full capacity.",
                    );
                } else {
                    if let Some(e) = existing {
                        party::send_public_message(
                            w,
                            e,
                            player,
                            "I leave this party to fight in a duel.",
                        );
                        party::remove(w, e, player);
                    }
                    party::on_accept(w, party, player, true);
                }
            }
            leader_index += party::CAPACITY;
        }
    }
}

fn set_party(w: &mut World, m: MobileId, p: Option<muo_server::interfaces::PartyId>) {
    if let Some(x) = w.mobile_mut(m) {
        x.party = p;
    }
}

/// `ClearIllegalItems()`.
pub fn clear_illegal_items(w: &mut World, id: u32) {
    for m in ctx(w, id).map(|c| c.all_players()).unwrap_or_default() {
        clear_illegal_items_of(w, id, m);
    }
}

/// `ClearIllegalItems(mob)`.
pub fn clear_illegal_items_of(w: &mut World, id: u32, mob: MobileId) {
    use super::allow_special_ability;
    if w.mobile(mob).is_some_and(|m| m.stun_ready)
        && !allow_special_ability(w, mob, "Stun", false)
        && let Some(m) = w.mobile_mut(mob)
    {
        m.stun_ready = false;
    }
    if w.mobile(mob).is_some_and(|m| m.disarm_ready)
        && !allow_special_ability(w, mob, "Disarm", false)
        && let Some(m) = w.mobile_mut(mob)
    {
        m.disarm_ready = false;
    }

    let Some(pack) = w.mobile(mob).and_then(|m| m.backpack) else {
        return;
    };
    let items: Vec<ItemId> = w.mobile(mob).map(|m| m.items.clone()).unwrap_or_default();
    for item in items.into_iter().rev() {
        let illegal = {
            let (Some(c), true) = (ctx(w, id), w.item(item).is_some()) else {
                continue;
            };
            !super::check_item_equip(w, c, item)
        };
        if illegal {
            if let Some(ty) = w.item(pack).map(|i| i.ty) {
                (ty.drop_item)(w, pack, item);
            }
            let (number, label) = {
                let is_weapon = w
                    .item(item)
                    .is_some_and(|i| i.ty.is(&muo_content::items::weapon::BASE_WEAPON));
                let is_shield = w
                    .item(item)
                    .is_some_and(|i| i.ty.is(&muo_content::items::armor::BASE_SHIELD));
                let is_armor_or_clothing = w.item(item).is_some_and(|i| {
                    i.ty.is(&muo_content::items::armor::BASE_ARMOR)
                        || muo_content::factory::item_is(w, item, "Server.Items.BaseClothing")
                });
                let number = if is_weapon {
                    1062001 // You can no longer wield your ~1_WEAPON~
                } else if !is_shield && is_armor_or_clothing {
                    1062002 // You can no longer wear your ~1_ARMOR~
                } else {
                    1062003 // You can no longer equip your ~1_SHIELD~
                };
                let label = w
                    .item(item)
                    .and_then(|i| i.name.clone())
                    .unwrap_or_else(|| {
                        format!(
                            "#{}",
                            (w.item(item).map_or(0, |i| (i.ty.label_number)(w, item)))
                        )
                    });
                (number, label)
            };
            mm::send_localized_message(w, mob, number, &label, 0x3B2);
        }
    }

    let in_hand = w.mobile(mob).and_then(|m| m.holding);
    if let Some(in_hand) = in_hand
        && w.item(in_hand).is_some()
        && ctx(w, id).is_some_and(|c| !super::check_item_equip(w, c, in_hand))
    {
        muo_server::mobile::part1::props_a::set_holding(w, mob, None);
        let from_mob = w
            .item(in_hand)
            .and_then(|i| i.bounce.as_ref())
            .is_some_and(|b| b.parent == Some(muo_server::ids::EntityId::Mobile(mob)));
        if from_mob {
            if let Some(ty) = w.item(pack).map(|i| i.ty) {
                (ty.drop_item)(w, pack, in_hand);
            }
        } else {
            muo_server::item::ops::bounce(w, in_hand, mob);
        }
        muo_server::item::ops::clear_bounce(w, in_hand);
    }
}

// ---- begin / ready ---------------------------------------------------------------------------------------------------

/// `RemoveAggressions(mob)`.
pub fn remove_aggressions(w: &mut World, id: u32, mob: MobileId) {
    use muo_server::mobile::part1::combat;
    for other in ctx(w, id).map(|c| c.all_players()).unwrap_or_default() {
        if other == mob {
            continue;
        }
        combat::remove_aggressed(w, mob, other);
        combat::remove_aggressor(w, mob, other);
        combat::remove_aggressed(w, other, mob);
        combat::remove_aggressor(w, other, mob);
    }
}

/// `SendBeginGump(count)`.
pub fn send_begin_gump(w: &mut World, id: u32, count: i32) {
    let Some(c) = ctx(w, id) else { return };
    if !c.registered || c.finished {
        return;
    }
    if count == 10 {
        create_wall(w, id);
        build_parties(w, id);
        clear_illegal_items(w, id);
    } else if count == 0 {
        destroy_wall(w, id);
    }
    if let Some(c) = ctx_mut(w, id) {
        c.started_begin_countdown = true;
    }
    if count == 0 {
        if let Some(c) = ctx_mut(w, id) {
            c.started = true;
        }
        begin_auto_tie(w, id);
    }
    for mob in ctx(w, id).map(|c| c.all_players()).unwrap_or_default() {
        if count > 0 {
            if count == 10 {
                gumps::close_before_begin(w, mob);
                gumps::display_begin(w, mob);
            }
            set_frozen(w, mob, true);
        } else {
            gumps::close_begin(w, mob);
            set_frozen(w, mob, false);
        }
    }
}

/// `SendReadyUpGump()`.
pub fn send_ready_up_gump(w: &mut World, id: u32) {
    let Some(c) = ctx_mut(w, id) else { return };
    if !c.registered {
        return;
    }
    c.ready_wait = true;
    c.ready_count = -1;
    let tournament = c.tournament.is_some();
    for m in c.all_players() {
        if !tournament {
            gumps::display_ready_up(w, m, id);
        }
    }
}

/// `ValidateStart()`: why the duel cannot start, if it cannot.
pub fn validate_start(w: &mut World, id: u32) -> Option<String> {
    let c = ctx(w, id)?;
    let tournament = c.tournament.is_some();
    if !tournament && tournament::is_active(w) {
        return Some("a tournament is active".to_owned());
    }
    let slots: Vec<Option<MobileId>> = c
        .participants
        .iter()
        .flat_map(|p| p.players.iter().map(|s| s.map(|d| d.mobile)))
        .collect();
    for slot in slots {
        let Some(dp) = slot else {
            return Some("a slot is empty".to_owned());
        };
        let n = name(w, dp);
        let region = w.mobile(dp).map(|m| m.region);
        if region.is_some_and(|r| {
            muo_server::region::is_part_of_type(w, r, &muo_content::jail::JAIL_REGION)
        }) {
            return Some(format!("{n} is in jail"));
        }
        if muo_content::spells::hooks::hooks(w)
            .and_then(|h| h.sigil_exists_on)
            .is_some_and(|f| f(w, dp))
        {
            return Some(format!("{n} is holding a sigil"));
        }
        if !muo_content::mx::alive(w, dp) {
            if !tournament {
                return Some(format!("{n} is dead"));
            }
            if let Some(ty) = w.mobile(dp).map(|m| m.ty) {
                (ty.resurrect)(w, dp);
            }
        }
        if !tournament && check_combat(w, dp) {
            return Some(format!("{n} is in combat"));
        }
        if let Some(mount) = muo_server::mobile::part1::props_b::mount(w, dp) {
            if tournament {
                muo_content::creature::mount::set_rider(w, mount, None);
            } else {
                return Some(format!("{n} is mounted"));
            }
        }
    }
    None
}

/// `SendReadyGump()`.
pub fn send_ready_gump(w: &mut World, id: u32) {
    send_ready_gump_count(w, id, -1);
}

/// `SendReadyGump(count)`.
pub fn send_ready_gump_count(w: &mut World, id: u32, count: i32) {
    let Some(c) = ctx(w, id) else { return };
    if !c.registered {
        return;
    }
    if count != -1
        && let Some(c) = ctx_mut(w, id)
    {
        c.started_ready_countdown = true;
    }
    if let Some(c) = ctx_mut(w, id) {
        c.ready_count = count;
    }

    if count == 0 {
        begin_duel(w, id);
        return;
    }

    if let Some(c) = ctx_mut(w, id) {
        c.ready_wait = true;
    }
    let c = ctx(w, id).expect("context");
    let tournament = c.tournament.is_some();
    let mut is_all_ready = true;
    let mut show = Vec::new();
    for p in c
        .participants
        .iter()
        .flat_map(|p| p.players.iter().flatten())
    {
        if p.ready {
            if !tournament {
                show.push(p.mobile);
            }
        } else {
            is_all_ready = false;
        }
    }
    for m in show {
        gumps::display_ready(w, m, id, count);
    }
    if count == -1 && is_all_ready {
        start_countdown(w, id, 3, CountdownKind::Ready);
    }
}

/// The `count == 0` branch of `SendReadyGump(count)`: pick an arena and send everyone in.
fn begin_duel(w: &mut World, id: u32) {
    let players_all = ctx(w, id).map(|c| c.all_players()).unwrap_or_default();
    let tell_all = |w: &mut World, text: &str| {
        for m in &players_all {
            msg(w, *m, text);
        }
    };

    if let Some(error) = validate_start(w, id) {
        tell_all(
            w,
            &format!("The duel could not be started because {error}."),
        );
        start_countdown(w, id, 10, CountdownKind::Ready);
        return;
    }

    if let Some(c) = ctx_mut(w, id) {
        c.ready_wait = false;
    }

    let override_arena = ctx(w, id).and_then(|c| c.override_arena);
    let arena_id = override_arena.or_else(|| arena::find_arena(w, &players_all));
    let Some(arena_id) = arena_id else {
        tell_all(
            w,
            "The duel could not be started because there are no arenas. If you want to stop waiting for a free arena, yield the duel.",
        );
        start_countdown(w, id, 10, CountdownKind::Ready);
        return;
    };

    if arena::is_occupied(w, arena_id) {
        tell_all(
            w,
            "The duel could not be started because all arenas are full. If you want to stop waiting for a free arena, yield the duel.",
        );
        start_countdown(w, id, 10, CountdownKind::Ready);
        return;
    }

    let initiator = ctx(w, id).expect("context").initiator;
    let (gate_point, gate_facet) = match w.mobile(initiator) {
        Some(m) if m.map.is_none() => (m.logout_location, m.logout_map),
        Some(m) => (m.location, m.map),
        None => (Point3D::default(), None),
    };
    if let Some(c) = ctx_mut(w, id) {
        c.arena = Some(arena_id);
        c.gate_point = gate_point;
        c.gate_facet = gate_facet;
    }

    let tp = arena::ensure_teleporter(w, arena_id);
    let (gate_in, outside, facet) = arena::data(w, arena_id)
        .map(|d| (d.gate_in, d.outside, d.facet))
        .unwrap_or_default();
    let mg = super::spectator::new_arena_moongate(
        w,
        if gate_in == Point3D::default() {
            outside
        } else {
            gate_in
        },
        facet,
        tp,
    );

    if let Some(c) = ctx_mut(w, id) {
        c.started_begin_countdown = true;
    }

    let parts: Vec<Vec<Option<MobileId>>> = ctx(w, id)
        .map(|c| {
            c.participants
                .iter()
                .map(|p| p.players.iter().map(|s| s.map(|d| d.mobile)).collect())
                .collect()
        })
        .unwrap_or_default();
    for (i, players) in parts.iter().enumerate() {
        for m in players.iter().flatten() {
            if let Some(tp) = tp {
                arena::teleporter_register(w, tp, *m);
            }
            set_frozen(w, *m, false); // reset timer just in case
            set_frozen(w, *m, true);
            debuff(w, *m);
            cancel_spell(w, *m);
            if let Some(ty) = w.mobile(*m).map(|x| x.ty) {
                (ty.delta)(w, *m, MobileDelta::NOTO);
            }
        }
        arena::move_inside(w, arena_id, players, i);
    }

    super::games::on_start(w, id);

    start_countdown(w, id, 10, CountdownKind::Begin);

    let (gate_point, gate_facet) = ctx(w, id)
        .map(|c| (c.gate_point, c.gate_facet))
        .unwrap_or_default();
    super::spectator::appear(w, mg, gate_point, gate_facet);
}
