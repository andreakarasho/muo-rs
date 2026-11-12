//! The chat system: `ChatSystem`, `ChatCommand`, `ChatPackets` (0xB5 open window, 0xB3 chat action, 0xB2 out),
//! `ChatUser`, `Channel` and `ChatActionHandlers`.
//!
//! A `ChatUser` and a `Channel` are ids into [`ChatState`] (instead of referencing each other).

use indexmap::IndexMap;
use muo_io::{SpanReader, SpanWriter};
use muo_net::net_state::NetState;
use muo_primitives::dotnet::IgnoreCase;
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part1::props_b;
use tracing::info;

/// `ChatCommand`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum ChatCommand {
    AddChannel = 0x3E8,
    RemoveChannel = 0x3E9,
    AskNewNickname = 0x3EB,
    CloseChatWindow = 0x3EC,
    OpenChatWindow = 0x3ED,
    AddUserToChannel = 0x3EE,
    RemoveUserFromChannel = 0x3EF,
    LeaveChannel = 0x3F0,
    JoinedChannel = 0x3F1,
}

pub const NORMAL_COLOR_CHARACTER: char = '0';
pub const MODERATOR_COLOR_CHARACTER: char = '1';
pub const VOICED_COLOR_CHARACTER: char = '2';

pub type UserId = u32;
pub type ChannelId = u32;

/// `ChatUser`.
#[derive(Debug)]
pub struct ChatUser {
    pub mobile: MobileId,
    pub ignored: Vec<UserId>,
    pub ignoring: Vec<UserId>,
    pub username: String,
    pub current_channel: Option<ChannelId>,
    pub anonymous: bool,
    pub ignore_private_message: bool,
}

/// `Channel`.
#[derive(Debug)]
pub struct Channel {
    name: String,
    password: Option<String>,
    voice_restricted: bool,
    pub always_available: bool,
    users: Vec<UserId>,
    banned: Vec<UserId>,
    moderators: Vec<UserId>,
    voices: Vec<UserId>,
}

/// The statics of `ChatSystem`, `ChatUser` and `Channel`.
#[derive(Default)]
pub struct ChatState {
    pub enabled: bool,
    users: IndexMap<UserId, ChatUser>,
    by_mobile: IndexMap<MobileId, UserId>,
    channels: IndexMap<ChannelId, Channel>,
    next_user: u32,
    next_channel: u32,
}

fn st(w: &mut World) -> &mut ChatState {
    w.resources.get_mut::<ChatState>()
}

fn st_ref(w: &World) -> Option<&ChatState> {
    w.resources.get::<ChatState>()
}

/// `ChatSystem.Enabled`.
pub fn enabled(w: &World) -> bool {
    st_ref(w).is_some_and(|s| s.enabled)
}

// ---- packets -----------------------------------------------------------------------------------------------------

/// The 0xB2 packet of `SendChatMessage`.
pub fn chat_message_packet(lang: &str, number: i32, param1: &str, param2: &str) -> Vec<u8> {
    let len1 = param1.encode_utf16().count();
    let len2 = param2.encode_utf16().count();
    let length = 13 + (len1 + len2) * 2;
    let mut p = SpanWriter::new(length, false);
    let _ = p.write_u8(0xB2);
    let _ = p.write_u16(length as u16);
    let _ = p.write_u16((number - 20) as u16);
    let _ = p.write_ascii_fixed(lang, 4);
    let _ = p.write_big_uni_null(param1);
    let _ = p.write_big_uni_null(param2);
    p.span().to_vec()
}

fn send_chat_message(
    w: &mut World,
    mobile: MobileId,
    lang: Option<&str>,
    number: i32,
    p1: Option<&str>,
    p2: Option<&str>,
) {
    let Some(ns) = props_b::net_state(w, mobile) else {
        return;
    };
    if w.cannot_send_packets(ns) {
        return;
    }
    let p = chat_message_packet(
        lang.unwrap_or(""),
        number,
        p1.unwrap_or(""),
        p2.unwrap_or(""),
    );
    w.send(ns, &p);
}

/// `ChatSystem.SendCommandTo(to, type, param1, param2)`.
pub fn send_command_to(
    w: &mut World,
    to: MobileId,
    ty: ChatCommand,
    p1: Option<&str>,
    p2: Option<&str>,
) {
    send_chat_message(w, to, None, ty as i32 + 20, p1, p2);
}

/// `ChatSystem.Configure`.
pub fn configure(w: &mut World) {
    let e = w
        .config
        .get_or_update_setting("chat.enabled", false)
        .unwrap_or(false);
    st(w).enabled = e;
    muo_content::network::register_packet(w, 0xB5, 0x40, true, false, open_chat_window_request);
    muo_content::network::register_packet(w, 0xB3, 0, true, false, chat_action);
}

/// `Channel.Initialize`.
pub fn initialize(w: &mut World) {
    add_static_channel(w, "Newbie Help");
}

fn open_chat_window_request(w: &mut World, ns: &mut NetState, _body: &[u8]) {
    let id = muo_server::net_glue::ensure_net_state(w, ns);
    let Some(from) = w.net_state(id).and_then(|d| d.mobile) else {
        return;
    };
    if !enabled(w) {
        muo_content::mx::send_message(w, from, "The chat system has been disabled.");
        return;
    }
    // Newer clients don't send chat username anymore so we are ignoring the rest of this packet.
    let chat_name = match w.mobile(from).and_then(|m| m.name.clone()) {
        Some(n) => n,
        None => format!(
            "Unknown User {}",
            muo_primitives::utility::random_min_max(&mut w.rng, 1000000, 9999999)
        ),
    };
    send_command_to(w, from, ChatCommand::OpenChatWindow, Some(&chat_name), None);
    add_chat_user(w, from, &chat_name);
}

