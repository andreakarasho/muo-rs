//! The Haochi's Trials quest: the quest, its objectives and conversations
//! (namespace `Server.Engines.Quests.Samurai`).

use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::map::MapId;

use super::core::{
    ConvClass, Conversation, Msg, ObjClass, Objective, QClass, QUEST_SYSTEM, Quest, St,
    base_accept, bool_state, conv_bool_deserialize, conv_bool_serialize, never_logged,
    obj_bool_deserialize, obj_bool_serialize,
};
use super::util::mobile_is;
use crate::{conversation, objective};

macro_rules! conv {
    ($n:ident, $s:literal $($t:tt)*) => {
        conversation!($n = concat!("Server.Engines.Quests.Samurai.", $s) $($t)*);
    };
}
macro_rules! obj {
    ($n:ident, $s:literal $($t:tt)*) => {
        objective!($n = concat!("Server.Engines.Quests.Samurai.", $s) $($t)*);
    };
}

/// `HaochisTrialsQuest.HasLostHaochisKatana(from)`.
pub fn has_lost_haochis_katana(w: &World, from: MobileId) -> bool {
    let Some(q) = super::core::load(w, from) else {
        return false;
    };
    if !q.is(&QUEST) || !q.is_objective_in_progress(&FIFTH_TRIAL_RETURN_OBJECTIVE) {
        return false;
    }
    let pack = w.mobile(from).and_then(|m| m.backpack);
    pack.is_none_or(|p| {
        muo_content::factory::find_item_by_type(w, p, "Server.Engines.Quests.Samurai.HaochisKatana")
            .is_none()
    })
}

// ---- Conversations ----

conv!(ACCEPT_CONVERSATION, "AcceptConversation", msg 1049092,
    on_read: |w, q, _| q.add_objective(w, Objective::new(&FIND_HAOCHI_OBJECTIVE)));
conv!(RADAR_CONVERSATION, "RadarConversation", msg 1063033, logged: never_logged);
conv!(FIRST_TRIAL_INTRO_CONVERSATION, "FirstTrialIntroConversation", msg 1063029,
    on_read: |w, q, _| q.add_objective(w, Objective::new(&FIRST_TRIAL_INTRO_OBJECTIVE)));
conv!(FIRST_TRIAL_KILL_CONVERSATION, "FirstTrialKillConversation", msg 1063031,
    on_read: |w, q, _| q.add_objective(w, Objective::new(&FIRST_TRIAL_KILL_OBJECTIVE)));

/// `new GainKarmaConversation(cursedSoul)`.
pub fn gain_karma(cursed_soul: bool) -> Conversation {
    Conversation::with(&GAIN_KARMA_CONVERSATION, cursed_soul)
}

conv!(GAIN_KARMA_CONVERSATION, "GainKarmaConversation",
    new_state: bool_state,
    message: |_, c| if *c.state::<bool>() { 1063040 } else { 1063041 },
    logged: never_logged,
    child_deserialize: conv_bool_deserialize,
    child_serialize: conv_bool_serialize);

/// `new SecondTrialIntroConversation(cursedSoul)`.
pub fn second_trial_intro(cursed_soul: bool) -> Conversation {
    Conversation::with(&SECOND_TRIAL_INTRO_CONVERSATION, cursed_soul)
}

conv!(SECOND_TRIAL_INTRO_CONVERSATION, "SecondTrialIntroConversation",
    new_state: bool_state,
    message: |_, c| if *c.state::<bool>() { 1063045 } else { 1063046 },
    on_read: |w, q, _| q.add_objective(w, Objective::new(&SECOND_TRIAL_INTRO_OBJECTIVE)),
    child_deserialize: conv_bool_deserialize,
    child_serialize: conv_bool_serialize);

conv!(SECOND_TRIAL_ATTACK_CONVERSATION, "SecondTrialAttackConversation", msg 1063057,
    on_read: |w, q, _| q.add_objective(w, Objective::new(&SECOND_TRIAL_ATTACK_OBJECTIVE)));

