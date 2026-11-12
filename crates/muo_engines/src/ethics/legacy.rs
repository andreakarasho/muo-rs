//! `EthicsPersistence : Item`: the pre-static-system save record. It saves nothing, and its
//! `Deserialize` reads the two ethics and deletes the item at the next timer tick.

use super::persist::read_ethic;
use super::{Entity, EthicData, EthicKind, st_mut};
use muo_primitives::TimeSpan;
use muo_server::item::{ITEM_BASE, ItemType, base};
use muo_server::type_registry::TypeRegistry;

/// `Server.Ethics.EthicsPersistence` (`[TypeAlias("Server.Ethics.EthicsPersistance")]`).
pub static ETHICS_PERSISTENCE: ItemType = ItemType {
    full_name: "Server.Ethics.EthicsPersistence",
    parent: Some(&ITEM_BASE),
    construct: None,
    serialize: |_, _, _| {},
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        // `new HeroEthic().Deserialize(reader)`, then `new EvilEthic()`: each is a new entity in the system
        // (`Ethic.Ethics` is a fresh array on every read, so `RegisterEthic` always succeeds).
        for kind in [EthicKind::Hero, EthicKind::Evil] {
            let serial = st_mut(w).new_serial();
            let data = EthicData {
                kind,
                created: w.now,
                players: Vec::new(),
            };
            st_mut(w).entities.insert(serial, Entity::Ethic(data));
            read_ethic(w, r, serial, kind)?;
        }
        muo_server::timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
            base::delete(w, id)
        });
        Ok(())
    },
    ..ITEM_BASE
};

pub fn register(types: &mut TypeRegistry) {
    types.register_item(&ETHICS_PERSISTENCE, &["Server.Ethics.EthicsPersistance"]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use muo_io::{GenericReader, SaveReader};
    use muo_server::World;

    #[test]
    fn legacy_record_reads_both_ethics_after_the_item() {
        let mut w = World::new();
        *st_mut(&mut w) = Default::default();
        // each ethic: EthicsEntity 0, Ethic 1, class version i32 0
        let one = [0u8, 1, 0, 0, 0, 0];
        let bytes = [one, one].concat();
        let mut r = SaveReader::new(&bytes);
        for kind in [EthicKind::Hero, EthicKind::Evil] {
            let serial = st_mut(&mut w).new_serial();
            let d = EthicData {
                kind,
                created: w.now,
                players: Vec::new(),
            };
            st_mut(&mut w).entities.insert(serial, Entity::Ethic(d));
            read_ethic(&mut w, &mut r, serial, kind).unwrap();
        }
        assert_eq!(r.position(), 12);
    }
}
