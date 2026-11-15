//! The town crier (a plain `Mobile`), the global entry list, the prompts and
//! the `TownCrierGump` of the `[TownCriers` command. `TownCrier` is `[SerializationGenerator(0, false)]`.

use std::rc::Rc;

use muo_content::gumps::base::{DynamicGumpLogic, DynamicGumpObj, RelayInfo};
use muo_content::gumps::builder::DynamicGumpBuilder;
use muo_content::gumps::system::send_gump_to_mobile;
use muo_content::hooks::rnd;
use muo_content::mx;
use muo_primitives::{DateTime, MessageType, TimeSpan, utility};
use muo_server::World;
use muo_server::commands::{CommandEventArgs, CommandSystem};
use muo_server::ids::MobileId;
use muo_server::mobile::part1::props_b;
use muo_server::mobile::part2::messages;
use muo_server::mobile::{AccessLevel, MOBILE_BASE, MobileType};
use muo_server::net_state::NetStateId;
use muo_server::prompt::{Prompt, add_prompt};
use muo_server::timer::{self, TimerHandle};
use muo_server::type_registry::{DEFAULT_CALL_PRIORITY, Phase, TypeRegistry};

/// `TownCrierEntry`.
#[derive(Clone, Debug)]
pub struct TownCrierEntry {
    pub id: u32,
    pub lines: Vec<String>,
    pub expire_time: DateTime,
}

impl TownCrierEntry {
    fn new(w: &mut World, lines: Vec<String>, duration: TimeSpan) -> Self {
        let s = w.resources.get_mut::<CrierStatics>();
        s.next_entry += 1;
        let id = s.next_entry;
        let d = duration.max(TimeSpan::ZERO).min(TimeSpan::from_days(365));
        TownCrierEntry {
            id,
            lines,
            expire_time: w.now + d,
        }
    }

    fn expired(&self, now: DateTime) -> bool {
        now >= self.expire_time
    }
}

/// The two `ITownCrierEntryList`s: `GlobalTownCrierEntryList.Instance` and a crier.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Owner {
    Global,
    Crier(MobileId),
}

/// `GlobalTownCrierEntryList.Instance`, `TownCrier.Instances` and the entry counter.
#[derive(Default)]
pub struct CrierStatics {
    /// `GlobalTownCrierEntryList.Entries` (`None` when null).
    global: Option<Vec<TownCrierEntry>>,
    instances: Vec<MobileId>,
    next_entry: u32,
}

/// What `TownCrier` adds to a `Mobile`.
#[derive(Default)]
pub struct CrierData {
    entries: Option<Vec<TownCrierEntry>>,
    auto_shout_timer: TimerHandle,
    news_timer: TimerHandle,
}

fn st(w: &mut World) -> &mut CrierStatics {
    w.resources.get_mut::<CrierStatics>()
}

fn data(w: &World, id: MobileId) -> Option<&CrierData> {
    w.mobile(id)?.ext.get::<CrierData>()
}

fn data_mut(w: &mut World, id: MobileId) -> Option<&mut CrierData> {
    w.mobile_mut(id)?.ext.get_mut::<CrierData>()
}

fn is_crier(w: &World, id: MobileId) -> bool {
    data(w, id).is_some()
}

fn running(w: &World, t: TimerHandle) -> bool {
    t != TimerHandle::default() && t.running(w)
}

// ---- the entry lists ----

fn entries_of(w: &World, owner: Owner) -> Vec<TownCrierEntry> {
    match owner {
        Owner::Global => w
            .resources
            .get::<CrierStatics>()
            .and_then(|s| s.global.clone())
            .unwrap_or_default(),
        Owner::Crier(c) => data(w, c)
            .and_then(|d| d.entries.clone())
            .unwrap_or_default(),
    }
}

/// `GlobalTownCrierEntryList.IsEmpty`.
fn global_is_empty(w: &World) -> bool {
    w.resources
        .get::<CrierStatics>()
        .is_none_or(|s| s.global.as_ref().is_none_or(Vec::is_empty))
}