/// `new ThirdTrialIntroConversation(dragon)`.
pub fn third_trial_intro(dragon: bool) -> Conversation {
    Conversation::with(&THIRD_TRIAL_INTRO_CONVERSATION, dragon)
}

conv!(THIRD_TRIAL_INTRO_CONVERSATION, "ThirdTrialIntroConversation",
    new_state: bool_state,
    message: |_, c| if *c.state::<bool>() { 1063060 } else { 1063059 },
    on_read: |w, q, _| q.add_objective(w, Objective::new(&THIRD_TRIAL_INTRO_OBJECTIVE)),
    child_deserialize: conv_bool_deserialize,
    child_serialize: conv_bool_serialize);

conv!(THIRD_TRIAL_KILL_CONVERSATION, "ThirdTrialKillConversation", msg 1063062,
    on_read: |w, q, _| q.add_objective(w, Objective::new(&THIRD_TRIAL_KILL_OBJECTIVE)));
conv!(FOURTH_TRIAL_INTRO_CONVERSATION, "FourthTrialIntroConversation", msg 1063065,
    on_read: |w, q, _| q.add_objective(w, Objective::new(&FOURTH_TRIAL_INTRO_OBJECTIVE)));
conv!(FOURTH_TRIAL_CATS_CONVERSATION, "FourthTrialCatsConversation", msg 1063067,
    on_read: |w, q, _| q.add_objective(w, Objective::new(&FOURTH_TRIAL_CATS_OBJECTIVE)));

/// `new FifthTrialIntroConversation(killedCat)`.
pub fn fifth_trial_intro(killed_cat: bool) -> Conversation {
    Conversation::with(&FIFTH_TRIAL_INTRO_CONVERSATION, killed_cat)
}

conv!(FIFTH_TRIAL_INTRO_CONVERSATION, "FifthTrialIntroConversation",
    new_state: bool_state,
    message: |_, c| if *c.state::<bool>() { 1063071 } else { 1063070 },
    on_read: |w, q, _| q.add_objective(w, Objective::new(&FIFTH_TRIAL_INTRO_OBJECTIVE)),
    child_deserialize: conv_bool_deserialize,
    child_serialize: conv_bool_serialize);

conv!(FIFTH_TRIAL_RETURN_CONVERSATION, "FifthTrialReturnConversation", msg 1063248,
    on_read: |w, q, _| q.add_objective(w, Objective::new(&FIFTH_TRIAL_RETURN_OBJECTIVE)));
conv!(LOST_SWORD_CONVERSATION, "LostSwordConversation", msg 1063074, logged: never_logged);

/// `new SixthTrialIntroConversation(stolenTreasure)`.
pub fn sixth_trial_intro(stolen_treasure: bool) -> Conversation {
    Conversation::with(&SIXTH_TRIAL_INTRO_CONVERSATION, stolen_treasure)
}

conv!(SIXTH_TRIAL_INTRO_CONVERSATION, "SixthTrialIntroConversation",
    new_state: bool_state,
    message: |_, c| if *c.state::<bool>() { 1063077 } else { 1063076 },
    on_read: |w, q, _| q.add_objective(w, Objective::new(&SIXTH_TRIAL_INTRO_OBJECTIVE)),
    child_deserialize: conv_bool_deserialize,
    child_serialize: conv_bool_serialize);

conv!(SEVENTH_TRIAL_INTRO_CONVERSATION, "SeventhTrialIntroConversation", msg 1063079,
    on_read: |w, q, _| q.add_objective(w, Objective::new(&SEVENTH_TRIAL_INTRO_OBJECTIVE)));
conv!(END_CONVERSATION, "EndConversation", msg 1063125, on_read: |w, q, _| q.complete(w));

