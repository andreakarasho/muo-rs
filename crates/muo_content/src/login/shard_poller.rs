//! The `ShardPoller` item an administrator places to ask players a
//! question (`ShardPollOption`s, one vote per class C address), the `ShardPollGump` that shows and edits it, and the
//! `ShardPollPrompt` that edits option titles. Active polls are shown to every player who logs in and has not voted.
//!
//! Saves: `Server.Misc.ShardPoller` is the item part, a version int 1, `Title`, `Duration`, `StartTime`, `Active`,
//! then the encoded option count and each option (version int 1, title, encoded voter count, the addresses).
//!
//! `PropertiesGump` (the editor's button 2) is a hook ([`ShardPollerHooks`]); the `[CommandProperty]` members are the
//! [`PROPS`] table.

use std::collections::VecDeque;
use std::net::IpAddr;
use std::rc::Rc;

use muo_primitives::net::match_class_c;
use muo_primitives::{DateTime, TimeSpan, html};
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{ItemType, base, ops};
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part2::messages::send_message;
use muo_server::net_state::NetStateId;
use muo_server::prompt::{self, Prompt};
use muo_server::timer;
use muo_server::type_registry::TypeRegistry;

use crate::gumps::builder::DynamicGumpBuilder;
use crate::gumps::{DynamicGumpLogic, DynamicGumpObj, RelayInfo, send_gump_to_mobile};
use crate::network::events::{self, PlayerLogin};

const LABEL_COLOR32: i32 = 0xFFFFFF;

/// `ShardPollOption`.
#[derive(Clone, Debug, Default)]
pub struct ShardPollOption {
    pub title: Option<String>,
    pub voters: Vec<IpAddr>,
}

impl ShardPollOption {
    /// `Votes`.
    pub fn votes(&self) -> usize {
        self.voters.len()
    }

    /// `HasAlreadyVoted(ns)`: a voter from the same class C network.
    pub fn has_already_voted(&self, address: Option<IpAddr>) -> bool {
        address.is_some_and(|a| self.voters.iter().any(|v| match_class_c(*v, a)))
    }

    /// `GetBreaks()`: the lines of the title (`<br>` count plus one); a missing title is one.
    pub fn breaks(&self) -> i32 {
        match &self.title {
            None => 1,
            Some(t) => t.matches("<br>").count() as i32 + 1,
        }
    }

    /// `ComputeHeight()`.
    pub fn compute_height(&self) -> i32 {
        (self.breaks() * 18).max(30)
    }
}

/// What `ShardPoller` adds to `Item`.
#[derive(Clone, Debug)]
pub struct ShardPollerData {
    pub title: Option<String>,
    pub duration: TimeSpan,
    pub start_time: DateTime,
    pub active: bool,
    pub options: Vec<ShardPollOption>,
}

impl Default for ShardPollerData {
    fn default() -> Self {
        Self {
            title: None,
            duration: TimeSpan::from_hours(24),
            start_time: DateTime::MIN,
            active: false,
            options: Vec::new(),
        }
    }
}

/// `_activePollers`.
#[derive(Default)]
struct ActivePollers(Vec<ItemId>);

/// The one system the editor reaches that is not in this crate: `new PropertiesGump(from, poller)`.
#[derive(Default)]
pub struct ShardPollerHooks {
    pub properties_gump: Option<fn(&mut World, MobileId, ItemId)>,
}

fn data(w: &World, id: ItemId) -> Option<&ShardPollerData> {
    w.item(id)?.ext.get::<ShardPollerData>()
}

fn data_mut(w: &mut World, id: ItemId) -> Option<&mut ShardPollerData> {
    w.item_mut(id)?.ext.get_mut::<ShardPollerData>()
}

fn deleted(w: &World, id: ItemId) -> bool {
    w.item(id).is_none_or(|i| i.deleted())
}

/// `new ShardPoller()`.
pub fn new_shard_poller(w: &mut World) -> ItemId {
    let id = ops::new_item_with_id(w, &SHARD_POLLER, 0x1047);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(ShardPollerData::default());
    }
    ops::set_movable(w, id, false);
    id
}

/// `TimeRemaining`.
pub fn time_remaining(w: &World, id: ItemId) -> TimeSpan {
    match data(w, id) {
        Some(d) if d.start_time != DateTime::MIN && d.active => {
            (d.start_time + d.duration - w.now).max(TimeSpan::ZERO)
        }
        _ => TimeSpan::ZERO,
    }
}

