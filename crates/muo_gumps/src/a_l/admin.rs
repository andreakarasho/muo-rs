//! `AdminGump`: the `[Admin` interface (information, administer, clients, accounts, firewall).
//!
//! `m_List` (a `List<object>`) is [`ListItem`] and `m_State` (an `object`) is [`AdminState`]. Each gump owns its
//! copy of those lists and passes it on to the callbacks it hands out; sharing is not needed because the old gump
//! is closed (singleton) the moment the next one is sent.
//!
//! The process memory is the working set, the network stats come from
//! `AlHooks::network_stats` and `BanChannel.Retract` goes through `AlHooks::ban_retract` (both set in
//! `hooks::configure`).

use muo_primitives::dotnet::InvariantCase;
use std::cell::Cell;
use std::cmp::Ordering;
use std::net::IpAddr;
use std::rc::Rc;

use indexmap::IndexMap;
use muo_content::accounting::{self, AccountComment};
use muo_content::gumps::GumpButtonType;
use muo_content::gumps::base::{LegacyGumpLogic, LegacyGumpObj, RelayInfo};
use muo_content::gumps::legacy::LegacyLayout;
use muo_content::network::bans::BanReportQueue;
use muo_content::network::bans::firewall::{self, Firewall, FirewallEntry};
use muo_io::text::TextDefinition;
use muo_net::bans::ban_reasons;
use muo_net::network_stats::NetworkStats;
use muo_primitives::{MessageType, TimeSpan, html};
use muo_server::World;
use muo_server::commands::{CommandEventArgs, CommandSystem};
use muo_server::iaccount::AccountId;
use muo_server::ids::{EntityId, MobileId};
use muo_server::mobile::AccessLevel;
use muo_server::net_state::NetStateId;
use muo_server::prompt::{Prompt, add_prompt};

use super::common::{mobile_of, mobile_string, name_of, send};
use super::hooks::hooks;
use crate::m_z::warning;

const LABEL_COLOR: i32 = 0x7FFF;
const SELECTED_COLOR: i32 = 0x421F;
const LABEL_COLOR32: i32 = 0xFFFFFF;
const SELECTED_COLOR32: i32 = 0x8080FF;

const LABEL_HUE: i32 = 0x480;
const GREEN_HUE: i32 = 0x40;
const RED_HUE: i32 = 0x20;

const TYPE_NAME: &str = "Server.Gumps.AdminGump";

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum AdminGumpPage {
    InformationGeneral,
    InformationPerf,
    Administer,
    Clients,
    Accounts,
    AccountsShared,
    Firewall,
    AdministerWorldBuilding,
    AdministerServer,
    AdministerAccess,
    AdministerAccessLockdown,
    AdministerCommands,
    ClientInfo,
    AccountDetails,
    AccountDetailsInformation,
    AccountDetailsCharacters,
    AccountDetailsAccess,
    AccountDetailsAccessClientIps,
    AccountDetailsComments,
    AccountDetailsTags,
    AccountDetailsChangePassword,
    AccountDetailsChangeAccess,
    FirewallInfo,
}

use AdminGumpPage as P;

/// One entry of `m_List`.
#[derive(Clone, Debug)]
pub enum ListItem {
    Net(NetStateId),
    Account(AccountId),
    /// `KeyValuePair<IPAddress, List<Account>>`.
    Shared(IpAddr, Vec<AccountId>),
    Ip(IpAddr),
    Firewall(FirewallEntry),
}

/// `m_State`.
#[derive(Clone, Debug)]
pub enum AdminState {
    None,
    Mobile(MobileId),
    Account(AccountId),
    /// A `List<Account>` (the marked accounts).
    Accounts(Vec<AccountId>),
    /// A `List<object>`.
    Objects(Vec<ListItem>),
    Firewall(FirewallEntry),
}

pub struct AdminGump {
    from: MobileId,
    list: Option<Vec<ListItem>>,
    list_page: i32,
    page_type: P,
    state: AdminState,
}

const ACCESS_LEVEL_STRINGS: [&str; 7] = [
    "Player",
    "Counselor",
    "Game Master",
    "Seer",
    "Administrator",
    "Developer",
    "Owner",
];

/// `FormatAccessLevel`.
pub fn format_access_level(level: AccessLevel) -> &'static str {
    ACCESS_LEVEL_STRINGS
        .get(level as usize)
        .copied()
        .unwrap_or("Unknown")
}

/// `GetButtonID`.
pub fn get_button_id(kind: i32, index: i32) -> i32 {
    1 + index * 11 + kind
}

/// `FormatTimeSpan` of a span in ticks.
pub fn format_time_span(ts: TimeSpan) -> String {
    let t = ts.ticks();
    let secs = t / 10_000_000;
    format!(
        "{:02}:{:02}:{:02}:{:02}",
        secs / 86400,
        secs / 3600 % 24,
        secs / 60 % 60,
        secs % 60
    )
}

/// `FormatByteAmount`.
pub fn format_byte_amount(total: i64) -> String {
    if total > 1_000_000_000 {
        format!("{:.1} GB", total as f64 / 1_073_741_824.0)
    } else if total > 1_000_000 {
        format!("{:.1} MB", total as f64 / 1_048_576.0)
    } else if total > 1000 {
        format!("{:.1} KB", total as f64 / 1024.0)
    } else {
        format!("{total} Bytes")
    }
}

fn format_send_buffers(net: &NetworkStats, now_tick: i64) -> String {
    use format_byte_amount as f;
    let mut sb = format!(
        "Base: {} recv, {} send",
        f(net.recv_buffer_size.into()),
        f(net.send_buffer_size.into())
    );
    if net.initial_recv_buffer_size > 0 {
        sb += &format!(
            "; pre-auth {} / {}",
            f(net.initial_recv_buffer_size.into()),
            f(net.initial_send_buffer_size.into())
        );
    }
    let sweep = net.last_sweep;
    if sweep.ran {
        sb += &format!("<br>Base pools: {}", f(sweep.base_capacity_bytes));
    }
    let mut tier_bytes = 0i64;
    sb += "<br><br>Growth tiers (live):";
    for tier in &net.tiers {
        tier_bytes += i64::from(tier.capacity) * i64::from(tier.buffer_size);
        sb += &format!(
            "<br>  {}: {} / {} in use, floor {}",
            f(tier.buffer_size.into()),
            tier.in_use,
            tier.capacity,
            tier.retain_floor
        );
    }
    sb += &format!(
        "<br>Budget: {} of {}, max {} per socket",
        f(tier_bytes),
        f(net.send_buffer_growth_budget),
        f(net.max_send_buffer_size.into())
    );
    sb += &format!(
        "<br>Ceiling: {}% of {} available",
        net.memory_ceiling_percent,
        f(net.available_memory_bytes)
    );
    if sweep.ran {
        let age = (now_tick - sweep.tick) / 1000;
        sb += &format!(
            "<br><br>Last sweep {age}s ago: released {} tier, {} base",
            sweep.tier_buffers_released, sweep.base_buffers_released
        );
        sb += &format!(
            "<br>Refused: budget {}, at max {}, ceiling {}",
            sweep.budget_refusals, sweep.cap_refusals, sweep.ceiling_refusals
        );
    } else {
        sb += "<br><br>First sweep pending";
    }
    sb
}

/// `GetHueFor`.
pub fn get_hue_for(w: &World, m: Option<MobileId>) -> i32 {
    let Some(m) = m else { return LABEL_HUE };
    match muo_content::mx::access_level(w, m) {
        AccessLevel::Owner | AccessLevel::Developer | AccessLevel::Administrator => 0x516,
        AccessLevel::Seer => 0x144,
        AccessLevel::GameMaster => 0x21,
        AccessLevel::Counselor => 0x2,
        AccessLevel::Player => {
            if muo_server::mobile::base::murderer(w, m) {
                0x21
            } else if muo_server::mobile::base::criminal(w, m) {
                0x3B1
            } else {
                0x58
            }
        }
    }
}

/// `GetAccountInfo`: the highest access level on the account and whether any character is online.
pub fn get_account_info(w: &mut World, a: AccountId) -> (AccessLevel, bool) {
    let mut level = accounting::account(w, a).map_or(AccessLevel::Player, |x| x.access_level);
    let mut online = false;
    let len = accounting::account(w, a).map_or(0, |x| x.length());
    for j in 0..len {
        let Some(check) = accounting::mobile_at(w, a, j) else {
            continue;
        };
        let l = muo_content::mx::access_level(w, check);
        if l > level {
            level = l;
        }
        if w.mobile(check).is_some_and(|m| m.net_state.is_some()) {
            online = true;
        }
    }
    (level, online)
}

fn username(w: &World, a: AccountId) -> String {
    accounting::account(w, a).map_or_else(String::new, |x| x.username.clone())
}

fn banned(w: &mut World, a: AccountId) -> bool {
    accounting::is_banned(w, a)
}

/// `AccountComparer`: online first, higher access first, then by name.
fn sort_accounts(w: &mut World, list: &mut Vec<AccountId>) {
    let mut keyed: Vec<(bool, AccessLevel, String, AccountId)> = list
        .iter()
        .map(|&a| {
            let (l, o) = get_account_info(w, a);
            (o, l, username(w, a), a)
        })
        .collect();
    muo_primitives::dotnet::net_sort(&mut keyed, |x, y| {
        y.0.cmp(&x.0)
            .then_with(|| y.1.cmp(&x.1))
            .then_with(|| muo_primitives::dotnet::cmp_ignore_case(&x.2, &y.2)) // InsensitiveCompare: OrdinalIgnoreCase
    });
    *list = keyed.into_iter().map(|k| k.3).collect();
}

/// The username of the account a connection logged in with.
fn ns_account(w: &World, ns: NetStateId) -> Option<AccountId> {
    let name = w.net_state(ns)?.account.clone()?;
    accounting::get_account(w, &name)
}

/// `NetStateComparer`.
fn sort_net_states(w: &World, list: &mut [NetStateId]) {
    muo_primitives::dotnet::net_sort(list, |&x, &y| {
        let (a, b) = (w.net_mobile(x), w.net_mobile(y));
        match (a, b) {
            (None, None) => Ordering::Equal,
            (None, _) => Ordering::Greater,
            (_, None) => Ordering::Less,
            (Some(a), Some(b)) => {
                let (la, lb) = (
                    muo_content::mx::access_level(w, a),
                    muo_content::mx::access_level(w, b),
                );
                lb.cmp(&la).then_with(|| {
                    muo_primitives::dotnet::cmp_ignore_case(&name_of(w, a), &name_of(w, b))
                })
            }
        }
    });
}

/// `FilterAccess`.
fn filter_access(w: &World, list: &mut Vec<ListItem>, from: MobileId) {
    let level = muo_content::mx::access_level(w, from);
    list.retain(|item| match item {
        ListItem::Account(a) => accounting::account(w, *a).is_none_or(|x| x.access_level <= level),
        ListItem::Net(ns) => {
            let m_ok = w
                .net_mobile(*ns)
                .is_none_or(|m| muo_content::mx::access_level(w, m) <= level);
            let a_ok = ns_account(w, *ns)
                .and_then(|a| accounting::account(w, a))
                .is_none_or(|a| a.access_level <= level);
            m_ok && a_ok
        }
        _ => true,
    });
}

fn all_accounts(w: &mut World) -> Vec<AccountId> {
    accounting::accounts(w).ids()
}

fn login_ips(w: &World, a: AccountId) -> Vec<IpAddr> {
    accounting::account(w, a).map_or_else(Vec::new, |x| x.login_ips.clone())
}

/// `GetAllSharedAccounts`.
fn get_all_shared_accounts(w: &mut World) -> Vec<ListItem> {
    let mut table: IndexMap<IpAddr, Vec<AccountId>> = IndexMap::new();
    for a in all_accounts(w) {
        for ip in login_ips(w, a) {
            table.entry(ip).or_default().push(a);
        }
    }
    let mut list = Vec::new();
    for (ip, mut accts) in table {
        if accts.len() > 1 {
            sort_accounts(w, &mut accts);
            list.push(ListItem::Shared(ip, accts));
        }
    }
    let count = |i: &ListItem| match i {
        ListItem::Shared(_, v) => v.len(),
        _ => 0,
    };
    muo_primitives::dotnet::net_sort(&mut list, |a, b| count(b).cmp(&count(a))); // SharedAccountDescendingComparer
    list
}

/// `GetSharedAccounts(ips)`.
fn get_shared_accounts(w: &mut World, ips: &[IpAddr]) -> Vec<AccountId> {
    let mut list = Vec::new();
    for a in all_accounts(w) {
        if login_ips(w, a).iter().any(|i| ips.contains(i)) {
            list.push(a);
        }
    }
    sort_accounts(w, &mut list);
    list
}

// ---- layout helpers ----

fn add_black_alpha(l: &mut LegacyLayout, x: i32, y: i32, width: i32, height: i32) {
    l.add_image_tiled(x, y, width, height, 2624);
    l.add_alpha_region(x, y, width, height);
}

fn add_selected_button(l: &mut LegacyLayout, x: i32, y: i32, id: i32, text: &str, sel: bool) {
    l.add_button(x, y - 1, if sel { 4006 } else { 4005 }, 4007, id);
    l.add_html(
        x + 35,
        y,
        200,
        20,
        &html::color_rgb(text, if sel { SELECTED_COLOR32 } else { LABEL_COLOR32 }),
        false,
        false,
    );
}

fn add_button_labeled(l: &mut LegacyLayout, x: i32, y: i32, id: i32, text: &str) {
    l.add_button(x, y - 1, 4005, 4007, id);
    l.add_html(
        x + 35,
        y,
        240,
        20,
        &html::color_rgb(text, LABEL_COLOR32),
        false,
        false,
    );
}

fn add_text_field(l: &mut LegacyLayout, x: i32, y: i32, width: i32, height: i32, index: i32) {
    l.add_background(x - 2, y - 2, width + 4, height + 4, 0x2486);
    l.add_text_entry(x + 2, y + 2, width - 4, height - 4, 0, index, "");
}

