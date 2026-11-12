//! The Dark Tides quest: the quest, its objectives and conversations
//! (namespace `Server.Engines.Quests.Necro`).

use muo_primitives::{Point2D, Point3D, Rectangle2D, TimeSpan};
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::map::MapId;

use super::core::{
    ConvClass, Conversation, ItemInfo, Msg, ObjClass, Objective, QClass, QUEST_SYSTEM, Quest, St,
    base_accept, base_ignore_young_protection, info, never_logged,
};
use super::hooks::quest_hooks;
use super::util::mobile_is;
use crate::{conversation, objective};

macro_rules! conv {
    ($n:ident, $s:literal $($t:tt)*) => {
        conversation!($n = concat!("Server.Engines.Quests.Necro.", $s) $($t)*);
    };
}
macro_rules! obj {
    ($n:ident, $s:literal $($t:tt)*) => {
        objective!($n = concat!("Server.Engines.Quests.Necro.", $s) $($t)*);
    };
}

/// `Map == Map.Malas && From.InRange(p, range)`.
fn near_in_malas(w: &World, q: &Quest, p: Point3D, range: i32) -> bool {
    muo_content::mx::map(w, q.from) == Some(MapId::MALAS)
        && muo_content::mx::in_range_point(w, q.from, p, range)
}

/// `DarkTidesQuest.HasLostCallingScroll(from)`.
pub fn has_lost_calling_scroll(w: &World, from: MobileId) -> bool {
    let Some(q) = super::core::load(w, from) else {
        return false;
    };
    if !q.is(&QUEST) {
        return false;
    }
    if q.is_objective_in_progress(&FIND_MARDOTH_ABOUT_KRONUS_OBJECTIVE)
        || q.is_objective_in_progress(&FIND_WELL_OF_TEARS_OBJECTIVE)
        || q.is_objective_in_progress(&USE_CALLING_SCROLL_OBJECTIVE)
    {
        let pack = w.mobile(from).and_then(|m| m.backpack);
        return pack.is_none_or(|p| {
            muo_content::factory::find_item_by_type(
                w,
                p,
                "Server.Engines.Quests.Necro.KronusScroll",
            )
            .is_none()
        });
    }
    false
}

// ---- Conversations ----

static TELEPORTER_INFO: [ItemInfo; 1] = [info(1026153, 6178)]; // teleporter
static REANIMATE_INFO: [ItemInfo; 3] = [
    info(1026153, 6178), // teleporter
    info(1049117, 4036), // Horn of Retreat
    info(1048032, 3702), // a bag
];
static SPELLBOOK_INFO: [ItemInfo; 1] = [info(1023643, 8787)]; // spellbook
static GLOWING_RUNE_INFO: [ItemInfo; 1] = [info(1023676, 3679)]; // glowing rune

conv!(ACCEPT_CONVERSATION, "AcceptConversation", msg 1049092,
on_read: |w, q, _| {
    let bag = super::quester::get_new_container(w);
    if let Some(horn) = muo_content::factory::make_item(w, "Server.Engines.Quests.Necro.DarkTidesHorn") {
        muo_server::item::container::drop_item_default(w, bag, horn);
    }
    muo_server::mobile::part2::items::add_to_backpack(w, q.from, bag);
    q.add_conversation(w, Conversation::new(&REANIMATE_MAABUS_CONVERSATION));
});
conv!(REANIMATE_MAABUS_CONVERSATION, "ReanimateMaabusConversation", msg 1060099,
    info: |_, _| &REANIMATE_INFO,
    on_read: |w, q, _| q.add_objective(w, Objective::new(&FIND_MAABUS_TOMB_OBJECTIVE)));
conv!(MAABAS_CONVERSATION, "MaabasConversation", msg 1060103,
    info: |_, _| &TELEPORTER_INFO,
    on_read: |w, q, _| q.add_objective(w, Objective::new(&FIND_CRYSTAL_CAVE_OBJECTIVE)));
