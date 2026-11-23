//! `HouseRaffleStone`, `HouseRaffleDeed` and `HouseRaffleRegion` (with `RaffleEntry`).
//! The gumps (HouseRaffleManagementGump, the writ of lease and the ticket confirmation) are
//! `muo_engines::special_gumps`, reached through [`super::hooks::SpecialHooks`].

use std::net::{IpAddr, Ipv4Addr};

use indexmap::IndexSet;
use muo_content::items::gold::GOLD;
use muo_content::regions::{BASE_REGION, RegionData};
use muo_content::vendor::banker;
use muo_io::{GenericReader, Result};
use muo_primitives::{DateTime, Point3D, Rectangle2D, TimeSpan, net::match_class_c, utility};
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, LootType, container, deps, ops};
use muo_server::map::MapId;
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part2::{items as mobile_items, messages as mobile_messages, stats};
use muo_server::property_list::PropertyList;
use muo_server::region::{self, RegionId, RegionType};
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::target::{TargetId, Targeted};
use muo_server::timer::{self, TimerHandle};
use muo_server::type_registry::TypeRegistry;

use super::hooks::hooks;
use crate::skill_items::runebook::sextant_format;

const ENTRY_LIMIT_PER_IP: i32 = 4;
const DEFAULT_TICKET_PRICE: i32 = 5000;
const MESSAGE_HUE: i32 = 1153;
/// `HouseRaffleStone.DefaultDuration`.
pub const DEFAULT_DURATION: TimeSpan = TimeSpan::from_days(7);
/// `HouseRaffleStone.ExpirationTime`: how long the winner has to use the deed.
pub const EXPIRATION_TIME: TimeSpan = TimeSpan::from_days(30);

/// `HouseRaffleState`.
pub const INACTIVE: i32 = 0;
pub const ACTIVE: i32 = 1;
pub const COMPLETED: i32 = 2;

/// `HouseRaffleExpireAction`.
pub const EXPIRE_NONE: i32 = 0;
pub const EXPIRE_HIDE_STONE: i32 = 1;
pub const EXPIRE_DELETE_STONE: i32 = 2;

/// `RaffleEntry`.
#[derive(Clone, Debug, PartialEq)]
pub struct RaffleEntry {
    pub from: Option<MobileId>,
    pub address: IpAddr,
    pub date: DateTime,
}

/// What `HouseRaffleStone` adds to an `Item`.
#[derive(Clone, Debug)]
pub struct StoneData {
    pub current_state: i32,
    pub expire_action: i32,
    pub deed: Option<ItemId>,
    pub plot_bounds: Rectangle2D,
    pub plot_facet: Option<MapId>,
    pub winner: Option<MobileId>,
    pub ticket_price: i32,
    pub started: DateTime,
    pub duration: TimeSpan,
    pub entries: Vec<RaffleEntry>,
    /// `_region` (not serialized).
    pub region: Option<RegionId>,
}

impl Default for StoneData {
    fn default() -> Self {
        StoneData {
            current_state: INACTIVE,
            expire_action: EXPIRE_NONE,
            deed: None,
            plot_bounds: Rectangle2D::EMPTY,
            plot_facet: None,
            winner: None,
            ticket_price: DEFAULT_TICKET_PRICE,
            started: DateTime::MIN,
            duration: DEFAULT_DURATION,
            entries: Vec::new(),
            region: None,
        }
    }
}

/// What `HouseRaffleDeed` adds to an `Item`.
#[derive(Clone, Debug, Default)]
pub struct DeedData {
    pub stone: Option<ItemId>,
    pub plot_location: Point3D,
    pub plot_facet: Option<MapId>,
    pub awarded_to: Option<MobileId>,
}

/// `_allStones` and `_allStonesTimer`.
#[derive(Default)]
struct AllStones {
    stones: IndexSet<ItemId>,
    timer: Option<TimerHandle>,
}

pub fn stone(w: &World, id: ItemId) -> Option<&StoneData> {
    w.item(id)?.ext.get::<StoneData>()
}

fn stone_mut(w: &mut World, id: ItemId) -> Option<&mut StoneData> {
    w.item_mut(id)?.ext.get_mut::<StoneData>()
}

pub fn deed(w: &World, id: ItemId) -> Option<&DeedData> {
    w.item(id)?.ext.get::<DeedData>()
}

fn is_deleted(w: &World, id: ItemId) -> bool {
    w.item(id).is_none_or(|i| i.deleted())
}

