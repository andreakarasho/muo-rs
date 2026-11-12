//! The help system: `HelpEvents`, `HelpTopic`, `PageQueue`/`PageEntry`, `PagePrompt`, `PagePromptGump`,
//! `PageResponseGump`, `HelpGump` (+ `ContainedMenu`), `MessageSentGump`, `PageQueueGump`, `PredefinedResponse`,
//! `PredefGump`, `PageEntryGump`, `PageDiscord`, `SpeechLog` (+ `[SpeechLog`), `SpeechLogGump` and `StuckMenu`.
//!
//! `PageDiscord`'s HTTP POST goes through [`HelpHooks::discord_post`] (`muo_content::http_tls::post_json_detached`) and
//! `Email.SendQueueEmail` through [`HelpHooks::queue_email`] (`muo_content::misc::email`), both set in [`configure`].

use std::rc::Rc;

use indexmap::IndexMap;
use muo_content::gumps::base::{
    DynamicGumpLogic, LegacyGumpLogic, LegacyGumpObj, RelayInfo, StaticGumpLogic, StaticGumpObj,
};
use muo_content::gumps::layout::GumpButtonType;
use muo_content::gumps::{
    DynamicGumpBuilder, DynamicGumpObj, StaticGumpBuilder, close_gump, has_gump,
    send_gump_to_mobile,
};
use muo_content::mx;
use muo_content::player_mobile::{self, hooks as phooks};
use muo_primitives::utility::fix_html;
use muo_primitives::{DateTime, Point3D, TimeSpan, html};
use muo_server::World;
use muo_server::commands::{CommandEventArgs, CommandSystem};
use muo_server::ids::{EntityId, MobileId};
use muo_server::map::MapId;
use muo_server::menus::{self, Menu};
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part1::props_b;
use muo_server::mobile::part2::messages::{
    send_localized_message, send_localized_message_affix, send_message_hue,
};
use muo_server::net_state::NetStateId;
use muo_server::prompt::{Prompt, SimplePrompt};
use muo_server::target::{self, TargetFlags, Targeted};
use muo_server::timer::{self, TimerHandle};
use tracing::{debug, error, info};

pub use muo_content::player_mobile::members::SpeechLogEntry;

// ---- HelpTopic ---------------------------------------------------------------------------------------------------

pub mod help_topic {
    pub const HEALING: i32 = 29;
    pub const PLANT_GROWING: i32 = 48;
    pub const INFESTATION_LEVEL: i32 = 54;
    pub const FUNGUS_LEVEL: i32 = 56;
    pub const POISON_LEVEL: i32 = 58;
    pub const DISEASE_LEVEL: i32 = 60;
    pub const POLLINATION_STATE: i32 = 67;
    pub const SEED_PRODUCTION: i32 = 68;
    pub const RESOURCE_PRODUCTION: i32 = 69;
    pub const DECORATIVE_MODE: i32 = 70;
    pub const EMPTYING_BOWL: i32 = 71;
}

/// `NetState.SendDisplayHelpTopic(topicID, display)`: the 11 byte 0xBF/0x17 packet.
pub fn display_help_topic_packet(topic_id: i32, display: bool) -> Vec<u8> {
    let mut p = vec![0xBF, 0, 11, 0, 0x17, 1];
    p.extend_from_slice(&topic_id.to_be_bytes());
    p.push(display as u8);
    p
}

pub fn send_display_help_topic(w: &mut World, ns: NetStateId, topic_id: i32, display: bool) {
    if w.cannot_send_packets(ns) {
        return;
    }
    w.send(ns, &display_help_topic_packet(topic_id, display));
}

// ---- hooks and events --------------------------------------------------------------------------------------------

/// What the help system calls that other tasks own. Unset: the system is absent.
#[derive(Default)]
pub struct HelpHooks {
    /// `Sigil.ExistsOn(m)`. Unset: false.
    pub sigil_exists_on: Option<fn(&World, MobileId) -> bool>,
    /// `Email.SendQueueEmail(entry, pageTypeName)`.
    pub queue_email: Option<fn(&mut World, &PageEntry, &str)>,
    /// The HTTP POST of `PageDiscord` (url, json body).
    pub discord_post: Option<fn(&mut World, &str, &str)>,
}

/// `HelpEvents`.
#[derive(Default)]
pub struct HelpEvents {
    pub page_enqueued: Vec<fn(&mut World, &PageEntry)>,
    pub page_removed: Vec<fn(&mut World, &PageEntry)>,
    pub page_handler_changed: Vec<fn(&mut World, MobileId, MobileId, &PageEntry)>,
    pub page_waiting: Vec<fn(&mut World, &PageEntry)>,
    pub stuck_menu: Vec<fn(&mut World, MobileId)>,
}

fn events(w: &mut World) -> &mut HelpEvents {
    w.resources.get_mut::<HelpEvents>()
}

// ---- PageQueue ---------------------------------------------------------------------------------------------------

/// `PageType`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PageType {
    Bug,
    Stuck,
    Account,
    Question,
    Suggestion,
    Other,
    VerbalHarassment,
    PhysicalHarassment,
}

/// `PageEntry`.
#[derive(Clone)]
pub struct PageEntry {
    pub id: u32,
    pub sender: MobileId,
    pub handler: Option<MobileId>,
    pub sent: DateTime,
    pub message: String,
    pub ty: PageType,
    pub page_location: Point3D,
    pub page_map: Option<MapId>,
    pub speech_log: Option<Vec<SpeechLogEntry>>,
    timer: Option<TimerHandle>,
}

/// `PageQueue`'s statics.
#[derive(Default)]
pub struct PageQueue {
    pub list: Vec<PageEntry>,
    keyed_by_handler: IndexMap<MobileId, u32>,
    keyed_by_sender: IndexMap<MobileId, u32>,
    next_id: u32,
    /// The `Handler` of an entry that left the list: a `PageEntryGump` still holds the entry.
    gone_handlers: IndexMap<u32, Option<MobileId>>,
}

fn pq(w: &mut World) -> &mut PageQueue {
    w.resources.get_mut::<PageQueue>()
}

fn pq_ref(w: &World) -> Option<&PageQueue> {
    w.resources.get::<PageQueue>()
}

/// `PageQueue.List`.
pub fn list(w: &World) -> Vec<PageEntry> {
    pq_ref(w).map(|q| q.list.clone()).unwrap_or_default()
}

pub fn index_of(w: &World, id: u32) -> Option<usize> {
    pq_ref(w)?.list.iter().position(|e| e.id == id)
}

fn entry(w: &World, id: u32) -> Option<PageEntry> {
    pq_ref(w)?.list.iter().find(|e| e.id == id).cloned()
}

/// `PageQueue.GetEntry(sender)`.
pub fn get_entry(w: &World, sender: MobileId) -> Option<PageEntry> {
    let id = *pq_ref(w)?.keyed_by_sender.get(&sender)?;
    entry(w, id)
}

pub fn contains(w: &World, sender: MobileId) -> bool {
    pq_ref(w).is_some_and(|q| q.keyed_by_sender.contains_key(&sender))
}

pub fn is_handling(w: &World, m: MobileId) -> bool {
    pq_ref(w).is_some_and(|q| q.keyed_by_handler.contains_key(&m))
}

pub fn get_page_type_name(t: PageType) -> &'static str {
    match t {
        PageType::Bug => "Bug",
        PageType::Stuck => "Stuck",
        PageType::Account => "Account",
        PageType::Question => "Question",
        PageType::Suggestion => "Suggestion",
        PageType::Other => "Other",
        PageType::VerbalHarassment => "Verbal Harassment",
        PageType::PhysicalHarassment => "Physical Harassment",
    }
}

fn name_of(w: &World, m: MobileId) -> String {
    muo_server::mobile::part1::props_b::raw_name(w, m).unwrap_or_default()
}

fn map_name(w: &World, map: Option<MapId>) -> String {
    map.and_then(|m| w.maps.get(m))
        .map(|m| m.name().to_owned())
        .unwrap_or_default()
}

fn account_name(w: &World, m: MobileId) -> Option<String> {
    let a = w.mobile(m)?.account?;
    muo_content::accounting::account(w, a).map(|a| a.username.clone())
}

/// `new PageEntry(sender, message, type)`: starts the status timer.
pub fn new_page_entry(w: &mut World, sender: MobileId, message: &str, ty: PageType) -> PageEntry {
    let q = pq(w);
    q.next_id += 1;
    let id = q.next_id;
    let (loc, map) = w
        .mobile(sender)
        .map_or((Point3D::default(), None), |m| (m.location, m.map));
    let speech_log = (ty == PageType::VerbalHarassment)
        .then(|| {
            player_mobile::data(w, sender)
                .and_then(|d| d.speech_log.as_ref())
                .map(|l| l.iter().cloned().collect())
        })
        .flatten();
    let h = timer::start(
        w,
        TimeSpan::from_secs_f64(1.0),
        TimeSpan::from_mins_f64(2.0),
        0,
        move |w| status_tick(w, id),
    );
    PageEntry {
        id,
        sender,
        handler: None,
        sent: w.now,
        message: fix_html(message),
        ty,
        page_location: loc,
        page_map: map,
        speech_log,
        timer: Some(h),
    }
}

/// `PageEntry.InternalTimer.OnTick`.
fn status_tick(w: &mut World, id: u32) {
    let Some(e) = entry(w, id) else { return };
    let index = index_of(w, id);
    let online = props_b::net_state(w, e.sender).is_some();
    if online && let Some(index) = index {
        // Thank you for paging. Queue status :
        send_localized_message_affix(
            w,
            e.sender,
            1008077,
            true,
            &(index + 1).to_string(),
            "",
            0x3B2,
        );
        // You can reference our website at www.uo.com or contact us at support@uo.com. To cancel your page...
        send_localized_message(w, e.sender, 1008084, "", 0x3B2);
        if let Some(h) = e.handler
            && props_b::net_state(w, h).is_none()
        {
            set_handler(w, id, None);
        }
        if let Some(e) = entry(w, id) {
            for f in std::mem::take(&mut events(w).page_waiting) {
                f(w, &e);
                events(w).page_waiting.push(f);
            }
        }
    } else if index.is_some() {
        remove(w, id);
    }
}

