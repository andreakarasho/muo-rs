//! The vendor packet records and the `IVendor` interface.

use std::cmp::Ordering;

use muo_primitives::{DateTime, Serial, TimeSpan};

use crate::ids::{ItemId, MobileId};
use crate::world::World;

/// `IVendor`.
pub trait Vendor {
    fn last_restock(&self, w: &World, id: MobileId) -> DateTime;
    fn set_last_restock(&self, w: &mut World, id: MobileId, value: DateTime);
    fn restock_delay(&self, w: &World, id: MobileId) -> TimeSpan;
    fn on_buy_items(
        &self,
        w: &mut World,
        id: MobileId,
        from: MobileId,
        list: &[BuyItemResponse],
    ) -> bool;
    fn on_sell_items(
        &self,
        w: &mut World,
        id: MobileId,
        from: MobileId,
        list: &[SellItemResponse],
    ) -> bool;
    fn restock(&self, w: &mut World, id: MobileId);
}

#[derive(Clone, Debug)]
pub struct BuyItemResponse {
    pub serial: Serial,
    pub amount: i32,
}

impl BuyItemResponse {
    pub fn new(serial: Serial, amount: i32) -> Self {
        Self { serial, amount }
    }
}

#[derive(Clone, Debug)]
pub struct SellItemResponse {
    pub item: ItemId,
    pub amount: i32,
}

impl SellItemResponse {
    pub fn new(item: ItemId, amount: i32) -> Self {
        Self { item, amount }
    }
}

#[derive(Clone, Debug)]
pub struct SellItemState {
    pub item: ItemId,
    pub price: i32,
    pub name: String,
}

impl SellItemState {
    pub fn new(item: ItemId, price: i32, name: String) -> Self {
        Self { item, price, name }
    }
}

#[derive(Clone, Debug)]
pub struct BuyItemState {
    pub price: i32,
    pub my_serial: Serial,
    pub container_serial: Serial,
    pub item_id: i32,
    pub amount: i32,
    pub hue: i32,
    pub description: String,
}

impl BuyItemState {
    pub fn new(
        name: String,
        cont: Serial,
        serial: Serial,
        price: i32,
        amount: i32,
        item_id: i32,
        hue: i32,
    ) -> Self {
        Self {
            price,
            my_serial: serial,
            container_serial: cont,
            item_id,
            amount,
            hue,
            description: name,
        }
    }
}

/// `BuyItemStateComparer.Compare` (by `MySerial`, nulls first).
pub fn compare_buy_item_state(l: Option<&BuyItemState>, r: Option<&BuyItemState>) -> Ordering {
    match (l, r) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Less,
        (Some(_), None) => Ordering::Greater,
        (Some(l), Some(r)) => l.my_serial.value().cmp(&r.my_serial.value()),
    }
}

/// `SellItemStateComparer.Equals` (same item).
pub fn sell_item_state_equals(l: Option<&SellItemState>, r: Option<&SellItemState>) -> bool {
    match (l, r) {
        (None, None) => true,
        (Some(l), Some(r)) => l.item == r.item,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comparers() {
        let a = BuyItemState::new("a".into(), Serial::ZERO, Serial::new(5), 1, 1, 1, 0);
        let b = BuyItemState::new("b".into(), Serial::ZERO, Serial::new(9), 1, 1, 1, 0);
        assert_eq!(compare_buy_item_state(Some(&a), Some(&b)), Ordering::Less);
        assert_eq!(compare_buy_item_state(None, Some(&b)), Ordering::Less);
        assert_eq!(compare_buy_item_state(None, None), Ordering::Equal);
        assert!(sell_item_state_equals(None, None));
    }
}