pub static CONVERSATIONS: &[&ConvClass] = &[
    &ACCEPT_CONVERSATION,
    &RADAR_CONVERSATION,
    &FIRST_TRIAL_INTRO_CONVERSATION,
    &FIRST_TRIAL_KILL_CONVERSATION,
    &GAIN_KARMA_CONVERSATION,
    &SECOND_TRIAL_INTRO_CONVERSATION,
    &SECOND_TRIAL_ATTACK_CONVERSATION,
    &THIRD_TRIAL_INTRO_CONVERSATION,
    &THIRD_TRIAL_KILL_CONVERSATION,
    &FOURTH_TRIAL_INTRO_CONVERSATION,
    &FOURTH_TRIAL_CATS_CONVERSATION,
    &FIFTH_TRIAL_INTRO_CONVERSATION,
    &FIFTH_TRIAL_RETURN_CONVERSATION,
    &LOST_SWORD_CONVERSATION,
    &SIXTH_TRIAL_INTRO_CONVERSATION,
    &SEVENTH_TRIAL_INTRO_CONVERSATION,
    &END_CONVERSATION,
];

// ---- Objectives ----

obj!(FIND_HAOCHI_OBJECTIVE, "FindHaochiObjective", msg 1063026,
    on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&FIRST_TRIAL_INTRO_CONVERSATION)));
obj!(FIRST_TRIAL_INTRO_OBJECTIVE, "FirstTrialIntroObjective", msg 1063030,
    on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&FIRST_TRIAL_KILL_CONVERSATION)));

/// `FirstTrialKillObjective`'s `m_CursedSoulsKilled` and `m_YoungRoninKilled`.
#[derive(Clone, Default)]
struct FirstKills {
    cursed_souls: i32,
    young_ronin: i32,
}

obj!(FIRST_TRIAL_KILL_OBJECTIVE, "FirstTrialKillObjective", msg 1063032,
new_state: || St::new(FirstKills::default()),
max_progress: |_, _| 3,
on_kill: |w, q, i, creature, _| {
    if mobile_is(w, creature, "Server.Engines.Quests.Samurai.CursedSoul") {
        if q.objectives[i].state::<FirstKills>().cursed_souls == 0 {
            q.add_conversation(w, gain_karma(true));
        }
        let s = q.objectives[i].state_mut::<FirstKills>();
        s.cursed_souls += 1;
        let n = s.cursed_souls;
        muo_content::mx::msg_args(w, q.from, 1063038, &n.to_string());
    } else if mobile_is(w, creature, "Server.Engines.Quests.Samurai.YoungRonin") {
        if q.objectives[i].state::<FirstKills>().young_ronin == 0 {
            q.add_conversation(w, gain_karma(false));
        }
        let s = q.objectives[i].state_mut::<FirstKills>();
        s.young_ronin += 1;
        let n = s.young_ronin;
        muo_content::mx::msg_args(w, q.from, 1063039, &n.to_string());
    }
    let s = q.objectives[i].state::<FirstKills>();
    let progress = s.cursed_souls.max(s.young_ronin);
    q.set_progress(w, i, progress);
},
on_complete: |w, q, i| {
    let s = q.objectives[i].state::<FirstKills>();
    let cursed = s.cursed_souls > s.young_ronin;
    q.add_objective(w, Objective::with(&FIRST_TRIAL_RETURN_OBJECTIVE, cursed));
},
child_deserialize: |_, o, r| {
    r.read_encoded_int()?; // version
    let s = o.state_mut::<FirstKills>();
    s.cursed_souls = r.read_encoded_int()?;
    s.young_ronin = r.read_encoded_int()?;
    Ok(())
},
child_serialize: |_, o, out| {
    let s = o.state::<FirstKills>();
    out.write_encoded_int(0); // version
    out.write_encoded_int(s.cursed_souls);
    out.write_encoded_int(s.young_ronin);
});

obj!(FIRST_TRIAL_RETURN_OBJECTIVE, "FirstTrialReturnObjective", msg 1063044,
    new_state: bool_state,
    on_complete: |w, q, i| {
        let cursed = *q.objectives[i].state::<bool>();
        q.add_conversation(w, second_trial_intro(cursed));
    },
    child_deserialize: obj_bool_deserialize,
    child_serialize: obj_bool_serialize);