/// `PageEntry.Handler = value`.
pub fn set_handler(w: &mut World, id: u32, value: Option<MobileId>) {
    let Some(e) = entry(w, id) else { return };
    let old = e.handler;
    on_handler_changed(w, old, value, &e);
    if let Some(x) = pq(w).list.iter_mut().find(|x| x.id == id) {
        x.handler = value;
    }
}

/// `PageQueue.OnHandlerChanged`.
fn on_handler_changed(
    w: &mut World,
    old: Option<MobileId>,
    value: Option<MobileId>,
    e: &PageEntry,
) {
    if let Some(o) = old {
        pq(w).keyed_by_handler.shift_remove(&o);
    }
    if let Some(v) = value {
        pq(w).keyed_by_handler.insert(v, e.id);
    }
    discord_handler_update(w, e, old, value);
    if let (Some(o), Some(v)) = (old, value) {
        for f in events(w).page_handler_changed.clone() {
            f(w, o, v, e);
        }
    }
}

/// `PageQueue.Remove(entry)`.
pub fn remove(w: &mut World, id: u32) {
    let Some(e) = entry(w, id) else { return };
    if let Some(mut h) = e.timer {
        h.cancel(w);
    }
    if e.handler.is_some() {
        discord_completed(w, &e);
    }
    let q = pq(w);
    q.gone_handlers.insert(id, e.handler);
    q.list.retain(|x| x.id != id);
    q.keyed_by_sender.shift_remove(&e.sender);
    if let Some(h) = e.handler {
        q.keyed_by_handler.shift_remove(&h);
    }
    for f in events(w).page_removed.clone() {
        f(w, &e);
    }
}

/// `PageQueue.Remove(sender)`.
pub fn remove_sender(w: &mut World, sender: MobileId) {
    if let Some(e) = get_entry(w, sender) {
        remove(w, e.id);
    }
}

/// `PageQueue.Enqueue(entry)`.
pub fn enqueue(w: &mut World, e: PageEntry) {
    let (id, sender) = (e.id, e.sender);
    let q = pq(w);
    q.list.push(e);
    q.keyed_by_sender.insert(sender, id);

    let mut staff_online = false;
    let ns_mobiles: Vec<MobileId> = w.net_states.values().filter_map(|d| d.mobile).collect();
    for m in ns_mobiles {
        let Some(mm) = w.mobile(m) else { continue };
        let (level, notify, last_move) = (mm.access_level, mm.auto_page_notify, mm.last_move_time);
        if level >= AccessLevel::Counselor && notify && !is_handling(w, m) {
            mx::send_message(w, m, "A new page has been placed in the queue.");
        }
        if level >= AccessLevel::Counselor && notify && w.tick_count - last_move < 600000 {
            staff_online = true;
        }
    }
    if !staff_online {
        mx::send_message(
            w,
            sender,
            "We are sorry, but no staff members are currently available to assist you. Your page will remain in the queue until one becomes available, or until you cancel it manually.",
        );
    }
    let Some(e) = entry(w, id) else { return };
    if e.speech_log.is_some()
        && let Some(f) = w.resources.get::<HelpHooks>().and_then(|h| h.queue_email)
    {
        f(w, &e, get_page_type_name(e.ty));
    }
    discord_notification(w, &e);
    for f in events(w).page_enqueued.clone() {
        f(w, &e);
    }
}

/// `Email.SendQueueEmail(entry, pageType)`.
fn send_queue_email(w: &mut World, e: &PageEntry, page_type: &str) {
    use std::fmt::Write;
    let t = w.now;
    let acct = |m: MobileId| account_name(w, m);
    let name = |m: MobileId| {
        w.mobile(m)
            .and_then(|x| (x.ty.name)(w, m))
            .unwrap_or_default()
    };
    let mut text = String::new();
    // The text keeps this indentation and starts and ends with a line break.
    let _ = write!(
        text,
        "
          Speech Log Page - {page_type}

          From: '{}', Account: '{}'

          Location: {} [{}]
          Sent on: {}/{:02}/{:02} {}:{:02}:{:02}

          Message:
          '{}'

          Speech Log
          ==========
        
",
        name(e.sender),
        acct(e.sender).unwrap_or_else(|| " ??? ".into()),
        w.mobile(e.sender).map(|s| s.location).unwrap_or_default(),
        w.mobile(e.sender)
            .map_or_else(String::new, |s| map_name(w, s.map)),
        t.year(),
        t.month(),
        t.day(),
        t.hour(),
        t.minute(),
        t.second(),
        e.message,
    );
    for l in e.speech_log.iter().flatten() {
        let c = l.created;
        let _ = writeln!(
            text,
            "{}:{:02}:{:02} - {} ({}): '{}'",
            c.hour(),
            c.minute(),
            c.second(),
            name(l.from),
            acct(l.from).unwrap_or_else(|| "???".into()),
            l.speech
        );
    }
    muo_content::misc::email::send_text(
        w,
        |c| c.speech_log_page_address.clone(),
        "Speech Log Page Forwarding",
        &text,
    );
}

/// `PageQueue.CheckAllowedToPage(from)`.
pub fn check_allowed_to_page(w: &mut World, from: MobileId) -> bool {
    let Some(d) = player_mobile::data(w, from) else {
        return true;
    };
    let squelched = d.flag(player_mobile::PlayerFlag::PAGING_SQUELCHED);
    if phooks::design_bounds(w, from).is_some() {
        // You cannot request help while customizing a house or transferring a character.
        send_localized_message(w, from, 500182, "", 0x3B2);
        return false;
    }
    if squelched {
        mx::send_message(w, from, "You cannot request help, sorry.");
        return false;
    }
    true
}

fn pages_command(w: &mut World, e: &mut CommandEventArgs) {
    let from = e.mobile;
    let handled = pq_ref(w)
        .and_then(|q| q.keyed_by_handler.get(&from).copied())
        .and_then(|id| entry(w, id));
    if let Some(en) = handled {
        send(w, from, page_entry_gump(w, from, en));
    } else if !list(w).is_empty() {
        send_page_queue_gump(w, from);
    } else {
        mx::send_message(w, from, "The page queue is empty.");
    }
}

// ---- PageDiscord -------------------------------------------------------------------------------------------------

/// `PageDiscord` state: the `pages.discordWebhookUrl` setting.
#[derive(Default)]
pub struct PageDiscord {
    pub webhook_url: Option<String>,
}

impl PageDiscord {
    pub fn is_enabled(&self) -> bool {
        self.webhook_url.as_deref().is_some_and(|u| !u.is_empty())
    }
}

fn discord_enabled(w: &World) -> Option<String> {
    w.resources
        .get::<PageDiscord>()
        .and_then(|d| d.webhook_url.clone())
        .filter(|u| !u.is_empty())
}

fn page_type_color(t: PageType) -> i32 {
    match t {
        PageType::Bug => 0xFF0000,
        PageType::Stuck => 0xFFFF00,
        PageType::Account => 0x0000FF,
        PageType::Question => 0x00FFFF,
        PageType::Suggestion => 0x00FF00,
        PageType::Other => 0x888888,
        PageType::VerbalHarassment => 0xFF00FF,
        PageType::PhysicalHarassment => 0x800080,
    }
}

fn truncate_message(m: &str) -> String {
    if m.is_empty() {
        return "*(no message)*".to_owned();
    }
    if m.chars().count() > 1000 {
        format!("{}...", m.chars().take(997).collect::<String>())
    } else {
        m.to_owned()
    }
}

fn discord_timestamp(t: DateTime) -> String {
    format!("<t:{}:f>", t.to_unix_seconds())
}

fn discord_send(w: &mut World, url: &str, payload: serde_json::Value) {
    match w.resources.get::<HelpHooks>().and_then(|h| h.discord_post) {
        Some(f) => f(w, url, &payload.to_string()),
        None => debug!("Discord page webhook has no HTTP sender; payload dropped"),
    }
}

fn field(name: &str, value: String, inline: bool) -> serde_json::Value {
    serde_json::json!({ "name": name, "value": value, "inline": inline })
}

fn discord_notification(w: &mut World, e: &PageEntry) {
    let Some(url) = discord_enabled(w) else {
        return;
    };
    let type_name = get_page_type_name(e.ty);
    let location = format!(
        "{} ({})",
        e.page_location,
        if e.page_map.is_some() {
            map_name(w, e.page_map)
        } else {
            "Unknown".into()
        }
    );
    let sender = w
        .mobile(e.sender)
        .and_then(|x| (x.ty.name)(w, e.sender))
        .unwrap_or_else(|| "Unknown".into()); // `Sender?.Name ?? "Unknown"`
    let mut fields = vec![
        field("Player", sender, true),
        field(
            "Account",
            account_name(w, e.sender).unwrap_or_else(|| "Unknown".into()),
            true,
        ),
        field("Page Type", type_name.to_owned(), true),
        field("Location", location, true),
        field("Time Sent", discord_timestamp(e.sent), true),
        field("Queue Position", format!("#{}", list(w).len()), true),
        field("Message", truncate_message(&e.message), false),
    ];
    if let Some(l) = e.speech_log.as_ref().filter(|l| !l.is_empty()) {
        fields.push(field(
            "Speech Log Available",
            format!("Contains {} speech entries", l.len()),
            false,
        ));
    }
    let embed = serde_json::json!({
        "title": format!("New {type_name} Page"),
        "color": page_type_color(e.ty),
        "fields": fields,
        "footer": { "text": "UO Page System" },
        "timestamp": e.sent.to_iso_string(),
    });
    discord_send(
        w,
        &url,
        serde_json::json!({ "username": "UO Pages", "embeds": [embed] }),
    );
}

