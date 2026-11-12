//! `AquariumGump` and the house raffle gumps: `HouseRaffleManagementGump`, the writ of lease gump (of
//! `HouseRaffleDeed`) and the ticket confirmation (of `HouseRaffleStone`).
//! The items are `muo_items::special::{aquarium, raffle}`; `special_wiring` sets the hooks that open these.
//!
//! A gump that re-sends itself (`sender.Mobile.SendGump(this)`) is rebuilt here as a new object of the same class
//! with the state it kept (the layout is built from the world on every send).

use std::cmp::Ordering;
use std::net::IpAddr;
use std::rc::Rc;

use muo_content::gumps::GumpButtonType;
use muo_content::gumps::base::{DynamicGumpLogic, DynamicGumpObj, RelayInfo};
use muo_content::gumps::builder::DynamicGumpBuilder;
use muo_content::gumps::{close_gump, send_gump_to_mobile};
use muo_gumps::m_z::warning::{StaticWarning, new_static};
use muo_items::special::{aquarium, fish, raffle};
use muo_primitives::dotnet::{cmp_ignore_case, net_sort};
use muo_primitives::html::{center_rgb, color_rgb};
use muo_primitives::net::to_u128;
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::net_state::NetStateId;

fn mobile_of(w: &World, ns: NetStateId) -> Option<MobileId> {
    w.net_states.get(&ns).and_then(|n| n.mobile)
}

fn send(w: &mut World, to: MobileId, gump: impl muo_content::gumps::Gump + 'static) {
    send_gump_to_mobile(w, to, Box::new(gump), false);
}

fn has_net_state(w: &World, m: MobileId) -> bool {
    w.mobile(m).and_then(|m| m.net_state).is_some()
}

// ---- AquariumGump ----

struct Page {
    label: i32,
    description: i32,
    item_id: i32,
    hue: i32,
}

pub struct AquariumGump {
    aquarium: ItemId,
    edit: bool,
    pages: Vec<Page>,
}

fn new_aquarium_gump(w: &World, aq: ItemId, edit: bool) -> DynamicGumpObj<AquariumGump> {
    let pages = w
        .item(aq)
        .map(|a| a.items.clone())
        .unwrap_or_default()
        .into_iter()
        .filter_map(|i| {
            let it = w.item(i)?;
            let is_fish = it.ty.is(&fish::BASE_FISH);
            Some(Page {
                label: (it.ty.label_number)(w, i),
                description: if is_fish {
                    fish::description(w, i)
                } else {
                    1073634
                }, // An aquarium decoration
                item_id: (it.ty.item_id)(w, i),
                hue: (it.ty.hue)(w, i),
            })
        })
        .collect();
    DynamicGumpObj::new(
        100,
        100,
        AquariumGump {
            aquarium: aq,
            edit,
            pages,
        },
    )
}

/// `AquariumGump.DisplayTo(from, aquarium)`.
pub fn display_aquarium(w: &mut World, from: MobileId, aq: ItemId) {
    if !has_net_state(w, from) || w.item(aq).is_none_or(|i| i.deleted()) {
        return;
    }
    let edit = aquarium::has_access(w, aq, from);
    let g = new_aquarium_gump(w, aq, edit);
    send(w, from, g);
}

/// `from.CloseGump<AquariumGump>()`.
pub fn close_aquarium(w: &mut World, from: MobileId) {
    if let Some(ns) = w.mobile(from).and_then(|m| m.net_state) {
        close_gump::<AquariumGump>(w, ns);
    }
}