conv!(HORUS_CONVERSATION, "HorusConversation", msg 1060105,
    on_read: |w, q, _| q.add_objective(w, Objective::new(&FIND_MARDOTH_ABOUT_VAULT_OBJECTIVE)));
conv!(MARDOTH_VAULT_CONVERSATION, "MardothVaultConversation", msg 1060107,
    on_read: |w, q, _| q.add_objective(w, Objective::new(&FIND_CITY_OF_LIGHT_OBJECTIVE)));
conv!(VAULT_OF_SECRETS_CONVERSATION, "VaultOfSecretsConversation", msg 1060110,
    info: |_, _| &SPELLBOOK_INFO,
    on_read: |w, q, _| q.add_objective(w, Objective::new(&FETCH_ABRAXUS_SCROLL_OBJECTIVE)));
conv!(READ_ABRAXUS_SCROLL_CONVERSATION, "ReadAbraxusScrollConversation", msg 1060114,
    on_read: |w, q, _| q.add_objective(w, Objective::new(&READ_ABRAXUS_SCROLL_OBJECTIVE)));
conv!(SECOND_HORUS_CONVERSATION, "SecondHorusConversation", msg 1060118,
    on_read: |w, q, _| q.add_objective(w, Objective::new(&FIND_CALLING_SCROLL_OBJECTIVE)));
conv!(HEAL_CONVERSATION, "HealConversation", msg 1061610);
conv!(HORUS_REWARD_CONVERSATION, "HorusRewardConversation", msg 1060717, logged: never_logged);

/// `new LostCallingScrollConversation(fromMardoth)`.
pub fn lost_calling_scroll(from_mardoth: bool) -> Conversation {
    Conversation::with(&LOST_CALLING_SCROLL_CONVERSATION, from_mardoth)
}

conv!(LOST_CALLING_SCROLL_CONVERSATION, "LostCallingScrollConversation",
new_state: || St::new(false),
message: |_, c| if *c.state::<bool>() { 1062058 } else { 1060129 },
logged: never_logged,
child_deserialize: |_, c, r| {
    r.read_encoded_int()?; // version
    *c.st.get_mut::<bool>() = r.read_bool()?;
    Ok(())
},
child_serialize: |_, c, out| {
    out.write_encoded_int(0); // version
    out.write_bool(*c.state::<bool>());
});

conv!(MARDOTH_KRONUS_CONVERSATION, "MardothKronusConversation", msg 1060121,
    on_read: |w, q, _| q.add_objective(w, Objective::new(&FIND_WELL_OF_TEARS_OBJECTIVE)));
conv!(MARDOTH_END_CONVERSATION, "MardothEndConversation", msg 1060133,
    on_read: |w, q, _| q.add_objective(w, Objective::new(&FIND_BANK_OBJECTIVE)));
conv!(BANKER_CONVERSATION, "BankerConversation", msg 1060137, on_read: |w, q, _| q.complete(w));
conv!(RADAR_CONVERSATION, "RadarConversation", msg 1061692, logged: never_logged);

pub static CONVERSATIONS: &[&ConvClass] = &[
    &ACCEPT_CONVERSATION,
    &REANIMATE_MAABUS_CONVERSATION,
    &MAABAS_CONVERSATION,
    &HORUS_CONVERSATION,
    &MARDOTH_VAULT_CONVERSATION,
    &VAULT_OF_SECRETS_CONVERSATION,
    &READ_ABRAXUS_SCROLL_CONVERSATION,
    &SECOND_HORUS_CONVERSATION,
    &HEAL_CONVERSATION,
    &HORUS_REWARD_CONVERSATION,
    &LOST_CALLING_SCROLL_CONVERSATION,
    &MARDOTH_KRONUS_CONVERSATION,
    &MARDOTH_END_CONVERSATION,
    &BANKER_CONVERSATION,
    &RADAR_CONVERSATION,
];

// ---- Objectives ----

obj!(ANIMATE_MAABUS_CORPSE_OBJECTIVE, "AnimateMaabusCorpseObjective", msg 1060102,
    info: |_, _| &SPELLBOOK_INFO,
    on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&MAABAS_CONVERSATION)));