obj!(SECOND_TRIAL_INTRO_OBJECTIVE, "SecondTrialIntroObjective", msg 1063047,
    on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&SECOND_TRIAL_ATTACK_CONVERSATION)));
obj!(SECOND_TRIAL_ATTACK_OBJECTIVE, "SecondTrialAttackObjective", msg 1063058);

/// `SecondTrialReturnObjective.Dragon`.
pub fn dragon(o: &Objective) -> bool {
    *o.state::<bool>()
}

/// `new SecondTrialReturnObjective(dragon)`.
pub fn second_trial_return(dragon: bool) -> Objective {
    Objective::with(&SECOND_TRIAL_RETURN_OBJECTIVE, dragon)
}

obj!(SECOND_TRIAL_RETURN_OBJECTIVE, "SecondTrialReturnObjective", msg 1063229,
    new_state: bool_state,
    on_complete: |w, q, i| {
        let dragon = *q.objectives[i].state::<bool>();
        q.add_conversation(w, third_trial_intro(dragon));
    },
    child_deserialize: obj_bool_deserialize,
    child_serialize: obj_bool_serialize);

obj!(THIRD_TRIAL_INTRO_OBJECTIVE, "ThirdTrialIntroObjective", msg 1063061,
    on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&THIRD_TRIAL_KILL_CONVERSATION)));
obj!(THIRD_TRIAL_KILL_OBJECTIVE, "ThirdTrialKillObjective", msg 1063063,
    on_kill: |w, q, i, creature, _| {
        if mobile_is(w, creature, "Server.Engines.Quests.Samurai.InjuredWolf") {
            q.complete_objective(w, i);
        }
    },
    on_complete: |w, q, _| q.add_objective(w, Objective::new(&THIRD_TRIAL_RETURN_OBJECTIVE)));
obj!(THIRD_TRIAL_RETURN_OBJECTIVE, "ThirdTrialReturnObjective", msg 1063064,
    on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&FOURTH_TRIAL_INTRO_CONVERSATION)));
obj!(FOURTH_TRIAL_INTRO_OBJECTIVE, "FourthTrialIntroObjective", msg 1063066,
    on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&FOURTH_TRIAL_CATS_CONVERSATION)));
obj!(FOURTH_TRIAL_CATS_OBJECTIVE, "FourthTrialCatsObjective", msg 1063068,
on_kill: |w, q, i, creature, _| {
    if mobile_is(w, creature, "Server.Engines.Quests.Samurai.DiseasedCat") {
        q.complete_objective(w, i);
        q.add_objective(w, fourth_trial_return(true));
    }
});

/// `FourthTrialReturnObjective.KilledCat`.
pub fn killed_cat(o: &Objective) -> bool {
    *o.state::<bool>()
}

/// `new FourthTrialReturnObjective(killedCat)`.
pub fn fourth_trial_return(killed_cat: bool) -> Objective {
    Objective::with(&FOURTH_TRIAL_RETURN_OBJECTIVE, killed_cat)
}

obj!(FOURTH_TRIAL_RETURN_OBJECTIVE, "FourthTrialReturnObjective", msg 1063242,
    new_state: bool_state,
    on_complete: |w, q, i| {
        let cat = *q.objectives[i].state::<bool>();
        q.add_conversation(w, fifth_trial_intro(cat));
    },
    child_deserialize: obj_bool_deserialize,
    child_serialize: obj_bool_serialize);

/// `FifthTrialIntroObjective.StolenTreasure`.
pub fn stolen_treasure(o: &Objective) -> bool {
    *o.state::<bool>()
}

obj!(FIFTH_TRIAL_INTRO_OBJECTIVE, "FifthTrialIntroObjective", msg 1063072,
    new_state: bool_state,
    on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&FIFTH_TRIAL_RETURN_CONVERSATION)),
    child_deserialize: obj_bool_deserialize,
    child_serialize: obj_bool_serialize);

obj!(FIFTH_TRIAL_RETURN_OBJECTIVE, "FifthTrialReturnObjective", msg 1063073);
obj!(SIXTH_TRIAL_INTRO_OBJECTIVE, "SixthTrialIntroObjective", msg 1063078,
    on_complete: |w, q, _| q.add_objective(w, Objective::new(&SIXTH_TRIAL_RETURN_OBJECTIVE)));