/// `Title = value`: urls become links, and the item is marked dirty if it changed.
pub fn set_title(w: &mut World, id: ItemId, value: Option<&str>) {
    let value = value.map(url_to_href);
    if let Some(d) = data_mut(w, id)
        && d.title != value
    {
        d.title = value;
    }
}

/// `Active = value` and `OnActiveChanged`: starting a poll stamps its start time and lists it.
pub fn set_active(w: &mut World, id: ItemId, value: bool) {
    let now = w.now;
    let Some(d) = data_mut(w, id) else { return };
    if d.active == value {
        return;
    }
    d.active = value;
    if value {
        d.start_time = now;
    }
    let list = &mut w.resources.get_mut::<ActivePollers>().0;
    if value {
        list.push(id);
    } else {
        list.retain(|p| *p != id);
    }
}

/// `HasAlreadyVoted(ns)`.
pub fn has_already_voted(w: &World, id: ItemId, ns: NetStateId) -> bool {
    let address = w.net_state(ns).and_then(|d| d.address);
    data(w, id).is_some_and(|d| d.options.iter().any(|o| o.has_already_voted(address)))
}

/// `AddVote(ns, option)`.
fn add_vote(w: &mut World, id: ItemId, ns: NetStateId, option: usize) {
    let Some(address) = w.net_state(ns).and_then(|d| d.address) else {
        return;
    };
    if let Some(o) = data_mut(w, id).and_then(|d| d.options.get_mut(option)) {
        o.voters.push(address);
    }
}

fn remove_option(w: &mut World, id: ItemId, index: usize) {
    if let Some(d) = data_mut(w, id)
        && index < d.options.len()
    {
        d.options.remove(index);
    }
}

fn add_option(w: &mut World, id: ItemId, title: Option<String>) {
    if let Some(d) = data_mut(w, id) {
        d.options.push(ShardPollOption {
            title,
            voters: Vec::new(),
        });
    }
}

/// `OnLogin(pm)`.
fn on_login(w: &mut World, e: &mut PlayerLogin) {
    if w.resources.get_mut::<ActivePollers>().0.is_empty() {
        return;
    }
    let pm = e.0;
    timer::start(w, TimeSpan::from_secs(1), TimeSpan::ZERO, 1, move |w| {
        login_callback(w, pm)
    });
}

/// `EventSink_Login_Callback(from)`.
fn login_callback(w: &mut World, from: MobileId) {
    let Some(ns) = w.mobile(from).and_then(|m| m.net_state) else {
        return;
    };
    let pollers = w.resources.get_mut::<ActivePollers>().0.clone();
    let mut shown = false;
    for poller in pollers {
        let active = data(w, poller).is_some_and(|d| d.active);
        if deleted(w, poller) || !active {
            continue;
        }
        if time_remaining(w, poller) > TimeSpan::ZERO {
            if has_already_voted(w, poller, ns) {
                continue;
            }
            if !shown {
                shown = ShardPollGump::display_to(w, from, poller, false, VecDeque::new());
            } else if let Some(g) = crate::gumps::system::find_gump_mut::<ShardPollGump>(w, ns)
                && let Some(g) = g
                    .as_any_mut()
                    .downcast_mut::<DynamicGumpObj<ShardPollGump>>()
            {
                g.logic.polls.push_back(poller); // QueuePoll
            }
        } else {
            set_active(w, poller, false);
        }
    }
}

/// `ShardPollGump`.
pub struct ShardPollGump {
    from: MobileId,
    poller: ItemId,
    editing: bool,
    polls: VecDeque<ItemId>,
    /// The poll's data when the gump was built, for `BuildLayout`.
    snapshot: ShardPollerData,
}

impl ShardPollGump {
    /// `DisplayTo(from, poller, editing, polls)`: false when nothing was sent.
    pub fn display_to(
        w: &mut World,
        from: MobileId,
        poller: ItemId,
        editing: bool,
        polls: VecDeque<ItemId>,
    ) -> bool {
        if w.mobile(from).is_none_or(|m| m.net_state.is_none()) || deleted(w, poller) {
            return false;
        }
        let snapshot = data(w, poller).cloned().unwrap_or_default();
        let gump = DynamicGumpObj::new(
            50,
            50,
            ShardPollGump {
                from,
                poller,
                editing,
                polls,
                snapshot,
            },
        );
        send_gump_to_mobile(w, from, Box::new(gump), false);
        true
    }

