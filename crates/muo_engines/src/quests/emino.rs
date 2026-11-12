//! The Emino's Undertaking quest: the quest, its objectives and conversations
//! (namespace `Server.Engines.Quests.Ninja`).

use muo_primitives::{Point3D, TimeSpan};
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::map::MapId;

use super::core::{
    ConvClass, Conversation, Msg, ObjClass, Objective, QClass, QUEST_SYSTEM, Quest, St,
    base_accept, base_render_progress, never_logged,
};
use super::gumps::BLUE;
use super::util::mobile_is;
use crate::{conversation, objective};

macro_rules! conv {
    ($n:ident, $s:literal $($t:tt)*) => {
        conversation!($n = concat!("Server.Engines.Quests.Ninja.", $s) $($t)*);
    };
}
macro_rules! obj {
    ($n:ident, $s:literal $($t:tt)*) => {
        objective!($n = concat!("Server.Engines.Quests.Ninja.", $s) $($t)*);
    };
}

fn in_malas(w: &World, m: MobileId) -> bool {
    muo_content::mx::map(w, m) == Some(MapId::MALAS)
}

fn near(w: &World, q: &Quest, p: Point3D, range: i32) -> bool {
    in_malas(w, q.from) && muo_content::mx::in_range_point(w, q.from, p, range)
}

fn has_lost(w: &World, from: MobileId, objective: &ObjClass, item: &str) -> bool {
    let Some(q) = super::core::load(w, from) else {
        return false;
    };
    if !q.is(&QUEST) || !q.is_objective_in_progress(objective) {
        return false;
    }
    let pack = w.mobile(from).and_then(|m| m.backpack);
    pack.is_none_or(|p| muo_content::factory::find_item_by_type(w, p, item).is_none())
}

/// `EminosUndertakingQuest.HasLostNoteForZoel(from)`.
pub fn has_lost_note_for_zoel(w: &World, from: MobileId) -> bool {
    has_lost(
        w,
        from,
        &GIVE_ZOEL_NOTE_OBJECTIVE,
        "Server.Engines.Quests.Ninja.NoteForZoel",
    )
}

/// `EminosUndertakingQuest.HasLostEminosKatana(from)`.
pub fn has_lost_eminos_katana(w: &World, from: MobileId) -> bool {
    has_lost(
        w,
        from,
        &GIVE_EMINO_SWORD_OBJECTIVE,
        "Server.Engines.Quests.Ninja.EminosKatana",
    )
}

// ---- Conversations ----

conv!(ACCEPT_CONVERSATION, "AcceptConversation", msg 1049092,
    on_read: |w, q, _| q.add_objective(w, Objective::new(&FIND_EMINO_BEGIN_OBJECTIVE)));
conv!(FIND_ZOEL_CONVERSATION, "FindZoelConversation", msg 1063175,
    on_read: |w, q, _| q.add_objective(w, Objective::new(&FIND_ZOEL_OBJECTIVE)));
conv!(RADAR_CONVERSATION, "RadarConversation", msg 1063033, logged: never_logged);
conv!(ENTER_CAVE_CONVERSATION, "EnterCaveConversation", msg 1063177,
    on_read: |w, q, _| q.add_objective(w, Objective::new(&ENTER_CAVE_OBJECTIVE)));
conv!(SNEAK_PAST_GUARDIANS_CONVERSATION, "SneakPastGuardiansConversation", msg 1063180,
    on_read: |w, q, _| q.add_objective(w, sneak_past_guardians()));
conv!(NEED_TO_HIDE_CONVERSATION, "NeedToHideConversation", msg 1063181);
conv!(USE_TELEPORTER_CONVERSATION, "UseTeleporterConversation", msg 1063182,
    on_read: |w, q, _| q.add_objective(w, Objective::new(&USE_TELEPORTER_OBJECTIVE)));
conv!(GIVE_ZOEL_NOTE_CONVERSATION, "GiveZoelNoteConversation", msg 1063184,
    on_read: |w, q, _| q.add_objective(w, Objective::new(&GIVE_ZOEL_NOTE_OBJECTIVE)));
conv!(LOST_NOTE_CONVERSATION, "LostNoteConversation", msg 1063187, logged: never_logged);
conv!(GAIN_INN_INFORMATION_CONVERSATION, "GainInnInformationConversation", msg 1063189,
    on_read: |w, q, _| q.add_objective(w, Objective::new(&GAIN_INN_INFORMATION_OBJECTIVE)));
conv!(RETURN_FROM_INN_CONVERSATION, "ReturnFromInnConversation", msg 1063196,
    on_read: |w, q, _| q.add_objective(w, Objective::new(&RETURN_FROM_INN_OBJECTIVE)));
conv!(SEARCH_FOR_SWORD_CONVERSATION, "SearchForSwordConversation", msg 1063199,
    on_read: |w, q, _| q.add_objective(w, Objective::new(&SEARCH_FOR_SWORD_OBJECTIVE)));
