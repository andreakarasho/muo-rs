//! `FactionPersistence : Item`: the pre-static-system save record. It saves nothing, and its
//! `Deserialize` reads the faction and town states (`FactionState`, `TownState`) and deletes the item at the next
//! timer tick.
//! shortcut: the legacy states after the base item are not read (they are not migrated); upgrade by adding the
//! version 0 `FactionState`/`TownState` readers if an old world ever needs it. No such record exists in a save
//! written by a current server (only the type name is in the save's type table).

use muo_primitives::TimeSpan;
use muo_server::item::{ITEM_BASE, ItemType, base};
use muo_server::type_registry::TypeRegistry;

/// `Server.Factions.FactionPersistence` (`[TypeAlias("Server.Factions.FactionPersistance")]`).
pub static FACTION_PERSISTENCE: ItemType = ItemType {
    full_name: "Server.Factions.FactionPersistence",
    parent: Some(&ITEM_BASE),
    construct: None,
    make: None,
    serialize: |_, _, _| {},
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        muo_server::timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
            base::delete(w, id)
        });
        Ok(())
    },
    ..ITEM_BASE
};

pub fn register(types: &mut TypeRegistry) {
    types.register_item(
        &FACTION_PERSISTENCE,
        &["Server.Factions.FactionPersistance"],
    );
}
