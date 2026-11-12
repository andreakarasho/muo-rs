//! The Uzeraan Turmoil quest: the quest, its objectives and conversations
//! (namespace `Server.Engines.Quests.Haven`).

use muo_io::{GenericReader, GenericWriter};
use muo_primitives::{Point3D, TimeSpan};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::map::MapId;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};

use super::core::{
    ConvClass, Conversation, ItemInfo, Msg, ObjClass, Objective, QClass, QUEST_SYSTEM, Quest, St,
    base_accept, base_render_progress, info, never_logged,
};
use super::gumps::BLUE;
use super::util::{mobile_is, msg_hue};
use crate::{conversation, objective};

macro_rules! conv {
    ($n:ident, $s:literal $($t:tt)*) => {
        conversation!($n = concat!("Server.Engines.Quests.Haven.", $s) $($t)*);
    };
}
macro_rules! obj {
    ($n:ident, $s:literal $($t:tt)*) => {
        objective!($n = concat!("Server.Engines.Quests.Haven.", $s) $($t)*);
    };
}

const WARRIOR: i32 = 1;
const MAGICIAN: i32 = 2;
const PALADIN: i32 = 5;

/// `p.X >= x1 && p.X <= x2 && p.Y >= y1 && p.Y <= y2` on Trammel.
fn trammel_box(map: Option<MapId>, p: Point3D, x: (i32, i32), y: (i32, i32)) -> bool {
    map == Some(MapId::TRAMMEL) && p.x >= x.0 && p.x <= x.1 && p.y >= y.0 && p.y <= y.1
}

// ---- Has-lost checks ----

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

/// `UzeraanTurmoilQuest.HasLostScrollOfPower(from)`.
pub fn has_lost_scroll_of_power(w: &World, from: MobileId) -> bool {
    has_lost(
        w,
        from,
        &RETURN_SCROLL_OF_POWER_OBJECTIVE,
        "Server.Engines.Quests.Haven.SchmendrickScrollOfPower",
    )
}

/// `UzeraanTurmoilQuest.HasLostFertileDirt(from)`.
pub fn has_lost_fertile_dirt(w: &World, from: MobileId) -> bool {
    has_lost(
        w,
        from,
        &RETURN_FERTILE_DIRT_OBJECTIVE,
        "Server.Engines.Quests.Haven.QuestFertileDirt",
    )
}

/// `UzeraanTurmoilQuest.HasLostDaemonBlood(from)`.
pub fn has_lost_daemon_blood(w: &World, from: MobileId) -> bool {
    has_lost(
        w,
        from,
        &RETURN_DAEMON_BLOOD_OBJECTIVE,
        "Server.Engines.Quests.Haven.QuestDaemonBlood",
    )
}

/// `UzeraanTurmoilQuest.HasLostDaemonBone(from)`.
pub fn has_lost_daemon_bone(w: &World, from: MobileId) -> bool {
    has_lost(
        w,
        from,
        &RETURN_DAEMON_BONE_OBJECTIVE,
        "Server.Engines.Quests.Haven.QuestDaemonBone",
    )
}

// ---- Conversations ----

static FIRST_TASK_INFO: [ItemInfo; 1] = [info(1023676, 0xE68)]; // glowing rune
static REPORT_INFO: [ItemInfo; 2] = [
    info(1026153, 0x1822), // teleporter
    info(1048032, 0xE76),  // a bag
];
static SCHMENDRICK_INFO: [ItemInfo; 1] = [info(1023637, 0xE34)]; // scroll
static SCROLL_OF_POWER_INFO: [ItemInfo; 3] = [
    info(1048030, 0x14EB), // a Treasure Map
    info(1023969, 0xF81),  // Fertile Dirt
    info(1049117, 0xFC4),  // Horn of Retreat
];
static FERTILE_DIRT_INFO: [ItemInfo; 2] = [
    info(1023965, 0xF7D), // Daemon Blood
    info(1022581, 0xA22), // lantern
];
static DAEMON_BLOOD_INFO: [ItemInfo; 1] = [info(1017412, 0xF80)]; // Daemon Bone
static DAEMON_BLOOD_INFO_PALADIN: [ItemInfo; 2] = [
    info(1017412, 0xF80),  // Daemon Bone
    info(1060577, 0x1F14), // Recall Rune
];

