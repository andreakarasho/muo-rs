//! The new guild system gumps: `BaseGuildGump` (the tab strip and statics), `BaseGuildListGump<T>`
//! (as [`ListCore`]), `GuildInfoGump`, `GuildRosterGump`, `GuildDiplomacyGump`, `GuildAdvancedSearchGump`,
//! `GuildMemberInfoGump`, `GuildInvitationRequest`, `OtherGuildInfo`, `WarDeclarationGump` and `CreateGuildGump`.
//!
//! A gump takes its snapshot (names, ranks, sort keys) when it is made, and every re-send makes a new one from the
//! state the response changed, so the list is always the one at render time. The names are the
//! gumps' own: the old-system gumps of the same names are in `guilds`.
//!
//! `AllianceInfo.AllianceRosterGump` is `GuildDiplomacyGump { roster: true }` (its own type name).
//!
//! `Faction.Find` goes through `AlHooks::faction_find`.

use muo_primitives::dotnet::{IgnoreCase, InvariantCase};
use std::rc::Rc;

use muo_content::gumps::GumpButtonType;
use muo_content::gumps::base::{DynamicGumpLogic, DynamicGumpObj, RelayInfo};
use muo_content::gumps::builder::DynamicGumpBuilder;
use muo_content::gumps::system::{close_gump, has_gump};
use muo_content::misc::guild::RankFlags;
use muo_content::misc::guild::{LEADER_INDEX, LOWEST_INDEX, RANKS};
use muo_content::misc::guild_class as gc;
use muo_content::player_mobile::{self as pmob, PlayerFlag};
use muo_primitives::{TimeSpan, html, utility};
use muo_server::World;
use muo_server::guild::{self, GuildId, GuildType};
use muo_server::ids::{EntityId, MobileId};
use muo_server::mobile::AccessLevel;
use muo_server::net_state::NetStateId;
use muo_server::prompt::SimplePrompt;
use muo_server::target::{TargetFlags, Targeted};

use super::common::{mobile_of, name_of, send};
use super::guilds::GuildChangeTypeGump;
use super::hooks::hooks;

// ---- BaseGuildGump statics ----

/// `BaseGuildGump.IsLeader`.
pub fn is_leader(w: &World, m: MobileId, g: GuildId) -> bool {
    !(muo_content::mx::deleted(w, m)
        || gc::disbanded(w, g)
        || !muo_content::mx::player(w, m)
        || muo_content::mx::access_level(w, m) < AccessLevel::GameMaster
            && gc::data(w, g).and_then(|d| d.leader) != Some(m))
}

/// `BaseGuildGump.IsMember`.
pub fn is_member(w: &World, m: MobileId, g: GuildId) -> bool {
    !(muo_content::mx::deleted(w, m)
        || gc::disbanded(w, g)
        || !muo_content::mx::player(w, m)
        || muo_content::mx::access_level(w, m) < AccessLevel::GameMaster && !gc::is_member(w, g, m))
}

/// `BaseGuildGump.CheckProfanity(s, maxLength)`.
pub fn check_profanity(s: &str, max_length: i32) -> bool {
    muo_content::misc::name_verification::validate_with(
        s,
        1,
        max_length,
        true,
        true,
        false,
        0,
        Some(muo_content::misc::profanity_protection::EXCEPTIONS),
        muo_content::misc::profanity_protection::DISALLOWED,
        None,
    )
}

/// A `TextDefinition`: a cliloc number or an HTML string.
#[derive(Clone, Debug)]
pub enum Txt {
    Num(i32),
    Str(String),
}

/// `BaseGuildGump.AddHtmlText`.
#[allow(clippy::too_many_arguments)]
fn add_html_text(
    b: &mut DynamicGumpBuilder,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    t: &Txt,
    back: bool,
    scroll: bool,
) {
    match t {
        Txt::Num(n) if *n > 0 => b.add_html_localized(x, y, w, h, *n, back, scroll),
        Txt::Str(s) => b.add_html(x, y, w, h, s, back, scroll),
        Txt::Num(_) => {}
    }
}

/// `BaseGuildGump.BuildLayout` up to `BuildContent`.
fn start(b: &mut DynamicGumpBuilder, show_tab: bool) {
    b.add_page(0);
    if show_tab {
        b.add_background(0, 0, 600, 440, 0x24AE);
        b.add_background(66, 40, 150, 26, 0x2486);
        b.add_button(71, 45, 0x845, 0x846, 1);
        b.add_html_localized_color(96, 43, 110, 26, 1063014, 0x0, false, false); // My Guild
        b.add_background(236, 40, 150, 26, 0x2486);
        b.add_button(241, 45, 0x845, 0x846, 2);
        b.add_html_localized_color(266, 43, 110, 26, 1062974, 0x0, false, false); // Guild Roster
        b.add_background(401, 40, 150, 26, 0x2486);
        b.add_button(406, 45, 0x845, 0x846, 3);
        b.add_html_localized_color(431, 43, 110, 26, 1062978, 0x0, false, false); // Diplomacy
        b.add_page(1);
    }
}

fn hl(b: &mut DynamicGumpBuilder, x: i32, y: i32, w: i32, h: i32, n: i32, color: i32) {
    b.add_html_localized_color(x, y, w, h, n, color, false, false);
}

fn hlb(b: &mut DynamicGumpBuilder, x: i32, y: i32, w: i32, h: i32, n: i32, color: i32) {
    b.add_html_localized_color(x, y, w, h, n, color, true, false);
}

/// The player of a response when it is a `PlayerMobile` member of the guild.
fn member_of(w: &World, sender: NetStateId, g: GuildId) -> Option<MobileId> {
    let pm = mobile_of(w, sender)?;
    (muo_content::mx::player(w, pm) && is_member(w, pm, g)).then_some(pm)
}

/// `BaseGuildGump.OnResponse` (the three tabs).
fn base_response(w: &mut World, sender: NetStateId, g: GuildId, info: &RelayInfo) {
    let Some(pm) = member_of(w, sender, g) else {
        return;
    };
    match info.button_id {
        1 => send_info(w, pm, g),
        2 => {
            let gump = roster(w, pm, g);
            send(w, pm, gump);
        }
        3 => {
            let gump = diplomacy(w, pm, g);
            send(w, pm, gump);
        }
        _ => {}
    }
}

fn send_info(w: &mut World, pm: MobileId, g: GuildId) {
    let gump = info_gump(w, pm, g, false);
    send(w, pm, gump);
}

/// `pm.SendGump(new GuildInfoGump(pm, guild))`.
pub fn open_info_gump(w: &mut World, pm: MobileId, g: GuildId) {
    send_info(w, pm, g);
}

/// `pm.CloseGump<BaseGuildGump>()` over its subclasses.
fn close_base_guild_gumps(w: &mut World, ns: NetStateId) {
    close_gump::<GuildInfoGump>(w, ns);
    close_gump::<GuildRosterGump>(w, ns);
    close_gump::<GuildDiplomacyGump>(w, ns);
    close_gump::<GuildAdvancedSearchGump>(w, ns);
    close_gump::<GuildMemberInfoGump>(w, ns);
    close_gump::<GuildInvitationRequest>(w, ns);
    close_gump::<OtherGuildInfo>(w, ns);
    close_gump::<WarDeclarationGump>(w, ns);
}

/// `HasGump<BaseGuildGump>()`.
fn has_base_guild_gump(w: &World, ns: NetStateId) -> bool {
    has_gump::<GuildInfoGump>(w, ns)
        || has_gump::<GuildRosterGump>(w, ns)
        || has_gump::<GuildDiplomacyGump>(w, ns)
        || has_gump::<GuildAdvancedSearchGump>(w, ns)
        || has_gump::<GuildMemberInfoGump>(w, ns)
        || has_gump::<GuildInvitationRequest>(w, ns)
        || has_gump::<OtherGuildInfo>(w, ns)
        || has_gump::<WarDeclarationGump>(w, ns)
}

fn ns_of(w: &World, m: MobileId) -> Option<NetStateId> {
    w.mobile(m).and_then(|x| x.net_state)
}

fn gname(w: &World, g: GuildId) -> String {
    gc::name(w, g)
}

fn abbreviation(w: &World, g: GuildId) -> String {
    guild::guild(w, g)
        .map(|b| b.abbreviation.clone())
        .unwrap_or_default()
}

fn leader_of(w: &World, g: GuildId) -> Option<MobileId> {
    gc::data(w, g).and_then(|d| d.leader)
}

fn rank_of(w: &World, m: MobileId) -> usize {
    pmob::guild_rank(w, m).clamp(0, 4) as usize
}

fn can(w: &World, m: MobileId, flag: RankFlags) -> bool {
    RANKS[rank_of(w, m)].get_flag(flag)
}

fn faction_of(w: &World, m: Option<MobileId>) -> Option<(u32, String)> {
    let f = hooks(w)?.faction_find?;
    f(w, m?)
}

fn msg(w: &mut World, to: MobileId, n: i32) {
    muo_content::mx::msg(w, to, n);
}

fn msg_a(w: &mut World, to: MobileId, n: i32, args: &str) {
    muo_content::mx::msg_args(w, to, n, args);
}

fn hours_component(t: TimeSpan) -> i64 {
    t.ticks() / 36_000_000_000 % 24
}

fn minutes_component(t: TimeSpan) -> i64 {
    t.ticks() / 600_000_000 % 60
}

fn ymd(d: muo_primitives::DateTime) -> String {
    let (y, m, day) = d.ymd();
    format!("{y:04}-{m:02}-{day:02}")
}

fn set_rank(w: &mut World, m: MobileId, rank: i32) {
    if let Some(d) = pmob::data_mut(w, m) {
        d.guild_rank = rank;
    }
}

fn prompt(
    w: &mut World,
    pm: MobileId,
    handles_cancel: bool,
    f: impl Fn(&mut World, MobileId, &str) + 'static,
) {
    muo_server::mobile::part2::speech::begin_prompt(
        w,
        pm,
        SimplePrompt::new(Some(Rc::new(f)), handles_cancel),
    );
}

// ---- GuildInfoGump ----

pub struct GuildInfoGump {
    guild: GuildId,
    is_resigning: bool,
    is_leader: bool,
    name: String,
    alliance_name: Option<String>,
    order_chaos: bool,
    faction_text: Option<String>,
    charter: String,
    website: String,
    display_title: bool,
}

