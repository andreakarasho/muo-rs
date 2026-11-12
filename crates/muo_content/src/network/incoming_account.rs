//! Incoming account packets: character creation and play, the
//! login and game login, server selection, the client version, and the auth ids that tie a game login to the
//! account login that earned it.

use muo_primitives::dotnet::IgnoreCase;
use std::net::IpAddr;

use indexmap::IndexMap;
use muo_io::{SeekOrigin, SpanReader};
use muo_net::client_version::{ClientFlags, ClientVersion};
use muo_net::net_state::NetState;
use muo_primitives::{DateTime, TimeSpan};
use muo_server::World;
use muo_server::events::{self, AccountLoginEventArgs};
use muo_server::ids::MobileId;
use muo_server::map::MapId;
use muo_server::net_state::NetStateId;
use muo_server::packets::account::{ALRReason, PMMessage};
use muo_server::race::Races;
use muo_server::timer;

use super::game_server::{GameLoginEventArgs, game_server_login_event};
use super::gateway_server::{ServerListEventArgs, server_list_event};
use super::outgoing;
use crate::accounting::{self, handler};
use crate::login::character_creation::{CharacterCreatedEventArgs, character_created_event};

// Initial capacity and the point at which issuing sweeps expired ids. Not a cap; the window grows rather than
// evicting a live id.
const AUTH_ID_WINDOW_SIZE: usize = 128;

// The gap between PlayServerAck and the game login is seconds. Bounds how long a stolen id stays usable.
const AUTH_ID_LIFETIME: TimeSpan = TimeSpan::from_mins(2);

/// `AuthIDPersistence`.
#[derive(Clone)]
pub struct AuthIdEntry {
    pub age: DateTime,
    pub version: Option<ClientVersion>,
    /// GameLogin skips its password verify when both match, so the id is a bearer token and has to be bound to
    /// whatever earned it.
    pub account: Option<String>,
    pub address: IpAddr,
}

/// `AuthIdResult`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum AuthIdResult {
    /// No such id, or it was issued for a different account or address.
    Rejected,
    /// Right account and address, too old to stand in for the verify.
    Expired,
    /// Issued to this account, from this address, recently. Stands in for the password verify.
    Vouched,
}

/// `_authIDWindow` and `_authIdPurgeThreshold`.
pub struct AuthIdWindow {
    map: IndexMap<i32, AuthIdEntry>,
    purge_threshold: usize,
}

impl Default for AuthIdWindow {
    fn default() -> Self {
        Self {
            map: IndexMap::new(),
            purge_threshold: AUTH_ID_WINDOW_SIZE,
        }
    }
}

impl AuthIdWindow {
    pub fn count(&self) -> usize {
        self.map.len()
    }

    pub fn clear(&mut self) {
        self.map.clear();
        self.purge_threshold = AUTH_ID_WINDOW_SIZE;
    }

    fn purge_expired(&mut self, now: DateTime) {
        self.map.retain(|_, e| now - e.age <= AUTH_ID_LIFETIME);
    }

    /// `RegisterAuthId`.
    pub fn register(&mut self, now: DateTime, entry: AuthIdEntry) -> i32 {
        // Sweep the ids left behind by clients that picked a server and never arrived, but never evict a live one
        // to make room -- the client holding it is on its way to redeem it.
        if self.map.len() >= self.purge_threshold {
            self.purge_expired(now);
            self.purge_threshold = AUTH_ID_WINDOW_SIZE.max(self.map.len() * 2);
        }
        // The id stands in for a password verify, so it has to be unguessable. Zero is reserved: GameLogin reads
        // state.AuthId == 0 as "no auth id was issued".
        let id = loop {
            let mut b = [0u8; 4];
            // RandomNumberGenerator.GetInt32(int.MinValue, int.MaxValue)
            getrandom::fill(&mut b).expect("the operating system's random number generator failed");
            let id = i32::from_le_bytes(b);
            if id != 0 && id != i32::MAX && !self.map.contains_key(&id) {
                break id;
            }
        };
        self.map.insert(id, entry);
        id
    }

