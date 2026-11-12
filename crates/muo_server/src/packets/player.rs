//! Player packets (`LRReason` is in `crate::lr_reason`).
//!
//! The skill packets take [`SkillPacketEntry`] values and `send_server_change` the map size; the callers read
//! them from the world.

use muo_primitives::{DateTime, Point3D, Serial};

use super::Pk;
use crate::ids::MobileId;
use crate::lr_reason::LRReason;
use crate::net_state::NetStateId;
use crate::region::MusicName;
use crate::world::World;

pub const DRAG_EFFECT_PACKET_LENGTH: usize = 26;

/// What the skill packets read of one `Skill`.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct SkillPacketEntry {
    /// `Info.SkillID`.
    pub skill_id: i32,
    /// `NonRacialValue`.
    pub non_racial_value: f64,
    /// `BaseFixedPoint`.
    pub base_fixed_point: i32,
    /// `(byte)Lock`.
    pub lock: u8,
    /// `CapFixedPoint`.
    pub cap_fixed_point: i32,
}

impl SkillPacketEntry {
    /// `Math.Clamp((int)(v * 10), 0, 0xFFFF)`.
    fn unsigned_value(&self) -> u16 {
        ((self.non_racial_value * 10.0) as i32).clamp(0, 0xFFFF) as u16
    }
}

pub fn create_stat_lock_info(w: &World, m: MobileId) -> Option<Vec<u8>> {
    let m = w.mobile(m)?;
    let mut p = Pk::new(12);
    p.u8(0xBF).u16(12).i16(0x19).u8(2).serial(m.serial).u8(0);
    let lock_bits = ((m.str_lock as i32) << 4) | ((m.dex_lock as i32) << 2) | (m.int_lock as i32);
    p.u8(lock_bits as u8);
    Some(p.finish())
}

pub fn send_stat_lock_info(w: &mut World, ns: NetStateId, m: MobileId) {
    if w.cannot_send_packets(ns) {
        return;
    }
    if let Some(bytes) = create_stat_lock_info(w, m) {
        w.send(ns, &bytes);
    }
}

pub fn send_change_update_range(w: &mut World, ns: NetStateId, range: u8) {
    w.send(ns, &[0xC8, range]);
}

pub fn send_death_status(w: &mut World, ns: NetStateId) {
    w.send(ns, &[0x2C, 2]);
}

/// The lengths in the packet count UTF-16 units.
fn utf16_len(s: &str) -> usize {
    s.encode_utf16().count()
}

pub fn create_display_profile(m: Serial, header: &str, body: &str, footer: &str) -> Vec<u8> {
    let length = 12 + utf16_len(header) + utf16_len(footer) * 2 + utf16_len(body) * 2;
    let mut p = Pk::new(length);
    p.u8(0xB8).u16(length as u16).serial(m);
    let _ = p.write_latin1_null(header);
    let _ = p.write_big_uni_null(footer);
    let _ = p.write_big_uni_null(body);
    p.finish()
}

/// A null string is `""` (`header ??= ""`); pass `None` for it.
pub fn send_display_profile(
    w: &mut World,
    ns: NetStateId,
    m: Serial,
    header: Option<&str>,
    body: Option<&str>,
    footer: Option<&str>,
) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let bytes = create_display_profile(
        m,
        header.unwrap_or(""),
        body.unwrap_or(""),
        footer.unwrap_or(""),
    );
    w.send(ns, &bytes);
}

pub fn send_lift_reject(w: &mut World, ns: NetStateId, reason: LRReason) {
    w.send(ns, &[0x27, reason as u8]);
}

pub fn send_logout_ack(w: &mut World, ns: NetStateId) {
    w.send(ns, &[0xD1, 0x01]);
}

pub fn send_weather(w: &mut World, ns: NetStateId, kind: u8, density: u8, temp: u8) {
    w.send(ns, &[0x65, kind, density, temp]);
}

