//! The speech commands of the dueling system (`EventSink_Speech`: "i wish to duel", "change arena
//! preferences", "showladder", "viewladder", "i yield"), `OnLogin`, and the `vli` command.

use muo_primitives::dotnet::{IgnoreCase, InvariantCase};
use std::rc::Rc;

use muo_content::network::events::{self, PlayerLogin};
use muo_primitives::MessageType;
use muo_server::World;
use muo_server::commands::{CommandEventArgs, CommandSystem};
use muo_server::events::SpeechEventArgs;
use muo_server::ids::{EntityId, MobileId};
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part2::{messages as mm, stats};
use muo_server::target::{Target, TargetFlags, Targeted, add_target, set_mobile_target};

use super::flow::{self, name};
use super::gumps;
use super::ladder;
use super::{ctx, ctx_mut, duel_context_of};

fn msg(w: &mut World, m: MobileId, text: &str) {
    mm::send_message(w, m, text);
}

fn msg_hue(w: &mut World, m: MobileId, hue: i32, text: &str) {
    mm::send_message_hue(w, m, hue, text);
}

fn contains_ci(hay: &str, needle: &str) -> bool {
    hay.lower_inv().contains(needle)
}

fn equals_ci(a: &str, b: &str) -> bool {
    a.eq_ic(b) || a.lower_inv() == b
}

fn check_alive(w: &mut World, m: MobileId) -> bool {
    stats::check_alive(w, m, true)
}

/// `LadderGump.Rank(num)`: "1st", "12th", "1,234th".
pub fn rank(num: i32) -> String {
    let n = n0(num);
    if num % 100 > 10 && num % 100 < 20 {
        return format!("{n}th");
    }
    match num % 10 {
        1 => format!("{n}st"),
        2 => format!("{n}nd"),
        3 => format!("{n}rd"),
        _ => format!("{n}th"),
    }
}

/// `num.ToString("N0")`.
pub fn n0(num: i32) -> String {
    let s = num.unsigned_abs().to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    if num < 0 { format!("-{out}") } else { out }
}

/// `Configure`: the speech handler, the login handler and `vli`.
pub fn configure(w: &mut World) {
    w.events.speech.subscribe(on_speech);
    events::subscribe_ranked::<PlayerLogin>(w, 4, |w, e| on_login(w, e.0));
    CommandSystem::register(
        w,
        "vli",
        AccessLevel::GameMaster,
        &["ViewLadderInfo"],
        Rc::new(vli_on_command),
    );
}

/// `[Usage("vli")] [Aliases("ViewLadderInfo")] [Description("View ladder information.")]`.
fn vli_on_command(w: &mut World, e: &mut CommandEventArgs) {
    let mut t = Target::new(w, -1, false, TargetFlags::NONE);
    t.handlers.on_target = Some(Rc::new(vli_on_target));
    let id = add_target(w, t);
    set_mobile_target(w, e.mobile, Some(id));
}

fn vli_on_target(w: &mut World, from: MobileId, targeted: Targeted) {
    let Targeted::Entity(EntityId::Mobile(pm)) = targeted else {
        return;
    };
    if muo_content::player_mobile::data(w, pm).is_none() {
        return;
    }
    let Some(lad) = ladder::instance(w) else {
        return;
    };
    if ladder::find(w, lad, pm).is_some() {
        let g = muo_gumps::m_z::props::properties::new(
            w,
            from,
            muo_gumps::m_z::props::registry::SubObj::object(
                EntityId::Mobile(pm),
                super::props::LADDER_ENTRY_PROPS,
                "LadderEntry",
            ),
        );
        muo_gumps::m_z::common::send(w, from, g);
    }
}

/// `OnLogin(pm)`: the gump the player had when they left comes back.
pub fn on_login(w: &mut World, pm: MobileId) {
    let Some(id) = duel_context_of(w, pm) else {
        return;
    };
    let Some(c) = ctx(w, id) else { return };
    let ready = c.find(pm).is_some_and(|p| p.ready);
    let (ready_wait, sbc, started, finished, tournament, rc, initiator) = (
        c.ready_wait,
        c.started_begin_countdown,
        c.started,
        c.finished,
        c.tournament.is_some(),
        c.ready_count,
        c.initiator,
    );
    if ready_wait && ready && !started && !sbc && !finished {
        if !tournament {
            gumps::display_ready(w, pm, id, rc);
        }
    } else if ready_wait && !sbc && !started && !finished {
        if !tournament {
            gumps::display_ready_up(w, pm, id);
        }
    } else if initiator == pm && !ready_wait && !sbc && !started && !finished {
        gumps::display_duel_context(w, pm, id);
    }
}

