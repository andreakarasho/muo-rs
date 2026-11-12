//! The Ambitious Solen Queen quest: the quest, its objectives and conversations
//! (namespace `Server.Engines.Quests.Ambitious`).

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
        conversation!($n = concat!("Server.Engines.Quests.Ambitious.", $s) $($t)*);
    };
}
macro_rules! obj {
    ($n:ident, $s:literal $($t:tt)*) => {
        objective!($n = concat!("Server.Engines.Quests.Ambitious.", $s) $($t)*);
    };
}

/// `AmbitiousQueenQuest.RedSolen`.
pub fn red_solen(q: &Quest) -> bool {
    *q.state::<bool>()
}

/// `new AmbitiousQueenQuest(from, redSolen)`.
pub fn new_quest(w: &World, from: MobileId, red_solen: bool) -> Quest {
    Quest::with(w, &QUEST, from, red_solen)
}

/// `AmbitiousQueenQuest.GiveRewardTo(player, ref bagOfSending, ref powderOfTranslocation, ref gold)`.
pub fn give_reward_to(
    w: &mut World,
    player: MobileId,
    bag: &mut bool,
    powder: &mut bool,
    gold: &mut bool,
) {
    if *bag && give_to_backpack(w, player, "Server.Items.BagOfSending", None) {
        msg_hue(w, player, 1054074, 0x59); // You have been given a bag of sending.
        *bag = false;
    }
    if *powder {
        let amount = random_min_max(w, 10, 12);
        if give_to_backpack(
            w,
            player,
            "Server.Items.PowderOfTranslocation",
            Some(amount),
        ) {
            msg_hue(w, player, 1054075, 0x59); // You have been given some powder of translocation.
            *powder = false;
        }
    }
    if *gold {
        let amount = random_min_max(w, 250, 350);
        if give_to_backpack(w, player, "Server.Items.Gold", Some(amount)) {
            msg_hue(w, player, 1054076, 0x59); // You have been given some gold.
            *gold = false;
        }
    }
}

// ---- Conversations ----

conv!(DONT_OFFER_CONVERSATION, "DontOfferConversation", msg 1054059, logged: never_logged);
conv!(ACCEPT_CONVERSATION, "AcceptConversation", msg 1054061,
    on_read: |w, q, _| q.add_objective(w, Objective::new(&KILL_QUEENS_OBJECTIVE)));
conv!(DURING_KILL_QUEENS_CONVERSATION, "DuringKillQueensConversation", msg 1054066, logged: never_logged);
conv!(GATHER_FUNGI_CONVERSATION, "GatherFungiConversation", msg 1054068,
    on_read: |w, q, _| q.add_objective(w, Objective::new(&GATHER_FUNGI_OBJECTIVE)));
conv!(DURING_FUNGI_GATHERING_CONVERSATION, "DuringFungiGatheringConversation", msg 1054070, logged: never_logged);
conv!(END_CONVERSATION, "EndConversation", msg 1054073,
on_read: |w, q, _| {
    let (mut bag, mut powder, mut gold) = (true, true, true);
    give_reward_to(w, q.from, &mut bag, &mut powder, &mut gold);
    if !bag && !powder && !gold {
        q.complete(w);
    } else {
        q.add_conversation(w, full_backpack(true, bag, powder, gold));
    }
});

/// `FullBackpackConversation`'s fields: `m_Logged` (not saved), then the three saved flags.
#[derive(Clone)]
struct FullBackpack {
    logged: bool,
    bag: bool,
    powder: bool,
    gold: bool,
}

/// `new FullBackpackConversation(logged, bagOfSending, powderOfTranslocation, gold)`.
pub fn full_backpack(logged: bool, bag: bool, powder: bool, gold: bool) -> Conversation {
    Conversation::with(
        &FULL_BACKPACK_CONVERSATION,
        FullBackpack {
            logged,
            bag,
            powder,
            gold,
        },
    )
}

