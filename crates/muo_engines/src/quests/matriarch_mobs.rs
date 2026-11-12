//! The Solen Matriarch quest's `SolenMatriarch` mobiles (namespace `Server.Engines.Quests.Matriarch`).

use muo_server::World;
use muo_server::context_menu_entry::ContextMenuEntry;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::mobile::MobileType;
use muo_server::target::Targeted;

use super::core::{Conversation, ObjClass, Quest, can_offer_quest, has_quest, load, with_quest};
use super::matriarch::*;
use super::quester::{BASE_QUESTER, QUESTER_C, QUESTER_OPS, QUESTER_VC, QuesterOps};
use super::util::{amount_of, begin_target, deleted_item, item_is, say_to_number};

fn is_red(w: &World, id: MobileId) -> bool {
    w.mobile(id).is_some_and(|m| m.ty.is(&RED_SOLEN_MATRIARCH))
}

/// The player's quest when it is this matriarch's colony's.
fn our_quest(w: &World, player: MobileId, red: bool) -> Option<Quest> {
    load(w, player).filter(|q| q.is(&QUEST) && red_solen(q) == red)
}

fn turn_to(w: &mut World, matriarch: MobileId, player: MobileId) {
    let d = muo_content::mx::get_direction_to(w, matriarch, player);
    muo_content::mx::set_direction(w, matriarch, d);
}

fn can_talk_to(w: &mut World, matriarch: MobileId, to: MobileId) -> bool {
    let red = is_red(w, matriarch);
    is_friend(w, to, red) || our_quest(w, to, red).is_some()
}

/// The offer a seed starts: the offer gump, or the "not now" talk.
fn offer_or_refuse(w: &mut World, player: MobileId, red: bool) {
    let mut new_quest = new_quest(w, player, red);
    if !has_quest(w, player) && can_offer_quest(w, player, &QUEST) {
        new_quest.send_offer(w);
    } else {
        let friend = is_friend(w, player, red);
        new_quest.add_conversation(w, dont_offer(friend));
    }
}

fn on_talk(w: &mut World, matriarch: MobileId, player: MobileId, _context_menu: bool) {
    turn_to(w, matriarch, player);
    let red = is_red(w, matriarch);

    let Some(qs) = our_quest(w, player, red) else {
        if is_friend(w, player, red) {
            let mut new_quest = new_quest(w, player, red);
            if !has_quest(w, player) && can_offer_quest(w, player, &QUEST) {
                new_quest.send_offer(w);
            } else {
                new_quest.add_conversation(w, dont_offer(true));
            }
        }
        return;
    };

    let pending = |c: &ObjClass| {
        qs.find_objective(c)
            .filter(|&i| !qs.objectives[i].completed(&qs))
    };
    if qs.is_objective_in_progress(&KILL_INFILTRATORS_OBJECTIVE) {
        with_quest(w, player, |w, q| {
            q.add_conversation(w, Conversation::new(&DURING_KILL_INFILTRATORS_CONVERSATION))
        });
    } else if let Some(i) = pending(&RETURN_AFTER_KILLS_OBJECTIVE) {
        with_quest(w, player, |w, q| q.complete_objective(w, i));
    } else if qs.is_objective_in_progress(&GATHER_WATER_OBJECTIVE) {
        with_quest(w, player, |w, q| {
            q.add_conversation(w, Conversation::new(&DURING_WATER_GATHERING_CONVERSATION))
        });
    } else if let Some(i) = pending(&RETURN_AFTER_WATER_OBJECTIVE) {
        with_quest(w, player, |w, q| q.complete_objective(w, i));
    } else if qs.is_objective_in_progress(&PROCESS_FUNGI_OBJECTIVE) {
        with_quest(w, player, |w, q| {
            q.add_conversation(w, Conversation::new(&DURING_FUNGI_PROCESS_CONVERSATION))
        });
    } else if let Some(i) = pending(&GET_REWARD_OBJECTIVE) {
        if give_reward_to(w, player) {
            with_quest(w, player, |w, q| q.complete_objective(w, i));
        } else {
            with_quest(w, player, |w, q| {
                q.add_conversation(w, full_backpack(false))
            });
        }
    }
}

/// `BaseSolenMatriarch.OnGivenFungi(player, fungi)`.
fn on_given_fungi(w: &mut World, matriarch: MobileId, player: MobileId, fungi: ItemId) {
    turn_to(w, matriarch, player);
    let red = is_red(w, matriarch);
    let Some(qs) = our_quest(w, player, red) else {
        return;
    };
    let Some(obj) = qs
        .find_objective(&PROCESS_FUNGI_OBJECTIVE)
        .filter(|&i| !qs.objectives[i].completed(&qs))
    else {
        return;
    };

    let have = amount_of(w, fungi);
    let amount = (have / 2).min(100);
    if amount > 0 {
        if amount * 2 >= have {
            muo_server::item::base::delete(w, fungi);
        } else {
            muo_server::item::ops::set_amount(w, fungi, have - amount * 2);
        }
        if let Some(powder) =
            muo_content::factory::make_item(w, "Server.Items.PowderOfTranslocation")
        {
            muo_server::item::ops::set_amount(w, powder, amount);
            muo_server::mobile::part2::items::add_to_backpack(w, player, powder);
        }
        muo_content::mx::msg(w, player, 1054100); // You receive some powder of translocation.
        with_quest(w, player, |w, q| q.complete_objective(w, obj));
    }
}