conv!(ACCEPT_CONVERSATION, "AcceptConversation", msg 1049092,
    on_read: |w, q, _| q.add_objective(w, Objective::new(&FIND_UZERAAN_BEGIN_OBJECTIVE)));
conv!(UZERAAN_TITHE_CONVERSATION, "UzeraanTitheConversation", msg 1060209,
    on_read: |w, q, _| q.add_objective(w, tithe_gold()));
conv!(UZERAAN_FIRST_TASK_CONVERSATION, "UzeraanFirstTaskConversation",
    message: |q, _| match q.profession {
        WARRIOR => 1049088,
        MAGICIAN => 1049386,
        _ => 1060388,
    },
    info: |_, _| &FIRST_TASK_INFO,
    on_read: |w, q, _| q.add_objective(w, kill_horde_minions(KillHordeMinionsStep::First)));
conv!(UZERAAN_REPORT_CONVERSATION, "UzeraanReportConversation",
    message: |q, _| if q.profession == MAGICIAN { 1049387 } else { 1049119 },
    info: |_, _| &REPORT_INFO,
    on_read: |w, q, _| q.add_objective(w, Objective::new(&FIND_SCHMENDRICK_OBJECTIVE)));
conv!(SCHMENDRICK_CONVERSATION, "SchmendrickConversation",
    message: |q, _| if q.profession == PALADIN { 1060749 } else { 1049322 },
    info: |_, _| &SCHMENDRICK_INFO,
    on_read: |w, q, _| q.add_objective(w, Objective::new(&FIND_APPRENTICE_OBJECTIVE)));
conv!(UZERAAN_SCROLL_OF_POWER_CONVERSATION, "UzeraanScrollOfPowerConversation", msg 1049325,
    info: |_, _| &SCROLL_OF_POWER_INFO,
    on_read: |w, q, _| q.add_objective(w, Objective::new(&FIND_DRYAD_OBJECTIVE)));
conv!(DRYAD_CONVERSATION, "DryadConversation", msg 1049326,
    on_read: |w, q, _| q.add_objective(w, Objective::new(&RETURN_FERTILE_DIRT_OBJECTIVE)));
conv!(UZERAAN_FERTILE_DIRT_CONVERSATION, "UzeraanFertileDirtConversation",
    message: |q, _| if q.profession == MAGICIAN { 1049388 } else { 1049329 },
    info: |_, _| &FERTILE_DIRT_INFO,
    on_read: |w, q, _| q.add_objective(w, Objective::new(&GET_DAEMON_BLOOD_OBJECTIVE)));
conv!(UZERAAN_DAEMON_BLOOD_CONVERSATION, "UzeraanDaemonBloodConversation",
    message: |q, _| if q.profession == MAGICIAN { 1049389 } else { 1049333 },
    info: |q, _| if q.profession == PALADIN { &DAEMON_BLOOD_INFO_PALADIN } else { &DAEMON_BLOOD_INFO },
    on_read: |w, q, _| q.add_objective(w, Objective::new(&GET_DAEMON_BONE_OBJECTIVE)));
conv!(UZERAAN_DAEMON_BONE_CONVERSATION, "UzeraanDaemonBoneConversation", msg 1049335,
    on_read: |w, q, _| q.add_objective(w, Objective::new(&CASH_BANK_CHECK_OBJECTIVE)));
conv!(BANKER_CONVERSATION, "BankerConversation", msg 1060137, on_read: |w, q, _| q.complete(w));
conv!(RADAR_CONVERSATION, "RadarConversation", msg 1049660, logged: never_logged);

fn from_uzeraan_state() -> St {
    St::new(false)
}

fn from_uzeraan_de(
    _: &muo_server::World,
    c: &mut Conversation,
    r: &mut dyn GenericReader,
) -> muo_io::Result<()> {
    r.read_encoded_int()?; // version
    *c.st.get_mut::<bool>() = r.read_bool()?;
    Ok(())
}

fn from_uzeraan_ser(_: &muo_server::World, c: &Conversation, out: &mut dyn GenericWriter) {
    out.write_encoded_int(0); // version
    out.write_bool(*c.state::<bool>());
}

/// `new LostScrollOfPowerConversation(fromUzeraan)`.
pub fn lost_scroll_of_power(from_uzeraan: bool) -> Conversation {
    Conversation::with(&LOST_SCROLL_OF_POWER_CONVERSATION, from_uzeraan)
}

