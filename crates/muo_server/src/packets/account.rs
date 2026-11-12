//! Account packets: login, character list, features.
//!
//! The builders take plain values ([`AccountChars`], `&ExpansionInfo`, `&[CityInfo]`, `&[ServerInfo]`, `account_limit`,
//! a `map_size` callback); the `send_*` wrappers that read the world fill them in.

use muo_data::expansion::{CharacterListFlags, ExpansionInfo, FeatureFlags};
use muo_io::text::helpers::default_if_blank;
use muo_net::server_info::ServerInfo;

use super::Pk;
use crate::city_info::CityInfo;
use crate::ids::MobileId;
use crate::map::MapId;
use crate::net_state::{NetCaps, NetStateId};
use crate::world::World;

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
#[repr(u8)]
pub enum ALRReason {
    #[default]
    Invalid = 0,
    InUse = 1,
    Blocked = 2,
    BadPass = 3,
    Idle = 254,
    BadComm = 255,
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
#[repr(u8)]
pub enum PMMessage {
    #[default]
    None = 0,
    CharNoExist = 1,
    CharExists = 2,
    CharInWorld = 5,
    LoginSyncError = 6,
    IdleWarning = 7,
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
#[repr(u8)]
pub enum DeleteResultType {
    #[default]
    PasswordInvalid,
    CharNotExist,
    CharBeingPlayed,
    CharTooYoung,
    CharQueued,
    BadRequest,
}

/// A character slot of an `IAccount` that holds a mobile: its `RawName` (null stays `None`).
#[derive(Clone, Debug, Default)]
pub struct CharSlot {
    pub raw_name: Option<String>,
}

/// What the packets read of an `IAccount`: `a[i]` (null = `None`; past `Length` also null) and `Limit`.
// shortcut: the caller copies the account's fields out instead of passing an `IAccount`; the same bytes result.
#[derive(Clone, Debug, Default)]
pub struct AccountChars {
    pub slots: Vec<Option<CharSlot>>,
    pub limit: i32,
}

impl AccountChars {
    /// `a.Length`.
    pub fn length(&self) -> usize {
        self.slots.len()
    }

    fn get(&self, i: usize) -> Option<&CharSlot> {
        self.slots.get(i).and_then(|s| s.as_ref())
    }

    /// The last occupied slot, or -1.
    fn high_slot(&self) -> i32 {
        self.slots
            .iter()
            .rposition(|s| s.is_some())
            .map_or(-1, |i| i as i32)
    }
}

/// One 60 byte slot: `WriteLatin1(name, 30)` and a blank password, or 60 zeros.
fn write_slot(p: &mut Pk, slot: Option<&CharSlot>) {
    match slot {
        None => {
            p.zeros(60);
        }
        Some(m) => {
            let name = default_if_blank(m.raw_name.as_deref().map(str::trim), "-no name-");
            let _ = p.write_latin1_fixed(name, 30);
            p.zeros(30); // password (empty)
        }
    }
}

/// Packet 0x81, up to 425 bytes. Currently unused.
pub fn create_change_character(a: &AccountChars) -> Vec<u8> {
    let length = 5 + a.length() * 60;
    let mut p = Pk::new(length);
    p.u8(0x81).u16(length as u16).u16(0); // Count & Placeholder
    let mut count = 0;
    for i in 0..a.length() {
        if a.get(i).is_some() {
            count += 1;
        }
        write_slot(&mut p, a.get(i));
    }
    let mut v = p.finish();
    v[3] = count as u8;
    v
}

/// `SendChangeCharacter`: nothing for a null account; pass `None`.
pub fn send_change_character(w: &mut World, ns: NetStateId, a: Option<&AccountChars>) {
    let Some(a) = a else { return };
    w.send(ns, &create_change_character(a));
}

/// Packet 0xBD: asks the client for its version.
pub fn send_client_version_request(w: &mut World, ns: NetStateId) {
    w.send(ns, &[0xBD, 0x00, 0x03]);
}

/// Packet 0x85: the result of a deletion request.
pub fn send_character_delete_result(w: &mut World, ns: NetStateId, res: DeleteResultType) {
    w.send(ns, &[0x85, res as u8]);
}

/// Packet 0x53: a popup with a predetermined message.
pub fn send_popup_message(w: &mut World, ns: NetStateId, msg: PMMessage) {
    w.send(ns, &[0x53, msg as u8]);
}

/// Packet 0xB9, 3 or 5 bytes. `account_limit` is `ns.Account.Limit`.
pub fn create_supported_feature(
    caps: &NetCaps,
    core_expansion: &ExpansionInfo,
    account_limit: i32,
    force_old_animations: bool,
) -> Vec<u8> {
    let mut flags = core_expansion.supported_features;
    if account_limit >= 6 {
        flags |= FeatureFlags::LIVE_ACCOUNT;
        if account_limit > 6 {
            flags |= FeatureFlags::SEVENTH_CHARACTER_SLOT;
        } else {
            flags |= FeatureFlags::SIXTH_CHARACTER_SLOT;
        }
    }
    if force_old_animations {
        flags &= !FeatureFlags::LBR;
    }
    let extended = caps.extended_supported_features();
    let mut p = Pk::new(if extended { 5 } else { 3 });
    p.u8(0xB9);
    if extended {
        p.u32(flags.bits());
    } else {
        p.u16(flags.bits() as u16);
    }
    p.finish()
}

pub fn send_supported_feature(
    w: &mut World,
    ns: NetStateId,
    core_expansion: &ExpansionInfo,
    account_limit: i32,
    force_old_animations: bool,
) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let Some(d) = w.net_state(ns) else { return };
    let bytes =
        create_supported_feature(&d.caps, core_expansion, account_limit, force_old_animations);
    w.send(ns, &bytes);
}

/// `SendSupportedFeature()` with the core expansion's features, `Account.Limit` (`Core.SA ? 7 : Core.AOS ? 6 : 5`)
/// and the `expansion.forceOldAnimations` setting.
pub fn send_supported_feature_of_core(w: &mut World, ns: NetStateId) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let Some(info) = w.core_expansion_info().cloned() else {
        return;
    };
    let expansion = w.expansion();
    let limit = if expansion >= muo_data::expansion::Expansion::SA {
        7
    } else if expansion >= muo_data::expansion::Expansion::AOS {
        6
    } else {
        5
    };
    let force_old = w.config.get_setting("expansion.forceOldAnimations", false);
    send_supported_feature(w, ns, &info, limit, force_old);
}