fn chat_action(w: &mut World, ns: &mut NetState, body: &[u8]) {
    if !enabled(w) {
        return;
    }
    let id = muo_server::net_glue::ensure_net_state(w, ns);
    let Some(from) = w.net_state(id).and_then(|d| d.mobile) else {
        return;
    };
    let Some(user) = get_chat_user(w, from) else {
        return;
    };
    let mut r = SpanReader::new(body);
    let (Ok(_lang), Ok(action_id)) = (r.read_ascii_safe(Some(4)), r.read_i16()) else {
        return;
    };
    let Ok(param) = r.read_big_uni_safe(None) else {
        return;
    };

    let Some(handler) = get_handler(i32::from(action_id)) else {
        info!("Unknown chat action 0x{action_id:X}: {param}");
        return;
    };
    let channel = st_ref(w)
        .and_then(|s| s.users.get(&user))
        .and_then(|u| u.current_channel);
    if handler.require_conference && channel.is_none() {
        // You must be in a conference to do this. To join a conference, select one from the Conference menu.
        user_send_message(w, user, 31, None, None);
        return;
    }
    if handler.require_moderator && !user_is_moderator(w, user) {
        user_send_message(w, user, 29, None, None); // You must have operator status to do this.
        return;
    }
    (handler.callback)(w, user, channel, &param);
}

// ---- ChatUser ----------------------------------------------------------------------------------------------------

/// `ChatUser.GetChatUser(from)`.
pub fn get_chat_user(w: &World, from: MobileId) -> Option<UserId> {
    st_ref(w)?.by_mobile.get(&from).copied()
}

/// `ChatUser.GetChatUser(username)`.
pub fn get_chat_user_by_name(w: &World, username: &str) -> Option<UserId> {
    st_ref(w)?
        .users
        .iter()
        .find(|(_, u)| u.username == username)
        .map(|(i, _)| *i)
}

fn user(w: &World, u: UserId) -> Option<&ChatUser> {
    st_ref(w)?.users.get(&u)
}

fn user_mut(w: &mut World, u: UserId) -> Option<&mut ChatUser> {
    st(w).users.get_mut(&u)
}

pub fn user_is_online(w: &World, u: UserId) -> bool {
    user(w, u).is_some_and(|x| props_b::net_state(w, x.mobile).is_some())
}

pub fn user_is_moderator(w: &World, u: UserId) -> bool {
    user(w, u)
        .and_then(|x| x.current_channel)
        .is_some_and(|c| channel_is_moderator(w, c, u))
}

pub fn user_color_character(w: &World, u: UserId) -> char {
    if user_is_moderator(w, u) {
        MODERATOR_COLOR_CHARACTER
    } else if user(w, u)
        .and_then(|x| x.current_channel)
        .is_some_and(|c| channel_is_voiced(w, c, u))
    {
        VOICED_COLOR_CHARACTER
    } else {
        NORMAL_COLOR_CHARACTER
    }
}

fn username(w: &World, u: UserId) -> String {
    user(w, u).map(|x| x.username.clone()).unwrap_or_default()
}

/// `CheckOnline()`: false (after removing the user) when the player is gone.
pub fn user_check_online(w: &mut World, u: UserId) -> bool {
    if user_is_online(w, u) {
        return true;
    }
    remove_chat_user(w, u);
    false
}

/// `ChatUser.SendMessage(number, param1, param2)`.
pub fn user_send_message(
    w: &mut World,
    u: UserId,
    number: i32,
    p1: Option<&str>,
    p2: Option<&str>,
) {
    if let Some(m) = user(w, u).map(|x| x.mobile) {
        user_send_message_from(w, u, number, m, p1, p2);
    }
}

/// `ChatUser.SendMessage(number, from, param1, param2)`.
pub fn user_send_message_from(
    w: &mut World,
    u: UserId,
    number: i32,
    from: MobileId,
    p1: Option<&str>,
    p2: Option<&str>,
) {
    let Some(m) = user(w, u).map(|x| x.mobile) else {
        return;
    };
    let lang = w.mobile(from).and_then(|x| x.language.clone());
    send_chat_message(w, m, lang.as_deref(), number, p1, p2);
}

fn is_ignored(w: &World, u: UserId, check: UserId) -> bool {
    user(w, u).is_some_and(|x| x.ignored.contains(&check))
}

pub fn add_ignored(w: &mut World, u: UserId, target: UserId) {
    let name = username(w, target);
    if is_ignored(w, u, target) {
        user_send_message(w, u, 22, Some(&name), None); // You are already ignoring %1.
    } else {
        if let Some(x) = user_mut(w, u) {
            x.ignored.push(target);
        }
        if let Some(x) = user_mut(w, target) {
            x.ignoring.push(u);
        }
        user_send_message(w, u, 23, Some(&name), None); // You are now ignoring %1.
    }
}

pub fn remove_ignored(w: &mut World, u: UserId, target: UserId) {
    let name = username(w, target);
    if is_ignored(w, u, target) {
        if let Some(x) = user_mut(w, u) {
            x.ignored.retain(|&i| i != target);
        }
        if let Some(x) = user_mut(w, target) {
            x.ignoring.retain(|&i| i != u);
        }
        user_send_message(w, u, 24, Some(&name), None); // You are no longer ignoring %1.
        if user(w, u).is_some_and(|x| x.ignored.is_empty()) {
            user_send_message(w, u, 26, None, None); // You are no longer ignoring anyone.
        }
    } else {
        user_send_message(w, u, 25, Some(&name), None); // You are not ignoring %1.
    }
}