fn remove_expired(w: &mut World, owner: Owner) {
    let now = w.now;
    for e in entries_of(w, owner).iter().rev() {
        if e.expired(now) {
            remove_entry(w, owner, e.id);
        }
    }
}

/// `GetRandomEntry()`.
pub fn get_random_entry(w: &mut World, owner: Owner) -> Option<TownCrierEntry> {
    match owner {
        Owner::Global => {
            remove_expired(w, owner);
            let entries = st(w).global.clone()?;
            utility::random_element(&mut w.rng, &entries).cloned()
        }
        Owner::Crier(c) => {
            let count = data(w, c)
                .and_then(|d| d.entries.as_ref())
                .map_or(0, Vec::len);
            remove_expired(w, owner);
            let entry = get_random_entry(w, Owner::Global);
            if count > 0 && entry.is_some() && rnd::bool(w) {
                return entry;
            }
            let entries = data(w, c).and_then(|d| d.entries.clone())?;
            utility::random_element(&mut w.rng, &entries).cloned()
        }
    }
}

/// `AddEntry(lines, duration)`.
pub fn add_entry(
    w: &mut World,
    owner: Owner,
    lines: Vec<String>,
    duration: TimeSpan,
) -> TownCrierEntry {
    let tce = TownCrierEntry::new(w, lines, duration);
    match owner {
        Owner::Global => {
            st(w).global.get_or_insert_with(Vec::new).push(tce.clone());
            for c in st(w).instances.clone() {
                force_begin_auto_shout(w, c);
            }
        }
        Owner::Crier(c) => {
            if let Some(d) = data_mut(w, c) {
                d.entries.get_or_insert_with(Vec::new).push(tce.clone());
            }
            force_begin_auto_shout(w, c);
        }
    }
    tce
}

/// `RemoveEntry(tce)`.
pub fn remove_entry(w: &mut World, owner: Owner, entry: u32) {
    match owner {
        Owner::Global => {
            let Some(list) = st(w).global.as_mut() else {
                return;
            };
            list.retain(|e| e.id != entry);
            if list.is_empty() {
                st(w).global = None;
            }
        }
        Owner::Crier(c) => {
            let Some(list) = data_mut(w, c).and_then(|d| d.entries.as_mut()) else {
                return;
            };
            list.retain(|e| e.id != entry);
            if list.is_empty()
                && let Some(d) = data_mut(w, c)
            {
                d.entries = None;
            }
            let none = data(w, c).is_none_or(|d| d.entries.is_none());
            if none && global_is_empty(w) {
                let mut t = data_mut(w, c)
                    .map(|d| std::mem::take(&mut d.auto_shout_timer))
                    .unwrap_or_default();
                t.cancel(w);
            }
        }
    }
}

// ---- the crier ----

/// `ForceBeginAutoShout()`.
pub fn force_begin_auto_shout(w: &mut World, id: MobileId) {
    if data(w, id).is_some_and(|d| running(w, d.auto_shout_timer)) {
        return;
    }
    let h = timer::start(
        w,
        TimeSpan::from_secs(5),
        TimeSpan::from_mins(1),
        0,
        move |w| auto_shout(w, id),
    );
    if let Some(d) = data_mut(w, id) {
        d.auto_shout_timer = h;
    }
}

/// `ShoutNews_Callback` timer: a line every 3 seconds.
fn start_news(w: &mut World, id: MobileId, tce: TownCrierEntry) {
    let n = tce.lines.len() as i32;
    let mut index = 0usize;
    let h = timer::start(
        w,
        TimeSpan::from_secs(1),
        TimeSpan::from_secs(3),
        n,
        move |w| {
            // `_newsTimer.Index` counts the ticks done so far.
            if index >= tce.lines.len() {
                let mut t = data_mut(w, id)
                    .map(|d| std::mem::take(&mut d.news_timer))
                    .unwrap_or_default();
                t.cancel(w);
            } else {
                messages::public_overhead_message(
                    w,
                    id,
                    MessageType::REGULAR,
                    0x3B2,
                    false,
                    &tce.lines[index],
                    true,
                    AccessLevel::Player,
                );
                index += 1;
            }
        },
    );
    if let Some(d) = data_mut(w, id) {
        d.news_timer = h;
    }
}

