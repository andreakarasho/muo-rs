//! Paperdoll, Profile, Keywords, ValidationQueue, PacketThrottles and AutoRestart.

use muo_primitives::{DateTime, Serial, TimeSpan};
use muo_server::World;
use muo_server::commands::{CommandEventArgs, CommandSystem};
use muo_server::ids::MobileId;
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part2::messages::send_message;
use muo_server::packets::player::{send_display_paperdoll, send_display_profile};
use muo_server::timer;
use std::rc::Rc;

// ---- Paperdoll ----

/// `Paperdoll.Initialize`.
pub fn paperdoll_initialize(w: &mut World) {
    w.events
        .paperdoll_request
        .subscribe(|w, (beholder, beheld)| paperdoll_request(w, *beholder, *beheld));
}

/// `EventSink_PaperdollRequest`.
pub fn paperdoll_request(w: &mut World, beholder: MobileId, beheld: MobileId) {
    let Some(ns) = w.mobile(beholder).and_then(|m| m.net_state) else {
        return;
    };
    let title = crate::misc::titles::compute_title(w, beholder, beheld);
    let (serial, warmode) = w
        .mobile(beheld)
        .map_or((Serial::ZERO, false), |m| (m.serial, m.warmode));
    let can_lift = muo_server::mobile::base::allow_equip_from(w, beheld, beholder);
    send_display_paperdoll(w, ns, serial, &title, warmode, can_lift);

    let items: Vec<_> = w
        .mobile(beheld)
        .map(|m| m.items.clone())
        .unwrap_or_default();
    for item in items {
        muo_server::item::base::send_opl_packet_to(w, item, ns);
    }

    // NOTE: OSI sends MobileUpdate when opening your own paperdoll.
    // It has a very bad rubber-banding affect. What positive affects does it have?
}

// ---- Profile ----

/// `Profile.ChangeProfileRequest`.
pub fn change_profile_request(w: &mut World, beholder: MobileId, _beheld: MobileId, text: &str) {
    if w.mobile(beholder).is_some_and(|m| m.profile_locked) {
        send_message(
            w,
            beholder,
            "Your profile is locked. You may not change it.",
        );
    } else if let Some(m) = w.mobile_mut(beholder) {
        m.profile = Some(text.to_owned());
    }
}

/// `Profile.ProfileRequest`.
pub fn profile_request(w: &mut World, beholder: MobileId, beheld: MobileId) {
    use muo_server::mobile::base::can_see_mobile;
    let (Some(a), Some(b)) = (w.mobile(beholder), w.mobile(beheld)) else {
        return;
    };
    if !b.player {
        return;
    }
    let in_range = muo_primitives::utility::in_range_3d(a.location, b.location, 12);
    if a.map != b.map || !in_range || !can_see_mobile(w, beholder, beheld) {
        return;
    }
    let header = crate::misc::titles::compute_title(w, beholder, beheld);
    let mut footer = String::new();
    let (locked, serial) = w
        .mobile(beheld)
        .map_or((false, Serial::ZERO), |m| (m.profile_locked, m.serial));
    if locked {
        if beholder == beheld {
            footer = "Your profile has been locked.".to_owned();
        } else if w
            .mobile(beholder)
            .is_some_and(|m| m.access_level >= AccessLevel::Counselor)
        {
            footer = "This profile has been locked.".to_owned();
        }
    }
    if footer.is_empty() && beholder == beheld {
        footer = get_account_duration(w, beheld);
    }
    let body = w
        .mobile(beheld)
        .and_then(|m| m.profile.clone())
        .unwrap_or_default();
    let serial = if beholder != beheld || !locked {
        serial
    } else {
        Serial::ZERO
    };
    if let Some(ns) = w.mobile(beholder).and_then(|m| m.net_state) {
        send_display_profile(w, ns, serial, Some(&header), Some(&body), Some(&footer));
    }
}