/// `ChatUser.AddChatUser(from, username)`.
pub fn add_chat_user(w: &mut World, from: MobileId, username: &str) -> UserId {
    if let Some(u) = get_chat_user(w, from) {
        return u;
    }
    let s = st(w);
    s.next_user += 1;
    let id = s.next_user;
    s.users.insert(
        id,
        ChatUser {
            mobile: from,
            ignored: Vec::new(),
            ignoring: Vec::new(),
            username: username.to_owned(),
            current_channel: None,
            anonymous: false,
            ignore_private_message: false,
        },
    );
    s.by_mobile.insert(from, id);

    send_channels_to(w, id);
    let channels: Vec<ChannelId> = st(w).channels.keys().copied().collect();
    for c in channels {
        if channel_add_user(w, c, id, None) {
            break;
        }
    }
    id
}

/// `ChatUser.RemoveChatUser(user)`.
pub fn remove_chat_user(w: &mut World, u: UserId) {
    if user(w, u).is_none() {
        return;
    }
    // `for (i = 0; i < user.Ignoring.Count; ++i) user.Ignoring[i].RemoveIgnored(user)`: the list shrinks as it
    // goes, so every other entry is skipped
    let mut i = 0;
    while let Some(&other) = user(w, u).and_then(|x| x.ignoring.get(i)) {
        remove_ignored(w, other, u);
        i += 1;
    }
    let (mobile, channel) = match user(w, u) {
        Some(x) => (x.mobile, x.current_channel),
        None => return,
    };
    // `m_Users.Remove(user)`
    if st(w).users.contains_key(&u) {
        send_command_to(w, mobile, ChatCommand::CloseChatWindow, None, None);
        if let Some(c) = channel {
            channel_remove_user(w, c, u);
        }
        let s = st(w);
        s.by_mobile.shift_remove(&mobile);
        s.users.shift_remove(&u);
    }
}

/// `ChatUser.GlobalSendCommand(command, initiator, param1, param2)`.
pub fn global_send_command(
    w: &mut World,
    command: ChatCommand,
    initiator: Option<UserId>,
    p1: Option<&str>,
    p2: Option<&str>,
) {
    let mut i = 0;
    while let Some(&u) = st_ref(w).and_then(|s| s.users.get_index(i)).map(|(k, _)| k) {
        i += 1;
        if Some(u) == initiator {
            continue;
        }
        // an offline user is removed by CheckOnline and the `for` loop then skips the one that shifted into its place
        if user_check_online(w, u)
            && let Some(m) = user(w, u).map(|x| x.mobile)
        {
            send_command_to(w, m, command, p1, p2);
        }
    }
}

/// `ChatSystem.SearchForUser(from, name)`.
pub fn search_for_user(w: &mut World, from: UserId, name: &str) -> Option<UserId> {
    let u = get_chat_user_by_name(w, name);
    if u.is_none() {
        user_send_message(w, from, 32, Some(name), None); // There is no player named '%1'.
    }
    u
}

// ---- Channel -----------------------------------------------------------------------------------------------------

fn chan(w: &World, c: ChannelId) -> Option<&Channel> {
    st_ref(w)?.channels.get(&c)
}

fn chan_mut(w: &mut World, c: ChannelId) -> Option<&mut Channel> {
    st(w).channels.get_mut(&c)
}

pub fn channel_name(w: &World, c: ChannelId) -> String {
    chan(w, c).map(|x| x.name.clone()).unwrap_or_default()
}

pub fn channel_contains(w: &World, c: ChannelId, u: UserId) -> bool {
    chan(w, c).is_some_and(|x| x.users.contains(&u))
}
fn channel_is_banned(w: &World, c: ChannelId, u: UserId) -> bool {
    chan(w, c).is_some_and(|x| x.banned.contains(&u))
}
pub fn channel_is_moderator(w: &World, c: ChannelId, u: UserId) -> bool {
    chan(w, c).is_some_and(|x| x.moderators.contains(&u))
}
pub fn channel_is_voiced(w: &World, c: ChannelId, u: UserId) -> bool {
    chan(w, c).is_some_and(|x| x.voices.contains(&u))
}
/// `CanTalk`.
pub fn channel_can_talk(w: &World, c: ChannelId, u: UserId) -> bool {
    chan(w, c)
        .is_some_and(|x| !x.voice_restricted || x.voices.contains(&u) || x.moderators.contains(&u))
}

/// `Password` setter: trimmed, empty is null.
pub fn channel_set_password(w: &mut World, c: ChannelId, value: &str) {
    if let Some(x) = chan_mut(w, c) {
        let t = value.trim();
        x.password = (!t.is_empty()).then(|| t.to_owned());
    }
}

/// `Name` setter.
pub fn channel_rename(w: &mut World, c: ChannelId, value: &str) {
    let old = channel_name(w, c);
    channel_send_command(w, c, ChatCommand::RemoveChannel, None, Some(&old), None);
    if let Some(x) = chan_mut(w, c) {
        x.name = value.to_owned();
    }
    channel_send_command(w, c, ChatCommand::AddChannel, None, Some(value), None);
    channel_send_command(w, c, ChatCommand::JoinedChannel, None, Some(value), None);
}

/// `VoiceRestricted` setter.
pub fn channel_set_voice_restricted(w: &mut World, c: ChannelId, value: bool) {
    if let Some(x) = chan_mut(w, c) {
        x.voice_restricted = value;
    }
    // restricted: only moderators speak by default (56); otherwise everyone does (55)
    channel_send_message(w, c, if value { 56 } else { 55 }, None, None, None);
}

