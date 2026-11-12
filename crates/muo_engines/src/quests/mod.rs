//! The old (pre-ML) quest engine and its eleven quests.
//!
//! [`core`] is `Core/` (QuestSystem, QuestObjective, QuestConversation, QuestSerializer and the gumps), one
//! module per quest folder holds that quest's objectives, conversations, system class, mobiles and items.
//! A quest in progress is stored on the player in the bytes `QuestSerializer` writes (see [`core`]).

pub mod ambitious;
pub mod collector;
pub mod collector_mobs;
pub mod core;
pub mod dark_tides;
pub mod dark_tides_mobs;
pub mod emino;
pub mod emino_mobs;
pub mod external;
pub mod gumps;
pub mod haochi;
pub mod haochi_mobs;
pub mod hatchlings;
pub mod hooks;
pub mod items;
pub mod matriarch;
pub mod matriarch_mobs;
pub mod naturalist;
pub mod quester;
pub mod regions;
pub mod summoning;
pub mod summoning_mobs;
#[cfg(test)]
mod tests;
pub mod util;
pub mod uzeraan;
pub mod uzeraan_mobs;
pub mod witch;
pub mod witch_mobs;

use muo_content::player_mobile::hooks::SystemHooks;
use muo_server::World;
use muo_server::type_registry::{DEFAULT_CALL_PRIORITY, Phase, TypeRegistry};

use self::core::{
    ConvClass, ObjClass, QClass, callback_entry, load, start_timer, stop_timer, with_quest,
};

/// `QuestSystem.QuestTypes`, in order (a class's `slot` is its index).
pub static QUEST_CLASSES: &[&QClass] = &[
    &summoning::QUEST,
    &dark_tides::QUEST,
    &uzeraan::QUEST,
    &collector::QUEST,
    &witch::QUEST,
    &naturalist::QUEST,
    &matriarch::QUEST,
    &ambitious::QUEST,
    &emino::QUEST,
    &haochi::QUEST,
    &hatchlings::QUEST,
];

/// `AssemblyHandler.FindTypeByFullName` over the quest classes.
pub fn quest_class(name: &str) -> Option<&'static QClass> {
    QUEST_CLASSES.iter().find(|c| c.name == name).copied()
}

/// Every objective class of every quest.
pub static OBJECTIVE_LISTS: &[&[&ObjClass]] = &[
    summoning::OBJECTIVES,
    dark_tides::OBJECTIVES,
    uzeraan::OBJECTIVES,
    collector::OBJECTIVES,
    witch::OBJECTIVES,
    naturalist::OBJECTIVES,
    matriarch::OBJECTIVES,
    ambitious::OBJECTIVES,
    emino::OBJECTIVES,
    haochi::OBJECTIVES,
    hatchlings::OBJECTIVES,
];

/// Every conversation class of every quest.
pub static CONVERSATION_LISTS: &[&[&ConvClass]] = &[
    summoning::CONVERSATIONS,
    dark_tides::CONVERSATIONS,
    uzeraan::CONVERSATIONS,
    collector::CONVERSATIONS,
    witch::CONVERSATIONS,
    naturalist::CONVERSATIONS,
    matriarch::CONVERSATIONS,
    ambitious::CONVERSATIONS,
    emino::CONVERSATIONS,
    haochi::CONVERSATIONS,
    hatchlings::CONVERSATIONS,
];

pub fn objective_class(name: &str) -> Option<&'static ObjClass> {
    OBJECTIVE_LISTS
        .iter()
        .copied()
        .flatten()
        .find(|c| c.name == name)
        .copied()
}

pub fn conversation_class(name: &str) -> Option<&'static ConvClass> {
    CONVERSATION_LISTS
        .iter()
        .copied()
        .flatten()
        .find(|c| c.name == name)
        .copied()
}

