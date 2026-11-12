//! What the item code calls in other modules. Each function is one call (named in its doc) so a
//! module that lands later is wired in one place. A call whose target is a fn-pointer
//! table keeps the result for "nothing there" (no packet sent, region allows) until the owner fills it.

use muo_primitives::{Point3D, TimeSpan, utility};

use crate::ids::{EntityId, ItemId, MobileId};
use crate::item::ops;
use crate::map::MapId;
use crate::net_state::NetStateId;
use crate::packets::message::{AffixType, MessageType};
use crate::world::World;

pub fn core_ml(w: &World) -> bool {
    w.core_ml()
}

pub fn opl_enabled(w: &World) -> bool {
    w.core.opl_enabled
}

pub fn bank_access(w: &World) -> bool {
    w.core.features.bank_access
}

pub fn account_gold_enabled(w: &World) -> bool {
    crate::iaccount::gold_enabled(w)
}

pub fn account_gold_convert_on_bank(w: &World) -> bool {
    w.resources
        .get::<crate::iaccount::AccountGold>()
        .is_some_and(|a| a.convert_on_bank)
}

/// `Owner.Account.TotalGold`.
pub fn account_total_gold(w: &World, owner: MobileId) -> Option<i32> {
    crate::iaccount::account_totals(w, owner).map(|t| t.0)
}

pub fn insurance_enabled(w: &World) -> bool {
    w.core.features.insurance_enabled
}

// ---- Map ----

/// `Map.OnEnter(item)`.
pub fn map_on_enter(w: &mut World, map: MapId, id: ItemId) {
    if let Some(loc) = w.item(id).map(|i| i.location) {
        crate::map::sector::on_enter_item(w, map, loc, id);
    }
}

/// `Map.OnLeave(item)`.
pub fn map_on_leave(w: &mut World, map: MapId, id: ItemId) {
    if let Some(loc) = w.item(id).map(|i| i.location) {
        crate::map::sector::on_leave_item(w, map, loc, id);
    }
}

/// `Map.OnMove(oldLocation, item)`.
pub fn map_on_move(w: &mut World, map: MapId, old: Point3D, id: ItemId) {
    crate::map::sector::on_move_item(w, map, old, id);
}

/// `Map.GetClientsInRange(p, range)` as `(connection, its mobile)`.
pub fn clients_in_range(
    w: &World,
    map: MapId,
    p: Point3D,
    range: i32,
) -> Vec<(NetStateId, MobileId)> {
    crate::map::enumerate::clients_in_range(w, Some(map), p.x, p.y, range)
        .filter_map(|ns| w.net_mobile(ns).map(|m| (ns, m)))
        .collect()
}

/// `Map.GetItemsAt(p)`.
pub fn items_at(w: &World, map: MapId, p: Point3D) -> Vec<ItemId> {
    crate::map::enumerate::items_at(w, Some(map), p.x, p.y).collect()
}

// ---- Mobile ----

/// `Mobile.CanSee(item)`.
pub fn can_see(w: &mut World, m: MobileId, item: ItemId) -> bool {
    w.mobile(m)
        .map(|mob| mob.ty)
        .is_some_and(|ty| (ty.can_see_item)(w, m, item))
}

/// `Mobile.InRange(Point3D, range)`.
pub fn mobile_in_range(w: &World, m: MobileId, p: Point3D, range: i32) -> bool {
    w.mobile(m)
        .is_some_and(|mob| (mob.ty.in_range_point3d)(w, m, p, range))
}

/// `Mobile.NetState`.
pub fn net_state_of(w: &World, m: MobileId) -> Option<NetStateId> {
    w.mobile(m).and_then(|m| m.net_state)
}

/// `mobile.Deleted` (a mobile that no longer resolves counts as deleted).
pub fn mobile_deleted(w: &World, m: MobileId) -> bool {
    w.mobile(m).is_none_or(|m| m.deleted)
}

/// `Mobile.RemoveItem(item)`.
pub fn mobile_remove_item(w: &mut World, m: MobileId, item: ItemId) {
    crate::mobile::part2::items::remove_item(w, m, item);
}

/// `Mobile.AddItem(item)`.
pub fn mobile_add_item(w: &mut World, m: MobileId, item: ItemId) {
    crate::mobile::part2::items::add_item(w, m, item);
}

/// `Mobile.FindItemOnLayer(layer)`.
pub fn find_item_on_layer(w: &World, m: MobileId, layer: crate::layer::Layer) -> Option<ItemId> {
    crate::mobile::part2::items::find_item_on_layer(w, m, layer)
}

/// `Mobile.SendLocalizedMessage(number)`.
pub fn send_localized_message(w: &mut World, m: MobileId, number: i32) {
    crate::mobile::part2::messages::send_localized_message(w, m, number, "", 0x3B2);
}

/// `Mobile.SendMessage(text)`.
pub fn send_message(w: &mut World, m: MobileId, text: &str) {
    crate::mobile::part2::messages::send_message(w, m, text);
}

