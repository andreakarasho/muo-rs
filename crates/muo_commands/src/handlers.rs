//! `CommandHandlers`: the basic staff and player commands.
//!
//! `commandsystem.prefix` is read in `configure`; the rest of the commands are in `handlers_rest.rs`.

use std::rc::Rc;

use muo_server::commands::{CommandEventArgs, CommandSystem};
use muo_server::ids::MobileId;
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part2::messages::{send_ascii_message_hue, send_message, send_message_hue};
use muo_server::packets::movement::{SpeedControlSetting, send_speed_control};
use muo_server::world::World;

use crate::logging;

pub fn configure(w: &mut World) {
    // CommandSystem.Prefix = ServerConfiguration.GetOrUpdateSetting("commandsystem.prefix", "[")
    if let Ok(prefix) = w
        .config
        .get_or_update_setting("commandsystem.prefix", "[".to_string())
    {
        w.resources.get_mut::<CommandSystem>().prefix = prefix;
    }
    crate::handlers_rest::configure(w);
    let mut reg = |name: &str,
                   level: AccessLevel,
                   aliases: &[&str],
                   f: fn(&mut World, &mut CommandEventArgs)| {
        CommandSystem::register(w, name, level, aliases, Rc::new(f));
    };
    reg("Where", AccessLevel::Counselor, &[], where_on_command);
    reg(
        "AutoPageNotify",
        AccessLevel::Counselor,
        &["APN"],
        apn_on_command,
    );
    reg("Animate", AccessLevel::GameMaster, &[], animate_on_command);
    reg("Help", AccessLevel::Player, &[], help_on_command);
    reg(
        "SMsg",
        AccessLevel::Counselor,
        &["S", "SM"],
        staff_message_on_command,
    );
    reg(
        "BCast",
        AccessLevel::GameMaster,
        &["B", "BC"],
        broadcast_message_on_command,
    );
    reg("Echo", AccessLevel::Counselor, &[], echo_on_command);
    reg("Sound", AccessLevel::GameMaster, &[], sound_on_command);
    reg(
        "SpeedBoost",
        AccessLevel::Counselor,
        &[],
        speed_boost_on_command,
    );
}

pub(crate) fn name_of(w: &World, m: MobileId) -> String {
    w.mobile(m)
        .and_then(|x| (x.ty.name)(w, m))
        .unwrap_or_default()
}

/// `[Usage("SpeedBoost [true|false]")]`
fn speed_boost_on_command(w: &mut World, e: &mut CommandEventArgs) {
    let from = e.mobile;
    let ns = w.mobile(from).and_then(|m| m.net_state);
    if e.length() <= 1 {
        let Some(ns) = ns else { return };
        if e.length() == 1 && !e.get_boolean(0) {
            send_speed_control(w, ns, SpeedControlSetting::Disable);
            send_message(w, from, "Speed boost has been disabled.");
        } else {
            send_speed_control(w, ns, SpeedControlSetting::Mount);
            send_message(w, from, "Speed boost has been enabled.");
        }
    } else {
        send_message(w, from, "Format: SpeedBoost [true|false]");
    }
}

/// `[Usage("Where")]`
fn where_on_command(w: &mut World, e: &mut CommandEventArgs) {
    let from = e.mobile;
    let Some((loc, map)) = w.mobile(from).map(|m| (m.location, m.map)) else {
        return;
    };
    let map_name = map
        .and_then(|m| w.maps.get(m))
        .map_or_else(String::new, |m| m.name().to_string());
    send_message(
        w,
        from,
        &format!("You are at {} {} {} in {map_name}.", loc.x, loc.y, loc.z),
    );
    if map.is_some() {
        let mut reg = muo_server::region::find(w, loc, map);
        if !muo_server::region::is_default(w, reg) {
            let mut text = String::new();
            let mut first = true;
            while let Some(r) = w.region(reg) {
                let label = r.name.clone().unwrap_or_else(|| r.ty.name.to_string());
                if first {
                    text += &label;
                    first = false;
                } else {
                    text += &format!(" <- {label}");
                }
                match r.parent {
                    Some(p) => reg = p,
                    None => break,
                }
            }
            send_message(w, from, &format!("Your region is {text}."));
        }
    }
}

/// `[Usage("Sound <index> [toAll=true]")]`
fn sound_on_command(w: &mut World, e: &mut CommandEventArgs) {
    let from = e.mobile;
    match e.length() {
        1 => play_sound(w, from, e.get_int32(0), true),
        2 => play_sound(w, from, e.get_int32(0), e.get_boolean(1)),
        _ => send_message(w, from, "Format: Sound <index> [toAll]"),
    }
}