pub fn channel_voice_restricted(w: &World, c: ChannelId) -> bool {
    chan(w, c).is_some_and(|x| x.voice_restricted)
}

/// `ValidatePassword`.
fn validate_password(w: &World, c: ChannelId, password: Option<&str>) -> bool {
    match chan(w, c).and_then(|x| x.password.as_deref()) {
        None => true,
        Some(p) => password.is_some_and(|q| p.eq_ic(q)),
    }
}

/// `ValidateModerator(user)`.
fn validate_moderator(w: &mut World, c: ChannelId, u: Option<UserId>) -> bool {
    if let Some(u) = u
        && !channel_is_moderator(w, c, u)
    {
        user_send_message(w, u, 29, None, None); // You must have operator status to do this.
        return false;
    }
    true
}

fn access_of(w: &World, u: UserId) -> AccessLevel {
    user(w, u).map_or(AccessLevel::Player, |x| {
        muo_content::mx::access_level(w, x.mobile)
    })
}

/// `ValidateAccess(from, target)`.
fn validate_access(w: &mut World, from: Option<UserId>, target: UserId) -> bool {
    let Some(f) = from else { return true };
    if access_of(w, f) >= access_of(w, target) {
        return true;
    }
    if let Some(m) = user(w, f).map(|x| x.mobile) {
        muo_content::mx::send_message(w, m, "Your access level is too low to do this.");
    }
    false
}

/// `AddUser(user, password)`.
pub fn channel_add_user(w: &mut World, c: ChannelId, u: UserId, password: Option<&str>) -> bool {
    let name = channel_name(w, c);
    if channel_contains(w, c, u) {
        user_send_message(w, u, 46, Some(&name), None); // You are already in the conference '%1'.
        return true;
    }
    if channel_is_banned(w, c, u) {
        user_send_message(w, u, 64, None, None); // You have been banned from this conference.
        return false;
    }
    if !validate_password(w, c, password) {
        user_send_message(w, u, 34, None, None); // That is not the correct password.
        return false;
    }
    if let Some(cur) = user(w, u).and_then(|x| x.current_channel) {
        channel_remove_user(w, cur, u); // Remove them from their current channel first
    }
    let Some(mobile) = user(w, u).map(|x| x.mobile) else {
        return false;
    };
    send_command_to(w, mobile, ChatCommand::JoinedChannel, Some(&name), None);
    let colored = format!("{}{}", user_color_character(w, u), username(w, u));
    channel_send_command(
        w,
        c,
        ChatCommand::AddUserToChannel,
        None,
        Some(&colored),
        None,
    );

    if let Some(x) = chan_mut(w, c) {
        x.users.push(u);
    }
    if let Some(x) = user_mut(w, u) {
        x.current_channel = Some(c);
    }
    let always = chan(w, c).is_some_and(|x| x.always_available);
    let count = chan(w, c).map_or(0, |x| x.users.len());
    if muo_content::mx::access_level(w, mobile) >= AccessLevel::GameMaster
        || (!always && count == 1)
    {
        channel_add_moderator(w, c, u, None);
    }
    channel_send_users_to(w, c, u);
    true
}

/// `RemoveUser(user)`.
pub fn channel_remove_user(w: &mut World, c: ChannelId, u: UserId) {
    if !channel_contains(w, c, u) {
        return;
    }
    if let Some(x) = chan_mut(w, c) {
        x.users.retain(|&i| i != u);
        x.moderators.retain(|&i| i != u);
        x.voices.retain(|&i| i != u);
    }
    if let Some(x) = user_mut(w, u) {
        x.current_channel = None;
    }
    let name = username(w, u);
    channel_send_command(
        w,
        c,
        ChatCommand::RemoveUserFromChannel,
        Some(u),
        Some(&name),
        None,
    );
    if let Some(m) = user(w, u).map(|x| x.mobile) {
        send_command_to(w, m, ChatCommand::LeaveChannel, None, None);
    }
    let empty_and_closing = chan(w, c).is_some_and(|x| x.users.is_empty() && !x.always_available);
    if empty_and_closing {
        remove_channel(w, c);
    }
}

pub fn channel_add_ban(w: &mut World, c: ChannelId, u: UserId, moderator: Option<UserId>) {
    if !validate_moderator(w, c, moderator) || !validate_access(w, moderator, u) {
        return;
    }
    if let Some(x) = chan_mut(w, c)
        && !x.banned.contains(&u)
    {
        x.banned.push(u);
    }
    channel_kick(w, c, u, moderator, true);
}

pub fn channel_remove_ban(w: &mut World, c: ChannelId, u: UserId) {
    if let Some(x) = chan_mut(w, c) {
        x.banned.retain(|&i| i != u);
    }
}

/// `Kick(user, moderator, wasBanned)`.
pub fn channel_kick(
    w: &mut World,
    c: ChannelId,
    u: UserId,
    moderator: Option<UserId>,
    was_banned: bool,
) {
    if !validate_moderator(w, c, moderator) || !validate_access(w, moderator, u) {
        return;
    }
    let uname = username(w, u);
    if channel_contains(w, c, u) {
        if let Some(m) = moderator {
            let mname = username(w, m);
            // %1, a conference moderator, has banned (63) / kicked (45) you out of the conference.
            user_send_message(w, u, if was_banned { 63 } else { 45 }, Some(&mname), None);
        }
        channel_remove_user(w, c, u);
        if let Some(m) = user(w, u).map(|x| x.mobile) {
            let colored = format!("{}{}", user_color_character(w, u), uname);
            send_command_to(w, m, ChatCommand::AddUserToChannel, Some(&colored), None);
        }
        channel_send_message(w, c, 44, None, Some(&uname), None); // %1 has been kicked out of the conference.
    }
    if was_banned && let Some(m) = moderator {
        user_send_message(w, m, 62, Some(&uname), None); // You are banning %1 from this conference.
    }
}