    /// `ConsumeAuthId`: spends an auth id, but only for the account and address it was issued to.
    pub fn consume(
        &mut self,
        now: DateTime,
        auth_id: i32,
        username: &str,
        address: IpAddr,
    ) -> (AuthIdResult, Option<AuthIdEntry>) {
        let Some(entry) = self.map.get(&auth_id).cloned() else {
            return (AuthIdResult::Rejected, None);
        };
        // Look, then take: removing before ownership is proven would let anyone landing on a live id burn it.
        // Address before username, so a remote guesser never learns whether a username matched.
        // `Utility.Intern(address)` unmaps an IPv4-mapped IPv6 address first.
        if address.to_canonical() != entry.address
            || !entry.account.as_deref().is_some_and(|a| a.eq_ic(username))
        {
            return (AuthIdResult::Rejected, None);
        }
        // Theirs, so spend it. Expired counts as spent; it has done all it is ever going to do.
        self.map.shift_remove(&auth_id);
        let result = if now - entry.age > AUTH_ID_LIFETIME {
            AuthIdResult::Expired
        } else {
            AuthIdResult::Vouched
        };
        (result, Some(entry))
    }
}

fn window(w: &mut World) -> &mut AuthIdWindow {
    w.resources.get_mut::<AuthIdWindow>()
}

/// `EnsureAuthId`: one id per connection, by construction.
pub fn ensure_auth_id(
    w: &mut World,
    existing: i32,
    account: Option<String>,
    address: IpAddr,
    version: Option<ClientVersion>,
) -> i32 {
    if existing != 0 {
        return existing;
    }
    let now = w.now;
    window(w).register(
        now,
        AuthIdEntry {
            age: now,
            version,
            account,
            address,
        },
    )
}

/// The connection's id; also makes its world side if the transport has not announced the connection yet.
fn begin(w: &mut World, ns: &NetState) -> NetStateId {
    muo_server::net_glue::ensure_net_state(w, ns)
}

fn id_of(ns: &NetState) -> NetStateId {
    NetStateId(ns.id() as u32)
}

pub fn configure(w: &mut World) {
    use super::register_packet as reg;
    reg(w, 0x00, 104, false, true, create_character);
    reg(w, 0x5D, 73, false, true, play_character);
    reg(w, 0x80, 62, false, true, account_login);
    reg(w, 0x83, 39, false, true, delete_character);
    reg(w, 0x91, 65, false, true, game_login);
    reg(w, 0xA0, 3, false, true, play_server);
    reg(w, 0xBD, 0, false, false, client_version);
    reg(w, 0xE1, 0, false, false, client_type);
    reg(w, 0xEF, 21, false, true, login_server_seed);
    reg(w, 0xF8, 106, false, true, create_character);
}

pub fn initialize(w: &mut World) {
    // GameServer.GameServerLoginEvent -> AccountHandler.OnGameServerLogin, GatewayServer.ServerListEvent
    // -> ServerList.OnServerListEvent and CharacterCreatedEvent -> CharacterCreation.OnCharacterCreated are
    // subscribed by their own files.
    let _ = w;
}

/// `CreateCharacter` (0x00 and 0xF8).
pub fn create_character(w: &mut World, ns: &mut NetState, body: &[u8]) {
    let id = begin(w, ns);
    let mut r = SpanReader::new(body);
    let Some(fields) = read_create_character(&mut r, ns.new_character_creation()) else {
        ns.disconnect("Invalid character creation packet.");
        return;
    };
    let CreateFields {
        name,
        flags,
        prof,
        gender_race,
        stats,
        skills,
        hue,
        hair_val,
        hair_hue,
        hair_valf,
        hair_huef,
        city_index,
        shirt_hue,
        pants_hue,
    } = fields;

    /*
    Pre-7.0.0.0:
    0x00, 0x01 -> Human Male, Human Female
    0x02, 0x03 -> Elf Male, Elf Female

    Post-7.0.0.0:
    0x00, 0x01
    0x02, 0x03 -> Human Male, Human Female
    0x04, 0x05 -> Elf Male, Elf Female
    0x05, 0x06 -> Gargoyle Male, Gargoyle Female
    */
    let female = gender_race % 2 != 0;
    let race_id = if ns.stygian_abyss() {
        if gender_race < 4 {
            0
        } else {
            gender_race / 2 - 1
        }
    } else {
        gender_race / 2
    };
    let race = {
        let races = w.resources.get::<Races>();
        races
            .and_then(|r| {
                r.get(muo_server::race::RaceId(race_id))
                    .or_else(|| r.default_race())
            })
            .map(|r| r.id())
    };

    let cities = w
        .net_state(id)
        .map(|d| d.city_info.clone())
        .unwrap_or_default();
    let acct = outgoing::state_account(w, id);
    let (Some(a), false) = (
        acct,
        cities.is_empty() || city_index as usize >= cities.len(),
    ) else {
        ns.disconnect("Invalid city selected during character creation.");
        return;
    };

    // Check if anyone is using this account
    let len = accounting::account(w, a).map_or(0, |x| x.length());
    for i in 0..len {
        if let Some(check) = accounting::mobile_at(w, a, i)
            && w.mobile(check)
                .is_some_and(|m| m.map != Some(MapId::INTERNAL))
        {
            ns.log_info("Account in use");
            muo_server::packets::account::send_popup_message(w, id, PMMessage::CharInWorld);
            return;
        }
    }

    let flags = ClientFlags::from_bits_retain(flags as u32);
    ns.flags = flags;
    if let Some(d) = w.net_state_mut(id) {
        d.caps.flags = flags;
    }

    let mut args = CharacterCreatedEventArgs {
        state: id,
        account: a,
        mobile: None,
        name,
        female,
        hue,
        stats,
        city: cities[city_index as usize].clone(),
        skills,
        shirt_hue,
        pants_hue,
        hair_id: hair_val,
        hair_hue,
        beard_id: hair_valf,
        beard_hue: hair_huef,
        profession: prof,
        race,
    };

    // Straight to the transport: `BlockAllPackets` below would swallow it from the outbox.
    ns.send_client_version_request();
    ns.block_all_packets = true;

    character_created_event(w, &mut args);

    if let Some(m) = args.mobile {
        enter_world(w, ns, id, m);
    } else {
        ns.block_all_packets = false;
        ns.disconnect("Character creation blocked.");
    }
}