obj!(FIND_CRYSTAL_CAVE_OBJECTIVE, "FindCrystalCaveObjective", msg 1060104,
    on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&HORUS_CONVERSATION)));

obj!(FIND_MARDOTH_ABOUT_VAULT_OBJECTIVE, "FindMardothAboutVaultObjective", msg 1060106,
    on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&MARDOTH_VAULT_CONVERSATION)));

obj!(FIND_MAABUS_TOMB_OBJECTIVE, "FindMaabusTombObjective", msg 1060124,
    check_progress: |w, q, i| {
        if near_in_malas(w, q, Point3D::new(2024, 1240, -90), 3) {
            q.complete_objective(w, i);
        }
    },
    on_complete: |w, q, _| q.add_objective(w, Objective::new(&FIND_MAABUS_CORPSE_OBJECTIVE)));

obj!(FIND_MAABUS_CORPSE_OBJECTIVE, "FindMaabusCorpseObjective", msg 1061142,
    check_progress: |w, q, i| {
        if near_in_malas(w, q, Point3D::new(2024, 1223, -90), 3) {
            q.complete_objective(w, i);
        }
    },
    on_complete: |w, q, _| q.add_objective(w, Objective::new(&ANIMATE_MAABUS_CORPSE_OBJECTIVE)));

obj!(FIND_CITY_OF_LIGHT_OBJECTIVE, "FindCityOfLightObjective", msg 1060108,
    check_progress: |w, q, i| {
        if near_in_malas(w, q, Point3D::new(1076, 519, -90), 5) {
            q.complete_objective(w, i);
        }
    },
    on_complete: |w, q, _| q.add_objective(w, Objective::new(&FIND_VAULT_OF_SECRETS_OBJECTIVE)));

obj!(FIND_VAULT_OF_SECRETS_OBJECTIVE, "FindVaultOfSecretsObjective", msg 1060109,
    info: |_, _| &GLOWING_RUNE_INFO,
    check_progress: |w, q, i| {
        if near_in_malas(w, q, Point3D::new(1072, 455, -90), 1) {
            q.complete_objective(w, i);
        }
    },
    on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&VAULT_OF_SECRETS_CONVERSATION)));

obj!(FETCH_ABRAXUS_SCROLL_OBJECTIVE, "FetchAbraxusScrollObjective", msg 1060196,
    check_progress: |w, q, _| {
        let from = q.from;
        if muo_content::mx::map(w, from) != Some(MapId::MALAS)
            || !muo_content::mx::in_range_point(w, from, Point3D::new(1076, 450, -84), 5)
        {
            return;
        }
        // SummonFamiliarSpell.Table.TryGetValue(From, out var bc) && bc is HordeMinionFamiliar hmf
        // && hmf.InRange(From, 5) && hmf.TargetLocation == null, then TargetLocation = (1076, 450)
        let Some(f) = quest_hooks(w).and_then(|h| h.horde_minion_fetch) else { return };
        if f(w, from, Point2D::new(1076, 450)) {
            // You instinctively will your familiar to fetch the scroll for you.
            muo_content::mx::msg(w, from, 1060113);
        }
    },
    on_complete: |w, q, _| q.add_objective(w, Objective::new(&RETRIEVE_ABRAXUS_SCROLL_OBJECTIVE)));

obj!(RETRIEVE_ABRAXUS_SCROLL_OBJECTIVE, "RetrieveAbraxusScrollObjective", msg 1060199,
    on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&READ_ABRAXUS_SCROLL_CONVERSATION)));

obj!(READ_ABRAXUS_SCROLL_OBJECTIVE, "ReadAbraxusScrollObjective", msg 1060125,
    on_complete: |w, q, _| q.add_objective(w, Objective::new(&RETURN_TO_CRYSTAL_CAVE_OBJECTIVE)));

obj!(RETURN_TO_CRYSTAL_CAVE_OBJECTIVE, "ReturnToCrystalCaveObjective", msg 1060115,
    info: |_, _| &TELEPORTER_INFO,
    on_complete: |w, q, _| q.add_objective(w, Objective::new(&SPEAK_CAVE_PASSWORD_OBJECTIVE)));