fn play_sound(w: &mut World, m: MobileId, index: i32, to_all: bool) {
    let Some((map, loc)) = w.mobile(m).and_then(|x| x.map.map(|mm| (mm, x.location))) else {
        return;
    };
    let level = w.mobile(m).map_or(AccessLevel::Player, |x| x.access_level);
    let text = format!(
        "{level:?} {} playing sound {index} (toAll={to_all})",
        logging::format_entity(w, muo_server::ids::EntityId::Mobile(m))
    );
    logging::write_line(w, m, &text);
    let packet = muo_server::packets::effect::create_sound_effect(index, loc);
    for ns in muo_server::effects::clients_in_range(w, map, loc, Some(12)) {
        let sees = to_all
            || w.net_mobile(ns).is_some_and(|viewer| {
                muo_server::mobile::base::can_see_object(
                    w,
                    viewer,
                    Some(muo_server::ids::EntityId::Mobile(m)),
                )
            });
        if sees {
            w.send(ns, &packet);
        }
    }
}

/// `[Usage("Echo <text>")]`
fn echo_on_command(w: &mut World, e: &mut CommandEventArgs) {
    let to_echo = e.arg_string.trim().to_string();
    if to_echo.is_empty() {
        send_message(w, e.mobile, "Format: Echo \"<text>\"");
    } else {
        send_message(w, e.mobile, &to_echo);
    }
}

/// `[Usage("Help")]`: the commands the player may use, packed into lines under 256 characters.
fn help_on_command(w: &mut World, e: &mut CommandEventArgs) {
    let m = e.mobile;
    let level = w.mobile(m).map_or(AccessLevel::Player, |x| x.access_level);
    // `list.Sort()`: CommandEntry.CompareTo, ordinal by command name.
    // `CommandSystem.Entries.Values`: every entry, aliases too (not `GetList`, which drops entries sharing a handler)
    let mut names: Vec<String> = w
        .resources
        .get_mut::<CommandSystem>()
        .entries
        .values()
        .filter(|c| level >= c.access_level)
        .map(|c| c.command.clone())
        .collect();
    names.sort();
    let mut sb = String::new();
    if let Some(first) = names.first() {
        sb.push_str(first);
    }
    for v in names.iter().skip(1) {
        if sb.len() + 1 + v.len() >= 256 {
            send_ascii_message_hue(w, m, 0x482, &sb);
            sb.clear();
            sb.push_str(v);
        } else {
            sb.push(' ');
            sb.push_str(v);
        }
    }
    if !sb.is_empty() {
        send_ascii_message_hue(w, m, 0x482, &sb);
    }
}

/// `BroadcastMessage(ac, hue, message)`: to every connected mobile of at least that access level.
pub fn broadcast_message(w: &mut World, ac: AccessLevel, hue: i32, message: &str) {
    let targets: Vec<MobileId> = w.net_states.values().filter_map(|d| d.mobile).collect();
    for m in targets {
        if w.mobile(m).is_some_and(|x| x.access_level >= ac) {
            send_message_hue(w, m, hue, message);
        }
    }
}

/// `[Usage("SMsg <text>")]`
fn staff_message_on_command(w: &mut World, e: &mut CommandEventArgs) {
    let hue = w.mobile(e.mobile).map_or(0, |m| m.speech_hue);
    let text = format!("[{}] {}", name_of(w, e.mobile), e.arg_string);
    broadcast_message(w, AccessLevel::Counselor, hue, &text);
}

/// `[Usage("BCast <text>")]`
fn broadcast_message_on_command(w: &mut World, e: &mut CommandEventArgs) {
    let head = format!("Staff message from {}:", name_of(w, e.mobile));
    broadcast_message(w, AccessLevel::Player, 0x482, &head);
    broadcast_message(w, AccessLevel::Player, 0x482, &e.arg_string);
}

/// `[Usage("AutoPageNotify")]`
fn apn_on_command(w: &mut World, e: &mut CommandEventArgs) {
    let m = e.mobile;
    let on = match w.mobile_mut(m) {
        Some(x) => {
            x.auto_page_notify = !x.auto_page_notify;
            x.auto_page_notify
        }
        None => return,
    };
    send_message(
        w,
        m,
        if on {
            "Your auto-page-notify has been turned on."
        } else {
            "Your auto-page-notify has been turned off."
        },
    );
}

/// `[Usage("Animate <action> <frameCount> <repeatCount> <forward> <repeat> <delay>")]`
fn animate_on_command(w: &mut World, e: &mut CommandEventArgs) {
    if e.length() == 6 {
        muo_server::mobile::base::animate(
            w,
            e.mobile,
            e.get_int32(0),
            e.get_int32(1),
            e.get_int32(2),
            e.get_boolean(3),
            e.get_boolean(4),
            e.get_int32(5),
        );
    } else {
        send_message(
            w,
            e.mobile,
            "Format: Animate <action> <frameCount> <repeatCount> <forward> <repeat> <delay>",
        );
    }
}