/// `SendServerChange(p, map)`: `map_width`/`map_height` are `map.Width`/`map.Height`.
pub fn create_server_change(p3: Point3D, map_width: i32, map_height: i32) -> Vec<u8> {
    let mut p = Pk::new(16);
    p.u8(0x76)
        .i16(p3.x as i16)
        .i16(p3.y as i16)
        .i16(p3.z as i16)
        .u8(0)
        .i16(0)
        .i16(0)
        .i16(map_width as i16)
        .i16(map_height as i16);
    p.finish()
}

pub fn send_server_change(
    w: &mut World,
    ns: NetStateId,
    p3: Point3D,
    map_width: i32,
    map_height: i32,
) {
    if w.cannot_send_packets(ns) {
        return;
    }
    w.send(ns, &create_server_change(p3, map_width, map_height));
}

pub fn create_skills_update(skills: &[SkillPacketEntry]) -> Vec<u8> {
    let length = 6 + 9 * skills.len();
    let mut p = Pk::new(length);
    p.u8(0x3A).u16(length as u16).u8(0x02); // type: absolute, capped
    for s in skills {
        p.u16((s.skill_id + 1) as u16)
            .u16(s.unsigned_value())
            .u16(s.base_fixed_point as u16)
            .u8(s.lock)
            .u16(s.cap_fixed_point as u16);
    }
    p.i16(0); // terminate
    p.finish()
}

pub fn send_skills_update(w: &mut World, ns: NetStateId, skills: &[SkillPacketEntry]) {
    if w.cannot_send_packets(ns) {
        return;
    }
    w.send(ns, &create_skills_update(skills));
}

pub fn send_sequence(w: &mut World, ns: NetStateId, sequence: u8) {
    w.send(ns, &[0x7B, sequence]);
}

pub fn create_skill_change(skill: &SkillPacketEntry) -> Vec<u8> {
    let mut p = Pk::new(13);
    p.u8(0x3A).u16(13);
    p.u8(0xDF) // type: delta, capped
        .u16(skill.skill_id as u16)
        .u16(skill.unsigned_value())
        .u16(skill.base_fixed_point as u16)
        .u8(skill.lock)
        .u16(skill.cap_fixed_point as u16);
    p.finish()
}

pub fn send_skill_change(w: &mut World, ns: NetStateId, skill: &SkillPacketEntry) {
    if w.cannot_send_packets(ns) {
        return;
    }
    w.send(ns, &create_skill_change(skill));
}

pub fn create_launch_browser(uri: &str) -> Vec<u8> {
    let length = 4 + utf16_len(uri);
    let mut p = Pk::new(length);
    p.u8(0xA5).u16(length as u16);
    let _ = p.write_ascii_null(uri);
    p.finish()
}

pub fn send_launch_browser(w: &mut World, ns: NetStateId, uri: &str) {
    if w.cannot_send_packets(ns) {
        return;
    }
    w.send(ns, &create_launch_browser(uri));
}

/// `CreateDragEffect` into a fresh 26 byte buffer.
pub fn create_drag_effect(
    src_serial: Serial,
    src_location: Point3D,
    trg_serial: Serial,
    trg_location: Point3D,
    item_id: i32,
    hue: i32,
    amount: i32,
) -> Vec<u8> {
    let mut p = Pk::new(DRAG_EFFECT_PACKET_LENGTH);
    p.u8(0x23)
        .i16(item_id as i16)
        .u8(0)
        .i16(hue as i16)
        .i16(amount as i16)
        .serial(src_serial)
        .i16(src_location.x as i16)
        .i16(src_location.y as i16)
        .i8(src_location.z as i8)
        .serial(trg_serial)
        .i16(trg_location.x as i16)
        .i16(trg_location.y as i16)
        .i8(trg_location.z as i8);
    p.finish()
}

