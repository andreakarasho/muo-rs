//! Client verification: refuses, warns or annoys clients whose version the
//! server does not support.
//!
//! `AnnoyGump : StaticNoticeGump<AnnoyGump>` (a static notice over the static warning gump) is here, with the
//! warning gump's layout written out for it.

use muo_primitives::dotnet::InvariantCase;
use std::cmp::Ordering;

use muo_net::client_version::{ClientType, ClientVersion};
use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part2::messages::{send_message, send_message_hue};
use muo_server::net_state::NetStateId;
use muo_server::timer;
use tracing::info;

use crate::gumps::builder::{GumpStrings, StaticGumpBuilder};
use crate::gumps::{RelayInfo, StaticGumpLogic, StaticGumpObj, send_gump_to_mobile};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum InvalidClientResponse {
    Ignore,
    Warn,
    Annoy,
    LenientKick,
    Kick,
}

impl InvalidClientResponse {
    fn parse(s: &str) -> Option<Self> {
        [
            Self::Ignore,
            Self::Warn,
            Self::Annoy,
            Self::LenientKick,
            Self::Kick,
        ]
        .into_iter()
        .find(|r| format!("{r:?}").eq_ignore_ascii_case(s.trim()))
    }
}

pub struct ClientVerificationState {
    enable: bool,
    invalid_client_response: InvalidClientResponse,
    version_expression: Option<String>,
    age_leniency: TimeSpan,
    game_time_leniency: TimeSpan,
    allowed_clients_message: String,
    /// `AllowedClientTypes`, `ClientType` flag bits.
    pub allowed_client_types: u32,
    pub kick_delay: TimeSpan,
    pub min_required: Option<ClientVersion>,
    pub max_required: Option<ClientVersion>,
}

impl Default for ClientVerificationState {
    fn default() -> Self {
        Self {
            enable: true,
            invalid_client_response: InvalidClientResponse::Kick,
            version_expression: None,
            age_leniency: TimeSpan::from_days(10),
            game_time_leniency: TimeSpan::from_hours(25),
            allowed_clients_message: String::new(),
            allowed_client_types: ClientType::Classic as u32 | ClientType::Sa as u32,
            kick_delay: TimeSpan::from_secs(20),
            min_required: None,
            max_required: None,
        }
    }
}

fn state(w: &mut World) -> &mut ClientVerificationState {
    w.resources.get_mut::<ClientVerificationState>()
}

/// `Enum.TryParse<ClientType>` over a flags value ("Classic, SA", a number).
fn parse_client_types(s: &str) -> Option<u32> {
    if let Ok(n) = s.trim().parse::<u32>() {
        return Some(n);
    }
    let mut bits = 0;
    for part in s.split(',') {
        bits |= match part.trim().lower_inv().as_str() {
            "none" => 0,
            "classic" => ClientType::Classic as u32,
            "uotd" => ClientType::Uotd as u32,
            "kr" => ClientType::Kr as u32,
            "sa" => ClientType::Sa as u32,
            _ => return None,
        };
    }
    Some(bits)
}

fn type_bit(t: ClientType) -> u32 {
    t as u32
}

/// `GetAllowedClientsString`.
fn get_allowed_clients_string(allowed: u32) -> String {
    // Get total number of allowed clients
    let total_allowed = allowed.count_ones() as i32 - 1;
    if total_allowed == 0 {
        return "There are no clients supported at this time.".to_owned();
    }
    let mut s = String::from("Please connect with a ");
    let mut flags = 0u32;
    let mut i = 0;
    while flags < allowed {
        flags = 1u32 << i;
        if i > 0 {
            s.push_str(if i == total_allowed { " or " } else { ", " });
        }
        s.push_str(match flags {
            2 => ClientType::Uotd.type_name(),
            4 => ClientType::Kr.type_name(),
            8 => ClientType::Sa.type_name(),
            _ => ClientType::Classic.type_name(),
        });
        i += 1;
    }
    s
}

/// `ClientVerification.Configure`.
pub fn configure(w: &mut World) {
    let min = w
        .config
        .get_setting_opt::<ClientVersion>("clientVerification.minRequired");
    let max = w
        .config
        .get_setting_opt::<ClientVersion>("clientVerification.maxRequired");
    let enable = w
        .config
        .get_or_update_setting("clientVerification.enable", true)
        .unwrap_or(true);
    let response: String = w
        .config
        .get_or_update_setting(
            "clientVerification.invalidClientResponse",
            "Kick".to_owned(),
        )
        .unwrap_or_else(|_| "Kick".to_owned());
    let age = w
        .config
        .get_or_update_setting("clientVerification.ageLeniency", TimeSpan::from_days(10))
        .unwrap_or(TimeSpan::from_days(10));
    let game = w
        .config
        .get_or_update_setting(
            "clientVerification.gameTimeLeniency",
            TimeSpan::from_hours(25),
        )
        .unwrap_or(TimeSpan::from_hours(25));
    let kick = w
        .config
        .get_or_update_setting("clientVerification.kickDelay", TimeSpan::from_secs(20))
        .unwrap_or(TimeSpan::from_secs(20));
    let allowed: String = w.config.get_setting(
        "clientVerification.allowedClientTypes",
        "Classic, SA".to_owned(),
    );
    let allowed =
        parse_client_types(&allowed).unwrap_or(ClientType::Classic as u32 | ClientType::Sa as u32);

    let s = state(w);
    s.min_required = min;
    s.max_required = max;
    s.enable = enable;
    s.invalid_client_response =
        InvalidClientResponse::parse(&response).unwrap_or(InvalidClientResponse::Kick);
    s.age_leniency = age;
    s.game_time_leniency = game;
    s.kick_delay = kick;
    s.allowed_client_types = allowed;
    s.allowed_clients_message = get_allowed_clients_string(allowed);
}