impl DynamicGumpLogic for AquariumGump {
    const TYPE_NAME: &'static str = "Server.Items.AquariumGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.add_page(0);
        b.add_background(0, 0, 350, 323, 0xE10);
        b.add_image(0, 0, 0x2C96, 0);
        let count = self.pages.len() as i32;
        for (idx, p) in self.pages.iter().enumerate() {
            let page = idx as i32 + 1;
            b.add_page(page);
            // item name
            if p.label != 0 {
                b.add_html_localized_color(20, 217, 250, 20, p.label, 0x7FFF, false, false); // Name
            }
            // item details
            b.add_html_localized_color(20, 239, 315, 20, p.description, 0x7FFF, false, false);
            // item image
            b.add_item(150, 80, p.item_id, p.hue);
            // item number / all items
            b.add_html(
                20,
                195,
                250,
                20,
                &color_rgb(&format!("{page}/{count}"), 0xFFFFFF),
                false,
                false,
            );
            // remove item
            if self.edit {
                b.add_background(230, 195, 100, 26, 0x13BE);
                b.add_button(235, 200, 0x845, 0x846, page);
                b.add_html_localized_color(260, 198, 60, 26, 1073838, 0x0, false, false); // Remove
            }
            // next page
            if page < count {
                b.add_button_type(195, 280, 0xFA5, 0xFA7, 0, GumpButtonType::Page, page + 1);
                b.add_html_localized_color(230, 283, 100, 18, 1044045, 0x7FFF, false, false); // NEXT PAGE
            }
            // previous page
            if page > 1 {
                b.add_button_type(45, 280, 0xFAE, 0xFAF, 0, GumpButtonType::Page, page - 1);
                b.add_html_localized_color(80, 283, 100, 18, 1044044, 0x7FFF, false, false); // PREV PAGE
            }
        }
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        if w.item(self.aquarium).is_none_or(|i| i.deleted()) {
            return;
        }
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        let edit = aquarium::has_access(w, self.aquarium, from);
        let count = w.item(self.aquarium).map_or(0, |i| i.items.len() as i32);
        if info.button_id > 0 && info.button_id <= count && edit {
            aquarium::remove_item(w, self.aquarium, from, info.button_id - 1);
        }
        if info.button_id > 0 {
            let g = new_aquarium_gump(w, self.aquarium, self.edit);
            send(w, from, g);
        }
    }
}

// ---- the writ of lease ----

pub struct WritOfLeaseGump {
    text: String,
}

/// `WritOfLeaseGump.DisplayTo(from, deed)`.
pub fn display_writ(w: &mut World, from: MobileId, deed: ItemId) {
    if !has_net_state(w, from) || w.item(deed).is_none_or(|i| i.deleted()) {
        return;
    }
    let text = raffle::writ_description(w, deed);
    send(
        w,
        from,
        DynamicGumpObj::new(150, 50, WritOfLeaseGump { text }),
    );
}

impl DynamicGumpLogic for WritOfLeaseGump {
    const TYPE_NAME: &'static str = "Server.Items.HouseRaffleDeed+WritOfLeaseGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.add_page(0);
        for (x, y, id) in [
            (0, 0, 9380),
            (114, 0, 9381),
            (171, 0, 9382),
            (0, 140, 9383),
            (114, 140, 9384),
            (171, 140, 9385),
            (0, 182, 9383),
            (114, 182, 9384),
            (171, 182, 9385),
            (0, 224, 9383),
            (114, 224, 9384),
            (171, 224, 9385),
            (0, 266, 9386),
            (114, 266, 9387),
            (171, 266, 9388),
        ] {
            b.add_image(x, y, id, 0);
        }
        b.add_html_localized_color(30, 48, 229, 20, 1150484, 200, false, false); // WRIT OF LEASE
        b.add_html(28, 75, 231, 280, &self.text, false, true);
    }
}

// ---- the ticket confirmation ----

struct ConfirmTicketPurchase {
    content: String,
}

impl StaticWarning for ConfirmTicketPurchase {
    const TYPE_NAME: &'static str = "Server.Items.HouseRaffleStone+ConfirmTicketPurchaseGump";
    const WIDTH: i32 = 420;
    const HEIGHT: i32 = 280;

    fn header(&self) -> i32 {
        1150470 // CONFIRM TICKET PURCHASE
    }
    fn header_color(&self) -> i32 {
        0x7F00
    }
    fn content_color(&self) -> &str {
        "#FFFFFF"
    }
    fn content(&self) -> Option<String> {
        Some(self.content.clone())
    }
}

/// `from.SendGump(new ConfirmTicketPurchaseGump(bounds, center, facet, price, okay => Purchase_Callback(from, okay)))`.
pub fn display_ticket_gump(w: &mut World, from: MobileId, stone: ItemId) {
    let Some(d) = raffle::stone(w, stone).cloned() else {
        return;
    };
    let center = raffle::plot_center(w, stone);
    let content = format!(
        "You are about to purchase a raffle ticket for the house plot located at {}.  The ticket price is {}.  Tickets are non-refundable and you can only purchase one ticket per account.  Do you wish to continue?",
        raffle::format_location_bounds(w, d.plot_bounds, center, d.plot_facet),
        raffle::format_price(d.ticket_price)
    );
    let cb: muo_gumps::m_z::warning::WarningCallback =
        Rc::new(move |w, okay| raffle::purchase_callback(w, stone, from, okay));
    send(
        w,
        from,
        new_static(ConfirmTicketPurchase { content }, Some(cb)),
    );
}

