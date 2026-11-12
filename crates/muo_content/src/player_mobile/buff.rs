//! Engines/BuffIcons (`BuffInfo`, `BuffIconPackets`) and the buff members of `PlayerMobile`
//! (`SendAddBuffPacket`, `ResendBuffs`, `AddBuff`, `RemoveBuff`).

use indexmap::IndexMap;
use muo_io::text::TextDefinition;
use muo_primitives::{DateTime, TimeSpan};
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::packets::Pk;
use muo_server::timer::{self, TimerHandle};

pub use super::buff_icon::BuffIcon;
use super::{data, data_mut};

/// `BuffInfo`.
#[derive(Clone, Debug)]
pub struct BuffInfo {
    pub id: BuffIcon,
    pub title_cliloc: i32,
    pub secondary_cliloc: i32,
    pub start_time: DateTime,
    pub duration: TimeSpan,
    pub retain_through_death: bool,
    pub args: Option<TextDefinition>,
    timer: TimerHandle,
}

impl BuffInfo {
    /// `new BuffInfo(iconID, titleCliloc, duration, args, retainThroughDeath)`: the secondary cliloc is the title + 1.
    pub fn new(id: BuffIcon, title_cliloc: i32) -> BuffInfo {
        BuffInfo::with_secondary(id, title_cliloc, title_cliloc + 1)
    }

    pub fn with_secondary(id: BuffIcon, title_cliloc: i32, secondary_cliloc: i32) -> BuffInfo {
        BuffInfo {
            id,
            title_cliloc,
            secondary_cliloc,
            start_time: DateTime::MIN,
            duration: TimeSpan::ZERO,
            retain_through_death: false,
            args: None,
            timer: TimerHandle::default(),
        }
    }

    pub fn duration(mut self, duration: TimeSpan) -> BuffInfo {
        self.duration = duration;
        self
    }

    pub fn args(mut self, args: TextDefinition) -> BuffInfo {
        self.args = Some(args);
        self
    }

    pub fn retain_through_death(mut self) -> BuffInfo {
        self.retain_through_death = true;
        self
    }
}

/// `BuffInfo.Enabled`: `buffIcons.enable`, default `Core.ML`.
#[derive(Default)]
pub struct BuffConfig {
    pub enabled: bool,
}

/// `BuffInfo.Configure`.
pub fn configure(w: &mut World) {
    let default = crate::core::ml(w);
    let enabled = w
        .config
        .get_or_update_setting("buffIcons.enable", default)
        .unwrap_or(default);
    w.resources.get_mut::<BuffConfig>().enabled = enabled;
}

fn enabled(w: &World) -> bool {
    w.resources.get::<BuffConfig>().is_some_and(|c| c.enabled)
}

fn client_has_buff_icons(w: &World, id: MobileId) -> Option<muo_server::net_state::NetStateId> {
    let ns = w.mobile(id)?.net_state?;
    w.net_state(ns)?.caps.buff_icon().then_some(ns)
}

/// `BuffIconPackets.SendAddBuffPacket`.
fn send_add_packet(w: &mut World, id: MobileId, info: &BuffInfo, seconds: i64) {
    let Some(ns) = w.mobile(id).and_then(|m| m.net_state) else {
        return;
    };
    if w.cannot_send_packets(ns) {
        return;
    }
    let serial = w.mobile(id).map_or_else(Default::default, |m| m.serial);
    let args = info.args.as_ref().map(ToString::to_string);
    let length = args.as_ref().map_or(46, |a| a.chars().count() * 2 + 52);
    let mut p = Pk::new(length);
    p.u8(0xDF).u16(0).serial(serial);
    p.i16(info.id.0).i16(1).i32(0);
    p.i16(info.id.0).i16(1).i32(0);
    p.i16(seconds as i16).zeros(3);
    p.i32(info.title_cliloc).i32(info.secondary_cliloc);
    match args {
        Some(a) => {
            p.i32(0).i16(1).u16(0);
            p.bytes(&[b'\t', 0]);
            p.write_little_uni_null(&a).ok();
            p.i16(1).u16(0);
        }
        None => {
            p.zeros(10);
        }
    }
    let bytes = p.finish_with_length();
    w.send(ns, &bytes);
}

/// `BuffIconPackets.SendRemoveBuffPacket`.
fn send_remove_packet(w: &mut World, id: MobileId, icon: BuffIcon) {
    let Some(ns) = w.mobile(id).and_then(|m| m.net_state) else {
        return;
    };
    if w.cannot_send_packets(ns) {
        return;
    }
    let serial = w.mobile(id).map_or_else(Default::default, |m| m.serial);
    let mut p = Pk::new(15);
    p.u8(0xDF).u16(15).serial(serial).i16(icon.0).i16(0).i32(0);
    let bytes = p.finish();
    w.send(ns, &bytes);
}

/// `SendAddBuffPacket(BuffInfo)`
pub fn send_add_buff_packet(w: &mut World, id: MobileId, info: &BuffInfo) {
    if client_has_buff_icons(w, id).is_none() {
        return;
    }
    let remaining = info.duration - (w.now - info.start_time);
    if remaining <= TimeSpan::ZERO {
        send_add_packet(w, id, info, 0);
        return;
    }
    let seconds = remaining.total_seconds().round_ties_even();
    let offset = remaining.total_milliseconds() - seconds * 1000.0;
    if offset > 0.0 {
        let info = info.clone();
        timer::start(
            w,
            TimeSpan::from_millis(offset as i64),
            TimeSpan::ZERO,
            1,
            move |w| {
                // They are still online, we still have the buff icon in the table, and it is the same buff icon
                let same = data(w, id)
                    .and_then(|d| d.buff_table.get(&info.id.0))
                    .is_some_and(|b| {
                        b.start_time == info.start_time && b.duration == info.duration
                    });
                if w.mobile(id).is_some_and(|m| m.net_state.is_some()) && same {
                    send_add_packet(w, id, &info, seconds as i64);
                }
            },
        );
    } else {
        // Round up, will be removed a little bit early by the server
        send_add_packet(w, id, info, seconds as i64);
    }
}