struct CreateFields {
    name: String,
    flags: i32,
    prof: i32,
    gender_race: u8,
    stats: [u8; 3],
    skills: Vec<(i32, u8)>,
    hue: i32,
    hair_val: i32,
    hair_hue: i32,
    hair_valf: i32,
    hair_huef: i32,
    city_index: u8,
    shirt_hue: i32,
    pants_hue: i32,
}

fn read_create_character(r: &mut SpanReader<'_>, new_creation: bool) -> Option<CreateFields> {
    r.seek(9, SeekOrigin::Current).ok()?;
    let name = r.read_latin1_safe(Some(30)).ok()?;
    r.seek(2, SeekOrigin::Current).ok()?;
    let flags = r.read_i32().ok()?;
    r.seek(8, SeekOrigin::Current).ok()?;
    let prof = i32::from(r.read_byte().ok()?);
    r.seek(15, SeekOrigin::Current).ok()?;
    let gender_race = r.read_byte().ok()?;
    let stats = [
        r.read_byte().ok()?,
        r.read_byte().ok()?,
        r.read_byte().ok()?,
    ];
    let mut skills = Vec::new();
    for _ in 0..if new_creation { 4 } else { 3 } {
        let n = r.read_byte().ok()?;
        let v = r.read_byte().ok()?;
        skills.push((i32::from(n), v));
    }
    let hue = i32::from(r.read_u16().ok()?);
    let hair_val = i32::from(r.read_i16().ok()?);
    let hair_hue = i32::from(r.read_i16().ok()?);
    let hair_valf = i32::from(r.read_i16().ok()?);
    let hair_huef = i32::from(r.read_i16().ok()?);
    r.read_byte().ok()?;
    let city_index = r.read_byte().ok()?;
    r.seek(8, SeekOrigin::Current).ok()?;
    let shirt_hue = i32::from(r.read_i16().ok()?);
    let pants_hue = i32::from(r.read_i16().ok()?);
    Some(CreateFields {
        name,
        flags,
        prof,
        gender_race,
        stats,
        skills,
        hue,
        hair_val,
        hair_hue,
        hair_valf,
        hair_huef,
        city_index,
        shirt_hue,
        pants_hue,
    })
}

/// `state.Mobile = m; m.NetState = state; new LoginTimer(state, m).Start();`
fn enter_world(w: &mut World, ns: &mut NetState, id: NetStateId, m: MobileId) {
    if let Some(d) = w.net_state_mut(id) {
        d.mobile = Some(m);
    }
    // what the transport checks for in-game packets (`NetState.Mobile != null`)
    ns.attachment = muo_net::net_state::Attachment::Mobile(m.0.value());
    muo_server::mobile::part1::props_b::set_net_state(w, m, Some(id));
    start_login_timer(w, id, m);
}