/// `new LostFertileDirtConversation(fromUzeraan)`.
pub fn lost_fertile_dirt(from_uzeraan: bool) -> Conversation {
    Conversation::with(&LOST_FERTILE_DIRT_CONVERSATION, from_uzeraan)
}

conv!(LOST_SCROLL_OF_POWER_CONVERSATION, "LostScrollOfPowerConversation",
    new_state: from_uzeraan_state,
    message: |_, c| if *c.state::<bool>() { 1049377 } else { 1049345 },
    logged: never_logged,
    child_deserialize: from_uzeraan_de,
    child_serialize: from_uzeraan_ser);
conv!(LOST_FERTILE_DIRT_CONVERSATION, "LostFertileDirtConversation",
    new_state: from_uzeraan_state,
    message: |_, c| if *c.state::<bool>() { 1049374 } else { 1049359 },
    logged: never_logged,
    child_deserialize: from_uzeraan_de,
    child_serialize: from_uzeraan_ser);
conv!(DRYAD_APPLE_CONVERSATION, "DryadAppleConversation", msg 1049360, logged: never_logged);
conv!(LOST_DAEMON_BLOOD_CONVERSATION, "LostDaemonBloodConversation", msg 1049375, logged: never_logged);
conv!(LOST_DAEMON_BONE_CONVERSATION, "LostDaemonBoneConversation", msg 1049376, logged: never_logged);
conv!(FEW_REAGENTS_CONVERSATION, "FewReagentsConversation", msg 1049390, logged: never_logged);

pub static CONVERSATIONS: &[&ConvClass] = &[
    &ACCEPT_CONVERSATION,
    &UZERAAN_TITHE_CONVERSATION,
    &UZERAAN_FIRST_TASK_CONVERSATION,
    &UZERAAN_REPORT_CONVERSATION,
    &SCHMENDRICK_CONVERSATION,
    &UZERAAN_SCROLL_OF_POWER_CONVERSATION,
    &DRYAD_CONVERSATION,
    &UZERAAN_FERTILE_DIRT_CONVERSATION,
    &UZERAAN_DAEMON_BLOOD_CONVERSATION,
    &UZERAAN_DAEMON_BONE_CONVERSATION,
    &BANKER_CONVERSATION,
    &RADAR_CONVERSATION,
    &LOST_SCROLL_OF_POWER_CONVERSATION,
    &LOST_FERTILE_DIRT_CONVERSATION,
    &DRYAD_APPLE_CONVERSATION,
    &LOST_DAEMON_BLOOD_CONVERSATION,
    &LOST_DAEMON_BONE_CONVERSATION,
    &FEW_REAGENTS_CONVERSATION,
];

// ---- Objectives ----

obj!(FIND_UZERAAN_BEGIN_OBJECTIVE, "FindUzeraanBeginObjective", msg 1046039,
on_complete: |w, q, _| {
    if q.profession == PALADIN {
        q.add_conversation(w, Conversation::new(&UZERAAN_TITHE_CONVERSATION));
    } else {
        q.add_conversation(w, Conversation::new(&UZERAAN_FIRST_TASK_CONVERSATION));
    }
});

/// `TitheGoldObjective.m_OldTithingPoints` (not saved).
#[derive(Clone)]
struct Tithe(i32);

/// `new TitheGoldObjective()`.
pub fn tithe_gold() -> Objective {
    Objective::with(&TITHE_GOLD_OBJECTIVE, Tithe(-1))
}

obj!(TITHE_GOLD_OBJECTIVE, "TitheGoldObjective", msg 1060386,
    new_state: || St::new(Tithe(-1)),
    check_progress: |w, q, i| {
        let pm = q.from;
        let cur = w.mobile(pm).map_or(0, |m| m.tithing_points);
        let old = q.objectives[i].state::<Tithe>().0;
        if cur >= 500 {
            q.complete_objective(w, i);
        } else if cur > old && old >= 0 {
            // You must have at least 500 tithing points before you can continue in your quest.
            msg_hue(w, pm, 1060240, 0x41);
        }
        q.objectives[i].state_mut::<Tithe>().0 = cur;
    },
    on_complete: |w, q, _| q.add_objective(w, Objective::new(&FIND_UZERAAN_FIRST_TASK_OBJECTIVE)));