/// `ResendBuffs`
pub fn resend_buffs(w: &mut World, id: MobileId) {
    if !enabled(w) || client_has_buff_icons(w, id).is_none() {
        return;
    }
    let buffs: Vec<BuffInfo> =
        data(w, id).map_or_else(Vec::new, |d| d.buff_table.values().cloned().collect());
    for b in buffs {
        send_add_buff_packet(w, id, &b);
    }
}

/// `AddBuff`
pub fn add_buff(w: &mut World, id: MobileId, mut b: BuffInfo) {
    if !enabled(w) {
        return;
    }
    remove_buff(w, id, b.id); // Check, stop old timer, & subsequently remove the old one.
    // BuffInfo.StartTimer
    if b.duration != TimeSpan::ZERO {
        b.start_time = w.now;
        let icon = b.id;
        b.timer = timer::start(w, b.duration, TimeSpan::ZERO, 1, move |w| {
            remove_buff(w, id, icon)
        });
    }
    let Some(d) = data_mut(w, id) else { return };
    d.buff_table.insert(b.id.0, b.clone());
    send_add_buff_packet(w, id, &b);
}

/// `RemoveBuff`
pub fn remove_buff(w: &mut World, id: MobileId, icon: BuffIcon) {
    let Some(d) = data_mut(w, id) else { return };
    let Some(mut info) = d.buff_table.shift_remove(&icon.0) else {
        return;
    };
    info.timer.cancel(w); // BuffInfo.StopTimer
    if client_has_buff_icons(w, id).is_some() {
        send_remove_packet(w, id, icon);
    }
}

pub type BuffTable = IndexMap<i16, BuffInfo>;

#[cfg(test)]
mod tests {
    //! Buff icon packet tests. The legacy AddBuffPacket and
    //! RemoveBuffPacket writers are spelled out as the expected bytes.
    use super::*;
    use muo_server::net_state::{NetOut, NetStateData, NetStateId};

    fn client() -> (World, MobileId, NetStateId) {
        let mut w = muo_server::map::test_support::world();
        let m = w.new_mobile(&muo_server::mobile::MOBILE_BASE);
        let ns = NetStateId(1);
        w.net_states.insert(ns, NetStateData::default());
        w.mobile_mut(m).unwrap().net_state = Some(ns);
        (w, m, ns)
    }

    fn sent(w: &mut World) -> Vec<u8> {
        match w.net_out.pop() {
            Some(NetOut::Send(_, b)) => b,
            _ => Vec::new(),
        }
    }

    /// `new AddBuffPacket(mob, iconID, titleCliloc, secondaryCliloc, args, length)`.
    fn legacy_add(
        mob: u32,
        icon: i16,
        title: i32,
        secondary: i32,
        args: Option<&str>,
        seconds: i16,
    ) -> Vec<u8> {
        let mut b = vec![0xDF, 0, 0];
        b.extend(mob.to_be_bytes());
        b.extend(icon.to_be_bytes());
        b.extend(1i16.to_be_bytes());
        b.extend([0; 4]);
        b.extend(icon.to_be_bytes());
        b.extend(1i16.to_be_bytes());
        b.extend([0; 4]);
        b.extend(seconds.to_be_bytes());
        b.extend([0; 3]);
        b.extend(title.to_be_bytes());
        b.extend(secondary.to_be_bytes());
        match args {
            None => b.extend([0; 10]),
            Some(a) => {
                b.extend([0; 4]);
                b.extend(1i16.to_be_bytes());
                b.extend([0; 2]);
                for u in format!("\t{a}").encode_utf16() {
                    b.extend(u.to_le_bytes());
                }
                b.extend([0, 0]);
                b.extend(1i16.to_be_bytes());
                b.extend([0; 2]);
            }
        }
        let len = b.len() as u16;
        b[1..3].copy_from_slice(&len.to_be_bytes());
        b
    }

    #[test]
    fn add_buff_icon() {
        for (icon, title, secondary, args) in [
            (BuffIcon::Clumsy, 500100, 300200, Some("123456")),
            (BuffIcon::Disguised, 500102, 300203, None),
        ] {
            let (mut w, m, _) = client();
            let mut info = BuffInfo::with_secondary(icon, title, secondary);
            if let Some(a) = args {
                info = info.args(TextDefinition::of_text(a));
            }
            // (int)TimeSpan(8000 ticks).TotalMilliseconds is 0
            send_add_packet(&mut w, m, &info, 0);
            let serial = w.mobile(m).unwrap().serial.value();
            assert_eq!(
                sent(&mut w),
                legacy_add(serial, icon.0, title, secondary, args, 0),
                "{icon:?}"
            );
        }
    }

    #[test]
    fn remove_buff_icon() {
        let (mut w, m, _) = client();
        send_remove_packet(&mut w, m, BuffIcon::Disguised);
        let serial = w.mobile(m).unwrap().serial.value();
        // new RemoveBuffPacket(m, icon): 0xDF, length 15, serial, icon, 0, 4 zero bytes
        let mut want = vec![0xDF, 0, 15];
        want.extend(serial.to_be_bytes());
        want.extend(BuffIcon::Disguised.0.to_be_bytes());
        want.extend(0i16.to_be_bytes());
        want.extend([0; 4]);
        assert_eq!(sent(&mut w), want);
    }
}