fn get_account_duration(w: &World, m: MobileId) -> String {
    let Some(a) = w.mobile(m).and_then(|mm| mm.account) else {
        return String::new();
    };
    let age = crate::accounting::account_age(w, a);
    for (value, unit) in [
        (age.total_days(), "day"),
        (age.total_hours(), "hour"),
        (age.total_minutes(), "minute"),
        (age.total_seconds(), "second"),
    ] {
        if let Some(s) = format_duration(value, unit) {
            return s;
        }
    }
    String::new()
}

/// `Profile.Format`: "This account is {0} {unit}{1} old." when `value >= 1`.
pub fn format_duration(value: f64, unit: &str) -> Option<String> {
    (value >= 1.0).then(|| {
        let n = value as i64;
        format!(
            "This account is {n} {unit}{} old.",
            if n != 1 { "s" } else { "" }
        )
    })
}

// ---- Keywords ----

/// `Keywords.Initialize`.
pub fn keywords_initialize(w: &mut World) {
    w.events.speech.subscribe(keywords_speech);
}

/// `EventSink_Speech`.
// The 0x0032 keyword (`PlayerMurderSystem.ReportKillsToSelf`) is subscribed by the murder system itself.
pub fn keywords_speech(w: &mut World, args: &mut muo_server::events::SpeechEventArgs) {
    let from = args.mobile;
    for k in args.keywords.clone() {
        match k {
            0x002A => {
                // *i resign from my guild*: ((Guild)from.Guild)?.RemoveMember(from)
                if let Some(g) = crate::misc::guild_class::mobile_guild(w, from)
                    .filter(|&g| crate::misc::guild_class::is_guild(w, g))
                {
                    crate::misc::guild_class::remove_member(w, g, from, 1018028);
                }
            }
            0x0035 => {
                // i renounce my young player status*
                if crate::player_mobile::data(w, from).is_some()
                    && crate::player_mobile::young(w, from)
                    && let Some(f) = crate::hooks::hooks(w).and_then(|h| h.renounce_young_gump)
                {
                    f(w, from);
                }
            }
            _ => {}
        }
    }
}

// ---- ValidationQueue ----

/// `ValidationQueue<T>`: objects queue a check during load; `Initialize` runs them all once.
#[derive(Default)]
pub struct ValidationQueue(Vec<Box<ValidateFn>>);

type ValidateFn = dyn FnOnce(&mut World);

impl ValidationQueue {
    /// `ValidationQueue<T>.Add(obj)`, with `obj.Validate()` as the closure.
    pub fn add(w: &mut World, validate: impl FnOnce(&mut World) + 'static) {
        w.resources
            .get_mut::<ValidationQueue>()
            .0
            .push(Box::new(validate));
    }
}

/// `ValidationQueue.Initialize`.
pub fn validation_queue_initialize(w: &mut World) {
    let list = std::mem::take(&mut w.resources.get_mut::<ValidationQueue>().0);
    for f in list {
        f(w);
    }
}

// ---- PacketThrottles ----

const THROTTLES_CONFIGURATION: &str = "Configuration/throttles.json";

/// `Delays`: milliseconds per packet id.
pub struct PacketThrottles(pub [i32; 0x100]);

impl Default for PacketThrottles {
    fn default() -> Self {
        Self([0; 0x100])
    }
}

/// `PacketThrottles.Throttle(packetId, ns)`: `ns.Mobile is { AccessLevel: Player } && Core.TickCount -
/// ns.GetPacketTime(packetId) < Delays[packetId]`.
fn throttle(w: &mut World, packet_id: i32, ns: &mut muo_net::net_state::NetState) -> bool {
    let muo_net::net_state::Attachment::Mobile(serial) = ns.attachment else {
        return false;
    };
    let player = w
        .find_mobile(Serial::new(serial))
        .and_then(|m| w.mobile(m))
        .is_some_and(|m| m.access_level == AccessLevel::Player);
    let delay = usize::try_from(packet_id)
        .ok()
        .and_then(|i| w.resources.get::<PacketThrottles>().map(|t| t.0[i & 0xFF]))
        .unwrap_or(0);
    player && w.tick_count.wrapping_sub(ns.get_packet_time(packet_id)) < i64::from(delay)
}