fn discord_handler_update(
    w: &mut World,
    e: &PageEntry,
    old: Option<MobileId>,
    new: Option<MobileId>,
) {
    let Some(url) = discord_enabled(w) else {
        return;
    };
    if old.is_some() && new.is_none() && index_of(w, e.id).is_none() {
        return;
    }
    let (text, color) = match (old, new) {
        (None, Some(n)) => (
            format!(
                "\u{1F6E0}\u{FE0F} **{}** is now handling the page",
                name_of(w, n)
            ),
            0x00FF00,
        ),
        (Some(o), None) => (
            format!("\u{274C} **{}** stopped handling the page", name_of(w, o)),
            0xFF0000,
        ),
        _ => return,
    };
    let type_name = get_page_type_name(e.ty);
    let embed = serde_json::json!({
        "title": format!("{type_name} Page Update"),
        "description": text,
        "color": color,
        "fields": [
            field("Player", w.mobile(e.sender).and_then(|x| (x.ty.name)(w, e.sender)).unwrap_or_else(|| "Unknown".into()), true),
            field("Page Type", type_name.to_owned(), true),
            field("Queue Position", format!("#{}", index_of(w, e.id).map_or(0, |i| i + 1)), true),
        ],
        "footer": { "text": "UO Page System" },
        "timestamp": w.now.to_iso_string(),
    });
    discord_send(
        w,
        &url,
        serde_json::json!({ "username": "UO Pages", "embeds": [embed] }),
    );
}

fn discord_completed(w: &mut World, e: &PageEntry) {
    let Some(url) = discord_enabled(w) else {
        return;
    };
    let handler = e
        .handler
        .and_then(|h| (w.mobile(h)?.ty.name)(w, h))
        .unwrap_or_else(|| "Staff".to_owned());
    let minutes = (w.now - e.sent).total_minutes();
    // `{elapsed.TotalMinutes:F0}` rounds halves away from zero
    let text = if minutes < 1.0 {
        "< 1 minute".to_owned()
    } else {
        format!("{} minutes", (minutes + 0.5).floor() as i64)
    };
    let type_name = get_page_type_name(e.ty);
    let (y, mo, d) = w.now.ymd();
    let stamp = format!(
        "{y:04}-{mo:02}-{d:02}T{:02}:{:02}:{:02}Z",
        w.now.hour(),
        w.now.minute(),
        w.now.second()
    );
    let embed = serde_json::json!({
        "title": format!("\u{2705} {type_name} Page Completed"),
        "color": 0x00AA00,
        "fields": [
            field("Player", w.mobile(e.sender).and_then(|x| (x.ty.name)(w, e.sender)).unwrap_or_else(|| "Unknown".into()), true),
            field("Handled By", handler, true),
            field("Response Time", text, true),
        ],
        "footer": { "text": "UO Page System" },
        "timestamp": stamp,
    });
    discord_send(
        w,
        &url,
        serde_json::json!({ "username": "UO Pages", "embeds": [embed] }),
    );
}

// ---- gumps: helpers ----------------------------------------------------------------------------------------------

fn send(w: &mut World, to: MobileId, g: impl muo_content::gumps::Gump + 'static) {
    send_gump_to_mobile(w, to, Box::new(g), false);
}

fn mobile_of(w: &World, ns: NetStateId) -> Option<MobileId> {
    w.net_states.get(&ns).and_then(|n| n.mobile)
}

fn enhanced(w: &World, m: MobileId) -> bool {
    props_b::net_state(w, m)
        .and_then(|ns| w.net_state(ns))
        .is_some_and(|d| d.caps.is_enhanced_client())
}

// ---- PagePrompt / PagePromptGump / PageResponseGump -----------------------------------------------------------------

/// `new PagePrompt(type)` as the prompt `from.Prompt` takes.
pub fn page_prompt(ty: PageType) -> Prompt {
    let mut p = SimplePrompt::new(
        Some(Rc::new(move |w: &mut World, from: MobileId, text: &str| {
            // The next available Counselor/Game Master will respond as soon as possible.
            send_localized_message(w, from, 501234, "", 0x35);
            let e = new_page_entry(w, from, text, ty);
            enqueue(w, e);
        })),
        true,
    )
    .into_prompt();
    p.on_cancel = Some(Rc::new(|w, from| {
        send_localized_message(w, from, 501235, "", 0x35)
    })); // Help request aborted.
    p
}

pub struct PagePromptGump {
    from: MobileId,
    ty: PageType,
}

pub fn page_prompt_gump(from: MobileId, ty: PageType) -> StaticGumpObj<PagePromptGump> {
    StaticGumpObj::new(0, 0, PagePromptGump { from, ty })
}

impl StaticGumpLogic for PagePromptGump {
    const TYPE_NAME: &'static str = "Server.Engines.Help.PagePromptGump";
    fn singleton(&self) -> bool {
        true
    }
    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.add_background(50, 50, 540, 350, 2600);
        b.add_page(0);
        b.add_html_localized(264, 80, 200, 24, 1062524, false, false); // Enter Description
        // Please enter a brief description (up to 200 characters) of your problem:
        b.add_html_localized(120, 108, 420, 48, 1062638, false, false);
        b.add_background(100, 148, 440, 200, 3500);
        b.add_text_entry(120, 168, 400, 200, 1153, 0, "");
        b.add_button(175, 355, 2074, 2075, 1); // Okay
        b.add_button(405, 355, 2073, 2072, 0); // Cancel
    }
    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        if info.button_id == 0 {
            send_localized_message(w, self.from, 501235, "", 0x35); // Help request aborted.
            return;
        }
        let text = info
            .get_text_entry(0)
            .map(str::trim)
            .unwrap_or("")
            .to_owned();
        if text.is_empty() {
            send_message_hue(w, self.from, 0x35, "You must enter a description.");
            send(w, self.from, page_prompt_gump(self.from, self.ty));
        } else {
            send_localized_message(w, self.from, 501234, "", 0x35);
            let e = new_page_entry(w, self.from, &text, self.ty);
            enqueue(w, e);
        }
    }
}

pub struct PageResponseGump {
    from: MobileId,
    from_name: String,
    name: String,
    text: String,
}

pub fn page_response_gump(
    w: &World,
    from: MobileId,
    name: &str,
    text: &str,
) -> StaticGumpObj<PageResponseGump> {
    StaticGumpObj::new(
        0,
        0,
        PageResponseGump {
            from,
            from_name: name_of(w, from),
            name: name.to_owned(),
            text: text.to_owned(),
        },
    )
}

impl StaticGumpLogic for PageResponseGump {
    const TYPE_NAME: &'static str = "Server.Engines.Help.PageResponseGump";
    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.add_background(50, 25, 540, 430, 2600);
        b.add_page(0);
        b.add_html_localized(150, 40, 360, 40, 1062610, false, false); // <CENTER><U>Ultima Online Help Response</U></CENTER>
        b.add_html(
            80,
            90,
            480,
            290,
            &format!("{} tells {}: {}", self.name, self.from_name, self.text),
            true,
            true,
        );
        b.add_html_localized(80, 390, 480, 40, 1062611, false, false); // Clicking the OKAY button will remove...
        b.add_button(400, 417, 2074, 2075, 1); // OKAY
        b.add_button(475, 417, 2073, 2072, 0); // CANCEL
    }
    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        if info.button_id != 1 {
            send(
                w,
                self.from,
                message_sent_gump(w, self.from, &self.name, &self.text),
            );
        }
    }
}

// ---- MessageSentGump -----------------------------------------------------------------------------------------------

pub struct MessageSentGump {
    mobile: MobileId,
    name: String,
    text: String,
}

/// `new MessageSentGump(mobile, name, text)`.
pub fn message_sent_gump(
    w: &World,
    mobile: MobileId,
    name: &str,
    text: &str,
) -> LegacyGumpObj<MessageSentGump> {
    let mut g = LegacyGumpObj::new(
        30,
        30,
        MessageSentGump {
            mobile,
            name: name.to_owned(),
            text: text.to_owned(),
        },
    );
    let l = &mut g.layout;
    l.closable = false;
    l.add_page(0);
    l.add_background(0, 0, 92, 75, 0xA3C);
    if enhanced(w, mobile) {
        l.add_background(5, 7, 82, 61, 9300);
    } else {
        l.add_image_tiled(5, 7, 82, 61, 0xA40);
        l.add_alpha_region(5, 7, 82, 61);
    }
    l.add_image_tiled(9, 11, 21, 53, 0xBBC);
    l.add_button(10, 12, 0x7D2, 0x7D2, 0);
    l.add_html_localized_color(34, 28, 65, 24, 3001002, 0x7FFF, false, false); // Message
    g
}

impl LegacyGumpLogic for MessageSentGump {
    const TYPE_NAME: &'static str = "Server.Engines.Help.MessageSentGump";
    fn on_response(&mut self, w: &mut World, _sender: NetStateId, _info: &RelayInfo) {
        let g = page_response_gump(w, self.mobile, &self.name, &self.text);
        send(w, self.mobile, g);
    }
}

// ---- HelpGump ------------------------------------------------------------------------------------------------------

pub struct HelpGump {
    from: MobileId,
    young: bool,
}

fn is_young(w: &World, m: MobileId) -> bool {
    player_mobile::young(w, m)
}

pub fn help_gump(w: &World, from: MobileId) -> DynamicGumpObj<HelpGump> {
    DynamicGumpObj::new(
        0,
        0,
        HelpGump {
            from,
            young: is_young(w, from),
        },
    )
}

fn add_option(
    b: &mut DynamicGumpBuilder,
    y: i32,
    height: i32,
    button_id: i32,
    name: i32,
    ty: GumpButtonType,
    page: i32,
) {
    b.add_button_type(80, y, 5540, 5541, button_id, ty, page);
    b.add_html_localized(110, y, 450, height, name, true, true);
}

fn add_option_reply(b: &mut DynamicGumpBuilder, y: i32, button_id: i32, name: i32) {
    add_option(b, y, 74, button_id, name, GumpButtonType::Reply, 0);
}