/// `Map.ToString()`: the name, empty for null.
pub fn map_name(w: &World, map: Option<MapId>) -> String {
    map.and_then(|m| w.maps.get(m))
        .map_or_else(String::new, |m| m.name().to_owned())
}

/// `DateTime.ToString()` under the invariant culture: `MM/dd/yyyy HH:mm:ss`.
pub fn date_time_string(d: DateTime) -> String {
    let (y, m, day) = d.ymd();
    format!(
        "{m:02}/{day:02}/{y:04} {:02}:{:02}:{:02}",
        d.hour(),
        d.minute(),
        d.second()
    )
}

// ---- formatting ----

/// `ValidLocation(bounds, map)`.
pub fn valid_location(bounds: Rectangle2D, map: Option<MapId>) -> bool {
    bounds.start != muo_primitives::Point2D::ZERO
        && bounds.end != muo_primitives::Point2D::ZERO
        && map.is_some_and(|m| m != MapId::INTERNAL)
}

/// `FormatLocation(loc, map, displayMap)`.
pub fn format_location(w: &World, loc: Point3D, map: Option<MapId>, display_map: bool) -> String {
    let mut s = match sextant_format(w, loc, map) {
        Some(v) => format!(
            "{}\u{b0}{}'{},{}\u{b0}{}'{}",
            v.y_lat,
            v.y_mins,
            if v.y_south { "S" } else { "N" },
            v.x_long,
            v.x_mins,
            if v.x_east { "E" } else { "W" }
        ),
        None => format!("{},{}", loc.x, loc.y),
    };
    if display_map {
        s.push_str(&format!(" ({})", map_name(w, map)));
    }
    s
}

/// `FormatLocation(bounds, center, map)`.
pub fn format_location_bounds(
    w: &World,
    bounds: Rectangle2D,
    center: Point3D,
    map: Option<MapId>,
) -> String {
    if !valid_location(bounds, map) {
        "no location set".to_owned()
    } else {
        format_location(w, center, map, true)
    }
}

/// `FormatPrice(price)`.
pub fn format_price(price: i32) -> String {
    if price == 0 {
        "FREE".to_owned()
    } else {
        format!("{price} gold")
    }
}

/// `GetPlotCenter()`.
pub fn plot_center(w: &mut World, id: ItemId) -> Point3D {
    let Some(d) = stone(w, id) else {
        return Point3D::ZERO;
    };
    let (b, map) = (d.plot_bounds, d.plot_facet);
    let x = b.x() + b.width() / 2;
    let y = b.y() + b.height() / 2;
    let z = map.map_or(0, |m| w.maps.get_average_z(m, x, y));
    Point3D::new(x, y, z)
}

/// `IsExpired`.
pub fn is_expired(w: &World, id: ItemId) -> bool {
    let Some(d) = stone(w, id) else { return false };
    d.current_state == COMPLETED && w.now > d.started + d.duration + EXPIRATION_TIME
}

// ---- the running raffles ----

fn check_end_tick(w: &mut World) {
    let stones: Vec<ItemId> = w
        .resources
        .get_mut::<AllStones>()
        .stones
        .iter()
        .copied()
        .collect();
    for s in stones {
        check_end(w, s);
    }
}

fn add_raffle_stone(w: &mut World, id: ItemId) {
    let all = w.resources.get_mut::<AllStones>();
    if all.stones.insert(id) && all.stones.len() == 1 {
        let h = timer::start(
            w,
            TimeSpan::from_mins(1),
            TimeSpan::from_mins(1),
            0,
            check_end_tick,
        );
        w.resources.get_mut::<AllStones>().timer = Some(h);
    }
}

fn remove_raffle_stone(w: &mut World, id: ItemId) {
    let all = w.resources.get_mut::<AllStones>();
    if all.stones.shift_remove(&id)
        && all.stones.is_empty()
        && let Some(t) = all.timer.take()
    {
        timer::stop(w, t);
    }
}

// ---- setters ----

/// `CurrentState` setter.
pub fn set_state(w: &mut World, id: ItemId, value: i32) {
    let Some(cur) = stone(w, id).map(|d| d.current_state) else {
        return;
    };
    if cur == value {
        return;
    }
    if value == ACTIVE {
        if let Some(d) = stone_mut(w, id) {
            d.entries.clear();
        }
        set_winner(w, id, None);
        set_deed(w, id, None);
        set_started(w, id, w.now);
        add_raffle_stone(w, id);
    } else {
        remove_raffle_stone(w, id);
    }
    if let Some(d) = stone_mut(w, id) {
        d.current_state = value;
    }
    ops::invalidate_properties(w, id);
}

