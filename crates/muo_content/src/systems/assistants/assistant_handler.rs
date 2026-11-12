//! Assistant handler: asks a logging-in client for its assistant version, hands it
//! the disallowed features, and kicks or warns a client that never answers the handshake.
//!
//! `configure` registers packet 0xBE ([`assist_version`]) and the 0xF0 handshake reply. [`on_login`] is the
//! `PlayerMobile.PlayerLoginEvent` handler the player module calls. The failure warning gump (`StaticWarningGump`)
//! is drawn by the Gumps crate through [`AssistantHooks::warning_gump`].

use indexmap::IndexMap;
use muo_io::buffers::SpanReader;
use muo_io::text::TextDefinition;
use muo_net::net_state::NetState;
use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::mobile::AccessLevel;
use muo_server::net_state::NetStateId;
use muo_server::timer::{self, TimerHandle};

use super::assistant_configuration::AssistantConfiguration;
use super::assistant_protocol;

/// The statics: `Enabled` and `_handshakes`.
#[derive(Default)]
pub struct AssistantHandler {
    pub enabled: bool,
    handshakes: IndexMap<MobileId, TimerHandle>,
}

/// What the Gumps crate sets: shows `AssistantFailureWarningGump` (420x250, no cancel button) with this content.
#[derive(Default)]
pub struct AssistantHooks {
    pub warning_gump: Option<fn(&mut World, MobileId, &TextDefinition)>,
}

/// `AssistantHandler.Configure()`.
pub fn configure(w: &mut World) {
    let enabled = w
        .config
        .get_or_update_setting("assistants.enableNegotiation", false)
        .unwrap_or(false);
    w.resources.get_mut::<AssistantHandler>().enabled = enabled;
    assistant_protocol::register(w, 0xFF, false, handshake_response);
    // `IncomingPackets.Register(0xBE, 0, false, &AssistVersion)`
    crate::network::register_packet(w, 0xBE, 0, false, false, assist_version);
}

fn net_running(w: &World, m: MobileId) -> Option<NetStateId> {
    let ns = w.mobile(m)?.net_state?;
    (!w.cannot_send_packets(ns)).then_some(ns)
}

fn failed_negotiation(w: &mut World, m: MobileId) {
    let Some(mob) = w.mobile(m) else { return };
    let is_player = mob.access_level <= AccessLevel::Player;
    let Some(ns) = mob.net_state else { return };
    let (will_kick, delay, warn, message) = {
        let s = &w.resources.get_mut::<AssistantConfiguration>().settings;
        (
            s.kick_on_failure,
            s.disconnect_delay,
            s.warn_on_failure,
            s.warning_message.clone(),
        )
    };

    if will_kick && is_player && delay <= TimeSpan::ZERO {
        w.disconnect(ns, "Failed to negotiate assistant features.");
        return;
    }

    if warn && let Some(show) = w.resources.get_mut::<AssistantHooks>().warning_gump {
        // TextDefinition.op_Implicit(string): "1234" is a cliloc number, anything else text.
        show(
            w,
            m,
            &message.map_or(TextDefinition::EMPTY, |s| TextDefinition::parse(&s)),
        );
    }

    if is_player {
        let h = timer::start(w, delay, TimeSpan::ZERO, 1, move |w| {
            on_force_disconnect(w, m)
        });
        w.resources
            .get_mut::<AssistantHandler>()
            .handshakes
            .insert(m, h);
    }

    let desc = w
        .net_state(ns)
        .map(|d| d.description.clone())
        .unwrap_or_default();
    tracing::info!("Failed to negotiate assistant features. [{desc}]");
}

/// `OnLogin(PlayerMobile pm)`.
pub fn on_login(w: &mut World, pm: MobileId) {
    let Some(ns) = net_running(w, pm) else { return };
    if !w.resources.get_mut::<AssistantHandler>().enabled {
        return;
    }
    send_assist_version_req(w, ns);
    send_assist_handshake(w, ns);

    if let Some(mut t) = w
        .resources
        .get_mut::<AssistantHandler>()
        .handshakes
        .shift_remove(&pm)
    {
        t.cancel(w);
    }
    let h = timer::start(w, TimeSpan::from_secs(30), TimeSpan::ZERO, 1, move |w| {
        on_timeout(w, pm)
    });
    w.resources
        .get_mut::<AssistantHandler>()
        .handshakes
        .insert(pm, h);
}

/// `AssistVersion(state, reader)`: packet 0xBE. The old UOAssist protocol (an int and a client version) is not
/// supported, Razor Community Edition is.
pub fn assist_version(_w: &mut World, state: &mut NetState, body: &[u8]) {
    let mut reader = SpanReader::new(body);
    let Ok(version) = reader.read_ascii(None) else {
        return;
    };
    state.assistant = Some(if version.contains(' ') {
        version
    } else {
        format!("RazorCE {version}")
    });
}

fn handshake_response(w: &mut World, state: &mut NetState, _body: &[u8]) {
    let Some(m) = w.net_mobile(NetStateId(state.id() as u32)) else {
        return;
    };
    if let Some(mut t) = w
        .resources
        .get_mut::<AssistantHandler>()
        .handshakes
        .shift_remove(&m)
    {
        t.cancel(w);
    }
}

fn on_timeout(w: &mut World, m: MobileId) {
    // The timer that fired is this entry's own; drop it once stopped.
    if w.resources
        .get_mut::<AssistantHandler>()
        .handshakes
        .shift_remove(&m)
        .is_none()
    {
        return;
    }
    if net_running(w, m).is_none() || !w.resources.get_mut::<AssistantHandler>().enabled {
        return;
    }
    failed_negotiation(w, m);
}

fn on_force_disconnect(w: &mut World, m: MobileId) {
    // `Mobile.ToString()`: `{Serial} "{Name}"`
    let name = muo_server::mobile::part1::combat::to_string(w, m);
    if let Some(ns) = w.mobile(m).and_then(|mob| mob.net_state) {
        w.disconnect(
            ns,
            &format!("Player {name} kicked (Failed assistant handshake)"),
        );
    }
    w.resources
        .get_mut::<AssistantHandler>()
        .handshakes
        .shift_remove(&m);
}

/// `SendAssistVersionReq`.
pub fn send_assist_version_req(w: &mut World, ns: NetStateId) {
    if w.cannot_send_packets(ns) {
        return;
    }
    w.send(ns, &[0xBE, 0x00, 0x03]);
}

/// The 0xF0 / 0xFE packet: the disallowed features.
pub fn assist_handshake_packet(features: u64) -> [u8; 12] {
    let mut p = [0u8; 12];
    p[0] = 0xF0; // Packet ID
    p[1..3].copy_from_slice(&12u16.to_be_bytes());
    p[3] = 0xFE; // Command
    p[4..12].copy_from_slice(&features.to_be_bytes());
    p
}

/// `SendAssistHandshake`.
pub fn send_assist_handshake(w: &mut World, ns: NetStateId) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let features = w
        .resources
        .get_mut::<AssistantConfiguration>()
        .settings
        .disallowed_features
        .bits();
    w.send(ns, &assist_handshake_packet(features));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::assistants::assistant_features::AssistantFeatures;

    #[test]
    fn handshake_bytes() {
        let p = assist_handshake_packet(
            (AssistantFeatures::FILTER_WEATHER | AssistantFeatures::SMART_TARGET).bits(),
        );
        assert_eq!(p, [0xF0, 0x00, 0x0C, 0xFE, 0, 0, 0, 0, 0, 0, 0, 0x05]);
    }
}