obj!(FIND_UZERAAN_FIRST_TASK_OBJECTIVE, "FindUzeraanFirstTaskObjective", msg 1060387,
    on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&UZERAAN_FIRST_TASK_CONVERSATION)));

/// `KillHordeMinionsStep`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum KillHordeMinionsStep {
    First = 0,
    LearnKarma = 1,
    Others = 2,
}

fn step_of(o: &Objective) -> KillHordeMinionsStep {
    match *o.state::<i32>() {
        0 => KillHordeMinionsStep::First,
        1 => KillHordeMinionsStep::LearnKarma,
        _ => KillHordeMinionsStep::Others,
    }
}

/// `new KillHordeMinionsObjective(step)`.
pub fn kill_horde_minions(step: KillHordeMinionsStep) -> Objective {
    Objective::with(&KILL_HORDE_MINIONS_OBJECTIVE, step as i32)
}

/// `Haven island`.
fn on_haven_island(map: Option<MapId>, p: Point3D) -> bool {
    trammel_box(map, p, (3314, 3814), (2345, 3095))
}

obj!(KILL_HORDE_MINIONS_OBJECTIVE, "KillHordeMinionsObjective",
new_state: || St::new(0_i32),
message: |_, o| match step_of(o) {
    KillHordeMinionsStep::First => 1049089,
    KillHordeMinionsStep::LearnKarma => 1060389,
    KillHordeMinionsStep::Others => 1060507,
},
max_progress: |q, o| {
    if q.profession == PALADIN {
        match step_of(o) {
            KillHordeMinionsStep::First => 1,
            KillHordeMinionsStep::LearnKarma => 2,
            KillHordeMinionsStep::Others => 5,
        }
    } else {
        5
    }
},
completed: |q, o| {
    if step_of(o) == KillHordeMinionsStep::LearnKarma && o.has_been_read {
        return true;
    }
    o.cur_progress >= o.max_progress(q)
},
render_progress: |q, o, b| {
    if !o.completed(q) {
        // Horde Minions killed:
        super::gumps::add_html_object(b, 70, 260, 270, 100, Msg::Num(1049090), BLUE, false, false);
        b.add_label(70, 280, 0x64, &o.cur_progress.to_string());
    } else {
        base_render_progress(q, o, b);
    }
},
on_read: |w, q, i| q.check_completion_status(w, i),
ignore_young_protection: |w, _, _, from| {
    let (map, loc) = (muo_content::mx::map(w, from), muo_content::mx::location(w, from));
    mobile_is(w, from, "Server.Mobiles.HordeMinion") && on_haven_island(map, loc)
},
on_kill: |w, q, i, creature, corpse| {
    let Some(c) = corpse else { return };
    let Some(it) = w.item(c) else { return };
    if mobile_is(w, creature, "Server.Mobiles.HordeMinion") && on_haven_island(it.map, it.location) {
        if q.objectives[i].cur_progress == 0 {
            super::util::send_display_help_topic(w, q.from, 29, false); // HelpTopic.Healing
        }
        q.add_progress(w, i);
    }
},
on_complete: |w, q, i| {
    if q.profession == PALADIN {
        let progress = q.objectives[i].cur_progress;
        match step_of(&q.objectives[i]) {
            KillHordeMinionsStep::First => {
                q.add_objective(w, kill_horde_minions(KillHordeMinionsStep::LearnKarma));
                let n = q.objectives.len() - 1;
                q.set_progress(w, n, progress);
            }
            KillHordeMinionsStep::LearnKarma => {
                q.add_objective(w, kill_horde_minions(KillHordeMinionsStep::Others));
                let n = q.objectives.len() - 1;
                q.set_progress(w, n, progress);
            }
            KillHordeMinionsStep::Others => {
                q.add_objective(w, Objective::new(&FIND_UZERAAN_ABOUT_REPORT_OBJECTIVE));
            }
        }
    } else {
        q.add_objective(w, Objective::new(&FIND_UZERAAN_ABOUT_REPORT_OBJECTIVE));
    }
},
child_deserialize: |_, o, r| {
    r.read_encoded_int()?; // version
    *o.state_mut::<i32>() = r.read_encoded_int()?;
    Ok(())
},
child_serialize: |_, o, out| {
    out.write_encoded_int(0); // version
    out.write_encoded_int(*o.state::<i32>());
});