/// `new GuildInfoGump(pm, g, isResigning)`.
pub fn info_gump(
    w: &World,
    pm: MobileId,
    g: GuildId,
    is_resigning: bool,
) -> DynamicGumpObj<GuildInfoGump> {
    let alliance_name = gc::alliance(w, g)
        .filter(|&k| gc::alliance_is_member(w, k, g))
        .map(|k| gc::alliance_name(w, k));
    let gt = gc::guild_type(w, g);
    let faction_text = if gt != GuildType::Regular {
        Some(format!("{gt:?}"))
    } else {
        faction_of(w, leader_of(w, g)).map(|f| f.1)
    };
    let d = gc::data(w, g);
    let non_empty = |s: Option<&String>, default: &str| match s {
        Some(s) if !s.is_empty() => s.clone(),
        _ => default.to_owned(),
    };
    DynamicGumpObj::new(
        10,
        10,
        GuildInfoGump {
            guild: g,
            is_resigning,
            is_leader: is_leader(w, pm, g),
            name: gname(w, g),
            alliance_name,
            order_chaos: gc::order_chaos(w),
            faction_text,
            charter: non_empty(
                d.and_then(|d| d.charter.as_ref()),
                "The guild leader has not yet set the guild charter.",
            ),
            website: non_empty(
                d.and_then(|d| d.website.as_ref()),
                "Guild website not yet set.",
            ),
            display_title: w.mobile(pm).is_some_and(|m| m.display_guild_title),
        },
    )
}

impl DynamicGumpLogic for GuildInfoGump {
    const TYPE_NAME: &'static str = "Server.Guilds.GuildInfoGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        start(b, true);
        let is_leader = self.is_leader;
        hl(b, 96, 43, 110, 26, 1063014, 0xF); // My Guild
        b.add_image_tiled(65, 80, 160, 26, 0xA40);
        b.add_image_tiled(67, 82, 156, 22, 0xBBC);
        hl(b, 70, 83, 150, 20, 1062954, 0x0); // <i>Guild Name</i>
        b.add_html(233, 84, 320, 26, &self.name, false, false);
        b.add_image_tiled(65, 114, 160, 26, 0xA40);
        b.add_image_tiled(67, 116, 156, 22, 0xBBC);
        hl(b, 70, 117, 150, 20, 1063025, 0x0); // <i>Alliance</i>
        if let Some(n) = &self.alliance_name {
            b.add_html(233, 118, 320, 26, n, false, false);
            b.add_button(40, 120, 0x4B9, 0x4BA, 6); // Alliance Roster
        }
        if self.order_chaos && is_leader {
            b.add_button(40, 154, 0x4B9, 0x4BA, 100); // Guild Faction
        }
        b.add_image_tiled(65, 148, 160, 26, 0xA40);
        b.add_image_tiled(67, 150, 156, 22, 0xBBC);
        hl(b, 70, 151, 150, 20, 1063084, 0x0); // <i>Guild Faction</i>
        if let Some(t) = &self.faction_text {
            b.add_html(233, 152, 320, 26, t, false, false);
        }
        b.add_image_tiled(65, 196, 480, 4, 0x238D);
        b.add_html(65, 216, 480, 80, &self.charter, true, true);
        if is_leader {
            b.add_button(40, 251, 0x4B9, 0x4BA, 4); // Charter Edit button
        }
        b.add_html(65, 306, 480, 30, &self.website, true, false);
        if is_leader {
            b.add_button(40, 313, 0x4B9, 0x4BA, 5); // Website Edit button
        }
        b.add_checkbox(65, 370, 0xD2, 0xD3, self.display_title, 0);
        hl(b, 95, 370, 150, 26, 1063085, 0x0); // Show Guild Title
        b.add_background(450, 370, 100, 26, 0x2486);
        b.add_button(455, 375, 0x845, 0x846, 7);
        hl(
            b,
            480,
            373,
            60,
            26,
            3006115,
            if self.is_resigning { 0x5000 } else { 0 },
        ); // Resign
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let g = self.guild;
        base_response(w, sender, g, info);
        let Some(pm) = mobile_of(w, sender) else {
            return;
        };
        if !is_member(w, pm, g) {
            return;
        }
        muo_server::mobile::part1::props_b::set_display_guild_title(w, pm, info.is_switched(0));
        match info.button_id {
            4 => {
                if is_leader(w, pm, g) {
                    msg(w, pm, 1013071); // Enter the new guild charter (50 characters max):
                    prompt(w, pm, true, move |w, from, text| {
                        set_charter(w, from, g, text)
                    });
                }
            }
            5 => {
                if is_leader(w, pm, g) {
                    msg(w, pm, 1013072); // Enter the new website for the guild (50 characters max):
                    prompt(w, pm, true, move |w, from, text| {
                        set_website(w, from, g, text)
                    });
                }
            }
            6 => {
                if let Some(k) = gc::alliance(w, g).filter(|&k| gc::alliance_is_member(w, k, g)) {
                    let gump = alliance_roster(w, pm, g, k);
                    send(w, pm, gump);
                }
            }
            7 => {
                if !self.is_resigning {
                    msg(w, pm, 1063332); // Are you sure you wish to resign from your guild?
                    let gump = info_gump(w, pm, g, true);
                    send(w, pm, gump);
                } else {
                    gc::remove_member(w, g, pm, 1063411); // You resign from your guild.
                }
            }
            100 if gc::order_chaos(w) && is_leader(w, pm, g) => {
                GuildChangeTypeGump::display_to(w, pm, g);
            }
            _ => {}
        }
    }
}

fn set_charter(w: &mut World, from: MobileId, g: GuildId, text: &str) {
    if !is_leader(w, from, g) {
        return;
    }
    let charter = utility::fix_html(text.trim());
    if charter.chars().count() > 50 {
        msg_a(w, from, 1070774, "50"); // Your guild charter cannot exceed ~1_val~ characters.
    } else {
        if let Some(d) = gc::data_mut(w, g) {
            d.charter = Some(charter);
        }
        msg(w, from, 1070775); // You submit a new guild charter.
    }
}

fn set_website(w: &mut World, from: MobileId, g: GuildId, text: &str) {
    if !is_leader(w, from, g) {
        return;
    }
    let site = utility::fix_html(text.trim());
    if site.chars().count() > 50 {
        msg_a(w, from, 1070777, "50");
    } else {
        if let Some(d) = gc::data_mut(w, g) {
            d.website = Some(site);
        }
        msg(w, from, 1070778);
    }
}

// ---- BaseGuildListGump<T> ----

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Key {
    /// A null string: it sorts before every string, "" included.
    Null,
    S(String),
    I(i64),
}

fn skey(s: &str) -> Key {
    Key::S(s.upper_inv())
}

/// One entry of the original list, with everything the render needs.
struct Entry<T> {
    id: T,
    values: Vec<Txt>,
    rel: bool,
    /// `IsFiltered(o, filter)`.
    filtered: bool,
    /// The sort key of each field's comparer.
    keys: Vec<Key>,
}

#[derive(Clone)]
struct ListState {
    /// The index into the fields: each field's comparer is its own type.
    comparer: usize,
    ascending: bool,
    filter: String,
    start: i32,
}

const ITEMS_PER_PAGE: i32 = 8;

struct ListCore<T> {
    /// `InfoField`s: (name cliloc, width).
    fields: &'static [(i32, i32)],
    st: ListState,
    /// `_displayList`.
    rows: Vec<Entry<T>>,
}

enum ListAction<T> {
    None,
    Resend,
    Open(T),
}

impl<T: Copy> ListCore<T> {
    fn new(
        fields: &'static [(i32, i32)],
        mut st: ListState,
        will_filter: bool,
        entries: Vec<Entry<T>>,
    ) -> Self {
        st.filter = st.filter.trim().to_owned();
        let mut rows: Vec<Entry<T>> = if will_filter {
            entries.into_iter().filter(|e| !e.filtered).collect()
        } else {
            entries
        };
        let c = st.comparer;
        muo_primitives::dotnet::net_sort(&mut rows, |a, b| a.keys[c].cmp(&b.keys[c])); // List.Sort
        st.start = st.start.min(rows.len() as i32 - 1).max(0);
        Self { fields, st, rows }
    }

    /// `BuildContent`, with the gump's `DrawEndingEntry` and `BuildListExtras`.
    fn build(
        &self,
        b: &mut DynamicGumpBuilder,
        ending: impl Fn(&mut DynamicGumpBuilder, i32),
        extras: impl Fn(&mut DynamicGumpBuilder),
    ) {
        let st = &self.st;
        b.add_background(130, 75, 385, 30, 0xBB8);
        b.add_text_entry(135, 80, 375, 30, 0x481, 1, &st.filter);
        b.add_button(520, 75, 0x867, 0x868, 5); // Filter Button
        let mut width = 0;
        for (i, &(name, fw)) in self.fields.iter().enumerate() {
            b.add_image_tiled(65 + width, 110, fw + 10, 26, 0xA40);
            b.add_image_tiled(67 + width, 112, fw + 6, 22, 0xBBC);
            add_html_text(b, 70 + width, 113, fw, 20, &Txt::Num(name), false, false);
            let is_comparer = i == st.comparer;
            let button_id = if is_comparer {
                if st.ascending { 0x983 } else { 0x985 }
            } else {
                0x2716
            };
            b.add_button(
                59 + width + fw,
                117,
                button_id,
                button_id + i32::from(is_comparer),
                100 + i as i32,
            );
            width += fw + 12;
        }
        let count = self.rows.len() as i32;
        if st.start <= 0 {
            b.add_button_type(65, 80, 0x15E3, 0x15E7, 0, GumpButtonType::Page, 0);
        } else {
            b.add_button(65, 80, 0x15E3, 0x15E7, 6); // Back
        }
        if st.start + ITEMS_PER_PAGE > count {
            b.add_button_type(95, 80, 0x15E1, 0x15E5, 0, GumpButtonType::Page, 0);
        } else {
            b.add_button(95, 80, 0x15E1, 0x15E5, 7); // Forward
        }
        let mut item_number = 0;
        if st.ascending {
            let mut i = st.start;
            while i < st.start + ITEMS_PER_PAGE && i < count {
                self.draw_entry(b, &self.rows[i as usize], i, item_number);
                item_number += 1;
                i += 1;
            }
        } else {
            // descending, go from bottom of list to the top
            let mut i = count - 1 - st.start;
            while i >= 0 && i >= count - ITEMS_PER_PAGE - st.start {
                self.draw_entry(b, &self.rows[i as usize], i, item_number);
                item_number += 1;
                i -= 1;
            }
        }
        ending(b, item_number);
        extras(b);
    }

    fn draw_entry(&self, b: &mut DynamicGumpBuilder, e: &Entry<T>, index: i32, item_number: i32) {
        let mut width = 0;
        for (j, &(_, fw)) in self.fields.iter().enumerate() {
            b.add_image_tiled(65 + width, 138 + item_number * 28, fw + 10, 26, 0xA40);
            b.add_image_tiled(67 + width, 140 + item_number * 28, fw + 6, 22, 0xBBC);
            add_html_text(
                b,
                70 + width,
                141 + item_number * 28,
                fw,
                20,
                &e.values[j],
                false,
                false,
            );
            width += fw + 12;
        }
        if e.rel {
            b.add_button(40, 143 + item_number * 28, 0x8AF, 0x8AF, 200 + index); // Info Button
        } else {
            b.add_button(40, 143 + item_number * 28, 0x4B9, 0x4BA, 200 + index); // Info Button
        }
    }

