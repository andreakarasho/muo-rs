//! The faction `Silver` currency item. The other faction items (stones, sigils, monoliths, traps) live in
//! `factions`.

use muo_server::item::{ITEM_BASE, ItemType, ops};
use muo_server::type_registry::TypeRegistry;

/// `Server.Factions.Silver`.
pub static SILVER: ItemType = ItemType {
    full_name: "Server.Factions.Silver",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let id = ops::new_item_with_id(w, &SILVER, 0xEF0);
        ops::set_stackable(w, id, true);
        ops::set_amount(w, id, 1);
        id
    }),
    default_weight: |_, _| 0.02,
    get_drop_sound: |w, id| match w.item(id).map_or(0, |i| i.amount) {
        ..=1 => 0x2E4,
        2..=5 => 0x2E5,
        _ => 0x2E6,
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..ITEM_BASE
};

pub fn register(types: &mut TypeRegistry) {
    types.register_item(&SILVER, &[]);
    crate::factions::items::register(types);
    crate::factions::mobiles::register(types);
    crate::factions::power::register(types);
    crate::factions::traps::register(types);
}