/// `ViewLadder_OnTarget(from, obj, ladder)`.
fn view_ladder_on_target(
    w: &mut World,
    from: MobileId,
    targeted: Targeted,
    lad: muo_server::ids::ItemId,
) {
    let from_ns = w.mobile(from).and_then(|m| m.net_state);
    let Targeted::Entity(EntityId::Mobile(mob)) = targeted else {
        msg(w, from, "That's not a player.");
        return;
    };
    let hue = w.mobile(mob).map_or(0, |m| m.speech_hue);
    if muo_content::player_mobile::data(w, mob).is_some() {
        let Some(entry) = ladder::find(w, lad, mob) else {
            return;
        }; // sanity
        let level = ladder::get_level(entry.experience);
        let r = rank(entry.index as i32 + 1);
        let text = if from == mob {
            format!("You are ranked {r} at level {level}.")
        } else {
            format!("They are ranked {r} at level {level}.")
        };
        mm::private_overhead_message(w, mob, MessageType::REGULAR, hue, true, &text, from_ns);
    } else if muo_content::creature::misc::body_is_human(w, mob) {
        mm::private_overhead_message(
            w,
            mob,
            MessageType::REGULAR,
            hue,
            false,
            "I'm not a duelist, and quite frankly, I resent the implication.",
            from_ns,
        );
    } else {
        mm::private_overhead_message(
            w,
            mob,
            MessageType::REGULAR,
            0x3B2,
            true,
            "It's probably better than you.",
            from_ns,
        );
    }
}

/// `EventSink_Speech(e)`.
fn on_speech(w: &mut World, e: &mut SpeechEventArgs) {
    if e.handled {
        return;
    }
    let pm = e.mobile;
    if muo_content::player_mobile::data(w, pm).is_none() {
        return;
    }
    let speech = e.speech.clone();

    if contains_ci(&speech, "i wish to duel") {
        if !check_alive(w, pm)
            || w.mobile(pm).is_some_and(|m| {
                muo_server::region::is_part_of_type(w, m.region, &muo_content::jail::JAIL_REGION)
            })
        {
            // neither gets an answer (an empty block each)
        } else if flow::check_combat(w, pm) {
            msg_hue(
                w,
                pm,
                0x22,
                "You have recently been in combat with another player and must wait before starting a duel.",
            );
        } else if let Some(id) = duel_context_of(w, pm) {
            if ctx(w, id).is_some_and(|c| c.initiator == pm) {
                msg_hue(w, pm, 0x22, "You have already started a duel.");
            } else {
                msg_hue(w, pm, 0x22, "You have already been challenged in a duel.");
            }
        } else if super::tournament::is_active(w) {
            msg_hue(
                w,
                pm,
                0x22,
                "You may not start a duel while a tournament is active.",
            );
        } else {
            let id = flow::new_context(w, pm);
            gumps::display_duel_context(w, pm, id);
            e.handled = true;
        }
    } else if equals_ci(&speech, "change arena preferences") {
        if check_alive(w, pm) && ladder::preferences_instance(w).is_some() {
            gumps::display_preferences(w, pm);
        }
    } else if equals_ci(&speech, "showladder") {
        e.blocked = true;
        if !check_alive(w, pm) {
            return;
        }
        let Some(lad) = ladder::instance(w) else {
            return;
        };
        let Some(entry) = ladder::find(w, lad, pm) else {
            return;
        }; // sanity
        let level = ladder::get_level(entry.experience);
        let r = rank(entry.index as i32 + 1);
        let hue = w.mobile(pm).map_or(0, |m| m.speech_hue);
        mm::local_overhead_message(
            w,
            pm,
            MessageType::REGULAR,
            hue,
            true,
            &format!("You are ranked {r} at level {level}."),
        );
        let n = name(w, pm);
        mm::nonlocal_overhead_message(
            w,
            pm,
            MessageType::REGULAR,
            hue,
            true,
            &format!("{n} is ranked {r} at level {level}."),
        );
    } else if equals_ci(&speech, "viewladder") {
        e.blocked = true;
        if !check_alive(w, pm) {
            return;
        }
        let Some(lad) = ladder::instance(w) else {
            return;
        };
        msg(w, pm, "Target a player to view their ranking and level.");
        let mut t = Target::new(w, 16, false, TargetFlags::NONE);
        t.handlers.on_target = Some(Rc::new(
            move |w: &mut World, from: MobileId, targeted: Targeted| {
                view_ladder_on_target(w, from, targeted, lad)
            },
        ));
        let id = add_target(w, t);
        set_mobile_target(w, pm, Some(id));
    } else if contains_ci(&speech, "i yield") {
        yield_duel(w, pm);
    }
}