pub fn set_deed(w: &mut World, id: ItemId, deed: Option<ItemId>) {
    if let Some(d) = stone_mut(w, id) {
        d.deed = deed;
    }
}

pub fn set_winner(w: &mut World, id: ItemId, winner: Option<MobileId>) {
    match stone_mut(w, id) {
        Some(d) if d.winner != winner => d.winner = winner,
        _ => return,
    }
    ops::invalidate_properties(w, id);
}

pub fn set_started(w: &mut World, id: ItemId, started: DateTime) {
    match stone_mut(w, id) {
        Some(d) if d.started != started => d.started = started,
        _ => return,
    }
    ops::invalidate_properties(w, id);
}

/// `PlotBounds` setter.
pub fn set_plot_bounds(w: &mut World, id: ItemId, value: Rectangle2D) {
    match stone_mut(w, id) {
        Some(d) if d.plot_bounds != value => d.plot_bounds = value,
        _ => return,
    }
    ops::invalidate_properties(w, id);
    invalidate_region(w, id);
}

/// `PlotFacet` setter.
pub fn set_plot_facet(w: &mut World, id: ItemId, value: Option<MapId>) {
    match stone_mut(w, id) {
        Some(d) if d.plot_facet != value => d.plot_facet = value,
        _ => return,
    }
    ops::invalidate_properties(w, id);
    invalidate_region(w, id);
}

/// `TicketPrice` setter (`Math.Max(0, value)`).
pub fn set_ticket_price(w: &mut World, id: ItemId, value: i32) {
    let value = value.max(0);
    match stone_mut(w, id) {
        Some(d) if d.ticket_price != value => d.ticket_price = value,
        _ => return,
    }
    ops::invalidate_properties(w, id);
}

/// `Entries.Remove(entry)`: the first equal entry.
pub fn remove_entry(w: &mut World, id: ItemId, e: &RaffleEntry) {
    if let Some(d) = stone_mut(w, id)
        && let Some(p) = d.entries.iter().position(|x| x == e)
    {
        d.entries.remove(p);
    }
}

/// `InvalidateRegion()`.
fn invalidate_region(w: &mut World, id: ItemId) {
    if let Some(r) = stone_mut(w, id).and_then(|d| d.region.take()) {
        region::unregister(w, r);
    }
    let Some(d) = stone(w, id) else { return };
    let (bounds, map) = (d.plot_bounds, d.plot_facet);
    if !valid_location(bounds, map) {
        return;
    }
    let Some(map) = map else { return };
    let r = region::new_region(
        w,
        &HOUSE_RAFFLE_REGION,
        None,
        map,
        None,
        Some(region::DEFAULT_PRIORITY),
        region::convert_all_to_3d(&[bounds]),
    );
    if let Some(reg) = w.region_mut(r) {
        reg.ext = Ext::new(RegionData::default());
        reg.ext.insert(RaffleRegionStone(id));
    }
    region::register(w, r);
    if let Some(d) = stone_mut(w, id) {
        d.region = Some(r);
    }
}

// ---- entering ----

fn account_of(w: &World, m: MobileId) -> Option<muo_server::iaccount::AccountId> {
    w.mobile(m)?.account
}

fn address_of(w: &World, m: MobileId) -> Option<IpAddr> {
    let ns = w.mobile(m)?.net_state?;
    w.net_states.get(&ns)?.address
}

/// `HasEntered(from)`.
fn has_entered(w: &World, id: ItemId, from: MobileId) -> bool {
    let Some(acc) = account_of(w, from) else {
        return false;
    };
    stone(w, id).is_some_and(|d| {
        d.entries
            .iter()
            .any(|e| e.from.is_some_and(|f| account_of(w, f) == Some(acc)))
    })
}

/// `IsAtIPLimit(from)`.
fn is_at_ip_limit(w: &World, id: ItemId, from: MobileId) -> bool {
    let Some(ns) = w.mobile(from).and_then(|m| m.net_state) else {
        return false;
    };
    let Some(address) = w.net_states.get(&ns).and_then(|n| n.address) else {
        return false;
    };
    let mut tickets = 0;
    for e in stone(w, id)
        .map(|d| d.entries.as_slice())
        .unwrap_or_default()
    {
        if match_class_c(e.address, address) {
            tickets += 1;
            if tickets >= ENTRY_LIMIT_PER_IP {
                return true;
            }
        }
    }
    false
}

