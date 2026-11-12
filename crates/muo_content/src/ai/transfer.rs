//! `TransferItem` (the item that stands for a pet in a secure trade) and `ShrinkTable`
//! (Data/shrink.json), whose lookup picks its graphic.

use muo_server::Ext;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType};
use muo_server::property_list::PropertyList;
use muo_server::world::World;

use crate::creature::control::{control_master, set_control_master};
use crate::creature::data::{cd, cd_mut};
use crate::creature::vtable::ctype;
use crate::mx;

/// `ShrinkTable.DefaultItemID`: yellow virtue stone.
pub const DEFAULT_ITEM_ID: i32 = 0x1870;

/// `_shrinkTable`: body is the index, value the item id. Loaded on first use.
#[derive(Default)]
pub struct ShrinkTable(Option<Vec<i32>>);

fn to_int(s: &str) -> Option<i32> {
    let s = s.trim();
    match s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        Some(h) => i32::from_str_radix(h, 16).ok(),
        None => s.parse().ok(),
    }
}

fn load(w: &World) -> Vec<i32> {
    let path = w.core.base_directory.join("Data/shrink.json");
    let text = std::fs::read_to_string(path).unwrap_or_default();
    let map: indexmap::IndexMap<String, String> =
        muo_data::json::from_str(text.trim_start_matches('\u{feff}')).unwrap_or_default();
    let pairs: Vec<(usize, i32)> = map
        .iter()
        .filter_map(|(k, v)| Some((usize::try_from(to_int(k)?).ok()?, to_int(v)?)))
        .collect();
    let len = pairs.iter().map(|&(b, _)| b + 1).max().unwrap_or(0);
    let mut table = vec![0; len];
    for (b, item) in pairs {
        table[b] = item;
    }
    table
}

/// `ShrinkTable.Lookup(body, defaultValue)`.
pub fn shrink_lookup(w: &mut World, body: i32, default: i32) -> i32 {
    if w.resources
        .get::<ShrinkTable>()
        .is_none_or(|t| t.0.is_none())
    {
        let table = load(w);
        w.resources.get_mut::<ShrinkTable>().0 = Some(table);
    }
    let table = w.resources.get::<ShrinkTable>().and_then(|t| t.0.as_ref());
    match table.and_then(|t| usize::try_from(body).ok().and_then(|b| t.get(b))) {
        Some(&v) if v != 0 => v,
        _ => default,
    }
}

/// `TransferItem._creature`.
pub struct TransferData {
    pub creature: MobileId,
}

pub static TRANSFER_ITEM: ItemType = ItemType {
    full_name: "Server.Mobiles.TransferItem",
    parent: Some(&ITEM_BASE),
    construct: None,
    default_name,
    get_properties,
    skip_serialization: |_, _| true,
    allow_secure_trade,
    on_secure_trade,
    ..ITEM_BASE
};

/// `new TransferItem(creature)`.
pub fn new_transfer_item(w: &mut World, creature: MobileId) -> ItemId {
    let (body, hue) = w.mobile(creature).map_or((0, 0), |m| {
        (
            muo_server::mobile::part1::props_b::body(w, creature).0,
            (m.ty.hue)(w, creature),
        )
    });
    let graphic = shrink_lookup(w, body, DEFAULT_ITEM_ID);
    let id = w.new_item(&TRANSFER_ITEM);
    if let Some(it) = w.item_mut(id) {
        it.item_id = graphic;
        it.ext = Ext::new(TransferData { creature });
    }
    muo_server::item::ops::set_movable(w, id, false);
    muo_server::item::base::set_hue(w, id, hue & 0x0FFF);
    id
}

fn creature_of(w: &World, id: ItemId) -> Option<MobileId> {
    w.item(id)?.ext.get::<TransferData>().map(|d| d.creature)
}

fn default_name(w: &World, id: ItemId) -> Option<String> {
    creature_of(w, id).map(|c| crate::creature::type_name(w, c).to_owned())
}

fn get_properties(w: &World, id: ItemId, list: &mut PropertyList) {
    (ITEM_BASE.get_properties)(w, id, list);
    let Some(c) = creature_of(w, id) else { return };
    list.add(1041603); // This item represents a pet currently in consideration for trade
    list.add_str(
        1041601,
        &muo_server::mobile::base::name(w, c).unwrap_or_default(),
    ); // Pet Name: ~1_val~
    if let Some(m) = control_master(w, c) {
        list.add_str(
            1041602,
            &muo_server::mobile::base::name(w, m).unwrap_or_default(),
        ); // Owner: ~1_val~
    }
}

/// `IsInCombat(creature)`.
pub fn is_in_combat(w: &World, c: MobileId) -> bool {
    w.mobile(c)
        .is_some_and(|m| !m.aggressors.is_empty() || !m.aggressed.is_empty())
}