    /// `BaseGuildListGump.OnResponse` after the base's: what to do.
    fn respond(&mut self, info: &RelayInfo) -> ListAction<T> {
        let id = info.button_id;
        match id {
            5 => {
                self.st.filter = info.get_text_entry(1).unwrap_or("").trim().to_owned();
                self.st.start = 0;
                return ListAction::Resend;
            }
            6 => {
                self.st.start -= ITEMS_PER_PAGE;
                return ListAction::Resend;
            }
            7 => {
                self.st.start += ITEMS_PER_PAGE;
                return ListAction::Resend;
            }
            _ => {}
        }
        if id >= 100 && id < 100 + self.fields.len() as i32 {
            let c = (id - 100) as usize;
            if self.st.comparer == c {
                self.st.ascending = !self.st.ascending;
            } else {
                self.st.comparer = c;
            }
            self.st.start = 0;
            ListAction::Resend
        } else if id >= 200 {
            self.rows
                .get((id - 200) as usize)
                .map_or(ListAction::None, |e| ListAction::Open(e.id))
        } else {
            ListAction::None
        }
    }
}

// ---- GuildRosterGump ----

const ROSTER_FIELDS: [(i32, i32); 4] =
    [(1062955, 130), (1062956, 80), (1062952, 80), (1062953, 150)];
/// `LastOnComparer`.
const LAST_ON: usize = 2;

pub struct GuildRosterGump {
    guild: GuildId,
    core: ListCore<MobileId>,
}

/// `new GuildRosterGump(pm, g)`.
pub fn roster(w: &World, pm: MobileId, g: GuildId) -> DynamicGumpObj<GuildRosterGump> {
    roster_with(w, pm, g, LAST_ON, false, "", 0)
}

/// `new GuildRosterGump(pm, g, comparer, ascending, filter, startNumber)`.
pub fn roster_with(
    w: &World,
    pm: MobileId,
    g: GuildId,
    comparer: usize,
    ascending: bool,
    filter: &str,
    start: i32,
) -> DynamicGumpObj<GuildRosterGump> {
    let members: Vec<MobileId> = gc::data(w, g)
        .map(|d| {
            d.members
                .iter()
                .copied()
                .filter(|&m| muo_content::mx::player(w, m))
                .collect()
        })
        .unwrap_or_default();
    let fealty = w.mobile(pm).and_then(|m| m.guild_fealty);
    let leader = leader_of(w, g);
    let filter_t = filter.trim().lower_inv();
    let entries = members
        .into_iter()
        .map(|m| {
            let online = ns_of(w, m).is_some();
            let mut name = format!(
                "{}{}",
                name_of(w, m),
                if fealty == Some(m) && fealty != leader {
                    " *"
                } else {
                    ""
                }
            );
            if m == pm {
                name = html::color_rgb(&name, 0x006600);
            } else if online {
                name = html::color_rgb(&name, 0x000066);
            }
            let last_on = pmob::data(w, m).map_or(muo_primitives::DateTime::MIN, |d| d.last_online);
            let raw_title = w.mobile(m).and_then(|x| x.guild_title.clone());
            let title_key = raw_title.as_deref().map_or(Key::Null, skey);
            let title = raw_title.unwrap_or_default();
            Entry {
                id: m,
                values: vec![
                    Txt::Str(name),
                    Txt::Num(RANKS[rank_of(w, m)].name),
                    if online {
                        Txt::Num(1063015)
                    } else {
                        Txt::Str(ymd(last_on))
                    },
                    Txt::Str(title.clone()),
                ],
                rel: false,
                filtered: !name_of(w, m).lower_inv().contains(&filter_t),
                keys: vec![
                    skey(&name_of(w, m)),
                    Key::I(rank_of(w, m) as i64),
                    Key::I(if online { i64::MAX } else { last_on.ticks() }),
                    title_key,
                ],
            }
        })
        .collect();
    let st = ListState {
        comparer,
        ascending,
        filter: filter.to_owned(),
        start,
    };
    let will_filter = !filter.trim().is_empty();
    DynamicGumpObj::new(
        10,
        10,
        GuildRosterGump {
            guild: g,
            core: ListCore::new(&ROSTER_FIELDS, st, will_filter, entries),
        },
    )
}

impl DynamicGumpLogic for GuildRosterGump {
    const TYPE_NAME: &'static str = "Server.Guilds.GuildRosterGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        start(b, true);
        self.core.build(
            b,
            |b, item_number| {
                b.add_background(225, 148 + item_number * 28, 150, 26, 0x2486);
                b.add_button(230, 153 + item_number * 28, 0x845, 0x846, 8);
                hl(b, 255, 151 + item_number * 28, 110, 26, 1062992, 0x0); // Invite Player
            },
            |b| hl(b, 266, 43, 110, 26, 1062974, 0xF), // Guild Roster
        );
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let g = self.guild;
        base_response(w, sender, g, info);
        let Some(pm) = member_of(w, sender, g) else {
            return;
        };
        if info.button_id == 8 {
            if can(w, pm, RankFlags::CAN_INVITE_PLAYER) {
                msg(w, pm, 1063048); // Whom do you wish to invite into your guild?
                muo_server::target::begin_target(
                    w,
                    pm,
                    -1,
                    false,
                    TargetFlags::NONE,
                    move |w, from, targeted| invite_player(w, from, targeted, g),
                );
            } else {
                msg(w, pm, 503301); // You don't have permission to do that.
            }
            return;
        }
        match self.core.respond(info) {
            ListAction::Resend => {
                let st = self.core.st.clone();
                let gump = roster_with(w, pm, g, st.comparer, st.ascending, &st.filter, st.start);
                send(w, pm, gump);
            }
            ListAction::Open(m) => {
                let gump = member_info(w, pm, g, m, false, false);
                send(w, pm, gump);
            }
            ListAction::None => {}
        }
    }
}

fn invite_player(w: &mut World, from: MobileId, targeted: Targeted, g: GuildId) {
    let pm = Some(from).filter(|&m| muo_content::mx::player(w, m));
    let targ = match targeted {
        Targeted::Entity(EntityId::Mobile(m)) if muo_content::mx::player(w, m) => Some(m),
        _ => None,
    };
    let leader = leader_of(w, g);
    let guild_faction = faction_of(w, leader).map(|f| f.0);
    let target_faction = faction_of(w, targ).map(|f| f.0);
    let Some(pm) = pm.filter(|&p| is_member(w, p, g) && can(w, p, RankFlags::CAN_INVITE_PLAYER))
    else {
        msg(w, from, 503301);
        return;
    };
    let Some(targ) = targ else {
        msg(w, pm, 1063334); // That isn't a valid player.
        return;
    };
    let tname = name_of(w, targ);
    let accepts = pmob::data(w, targ).is_some_and(|d| d.flag(PlayerFlag::ACCEPT_GUILD_INVITES));
    if !accepts {
        msg_a(w, pm, 1063049, &tname);
    } else if gc::is_member(w, g, targ) {
        msg_a(w, pm, 1063050, &tname);
    } else if w.mobile(targ).is_some_and(|m| m.guild.is_some()) {
        msg_a(w, pm, 1063051, &tname);
    } else if ns_of(w, targ)
        .is_some_and(|ns| has_base_guild_gump(w, ns) || has_gump::<CreateGuildGump>(w, ns))
    {
        msg_a(w, pm, 1063052, &tname);
    } else if pmob::young(w, targ) && guild_faction.is_some() {
        msg(w, pm, 1070766);
    } else if guild_faction != target_faction {
        if guild_faction.is_none() {
            msg(w, pm, 1013027);
        } else if target_faction.is_none() {
            msg(w, pm, 1013026);
        } else {
            msg(w, pm, 1013028);
        }
    } else if hooks(w)
        .and_then(|h| h.faction_is_leaving)
        .is_some_and(|f| f(w, targ))
    {
        muo_content::mx::send_message(
            w,
            pm,
            "That person is quitting their faction and so you may not recruit them.",
        );
    } else {
        msg_a(w, pm, 1063053, &tname);
        let gump = invitation_request(w, targ, g, pm);
        send(w, targ, gump);
    }
}

// ---- GuildDiplomacyGump ----

/// `GuildDisplayType`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum GuildDisplayType {
    All = 0,
    AwaitingAction = 1,
    Relations = 2,
}

const DIPLOMACY_FIELDS: [(i32, i32); 3] = [(1062954, 280), (1062957, 50), (1062958, 120)];

pub struct GuildDiplomacyGump {
    guild: GuildId,
    display: GuildDisplayType,
    lower: Txt,
    /// The list the gump was given (`_originalList`).
    original: Vec<GuildId>,
    core: ListCore<GuildId>,
    /// `AllianceRosterGump`: `AllowAdvancedSearch => false`, and button 8 is ignored.
    roster: bool,
}

/// `new AllianceInfo.AllianceRosterGump(pm, g, alliance)`: the diplomacy list over the alliance's members, with
/// the alliance's name as its lower text.
pub fn alliance_roster(
    w: &mut World,
    pm: MobileId,
    g: GuildId,
    k: gc::AllianceKey,
) -> DynamicGumpObj<GuildDiplomacyGump> {
    let members = gc::alliance_members(w, k);
    let name = gc::alliance_name(w, k);
    let mut gump = diplomacy_with(
        w,
        pm,
        g,
        0,
        true,
        "",
        0,
        GuildDisplayType::All,
        Some(members),
        Some(Txt::Str(name)),
    );
    set_roster(&mut gump, true);
    gump
}

/// `AllianceInfo.AllianceRosterGump` is its own class: its name is what the type id hashes.
fn set_roster(g: &mut DynamicGumpObj<GuildDiplomacyGump>, roster: bool) {
    g.logic.roster = roster;
    g.core.type_id = muo_content::gumps::base::get_type_id(g.logic.gump_type_name());
}

/// `HasRelationship(g)`.
fn has_relationship(w: &mut World, me: GuildId, g: GuildId) -> bool {
    if g == me {
        return false;
    }
    if gc::find_pending_war(w, me, Some(g)).is_some() {
        return true;
    }
    if let Some(k) = gc::alliance(w, me) {
        if let Some(leader) = gc::alliance_leader(w, k) {
            if me == leader && gc::alliance_is_pending_member(w, k, g)
                || g == leader && gc::alliance_is_pending_member(w, k, me)
            {
                return true;
            }
        } else if gc::alliance_is_pending_member(w, k, g) {
            return true;
        }
    }
    false
}