// ---- HouseRaffleManagementGump ----

const LABEL_COLOR: i32 = 0xFFFFFF;
const HIGHLIGHT_COLOR: i32 = 0x11EE11;
const ROWS_PER_PAGE: usize = 10;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum SortMethod {
    Default,
    Name,
    Account,
    Address,
}

/// Buttons 0..=6; the per-row delete buttons start at 7.
const B_CLOSE: i32 = 0;
const B_PREV: i32 = 1;
const B_NEXT: i32 = 2;
const B_BY_NAME: i32 = 3;
const B_BY_ACCOUNT: i32 = 4;
const B_BY_ADDRESS: i32 = 5;
const B_REFRESH: i32 = 6;
const B_DELETE: i32 = 7;

struct Row {
    text: String,
    address: String,
    date: String,
    color: i32,
}

pub struct HouseRaffleManagementGump {
    stone: ItemId,
    // A snapshot of the stone's entries (page navigation, sort and delete work on it so the indices stay stable);
    // Refresh resyncs it. `None` is a cleared slot.
    entries: Vec<Option<raffle::RaffleEntry>>,
    sort: SortMethod,
    page: usize,
    // What `BuildLayout` reads from the live stone, taken when the gump is (re)sent.
    location: String,
    price: String,
    total: usize,
    rows: Vec<Row>,
}

fn username_of(w: &World, m: MobileId) -> Option<String> {
    let a = w.mobile(m)?.account?;
    muo_content::accounting::account(w, a).map(|a| a.username.clone())
}

fn raw_name(w: &World, m: MobileId) -> String {
    muo_server::mobile::part1::props_b::raw_name(w, m).unwrap_or_default()
}

fn sort_snapshot(w: &World, entries: &mut [Option<raffle::RaffleEntry>], sort: SortMethod) {
    if sort == SortMethod::Default || entries.len() <= 1 {
        return;
    }
    // A null (or From-less) entry sorts first.
    fn cmp_null<T>(x: &Option<T>, y: &Option<T>) -> Option<Ordering> {
        match (x, y) {
            (None, None) => Some(Ordering::Equal),
            (None, _) => Some(Ordering::Less),
            (_, None) => Some(Ordering::Greater),
            _ => None,
        }
    }
    net_sort(entries, |x, y| {
        let from = |e: &Option<raffle::RaffleEntry>| e.as_ref().and_then(|e| e.from);
        let (xf, yf) = (from(x), from(y));
        let date = |x: &Option<raffle::RaffleEntry>, y: &Option<raffle::RaffleEntry>| {
            x.as_ref().map(|e| e.date).cmp(&y.as_ref().map(|e| e.date))
        };
        match sort {
            SortMethod::Name => {
                if let Some(o) = cmp_null(&xf, &yf) {
                    return o;
                }
                let (Some(xf), Some(yf)) = (xf, yf) else {
                    return Ordering::Equal;
                };
                let r = cmp_ignore_case(&raw_name(w, xf), &raw_name(w, yf));
                if r == Ordering::Equal { date(x, y) } else { r }
            }
            SortMethod::Account => {
                if let Some(o) = cmp_null(&xf, &yf) {
                    return o;
                }
                let (Some(xf), Some(yf)) = (xf, yf) else {
                    return Ordering::Equal;
                };
                let (a, b) = (username_of(w, xf), username_of(w, yf));
                if let Some(o) = cmp_null(&a, &b) {
                    return o;
                }
                let r = cmp_ignore_case(&a.unwrap_or_default(), &b.unwrap_or_default());
                if r == Ordering::Equal { date(x, y) } else { r }
            }
            SortMethod::Address | SortMethod::Default => {
                let r = to_u128(x.as_ref().map_or(IpAddr::from([0, 0, 0, 0]), |e| e.address)).cmp(
                    &to_u128(y.as_ref().map_or(IpAddr::from([0, 0, 0, 0]), |e| e.address)),
                );
                if r != Ordering::Equal { r } else { date(x, y) }
            }
        }
    });
}