pub fn channel_add_voiced(w: &mut World, c: ChannelId, u: UserId, moderator: Option<UserId>) {
    if !validate_moderator(w, c, moderator) {
        return;
    }
    if !channel_is_banned(w, c, u) && !channel_is_moderator(w, c, u) && !channel_is_voiced(w, c, u)
    {
        if let Some(x) = chan_mut(w, c) {
            x.voices.push(u);
        }
        let name = username(w, u);
        if let Some(m) = moderator {
            let mname = username(w, m);
            // %1, a conference moderator, has granted you speaking privileges in this conference.
            user_send_message(w, u, 54, Some(&mname), None);
        }
        channel_send_message(w, c, 52, Some(u), Some(&name), None); // %1 now has speaking privileges...
        let colored = format!("{}{}", user_color_character(w, u), name);
        channel_send_command(
            w,
            c,
            ChatCommand::AddUserToChannel,
            Some(u),
            Some(&colored),
            None,
        );
    }
}

pub fn channel_remove_voiced(w: &mut World, c: ChannelId, u: UserId, moderator: Option<UserId>) {
    if !validate_moderator(w, c, moderator) || !validate_access(w, moderator, u) {
        return;
    }
    if !channel_is_moderator(w, c, u) && channel_is_voiced(w, c, u) {
        if let Some(x) = chan_mut(w, c) {
            x.voices.retain(|&i| i != u);
        }
        let name = username(w, u);
        if let Some(m) = moderator {
            let mname = username(w, m);
            // %1, a conference moderator, has removed your speaking privileges for this conference.
            user_send_message(w, u, 53, Some(&mname), None);
        }
        channel_send_message(w, c, 51, Some(u), Some(&name), None); // %1 no longer has speaking privileges...
        let colored = format!("{}{}", user_color_character(w, u), name);
        channel_send_command(
            w,
            c,
            ChatCommand::AddUserToChannel,
            Some(u),
            Some(&colored),
            None,
        );
    }
}

pub fn channel_add_moderator(w: &mut World, c: ChannelId, u: UserId, moderator: Option<UserId>) {
    if !validate_moderator(w, c, moderator) {
        return;
    }
    if channel_is_banned(w, c, u) || channel_is_moderator(w, c, u) {
        return;
    }
    if let Some(x) = chan_mut(w, c) {
        x.voices.retain(|&i| i != u);
        x.moderators.push(u);
    }
    let name = username(w, u);
    if let Some(m) = moderator {
        let mname = username(w, m);
        user_send_message(w, u, 50, Some(&mname), None); // %1 has made you a conference moderator.
    }
    channel_send_message(w, c, 48, Some(u), Some(&name), None); // %1 is now a conference moderator.
    let colored = format!("{}{}", user_color_character(w, u), name);
    channel_send_command(
        w,
        c,
        ChatCommand::AddUserToChannel,
        None,
        Some(&colored),
        None,
    );
}

pub fn channel_remove_moderator(w: &mut World, c: ChannelId, u: UserId, moderator: Option<UserId>) {
    if !validate_moderator(w, c, moderator) || !validate_access(w, moderator, u) {
        return;
    }
    if channel_is_moderator(w, c, u) {
        if let Some(x) = chan_mut(w, c) {
            x.moderators.retain(|&i| i != u);
        }
        let name = username(w, u);
        if let Some(m) = moderator {
            let mname = username(w, m);
            // %1 has removed you from the list of conference moderators.
            user_send_message(w, u, 49, Some(&mname), None);
        }
        channel_send_message(w, c, 47, Some(u), Some(&name), None); // %1 is no longer a conference moderator.
        let colored = format!("{}{}", user_color_character(w, u), name);
        channel_send_command(
            w,
            c,
            ChatCommand::AddUserToChannel,
            None,
            Some(&colored),
            None,
        );
    }
}

/// The `for (i = 0; i < m_Users.Count; ++i)` walk of the channel senders: `f(user)` for each user except the one
/// `skip` names; an offline user is dropped by `CheckOnline`, and the index steps back when that removed it.
fn for_each_user(
    w: &mut World,
    c: ChannelId,
    skip: impl Fn(&World, UserId) -> bool,
    check_online: bool,
    mut f: impl FnMut(&mut World, UserId),
) {
    let mut i = 0;
    while let Some(u) = chan(w, c).and_then(|x| x.users.get(i)).copied() {
        if skip(w, u) {
            i += 1;
            continue;
        }
        let ok = !check_online || user_check_online(w, u);
        if ok {
            f(w, u);
            i += 1;
        } else if channel_contains(w, c, u) {
            i += 1;
        }
        // else: the user left the list, the next one is now at `i`
    }
}

/// `SendMessage(number, initiator, param1, param2)`.
pub fn channel_send_message(
    w: &mut World,
    c: ChannelId,
    number: i32,
    initiator: Option<UserId>,
    p1: Option<&str>,
    p2: Option<&str>,
) {
    for_each_user(
        w,
        c,
        |_, u| Some(u) == initiator,
        true,
        |w, u| user_send_message(w, u, number, p1, p2),
    );
}

