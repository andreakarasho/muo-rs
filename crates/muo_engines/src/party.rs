//! Parties: `Party`, `PartyMemberInfo`, `PartyPackets`, `PartyCommandHandlers`, `AddPartyTarget`,
//! `RemovePartyTarget`, `DeclineTimer` and `RemoveFromPartyEntry`'s click.
//!
//! `Mobile.Party` is a [`PartyId`]. A real party is `0x8000_0000 | n`; a pending invitation (the inviting
//! `Mobile` held in `Mobile.Party`) is the leader's mobile serial, which never has the high bit.

use indexmap::{IndexMap, IndexSet};
use muo_content::context_menus::{ContextMenuHooks, PartyInfo};
use muo_content::hooks::{WorldHooks, world_hooks};
use muo_content::items::common::hooks_mut as common_item_hooks_mut;
use muo_content::mx;
use muo_content::network::incoming_game::{GameHooks, PartyHandler};
use muo_primitives::{Serial, TimeSpan};
use muo_server::ids::MobileId;
use muo_server::interfaces::PartyId;
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part1::{combat, props_b};
use muo_server::mobile::part2::deps as mdeps;
use muo_server::packets::message::{self, AffixType, MessageType};
use muo_server::packets::mobile as mpk;
use muo_server::target::{self, TargetFlags, Targeted};
use muo_server::timer;
use muo_server::world::World;

/// `Party.Capacity`.
pub const CAPACITY: usize = 10;
const PARTY_BIT: u32 = 0x8000_0000;

/// `PartyMemberInfo`.
#[derive(Copy, Clone, Debug)]
pub struct MemberInfo {
    pub mobile: MobileId,
    pub can_loot: bool,
}

/// One `Party`.
#[derive(Debug)]
pub struct PartyState {
    pub leader: MobileId,
    pub members: Vec<MemberInfo>,
    pub candidates: Vec<MobileId>,
    /// `m_Listeners`: staff listening.
    pub listeners: IndexSet<MobileId>,
}

/// Every live party, and `DeclineTimer.m_Table`.
#[derive(Default)]
pub struct Parties {
    parties: IndexMap<u32, PartyState>,
    next: u32,
    decline: IndexMap<MobileId, timer::TimerHandle>,
}

fn pid(n: u32) -> PartyId {
    PartyId(PARTY_BIT | n)
}

fn key(id: PartyId) -> Option<u32> {
    (id.0 & PARTY_BIT != 0).then_some(id.0 & !PARTY_BIT)
}

/// `Party.Get(m)`: `m?.Party as Party`.
pub fn get(w: &World, m: MobileId) -> Option<PartyId> {
    let id = w.mobile(m)?.party?;
    let k = key(id)?;
    w.resources
        .get::<Parties>()?
        .parties
        .contains_key(&k)
        .then_some(id)
}

/// `m.Party as Mobile`: the leader a pending invitation names.
pub fn invited_by(w: &World, m: MobileId) -> Option<MobileId> {
    let id = w.mobile(m)?.party?;
    key(id).is_none().then(|| MobileId(Serial::new(id.0)))
}

pub fn state(w: &World, id: PartyId) -> Option<&PartyState> {
    w.resources.get::<Parties>()?.parties.get(&key(id)?)
}

fn state_mut(w: &mut World, id: PartyId) -> Option<&mut PartyState> {
    w.resources.get_mut::<Parties>().parties.get_mut(&key(id)?)
}

fn set_party(w: &mut World, m: MobileId, value: Option<PartyId>) {
    if let Some(x) = w.mobile_mut(m) {
        x.party = value;
    }
}

/// `Contains(m)`.
pub fn contains(w: &World, p: PartyId, m: MobileId) -> bool {
    state(w, p).is_some_and(|s| s.members.iter().any(|i| i.mobile == m))
}

fn members(w: &World, p: PartyId) -> Vec<MobileId> {
    state(w, p).map_or_else(Vec::new, |s| s.members.iter().map(|i| i.mobile).collect())
}

fn ns_of(w: &World, m: MobileId) -> Option<muo_server::net_state::NetStateId> {
    props_b::net_state(w, m)
}