conv!(HALLWAY_WALK_CONVERSATION, "HallwayWalkConversation", msg 1063201,
    on_read: |w, q, _| q.add_objective(w, hallway_walk()));
conv!(RETURN_SWORD_CONVERSATION, "ReturnSwordConversation", msg 1063203,
    on_read: |w, q, _| q.add_objective(w, Objective::new(&RETURN_SWORD_OBJECTIVE)));
conv!(SLAY_HENCHMEN_CONVERSATION, "SlayHenchmenConversation", msg 1063205,
    on_read: |w, q, _| q.add_objective(w, Objective::new(&SLAY_HENCHMEN_OBJECTIVE)));
conv!(CONTINUE_SLAY_HENCHMEN_CONVERSATION, "ContinueSlayHenchmenConversation", msg 1063208, logged: never_logged);
conv!(GIVE_EMINO_SWORD_CONVERSATION, "GiveEminoSwordConversation", msg 1063211,
    on_read: |w, q, _| q.add_objective(w, Objective::new(&GIVE_EMINO_SWORD_OBJECTIVE)));
conv!(LOST_SWORD_CONVERSATION, "LostSwordConversation", msg 1063212, logged: never_logged);
conv!(EARN_GIFTS_CONVERSATION, "EarnGiftsConversation", msg 1063216, on_read: |w, q, _| q.complete(w));
conv!(EARN_LESS_GIFTS_CONVERSATION, "EarnLessGiftsConversation", msg 1063217, on_read: |w, q, _| q.complete(w));

pub static CONVERSATIONS: &[&ConvClass] = &[
    &ACCEPT_CONVERSATION,
    &FIND_ZOEL_CONVERSATION,
    &RADAR_CONVERSATION,
    &ENTER_CAVE_CONVERSATION,
    &SNEAK_PAST_GUARDIANS_CONVERSATION,
    &NEED_TO_HIDE_CONVERSATION,
    &USE_TELEPORTER_CONVERSATION,
    &GIVE_ZOEL_NOTE_CONVERSATION,
    &LOST_NOTE_CONVERSATION,
    &GAIN_INN_INFORMATION_CONVERSATION,
    &RETURN_FROM_INN_CONVERSATION,
    &SEARCH_FOR_SWORD_CONVERSATION,
    &HALLWAY_WALK_CONVERSATION,
    &RETURN_SWORD_CONVERSATION,
    &SLAY_HENCHMEN_CONVERSATION,
    &CONTINUE_SLAY_HENCHMEN_CONVERSATION,
    &GIVE_EMINO_SWORD_CONVERSATION,
    &LOST_SWORD_CONVERSATION,
    &EARN_GIFTS_CONVERSATION,
    &EARN_LESS_GIFTS_CONVERSATION,
];

// ---- Objectives ----

obj!(FIND_EMINO_BEGIN_OBJECTIVE, "FindEminoBeginObjective", msg 1063174,
    on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&FIND_ZOEL_CONVERSATION)));
obj!(FIND_ZOEL_OBJECTIVE, "FindZoelObjective", msg 1063176,
    on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&ENTER_CAVE_CONVERSATION)));
obj!(ENTER_CAVE_OBJECTIVE, "EnterCaveObjective", msg 1063179,
    check_progress: |w, q, i| {
        if near(w, q, Point3D::new(406, 1141, 0), 2) {
            q.complete_objective(w, i);
        }
    },
    on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&SNEAK_PAST_GUARDIANS_CONVERSATION)));

/// `SneakPastGuardiansObjective.TaughtHowToUseSkills`.
pub fn taught_how_to_use_skills(o: &Objective) -> bool {
    *o.state::<bool>()
}

/// `new SneakPastGuardiansObjective()`.
pub fn sneak_past_guardians() -> Objective {
    Objective::with(&SNEAK_PAST_GUARDIANS_OBJECTIVE, false)
}

obj!(SNEAK_PAST_GUARDIANS_OBJECTIVE, "SneakPastGuardiansObjective", msg 1063261,
new_state: || St::new(false),
check_progress: |w, q, i| {
    if near(w, q, Point3D::new(412, 1123, 0), 3) {
        q.complete_objective(w, i);
    }
},
on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&USE_TELEPORTER_CONVERSATION)),
child_deserialize: |_, o, r| {
    r.read_encoded_int()?; // version
    *o.state_mut::<bool>() = r.read_bool()?;
    Ok(())
},
child_serialize: |_, o, out| {
    out.write_encoded_int(0); // version
    out.write_bool(*o.state::<bool>());
});

obj!(USE_TELEPORTER_OBJECTIVE, "UseTeleporterObjective", msg 1063183,
    on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&GIVE_ZOEL_NOTE_CONVERSATION)));
obj!(GIVE_ZOEL_NOTE_OBJECTIVE, "GiveZoelNoteObjective", msg 1063185,
    on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&GAIN_INN_INFORMATION_CONVERSATION)));