/// `DeleteCharacter` (0x83).
pub fn delete_character(w: &mut World, ns: &mut NetState, body: &[u8]) {
    let mut r = SpanReader::new(body);
    let index = r.seek(30, SeekOrigin::Current).and_then(|_| r.read_i32());
    match index {
        Ok(index) => {
            let id = begin(w, ns);
            handler::delete_request(w, id, index)
        }
        Err(_) => ns.disconnect("Invalid packet."),
    }
}

/// `ClientVersion` (0xBD).
pub fn client_version(w: &mut World, ns: &mut NetState, body: &[u8]) {
    let mut r = SpanReader::new(body);
    let Ok(text) = r.read_ascii(None) else { return };
    set_version(w, ns, ClientVersion::parse(&text));
}

/// `ClientType` (0xE1).
pub fn client_type(w: &mut World, ns: &mut NetState, body: &[u8]) {
    let mut r = SpanReader::new(body);
    let Ok(text) = r
        .read_u16()
        .and_then(|_| r.read_u16())
        .and_then(|_| r.read_ascii(None))
    else {
        return;
    };
    set_version(w, ns, ClientVersion::parse(&text));
}

fn set_version(w: &mut World, ns: &mut NetState, version: ClientVersion) {
    let id = begin(w, ns);
    ns.set_version(Some(version.clone()));
    let now = w.tick_count;
    if let Some(d) = w.net_state_mut(id) {
        d.caps.set_version(Some(version.clone()));
        // Record RTT if this is a response to our probe
        d.movement.record_rtt_measurement(now);
    }
    crate::login::client_verification::client_version_received(w, id, &version);
}

/// `PlayCharacter` (0x5D).
pub fn play_character(w: &mut World, ns: &mut NetState, body: &[u8]) {
    let id = begin(w, ns);
    let mut r = SpanReader::new(body);
    let read = (|| {
        r.seek(36, SeekOrigin::Current).ok()?; // 4 = 0xEDEDEDED, 30 = Name, 2 = unknown
        let flags = r.read_i32().ok()?;
        r.seek(24, SeekOrigin::Current).ok()?;
        let slot = r.read_i32().ok()?;
        Some((flags, slot))
    })();
    let Some((flags, char_slot)) = read else {
        ns.disconnect("Invalid packet.");
        return;
    };

    let a = outgoing::state_account(w, id);
    let len = a
        .and_then(|a| accounting::account(w, a))
        .map_or(0, |x| x.length());
    let (Some(a), true) = (a, char_slot >= 0 && char_slot < len) else {
        ns.disconnect("Invalid character slot selected.");
        return;
    };

    let m = accounting::mobile_at(w, a, char_slot);

    // Check if anyone is using this account
    for i in 0..len {
        if let Some(check) = accounting::mobile_at(w, a, i)
            && w.mobile(check)
                .is_some_and(|c| c.map != Some(MapId::INTERNAL))
            && Some(check) != m
        {
            ns.log_info("Account in use");
            muo_server::packets::account::send_popup_message(w, id, PMMessage::CharInWorld);
            return;
        }
    }

    let Some(m) = m else {
        ns.disconnect("Empty character slot selected.");
        return;
    };

    if let Some(other) = w.mobile(m).and_then(|mm| mm.net_state) {
        w.disconnect(other, "Character selected for a player already logged in.");
    }

    // Straight to the transport: `BlockAllPackets` below would swallow it from the outbox.
    ns.send_client_version_request();
    ns.block_all_packets = true;

    let flags = ClientFlags::from_bits_retain(flags as u32);
    ns.flags = flags;
    if let Some(d) = w.net_state_mut(id) {
        d.caps.flags = flags;
    }

    enter_world(w, ns, id, m);
}

/// `LoginTimer`: every 64 ms until the client version is known, then `DoLogin`.
fn start_login_timer(w: &mut World, id: NetStateId, m: MobileId) {
    timer::start(
        w,
        TimeSpan::from_millis(64),
        TimeSpan::from_millis(64),
        0,
        move |w| {
            let Some(me) = timer::current(w) else { return };
            let account = w.net_state(id).map(|d| d.account.clone());
            match account {
                // the connection is gone
                None => {}
                Some(None) => w.disconnect(id, "Account was deleted during the login process."),
                Some(Some(_)) if w.mobile(m).is_none() => {
                    w.disconnect(id, "Player was deleted during the login process.");
                }
                Some(Some(_)) => {
                    if w.net_state(id).is_some_and(|d| d.caps.version.is_some()) {
                        // BlockAllPackets = false: the transport's flag, released through the outbox.
                        w.net_out
                            .push(muo_server::net_state::NetOut::UnblockPackets(id));
                        // the mirror of `CannotSendPackets` says "blocked" until the next pump
                        if let Some(d) = w.net_state_mut(id) {
                            d.cannot_send = false;
                        }
                        outgoing::do_login(w, id, m);
                    } else {
                        return; // Waiting to receive the client version before we continue the login process
                    }
                }
            }
            timer::stop(w, me);
        },
    );
}