conv!(FULL_BACKPACK_CONVERSATION, "FullBackpackConversation", msg 1054077,
new_state: || St::new(FullBackpack { logged: true, bag: false, powder: false, gold: false }),
logged: |_, c| c.state::<FullBackpack>().logged,
on_read: |w, q, c| {
    let s = c.state::<FullBackpack>();
    if s.logged {
        q.add_objective(w, get_reward(s.bag, s.powder, s.gold));
    }
},
child_deserialize: |_, c, r| {
    r.read_encoded_int()?; // version
    let s = c.st.get_mut::<FullBackpack>();
    s.bag = r.read_bool()?;
    s.powder = r.read_bool()?;
    s.gold = r.read_bool()?;
    Ok(())
},
child_serialize: |_, c, out| {
    let s = c.state::<FullBackpack>();
    out.write_encoded_int(0); // version
    out.write_bool(s.bag);
    out.write_bool(s.powder);
    out.write_bool(s.gold);
});

conv!(END2_CONVERSATION, "End2Conversation", msg 1054078, on_read: |w, q, _| q.complete(w));

pub static CONVERSATIONS: &[&ConvClass] = &[
    &DONT_OFFER_CONVERSATION,
    &ACCEPT_CONVERSATION,
    &DURING_KILL_QUEENS_CONVERSATION,
    &GATHER_FUNGI_CONVERSATION,
    &DURING_FUNGI_GATHERING_CONVERSATION,
    &END_CONVERSATION,
    &FULL_BACKPACK_CONVERSATION,
    &END2_CONVERSATION,
];

// ---- Objectives ----

fn queen_class(q: &Quest) -> &'static str {
    if red_solen(q) {
        "Server.Mobiles.RedSolenQueen"
    } else {
        "Server.Mobiles.BlackSolenQueen"
    }
}

obj!(KILL_QUEENS_OBJECTIVE, "KillQueensObjective",
    message: |q, _| if red_solen(q) { 1054062 } else { 1054063 },
    max_progress: |_, _| 5,
    render_progress: |q, o, b| {
        if !o.completed(q) {
            // Red/Black Solen Queens killed:
            b.add_html_localized_color(70, 260, 270, 100, if red_solen(q) { 1054064 } else { 1054065 }, BLUE, false, false);
            b.add_label(70, 280, 0x64, &o.cur_progress.to_string());
            b.add_label(100, 280, 0x64, "/");
            b.add_label(130, 280, 0x64, &o.max_progress(q).to_string());
        } else {
            base_render_progress(q, o, b);
        }
    },
    ignore_young_protection: |w, q, i, from| {
        !q.objectives[i].completed(q) && mobile_is(w, from, queen_class(q))
    },
    on_kill: |w, q, i, creature, _| {
        if mobile_is(w, creature, queen_class(q)) {
            q.add_progress(w, i);
        }
    },
    on_complete: |w, q, _| q.add_objective(w, Objective::new(&RETURN_AFTER_KILLS_OBJECTIVE)));

obj!(RETURN_AFTER_KILLS_OBJECTIVE, "ReturnAfterKillsObjective", msg 1054067,
    on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&GATHER_FUNGI_CONVERSATION)));

obj!(GATHER_FUNGI_OBJECTIVE, "GatherFungiObjective", msg 1054069,
    on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&END_CONVERSATION)));

/// `GetRewardObjective`'s `BagOfSending`, `PowderOfTranslocation` and `Gold`.
#[derive(Clone, Default)]
pub struct Reward {
    pub bag: bool,
    pub powder: bool,
    pub gold: bool,
}

/// `new GetRewardObjective(bagOfSending, powderOfTranslocation, gold)`.
pub fn get_reward(bag: bool, powder: bool, gold: bool) -> Objective {
    Objective::with(&GET_REWARD_OBJECTIVE, Reward { bag, powder, gold })
}

obj!(GET_REWARD_OBJECTIVE, "GetRewardObjective", msg 1054148,
new_state: || St::new(Reward::default()),
on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&END2_CONVERSATION)),
child_deserialize: |_, o, r| {
    r.read_encoded_int()?; // version
    let s = o.state_mut::<Reward>();
    s.bag = r.read_bool()?;
    s.powder = r.read_bool()?;
    s.gold = r.read_bool()?;
    Ok(())
},
child_serialize: |_, o, out| {
    let s = o.state::<Reward>();
    out.write_encoded_int(0); // version
    out.write_bool(s.bag);
    out.write_bool(s.powder);
    out.write_bool(s.gold);
});