obj!(FIND_UZERAAN_ABOUT_REPORT_OBJECTIVE, "FindUzeraanAboutReportObjective", msg 1049091,
    on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&UZERAAN_REPORT_CONVERSATION)));

obj!(FIND_SCHMENDRICK_OBJECTIVE, "FindSchmendrickObjective", msg 1049120,
    ignore_young_protection: |w, q, i, from| {
        let (map, loc) = (muo_content::mx::map(w, from), muo_content::mx::location(w, from));
        q.objectives[i].completed(q)
            && mobile_is(w, from, "Server.Mobiles.RestlessSoul")
            && trammel_box(map, loc, (5199, 5271), (1812, 1865)) // Schmendrick's cave
    },
    on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&SCHMENDRICK_CONVERSATION)));

obj!(FIND_APPRENTICE_OBJECTIVE, "FindApprenticeObjective", msg 1049323,
    on_complete: |w, q, _| q.add_objective(w, Objective::new(&RETURN_SCROLL_OF_POWER_OBJECTIVE)));

obj!(RETURN_SCROLL_OF_POWER_OBJECTIVE, "ReturnScrollOfPowerObjective", msg 1049324,
    on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&UZERAAN_SCROLL_OF_POWER_CONVERSATION)));

obj!(FIND_DRYAD_OBJECTIVE, "FindDryadObjective", msg 1049358,
    on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&DRYAD_CONVERSATION)));

obj!(RETURN_FERTILE_DIRT_OBJECTIVE, "ReturnFertileDirtObjective", msg 1049327,
    on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&UZERAAN_FERTILE_DIRT_CONVERSATION)));

obj!(GET_DAEMON_BLOOD_OBJECTIVE, "GetDaemonBloodObjective", msg 1049361,
new_state: || St::new(false),
check_progress: |w, q, i| {
    let player = q.from;
    let ambushed = *q.objectives[i].state::<bool>();
    if !ambushed
        && muo_content::mx::map(w, player) == Some(MapId::TRAMMEL)
        && muo_content::mx::in_range_point(w, player, Point3D::new(3456, 2558, 50), 30)
    {
        let loc = muo_content::mx::location(w, player);
        let (x, y) = (loc.x - 1, loc.y - 2);
        let z = w.maps.get_average_z(MapId::TRAMMEL, x, y);
        if muo_server::map::fit::can_spawn_mobile_at(w, MapId::TRAMMEL, x, y, z) {
            *q.objectives[i].state_mut::<bool>() = true;
            // You have been ambushed! Fight for your honor!!!
            super::util::local_overhead_number(w, player, 0x3B2, 1049330);
            if let Some(c) = muo_content::factory::make_mobile(w, "Server.Mobiles.HordeMinion") {
                muo_server::mobile::base::move_to_world(w, c, Point3D::new(x, y, z), MapId::TRAMMEL);
                muo_content::mx::set_combatant(w, c, Some(player));
            }
        }
    }
},
on_complete: |w, q, _| q.add_objective(w, Objective::new(&RETURN_DAEMON_BLOOD_OBJECTIVE)),
child_deserialize: |_, o, r| {
    r.read_encoded_int()?; // version
    *o.state_mut::<bool>() = r.read_bool()?;
    Ok(())
},
child_serialize: |_, o, out| {
    out.write_encoded_int(0); // version
    out.write_bool(*o.state::<bool>());
});

obj!(RETURN_DAEMON_BLOOD_OBJECTIVE, "ReturnDaemonBloodObjective", msg 1049332,
    on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&UZERAAN_DAEMON_BLOOD_CONVERSATION)));

/// `GetDaemonBoneObjective.CorpseWithBone`.
pub fn corpse_with_bone(o: &Objective) -> Option<ItemId> {
    *o.state::<Option<ItemId>>()
}