/// Packet 0x1B, 37 bytes. `map_size(map)` is `(map.Width, map.Height)` of the map picked (`m.Map`, or
/// `m.LogoutMap` when that is null or Internal); `None` is a null map and the callback answers Felucca's size.
// shortcut: the caller supplies the map sizes as a callback; same bytes as reading `Map.Width/Height` here.
pub fn create_login_confirmation(
    w: &World,
    m: MobileId,
    map_size: impl Fn(Option<MapId>) -> (i32, i32),
) -> Option<Vec<u8>> {
    let mid = m;
    let m = w.mobile(m)?;
    let mut p = Pk::new(37);
    p.u8(0x1B)
        .serial(m.serial)
        .i32(0)
        .i16(crate::mobile::part1::props_b::body(w, mid).body_id() as i16)
        .i16(m.location.x as i16)
        .i16(m.location.y as i16)
        .i16(m.location.z as i16)
        .u8(m.direction.bits())
        .u8(0)
        .i32(-1)
        .i32(0);
    let mut map = m.map;
    if map.is_none() || map == Some(MapId::INTERNAL) {
        map = m.logout_map;
    }
    let (width, height) = map_size(map);
    p.i16(width as i16).i16(height as i16);
    let rest = 37 - p.bytes_written();
    p.zeros(rest); // remaining is zero
    Some(p.finish())
}

pub fn send_login_confirmation(
    w: &mut World,
    ns: NetStateId,
    m: MobileId,
    map_size: impl Fn(Option<MapId>) -> (i32, i32),
) {
    if w.cannot_send_packets(ns) {
        return;
    }
    if let Some(bytes) = create_login_confirmation(w, m, map_size) {
        w.send(ns, &bytes);
    }
}

/// Packet 0x55.
pub fn send_login_complete(w: &mut World, ns: NetStateId) {
    w.send(ns, &[0x55]);
}

/// Packet 0x86, up to 424 bytes.
pub fn create_character_list_update(a: &AccountChars) -> Vec<u8> {
    let count = (a.high_slot() + 1).max(a.limit).max(5) as usize;
    let length = 4 + count * 60;
    let mut p = Pk::new(length);
    p.u8(0x86).u16(length as u16).u8(count as u8);
    for i in 0..count {
        write_slot(&mut p, a.get(i));
    }
    p.finish()
}

/// `SendCharacterListUpdate`: nothing for a null account; pass `None`.
pub fn send_character_list_update(w: &mut World, ns: NetStateId, a: Option<&AccountChars>) {
    let Some(a) = a else { return };
    w.send(ns, &create_character_list_update(a));
}