fn snapshot_of(w: &World, stone: ItemId) -> Vec<Option<raffle::RaffleEntry>> {
    raffle::stone(w, stone)
        .map(|d| d.entries.iter().cloned().map(Some).collect())
        .unwrap_or_default()
}

fn build_gump(
    w: &mut World,
    stone: ItemId,
    entries: Vec<Option<raffle::RaffleEntry>>,
    sort: SortMethod,
    page: usize,
) -> DynamicGumpObj<HouseRaffleManagementGump> {
    let Some(d) = raffle::stone(w, stone).cloned() else {
        return DynamicGumpObj::new(
            40,
            40,
            HouseRaffleManagementGump {
                stone,
                entries,
                sort,
                page,
                location: String::new(),
                price: String::new(),
                total: 0,
                rows: Vec::new(),
            },
        );
    };
    let center = raffle::plot_center(w, stone);
    let location = raffle::format_location_bounds(w, d.plot_bounds, center, d.plot_facet);
    let start = page * ROWS_PER_PAGE;
    let end = (start + ROWS_PER_PAGE).min(entries.len());
    let mut rows = Vec::new();
    for e in entries.get(start..end).unwrap_or_default() {
        let Some(e) = e else {
            rows.push(Row {
                text: String::new(),
                address: String::new(),
                date: String::new(),
                color: LABEL_COLOR,
            });
            continue;
        };
        let color = if d.winner.is_some() && e.from == d.winner {
            HIGHLIGHT_COLOR
        } else {
            LABEL_COLOR
        };
        let text = match e.from {
            Some(f) => match username_of(w, f) {
                Some(acc) => color_rgb(&format!("{} ({acc})", raw_name(w, f)), color),
                None => color_rgb(&raw_name(w, f), color),
            },
            None => String::new(),
        };
        rows.push(Row {
            text,
            address: center_rgb(&e.address.to_string(), color),
            date: center_rgb(&raffle::date_time_string(e.date), color),
            color,
        });
    }
    DynamicGumpObj::new(
        40,
        40,
        HouseRaffleManagementGump {
            stone,
            entries,
            sort,
            page,
            location,
            price: raffle::format_price(d.ticket_price),
            total: d.entries.len(),
            rows,
        },
    )
}

/// `HouseRaffleManagementGump.DisplayTo(from, stone, sort, page)`.
pub fn display_management(w: &mut World, from: MobileId, stone: ItemId) {
    if !has_net_state(w, from) || w.item(stone).is_none_or(|i| i.deleted()) {
        return;
    }
    let entries = snapshot_of(w, stone);
    let g = build_gump(w, stone, entries, SortMethod::Default, 0);
    send(w, from, g);
}

impl DynamicGumpLogic for HouseRaffleManagementGump {
    const TYPE_NAME: &'static str = "Server.Gumps.HouseRaffleManagementGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        let count = self.entries.len();
        b.add_page(0);
        b.add_background(0, 0, 618, 354, 9270);
        b.add_alpha_region(10, 10, 598, 334);

        b.add_html(
            10,
            10,
            598,
            20,
            &center_rgb("Raffle Management", LABEL_COLOR),
            false,
            false,
        );

        let label = |b: &mut DynamicGumpBuilder, x, y, w, text: &str| {
            b.add_html(x, y, w, 20, &color_rgb(text, LABEL_COLOR), false, false);
        };
        label(b, 45, 35, 100, "Location:");
        label(b, 145, 35, 250, &self.location);
        label(b, 45, 55, 100, "Ticket Price:");
        label(b, 145, 55, 250, &self.price);
        label(b, 45, 75, 100, "Total Entries:");
        label(b, 145, 75, 250, &self.total.to_string());

        b.add_button(440, 33, 0xFA5, 0xFA7, B_BY_NAME);
        label(b, 474, 35, 120, "Sort by name");
        b.add_button(440, 53, 0xFA5, 0xFA7, B_BY_ACCOUNT);
        label(b, 474, 55, 120, "Sort by account");
        b.add_button(440, 73, 0xFA5, 0xFA7, B_BY_ADDRESS);
        label(b, 474, 75, 120, "Sort by address");

        b.add_image_tiled(13, 99, 592, 242, 9264);
        b.add_image_tiled(14, 100, 590, 240, 9274);
        b.add_alpha_region(14, 100, 590, 240);

