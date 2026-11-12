//! `ClientGump` (the `[Client` user information gump).
//!
//! `NetState.Assistant`, `BaseCommand.IsAccessible`, `CommandLogging.WriteLine`, `PropertiesGump` and `SkillsGump`
//! are reached through [`AlHooks`](super::hooks::AlHooks) (all set in `hooks::configure`).

use muo_content::gumps::base::{LegacyGumpLogic, LegacyGumpObj, RelayInfo};
use muo_primitives::html;
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::mobile::AccessLevel;
use muo_server::net_state::NetStateId;

use super::common::{map_name, mobile_of, mobile_string, name_of, send};
use super::hooks::hooks;

const LABEL_COLOR32: i32 = 0xFFFFFF;

pub struct ClientGump {
    state: Option<NetStateId>,
}

fn access(w: &World, m: MobileId) -> AccessLevel {
    muo_content::mx::access_level(w, m)
}

fn log(w: &mut World, from: MobileId, what: &str) {
    if let Some(f) = hooks(w).and_then(|h| h.command_log) {
        let line = format!(
            "{:?} {} {what}",
            access(w, from),
            muo_commands::logging::format_entity(w, from.into())
        ); // `CommandLogging.Format(from)`
        f(w, from, &line);
    }
}

/// `new ClientGump(from, state, initialText)`.
pub fn new(
    w: &World,
    from: MobileId,
    state: Option<NetStateId>,
    initial_text: &str,
) -> LegacyGumpObj<ClientGump> {
    let mut g = LegacyGumpObj::new(30, 20, ClientGump { state });
    let Some(state) = state else { return g };
    let l = &mut g.layout;
    let c = |s: &str| html::color_rgb(s, LABEL_COLOR32);

    l.add_page(0);
    l.add_background(0, 0, 420, 314, 5054);
    l.add_image_tiled(10, 10, 400, 19, 0xA40);
    l.add_alpha_region(10, 10, 400, 19);
    l.add_image_tiled(10, 32, 400, 272, 0xA40);
    l.add_alpha_region(10, 32, 400, 272);
    l.add_html(
        10,
        10,
        380,
        20,
        &html::center_rgb("User Information", LABEL_COLOR32),
        false,
        false,
    );

    let mut line = 0;
    let data = w.net_state(state);

    l.add_html(14, 36 + line * 20, 200, 20, &c("Address:"), false, false);
    let desc = data.map_or(String::new(), |d| d.description.clone());
    l.add_html(70, 36 + line * 20, 200, 20, &c(&desc), false, false);
    line += 1;

    l.add_html(14, 36 + line * 20, 200, 20, &c("Client:"), false, false);
    // `(state.Version?.ToString().DefaultIfNullOrEmpty("(null)")).Color(..)`: no version at all is blank
    let version = data
        .and_then(|d| d.caps.version.as_ref())
        .map(|v| v.to_string())
        .map(|s| if s.is_empty() { "(null)".to_owned() } else { s });
    l.add_html(
        72,
        36 + line * 20,
        200,
        20,
        &c(&version.unwrap_or_default()),
        false,
        false,
    );
    line += 1;

    l.add_html(14, 36 + line * 20, 200, 20, &c("Assistant:"), false, false);
    let assistant = hooks(w)
        .and_then(|h| h.client_assistant)
        .and_then(|f| f(w, state));
    l.add_html(
        72,
        36 + line * 20,
        200,
        20,
        &c(&assistant.unwrap_or_else(|| "(-none-)".to_owned())),
        false,
        false,
    );
    line += 1;

    l.add_html(14, 36 + line * 20, 200, 20, &c("Version:"), false, false);
    let expansion = data
        .zip(w.expansion_table())
        .map(|(d, t)| d.caps.expansion_info(t).name.clone())
        .unwrap_or_default();
    l.add_html(72, 36 + line * 20, 200, 20, &c(&expansion), false, false);
    line += 1;

    let m = data.and_then(|d| d.mobile);
    let account = data.and_then(|d| d.account.clone());

    if access(w, from) >= AccessLevel::GameMaster
        && let Some(a) = account
    {
        l.add_html(14, 36 + line * 20, 200, 20, &c("Account:"), false, false);
        l.add_html(72, 36 + line * 20, 200, 20, &c(&a), false, false);
        line += 1;
    }

    if let Some(m) = m {
        l.add_html(14, 36 + line * 20, 200, 20, &c("Mobile:"), false, false);
        let who = format!(
            "{} ({})",
            name_of(w, m),
            w.mobile(m)
                .map(|mm| mm.serial.to_string())
                .unwrap_or_default()
        );
        l.add_html(72, 36 + line * 20, 200, 20, &c(&who), false, false);
        line += 1;

        l.add_html(14, 36 + line * 20, 200, 20, &c("Location:"), false, false);
        let loc = muo_content::mx::location(w, m);
        let map = muo_content::mx::map(w, m).map_or_else(String::new, |mp| map_name(w, mp)); // `{m.Map}` of null is empty
        l.add_html(
            72,
            36 + line * 20,
            200,
            20,
            &c(&format!("{loc} [{map}]")),
            false,
            false,
        );
        line += 1;
        let _ = line;

        l.add_button(13, 197, 0xFAB, 0xFAD, 1);
        l.add_html(48, 198, 200, 20, &c("Send Message"), false, false);

        l.add_image_tiled(12, 222, 376, 80, 0xA40);
        l.add_image_tiled(13, 223, 374, 78, 0xBBC);
        l.add_text_entry(15, 223, 372, 78, 0x480, 0, initial_text);

        l.add_image_tiled(265, 35, 142, 144, 5058);

        l.add_image_tiled(266, 36, 140, 142, 0xA40);
        l.add_alpha_region(266, 36, 140, 142);

        let mut line = 0;
        let mut entry = |l: &mut muo_content::gumps::legacy::LegacyLayout, id: i32, text: &str| {
            l.add_button(266, 36 + line * 20, 0xFA5, 0xFA7, id);
            l.add_html(300, 38 + line * 20, 100, 20, &c(text), false, false);
            line += 1;
        };

        let accessible = hooks(w)
            .and_then(|h| h.is_accessible)
            .is_none_or(|f| f(w, from, m));
        if accessible {
            entry(l, 4, "Properties");
        }

        if from != m {
            entry(l, 5, "Go to them");
            entry(l, 6, "Bring them here");
        }

        entry(l, 7, "Move to target");

        let (fa, ma) = (access(w, from), access(w, m));
        if fa >= AccessLevel::GameMaster && fa > ma {
            entry(l, 8, "Disconnect");
            if muo_content::mx::alive(w, m) {
                entry(l, 9, "Kill");
            } else {
                entry(l, 10, "Resurrect");
            }
        }

        if fa >= AccessLevel::Counselor && fa > ma {
            entry(l, 11, "Skills browser");
        }
    }
    g
}