obj!(SIXTH_TRIAL_RETURN_OBJECTIVE, "SixthTrialReturnObjective", msg 1063252,
    on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&SEVENTH_TRIAL_INTRO_CONVERSATION)));
obj!(SEVENTH_TRIAL_INTRO_OBJECTIVE, "SeventhTrialIntroObjective", msg 1063080,
    max_progress: |_, _| 3,
    on_kill: |w, q, i, creature, _| {
        if mobile_is(w, creature, "Server.Engines.Quests.Samurai.YoungNinja") {
            q.add_progress(w, i);
        }
    },
    on_complete: |w, q, _| q.add_objective(w, Objective::new(&SEVENTH_TRIAL_RETURN_OBJECTIVE)));
obj!(SEVENTH_TRIAL_RETURN_OBJECTIVE, "SeventhTrialReturnObjective", msg 1063253,
    on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&END_CONVERSATION)));

pub static OBJECTIVES: &[&ObjClass] = &[
    &FIND_HAOCHI_OBJECTIVE,
    &FIRST_TRIAL_INTRO_OBJECTIVE,
    &FIRST_TRIAL_KILL_OBJECTIVE,
    &FIRST_TRIAL_RETURN_OBJECTIVE,
    &SECOND_TRIAL_INTRO_OBJECTIVE,
    &SECOND_TRIAL_ATTACK_OBJECTIVE,
    &SECOND_TRIAL_RETURN_OBJECTIVE,
    &THIRD_TRIAL_INTRO_OBJECTIVE,
    &THIRD_TRIAL_KILL_OBJECTIVE,
    &THIRD_TRIAL_RETURN_OBJECTIVE,
    &FOURTH_TRIAL_INTRO_OBJECTIVE,
    &FOURTH_TRIAL_CATS_OBJECTIVE,
    &FOURTH_TRIAL_RETURN_OBJECTIVE,
    &FIFTH_TRIAL_INTRO_OBJECTIVE,
    &FIFTH_TRIAL_RETURN_OBJECTIVE,
    &SIXTH_TRIAL_INTRO_OBJECTIVE,
    &SIXTH_TRIAL_RETURN_OBJECTIVE,
    &SEVENTH_TRIAL_INTRO_OBJECTIVE,
    &SEVENTH_TRIAL_RETURN_OBJECTIVE,
];

// ---- The quest ----

/// `HaochisTrialsQuest`: the state is `m_SentRadarConversion`.
pub static QUEST: QClass = QClass {
    name: "Server.Engines.Quests.Samurai.HaochisTrialsQuest",
    slot: 9,
    name_msg: Msg::Num(1063022),
    offer_message: |_, _| Msg::Num(1063023),
    picture: |_, _| 0x15D7,
    is_tutorial: true,
    restart_delay: TimeSpan::MAX,
    professions: &[6], // samurai
    new_state: bool_state,
    accept: |w, q: &mut Quest| {
        base_accept(w, q);
        q.add_conversation(w, Conversation::new(&ACCEPT_CONVERSATION));
    },
    slice: |w, q| {
        let loc = muo_content::mx::location(w, q.from);
        let away = muo_content::mx::map(w, q.from) != Some(MapId::MALAS)
            || loc.x < 360
            || loc.x > 400
            || loc.y < 760
            || loc.y > 780;
        if !*q.state::<bool>() && away {
            *q.state_mut::<bool>() = true;
            q.add_conversation(w, Conversation::new(&RADAR_CONVERSATION));
        }
        q.slice(w);
    },
    child_deserialize: |_, q, r| {
        r.read_encoded_int()?; // version
        *q.state_mut::<bool>() = r.read_bool()?;
        Ok(())
    },
    child_serialize: |_, q, out| {
        out.write_encoded_int(0); // version
        out.write_bool(*q.state::<bool>());
    },
    ..QUEST_SYSTEM
};