/// `new GuildDiplomacyGump(pm, g)`.
pub fn diplomacy(w: &mut World, pm: MobileId, g: GuildId) -> DynamicGumpObj<GuildDiplomacyGump> {
    diplomacy_with(w, pm, g, 0, true, "", 0, GuildDisplayType::All, None, None)
}

/// The constructors with a list and lower text (`list` None is `World.Guilds.Values`; `lower` None is
/// `1063136 + display`).
#[allow(clippy::too_many_arguments)]
pub fn diplomacy_with(
    w: &mut World,
    _pm: MobileId,
    g: GuildId,
    comparer: usize,
    ascending: bool,
    filter: &str,
    start: i32,
    display: GuildDisplayType,
    list: Option<Vec<GuildId>>,
    lower: Option<Txt>,
) -> DynamicGumpObj<GuildDiplomacyGump> {
    let original = list.unwrap_or_else(|| guild::guild_ids(w));
    let lower = lower.unwrap_or(Txt::Num(1063136 + display as i32));
    let filter_t = filter.trim().lower_inv();
    let mut entries = Vec::new();
    for &o in &original {
        let name = gname(w, o);
        let abbr = abbreviation(w, o);
        let ally = gc::is_ally(w, g, o);
        let war = gc::is_war(w, g, Some(o));
        let status = if ally {
            1
        } else if war {
            2
        } else {
            0
        };
        let status_txt = if ally {
            let k = gc::alliance(w, g);
            if k.and_then(|k| gc::alliance_leader(w, k)) == Some(o) {
                Txt::Num(1063237) // Alliance Leader
            } else {
                Txt::Num(1062964) // Ally
            }
        } else if war {
            Txt::Num(3000086) // War
        } else {
            Txt::Num(3000085) // Peace
        };
        let rel = has_relationship(w, g, o);
        let filtered = match display {
            GuildDisplayType::Relations => !(gc::find_active_war(w, g, Some(o)).is_some() || ally),
            GuildDisplayType::AwaitingAction => !rel,
            GuildDisplayType::All => {
                !(name.lower_inv().contains(&filter_t) || abbr.lower_inv().contains(&filter_t))
            }
        };
        entries.push(Entry {
            id: o,
            values: vec![
                if o == g {
                    Txt::Str(html::color_rgb(&name, 0x006600))
                } else {
                    Txt::Str(name.clone())
                },
                Txt::Str(abbr.clone()),
                status_txt,
            ],
            rel,
            filtered,
            keys: vec![skey(&name), skey(&abbr), Key::I(status)],
        });
    }
    let st = ListState {
        comparer,
        ascending,
        filter: filter.to_owned(),
        start,
    };
    let will_filter = display != GuildDisplayType::All || !filter.trim().is_empty();
    DynamicGumpObj::new(
        10,
        10,
        GuildDiplomacyGump {
            guild: g,
            display,
            lower,
            original,
            core: ListCore::new(&DIPLOMACY_FIELDS, st, will_filter, entries),
            roster: false,
        },
    )
}

impl DynamicGumpLogic for GuildDiplomacyGump {
    const TYPE_NAME: &'static str = "Server.Guilds.GuildDiplomacyGump";

    fn gump_type_name(&self) -> &'static str {
        if self.roster {
            "Server.Guilds.AllianceInfo+AllianceRosterGump"
        } else {
            Self::TYPE_NAME
        }
    }

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        start(b, true);
        let lower = self.lower.clone();
        let allow_advanced_search = !self.roster;
        self.core.build(
            b,
            move |b, item_number| {
                match &lower {
                    Txt::Num(n) if *n > 0 => hl(b, 66, 153 + item_number * 28, 280, 26, *n, 0xF),
                    Txt::Str(s) => b.add_html(
                        66,
                        153 + item_number * 28,
                        280,
                        26,
                        &html::color_rgb(s, 0x99),
                        false,
                        false,
                    ),
                    Txt::Num(_) => {}
                }
                if allow_advanced_search {
                    b.add_background(350, 148 + item_number * 28, 200, 26, 0x2486);
                    b.add_button(355, 153 + item_number * 28, 0x845, 0x846, 8);
                    hl(b, 380, 151 + item_number * 28, 160, 26, 1063083, 0x0); // Advanced Search
                }
            },
            |b| hl(b, 431, 43, 110, 26, 1062978, 0xF), // Diplomacy
        );
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        if self.roster && info.button_id == 8 {
            return; // So that they can't get to the AdvancedSearch button
        }
        let g = self.guild;
        base_response(w, sender, g, info);
        let Some(pm) = member_of(w, sender, g) else {
            return;
        };
        if info.button_id == 8 {
            let st = self.core.st.clone();
            let (original, lower) = (self.original.clone(), self.lower.clone());
            let cb: SearchCallback = Rc::new(move |w: &mut World, d: GuildDisplayType| {
                let gump = diplomacy_with(
                    w,
                    pm,
                    g,
                    st.comparer,
                    st.ascending,
                    &st.filter,
                    st.start,
                    d,
                    Some(original.clone()),
                    Some(lower.clone()),
                );
                send(w, pm, gump);
            });
            let gump = advanced_search(g, self.display, cb);
            send(w, pm, gump);
            return;
        }
        match self.core.respond(info) {
            ListAction::Resend => {
                let st = self.core.st.clone();
                let mut gump = diplomacy_with(
                    w,
                    pm,
                    g,
                    st.comparer,
                    st.ascending,
                    &st.filter,
                    st.start,
                    self.display,
                    Some(self.original.clone()),
                    Some(self.lower.clone()),
                );
                set_roster(&mut gump, self.roster);
                send(w, pm, gump);
            }
            ListAction::Open(o) => {
                if o == g {
                    send_info(w, pm, g);
                } else {
                    let gump = other_guild_info(w, pm, g, o);
                    send(w, pm, gump);
                }
            }
            ListAction::None => {}
        }
    }
}

// ---- GuildAdvancedSearchGump ----

pub type SearchCallback = Rc<dyn Fn(&mut World, GuildDisplayType)>;

pub struct GuildAdvancedSearchGump {
    guild: GuildId,
    display: GuildDisplayType,
    callback: SearchCallback,
}

/// `new GuildAdvancedSearchGump(pm, g, display, callback)`.
pub fn advanced_search(
    g: GuildId,
    display: GuildDisplayType,
    callback: SearchCallback,
) -> DynamicGumpObj<GuildAdvancedSearchGump> {
    DynamicGumpObj::new(
        10,
        10,
        GuildAdvancedSearchGump {
            guild: g,
            display,
            callback,
        },
    )
}

impl DynamicGumpLogic for GuildAdvancedSearchGump {
    const TYPE_NAME: &'static str = "Server.Guilds.GuildAdvancedSearchGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        start(b, true);
        hl(b, 431, 43, 110, 26, 1062978, 0xF); // Diplomacy
        hlb(b, 65, 80, 480, 26, 1063124, 0xF); // <i>Advanced Search Options</i>
        // Showing All Guilds/w/Relation/Waiting Relation
        hl(b, 65, 110, 480, 26, 1063136 + self.display as i32, 0xF);
        b.add_group(1);
        b.add_radio(75, 140, 0xD2, 0xD3, false, 2);
        hl(b, 105, 140, 200, 26, 1063006, 0x0); // Show Guilds with Relationship
        b.add_radio(75, 170, 0xD2, 0xD3, false, 1);
        hl(b, 105, 170, 200, 26, 1063005, 0x0); // Show Guilds Awaiting Action
        b.add_radio(75, 200, 0xD2, 0xD3, false, 0);
        hl(b, 105, 200, 200, 26, 1063007, 0x0); // Show All Guilds
        b.add_background(450, 370, 100, 26, 0x2486);
        b.add_button(455, 375, 0x845, 0x846, 5);
        hl(b, 480, 373, 60, 26, 1006044, 0x0); // OK
        b.add_background(340, 370, 100, 26, 0x2486);
        b.add_button(345, 375, 0x845, 0x846, 0);
        hl(b, 370, 373, 60, 26, 1006045, 0x0); // Cancel
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let g = self.guild;
        base_response(w, sender, g, info);
        if member_of(w, sender, g).is_none() {
            return;
        }
        if info.button_id == 5 {
            for i in 0..3 {
                if info.is_switched(i) {
                    let d = match i {
                        0 => GuildDisplayType::All,
                        1 => GuildDisplayType::AwaitingAction,
                        _ => GuildDisplayType::Relations,
                    };
                    (self.callback.clone())(w, d);
                    break;
                }
            }
        }
    }
}

// ---- GuildInvitationRequest ----

pub struct GuildInvitationRequest {
    player: MobileId,
    guild: GuildId,
    inviter: MobileId,
    name: String,
}

/// `new GuildInvitationRequest(pm, g, inviter)`.
pub fn invitation_request(
    w: &World,
    pm: MobileId,
    g: GuildId,
    inviter: MobileId,
) -> DynamicGumpObj<GuildInvitationRequest> {
    DynamicGumpObj::new(
        10,
        10,
        GuildInvitationRequest {
            player: pm,
            guild: g,
            inviter,
            name: gname(w, g),
        },
    )
}

impl DynamicGumpLogic for GuildInvitationRequest {
    const TYPE_NAME: &'static str = "Server.Guilds.GuildInvitationRequest";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        start(b, false);
        b.add_background(0, 0, 350, 170, 0x2422);
        // <center>You have been invited to join a guild! (Warning: Accepting will make you attackable!)</center>
        hlb(b, 25, 20, 300, 45, 1062946, 0x0);
        b.add_html(
            25,
            75,
            300,
            25,
            &format!("<center>{}</center>", self.name),
            true,
            false,
        );
        b.add_button(265, 130, 0xF7, 0xF8, 1);
        b.add_button(195, 130, 0xF2, 0xF1, 0);
        b.add_button(20, 130, 0xD2, 0xD3, 2);
        hl(b, 45, 130, 150, 30, 1062943, 0x0); // <i>Ignore Guild Invites</i>
    }

    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        let (g, pm, inviter) = (self.guild, self.player, self.inviter);
        if gc::disbanded(w, g) || w.mobile(pm).is_some_and(|m| m.guild.is_some()) {
            return;
        }
        match info.button_id {
            0 => {
                // ~1_val~ has declined your invitation to join ~2_val~.
                msg_a(
                    w,
                    inviter,
                    1063250,
                    &format!("{}\t{}", name_of(w, pm), gname(w, g)),
                );
            }
            1 => {
                gc::add_member(w, g, pm);
                msg_a(w, pm, 1063056, &gname(w, g)); // You have joined ~1_val~.
                msg_a(
                    w,
                    inviter,
                    1063249,
                    &format!("{}\t{}", name_of(w, pm), gname(w, g)),
                );
            }
            2 => {
                if let Some(d) = pmob::data_mut(w, pm) {
                    d.set_flag(PlayerFlag::ACCEPT_GUILD_INVITES, false);
                }
                msg(w, pm, 1070698); // You are now ignoring guild invitations.
            }
            _ => {}
        }
    }
}