impl DynamicGumpLogic for HelpGump {
    const TYPE_NAME: &'static str = "Server.Engines.Help.HelpGump";
    fn singleton(&self) -> bool {
        true
    }
    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        let total_height = 110 + if self.young { 64 } else { 0 } + 4 * 80;
        b.add_background(50, 25, 540, total_height, 2600);
        b.add_page(0);
        b.add_html_localized(150, 50, 360, 40, 1001002, false, false); // <CENTER><U>Ultima Online Help Menu</U></CENTER>
        b.add_button(425, total_height - 15, 2073, 2072, 0); // Close
        b.add_page(1);
        let delta = 80;
        let height = 74;
        let mut y;
        if self.young {
            // <U>Young player haven transport</U>
            add_option(b, 90, 48, 9, 1041525, GumpButtonType::Reply, 2);
            y = 90 + 54;
        } else {
            y = 90;
        }
        add_option(b, y, height, 1, 1001003, GumpButtonType::Reply, 2); // General question about Ultima Online
        y += delta;
        add_option(b, y, height, 2, 1001004, GumpButtonType::Reply, 0); // My character is physically stuck
        y += delta;
        add_option(b, y, height, 0, 1001005, GumpButtonType::Page, 3); // Another player is harassing me
        add_option(b, y + delta, height, 0, 1001006, GumpButtonType::Page, 2); // Other

        b.add_page(2);
        let mut y = 90;
        add_option_reply(b, y, 3, 1001009); // Report a bug
        y += delta;
        add_option_reply(b, y, 4, 1074795); // Suggestion for the Game
        y += delta;
        add_option_reply(b, y, 5, 1074796); // Visit the Ultima Online Knowledge Base
        add_option_reply(b, y + delta, 6, 1001006); // Other

        b.add_page(3);
        let mut y = 90;
        let delta = 150;
        add_option(b, y, 145, 7, 1062572, GumpButtonType::Reply, 0); // verbal harassment / exploiting
        y += delta;
        add_option(b, y, 145, 8, 1062573, GumpButtonType::Reply, 0); // physical harassment
        b.add_button_type(150, y + 150, 5540, 5541, 0, GumpButtonType::Page, 1);
        b.add_html_localized(180, y + 150, 335, 40, 1001015, false, false); // NO  - I meant to ask for help with another matter.
    }

    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        let from = self.from;
        let mut ty: Option<PageType> = None;
        match info.button_id {
            0 => send_localized_message(w, from, 501235, "", 0x35), // Help request aborted.
            1 => ty = Some(PageType::Question),
            2 => {
                let house = phooks::house_at(w, from, from);
                if let Some(h) = house.filter(|h| h.is_aos_rules && !phooks::in_safe_zone(w, from))
                {
                    // from.Location = house.BanLocation
                    muo_server::mobile::part1::lifecycle::set_location(w, from, h.ban_location);
                } else if phooks::in_jail_region(w, from) {
                    send_localized_message(w, from, 1114345, "", 0x35); // You'll need a better jailbreak plan than that!
                } else if sigil_exists_on(w, from) {
                    send_localized_message(w, from, 1061632, "", 0x3B2); // You can't do that while carrying the sigil.
                } else if player_mobile::data(w, from).is_some()
                    && player_mobile::can_use_stuck_menu(w, from)
                    && region_can_use_stuck_menu(w, from)
                    && !check_combat(w, from)
                    && !w.mobile(from).is_some_and(|m| m.frozen)
                    && !muo_server::mobile::base::criminal(w, from)
                    && (w.core_aos() || !muo_server::mobile::base::murderer(w, from))
                {
                    let menu = stuck_menu(w, from, from, true);
                    begin_close(w, from);
                    send(w, from, menu);
                } else {
                    ty = Some(PageType::Stuck);
                }
            }
            3 => ty = Some(PageType::Bug),
            4 => ty = Some(PageType::Suggestion),
            5 => ty = Some(PageType::Account),
            6 => ty = Some(PageType::Other),
            7 => ty = Some(PageType::VerbalHarassment),
            8 => ty = Some(PageType::PhysicalHarassment),
            9 if is_young(w, from) => {
                if phooks::in_jail_region(w, from) {
                    send_localized_message(w, from, 1114345, "", 0x35);
                } else if muo_server::region::is_part_of_name(
                    w,
                    props_b::region(w, from),
                    "Haven Island",
                    false,
                ) {
                    send_localized_message(w, from, 1041529, "", 0x3B2); // You're already in Haven
                } else {
                    muo_server::mobile::base::move_to_world(
                        w,
                        from,
                        Point3D::new(3503, 2574, 14),
                        MapId::TRAMMEL,
                    );
                }
            }
            _ => {}
        }
        if let Some(t) = ty
            && check_allowed_to_page(w, from)
        {
            send(w, from, page_prompt_gump(from, t));
        }
    }
}

fn sigil_exists_on(w: &World, m: MobileId) -> bool {
    w.resources
        .get::<HelpHooks>()
        .and_then(|h| h.sigil_exists_on)
        .is_some_and(|f| f(w, m))
}

fn region_can_use_stuck_menu(w: &mut World, m: MobileId) -> bool {
    let rid = props_b::region(w, m);
    match w.region(rid).map(|r| r.ty) {
        Some(rt) => (rt.can_use_stuck_menu)(w, rid, m),
        None => true,
    }
}

/// `HelpGump.CheckCombat(m)`.
pub fn check_combat(w: &World, m: MobileId) -> bool {
    w.mobile(m).is_some_and(|x| {
        x.aggressed
            .iter()
            .any(|i| w.now - i.last_combat_time < TimeSpan::from_secs_f64(30.0))
    })
}

/// `HelpGump.HelpRequest(m)`.
pub fn help_request(w: &mut World, m: MobileId) {
    let Some(ns) = props_b::net_state(w, m) else {
        return;
    };
    if has_gump::<HelpGump>(w, ns) {
        return;
    }
    if !check_allowed_to_page(w, m) {
        return;
    }
    if contains(w, m) {
        send_contained_menu(w, ns, m);
    } else {
        send(w, m, help_gump(w, m));
    }
}

/// `ContainedMenu`.
fn send_contained_menu(w: &mut World, ns: NetStateId, from: MobileId) {
    let mut menu = Menu::question(
        w,
        "You already have an open help request. We will have someone assist you as soon as possible.  What would you like to do?",
        vec![
            "Leave my old help request like it is.".into(),
            "Remove my help request from the queue.".into(),
        ],
    );
    menu.on_cancel = Some(Rc::new(move |w, _| {
        send_localized_message(w, from, 1005306, "", 0x35)
    })); // Help request unchanged.
    menu.on_response = Some(Rc::new(move |w, _, index| match index {
        0 => send_localized_message(w, from, 1005306, "", 0x35),
        1 => match get_entry(w, from) {
            Some(e) if e.handler.is_none() => {
                send_localized_message(w, from, 1005307, "", 0x35); // Removed help request.
                remove(w, e.id);
            }
            _ => send_localized_message(w, from, 1005306, "", 0x35),
        },
        _ => {}
    }));
    menus::send_to(w, ns, menu);
}

// ---- PageQueueGump -------------------------------------------------------------------------------------------------

pub struct PageQueueGump {
    list: Vec<u32>,
}

/// `new PageQueueGump()`: drops the entries whose sender left, then lists the rest.
pub fn page_queue_gump(w: &mut World) -> LegacyGumpObj<PageQueueGump> {
    let mut i = 0;
    while let Some(e) = list(w).get(i).cloned() {
        let gone = w.mobile(e.sender).is_none() || props_b::net_state(w, e.sender).is_none();
        if gone {
            remove(w, e.id);
        } else {
            i += 1;
        }
    }
    let entries = list(w);
    let mut g = LegacyGumpObj::new(
        30,
        30,
        PageQueueGump {
            list: entries.iter().map(|e| e.id).collect(),
        },
    );
    let l = &mut g.layout;
    l.add_page(0);
    l.add_image_tiled(0, 0, 410, 448, 0xA40);
    l.add_alpha_region(1, 1, 408, 446);
    l.add_label(180, 12, 2100, "Page Queue");
    if entries.is_empty() {
        l.add_label(12, 44, 2100, "The page queue is empty.");
        return g;
    }
    l.add_page(1);
    for (i, e) in entries.iter().enumerate() {
        let i = i as i32;
        if i >= 5 && i % 5 == 0 {
            l.add_button_type(368, 12, 0xFA5, 0xFA7, 0, GumpButtonType::Page, i / 5 + 1);
            l.add_label(298, 12, 2100, "Next Page");
            l.add_page(i / 5 + 1);
            l.add_button_type(12, 12, 0xFAE, 0xFB0, 0, GumpButtonType::Page, i / 5);
            l.add_label(48, 12, 2100, "Previous Page");
        }
        let type_string = get_page_type_name(e.ty);
        let (color, state) = if e.handler.is_none() {
            (0xFF0000, "Unhandled")
        } else {
            (0xFF, "Handling")
        };
        let text = format!(
            "[{type_string}] {} <basefont color=#{color:06X}>[<u>{state}</u>]</basefont>",
            e.message
        );
        l.add_html(12, 44 + i % 5 * 80, 350, 70, &text, true, true);
        l.add_button(370, 44 + i % 5 * 80 + 24, 0xFA5, 0xFA7, i + 1);
    }
    g
}

fn send_page_queue_gump(w: &mut World, to: MobileId) {
    let g = page_queue_gump(w);
    send(w, to, g);
}

impl LegacyGumpLogic for PageQueueGump {
    const TYPE_NAME: &'static str = "Server.Engines.Help.PageQueueGump";
    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(m) = mobile_of(w, sender) else {
            return;
        };
        if info.button_id >= 1 && info.button_id as usize <= self.list.len() {
            let id = self.list[info.button_id as usize - 1];
            if let Some(e) = entry(w, id) {
                let g = page_entry_gump(w, m, e);
                send(w, m, g);
            } else {
                send_page_queue_gump(w, m);
                mx::send_message(w, m, "That page has been removed.");
            }
        }
    }
}

// ---- PredefinedResponse / PredefGump -------------------------------------------------------------------------------

/// `PredefinedResponse`.
#[derive(Clone, Debug, PartialEq)]
pub struct PredefinedResponse {
    pub title: String,
    pub message: String,
}

/// A `PredefinedResponse` object: PredefGump holds it by reference, so it outlives its place in the list.
pub type Response = Rc<std::cell::RefCell<PredefinedResponse>>;

/// `PredefinedResponse.List`: loaded lazily from `Data/pageresponse.cfg`.
#[derive(Default)]
pub struct PredefinedResponses {
    pub list: Option<Vec<Response>>,
}

