//! The ML quest commands that need other systems: `InvalidQuestItems`, and the target commands
//! `ViewQuests` and `ViewMLContext` (`ViewQuestsCommand`, `ViewContextCommand`).

use std::rc::Rc;

use muo_commands::generic::implementors::register_command;
use muo_commands::generic::{
    BaseCommand, CmdSpec, CommandSupport, Obj, ObjectTypes, Run, as_mobile,
};
use muo_commands::hooks::CommandHooks;
use muo_commands::logging::log_text;
use muo_content::props::{Kind, PropDef, Value};
use muo_content::utilities::types::PropType;
use muo_gumps::m_z::props::registry::SubObj;
use muo_server::commands::CommandEventArgs;
use muo_server::ids::{EntityId, MobileId};
use muo_server::item::ImplFlag;
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part2::messages::send_message;
use muo_server::world::World;

use super::{engine, gumps};

fn is_player(w: &World, m: MobileId) -> bool {
    muo_content::player_mobile::data(w, m).is_some()
}

/// `MLQuestContext`'s `[CommandProperty]` members, read off the owner (a `PlayerMobile`) through its context.
macro_rules! context_flag {
    ($name:literal, $flag:expr) => {
        PropDef {
            name: $name,
            kind: Kind::Plain(PropType::Bool),
            read: AccessLevel::GameMaster,
            write: AccessLevel::GameMaster,
            get: |w, e| match e {
                EntityId::Mobile(m) => {
                    Value::Bool(engine::get_context(w, m).is_some_and(|c| c.get_flag($flag)))
                }
                _ => Value::Null,
            },
            set: Some(|w, e, v| {
                if let EntityId::Mobile(m) = e {
                    engine::get_or_create_context(w, m).set_flag($flag, v.bool());
                }
                Ok(())
            }),
        }
    };
}

static ML_CONTEXT_PROPS: &[PropDef] = &[
    PropDef {
        name: "IsFull",
        kind: Kind::Plain(PropType::Bool),
        read: AccessLevel::GameMaster,
        write: AccessLevel::GameMaster,
        get: |w, e| match e {
            EntityId::Mobile(m) => {
                Value::Bool(engine::get_context(w, m).is_some_and(|c| c.is_full()))
            }
            _ => Value::Null,
        },
        set: None,
    },
    context_flag!("Spellweaving", 0x01),
    context_flag!("SummonFey", 0x02),
    context_flag!("SummonFiend", 0x04),
    context_flag!("BedlamAccess", 0x08),
];

/// `InvalidQuestItems_OnCommand`: the quest items not in the top level of a player's backpack.
pub fn invalid_quest_items(w: &mut World, e: &mut CommandEventArgs) {
    let m = e.mobile;

    let mut found = Vec::new();
    for (&id, item) in &w.items {
        if !item.flags.contains(ImplFlag::QUEST_ITEM) {
            continue;
        }
        // `item.Parent is Backpack pack && pack.Parent is PlayerMobile player && player.Backpack == pack`
        if let Some(EntityId::Item(pack)) = item.parent
            && super::engine::item_is(w, pack, "Backpack")
            && let Some(EntityId::Mobile(player)) = w.item(pack).and_then(|p| p.parent)
            && is_player(w, player)
            && w.mobile(player).and_then(|p| p.backpack) == Some(pack)
        {
            continue;
        }
        found.push(EntityId::Item(id));
    }

    if found.is_empty() {
        send_message(w, m, "No matching objects found.");
    } else {
        // `new InterfaceGump(m, new[] { "Object" }, found, 0, null)`: the columns after "Object" are none.
        match w.resources.get_mut::<CommandHooks>().interface_gump {
            Some(g) => g(w, m, found, Vec::new()),
            None => send_message(
                w,
                m,
                &format!(
                    "{} objects matched; the interface gump is not available.",
                    found.len()
                ),
            ),
        }
    }
}

// ---- ViewQuestsCommand ----------------------------------------------------------------------------------------

struct ViewQuestsCommand {
    spec: CmdSpec,
}

impl BaseCommand for ViewQuestsCommand {
    fn spec(&self) -> &CmdSpec {
        &self.spec
    }

    fn execute(&self, w: &mut World, run: &mut Run, e: &CommandEventArgs, obj: Obj) {
        let from = e.mobile;
        let Some(pm) = as_mobile(obj).filter(|&m| is_player(w, m)) else {
            run.log_failure("That is not a player.");
            return;
        };
        log_text(
            w,
            from,
            &format!(
                "viewing quest overview of {}",
                muo_commands::logging::format_entity(w, EntityId::Mobile(pm))
            ),
        );
        gumps::display_log(w, from, pm, false);
    }
}

// ---- ViewContextCommand ---------------------------------------------------------------------------------------

struct ViewContextCommand {
    spec: CmdSpec,
}

impl BaseCommand for ViewContextCommand {
    fn spec(&self) -> &CmdSpec {
        &self.spec
    }

    fn execute(&self, w: &mut World, run: &mut Run, e: &CommandEventArgs, obj: Obj) {
        let Some(pm) = as_mobile(obj).filter(|&m| is_player(w, m)) else {
            run.log_failure("They have no ML quest context.");
            return;
        };
        // `GetOrCreateContext(pm)`
        engine::get_or_create_context(w, pm);
        let g = muo_gumps::m_z::props::properties::new(
            w,
            e.mobile,
            SubObj::object(EntityId::Mobile(pm), ML_CONTEXT_PROPS, "MLQuestContext"),
        );
        muo_gumps::m_z::common::send(w, e.mobile, g);
    }
}

/// `TargetCommands.Register(new ViewQuestsCommand()); TargetCommands.Register(new ViewContextCommand())`.
pub fn register_target_commands(w: &mut World) {
    register_command(
        w,
        Rc::new(ViewQuestsCommand {
            spec: CmdSpec::new(
                &["ViewQuests"],
                AccessLevel::GameMaster,
                CommandSupport::SIMPLE,
                ObjectTypes::Mobiles,
                "ViewQuests",
                "Displays a targeted mobile's quest overview.",
            ),
        }),
    );
    register_command(
        w,
        Rc::new(ViewContextCommand {
            spec: CmdSpec::new(
                &["ViewMLContext"],
                AccessLevel::GameMaster,
                CommandSupport::SIMPLE,
                ObjectTypes::Mobiles,
                "ViewMLContext",
                "Opens the ML quest context for a targeted mobile.",
            ),
        }),
    );
}
