//! UO gateway: the shard statistics a gateway client asks the freeshard
//! protocol (0xF1) for, sub-commands 0xFE (compact, binary) and 0xFF (extended, text).
//!
//! `GC.GetTotalMemory(false)` is the process working set (`muo_primitives::process_memory`).

use muo_net::net_state::NetState;
use muo_server::World;
use muo_server::net_state::NetStateId;

use super::incoming_game::begin;
use super::protocol_extensions;

/// `UOGateway.Configure`.
pub fn configure(w: &mut World) {
    let enabled = w
        .config
        .get_or_update_setting("uogateway.enabled", true)
        .unwrap_or(true);
    if enabled {
        protocol_extensions::register(w, 0xFE, false, false, query_compact_shard_stats);
        protocol_extensions::register(w, 0xFF, false, false, query_extended_shard_stats);
    }
}

fn clients(w: &World) -> i32 {
    w.net_states.len() as i32 - 1 // Shame if you modify this!
}

fn query_compact_shard_stats(w: &mut World, ns: &mut NetState, _body: &[u8]) {
    let (id, _) = begin(w, ns);
    let age = (w.core.uptime(w.tick_count) / 1000) as u32;
    let (c, i, m) = (clients(w), w.items.len() as i32, w.mobiles.len() as i32);
    send_compact_shard_stats(
        w,
        id,
        age,
        c,
        i,
        m,
        muo_primitives::process_memory::process_memory_bytes(),
    );
}

fn query_extended_shard_stats(w: &mut World, ns: &mut NetState, _body: &[u8]) {
    const TICKS_IN_HOUR: i64 = 1000 * 60 * 60;
    let (id, _) = begin(w, ns);
    let name = crate::login::server_list::server_name(w);
    let age = (w.core.uptime(w.tick_count) / TICKS_IN_HOUR) as i32;
    let (c, i, m) = (clients(w), w.items.len() as i32, w.mobiles.len() as i32);
    send_extended_shard_stats(
        w,
        id,
        &name,
        age,
        c,
        i,
        m,
        (muo_primitives::process_memory::process_memory_bytes() / 1024) as i32,
    );
}

/// `CreateCompactShardStats`: 0x51, length 27.
pub fn create_compact_shard_stats(
    age: u32,
    clients: i32,
    items: i32,
    mobiles: i32,
    mem: i64,
) -> Vec<u8> {
    let mut p = vec![0x51, 0, 27];
    p.extend_from_slice(&clients.to_be_bytes());
    p.extend_from_slice(&items.to_be_bytes());
    p.extend_from_slice(&mobiles.to_be_bytes());
    p.extend_from_slice(&age.to_be_bytes());
    p.extend_from_slice(&mem.to_be_bytes());
    p
}

/// `SendCompactShardStats`.
pub fn send_compact_shard_stats(
    w: &mut World,
    ns: NetStateId,
    age: u32,
    clients: i32,
    items: i32,
    mobiles: i32,
    mem: i64,
) {
    if w.cannot_send_packets(ns) {
        return;
    }
    w.send(
        ns,
        &create_compact_shard_stats(age, clients, items, mobiles, mem),
    );
}

/// The text of `SendExtendedShardStats`: UTF-8, NUL-terminated.
pub fn create_extended_shard_stats(
    name: &str,
    age: i32,
    clients: i32,
    items: i32,
    mobiles: i32,
    mem: i32,
) -> Vec<u8> {
    format!(
        "muo, Name={name}, Age={age}, Clients={clients}, Items={items}, Chars={mobiles}, Mem={mem}K, Ver=2\0"
    )
    .into_bytes()
}

/// `SendExtendedShardStats`.
#[allow(clippy::too_many_arguments)]
pub fn send_extended_shard_stats(
    w: &mut World,
    ns: NetStateId,
    name: &str,
    age: i32,
    clients: i32,
    items: i32,
    mobiles: i32,
    mem: i32,
) {
    if w.cannot_send_packets(ns) {
        return;
    }
    w.send(
        ns,
        &create_extended_shard_stats(name, age, clients, items, mobiles, mem),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_stats_layout() {
        let p = create_compact_shard_stats(3, 1, 2, 4, 5);
        assert_eq!(
            p,
            [
                0x51, 0, 27, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 4, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 5
            ]
        );
    }

    #[test]
    fn extended_stats_text() {
        assert_eq!(
            create_extended_shard_stats("Test", 2, 1, 10, 20, 5),
            b"muo, Name=Test, Age=2, Clients=1, Items=10, Chars=20, Mem=5K, Ver=2\0"
        );
    }
}
