//! `MLQuestSystem.Configure` / `Initialize` and the static constructor (Data/MLQuests.cfg).

use muo_content::ml_quest_hooks::MlQuestCreatureHooks;
use muo_content::player_mobile::hooks::SystemHooks;
use muo_primitives::dotnet::IgnoreCase;
use muo_server::commands::{CommandEventArgs, CommandSystem};
use muo_server::ids::MobileId;
use muo_server::mobile::AccessLevel;
use muo_server::type_registry::{Phase, TypeRegistry};
use muo_server::world::World;
use std::rc::Rc;

use super::engine::{self, EscortHooks};
use super::gumps;
use super::model::*;
use super::persistence;
use super::quester::{Quester, QuesterHooks, quests_of};

/// Registers the item type and the phases.
pub fn register(types: &mut TypeRegistry) {
    types.register_item(&persistence::ML_QUEST_PERSISTENCE, &[]);
    types.register_phase_default(
        Phase::Configure,
        "Server.Engines.MLQuests.MLQuestSystem.Configure",
        configure,
    );
    types.register_phase_default(
        Phase::Initialize,
        "Server.Engines.MLQuests.MLQuestSystem.Initialize",
        initialize,
    );
    // `TargetCommands.Register` needs the implementors `TargetCommands.Configure` makes: after the default priority.
    types.register_phase(
        Phase::Configure,
        "Server.Engines.MLQuests.MLQuestSystem.ConfigureTargetCommands",
        muo_server::type_registry::DEFAULT_CALL_PRIORITY + 1,
        super::commands::register_target_commands,
    );
    super::items::register(types);
    super::npcs::register(types);
}

/// The static constructor: `Data/MLQuests.cfg`.
fn load_config(w: &mut World) {
    let path = w.core.base_directory.join("Data").join("MLQuests.cfg");
    let Ok(text) = std::fs::read_to_string(path) else {
        return;
    };

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let split: Vec<&str> = line.split('\t').collect();
        let Some(class) = super::defs::ALL.iter().find(|c| {
            c.name().eq_ic(split[0]) // `FindTypeByName(name)`: the short name only
        }) else {
            continue;
        };

        // Register(type, quest), then each quester. A quester name without a class is registered too:
        // nothing asks for it, and it is the same name once the class exists.
        engine::register_quest(w, class, &split[1..]);
    }
}