/// `PacketThrottles.Configure`.
pub fn packet_throttles_configure(w: &mut World) {
    CommandSystem::register(
        w,
        "GetThrottle",
        AccessLevel::Administrator,
        &[],
        Rc::new(get_throttle),
    );
    CommandSystem::register(
        w,
        "SetThrottle",
        AccessLevel::Administrator,
        &[],
        Rc::new(set_throttle),
    );
}

/// `PacketThrottles.Initialize`.
pub fn packet_throttles_initialize(w: &mut World) {
    let path = w.core.base_directory.join(THROTTLES_CONFIGURATION);
    let mut delays = [0i32; 0x100];
    if let Ok(text) = std::fs::read_to_string(&path) {
        if let Ok(map) = muo_data::json::from_str::<std::collections::BTreeMap<String, i32>>(&text)
        {
            for (k, v) in map {
                let id = k.strip_prefix("0x").map_or_else(
                    || k.parse::<i32>().ok(),
                    |h| i32::from_str_radix(h, 16).ok(),
                );
                match id {
                    Some(i) if (0..0x100).contains(&i) => delays[i as usize] = v,
                    _ => tracing::warn!(
                        "Packet Throttles: Error deserializing {k} from {THROTTLES_CONFIGURATION}"
                    ),
                }
            }
        }
    } else {
        delays[0x03] = 25; // Speech
        delays[0xAD] = 25; // Speech
        delays[0x12] = 25; // Text Commands
        delays[0x75] = 500; // Rename request
    }
    for (i, d) in delays.iter().enumerate() {
        if *d > 0 {
            crate::network::register_throttler(w, i as u8, throttle);
        }
    }
    w.resources.get_mut::<PacketThrottles>().0 = delays;
    save_delays(w);
}

fn save_delays(w: &World) {
    let Some(t) = w.resources.get::<PacketThrottles>() else {
        return;
    };
    let table: std::collections::BTreeMap<String, i32> =
        t.0.iter()
            .enumerate()
            .filter(|(_, d)| **d != 0)
            .map(|(i, d)| (format!("0x{i:02X}"), *d))
            .collect();
    let path = w.core.base_directory.join(THROTTLES_CONFIGURATION);
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Ok(text) = serde_json::to_string_pretty(&table) {
        let _ = std::fs::write(path, text);
    }
}

/// `GetThrottle <packetID>`.
fn get_throttle(w: &mut World, e: &mut CommandEventArgs) {
    let from = e.mobile;
    if e.length() != 1 {
        send_message(
            w,
            from,
            "Invalid Command Format. Should be [GetThrottle <packetID>",
        );
        return;
    }
    let packet_id = e.get_int32(0);
    if !(0..=0x100).contains(&packet_id) {
        send_message(
            w,
            from,
            "Invalid Command Format. PacketID must be between 0 and 0x100.",
        );
        return;
    }
    let d = w
        .resources
        .get_mut::<PacketThrottles>()
        .0
        .get(packet_id as usize)
        .copied()
        .unwrap_or(0);
    send_message(
        w,
        from,
        &format!("Packet 0x{packet_id:X} throttle is currently {d}ms."),
    );
}

/// `SetThrottle <packetID> <timeInMilliseconds>`.
fn set_throttle(w: &mut World, e: &mut CommandEventArgs) {
    let from = e.mobile;
    if e.length() != 2 {
        send_message(
            w,
            from,
            "Invalid Command Format. Should be [SetThrottle <packetID> <timeInMilliseconds>",
        );
        return;
    }
    let packet_id = e.get_int32(0);
    let delay = e.get_int32(1);
    if !(0..=0x100).contains(&packet_id) {
        send_message(
            w,
            from,
            "Invalid Command Format. PacketID must be between 0 and 0x100.",
        );
        return;
    }
    if delay > 5000 {
        send_message(
            w,
            from,
            "Invalid Command Format. Delay cannot exceed 5000 milliseconds.",
        );
        return;
    }
    let i = packet_id as usize;
    if i >= 0x100 {
        return; // out of range of the 0x100 delay slots
    }
    let old = w.resources.get_mut::<PacketThrottles>().0[i];
    if old == 0 && delay > 0 {
        crate::network::register_throttler(w, packet_id as u8, throttle);
        send_message(
            w,
            from,
            &format!("Set throttle for packet 0x{packet_id:02X} to {delay}ms."),
        );
    } else if old > 0 && delay == 0 {
        // `RegisterThrottler(id, null)`: the registered callback never throttles a 0 delay.
        send_message(
            w,
            from,
            &format!("Removed throttle for packet 0x{packet_id:02X}"),
        );
    }
    w.resources.get_mut::<PacketThrottles>().0[i] = delay;
    save_delays(w);
}

