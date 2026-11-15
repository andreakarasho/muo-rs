//! `BaseHouse`: transferring a house to another player (`BeginConfirmTransfer`, `EndConfirmTransfer`, the
//! `TransferItem` the two traders exchange).

use muo_content::items::house_deps::{WarningGump, deps};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, ops};
use muo_server::type_registry::TypeRegistry;

use super::access::{has_account_house, is_inside_mobile, is_owner, msg, set_owner};
use super::doors::{change_locks as change_locks_owner, remove_keys as remove_keys_of};
use super::{DecayLevel, data, data_mut, decay, deleted, map_of, new_vendor_system};

fn in_range(w: &World, m: MobileId, p: muo_primitives::Point3D, range: i32) -> bool {
    w.mobile(m)
        .is_some_and(|x| (x.location.x - p.x).abs().max((x.location.y - p.y).abs()) <= range)
}

/// `CheckTransferPosition(from, to)`.
#[allow(clippy::if_same_then_else)] // a chain of checks, some with the same outcome
pub fn check_transfer_position(w: &mut World, house: ItemId, from: MobileId, to: MobileId) -> bool {
    let sign = super::sign(w, house);
    let p = sign
        .map(|s| ops::get_world_location(w, s))
        .unwrap_or(muo_primitives::Point3D::ZERO);
    let hmap = map_of(w, house);
    let fmap = w.mobile(from).and_then(|m| m.map);
    let tmap = w.mobile(to).and_then(|m| m.map);
    let valid = if fmap != hmap || tmap != hmap {
        false
    } else if sign.is_none() {
        false
    } else if fmap != sign.and_then(|s| map_of(w, s)) || tmap != sign.and_then(|s| map_of(w, s)) {
        false
    } else {
        !is_inside_mobile(w, house, from)
            && !is_inside_mobile(w, house, to)
            && in_range(w, from, p, 2)
            && in_range(w, to, p, 2)
    };
    if !valid {
        // In order to transfer the house, you and the recipient must both be outside the building and within two
        // paces of the house sign.
        msg(w, from, 1062067);
    }
    valid
}

pub fn has_personal_vendors(w: &World, house: ItemId) -> bool {
    let rented = deps(w).and_then(|x| x.vendor_is_rented);
    data(w, house).is_some_and(|h| {
        h.player_vendors
            .iter()
            .any(|&v| !rented.is_some_and(|f| f(w, v)))
    })
}

/// `BeginConfirmTransfer(from, to)`.
pub fn begin_confirm_transfer(w: &mut World, house: ItemId, from: MobileId, to: MobileId) {
    if deleted(w, house)
        || !muo_server::mobile::part2::stats::check_alive(w, from, true)
        || !is_owner(w, house, from)
    {
        return;
    }
    if new_vendor_system(w) && has_personal_vendors(w, house) {
        msg(w, from, 1062467); // You cannot trade this house while you still have personal vendors inside.
    } else if decay::decay_level(w, house) == DecayLevel::DemolitionPending {
        msg(w, from, 1005321); // This house has been marked for demolition, and it cannot be transferred.
    } else if from == to {
        msg(w, from, 1005330); // You cannot transfer a house to yourself, silly.
    } else if has_account_house(w, to) {
        msg(w, from, 501388); // You cannot transfer ownership to another house owner or co-owner!
    } else if check_transfer_position(w, house, from, to) {
        msg(w, from, 1005326); // Please wait while the other player verifies the transfer.
        if decay::rented_vendors(w, house) {
            if let Some(f) = deps(w).and_then(|x| x.warning_gump) {
                /* You are about to be traded a home that has active vendor contracts. While there are active vendor
                 * contracts in this house, you cannot demolish OR customize the home. When you accept this house,
                 * you also accept landlordship for every contract vendor in the house. */
                let g = WarningGump {
                    name: "Server.Multis.BaseHouse+TradeHouseWarningGump",
                    header: 1060635,
                    header_color: 0x7800,
                    content: 1062487,
                    width: 420,
                    height: 280,
                };
                f(
                    w,
                    to,
                    g,
                    Box::new(move |w, ok| confirm_transfer_callback(w, house, to, ok, from)),
                );
            }
        } else if let Some(f) = deps(w).and_then(|x| x.accept_house_transfer_gump) {
            f(w, to, from, house);
        }
    }
}

fn confirm_transfer_callback(
    w: &mut World,
    house: ItemId,
    transferee: MobileId,
    ok: bool,
    transferor: MobileId,
) {
    if !ok
        || deleted(w, house)
        || !muo_server::mobile::part2::stats::check_alive(w, transferor, true)
        || !is_owner(w, house, transferor)
    {
        return;
    }
    if check_transfer_position(w, house, transferor, transferee)
        && let Some(f) = deps(w).and_then(|x| x.accept_house_transfer_gump)
    {
        f(w, transferee, transferor, house);
    }
}