fn new_response(title: &str, message: &str) -> Response {
    Rc::new(std::cell::RefCell::new(PredefinedResponse {
        title: title.to_owned(),
        message: message.to_owned(),
    }))
}

fn responses_path(w: &World) -> std::path::PathBuf {
    w.core.base_directory.join("Data/pageresponse.cfg")
}

/// `PredefinedResponse.Load()`.
pub fn load_responses(path: &std::path::Path) -> Vec<Response> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let split: Vec<&str> = line.split('\t').collect();
        if split.len() == 2 {
            out.push(new_response(split[0], split[1]));
        }
    }
    out
}

fn responses(w: &mut World) -> &mut Vec<Response> {
    if w.resources.get_mut::<PredefinedResponses>().list.is_none() {
        let l = load_responses(&responses_path(w));
        w.resources.get_mut::<PredefinedResponses>().list = Some(l);
    }
    w.resources
        .get_mut::<PredefinedResponses>()
        .list
        .as_mut()
        .expect("loaded above")
}

fn save_responses(w: &mut World) {
    let path = responses_path(w);
    let text: String = responses(w)
        .iter()
        .map(|r| format!("{}\t{}\n", r.borrow().title, r.borrow().message))
        .collect();
    if let Err(e) = std::fs::write(&path, text) {
        error!("{e}");
    }
}

pub struct PredefGump {
    from: MobileId,
    /// `m_Response`: the edited response object; None is the overview.
    response: Option<Response>,
}

const LABEL_COLOR: i32 = 0xFFFFFF;

fn add_text_input(
    l: &mut muo_content::gumps::legacy::LegacyLayout,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    id: i32,
    def: &str,
) {
    l.add_image_tiled(x, y, w, h, 0xA40);
    l.add_image_tiled(x + 1, y + 1, w - 2, h - 2, 0xBBC);
    l.add_text_entry(x + 3, y + 1, w - 4, h - 2, 0x480, id, def);
}

/// `new PredefGump(from, responses[index])`: the response object found now.
pub fn predef_gump(
    w: &mut World,
    from: MobileId,
    response: Option<usize>,
) -> LegacyGumpObj<PredefGump> {
    let r = response.and_then(|i| responses(w).get(i).cloned());
    predef_gump_of(w, from, r)
}

pub fn predef_gump_of(
    w: &mut World,
    from: MobileId,
    response: Option<Response>,
) -> LegacyGumpObj<PredefGump> {
    let can_edit = mx::access_level(w, from) >= AccessLevel::GameMaster;
    let enh = enhanced(w, from);
    let list: Vec<PredefinedResponse> = responses(w).iter().map(|r| r.borrow().clone()).collect();
    let mut g = LegacyGumpObj::new(
        30,
        30,
        PredefGump {
            from,
            response: response.clone(),
        },
    );
    g.logic.singleton();
    let l = &mut g.layout;
    l.add_page(0);
    match response.as_ref() {
        None => {
            if enh {
                l.add_background(1, 1, 408, 446, 9300);
            } else {
                l.add_image_tiled(0, 0, 410, 448, 0xA40);
                l.add_alpha_region(1, 1, 408, 446);
            }
            l.add_html(
                10,
                10,
                390,
                20,
                &html::center_rgb("Predefined Responses", LABEL_COLOR),
                false,
                false,
            );
            l.add_page(1);
            let paged = |l: &mut muo_content::gumps::legacy::LegacyLayout, i: i32| {
                l.add_button_type(368, 10, 0xFA5, 0xFA7, 0, GumpButtonType::Page, i / 5 + 1);
                l.add_label(298, 10, 2100, "Next Page");
                l.add_page(i / 5 + 1);
                l.add_button_type(12, 10, 0xFAE, 0xFB0, 0, GumpButtonType::Page, i / 5);
                l.add_label(48, 10, 2100, "Previous Page");
            };
            let mut i = 0i32;
            while (i as usize) < list.len() {
                if i >= 5 && i % 5 == 0 {
                    paged(l, i);
                }
                let r = &list[i as usize];
                l.add_html(
                    12,
                    44 + i % 5 * 80,
                    350,
                    70,
                    &format!("<u>{}</u><br>{}", r.title, r.message),
                    true,
                    true,
                );
                if can_edit {
                    l.add_button(370, 44 + i % 5 * 80 + 24, 0xFA5, 0xFA7, 2 + i * 3);
                    if i > 0 {
                        l.add_button(377, 44 + i % 5 * 80 + 2, 0x15E0, 0x15E4, 3 + i * 3);
                    } else {
                        l.add_image(377, 44 + i % 5 * 80 + 2, 0x25E4, 0);
                    }
                    if (i as usize) < list.len() - 1 {
                        l.add_button(
                            377,
                            44 + i % 5 * 80 + 70 - 2 - 16,
                            0x15E2,
                            0x15E6,
                            4 + i * 3,
                        );
                    } else {
                        l.add_image(377, 44 + i % 5 * 80 + 70 - 2 - 16, 0x25E8, 0);
                    }
                }
                i += 1;
            }
            if can_edit {
                if i >= 5 && i % 5 == 0 {
                    paged(l, i);
                }
                l.add_button(12, 44 + i % 5 * 80, 0xFAB, 0xFAD, 1);
                l.add_html(
                    45,
                    44 + i % 5 * 80,
                    200,
                    20,
                    &html::color_rgb("New Response", LABEL_COLOR),
                    false,
                    false,
                );
            }
        }
        Some(r) if can_edit => {
            let r = r.borrow().clone();
            l.add_image_tiled(0, 0, 410, 250, 0xA40);
            if enh {
                l.add_background(1, 1, 408, 248, 9300);
            } else {
                l.add_alpha_region(1, 1, 408, 248);
            }
            l.add_html(
                10,
                10,
                390,
                20,
                &html::center_rgb("Predefined Response Editor", LABEL_COLOR),
                false,
                false,
            );
            l.add_button(10, 40, 0xFB1, 0xFB3, 1);
            l.add_html(
                45,
                40,
                200,
                20,
                &html::color_rgb("Remove", LABEL_COLOR),
                false,
                false,
            );
            l.add_button(10, 70, 0xFA5, 0xFA7, 2);
            l.add_html(
                45,
                70,
                200,
                20,
                &html::color_rgb("Title:", LABEL_COLOR),
                false,
                false,
            );
            add_text_input(l, 10, 90, 300, 20, 0, &r.title);
            l.add_button(10, 120, 0xFA5, 0xFA7, 3);
            l.add_html(
                45,
                120,
                200,
                20,
                &html::color_rgb("Message:", LABEL_COLOR),
                false,
                false,
            );
            add_text_input(l, 10, 140, 390, 100, 1, &r.message);
        }
        Some(_) => {}
    }
    g
}

impl LegacyGumpLogic for PredefGump {
    const TYPE_NAME: &'static str = "Server.Engines.Help.PredefGump";
    fn singleton(&self) -> bool {
        true
    }
    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        let from = self.from;
        if mx::access_level(w, from) < AccessLevel::Administrator {
            return;
        }
        match self.response.clone() {
            None => {
                let index = info.button_id - 1;
                if index == 0 {
                    let resp = new_response("", "");
                    responses(w).push(resp.clone());
                    let g = predef_gump_of(w, from, Some(resp));
                    send(w, from, g);
                } else {
                    let index = index - 1;
                    let (ty, index) = (index % 3, index / 3);
                    let len = responses(w).len() as i32;
                    if index >= 0 && index < len {
                        let index = index as usize;
                        let resp = responses(w)[index].clone();
                        match ty {
                            0 => {
                                let g = predef_gump_of(w, from, Some(resp));
                                send(w, from, g);
                            }
                            1 if index > 0 => {
                                responses(w).remove(index);
                                responses(w).insert(index - 1, resp);
                                save_responses(w);
                                let g = predef_gump_of(w, from, None);
                                send(w, from, g);
                            }
                            2 if index + 1 < len as usize => {
                                responses(w).remove(index);
                                responses(w).insert(index + 1, resp);
                                save_responses(w);
                                let g = predef_gump_of(w, from, None);
                                send(w, from, g);
                            }
                            _ => {}
                        }
                    }
                }
            }
            Some(resp) => match info.button_id {
                1 => {
                    // list.Remove(m_Response): by reference, nothing when it is no longer in the list
                    if let Some(at) = responses(w).iter().position(|r| Rc::ptr_eq(r, &resp)) {
                        responses(w).remove(at);
                    }
                    save_responses(w);
                    let g = predef_gump_of(w, from, None);
                    send(w, from, g);
                }
                2 => {
                    resp.borrow_mut().title = info.get_text_entry(0).unwrap_or("").to_owned();
                    save_responses(w);
                    let g = predef_gump_of(w, from, Some(resp));
                    send(w, from, g);
                }
                3 => {
                    resp.borrow_mut().message = info.get_text_entry(1).unwrap_or("").to_owned();
                    save_responses(w);
                    let g = predef_gump_of(w, from, Some(resp));
                    send(w, from, g);
                }
                _ => {}
            },
        }
    }
}

// ---- PageEntryGump -------------------------------------------------------------------------------------------------

const ACCESS_LEVEL_HUES: [i32; 7] = [2100, 2122, 2117, 2129, 2415, 2415, 2415];

pub struct PageEntryGump {
    mobile: MobileId,
    entry: u32,
}

fn display_loc(w: &World, loc: Point3D, map: Option<MapId>) -> String {
    format!("{loc} [{}]", map_name(w, map))
}