/// `AutoShout_Callback()`.
fn auto_shout(w: &mut World, id: MobileId) {
    let tce = get_random_entry(w, Owner::Crier(id));
    match tce {
        None => {
            let mut t = data_mut(w, id)
                .map(|d| std::mem::take(&mut d.auto_shout_timer))
                .unwrap_or_default();
            t.cancel(w);
        }
        Some(tce) => {
            if !data(w, id).is_some_and(|d| running(w, d.news_timer)) {
                start_news(w, id, tce);
                messages::public_overhead_message_number(
                    w,
                    id,
                    MessageType::REGULAR,
                    0x3B2,
                    502976,
                    "",
                    false,
                ); // Hear ye! Hear ye!
            }
        }
    }
}

/// `TownCrier()`.
fn crier_new(w: &mut World) -> MobileId {
    let id = w.new_mobile(&TOWN_CRIER);
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(CrierData::default());
    }
    st(w).instances.push(id);
    muo_content::templates::apply_mobile(w, id);
    id
}

fn on_double_click(w: &mut World, id: MobileId, from: MobileId) {
    if mx::access_level(w, from) >= AccessLevel::GameMaster {
        display_to(w, from, Owner::Crier(id));
    } else {
        (MOBILE_BASE.on_double_click)(w, id, from);
    }
}

fn handles_on_speech(w: &World, id: MobileId, from: MobileId) -> bool {
    !data(w, id).is_some_and(|d| running(w, d.news_timer))
        && mx::alive(w, from)
        && mx::in_range(w, id, from, 12)
}

fn on_speech(w: &mut World, id: MobileId, e: &mut muo_server::events::SpeechEventArgs) {
    let from = e.mobile;
    let busy = data(w, id).is_some_and(|d| running(w, d.news_timer));
    if !busy && e.has_keyword(0x30) && mx::alive(w, from) && mx::in_range(w, id, from, 12) {
        // *news*
        let d = mx::get_direction_to(w, id, from);
        mx::set_direction(w, id, d);
        match get_random_entry(w, Owner::Crier(id)) {
            None => {
                messages::public_overhead_message_number(
                    w,
                    id,
                    MessageType::REGULAR,
                    0x3B2,
                    1005643,
                    "",
                    false,
                ); // I have no news at this time.
            }
            Some(tce) => {
                start_news(w, id, tce);
                messages::public_overhead_message_number(
                    w,
                    id,
                    MessageType::REGULAR,
                    0x3B2,
                    502978,
                    "",
                    false,
                ); // Some of the latest news!
            }
        }
    }
}

fn on_delete(w: &mut World, id: MobileId) {
    st(w).instances.retain(|&m| m != id);
    (MOBILE_BASE.on_delete)(w, id);
}