fn send_to(w: &mut World, m: MobileId, bytes: &[u8]) {
    if let Some(ns) = ns_of(w, m) {
        w.send(ns, bytes);
    }
}

// ---- PartyPackets ----

/// `CreatePartyMemberList`.
pub fn member_list_packet(members: &[MobileId]) -> Vec<u8> {
    let len = 7 + members.len() * 4;
    let mut b = vec![0xBF];
    b.extend((len as u16).to_be_bytes());
    b.extend(0x06u16.to_be_bytes());
    b.push(0x01);
    b.push(members.len() as u8);
    for m in members {
        b.extend(m.0.value().to_be_bytes());
    }
    b
}

/// `CreatePartyRemoveMember(buffer, removed, p)` (`p` empty for the empty party sent to the removed player).
pub fn remove_member_packet(removed: Serial, members: &[MobileId]) -> Vec<u8> {
    let len = 11 + members.len() * 4;
    let mut b = vec![0xBF];
    b.extend((len as u16).to_be_bytes());
    b.extend(0x06u16.to_be_bytes());
    b.push(0x02);
    b.push(members.len() as u8);
    b.extend(removed.value().to_be_bytes());
    for m in members {
        b.extend(m.0.value().to_be_bytes());
    }
    b
}

/// `CreatePartyTextMessage`.
pub fn text_message_packet(from: Serial, text: &str, to_all: bool) -> Vec<u8> {
    let units: Vec<u16> = text.encode_utf16().collect();
    let len = 12 + units.len() * 2;
    let mut b = vec![0xBF];
    b.extend((len as u16).to_be_bytes());
    b.extend(0x06u16.to_be_bytes());
    b.push(if to_all { 0x04 } else { 0x03 });
    b.extend(from.value().to_be_bytes());
    for u in units {
        b.extend(u.to_be_bytes());
    }
    b.extend([0, 0]);
    b
}

/// `SendPartyInvitation` body.
pub fn invitation_packet(leader: Serial) -> Vec<u8> {
    let mut b = vec![0xBF, 0, 10];
    b.extend(0x06u16.to_be_bytes());
    b.push(0x07);
    b.extend(leader.value().to_be_bytes());
    b
}

fn send_remove_member(w: &mut World, to: MobileId, removed: MobileId, p_members: &[MobileId]) {
    let bytes = remove_member_packet(removed.0, p_members);
    send_to(w, to, &bytes);
}

// ---- Party ----

pub fn new_party(w: &mut World, leader: MobileId) -> PartyId {
    let ml = muo_content::core::ml(w);
    let parties = w.resources.get_mut::<Parties>();
    parties.next += 1;
    let n = parties.next;
    parties.parties.insert(
        n,
        PartyState {
            leader,
            members: vec![MemberInfo {
                mobile: leader,
                can_loot: !ml,
            }],
            candidates: Vec::new(),
            listeners: IndexSet::new(),
        },
    );
    pid(n)
}

/// `Active`.
pub fn active(w: &World, p: PartyId) -> bool {
    state(w, p).is_some_and(|s| s.members.len() > 1)
}

fn attr_update(w: &mut World, p: PartyId, m: MobileId, packet: Vec<u8>) {
    for c in members(w, p) {
        let same = w
            .mobile(c)
            .map(|x| (x.map, x.location))
            .zip(w.mobile(m).map(|x| (x.map, x.location)));
        let Some(((cmap, cloc), (mmap, mloc))) = same else {
            continue;
        };
        if c != m
            && ns_of(w, c).is_some()
            && cmap == mmap
            && combat::in_update_range(cloc, mloc)
            && mdeps::can_see_mobile(w, c, m)
        {
            send_to(w, c, &packet);
        }
    }
}

/// `OnStamChanged(m)`.
pub fn on_stam_changed(w: &mut World, p: PartyId, m: MobileId) {
    let packet = mpk::create_mobile_stam(w, m, true);
    attr_update(w, p, m, packet);
}

/// `OnManaChanged(m)`.
pub fn on_mana_changed(w: &mut World, p: PartyId, m: MobileId) {
    let packet = mpk::create_mobile_mana(w, m, true);
    attr_update(w, p, m, packet);
}