/// `MLQuestSystem.Configure`.
fn configure(w: &mut World) {
    let default = w.core_ml();
    let enabled = w
        .config
        .get_or_update_setting("questSystem.enableMLQuests", default)
        .unwrap_or(default);
    state(w).enabled = enabled;

    load_config(w);

    // `book.Content = (1ul << book.BookCount) - 1` (shift counts are masked to 6 bits)
    w.resources
        .get_mut::<super::defs::special::QuestItemHooks>()
        .spellbook_fill = Some(|w, book| {
        use muo_items::skill_items::spellbook as sb;
        if sb::is_spellbook(w, book) {
            let count = (sb::class(w, book).book_count)(w);
            sb::set_content(w, book, (1u64 << (count & 63)) - 1);
        }
    });
    // The skillgain scrolls' check: a live `GainSkillObjectiveInstance` that handles the skill.
    muo_items::special::hooks::hooks_mut(w).gain_skill_quest = Some(|w, pm, skill| {
        let Some(ctx) = engine::get_context(w, pm) else {
            return false;
        };
        ctx.quest_instances.iter().any(|&inst| {
            engine::instance(w, inst).is_some_and(|ins| {
                ins.objectives.iter().enumerate().any(|(idx, o)| {
                    !o.expired
                        && matches!(
                            engine::quest(w, ins.quest).objectives.get(idx).map(|q| &q.kind),
                            Some(ObjectiveKind::GainSkill { skill: s, .. }) if *s == skill
                        )
                })
            })
        })
    });
    let sys = w.resources.get_mut::<SystemHooks>();
    sys.ml_quest_enabled = Some(engine::enabled);
    // `MLQuestSystem.GetContext(pm)?.Spellweaving / SummonFey / SummonFiend == true`
    muo_content::spells::hooks::hooks_mut(w).ml_quest_flag = Some(|w, m, name| {
        let f = match name {
            "Spellweaving" => flag::SPELLWEAVING,
            "SummonFey" => flag::SUMMON_FEY,
            _ => flag::SUMMON_FIEND,
        };
        engine::get_context(w, m).is_some_and(|c| c.get_flag(f))
    });
    let sys = w.resources.get_mut::<SystemHooks>();
    sys.ml_quest_skill_gain = Some(engine::handle_skill_gain);
    sys.ml_quest_mark_item = Some(engine::mark_quest_item);
    sys.ml_quest_close_gumps = Some(gumps::close_other_gumps);

    let c = w.resources.get_mut::<MlQuestCreatureHooks>();
    c.can_give_quest = Some(can_give_quest);
    c.double_click = Some(creature_double_click);
    c.drag_drop = Some(creature_drag_drop);
    c.after_delete = Some(creature_after_delete);
    c.movement = Some(creature_movement);
    c.handle_kill = Some(creature_handle_kill);

    w.resources
        .get_mut::<muo_content::network::incoming_game::GameHooks>()
        .quest_gump_request = Some(engine::quest_gump_request);

    muo_content::player_mobile::subscribe_death(w, engine::handle_death);
    muo_content::player_mobile::subscribe_deleted(w, engine::handle_player_deletion);

    // BaseEscortable (Mobiles/Townfolk, the npcs task) owns is_being_deleted / is_prisoner / begin_delete and the
    // dynamic quest list: they stay unset until that class exists.
    w.resources.get_mut::<EscortHooks>().award_compassion = Some(|w, pm, amount| {
        crate::virtues::award_virtue(w, pm, crate::virtues::VirtueName::Compassion, amount);
    });
    w.resources.get_mut::<QuesterHooks>();
    // `item is Pitcher { Content: BeverageType.Water, Quantity: > 0 }`
    w.resources
        .get_mut::<super::defs::special::QuestItemHooks>()
        .pitcher_has_water = Some(|w, item| {
        engine::item_is(w, item, "Pitcher")
            && muo_items::food::beverage::content(w, item)
                == muo_items::food::beverage::BeverageType::Water
            && muo_items::food::beverage::quantity(w, item) > 0
    });

    register_commands(w);
}

/// `MLQuestSystem.Initialize`.
fn initialize(w: &mut World) {
    if !engine::enabled(w) {
        return;
    }

    // AutoGenerateNew
    for i in 0..state(w).quests.len() {
        let q = engine::quest(w, QuestId(i));
        if !q.deserialized
            && let Some(f) = q.class.generate
        {
            f(w);
        }
    }

    persistence::ensure_existence(w);
}

// ---- BaseCreature hooks ----------------------------------------------------------------------------------

fn can_give_quest(w: &World, m: MobileId) -> bool {
    engine::enabled(w) && !quests_of(w, Quester::Mobile(m)).is_empty()
}

fn is_player(w: &World, m: MobileId) -> bool {
    muo_content::player_mobile::data(w, m).is_some()
}

fn creature_double_click(w: &mut World, id: MobileId, from: MobileId) {
    if can_give_quest(w, id) && is_player(w, from) {
        engine::on_double_click(w, Quester::Mobile(id), from);
    }
}

fn creature_drag_drop(w: &mut World, id: MobileId, from: MobileId) -> bool {
    if can_give_quest(w, id) && is_player(w, from) {
        // You need to mark your quest items so I don't take the wrong object.  Then speak to me.
        super::quester::tell(w, Quester::Mobile(id), from, 1074893);
        return true;
    }
    false
}

fn creature_after_delete(w: &mut World, id: MobileId) {
    if engine::enabled(w) {
        engine::handle_quester_deletion(w, Quester::Mobile(id));
    }
    state(w).next_shout.remove(&id);
}

fn creature_movement(w: &mut World, id: MobileId, m: MobileId, old: muo_primitives::Point3D) {
    if engine::enabled(w) && is_player(w, m) {
        engine::check_shout(w, id, m, old, 8);
    }
}