/// `GameLogin` (0x91).
pub fn game_login(w: &mut World, ns: &mut NetState, body: &[u8]) {
    let id = begin(w, ns);
    if ns.sent_first_packet {
        ns.disconnect("Duplicate game login packet received.");
        return;
    }
    ns.sent_first_packet = true;

    let mut r = SpanReader::new(body);
    let Ok(auth_id) = r.read_i32() else {
        ns.disconnect("Invalid packet.");
        return;
    };

    if ns.auth_id != 0 && auth_id != ns.auth_id || ns.auth_id == 0 && auth_id != ns.seed {
        ns.log_info("Invalid client detected, disconnecting...");
        ns.disconnect("Invalid auth id in game login packet.");
        return;
    }

    let (Ok(username), Ok(password)) = (r.read_latin1_safe(Some(30)), r.read_latin1_safe(Some(30)))
    else {
        ns.disconnect("Invalid packet.");
        return;
    };

    let now = w.now;
    let (result, entry) = window(w).consume(now, auth_id, &username, ns.address);
    let Some(entry) = entry.filter(|_| result != AuthIdResult::Rejected) else {
        ns.log_info("Invalid client detected, disconnecting...");
        ns.disconnect("Unable to find auth id.");
        return;
    };

    ns.set_version(entry.version.clone());
    ns.seeded = true;
    if let Some(d) = w.net_state_mut(id) {
        d.caps.set_version(entry.version.clone());
    }

    // Expired carries a usable entry; only the password verify skip is withheld.
    let mut e = GameLoginEventArgs::new(id, username, password, result == AuthIdResult::Vouched);
    game_server_login_event(w, &mut e);
    // The handler sets the account on the world's side of the connection.
    ns.account = w.net_state(id).and_then(|d| d.account.clone());

    if e.accepted {
        if let Some(d) = w.net_state_mut(id) {
            d.city_info = e.city_info;
        }
        // Comment out these lines to turn off huffman compression
        ns.compression_enabled = true;

        outgoing::send_supported_feature(w, id);
        outgoing::send_character_list(w, id);
    } else {
        ns.disconnect("Login rejected by GameLogin packet handler.");
    }
}

/// `PlayServer` (0xA0).
pub fn play_server(w: &mut World, ns: &mut NetState, body: &[u8]) {
    let id = begin(w, ns);
    // A server is picked once per connection. Picking again hands back an id this connection may already have
    // spent on a game login, which the client could never redeem.
    if ns.auth_id != 0 {
        ns.disconnect("Duplicate play server packet sent.");
        return;
    }
    let mut r = SpanReader::new(body);
    let Ok(index) = r.read_i16() else {
        ns.disconnect("Invalid packet.");
        return;
    };
    let info = w
        .net_state(id)
        .map(|d| d.server_info.clone())
        .unwrap_or_default();
    let has_account = outgoing::state_account(w, id).is_some();
    if info.is_empty() || !has_account || index < 0 || index as usize >= info.len() {
        ns.disconnect("Invalid server selected.");
        return;
    }
    let si = &info[index as usize];
    let version = ns.version().cloned();
    let acct = w.net_state(id).and_then(|d| d.account.clone());
    ns.auth_id = ensure_auth_id(w, ns.auth_id, acct, ns.address, version);
    ns.sent_first_packet = false;
    // Straight to the transport, not the outbox: the transport queues its own graceful close as soon as this
    // handler returns, and what the outbox holds would reach it too late.
    ns.send(&muo_server::packets::account::create_play_server_ack(
        si, ns.auth_id,
    ));
}