/// `SendIgnorableMessage(number, from, param1, param2)`.
pub fn channel_send_ignorable_message(
    w: &mut World,
    c: ChannelId,
    number: i32,
    from: UserId,
    p1: &str,
    p2: &str,
) {
    let fm = user(w, from).map(|x| x.mobile);
    let Some(fm) = fm else { return };
    for_each_user(
        w,
        c,
        move |w, u| is_ignored(w, u, from),
        true,
        |w, u| user_send_message_from(w, u, number, fm, Some(p1), Some(p2)),
    );
}

/// `SendCommand(command, initiator, param1, param2)`.
pub fn channel_send_command(
    w: &mut World,
    c: ChannelId,
    command: ChatCommand,
    initiator: Option<UserId>,
    p1: Option<&str>,
    p2: Option<&str>,
) {
    for_each_user(
        w,
        c,
        |_, u| Some(u) == initiator,
        true,
        |w, u| {
            if let Some(m) = user(w, u).map(|x| x.mobile) {
                send_command_to(w, m, command, p1, p2);
            }
        },
    );
}

/// `SendUsersTo(to)`.
pub fn channel_send_users_to(w: &mut World, c: ChannelId, to: UserId) {
    let users: Vec<UserId> = chan(w, c).map(|x| x.users.clone()).unwrap_or_default();
    let Some(tm) = user(w, to).map(|x| x.mobile) else {
        return;
    };
    for u in users {
        let colored = format!("{}{}", user_color_character(w, u), username(w, u));
        send_command_to(w, tm, ChatCommand::AddUserToChannel, Some(&colored), None);
    }
}

/// `Channel.SendChannelsTo(user)`.
pub fn send_channels_to(w: &mut World, u: UserId) {
    let Some(m) = user(w, u).map(|x| x.mobile) else {
        return;
    };
    let list: Vec<(ChannelId, String)> = st(w)
        .channels
        .iter()
        .map(|(k, c)| (*k, c.name.clone()))
        .collect();
    for (c, name) in list {
        if !channel_is_banned(w, c, u) {
            send_command_to(w, m, ChatCommand::AddChannel, Some(&name), Some("0"));
        }
    }
}

/// `Channel.AddChannel(name, password)`.
pub fn add_channel(w: &mut World, name: &str, password: Option<&str>) -> ChannelId {
    let c = match find_channel_by_name(w, name) {
        Some(c) => c,
        None => {
            let s = st(w);
            s.next_channel += 1;
            let id = s.next_channel;
            s.channels.insert(
                id,
                Channel {
                    name: name.to_owned(),
                    password: password.map(str::to_owned),
                    voice_restricted: false,
                    always_available: false,
                    users: Vec::new(),
                    banned: Vec::new(),
                    moderators: Vec::new(),
                    voices: Vec::new(),
                },
            );
            id
        }
    };
    global_send_command(w, ChatCommand::AddChannel, None, Some(name), Some("0"));
    c
}

/// `Channel.RemoveChannel(channel)`.
pub fn remove_channel(w: &mut World, c: ChannelId) {
    let Some(ch) = chan(w, c) else { return };
    if ch.users.is_empty() {
        let name = ch.name.clone();
        global_send_command(w, ChatCommand::RemoveChannel, None, Some(&name), None);
        st(w).channels.shift_remove(&c);
    }
}

pub fn find_channel_by_name(w: &World, name: &str) -> Option<ChannelId> {
    st_ref(w)?
        .channels
        .iter()
        .find(|(_, c)| c.name == name)
        .map(|(k, _)| *k)
}

/// `Channel.AddStaticChannel(name)`.
pub fn add_static_channel(w: &mut World, name: &str) {
    let c = add_channel(w, name, None);
    if let Some(x) = chan_mut(w, c) {
        x.always_available = true;
    }
}

// ---- ChatActionHandlers --------------------------------------------------------------------------------------------

pub type OnChatAction = fn(&mut World, UserId, Option<ChannelId>, &str);

/// `ChatActionHandler`.
pub struct ChatActionHandler {
    pub require_moderator: bool,
    pub require_conference: bool,
    pub callback: OnChatAction,
}

/// `ChatActionHandlers.GetHandler(actionID)`: the built-in table (`Register` is only called by its own
/// static constructor).
pub fn get_handler(action_id: i32) -> Option<ChatActionHandler> {
    let h = |m: bool, c: bool, callback: OnChatAction| {
        Some(ChatActionHandler {
            require_moderator: m,
            require_conference: c,
            callback,
        })
    };
    match action_id {
        0x41 => h(true, true, change_channel_password),
        0x58 => h(false, false, leave_chat),
        0x61 => h(false, true, channel_message),
        0x62 => h(false, false, join_channel),
        0x63 => h(false, false, join_new_channel),
        0x64 => h(true, true, rename_channel),
        0x65 => h(false, false, private_message),
        0x66 => h(false, false, add_ignore),
        0x67 => h(false, false, remove_ignore),
        0x68 => h(false, false, toggle_ignore),
        0x69 => h(true, true, add_voice),
        0x6A => h(true, true, remove_voice),
        0x6B => h(true, true, toggle_voice),
        0x6C => h(true, true, add_moderator),
        0x6D => h(true, true, remove_moderator),
        0x6E => h(true, true, toggle_moderator),
        0x6F => h(false, false, allow_private_messages),
        0x70 => h(false, false, disallow_private_messages),
        0x71 => h(false, false, toggle_private_messages),
        0x72 => h(false, false, show_character_name),
        0x73 => h(false, false, hide_character_name),
        0x74 => h(false, false, toggle_character_name),
        0x75 => h(false, false, query_who_is),
        0x76 => h(true, true, kick),
        0x77 => h(true, true, enable_default_voice),
        0x78 => h(true, true, disable_default_voice),
        0x79 => h(true, true, toggle_default_voice),
        0x7A => h(false, true, emote_message),
        _ => None,
    }
}

