//! The Terrible Hatchlings quest: the quest, its objectives and conversations
//! (namespace `Server.Engines.Quests.Zento`).

use muo_content::gumps::DynamicGumpBuilder;
use muo_primitives::TimeSpan;
use muo_server::World;

use super::core::{
    ConvClass, Conversation, Msg, ObjClass, Objective, QClass, QUEST_SYSTEM, Quest, base_accept,
    base_render_progress, never_logged,
};
use super::util::mobile_is;
use crate::{conversation, objective};

// ---- Conversations ----

conversation!(ACCEPT_CONVERSATION = "Server.Engines.Quests.Zento.AcceptConversation", msg 1049092,
    on_read: |w, q, _| q.add_objective(w, Objective::new(&FIRST_KILL_OBJECTIVE)));
conversation!(DIRECTION_CONVERSATION = "Server.Engines.Quests.Zento.DirectionConversation", msg 1063323,
    logged: never_logged);
conversation!(TAKE_CARE_CONVERSATION = "Server.Engines.Quests.Zento.TakeCareConversation", msg 1063324,
    logged: never_logged);
conversation!(END_CONVERSATION = "Server.Engines.Quests.Zento.EndConversation", msg 1063321,
    on_read: |w, q, _| q.complete(w));

pub static CONVERSATIONS: &[&ConvClass] = &[
    &ACCEPT_CONVERSATION,
    &DIRECTION_CONVERSATION,
    &TAKE_CARE_CONVERSATION,
    &END_CONVERSATION,
];

// ---- Objectives ----

/// The progress block of the three kill objectives: "Deathwatch Beetle Hatchlings killed: n/10".
fn kills_progress(q: &Quest, b: &mut DynamicGumpBuilder, o: &Objective, shown: &str) {
    if !o.completed(q) {
        // Deathwatch Beetle Hatchlings killed:
        b.add_html_localized_color(70, 260, 270, 100, 1063318, 0xC6BF, false, false);
        b.add_label(70, 280, 0x64, shown);
        b.add_label(100, 280, 0x64, "/");
        b.add_label(130, 280, 0x64, "10");
    } else {
        base_render_progress(q, o, b);
    }
}

fn is_hatchling(w: &muo_server::World, m: muo_server::ids::MobileId) -> bool {
    mobile_is(w, m, "Server.Mobiles.DeathwatchBeetleHatchling")
}

objective!(FIRST_KILL_OBJECTIVE = "Server.Engines.Quests.Zento.FirstKillObjective", msg 1063316,
    render_progress: |q, o, b| kills_progress(q, b, o, "0"),
    on_kill: |w, q, i, creature, _| {
        if is_hatchling(w, creature) {
            q.complete_objective(w, i);
        }
    },
    on_complete: |w, q, _| q.add_objective(w, Objective::new(&SECOND_KILL_OBJECTIVE)));

objective!(SECOND_KILL_OBJECTIVE = "Server.Engines.Quests.Zento.SecondKillObjective", msg 1063320,
render_progress: |q, o, b| kills_progress(q, b, o, "1"),
on_kill: |w, q, i, creature, _| {
    if is_hatchling(w, creature) {
        q.complete_objective(w, i);
        q.add_objective(w, Objective::with_progress(&THIRD_KILL_OBJECTIVE, 2));
    }
},
on_read: |w, q, i| {
    if !q.objectives[i].completed(q) {
        q.complete_objective(w, i);
        q.add_objective(w, Objective::with_progress(&THIRD_KILL_OBJECTIVE, 1));
    }
});

objective!(THIRD_KILL_OBJECTIVE = "Server.Engines.Quests.Zento.ThirdKillObjective", msg 1063319,
    max_progress: |_, _| 10,
    render_progress: |q, o, b| kills_progress(q, b, o, &o.cur_progress.to_string()),
    on_kill: |w, q, i, creature, _| {
        if is_hatchling(w, creature) {
            q.add_progress(w, i);
        }
    },
    on_complete: |w, q, _| q.add_objective(w, Objective::new(&RETURN_OBJECTIVE)));

objective!(RETURN_OBJECTIVE = "Server.Engines.Quests.Zento.ReturnObjective", msg 1063313,
    on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&END_CONVERSATION)));

pub static OBJECTIVES: &[&ObjClass] = &[
    &FIRST_KILL_OBJECTIVE,
    &SECOND_KILL_OBJECTIVE,
    &THIRD_KILL_OBJECTIVE,
    &RETURN_OBJECTIVE,
];

// ---- The quest ----

/// `TerribleHatchlingsQuest`.
pub static QUEST: QClass = QClass {
    name: "Server.Engines.Quests.Zento.TerribleHatchlingsQuest",
    slot: 10,
    name_msg: Msg::Num(1063314),
    offer_message: |_, _| Msg::Num(1063315),
    picture: |_, _| 0x15CF,
    is_tutorial: true,
    restart_delay: TimeSpan::MAX,
    accept: |w, q: &mut Quest| {
        base_accept(w, q);
        q.add_conversation(w, Conversation::new(&ACCEPT_CONVERSATION));
    },
    ..QUEST_SYSTEM
};

// ---- Mobiles (AnsellaGryen) ----

use muo_server::ids::MobileId;
use muo_server::mobile::MobileType;