fn resend(w: &mut World, to: MobileId, state: Option<NetStateId>, info: &RelayInfo) {
    let g = new(w, to, state, info.get_text_entry(0).unwrap_or(""));
    send(w, to, g);
}

impl LegacyGumpLogic for ClientGump {
    const TYPE_NAME: &'static str = "Server.Gumps.ClientGump";

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(state) = self.state else { return };
        let Some(from) = mobile_of(w, sender) else {
            return;
        };

        let Some(focus) = w.net_state(state).and_then(|d| d.mobile) else {
            muo_content::mx::send_message(w, from, "That character is no longer online.");
            return;
        };

        if muo_content::mx::deleted(w, focus) {
            muo_content::mx::send_message(w, from, "That character no longer exists.");
            return;
        }

        let focus_hidden = w.mobile(focus).is_some_and(|m| m.hidden);
        if from != focus
            && focus_hidden
            && access(w, from) < access(w, focus)
            && !muo_content::player_mobile::data(w, focus)
                .is_some_and(|d| d.visibility_list.contains(&from))
        {
            muo_content::mx::send_message(w, from, "That character is no longer visible.");
            return;
        }

        let (fa, fo) = (access(w, from), access(w, focus));
        let (from_s, focus_s) = (
            mobile_string(w, from),
            muo_commands::logging::format_entity(w, focus.into()),
        );
        match info.button_id {
            1 => {
                // Tell
                let text = info
                    .get_text_entry(0)
                    .map(str::trim)
                    .unwrap_or("")
                    .to_owned();
                if !text.is_empty() {
                    let from_name = name_of(w, from);
                    muo_server::mobile::part2::messages::send_message_hue(
                        w,
                        focus,
                        0x482,
                        &format!("{from_name} tells you:"),
                    );
                    muo_server::mobile::part2::messages::send_message_hue(w, focus, 0x482, &text);
                    log(w, from, &format!("telling {focus_s} \"{text}\""));
                }
                let g = new(w, from, self.state, "");
                send(w, from, g);
            }
            4 => {
                // Props
                resend(w, from, self.state, info);
                let accessible = hooks(w)
                    .and_then(|h| h.is_accessible)
                    .is_none_or(|f| f(w, from, focus));
                if !accessible {
                    muo_content::mx::msg(w, from, 500447); // That is not accessible.
                } else {
                    if let Some(f) = hooks(w).and_then(|h| h.open_properties_gump) {
                        f(w, from, focus);
                    }
                    log(w, from, &format!("opening properties gump of {focus_s}"));
                }
            }
            5 => {
                // Go to
                let map = muo_content::mx::map(w, focus);
                match map {
                    Some(map) if map != muo_server::map::MapId::INTERNAL => {
                        let loc = muo_content::mx::location(w, focus);
                        muo_server::mobile::base::move_to_world(w, from, loc, map);
                        resend(w, from, self.state, info);
                        log(
                            w,
                            from,
                            &format!("going to {focus_s}, {loc} ({})", map_name(w, map)),
                        );
                    }
                    _ => muo_content::mx::send_message(
                        w,
                        from,
                        "That character is not in the world.",
                    ),
                }
            }
            6 => {
                // Get
                match muo_content::mx::map(w, from) {
                    Some(map) if map != muo_server::map::MapId::INTERNAL => {
                        let loc = muo_content::mx::location(w, from);
                        muo_server::mobile::base::move_to_world(w, focus, loc, map);
                        resend(w, from, self.state, info);
                        log(
                            w,
                            from,
                            &format!("bringing {focus_s} to {loc} ({})", map_name(w, map)),
                        );
                    }
                    _ => {
                        muo_content::mx::send_message(w, from, "You cannot bring that person here.")
                    }
                }
            }
            7 => {
                // Move
                muo_content::targets::move_target(w, from, focus.into());
                resend(w, from, self.state, info);
            }
            8 => {
                // Kick
                if fa >= AccessLevel::GameMaster && fa > fo {
                    muo_server::mobile::part2::messages::say_text(w, focus, "I've been kicked!");
                    w.disconnect(state, &format!("Kicked by {from_s}."));
                    log(w, from, &format!("kicking {focus_s}"));
                }
            }
            9 => {
                // Kill
                if fa >= AccessLevel::GameMaster && fa > fo {
                    muo_server::mobile::base::kill(w, focus);
                    log(w, from, &format!("killing {focus_s}"));
                }
                resend(w, from, self.state, info);
            }
            10 => {
                // Res
                if fa >= AccessLevel::GameMaster && fa > fo {
                    muo_content::mx::play_sound(w, focus, 0x214);
                    muo_content::mx::fixed_effect(w, focus, 0x376A, 10, 16);
                    muo_server::mobile::base::resurrect(w, focus);
                    log(w, from, &format!("resurrecting {focus_s}"));
                }
                resend(w, from, self.state, info);
            }
            11 => {
                // Skills
                resend(w, from, self.state, info);
                if fa > fo {
                    if let Some(f) = hooks(w).and_then(|h| h.open_skills_gump) {
                        f(w, from, focus);
                    }
                    log(w, from, &format!("opening Skills gump of {focus_s}"));
                }
            }
            _ => {}
        }
    }
}