// ---- GuildMemberInfoGump ----

pub struct GuildMemberInfoGump {
    player: MobileId,
    guild: GuildId,
    member: MobileId,
    to_kick: bool,
    to_leader: bool,
    name: String,
    rank_name: i32,
    title: String,
    vote_text: i32,
}

/// `new GuildMemberInfoGump(pm, g, member, toKick, toPromoteToLeader)`.
pub fn member_info(
    w: &World,
    pm: MobileId,
    g: GuildId,
    member: MobileId,
    to_kick: bool,
    to_leader: bool,
) -> DynamicGumpObj<GuildMemberInfoGump> {
    let fealty = w.mobile(pm).and_then(|m| m.guild_fealty);
    DynamicGumpObj::new(
        10,
        40,
        GuildMemberInfoGump {
            player: pm,
            guild: g,
            member,
            to_kick,
            to_leader,
            name: name_of(w, member),
            rank_name: RANKS[rank_of(w, member)].name,
            title: w
                .mobile(member)
                .and_then(|m| m.guild_title.clone())
                .unwrap_or_default(),
            vote_text: if fealty == Some(member) && leader_of(w, g) != Some(member) {
                1063082
            } else {
                1062996
            },
        },
    )
}

impl DynamicGumpLogic for GuildMemberInfoGump {
    const TYPE_NAME: &'static str = "Server.Guilds.GuildMemberInfoGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        start(b, false);
        b.add_background(0, 0, 350, 255, 0x242C);
        hl(b, 20, 15, 310, 26, 1063018, 0x0); // <div align=center><i>Guild Member Information</i></div>
        b.add_image_tiled(20, 40, 310, 2, 0x2711);
        hlb(b, 20, 50, 150, 26, 1062955, 0x0); // <i>Name</i>
        b.add_html(180, 53, 150, 26, &self.name, false, false);
        hlb(b, 20, 80, 150, 26, 1062956, 0x0); // <i>Rank</i>
        hl(b, 180, 83, 150, 26, self.rank_name, 0x0);
        hlb(b, 20, 110, 150, 26, 1062953, 0x0); // <i>Guild Title</i>
        b.add_html(180, 113, 150, 26, &self.title, false, false);
        b.add_image_tiled(20, 142, 310, 2, 0x2711);
        b.add_background(20, 150, 310, 26, 0x2486);
        b.add_button(25, 155, 0x845, 0x846, 4);
        hl(b, 50, 153, 270, 26, self.vote_text, 0x0); // Clear/Cast Vote For This Member
        b.add_background(20, 180, 150, 26, 0x2486);
        b.add_button(25, 185, 0x845, 0x846, 1);
        hl(
            b,
            50,
            183,
            110,
            26,
            1062993,
            if self.to_leader { 0x990000 } else { 0 },
        ); // Promote
        b.add_background(180, 180, 150, 26, 0x2486);
        b.add_button(185, 185, 0x845, 0x846, 3);
        hl(b, 210, 183, 110, 26, 1062995, 0x0); // Set Guild Title
        b.add_background(20, 210, 150, 26, 0x2486);
        b.add_button(25, 215, 0x845, 0x846, 2);
        hl(b, 50, 213, 110, 26, 1062994, 0x0); // Demote
        b.add_background(180, 210, 150, 26, 0x2486);
        b.add_button(185, 215, 0x845, 0x846, 5);
        hl(
            b,
            210,
            213,
            110,
            26,
            1062997,
            if self.to_kick { 0x5000 } else { 0 },
        ); // Kick
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let (g, member, player) = (self.guild, self.member, self.player);
        let Some(pm) = member_of(w, sender, g) else {
            return;
        };
        if !is_member(w, member, g) {
            return;
        }
        let player_rank = rank_of(w, pm);
        let target_rank = rank_of(w, member);
        let mname = name_of(w, member);
        match info.button_id {
            1 => {
                // Promote
                if RANKS[player_rank].get_flag(RankFlags::CAN_PROMOTE_DEMOTE)
                    && (RANKS[player_rank].rank - 1 > RANKS[target_rank].rank
                        || player_rank as i32 == LEADER_INDEX
                            && RANKS[player_rank].rank > RANKS[target_rank].rank)
                {
                    let new_rank = target_rank + 1;
                    if new_rank as i32 == LEADER_INDEX {
                        if self.to_leader {
                            set_rank(w, member, new_rank as i32);
                            msg_a(w, pm, 1063156, &mname); // The guild information for ~1_val~ has been updated.
                            let pname = name_of(w, pm);
                            msg_a(w, pm, 1063156, &pname);
                            gc::set_leader(w, g, Some(member));
                        } else {
                            msg(w, pm, 1063144); // Are you sure you wish to make this member the new guild leader?
                            let gump = member_info(w, player, g, member, false, true);
                            send(w, pm, gump);
                        }
                    } else {
                        set_rank(w, member, new_rank as i32);
                        msg_a(w, pm, 1063156, &mname);
                    }
                } else {
                    msg(w, pm, 1063143); // You don't have permission to promote this member.
                }
            }
            2 => {
                // Demote
                if RANKS[player_rank].get_flag(RankFlags::CAN_PROMOTE_DEMOTE)
                    && RANKS[player_rank].rank > RANKS[target_rank].rank
                {
                    if target_rank as i32 == LOWEST_INDEX {
                        if RANKS[LOWEST_INDEX as usize].name == 1062963 {
                            msg(w, pm, 1063333); // You can't demote a ronin.
                        } else {
                            muo_content::mx::send_message(w, pm, "You can't demote a member.");
                        }
                    } else {
                        set_rank(w, member, target_rank as i32 - 1);
                        msg_a(w, pm, 1063156, &mname);
                    }
                } else {
                    msg(w, pm, 1063146); // You don't have permission to demote this member.
                }
            }
            3 => {
                // Set Guild title
                if RANKS[player_rank].get_flag(RankFlags::CAN_SET_GUILD_TITLE)
                    && (RANKS[player_rank].rank > RANKS[target_rank].rank || member == pm)
                {
                    msg(w, pm, 1011128); // Enter the new title for this guild member or 'none' to remove a title:
                    prompt(w, pm, false, move |w, from, text| {
                        set_title(w, from, member, text)
                    });
                } else if w
                    .mobile(member)
                    .and_then(|m| m.guild_title.as_deref())
                    .is_none_or(str::is_empty)
                {
                    msg(w, pm, 1070746);
                } else {
                    msg(w, pm, 1063148);
                }
            }
            4 => {
                // Vote
                let fealty = w.mobile(pm).and_then(|m| m.guild_fealty);
                if fealty == Some(member) && leader_of(w, g) != Some(member) {
                    msg(w, pm, 1063158); // You have cleared your vote for guild leader.
                } else if gc::can_vote(w, g, Some(member)) {
                    if leader_of(w, g) == Some(member) {
                        msg(w, pm, 1063424);
                    } else if !gc::can_be_voted_for(w, g, Some(member)) {
                        msg(w, pm, 1063425);
                    } else {
                        if let Some(m) = w.mobile_mut(pm) {
                            m.guild_fealty = Some(member);
                        }
                        msg_a(w, pm, 1063159, &mname);
                    }
                } else {
                    msg(w, pm, 1063149);
                }
            }
            5 => {
                // Kick
                if RANKS[player_rank].get_flag(RankFlags::REMOVE_PLAYERS)
                    && RANKS[player_rank].rank > RANKS[target_rank].rank
                    || RANKS[player_rank].get_flag(RankFlags::REMOVE_LOWEST_RANK)
                        && target_rank as i32 == LOWEST_INDEX
                {
                    if self.to_kick {
                        gc::remove_member(w, g, member, 1018028);
                        msg(w, pm, 1063157);
                    } else {
                        msg(w, pm, 1063152); // Are you sure you wish to kick this member from the guild?
                        let gump = member_info(w, player, g, member, true, false);
                        send(w, pm, gump);
                    }
                } else {
                    msg(w, pm, 1063151);
                }
            }
            _ => {}
        }
    }
}

fn set_title(w: &mut World, from: MobileId, member: MobileId, text: &str) {
    if !muo_content::mx::player(w, from) {
        return;
    }
    let Some(g) = w
        .mobile(member)
        .and_then(|m| m.guild)
        .filter(|&g| gc::data(w, g).is_some())
    else {
        return;
    };
    let from_rank = rank_of(w, from);
    let ok = is_member(w, from, g)
        && RANKS[from_rank].get_flag(RankFlags::CAN_SET_GUILD_TITLE)
        && (RANKS[from_rank].rank > RANKS[rank_of(w, member)].rank || from == member);
    if !ok {
        if w.mobile(member)
            .and_then(|m| m.guild_title.as_deref())
            .is_none_or(str::is_empty)
        {
            msg(w, from, 1070746);
        } else {
            msg(w, from, 1063148);
        }
        return;
    }
    let title = utility::fix_html(text.trim());
    if title.chars().count() > 20 {
        msg(w, from, 501178); // That title is too long.
    } else if !check_profanity(&title, 50) {
        msg(w, from, 501179); // That title is disallowed.
    } else {
        let t = if title.eq_ic("none") {
            None
        } else {
            Some(title)
        };
        muo_server::mobile::part1::props_b::set_guild_title(w, member, t);
        let mname = name_of(w, member);
        msg_a(w, from, 1063156, &mname);
    }
}

// ---- OtherGuildInfo ----

pub struct OtherGuildInfo {
    guild: GuildId,
    other: GuildId,
    other_name: String,
    other_abbr: String,
    other_alliance_name: Option<String>,
    kills: String,
    time: String,
    other_kills: String,
    number: i32,
    /// (x, y, button id, cliloc).
    buttons: Vec<(i32, i32, i32, i32)>,
    pending_alliance_name: Option<String>,
}