pub fn page_entry_gump(w: &World, m: MobileId, e: PageEntry) -> LegacyGumpObj<PageEntryGump> {
    let mut g = LegacyGumpObj::new(
        30,
        30,
        PageEntryGump {
            mobile: m,
            entry: e.id,
        },
    );
    let l = &mut g.layout;
    let mut buttons = 0;
    let bottom = 356;
    l.add_page(0);
    if enhanced(w, m) {
        l.add_background(1, 1, 408, 454, 9300);
    } else {
        l.add_image_tiled(0, 0, 410, 456, 0xA40);
        l.add_alpha_region(1, 1, 408, 454);
    }
    l.add_page(1);
    l.add_label(18, 18, 2100, "Sent:");
    l.add_label_cropped(128, 18, 264, 20, 2100, &e.sent.to_string());
    l.add_label(18, 38, 2100, "Sender:");
    let (sloc, smap) = w
        .mobile(e.sender)
        .map_or((Point3D::default(), None), |s| (s.location, s.map));
    let raw = muo_server::mobile::part1::props_b::raw_name(w, e.sender).unwrap_or_default();
    l.add_label_cropped(
        128,
        38,
        264,
        20,
        2100,
        &format!("{raw} {}", display_loc(w, sloc, smap)),
    );

    l.add_button(18, bottom - buttons * 22, 0xFAB, 0xFAD, 8);
    l.add_image_tiled(52, bottom - buttons * 22 + 1, 340, 80, 0xA40);
    l.add_image_tiled(53, bottom - buttons * 22 + 2, 338, 78, 0xBBC);
    l.add_text_entry(55, bottom - buttons * 22 + 2, 336, 78, 0x480, 0, "");
    buttons += 1;

    l.add_button_type(
        18,
        bottom - buttons * 22,
        0xFA5,
        0xFA7,
        0,
        GumpButtonType::Page,
        2,
    );
    l.add_label(52, bottom - buttons * 22, 2100, "Predefined Response");
    buttons += 1;

    if e.sender != m {
        l.add_button(18, bottom - buttons * 22, 0xFA5, 0xFA7, 1);
        l.add_label(52, bottom - buttons * 22, 2100, "Go to Sender");
        buttons += 1;
    }
    l.add_label(18, 58, 2100, "Handler:");
    match e.handler {
        None => {
            l.add_label_cropped(128, 58, 264, 20, 2100, "Unhandled");
            l.add_button(18, bottom - buttons * 22, 0xFB1, 0xFB3, 5);
            l.add_label(52, bottom - buttons * 22, 2100, "Delete Page");
            buttons += 1;
            l.add_button(18, bottom - buttons * 22, 0xFB7, 0xFB9, 4);
            l.add_label(52, bottom - buttons * 22, 2100, "Handle Page");
            buttons += 1;
        }
        Some(h) => {
            let hue = ACCESS_LEVEL_HUES[(mx::access_level(w, h) as usize).min(6)];
            l.add_label_cropped(128, 58, 264, 20, hue, &name_of(w, h));
            if h != m {
                l.add_button(18, bottom - buttons * 22, 0xFA5, 0xFA7, 2);
                l.add_label(52, bottom - buttons * 22, 2100, "Go to Handler");
                buttons += 1;
            } else {
                l.add_button(18, bottom - buttons * 22, 0xFA2, 0xFA4, 6);
                l.add_label(52, bottom - buttons * 22, 2100, "Abandon Page");
                buttons += 1;
                l.add_button(18, bottom - buttons * 22, 0xFB7, 0xFB9, 7);
                l.add_label(52, bottom - buttons * 22, 2100, "Page Handled");
                buttons += 1;
            }
        }
    }
    l.add_label(18, 78, 2100, "Page Location:");
    l.add_label_cropped(
        128,
        78,
        264,
        20,
        2100,
        &display_loc(w, e.page_location, e.page_map),
    );
    l.add_button(18, bottom - buttons * 22, 0xFA5, 0xFA7, 3);
    l.add_label(52, bottom - buttons * 22, 2100, "Go to Page Location");
    buttons += 1;
    if e.speech_log.is_some() {
        l.add_button(18, bottom - buttons * 22, 0xFA5, 0xFA7, 10);
        l.add_label(52, bottom - buttons * 22, 2100, "View Speech Log");
    }
    l.add_label(18, 98, 2100, "Page Type:");
    l.add_label_cropped(128, 98, 264, 20, 2100, get_page_type_name(e.ty));
    l.add_label(18, 118, 2100, "Message:");
    l.add_html(128, 118, 250, 100, &e.message, true, true);

    l.add_page(2);
    let pre: Vec<PredefinedResponse> = w
        .resources
        .get::<PredefinedResponses>()
        .and_then(|p| p.list.clone())
        .unwrap_or_else(|| load_responses(&responses_path(w)))
        .iter()
        .map(|r| r.borrow().clone())
        .collect();
    l.add_button_type(18, 18, 0xFAE, 0xFB0, 0, GumpButtonType::Page, 1);
    l.add_button(410 - 18 - 32, 18, 0xFAB, 0xFAC, 9);
    if pre.is_empty() {
        l.add_label(52, 18, 2100, "There are no predefined responses.");
    } else {
        l.add_label(52, 18, 2100, "Back");
        for (i, r) in pre.iter().enumerate() {
            let i = i as i32;
            l.add_button(18, 40 + i * 22, 0xFA5, 0xFA7, 100 + i);
            l.add_label(52, 40 + i * 22, 2100, &r.title);
        }
    }
    g
}

impl LegacyGumpLogic for PageEntryGump {
    const TYPE_NAME: &'static str = "Server.Engines.Help.PageEntryGump";
    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(me) = mobile_of(w, sender) else {
            return;
        };
        let b = info.button_id;
        let id = self.entry;
        let Some(e) = entry(w, id) else {
            if b != 0 {
                send_page_queue_gump(w, me);
                mx::send_message(w, me, "That page has been removed.");
            } else if pq_ref(w)
                .and_then(|q| q.gone_handlers.get(&id))
                .copied()
                .flatten()
                != Some(me)
            {
                // `m_Entry.Handler != state.Mobile`: the entry keeps its handler after it leaves the list
                send_page_queue_gump(w, me);
            }
            return;
        };
        let resend = |w: &mut World| {
            let g = page_entry_gump(w, self.mobile, e.clone());
            send(w, me, g);
        };
        let go = |w: &mut World, who: MobileId| -> bool {
            // true when `me` was moved
            let (deleted, map, loc) = match w.mobile(who) {
                None => (true, None, Point3D::default()),
                Some(m) => (false, m.map, m.location),
            };
            if deleted {
                mx::send_message(w, me, "That character no longer exists.");
            } else if map.is_none_or(|m| m == MapId::INTERNAL) {
                mx::send_message(w, me, "That character is not in the world.");
            } else {
                muo_server::mobile::base::move_to_world(w, me, loc, map.expect("checked"));
                return true;
            }
            false
        };
        match b {
            0 => {
                if e.handler != Some(me) {
                    send_page_queue_gump(w, me);
                }
            }
            1 => {
                if go(w, e.sender) {
                    mx::send_message(w, me, "You have been teleported to that page's sender.");
                    resend(w);
                }
            }
            2 => match e.handler {
                Some(h) => {
                    if go(w, h) {
                        mx::send_message(w, me, "You have been teleported to that page's handler.");
                        resend(w);
                    }
                }
                None => {
                    mx::send_message(w, me, "Nobody is handling that page.");
                    resend(w);
                }
            },
            3 => {
                if e.page_map.is_none_or(|m| m == MapId::INTERNAL) {
                    mx::send_message(w, me, "That location is not in the world.");
                } else {
                    muo_server::mobile::base::move_to_world(
                        w,
                        me,
                        e.page_location,
                        e.page_map.expect("checked"),
                    );
                    mx::send_message(
                        w,
                        me,
                        "You have been teleported to the original page location.",
                    );
                    resend(w);
                }
            }
            4 => {
                if e.handler.is_none() {
                    set_handler(w, id, Some(me));
                    mx::send_message(w, me, "You are now handling the page.");
                } else {
                    mx::send_message(w, me, "Someone is already handling that page.");
                }
                resend(w);
            }
            5 => {
                if e.handler.is_none() {
                    remove(w, id);
                    mx::send_message(w, me, "You delete the page.");
                    send_page_queue_gump(w, me);
                } else {
                    mx::send_message(
                        w,
                        me,
                        "Someone is handling that page, it can not be deleted.",
                    );
                    resend(w);
                }
            }
            6 => {
                if e.handler == Some(me) {
                    mx::send_message(w, me, "You abandon the page.");
                    set_handler(w, id, None);
                } else {
                    mx::send_message(w, me, "You are not handling that page.");
                }
                resend(w);
            }
            7 => {
                if e.handler == Some(me) {
                    remove(w, id);
                    pq(w).gone_handlers.insert(id, None); // `m_Entry.Handler = null` after the remove
                    mx::send_message(
                        w,
                        me,
                        "You mark the page as handled, and remove it from the queue.",
                    );
                    send_page_queue_gump(w, me);
                } else {
                    mx::send_message(w, me, "You are not handling that page.");
                    resend(w);
                }
            }
            8 => {
                if let Some(text) = info.get_text_entry(0) {
                    let name = name_of(w, me);
                    let g = message_sent_gump(w, e.sender, &name, text);
                    send(w, e.sender, g);
                }
                resend(w);
            }
            9 => {
                resend(w);
                let g = predef_gump(w, me, None);
                send(w, me, g);
            }
            10 => {
                resend(w);
                if let Some(log) = e.speech_log.clone() {
                    let g = speech_log_gump(w, e.sender, log);
                    send(w, me, g);
                }
            }
            _ => {
                let index = b - 100;
                let pre: Vec<PredefinedResponse> =
                    responses(w).iter().map(|r| r.borrow().clone()).collect();
                if index >= 0 && (index as usize) < pre.len() {
                    let name = name_of(w, me);
                    let g = message_sent_gump(w, e.sender, &name, &pre[index as usize].message);
                    send(w, e.sender, g);
                }
                resend(w);
            }
        }
    }
}

// ---- SpeechLog -----------------------------------------------------------------------------------------------------

const MAX_ENTRIES_PER_PAGE: usize = 30;

pub struct SpeechLogGump {
    player: MobileId,
    log: Vec<SpeechLogEntry>,
    page: i64,
}

pub fn speech_log_gump(
    w: &World,
    player: MobileId,
    log: Vec<SpeechLogEntry>,
) -> LegacyGumpObj<SpeechLogGump> {
    let last = (log.len() as i64 - 1).div_euclid(MAX_ENTRIES_PER_PAGE as i64);
    speech_log_gump_page(w, player, log, last)
}