fn send_msg(w: &mut World, to: MobileId, text: &str) {
    deps::send_message_hue(w, to, MESSAGE_HUE, text);
}

/// `Purchase_Callback(from, okay)`.
pub fn purchase_callback(w: &mut World, id: ItemId, from: MobileId, okay: bool) {
    if is_deleted(w, id)
        || stone(w, id).is_none_or(|d| d.current_state != ACTIVE)
        || !stats::check_alive(w, from, true)
        || has_entered(w, id, from)
        || is_at_ip_limit(w, id, from)
    {
        return;
    }
    if account_of(w, from).is_none() {
        return;
    }
    if okay {
        let price = stone(w, id).map_or(0, |d| d.ticket_price);
        let pack = w.mobile(from).and_then(|m| m.backpack);
        let paid = price == 0
            || pack.is_some_and(|p| container::consume_total(w, p, &GOLD, price, true, None))
            || banker::withdraw(w, from, price);
        if paid {
            let entry = RaffleEntry {
                from: Some(from),
                address: address_of(w, from).unwrap_or(IpAddr::V4(Ipv4Addr::BROADCAST)),
                date: w.now,
            };
            if let Some(d) = stone_mut(w, id) {
                d.entries.push(entry);
            }
            send_msg(w, from, "You have successfully entered the plot's raffle.");
        } else {
            send_msg(
                w,
                from,
                &format!(
                    "You do not have the {} required to enter the raffle.",
                    format_price(price)
                ),
            );
        }
    } else {
        send_msg(w, from, "You have chosen not to enter the raffle.");
    }
}

/// `CheckEnd()`.
pub fn check_end(w: &mut World, id: ItemId) {
    let Some(d) = stone(w, id) else { return };
    if d.current_state != ACTIVE || w.now < d.started + d.duration {
        return;
    }
    set_state(w, id, COMPLETED);

    let Some(d) = stone(w, id) else { return };
    if d.region.is_some() && !d.entries.is_empty() {
        let entries = d.entries.clone();
        let pick = utility::random_element(&mut w.rng, &entries).and_then(|e| e.from);
        set_winner(w, id, pick);
        if let Some(winner) = pick {
            let new_deed = deed_new_with(w, Some(id), Some(winner));
            set_deed(w, id, Some(new_deed));
            let name = w
                .mobile(winner)
                .and_then(|m| (m.ty.name)(w, winner))
                .unwrap_or_default();
            let center = plot_center(w, id);
            let (bounds, facet) = stone(w, id).map_or((Rectangle2D::EMPTY, None), |d| {
                (d.plot_bounds, d.plot_facet)
            });
            let at = format_location_bounds(w, bounds, center, facet);
            send_msg(
                w,
                winner,
                &format!(
                    "Congratulations, {name}! You have won the raffle for the plot located at {at}."
                ),
            );
            // AddToBackpack: the pack, else the bank box.
            let has_pack = w.mobile(winner).and_then(|m| m.backpack).is_some();
            let placed = has_pack && mobile_items::place_in_backpack(w, winner, new_deed);
            if placed {
                send_msg(
                    w,
                    winner,
                    "The writ of lease has been placed in your backpack.",
                );
            } else {
                if let Some(bank) = muo_server::mobile::part1::props_b::bank_box(w, winner)
                    && let Some(ty) = w.item(bank).map(|i| i.ty)
                {
                    (ty.drop_item)(w, bank, new_deed);
                }
                send_msg(
                    w,
                    winner,
                    "As your backpack is full, the writ of lease has been placed in your bank box.",
                );
            }
        }
    }
    ops::invalidate_properties(w, id);
}

// ---- HouseRaffleRegion ----

/// The stone a `HouseRaffleRegion` belongs to.
struct RaffleRegionStone(ItemId);

fn region_stone(w: &World, id: RegionId) -> Option<ItemId> {
    w.region(id)?.ext.get::<RaffleRegionStone>().map(|s| s.0)
}

fn contains_deed(w: &World, cont: ItemId, stone_deed: ItemId) -> bool {
    muo_server::item::enumerable::find_items_by_type(w, cont, true, &HOUSE_RAFFLE_DEED, |_, _| true)
        .into_iter()
        .any(|d| d == stone_deed)
}