    /// `from.SendGump(this)`: the same gump again, rebuilt from the poll as it is now.
    fn resend(&mut self, w: &mut World) {
        let polls = std::mem::take(&mut self.polls);
        Self::display_to(w, self.from, self.poller, self.editing, polls);
    }
}

impl DynamicGumpLogic for ShardPollGump {
    const TYPE_NAME: &'static str = "Server.Misc.ShardPollGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        let poller = &self.snapshot;
        let editing = self.editing;

        b.set_no_close();
        b.add_page(0);

        let mut total_votes = 0;
        let mut total_option_height = 0;
        for o in &poller.options {
            total_votes += o.votes() as i32;
            total_option_height += o.compute_height() + 5;
        }

        let is_viewing_results = editing && poller.active;
        let is_completed = total_votes > 0 && !poller.active;
        if editing && !is_viewing_results {
            total_option_height += 35;
        }
        let height = 115 + total_option_height;

        b.add_background(1, 1, 398, height - 2, 3600);
        b.add_alpha_region(16, 15, 369, height - 31);
        b.add_item(308, 30, 0x1E5E, 0);

        let title = if editing {
            if is_completed {
                "Poll Completed"
            } else {
                "Poll Editor"
            }
        } else {
            "Shard Poll"
        };
        b.add_html(
            22,
            22,
            294,
            20,
            &html::center_rgb(title, LABEL_COLOR32),
            false,
            false,
        );
        if editing {
            b.add_html(
                22,
                22,
                294,
                20,
                &html::color_rgb(&format!("{total_votes} total"), LABEL_COLOR32),
                false,
                false,
            );
            b.add_button(287, 23, 0x2622, 0x2623, 2);
        }
        b.add_html(
            22,
            50,
            294,
            40,
            &html::color_rgb(poller.title.as_deref().unwrap_or_default(), 0x99CC66),
            false,
            false,
        );
        b.add_image_tiled(32, 88, 264, 1, 9107);
        b.add_image_tiled(42, 90, 264, 1, 9157);

        let mut y = 100;
        for (i, option) in poller.options.iter().enumerate() {
            let mut text = option.title.clone().unwrap_or_default();
            if editing && total_votes > 0 {
                let perc = option.votes() as f64 / f64::from(total_votes);
                text = format!("[{}: {}%] {text}", option.votes(), (perc * 100.0) as i32);
            }
            let opt_height = option.compute_height();
            y += opt_height / 2;
            if is_viewing_results {
                b.add_image(24, y - 15, 0x25FE, 0);
            } else {
                b.add_radio(24, y - 15, 0x25F9, 0x25FC, false, 1 + i as i32);
            }
            let line_breaks = option.breaks();
            b.add_html(
                60,
                y - 9 * line_breaks,
                250,
                18 * line_breaks,
                &html::color_rgb(&text, LABEL_COLOR32),
                false,
                false,
            );
            y += opt_height / 2;
            y += 5;
        }

        if editing && !is_viewing_results {
            b.add_radio(
                24,
                y + 15 - 15,
                0x25F9,
                0x25FC,
                false,
                1 + poller.options.len() as i32,
            );
            b.add_html(
                60,
                y + 15 - 9,
                250,
                18,
                &html::color_rgb("Create new option.", 0x99CC66),
                false,
                false,
            );
        }
        b.add_button(314, height - 73, 247, 248, 1);
        b.add_button(314, height - 47, 242, 241, 0);
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        if let Some(next) = self.polls.pop_front() {
            let (from, rest) = (self.from, self.polls.clone());
            timer::start(w, TimeSpan::from_secs(1), TimeSpan::ZERO, 1, move |w| {
                ShardPollGump::display_to(w, from, next, false, rest.clone());
            });
        }

