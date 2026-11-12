//! `MapUO`: the assistant protocol's party (0x00) and guild (0x01) member
//! location queries, answered with a 0xF0 packet (sub-command 0x01 party, 0x02 guild) that lists the members the
//! asker cannot already see.
//!
//! Parties and guilds live elsewhere: [`MapUoHooks`] supplies their member lists. Unset, the asker has no party and
//! no guild, which sends nothing.

use muo_io::SpanReader;
use muo_net::net_state::NetState;
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::mobile::part1::combat::in_update_range;
use muo_server::net_state::NetStateId;

use super::incoming_game::begin;
use crate::systems::assistants::assistant_protocol;

type Members = Option<Vec<Option<MobileId>>>;

/// The member lists of `Party.Get(from)` and `from.Guild as Guild`; a member that no longer exists is `None`.
#[derive(Default)]
pub struct MapUoHooks {
    pub party_members: Option<fn(&World, MobileId) -> Members>,
    pub guild_members: Option<fn(&World, MobileId) -> Members>,
}

/// `MapUO.Configure`.
pub fn configure(w: &mut World) {
    assistant_protocol::register(w, 0x00, true, query_party_member_locations);
    assistant_protocol::register(w, 0x01, true, query_guild_member_locations);
}

fn query_guild_member_locations(w: &mut World, ns: &mut NetState, body: &[u8]) {
    let (id, from) = begin(w, ns);
    let Some(from) = from else { return };
    let send_locations = SpanReader::new(body).read_boolean().unwrap_or(false);
    let members = w
        .resources
        .get::<MapUoHooks>()
        .and_then(|h| h.guild_members)
        .and_then(|f| f(w, from));
    send_guild_member_locations(w, id, from, members.as_deref(), send_locations);
}

fn query_party_member_locations(w: &mut World, ns: &mut NetState, _body: &[u8]) {
    let (id, from) = begin(w, ns);
    let Some(from) = from else { return };
    let members = w
        .resources
        .get::<MapUoHooks>()
        .and_then(|h| h.party_members)
        .and_then(|f| f(w, from));
    if let Some(members) = members {
        send_party_member_locations(w, id, from, Some(&members));
    }
}

fn map_id(w: &World, m: MobileId) -> u8 {
    w.mobile(m).and_then(|m| m.map).map_or(0, |m| m.0)
}

/// Members with a connection, other than `from`, for the packet. `skip_visible` leaves out those `from` sees.
fn listed(
    w: &mut World,
    from: MobileId,
    members: &[Option<MobileId>],
    skip_visible: bool,
) -> Vec<MobileId> {
    let floc = w.mobile(from).map(|m| m.location).unwrap_or_default();
    let from_ty = w.mobile(from).map(|f| f.ty);
    let mut out = Vec::new();
    for m in members.iter().flatten().copied() {
        let Some(mm) = w.mobile(m) else { continue };
        if mm.net_state.is_none() || m == from {
            continue;
        }
        let near = in_update_range(floc, mm.location);
        if !(skip_visible && near && from_ty.is_some_and(|f| (f.can_see_mobile)(w, from, m))) {
            out.push(m);
        }
    }
    out
}

fn finish(mut p: Vec<u8>) -> Vec<u8> {
    p.extend_from_slice(&0i32.to_be_bytes());
    let len = p.len() as u16;
    p[1..3].copy_from_slice(&len.to_be_bytes());
    p
}

/// `SendGuildMemberLocations(ns, from, guild, sendLocations)`.
pub fn send_guild_member_locations(
    w: &mut World,
    ns: NetStateId,
    from: MobileId,
    members: Option<&[Option<MobileId>]>,
    send_locations: bool,
) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let members = members.unwrap_or(&[]);
    let list = listed(w, from, members, send_locations);
    if list.is_empty() {
        return;
    }
    let mut p = vec![
        0xF0,
        0,
        0,
        0x02,
        u8::from(!members.is_empty() && send_locations),
    ];
    for m in list {
        let Some(mm) = w.mobile(m) else { continue };
        p.extend_from_slice(&mm.serial.value().to_be_bytes());
        if send_locations {
            p.extend_from_slice(&(mm.location.x as i16).to_be_bytes());
            p.extend_from_slice(&(mm.location.y as i16).to_be_bytes());
            p.push(map_id(w, m));
            p.push(if (mm.ty.alive)(w, m) {
                let max = (mm.ty.hits_max)(w, m).max(1);
                (mm.hits * 100 / max) as u8
            } else {
                0
            });
        }
    }
    w.send(ns, &finish(p));
}

/// `SendPartyMemberLocations(ns, from, party)`.
pub fn send_party_member_locations(
    w: &mut World,
    ns: NetStateId,
    from: MobileId,
    members: Option<&[Option<MobileId>]>,
) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let list = listed(w, from, members.unwrap_or(&[]), true);
    if list.is_empty() {
        return;
    }
    let mut p = vec![0xF0, 0, 0, 0x01];
    for m in list {
        let Some(mm) = w.mobile(m) else { continue };
        p.extend_from_slice(&mm.serial.value().to_be_bytes());
        p.extend_from_slice(&(mm.location.x as i16).to_be_bytes());
        p.extend_from_slice(&(mm.location.y as i16).to_be_bytes());
        p.push(map_id(w, m));
    }
    w.send(ns, &finish(p));
}