/// `LoginServerSeed` (0xEF).
pub fn login_server_seed(_w: &mut World, ns: &mut NetState, body: &[u8]) {
    // Seeding happens once per connection. A second one restarts a handshake this connection already completed,
    // which no real client does.
    if ns.seeded {
        ns.disconnect("Duplicate login server seed packet sent.");
        return;
    }
    let mut r = SpanReader::new(body);
    let Ok(seed) = r.read_i32() else {
        ns.disconnect("Invalid packet.");
        return;
    };
    ns.seed = seed;
    ns.seeded = true;

    if ns.seed == 0 {
        ns.log_info("Invalid client detected, disconnecting");
        ns.disconnect("Invalid client detected");
        return;
    }

    let (Ok(maj), Ok(min), Ok(rev), Ok(pat)) =
        (r.read_i32(), r.read_i32(), r.read_i32(), r.read_i32())
    else {
        ns.disconnect("Invalid packet.");
        return;
    };
    ns.set_version(Some(ClientVersion::new(
        maj,
        min,
        rev,
        pat,
        muo_net::client_version::ClientType::Classic,
    )));
}

/// `AccountLogin` (0x80).
pub fn account_login(w: &mut World, ns: &mut NetState, body: &[u8]) {
    let id = begin(w, ns);
    if ns.sent_first_packet {
        ns.disconnect("Duplicate account login packet sent.");
        return;
    }
    ns.sent_first_packet = true;

    let mut r = SpanReader::new(body);
    let (Ok(username), Ok(password)) = (r.read_latin1_safe(Some(30)), r.read_latin1_safe(Some(30)))
    else {
        ns.disconnect("Invalid packet.");
        return;
    };

    let mut e = AccountLoginEventArgs::new(id.0 as usize, username, password);
    events::invoke_account_login(w, &mut e);
    // The handler set the account on the world's side of the connection.
    ns.account = w.net_state(id).and_then(|d| d.account.clone());

    // The password check moved off the loop; whoever took it replies when the verdict lands.
    if e.deferred {
        return;
    }
    complete_account_login(w, ns, e.accepted, e.reject_reason);
}

/// `CompleteAccountLogin`: replies to an account login.
pub fn complete_account_login(w: &mut World, ns: &mut NetState, accepted: bool, reject_reason: u8) {
    let id = begin(w, ns);
    if accepted {
        let mut e =
            ServerListEventArgs::new(id, ns.local_end_point(), ns.address, ns.account.clone());
        server_list_event(w, &mut e);
        if e.rejected {
            clear_account(w, ns);
            account_login_reply_rej(w, ns, ALRReason::BadComm);
        } else {
            if let Some(d) = w.net_state_mut(id) {
                d.server_info = e.servers;
            }
            outgoing::send_account_login_ack(w, id);
        }
    } else {
        clear_account(w, ns);
        let reason = match reject_reason {
            1 => ALRReason::InUse,
            2 => ALRReason::Blocked,
            3 => ALRReason::BadPass,
            254 => ALRReason::Idle,
            255 => ALRReason::BadComm,
            _ => ALRReason::Invalid,
        };
        account_login_reply_rej(w, ns, reason);
    }
}

/// `CompleteAccountLogin` for a login whose verdict landed after the packet handler returned: the transport is
/// reached through the world's queue (`NetOut`) instead of the connection.
pub fn complete_account_login_deferred(
    w: &mut World,
    id: NetStateId,
    accepted: bool,
    reject_reason: u8,
) {
    use muo_server::net_state::NetOut;
    let Some(d) = w.net_state(id) else { return };
    let (local, address, account) = (d.local_end_point, d.address, d.account.clone());
    let Some(address) = address else { return };
    let reject = |w: &mut World, reason: ALRReason| {
        if let Some(d) = w.net_state_mut(id) {
            d.account = None;
        }
        w.net_out.push(NetOut::SetAccount(id, None));
        w.net_out.push(NetOut::Send(id, vec![0x82, reason as u8]));
        w.disconnect(id, &format!("Account login rejected due to {reason:?}"));
    };
    if accepted {
        w.net_out.push(NetOut::SetAccount(id, account.clone()));
        let mut e = ServerListEventArgs::new(id, local, address, account);
        server_list_event(w, &mut e);
        if e.rejected {
            reject(w, ALRReason::BadComm);
        } else {
            if let Some(d) = w.net_state_mut(id) {
                d.server_info = e.servers;
            }
            outgoing::send_account_login_ack(w, id);
        }
    } else {
        let reason = match reject_reason {
            1 => ALRReason::InUse,
            2 => ALRReason::Blocked,
            3 => ALRReason::BadPass,
            254 => ALRReason::Idle,
            255 => ALRReason::BadComm,
            _ => ALRReason::Invalid,
        };
        reject(w, reason);
    }
}