/// `ClientVerification.Initialize`.
pub fn initialize(w: &mut World) {
    let server_version = w
        .resources
        .get::<muo_data::client::UoClient>()
        .and_then(|c| c.server_client_version.clone())
        .or_else(|| {
            w.config
                .get_setting_opt::<ClientVersion>("clientData.clientVersion")
        });
    let s = state(w);
    if s.min_required.is_none() && s.max_required.is_none() {
        s.min_required = server_version;
    }
    if s.min_required.is_some() || s.max_required.is_some() {
        let expr = version_expression(s);
        let action = format!("{:?}", s.invalid_client_response);
        info!("Restricting client version to {expr}. Action to be taken: {action}");
    }
}

fn version_expression(s: &mut ClientVerificationState) -> String {
    if s.version_expression.is_none() {
        s.version_expression = Some(match (&s.min_required, &s.max_required) {
            (Some(a), Some(b)) => format!("{a}-{b}"),
            (Some(a), None) => format!("{a} or newer"),
            (None, Some(b)) => format!("{b} or older"),
            (None, None) => String::new(),
        });
    }
    s.version_expression.clone().unwrap_or_default()
}

/// `ClientVersionReceived(state, version)`.
pub fn client_version_received(w: &mut World, ns: NetStateId, version: &ClientVersion) {
    let Some(mobile) = w.net_state(ns).and_then(|d| d.mobile) else {
        return;
    };
    if !state(w).enable || w.mobile(mobile).map(|m| m.access_level) != Some(AccessLevel::Player) {
        return;
    }
    let created = w.mobile(mobile).map(|m| m.created);
    let game_time = crate::player_mobile::data(w, mobile).map(|d| d.game_time);
    let now = w.now;

    let (response, age_leniency, game_leniency) = {
        let s = state(w);
        (
            s.invalid_client_response,
            s.age_leniency,
            s.game_time_leniency,
        )
    };
    let strict_requirement = response == InvalidClientResponse::Kick
        || response == InvalidClientResponse::LenientKick
            && created.is_some_and(|c| now - c > age_leniency)
            && game_time.is_some_and(|g| g > game_leniency);

    let is_uotd = w.net_state(ns).is_some_and(|d| d.caps.is_uotd_client());
    let mut text = String::new();
    let mut should_kick = false;
    let is_kr = version.kind == ClientType::Kr;
    let (min, max, allowed, allowed_message) = {
        let s = state(w);
        (
            s.min_required.clone(),
            s.max_required.clone(),
            s.allowed_client_types,
            s.allowed_clients_message.clone(),
        )
    };

    if !is_kr
        && min
            .as_ref()
            .is_some_and(|m| version.compare_to(m) == Ordering::Less)
    {
        text.push_str(&format!(
            "This server doesn't support clients older than {}.",
            min.as_ref().expect("checked")
        ));
        should_kick = strict_requirement;
    } else if !is_kr
        && max
            .as_ref()
            .is_some_and(|m| version.compare_to(m) == Ordering::Greater)
    {
        text.push_str(&format!(
            "This server doesn't support clients newer than {}.",
            max.as_ref().expect("checked")
        ));
        should_kick = strict_requirement;
    } else {
        let allow = |t: ClientType| allowed & type_bit(t) != 0;
        if !allow(ClientType::Classic) && version.kind == ClientType::Classic {
            text.push_str("This server does not allow classic clients to connect.");
            should_kick = true;
        } else if !allow(ClientType::Uotd) && is_uotd {
            text.push_str("This server does not allow UO:TD clients to connect.");
            should_kick = true;
        } else if !allow(ClientType::Kr) && version.kind == ClientType::Kr {
            text.push_str("This server does not allow UO:KR clients to connect.");
            should_kick = true;
        } else if !allow(ClientType::Sa) && version.kind == ClientType::Sa {
            text.push_str("This server does not allow UO:SA clients to connect.");
            should_kick = true;
        }
        if !text.is_empty() {
            text.push_str(&allowed_message);
        }
    }

    if !text.is_empty() {
        send_message_hue(w, mobile, 0x22, &text);
    }

    if should_kick {
        let delay = state(w).kick_delay;
        send_message_hue(
            w,
            mobile,
            0x22,
            &format!(
                "You will be disconnected in {:.0} seconds.",
                delay.total_seconds()
            ),
        );
        timer::start(w, delay, TimeSpan::ZERO, 1, move |w| on_kick(w, ns));
        return;
    }

    if !text.is_empty() && response == InvalidClientResponse::Warn {
        let expr = version_expression(state(w));
        send_message_hue(
            w,
            mobile,
            0x22,
            &format!("This server recommends that your client version is {expr}."),
        );
    }
    if matches!(
        response,
        InvalidClientResponse::LenientKick | InvalidClientResponse::Annoy
    ) {
        send_annoy_gump(w, mobile);
    }
}

