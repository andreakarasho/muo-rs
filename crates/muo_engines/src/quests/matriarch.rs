//! The Solen Matriarch quest: the quest, its objectives and conversations
//! (namespace `Server.Engines.Quests.Matriarch`).

use muo_content::player_mobile::{self as pmob, SolenFriendship};
use muo_io::{GenericReader, GenericWriter};
use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::ids::MobileId;

use super::core::{
    ConvClass, Conversation, Msg, ObjClass, Objective, QClass, QUEST_SYSTEM, Quest, St,
    base_accept, base_render_progress, never_logged,
};
use super::gumps::BLUE;
use super::util::{give_to_backpack, mobile_is, msg_hue, random_min_max};
use crate::{conversation, objective};

macro_rules! conv {
    ($n:ident, $s:literal $($t:tt)*) => {
        conversation!($n = concat!("Server.Engines.Quests.Matriarch.", $s) $($t)*);
    };
}
macro_rules! obj {
    ($n:ident, $s:literal $($t:tt)*) => {
        objective!($n = concat!("Server.Engines.Quests.Matriarch.", $s) $($t)*);
    };
}

/// `SolenMatriarchQuest.RedSolen`.
pub fn red_solen(q: &Quest) -> bool {
    *q.state::<bool>()
}

/// `new SolenMatriarchQuest(from, redSolen)`.
pub fn new_quest(w: &World, from: MobileId, red_solen: bool) -> Quest {
    Quest::with(w, &QUEST, from, red_solen)
}

/// `SolenMatriarchQuest.IsFriend(player, redSolen)`.
pub fn is_friend(w: &World, player: MobileId, red_solen: bool) -> bool {
    let f = pmob::data(w, player).map_or(SolenFriendship::None, |d| d.solen_friendship);
    if red_solen {
        f == SolenFriendship::Red
    } else {
        f == SolenFriendship::Black
    }
}

/// `SolenMatriarchQuest.GiveRewardTo(player)`.
pub fn give_reward_to(w: &mut World, player: MobileId) -> bool {
    let amount = random_min_max(w, 250, 350);
    if give_to_backpack(w, player, "Server.Items.Gold", Some(amount)) {
        msg_hue(w, player, 1054076, 0x3B2); // You have been given some gold.
        true
    } else {
        false
    }
}

// ---- Conversations ----

/// The `bool m_Friend` of `DontOfferConversation` and `ProcessFungiConversation`.
fn friend_state() -> St {
    St::new(false)
}

fn friend_de(_: &World, c: &mut Conversation, r: &mut dyn GenericReader) -> muo_io::Result<()> {
    r.read_encoded_int()?; // version
    *c.st.get_mut::<bool>() = r.read_bool()?;
    Ok(())
}

fn friend_ser(_: &World, c: &Conversation, out: &mut dyn GenericWriter) {
    out.write_encoded_int(0); // version
    out.write_bool(*c.state::<bool>());
}

/// `new DontOfferConversation(friend)`.
pub fn dont_offer(friend: bool) -> Conversation {
    Conversation::with(&DONT_OFFER_CONVERSATION, friend)
}

conv!(DONT_OFFER_CONVERSATION, "DontOfferConversation",
    new_state: friend_state,
    message: |_, c| if *c.state::<bool>() { 1054081 } else { 1054079 },
    logged: never_logged,
    child_deserialize: friend_de,
    child_serialize: friend_ser);

conv!(ACCEPT_CONVERSATION, "AcceptConversation", msg 1054084,
    on_read: |w, q, _| q.add_objective(w, Objective::new(&KILL_INFILTRATORS_OBJECTIVE)));
conv!(DURING_KILL_INFILTRATORS_CONVERSATION, "DuringKillInfiltratorsConversation", msg 1054089, logged: never_logged);
conv!(GATHER_WATER_CONVERSATION, "GatherWaterConversation", msg 1054091,
    on_read: |w, q, _| q.add_objective(w, Objective::new(&GATHER_WATER_OBJECTIVE)));
conv!(DURING_WATER_GATHERING_CONVERSATION, "DuringWaterGatheringConversation", msg 1054094, logged: never_logged);

/// `new ProcessFungiConversation(friend)`.
pub fn process_fungi(friend: bool) -> Conversation {
    Conversation::with(&PROCESS_FUNGI_CONVERSATION, friend)
}