/// `new OtherGuildInfo(pm, g, otherGuild)`: runs `g.CheckExpiredWars()` first.
pub fn other_guild_info(
    w: &mut World,
    _pm: MobileId,
    g: GuildId,
    other: GuildId,
) -> DynamicGumpObj<OtherGuildInfo> {
    gc::check_expired_wars(w, g);
    let gl = gc::get_alliance_leader(w, g);
    let ol = gc::get_alliance_leader(w, other);
    let war = gc::find_pending_war(w, gl, Some(ol));
    let active = gc::find_active_war(w, gl, Some(ol));
    let alliance = gc::alliance(w, g);
    let other_alliance = gc::alliance(w, other);
    let color = |s: String| html::color_rgb(&s, 0x990000);
    let mut kills = "0/0".to_owned();
    let mut time = "00:00".to_owned();
    let mut other_kills = "0/0".to_owned();
    if let Some(i) = active {
        let a = gc::data(w, gl).map(|d| d.accepted_wars[i].clone()).unwrap();
        kills = format!("{}/{}", a.kills, a.max_kills);
        let mut remaining = TimeSpan::ZERO;
        if a.war_length != TimeSpan::ZERO && a.war_beginning + a.war_length > w.now {
            remaining = (a.war_beginning + a.war_length) - w.now;
        }
        time = format!(
            "{:02}:{:02}",
            hours_component(remaining),
            minutes_component(remaining)
        );
        if let Some(j) = gc::find_active_war(w, other, Some(g)) {
            let o = gc::data(w, other)
                .map(|d| d.accepted_wars[j].clone())
                .unwrap();
            other_kills = format!("{}/{}", o.kills, o.max_kills);
        }
    } else if let Some(i) = war {
        let p = gc::data(w, gl).map(|d| d.pending_wars[i].clone()).unwrap();
        kills = color(format!("{}/{}", p.kills, p.max_kills));
        time = color(format!(
            "{:02}:{:02}",
            hours_component(p.war_length),
            minutes_component(p.war_length)
        ));
        if let Some(j) = gc::find_pending_war(w, other, Some(g)) {
            let o = gc::data(w, other)
                .map(|d| d.pending_wars[j].clone())
                .unwrap();
            other_kills = color(format!("{}/{}", o.kills, o.max_kills));
        }
    }
    let mut buttons = Vec::new();
    let mut number = 1062973; // You are at peace with this guild.
    let mut pending_alliance_name = None;
    if let Some(i) = war {
        let p = gc::data(w, gl).map(|d| d.pending_wars[i].clone()).unwrap();
        if p.war_requester {
            number = 1063027;
        } else {
            number = 1062969;
            buttons.push((20, 260, 5, 1062981));
            buttons.push((275, 260, 6, 1062983));
        }
        buttons.push((20, 290, 7, 1062982)); // Dismiss Challenge, for both the challenger and the challenged
    } else if active.is_some() {
        number = 1062965;
        buttons.push((20, 290, 8, 1062980));
    } else if alliance.is_some() && alliance == other_alliance {
        let k = alliance.unwrap();
        let leader = gc::alliance_leader(w, k);
        if gc::alliance_is_member(w, k, g) && gc::alliance_is_member(w, k, other) {
            number = 1062970;
            if leader == Some(g) {
                buttons.push((20, 260, 12, 1062984));
                buttons.push((275, 260, 13, 1063433));
            }
            buttons.push((148, 215, 10, 1063164));
            buttons.push((20, 290, 11, 1062985));
        } else if leader == Some(g) && gc::alliance_is_pending_member(w, k, other) {
            number = 1062971;
            buttons.push((148, 245, 10, 1063164));
            buttons.push((20, 290, 14, 1062986));
            pending_alliance_name = Some(gc::alliance_name(w, k));
        } else if leader == Some(other) && gc::alliance_is_pending_member(w, k, g) {
            number = 1062972;
            buttons.push((148, 215, 10, 1063164));
            buttons.push((20, 260, 15, 1062988));
            buttons.push((20, 290, 16, 1062987));
            pending_alliance_name = Some(gc::alliance_name(w, k));
        }
    } else {
        buttons.push((20, 260, 2, 1062990));
        buttons.push((20, 290, 1, 1062989));
    }
    buttons.push((275, 290, 0, 3000091));
    DynamicGumpObj::new(
        10,
        40,
        OtherGuildInfo {
            guild: g,
            other,
            other_name: gname(w, other),
            other_abbr: abbreviation(w, other),
            other_alliance_name: other_alliance
                .filter(|&k| gc::alliance_is_member(w, k, other))
                .map(|k| gc::alliance_name(w, k)),
            kills,
            time,
            other_kills,
            number,
            buttons,
            pending_alliance_name,
        },
    )
}

fn button_and_background(b: &mut DynamicGumpBuilder, x: i32, y: i32, id: i32, loc: i32) {
    b.add_background(x, y, 225, 26, 0x2486);
    b.add_button(x + 5, y + 5, 0x845, 0x846, id);
    hl(b, x + 30, y + 3, 185, 26, loc, 0x0);
}

impl DynamicGumpLogic for OtherGuildInfo {
    const TYPE_NAME: &'static str = "Server.Guilds.OtherGuildInfo";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        start(b, false);
        b.add_background(0, 0, 520, 335, 0x242C);
        hl(b, 20, 15, 480, 26, 1062975, 0x0); // <div align=center><i>Guild Relationship</i></div>
        b.add_image_tiled(20, 40, 480, 2, 0x2711);
        hlb(b, 20, 50, 120, 26, 1062954, 0x0); // <i>Guild Name</i>
        b.add_html(150, 53, 360, 26, &self.other_name, false, false);
        hlb(b, 20, 80, 120, 26, 1063025, 0x0); // <i>Alliance</i>
        if let Some(n) = &self.other_alliance_name {
            b.add_html(150, 83, 360, 26, n, false, false);
        }
        hlb(b, 20, 110, 120, 26, 1063139, 0x0); // <i>Abbreviation</i>
        b.add_html(150, 113, 120, 26, &self.other_abbr, false, false);
        hlb(b, 280, 110, 120, 26, 1062966, 0x0); // <i>Your Kills</i>
        b.add_html(410, 113, 120, 26, &self.kills, false, false);
        hlb(b, 20, 140, 120, 26, 1062968, 0x0); // <i>Time Remaining</i>
        b.add_html(150, 143, 120, 26, &self.time, false, false);
        hlb(b, 280, 140, 120, 26, 1062967, 0x0); // <i>Their Kills</i>
        b.add_html(410, 143, 120, 26, &self.other_kills, false, false);
        b.add_image_tiled(20, 172, 480, 2, 0x2711);
        // the alliance name of a pending alliance is drawn right after the branch's last button
        let mut named = false;
        for &(x, y, id, loc) in &self.buttons {
            button_and_background(b, x, y, id, loc);
            if !named && matches!((self.number, id), (1062971, 14) | (1062972, 16)) {
                if let Some(n) = &self.pending_alliance_name {
                    b.add_html(150, 83, 360, 26, &html::color_rgb(n, 0x99), false, false);
                }
                named = true;
            }
        }
        hlb(b, 20, 180, 480, 30, self.number, 0x0);
        b.add_image_tiled(20, 245, 480, 2, 0x2711);
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let (g, other) = (self.guild, self.other);
        let Some(pm) = member_of(w, sender, g) else {
            return;
        };
        respond_other(w, pm, g, other, info.button_id);
    }
}