#[allow(clippy::too_many_arguments)]
pub fn send_drag_effect(
    w: &mut World,
    ns: NetStateId,
    src_serial: Serial,
    src_location: Point3D,
    trg_serial: Serial,
    trg_location: Point3D,
    item_id: i32,
    hue: i32,
    amount: i32,
) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let bytes = create_drag_effect(
        src_serial,
        src_location,
        trg_serial,
        trg_location,
        item_id,
        hue,
        amount,
    );
    w.send(ns, &bytes);
}

pub fn send_season_change(w: &mut World, ns: NetStateId, season: u8, play_sound: bool) {
    w.send(ns, &[0xBC, season, play_sound as u8]);
}

pub fn create_display_paperdoll(m: Serial, title: &str, warmode: bool, can_lift: bool) -> Vec<u8> {
    let mut flags = 0x00u8;
    if warmode {
        flags |= 0x01;
    }
    if can_lift {
        flags |= 0x02;
    }
    let mut p = Pk::new(66);
    p.u8(0x88).serial(m);
    let _ = p.write_latin1_fixed(title, 60);
    p.u8(flags);
    p.finish()
}

pub fn send_display_paperdoll(
    w: &mut World,
    ns: NetStateId,
    m: Serial,
    title: &str,
    warmode: bool,
    can_lift: bool,
) {
    if w.cannot_send_packets(ns) {
        return;
    }
    w.send(ns, &create_display_paperdoll(m, title, warmode, can_lift));
}

pub fn send_play_music(w: &mut World, ns: NetStateId, music: MusicName) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let mut p = Pk::new(3);
    p.u8(0x6D).i16(music as i32 as i16);
    w.send(ns, &p.finish());
}

pub fn send_stop_music(w: &mut World, ns: NetStateId) {
    w.send(ns, &[0x6D, 0x1F, 0xFF]);
}

pub fn create_scroll_message(kind: i32, tip: i32, text: &str) -> Vec<u8> {
    let len16 = utf16_len(text);
    let length = 10 + len16;
    let mut p = Pk::new(length);
    p.u8(0xA6)
        .u16(length as u16)
        .u8(kind as u8)
        .i32(tip)
        .u16(len16 as u16);
    let _ = p.write_latin1(text);
    p.finish()
}

/// A null `text` is `""`.
pub fn send_scroll_message(w: &mut World, ns: NetStateId, kind: i32, tip: i32, text: Option<&str>) {
    if w.cannot_send_packets(ns) {
        return;
    }
    w.send(ns, &create_scroll_message(kind, tip, text.unwrap_or("")));
}

/// `SendCurrentTime()`: `Core.Now`.
pub fn send_current_time(w: &mut World, ns: NetStateId) {
    let now = w.now;
    send_current_time_at(w, ns, now);
}

pub fn send_current_time_at(w: &mut World, ns: NetStateId, date: DateTime) {
    w.send(
        ns,
        &[
            0x5B,
            date.hour() as u8,
            date.minute() as u8,
            date.second() as u8,
        ],
    );
}

pub fn send_pathfind_message(w: &mut World, ns: NetStateId, p3: Point3D) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let mut p = Pk::new(7);
    p.u8(0x38)
        .i16(p3.x as i16)
        .i16(p3.y as i16)
        .i16(p3.z as i16);
    w.send(ns, &p.finish());
}

pub fn send_ping_ack(w: &mut World, ns: NetStateId, ping: u8) {
    w.send(ns, &[0x73, ping]);
}