/// `OnStatsQuery(beholder, beheld)`.
pub fn on_stats_query(w: &mut World, p: PartyId, beholder: MobileId, beheld: MobileId) {
    let near = w
        .mobile(beholder)
        .zip(w.mobile(beheld))
        .is_some_and(|(a, b)| a.map == b.map && combat::in_update_range(a.location, b.location));
    if beholder != beheld && contains(w, p, beholder) && near {
        let Some(ns) = ns_of(w, beholder) else { return };
        if !mdeps::can_see_mobile(w, beholder, beheld) {
            let renamed = muo_server::mobile::base::can_be_renamed_by(w, beheld, beholder);
            mpk::send_mobile_status_compact(w, ns, beheld, renamed);
        }
        mpk::send_mobile_attributes(w, ns, beheld, true);
    }
}

/// `Add(m)`.
pub fn add(w: &mut World, p: PartyId, m: MobileId) {
    if contains(w, p, m) {
        return;
    }
    let ml = muo_content::core::ml(w);
    let ns = ns_of(w, m);
    if let Some(s) = state_mut(w, p) {
        s.members.push(MemberInfo {
            mobile: m,
            can_loot: !ml,
        });
    }
    set_party(w, m, Some(p));

    let all = members(w, p);
    let list = member_list_packet(&all);
    for f in all {
        send_to(w, f, &list);
        if f != m {
            let fns = ns_of(w, f);
            if let Some(fns) = fns {
                let renamed = muo_server::mobile::base::can_be_renamed_by(w, m, f);
                mpk::send_mobile_status_compact(w, fns, m, renamed);
                let attrs = mpk::create_mobile_attributes(w, m, true);
                w.send(fns, &attrs);
            }
            if let Some(ns) = ns {
                let renamed = muo_server::mobile::base::can_be_renamed_by(w, f, m);
                mpk::send_mobile_status_compact(w, ns, f, renamed);
                mpk::send_mobile_attributes(w, ns, f, true);
            }
        }
    }
}

fn name_of(w: &World, m: MobileId) -> String {
    w.mobile(m)
        .and_then(|x| (x.ty.name)(w, m))
        .unwrap_or_default()
}

/// `SendToAll(span)`.
fn send_to_all_bytes(w: &mut World, p: PartyId, bytes: &[u8]) {
    for m in members(w, p) {
        send_to(w, m, bytes);
    }
}

/// `SendToAllListeners(span)`.
fn send_to_all_listeners_bytes(w: &mut World, p: PartyId, bytes: &[u8]) {
    let listeners: Vec<MobileId> =
        state(w, p).map_or_else(Vec::new, |s| s.listeners.iter().copied().collect());
    for l in listeners {
        if w.mobile(l).is_some_and(|x| x.party != Some(p)) {
            send_to(w, l, bytes);
        }
    }
}

/// `SendToAllListeners(text)`.
fn send_to_all_listeners_text(w: &mut World, p: PartyId, text: &str) {
    if state(w, p).is_none_or(|s| s.listeners.is_empty()) {
        return;
    }
    let bytes = message::create_message(
        Serial::MINUS_ONE,
        -1,
        MessageType::REGULAR,
        0x3B2,
        3,
        false,
        Some("ENU"),
        "System",
        text,
    );
    send_to_all_listeners_bytes(w, p, &bytes);
}

/// `SendToAll(number, args, hue)`.
pub fn send_to_all(w: &mut World, p: PartyId, number: i32, args: &str, hue: i32) {
    let bytes = message::create_message_localized(
        Serial::MINUS_ONE,
        -1,
        MessageType::REGULAR,
        hue,
        3,
        number,
        "System",
        args,
    );
    send_to_all_bytes(w, p, &bytes);
    send_to_all_listeners_bytes(w, p, &bytes);
}

fn affix_message(name: &str, number: i32) -> Vec<u8> {
    message::create_message_localized_affix(
        Serial::MINUS_ONE,
        -1,
        MessageType::LABEL,
        0x3B2,
        3,
        number,
        "",
        AffixType::PREPEND | AffixType::SYSTEM,
        name,
        "",
    )
}