        b.add_html(
            14,
            100,
            590,
            20,
            &center_rgb("Entries", LABEL_COLOR),
            false,
            false,
        );

        b.add_button(545, 104, 0x845, 0x846, B_REFRESH);

        if self.page > 0 {
            b.add_button(567, 104, 0x15E3, 0x15E7, B_PREV);
        } else {
            b.add_image(567, 104, 0x25EA, 0);
        }
        if (self.page + 1) * ROWS_PER_PAGE < count {
            b.add_button(584, 104, 0x15E1, 0x15E5, B_NEXT);
        } else {
            b.add_image(584, 104, 0x25E6, 0);
        }

        b.add_html(
            14,
            120,
            30,
            20,
            &center_rgb("DEL", LABEL_COLOR),
            false,
            false,
        );
        label(b, 47, 120, 250, "Name");
        b.add_html(
            295,
            120,
            100,
            20,
            &center_rgb("Address", LABEL_COLOR),
            false,
            false,
        );
        b.add_html(
            395,
            120,
            150,
            20,
            &center_rgb("Date", LABEL_COLOR),
            false,
            false,
        );
        b.add_html(
            545,
            120,
            60,
            20,
            &center_rgb("Num", LABEL_COLOR),
            false,
            false,
        );

        let page_start = self.page * ROWS_PER_PAGE;
        for (idx, row) in self.rows.iter().enumerate() {
            let i = page_start + idx;
            if self.entries.get(i).is_none_or(|e| e.is_none()) {
                continue;
            }
            let y = idx as i32 * 20;
            b.add_button(13, 138 + y, 4002, 4004, B_DELETE + i as i32);
            let mut x = 45;
            if !row.text.is_empty() {
                b.add_html(x + 2, 140 + y, 250, 20, &row.text, false, false);
            }
            x += 250;
            b.add_html(x, 140 + y, 100, 20, &row.address, false, false);
            x += 100;
            b.add_html(x, 140 + y, 150, 20, &row.date, false, false);
            x += 150;
            b.add_html(
                x,
                140 + y,
                60,
                20,
                &center_rgb("1", row.color),
                false,
                false,
            );
        }
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        if w.item(self.stone).is_none_or(|i| i.deleted()) {
            return;
        }
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        let mut entries = std::mem::take(&mut self.entries);
        let mut page = self.page;
        let mut sort = self.sort;
        match info.button_id {
            B_CLOSE => return,
            B_PREV => page = page.saturating_sub(1),
            B_NEXT => {
                if (page + 1) * ROWS_PER_PAGE < entries.len() {
                    page += 1;
                }
            }
            B_BY_NAME | B_BY_ACCOUNT | B_BY_ADDRESS => {
                sort = match info.button_id {
                    B_BY_NAME => SortMethod::Name,
                    B_BY_ACCOUNT => SortMethod::Account,
                    _ => SortMethod::Address,
                };
                page = 0;
                sort_snapshot(w, &mut entries, sort);
            }
            B_REFRESH => {
                entries = snapshot_of(w, self.stone);
                sort_snapshot(w, &mut entries, sort);
                let max_page = if entries.is_empty() {
                    0
                } else {
                    (entries.len() - 1) / ROWS_PER_PAGE
                };
                page = page.min(max_page);
            }
            id => {
                let delete_index = id - B_DELETE;
                if delete_index < 0 || delete_index as usize >= entries.len() {
                    self.entries = entries;
                    return;
                }
                let at = delete_index as usize;
                if let Some(target) = entries[at].clone() {
                    raffle::remove_entry(w, self.stone, &target);
                }
                // Compact the snapshot in place: it stays sorted.
                entries.remove(at);
                if page > 0 && page * ROWS_PER_PAGE >= entries.len() {
                    page -= 1;
                }
            }
        }
        let g = build_gump(w, self.stone, entries, sort, page);
        send(w, from, g);
    }
}

/// Opens the properties gump of `entity` for `from` (`new PropertiesGump(from, entity)`).
pub fn open_properties(w: &mut World, from: MobileId, entity: muo_server::ids::EntityId) {
    use muo_gumps::m_z::props::{PropObj, properties};
    let g = properties::new(w, from, PropObj::Entity(entity));
    send(w, from, g);
}