obj!(SPEAK_CAVE_PASSWORD_OBJECTIVE, "SpeakCavePasswordObjective", msg 1060117,
    on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&SECOND_HORUS_CONVERSATION)));

/// `FindCallingScrollObjective`'s fields, all saved.
#[derive(Clone, Default)]
struct CallingScroll {
    heal_conversation_shown: bool,
    hoppers_disposed: bool,
    hoppers_killed: i32,
}

obj!(FIND_CALLING_SCROLL_OBJECTIVE, "FindCallingScrollObjective", msg 1060119,
new_state: || St::new(CallingScroll::default()),
ignore_young_protection: |w, q, i, from| {
    !q.objectives[i].state::<CallingScroll>().hoppers_disposed
        && mobile_is(w, from, "Server.Mobiles.SkitteringHopper")
},
get_kill_event: |_, q, i, _, _| !q.objectives[i].state::<CallingScroll>().hoppers_disposed,
on_kill: |w, q, i, creature, _| {
    if mobile_is(w, creature, "Server.Mobiles.SkitteringHopper") {
        if !q.objectives[i].state::<CallingScroll>().heal_conversation_shown {
            q.objectives[i].state_mut::<CallingScroll>().heal_conversation_shown = true;
            q.add_conversation(w, Conversation::new(&HEAL_CONVERSATION));
        }
        let s = q.objectives[i].state_mut::<CallingScroll>();
        s.hoppers_killed += 1;
        if s.hoppers_killed >= 5 {
            s.hoppers_disposed = true;
            q.add_objective(w, Objective::new(&FIND_HORUS_ABOUT_REWARD_OBJECTIVE));
        }
    }
},
on_complete: |w, q, _| q.add_objective(w, Objective::new(&FIND_MARDOTH_ABOUT_KRONUS_OBJECTIVE)),
child_deserialize: |_, o, r| {
    r.read_encoded_int()?; // version
    let s = o.state_mut::<CallingScroll>();
    s.hoppers_killed = r.read_encoded_int()?;
    s.heal_conversation_shown = r.read_bool()?;
    s.hoppers_disposed = r.read_bool()?;
    Ok(())
},
child_serialize: |_, o, out| {
    let s = o.state::<CallingScroll>();
    out.write_encoded_int(0); // version
    out.write_encoded_int(s.hoppers_killed);
    out.write_bool(s.heal_conversation_shown);
    out.write_bool(s.hoppers_disposed);
});

obj!(FIND_HORUS_ABOUT_REWARD_OBJECTIVE, "FindHorusAboutRewardObjective", msg 1060126,
    on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&HORUS_REWARD_CONVERSATION)));

obj!(FIND_MARDOTH_ABOUT_KRONUS_OBJECTIVE, "FindMardothAboutKronusObjective", msg 1060127,
    on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&MARDOTH_KRONUS_CONVERSATION)));

const WELL_OF_TEARS: Rectangle2D = Rectangle2D::new(2080, 1346, 10, 10);

/// `FindWellOfTearsObjective.m_Inside` (not saved).
#[derive(Clone, Default)]
struct Inside(bool);

obj!(FIND_WELL_OF_TEARS_OBJECTIVE, "FindWellOfTearsObjective", msg 1060128,
    new_state: || St::new(Inside::default()),
    check_progress: |w, q, i| {
        let from = q.from;
        let loc = muo_content::mx::location(w, from);
        if muo_content::mx::map(w, from) == Some(MapId::MALAS) && WELL_OF_TEARS.contains_xy(loc.x, loc.y) {
            if has_lost_calling_scroll(w, from) {
                if !q.objectives[i].state::<Inside>().0 {
                    q.add_conversation(w, lost_calling_scroll(false));
                }
            } else {
                q.complete_objective(w, i);
            }
            q.objectives[i].state_mut::<Inside>().0 = true;
        } else {
            q.objectives[i].state_mut::<Inside>().0 = false;
        }
    },
    on_complete: |w, q, _| q.add_objective(w, Objective::new(&USE_CALLING_SCROLL_OBJECTIVE)));