/// `Server.Regions.HouseRaffleRegion`.
pub static HOUSE_RAFFLE_REGION: RegionType = RegionType {
    name: "HouseRaffleRegion",
    parent: Some(&BASE_REGION),
    allow_housing: |w, id, from, _p| {
        let Some(s) = region_stone(w, id) else {
            return false;
        };
        if is_expired(w, s) {
            return true;
        }
        let Some(deed) = stone(w, s).and_then(|d| d.deed) else {
            return false;
        };
        if let Some(pack) = w.mobile(from).and_then(|m| m.backpack)
            && contains_deed(w, pack, deed)
        {
            return true;
        }
        // FindBankNoCreate
        let bank = mobile_items::find_item_on_layer(w, from, muo_server::layer::Layer::Bank)
            .filter(|&b| muo_server::mobile::part2::deps::item_is_bank_box(w, b));
        bank.is_some_and(|b| contains_deed(w, b, deed))
    },
    on_target: |w, id, m, t: TargetId, o: &Targeted| {
        let is_mark = w
            .mobile(m)
            .and_then(|x| x.spell)
            .and_then(|s| muo_content::spells::spell::spell_type(w, s))
            .is_some_and(|t| t.full_name == "Server.Spells.Sixth.MarkSpell");
        if is_mark
            && w.mobile(m)
                .is_some_and(|x| x.access_level == AccessLevel::Player)
        {
            mobile_messages::send_localized_message(w, m, 501800, "", 0x3B2); // You cannot mark an object at that location.
            return false;
        }
        (BASE_REGION.on_target)(w, id, m, t, o)
    },
    ..BASE_REGION
};

// ---- HouseRaffleDeed ----

/// `HouseRaffleDeed.IsExpired`.
pub fn deed_is_expired(w: &World, id: ItemId) -> bool {
    match deed(w, id).and_then(|d| d.stone) {
        Some(s) => is_deleted(w, s) || is_expired(w, s),
        None => true,
    }
}

/// `ValidLocation()`.
pub fn deed_valid_location(w: &World, id: ItemId) -> bool {
    deed(w, id).is_some_and(|d| {
        d.plot_location != Point3D::ZERO && d.plot_facet.is_some_and(|m| m != MapId::INTERNAL)
    })
}

/// `new HouseRaffleDeed(stone, m)`.
pub fn deed_new_with(w: &mut World, stone: Option<ItemId>, m: Option<MobileId>) -> ItemId {
    let id = crate::simple::from_template(w, &HOUSE_RAFFLE_DEED, "Server.Items.HouseRaffleDeed");
    let mut d = DeedData {
        stone,
        awarded_to: m,
        ..DeedData::default()
    };
    if let Some(s) = stone {
        d.plot_location = plot_center(w, s);
        d.plot_facet = self::stone(w, s).and_then(|x| x.plot_facet);
    }
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(d);
    }
    ops::set_loot_type(w, id, LootType::Blessed);
    super::hue(w, id, 0x501);
    id
}

/// The text of the writ of lease gump (`FormatDescription`).
pub fn writ_description(w: &World, id: ItemId) -> String {
    let Some(d) = deed(w, id) else {
        return String::new();
    };
    let at = format_location(w, d.plot_location, d.plot_facet, false);
    let facet = map_name(w, d.plot_facet);
    if deed_is_expired(w, id) {
        return format!(
            "<bodytextblack>This deed once entitled the bearer to build a house on the plot of land located at {at} on the {facet} facet.<br><br>The deed has expired, and now the indicated plot of land is subject to normal house construction rules.<br><br>This deed functions as a recall rune marked for the location of the plot it represents.</bodytextblack>"
        );
    }
    // The stone is set when the deed is not expired.
    let Some(s) = d.stone.and_then(|s| stone(w, s)) else {
        return String::new();
    };
    let left = s.started + s.duration + EXPIRATION_TIME - w.now;
    let days_left = (left.ticks() as f64 / TimeSpan::from_days(1).ticks() as f64).ceil() as i32;
    let plural = if days_left == 1 { "" } else { "s" };
    format!(
        "<bodytextblack>This deed entitles the bearer to build a house on the plot of land located at {at} on the {facet} facet.<br><br>The deed will expire after {days_left} more day{plural} have passed, and at that time the right to place a house reverts to normal house construction rules.<br><br>This deed functions as a recall rune marked for the location of the plot it represents.<br><br>To place a house on the deeded plot, you must simply have this deed in your backpack or bank box when using a House Placement Tool there.</bodytextblack>"
    )
}