        let options = data(w, self.poller).map_or(0, |d| d.options.len());
        let active = data(w, self.poller).is_some_and(|d| d.active);
        if info.button_id == 1 {
            let Some(&first) = info.switches.first() else {
                return;
            };
            let switched = first - 1;
            let opt = (switched >= 0 && (switched as usize) < options).then_some(switched as usize);
            if opt.is_none() && !self.editing {
                return;
            }
            if self.editing {
                if !active {
                    if opt.is_none() {
                        send_message(
                            w,
                            self.from,
                            "Enter a title for the option. Escape to cancel.",
                        );
                    } else {
                        send_message(
                            w,
                            self.from,
                            "Enter a title for the option. Escape to cancel. Use \"DEL\" to delete.",
                        );
                    }
                    set_option_prompt(w, self.from, self.poller, opt);
                } else {
                    send_message(
                        w,
                        self.from,
                        "You may not edit an active poll. Deactivate it first.",
                    );
                    self.resend(w);
                }
            } else if !active {
                send_message(w, self.from, "The poll has been deactivated.");
            } else if has_already_voted(w, self.poller, sender) {
                send_message(w, self.from, "You have already voted on this poll.");
            } else if let Some(o) = opt {
                add_vote(w, self.poller, sender, o);
            }
        } else if info.button_id == 2 && self.editing {
            self.resend(w);
            if let Some(f) = w
                .resources
                .get::<ShardPollerHooks>()
                .and_then(|h| h.properties_gump)
            {
                f(w, self.from, self.poller);
            }
        }
    }
}

/// `from.Prompt = new ShardPollPrompt(poller, opt)`.
fn set_option_prompt(w: &mut World, from: MobileId, poller: ItemId, option: Option<usize>) {
    let p = Prompt {
        on_cancel: Some(Rc::new(move |w, from| {
            ShardPollGump::display_to(w, from, poller, true, VecDeque::new());
        })),
        on_response: Some(Rc::new(move |w, from, text| {
            prompt_response(w, from, poller, option, text)
        })),
    };
    let id = prompt::add_prompt(w, p);
    muo_server::mobile::part1::props_b::set_prompt(w, from, Some(id));
}

/// `ShardPollPrompt.OnResponse(from, text)`.
fn prompt_response(
    w: &mut World,
    from: MobileId,
    poller: ItemId,
    option: Option<usize>,
    text: &str,
) {
    let active = data(w, poller).is_some_and(|d| d.active);
    if active {
        send_message(
            w,
            from,
            "You may not edit an active poll. Deactivate it first.",
        );
    } else if text == "DEL" {
        if let Some(o) = option {
            remove_option(w, poller, o);
        }
    } else {
        let text = url_to_href(text);
        match option {
            None => add_option(w, poller, Some(text)),
            Some(o) => {
                if let Some(opt) = data_mut(w, poller).and_then(|d| d.options.get_mut(o)) {
                    opt.title = Some(text);
                }
            }
        }
    }
    ShardPollGump::display_to(w, from, poller, true, VecDeque::new());
}