fn centered(l: &mut LegacyLayout, y: i32, text: &str) {
    l.add_html(
        10,
        y,
        400,
        20,
        &html::center_rgb(text, LABEL_COLOR32),
        false,
        false,
    );
}

fn add_client_header(l: &mut LegacyLayout) {
    add_text_field(l, 200, 20, 200, 20, 0);
    add_button_labeled(l, 200, 50, get_button_id(4, 0), "Search For Name");
    add_button_labeled(l, 200, 80, get_button_id(4, 1), "Search For IP Address");
}

fn add_account_header(l: &mut LegacyLayout) {
    l.add_page(1);
    l.add_label(200, 20, LABEL_HUE, "Name:");
    add_text_field(l, 250, 20, 150, 20, 0);
    l.add_label(200, 50, LABEL_HUE, "Pass:");
    add_text_field(l, 250, 50, 150, 20, 1);
    add_button_labeled(l, 200, 80, get_button_id(5, 6), "Add");
    add_button_labeled(l, 290, 80, get_button_id(5, 7), "Search");
    l.add_button_type(384, 84, 0x15E1, 0x15E5, 0, GumpButtonType::Page, 2);
    l.add_page(2);
    add_button_labeled(l, 200, 20, get_button_id(5, 31), "View All: Inactive");
    add_button_labeled(l, 200, 40, get_button_id(5, 32), "View All: Banned");
    add_button_labeled(l, 200, 60, get_button_id(5, 26), "View All: Shared");
    add_button_labeled(l, 200, 80, get_button_id(5, 30), "View All: Empty");
    l.add_button_type(384, 84, 0x15E1, 0x15E5, 0, GumpButtonType::Page, 1);
    l.add_page(0);
}

fn add_firewall_header(l: &mut LegacyLayout) {
    add_text_field(l, 200, 20, 200, 20, 0);
    add_button_labeled(l, 320, 50, get_button_id(6, 0), "Search");
    add_button_labeled(l, 200, 50, get_button_id(6, 1), "Add (Input)");
    add_button_labeled(l, 200, 80, get_button_id(6, 2), "Add (Target)");
}

/// The previous and next page buttons of a list (`y` is where they sit, `per_page` how many rows a page has).
fn add_pager(
    l: &mut LegacyLayout,
    x_prev: i32,
    x_next: i32,
    y: i32,
    list_page: i32,
    per_page: i32,
    count: usize,
) {
    if list_page > 0 {
        l.add_button(x_prev, y, 0x15E3, 0x15E7, get_button_id(1, 0));
    } else {
        l.add_image(x_prev, y, 0x25EA, 0);
    }
    if (list_page + 1) * per_page < count as i32 {
        l.add_button(x_next, y, 0x15E1, 0x15E5, get_button_id(1, 1));
    } else {
        l.add_image(x_next, y, 0x25E6, 0);
    }
}

fn status_label(w: &mut World, l: &mut LegacyLayout, x: i32, y: i32, a: AccountId, online: bool) {
    if online {
        l.add_label_cropped(x, y, 120, 20, GREEN_HUE, "Online");
    } else if banned(w, a) {
        l.add_label_cropped(x, y, 120, 20, RED_HUE, "Banned");
    } else {
        l.add_label_cropped(x, y, 120, 20, RED_HUE, "Offline");
    }
}

struct Ctx {
    page_type: P,
    from: MobileId,
    from_level: AccessLevel,
}

impl Ctx {
    #[allow(clippy::too_many_arguments)]
    fn page_button(
        &self,
        l: &mut LegacyLayout,
        x: i32,
        y: i32,
        id: i32,
        text: &str,
        page: P,
        subs: &[P],
    ) {
        let sel = self.page_type == page || subs.contains(&self.page_type);
        add_selected_button(l, x, y, id, text, sel);
    }

    fn administer(&self, l: &mut LegacyLayout) {
        self.page_button(
            l,
            200,
            20,
            get_button_id(3, 0),
            "World Building",
            P::AdministerWorldBuilding,
            &[],
        );
        self.page_button(
            l,
            200,
            40,
            get_button_id(3, 1),
            "Server",
            P::AdministerServer,
            &[],
        );
        self.page_button(
            l,
            200,
            60,
            get_button_id(3, 2),
            "Access",
            P::AdministerAccess,
            &[P::AdministerAccessLockdown],
        );
        self.page_button(
            l,
            200,
            80,
            get_button_id(3, 3),
            "Commands",
            P::AdministerCommands,
            &[],
        );
    }

    fn account_details(&self, l: &mut LegacyLayout) {
        self.page_button(
            l,
            190,
            10,
            get_button_id(5, 0),
            "Information",
            P::AccountDetailsInformation,
            &[
                P::AccountDetailsChangeAccess,
                P::AccountDetailsChangePassword,
            ],
        );
        self.page_button(
            l,
            190,
            30,
            get_button_id(5, 1),
            "Characters",
            P::AccountDetailsCharacters,
            &[],
        );
        self.page_button(
            l,
            190,
            50,
            get_button_id(5, 13),
            "Access",
            P::AccountDetailsAccess,
            &[P::AccountDetailsAccessClientIps],
        );
        self.page_button(
            l,
            190,
            70,
            get_button_id(5, 2),
            "Comments",
            P::AccountDetailsComments,
            &[],
        );
        self.page_button(
            l,
            190,
            90,
            get_button_id(5, 3),
            "Tags",
            P::AccountDetailsTags,
            &[],
        );
    }
}

/// `new AdminGump(from, pageType, listPage, list, notice, state)`.
pub fn new(
    w: &mut World,
    from: MobileId,
    page_type: P,
    list_page: i32,
    list: Option<Vec<ListItem>>,
    notice: Option<&str>,
    state: AdminState,
) -> LegacyGumpObj<AdminGump> {
    let mut list = list;
    if let Some(v) = list.as_mut() {
        filter_access(w, v, from);
    }
    let from_level = muo_content::mx::access_level(w, from);
    let cx = Ctx {
        page_type,
        from,
        from_level,
    };
    let mut l = LegacyLayout::default();
    build(w, &cx, &mut l, list_page, &mut list, notice, &state);
    let mut g = LegacyGumpObj::new(
        50,
        40,
        AdminGump {
            from,
            list,
            list_page,
            page_type,
            state,
        },
    );
    g.layout = l;
    g
}

fn open(
    w: &mut World,
    from: MobileId,
    page: P,
    list_page: i32,
    list: Option<Vec<ListItem>>,
    notice: Option<&str>,
    state: AdminState,
) {
    let g = new(w, from, page, list_page, list, notice, state);
    send(w, from, g);
}