/// `OnAccept(from, force)`.
pub fn on_accept(w: &mut World, p: PartyId, from: MobileId, force: bool) {
    let leader = state(w, p).map(|s| s.leader);
    let ours = leader.and_then(|l| crate::factions::find(w, l));
    let theirs = crate::factions::find(w, from);
    if !force && ours.is_some() && theirs.is_some() && ours != theirs {
        return;
    }
    // : joined the party.
    let bytes = affix_message(&name_of(w, from), 1008094);
    send_to_all_bytes(w, p, &bytes);
    send_to_all_listeners_bytes(w, p, &bytes);

    mx::msg(w, from, 1005445); // You have been added to the party.

    if let Some(s) = state_mut(w, p) {
        s.candidates.retain(|c| *c != from);
    }
    add(w, p, from);
}

/// `OnDecline(from, leader)`.
pub fn on_decline(w: &mut World, p: PartyId, from: MobileId, leader: MobileId) {
    mx::msg_args(w, leader, 1008091, &name_of(w, from)); // : Does not wish to join the party.
    mx::msg(w, from, 1008092); // You notify them that you do not wish to join the party.

    if let Some(s) = state_mut(w, p) {
        s.candidates.retain(|c| *c != from);
    }
    send_remove_member(w, from, from, &[]);

    let (cands, count) = state(w, p).map_or((0, 0), |s| (s.candidates.len(), s.members.len()));
    if cands != 0 || count > 1 {
        return;
    }
    for m in members(w, p) {
        send_remove_member(w, m, m, &[]);
        set_party(w, m, None);
    }
    if let Some(s) = state_mut(w, p) {
        s.members.clear();
    }
    forget_if_empty(w, p);
}

/// A party whose `Members` list is cleared is dead; nothing refers to it once every member's `Party` is cleared,
/// so the table drops it.
fn forget_if_empty(w: &mut World, p: PartyId) {
    if state(w, p).is_some_and(|s| s.members.is_empty())
        && let Some(k) = key(p)
    {
        w.resources.get_mut::<Parties>().parties.shift_remove(&k);
    }
}

/// `Remove(m)`.
pub fn remove(w: &mut World, p: PartyId, m: MobileId) {
    if state(w, p).is_some_and(|s| s.leader == m) {
        disband(w, p);
        return;
    }
    let removed = state_mut(w, p).is_some_and(|s| {
        s.members
            .iter()
            .position(|i| i.mobile == m)
            .map(|i| s.members.remove(i))
            .is_some()
    });
    if removed {
        // Send empty party to the player removed
        send_remove_member(w, m, m, &[]);
        set_party(w, m, None);
        mx::msg(w, m, 1005451); // You have been removed from the party.
        let rest = members(w, p);
        let bytes = remove_member_packet(m.0, &rest);
        send_to_all_bytes(w, p, &bytes);
        send_to_all(w, p, 1005452, "", 0x3B2); // A player has been removed from your party.
    }
    if state(w, p).is_some_and(|s| s.members.len() == 1) {
        send_to_all(w, p, 1005450, "", 0x3B2); // The last person has left the party...
        disband(w, p);
    }
}

/// `Disband()`.
pub fn disband(w: &mut World, p: PartyId) {
    send_to_all(w, p, 1005449, "", 0x3B2); // Your party has disbanded.
    for m in members(w, p) {
        send_remove_member(w, m, m, &[]);
        set_party(w, m, None);
    }
    if let Some(s) = state_mut(w, p) {
        s.members.clear();
    }
    forget_if_empty(w, p);
}

/// `Invite(from, target)`.
pub fn invite(w: &mut World, from: MobileId, target: MobileId) {
    let ours = crate::factions::find(w, from);
    let theirs = crate::factions::find(w, target);
    if ours.is_some() && theirs.is_some() && ours != theirs {
        mx::msg(w, from, 1008088); // You cannot have players from opposing factions in the same party!
        mx::msg(w, target, 1008093); // The party cannot have members from opposing factions.
        return;
    }
    let p = match get(w, from) {
        Some(p) => p,
        None => {
            let p = new_party(w, from);
            set_party(w, from, Some(p));
            p
        }
    };
    if let Some(s) = state_mut(w, p)
        && !s.candidates.contains(&target)
    {
        s.candidates.push(target);
    }
    // : You are invited to join the party. Type /accept to join or /decline to decline the offer.
    if let Some(ns) = ns_of(w, target) {
        message::send_message_localized_affix(
            w,
            ns,
            Serial::MINUS_ONE,
            -1,
            MessageType::LABEL,
            0x3B2,
            3,
            1008089,
            "",
            AffixType::PREPEND | AffixType::SYSTEM,
            &name_of(w, from),
            "",
        );
    }
    mx::msg(w, from, 1008090); // You have invited them to join the party.

    let inv = invitation_packet(from.0);
    send_to(w, target, &inv);
    set_party(w, target, Some(PartyId(from.0.value())));

    start_decline_timer(w, target, from);
}