// ---- AutoRestart ----

const RESTART_TIME: TimeSpan = TimeSpan::from_hours(2); // time of day at which to restart
// how long the server should remain active before restart (period of 'server wars')
const RESTART_DELAY: TimeSpan = TimeSpan::ZERO;
// at what interval should the shutdown message be displayed?
const WARNING_DELAY: TimeSpan = TimeSpan::from_mins(1);

#[derive(Default)]
pub struct AutoRestartState {
    enabled: bool,
    timer: Option<timer::TimerHandle>,
    restart_time: DateTime,
    pub restarting: bool,
}

fn ar(w: &mut World) -> &mut AutoRestartState {
    w.resources.get_mut::<AutoRestartState>()
}

/// `AutoRestart.Enabled` getter.
pub fn auto_restart_enabled(w: &World) -> bool {
    w.resources
        .get::<AutoRestartState>()
        .is_some_and(|a| a.enabled)
}

/// `AutoRestart.Enabled` setter.
pub fn auto_restart_set_enabled(w: &mut World, value: bool) {
    ar(w).enabled = value;
    if value {
        if ar(w).timer.is_none() {
            // new AutoRestart(): the restart is at 02:00 of today or tomorrow
            let now = w.now;
            let mut t = now.date() + RESTART_TIME;
            if t < now {
                t += TimeSpan::from_days(1);
            }
            ar(w).restart_time = t;
            let h = timer::start(
                w,
                TimeSpan::from_secs(1),
                TimeSpan::from_secs(1),
                0,
                auto_restart_tick,
            );
            ar(w).timer = Some(h);
        }
    } else if let Some(mut h) = ar(w).timer.take() {
        h.cancel(w);
    }
}

/// `AutoRestart.Configure`.
pub fn auto_restart_configure(w: &mut World) {
    let enabled = w
        .config
        .get_or_update_setting("world.enableAutoRestart", false)
        .unwrap_or(false);
    auto_restart_set_enabled(w, enabled);
    CommandSystem::register(
        w,
        "Restart",
        AccessLevel::Administrator,
        &[],
        Rc::new(restart_on_command),
    );
}

/// `Restart_OnCommand`.
fn restart_on_command(w: &mut World, e: &mut CommandEventArgs) {
    let from = e.mobile;
    if ar(w).restarting {
        send_message(w, from, "The server is already restarting.");
    } else {
        send_message(w, from, "You have initiated server shutdown.");
        auto_restart_set_enabled(w, true);
        let now = w.now;
        ar(w).restart_time = now;
    }
}

fn auto_restart_tick(w: &mut World) {
    if ar(w).restarting || !ar(w).enabled {
        return;
    }
    let (now, restart_time) = (w.now, ar(w).restart_time);
    if now < restart_time {
        return;
    }
    if WARNING_DELAY > TimeSpan::ZERO {
        warning_callback(w);
        timer::start(w, WARNING_DELAY, WARNING_DELAY, 0, warning_callback);
    }
    // AutoSave.Save()
    crate::systems::world_saves::auto_save::save(w);
    ar(w).restarting = true;
    timer::start(w, RESTART_DELAY, TimeSpan::ZERO, 1, |w| w.core.kill(true));
}

fn warning_callback(w: &mut World) {
    w.broadcast(0x22, true, "The server is going down shortly.");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn account_age_text() {
        assert_eq!(format_duration(0.5, "day"), None);
        assert_eq!(
            format_duration(1.0, "day").unwrap(),
            "This account is 1 day old."
        );
        assert_eq!(
            format_duration(2.7, "hour").unwrap(),
            "This account is 2 hours old."
        );
    }
}