/// `Mobile.SendMessage(hue, text)`.
pub fn send_message_hue(w: &mut World, m: MobileId, hue: i32, text: &str) {
    crate::mobile::part2::messages::send_message_hue(w, m, hue, text);
}

/// `Mobile.LocalOverheadMessage(type, hue, number)`.
pub fn local_overhead_message(
    w: &mut World,
    m: MobileId,
    kind: MessageType,
    hue: i32,
    number: i32,
) {
    crate::mobile::part2::messages::local_overhead_message_number(
        w,
        m,
        muo_primitives::MessageType::from_bits_retain(i32::from(kind.bits())),
        hue,
        number,
        "",
    );
}

/// `Mobile.SendSound(soundID, p)`.
pub fn send_sound(w: &mut World, m: MobileId, sound: i32, p: Point3D) {
    crate::mobile::part2::effects::send_sound_at(w, m, sound, p);
}

/// `Mobile.CheckNonlocalDrop(from, item, target)`.
pub fn check_nonlocal_drop(
    w: &mut World,
    owner: MobileId,
    from: MobileId,
    item: ItemId,
    target: ItemId,
) -> bool {
    match w.mobile(owner).map(|m| m.ty) {
        Some(ty) => (ty.check_nonlocal_drop)(w, owner, from, item, target),
        None => true,
    }
}

/// `Mobile.InLOS(object)` (to an item): the shortcuts (deleted, no map, access level, own item) included.
pub fn in_los_item(w: &mut World, m: MobileId, target: ItemId) -> bool {
    crate::mobile::part1::combat::in_los_object(w, m, crate::map::los::LosObject::Item(target))
}

/// `Mobile.InLOS(Mobile)`.
pub fn in_los_mobile(w: &mut World, m: MobileId, target: MobileId) -> bool {
    crate::mobile::part1::combat::in_los(w, m, target)
}

/// `Mobile.InLOS(Point3D)`.
pub fn in_los_point(w: &mut World, m: MobileId, p: Point3D) -> bool {
    crate::mobile::part1::combat::in_los_point(w, m, p)
}

/// `Mobile.CheckTrade(to, item, cont, message, checkItems, plusItems, plusWeight)`.
#[allow(clippy::too_many_arguments)]
pub fn check_trade(
    w: &mut World,
    m: MobileId,
    to: MobileId,
    item: ItemId,
    cont: ItemId,
    message: bool,
    check_items: bool,
    plus_items: i32,
    plus_weight: i32,
) -> bool {
    let Some(ty) = w.mobile(m).map(|m| m.ty) else {
        return false;
    };
    (ty.check_trade)(
        w,
        m,
        to,
        item,
        cont,
        message,
        check_items,
        plus_items,
        plus_weight,
    )
}

/// `Mobile.AddToBackpack(item)`.
pub fn add_to_backpack(w: &mut World, m: MobileId, item: ItemId) {
    crate::mobile::part2::items::add_to_backpack(w, m, item);
}

/// `Mobile.PrivateOverheadMessage(type, hue, ascii, text, ns)`.
pub fn private_overhead_message(w: &mut World, m: MobileId, hue: i32, text: &str) {
    let ns = net_state_of(w, m);
    crate::mobile::part2::messages::private_overhead_message(
        w,
        m,
        muo_primitives::MessageType::REGULAR,
        hue,
        true,
        text,
        ns,
    );
}

/// `Race.IsAllowedRace(from.Race, required)`: `(allowed, race plural name, race is Gargoyle)`.
pub fn race_allows(w: &World, m: MobileId, required: i32) -> (bool, String, bool) {
    let race = w.mobile(m).and_then(|m| m.race);
    let race = race.and_then(|r| w.resources.get::<crate::race::Races>()?.get(r));
    match race {
        Some(r) => (
            r.is_allowed_race(required),
            r.plural_name.clone(),
            r.name == "Gargoyle",
        ),
        None => (false, String::new(), false),
    }
}

// ---- Messages ----

/// `NetState.SendMessageLocalized(serial, graphic, Label, 0x3B2, 3, number, name, args)`.
pub fn label_localized(
    w: &mut World,
    ns: NetStateId,
    id: ItemId,
    number: i32,
    name: &str,
    args: &str,
) {
    let Some(i) = w.item(id) else { return };
    let (serial, graphic) = (i.serial, i.item_id);
    crate::packets::message::send_message_localized(
        w,
        ns,
        serial,
        graphic,
        MessageType::LABEL,
        0x3B2,
        3,
        number,
        name,
        args,
    );
}

/// `NetState.SendMessageLocalizedAffix(serial, graphic, Label, 0x3B2, 3, number, "", type, affix, args)`.
pub fn label_localized_affix(
    w: &mut World,
    ns: NetStateId,
    id: ItemId,
    number: i32,
    affix_type: AffixType,
    affix: &str,
    args: &str,
) {
    let Some(i) = w.item(id) else { return };
    let (serial, graphic) = (i.serial, i.item_id);
    crate::packets::message::send_message_localized_affix(
        w,
        ns,
        serial,
        graphic,
        MessageType::LABEL,
        0x3B2,
        3,
        number,
        "",
        affix_type,
        affix,
        args,
    );
}