/// `UrlToHref(text)`: `[url=link]text[/url]` and `[url]link[/url]` (any case) become anchors.
pub fn url_to_href(text: &str) -> String {
    let lower = text.to_ascii_lowercase();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < text.len() {
        if lower[i..].starts_with("[url")
            && let Some((end, g1, g2)) = match_url(text, &lower, i)
        {
            match g1 {
                Some(g1) => out.push_str(&format!("<a href=\"{g1}\">{g2}</a>")),
                None => out.push_str(&format!("<a href=\"{g2}\">{g2}</a>")),
            }
            i = end;
            continue;
        }
        let ch = text[i..].chars().next().unwrap_or(' ');
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

/// The regex `\[url(?:=(.*?))?\](.*?)\[/url\]` at `at`: the end of the match, group 1 and group 2 (`.` excludes
/// a line feed, both groups are lazy).
fn match_url<'a>(
    text: &'a str,
    lower: &str,
    at: usize,
) -> Option<(usize, Option<&'a str>, &'a str)> {
    let rest = at + 4;
    let close = |from: usize| -> Option<(usize, &'a str)> {
        let tail = &lower[from..];
        let end = tail.find("[/url]")?;
        (!tail[..end].contains('\n')).then_some((from + end + 6, &text[from..from + end]))
    };
    match lower.as_bytes().get(rest)? {
        b']' => close(rest + 1).map(|(end, g2)| (end, None, g2)),
        b'=' => {
            let mut p = rest + 1;
            loop {
                let rel = lower[p..].find(']')?;
                let bracket = p + rel;
                if lower[rest + 1..bracket].contains('\n') {
                    return None;
                }
                if let Some((end, g2)) = close(bracket + 1) {
                    return Some((end, Some(&text[rest + 1..bracket]), g2));
                }
                p = bracket + 1;
            }
        }
        _ => None,
    }
}

// ---- the item ----

fn serialize(w: &World, id: ItemId, out: &mut dyn muo_io::GenericWriter) {
    base::serialize_base(w, id, out);
    let d = data(w, id).cloned().unwrap_or_default();
    out.write_i32(1); // SerializationVersion
    out.write_opt_string(d.title.as_deref());
    out.write_time_span_ticks(d.duration.ticks());
    out.write_date_time_ticks(d.start_time.ticks());
    out.write_bool(d.active);
    out.write_encoded_int(d.options.len() as i32);
    for o in &d.options {
        out.write_i32(1); // ShardPollOption.SerializationVersion
        out.write_opt_string(o.title.as_deref());
        out.write_encoded_int(o.voters.len() as i32);
        for v in &o.voters {
            out.write_ip_address(*v);
        }
    }
}

fn deserialize(w: &mut World, id: ItemId, r: &mut dyn muo_io::GenericReader) -> muo_io::Result<()> {
    base::deserialize_base(w, id, r)?;
    let version = r.read_i32()?;
    let mut d = ShardPollerData {
        title: r.read_string()?,
        duration: TimeSpan::from_ticks(r.read_time_span_ticks()?),
        start_time: DateTime::from_ticks(r.read_date_time_ticks()?),
        active: r.read_bool()?,
        options: Vec::new(),
    };
    // The generated reader counts options with an encoded int; the version 0 `Deserialize(reader, version)` with `ReadInt`.
    let count = if version < 1 {
        r.read_i32()?
    } else {
        r.read_encoded_int()?
    }
    .max(0);
    for _ in 0..count {
        // `option.Deserialize(reader)`: its own version, a manual body below version 1.
        let option_version = r.read_i32()?;
        let title = r.read_string()?;
        let voters = if option_version < 1 {
            r.read_i32()?
        } else {
            r.read_encoded_int()?
        }
        .max(0);
        let mut v = Vec::new();
        for _ in 0..voters {
            v.push(r.read_ip_address()?);
        }
        d.options.push(ShardPollOption { title, voters: v });
    }
    let active = d.active;
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(d);
    }
    // AfterDeserialization
    if active {
        w.resources.get_mut::<ActivePollers>().0.push(id);
    }
    Ok(())
}

/// `Server.Misc.ShardPoller`.
pub static SHARD_POLLER: ItemType = ItemType {
    full_name: "Server.Misc.ShardPoller",
    parent: Some(&muo_server::item::ITEM_BASE),
    construct: Some(new_shard_poller),
    default_name: |_, _| Some("shard poller".to_owned()),
    on_double_click: |w, id, from| {
        let admin = w
            .mobile(from)
            .is_some_and(|m| m.access_level >= AccessLevel::Administrator);
        if admin {
            ShardPollGump::display_to(w, from, id, true, VecDeque::new());
        }
    },
    on_delete: |w, id| {
        base::on_delete_base(w, id);
        set_active(w, id, false);
    },
    serialize,
    deserialize,
    ..muo_server::item::ITEM_BASE
};

fn poller(e: muo_server::ids::EntityId) -> Option<ItemId> {
    match e {
        muo_server::ids::EntityId::Item(i) => Some(i),
        muo_server::ids::EntityId::Mobile(_) => None,
    }
}

const GM: AccessLevel = AccessLevel::GameMaster;
const ADMIN: AccessLevel = AccessLevel::Administrator;