/// `SendAnnoyGump(m)`.
fn send_annoy_gump(w: &mut World, m: MobileId) {
    let Some(ns) = w.mobile(m).and_then(|mm| mm.net_state) else {
        return;
    };
    let version = w
        .net_state(ns)
        .and_then(|d| d.caps.version.clone())
        .map(|v| v.to_string())
        .unwrap_or_default();
    let content = format!(
        "Your client is invalid.<br>This server recommends that your client version is {}.<br><br>You are currently using version {version}.",
        version_expression(state(w))
    );
    let gump = StaticGumpObj::new(
        (640 - AnnoyGump::WIDTH) / 2,
        (480 - AnnoyGump::HEIGHT) / 2,
        AnnoyGump { mobile: m, content },
    );
    send_gump_to_mobile(w, m, Box::new(gump), false);
}

/// `KickMessage(from)`.
fn kick_message(w: &mut World, from: MobileId) {
    send_message(w, from, "You will be reminded of this again.");
    let (response, age, game) = {
        let s = state(w);
        (
            s.invalid_client_response,
            s.age_leniency,
            s.game_time_leniency,
        )
    };
    if response == InvalidClientResponse::LenientKick {
        send_message(
            w,
            from,
            &format!(
                "Invalid clients will be kicked after {} days of character age and {} hours of play time",
                age, game
            ),
        );
    }
    let minutes = i64::from(crate::hooks::rnd::from(w, 5, 10));
    timer::start(
        w,
        TimeSpan::from_mins(minutes),
        TimeSpan::ZERO,
        1,
        move |w| send_annoy_gump(w, from),
    );
}

struct AnnoyGump {
    mobile: MobileId,
    content: String,
}

impl AnnoyGump {
    const WIDTH: i32 = 480;
    const HEIGHT: i32 = 360;
}

impl StaticGumpLogic for AnnoyGump {
    const TYPE_NAME: &'static str = "Server.Misc.ClientVerification+AnnoyGump";

    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.set_no_dispose();
        b.set_no_resize();
        b.set_no_move();
        // StaticWarningGump.BuildLayout; StaticNoticeGump: Header 1060637, no cancel button
        let (width, height) = (Self::WIDTH, Self::HEIGHT);
        b.set_no_close();
        b.add_page(0);
        b.add_background(0, 0, width, height, 5054);
        b.add_image_tiled(10, 10, width - 20, 20, 2624);
        b.add_alpha_region(10, 10, width - 20, 20);
        b.add_html_localized_color(10, 10, width - 20, 20, 1060637, 0x7800, false, false); // NOTICE
        b.add_image_tiled(10, 40, width - 20, height - 80, 2624);
        b.add_alpha_region(10, 40, width - 20, height - 80);
        b.add_html_placeholder(10, 40, width - 20, height - 80, "content", false, true);
        b.add_image_tiled(10, height - 30, width - 20, 20, 2624);
        b.add_alpha_region(10, height - 30, width - 20, 20);
        b.add_button(10, height - 30, 4005, 4007, 1);
        b.add_html_localized_color(40, height - 30, 170, 20, 1011036, 32767, false, false); // OKAY
    }

    fn build_strings(&self, strings: &mut GumpStrings) {
        strings.set_html_text("content", &self.content, "#FFC000", -1);
    }

    /// The callback: `() => KickMessage(m)`, whatever button.
    fn on_response(&mut self, w: &mut World, _sender: NetStateId, _info: &RelayInfo) {
        kick_message(w, self.mobile);
    }
}

/// `OnKick`.
fn on_kick(w: &mut World, ns: NetStateId) {
    if w.net_state(ns).is_none() || w.cannot_send_packets(ns) {
        return;
    }
    let version = w.net_state(ns).and_then(|d| d.caps.version.clone());
    let v = version.map(|v| v.to_string()).unwrap_or_default();
    info!(
        "Client: {}: Disconnecting, bad version ({v})",
        w.net_state(ns)
            .map(|d| d.description.clone())
            .unwrap_or_default()
    );
    w.disconnect(ns, &format!("Invalid client version {v}."));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allowed_clients_text() {
        assert_eq!(
            get_allowed_clients_string(ClientType::Classic as u32 | ClientType::Sa as u32),
            "Please connect with a classic or UO:TD, UO:KR, UO:SA, classic"
        );
        assert_eq!(
            get_allowed_clients_string(ClientType::Sa as u32),
            "There are no clients supported at this time."
        );
        assert_eq!(parse_client_types("Classic, SA"), Some(9));
    }
}