/// `DeclineTimer.Start(m, leader)`.
fn start_decline_timer(w: &mut World, m: MobileId, leader: MobileId) {
    if let Some(mut old) = w.resources.get_mut::<Parties>().decline.shift_remove(&m) {
        old.cancel(w);
    }
    let h = timer::start(w, TimeSpan::from_secs(30), TimeSpan::ZERO, 1, move |w| {
        w.resources.get_mut::<Parties>().decline.shift_remove(&m);
        if invited_by(w, m) == Some(leader) {
            handler_decline(w, m, Some(leader));
        }
    });
    w.resources.get_mut::<Parties>().decline.insert(m, h);
}

/// `SendPublicMessage(from, text)`.
pub fn send_public_message(w: &mut World, p: PartyId, from: MobileId, text: &str) {
    let bytes = text_message_packet(from.0, text, true);
    send_to_all_bytes(w, p, &bytes);
    let name = name_of(w, from);
    send_to_all_listeners_text(w, p, &format!("[{name}]: {text}"));
    send_to_staff_message(w, p, from, &format!("[Party]: {text}"));
}

/// `SendPrivateMessage(from, to, text)`.
pub fn send_private_message(w: &mut World, p: PartyId, from: MobileId, to: MobileId, text: &str) {
    let bytes = text_message_packet(from.0, text, false);
    send_to(w, to, &bytes);
    let (a, b) = (name_of(w, from), name_of(w, to));
    send_to_all_listeners_text(w, p, &format!("[{a}]->[{b}]: {text}"));
    send_to_staff_message(w, p, from, &format!("[Party]->[{b}]: {text}"));
}

fn send_to_staff_message(w: &mut World, p: PartyId, from: MobileId, text: &str) {
    let Some(f) = w.mobile(from) else { return };
    let (map, loc, from_access, body, hue, lang, name) = (
        f.map,
        f.location,
        f.access_level,
        muo_server::mobile::part1::props_b::body(w, from),
        f.speech_hue,
        f.language.clone(),
        name_of(w, from),
    );
    let Some(map) = map else { return };
    for ns in muo_server::effects::clients_in_range(w, map, loc, Some(8)) {
        let Some(mob) = w.net_state(ns).and_then(|d| d.mobile) else {
            continue;
        };
        let Some(m) = w.mobile(mob) else { continue };
        if m.access_level >= AccessLevel::GameMaster
            && m.access_level > from_access
            && m.party != Some(p)
            && !state(w, p).is_some_and(|s| s.listeners.contains(&mob))
        {
            let bytes = message::create_message(
                from.0,
                body.body_id(),
                MessageType::REGULAR,
                hue,
                3,
                false,
                lang.as_deref(),
                &name,
                text,
            );
            w.send(ns, &bytes);
        }
    }
}

// ---- events ----

fn on_logout(w: &mut World, m: MobileId) {
    if let Some(p) = get(w, m) {
        remove(w, p, m);
    }
    set_party(w, m, None);
}

fn on_player_death(w: &mut World, from: MobileId) {
    let Some(p) = get(w, from) else { return };
    let killer = w.mobile(from).and_then(|m| m.last_killer);
    let text = match killer {
        Some(k) if k == from => "I killed myself !!".to_owned(),
        None => "I was killed !!".to_owned(),
        Some(k) => format!("I was killed by {} !!", name_of(w, k)),
    };
    send_public_message(w, p, from, &text);
}

fn on_login(w: &mut World, from: MobileId) {
    if get(w, from).is_some() {
        timer::start(w, TimeSpan::from_secs(1), TimeSpan::ZERO, 1, move |w| {
            rejoin(w, from)
        });
    } else {
        set_party(w, from, None);
    }
}