/// `[SerializedCommandProperty(GameMaster, Administrator)]` members `Title`, `Duration`, `StartTime`, `Active` and
/// `[CommandProperty(GameMaster, Administrator)] TimeRemaining`.
static PROPS: &[crate::props::PropDef] = &[
    crate::props::PropDef {
        name: "Title",
        kind: crate::props::Kind::Plain(crate::utilities::types::PropType::Str),
        read: GM,
        write: ADMIN,
        get: |w, e| match poller(e)
            .and_then(|i| data(w, i))
            .and_then(|d| d.title.clone())
        {
            Some(t) => crate::props::Value::Str(t),
            None => crate::props::Value::Null,
        },
        set: Some(|w, e, v| {
            if let Some(i) = poller(e) {
                match v {
                    crate::props::Value::Str(t) | crate::props::Value::Text(t) => {
                        set_title(w, i, Some(&t))
                    }
                    _ => set_title(w, i, None),
                }
            }
            Ok(())
        }),
    },
    crate::props::PropDef {
        name: "Duration",
        kind: crate::props::Kind::TimeSpan,
        read: GM,
        write: ADMIN,
        get: |w, e| {
            crate::props::Value::TimeSpan(
                poller(e)
                    .and_then(|i| data(w, i))
                    .map_or(TimeSpan::ZERO, |d| d.duration),
            )
        },
        set: Some(|w, e, v| match (poller(e), v) {
            (Some(i), crate::props::Value::TimeSpan(t)) => {
                if let Some(d) = data_mut(w, i) {
                    d.duration = t;
                }
                Ok(())
            }
            _ => Err("not a time span".to_owned()),
        }),
    },
    crate::props::PropDef {
        name: "StartTime",
        kind: crate::props::Kind::DateTime,
        read: GM,
        write: ADMIN,
        get: |w, e| {
            crate::props::Value::DateTime(
                poller(e)
                    .and_then(|i| data(w, i))
                    .map_or(DateTime::MIN, |d| d.start_time),
            )
        },
        set: Some(|w, e, v| match (poller(e), v) {
            (Some(i), crate::props::Value::DateTime(t)) => {
                if let Some(d) = data_mut(w, i) {
                    d.start_time = t;
                }
                Ok(())
            }
            _ => Err("not a date".to_owned()),
        }),
    },
    crate::props::PropDef {
        name: "TimeRemaining",
        kind: crate::props::Kind::TimeSpan,
        read: GM,
        write: ADMIN,
        get: |w, e| {
            crate::props::Value::TimeSpan(
                poller(e).map_or(TimeSpan::ZERO, |i| time_remaining(w, i)),
            )
        },
        set: None,
    },
    crate::props::PropDef {
        name: "Active",
        kind: crate::props::Kind::Plain(crate::utilities::types::PropType::Bool),
        read: GM,
        write: ADMIN,
        get: |w, e| {
            crate::props::Value::Bool(poller(e).and_then(|i| data(w, i)).is_some_and(|d| d.active))
        },
        set: Some(|w, e, v| {
            if let (Some(i), crate::props::Value::Bool(b)) = (poller(e), v) {
                set_active(w, i, b);
            }
            Ok(())
        }),
    },
];