/// `Server.Items.HouseRaffleDeed`.
pub static HOUSE_RAFFLE_DEED: ItemType = ItemType {
    full_name: "Server.Items.HouseRaffleDeed",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| deed_new_with(w, None, None)),
    default_name: crate::class_props::default_name,
    default_weight: crate::class_props::default_weight,
    get_properties: |w, id, list: &mut PropertyList| {
        (ITEM_BASE.get_properties)(w, id, list);
        if deed_valid_location(w, id) {
            let (loc, facet) = deed(w, id)
                .map(|d| (d.plot_location, d.plot_facet))
                .unwrap_or_default();
            // ~1_val~: ~2_val~
            list.add_str(
                1060658,
                &format!("location\t{}", format_location(w, loc, facet, false)),
            );
            list.add_str(1060659, &format!("facet\t{}", map_name(w, facet)));
            list.add(1150486); // [Marked Item]
        }
        if deed_is_expired(w, id) {
            list.add(1150487); // [Expired]
        }
    },
    on_double_click: |w, id, from| {
        if !deed_valid_location(w, id) {
            return;
        }
        let pack = w.mobile(from).and_then(|m| m.backpack).map(EntityId::Item);
        if ops::is_child_of(w, id, pack) {
            if let Some(f) = hooks(w).and_then(|h| h.writ_of_lease_gump) {
                f(w, from, id);
            }
        } else {
            deps::send_localized_message(w, from, 1042001); // That must be in your pack for you to use it.
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
        let d = deed(w, id).cloned().unwrap_or_default();
        out.write_item(w, d.stone);
        out.write_point3d(d.plot_location);
        out.write_map(d.plot_facet);
        out.write_mobile(w, d.awarded_to);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let stone = r
            .read_item(w)?
            .filter(|s| w.item(*s).is_some_and(|x| x.ty.is(&HOUSE_RAFFLE_STONE)));
        let plot_location = r.read_point3d()?;
        let plot_facet = r.read_map()?;
        let awarded_to = r.read_mobile(w)?;
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(DeedData {
                stone,
                plot_location,
                plot_facet,
                awarded_to,
            });
        }
        Ok(())
    },
    ..ITEM_BASE
};

// ---- HouseRaffleStone ----

/// `new HouseRaffleStone()`.
pub fn stone_new(w: &mut World) -> ItemId {
    let id = crate::simple::from_template(w, &HOUSE_RAFFLE_STONE, "Server.Items.HouseRaffleStone");
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(StoneData::default());
    }
    ops::set_movable(w, id, false);
    id
}

fn is_seer(w: &World, m: MobileId) -> bool {
    w.mobile(m)
        .is_some_and(|m| m.access_level >= AccessLevel::Seer)
}

fn live_stone(w: &World, s: ItemId) -> bool {
    !is_deleted(w, s) && w.item(s).is_some_and(|i| i.ty.is(&HOUSE_RAFFLE_STONE))
}

fn entry(
    number: i32,
    enabled: bool,
    on_click: impl Fn(&mut World, MobileId, ItemId) + 'static,
) -> muo_server::context_menu_entry::ContextMenuEntry {
    let mut e = muo_server::context_menu_entry::ContextMenuEntry::new(number, -1);
    e.enabled = enabled;
    e.on_click = Some(std::rc::Rc::new(move |w: &mut World, from, target| {
        if let EntityId::Item(s) = target {
            on_click(w, from, s);
        }
    }));
    e
}

fn stone_after_deserialization(w: &mut World, id: ItemId) {
    invalidate_region(w, id);
    // Only check for expirations on world load since the wait time is so long (30 days)
    if is_expired(w, id) {
        match stone(w, id).map_or(EXPIRE_NONE, |d| d.expire_action) {
            EXPIRE_HIDE_STONE => {
                if w.item(id).is_some_and(|i| i.visible()) {
                    ops::set_visible(w, id, false);
                    super::item_id_of(w, id, 0x1B7B); // Non-blocking ItemID
                }
            }
            EXPIRE_DELETE_STONE => {
                timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
                    if let Some(ty) = w.item(id).map(|i| i.ty) {
                        (ty.delete)(w, id);
                    }
                });
            }
            _ => {}
        }
    } else if stone(w, id).is_some_and(|d| d.current_state == ACTIVE) {
        add_raffle_stone(w, id);
    }
}