fn creature_handle_kill(w: &mut World, pm: MobileId, creature: MobileId) {
    if engine::enabled(w) {
        engine::handle_kill(w, pm, creature);
    }
}

// ---- commands --------------------------------------------------------------------------------------------

fn find_quest_by_arg(w: &World, name: &str) -> Option<QuestId> {
    let s = state_ref(w)?;
    s.by_class
        .iter()
        .find(|(k, _)| k.eq_ic(name) || short_name(k).eq_ic(name))
        .map(|(_, v)| *v)
}

fn register_commands(w: &mut World) {
    CommandSystem::register(
        w,
        "MLQuestsInfo",
        AccessLevel::Administrator,
        &[],
        Rc::new(mlquests_info),
    );
    CommandSystem::register(
        w,
        "SaveQuest",
        AccessLevel::Administrator,
        &[],
        Rc::new(save_quest),
    );
    CommandSystem::register(
        w,
        "SaveAllQuests",
        AccessLevel::Administrator,
        &[],
        Rc::new(save_all_quests),
    );
    CommandSystem::register(
        w,
        "InvalidQuestItems",
        AccessLevel::Administrator,
        &[],
        Rc::new(super::commands::invalid_quest_items),
    );
}

fn send(w: &mut World, m: MobileId, text: &str) {
    muo_server::mobile::part2::messages::send_message(w, m, text);
}

/// `MLQuestsInfo_OnCommand`.
fn mlquests_info(w: &mut World, e: &mut CommandEventArgs) {
    let m = e.mobile;

    if e.length() == 0 {
        let n = state(w).by_class.len();
        send(w, m, &format!("Quest table length: {n}"));
        return;
    }

    let Some(qid) = find_quest_by_arg(w, &e.get_string(0)) else {
        send(w, m, "Invalid quest type name.");
        return;
    };

    let q = engine::quest(w, qid);
    let lines = [
        format!("Activated: {}", if q.activated { "True" } else { "False" }),
        format!("Number of objectives: {}", q.objectives.len()),
        format!(
            "Objective type: {}",
            if q.objective_type == ObjectiveType::All {
                "All"
            } else {
                "Any"
            }
        ),
        format!("Number of active instances: {}", q.instances.len()),
    ];
    for l in lines {
        send(w, m, &l);
    }
}

/// `SaveQuest_OnCommand`.
fn save_quest(w: &mut World, e: &mut CommandEventArgs) {
    let m = e.mobile;

    if e.length() == 0 || e.length() > 2 {
        send(w, m, "Syntax: SaveQuest <id> [saveEnabled=true]");
        return;
    }

    let Some(qid) = find_quest_by_arg(w, &e.get_string(0)) else {
        send(w, m, "Invalid quest type name.");
        return;
    };

    let enable = if e.length() == 2 {
        e.get_boolean(1)
    } else {
        true
    };
    state(w).quests[qid.0].save_enabled = enable;
    let name = state(w).quests[qid.0].class.name();

    if enable {
        send(
            w,
            m,
            &format!("Serialization for quest {name} is now enabled."),
        );
    } else {
        send(
            w,
            m,
            &format!("Serialization for quest {name} is now disabled."),
        );
    }

    if !enable {
        send(
            w,
            m,
            "Please note that automatic generation of new quests is ON. This quest will be regenerated on the next server start.",
        );
    }
}

/// `SaveAllQuests_OnCommand`.
fn save_all_quests(w: &mut World, e: &mut CommandEventArgs) {
    let m = e.mobile;

    if e.length() > 1 {
        send(w, m, "Syntax: SaveAllQuests [saveEnabled=true]");
        return;
    }

    let enable = if e.length() == 1 {
        e.get_boolean(0)
    } else {
        true
    };

    for q in &mut state(w).quests {
        q.save_enabled = enable;
    }

    if enable {
        send(w, m, "Serialization for all quests is now enabled.");
    } else {
        send(w, m, "Serialization for all quests is now disabled.");
    }

    if !enable {
        send(
            w,
            m,
            "Please note that automatic generation of new quests is ON. All quests will be regenerated on the next server start.",
        );
    }
}