obj!(GET_DAEMON_BONE_OBJECTIVE, "GetDaemonBoneObjective",
new_state: || St::new(None::<ItemId>),
message: |q, _| if q.profession == PALADIN { 1060755 } else { 1049362 },
on_complete: |w, q, _| q.add_objective(w, Objective::new(&RETURN_DAEMON_BONE_OBJECTIVE)),
ignore_young_protection: |w, _, _, from| {
    let (map, loc) = (muo_content::mx::map(w, from), muo_content::mx::location(w, from));
    (mobile_is(w, from, "Server.Mobiles.Zombie") || mobile_is(w, from, "Server.Mobiles.Skeleton"))
        && trammel_box(map, loc, (3391, 3424), (2639, 2664)) // Haven graveyard
},
get_kill_event: |w, q, i, _, _| !q.objectives[i].completed(q) || has_lost_daemon_bone(w, q.from),
on_kill: |w, q, i, creature, corpse| {
    let Some(c) = corpse else { return };
    let Some(it) = w.item(c) else { return };
    if (mobile_is(w, creature, "Server.Mobiles.Zombie") || mobile_is(w, creature, "Server.Mobiles.Skeleton"))
        && trammel_box(it.map, it.location, (3391, 3424), (2639, 2664)) // Haven graveyard
        && muo_content::hooks::rnd::double(w) < 0.25
    {
        *q.objectives[i].state_mut::<Option<ItemId>>() = Some(c);
    }
},
child_deserialize: |w, o, r| {
    r.read_encoded_int()?; // version
    *o.state_mut::<Option<ItemId>>() = r.read_item(w)?;
    Ok(())
},
child_serialize: |w, o, out| {
    let mut corpse = corpse_with_bone(o);
    if corpse.is_some_and(|c| w.item(c).is_none_or(|i| i.deleted())) {
        corpse = None;
    }
    out.write_encoded_int(0); // version
    out.write_item(w, corpse);
});

obj!(RETURN_DAEMON_BONE_OBJECTIVE, "ReturnDaemonBoneObjective", msg 1049334,
    on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&UZERAAN_DAEMON_BONE_CONVERSATION)));

obj!(CASH_BANK_CHECK_OBJECTIVE, "CashBankCheckObjective", msg 1049336,
    on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&BANKER_CONVERSATION)));

pub static OBJECTIVES: &[&ObjClass] = &[
    &FIND_UZERAAN_BEGIN_OBJECTIVE,
    &TITHE_GOLD_OBJECTIVE,
    &FIND_UZERAAN_FIRST_TASK_OBJECTIVE,
    &KILL_HORDE_MINIONS_OBJECTIVE,
    &FIND_UZERAAN_ABOUT_REPORT_OBJECTIVE,
    &FIND_SCHMENDRICK_OBJECTIVE,
    &FIND_APPRENTICE_OBJECTIVE,
    &RETURN_SCROLL_OF_POWER_OBJECTIVE,
    &FIND_DRYAD_OBJECTIVE,
    &RETURN_FERTILE_DIRT_OBJECTIVE,
    &GET_DAEMON_BLOOD_OBJECTIVE,
    &RETURN_DAEMON_BLOOD_OBJECTIVE,
    &GET_DAEMON_BONE_OBJECTIVE,
    &RETURN_DAEMON_BONE_OBJECTIVE,
    &CASH_BANK_CHECK_OBJECTIVE,
];

// ---- The quest ----

/// `UzeraanTurmoilQuest`: the state is `m_HasLeftTheMansion`.
pub static QUEST: QClass = QClass {
    name: "Server.Engines.Quests.Haven.UzeraanTurmoilQuest",
    slot: 2,
    name_msg: Msg::Num(1049007),
    offer_message: |_, _| Msg::Num(1049008),
    picture: |_, q| match q.profession {
        WARRIOR => 0x15C9,
        MAGICIAN => 0x15C1,
        _ => 0x15D3,
    },
    is_tutorial: true,
    restart_delay: TimeSpan::MAX,
    professions: &[WARRIOR, MAGICIAN, PALADIN],
    new_state: || St::new(false),
    slice: |w, q| {
        let from = q.from;
        let loc = muo_content::mx::location(w, from);
        let left = muo_content::mx::map(w, from) != Some(MapId::TRAMMEL)
            || loc.x < 3573
            || loc.x > 3611
            || loc.y < 2568
            || loc.y > 2606;
        if !*q.state::<bool>() && left {
            *q.state_mut::<bool>() = true;
            q.add_conversation(w, Conversation::new(&RADAR_CONVERSATION));
        }
        q.slice(w);
    },
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
        out.write_bool(*q.state::<bool>());
    },
    ..QUEST_SYSTEM
};