/// `NetState.SendMessage(serial, graphic, Label, 0x3B2, 3, false, "ENU", "", text)`.
pub fn label_text(w: &mut World, ns: NetStateId, id: ItemId, text: &str) {
    let Some(i) = w.item(id) else { return };
    let (serial, graphic) = (i.serial, i.item_id);
    crate::packets::message::send_message(
        w,
        ns,
        serial,
        graphic,
        MessageType::LABEL,
        0x3B2,
        3,
        false,
        Some("ENU"),
        "",
        text,
    );
}

// ---- Region ----

/// `Region.Find(GetWorldLocation(), Map)`'s virtual table and id.
fn region_of(
    w: &World,
    id: ItemId,
) -> Option<(crate::region::RegionId, &'static crate::region::RegionType)> {
    let map = w.item(id)?.map;
    let loc = ops::get_world_location(w, id);
    let r = crate::region::find(w, loc, map);
    let ty = w.maps.regions.get(&r)?.ty;
    Some((r, ty))
}

/// `Region.Find(location, map).OnDecay(item)`.
pub fn region_on_decay(w: &mut World, id: ItemId) -> bool {
    match region_of(w, id) {
        Some((r, ty)) => (ty.on_decay)(w, r, id),
        None => true,
    }
}

/// `Region.Find(location, map).CheckAccessibility(item, check)`.
pub fn region_check_accessibility(w: &mut World, id: ItemId, check: MobileId) -> bool {
    match region_of(w, id) {
        Some((r, ty)) => (ty.check_accessibility)(w, r, id, check),
        None => true,
    }
}

// ---- Secure trade ----

/// What the item code reads of a `SecureTrade`.
#[derive(Clone, Debug, Default)]
pub struct TradeSnapshot {
    /// `Trade.From.Mobile`, `Trade.To.Mobile`.
    pub from: Option<MobileId>,
    pub to: Option<MobileId>,
    /// `Trade.From.Container`, `Trade.To.Container`.
    pub from_container: Option<ItemId>,
    pub to_container: Option<ItemId>,
    /// `Trade.From.VirtualCheck`, `Trade.To.VirtualCheck`.
    pub from_check: Option<ItemId>,
    pub to_check: Option<ItemId>,
    /// `Trade.Valid`.
    pub valid: bool,
}

/// The state of a trade by its id.
pub fn trade_snapshot(w: &World, trade: crate::net_state::TradeId) -> Option<TradeSnapshot> {
    crate::secure_trade::snapshot(w, trade)
}

/// The two parties of `SecureTradeContainer.Trade` (`Trade.From.Mobile`, `Trade.To.Mobile`).
pub fn trade_parties(
    w: &World,
    trade: crate::net_state::TradeId,
) -> (Option<MobileId>, Option<MobileId>) {
    trade_snapshot(w, trade).map_or((None, None), |t| (t.from, t.to))
}

/// `Trade.From.Accepted = false; Trade.To.Accepted = false; Trade.Update()` (each unless disposed).
pub fn trade_clear_accepted_and_update(w: &mut World, trade: crate::net_state::TradeId) {
    crate::secure_trade::clear_accepted_and_update(w, trade);
}

/// `Trade.UpdateFromCurrency()` / `UpdateToCurrency()`.
pub fn trade_update_currency(w: &mut World, trade: crate::net_state::TradeId, from: bool) {
    if from {
        crate::secure_trade::update_from_currency(w, trade);
    } else {
        crate::secure_trade::update_to_currency(w, trade);
    }
}

/// `m.NetState is { NewSecureTrading: true }`.
pub fn new_secure_trading(w: &World, m: MobileId) -> bool {
    net_state_of(w, m)
        .and_then(|ns| w.net_state(ns))
        .is_some_and(|d| d.caps.new_secure_trading())
}

// ---- Sentinels shared by the item code ----

/// `Core.Now`.
pub fn now(w: &World) -> muo_primitives::DateTime {
    w.now
}

/// `entity.Deleted` for either kind.
pub fn entity_deleted(w: &World, e: EntityId) -> bool {
    match e {
        EntityId::Item(i) => ops::is_deleted(w, i),
        EntityId::Mobile(m) => mobile_deleted(w, m),
    }
}

/// `Timer.DelayCall(Delete)` on the item.
pub fn delay_delete(w: &mut World, id: ItemId) {
    crate::timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
        if let Some(ty) = w.item(id).map(|i| i.ty) {
            (ty.delete)(w, id);
        }
    });
}

/// Distance helper used by drops.
pub fn in_range_xy(a: Point3D, b: Point3D, range: i32) -> bool {
    utility::in_range(a.x, a.y, b.x, b.y, range)
}