fn channel_message(w: &mut World, from: UserId, c: Option<ChannelId>, param: &str) {
    let Some(c) = c else { return };
    if channel_can_talk(w, c, from) {
        let name = format!("{}{}", user_color_character(w, from), username(w, from));
        channel_send_ignorable_message(w, c, 57, from, &name, param); // %1: %2
    } else {
        user_send_message(w, from, 36, None, None); // The moderator of this conference has not given you speaking privileges.
    }
}

fn emote_message(w: &mut World, from: UserId, c: Option<ChannelId>, param: &str) {
    let Some(c) = c else { return };
    if channel_can_talk(w, c, from) {
        let name = format!("{}{}", user_color_character(w, from), username(w, from));
        channel_send_ignorable_message(w, c, 58, from, &name, param); // %1 %2
    } else {
        user_send_message(w, from, 36, None, None);
    }
}

fn private_message(w: &mut World, from: UserId, _c: Option<ChannelId>, param: &str) {
    // `param[..IndexOf(' ')]` throws without a space; the caller's catch only prints it
    let Some(i) = param.find(' ') else {
        info!("chat: private message without a space: {param}");
        return;
    };
    let (name, text) = (&param[..i], &param[i + 1..]);
    let Some(target) = search_for_user(w, from, name) else {
        return;
    };
    let tname = username(w, target);
    if is_ignored(w, target, from) {
        user_send_message(w, from, 35, Some(&tname), None); // %1 has chosen to ignore you...
    } else if user(w, target).is_some_and(|x| x.ignore_private_message) {
        user_send_message(w, from, 42, Some(&tname), None); // %1 has chosen to not receive private messages...
    } else if let Some(fm) = user(w, from).map(|x| x.mobile) {
        let sender = format!("{}{}", user_color_character(w, from), username(w, from));
        user_send_message_from(w, target, 59, fm, Some(&sender), Some(text)); // [%1]: %2
    }
}

fn leave_chat(w: &mut World, from: UserId, _c: Option<ChannelId>, _p: &str) {
    remove_chat_user(w, from);
}

fn change_channel_password(w: &mut World, from: UserId, c: Option<ChannelId>, param: &str) {
    if let Some(c) = c {
        channel_set_password(w, c, param);
    }
    user_send_message(w, from, 60, None, None); // The password to the conference has been changed.
}

fn set_ignore_pm(w: &mut World, from: UserId, ignore: bool) {
    if let Some(x) = user_mut(w, from) {
        x.ignore_private_message = ignore;
    }
}

fn allow_private_messages(w: &mut World, from: UserId, _c: Option<ChannelId>, _p: &str) {
    set_ignore_pm(w, from, false);
    user_send_message(w, from, 37, None, None); // You can now receive private messages.
}

fn disallow_private_messages(w: &mut World, from: UserId, _c: Option<ChannelId>, _p: &str) {
    set_ignore_pm(w, from, true);
    user_send_message(w, from, 38, None, None); // You will no longer receive private messages...
}

fn toggle_private_messages(w: &mut World, from: UserId, _c: Option<ChannelId>, _p: &str) {
    let now = !user(w, from).is_some_and(|x| x.ignore_private_message);
    set_ignore_pm(w, from, now);
    user_send_message(w, from, if now { 38 } else { 37 }, None, None);
}

fn set_anonymous(w: &mut World, from: UserId, v: bool) {
    if let Some(x) = user_mut(w, from) {
        x.anonymous = v;
    }
}

fn show_character_name(w: &mut World, from: UserId, _c: Option<ChannelId>, _p: &str) {
    set_anonymous(w, from, false);
    user_send_message(w, from, 39, None, None); // You are now showing your character name...
}

fn hide_character_name(w: &mut World, from: UserId, _c: Option<ChannelId>, _p: &str) {
    set_anonymous(w, from, true);
    user_send_message(w, from, 40, None, None); // You are no longer showing your character name...
}

fn toggle_character_name(w: &mut World, from: UserId, _c: Option<ChannelId>, _p: &str) {
    let now = !user(w, from).is_some_and(|x| x.anonymous);
    set_anonymous(w, from, now);
    user_send_message(w, from, if now { 40 } else { 39 }, None, None);
}

fn default_if_empty(s: Option<&str>) -> Option<String> {
    s.map(str::trim)
        .filter(|t| !t.is_empty())
        .map(str::to_owned)
}

fn join_channel(w: &mut World, from: UserId, _c: Option<ChannelId>, param: &str) {
    let (name, password): (String, Option<String>);
    if let Some(start) = param.find('"') {
        let start = start + 1;
        if let Some(end) = param[start..].find('"').map(|e| e + start) {
            name = param[start..end].to_owned();
            password = Some(param[end + 1..].to_owned());
        } else {
            name = param[start..].to_owned();
            password = None;
        }
    } else if let Some(i) = param.find(' ') {
        name = param[..i].to_owned();
        password = Some(param[i + 1..].to_owned());
    } else {
        name = param.to_owned();
        password = None;
    }
    let password = default_if_empty(password.as_deref());
    match find_channel_by_name(w, &name) {
        None => user_send_message(w, from, 33, Some(&name), None), // There is no conference named '%1'.
        Some(c) => {
            channel_add_user(w, c, from, password.as_deref());
        }
    }
}

