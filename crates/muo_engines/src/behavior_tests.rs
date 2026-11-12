//! Packet tests of the engines (Chat, Help, ML Quests, Party, Veteran Rewards): the expected bytes are written out
//! from each packet's wire layout and compared with what the server produces.

use muo_primitives::Serial;
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::net_state::{NetCaps, NetOut, NetStateData, NetStateId};

fn sent(f: impl FnOnce(&mut World, NetStateId)) -> Vec<u8> {
    let mut w = World::new();
    let ns = NetStateId(1);
    w.net_states.insert(
        ns,
        NetStateData {
            caps: NetCaps::default(),
            ..Default::default()
        },
    );
    f(&mut w, ns);
    match w.net_out.pop() {
        Some(NetOut::Send(_, b)) => b,
        _ => Vec::new(),
    }
}

/// `Stream.WriteBigUniNull`.
fn big_uni_null(s: &str) -> Vec<u8> {
    let mut v: Vec<u8> = s.encode_utf16().flat_map(u16::to_be_bytes).collect();
    v.extend([0, 0]);
    v
}

fn chat_expected(lang: Option<&str>, number: i32, p1: &str, p2: &str) -> Vec<u8> {
    let mut v = vec![0xB2, 0, 0];
    v.extend(((number - 20) as u16).to_be_bytes());
    let mut l = [0u8; 4];
    if let Some(lang) = lang {
        l[..lang.len()].copy_from_slice(lang.as_bytes());
    }
    v.extend(l);
    v.extend(big_uni_null(p1));
    v.extend(big_uni_null(p2));
    let n = v.len() as u16;
    v[1..3].copy_from_slice(&n.to_be_bytes());
    v
}

#[test]
fn test_send_chat_message() {
    // [InlineData(null, 100, "some param", "some other param")]
    assert_eq!(
        crate::chat::chat_message_packet("", 100, "some param", "some other param"),
        chat_expected(None, 100, "some param", "some other param")
    );
    // [InlineData("ENU", 200, "a third param", "another param")]
    assert_eq!(
        crate::chat::chat_message_packet("ENU", 200, "a third param", "another param"),
        chat_expected(Some("ENU"), 200, "a third param", "another param")
    );
}

#[test]
fn test_display_help_topic() {
    // [InlineData(HelpTopic.Healing, false)], [InlineData(HelpTopic.EmptyingBowl, true)]
    let healing = crate::help::help_topic::HEALING;
    let emptying_bowl = crate::help::help_topic::EMPTYING_BOWL;
    assert_eq!(emptying_bowl, 71);
    for (topic, display) in [(healing, false), (emptying_bowl, true)] {
        let mut want = vec![0xBF, 0, 11, 0, 0x17, 1];
        want.extend(topic.to_be_bytes());
        want.push(display as u8);
        let got = sent(|w, ns| crate::help::send_display_help_topic(w, ns, topic, display));
        assert_eq!(got, want);
    }
}

#[test]
fn test_race_changer() {
    // [InlineData(true, 1)], [InlineData(false, 2)]: the legacy packet writes RaceID + 1
    for (female, race_id) in [(true, 1), (false, 2)] {
        let got =
            sent(|w, ns| crate::ml_quests::race_change::send_race_changer(w, ns, female, race_id));
        assert_eq!(
            got,
            [0xBF, 0, 7, 0, 0x2A, female as u8, (race_id + 1) as u8]
        );
    }
}

#[test]
fn test_close_race_changer() {
    let got = sent(crate::ml_quests::race_change::send_close_race_changer);
    assert_eq!(got, [0xBF, 0, 7, 0, 0x2A, 0, 0xFF]);
}

fn mob(n: u32) -> MobileId {
    MobileId(Serial::new(n))
}

#[test]
fn test_party_remove_member_without_party_sends_empty_list() {
    // PartyEmptyList(m): 0xBF, length 7 (EnsureCapacity only; the legacy packet is variable length, so the real
    // length is the written size), 0x0006, 0x02, 0, serial.
    let got = crate::party::remove_member_packet(Serial::new(0x1024), &[]);
    assert_eq!(got, [0xBF, 0, 11, 0, 6, 2, 0, 0, 0, 0x10, 0x24]);
}

#[test]
fn test_party_remove_member() {
    let (leader, member) = (mob(0x1), mob(0x2));
    let got = crate::party::remove_member_packet(member.0, &[leader, member]);
    assert_eq!(
        got,
        [0xBF, 0, 19, 0, 6, 2, 2, 0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0, 2]
    );
}

#[test]
fn test_party_member_list() {
    let (leader, member) = (mob(0x1), mob(0x2));
    let got = crate::party::member_list_packet(&[leader, member]);
    assert_eq!(got, [0xBF, 0, 15, 0, 6, 1, 2, 0, 0, 0, 1, 0, 0, 0, 2]);
}

#[test]
fn test_party_text_message() {
    let text = "[Party] Stuff Happens";
    for (to_all, kind) in [(true, 4u8), (false, 3u8)] {
        let mut want = vec![0xBF, 0, 0, 0, 6, kind, 0, 0, 0x10, 0x24];
        want.extend(big_uni_null(text));
        let n = want.len() as u16;
        want[1..3].copy_from_slice(&n.to_be_bytes());
        assert_eq!(
            crate::party::text_message_packet(Serial::new(0x1024), text, to_all),
            want
        );
    }
}

#[test]
fn test_party_invitation() {
    assert_eq!(
        crate::party::invitation_packet(Serial::new(0x1024)),
        [0xBF, 0, 10, 0, 6, 7, 0, 0, 0x10, 0x24]
    );
}

#[test]
fn test_send_statue_animation() {
    // [InlineData(0x1024u, 1, 100, 200)]
    let p = crate::rewards::statue::statue_animation_packet(Serial::new(0x1024), 1, 100, 200);
    assert_eq!(
        p,
        [
            0xBF, 0, 17, 0, 0x19, 5, 0, 0, 0x10, 0x24, 0, 0xFF, 1, 0, 100, 0, 200
        ]
    );
}