conv!(PROCESS_FUNGI_CONVERSATION, "ProcessFungiConversation",
    new_state: friend_state,
    message: |_, c| if *c.state::<bool>() { 1054097 } else { 1054096 },
    on_read: |w, q, _| q.add_objective(w, Objective::new(&PROCESS_FUNGI_OBJECTIVE)),
    child_deserialize: friend_de,
    child_serialize: friend_ser);

conv!(DURING_FUNGI_PROCESS_CONVERSATION, "DuringFungiProcessConversation", msg 1054099, logged: never_logged);

/// `new FullBackpackConversation(logged)`; the flag is not saved.
pub fn full_backpack(logged: bool) -> Conversation {
    Conversation::with(&FULL_BACKPACK_CONVERSATION, logged)
}

conv!(FULL_BACKPACK_CONVERSATION, "FullBackpackConversation", msg 1054102,
new_state: || St::new(true),
logged: |_, c| *c.state::<bool>(),
on_read: |w, q, c| {
    if *c.state::<bool>() {
        q.add_objective(w, Objective::new(&GET_REWARD_OBJECTIVE));
    }
});

conv!(END_CONVERSATION, "EndConversation", msg 1054101, on_read: |w, q, _| q.complete(w));

pub static CONVERSATIONS: &[&ConvClass] = &[
    &DONT_OFFER_CONVERSATION,
    &ACCEPT_CONVERSATION,
    &DURING_KILL_INFILTRATORS_CONVERSATION,
    &GATHER_WATER_CONVERSATION,
    &DURING_WATER_GATHERING_CONVERSATION,
    &PROCESS_FUNGI_CONVERSATION,
    &DURING_FUNGI_PROCESS_CONVERSATION,
    &FULL_BACKPACK_CONVERSATION,
    &END_CONVERSATION,
];

// ---- Objectives ----

/// The infiltrators the quest is about: the other colony's warriors and queens.
fn infiltrators(q: &Quest) -> [&'static str; 2] {
    if red_solen(q) {
        [
            "Server.Mobiles.BlackSolenInfiltratorWarrior",
            "Server.Mobiles.BlackSolenInfiltratorQueen",
        ]
    } else {
        [
            "Server.Mobiles.RedSolenInfiltratorWarrior",
            "Server.Mobiles.RedSolenInfiltratorQueen",
        ]
    }
}

obj!(KILL_INFILTRATORS_OBJECTIVE, "KillInfiltratorsObjective",
    message: |q, _| if red_solen(q) { 1054086 } else { 1054085 },
    max_progress: |_, _| 7,
    render_progress: |q, o, b| {
        if !o.completed(q) {
            // Black/Red Solen Infiltrators killed:
            b.add_html_localized_color(70, 260, 270, 100, if red_solen(q) { 1054088 } else { 1054087 }, BLUE, false, false);
            b.add_label(70, 280, 0x64, &o.cur_progress.to_string());
            b.add_label(100, 280, 0x64, "/");
            b.add_label(130, 280, 0x64, &o.max_progress(q).to_string());
        } else {
            base_render_progress(q, o, b);
        }
    },
    ignore_young_protection: |w, q, i, from| {
        !q.objectives[i].completed(q) && infiltrators(q).iter().any(|n| mobile_is(w, from, n))
    },
    on_kill: |w, q, i, creature, _| {
        if infiltrators(q).iter().any(|n| mobile_is(w, creature, n)) {
            q.add_progress(w, i);
        }
    },
    on_complete: |w, q, _| q.add_objective(w, Objective::new(&RETURN_AFTER_KILLS_OBJECTIVE)));

obj!(RETURN_AFTER_KILLS_OBJECTIVE, "ReturnAfterKillsObjective", msg 1054090,
    on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&GATHER_WATER_CONVERSATION)));

obj!(GATHER_WATER_OBJECTIVE, "GatherWaterObjective", msg 1054092,
    max_progress: |_, _| 40,
    render_progress: |q, o, b| {
        if !o.completed(q) {
            b.add_html_localized_color(70, 260, 270, 100, 1054093, BLUE, false, false); // Gallons of Water gathered:
            b.add_label(70, 280, 0x64, &(o.cur_progress / 5).to_string());
            b.add_label(100, 280, 0x64, "/");
            b.add_label(130, 280, 0x64, &(o.max_progress(q) / 5).to_string());
        } else {
            base_render_progress(q, o, b);
        }
    },
    on_complete: |w, q, _| q.add_objective(w, Objective::new(&RETURN_AFTER_WATER_OBJECTIVE)));