fn join_new_channel(w: &mut World, from: UserId, _c: Option<ChannelId>, param: &str) {
    let param = param.trim();
    if param.is_empty() {
        return;
    }
    let (name, password): (String, Option<String>) = match param.find('{') {
        Some(start) => {
            let name = param[..start].to_owned();
            let rest = &param[start + 1..];
            (name, rest.find('}').map(|e| rest[..e].to_owned()))
        }
        None => (param.to_owned(), None),
    };
    let password = default_if_empty(password.as_deref());
    let c = add_channel(w, &name, password.as_deref());
    channel_add_user(w, c, from, password.as_deref());
}

fn add_ignore(w: &mut World, from: UserId, _c: Option<ChannelId>, param: &str) {
    if let Some(t) = search_for_user(w, from, param) {
        add_ignored(w, from, t);
    }
}

fn remove_ignore(w: &mut World, from: UserId, _c: Option<ChannelId>, param: &str) {
    if let Some(t) = search_for_user(w, from, param) {
        remove_ignored(w, from, t);
    }
}

fn toggle_ignore(w: &mut World, from: UserId, _c: Option<ChannelId>, param: &str) {
    let Some(t) = search_for_user(w, from, param) else {
        return;
    };
    if is_ignored(w, from, t) {
        remove_ignored(w, from, t);
    } else {
        add_ignored(w, from, t);
    }
}

fn add_voice(w: &mut World, from: UserId, c: Option<ChannelId>, param: &str) {
    if let (Some(c), Some(t)) = (c, search_for_user(w, from, param)) {
        channel_add_voiced(w, c, t, Some(from));
    }
}

fn remove_voice(w: &mut World, from: UserId, c: Option<ChannelId>, param: &str) {
    if let (Some(c), Some(t)) = (c, search_for_user(w, from, param)) {
        channel_remove_voiced(w, c, t, Some(from));
    }
}

fn toggle_voice(w: &mut World, from: UserId, c: Option<ChannelId>, param: &str) {
    let Some(t) = search_for_user(w, from, param) else {
        return;
    };
    let Some(c) = c else { return };
    if channel_is_voiced(w, c, t) {
        channel_remove_voiced(w, c, t, Some(from));
    } else {
        channel_add_voiced(w, c, t, Some(from));
    }
}

fn add_moderator(w: &mut World, from: UserId, c: Option<ChannelId>, param: &str) {
    if let (Some(c), Some(t)) = (c, search_for_user(w, from, param)) {
        channel_add_moderator(w, c, t, Some(from));
    }
}

fn remove_moderator(w: &mut World, from: UserId, c: Option<ChannelId>, param: &str) {
    if let (Some(c), Some(t)) = (c, search_for_user(w, from, param)) {
        channel_remove_moderator(w, c, t, Some(from));
    }
}

fn toggle_moderator(w: &mut World, from: UserId, c: Option<ChannelId>, param: &str) {
    let Some(t) = search_for_user(w, from, param) else {
        return;
    };
    let Some(c) = c else { return };
    if channel_is_moderator(w, c, t) {
        channel_remove_moderator(w, c, t, Some(from));
    } else {
        channel_add_moderator(w, c, t, Some(from));
    }
}

fn rename_channel(w: &mut World, _from: UserId, c: Option<ChannelId>, param: &str) {
    if let Some(c) = c {
        channel_rename(w, c, param);
    }
}

fn query_who_is(w: &mut World, from: UserId, _c: Option<ChannelId>, param: &str) {
    let Some(t) = search_for_user(w, from, param) else {
        return;
    };
    let tname = username(w, t);
    if user(w, t).is_some_and(|x| x.anonymous) {
        user_send_message(w, from, 41, Some(&tname), None); // %1 is remaining anonymous.
    } else {
        let real = user(w, t)
            .and_then(|x| w.mobile(x.mobile).and_then(|m| (m.ty.name)(w, x.mobile)))
            .unwrap_or_default();
        user_send_message(w, from, 43, Some(&tname), Some(&real)); // %1 is known in the lands of Britannia as %2.
    }
}

fn kick(w: &mut World, from: UserId, c: Option<ChannelId>, param: &str) {
    if let (Some(c), Some(t)) = (c, search_for_user(w, from, param)) {
        channel_kick(w, c, t, Some(from), false);
    }
}

fn enable_default_voice(w: &mut World, _from: UserId, c: Option<ChannelId>, _p: &str) {
    if let Some(c) = c {
        channel_set_voice_restricted(w, c, false);
    }
}

fn disable_default_voice(w: &mut World, _from: UserId, c: Option<ChannelId>, _p: &str) {
    if let Some(c) = c {
        channel_set_voice_restricted(w, c, true);
    }
}

fn toggle_default_voice(w: &mut World, _from: UserId, c: Option<ChannelId>, _p: &str) {
    if let Some(c) = c {
        let now = !channel_voice_restricted(w, c);
        channel_set_voice_restricted(w, c, now);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_packet_layout() {
        // OpenChatWindow (0x3ED + 20 - 20) with "ab"
        let p = chat_message_packet("", ChatCommand::OpenChatWindow as i32 + 20, "ab", "");
        assert_eq!(p[0], 0xB2);
        assert_eq!(u16::from_be_bytes([p[1], p[2]]) as usize, p.len());
        assert_eq!(u16::from_be_bytes([p[3], p[4]]), 0x3ED);
        assert_eq!(&p[5..9], &[0, 0, 0, 0]);
        assert_eq!(&p[9..], &[0, b'a', 0, b'b', 0, 0, 0, 0]);
        assert_eq!(p.len(), 17);
    }

    #[test]
    fn handler_table_matches() {
        assert!(get_handler(0x61).is_some_and(|h| !h.require_moderator && h.require_conference));
        assert!(get_handler(0x76).is_some_and(|h| h.require_moderator && h.require_conference));
        assert!(get_handler(0x42).is_none());
    }
}
