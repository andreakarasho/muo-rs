//! The container of one side of a secure trade. Not serialized with the world (its owner trade is gone after a
//! restart); only the `Container` part would be written.

use crate::Ext;
use crate::ids::{ItemId, MobileId};
use crate::item::ItemType;
use crate::item::container::{self, CONTAINER};
use crate::item::deps;
use crate::item::ops;
use crate::item::virtual_check::VIRTUAL_CHECK;
use crate::lr_reason::LRReason;
use crate::net_state::TradeId;
use crate::world::World;

/// `Server.Items.SecureTradeContainer`.
pub static SECURE_TRADE_CONTAINER: ItemType = ItemType {
    full_name: "Server.Items.SecureTradeContainer",
    parent: Some(&CONTAINER),
    construct: None,
    check_hold,
    check_lift,
    is_accessible_to,
    on_item_added: |w, id, item| clear_if_not_check(w, id, item),
    on_item_removed: |w, id, item| clear_if_not_check(w, id, item),
    on_sub_item_added: |w, id, item| clear_if_not_check(w, id, item),
    on_sub_item_removed: |w, id, item| clear_if_not_check(w, id, item),
    is_child_visible_to,
    serialize: |w, id, wr| {
        (CONTAINER.serialize)(w, id, wr);
        wr.write_i32(0); // version
    },
    deserialize: |w, id, r| {
        (CONTAINER.deserialize)(w, id, r)?;
        r.read_i32()?;
        Ok(())
    },
    ..CONTAINER
};

/// The field `SecureTradeContainer` adds: `Trade`.
#[derive(Clone, Debug, Default)]
pub struct TradeData {
    pub trade: Option<TradeId>,
}

/// `Trade` of the container `id` (or of the secure trade container above `id`'s own).
pub fn trade_of(w: &World, id: ItemId) -> Option<TradeId> {
    let cont = ops::get_secure_trade_cont(w, id)?;
    w.item(cont)?.ext.get::<TradeData>()?.trade
}

/// `new SecureTradeContainer(trade)`.
pub fn new_secure_trade_container(w: &mut World, trade: Option<TradeId>) -> ItemId {
    let id = container::new_container(w, &SECURE_TRADE_CONTAINER, 0x1E5E);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(TradeData { trade });
    }
    ops::set_movable(w, id, false);
    id
}

#[allow(clippy::too_many_arguments)]
fn check_hold(
    w: &mut World,
    id: ItemId,
    m: Option<MobileId>,
    item: ItemId,
    message: bool,
    check_items: bool,
    plus_items: i32,
    plus_weight: i32,
) -> bool {
    let Some(snapshot) = trade_of(w, id).and_then(|t| deps::trade_snapshot(w, t)) else {
        // Without a trade nothing can be held.
        return false;
    };
    if Some(item) == snapshot.from_check || Some(item) == snapshot.to_check {
        return true;
    }

    let to = if snapshot.from_container != Some(id) {
        snapshot.from
    } else {
        snapshot.to
    };
    match (m, to) {
        (Some(m), Some(to)) => deps::check_trade(
            w,
            m,
            to,
            item,
            id,
            message,
            check_items,
            plus_items,
            plus_weight,
        ),
        _ => false,
    }
}

fn check_lift(
    _w: &mut World,
    _id: ItemId,
    _from: MobileId,
    _item: ItemId,
    reject: &mut LRReason,
) -> bool {
    *reject = LRReason::CannotLift;
    false
}

fn is_accessible_to(w: &mut World, id: ItemId, check: MobileId) -> bool {
    let valid = trade_of(w, id)
        .and_then(|t| deps::trade_snapshot(w, t))
        .is_some_and(|t| t.valid);
    ops::is_child_of(w, id, Some(check.into()))
        && valid
        && crate::item::base::is_accessible_to_base(w, id, check)
}

/// `OnItemAdded`/`OnItemRemoved`/`OnSubItemAdded`/`OnSubItemRemoved`: these overrides do not call `base`
/// (so the container's version is not bumped), they only reset the accept checkmarks.
fn clear_if_not_check(w: &mut World, id: ItemId, item: ItemId) {
    let is_check = w.item(item).is_some_and(|i| i.ty.is(&VIRTUAL_CHECK));
    if !is_check {
        clear_checks(w, id);
    }
}

/// `ClearChecks()`.
pub fn clear_checks(w: &mut World, id: ItemId) {
    if let Some(t) = trade_of(w, id) {
        deps::trade_clear_accepted_and_update(w, t);
    }
}

fn is_child_visible_to(w: &World, id: ItemId, m: MobileId, child: ItemId) -> bool {
    if w.item(child).is_some_and(|i| i.ty.is(&VIRTUAL_CHECK)) {
        deps::account_gold_enabled(w) && !deps::new_secure_trading(w, m)
    } else {
        crate::item::base::is_child_visible_to_base(w, id, m, child)
    }
}