pub static TOWN_CRIER: MobileType = MobileType {
    full_name: "Server.Mobiles.TownCrier",
    parent: Some(&MOBILE_BASE),
    construct: Some(crier_new),
    serialize: |w, id, out| {
        (MOBILE_BASE.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (MOBILE_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        if let Some(m) = w.mobile_mut(id) {
            m.ext.insert(CrierData::default());
        }
        // [AfterDeserialization]
        st(w).instances.push(id);
        Ok(())
    },
    on_double_click,
    handles_on_speech,
    on_speech,
    can_be_damaged: |_w, _id| false,
    on_delete,
    ..MOBILE_BASE
};

// ---- gump ----

struct TownCrierGump {
    owner: Owner,
    entries: Vec<TownCrierEntry>,
    now: DateTime,
}

fn display_to(w: &mut World, from: MobileId, owner: Owner) {
    if w.mobile(from).is_none_or(|m| m.net_state.is_none()) {
        return;
    }
    remove_expired(w, owner); // force expiration checks
    let g = TownCrierGump {
        owner,
        entries: entries_of(w, owner),
        now: w.now,
    };
    send_gump_to_mobile(w, from, Box::new(DynamicGumpObj::new(50, 50, g)), true);
}

impl DynamicGumpLogic for TownCrierGump {
    const TYPE_NAME: &'static str = "Server.Mobiles.TownCrierGump";
    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.add_page(0);
        let count = self.entries.len() as i32;
        let h = 38 + if count == 0 { 20 } else { count * 85 };
        b.add_image_tiled(0, 0, 300, h, 0xA40);
        b.add_alpha_region(1, 1, 298, 36 + if count == 0 { 20 } else { count * 85 });
        b.add_html(
            8,
            8,
            300 - 8 - 30,
            20,
            "<basefont color=#FFFFFF><center>TOWN CRIER MESSAGES</center></basefont>",
            false,
            false,
        );
        b.add_button(300 - 8 - 30, 8, 0xFAB, 0xFAD, 1);
        if count == 0 {
            b.add_html(
                8,
                30,
                284,
                20,
                "<basefont color=#FFFFFF>The crier has no news.</basefont>",
                false,
                false,
            );
            return;
        }
        for (i, tce) in self.entries.iter().enumerate() {
            let to_expire = (tce.expire_time - self.now).max(TimeSpan::ZERO);
            let mut s = String::from("[Expires: ");
            if to_expire.total_hours() >= 1.0 {
                s.push_str(&format!(
                    "{}:{:02}",
                    to_expire.total_hours() as i32,
                    to_expire.minutes()
                ));
            } else {
                s.push_str(&to_expire.minutes().to_string());
            }
            s.push_str(&format!(":{:02}] ", to_expire.seconds()));
            s.push_str(&tce.lines.join("<br>"));
            let y = 35 + i as i32 * 85;
            b.add_html(8, y, 254, 80, &s, true, true);
            b.add_button(300 - 8 - 26, y, 0x15E1, 0x15E5, 2 + i as i32);
        }
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(from) = w.net_state(sender).and_then(|n| n.mobile) else {
            return;
        };
        if info.button_id == 1 {
            mx::send_message(
                w,
                from,
                "Enter the duration for the new message. Format: <hours:minutes:seconds, 00:00:00>",
            );
            set_prompt(w, from, duration_prompt(self.owner));
        } else if info.button_id > 1 {
            let entries = entries_of(w, self.owner);
            let index = (info.button_id - 2) as usize;
            if let Some(tce) = entries.get(index) {
                let ts = (tce.expire_time - w.now).max(TimeSpan::ZERO);
                mx::send_message(w, from, &format!("Editing entry #{}.", index + 1));
                mx::send_message(w, from, "Enter the first line to shout:");
                set_prompt(
                    w,
                    from,
                    lines_prompt(
                        self.owner,
                        Some(tce.id),
                        Rc::new(std::cell::RefCell::new(Vec::new())),
                        ts,
                    ),
                );
            }
        }
    }
}

/// `TimeSpan.ToString()`: `[-][d.]hh:mm:ss[.fffffff]`.
fn format_time_span(ts: TimeSpan) -> String {
    let ticks = ts.ticks();
    let (sign, t) = if ticks < 0 {
        ("-", -ticks)
    } else {
        ("", ticks)
    };
    let (days, rem) = (t / 864_000_000_000, t % 864_000_000_000);
    let (h, m, s, frac) = (
        rem / 36_000_000_000,
        rem / 600_000_000 % 60,
        rem / 10_000_000 % 60,
        rem % 10_000_000,
    );
    let mut out = format!(
        "{sign}{}{h:02}:{m:02}:{s:02}",
        if days > 0 {
            format!("{days}.")
        } else {
            String::new()
        }
    );
    if frac > 0 {
        out.push_str(&format!(".{frac:07}"));
    }
    out
}

fn set_prompt(w: &mut World, from: MobileId, p: Prompt) {
    let id = add_prompt(w, p);
    props_b::set_prompt(w, from, Some(id));
}

fn duration_prompt(owner: Owner) -> Prompt {
    Prompt {
        on_response: Some(Rc::new(move |w, from, text| {
            let Some(mut ts) = TimeSpan::try_parse(text) else {
                mx::send_message(
                    w,
                    from,
                    "Value was not properly formatted. Use: <hours:minutes:seconds, 00:00:00>",
                );
                display_to(w, from, owner);
                return;
            };
            if ts < TimeSpan::ZERO {
                ts = TimeSpan::ZERO;
            }
            mx::send_message(
                w,
                from,
                &format!("Duration set to: {}", format_time_span(ts)),
            );
            mx::send_message(w, from, "Enter the first line to shout:");
            set_prompt(
                w,
                from,
                lines_prompt(
                    owner,
                    None,
                    Rc::new(std::cell::RefCell::new(Vec::new())),
                    ts,
                ),
            );
        })),
        on_cancel: Some(Rc::new(move |w, from| {
            mx::msg(w, from, 502980); // Message entry cancelled.
            display_to(w, from, owner);
        })),
    }
}

fn lines_prompt(
    owner: Owner,
    entry: Option<u32>,
    lines: Rc<std::cell::RefCell<Vec<String>>>,
    duration: TimeSpan,
) -> Prompt {
    let l1 = lines.clone();
    Prompt {
        on_response: Some(Rc::new(move |w, from, text| {
            l1.borrow_mut().push(text.to_owned());
            mx::send_message(
                w,
                from,
                "Enter the next line to shout, or press <ESC> if the message is finished.",
            );
            set_prompt(w, from, lines_prompt(owner, entry, l1.clone(), duration));
        })),
        on_cancel: Some(Rc::new(move |w, from| {
            if let Some(e) = entry {
                remove_entry(w, owner, e);
            }
            let l = lines.borrow().clone();
            if !l.is_empty() {
                add_entry(w, owner, l, duration);
                mx::send_message(w, from, "Message has been set.");
            } else if entry.is_some() {
                mx::send_message(w, from, "Message deleted.");
            } else {
                mx::msg(w, from, 502980); // Message entry cancelled.
            }
            display_to(w, from, owner);
        })),
    }
}

/// `GlobalTownCrierEntryList.Configure`: `[TownCriers`.
fn configure(w: &mut World) {
    CommandSystem::register(
        w,
        "TownCriers",
        AccessLevel::GameMaster,
        &[],
        Rc::new(|w, e: &mut CommandEventArgs| {
            display_to(w, e.mobile, Owner::Global);
        }),
    );
}

pub(crate) fn register(types: &mut TypeRegistry) {
    types.register_mobile(&TOWN_CRIER, &[]);
    types.register_phase(
        Phase::Configure,
        "GlobalTownCrierEntryList.Configure",
        DEFAULT_CALL_PRIORITY,
        configure,
    );
}

#[allow(dead_code)]
fn _unused(w: &World, id: MobileId) -> bool {
    is_crier(w, id)
}

#[cfg(test)]
mod duration_tests {
    use super::*;

    #[test]
    fn duration_prompt_reads_a_bare_number_as_days_like_timespan_tryparse() {
        // `TimeSpan.TryParse("2")` is two days; the prompt asks for hh:mm:ss but takes what the parser takes.
        assert_eq!(TimeSpan::try_parse("2"), Some(TimeSpan::from_days(2)));
        assert_eq!(TimeSpan::try_parse("01:30"), Some(TimeSpan::from_mins(90)));
    }
}