fn respond_other(w: &mut World, pm: MobileId, g: GuildId, other: GuildId, button: i32) {
    let player_rank = rank_of(w, pm);
    let flag = |f: RankFlags| RANKS[player_rank].get_flag(f);
    let guild_leader = gc::get_alliance_leader(w, g);
    let other_guild = gc::get_alliance_leader(w, other);
    let war = gc::find_pending_war(w, guild_leader, Some(other_guild));
    let active = gc::find_active_war(w, guild_leader, Some(other_guild));
    let other_war = gc::find_pending_war(w, other_guild, Some(guild_leader));
    let alliance = gc::alliance(w, g);
    let other_alliance = gc::alliance(w, other_guild);
    let gn = gname(w, g);
    let on = gname(w, other);
    let war_requester =
        war.and_then(|i| gc::data(w, guild_leader).map(|d| d.pending_wars[i].war_requester));

    // `alliance != null && alliance.Leader != Guild`, with its two messages
    let not_led = |w: &mut World| -> bool {
        let Some(k) = alliance else { return false };
        if gc::alliance_leader(w, k) == Some(g) {
            return false;
        }
        msg_a(
            w,
            pm,
            1063239,
            &format!("{gn}\t{}", gc::alliance_name(w, k)),
        );
        if let Some(l) = gc::alliance_leader(w, k) {
            let ln = gname(w, l);
            msg_a(w, pm, 1070707, &ln);
        }
        true
    };

    match button {
        5 => {
            if war_requester == Some(false) && active.is_none() {
                if !flag(RankFlags::CONTROL_WAR_STATUS) {
                    msg(w, pm, 1063440);
                } else if !not_led(w) {
                    // Accept the war
                    let now = w.now;
                    if let (Some(wi), Some(d)) = (war, gc::data_mut(w, g)) {
                        let mut x = d.pending_wars.remove(wi);
                        x.war_beginning = now;
                        d.accepted_wars.push(x);
                    }
                    let target = other_alliance
                        .map(|k| gc::alliance_name(w, k))
                        .unwrap_or_else(|| gname(w, other_guild));
                    if let Some(k) = alliance.filter(|&k| gc::alliance_is_member(w, k, g)) {
                        gc::alliance_message_args(w, k, 1070769, &target, 0x3B2);
                        gc::alliance_invalidate_member_properties(w, k, false);
                    } else {
                        gc::guild_message_args(w, g, 1070769, &target, 0x3B2);
                        gc::invalidate_member_properties(w, g, false);
                    }
                    if let (Some(ow), Some(od)) = (other_war, gc::data_mut(w, other_guild)) {
                        let mut x = od.pending_wars.remove(ow);
                        x.war_beginning = now;
                        od.accepted_wars.push(x);
                    }
                    let mine = alliance
                        .map(|k| gc::alliance_name(w, k))
                        .unwrap_or_else(|| gname(w, g));
                    if let Some(k) = other_alliance.filter(|&k| gc::alliance_is_member(w, k, other))
                    {
                        gc::alliance_message_args(w, k, 1070769, &mine, 0x3B2);
                        gc::alliance_invalidate_member_properties(w, k, false);
                    } else {
                        gc::guild_message_args(w, other_guild, 1070769, &mine, 0x3B2);
                        gc::invalidate_member_properties(w, other_guild, false);
                    }
                }
            }
        }
        6 => {
            if war_requester == Some(false) && active.is_none() {
                if !flag(RankFlags::CONTROL_WAR_STATUS) {
                    msg(w, pm, 1063440);
                } else if !not_led(w) {
                    let gump = war_declaration(w, pm, g, other_guild);
                    send(w, pm, gump);
                }
            }
        }
        7 => {
            if let Some(wi) = war {
                if !flag(RankFlags::CONTROL_WAR_STATUS) {
                    msg(w, pm, 1063440);
                } else if !not_led(w) {
                    if let Some(d) = gc::data_mut(w, g) {
                        d.pending_wars.remove(wi);
                    }
                    if let (Some(ow), Some(od)) = (other_war, gc::data_mut(w, other_guild)) {
                        od.pending_wars.remove(ow);
                    }
                    msg(w, pm, 1070752); // The proposal has been updated.
                }
            }
        }
        8 => {
            if !flag(RankFlags::CONTROL_WAR_STATUS) {
                msg(w, pm, 1063440);
            } else if !not_led(w)
                && let Some(ai) = active
            {
                let target = other_alliance
                    .map(|k| gc::alliance_name(w, k))
                    .unwrap_or_else(|| gname(w, other_guild));
                if let Some(k) = alliance.filter(|&k| gc::alliance_is_member(w, k, g)) {
                    gc::alliance_message_args(w, k, 1070740, &target, 0x3B2);
                    gc::alliance_invalidate_member_properties(w, k, false);
                } else {
                    gc::guild_message_args(w, g, 1070740, &target, 0x3B2);
                    gc::invalidate_member_properties(w, g, false);
                }
                if let Some(d) = gc::data_mut(w, g)
                    && ai < d.accepted_wars.len()
                {
                    d.accepted_wars.remove(ai);
                }
                let mine = alliance
                    .map(|k| gc::alliance_name(w, k))
                    .unwrap_or_else(|| gname(w, g));
                if let Some(k) =
                    other_alliance.filter(|&k| gc::alliance_is_member(w, k, other_guild))
                {
                    gc::alliance_message_args(w, k, 1070739, &mine, 0x3B2);
                    gc::alliance_invalidate_member_properties(w, k, false);
                } else {
                    gc::guild_message_args(w, other_guild, 1070739, &mine, 0x3B2);
                    gc::invalidate_member_properties(w, other_guild, false);
                }
                if let Some(oi) = gc::find_active_war(w, other_guild, Some(g))
                    && let Some(od) = gc::data_mut(w, other_guild)
                {
                    od.accepted_wars.remove(oi);
                }
            }
        }
        1 => {
            if war.is_none() && active.is_none() {
                if !flag(RankFlags::CONTROL_WAR_STATUS) {
                    msg(w, pm, 1063440);
                } else if not_led(w) {
                } else if let Some(k) =
                    other_alliance.filter(|&k| gc::alliance_leader(w, k) != Some(other))
                {
                    msg_a(
                        w,
                        pm,
                        1063239,
                        &format!("{on}\t{}", gc::alliance_name(w, k)),
                    );
                    if let Some(l) = gc::alliance_leader(w, k) {
                        let ln = gname(w, l);
                        msg_a(w, pm, 1070707, &ln);
                    }
                } else {
                    let gump = war_declaration(w, pm, g, other);
                    send(w, pm, gump);
                }
            }
        }
        2 => request_alliance(w, pm, g, other, alliance, other_alliance),
        10 => {
            if let (Some(k), true) = (alliance, alliance == other_alliance) {
                let gump = alliance_roster(w, pm, g, k);
                send(w, pm, gump);
            }
        }
        11 => {
            if !flag(RankFlags::ALLIANCE_CONTROL) {
                msg(w, pm, 1063436);
            } else if alliance.is_some_and(|k| gc::alliance_is_member(w, k, g)) {
                gc::set_alliance(w, g, None);
                gc::invalidate_war_notoriety(w, other);
                gc::invalidate_member_notoriety(w, g);
            }
        }
        12 => {
            if !flag(RankFlags::ALLIANCE_CONTROL) {
                msg(w, pm, 1063436);
            } else if let Some(k) = alliance.filter(|&k| gc::alliance_leader(w, k) != Some(g)) {
                msg_a(
                    w,
                    pm,
                    1063239,
                    &format!("{gn}\t{}", gc::alliance_name(w, k)),
                );
            } else if alliance.is_some_and(|k| {
                gc::alliance_is_member(w, k, g) && gc::alliance_is_member(w, k, other)
            }) {
                gc::set_alliance(w, other, None);
                gc::invalidate_member_notoriety(w, other);
                gc::invalidate_war_notoriety(w, g);
            }
        }
        13 => {
            if !flag(RankFlags::ALLIANCE_CONTROL) {
                msg(w, pm, 1063436);
            } else if let Some(k) = alliance.filter(|&k| gc::alliance_leader(w, k) != Some(g)) {
                msg_a(
                    w,
                    pm,
                    1063239,
                    &format!("{gn}\t{}", gc::alliance_name(w, k)),
                );
            } else if let Some(k) = alliance
                .filter(|&k| gc::alliance_is_member(w, k, g) && gc::alliance_is_member(w, k, other))
            {
                msg_a(
                    w,
                    pm,
                    1063434,
                    &format!("{on}\t{}", gc::alliance_name(w, k)),
                );
                gc::alliance_set_leader(w, k, Some(other));
            }
        }
        14 => {
            if !flag(RankFlags::ALLIANCE_CONTROL) {
                msg(w, pm, 1063436);
            } else if alliance.is_some_and(|k| {
                gc::alliance_leader(w, k) == Some(g) && gc::alliance_is_pending_member(w, k, other)
            }) {
                gc::set_alliance(w, other, None);
                msg(w, pm, 1070752);
            }
        }
        15 => {
            if !flag(RankFlags::ALLIANCE_CONTROL) {
                msg(w, pm, 1063436);
            } else if let (Some(k), Some(ok)) = (alliance, other_alliance)
                && gc::alliance_leader(w, k) == Some(other)
                && gc::alliance_is_pending_member(w, ok, g)
            {
                msg(w, pm, 1070752);
                gc::set_alliance(w, g, None);
            }
        }
        16 => {
            if !flag(RankFlags::ALLIANCE_CONTROL) {
                msg(w, pm, 1063436);
            } else if let Some(ok) = other_alliance.filter(|&ok| {
                gc::alliance_leader(w, ok) == Some(other)
                    && gc::alliance_is_pending_member(w, ok, g)
            }) {
                msg(w, pm, 1070752);
                gc::alliance_turn_to_member(w, ok, other);
                gc::alliance_turn_to_member(w, ok, g);
            }
        }
        _ => {}
    }
}

fn wars(w: &World, x: GuildId) -> bool {
    gc::data(w, x).is_some_and(|d| !d.accepted_wars.is_empty() || !d.pending_wars.is_empty())
}

fn faction_differs(w: &World, g: GuildId, other: GuildId) -> bool {
    faction_of(w, leader_of(w, g)).map(|f| f.0) != faction_of(w, leader_of(w, other)).map(|f| f.0)
}

fn other_in_alliance_msg(
    w: &mut World,
    pm: MobileId,
    other: GuildId,
    other_alliance: gc::AllianceKey,
) {
    let on = gname(w, other);
    if gc::alliance_is_pending_member(w, other_alliance, other) {
        msg_a(w, pm, 1063416, &on);
    } else {
        msg_a(w, pm, 1063426, &on);
    }
}

/// Case 2: Request Alliance.
fn request_alliance(
    w: &mut World,
    pm: MobileId,
    g: GuildId,
    other: GuildId,
    alliance: Option<gc::AllianceKey>,
    other_alliance: Option<gc::AllianceKey>,
) {
    let (gn, on) = (gname(w, g), gname(w, other));
    match alliance {
        None => {
            if !can(w, pm, RankFlags::ALLIANCE_CONTROL) {
                msg(w, pm, 1070747);
            } else if faction_differs(w, g, other) {
                msg(w, pm, 1070758);
            } else if let Some(ok) = other_alliance {
                other_in_alliance_msg(w, pm, other, ok);
            } else if wars(w, other) {
                msg_a(w, pm, 1063427, &on);
            } else if wars(w, g) {
                msg_a(w, pm, 1063427, &gn);
            } else {
                msg(w, pm, 1063439); // Enter a name for the new alliance:
                prompt(w, pm, false, move |w, from, text| {
                    create_alliance(w, from, g, other, text)
                });
            }
        }
        Some(k) => {
            if !can(w, pm, RankFlags::ALLIANCE_CONTROL) {
                msg(w, pm, 1063436);
            } else if gc::alliance_leader(w, k) != Some(g) {
                msg_a(
                    w,
                    pm,
                    1063239,
                    &format!("{gn}\t{}", gc::alliance_name(w, k)),
                );
            } else if let Some(ok) = other_alliance {
                other_in_alliance_msg(w, pm, other, ok);
            } else if gc::alliance_is_pending_member(w, k, g) {
                msg_a(w, pm, 1063416, &gn);
            } else if wars(w, other) {
                msg_a(w, pm, 1063427, &on);
            } else if wars(w, g) {
                msg_a(w, pm, 1063427, &gn);
            } else if faction_differs(w, g, other) {
                msg(w, pm, 1070758);
            } else {
                msg_a(w, pm, 1070750, &on); // An invitation to join your alliance has been sent to ~1_val~.
                gc::guild_message_args(w, other, 1070780, &gn, 0x3B2); // ~1_val~ has proposed an alliance.
                gc::set_alliance(w, other, Some(k)); // Calls addPendingGuild
            }
        }
    }
}

fn create_alliance(w: &mut World, from: MobileId, g: GuildId, other: GuildId, text: &str) {
    if !muo_content::mx::player(w, from) {
        return;
    }
    let other_alliance = gc::alliance(w, other);
    if !is_member(w, from, g) || gc::alliance(w, g).is_some() {
        return;
    }
    let (gn, on) = (gname(w, g), gname(w, other));
    if !can(w, from, RankFlags::ALLIANCE_CONTROL) {
        msg(w, from, 1070747);
    } else if faction_differs(w, g, other) {
        msg(w, from, 1070758);
    } else if let Some(ok) = other_alliance {
        other_in_alliance_msg(w, from, other, ok);
    } else if wars(w, other) {
        msg_a(w, from, 1063427, &on);
    } else if wars(w, g) {
        msg_a(w, from, 1063427, &gn);
    } else {
        let name = utility::fix_html(text.trim());
        if !check_profanity(&name, 50) {
            msg(w, from, 1070886);
        } else if name.chars().count() > gc::NAME_LIMIT {
            msg_a(w, from, 1070887, &gc::NAME_LIMIT.to_string());
        } else if gc::alliance_name_taken(w, &name.lower_inv()) {
            msg(w, from, 1063428);
        } else {
            msg_a(w, from, 1070750, &on);
            gc::guild_message_args(w, other, 1070780, &gn, 0x3B2);
            gc::create_alliance(w, g, &name, other);
        }
    }
}

// ---- WarDeclarationGump ----

pub struct WarDeclarationGump {
    guild: GuildId,
    other: GuildId,
    hours: i64,
    max_kills: i32,
}

/// `new WarDeclarationGump(pm, g, otherGuild)`.
pub fn war_declaration(
    w: &World,
    _pm: MobileId,
    g: GuildId,
    other: GuildId,
) -> DynamicGumpObj<WarDeclarationGump> {
    let war = gc::find_pending_war(w, g, Some(other))
        .and_then(|i| gc::data(w, g).map(|d| d.pending_wars[i].clone()));
    DynamicGumpObj::new(
        10,
        10,
        WarDeclarationGump {
            guild: g,
            other,
            hours: war.as_ref().map_or(0, |x| hours_component(x.war_length)),
            max_kills: war.as_ref().map_or(0, |x| x.max_kills),
        },
    )
}