use super::core::{Quest as QuestT, can_offer_quest_ex, has_quest, load, with_quest};
use super::quester::{
    BASE_QUESTER, QUESTER_C, QUESTER_OPS, QUESTER_VC, QuesterOps, get_new_container,
};
use super::util::{add_item_hued, add_item_named, say_to_number, set_body};

fn ansella_on_talk(w: &mut World, quester: MobileId, player: MobileId, context_menu: bool) {
    let Some(qs) = load(w, player).filter(|q| q.is(&QUEST)) else {
        let new_quest = QuestT::new(w, &QUEST, player);
        if has_quest(w, player) {
            if context_menu {
                // Before you can help me with the Terrible Hatchlings, you'll need to finish the quest you've
                // already taken!
                say_to_number(w, quester, player, 1063322);
            }
        } else {
            let (can, in_restart) = can_offer_quest_ex(w, player, &QUEST);
            if can {
                new_quest.send_offer(w);
            } else if in_restart && context_menu {
                say_to_number(w, quester, player, 1049357); // I have nothing more for you at this time.
            }
        }
        return;
    };

    if qs.is_objective_in_progress(&FIRST_KILL_OBJECTIVE) {
        with_quest(w, player, |w, q| {
            q.add_conversation(w, Conversation::new(&DIRECTION_CONVERSATION))
        });
        return;
    }
    if qs.is_objective_in_progress(&SECOND_KILL_OBJECTIVE)
        || qs.is_objective_in_progress(&THIRD_KILL_OBJECTIVE)
    {
        with_quest(w, player, |w, q| {
            q.add_conversation(w, Conversation::new(&TAKE_CARE_CONVERSATION))
        });
        return;
    }

    let Some(obj) = qs
        .find_objective(&RETURN_OBJECTIVE)
        .filter(|&i| !qs.objectives[i].completed(&qs))
    else {
        return;
    };
    let cont = get_new_container(w);
    let gold = super::util::random_min_max(w, 100, 200);
    if let Some(g) = muo_content::factory::make_item(w, "Server.Items.Gold") {
        muo_server::item::ops::set_amount(w, g, gold);
        muo_server::item::container::drop_item_default(w, cont, g);
    }

    if muo_content::hooks::rnd::bool(w) {
        let weapon = muo_content::loot::construct_any(w, muo_content::loot::SE_WEAPON_TYPES);
        if let Some(weapon) = weapon {
            apply_runic(w, weapon, 3, 10, 30);
            muo_server::item::container::drop_item_default(w, cont, weapon);
        }
    } else if let Some(armor) =
        muo_content::loot::construct_any(w, muo_content::loot::SE_ARMOR_TYPES)
    {
        apply_runic(w, armor, 1, 10, 20);
        muo_server::item::container::drop_item_default(w, cont, armor);
    }

    if muo_server::mobile::part2::items::place_in_backpack(w, player, cont) {
        with_quest(w, player, |w, q| q.complete_objective(w, obj));
    } else {
        muo_server::item::base::delete(w, cont);
        // You need to clear some space in your inventory to continue with the quest.  Come back here when you have
        // more space in your inventory.
        muo_content::mx::msg(w, player, 1046260);
    }
}

/// `BaseRunicTool.ApplyAttributesTo(item, attributeCount, min, max)`.
fn apply_runic(w: &mut World, item: muo_server::ids::ItemId, count: i32, min: i32, max: i32) {
    if let Some(f) = super::hooks::quest_hooks(w).and_then(|h| h.apply_runic_attributes) {
        f(w, item, count, min, max);
    }
}

static ANSELLA_OPS: QuesterOps = QuesterOps {
    on_talk: ansella_on_talk,
    auto_talk_range: |w, _, m| if has_quest(w, m) { -1 } else { 3 },
    ..QUESTER_OPS
};

crate::quester! {
    /// `Server.Engines.Quests.Zento.AnsellaGryen`.
    ANSELLA_GRYEN, ANSELLA_GRYEN_C, ANSELLA_GRYEN_V = "Server.Engines.Quests.Zento.AnsellaGryen"
        : BASE_QUESTER / QUESTER_C / QUESTER_VC,
        ops ANSELLA_OPS, encoded true, title None, extra |_, _| {};
    v {
        turn_to_tokuno: |_, _| {}, // TurnToTokuno() {}: she keeps her name on the Tokuno map
        init_body: |w, id| {
            muo_server::mobile::part2::stats::init_stats(w, id, 100, 100, 25);
            muo_server::mobile::base::set_hue(w, id, 0x83EA);
            muo_server::mobile::part1::props_b::set_female(w, id, true);
            set_body(w, id, 0x191);
        },
        init_outfit: |w, id| {
            muo_server::mobile::part1::props_b::set_hair_item_id(w, id, 0x203B);
            muo_server::mobile::part1::props_b::set_hair_hue(w, id, 0x1BB);
            add_item_hued(w, id, "Server.Items.SamuraiTabi", 0x8FD);
            add_item_hued(w, id, "Server.Items.FemaleKimono", 0x4B6);
            add_item_hued(w, id, "Server.Items.Obi", 0x526);
            add_item_named(w, id, "Server.Items.GoldBracelet");
        },
    }
    c { default_name: |_, _| Some("Ansella Gryen"), }
    m { }
}

/// The mobile classes of this folder.
pub static MOBILES: &[&MobileType] = &[&ANSELLA_GRYEN];