/// `RejoinTimer.OnTick`.
fn rejoin(w: &mut World, who: MobileId) {
    let Some(p) = get(w, who) else { return };
    mx::msg(w, who, 1005437); // You have rejoined the party.
    let all = members(w, p);
    let list = member_list_packet(&all);
    send_to(w, who, &list);
    let ns = ns_of(w, who);
    let who_name = name_of(w, who);
    for m in all {
        if m == who {
            continue;
        }
        let bytes = affix_message(&who_name, 1008087);
        send_to(w, m, &bytes);
        if let Some(mns) = ns_of(w, m) {
            let renamed = muo_server::mobile::base::can_be_renamed_by(w, who, m);
            mpk::send_mobile_status_compact(w, mns, who, renamed);
            let attrs = mpk::create_mobile_attributes(w, who, true);
            w.send(mns, &attrs);
        }
        if let Some(ns) = ns {
            let renamed = muo_server::mobile::base::can_be_renamed_by(w, m, who);
            mpk::send_mobile_status_compact(w, ns, m, renamed);
            mpk::send_mobile_attributes(w, ns, m, true);
        }
    }
}

// ---- ListenToParty ----

fn listen_command(w: &mut World, e: &mut muo_server::commands::CommandEventArgs) {
    let from = e.mobile;
    target::begin_target(w, from, -1, false, TargetFlags::NONE, move |w, from, t| {
        if let Targeted::Entity(muo_server::ids::EntityId::Mobile(m)) = t {
            let Some(p) = get(w, m) else {
                mx::send_message(w, from, "They are not in a party.");
                return;
            };
            let removed = state_mut(w, p).is_some_and(|s| s.listeners.shift_remove(&from));
            if removed {
                mx::send_message(w, from, "You are no longer listening to that party.");
            } else {
                if let Some(s) = state_mut(w, p) {
                    s.listeners.insert(from);
                }
                mx::send_message(w, from, "You are now listening to that party.");
            }
        }
    });
    mx::send_message(w, from, "Target a partied player.");
}

// ---- PartyCommandHandlers ----

fn handler_add(w: &mut World, from: MobileId) {
    let p = get(w, from);
    let info =
        p.and_then(|p| state(w, p).map(|s| (s.leader, s.members.len() + s.candidates.len())));
    match info {
        Some((leader, _)) if leader != from => mx::msg(w, from, 1005453), // only the leader may add
        Some((_, n)) if n >= CAPACITY => mx::msg(w, from, 1008095),       // 10 in your party
        _ => {
            mx::msg(w, from, 1005454); // Who would you like to add to your party?
            target::begin_target(w, from, 8, false, TargetFlags::NONE, add_party_target);
        }
    }
}

/// `AddPartyTarget.OnTarget`.
fn add_party_target(w: &mut World, from: MobileId, t: Targeted) {
    let Targeted::Entity(muo_server::ids::EntityId::Mobile(m)) = t else {
        mx::msg(w, from, 1005442); // You may only add living things to your party!
        return;
    };
    let p = get(w, from);
    let mp = get(w, m);
    let leader_other = p
        .and_then(|p| state(w, p))
        .is_some_and(|s| s.leader != from);
    let full = p
        .and_then(|p| state(w, p))
        .is_some_and(|s| s.members.len() + s.candidates.len() >= CAPACITY);
    let human = w.mobile(m).is_some_and(|_| {
        w.resources
            .get::<muo_primitives::BodyTable>()
            .is_some_and(|t| t.is_human(muo_server::mobile::part1::props_b::body(w, m)))
    });
    let player = w.mobile(m).is_some_and(|x| x.player);
    if from == m {
        mx::msg(w, from, 1005439); // You cannot add yourself to a party.
    } else if leader_other {
        mx::msg(w, from, 1005453);
    } else if invited_by(w, m).is_some() {
        // already has an invitation: nothing
    } else if full {
        mx::msg(w, from, 1008095);
    } else if !player && human {
        mx::say_to(
            w,
            m,
            from,
            "Nay, I would rather stay here and watch a nail rust.",
        ); // 1005443
    } else if !player {
        mx::msg(w, from, 1005444); // The creature ignores your offer.
    } else if mp.is_some() && mp == p {
        mx::msg(w, from, 1005440); // This person is already in your party!
    } else if mp.is_some() {
        mx::msg(w, from, 1005441); // This person is already in a party!
    } else {
        invite(w, from, m);
    }
}