/// Packet 0xA9, 1410 or more bytes: the characters and the starting cities. `NewCharacterList` picks the 70130 form.
pub fn create_character_list(
    caps: &NetCaps,
    acct: &AccountChars,
    city_info: &[CityInfo],
    core_expansion: &ExpansionInfo,
) -> Vec<u8> {
    let client70130 = caps.new_character_list();
    let text_length: usize = if client70130 { 32 } else { 31 };

    // Supported values are 1, 5, 6, or 7
    let mut count = (acct.high_slot() + 1).max(acct.limit);
    if count != 1 && count < 5 {
        count = 5;
    }
    let count = count as usize;

    let length = (if client70130 {
        11 + (text_length * 2 + 25) * city_info.len()
    } else {
        9 + (text_length * 2 + 1) * city_info.len()
    }) + count * 60;
    let mut p = Pk::new(length);
    p.u8(0xA9).u16(length as u16).u8(count as u8);
    for i in 0..count {
        write_slot(&mut p, acct.get(i));
    }

    p.u8(city_info.len() as u8);
    for (i, ci) in city_info.iter().enumerate() {
        p.u8(i as u8);
        let _ = p.write_latin1_fixed(&ci.city, text_length);
        let _ = p.write_latin1_fixed(&ci.building, text_length);
        if client70130 {
            p.i32(ci.x()).i32(ci.y()).i32(ci.z());
            p.i32(ci.map.0 as i32);
            p.i32(ci.description);
            p.i32(0);
        }
    }

    let mut flags = core_expansion.character_list_flags;
    if count > 6 {
        flags |=
            CharacterListFlags::SEVENTH_CHARACTER_SLOT | CharacterListFlags::SIXTH_CHARACTER_SLOT;
    } else if count == 6 {
        flags |= CharacterListFlags::SIXTH_CHARACTER_SLOT;
    } else if acct.limit == 1 {
        // ANDs the two flags (OR was meant): this adds nothing. Kept for client compatibility.
        flags |= CharacterListFlags::SLOT_LIMIT & CharacterListFlags::ONE_CHARACTER_SLOT;
    }
    p.i32(flags.bits() as i32);
    if client70130 {
        p.i16(-1);
    }
    p.finish()
}

/// `SendCharacterList`: `acct` is `ns.Account` (nothing when null, pass `None`), `city_info` is `ns.CityInfo`.
pub fn send_character_list(
    w: &mut World,
    ns: NetStateId,
    acct: Option<&AccountChars>,
    city_info: &[CityInfo],
    core_expansion: &ExpansionInfo,
) {
    let Some(acct) = acct else { return };
    let Some(d) = w.net_state(ns) else { return };
    let bytes = create_character_list(&d.caps, acct, city_info, core_expansion);
    w.send(ns, &bytes);
}

/// Packet 0xA8: the server list, 6 + 40 bytes per server.
pub fn create_account_login_ack(info: &[ServerInfo]) -> Vec<u8> {
    let length = 6 + 40 * info.len();
    let mut p = Pk::new(length);
    p.u8(0xA8)
        .u16(length as u16)
        .u8(0x5D)
        .u16(info.len() as u16);
    for (i, si) in info.iter().enumerate() {
        p.u16(i as u16);
        let _ = p.write_latin1_fixed(&si.name, 32);
        p.u8(si.full_percent as u8).i8(si.time_zone as i8);
        // UO only supports IPv4
        p.u32(si.raw_address());
    }
    p.finish()
}

/// `SendAccountLoginAck`: `info` is `ns.ServerInfo`.
pub fn send_account_login_ack(w: &mut World, ns: NetStateId, info: &[ServerInfo]) {
    if w.cannot_send_packets(ns) {
        return;
    }
    w.send(ns, &create_account_login_ack(info));
}

/// Packet 0x8C, 11 bytes: acknowledges the server the client picked.
pub fn create_play_server_ack(si: &ServerInfo, auth_id: i32) -> Vec<u8> {
    let mut p = Pk::new(11);
    p.u8(0x8C);
    let _ = p.write_u32_le(si.raw_address());
    p.i16(si.address().port() as i16).i32(auth_id);
    p.finish()
}

#[cfg(test)]
pub fn send_account_login_rejected(w: &mut World, ns: NetStateId, reason: ALRReason) {
    w.send(ns, &[0x82, reason as u8]);
}