obj!(GAIN_INN_INFORMATION_OBJECTIVE, "GainInnInformationObjective", msg 1063190,
    check_progress: |w, q, i| {
        let loc = muo_content::mx::location(w, q.from);
        if in_malas(w, q.from) && loc.x > 399 && loc.x < 408 && loc.y > 1091 && loc.y < 1099 {
            q.complete_objective(w, i);
        }
    },
    on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&RETURN_FROM_INN_CONVERSATION)));
obj!(RETURN_FROM_INN_OBJECTIVE, "ReturnFromInnObjective", msg 1063197,
    on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&SEARCH_FOR_SWORD_CONVERSATION)));
obj!(SEARCH_FOR_SWORD_OBJECTIVE, "SearchForSwordObjective", msg 1063200,
    on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&HALLWAY_WALK_CONVERSATION)));

/// `HallwayWalkObjective.StolenTreasure`.
pub fn stolen_treasure(o: &Objective) -> bool {
    *o.state::<bool>()
}

/// `new HallwayWalkObjective()`.
pub fn hallway_walk() -> Objective {
    Objective::with(&HALLWAY_WALK_OBJECTIVE, false)
}

obj!(HALLWAY_WALK_OBJECTIVE, "HallwayWalkObjective", msg 1063202,
new_state: || St::new(false),
on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&RETURN_SWORD_CONVERSATION)),
child_deserialize: |_, o, r| {
    r.read_encoded_int()?; // version
    *o.state_mut::<bool>() = r.read_bool()?;
    Ok(())
},
child_serialize: |_, o, out| {
    out.write_encoded_int(0); // version
    out.write_bool(*o.state::<bool>());
});

obj!(RETURN_SWORD_OBJECTIVE, "ReturnSwordObjective", msg 1063204,
    check_progress: |w, q, i| {
        let loc = muo_content::mx::location(w, q.from);
        if !in_malas(w, q.from) || loc.y > 992 {
            q.complete_objective(w, i);
        }
    },
    on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&SLAY_HENCHMEN_CONVERSATION)));

obj!(SLAY_HENCHMEN_OBJECTIVE, "SlayHenchmenObjective", msg 1063206,
    max_progress: |_, _| 3,
    render_progress: |q, o, b| {
        if !o.completed(q) {
            b.add_html_localized_color(70, 260, 270, 100, 1063207, BLUE, false, false);
            b.add_label(70, 280, 0x64, &o.cur_progress.to_string());
            b.add_label(100, 280, 0x64, "/");
            b.add_label(130, 280, 0x64, &o.max_progress(q).to_string());
        } else {
            base_render_progress(q, o, b);
        }
    },
    on_kill: |w, q, i, creature, _| {
        if mobile_is(w, creature, "Server.Engines.Quests.Ninja.Henchman") {
            q.add_progress(w, i);
        }
    },
    on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&GIVE_EMINO_SWORD_CONVERSATION)));

obj!(GIVE_EMINO_SWORD_OBJECTIVE, "GiveEminoSwordObjective", msg 1063210);

pub static OBJECTIVES: &[&ObjClass] = &[
    &FIND_EMINO_BEGIN_OBJECTIVE,
    &FIND_ZOEL_OBJECTIVE,
    &ENTER_CAVE_OBJECTIVE,
    &SNEAK_PAST_GUARDIANS_OBJECTIVE,
    &USE_TELEPORTER_OBJECTIVE,
    &GIVE_ZOEL_NOTE_OBJECTIVE,
    &GAIN_INN_INFORMATION_OBJECTIVE,
    &RETURN_FROM_INN_OBJECTIVE,
    &SEARCH_FOR_SWORD_OBJECTIVE,
    &HALLWAY_WALK_OBJECTIVE,
    &RETURN_SWORD_OBJECTIVE,
    &SLAY_HENCHMEN_OBJECTIVE,
    &GIVE_EMINO_SWORD_OBJECTIVE,
];

// ---- The quest ----

/// `EminosUndertakingQuest`: the state is `m_SentRadarConversion`.
pub static QUEST: QClass = QClass {
    name: "Server.Engines.Quests.Ninja.EminosUndertakingQuest",
    slot: 8,
    name_msg: Msg::Num(1063173),
    offer_message: |_, _| Msg::Num(1063174),
    picture: |_, _| 0x15D5,
    is_tutorial: true,
    restart_delay: TimeSpan::MAX,
    professions: &[7], // ninja
    new_state: || St::new(false),
    accept: |w, q: &mut Quest| {
        base_accept(w, q);
        q.add_conversation(w, Conversation::new(&ACCEPT_CONVERSATION));
    },
    slice: |w, q| {
        let loc = muo_content::mx::location(w, q.from);
        let away = !in_malas(w, q.from) || loc.x < 407 || loc.x > 431 || loc.y < 801 || loc.y > 830;
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