fn handler_remove(w: &mut World, from: MobileId, target_m: Option<MobileId>) {
    let Some(p) = get(w, from) else {
        mx::msg(w, from, 3000211); // You are not in a party.
        return;
    };
    let leader = state(w, p).map(|s| s.leader);
    if leader == Some(from) && target_m.is_none() {
        mx::msg(w, from, 1005455); // Who would you like to remove from your party?
        target::begin_target(w, from, 8, false, TargetFlags::NONE, |w, from, t| {
            if let Targeted::Entity(muo_server::ids::EntityId::Mobile(m)) = t {
                remove_target(w, from, m);
            }
        });
    } else if (leader == Some(from) || target_m == Some(from))
        && let Some(m) = target_m
    {
        remove(w, p, m);
    }
}

/// `RemovePartyTarget.OnTarget` and `RemoveFromPartyEntry.OnClick`.
pub fn remove_target(w: &mut World, from: MobileId, m: MobileId) {
    let Some(p) = get(w, from) else { return };
    if state(w, p).is_none_or(|s| s.leader != from) || !contains(w, p, m) {
        return;
    }
    if from == m {
        mx::msg(w, from, 1005446); // You may only remove yourself from a party if you are not the leader.
    } else {
        remove(w, p, m);
    }
}

fn handler_private(w: &mut World, from: MobileId, target_m: Option<MobileId>, text: &str) {
    let text = text.trim();
    if text.chars().count() > 128 || text.is_empty() {
        return;
    }
    match (get(w, from), target_m) {
        (Some(p), Some(t)) if contains(w, p, t) => send_private_message(w, p, from, t, text),
        _ => mx::msg(w, from, 3000211), // You are not in a party.
    }
}

fn handler_public(w: &mut World, from: MobileId, text: &str) {
    // the length is tested before trimming
    if text.chars().count() > 128 || text.trim().is_empty() {
        return;
    }
    match get(w, from) {
        Some(p) => send_public_message(w, p, from, text.trim()),
        None => mx::msg(w, from, 3000211),
    }
}

fn handler_can_loot(w: &mut World, from: MobileId, can_loot: bool) {
    let Some(p) = get(w, from) else {
        mx::msg(w, from, 3000211);
        return;
    };
    let found = state_mut(w, p)
        .and_then(|s| s.members.iter_mut().find(|i| i.mobile == from))
        .map(|i| i.can_loot = can_loot)
        .is_some();
    if found {
        if can_loot {
            mx::msg(w, from, 1005447); // You have chosen to allow your party to loot your corpse.
        } else {
            mx::msg(w, from, 1005448); // You have chosen to prevent your party from looting your corpse.
        }
    }
}

fn handler_accept(w: &mut World, from: MobileId, _sent_leader: Option<MobileId>) {
    let leader = invited_by(w, from);
    set_party(w, from, None);
    let p = leader.and_then(|l| get(w, l));
    let is_candidate = p
        .and_then(|p| state(w, p))
        .is_some_and(|s| s.candidates.contains(&from));
    match p.filter(|_| leader.is_some() && is_candidate) {
        None => mx::msg(w, from, 3000222), // No one has invited you to be in a party.
        Some(p) => {
            if state(w, p).is_some_and(|s| s.members.len() + s.candidates.len() <= CAPACITY) {
                on_accept(w, p, from, false);
            }
        }
    }
}

fn handler_decline(w: &mut World, from: MobileId, _sent_leader: Option<MobileId>) {
    let leader = invited_by(w, from);
    set_party(w, from, None);
    let p = leader.and_then(|l| get(w, l));
    let is_candidate = p
        .and_then(|p| state(w, p))
        .is_some_and(|s| s.candidates.contains(&from));
    match (p, leader) {
        (Some(p), Some(l)) if is_candidate => on_decline(w, p, from, l),
        _ => mx::msg(w, from, 3000222),
    }
}