obj!(USE_CALLING_SCROLL_OBJECTIVE, "UseCallingScrollObjective", msg 1060130);

/// `new FindMardothEndObjective(victory)`.
pub fn find_mardoth_end(victory: bool) -> Objective {
    Objective::with(&FIND_MARDOTH_END_OBJECTIVE, victory)
}

obj!(FIND_MARDOTH_END_OBJECTIVE, "FindMardothEndObjective",
new_state: || St::new(false),
message: |_, o| if *o.state::<bool>() { 1060131 } else { 1060132 },
on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&MARDOTH_END_CONVERSATION)),
child_deserialize: |_, o, r| {
    r.read_encoded_int()?; // version
    *o.state_mut::<bool>() = r.read_bool()?;
    Ok(())
},
child_serialize: |_, o, out| {
    out.write_encoded_int(0); // version
    out.write_bool(*o.state::<bool>());
});

obj!(FIND_BANK_OBJECTIVE, "FindBankObjective", msg 1060134,
    check_progress: |w, q, i| {
        if near_in_malas(w, q, Point3D::new(2048, 1345, -84), 5) {
            q.complete_objective(w, i);
        }
    },
    on_complete: |w, q, _| q.add_objective(w, Objective::new(&CASH_BANK_CHECK_OBJECTIVE)));

obj!(CASH_BANK_CHECK_OBJECTIVE, "CashBankCheckObjective", msg 1060644,
    on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&BANKER_CONVERSATION)));

pub static OBJECTIVES: &[&ObjClass] = &[
    &ANIMATE_MAABUS_CORPSE_OBJECTIVE,
    &FIND_CRYSTAL_CAVE_OBJECTIVE,
    &FIND_MARDOTH_ABOUT_VAULT_OBJECTIVE,
    &FIND_MAABUS_TOMB_OBJECTIVE,
    &FIND_MAABUS_CORPSE_OBJECTIVE,
    &FIND_CITY_OF_LIGHT_OBJECTIVE,
    &FIND_VAULT_OF_SECRETS_OBJECTIVE,
    &FETCH_ABRAXUS_SCROLL_OBJECTIVE,
    &RETRIEVE_ABRAXUS_SCROLL_OBJECTIVE,
    &READ_ABRAXUS_SCROLL_OBJECTIVE,
    &RETURN_TO_CRYSTAL_CAVE_OBJECTIVE,
    &SPEAK_CAVE_PASSWORD_OBJECTIVE,
    &FIND_CALLING_SCROLL_OBJECTIVE,
    &FIND_HORUS_ABOUT_REWARD_OBJECTIVE,
    &FIND_MARDOTH_ABOUT_KRONUS_OBJECTIVE,
    &FIND_WELL_OF_TEARS_OBJECTIVE,
    &USE_CALLING_SCROLL_OBJECTIVE,
    &FIND_MARDOTH_END_OBJECTIVE,
    &FIND_BANK_OBJECTIVE,
    &CASH_BANK_CHECK_OBJECTIVE,
];

// ---- The quest ----

/// `DarkTidesQuest`.
pub static QUEST: QClass = QClass {
    name: "Server.Engines.Quests.Necro.DarkTidesQuest",
    slot: 1,
    name_msg: Msg::Num(1060095),
    offer_message: |_, _| Msg::Num(1060094),
    picture: |_, _| 0x15B5,
    is_tutorial: true,
    restart_delay: TimeSpan::MAX,
    professions: &[4], // necromancer
    accept: |w, q: &mut Quest| {
        base_accept(w, q);
        q.add_conversation(w, Conversation::new(&ACCEPT_CONVERSATION));
    },
    ignore_young_protection: |w, q, from| {
        mobile_is(w, from, "Server.Engines.Quests.Necro.SummonedPaladin")
            || base_ignore_young_protection(w, q, from)
    },
    ..QUEST_SYSTEM
};