fn read_entry(r: &mut dyn GenericReader, w: &World) -> Result<RaffleEntry> {
    let _version = r.read_encoded_int()?;
    let from = r.read_mobile(w)?;
    let address = r.read_ip_address()?;
    let date = DateTime::try_from_ticks(r.read_date_time_ticks()?).unwrap_or(DateTime::MIN);
    Ok(RaffleEntry {
        from,
        address,
        date,
    })
}

/// `Server.Items.HouseRaffleStone`.
pub static HOUSE_RAFFLE_STONE: ItemType = ItemType {
    full_name: "Server.Items.HouseRaffleStone",
    parent: Some(&ITEM_BASE),
    construct: Some(stone_new),
    default_name: crate::class_props::default_name,
    display_weight: |_, _| false,
    get_properties: |w, id, list: &mut PropertyList| {
        (ITEM_BASE.get_properties)(w, id, list);
        let Some(d) = stone(w, id).cloned() else {
            return;
        };
        if valid_location(d.plot_bounds, d.plot_facet) {
            // GetPlotCenter reads the map; the list is built from `&World`, so the cached average is not used:
            // the center's z is only shown as part of the sextant text, which ignores z.
            let (x, y) = (
                d.plot_bounds.x() + d.plot_bounds.width() / 2,
                d.plot_bounds.y() + d.plot_bounds.height() / 2,
            );
            let text = format_location(w, Point3D::new(x, y, 0), d.plot_facet, true);
            list.add_text(&text);
        }
        match d.current_state {
            ACTIVE => {
                list.add_str(
                    1060658,
                    &format!("ticket price\t{}", format_price(d.ticket_price)),
                ); // ~1_val~: ~2_val~
                list.add_str(
                    1060659,
                    &format!("ends\t{}", date_time_string(d.started + d.duration)),
                );
            }
            COMPLETED => {
                let name = d
                    .winner
                    .and_then(|m| w.mobile(m).and_then(|mm| (mm.ty.name)(w, m)))
                    .unwrap_or_else(|| "Unknown".to_owned());
                list.add_str(1060658, &format!("winner\t{name}"));
            }
            _ => {}
        }
    },
    on_single_click: |w, id, from| {
        (ITEM_BASE.on_single_click)(w, id, from);
        let Some(d) = stone(w, id).cloned() else {
            return;
        };
        match d.current_state {
            ACTIVE => ops::label_to_number(
                w,
                id,
                from,
                1060658,
                &format!("Ends\t{}", date_time_string(d.started + d.duration)),
            ),
            COMPLETED => {
                let name = d
                    .winner
                    .and_then(|m| w.mobile(m).and_then(|mm| (mm.ty.name)(w, m)))
                    .unwrap_or_else(|| "Unknown".to_owned());
                ops::label_to_number(w, id, from, 1060658, &format!("Winner\t{name}"));
            }
            _ => {}
        }
    },
    get_context_menu_entries: |w, id, from, list| {
        (ITEM_BASE.get_context_menu_entries)(w, id, from, list);
        if !is_seer(w, from) {
            return;
        }
        list.push(entry(5101, true, |w, from, s| {
            if is_seer(w, from)
                && live_stone(w, s)
                && let Some(f) = hooks(w).and_then(|h| h.properties_gump)
            {
                f(w, from, EntityId::Item(s));
            }
        }));
        let d = stone(w, id);
        if d.is_some_and(|d| d.current_state == INACTIVE) {
            let valid = d.is_some_and(|d| valid_location(d.plot_bounds, d.plot_facet));
            list.push(entry(5113, valid, |w, from, s| {
                if is_seer(w, from)
                    && live_stone(w, s)
                    && stone(w, s).is_some_and(|d| valid_location(d.plot_bounds, d.plot_facet))
                {
                    set_state(w, s, ACTIVE);
                }
            }));
        } else {
            list.push(entry(5032, true, |w, from, s| {
                if is_seer(w, from)
                    && live_stone(w, s)
                    && let Some(f) = hooks(w).and_then(|h| h.raffle_management_gump)
                {
                    f(w, from, s);
                }
            }));
        }
    },
    on_double_click: |w, id, from| {
        if stone(w, id).is_none_or(|d| d.current_state != ACTIVE)
            || !stats::check_alive(w, from, true)
        {
            return;
        }
        let loc = ops::get_world_location(w, id);
        if !deps::mobile_in_range(w, from, loc, 2) {
            deps::local_overhead_message(
                w,
                from,
                muo_server::packets::message::MessageType::REGULAR,
                0x3B2,
                1019045,
            ); // I can't reach that.
            return;
        }
        if has_entered(w, id, from) {
            send_msg(w, from, "You have already entered this plot's raffle.");
        } else if is_at_ip_limit(w, id, from) {
            send_msg(w, from, "You may not enter this plot's raffle.");
        } else if let Some(f) = hooks(w).and_then(|h| h.raffle_ticket_gump) {
            f(w, from, id);
        }
    },
    on_delete: |w, id| {
        if let Some(r) = stone_mut(w, id).and_then(|d| d.region.take()) {
            region::unregister(w, r);
        }
        remove_raffle_stone(w, id);
        (ITEM_BASE.on_delete)(w, id);
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(4);
        let d = stone(w, id).cloned().unwrap_or_default();
        out.write_enum(4, i64::from(d.current_state));
        out.write_enum(4, i64::from(d.expire_action));
        out.write_item(w, d.deed);
        out.write_rect2d(d.plot_bounds);
        out.write_map(d.plot_facet);
        out.write_mobile(w, d.winner);
        out.write_i32(d.ticket_price);
        out.write_date_time_ticks(d.started.ticks());
        out.write_time_span_ticks(d.duration.ticks());
        out.write_encoded_int(d.entries.len() as i32);
        for e in &d.entries {
            out.write_encoded_int(0);
            out.write_mobile(w, e.from);
            out.write_ip_address(e.address);
            out.write_date_time_ticks(e.date.ticks());
        }
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let version = r.read_i32()?;
        let mut d = StoneData::default();
        if version >= 4 {
            d.current_state = r.read_enum(4)? as i32;
            d.expire_action = r.read_enum(4)? as i32;
        } else {
            d.current_state = r.read_encoded_int()?;
            d.expire_action = r.read_encoded_int()?;
        }
        d.deed = r
            .read_item(w)?
            .filter(|s| w.item(*s).is_some_and(|x| x.ty.is(&HOUSE_RAFFLE_DEED)));
        d.plot_bounds = r.read_rect2d()?;
        d.plot_facet = r.read_map()?;
        d.winner = r.read_mobile(w)?;
        d.ticket_price = r.read_i32()?;
        d.started = DateTime::try_from_ticks(r.read_date_time_ticks()?).unwrap_or(DateTime::MIN);
        d.duration = TimeSpan::from_ticks(r.read_time_span_ticks()?);
        if version >= 4 {
            let n = r.read_encoded_int()?;
            for _ in 0..n.max(0) {
                d.entries.push(read_entry(r, w)?);
            }
        } else {
            let n = r.read_i32()?;
            for _ in 0..n.max(0) {
                let e = read_entry(r, w)?;
                if e.from.is_some() {
                    d.entries.push(e); // Character was deleted
                }
            }
        }
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(d);
        }
        stone_after_deserialization(w, id);
        Ok(())
    },
    ..ITEM_BASE
};