/// `EndConfirmTransfer(transferor, transferee)`.
pub fn end_confirm_transfer(
    w: &mut World,
    house: ItemId,
    transferor: MobileId,
    transferee: MobileId,
) {
    use muo_server::mobile::part1::props_b::has_trade;
    if deleted(w, house)
        || !muo_server::mobile::part2::stats::check_alive(w, transferor, true)
        || !is_owner(w, house, transferor)
    {
        return;
    }
    if new_vendor_system(w) && has_personal_vendors(w, house) {
        msg(w, transferor, 1062467); // You cannot trade this house while you still have personal vendors inside.
    } else if decay::decay_level(w, house) == DecayLevel::DemolitionPending {
        msg(w, transferor, 1005321); // This house has been marked for demolition, and it cannot be transferred.
    } else if transferor == transferee {
        msg(w, transferor, 1005330); // You cannot transfer a house to yourself, silly.
    } else if has_account_house(w, transferee) {
        msg(w, transferor, 501388); // You cannot transfer ownership to another house owner or co-owner!
    } else if check_transfer_position(w, house, transferor, transferee) {
        let from_state = w.mobile(transferor).and_then(|m| m.net_state);
        let to_state = w.mobile(transferee).and_then(|m| m.net_state);
        if let (Some(f), Some(t)) = (from_state, to_state) {
            if has_trade(w, transferor) {
                msg(w, transferor, 1062071); // You cannot trade a house while you have other trades pending.
            } else if has_trade(w, transferee) {
                msg(w, transferee, 1062071); // You cannot trade a house while you have other trades pending.
            } else if !muo_server::mobile::base::alive(w, transferee) {
                msg(w, transferor, 1062069); // You cannot transfer this house to that person.
            } else if let Some(cont) = muo_server::secure_trade::add_trade(w, f, t) {
                let item = new_transfer_item(w, house);
                if let Some(ty) = w.item(cont).map(|i| i.ty) {
                    (ty.drop_item)(w, cont, item);
                }
            }
        }
    }
}

// ---- TransferItem ----

#[derive(Clone, Debug, Default)]
struct TransferData {
    house: Option<ItemId>,
}

fn transfer_house(w: &World, id: ItemId) -> Option<ItemId> {
    w.item(id)?.ext.get::<TransferData>()?.house
}

fn new_transfer_item(w: &mut World, house: ItemId) -> ItemId {
    let id = ops::new_item_with_id(w, &TRANSFER_ITEM, 0x14F0);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(TransferData { house: Some(house) });
    }
    muo_server::item::base::set_hue(w, id, 0x480);
    ops::set_movable(w, id, false);
    id
}

/// `Server.Multis.BaseHouse.TransferItem`.
pub static TRANSFER_ITEM: ItemType = ItemType {
    full_name: "Server.Multis.BaseHouse.TransferItem",
    parent: Some(&ITEM_BASE),
    construct: None,
    skip_serialization: |_, _| true,
    default_name: |w, id| crate::template_props::item_name(w, id, ITEM_BASE.default_name),
    get_properties: |w, id, list| {
        (ITEM_BASE.get_properties)(w, id, list);
        let house = transfer_house(w, id);
        let house_name = house.and_then(|h| super::sign(w, h)).map_or_else(
            || "an unnamed house".to_string(),
            |s| crate::sign::get_name(w, s),
        );
        let owner = house
            .and_then(|h| decay::owner_name(w, h))
            .unwrap_or_else(|| "nobody".to_string());
        let formatted = house.and_then(|h| {
            deps(w)
                .and_then(|x| x.sextant_format)
                .and_then(|f| f(w, super::location(w, h), map_of(w, h)))
        });
        list.add_str(1061112, &muo_primitives::utility::fix_html(&house_name)); // House Name: ~1_val~
        list.add_str(1061113, &owner); // Owner: ~1_val~
        list.add_str(1061114, &formatted.unwrap_or_else(|| "unknown".to_string())); // Location: ~1_val~
    },
    allow_secure_trade: |w, id, from, to, new_owner, accepted| {
        if !(ITEM_BASE.allow_secure_trade)(w, id, from, to, new_owner, accepted) {
            return false;
        }
        if !accepted {
            return true;
        }
        let Some(house) = transfer_house(w, id).filter(|&h| !deleted(w, h)) else {
            return false;
        };
        if deleted(w, id)
            || !is_owner(w, house, from)
            || !muo_server::mobile::part2::stats::check_alive(w, from, true)
            || !muo_server::mobile::part2::stats::check_alive(w, to, true)
        {
            return false;
        }
        if has_account_house(w, to) {
            msg(w, from, 501388); // You cannot transfer ownership to another house owner or co-owner!
            return false;
        }
        check_transfer_position(w, house, from, to)
    },
    on_secure_trade: |w, id, from, to, _new_owner, accepted| {
        if deleted(w, id) {
            return;
        }
        if let Some(ty) = w.item(id).map(|i| i.ty) {
            (ty.delete)(w, id);
        }
        let Some(house) = transfer_house(w, id).filter(|&h| !deleted(w, h)) else {
            return;
        };
        if !is_owner(w, house, from)
            || !muo_server::mobile::part2::stats::check_alive(w, from, true)
            || !muo_server::mobile::part2::stats::check_alive(w, to, true)
        {
            return;
        }
        if !accepted {
            return;
        }
        msg(w, from, 501338); // You have transferred ownership of the house.

        /* You are now the owner of this house. The house's co-owner, friend, ban, and access lists have been
         * cleared. You should double-check the security settings on any doors and teleporters in the house. */
        msg(w, to, 501339);

        remove_keys_of(w, house, from);
        set_owner(w, house, Some(to));
        if let Some(h) = data_mut(w, house) {
            h.bans.clear();
            h.friends.clear();
            h.co_owners.clear();
        }
        change_locks_owner(w, house, to);
        let now = w.now;
        if let Some(h) = data_mut(w, house) {
            h.last_traded = now;
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        Ok(())
    },
    ..ITEM_BASE
};

pub fn register(types: &mut TypeRegistry) {
    types.register_item(&TRANSFER_ITEM, &[]);
}