fn build(
    w: &mut World,
    cx: &Ctx,
    l: &mut LegacyLayout,
    list_page: i32,
    list: &mut Option<Vec<ListItem>>,
    notice: Option<&str>,
    state: &AdminState,
) {
    let from_level = cx.from_level;
    l.add_page(0);
    l.add_background(0, 0, 420, 480, 5054);
    add_black_alpha(l, 10, 10, 170, 100);
    add_black_alpha(l, 190, 10, 220, 100);
    add_black_alpha(l, 10, 120, 400, 300);
    add_black_alpha(l, 10, 430, 400, 40);

    cx.page_button(
        l,
        10,
        10,
        get_button_id(0, 0),
        "INFORMATION",
        P::InformationGeneral,
        &[P::InformationPerf],
    );
    cx.page_button(
        l,
        10,
        30,
        get_button_id(0, 1),
        "ADMINISTER",
        P::Administer,
        &[
            P::AdministerAccess,
            P::AdministerCommands,
            P::AdministerServer,
            P::AdministerWorldBuilding,
            P::AdministerAccessLockdown,
        ],
    );
    cx.page_button(
        l,
        10,
        50,
        get_button_id(0, 2),
        "CLIENT LIST",
        P::Clients,
        &[P::ClientInfo],
    );
    cx.page_button(
        l,
        10,
        70,
        get_button_id(0, 3),
        "ACCOUNT LIST",
        P::Accounts,
        &[
            P::AccountsShared,
            P::AccountDetails,
            P::AccountDetailsInformation,
            P::AccountDetailsCharacters,
            P::AccountDetailsAccess,
            P::AccountDetailsAccessClientIps,
            P::AccountDetailsComments,
            P::AccountDetailsTags,
            P::AccountDetailsChangeAccess,
            P::AccountDetailsChangePassword,
        ],
    );
    cx.page_button(
        l,
        10,
        90,
        get_button_id(0, 4),
        "FIREWALL",
        P::Firewall,
        &[P::FirewallInfo],
    );

    if let Some(n) = notice {
        l.add_html(
            20,
            440,
            396,
            36,
            &html::color_rgb(n, LABEL_COLOR32),
            false,
            false,
        );
    }

    match cx.page_type {
        P::InformationGeneral => {
            let (mut banned_n, mut active) = (0, 0);
            for a in all_accounts(w) {
                if banned(w, a) {
                    banned_n += 1;
                } else {
                    active += 1;
                }
            }
            let fw = w.resources.get::<Firewall>().map_or(0, Firewall::count);
            let clients = w.net_states.len();
            let rows: [(&str, String); 11] = [
                ("Active Accounts:", active.to_string()),
                ("Banned Accounts:", banned_n.to_string()),
                ("Firewalled:", fw.to_string()),
                ("Clients:", clients.to_string()),
                ("Mobiles:", w.mobiles.len().to_string()),
                ("Mobile Scripts:", w.core.script_mobiles.to_string()),
                ("Items:", w.items.len().to_string()),
                ("Item Scripts:", w.core.script_items.to_string()),
                (
                    "Uptime:",
                    format_time_span(TimeSpan::from_ticks(w.core.uptime(w.tick_count) * 10_000)),
                ),
                (
                    "Memory:",
                    format_byte_amount(muo_primitives::process_memory::process_memory_bytes()),
                ),
                ("Framework:", format!("Rust {}", env!("CARGO_PKG_VERSION"))),
            ];
            for (i, (k, v)) in rows.iter().enumerate() {
                let y = 130 + i as i32 * 20;
                l.add_label(20, y, LABEL_HUE, k);
                l.add_label(150, y, LABEL_HUE, v);
            }
            l.add_label(20, 350, LABEL_HUE, "Operating System: ");
            l.add_label(150, 350, LABEL_HUE, std::env::consts::OS);
            add_info_buttons(cx, l);
        }
        P::InformationPerf => {
            let loop_status = if w.core.idle_sleep_unsupported {
                "Spinning - host cannot honor short waits"
            } else if w.core.event_loop_idle_wait_ms == 0 {
                "Spinning (configured)"
            } else if w.core.idle_sleep_suspended(w.tick_count) {
                "Sleep suspended - host returning waits late"
            } else {
                "Healthy"
            };
            l.add_label(20, 130, LABEL_HUE, "Event Loop:");
            l.add_label(40, 150, LABEL_HUE, loop_status);
            let net = hooks(w)
                .and_then(|h| h.network_stats)
                .and_then(|f| f(w))
                .unwrap_or_default();
            l.add_label(20, 170, LABEL_HUE, "Connections:");
            l.add_label(
                150,
                170,
                LABEL_HUE,
                &format!(
                    "{} / {} ({} authenticated)",
                    net.connected, net.max_connections, net.authenticated
                ),
            );
            l.add_label(20, 190, LABEL_HUE, "Loop Queues:");
            l.add_label(
                150,
                190,
                LABEL_HUE,
                &format!(
                    "throttled {}, flush {}, closing {}",
                    net.throttled, net.flush_pending, net.pending_disconnects
                ),
            );
            l.add_label(20, 210, LABEL_HUE, "Send Buffers:");
            l.add_html(
                20,
                230,
                380,
                180,
                &format_send_buffers(&net, w.tick_count),
                true,
                true,
            );
            add_info_buttons(cx, l);
        }
        P::AdministerWorldBuilding => {
            centered(l, 125, "Generating");
            let rows: [(i32, &str, i32, &str); 7] = [
                (101, "Teleporters", 102, "Moongates"),
                (103, "Spawners", 106, "Decorations"),
                (104, "Doors", 105, "Signs"),
                (107, "Champions", 108, "Magincia (AOS+)"),
                (109, "Stealable Artifacts", 110, "SH Teleporters"),
                (111, "Secret Locations", 112, "Doom"),
                (113, "Khaldun Puzzles", 114, "Do everything"),
            ];
            for (i, (a, an, b, bn)) in rows.iter().enumerate() {
                let y = 175 + i as i32 * 25;
                add_button_labeled(l, 20, y, get_button_id(3, *a), an);
                add_button_labeled(l, 220, y, get_button_id(3, *b), bn);
            }
            cx.administer(l);
        }
        P::AdministerServer => {
            centered(l, 125, "Server");
            add_button_labeled(l, 20, 150, get_button_id(3, 200), "Save");
            add_button_labeled(l, 20, 180, get_button_id(3, 201), "Shutdown (With Save)");
            add_button_labeled(l, 20, 200, get_button_id(3, 202), "Shutdown (Without Save)");
            add_button_labeled(
                l,
                20,
                230,
                get_button_id(3, 203),
                "Shutdown & Restart (With Save)",
            );
            add_button_labeled(
                l,
                20,
                250,
                get_button_id(3, 204),
                "Shutdown & Restart (Without Save)",
            );
            add_button_labeled(
                l,
                20,
                270,
                get_button_id(3, 205),
                "Shutdown (With 15m Delay & Save)",
            );
            centered(l, 295, "Broadcast");
            add_text_field(l, 20, 320, 380, 20, 0);
            add_button_labeled(l, 20, 350, get_button_id(3, 210), "To Everyone");
            add_button_labeled(l, 220, 350, get_button_id(3, 211), "To Staff");
            cx.administer(l);
        }
        P::AdministerAccessLockdown => {
            centered(l, 125, "Server Lockdown");
            l.add_html(
                20,
                150,
                380,
                80,
                &html::color_rgb(
                    "When enabled, only clients with an access level equal to or greater than the specified lockdown level may access the server. After setting a lockdown level, use the <em>Purge Invalid Clients</em> button to disconnect those clients without access.",
                    LABEL_COLOR32,
                ),
                false,
                false,
            );
            let level = accounting::handler::lockdown_level(w);
            let locked = level > AccessLevel::Player;
            add_selected_button(
                l,
                20,
                230,
                get_button_id(3, 500),
                "Not Locked Down",
                !locked,
            );
            add_selected_button(
                l,
                20,
                260,
                get_button_id(3, 504),
                "Administrators",
                locked && level <= AccessLevel::Administrator,
            );
            add_selected_button(
                l,
                20,
                280,
                get_button_id(3, 503),
                "Seers",
                locked && level <= AccessLevel::Seer,
            );
            add_selected_button(
                l,
                20,
                300,
                get_button_id(3, 502),
                "Game Masters",
                locked && level <= AccessLevel::GameMaster,
            );
            add_selected_button(
                l,
                20,
                320,
                get_button_id(3, 501),
                "Counselors",
                locked && level <= AccessLevel::Counselor,
            );
            add_button_labeled(l, 20, 350, get_button_id(3, 510), "Purge Invalid Clients");
            cx.administer(l);
        }
        P::AdministerAccess => {
            centered(l, 125, "Access");
            centered(l, 155, "Connectivity");
            add_button_labeled(l, 20, 180, get_button_id(3, 300), "Kick");
            add_button_labeled(l, 220, 180, get_button_id(3, 301), "Ban");
            add_button_labeled(l, 20, 210, get_button_id(3, 302), "Firewall");
            add_button_labeled(l, 220, 210, get_button_id(3, 303), "Lockdown");
            centered(l, 245, "Staff");
            add_button_labeled(l, 20, 270, get_button_id(3, 310), "Make Player");
            add_button_labeled(l, 20, 290, get_button_id(3, 311), "Make Counselor");
            add_button_labeled(l, 20, 310, get_button_id(3, 312), "Make Game Master");
            add_button_labeled(l, 20, 330, get_button_id(3, 313), "Make Seer");
            if from_level > AccessLevel::Administrator {
                add_button_labeled(l, 220, 270, get_button_id(3, 314), "Make Administrator");
                if from_level > AccessLevel::Developer {
                    add_button_labeled(l, 220, 290, get_button_id(3, 315), "Make Developer");
                    if from_level >= AccessLevel::Owner {
                        add_button_labeled(l, 220, 310, get_button_id(3, 316), "Make Owner");
                    }
                }
            }
            cx.administer(l);
        }
        P::AdministerCommands => {
            centered(l, 125, "Commands");
            let rows: [(i32, &str, i32, &str, i32); 9] = [
                (400, "Add", 401, "Remove", 150),
                (402, "Dupe", 403, "Dupe in bag", 170),
                (404, "Properties", 405, "Skills", 200),
                (406, "Mortal", 407, "Immortal", 230),
                (408, "Squelch", 409, "Unsquelch", 250),
                (412, "Hide", 413, "Unhide", 290),
                (414, "Kill", 415, "Resurrect", 310),
                (416, "Move", 417, "Wipe", 330),
                (418, "Teleport", 419, "Teleport (Multiple)", 350),
            ];
            for (a, an, b, bn, y) in rows.iter() {
                add_button_labeled(l, 20, *y, get_button_id(3, *a), an);
                add_button_labeled(l, 220, *y, get_button_id(3, *b), bn);
            }
            cx.administer(l);
        }
        P::Administer => cx.administer(l),
        P::Clients => {
            if list.is_none() {
                let mut states: Vec<NetStateId> = w.net_states.keys().copied().collect();
                sort_net_states(w, &mut states);
                let mut v: Vec<ListItem> = states.into_iter().map(ListItem::Net).collect();
                filter_access(w, &mut v, cx.from);
                *list = Some(v);
            }
            let items = list.clone().unwrap_or_default();
            add_client_header(l);
            l.add_label_cropped(12, 120, 81, 20, LABEL_HUE, "Name");
            l.add_label_cropped(95, 120, 81, 20, LABEL_HUE, "Account");
            l.add_label_cropped(178, 120, 81, 20, LABEL_HUE, "Access Level");
            l.add_label_cropped(273, 120, 109, 20, LABEL_HUE, "IP Address");
            add_pager(l, 375, 392, 122, list_page, 12, items.len());
            if items.is_empty() {
                l.add_label(12, 140, LABEL_HUE, "There are no clients to display.");
            }
            let mut index = list_page * 12;
            let mut i = 0;
            while i < 12 && index >= 0 && (index as usize) < items.len() {
                if let ListItem::Net(ns) = items[index as usize] {
                    let m = w.net_mobile(ns);
                    let a = ns_account(w, ns);
                    let offset = 140 + i * 20;
                    match m {
                        None => l.add_label_cropped(12, offset, 81, 20, LABEL_HUE, "(logging in)"),
                        Some(m) => l.add_label_cropped(
                            12,
                            offset,
                            81,
                            20,
                            get_hue_for(w, Some(m)),
                            &name_of(w, m),
                        ),
                    }
                    let uname = a.map(|a| username(w, a));
                    l.add_label_cropped(
                        95,
                        offset,
                        81,
                        20,
                        LABEL_HUE,
                        uname.as_deref().unwrap_or("(no account)"),
                    );
                    let level_text = match (m, a) {
                        (None, Some(a)) => format_access_level(
                            accounting::account(w, a)
                                .map_or(AccessLevel::Player, |x| x.access_level),
                        ),
                        (None, None) => "",
                        (Some(m), _) => format_access_level(muo_content::mx::access_level(w, m)),
                    };
                    l.add_label_cropped(178, offset, 81, 20, LABEL_HUE, level_text);
                    let d = w
                        .net_state(ns)
                        .map(|d| d.description.clone())
                        .unwrap_or_default();
                    l.add_label_cropped(273, offset, 109, 20, LABEL_HUE, &d);
                    if a.is_some() || m.is_some() {
                        l.add_button(380, offset - 1, 0xFA5, 0xFA7, get_button_id(4, index + 2));
                    }
                }
                i += 1;
                index += 1;
            }
        }
        P::ClientInfo => {
            let AdminState::Mobile(m) = *state else {
                return;
            };
            add_client_header(l);
            centered(l, 125, "Information");
            let mut y = 146;
            l.add_label(20, y, LABEL_HUE, "Name:");
            l.add_label(200, y, get_hue_for(w, Some(m)), &name_of(w, m));
            y += 20;
            let a = w.mobile(m).and_then(|x| x.account);
            l.add_label(20, y, LABEL_HUE, "Account:");
            let a_banned = a.is_some_and(|a| banned(w, a));
            l.add_label(
                200,
                y,
                if a_banned { RED_HUE } else { LABEL_HUE },
                &a.map_or_else(|| "(no account)".to_owned(), |a| username(w, a)),
            );
            let m_level = muo_content::mx::access_level(w, m);
            let a_level = a
                .and_then(|a| accounting::account(w, a))
                .map(|x| x.access_level);
            if m_level > from_level || a_level.is_some_and(|x| x > from_level) {
                l.add_label(
                    20,
                    y + 20,
                    LABEL_HUE,
                    "You do not have permission to view this client's information.",
                );
                return;
            }
            l.add_button(380, y, 0xFA5, 0xFA7, get_button_id(7, 14));
            y += 20;
            let ns = w.mobile(m).and_then(|x| x.net_state);
            let loc = format!(
                "{} [{}]",
                muo_content::mx::location(w, m),
                muo_content::mx::map(w, m)
                    .map_or_else(String::new, |mm| super::common::map_name(w, mm))
            );
            match ns {
                None => {
                    l.add_label(20, y, LABEL_HUE, "Address:");
                    l.add_label(200, y, RED_HUE, "Offline");
                    y += 20;
                    l.add_label(20, y, LABEL_HUE, "Location:");
                    l.add_label(200, y, LABEL_HUE, &loc);
                    y += 44;
                }
                Some(ns) => {
                    l.add_label(20, y, LABEL_HUE, "Address:");
                    let d = w
                        .net_state(ns)
                        .map(|d| d.description.clone())
                        .unwrap_or_default();
                    l.add_label(200, y, GREEN_HUE, &d);
                    y += 20;
                    let v = w
                        .net_state(ns)
                        .and_then(|d| d.caps.version.as_ref())
                        .map_or_else(|| "(null)".to_owned(), |v| v.source_string().to_owned());
                    l.add_label(20, y, LABEL_HUE, "Version:");
                    let assistant = hooks(w)
                        .and_then(|h| h.client_assistant)
                        .and_then(|f| f(w, ns));
                    match assistant {
                        Some(asst) => l.add_label(200, y, LABEL_HUE, &format!("{v} ({asst})")),
                        None => l.add_label(200, y, LABEL_HUE, &v),
                    }
                    y += 20;
                    l.add_label(20, y, LABEL_HUE, "Location:");
                    l.add_label(200, y, LABEL_HUE, &loc);
                    y += 24;
                }
            }
            let pairs: [(&str, i32, &str, i32, i32); 7] = [
                ("Go to", 0, "Get", 1, 20),
                ("Kick", 2, "Ban", 3, 20),
                ("Properties", 4, "Skills", 5, 20),
                ("Mortal", 6, "Immortal", 7, 20),
                ("Squelch", 8, "Unsquelch", 9, 20),
                ("Kill", 12, "Resurrect", 13, 20),
                ("Jail", 15, "Unjail", 16, 25),
            ];
            for (an, ai, bn, bi, step) in pairs {
                add_button_labeled(l, 20, y, get_button_id(7, ai), an);
                add_button_labeled(l, 200, y, get_button_id(7, bi), bn);
                y += step;
            }
            l.add_label(20, y, LABEL_HUE, "Jail Reason:");
            add_text_field(l, 100, y, 300, 20, 1);
        }
        P::AccountsShared => {
            if list.is_none() {
                *list = Some(get_all_shared_accounts(w));
            }
            let items = list.clone().unwrap_or_default();
            l.add_label_cropped(12, 120, 60, 20, LABEL_HUE, "Count");
            l.add_label_cropped(72, 120, 120, 20, LABEL_HUE, "Address");
            l.add_label_cropped(192, 120, 180, 20, LABEL_HUE, "Accounts");
            add_pager(l, 375, 392, 122, list_page, 12, items.len());
            if items.is_empty() {
                l.add_label(12, 140, LABEL_HUE, "There are no accounts to display.");
            }
            let mut index = list_page * 12;
            let mut i = 0;
            while i < 12 && index >= 0 && (index as usize) < items.len() {
                if let ListItem::Shared(ip, accts) = &items[index as usize] {
                    let offset = 140 + i * 20;
                    l.add_label_cropped(12, offset, 60, 20, LABEL_HUE, &accts.len().to_string());
                    l.add_label_cropped(72, offset, 120, 20, LABEL_HUE, &ip.to_string());
                    let mut sb = String::new();
                    for (j, &acct) in accts.iter().enumerate() {
                        if j > 0 {
                            sb += ", ";
                        }
                        if j < 4 {
                            sb += &username(w, acct);
                        } else {
                            sb += "...";
                            break;
                        }
                    }
                    l.add_label_cropped(192, offset, 180, 20, LABEL_HUE, &sb);
                    l.add_button(380, offset - 1, 0xFA5, 0xFA7, get_button_id(5, index + 56));
                }
                i += 1;
                index += 1;
            }
        }
        P::Accounts => {
            let items = list.get_or_insert_with(Vec::new).clone();
            let rads = if let AdminState::Accounts(r) = state {
                Some(r.clone())
            } else {
                None
            };
            add_account_header(l);
            if rads.is_none() {
                l.add_label_cropped(12, 120, 120, 20, LABEL_HUE, "Name");
            } else {
                l.add_label_cropped(32, 120, 100, 20, LABEL_HUE, "Name");
            }
            l.add_label_cropped(132, 120, 120, 20, LABEL_HUE, "Access Level");
            l.add_label_cropped(252, 120, 120, 20, LABEL_HUE, "Status");
            add_pager(l, 375, 392, 122, list_page, 12, items.len());
            if items.is_empty() {
                l.add_label(12, 140, LABEL_HUE, "There are no accounts to display.");
            }
            if rads.is_some() && notice.is_none() {
                add_button_labeled(l, 10, 390, get_button_id(5, 27), "Ban marked");
                add_button_labeled(l, 10, 410, get_button_id(5, 28), "Delete marked");
                add_button_labeled(l, 210, 390, get_button_id(5, 29), "Mark all");
                add_button_labeled(l, 210, 410, get_button_id(5, 35), "Unmark house owners");
            }
            let mut index = list_page * 12;
            let mut i = 0;
            while i < 12 && index >= 0 && (index as usize) < items.len() {
                if let ListItem::Account(a) = items[index as usize] {
                    let offset = 140 + i * 20;
                    let (level, online) = get_account_info(w, a);
                    let uname = username(w, a);
                    match &rads {
                        None => l.add_label_cropped(12, offset, 120, 20, LABEL_HUE, &uname),
                        Some(r) => {
                            l.add_check(10, offset, 0xD2, 0xD3, r.contains(&a), index);
                            l.add_label_cropped(32, offset, 100, 20, LABEL_HUE, &uname);
                        }
                    }
                    l.add_label_cropped(
                        132,
                        offset,
                        120,
                        20,
                        LABEL_HUE,
                        format_access_level(level),
                    );
                    status_label(w, l, 252, offset, a, online);
                    l.add_button(380, offset - 1, 0xFA5, 0xFA7, get_button_id(5, index + 56));
                }
                i += 1;
                index += 1;
            }
        }
        P::AccountDetails => cx.account_details(l),
        P::AccountDetailsChangePassword => {
            let AdminState::Account(a) = *state else {
                return;
            };
            centered(l, 125, "Change Password");
            l.add_label(20, 150, LABEL_HUE, "Username:");
            l.add_label(200, 150, LABEL_HUE, &username(w, a));
            l.add_label(20, 180, LABEL_HUE, "Password:");
            add_text_field(l, 200, 180, 160, 20, 0);
            l.add_label(20, 210, LABEL_HUE, "Confirm:");
            add_text_field(l, 200, 210, 160, 20, 1);
            add_button_labeled(l, 20, 240, get_button_id(5, 12), "Submit Change");
            cx.account_details(l);
        }
        P::AccountDetailsChangeAccess => {
            let AdminState::Account(a) = *state else {
                return;
            };
            centered(l, 125, "Change Access Level");
            l.add_label(20, 150, LABEL_HUE, "Username:");
            l.add_label(200, 150, LABEL_HUE, &username(w, a));
            l.add_label(20, 170, LABEL_HUE, "Current Level:");
            let al = accounting::account(w, a).map_or(AccessLevel::Player, |x| x.access_level);
            l.add_label(200, 170, LABEL_HUE, format_access_level(al));
            add_button_labeled(l, 20, 200, get_button_id(5, 20), "Player");
            add_button_labeled(l, 20, 220, get_button_id(5, 21), "Counselor");
            add_button_labeled(l, 20, 240, get_button_id(5, 22), "Game Master");
            add_button_labeled(l, 20, 260, get_button_id(5, 23), "Seer");
            if from_level > AccessLevel::Administrator {
                add_button_labeled(l, 20, 280, get_button_id(5, 24), "Administrator");
                if from_level > AccessLevel::Developer {
                    add_button_labeled(l, 20, 300, get_button_id(5, 33), "Developer");
                    if from_level >= AccessLevel::Owner {
                        add_button_labeled(l, 20, 320, get_button_id(5, 34), "Owner");
                    }
                }
            }
            cx.account_details(l);
        }
        P::AccountDetailsInformation => {
            let AdminState::Account(a) = *state else {
                return;
            };
            let mut char_count = 0;
            let len = accounting::account(w, a).map_or(0, |x| x.length());
            for i in 0..len {
                if accounting::mobile_at(w, a, i).is_some() {
                    char_count += 1;
                }
            }
            let is_banned = banned(w, a);
            let Some(acc) = accounting::account(w, a) else {
                return;
            };
            centered(l, 125, "Information");
            l.add_label(20, 150, LABEL_HUE, "Username:");
            l.add_label(200, 150, LABEL_HUE, &acc.username);
            l.add_label(20, 170, LABEL_HUE, "Access Level:");
            l.add_label(200, 170, LABEL_HUE, format_access_level(acc.access_level));
            l.add_label(20, 190, LABEL_HUE, "Status:");
            l.add_label(
                200,
                190,
                if is_banned { RED_HUE } else { GREEN_HUE },
                if is_banned { "Banned" } else { "Active" },
            );
            let (has_tags, ban_time, ban_duration) = acc.get_ban_tags();
            if is_banned && has_tags {
                if ban_duration == TimeSpan::MAX {
                    l.add_label(250, 190, LABEL_HUE, "(Infinite)");
                } else if ban_duration == TimeSpan::ZERO {
                    l.add_label(250, 190, LABEL_HUE, "(Zero)");
                } else {
                    let elapsed = (w.now - ban_time).ticks();
                    let remaining = elapsed.clamp(0, ban_duration.ticks());
                    let perc = remaining as f64 / ban_duration.ticks() as f64;
                    l.add_label(
                        250,
                        190,
                        LABEL_HUE,
                        &format!("{} [{:.0}%]", format_time_span(ban_duration), perc * 100.0), // F0 rounds exact ties to even
                    );
                }
            } else if is_banned {
                l.add_label(250, 190, LABEL_HUE, "(Unspecified)");
            }
            l.add_label(20, 210, LABEL_HUE, "Created:");
            l.add_label(200, 210, LABEL_HUE, &acc.created.to_string());
            l.add_label(20, 230, LABEL_HUE, "Last Login:");
            l.add_label(200, 230, LABEL_HUE, &acc.last_login.to_string());
            l.add_label(20, 250, LABEL_HUE, "Character Count:");
            l.add_label(200, 250, LABEL_HUE, &char_count.to_string());
            l.add_label(20, 270, LABEL_HUE, "Comment Count:");
            l.add_label(200, 270, LABEL_HUE, &acc.comments.len().to_string());
            l.add_label(20, 290, LABEL_HUE, "Tag Count:");
            l.add_label(200, 290, LABEL_HUE, &acc.tags.len().to_string());
            add_button_labeled(l, 20, 320, get_button_id(5, 8), "Change Password");
            add_button_labeled(l, 200, 320, get_button_id(5, 9), "Change Access Level");
            if !is_banned {
                add_button_labeled(l, 20, 350, get_button_id(5, 10), "Ban Account");
            } else {
                add_button_labeled(l, 20, 350, get_button_id(5, 11), "Unban Account");
            }
            add_button_labeled(l, 200, 350, get_button_id(5, 25), "Delete Account");
            cx.account_details(l);
        }
        P::AccountDetailsAccess => {
            let AdminState::Account(_) = *state else {
                return;
            };
            centered(l, 125, "Access");
            cx.page_button(
                l,
                20,
                150,
                get_button_id(5, 14),
                "View client addresses",
                P::AccountDetailsAccessClientIps,
                &[],
            );
            cx.account_details(l);
        }
        P::AccountDetailsAccessClientIps => {
            let AdminState::Account(a) = *state else {
                return;
            };
            if list.is_none() {
                *list = Some(login_ips(w, a).into_iter().map(ListItem::Ip).collect());
            }
            let items = list.clone().unwrap_or_default();
            centered(l, 195, "Client Addresses");
            add_button_labeled(
                l,
                227,
                225,
                get_button_id(5, 16),
                "View all shared accounts",
            );
            add_button_labeled(l, 227, 245, get_button_id(5, 17), "Ban all shared accounts");
            add_button_labeled(l, 227, 265, get_button_id(5, 18), "Firewall all addresses");
            add_button_labeled(l, 227, 285, get_button_id(5, 36), "Clear all addresses");
            l.add_html(
                225,
                315,
                180,
                80,
                &html::color_rgb(
                    "List of IP addresses which have accessed this account.",
                    LABEL_COLOR32,
                ),
                false,
                false,
            );
            l.add_image_tiled(15, 219, 206, 156, 0xBBC);
            add_black_alpha(l, 16, 220, 204, 154);
            l.add_html(
                18,
                221,
                114,
                20,
                &html::color_rgb("IP Address", LABEL_COLOR32),
                false,
                false,
            );
            add_pager(l, 184, 201, 223, list_page, 6, items.len());
            if items.is_empty() {
                l.add_html(
                    18,
                    243,
                    200,
                    60,
                    &html::color_rgb("This account has not yet been accessed.", LABEL_COLOR32),
                    false,
                    false,
                );
            }
            let mut index = list_page * 6;
            let mut i = 0;
            while i < 6 && index >= 0 && (index as usize) < items.len() {
                if let ListItem::Ip(ip) = &items[index as usize] {
                    l.add_html(
                        18,
                        243 + i * 22,
                        114,
                        20,
                        &html::color_rgb(&ip.to_string(), LABEL_COLOR32),
                        false,
                        false,
                    );
                }
                l.add_button(130, 242 + i * 22, 0xFA2, 0xFA4, get_button_id(8, index));
                l.add_button(160, 242 + i * 22, 0xFA8, 0xFAA, get_button_id(9, index));
                l.add_button(190, 242 + i * 22, 0xFB1, 0xFB3, get_button_id(10, index));
                i += 1;
                index += 1;
            }
            // goto case AccountDetails_Access
            centered(l, 125, "Access");
            cx.page_button(
                l,
                20,
                150,
                get_button_id(5, 14),
                "View client addresses",
                P::AccountDetailsAccessClientIps,
                &[],
            );
            cx.account_details(l);
        }
        P::AccountDetailsCharacters => {
            let AdminState::Account(a) = *state else {
                return;
            };
            centered(l, 125, "Characters");
            l.add_label_cropped(12, 150, 120, 20, LABEL_HUE, "Name");
            l.add_label_cropped(132, 150, 120, 20, LABEL_HUE, "Access Level");
            l.add_label_cropped(252, 150, 120, 20, LABEL_HUE, "Status");
            let mut index = 0;
            let len = accounting::account(w, a).map_or(0, |x| x.length());
            for i in 0..len {
                let Some(m) = accounting::mobile_at(w, a, i) else {
                    continue;
                };
                let offset = 170 + index * 20;
                l.add_label_cropped(12, offset, 120, 20, get_hue_for(w, Some(m)), &name_of(w, m));
                l.add_label_cropped(
                    132,
                    offset,
                    120,
                    20,
                    LABEL_HUE,
                    format_access_level(muo_content::mx::access_level(w, m)),
                );
                if w.mobile(m).is_some_and(|x| x.net_state.is_some()) {
                    l.add_label_cropped(252, offset, 120, 20, GREEN_HUE, "Online");
                } else {
                    l.add_label_cropped(252, offset, 120, 20, RED_HUE, "Offline");
                }
                l.add_button(380, offset - 1, 0xFA5, 0xFA7, get_button_id(5, i + 50));
                index += 1;
            }
            if index == 0 {
                l.add_label(12, 170, LABEL_HUE, "The character list is empty.");
            }
            cx.account_details(l);
        }
        P::AccountDetailsComments => {
            let AdminState::Account(a) = *state else {
                return;
            };
            centered(l, 125, "Comments");
            add_button_labeled(l, 20, 150, get_button_id(5, 4), "Add Comment");
            let mut sb = String::new();
            let comments = accounting::account(w, a)
                .map(|x| x.comments.clone())
                .unwrap_or_default();
            if comments.is_empty() {
                sb += "There are no comments for this account.";
            }
            for (i, c) in comments.iter().enumerate() {
                if i > 0 {
                    sb += "<BR><BR>";
                }
                sb += &format!("[{} on {}]<BR>{}", c.added_by, c.last_modified, c.content());
            }
            l.add_html(20, 180, 380, 190, &sb, true, true);
            cx.account_details(l);
        }
        P::AccountDetailsTags => {
            let AdminState::Account(a) = *state else {
                return;
            };
            centered(l, 125, "Tags");
            add_button_labeled(l, 20, 150, get_button_id(5, 5), "Add Tag");
            let mut sb = String::new();
            let tags = accounting::account(w, a)
                .map(|x| x.tags.clone())
                .unwrap_or_default();
            if tags.is_empty() {
                sb += "There are no tags for this account.";
            }
            for (i, t) in tags.iter().enumerate() {
                if i > 0 {
                    sb += "<BR>";
                }
                sb += &format!("{} = {}", t.name, t.value.as_deref().unwrap_or(""));
            }
            l.add_html(20, 180, 380, 190, &sb, true, true);
            cx.account_details(l);
        }
        P::Firewall => {
            add_firewall_header(l);
            if list.is_none() {
                let v = w
                    .resources
                    .get::<Firewall>()
                    .map(|f| f.entries().map(ListItem::Firewall).collect())
                    .unwrap_or_default();
                *list = Some(v);
            }
            let items = list.clone().unwrap_or_default();
            l.add_label_cropped(12, 120, 358, 20, LABEL_HUE, "IP Address");
            add_pager(l, 375, 392, 122, list_page, 12, items.len());
            if items.is_empty() {
                l.add_label(12, 140, LABEL_HUE, "The firewall list is empty.");
            } else {
                // the list is walked from its start whatever the page, numbering the buttons from the page's start
                for (i, item) in items.iter().take(12).enumerate() {
                    let offset = 140 + i as i32 * 20;
                    if let ListItem::Firewall(e) = item {
                        l.add_label_cropped(12, offset, 358, 20, LABEL_HUE, &e.to_string());
                    }
                    l.add_button(
                        380,
                        offset - 1,
                        0xFA5,
                        0xFA7,
                        get_button_id(6, list_page * 12 + i as i32 + 4),
                    );
                }
            }
        }
        P::FirewallInfo => {
            add_firewall_header(l);
            let AdminState::Firewall(entry) = *state else {
                return;
            };
            l.add_html(
                10,
                125,
                400,
                20,
                &html::center_rgb(&entry.to_string(), LABEL_COLOR32),
                false,
                false,
            );
            add_button_labeled(l, 20, 150, get_button_id(6, 3), "Remove");
            centered(l, 175, "Potentially Affected Accounts");
            if list.is_none() {
                let mut blocked = Vec::new();
                for a in all_accounts(w) {
                    if login_ips(w, a).iter().any(|ip| entry.is_blocked(*ip)) {
                        blocked.push(a);
                    }
                }
                sort_accounts(w, &mut blocked);
                *list = Some(blocked.into_iter().map(ListItem::Account).collect());
            }
            let items = list.clone().unwrap_or_default();
            add_pager(l, 375, 392, 177, list_page, 12, items.len());
            if items.is_empty() {
                l.add_label_cropped(12, 200, 398, 20, LABEL_HUE, "No accounts found.");
            }
            let mut index = list_page * 9;
            let mut i = 0;
            while i < 9 && index >= 0 && (index as usize) < items.len() {
                if let ListItem::Account(a) = items[index as usize] {
                    let offset = 200 + i * 20;
                    let (level, online) = get_account_info(w, a);
                    l.add_label_cropped(12, offset, 120, 20, LABEL_HUE, &username(w, a));
                    l.add_label_cropped(
                        132,
                        offset,
                        120,
                        20,
                        LABEL_HUE,
                        format_access_level(level),
                    );
                    status_label(w, l, 252, offset, a, online);
                    l.add_button(380, offset - 1, 0xFA5, 0xFA7, get_button_id(5, index + 56));
                }
                i += 1;
                index += 1;
            }
        }
    }
}