impl DynamicGumpLogic for WarDeclarationGump {
    const TYPE_NAME: &'static str = "Server.Guilds.WarDeclarationGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        start(b, false);
        b.add_background(0, 0, 500, 340, 0x24AE);
        b.add_background(65, 50, 370, 30, 0x2486);
        hl(b, 75, 55, 370, 26, 1062979, 0x3C00); // <div align=center><i>Declaration of War</i></div>
        b.add_image(410, 45, 0x232C, 0);
        hl(b, 65, 95, 200, 20, 1063009, 0x14AF); // <i>Duration of War</i>
        hl(b, 65, 120, 400, 20, 1063010, 0x0); // Enter the number of hours the war will last.
        b.add_background(65, 150, 40, 30, 0x2486);
        b.add_text_entry(70, 154, 50, 30, 0x481, 10, &self.hours.to_string());
        hl(b, 65, 195, 200, 20, 1063011, 0x14AF); // <i>Victory Condition</i>
        hl(b, 65, 220, 400, 20, 1063012, 0x0); // Enter the winning number of kills.
        b.add_background(65, 250, 40, 30, 0x2486);
        b.add_text_entry(70, 254, 50, 30, 0x481, 11, &self.max_kills.to_string());
        b.add_background(190, 270, 130, 26, 0x2486);
        b.add_button(195, 275, 0x845, 0x846, 0);
        hl(b, 220, 273, 90, 26, 1006045, 0x0); // Cancel
        b.add_background(330, 270, 130, 26, 0x2486);
        b.add_button(335, 275, 0x845, 0x846, 1);
        hl(b, 360, 273, 90, 26, 1062989, 0x5000); // Declare War!
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let (g, other) = (self.guild, self.other);
        let Some(pm) = member_of(w, sender, g) else {
            return;
        };
        if info.button_id != 1 {
            let gump = other_guild_info(w, pm, g, other);
            send(w, pm, gump);
            return;
        }
        let alliance = gc::alliance(w, g);
        let other_alliance = gc::alliance(w, other);
        let (gn, on) = (gname(w, g), gname(w, other));
        if !can(w, pm, RankFlags::CONTROL_WAR_STATUS) {
            msg(w, pm, 1063440);
        } else if let Some(k) = alliance.filter(|&k| gc::alliance_leader(w, k) != Some(g)) {
            msg_a(
                w,
                pm,
                1063239,
                &format!("{gn}\t{}", gc::alliance_name(w, k)),
            );
            if let Some(l) = gc::alliance_leader(w, k) {
                let ln = gname(w, l);
                msg_a(w, pm, 1070707, &ln);
            }
        } else if let Some(k) = other_alliance.filter(|&k| gc::alliance_leader(w, k) != Some(other))
        {
            msg_a(
                w,
                pm,
                1063239,
                &format!("{on}\t{}", gc::alliance_name(w, k)),
            );
            if let Some(l) = gc::alliance_leader(w, k) {
                let ln = gname(w, l);
                msg_a(w, pm, 1070707, &ln);
            }
        } else if gc::find_active_war(w, g, Some(other)).is_none() {
            let war = gc::find_pending_war(w, g, Some(other));
            let other_war = gc::find_pending_war(w, other, Some(g));
            let max_kills = info
                .get_text_entry(11)
                .map_or(0, |t| muo_primitives::parse::to_i32(t).clamp(0, 0xFFFF));
            let hours = info
                .get_text_entry(10)
                .map_or(0, |t| muo_primitives::parse::to_i32(t).clamp(0, 0xFFFF));
            let war_length = TimeSpan::from_ticks(i64::from(hours) * 36_000_000_000);
            if let Some(i) = war {
                if let Some(d) = gc::data_mut(w, g) {
                    let x = &mut d.pending_wars[i];
                    x.max_kills = max_kills;
                    x.war_length = war_length;
                    x.war_requester = true;
                }
            } else if let Some(d) = gc::data_mut(w, g) {
                d.pending_wars.push(gc::WarDeclaration::new(
                    g, other, max_kills, war_length, true,
                ));
            }
            if let Some(i) = other_war {
                if let Some(d) = gc::data_mut(w, other) {
                    let x = &mut d.pending_wars[i];
                    x.max_kills = max_kills;
                    x.war_length = war_length;
                    x.war_requester = false;
                }
            } else if let Some(d) = gc::data_mut(w, other) {
                d.pending_wars.push(gc::WarDeclaration::new(
                    other, g, max_kills, war_length, false,
                ));
            }
            if war.is_some() {
                msg(w, pm, 1070752); // The proposal has been updated.
            } else {
                let nm = alliance.map(|k| gc::alliance_name(w, k)).unwrap_or(gn);
                gc::guild_message_args(w, other, 1070781, &nm, 0x3B2); // ~1_val~ has proposed a war.
            }
            let nm2 = other_alliance
                .map(|k| gc::alliance_name(w, k))
                .unwrap_or(on);
            msg_a(w, pm, 1070751, &nm2); // War proposal has been sent to ~1_val~.
        }
    }
}

// ---- CreateGuildGump ----

pub struct CreateGuildGump {
    guild_name: String,
    guild_abbrev: String,
    accepts_invites: bool,
}

/// `new CreateGuildGump(pm, guildName, guildAbbrev)`: closes the player's `BaseGuildGump`s first.
pub fn create_guild_gump(
    w: &mut World,
    pm: MobileId,
    guild_name: &str,
    guild_abbrev: &str,
) -> DynamicGumpObj<CreateGuildGump> {
    if let Some(ns) = ns_of(w, pm) {
        close_base_guild_gumps(w, ns);
    }
    DynamicGumpObj::new(
        10,
        10,
        CreateGuildGump {
            guild_name: guild_name.to_owned(),
            guild_abbrev: guild_abbrev.to_owned(),
            accepts_invites: pmob::data(w, pm)
                .is_some_and(|d| d.flag(PlayerFlag::ACCEPT_GUILD_INVITES)),
        },
    )
}

/// `new CreateGuildGump(pm)`.
pub fn create_guild_default(w: &mut World, pm: MobileId) -> DynamicGumpObj<CreateGuildGump> {
    create_guild_gump(w, pm, "Guild Name", "")
}

impl DynamicGumpLogic for CreateGuildGump {
    const TYPE_NAME: &'static str = "Server.Guilds.CreateGuildGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.add_page(0);
        b.add_background(0, 0, 500, 300, 0x2422);
        hlb(b, 25, 20, 450, 25, 1062939, 0x0); // <center>GUILD MENU</center>
        // As you are not a member of any guild, you can create your own by providing a unique guild name and
        // paying the standard guild registration fee.
        hl(b, 25, 60, 450, 60, 1062940, 0x0);
        hl(b, 25, 135, 120, 25, 1062941, 0x0); // Registration Fee:
        b.add_label(155, 135, 0x481, &gc::REGISTRATION_FEE.to_string());
        hl(b, 25, 165, 120, 25, 1011140, 0x0); // Enter Guild Name:
        b.add_background(155, 160, 320, 26, 0xBB8);
        b.add_text_entry(160, 163, 315, 21, 0x481, 5, &self.guild_name);
        hl(b, 25, 191, 120, 26, 1063035, 0x0); // Abbreviation:
        b.add_background(155, 186, 320, 26, 0xBB8);
        b.add_text_entry(160, 189, 315, 21, 0x481, 6, &self.guild_abbrev);
        b.add_button(415, 217, 0xF7, 0xF8, 1);
        b.add_button(345, 217, 0xF2, 0xF1, 0);
        if self.accepts_invites {
            b.add_button(20, 260, 0xD2, 0xD3, 2);
        } else {
            b.add_button(20, 260, 0xD3, 0xD2, 2);
        }
        hl(b, 45, 260, 200, 30, 1062943, 0x0); // <i>Ignore Guild Invites</i>
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(pm) = mobile_of(w, sender) else {
            return;
        };
        if !muo_content::mx::player(w, pm) || w.mobile(pm).is_some_and(|m| m.guild.is_some()) {
            return; // Sanity
        }
        match info.button_id {
            1 => {
                let name = utility::fix_html(info.get_text_entry(5).unwrap_or(""));
                let abbrev = utility::fix_html(info.get_text_entry(6).unwrap_or(""));
                let ids = guild::guild_ids(w);
                let taken_abbrev =
                    guild::find_by_abbrev(ids.iter().filter_map(|&g| guild::guild(w, g)), &abbrev)
                        .is_some();
                let taken_name =
                    guild::find_by_name(ids.iter().filter_map(|&g| guild::guild(w, g)), &name)
                        .is_some();
                if name.is_empty() {
                    msg(w, pm, 1070884); // Guild name cannot be blank.
                } else if abbrev.is_empty() {
                    msg(w, pm, 1070885); // You must provide a guild abbreviation.
                } else if name.chars().count() > gc::NAME_LIMIT {
                    msg_a(w, pm, 1063036, &gc::NAME_LIMIT.to_string());
                } else if abbrev.chars().count() > gc::ABBREV_LIMIT {
                    msg_a(w, pm, 1063037, &gc::ABBREV_LIMIT.to_string());
                } else if taken_abbrev || !check_profanity(&abbrev, 50) {
                    msg(w, pm, 501153); // That abbreviation is not available.
                } else if taken_name || !check_profanity(&name, 50) {
                    msg(w, pm, 1063000); // That guild name is not available.
                } else if !muo_content::vendor::banker::withdraw(w, pm, gc::REGISTRATION_FEE) {
                    msg_a(w, pm, 1063001, &gc::REGISTRATION_FEE.to_string());
                } else {
                    msg_a(w, pm, 1060398, &gc::REGISTRATION_FEE.to_string());
                    msg(w, pm, 1063238); // Your new guild has been founded.
                    gc::new_guild(w, pm, &name, &abbrev);
                }
            }
            2 => {
                let now =
                    pmob::data(w, pm).is_some_and(|d| d.flag(PlayerFlag::ACCEPT_GUILD_INVITES));
                if let Some(d) = pmob::data_mut(w, pm) {
                    d.set_flag(PlayerFlag::ACCEPT_GUILD_INVITES, !now);
                }
                if !now {
                    msg(w, pm, 1070699); // You are now accepting guild invitations.
                } else {
                    msg(w, pm, 1070698);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod key_tests {
    use super::*;

    /// `TitleComparer`: `string.Compare` puts the null title before the empty one, and ignores case.
    #[test]
    fn a_null_guild_title_sorts_before_an_empty_one() {
        assert!(Key::Null < skey(""));
        assert!(skey("") < skey("a"));
        assert_eq!(skey("Abc"), skey("aBC"));
    }
}