pub fn speech_log_gump_page(
    w: &World,
    player: MobileId,
    log: Vec<SpeechLogEntry>,
    page: i64,
) -> LegacyGumpObj<SpeechLogGump> {
    let mut g = LegacyGumpObj::new(
        500,
        30,
        SpeechLogGump {
            player,
            log: log.clone(),
            page,
        },
    );
    let l = &mut g.layout;
    l.add_image_tiled(0, 0, 300, 425, 0xA40);
    l.add_alpha_region(1, 1, 298, 423);
    let player_name = name_of(w, player);
    let account = account_name(w, player).unwrap_or_else(|| "???".into());
    l.add_html(
        10,
        10,
        280,
        20,
        &html::center_rgb(
            &format!("SPEECH LOG - {player_name} (<i>{}</i>)", fix_html(&account)),
            0xA0A0FF,
        ),
        false,
        false,
    );
    let last_page = (log.len() as i64 - 1).div_euclid(MAX_ENTRIES_PER_PAGE as i64);
    let s_log = if page < 0 || page > last_page {
        String::new()
    } else {
        let max = log.len() as i64 - (last_page - page) * MAX_ENTRIES_PER_PAGE as i64;
        let min = (max - MAX_ENTRIES_PER_PAGE as i64).max(0);
        let mut out = String::new();
        for i in min..max {
            let e = &log[i as usize];
            let acct = account_name(w, e.from).unwrap_or_else(|| "???".into());
            if i != min {
                out.push_str("<br>");
            }
            out.push_str(&format!(
                "<u>{}</u> (<i>{}</i>): {}",
                name_of(w, e.from),
                fix_html(&acct),
                fix_html(&e.speech)
            ));
        }
        out
    };
    l.add_html(10, 40, 280, 350, &s_log, false, true);
    if page > 0 {
        l.add_button(10, 395, 0xFAE, 0xFB0, 1);
    }
    l.add_label(
        45,
        395,
        0x481,
        &format!("Current page: {}/{}", page + 1, last_page + 1),
    );
    if page < last_page {
        l.add_button(261, 395, 0xFA5, 0xFA7, 2);
    }
    g
}

impl LegacyGumpLogic for SpeechLogGump {
    const TYPE_NAME: &'static str = "Server.Engines.Help.SpeechLogGump";
    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        match info.button_id {
            1 if self.page > 0 => {
                let g = speech_log_gump_page(w, self.player, self.log.clone(), self.page - 1);
                send(w, from, g);
            }
            2 if (self.page + 1) * (MAX_ENTRIES_PER_PAGE as i64) < self.log.len() as i64 => {
                let g = speech_log_gump_page(w, self.player, self.log.clone(), self.page + 1);
                send(w, from, g);
            }
            _ => {}
        }
    }
}

fn speech_log_command(w: &mut World, e: &mut CommandEventArgs) {
    let from = e.mobile;
    mx::send_message(w, from, "Target a player to view his speech log.");
    target::begin_target(w, from, -1, false, TargetFlags::NONE, |w, from, t| {
        let Targeted::Entity(EntityId::Mobile(pm)) = t else {
            mx::send_message(w, from, "Speech logs aren't supported on that target.");
            return;
        };
        let Some(d) = player_mobile::data(w, pm) else {
            mx::send_message(w, from, "Speech logs aren't supported on that target.");
            return;
        };
        let female = w.mobile(pm).is_some_and(|m| m.female);
        let (their_level, my_level) = (mx::access_level(w, pm), mx::access_level(w, from));
        let (his, he) = if female {
            ("her", "She")
        } else {
            ("his", "He")
        };
        if from != pm && my_level <= their_level && my_level != AccessLevel::Owner {
            mx::send_message(
                w,
                from,
                &format!("You don't have the required access level to view {his} speech log."),
            );
        } else if d.speech_log.is_none() {
            mx::send_message(w, from, &format!("{he} has no speech log."));
        } else {
            let log: Vec<SpeechLogEntry> = d
                .speech_log
                .as_ref()
                .map(|l| l.iter().cloned().collect())
                .unwrap_or_default();
            let text = format!(
                "{:?} {} viewing speech log of {}",
                my_level,
                muo_content::targets::format_entity(w, EntityId::Mobile(from)),
                muo_content::targets::format_entity(w, EntityId::Mobile(pm))
            );
            muo_content::targets::command_log(w, from, &text);
            let g = speech_log_gump(w, pm, log);
            send(w, from, g);
        }
    });
}

// ---- StuckMenu -----------------------------------------------------------------------------------------------------

struct StuckMenuEntry {
    name: i32,
    locations: &'static [Point3D],
}

const fn p(x: i32, y: i32, z: i32) -> Point3D {
    Point3D { x, y, z }
}

static ENTRIES: [StuckMenuEntry; 6] = [
    StuckMenuEntry {
        name: 1011028,
        locations: &[
            p(1522, 1757, 28),
            p(1519, 1619, 10),
            p(1457, 1538, 30),
            p(1607, 1568, 20),
            p(1643, 1680, 18),
        ],
    },
    StuckMenuEntry {
        name: 1011029,
        locations: &[
            p(2005, 2754, 30),
            p(1993, 2827, 0),
            p(2044, 2883, 0),
            p(1876, 2859, 20),
            p(1865, 2687, 0),
        ],
    },
    StuckMenuEntry {
        name: 1011030,
        locations: &[
            p(2973, 891, 0),
            p(3003, 776, 0),
            p(2910, 727, 0),
            p(2865, 804, 0),
            p(2832, 927, 0),
        ],
    },
    StuckMenuEntry {
        name: 1011031,
        locations: &[
            p(2498, 392, 0),
            p(2433, 541, 0),
            p(2445, 501, 15),
            p(2501, 469, 15),
            p(2444, 420, 15),
        ],
    },
    StuckMenuEntry {
        name: 1011032,
        locations: &[
            p(490, 1166, 0),
            p(652, 1098, 0),
            p(650, 1013, 0),
            p(536, 979, 0),
            p(464, 970, 0),
        ],
    },
    StuckMenuEntry {
        name: 1011033,
        locations: &[
            p(2230, 1159, 0),
            p(2218, 1203, 0),
            p(2247, 1194, 0),
            p(2236, 1224, 0),
            p(2273, 1231, 0),
        ],
    },
];

static T2A_ENTRIES: [StuckMenuEntry; 2] = [
    StuckMenuEntry {
        name: 1011057,
        locations: &[
            p(5720, 3109, -1),
            p(5677, 3176, -3),
            p(5678, 3227, 0),
            p(5769, 3206, -2),
            p(5777, 3270, -1),
        ],
    },
    StuckMenuEntry {
        name: 1011058,
        locations: &[
            p(5216, 4033, 37),
            p(5262, 4049, 37),
            p(5284, 4006, 37),
            p(5189, 3971, 39),
            p(5243, 3960, 37),
        ],
    },
];

/// `StuckMenu`'s timers, by the mobile they act on.
#[derive(Default)]
pub struct StuckMenuState {
    close_timers: IndexMap<MobileId, TimerHandle>,
}

fn is_in_second_age_area(w: &World, m: MobileId) -> bool {
    let Some(mm) = w.mobile(m) else { return false };
    (mm.map == Some(MapId::TRAMMEL) || mm.map == Some(MapId::FELUCCA))
        && (mm.location.x >= 5120 && mm.location.y >= 2304
            || muo_server::region::is_part_of_name(w, mm.region, "Terathan Keep", false))
}

fn entries_for(w: &World, m: MobileId) -> &'static [StuckMenuEntry] {
    if is_in_second_age_area(w, m) {
        &T2A_ENTRIES
    } else {
        &ENTRIES
    }
}

pub struct StuckMenu {
    mobile: MobileId,
    sender: MobileId,
    mark_use: bool,
}

/// `new StuckMenu(beholder, beheld, markUse)`.
pub fn stuck_menu(
    w: &World,
    beholder: MobileId,
    beheld: MobileId,
    mark_use: bool,
) -> LegacyGumpObj<StuckMenu> {
    let entries = entries_for(w, beheld);
    let mut g = LegacyGumpObj::new(
        150,
        50,
        StuckMenu {
            mobile: beheld,
            sender: beholder,
            mark_use,
        },
    );
    let l = &mut g.layout;
    l.closable = false;
    l.draggable = false;
    l.disposable = false;
    l.add_background(0, 0, 270, 320, 2600);
    l.add_html_localized(50, 20, 250, 35, 1011027, false, false); // Chose a town:
    for (i, e) in entries.iter().enumerate() {
        let i = i as i32;
        l.add_button(50, 55 + 35 * i, 208, 209, i + 1);
        l.add_html_localized(75, 55 + 35 * i, 335, 40, e.name, false, false);
    }
    l.add_button(55, 263, 4005, 4007, 0);
    l.add_html_localized(90, 265, 200, 35, 1011012, false, false); // CANCEL
    g
}

/// `StuckMenu.BeginClose()`.
pub fn begin_close(w: &mut World, m: MobileId) {
    stop_close(w, m);
    let end = w.now + TimeSpan::from_mins_f64(3.0);
    let h = timer::start(
        w,
        TimeSpan::ZERO,
        TimeSpan::from_secs_f64(1.0),
        0,
        move |w| {
            let gone = props_b::net_state(w, m).is_none();
            if gone || w.now > end {
                mx::set_frozen(w, m, false);
                if let Some(ns) = props_b::net_state(w, m) {
                    close_gump::<StuckMenu>(w, ns);
                }
                if let Some(mut h) = w
                    .resources
                    .get_mut::<StuckMenuState>()
                    .close_timers
                    .shift_remove(&m)
                {
                    h.cancel(w);
                }
            } else {
                mx::set_frozen(w, m, true);
            }
        },
    );
    w.resources
        .get_mut::<StuckMenuState>()
        .close_timers
        .insert(m, h);
    mx::set_frozen(w, m, true);
}

/// `StuckMenu.StopClose()`.
pub fn stop_close(w: &mut World, m: MobileId) {
    if let Some(mut h) = w
        .resources
        .get_mut::<StuckMenuState>()
        .close_timers
        .shift_remove(&m)
    {
        h.cancel(w);
    }
    mx::set_frozen(w, m, false);
}