fn add_info_buttons(cx: &Ctx, l: &mut LegacyLayout) {
    cx.page_button(
        l,
        200,
        20,
        get_button_id(0, 0),
        "General",
        P::InformationGeneral,
        &[],
    );
    cx.page_button(
        l,
        200,
        40,
        get_button_id(0, 5),
        "Performance",
        P::InformationPerf,
        &[],
    );
}

// ---- actions ----

/// `InvokeCommand(c)`.
fn invoke_command(w: &mut World, from: MobileId, c: &str) {
    let prefix = w.resources.get_mut::<CommandSystem>().prefix.clone();
    CommandSystem::handle(w, from, &format!("{prefix}{c}"), MessageType::REGULAR);
}

fn log(w: &mut World, from: MobileId, what: &str) {
    let level = muo_content::mx::access_level(w, from);
    let who = muo_commands::logging::format_entity(w, EntityId::Mobile(from));
    muo_commands::logging::write_line(w, from, &format!("{level:?} {who} {what}"));
}

fn format_mobile(w: &World, m: MobileId) -> String {
    muo_commands::logging::format_entity(w, EntityId::Mobile(m))
}

fn generate_spawners(w: &mut World, from: MobileId) {
    use muo_data::expansion::MapSelectionFlags as F;
    let sa = muo_content::core::sa(w);
    let folder = if sa { "post-uoml" } else { "uoml" };
    let maps = w
        .core_expansion_info()
        .map_or(F::FELUCCA | F::TRAMMEL, |i| i.map_selection_flags);
    if sa && maps.contains(F::TER_MUR) {
        invoke_command(
            w,
            from,
            "GenerateSpawners Data/Spawns/post-uoml/termur/**.json",
        );
        invoke_command(
            w,
            from,
            "GenerateSpawners Data/Spawns/shared/termur/**.json",
        );
    }
    for (flag, name) in [
        (F::MALAS, "malas"),
        (F::TOKUNO, "tokuno"),
        (F::ILSHENAR, "ilshenar"),
        (F::TRAMMEL, "trammel"),
        (F::FELUCCA, "felucca"),
    ] {
        if maps.contains(flag) {
            invoke_command(
                w,
                from,
                &format!("GenerateSpawners Data/Spawns/{folder}/{name}/**.json"),
            );
            invoke_command(
                w,
                from,
                &format!("GenerateSpawners Data/Spawns/shared/{name}/**.json"),
            );
        }
    }
}