/// The hooks into Mobile, corpses, the context menu and the 0xBF packet; the events; `[ListenToParty`.
pub fn configure(w: &mut World) {
    w.events.logout.subscribe(|w, m| on_logout(w, *m));
    muo_server::commands::CommandSystem::register(
        w,
        "ListenToParty",
        AccessLevel::GameMaster,
        &[],
        std::rc::Rc::new(listen_command),
    );

    w.resources.get_mut::<GameHooks>().party = Some(PartyHandler {
        on_add: handler_add,
        on_remove: handler_remove,
        on_private_message: handler_private,
        on_public_message: handler_public,
        on_set_can_loot: handler_can_loot,
        on_accept: handler_accept,
        on_decline: handler_decline,
    });

    let h = w.resources.get_mut::<ContextMenuHooks>();
    h.party_capacity = Some(CAPACITY);
    h.party_invite = Some(invite);
    h.party_get = Some(|w, m| {
        let id = get(w, m)?;
        let s = state(w, id)?;
        Some(PartyInfo {
            id,
            leader: s.leader,
            members: s.members.len(),
            candidates: s.candidates.len(),
        })
    });

    // `Party.Get(from)?.Contains(to)`
    muo_content::spells::hooks::hooks_mut(w).party_contains =
        Some(|w, from, to| get(w, from).is_some_and(|p| contains(w, p, to)));

    common_item_hooks_mut(w).party_can_loot = Some(|w, owner, from| {
        get(w, owner).and_then(|p| state(w, p)).is_some_and(|s| {
            s.members.iter().any(|i| i.mobile == from)
                && s.members.iter().any(|i| i.mobile == owner && i.can_loot)
        })
    });

    w.resources.get_mut::<WorldHooks>().party_members = Some(|w, m| match get(w, m) {
        Some(p) => members(w, p),
        None => Vec::new(),
    });
    w.resources
        .get_mut::<muo_content::network::map_uo::MapUoHooks>()
        .party_members = Some(|w, m| {
        let p = get(w, m)?;
        Some(members(w, p).into_iter().map(Some).collect())
    });

    w.resources
        .get_mut::<muo_content::player_mobile::hooks::SystemHooks>()
        .party_remove_member = Some(remove_target);

    *w.resources.get_mut::<mdeps::PartyDeps>() = mdeps::PartyDeps {
        on_stats_query: Some(on_stats_query),
        on_mana_changed: Some(on_mana_changed),
        on_stam_changed: Some(on_stam_changed),
    };
}

/// `Initialize`: the `[OnEvent]` handlers of PlayerMobile.
pub fn initialize(w: &mut World) {
    muo_content::network::events::subscribe_ranked::<muo_content::network::events::PlayerLogin>(
        w,
        7,
        |w, e| on_login(w, e.0),
    );
    muo_content::player_mobile::subscribe_death(w, on_player_death);
}

#[allow(dead_code)]
fn _unused(w: &World) -> bool {
    world_hooks(w).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packets_match_the_expected_layouts() {
        let a = MobileId(Serial::new(1));
        let b = MobileId(Serial::new(2));
        assert_eq!(
            member_list_packet(&[a, b]),
            [0xBF, 0, 15, 0, 6, 1, 2, 0, 0, 0, 1, 0, 0, 0, 2]
        );
        assert_eq!(
            remove_member_packet(Serial::new(3), &[a]),
            [0xBF, 0, 15, 0, 6, 2, 1, 0, 0, 0, 3, 0, 0, 0, 1]
        );
        assert_eq!(
            text_message_packet(Serial::new(1), "hi", true),
            [0xBF, 0, 16, 0, 6, 4, 0, 0, 0, 1, 0, b'h', 0, b'i', 0, 0]
        );
        assert_eq!(
            invitation_packet(Serial::new(5)),
            [0xBF, 0, 10, 0, 6, 7, 0, 0, 0, 5]
        );
    }

    #[test]
    fn invitation_ids_never_look_like_parties() {
        let leader = MobileId(Serial::new(0x1234));
        assert!(key(PartyId(leader.0.value())).is_none());
        assert_eq!(key(pid(7)), Some(7));
    }
}