pub static OBJECTIVES: &[&ObjClass] = &[
    &KILL_QUEENS_OBJECTIVE,
    &RETURN_AFTER_KILLS_OBJECTIVE,
    &GATHER_FUNGI_OBJECTIVE,
    &GET_REWARD_OBJECTIVE,
];

// ---- The quest ----

/// `AmbitiousQueenQuest`: the state is `RedSolen`.
pub static QUEST: QClass = QClass {
    name: "Server.Engines.Quests.Ambitious.AmbitiousQueenQuest",
    slot: 7,
    name_msg: Msg::Num(1054146),
    offer_message: |_, _| Msg::Num(1054060),
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

// ---- Mobiles (AmbitiousSolenQueen) ----

use muo_server::ids::ItemId;
use muo_server::mobile::MobileType;

use super::core::{can_offer_quest, load, with_quest};
use super::quester::{BASE_QUESTER, QUESTER_C, QUESTER_OPS, QUESTER_VC, QuesterOps};

fn is_red(w: &World, id: MobileId) -> bool {
    w.mobile(id)
        .is_some_and(|m| m.ty.is(&RED_AMBITIOUS_SOLEN_QUEEN))
}

/// `BaseAmbitiousSolenQueen.InitBody()`.
fn init_body(w: &mut World, id: MobileId) {
    super::util::set_body(w, id, 0x30F);
    if !is_red_class(w, id) {
        muo_server::mobile::base::set_hue(w, id, 0x453);
    }
    if let Some(m) = w.mobile_mut(id) {
        m.speech_hue = 0;
    }
}

/// During construction the type is already set, so the abstract `RedSolen` reads the same as later.
fn is_red_class(w: &World, id: MobileId) -> bool {
    is_red(w, id)
}

fn queen_on_talk(w: &mut World, queen: MobileId, player: MobileId, _context_menu: bool) {
    let d = muo_content::mx::get_direction_to(w, queen, player);
    muo_content::mx::set_direction(w, queen, d);
    let red = is_red(w, queen);

    let ours = load(w, player).is_some_and(|q| q.is(&QUEST) && red_solen(&q) == red);
    if ours {
        with_quest(w, player, |w, qs| {
            if qs.is_objective_in_progress(&KILL_QUEENS_OBJECTIVE) {
                qs.add_conversation(w, Conversation::new(&DURING_KILL_QUEENS_CONVERSATION));
                return;
            }
            let after_kills = qs
                .find_objective(&RETURN_AFTER_KILLS_OBJECTIVE)
                .filter(|&i| !qs.objectives[i].completed(qs));
            if let Some(i) = after_kills {
                qs.complete_objective(w, i);
            } else if qs.is_objective_in_progress(&GATHER_FUNGI_OBJECTIVE) {
                qs.add_conversation(w, Conversation::new(&DURING_FUNGI_GATHERING_CONVERSATION));
            } else if let Some(i) = qs
                .find_objective(&GET_REWARD_OBJECTIVE)
                .filter(|&i| !qs.objectives[i].completed(qs))
            {
                let r = qs.objectives[i].state::<Reward>().clone();
                let (mut bag, mut powder, mut gold) = (r.bag, r.powder, r.gold);
                give_reward_to(w, player, &mut bag, &mut powder, &mut gold);
                *qs.objectives[i].state_mut::<Reward>() = Reward { bag, powder, gold };
                if !bag && !powder && !gold {
                    qs.complete_objective(w, i);
                } else {
                    qs.add_conversation(w, full_backpack(false, bag, powder, gold));
                }
            }
        });
    } else {
        let mut new_quest = new_quest(w, player, red);
        if !super::core::has_quest(w, player) && can_offer_quest(w, player, &QUEST) {
            new_quest.send_offer(w);
        } else {
            new_quest.add_conversation(w, Conversation::new(&DONT_OFFER_CONVERSATION));
        }
    }
}

fn queen_on_drag_drop(w: &mut World, queen: MobileId, from: MobileId, dropped: ItemId) -> bool {
    let d = muo_content::mx::get_direction_to(w, queen, from);
    muo_content::mx::set_direction(w, queen, d);
    let red = is_red(w, queen);

    let fallback = |w: &mut World| (BASE_QUESTER.on_drag_drop)(w, queen, from, dropped);
    let Some(qs) = load(w, from).filter(|q| q.is(&QUEST) && red_solen(q) == red) else {
        return fallback(w);
    };
    let Some(obj) = qs.find_objective(&GATHER_FUNGI_OBJECTIVE) else {
        return fallback(w);
    };
    if qs.objectives[obj].completed(&qs)
        || !super::util::item_is(w, dropped, "Server.Items.ZoogiFungus")
    {
        return fallback(w);
    }
    let amount = w.item(dropped).map_or(0, |i| i.amount);
    if amount < 50 {
        super::util::say_to_number(w, queen, from, 1054072);
        return false;
    }
    with_quest(w, from, |w, q| q.complete_objective(w, obj));
    muo_server::item::base::consume(w, dropped, 50);
    w.item(dropped).is_none_or(|i| i.deleted())
}

static QUEEN_OPS: QuesterOps = QuesterOps {
    on_talk: queen_on_talk,
    auto_talk_range: |_, _, _| -1,
    ..QUESTER_OPS
};

crate::quester! {
    /// `Server.Engines.Quests.Ambitious.BaseAmbitiousSolenQueen`.
    BASE_AMBITIOUS_SOLEN_QUEEN, BASE_AMBITIOUS_SOLEN_QUEEN_C, BASE_AMBITIOUS_SOLEN_QUEEN_V
        = "Server.Engines.Quests.Ambitious.BaseAmbitiousSolenQueen" : BASE_QUESTER / QUESTER_C / QUESTER_VC,
        ops QUEEN_OPS, encoded true, abstract;
    v { init_body: init_body, }
    c {
        default_name: |_, _| Some("an ambitious solen queen"),
        disallow_all_moves: |_, _| false,
    }
    m {
        get_idle_sound: |_, _| 0x10D,
        on_drag_drop: queen_on_drag_drop,
    }
}

crate::quester! {
    /// `Server.Engines.Quests.Ambitious.RedAmbitiousSolenQueen`.
    RED_AMBITIOUS_SOLEN_QUEEN, RED_AMBITIOUS_SOLEN_QUEEN_C, RED_AMBITIOUS_SOLEN_QUEEN_V
        = "Server.Engines.Quests.Ambitious.RedAmbitiousSolenQueen"
        : BASE_AMBITIOUS_SOLEN_QUEEN / BASE_AMBITIOUS_SOLEN_QUEEN_C / BASE_AMBITIOUS_SOLEN_QUEEN_V,
        ops QUEEN_OPS, encoded true, title None, extra |_, _| {};
    v { }
    c { }
    m { }
}

crate::quester! {
    /// `Server.Engines.Quests.Ambitious.BlackAmbitiousSolenQueen`.
    BLACK_AMBITIOUS_SOLEN_QUEEN, BLACK_AMBITIOUS_SOLEN_QUEEN_C, BLACK_AMBITIOUS_SOLEN_QUEEN_V
        = "Server.Engines.Quests.Ambitious.BlackAmbitiousSolenQueen"
        : BASE_AMBITIOUS_SOLEN_QUEEN / BASE_AMBITIOUS_SOLEN_QUEEN_C / BASE_AMBITIOUS_SOLEN_QUEEN_V,
        ops QUEEN_OPS, encoded true, title None, extra |_, _| {};
    v { }
    c { }
    m { }
}

/// The mobile classes of this folder.
pub static MOBILES: &[&MobileType] = &[
    &BASE_AMBITIOUS_SOLEN_QUEEN,
    &RED_AMBITIOUS_SOLEN_QUEEN,
    &BLACK_AMBITIOUS_SOLEN_QUEEN,
];