/// Sets the hooks the rest of the content calls into the quests through.
fn configure(w: &mut World) {
    external::install(w);
    // `BaseRunicTool.ApplyAttributesTo(weapon or armor, attributeCount, min, max)` (quest rewards).
    w.resources
        .get_mut::<hooks::QuestHooks>()
        .apply_runic_attributes = Some(|w, item, count, min, max| {
        muo_content::items::runic::apply_to_item(w, item, false, 0, count, min, max)
    });
    {
        let q = w.resources.get_mut::<hooks::QuestHooks>();
        // `PMList.Trammel/Felucca/Malas.Entries`
        q.moongates = Some(|_, map| {
            use muo_items::misc::moongates as mg;
            [&mg::TRAMMEL, &mg::FELUCCA, &mg::MALAS]
                .into_iter()
                .find(|l| l.map == map)
                .map_or_else(Vec::new, |l| l.entries.iter().map(|e| e.location).collect())
        });
        // `SummonFamiliarSpell.Table.TryGetValue(from, out bc) && bc is HordeMinionFamiliar hmf && hmf.InRange(from, 5)
        // && hmf.TargetLocation == null`, then `hmf.TargetLocation = target`
        q.horde_minion_fetch = Some(|w, from, target| {
            let Some(hmf) = muo_spells::necromancy::summon_familiar::familiar_of(w, from) else {
                return false;
            };
            if !muo_content::factory::mobile_is(w, hmf, "Server.Mobiles.HordeMinionFamiliar")
                || !muo_content::mx::in_range(w, hmf, from, 5)
                || muo_content::creature::data::cd(w, hmf)
                    .is_none_or(|d| d.target_location.is_some())
            {
                return false;
            }
            muo_content::creature::data::edit(w, hmf, |d| d.target_location = Some(target));
            true
        });
    }
    w.resources
        .get_mut::<muo_items::food::beverage::BeverageHooks>()
        .vat_pour = Some(matriarch::pour_on_vat);
    w.resources
        .get_mut::<muo_content::targets::TargetHooks>()
        .witch_apprentice_red_mushroom = Some(witch::slice_red_mushroom);
    let h = w.resources.get_mut::<SystemHooks>();
    h.quest_start_timer = Some(start_timer);
    h.quest_stop_timer = Some(stop_timer);
    h.quest_ignore_young_protection =
        Some(|w, m, from| load(w, m).is_some_and(|q| q.ignore_young_protection(w, from)));
    h.quest_haven_schmendrick_in_progress = Some(|w, m| {
        load(w, m).is_some_and(|q| {
            q.is(&uzeraan::QUEST)
                && q.is_objective_in_progress(&uzeraan::FIND_SCHMENDRICK_OBJECTIVE)
        })
    });
    h.quest_context_menu_entries = Some(|w, m, list| {
        // QuestSystem.GetContextMenuEntries
        let Some(q) = load(w, m) else { return };
        if !q.objectives.is_empty() {
            list.push(callback_entry(6154, move |w, from| {
                // View Quest Log
                with_quest(w, from, |w, q| q.show_quest_log(w));
            }));
        }
        if !q.conversations.is_empty() {
            list.push(callback_entry(6156, move |w, from| {
                // Quest Conversation
                with_quest(w, from, |w, q| q.show_quest_conversation(w));
            }));
        }
        list.push(callback_entry(6155, move |w, from| {
            // Cancel Quest
            with_quest(w, from, |w, q| q.begin_cancel_quest(w));
        }));
    });

    let d = w.resources.get_mut::<muo_content::hooks::DeathHooks>();
    d.quest_on_kill = Some(|w, pm, creature, corpse| {
        with_quest(w, pm, |w, q| q.on_kill(w, creature, corpse)).is_some()
    });
}

/// Every mobile class of the quests.
fn mobile_lists() -> [&'static [&'static muo_server::mobile::MobileType]; 12] {
    [
        ambitious::MOBILES,
        collector_mobs::MOBILES,
        dark_tides_mobs::MOBILES,
        emino_mobs::MOBILES,
        haochi_mobs::MOBILES,
        hatchlings::MOBILES,
        matriarch_mobs::MOBILES,
        naturalist::MOBILES,
        summoning_mobs::MOBILES,
        uzeraan_mobs::MOBILES,
        witch_mobs::MOBILES,
        QUESTER_BASES,
    ]
}

/// Every item class of the quests.
fn item_lists() -> [&'static [&'static muo_server::item::ItemType]; 8] {
    [
        items::ITEMS,
        collector_mobs::ITEMS,
        dark_tides_mobs::ITEMS,
        emino_mobs::ITEMS,
        haochi_mobs::ITEMS,
        summoning_mobs::ITEMS,
        uzeraan_mobs::ITEMS,
        witch_mobs::ITEMS,
    ]
}

/// `BaseQuester`.
static QUESTER_BASES: &[&muo_server::mobile::MobileType] = &[&quester::BASE_QUESTER];

pub fn register(types: &mut TypeRegistry) {
    for list in item_lists() {
        for ty in list {
            types.register_item(ty, &[]);
        }
    }
    for list in mobile_lists() {
        for ty in list {
            types.register_mobile(ty, &[]);
        }
    }
    types.register_phase(
        Phase::Configure,
        "Server.Engines.Quests.Configure",
        DEFAULT_CALL_PRIORITY,
        configure,
    );
}