/// `Shutdown(restart, save)`.
fn shutdown(w: &mut World, from: MobileId, restart: bool, save: bool) {
    log(
        w,
        from,
        &format!(
            "shutting down server (Restart: {}) (Save: {})",
            cap(restart),
            cap(save)
        ),
    );
    if save {
        invoke_command(w, from, "Save");
    }
    // wait for the save writer before the process exits
    w.wait_for_write_completion();
    w.core.kill(restart);
}

/// A bool as the gump shows it: `True`/`False`.
fn cap(b: bool) -> &'static str {
    if b { "True" } else { "False" }
}

/// `ShutdownTimer`.
fn start_shutdown_timer(w: &mut World, from: MobileId) {
    const TIMES: [i32; 8] = [15, 10, 5, 4, 3, 2, 1, 0];
    let index = Rc::new(Cell::new(0usize));
    muo_server::timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 8, move |w| {
        let i = index.get();
        index.set(i + 1);
        if i >= 7 {
            muo_content::world_saves::auto_save::set_saves_enabled(w, false);
            shutdown(w, from, false, true);
            return;
        }
        let time = TIMES[i];
        muo_content::mx::send_message(
            w,
            from,
            &format!(
                "The shard will shutdown in {time} minute{} for maintenance.",
                if time == 1 { "s" } else { "" }
            ),
        );
        if let Some(h) = muo_server::timer::current(w) {
            muo_server::timer::set_interval(
                w,
                h,
                TimeSpan::from_ticks(i64::from(time - TIMES[i + 1]) * 600_000_000),
            );
        }
    });
}

fn lockdown_level_from(index: i32) -> AccessLevel {
    AccessLevel::from_i32(index).unwrap_or(AccessLevel::Player)
}

fn firewall_add(w: &mut World, entry: FirewallEntry) {
    firewall::add(w, entry, TimeSpan::ZERO, true);
    if let Some(ip) = entry.single_address() {
        w.resources.get_mut::<BanReportQueue>().push(
            ip,
            std::time::Duration::ZERO,
            ban_reasons::MANUAL,
        );
    }
}

fn warn(
    w: &mut World,
    from: MobileId,
    text: &str,
    h: i32,
    cb: impl Fn(&mut World, bool) + 'static,
) {
    let g = warning::new(
        TextDefinition::of_text(text),
        420,
        h,
        Some(Rc::new(cb)),
        true,
    );
    send(w, from, g);
}

fn notice_gump(w: &mut World, from: MobileId, content: &str, cb: impl Fn(&mut World) + 'static) {
    let g = warning::new_static_notice(AdminNotice(content.to_owned()), Some(Rc::new(cb)));
    send(w, from, g);
}

/// `AdminNoticeGump`.
struct AdminNotice(String);

impl warning::StaticNotice for AdminNotice {
    const TYPE_NAME: &'static str = "Server.Gumps.AdminGump+AdminNoticeGump";
    const WIDTH: i32 = 420;
    const HEIGHT: i32 = 280;

    fn content(&self) -> Option<String> {
        Some(self.0.clone())
    }
}

fn is_admin(w: &World, from: MobileId) -> bool {
    muo_content::mx::access_level(w, from) >= AccessLevel::Administrator
}

fn ban_shared_callback(w: &mut World, from: MobileId, okay: bool, a: AccountId) {
    if !is_admin(w, from) {
        return;
    }
    let mut list = Vec::new();
    let notice = if okay {
        let ips = login_ips(w, a);
        list = get_shared_accounts(w, &ips);
        let who = mobile_string(w, from);
        for &x in &list {
            if let Some(acc) = accounting::account_mut(w, x) {
                acc.set_unspecified_ban(Some(&who));
            }
            accounting::set_banned(w, x, true);
        }
        "All addresses in the list have been banned."
    } else {
        "You have chosen not to ban all shared accounts."
    };
    open(
        w,
        from,
        P::AccountDetailsAccessClientIps,
        0,
        None,
        Some(notice),
        AdminState::Account(a),
    );
    if okay {
        send(w, from, super::ban_duration::new(list));
    }
}

fn account_delete_callback(w: &mut World, from: MobileId, okay: bool, a: AccountId) {
    if !is_admin(w, from) {
        return;
    }
    if okay {
        let name = username(w, a);
        log(w, from, &format!("deleting account {name}"));
        accounting::delete_account(w, a);
        open(
            w,
            from,
            P::Accounts,
            0,
            None,
            Some(&format!("{name} : The account has been deleted.")),
            AdminState::None,
        );
    } else {
        open(
            w,
            from,
            P::AccountDetailsInformation,
            0,
            None,
            Some("You have chosen not to delete the account."),
            AdminState::Account(a),
        );
    }
}

fn resend_gump_callback(
    w: &mut World,
    from: MobileId,
    list: Vec<ListItem>,
    rads: Vec<AccountId>,
    page: i32,
) {
    if !is_admin(w, from) {
        return;
    }
    open(
        w,
        from,
        P::Accounts,
        page,
        Some(list),
        None,
        AdminState::Accounts(rads),
    );
}

fn marked_callback(
    w: &mut World,
    from: MobileId,
    okay: bool,
    ban: bool,
    mut list: Vec<ListItem>,
    mut rads: Vec<AccountId>,
    page: i32,
) {
    if !is_admin(w, from) {
        return;
    }
    let plural = |n: usize| if n == 1 { "" } else { "s" };
    if okay {
        let who = mobile_string(w, from);
        let mut i = 0;
        while i < rads.len() {
            let acct = rads[i];
            let name = username(w, acct);
            if ban {
                log(w, from, &format!("banning account {name}"));
                if let Some(acc) = accounting::account_mut(w, acct) {
                    acc.set_unspecified_ban(Some(&who));
                }
                accounting::set_banned(w, acct, true);
                i += 1;
            } else {
                log(w, from, &format!("deleting account {name}"));
                accounting::delete_account(w, acct);
                rads.remove(i);
                if let Some(p) = list
                    .iter()
                    .position(|x| matches!(x, ListItem::Account(b) if *b == acct))
                {
                    list.remove(p);
                }
            }
        }
        let msg = format!(
            "You have {} the account{}.",
            if ban { "banned" } else { "deleted" },
            plural(rads.len())
        );
        let (l2, r2) = (list, rads.clone());
        let p = if ban { page } else { 0 };
        notice_gump(w, from, &msg, move |w| {
            resend_gump_callback(w, from, l2.clone(), r2.clone(), p)
        });
        if ban {
            send(w, from, super::ban_duration::new(rads));
        }
    } else {
        let msg = format!(
            "You have chosen not to {} the account{}.",
            if ban { "ban" } else { "delete" },
            plural(rads.len())
        );
        notice_gump(w, from, &msg, move |w| {
            resend_gump_callback(w, from, list.clone(), rads.clone(), page)
        });
    }
}

fn firewall_shared_callback(w: &mut World, from: MobileId, okay: bool, a: AccountId) {
    if !is_admin(w, from) {
        return;
    }
    let notice = if okay {
        for ip in login_ips(w, a) {
            firewall_add(w, FirewallEntry::single(ip));
            // `Firewall.Add(single)` then `BanChannel.Report` for every address: `firewall_add` reported it
        }
        "All addresses in the list have been firewalled."
    } else {
        "You have chosen not to firewall all addresses."
    };
    open(
        w,
        from,
        P::AccountDetailsAccessClientIps,
        0,
        None,
        Some(notice),
        AdminState::Account(a),
    );
}

fn firewall_callback(w: &mut World, from: MobileId, okay: bool, a: AccountId, ip: IpAddr) {
    if !is_admin(w, from) {
        return;
    }
    let notice = if okay {
        firewall_add(w, FirewallEntry::single(ip));
        format!("{ip} : Added to firewall.")
    } else {
        "You have chosen not to firewall the address.".to_owned()
    };
    open(
        w,
        from,
        P::AccountDetailsAccessClientIps,
        0,
        None,
        Some(&notice),
        AdminState::Account(a),
    );
}

fn dec_ip_table(w: &mut World, ip: IpAddr) {
    if let Some(v) = accounting::handler::ip_table(w).get_mut(&ip) {
        *v -= 1;
    }
}

fn remove_login_ip_callback(w: &mut World, from: MobileId, okay: bool, a: AccountId, ip: IpAddr) {
    if !is_admin(w, from) {
        return;
    }
    let notice = if okay {
        let ips = login_ips(w, a);
        if ips.first() == Some(&ip) {
            dec_ip_table(w, ip);
        }
        if let Some(acc) = accounting::account_mut(w, a)
            && let Some(p) = acc.login_ips.iter().position(|x| *x == ip)
        {
            acc.login_ips.remove(p);
        }
        format!("{ip} : Removed address.")
    } else {
        "You have chosen not to remove the address.".to_owned()
    };
    open(
        w,
        from,
        P::AccountDetailsAccessClientIps,
        0,
        None,
        Some(&notice),
        AdminState::Account(a),
    );
}

fn remove_login_ips_callback(w: &mut World, from: MobileId, okay: bool, a: AccountId) {
    if !is_admin(w, from) {
        return;
    }
    let notice = if okay {
        if let Some(first) = login_ips(w, a).first().copied() {
            dec_ip_table(w, first);
        }
        if let Some(acc) = accounting::account_mut(w, a) {
            acc.login_ips.clear();
        }
        "All addresses in the list have been removed."
    } else {
        "You have chosen not to clear all addresses."
    };
    open(
        w,
        from,
        P::AccountDetailsAccessClientIps,
        0,
        None,
        Some(notice),
        AdminState::Account(a),
    );
}

fn set_prompt(w: &mut World, from: MobileId, p: Prompt) {
    let id = add_prompt(w, p);
    muo_server::mobile::part1::props_b::set_prompt(w, from, Some(id));
}

fn add_comment_prompt(w: &mut World, from: MobileId, a: Option<AccountId>) {
    let st = a.map_or(AdminState::None, AdminState::Account);
    let st2 = st.clone();
    set_prompt(
        w,
        from,
        Prompt {
            on_cancel: Some(Rc::new(move |w: &mut World, from| {
                open(
                    w,
                    from,
                    P::AccountDetailsComments,
                    0,
                    None,
                    Some("Request to add comment was canceled."),
                    st.clone(),
                );
            })),
            on_response: Some(Rc::new(move |w: &mut World, from, text: &str| {
                if let Some(a) = a {
                    let raw = muo_server::mobile::part1::props_b::name(w, from).unwrap_or_default();
                    let c = AccountComment::new(w, &raw, text);
                    if let Some(acc) = accounting::account_mut(w, a) {
                        acc.comments.push(c);
                    }
                    // The gump is sent only when the account exists.
                    open(
                        w,
                        from,
                        P::AccountDetailsComments,
                        0,
                        None,
                        Some("Comment added."),
                        st2.clone(),
                    );
                }
            })),
        },
    );
}

pub(super) fn add_tag_value_prompt(
    w: &mut World,
    from: MobileId,
    a: Option<AccountId>,
    name: String,
) {
    let st = a.map_or(AdminState::None, AdminState::Account);
    let st2 = st.clone();
    set_prompt(
        w,
        from,
        Prompt {
            on_cancel: Some(Rc::new(move |w: &mut World, from| {
                open(
                    w,
                    from,
                    P::AccountDetailsTags,
                    0,
                    None,
                    Some("Request to add tag was canceled."),
                    st.clone(),
                );
            })),
            on_response: Some(Rc::new(move |w: &mut World, from, text: &str| {
                // `if (m_Account != null)`: with no account nothing is added and no gump is sent
                if let Some(a) = a {
                    if let Some(acc) = accounting::account_mut(w, a) {
                        acc.add_tag(&name, text);
                    }
                    open(
                        w,
                        from,
                        P::AccountDetailsTags,
                        0,
                        None,
                        Some("Tag added."),
                        st2.clone(),
                    );
                }
            })),
        },
    );
}