fn clear_account(w: &mut World, ns: &mut NetState) {
    ns.account = None;
    if let Some(d) = w.net_state_mut(id_of(ns)) {
        d.account = None;
    }
}

fn account_login_reply_rej(w: &mut World, ns: &mut NetState, reason: ALRReason) {
    let _ = w;
    // Straight to the transport for the same reason as the play server ack.
    ns.send(&[0x82, reason as u8]);
    ns.disconnect(&format!("Account login rejected due to {reason:?}"));
}

#[cfg(test)]
mod auth_id_tests {
    //! Auth id tests (the window and EnsureAuthId; the two
    //! GameLogin password tests are in accounting/handler.rs).
    use super::*;

    fn x() -> IpAddr {
        "203.0.113.10".parse().unwrap()
    }
    fn y() -> IpAddr {
        "203.0.113.11".parse().unwrap()
    }
    fn now() -> DateTime {
        DateTime::from_unix_seconds(1_800_000_000).unwrap()
    }
    fn version() -> Option<ClientVersion> {
        Some(ClientVersion::new(
            7,
            0,
            0,
            0,
            muo_net::client_version::ClientType::Classic,
        ))
    }

    fn register(win: &mut AuthIdWindow, at: DateTime, account: &str, address: IpAddr) -> i32 {
        win.register(
            at,
            AuthIdEntry {
                age: at,
                version: version(),
                account: Some(account.to_string()),
                address,
            },
        )
    }

    #[test]
    fn vouches_for_the_account_and_address_it_was_issued_to() {
        let mut win = AuthIdWindow::default();
        let id = register(&mut win, now(), "authid-match-user", x());
        let (result, entry) = win.consume(now(), id, "authid-match-user", x());
        assert_eq!(result, AuthIdResult::Vouched);
        assert_eq!(entry.unwrap().account.as_deref(), Some("authid-match-user"));
    }

    #[test]
    fn rejects_a_different_account() {
        let mut win = AuthIdWindow::default();
        let id = register(&mut win, now(), "authid-owner-user", x());
        assert_eq!(
            win.consume(now(), id, "authid-other-user", x()).0,
            AuthIdResult::Rejected
        );
    }

    #[test]
    fn rejects_a_different_address() {
        let mut win = AuthIdWindow::default();
        let id = register(&mut win, now(), "authid-switch-user", x());
        assert_eq!(
            win.consume(now(), id, "authid-switch-user", y()).0,
            AuthIdResult::Rejected
        );
    }

    #[test]
    fn matches_the_username_case_insensitively() {
        let mut win = AuthIdWindow::default();
        let id = register(&mut win, now(), "AuthId-Case-User", x());
        assert_eq!(
            win.consume(now(), id, "authid-case-user", x()).0,
            AuthIdResult::Vouched
        );
    }

    #[test]
    fn matches_an_ipv4_mapped_ipv6_address() {
        let mut win = AuthIdWindow::default();
        let id = register(&mut win, now(), "authid-mapped-user", x());
        let mapped: IpAddr = "::ffff:203.0.113.10".parse().unwrap();
        assert_eq!(
            win.consume(now(), id, "authid-mapped-user", mapped).0,
            AuthIdResult::Vouched
        );
    }

    #[test]
    fn rejects_an_unknown_auth_id() {
        let mut win = AuthIdWindow::default();
        let id = register(&mut win, now(), "authid-unknown-user", x());
        assert_eq!(
            win.consume(now(), id.wrapping_add(1), "authid-unknown-user", x())
                .0,
            AuthIdResult::Rejected
        );
    }

    #[test]
    fn is_single_use_after_a_success() {
        let mut win = AuthIdWindow::default();
        let id = register(&mut win, now(), "authid-once-user", x());
        assert_eq!(
            win.consume(now(), id, "authid-once-user", x()).0,
            AuthIdResult::Vouched
        );
        assert_eq!(
            win.consume(now(), id, "authid-once-user", x()).0,
            AuthIdResult::Rejected
        );
    }

    // A rejected attempt must not consume the id, or anyone landing on a live one could burn it.
    #[test]
    fn survives_an_attempt_from_the_wrong_address() {
        let mut win = AuthIdWindow::default();
        let id = register(&mut win, now(), "authid-not-burned-address-user", x());
        assert_eq!(
            win.consume(now(), id, "authid-not-burned-address-user", y())
                .0,
            AuthIdResult::Rejected
        );
        assert_eq!(win.count(), 1);
        assert_eq!(
            win.consume(now(), id, "authid-not-burned-address-user", x())
                .0,
            AuthIdResult::Vouched
        );
    }