/// The registration: the item type, and the login event.
pub fn register(t: &mut TypeRegistry) {
    t.register_item(&SHARD_POLLER, &[]);
    t.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "Server.Misc.ShardPoller.Props",
        |w| crate::props::add(w, "Server.Misc.ShardPoller", PROPS),
    );
    t.register_phase_default(
        muo_server::type_registry::Phase::Initialize,
        "Server.Misc.ShardPoller.OnLogin",
        |w| events::subscribe_ranked::<PlayerLogin>(w, 13, on_login),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urls_become_links() {
        assert_eq!(
            url_to_href("see [url=http://a.b/c]here[/url] now"),
            "see <a href=\"http://a.b/c\">here</a> now"
        );
        assert_eq!(
            url_to_href("[URL]http://x.y[/url]"),
            "<a href=\"http://x.y\">http://x.y</a>"
        );
        assert_eq!(url_to_href("no links ]["), "no links ][");
        assert_eq!(url_to_href("[url=a]b"), "[url=a]b");
        assert_eq!(
            url_to_href("[url=a]x[/url] and [url=b]y[/url]"),
            "<a href=\"a\">x</a> and <a href=\"b\">y</a>"
        );
        assert_eq!(url_to_href("[url=a]\n[/url]"), "[url=a]\n[/url]");
    }

    #[test]
    fn option_heights_and_breaks() {
        let o = |t: Option<&str>| ShardPollOption {
            title: t.map(str::to_owned),
            voters: vec![],
        };
        assert_eq!((o(None).breaks(), o(None).compute_height()), (1, 30));
        assert_eq!(o(Some("a<br>b")).breaks(), 2);
        assert_eq!(o(Some("a<br>b<br>c")).compute_height(), 54);
    }

    #[test]
    fn a_class_c_neighbour_has_already_voted() {
        let mut o = ShardPollOption::default();
        o.voters.push("10.1.2.3".parse().unwrap());
        assert!(o.has_already_voted(Some("10.1.2.200".parse().unwrap())));
        assert!(!o.has_already_voted(Some("10.1.3.3".parse().unwrap())));
        assert!(!o.has_already_voted(None));
    }

    #[test]
    fn command_properties_read_and_write_the_poll() {
        use crate::props::{Access, Value, add, register_core, resolve_chain};
        use muo_server::ids::EntityId;
        let mut w = World::new();
        register_core(&mut w);
        add(&mut w, "Server.Misc.ShardPoller", PROPS);
        w.now = DateTime::MIN + TimeSpan::from_hours(1);
        let id = new_shard_poller(&mut w);
        let e = EntityId::Item(id);
        let get = |w: &mut World, name: &str| {
            let (o, d) =
                resolve_chain(w, AccessLevel::Administrator, e, name, Access::READ).unwrap();
            (d.get)(w, o)
        };
        let (o, d) = resolve_chain(
            &mut w,
            AccessLevel::Administrator,
            e,
            "Active",
            Access::WRITE,
        )
        .unwrap();
        (d.set.unwrap())(&mut w, o, Value::Bool(true)).unwrap();
        assert_eq!(get(&mut w, "Active"), Value::Bool(true));
        assert_eq!(
            get(&mut w, "TimeRemaining"),
            Value::TimeSpan(TimeSpan::from_hours(24))
        );
        let (o, d) = resolve_chain(
            &mut w,
            AccessLevel::Administrator,
            e,
            "Title",
            Access::WRITE,
        )
        .unwrap();
        (d.set.unwrap())(&mut w, o, Value::Str("[url=http://a.b]x[/url]".into())).unwrap();
        assert_eq!(
            get(&mut w, "Title"),
            Value::Str("<a href=\"http://a.b\">x</a>".into())
        );
    }

    #[test]
    fn the_item_saves_and_loads_back() {
        let mut w = World::new();
        let id = new_shard_poller(&mut w);
        set_title(&mut w, id, Some("Pick [url=http://a]one[/url]"));
        add_option(&mut w, id, Some("Yes".into()));
        add_option(&mut w, id, None);
        data_mut(&mut w, id).unwrap().options[0]
            .voters
            .push("1.2.3.4".parse().unwrap());
        set_active(&mut w, id, true);
        assert_eq!(w.resources.get_mut::<ActivePollers>().0, [id]);

        let mut bytes = muo_io::SaveWriter::new(true);
        serialize(&w, id, &mut bytes);
        let saved = bytes.into_vec();

        let id2 = new_shard_poller(&mut w);
        let mut r = muo_io::SaveReader::new(&saved);
        deserialize(&mut w, id2, &mut r).unwrap();
        let d = data(&w, id2).unwrap();
        assert_eq!(
            d.title.as_deref(),
            Some("Pick <a href=\"http://a\">one</a>")
        );
        assert!(d.active);
        assert_eq!(d.options.len(), 2);
        assert_eq!(d.options[0].voters, ["1.2.3.4".parse::<IpAddr>().unwrap()]);
        assert_eq!(d.options[1].title, None);
        assert!(w.resources.get_mut::<ActivePollers>().0.contains(&id2));
    }

    #[test]
    fn a_version_0_poll_loads_with_plain_ints() {
        use muo_io::GenericWriter;
        let mut w = World::new();
        let id = new_shard_poller(&mut w);
        let mut out = muo_io::SaveWriter::new(true);
        base::serialize_base(&w, id, &mut out); // base body
        out.write_i32(0);
        out.write_opt_string(Some("T"));
        out.write_time_span_ticks(TimeSpan::from_hours(1).ticks());
        out.write_date_time_ticks(DateTime::MIN.ticks());
        out.write_bool(false);
        out.write_i32(1); // option count, not encoded
        out.write_i32(0); // the option's own version
        out.write_opt_string(Some("A"));
        out.write_i32(1);
        out.write_ip_address("1.2.3.4".parse().unwrap());
        let bytes = out.into_vec();
        let id2 = new_shard_poller(&mut w);
        deserialize(&mut w, id2, &mut muo_io::SaveReader::new(&bytes)).unwrap();
        let d = data(&w, id2).unwrap();
        assert_eq!(d.options[0].title.as_deref(), Some("A"));
        assert_eq!(d.options[0].voters, ["1.2.3.4".parse::<IpAddr>().unwrap()]);
    }
}
