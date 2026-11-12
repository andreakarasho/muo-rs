//! `SendBatchEntities`, many 0xF3 world entities in one 0xF7
//! container (High Seas clients only).

use muo_server::World;
use muo_server::ids::EntityId;
use muo_server::net_state::NetStateId;
use muo_server::packets::container_builder::PacketContainerBuilder;
use muo_server::packets::entity::create_world_entity;

/// `SendBatchEntities(ns, entities)`.
pub fn send_batch_entities(w: &mut World, ns: NetStateId, entities: &[EntityId]) {
    if !w.net_state(ns).is_some_and(|d| d.caps.high_seas()) || w.cannot_send_packets(ns) {
        return;
    }
    let mut builder = PacketContainerBuilder::new();
    for &e in entities {
        builder.push(&create_world_entity(w, e, true));
    }
    w.send(ns, &builder.finish());
}
