//! `ToggleItem`: an unmovable item that switches between two graphics, and the `[Toggle` target command.

use std::rc::Rc;

use muo_commands::generic::implementors::register_command;
use muo_commands::generic::{BaseCommand, CmdSpec, CommandSupport, Obj, ObjectTypes, Run, as_item};
use muo_content::props::{self, Kind, PropDef, Value};
use muo_content::utilities::types::{Numeric, PropType};

use muo_server::World;
use muo_server::commands::CommandEventArgs;
use muo_server::ids::{EntityId, ItemId};
use muo_server::item::{ITEM_BASE, ItemType, deps, ops};
use muo_server::mobile::AccessLevel;
use muo_server::type_registry::{DEFAULT_CALL_PRIORITY, Phase, TypeRegistry};

/// `_inactiveItemId`, `_activeItemId`, `_playersCanToggle`.
#[derive(Clone, Copy, Debug, Default)]
struct ToggleData {
    inactive: i32,
    active: i32,
    players_can_toggle: bool,
}

fn data(w: &World, id: ItemId) -> ToggleData {
    w.item(id)
        .and_then(|i| i.ext.get::<ToggleData>())
        .copied()
        .unwrap_or_default()
}

fn set_data(w: &mut World, id: ItemId, d: ToggleData) {
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(d);
    }
}

/// `ToggleItem(inactiveItemID, activeItemID, playersCanToggle)`.
pub fn toggle_item_new(
    w: &mut World,
    inactive_item_id: i32,
    active_item_id: i32,
    players_can_toggle: Option<bool>,
) -> ItemId {
    let id = ops::new_item_with_id(w, &TOGGLE_ITEM, inactive_item_id);
    ops::set_movable(w, id, false);
    set_data(
        w,
        id,
        ToggleData {
            inactive: inactive_item_id,
            active: active_item_id,
            players_can_toggle: players_can_toggle.unwrap_or(false),
        },
    );
    id
}

/// `Toggle()`.
pub fn toggle(w: &mut World, id: ItemId) {
    let d = data(w, id);
    let cur = w.item(id).map_or(0, |i| i.item_id);
    let next = if cur == d.active {
        d.inactive
    } else {
        d.active
    };
    crate::simple::item_id(w, id, next);
    let cur = w.item(id).map_or(0, |i| i.item_id);
    ops::set_visible(w, id, cur != 0x1);
}

/// `Server.Items.ToggleItem`.
pub static TOGGLE_ITEM: ItemType = ItemType {
    full_name: "Server.Items.ToggleItem",
    parent: Some(&ITEM_BASE),
    // The constructor takes the two graphics: no `[Constructible]` form without them.
    construct: None,
    make: Some(|w| toggle_item_new(w, 0, 0, None)),
    on_double_click: |w, id, from| {
        let gm = muo_content::mx::access_level(w, from) >= AccessLevel::GameMaster;
        if gm {
            toggle(w, id);
        } else if !data(w, id).players_can_toggle {
            return;
        }
        if deps::mobile_in_range(w, from, ops::get_world_location(w, id), 1) {
            toggle(w, id);
        } else {
            deps::send_localized_message(w, from, 500446); // That is too far away.
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
        let d = data(w, id);
        out.write_i32(d.inactive);
        out.write_i32(d.active);
        out.write_bool(d.players_can_toggle);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let inactive = r.read_i32()?;
        let active = r.read_i32()?;
        let players_can_toggle = r.read_bool()?;
        set_data(
            w,
            id,
            ToggleData {
                inactive,
                active,
                players_can_toggle,
            },
        );
        Ok(())
    },
    ..ITEM_BASE
};

const INT: Kind = Kind::Plain(PropType::Numeric(Numeric::Int));

/// `[SerializedCommandProperty(GameMaster)]` of the three fields.
static PROPS: &[PropDef] = &[
    PropDef {
        name: "InactiveItemId",
        kind: INT,
        read: AccessLevel::GameMaster,
        write: AccessLevel::GameMaster,
        get: |w, e| match e {
            EntityId::Item(i) => Value::Int(i64::from(data(w, i).inactive)),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err("not an item".to_string());
            };
            let mut d = data(w, i);
            d.inactive = v.i32();
            set_data(w, i, d);
            Ok(())
        }),
    },
    PropDef {
        name: "ActiveItemId",
        kind: INT,
        read: AccessLevel::GameMaster,
        write: AccessLevel::GameMaster,
        get: |w, e| match e {
            EntityId::Item(i) => Value::Int(i64::from(data(w, i).active)),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err("not an item".to_string());
            };
            let mut d = data(w, i);
            d.active = v.i32();
            set_data(w, i, d);
            Ok(())
        }),
    },
    PropDef {
        name: "PlayersCanToggle",
        kind: Kind::Plain(PropType::Bool),
        read: AccessLevel::GameMaster,
        write: AccessLevel::GameMaster,
        get: |w, e| match e {
            EntityId::Item(i) => Value::Bool(data(w, i).players_can_toggle),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err("not an item".to_string());
            };
            let mut d = data(w, i);
            d.players_can_toggle = v.bool();
            set_data(w, i, d);
            Ok(())
        }),
    },
];

/// `ToggleItem.ToggleCommand`.
struct ToggleCommand {
    spec: CmdSpec,
}

impl BaseCommand for ToggleCommand {
    fn spec(&self) -> &CmdSpec {
        &self.spec
    }

    fn execute(&self, w: &mut World, run: &mut Run, _e: &CommandEventArgs, obj: Obj) {
        match as_item(obj).filter(|&i| w.item(i).is_some_and(|x| x.ty.is(&TOGGLE_ITEM))) {
            Some(i) => {
                toggle(w, i);
                run.add_response("The item has been toggled.");
            }
            None => run.log_failure("That is not a ToggleItem."),
        }
    }
}

/// `TargetCommands.Register(new ToggleCommand())`.
fn register_commands(w: &mut World) {
    register_command(
        w,
        Rc::new(ToggleCommand {
            spec: CmdSpec::new(
                &["Toggle"],
                AccessLevel::GameMaster,
                CommandSupport::ALL_ITEMS,
                ObjectTypes::Items,
                "Toggle",
                "Toggles a targeted ToggleItem.",
            ),
        }),
    );
}

fn configure(w: &mut World) {
    props::add(w, "Server.Items.ToggleItem", PROPS);
}

pub fn register(types: &mut TypeRegistry) {
    types.register_item(&TOGGLE_ITEM, &[]);
    types.register_phase_default(
        Phase::Configure,
        "Server.Items.ToggleItem.Configure",
        configure,
    );
    // `TargetCommands.Register` needs the implementors `TargetCommands.Configure` makes: after the default priority.
    types.register_phase(
        Phase::Configure,
        "Server.Items.ToggleItem.ConfigureCommands",
        DEFAULT_CALL_PRIORITY + 1,
        register_commands,
    );
}