fn add_tag_name_prompt(w: &mut World, from: MobileId, a: Option<AccountId>) {
    let st = a.map_or(AdminState::None, AdminState::Account);
    set_prompt(
        w,
        from,
        Prompt {
            on_cancel: Some(Rc::new(move |w: &mut World, from| {
                open(
                    w,
                    from,
                    P::AccountDetailsTags,
                    0,
                    None,
                    Some("Request to add tag was canceled."),
                    st.clone(),
                );
            })),
            on_response: Some(Rc::new(move |w: &mut World, from, text: &str| {
                add_tag_value_prompt(w, from, a, text.to_owned());
                muo_content::mx::send_message(w, from, "Enter the new tag value.");
            })),
        },
    );
}

impl AdminGump {
    fn reopen(
        &self,
        w: &mut World,
        page: P,
        list_page: i32,
        list: Option<Vec<ListItem>>,
        notice: Option<&str>,
        state: AdminState,
    ) {
        open(w, self.from, page, list_page, list, notice, state);
    }

    fn account(&self) -> Option<AccountId> {
        if let AdminState::Account(a) = self.state {
            Some(a)
        } else {
            None
        }
    }

    /// `m_State` as the page's state for sub pages that just re-show it.
    fn goto(&self, w: &mut World, page: P) {
        self.reopen(w, page, 0, None, None, self.state.clone());
    }

    fn search_clients(&self, w: &mut World, info: &RelayInfo, for_name: bool) {
        let from = self.from;
        let mut results: Vec<NetStateId> = Vec::new();
        let m = info.get_text_entry(0).map(|s| s.trim().lower_inv());
        let mut notice = None;
        match m.as_deref() {
            None | Some("") => {
                notice = Some(format!(
                    "You must enter {} to search.",
                    if for_name { "a name" } else { "an ip address" }
                ));
            }
            Some(m) => {
                let ids: Vec<NetStateId> = w.net_states.keys().copied().collect();
                for ns in ids {
                    let is_match = if for_name {
                        let mm = w
                            .net_mobile(ns)
                            .is_some_and(|x| name_of(w, x).lower_inv().contains(m));
                        let aa = ns_account(w, ns)
                            .is_some_and(|a| username(w, a).lower_inv().contains(m));
                        mm || aa
                    } else {
                        w.net_state(ns).is_some_and(|d| d.description.contains(m))
                    };
                    if is_match {
                        results.push(ns);
                    }
                }
                sort_net_states(w, &mut results);
            }
        }
        if results.len() == 1 {
            let ns = results[0];
            let one = Some("One match found.");
            if let Some(mob) = w.net_mobile(ns) {
                self.reopen(w, P::ClientInfo, 0, None, one, AdminState::Mobile(mob));
            } else if let Some(a) = ns_account(w, ns) {
                self.reopen(
                    w,
                    P::AccountDetailsInformation,
                    0,
                    None,
                    one,
                    AdminState::Account(a),
                );
            } else {
                let l = results.into_iter().map(ListItem::Net).collect();
                self.reopen(w, P::Clients, 0, Some(l), one, AdminState::None);
            }
        } else {
            let msg = notice.or_else(|| {
                results
                    .is_empty()
                    .then(|| "Nothing matched your search terms.".to_owned())
            });
            let l = results.into_iter().map(ListItem::Net).collect();
            open(
                w,
                from,
                P::Clients,
                0,
                Some(l),
                msg.as_deref(),
                AdminState::None,
            );
        }
    }

    /// Type 3 (administer).
    fn administer(&mut self, w: &mut World, index: i32, info: &RelayInfo) {
        let from = self.from;
        let from_level = muo_content::mx::access_level(w, from);
        let mut notice: Option<String> = None;
        let mut page = P::Administer;
        if index >= 500 {
            page = P::AdministerAccessLockdown;
        } else if index >= 400 {
            page = P::AdministerCommands;
        } else if index >= 300 {
            page = P::AdministerAccess;
        } else if index >= 200 {
            page = P::AdministerServer;
        } else if index >= 100 {
            page = P::AdministerWorldBuilding;
        }
        let say = |s: &str| Some(s.to_owned());
        let inv = |w: &mut World, c: &str| invoke_command(w, from, c);
        match index {
            0 => page = P::AdministerWorldBuilding,
            1 => page = P::AdministerServer,
            2 => page = P::AdministerAccess,
            3 => page = P::AdministerCommands,
            101 => {
                inv(w, "TelGen");
                notice = say("Teleporters have been generated.");
            }
            102 => {
                inv(w, "MoonGen");
                notice = say("Moongates have been generated.");
            }
            104 => {
                inv(w, "DoorGen");
                notice = say("Doors have been generated.");
            }
            103 => {
                generate_spawners(w, from);
                notice = say("Spawners have been generated.");
            }
            105 => {
                inv(w, "SignGen");
                notice = say("Signs have been generated.");
            }
            106 => {
                inv(w, "Decorate");
                notice = say("Decorations have been generated.");
            }
            107 => {
                inv(w, "GenChamps");
                notice = say("Champions have been generated.");
            }
            108 => {
                inv(w, "DecorateMag");
                notice = say("Magincia Ruins decorations have been generated.");
            }
            109 => {
                inv(w, "GenStealArties");
                notice = say("Stealable Artifacts have been generated.");
            }
            110 => {
                inv(w, "SHTelGen");
                notice = say("Solen Hive teleporters have been generated.");
            }
            111 => {
                inv(w, "SecretLocGen");
                notice = say("Secret Locations have been generated.");
            }
            112 => {
                inv(w, "GenLeverPuzzle");
                inv(w, "GenGauntlet");
                notice = say("Doom  has been generated.");
            }
            113 => {
                inv(w, "GenKhaldun");
                notice = say("Khaldun puzzles have been generated.");
            }
            114 => {
                for c in ["TelGen", "MoonGen", "DoorGen"] {
                    inv(w, c);
                }
                generate_spawners(w, from);
                for c in [
                    "SignGen",
                    "Decorate",
                    "GenChamps",
                    "DecorateMag",
                    "GenStealArties",
                    "SHTelGen",
                    "SecretLocGen",
                    "GenLeverPuzzle",
                    "GenGauntlet",
                    "GenKhaldun",
                ] {
                    inv(w, c);
                }
                notice = say("All the above options have been generated at once.");
            }
            200 => {
                inv(w, "Save");
                notice = say("The world has been saved.");
            }
            201 => shutdown(w, from, false, true),
            202 => shutdown(w, from, false, false),
            203 => shutdown(w, from, true, true),
            204 => shutdown(w, from, true, false),
            205 => start_shutdown_timer(w, from),
            210 | 211 => {
                let text = info.get_text_entry(0).map(str::trim).unwrap_or("");
                if text.is_empty() {
                    notice = say("You must enter text to broadcast it.");
                } else {
                    notice = say("Your message has been broadcasted.");
                    inv(
                        w,
                        &format!("{} {text}", if index == 210 { "BC" } else { "SM" }),
                    );
                }
            }
            300 => {
                inv(w, "Kick");
                notice = say("Target the player to kick.");
            }
            301 => {
                inv(w, "Ban");
                notice = say("Target the player to ban.");
            }
            302 => {
                inv(w, "Firewall");
                notice = say("Target the player to firewall.");
            }
            303 => page = P::AdministerAccessLockdown,
            310 => {
                inv(w, "Set AccessLevel Player");
                notice = say("Target the player to change their access level. (Player)");
            }
            311 => {
                inv(w, "Set AccessLevel Counselor");
                notice = say("Target the player to change their access level. (Counselor)");
            }
            312 => {
                inv(w, "Set AccessLevel GameMaster");
                notice = say("Target the player to change their access level. (Game Master)");
            }
            313 => {
                inv(w, "Set AccessLevel Seer");
                notice = say("Target the player to change their access level. (Seer)");
            }
            314 => {
                if from_level > AccessLevel::Administrator {
                    inv(w, "Set AccessLevel Administrator");
                    notice = say("Target the player to change their access level. (Administrator)");
                }
            }
            315 => {
                if from_level > AccessLevel::Developer {
                    inv(w, "Set AccessLevel Developer");
                    notice = say("Target the player to change their access level. (Developer)");
                }
            }
            316 => {
                if from_level >= AccessLevel::Owner {
                    inv(w, "Set AccessLevel Owner");
                    notice = say("Target the player to change their access level. (Owner)");
                }
            }
            400 => notice = say("Enter search terms to add objects."),
            401 => {
                inv(w, "Remove");
                notice = say("Target the item or mobile to remove.");
            }
            402 => {
                inv(w, "Dupe");
                notice = say("Target the item to dupe.");
            }
            403 => {
                inv(w, "DupeInBag");
                notice = say(
                    "Target the item to dupe. The item will be duped at it's current location.",
                );
            }
            404 => {
                inv(w, "Props");
                notice = say("Target the item or mobile to inspect.");
            }
            405 => {
                inv(w, "Skills");
                notice = say("Target a mobile to view their skills.");
            }
            406 => {
                inv(w, "Set Blessed False");
                notice = say("Target the mobile to make mortal.");
            }
            407 => {
                inv(w, "Set Blessed True");
                notice = say("Target the mobile to make immortal.");
            }
            408 => {
                inv(w, "Set Squelched True");
                notice = say("Target the mobile to squelch.");
            }
            409 => {
                inv(w, "Set Squelched False");
                notice = say("Target the mobile to unsquelch.");
            }
            410 => {
                inv(w, "Set Frozen True");
                notice = say("Target the mobile to freeze.");
            }
            411 => {
                inv(w, "Set Frozen False");
                notice = say("Target the mobile to unfreeze.");
            }
            412 => {
                inv(w, "Set Hidden True");
                notice = say("Target the mobile to hide.");
            }
            413 => {
                inv(w, "Set Hidden False");
                notice = say("Target the mobile to unhide.");
            }
            414 => {
                inv(w, "Kill");
                notice = say("Target the mobile to kill.");
            }
            415 => {
                inv(w, "Resurrect");
                notice = say("Target the mobile to resurrect.");
            }
            416 => {
                inv(w, "Move");
                notice = say("Target the item or mobile to move.");
            }
            417 => {
                inv(w, "Wipe");
                notice = say("Target bounding points.");
            }
            418 => {
                inv(w, "Tele");
                notice = say("Choose your destination.");
            }
            419 => {
                inv(w, "Multi Tele");
                notice = say("Choose your destination.");
            }
            500..=504 => {
                let level = lockdown_level_from(index - 500);
                accounting::handler::state(w).lockdown_level = level;
                notice = say(if level > AccessLevel::Player {
                    "The lockdown level has been changed."
                } else {
                    "The server is now accessible to everyone."
                });
            }
            510 => {
                let level = accounting::handler::lockdown_level(w);
                if level > AccessLevel::Player {
                    let mut count = 0;
                    let ids: Vec<NetStateId> = w.net_states.keys().copied().collect();
                    for ns in ids {
                        let Some(a) = ns_account(w, ns) else { continue };
                        let mut has_access =
                            accounting::account(w, a).is_some_and(|x| x.access_level >= level);
                        let len = accounting::account(w, a).map_or(0, |x| x.length());
                        let mut j = 0;
                        while !has_access && j < len {
                            if let Some(m) = accounting::mobile_at(w, a, j)
                                && muo_content::mx::access_level(w, m) >= level
                            {
                                has_access = true;
                            }
                            j += 1;
                        }
                        if !has_access {
                            w.disconnect(ns, "Server has been locked down.");
                            count += 1;
                        }
                    }
                    notice = Some(if count == 0 {
                        "Nobody without access was found to disconnect.".to_owned()
                    } else {
                        format!("Number of players disconnected: {count}")
                    });
                } else {
                    notice = say("The server is not currently locked down.");
                }
            }
            _ => {}
        }
        open(w, from, page, 0, None, notice.as_deref(), AdminState::None);
        if index == 400 {
            inv(w, "Add");
        }
    }

    /// Type 4 (clients).
    fn clients(&mut self, w: &mut World, index: i32, info: &RelayInfo) {
        let from = self.from;
        match index {
            0 | 1 => self.search_clients(w, info, index == 0),
            _ => {
                let i = (index - 2) as usize;
                let Some(ListItem::Net(ns)) = self.list.as_ref().and_then(|l| l.get(i)).cloned()
                else {
                    return;
                };
                let m = w.net_mobile(ns);
                let a = ns_account(w, ns);
                if let Some(m) = m {
                    open(w, from, P::ClientInfo, 0, None, None, AdminState::Mobile(m));
                } else if let Some(a) = a {
                    open(
                        w,
                        from,
                        P::AccountDetailsInformation,
                        0,
                        None,
                        None,
                        AdminState::Account(a),
                    );
                }
            }
        }
    }

    fn match_results(&self, w: &mut World, results: Vec<AccountId>, state_for_list: AdminState) {
        if results.len() == 1 {
            self.reopen(
                w,
                P::AccountDetailsInformation,
                0,
                None,
                Some("One match found."),
                AdminState::Account(results[0]),
            );
        } else {
            let notice = results
                .is_empty()
                .then_some("Nothing matched your search terms.");
            let l = results.into_iter().map(ListItem::Account).collect();
            self.reopen(w, P::Accounts, 0, Some(l), notice, state_for_list);
        }
    }

    fn view_shared(&self, w: &mut World, a: AccountId) {
        let ips = login_ips(w, a);
        let list = get_shared_accounts(w, &ips);
        if list.len() > 1 || list.len() == 1 && !list.contains(&a) {
            let l = list.into_iter().map(ListItem::Account).collect();
            self.reopen(
                w,
                P::Accounts,
                0,
                Some(l),
                None,
                AdminState::Objects(vec![]),
            );
        } else if !ips.is_empty() {
            self.reopen(
                w,
                P::AccountDetailsAccessClientIps,
                0,
                None,
                Some("There are no other accounts which share an address with this one."),
                self.state.clone(),
            );
        } else {
            self.reopen(
                w,
                P::AccountDetailsAccessClientIps,
                0,
                None,
                Some("This account has not yet been accessed."),
                self.state.clone(),
            );
        }
    }