fn on_drag_drop(w: &mut World, matriarch: MobileId, from: MobileId, dropped: ItemId) -> bool {
    let is_player = w
        .mobile(from)
        .is_some_and(|m| m.ty.is(&muo_content::player_mobile::PLAYER_MOBILE));
    if !is_player {
        return (BASE_QUESTER.on_drag_drop)(w, matriarch, from, dropped);
    }
    if !item_is(w, dropped, "Server.Engines.Plants.Seed") {
        if item_is(w, dropped, "Server.Items.ZoogiFungus") {
            on_given_fungi(w, matriarch, from, dropped);
            return deleted_item(w, dropped);
        }
        return (BASE_QUESTER.on_drag_drop)(w, matriarch, from, dropped);
    }

    let red = is_red(w, matriarch);
    if our_quest(w, from, red).is_some() {
        say_to_number(w, matriarch, from, 1054080); // Thank you for that plant seed. Those have such wonderful flavor.
    } else {
        offer_or_refuse(w, from, red);
    }
    muo_server::item::base::delete(w, dropped);
    true
}

fn context_menu_entries(
    w: &mut World,
    matriarch: MobileId,
    from: MobileId,
    list: &mut Vec<ContextMenuEntry>,
) {
    (BASE_QUESTER.get_context_menu_entries)(w, matriarch, from, list);

    let red = is_red(w, matriarch);
    if muo_content::mx::alive(w, from)
        && our_quest(w, from, red)
            .is_some_and(|q| q.is_objective_in_progress(&PROCESS_FUNGI_OBJECTIVE))
    {
        // ProcessZoogiFungusEntry
        let mut e = ContextMenuEntry::new(6184, -1);
        e.on_click = Some(std::rc::Rc::new(
            |w: &mut World, from: MobileId, target: EntityId| {
                let EntityId::Mobile(matriarch) = target else {
                    return;
                };
                if muo_content::mx::alive(w, from) {
                    // ProcessFungiTarget
                    begin_target(
                        w,
                        from,
                        true,
                        move |w, from, targeted| {
                            let Targeted::Entity(EntityId::Item(fungus)) = targeted else {
                                return;
                            };
                            if !item_is(w, fungus, "Server.Items.ZoogiFungus") {
                                return;
                            }
                            if super::util::in_backpack(w, fungus, from) {
                                on_given_fungi(w, matriarch, from, fungus);
                            } else {
                                muo_content::mx::msg(w, from, 1042001); // That must be in your pack for you to use it.
                            }
                        },
                        Some(|w, from| super::util::msg_hue(w, from, 1042021, 0x59)), // Cancelled.
                    );
                }
            },
        ));
        list.push(e);
    }
}

static MATRIARCH_OPS: QuesterOps = QuesterOps {
    on_talk,
    can_talk_to,
    ..QUESTER_OPS
};

/// `BaseSolenMatriarch()`: the body, the black colony's hue, no speech hue.
fn extra(w: &mut World, id: MobileId) {
    super::util::set_body(w, id, 0x328);
    if !is_red(w, id) {
        muo_server::mobile::base::set_hue(w, id, 0x44E);
    }
    if let Some(m) = w.mobile_mut(id) {
        m.speech_hue = 0;
    }
}

crate::quester! {
    /// `Server.Engines.Quests.Matriarch.BaseSolenMatriarch`.
    BASE_SOLEN_MATRIARCH, BASE_SOLEN_MATRIARCH_C, BASE_SOLEN_MATRIARCH_V
        = "Server.Engines.Quests.Matriarch.BaseSolenMatriarch" : BASE_QUESTER / QUESTER_C / QUESTER_VC,
        ops MATRIARCH_OPS, encoded true, abstract;
    v { }
    c {
        default_name: |_, _| Some("the solen matriarch"),
        disallow_all_moves: |_, _| false,
    }
    m {
        get_idle_sound: |_, _| 0x10D,
        on_drag_drop: on_drag_drop,
        get_context_menu_entries: context_menu_entries,
    }
}

crate::quester! {
    /// `Server.Engines.Quests.Matriarch.RedSolenMatriarch`.
    RED_SOLEN_MATRIARCH, RED_SOLEN_MATRIARCH_C, RED_SOLEN_MATRIARCH_V
        = "Server.Engines.Quests.Matriarch.RedSolenMatriarch"
        : BASE_SOLEN_MATRIARCH / BASE_SOLEN_MATRIARCH_C / BASE_SOLEN_MATRIARCH_V,
        ops MATRIARCH_OPS, encoded true, title None, extra extra;
    v { }
    c { }
    m { }
}

crate::quester! {
    /// `Server.Engines.Quests.Matriarch.BlackSolenMatriarch`.
    BLACK_SOLEN_MATRIARCH, BLACK_SOLEN_MATRIARCH_C, BLACK_SOLEN_MATRIARCH_V
        = "Server.Engines.Quests.Matriarch.BlackSolenMatriarch"
        : BASE_SOLEN_MATRIARCH / BASE_SOLEN_MATRIARCH_C / BASE_SOLEN_MATRIARCH_V,
        ops MATRIARCH_OPS, encoded true, title None, extra extra;
    v { }
    c { }
    m { }
}

/// The mobile classes of this folder.
pub static MOBILES: &[&MobileType] = &[
    &BASE_SOLEN_MATRIARCH,
    &RED_SOLEN_MATRIARCH,
    &BLACK_SOLEN_MATRIARCH,
];