impl LegacyGumpLogic for StuckMenu {
    const TYPE_NAME: &'static str = "Server.Menus.Questions.StuckMenu";
    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        let m = self.mobile;
        stop_close(w, m);
        if sigil_exists_on(w, m) {
            send_localized_message(w, m, 1061632, "", 0x3B2); // You can't do that while carrying the sigil.
        } else if info.button_id == 0 && m == self.sender {
            send_localized_message(w, m, 1010588, "", 0x3B2); // You choose not to go to any city.
        }
        let index = info.button_id - 1;
        let entries = entries_for(w, m);
        if index >= 0 && (index as usize) < entries.len() {
            teleport(w, m, &entries[index as usize], self.mark_use);
        }
        for f in events(w).stuck_menu.clone() {
            f(w, m);
        }
    }
}

fn teleport(w: &mut World, m: MobileId, entry: &'static StuckMenuEntry, mark_use: bool) {
    let delay = if mark_use {
        send_localized_message(w, m, 1010589, "", 0x3B2); // You will be teleported within the next two minutes.
        let d = TimeSpan::from_secs_f64(
            10.0 + muo_primitives::utility::random_double(&mut w.rng) * 110.0,
        );
        if player_mobile::data(w, m).is_some() {
            player_mobile::used_stuck_menu(w, m);
        }
        d
    } else {
        TimeSpan::ZERO
    };
    let end = w.now + delay;
    let slot = std::rc::Rc::new(std::cell::Cell::new(None::<TimerHandle>));
    let slot2 = slot.clone();
    let h = timer::start(
        w,
        TimeSpan::ZERO,
        TimeSpan::from_secs_f64(1.0),
        0,
        move |w| {
            if w.now < end {
                mx::set_frozen(w, m, true);
                return;
            }
            mx::set_frozen(w, m, false);
            if let Some(mut t) = slot2.get() {
                t.cancel(w);
            }
            if sigil_exists_on(w, m) {
                send_localized_message(w, m, 1061632, "", 0x3B2);
                return;
            }
            let Some(&dest) = muo_primitives::utility::random_element(&mut w.rng, entry.locations)
            else {
                return;
            };
            let cur = w.mobile(m).and_then(|x| x.map);
            let dest_map = if cur == Some(MapId::TRAMMEL) {
                MapId::TRAMMEL
            } else if cur == Some(MapId::FELUCCA) || muo_server::mobile::base::murderer(w, m) {
                MapId::FELUCCA
            } else {
                MapId::TRAMMEL
            };
            muo_content::creature::pets::teleport_pets(w, m, dest, dest_map, false);
            muo_server::mobile::base::move_to_world(w, m, dest, dest_map);
        },
    );
    slot.set(Some(h));
}

// ---- registration --------------------------------------------------------------------------------------------------

/// `PageQueue.Configure`, `SpeechLog.Configure`, and the hooks the packet handlers and commands call.
pub fn configure(w: &mut World) {
    CommandSystem::register(
        w,
        "Pages",
        AccessLevel::Counselor,
        &[],
        Rc::new(pages_command),
    );
    CommandSystem::register(
        w,
        "SpeechLog",
        AccessLevel::Counselor,
        &[],
        Rc::new(speech_log_command),
    );
    // PageDiscord.Configure
    let url: Option<String> = w
        .config
        .get_or_update_nullable_setting("pages.discordWebhookUrl", None)
        .ok()
        .flatten();
    if url.as_deref().is_some_and(|u| !u.is_empty()) {
        info!("Relaying pages to discord enabled.");
    }
    w.resources.get_mut::<PageDiscord>().webhook_url = url;

    let h = w.resources.get_mut::<HelpHooks>();
    h.discord_post = Some(|_, url, body| muo_content::http_tls::post_json_detached(url, body));
    h.queue_email = Some(send_queue_email);
    h.sigil_exists_on = Some(crate::factions::items::exists_on);

    w.resources
        .get_mut::<muo_content::hooks::ContentHooks>()
        .page_account_password = Some(|w, from, text| {
        if let Some(en) = get_entry(w, from) {
            let m = if en.message.starts_with("[Automated: Change Password]") {
                "You already have a password change request in the help system queue."
            } else {
                "Your IP address does not match that which created this account."
            };
            mx::send_message(w, from, m);
        } else if check_allowed_to_page(w, from) {
            mx::send_message(
                w,
                from,
                "Your IP address does not match that which created this account.  A page has been entered into the help system on your behalf.",
            );
            // The next available Counselor/Game Master will respond as soon as possible.
            send_localized_message(w, from, 501234, "", 0x35);
            let e = new_page_entry(w, from, text, PageType::Account);
            enqueue(w, e);
        }
    });

    let c = w.resources.get_mut::<muo_commands::hooks::CommandHooks>();
    c.message_sent_gump = Some(|w, to, name, text| {
        let g = message_sent_gump(w, to, name, text);
        send(w, to, g);
    });
    c.stuck_menu = Some(|w, from, mobile| {
        let g = stuck_menu(w, from, mobile, false);
        send(w, from, g);
    });
    c.page_handler = Some(|w, m| get_entry(w, m).and_then(|e| e.handler));
    w.resources
        .get_mut::<muo_content::network::incoming_game::GameHooks>()
        .help_request = Some(help_request);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_topic_packet_layout() {
        assert_eq!(
            display_help_topic_packet(29, true),
            [0xBF, 0, 11, 0, 0x17, 1, 0, 0, 0, 29, 1]
        );
    }

    #[test]
    fn discord_colors_and_empty_messages_follow_page_discord() {
        assert_eq!(page_type_color(PageType::Stuck), 0xFFFF00);
        assert_eq!(page_type_color(PageType::Other), 0x888888);
        assert_eq!(page_type_color(PageType::Bug), 0xFF0000);
        assert_eq!(truncate_message(""), "*(no message)*");
        assert_eq!(truncate_message(&"x".repeat(1001)).len(), 1000);
        assert!(truncate_message(&"x".repeat(1001)).ends_with("..."));
    }

    /// `PageEntryGump` holds the `PageEntry`, which keeps its `Handler` after `PageQueue.Remove`: closing the gump of
    /// a gone entry reopens the queue only when the closer is not the handler.
    #[test]
    fn a_removed_entry_keeps_its_handler_for_the_close_check() {
        use muo_server::mobile::MOBILE_BASE;
        let mut w = World::new();
        let (sender, handler) = (w.new_mobile(&MOBILE_BASE), w.new_mobile(&MOBILE_BASE));
        let now = w.now;
        pq(&mut w).list.push(PageEntry {
            id: 7,
            sender,
            handler: Some(handler),
            sent: now,
            message: String::new(),
            ty: PageType::Other,
            page_location: Point3D::default(),
            page_map: None,
            speech_log: None,
            timer: None,
        });
        remove(&mut w, 7);
        let gone = pq_ref(&w).unwrap().gone_handlers.get(&7).copied().flatten();
        assert_eq!(gone, Some(handler));
        assert_ne!(gone, Some(sender));
    }
    #[test]
    fn page_type_names() {
        assert_eq!(
            get_page_type_name(PageType::VerbalHarassment),
            "Verbal Harassment"
        );
        assert_eq!(get_page_type_name(PageType::Bug), "Bug");
    }

    #[test]
    fn predefined_responses_parse_tab_lines() {
        let dir = std::env::temp_dir().join(format!("muo_help_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("pageresponse.cfg");
        std::fs::write(&p, "# c\n\nHello\tHi there\nbad line\nA\tB\tC\n").unwrap();
        let r = load_responses(&p);
        assert_eq!(r.len(), 1);
        assert_eq!(
            *r[0].borrow(),
            PredefinedResponse {
                title: "Hello".into(),
                message: "Hi there".into()
            }
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// PredefGump holds the `PredefinedResponse` object: when the list changes between the send and the response
    /// the buttons act on that object, not on whatever sits at its old index.
    #[test]
    fn predef_gump_holds_its_response_by_reference() {
        use muo_server::mobile::{AccessLevel, MOBILE_BASE};
        let dir = std::env::temp_dir().join(format!("muo_predef_{}", std::process::id()));
        std::fs::create_dir_all(dir.join("Data")).unwrap();
        let mut w = World::new();
        w.core.base_directory = dir.clone();
        let admin = w.new_mobile(&MOBILE_BASE);
        w.mobile_mut(admin).unwrap().access_level = AccessLevel::Administrator;
        let list = vec![
            new_response("a", "1"),
            new_response("b", "2"),
            new_response("c", "3"),
        ];
        w.resources.get_mut::<PredefinedResponses>().list = Some(list);
        let ns = NetStateId(1);
        let press = |w: &mut World,
                     g: &mut LegacyGumpObj<PredefGump>,
                     button: i32,
                     texts: Vec<(u16, String)>| {
            g.logic
                .on_response(w, ns, &RelayInfo::new(button, vec![], texts));
        };
        let titles = |w: &mut World| {
            responses(w)
                .iter()
                .map(|r| r.borrow().title.clone())
                .collect::<Vec<_>>()
        };
        // the editor of "b" (index 1); a new first entry shifts it to index 2
        let mut g = predef_gump(&mut w, admin, Some(1));
        let first = new_response("new", "0");
        responses(&mut w).insert(0, first);
        press(&mut w, &mut g, 2, vec![(0, "B2".into())]);
        assert_eq!(titles(&mut w), ["new", "a", "B2", "c"]);
        press(&mut w, &mut g, 3, vec![(1, "msg".into())]);
        assert_eq!(responses(&mut w)[2].borrow().message, "msg");
        // removed behind its back: Remove does nothing (list.Remove(obj)), a title edit lands on the detached object
        let gone = responses(&mut w).remove(2);
        press(&mut w, &mut g, 2, vec![(0, "detached".into())]);
        assert_eq!(gone.borrow().title, "detached");
        press(&mut w, &mut g, 1, vec![]);
        assert_eq!(titles(&mut w), ["new", "a", "c"]);
        // Remove takes it out of the list wherever it moved
        let mut g = predef_gump(&mut w, admin, Some(0));
        let moved = responses(&mut w).remove(0);
        responses(&mut w).push(moved);
        press(&mut w, &mut g, 1, vec![]);
        assert_eq!(titles(&mut w), ["a", "c"]);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