pub fn send_display_hue_picker(
    w: &mut World,
    ns: NetStateId,
    hue_picker_serial: Serial,
    hue_picker_item_id: i32,
) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let mut p = Pk::new(9);
    p.u8(0x95)
        .serial(hue_picker_serial)
        .i16(0)
        .i16(hue_picker_item_id as i16);
    w.send(ns, &p.finish());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mobile::{MOBILE_BASE, StatLockType};
    use crate::net_state::{NetOut, NetStateData};

    fn sent(f: impl FnOnce(&mut World, NetStateId)) -> Vec<u8> {
        let mut w = World::new();
        let ns = NetStateId(1);
        w.net_states.insert(ns, NetStateData::default());
        f(&mut w, ns);
        match w.net_out.pop() {
            Some(NetOut::Send(_, b)) => b,
            _ => Vec::new(),
        }
    }

    fn s(v: u32) -> Serial {
        Serial::new(v)
    }

    #[test]
    fn stat_lock_info() {
        let mut w = World::new();
        let ns = NetStateId(1);
        w.net_states.insert(ns, NetStateData::default());
        let id = w.new_mobile(&MOBILE_BASE);
        let serial = {
            let m = w.mobile_mut(id).unwrap();
            m.str_lock = StatLockType::Down;
            m.dex_lock = StatLockType::Locked;
            m.int_lock = StatLockType::Up;
            m.serial
        };
        send_stat_lock_info(&mut w, ns, id);
        let Some(NetOut::Send(_, b)) = w.net_out.pop() else {
            panic!("nothing sent")
        };
        let mut want = vec![0xBF, 0, 12, 0, 0x19, 2];
        want.extend(serial.value().to_be_bytes());
        want.extend([0, (1 << 4) | (2 << 2)]);
        assert_eq!(b, want);
    }

    #[test]
    fn fixed_packets() {
        assert_eq!(
            sent(|w, ns| send_change_update_range(w, ns, 18)),
            [0xC8, 18]
        );
        assert_eq!(sent(send_death_status), [0x2C, 2]);
        assert_eq!(
            sent(|w, ns| send_lift_reject(w, ns, LRReason::TryToSteal)),
            [0x27, 3]
        );
        assert_eq!(sent(send_logout_ack), [0xD1, 1]);
        assert_eq!(sent(|w, ns| send_weather(w, ns, 1, 2, 3)), [0x65, 1, 2, 3]);
        assert_eq!(sent(|w, ns| send_sequence(w, ns, 7)), [0x7B, 7]);
        assert_eq!(
            sent(|w, ns| send_season_change(w, ns, 2, true)),
            [0xBC, 2, 1]
        );
        assert_eq!(sent(send_stop_music), [0x6D, 0x1F, 0xFF]);
        assert_eq!(sent(|w, ns| send_ping_ack(w, ns, 9)), [0x73, 9]);
        assert_eq!(
            sent(|w, ns| send_play_music(w, ns, MusicName::Create1)),
            [0x6D, 0, 1]
        );
        let date = DateTime::from_ymd_hms(2020, 1, 1, 14, 10, 5).unwrap();
        assert_eq!(
            sent(|w, ns| send_current_time_at(w, ns, date)),
            [0x5B, 14, 10, 5]
        );
        assert_eq!(
            sent(|w, ns| send_play_music(w, ns, MusicName::ValoriaShips)),
            [0x6D, 0, 66]
        );
        assert_eq!(
            sent(|w, ns| send_play_music(w, ns, MusicName::Invalid)),
            [0x6D, 0xFF, 0xFF]
        );
    }

    #[test]
    fn display_profile() {
        let b = sent(|w, ns| send_display_profile(w, ns, s(0x1024), Some("Hd"), Some("Bo"), None));
        // 12 + 2 + 0 + 4 = 18
        let want = [
            0xB8, 0, 18, 0, 0, 0x10, 0x24, b'H', b'd', 0, // header, latin1 null
            0, 0, // footer: empty big-endian unicode null
            0, b'B', 0, b'o', 0, 0, // body
        ];
        assert_eq!(b, want);
    }

    #[test]
    fn display_profile_nulls() {
        let b = sent(|w, ns| send_display_profile(w, ns, s(0x1000), None, None, None));
        assert_eq!(b, [0xB8, 0, 12, 0, 0, 0x10, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn server_change_and_pathfind() {
        let p3 = Point3D::new(100, 200, -5);
        let b = sent(|w, ns| send_server_change(w, ns, p3, 7168, 4096));
        assert_eq!(
            b,
            [
                0x76, 0, 100, 0, 200, 0xFF, 0xFB, 0, 0, 0, 0, 0, 0x1C, 0, 0x10, 0
            ]
        );
        let b = sent(|w, ns| send_pathfind_message(w, ns, p3));
        assert_eq!(b, [0x38, 0, 100, 0, 200, 0xFF, 0xFB]);
    }

    #[test]
    fn skills() {
        let e = SkillPacketEntry {
            skill_id: 3,
            non_racial_value: 50.5,
            base_fixed_point: 505,
            lock: 1,
            cap_fixed_point: 1000,
        };
        let b = sent(|w, ns| send_skills_update(w, ns, &[e]));
        assert_eq!(
            b,
            [
                0x3A, 0, 15, 2, 0, 4, 0x01, 0xF9, 0x01, 0xF9, 1, 0x03, 0xE8, 0, 0
            ]
        );
        let b = sent(|w, ns| send_skill_change(w, ns, &e));
        assert_eq!(
            b,
            [
                0x3A, 0, 13, 0xDF, 0, 3, 0x01, 0xF9, 0x01, 0xF9, 1, 0x03, 0xE8
            ]
        );
        let big = SkillPacketEntry {
            non_racial_value: 9999.0,
            ..e
        };
        assert_eq!(big.unsigned_value(), 0xFFFF);
        let neg = SkillPacketEntry {
            non_racial_value: -3.0,
            ..e
        };
        assert_eq!(neg.unsigned_value(), 0);
    }

    #[test]
    fn launch_browser_drag_paperdoll_scroll_hue() {
        let b = sent(|w, ns| send_launch_browser(w, ns, "http://x"));
        let mut want = vec![0xA5, 0, 12];
        want.extend(b"http://x\0");
        assert_eq!(b, want);

        let b = sent(|w, ns| {
            send_drag_effect(
                w,
                ns,
                s(0x40000001),
                Point3D::new(1, 2, 3),
                s(0x2),
                Point3D::new(4, 5, -6),
                0xEED,
                0x44,
                7,
            )
        });
        assert_eq!(b.len(), DRAG_EFFECT_PACKET_LENGTH);
        assert_eq!(
            b,
            [
                0x23, 0x0E, 0xED, 0, 0, 0x44, 0, 7, 0x40, 0, 0, 1, 0, 1, 0, 2, 3, 0, 0, 0, 2, 0, 4,
                0, 5, 0xFA
            ]
        );

        let b = sent(|w, ns| send_display_paperdoll(w, ns, s(0x1), "Lord", true, true));
        assert_eq!(b.len(), 66);
        assert_eq!(&b[..9], [0x88, 0, 0, 0, 1, b'L', b'o', b'r', b'd']);
        assert_eq!(b[65], 3);

        let b = sent(|w, ns| send_scroll_message(w, ns, 2, 0x10, Some("hi")));
        assert_eq!(b, [0xA6, 0, 12, 2, 0, 0, 0, 0x10, 0, 2, b'h', b'i']);

        let b = sent(|w, ns| send_display_hue_picker(w, ns, s(0x1024), 0xEED));
        assert_eq!(b, [0x95, 0, 0, 0x10, 0x24, 0, 0, 0x0E, 0xED]);
    }

    // ---- SkillPacketsTests: the packet against the legacy `SkillChange`/`SkillUpdate` ----

    use crate::skills::{self, SkillName, SkillTable};

    fn skilled() -> (World, NetStateId, MobileId) {
        let mut w = World::new();
        let info = |i: i32| muo_data::skill_info::SkillInfo {
            skill_id: i,
            name: format!("S{i}"),
            title: String::new(),
            str_scale: 0.0,
            dex_scale: 0.0,
            int_scale: 0.0,
            stat_total: 0.0,
            str_gain: 0.0,
            dex_gain: 0.0,
            int_gain: 0.0,
            gain_factor: 0.0,
            profession_skill_name: String::new(),
            primary_stat: muo_data::skill_info::Stat::Str,
            secondary_stat: muo_data::skill_info::Stat::Dex,
        };
        w.resources.get_mut::<SkillTable>().info = (0..58).map(info).collect();
        let ns = NetStateId(1);
        w.net_states.insert(ns, NetStateData::default());
        let m = w.new_mobile(&MOBILE_BASE);
        crate::mobile::part2::stats::default_mobile_init(&mut w, m);
        (w, ns, m)
    }

    fn sent_bytes(w: &mut World) -> Vec<u8> {
        match w.net_out.pop() {
            Some(NetOut::Send(_, b)) => b,
            _ => Vec::new(),
        }
    }

    #[test]
    fn test_skill_change() {
        for (skill, base_fixed, cap_fixed) in [
            (SkillName::Alchemy, 0, 1),
            (SkillName::Archery, 10, 1000),
            (SkillName::Begging, 100000, 1000),
        ] {
            let (mut w, ns, m) = skilled();
            skills::set_base_fixed(&mut w, m, skill, base_fixed);
            skills::set_cap_fixed(&mut w, m, skill, cap_fixed);

            // SkillChange: 0x3A, delta capped, the unadjusted skill id
            let v = skills::non_racial_value(&mut w, m, skill);
            let uv = ((v * 10.0) as i32).clamp(0, 0xFFFF);
            let mut expected = vec![0x3A, 0, 13, 0xDF];
            expected.extend_from_slice(&(skill as u16).to_be_bytes());
            expected.extend_from_slice(&(uv as u16).to_be_bytes());
            expected
                .extend_from_slice(&(skills::base_fixed(&mut w, m, skill) as u16).to_be_bytes());
            expected.push(skills::lock(&mut w, m, skill) as u8);
            expected.extend_from_slice(&(skills::cap_fixed(&mut w, m, skill) as u16).to_be_bytes());

            let entry = skills::packet_entry(&mut w, m, skill);
            send_skill_change(&mut w, ns, &entry);

            assert_eq!(sent_bytes(&mut w), expected, "{skill:?}");
        }
    }

    #[test]
    fn test_skills_update() {
        let (mut w, ns, m) = skilled();
        skills::set_base_fixed(&mut w, m, SkillName::Anatomy, 1000); // SkillsInfo.RandomSkill()

        // SkillUpdate: 0x3A, absolute capped, every skill, a terminating short
        let count = 58i32;
        let mut expected = vec![0x3A, 0, 0, 0x02];
        let mut entries = Vec::new();
        for i in 0..count {
            let skill = SkillName::from_i32(i).unwrap();
            let v = skills::non_racial_value(&mut w, m, skill);
            let uv = ((v * 10.0) as i32).clamp(0, 0xFFFF);
            expected.extend_from_slice(&((i + 1) as u16).to_be_bytes());
            expected.extend_from_slice(&(uv as u16).to_be_bytes());
            expected
                .extend_from_slice(&(skills::base_fixed(&mut w, m, skill) as u16).to_be_bytes());
            expected.push(skills::lock(&mut w, m, skill) as u8);
            expected.extend_from_slice(&(skills::cap_fixed(&mut w, m, skill) as u16).to_be_bytes());
            entries.push(skills::packet_entry(&mut w, m, skill));
        }
        expected.extend_from_slice(&0i16.to_be_bytes());
        let len = expected.len() as u16;
        expected[1..3].copy_from_slice(&len.to_be_bytes());

        send_skills_update(&mut w, ns, &entries);

        assert_eq!(sent_bytes(&mut w), expected);
    }
}