#[cfg(test)]
pub fn send_play_server_ack(w: &mut World, ns: NetStateId, si: &ServerInfo, auth_id: i32) {
    if w.cannot_send_packets(ns) {
        return;
    }
    w.send(ns, &create_play_server_ack(si, auth_id));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mobile::MOBILE_BASE;
    use crate::net_state::{NetOut, NetStateData};
    use muo_net::protocol_changes::ProtocolChanges;
    use muo_primitives::{Body, Direction, Point3D};

    fn sent(caps: NetCaps, f: impl FnOnce(&mut World, NetStateId)) -> Vec<u8> {
        let mut w = World::new();
        let ns = NetStateId(1);
        w.net_states.insert(
            ns,
            NetStateData {
                caps,
                ..Default::default()
            },
        );
        f(&mut w, ns);
        match w.net_out.pop() {
            Some(NetOut::Send(_, b)) => b,
            _ => Vec::new(),
        }
    }

    fn caps(pc: ProtocolChanges) -> NetCaps {
        NetCaps {
            protocol_changes: pc,
            ..Default::default()
        }
    }

    fn name30(s: &str) -> Vec<u8> {
        let mut v = s.as_bytes().to_vec();
        v.resize(30, 0);
        v.resize(60, 0);
        v
    }

    fn slot(n: Option<&str>) -> Option<CharSlot> {
        Some(CharSlot {
            raw_name: n.map(str::to_string),
        })
    }

    fn acct(first_at: usize, name: Option<&str>) -> AccountChars {
        let mut slots = vec![None; 5];
        slots[first_at] = slot(name);
        AccountChars { slots, limit: 5 }
    }

    fn expansion() -> ExpansionInfo {
        ExpansionInfo {
            supported_features: FeatureFlags::LBR | FeatureFlags::T2A,
            character_list_flags: CharacterListFlags::CONTEXT_MENUS,
            ..Default::default()
        }
    }

    fn server() -> ServerInfo {
        ServerInfo::new("Test Server", 0, 0, "127.0.0.1:2593".parse().unwrap()).unwrap()
    }

    #[test]
    fn change_character() {
        let a = AccountChars {
            slots: vec![slot(Some("Test Mobile")), None, slot(None)],
            limit: 5,
        };
        let b = sent(NetCaps::default(), |w, ns| {
            send_change_character(w, ns, Some(&a))
        });
        let mut want = vec![0x81, 0, 185, 2, 0];
        want.extend(name30("Test Mobile"));
        want.extend([0; 60]);
        want.extend(name30("-no name-"));
        assert_eq!(b, want);
    }

    #[test]
    fn simple_packets() {
        let c = NetCaps::default();
        assert_eq!(sent(c.clone(), send_client_version_request), [0xBD, 0, 3]);
        assert_eq!(
            sent(c.clone(), |w, ns| send_character_delete_result(
                w,
                ns,
                DeleteResultType::BadRequest
            )),
            [0x85, 5]
        );
        assert_eq!(
            sent(c.clone(), |w, ns| send_popup_message(
                w,
                ns,
                PMMessage::LoginSyncError
            )),
            [0x53, 6]
        );
        assert_eq!(sent(c.clone(), send_login_complete), [0x55]);
        assert_eq!(
            sent(c, |w, ns| send_account_login_rejected(
                w,
                ns,
                ALRReason::BadComm
            )),
            [0x82, 255]
        );
    }

    #[test]
    fn supported_features() {
        let e = expansion();
        let b = sent(NetCaps::default(), |w, ns| {
            send_supported_feature(w, ns, &e, 5, false)
        });
        let f = e.supported_features.bits();
        assert_eq!(b, [0xB9, (f >> 8) as u8, f as u8]);
        let b = sent(
            caps(ProtocolChanges::EXTENDED_SUPPORTED_FEATURES),
            |w, ns| send_supported_feature(w, ns, &e, 7, true),
        );
        let flags = (e.supported_features
            | FeatureFlags::LIVE_ACCOUNT
            | FeatureFlags::SEVENTH_CHARACTER_SLOT)
            & !FeatureFlags::LBR;
        let mut want = vec![0xB9];
        want.extend(flags.bits().to_be_bytes());
        assert_eq!(b, want);
        let b = sent(NetCaps::default(), |w, ns| {
            send_supported_feature(w, ns, &e, 6, false)
        });
        let flags =
            e.supported_features | FeatureFlags::LIVE_ACCOUNT | FeatureFlags::SIXTH_CHARACTER_SLOT;
        assert_eq!(b, [0xB9, (flags.bits() >> 8) as u8, flags.bits() as u8]);
    }

    #[test]
    fn login_confirmation() {
        let mut w = World::new();
        let id = w.new_mobile(&MOBILE_BASE);
        let serial = {
            let m = w.mobile_mut(id).unwrap();
            m.body = Body(0x100);
            m.location = Point3D::new(100, 10, -10);
            m.direction = Direction::DOWN;
            m.map = Some(MapId::INTERNAL);
            m.logout_map = Some(MapId(0));
            m.serial
        };
        let b = create_login_confirmation(&w, id, |m| {
            assert_eq!(m, Some(MapId(0)));
            (7168, 4096)
        })
        .unwrap();
        let mut want = vec![0x1B];
        want.extend(serial.value().to_be_bytes());
        want.extend([0, 0, 0, 0, 0x01, 0x00, 0, 100, 0, 10, 0xFF, 0xF6, 3, 0]);
        want.extend([0xFF; 4]);
        want.extend([0; 4]);
        want.extend([0x1C, 0x00, 0x10, 0x00]);
        want.resize(37, 0);
        assert_eq!(b, want);
    }

    #[test]
    fn character_list_update() {
        let a = acct(1, Some("Test Mobile"));
        let b = sent(NetCaps::default(), |w, ns| {
            send_character_list_update(w, ns, Some(&a))
        });
        let mut want = vec![0x86, 0x01, 0x30, 5];
        want.extend([0; 60]);
        want.extend(name30("Test Mobile"));
        want.extend([0; 180]);
        assert_eq!(b, want);
    }

    fn test_city() -> [CityInfo; 1] {
        [CityInfo::new(
            "Test City",
            "Test Building",
            50,
            100,
            10,
            -10,
            MapId(1),
        )]
    }

    #[test]
    fn character_list_70130() {
        let a = acct(1, Some("Test Mobile"));
        let info = test_city();
        let b = sent(caps(ProtocolChanges::NEW_CHARACTER_LIST), |w, ns| {
            send_character_list(w, ns, Some(&a), &info, &expansion())
        });
        let len = 11 + 89 + 300;
        let mut want = vec![0xA9, (len >> 8) as u8, len as u8, 5];
        want.extend([0; 60]);
        want.extend(name30("Test Mobile"));
        want.extend([0; 180]);
        want.extend([1, 0]);
        let mut city = b"Test City".to_vec();
        city.resize(32, 0);
        want.extend(city);
        let mut building = b"Test Building".to_vec();
        building.resize(32, 0);
        want.extend(building);
        for v in [100i32, 10, -10, 1, 50, 0] {
            want.extend(v.to_be_bytes());
        }
        want.extend((CharacterListFlags::CONTEXT_MENUS.bits() as i32).to_be_bytes());
        want.extend([0xFF, 0xFF]);
        assert_eq!(b.len(), len);
        assert_eq!(b, want);
    }

    #[test]
    fn character_list_old() {
        let a = acct(1, Some("Test Mobile"));
        let info = test_city();
        let b = sent(NetCaps::default(), |w, ns| {
            send_character_list(w, ns, Some(&a), &info, &expansion())
        });
        let len = 9 + 63 + 300;
        let mut want = vec![0xA9, (len >> 8) as u8, len as u8, 5];
        want.extend([0; 60]);
        want.extend(name30("Test Mobile"));
        want.extend([0; 180]);
        want.extend([1, 0]);
        let mut city = b"Test City".to_vec();
        city.resize(31, 0);
        want.extend(city);
        let mut building = b"Test Building".to_vec();
        building.resize(31, 0);
        want.extend(building);
        want.extend((CharacterListFlags::CONTEXT_MENUS.bits() as i32).to_be_bytes());
        assert_eq!(b, want);
    }

    #[test]
    fn account_login_ack_and_play_server_ack() {
        let si = server();
        let b = sent(NetCaps::default(), |w, ns| {
            send_account_login_ack(w, ns, &[server()])
        });
        let mut want = vec![0xA8, 0, 46, 0x5D, 0, 1, 0, 0];
        let mut n = b"Test Server".to_vec();
        n.resize(32, 0);
        want.extend(n);
        want.extend([0, 0]); // full percent, time zone
        want.extend(si.raw_address().to_be_bytes());
        assert_eq!(b, want);

        let b = sent(NetCaps::default(), |w, ns| {
            send_play_server_ack(w, ns, &si, 0x123456)
        });
        assert_eq!(b, [0x8C, 127, 0, 0, 1, 0x0A, 0x21, 0, 0x12, 0x34, 0x56]);
    }
}