    /// Type 5 (accounts).
    fn accounts(&mut self, w: &mut World, index: i32, info: &RelayInfo) {
        let from = self.from;
        let who = mobile_string(w, from);
        let acct = self.account();
        match index {
            0 => self.goto(w, P::AccountDetailsInformation),
            1 => self.goto(w, P::AccountDetailsCharacters),
            2 => self.goto(w, P::AccountDetailsComments),
            3 => self.goto(w, P::AccountDetailsTags),
            13 => self.goto(w, P::AccountDetailsAccess),
            14 => self.goto(w, P::AccountDetailsAccessClientIps),
            4 => {
                add_comment_prompt(w, from, acct);
                muo_content::mx::send_message(w, from, "Enter the new account comment.");
            }
            5 => {
                add_tag_name_prompt(w, from, acct);
                muo_content::mx::send_message(w, from, "Enter the new tag name.");
            }
            6 => {
                let un = info
                    .get_text_entry(0)
                    .map(str::trim)
                    .unwrap_or("")
                    .to_owned();
                let pw = info
                    .get_text_entry(1)
                    .map(str::trim)
                    .unwrap_or("")
                    .to_owned();
                let mut disp = None;
                let notice;
                if un.is_empty() {
                    notice = "You must enter a username to add an account.".to_owned();
                } else if pw.is_empty() {
                    notice = "You must enter a password to add an account.".to_owned();
                } else if accounting::get_account(w, &un).is_some() {
                    notice = "There is already an account with that username.".to_owned();
                } else {
                    disp = Some(accounting::create_account(w, &un, &pw));
                    notice = format!("{un} : Account added.");
                    log(w, from, &format!("adding new account: {un}"));
                }
                let st = disp.map_or_else(|| self.state.clone(), AdminState::Account);
                let page = if disp.is_some() {
                    P::AccountDetailsInformation
                } else {
                    self.page_type
                };
                self.reopen(
                    w,
                    page,
                    self.list_page,
                    self.list.clone(),
                    Some(&notice),
                    st,
                );
            }
            7 => {
                let m = info
                    .get_text_entry(0)
                    .map(|s| s.trim().lower_inv())
                    .unwrap_or_default();
                let mut results = all_accounts(w);
                if !m.is_empty() {
                    results.retain(|&a| username(w, a).lower_inv().contains(&m));
                }
                sort_accounts(w, &mut results);
                self.match_results(w, results, AdminState::Objects(vec![]));
            }
            8 => self.goto(w, P::AccountDetailsChangePassword),
            9 => self.goto(w, P::AccountDetailsChangeAccess),
            10 | 11 => {
                let Some(a) = acct else { return };
                if let Some(acc) = accounting::account_mut(w, a) {
                    acc.set_unspecified_ban(Some(&who));
                }
                accounting::set_banned(w, a, index == 10);
                let b = banned(w, a);
                log(
                    w,
                    from,
                    &format!(
                        "{} account {}",
                        username(w, a),
                        if b { "banning" } else { "unbanning" }
                    ),
                );
                let n = format!(
                    "The account has been {}.",
                    if b { "banned" } else { "unbanned" }
                );
                self.reopen(
                    w,
                    self.page_type,
                    self.list_page,
                    self.list.clone(),
                    Some(&n),
                    self.state.clone(),
                );
                if index == 10 {
                    send(w, from, super::ban_duration::new_one(a));
                }
            }
            12 => {
                let Some(a) = acct else { return };
                let password = info
                    .get_text_entry(0)
                    .map(str::trim)
                    .unwrap_or("")
                    .to_owned();
                let confirm = info.get_text_entry(1).map(str::trim);
                let mut page = P::AccountDetailsChangePassword;
                let notice;
                if password.is_empty() {
                    notice = "You must enter the password.";
                } else if confirm != Some(password.as_str()) {
                    notice = "You must confirm the password. That field must precisely match the password field.";
                } else {
                    notice = "The password has been changed.";
                    accounting::password_worker::set_password(w, a, &password, None);
                    page = P::AccountDetailsInformation;
                    log(
                        w,
                        from,
                        &format!("changing password of account {}", username(w, a)),
                    );
                }
                self.reopen(w, page, 0, None, Some(notice), self.state.clone());
            }
            16 => {
                let Some(a) = acct else { return };
                self.view_shared(w, a);
            }
            17 => {
                let Some(a) = acct else { return };
                let ips = login_ips(w, a);
                let list = get_shared_accounts(w, &ips);
                if !list.is_empty() {
                    let mut sb = format!(
                        "You are about to ban {}{} Do you wish to continue?",
                        list.len(),
                        if list.len() != 1 {
                            "accounts."
                        } else {
                            "account."
                        }
                    );
                    for &x in &list {
                        sb += &format!("<br>- {}", username(w, x));
                    }
                    warn(w, from, &sb, 400, move |w, okay| {
                        ban_shared_callback(w, from, okay, a)
                    });
                } else if !ips.is_empty() {
                    self.reopen(
                        w,
                        P::AccountDetailsAccessClientIps,
                        0,
                        None,
                        Some("There are no accounts which share an address with this one."),
                        self.state.clone(),
                    );
                } else {
                    self.reopen(
                        w,
                        P::AccountDetailsAccessClientIps,
                        0,
                        None,
                        Some("This account has not yet been accessed."),
                        self.state.clone(),
                    );
                }
            }
            18 => {
                let Some(a) = acct else { return };
                let n = login_ips(w, a).len();
                if n > 0 {
                    let t = format!(
                        "You are about to firewall {n} address{}. Do you wish to continue?",
                        if n != 1 { "s" } else { "" }
                    );
                    warn(w, from, &t, 400, move |w, okay| {
                        firewall_shared_callback(w, from, okay, a)
                    });
                } else {
                    self.reopen(
                        w,
                        P::AccountDetailsAccessClientIps,
                        0,
                        None,
                        Some("This account has not yet been accessed."),
                        self.state.clone(),
                    );
                }
            }
            20..=24 | 33 | 34 => {
                let Some(a) = acct else { return };
                let new_level = match index {
                    21 => AccessLevel::Counselor,
                    22 => AccessLevel::GameMaster,
                    23 => AccessLevel::Seer,
                    24 => AccessLevel::Administrator,
                    33 => AccessLevel::Developer,
                    34 => AccessLevel::Owner,
                    _ => AccessLevel::Player,
                };
                let from_level = muo_content::mx::access_level(w, from);
                // nothing happens (no log, no gump) unless the new level is below the admin's own
                if (new_level < from_level || from_level == AccessLevel::Owner)
                    && let Some(acc) = accounting::account_mut(w, a)
                {
                    acc.access_level = new_level;
                    log(
                        w,
                        from,
                        &format!(
                            "changing access level of account {} to {new_level:?}",
                            username(w, a)
                        ),
                    );
                    self.reopen(
                        w,
                        P::AccountDetailsInformation,
                        0,
                        None,
                        Some("The access level has been changed."),
                        self.state.clone(),
                    );
                }
            }
            25 => {
                let Some(a) = acct else { return };
                let t = format!(
                    "<center>Account of {}</center><br>You are about to <em><basefont color=red>permanently delete</basefont></em> the account. Likewise, all characters on the account will be deleted, including equipped, inventory, and banked items. Any houses tied to the account will be demolished.<br><br>Do you wish to continue?",
                    username(w, a)
                );
                warn(w, from, &t, 280, move |w, okay| {
                    account_delete_callback(w, from, okay, a)
                });
            }
            26 => open(w, from, P::AccountsShared, 0, None, None, AdminState::None),
            27 | 28 => {
                let ban = index == 27;
                let AdminState::Accounts(rads) = self.state.clone() else {
                    return;
                };
                let Some(list) = self.list.clone() else {
                    return;
                };
                let page = self.list_page;
                if !rads.is_empty() {
                    let s = if rads.len() == 1 { "" } else { "s" };
                    let t = if ban {
                        format!(
                            "You are about to ban {} marked account{s}. Be cautioned, the only way to reverse this is by hand--manually unbanning each account.<br><br>Do you wish to continue?",
                            rads.len()
                        )
                    } else {
                        format!(
                            "You are about to <em><basefont color=red>permanently delete</basefont></em> {} marked account{s}. Likewise, all characters on the account{s} will be deleted, including equipped, inventory, and banked items. Any houses tied to the account{s} will be demolished.<br><br>Do you wish to continue?",
                            rads.len()
                        )
                    };
                    warn(w, from, &t, 280, move |w, okay| {
                        marked_callback(w, from, okay, ban, list.clone(), rads.clone(), page)
                    });
                } else {
                    notice_gump(
                        w,
                        from,
                        "You have not yet marked any accounts. Place a check mark next to the accounts you wish to ban and then try again.",
                        move |w| resend_gump_callback(w, from, list.clone(), rads.clone(), page),
                    );
                }
            }
            29 => {
                let (Some(list), AdminState::Objects(_)) = (self.list.clone(), &self.state) else {
                    return;
                };
                let all: Vec<AccountId> = list
                    .iter()
                    .filter_map(|x| {
                        if let ListItem::Account(a) = x {
                            Some(*a)
                        } else {
                            None
                        }
                    })
                    .collect();
                // `m_List.ToList()` is a List<object>, so the next gump's `state as List<Account>` is null
                self.reopen(
                    w,
                    P::Accounts,
                    self.list_page,
                    Some(list),
                    None,
                    AdminState::Objects(all.into_iter().map(ListItem::Account).collect()),
                );
            }
            30..=32 => {
                let mut results = Vec::new();
                for a in all_accounts(w) {
                    let hit = match index {
                        30 => {
                            let len = accounting::account(w, a).map_or(0, |x| x.length());
                            (0..len).all(|i| accounting::mobile_at(w, a, i).is_none())
                        }
                        31 => accounting::is_inactive(w, a),
                        _ => banned(w, a),
                    };
                    if hit {
                        results.push(a);
                    }
                }
                self.match_results(w, results, AdminState::Objects(vec![]));
            }
            35 => {
                let (Some(list), AdminState::Accounts(rads)) =
                    (self.list.clone(), self.state.clone())
                else {
                    return;
                };
                let mut new_rads = Vec::new();
                for acct in rads {
                    let len = accounting::account(w, acct).map_or(0, |x| x.length());
                    let mut has_house = false;
                    let mut i = 0;
                    while i < len && !has_house {
                        if let Some(m) = accounting::mobile_at(w, acct, i)
                            && muo_multis::house::access::has_house(w, m)
                        {
                            has_house = true;
                        }
                        i += 1;
                    }
                    if !has_house {
                        new_rads.push(acct);
                    }
                }
                self.reopen(
                    w,
                    P::Accounts,
                    self.list_page,
                    Some(list),
                    None,
                    AdminState::Accounts(new_rads),
                );
            }
            36 => {
                let Some(a) = acct else { return };
                let ips = login_ips(w, a);
                if ips.is_empty() {
                    self.reopen(
                        w,
                        P::AccountDetailsAccessClientIps,
                        0,
                        None,
                        Some("This account has not yet been accessed."),
                        self.state.clone(),
                    );
                } else {
                    let t = format!(
                        "You are about to clear the address list for account {} containing {} {}. Do you wish to continue?",
                        username(w, a),
                        ips.len(),
                        if ips.len() == 1 { "entry" } else { "entries" }
                    );
                    warn(w, from, &t, 280, move |w, okay| {
                        remove_login_ips_callback(w, from, okay, a)
                    });
                }
            }
            _ => {
                let mut index = index - 50;
                let len = acct
                    .and_then(|a| accounting::account(w, a))
                    .map_or(0, |x| x.length());
                if let (Some(a), true) = (acct, index >= 0 && index < len) {
                    if let Some(m) = accounting::mobile_at(w, a, index) {
                        open(w, from, P::ClientInfo, 0, None, None, AdminState::Mobile(m));
                    }
                } else {
                    index -= 6;
                    let Some(item) = self
                        .list
                        .as_ref()
                        .and_then(|l| usize::try_from(index).ok().and_then(|i| l.get(i)))
                        .cloned()
                    else {
                        return;
                    };
                    match item {
                        ListItem::Account(a) => open(
                            w,
                            from,
                            P::AccountDetailsInformation,
                            0,
                            None,
                            None,
                            AdminState::Account(a),
                        ),
                        ListItem::Shared(_, accts) => {
                            let l = accts.into_iter().map(ListItem::Account).collect();
                            open(
                                w,
                                from,
                                P::Accounts,
                                0,
                                Some(l),
                                None,
                                AdminState::Objects(vec![]),
                            );
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    /// Type 6 (firewall).
    fn firewall(&mut self, w: &mut World, index: i32, info: &RelayInfo) {
        let from = self.from;
        match index {
            0 => {
                let m = info
                    .get_text_entry(0)
                    .map(str::trim)
                    .unwrap_or("")
                    .to_owned();
                let mut notice = None;
                let mut results = Vec::new();
                if m.is_empty() {
                    notice = Some("You must enter an IP to search.".to_owned());
                } else if let Some(f) = w.resources.get::<Firewall>() {
                    for e in f.entries() {
                        if e.to_string().contains(&m) {
                            results.push(e);
                        }
                    }
                }
                if results.len() == 1 {
                    self.reopen(
                        w,
                        P::FirewallInfo,
                        0,
                        None,
                        Some("One match found."),
                        AdminState::Firewall(results[0]),
                    );
                } else if results.len() > 1 {
                    let l = results.into_iter().map(ListItem::Firewall).collect();
                    self.reopen(
                        w,
                        P::Firewall,
                        0,
                        Some(l),
                        Some(&format!("Search results for : {m}")),
                        self.state.clone(),
                    );
                } else {
                    let n =
                        notice.unwrap_or_else(|| "Nothing matched your search terms.".to_owned());
                    self.reopen(
                        w,
                        self.page_type,
                        self.list_page,
                        self.list.clone(),
                        Some(&n),
                        self.state.clone(),
                    );
                }
            }
            1 => {
                let text = info
                    .get_text_entry(0)
                    .map(str::trim)
                    .unwrap_or("")
                    .to_owned();
                if text.is_empty() {
                    self.reopen(
                        w,
                        self.page_type,
                        self.list_page,
                        self.list.clone(),
                        Some("You must enter an address or CIDR to add."),
                        self.state.clone(),
                    );
                } else {
                    let Some(entry) = FirewallEntry::parse(&text) else {
                        self.reopen(
                            w,
                            self.page_type,
                            self.list_page,
                            self.list.clone(),
                            Some("That is not a valid address or CIDR."),
                            self.state.clone(),
                        );
                        return;
                    };
                    log(w, from, &format!("firewalling {entry}"));
                    firewall_add(w, entry);
                    self.reopen(
                        w,
                        P::FirewallInfo,
                        0,
                        None,
                        Some(&format!("{entry} : Added to firewall.")),
                        AdminState::Firewall(entry),
                    );
                }
            }
            2 => {
                invoke_command(w, from, "Firewall");
                self.reopen(
                    w,
                    self.page_type,
                    self.list_page,
                    self.list.clone(),
                    Some("Target the player to firewall."),
                    self.state.clone(),
                );
            }
            3 => {
                if let AdminState::Firewall(e) = self.state {
                    log(w, from, &format!("removing {e} from firewall list"));
                    firewall::remove(w, &e);
                    if let Some(ip) = e.single_address()
                        && let Some(f) = hooks(w).and_then(|h| h.ban_retract)
                    {
                        f(w, ip);
                    }
                    self.reopen(
                        w,
                        P::Firewall,
                        0,
                        None,
                        Some(&format!("{e} : Removed from firewall.")),
                        AdminState::None,
                    );
                }
            }
            _ => {
                let i = (index - 4) as usize;
                if let Some(ListItem::Firewall(e)) =
                    self.list.as_ref().and_then(|l| l.get(i)).cloned()
                {
                    self.reopen(w, P::FirewallInfo, 0, None, None, AdminState::Firewall(e));
                } else if self.list.as_ref().is_some_and(|l| i < l.len()) {
                    self.reopen(w, P::FirewallInfo, 0, None, None, AdminState::None);
                }
            }
        }
    }

    /// Type 7 (client info).
    fn client_info(&mut self, w: &mut World, index: i32, info: &RelayInfo) {
        let from = self.from;
        let AdminState::Mobile(m) = self.state else {
            return;
        };
        let from_level = muo_content::mx::access_level(w, from);
        let acct = w.mobile(m).and_then(|x| x.account);
        if muo_content::mx::access_level(w, m) > from_level
            || acct
                .and_then(|a| accounting::account(w, a))
                .is_some_and(|a| a.access_level > from_level)
        {
            return;
        }
        let mut notice: Option<String> = None;
        let mut send_gump = true;
        let target = EntityId::Mobile(m);
        let set = |w: &mut World, name: &str, v: &str| {
            muo_commands::properties::set_value(w, from, target, name, v);
        };
        match index {
            0 => {
                let (mut map, mut loc) =
                    (muo_content::mx::map(w, m), muo_content::mx::location(w, m));
                if map.is_none() || map == Some(muo_server::map::MapId::INTERNAL) {
                    let mm = w.mobile(m);
                    map = mm.and_then(|x| x.logout_map);
                    loc = mm.map_or(loc, |x| x.logout_location);
                }
                if let Some(map) = map.filter(|x| *x != muo_server::map::MapId::INTERNAL) {
                    muo_server::mobile::base::move_to_world(w, from, loc, map);
                    notice = Some("You have been teleported to their location.".into());
                }
            }
            1 => {
                let (loc, map) = (
                    muo_content::mx::location(w, from),
                    muo_content::mx::map(w, from),
                );
                if let Some(map) = map {
                    muo_server::mobile::base::move_to_world(w, m, loc, map);
                }
                notice = Some("They have been teleported to your location.".into());
            }
            2 => {
                if let Some(ns) = w.mobile(m).and_then(|x| x.net_state) {
                    let fm = format_mobile(w, m);
                    log(w, from, &format!("kicking {fm}"));
                    let by = super::common::mobile_string(w, from);
                    w.disconnect(ns, &format!("Kicked by {by}."));
                    notice = Some("They have been kicked.".into());
                } else {
                    notice = Some("They are already disconnected.".into());
                }
            }
            3 => {
                if let Some(a) = acct {
                    let fm = format_mobile(w, m);
                    log(w, from, &format!("banning {fm}"));
                    accounting::set_banned(w, a, true);
                    if let Some(ns) = w.mobile(m).and_then(|x| x.net_state) {
                        let by = super::common::mobile_string(w, from);
                        w.disconnect(ns, &format!("Banned by {by}."));
                    }
                    notice = Some("They have been banned.".into());
                }
            }
            6 => {
                set(w, "Blessed", "False");
                notice = Some("They are now mortal.".into());
            }
            7 => {
                set(w, "Blessed", "True");
                notice = Some("They are now immortal.".into());
            }
            8 => {
                set(w, "Squelched", "True");
                notice = Some("They are now squelched.".into());
            }
            9 => {
                set(w, "Squelched", "False");
                notice = Some("They are now unsquelched.".into());
            }
            10 => {
                set(w, "Hidden", "True");
                notice = Some("They are now hidden.".into());
            }
            11 => {
                set(w, "Hidden", "False");
                notice = Some("They are now unhidden.".into());
            }
            12 => {
                let fm = format_mobile(w, m);
                log(w, from, &format!("killing {fm}"));
                muo_server::mobile::base::kill(w, m);
                notice = Some("They have been killed.".into());
            }
            13 => {
                let fm = format_mobile(w, m);
                log(w, from, &format!("resurrecting {fm}"));
                muo_server::mobile::base::resurrect(w, m);
                notice = Some("They have been resurrected.".into());
            }
            14 => {
                let st = acct.map_or(AdminState::None, AdminState::Account);
                open(w, from, P::AccountDetailsInformation, 0, None, None, st);
                send_gump = false;
            }
            15 => {
                let reason = info
                    .get_text_entry(1)
                    .map(str::trim)
                    .unwrap_or("")
                    .to_owned();
                let fm = format_mobile(w, m);
                log(w, from, &format!("jailing {fm} - Reason: {reason}"));
                invoke_command(w, from, &format!("Jail {} \"{reason}\"", name_of(w, m)));
                notice = Some(format!("Player has been sent to jail. Reason: {reason}"));
            }
            16 => {
                let fm = format_mobile(w, m);
                log(w, from, &format!("unjailing {fm}"));
                invoke_command(w, from, &format!("Unjail {}", name_of(w, m)));
                notice = Some("Player has been unjailed.".into());
            }
            _ => {}
        }
        if send_gump {
            open(
                w,
                from,
                P::ClientInfo,
                0,
                None,
                notice.as_deref(),
                self.state.clone(),
            );
        }
        match index {
            3 => {
                if let Some(a) = acct {
                    send(w, from, super::ban_duration::new_one(a));
                }
            }
            4 => {
                if let Some(f) = hooks(w).and_then(|h| h.open_properties_gump) {
                    f(w, from, m);
                }
            }
            5 => {
                if let Some(f) = hooks(w).and_then(|h| h.open_skills_gump) {
                    f(w, from, m);
                }
            }
            _ => {}
        }
    }

    /// Types 8 to 10 (the client addresses of an account).
    fn client_ip(&mut self, w: &mut World, kind: i32, index: i32) {
        let from = self.from;
        let Some(item) = self
            .list
            .as_ref()
            .and_then(|l| usize::try_from(index).ok().and_then(|i| l.get(i)))
            .cloned()
        else {
            return;
        };
        if self.page_type != P::AccountDetailsAccessClientIps {
            return;
        }
        let Some(a) = self.account() else { return };
        let ListItem::Ip(ip) = item else { return };
        match kind {
            8 => {
                let t = format!(
                    "You are about to firewall {ip}. All connection attempts from a matching IP will be refused. Are you sure?"
                );
                warn(w, from, &t, 280, move |w, okay| {
                    firewall_callback(w, from, okay, a, ip)
                });
            }
            9 => {
                let list = get_shared_accounts(w, &[ip]);
                if list.len() > 1 || list.len() == 1 && !list.contains(&a) {
                    let l = list.into_iter().map(ListItem::Account).collect();
                    open(
                        w,
                        from,
                        P::Accounts,
                        0,
                        Some(l),
                        None,
                        AdminState::Objects(vec![]),
                    );
                } else {
                    open(
                        w,
                        from,
                        P::AccountDetailsAccessClientIps,
                        0,
                        None,
                        Some("There are no other accounts which share that address."),
                        AdminState::Account(a),
                    );
                }
            }
            _ => {
                let t = format!(
                    "You are about to remove address {ip} from account {}. Do you wish to continue?",
                    username(w, a)
                );
                warn(w, from, &t, 280, move |w, okay| {
                    remove_login_ip_callback(w, from, okay, a, ip)
                });
            }
        }
    }
}

impl LegacyGumpLogic for AdminGump {
    const TYPE_NAME: &'static str = TYPE_NAME;

    fn singleton(&self) -> bool {
        true
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let val = info.button_id - 1;
        if val < 0 {
            return;
        }
        let from = self.from;
        let _ = mobile_of(w, sender);
        if !is_admin(w, from) {
            return;
        }
        if self.page_type == P::Accounts
            && let (AdminState::Accounts(rads), Some(list)) = (&mut self.state, &self.list)
        {
            let start = self.list_page * 12;
            for v in start..start + 12 {
                if v as usize >= list.len() {
                    break;
                }
                if let ListItem::Account(acct) = list[v as usize] {
                    if !info.is_switched(v) {
                        rads.retain(|x| *x != acct);
                    } else if !rads.contains(&acct) {
                        rads.push(acct);
                    }
                }
            }
        }
        let kind = val % 11;
        let index = val / 11;
        match kind {
            0 => {
                let page = match index {
                    0 => P::InformationGeneral,
                    1 => P::Administer,
                    2 => P::Clients,
                    3 => P::Accounts,
                    4 => P::Firewall,
                    5 => P::InformationPerf,
                    _ => return,
                };
                open(w, from, page, 0, None, None, AdminState::None);
            }
            1 => match index {
                0 => {
                    if self.list.is_some() && self.list_page > 0 {
                        self.reopen(
                            w,
                            self.page_type,
                            self.list_page - 1,
                            self.list.clone(),
                            None,
                            self.state.clone(),
                        );
                    }
                }
                1 if self.list.is_some() => {
                    self.reopen(
                        w,
                        self.page_type,
                        self.list_page + 1,
                        self.list.clone(),
                        None,
                        self.state.clone(),
                    );
                }
                _ => {}
            },
            3 => self.administer(w, index, info),
            4 => self.clients(w, index, info),
            5 => self.accounts(w, index, info),
            6 => self.firewall(w, index, info),
            7 => self.client_info(w, index, info),
            8..=10 => self.client_ip(w, kind, index),
            _ => {}
        }
    }
}

/// `AdminGump.Configure`: `CommandSystem.Register("Admin", AccessLevel.Administrator, Admin_OnCommand)`.
pub fn configure(w: &mut World) {
    CommandSystem::register(
        w,
        "Admin",
        AccessLevel::Administrator,
        &[],
        Rc::new(|w: &mut World, e: &mut CommandEventArgs| {
            // Usage: Admin; Description: Opens an interface providing server information and administration
            // features including client, account, and firewall management.
            open(w, e.mobile, P::Clients, 0, None, None, AdminState::None);
        }),
    );
}

#[allow(dead_code)]
const _UNUSED: (i32, i32) = (LABEL_COLOR, SELECTED_COLOR);
#[allow(dead_code)]
const _UNUSED2: i32 = SELECTED_COLOR32;

#[cfg(test)]
mod prompt_tests {
    use super::*;
    use muo_server::mobile::MOBILE_BASE;
    use muo_server::net_state::NetStateData;

    /// `AddCommentPrompt.OnResponse` with no account: nothing happens, no gump (it was sent either way before).
    #[test]
    fn add_comment_prompt_without_an_account_sends_no_gump() {
        let mut w = World::new();
        let gm = w.new_mobile(&MOBILE_BASE);
        let ns = NetStateId(1);
        w.net_states.insert(
            ns,
            NetStateData {
                mobile: Some(gm),
                ..Default::default()
            },
        );
        w.mobile_mut(gm).unwrap().net_state = Some(ns);
        add_comment_prompt(&mut w, gm, None);
        let pid = w.mobile(gm).unwrap().prompt.expect("prompt set");
        let f = w
            .prompts
            .get(&pid)
            .and_then(|p| p.on_response.clone())
            .unwrap();
        f(&mut w, gm, "hello");
        assert!(!muo_content::gumps::has_gump::<AdminGump>(&w, ns));
    }

    /// `SetUnspecifiedBan(from)` stores `from.ToString()` (`0x.. "Name"`), not the bare name.
    #[test]
    fn marked_ban_stores_the_mobile_string_as_ban_dealer() {
        let mut w = World::new();
        let gm = w.new_mobile(&MOBILE_BASE);
        w.mobile_mut(gm).unwrap().name = Some("Boss".into());
        let a = accounting::create_account(&mut w, "victim", "pw");
        let expected = mobile_string(&w, gm);
        assert!(expected.contains("\"Boss\""));
        let who = mobile_string(&w, gm);
        accounting::account_mut(&mut w, a)
            .unwrap()
            .set_unspecified_ban(Some(&who));
        assert_eq!(
            accounting::account(&w, a).unwrap().get_tag("BanDealer"),
            Some(expected.as_str())
        );
    }
}