obj!(RETURN_AFTER_WATER_OBJECTIVE, "ReturnAfterWaterObjective", msg 1054095,
on_complete: |w, q, _| {
    let player = q.from;
    let red = red_solen(q);
    let friend = is_friend(w, player, red);
    q.add_conversation(w, process_fungi(friend));
    if let Some(d) = pmob::data_mut(w, player) {
        d.solen_friendship = if red { SolenFriendship::Red } else { SolenFriendship::Black };
    }
});

obj!(PROCESS_FUNGI_OBJECTIVE, "ProcessFungiObjective", msg 1054098,
on_complete: |w, q, _| {
    if give_reward_to(w, q.from) {
        q.complete(w);
    } else {
        q.add_conversation(w, full_backpack(true));
    }
});

obj!(GET_REWARD_OBJECTIVE, "GetRewardObjective", msg 1054149,
    on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&END_CONVERSATION)));

pub static OBJECTIVES: &[&ObjClass] = &[
    &KILL_INFILTRATORS_OBJECTIVE,
    &RETURN_AFTER_KILLS_OBJECTIVE,
    &GATHER_WATER_OBJECTIVE,
    &RETURN_AFTER_WATER_OBJECTIVE,
    &PROCESS_FUNGI_OBJECTIVE,
    &GET_REWARD_OBJECTIVE,
];

// ---- The quest ----

/// `SolenMatriarchQuest`: the state is `RedSolen`.
pub static QUEST: QClass = QClass {
    name: "Server.Engines.Quests.Matriarch.SolenMatriarchQuest",
    slot: 6,
    name_msg: Msg::Num(1054147),
    offer_message: |w, q| {
        if is_friend(w, q.from, red_solen(q)) {
            Msg::Num(1054083)
        } else {
            Msg::Num(1054082)
        }
    },
    picture: |_, _| 0x15C9,
    is_tutorial: false,
    restart_delay: TimeSpan::ZERO,
    new_state: || St::new(false),
    accept: |w, q: &mut Quest| {
        base_accept(w, q);
        q.add_conversation(w, Conversation::new(&ACCEPT_CONVERSATION));
    },
    child_deserialize: |_, q, r| {
        r.read_encoded_int()?; // version
        *q.state_mut::<bool>() = r.read_bool()?;
        Ok(())
    },
    child_serialize: |_, q, out| {
        out.write_encoded_int(0); // version
        out.write_bool(red_solen(q));
    },
    ..QUEST_SYSTEM
};

/// The water vat branch of `BaseBeverage.Pour_OnTarget`: true when `item` is a component of a `WaterVatEast` or
/// `WaterVatSouth` (the pour is then handled, whatever the quest says).
pub fn pour_on_vat(
    w: &mut World,
    from: MobileId,
    item: muo_server::ids::ItemId,
    bev: muo_server::ids::ItemId,
) -> bool {
    use muo_items::food::beverage;
    let Some(vat) = muo_content::items::addon::component_addon(w, item) else {
        return false;
    };
    let is_vat = w.item(vat).is_some_and(|i| {
        muo_content::items::is_named(i.ty, "Server.Items.WaterVatEast")
            || muo_content::items::is_named(i.ty, "Server.Items.WaterVatSouth")
    });
    if !is_vat {
        return false;
    }
    let (loc, map) = w
        .item(vat)
        .map_or_else(Default::default, |i| (i.location, i.map));
    super::core::with_quest_of(w, from, &QUEST, |w, q| {
        let Some(i) = q
            .find_objective(&GATHER_WATER_OBJECTIVE)
            .filter(|&i| !q.objectives[i].completed(q))
        else {
            return;
        };
        let red = red_solen(q);
        let in_area = loc.x > 5784 && loc.x < 5814 && loc.y > 1903 && loc.y < 1934;
        let right_facet = (red && map == Some(muo_server::map::MapId::TRAMMEL))
            || (!red && map == Some(muo_server::map::MapId::FELUCCA));
        if !(in_area && right_facet) {
            return;
        }
        let (cur, max) = (
            q.objectives[i].cur_progress,
            q.objectives[i].max_progress(q),
        );
        let quantity = beverage::quantity(w, bev);
        if cur + quantity > max {
            beverage::set_quantity(w, bev, quantity - (max - cur));
            q.set_progress(w, i, max);
        } else {
            q.set_progress(w, i, cur + quantity);
            beverage::set_quantity(w, bev, 0);
        }
    });
    true
}