/// The "i yield" branch.
fn yield_duel(w: &mut World, pm: MobileId) {
    if !check_alive(w, pm) {
        return;
    }
    let Some(id) = duel_context_of(w, pm) else {
        return;
    };
    let Some(c) = ctx(w, id) else { return };
    if c.finished {
        msg_hue(w, pm, 0x22, "The duel is already finished.");
        return;
    }
    let init = c.initiator;

    if !c.started {
        if c.started_begin_countdown {
            msg_hue(w, pm, 0x22, "The duel has not yet started.");
            return;
        }
        let Some(pi) = c.participant_of(pm) else {
            return;
        };
        let ready_wait = c.ready_wait;
        let started_ready = c.started_ready_countdown;
        let uid = c.participants[pi].uid;
        let broadcast = |w: &mut World| {
            let players: Vec<MobileId> = ctx(w, id)
                .map(|c| {
                    c.participants[pi]
                        .players
                        .iter()
                        .flatten()
                        .map(|d| d.mobile)
                        .collect()
                })
                .unwrap_or_default();
            for m in players {
                let n = name(w, m);
                mm::nonlocal_overhead_message(
                    w,
                    m,
                    MessageType::REGULAR,
                    0x22,
                    false,
                    &format!("{n} has yielded."),
                );
                mm::local_overhead_message(
                    w,
                    m,
                    MessageType::REGULAR,
                    0x22,
                    false,
                    "You have yielded.",
                );
            }
        };

        if !ready_wait {
            // still setting stuff up
            broadcast(w);
            if init == pm {
                flow::unregister(w, id, false);
            } else {
                nullify(w, id, pi, pm);
                redisplay_setup(w, init, id, uid, false);
            }
        } else {
            if started_ready {
                flow::stop_countdown(w, id);
                if let Some(c) = ctx_mut(w, id) {
                    c.started_ready_countdown = false;
                }
            }
            broadcast(w);
            if let Some(c) = ctx_mut(w, id) {
                c.yielding = true;
            }
            flow::reject_ready(w, id, pm, None);
            if let Some(c) = ctx_mut(w, id) {
                c.yielding = false;
            }
            if init == pm {
                flow::unregister(w, id, false);
            } else if ctx(w, id).is_some_and(|c| c.registered) {
                nullify(w, id, pi, pm);
                redisplay_setup(w, init, id, uid, true);
            }
        }
        return;
    }

    // The duel has started.
    let Some(_) = c.find(pm) else {
        msg_hue(w, pm, 0x22, "BUG: Unable to find duel context.");
        return;
    };
    if c.is_one_vs_one() {
        msg_hue(w, pm, 0x22, "You may not yield a 1 on 1 match.");
    } else if c.find(pm).is_some_and(|p| p.eliminated) {
        msg_hue(w, pm, 0x22, "You have already been eliminated.");
    } else {
        mm::local_overhead_message(
            w,
            pm,
            MessageType::REGULAR,
            0x22,
            false,
            "You have yielded.",
        );
        let n = name(w, pm);
        mm::nonlocal_overhead_message(
            w,
            pm,
            MessageType::REGULAR,
            0x22,
            false,
            &format!("{n} has yielded."),
        );

        if let Some(c) = ctx_mut(w, id) {
            c.yielding = true;
        }
        if let Some(ty) = w.mobile(pm).map(|m| m.ty) {
            (ty.kill)(w, pm);
        }
        if let Some(c) = ctx_mut(w, id) {
            c.yielding = false;
        }

        if muo_content::mx::alive(w, pm) {
            // invul, ...
            super::set_eliminated(w, id, pm);
            flow::remove_aggressions(w, id, pm);
            flow::send_outside(w, id, pm);
            flow::refresh(w, pm, None);
            flow::debuff(w, pm);
            flow::cancel_spell(w, pm);
            muo_server::mobile::part1::props_a::set_frozen(w, pm, false);
            if let Some(winner) = flow::check_completion(w, id) {
                flow::finish(w, id, winner);
            }
        }
    }
}

/// `p.Nullify(pl); pm.DuelPlayer = null;`
fn nullify(w: &mut World, id: u32, pi: usize, pm: MobileId) {
    if let Some(c) = ctx_mut(w, id) {
        c.participants[pi].nullify(pm);
    }
    super::release_mobile(w, pm);
}

/// The initiator's open participant or setup gump is shown again (the first of either in the order the gumps were
/// sent); `fallback` shows the setup gump when none is open.
fn redisplay_setup(w: &mut World, init: MobileId, id: u32, uid: u32, fallback: bool) {
    use muo_content::gumps::system::{downcast_logic, gumps_of};
    // Some(true) = a ParticipantGump of `uid`, Some(false) = this context's DuelContextGump.
    let first = w.mobile(init).and_then(|m| m.net_state).and_then(|ns| {
        gumps_of(w, ns).find_map(|g| {
            if downcast_logic::<gumps::ParticipantGump>(g).is_some_and(|g| g.participant == uid) {
                Some(true)
            } else if downcast_logic::<gumps::DuelContextGump>(g).is_some_and(|g| g.ctx == id) {
                Some(false)
            } else {
                None
            }
        })
    });
    match first {
        Some(true) => gumps::display_participant(w, init, id, uid),
        Some(false) => gumps::display_duel_context(w, init, id),
        None if fallback => gumps::display_duel_context(w, init, id),
        None => {}
    }
}