/// `[add HouseRaffleDeed [<stone> [<mobile>]]`: `HouseRaffleDeed(HouseRaffleStone stone = null, Mobile m = null)`.
static DEED_PARAMS: [muo_content::props::Param; 2] = [
    muo_content::props::Param {
        name: "stone",
        kind: muo_content::props::Kind::Plain(muo_content::utilities::types::PropType::Entity),
        default: Some(muo_content::props::Value::Entity(None)),
        ty: "HouseRaffleStone",
    },
    muo_content::props::Param {
        name: "m",
        kind: muo_content::props::Kind::Plain(muo_content::utilities::types::PropType::Entity),
        default: Some(muo_content::props::Value::Entity(None)),
        ty: "Mobile",
    },
];

static DEED_CTORS: [muo_content::props::Ctor; 1] = [muo_content::props::Ctor {
    access: None,
    params: &DEED_PARAMS,
    make: |w, v| {
        let stone = match v[0].entity() {
            Some(EntityId::Item(i)) if self::stone(w, i).is_some() => Some(i),
            _ => None,
        };
        EntityId::Item(deed_new_with(w, stone, v[1].mobile()))
    },
}];

/// Registers the classes of this file.
pub fn register(types: &mut TypeRegistry) {
    types.register_item(&HOUSE_RAFFLE_STONE, &[]);
    types.register_item(&HOUSE_RAFFLE_DEED, &[]);
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "HouseRaffleDeed.Ctors",
        |w| {
            muo_content::props::add_ctors(w, "Server.Items.HouseRaffleDeed", &DEED_CTORS);
        },
    );
}