    #[test]
    fn survives_an_attempt_for_the_wrong_account() {
        let mut win = AuthIdWindow::default();
        let id = register(&mut win, now(), "authid-not-burned-account-user", x());
        assert_eq!(
            win.consume(now(), id, "not-the-owner", x()).0,
            AuthIdResult::Rejected
        );
        assert_eq!(win.count(), 1);
        assert_eq!(
            win.consume(now(), id, "authid-not-burned-account-user", x())
                .0,
            AuthIdResult::Vouched
        );
    }

    #[test]
    fn a_rejected_attempt_yields_no_entry() {
        let mut win = AuthIdWindow::default();
        let id = register(&mut win, now(), "authid-no-leak-user", x());
        assert!(win.consume(now(), id, "not-the-owner", x()).1.is_none());
    }

    #[test]
    fn an_expired_id_is_spent_by_its_owner() {
        let mut win = AuthIdWindow::default();
        let id = register(&mut win, now(), "authid-expired-spent-user", x());
        let later = now() + TimeSpan::from_mins(30);
        assert_eq!(
            win.consume(later, id, "authid-expired-spent-user", x()).0,
            AuthIdResult::Expired
        );
        assert_eq!(win.count(), 0);
    }

    // Expiry is not a lockout: the game login falls back to the password verify.
    #[test]
    fn expires_into_a_password_verify_rather_than_a_rejection() {
        let mut win = AuthIdWindow::default();
        let id = register(&mut win, now(), "authid-expired-user", x());
        let later = now() + TimeSpan::from_mins(30);
        let (result, entry) = win.consume(later, id, "authid-expired-user", x());
        assert_eq!(result, AuthIdResult::Expired);
        // still carries the client version the game login needs
        assert_eq!(entry.unwrap().version, version());
    }

    #[test]
    fn an_expired_id_from_another_address_is_still_rejected() {
        let mut win = AuthIdWindow::default();
        let id = register(&mut win, now(), "authid-expired-elsewhere-user", x());
        let later = now() + TimeSpan::from_mins(30);
        assert_eq!(
            win.consume(later, id, "authid-expired-elsewhere-user", y())
                .0,
            AuthIdResult::Rejected
        );
    }

    #[test]
    fn re_selecting_returns_the_same_id_and_adds_nothing_to_the_window() {
        let mut w = World::new();
        w.resources.get_mut::<AuthIdWindow>().clear();
        let first = ensure_auth_id(
            &mut w,
            0,
            Some("authid-reselect-user".into()),
            x(),
            version(),
        );
        for _ in 0..10 {
            assert_eq!(
                ensure_auth_id(
                    &mut w,
                    first,
                    Some("authid-reselect-user".into()),
                    x(),
                    version()
                ),
                first
            );
        }
        assert_eq!(w.resources.get_mut::<AuthIdWindow>().count(), 1);
        let t = w.now;
        assert_eq!(
            w.resources
                .get_mut::<AuthIdWindow>()
                .consume(t, first, "authid-reselect-user", x())
                .0,
            AuthIdResult::Vouched
        );
    }

    #[test]
    fn abandoned_ids_are_swept_when_new_ones_are_issued() {
        let mut win = AuthIdWindow::default();
        for _ in 0..128 {
            register(&mut win, now(), "authid-abandoned-user", x());
        }
        assert_eq!(win.count(), 128);

        let later = now() + TimeSpan::from_mins(30);
        let live = register(&mut win, later, "authid-live-user", x());

        assert_eq!(win.count(), 1);
        assert_eq!(
            win.consume(later, live, "authid-live-user", x()).0,
            AuthIdResult::Vouched
        );
    }

    // A login rush is not a backlog: none may be discarded to hold the window at some arbitrary size.
    #[test]
    fn a_login_rush_does_not_evict_anyones_auth_id() {
        let mut win = AuthIdWindow::default();
        let ids: Vec<i32> = (0..800)
            .map(|_| register(&mut win, now(), "authid-rush-user", x()))
            .collect();
        assert_eq!(win.count(), ids.len());

        // every id issued during the rush is still redeemable, including the first one
        for id in ids {
            assert_eq!(
                win.consume(now(), id, "authid-rush-user", x()).0,
                AuthIdResult::Vouched
            );
        }
    }
}