fn is_invalid_trade(w: &mut World, item: ItemId, from: MobileId, to: MobileId) -> bool {
    let deleted = w.item(item).is_none_or(|i| i.deleted());
    let creature = creature_of(w, item).filter(|&c| !mx::deleted(w, c));
    let Some(c) = creature else { return true };
    deleted
        || control_master(w, c) != Some(from)
        || !muo_server::mobile::part2::stats::check_alive(w, from, true)
        || !muo_server::mobile::part2::stats::check_alive(w, to, true)
        || mx::map(w, from) != mx::map(w, c)
        || !mx::in_range(w, from, c, 14)
}

fn young(w: &World, m: MobileId) -> bool {
    crate::hooks::pet_player_hooks(w)
        .and_then(|h| h.young)
        .is_some_and(|f| f(w, m))
}

fn allow_secure_trade(
    w: &mut World,
    id: ItemId,
    from: MobileId,
    to: MobileId,
    new_owner: MobileId,
    accepted: bool,
) -> bool {
    if !(ITEM_BASE.allow_secure_trade)(w, id, from, to, new_owner, accepted)
        || is_invalid_trade(w, id, from, to)
    {
        return false;
    }
    !accepted || handle_accepted_trade(w, id, from, to)
}

fn handle_accepted_trade(w: &mut World, item: ItemId, from: MobileId, to: MobileId) -> bool {
    let Some(c) = creature_of(w, item) else {
        return false;
    };
    validate_young_status(w, from, to)
        && validate_control_status(w, c, from, to)
        && validate_follower_limit(w, c, to)
        && !is_in_combat(w, c)
}

fn validate_follower_limit(w: &mut World, c: MobileId, to: MobileId) -> bool {
    let slots = cd(w, c).map_or(1, |d| d.control_slots);
    let (followers, max) = w
        .mobile(to)
        .map_or((0, 0), |m| (m.followers, m.followers_max));
    if followers + slots > max {
        mx::msg(w, to, 1049607); // You have too many followers to control that creature.
        return false;
    }
    true
}

fn validate_young_status(w: &mut World, from: MobileId, to: MobileId) -> bool {
    let (yf, yt) = (young(w, from), young(w, to));
    if yf && !yt {
        mx::msg(w, from, 502051); // As a young player, you may not transfer pets to older players.
        return false;
    }
    if !yf && yt {
        mx::msg(w, from, 502052); // As an older player, you may not transfer pets to young players.
        return false;
    }
    true
}

fn validate_control_status(w: &mut World, c: MobileId, from: MobileId, to: MobileId) -> bool {
    let Some(t) = ctype(w, c) else { return false };
    if !(t.can_be_controlled_by)(w, c, to) {
        send_transfer_refusal(w, from, to, 1043248, 1043249);
        return false;
    }
    if !(t.can_be_controlled_by)(w, c, from) {
        send_transfer_refusal(w, from, to, 1043250, 1043251);
        return false;
    }
    true
}

/// `SendTransferRefusalMessages`.
pub fn send_transfer_refusal(
    w: &mut World,
    from: MobileId,
    to: MobileId,
    from_message: i32,
    to_message: i32,
) {
    let name = |w: &World, m| muo_server::mobile::base::name(w, m).unwrap_or_default();
    let args = format!("{}\t{}\t ", name(w, to), name(w, from));
    mx::msg_args(w, from, from_message, &args);
    mx::msg_args(w, to, to_message, &args);
}

fn on_secure_trade(
    w: &mut World,
    id: ItemId,
    from: MobileId,
    to: MobileId,
    _new_owner: MobileId,
    accepted: bool,
) {
    let invalid = w.item(id).is_none_or(|i| i.deleted()) || is_invalid_trade(w, id, from, to);
    let creature = creature_of(w, id);
    muo_server::item::base::delete(w, id);
    if invalid {
        return;
    }
    let Some(c) = creature else { return };
    if !accepted || !set_control_master(w, c, Some(to)) {
        return;
    }
    transfer_pet_ownership(w, c, from, to);
}

fn transfer_pet_ownership(w: &mut World, c: MobileId, from: MobileId, to: MobileId) {
    crate::creature::control::clear_pet_friends(w, c);
    crate::creature::control::issue_order(
        w,
        c,
        crate::creature::enums::OrderType::Follow,
        None,
        Some(to),
    );
    if let Some(d) = cd_mut(w, c) {
        d.bonding_begin = muo_primitives::DateTime::MIN;
        d.owner_abandon_time = muo_primitives::DateTime::MIN;
        d.is_bonded = false;
    }
    let name = |w: &World, m| muo_server::mobile::base::name(w, m).unwrap_or_default();
    let args = format!("{}\t{}\t{}", name(w, from), name(w, c), name(w, to));
    mx::msg_args(w, from, 1043253, &args); // You have transferred your pet to ~3_GETTER~.
    mx::msg_args(w, to, 1043252, &args); // ~1_NAME~ has transferred the allegiance of ~2_PET_NAME~ to you.
}
