//! GenericBuy, GenericSell, AnimalBuy, BeverageBuy, PresetMapBuy and SBInfo: what a
//! vendor sells (`BuyInfo`, one per `GenericBuyInfo`) and buys (`SellInfo`, a `GenericSellInfo`), and the
//! `SbInstance` an `SBInfo` subclass builds for one vendor (`new SBAlchemist()`).

use indexmap::IndexMap;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::type_registry::EntityType;
use muo_server::world::World;

use crate::factory;
use crate::items::{armor, weapon};

/// Which `GenericBuyInfo` subclass built the entry.
#[derive(Clone, Debug)]
pub enum BuyKind {
    /// `GenericBuyInfo`.
    Generic,
    /// `AnimalBuyInfo`: `ControlSlots`.
    Animal(i32),
    /// `BeverageBuyInfo`: the `BeverageType` of the content.
    Beverage(i32),
    /// `PresetMapBuyInfo`: the index in `PresetMapEntry.Table`.
    PresetMap(usize),
}

/// One `GenericBuyInfo`.
#[derive(Clone, Debug)]
pub struct BuyInfo {
    pub kind: BuyKind,
    /// `Type.FullName` of the class built on a purchase (`Server.Items.Garlic`); empty for a preset map.
    pub type_name: &'static str,
    /// `Args[0]` of `new object[] { 500 }` (the only constructor arguments the SBInfo tables use).
    pub arg: Option<i32>,
    pub name: String,
    /// `m_Price`.
    pub price: i32,
    /// `DefaultPrice`, the `PriceScalar`.
    pub price_scalar: i32,
    pub item_id: i32,
    pub hue: i32,
    pub amount: i32,
    pub max_amount: i32,
    /// `m_DisplayEntity`.
    pub display: Option<EntityId>,
}

fn default_name(item_id: i32) -> String {
    if item_id < 0x4000 {
        (1020000 + item_id).to_string()
    } else {
        (1078872 + item_id).to_string()
    }
}

impl BuyInfo {
    /// `new GenericBuyInfo(name, type, price, amount, itemID, hue)`; `name` is a cliloc number as text, or text.
    pub fn new(
        name: Option<&str>,
        type_name: &'static str,
        price: i32,
        amount: i32,
        item_id: i32,
        hue: i32,
    ) -> BuyInfo {
        BuyInfo {
            kind: BuyKind::Generic,
            type_name,
            arg: None,
            name: name.map_or_else(|| default_name(item_id), str::to_owned),
            price,
            price_scalar: 0,
            item_id,
            hue,
            amount,
            max_amount: amount,
            display: None,
        }
    }

    /// `new GenericBuyInfo(.., new object[] { arg })`.
    pub fn with_arg(mut self, arg: i32) -> BuyInfo {
        self.arg = Some(arg);
        self
    }

    /// `new AnimalBuyInfo(controlSlots, name, type, price, amount, itemID, hue)`.
    pub fn animal(
        control_slots: i32,
        name: Option<&str>,
        type_name: &'static str,
        price: i32,
        amount: i32,
        item_id: i32,
        hue: i32,
    ) -> BuyInfo {
        let mut b = BuyInfo::new(name, type_name, price, amount, item_id, hue);
        b.kind = BuyKind::Animal(control_slots);
        b
    }

    /// `new BeverageBuyInfo(name, type, content, price, amount, itemID, hue)`: the name is the cliloc of the
    /// container class and content.
    pub fn beverage(
        name: Option<&str>,
        type_name: &'static str,
        content: i32,
        price: i32,
        amount: i32,
        item_id: i32,
        hue: i32,
    ) -> BuyInfo {
        let mut b = BuyInfo::new(name, type_name, price, amount, item_id, hue);
        b.kind = BuyKind::Beverage(content);
        match type_name {
            "Server.Items.Pitcher" => b.name = (1048128 + content).to_string(),
            "Server.Items.BeverageBottle" => b.name = (1042959 + content).to_string(),
            "Server.Items.Jug" => b.name = (1042965 + content).to_string(),
            _ => {}
        }
        b
    }

    /// `new PresetMapBuyInfo(entry, price, amount)`: `name` is `entry.Name.ToString()`.
    pub fn preset_map(index: usize, name: i32, price: i32, amount: i32) -> BuyInfo {
        let mut b = BuyInfo::new(Some(&name.to_string()), "", price, amount, 0x14EC, 0);
        b.kind = BuyKind::PresetMap(index);
        b
    }

    /// `ControlSlots`.
    pub fn control_slots(&self) -> i32 {
        match self.kind {
            BuyKind::Animal(n) => n,
            _ => 0,
        }
    }

    /// `Price`: `m_Price` scaled by `DefaultPrice` (percent) with rounding, unscaled when that is 0.
    pub fn price(&self) -> i32 {
        if self.price_scalar != 0 {
            if self.price > 5_000_000 {
                let p = (i64::from(self.price) * i64::from(self.price_scalar) + 50) / 100;
                return p.min(i64::from(i32::MAX)) as i32;
            }
            // (m_Price * DefaultPrice + 50) / 100 in 32-bit arithmetic.
            return self.price.wrapping_mul(self.price_scalar).wrapping_add(50) / 100;
        }
        self.price
    }

    /// `Amount` setter: never below 0.
    pub fn set_amount(&mut self, value: i32) {
        self.amount = value.max(0);
    }

    /// `OnRestock`.
    pub fn on_restock(&mut self) {
        if self.amount <= 0 {
            self.max_amount = 999.min(self.max_amount.wrapping_mul(2));
        } else {
            let mut half = self.max_amount;
            if half >= 999 {
                half = 640;
            } else if half > 20 {
                half /= 2;
            }
            if self.amount >= half {
                self.max_amount = half;
            }
        }
        self.amount = self.max_amount;
    }
}

/// `GenericBuyInfo.GetEntity()`: a new instance of what is sold. A class with a constructor argument is made
/// through [`super::VendorHooks::construct_with_arg`].
pub fn get_entity(w: &mut World, info: &BuyInfo) -> Option<EntityId> {
    let hooks = w.resources.get::<super::VendorHooks>();
    match &info.kind {
        BuyKind::PresetMap(i) => {
            let f = hooks.and_then(|h| h.make_preset_map)?;
            f(w, *i).map(EntityId::Item)
        }
        BuyKind::Beverage(content) => {
            let f = hooks.and_then(|h| h.construct_with_arg);
            let id = match f {
                Some(f) => f(w, info.type_name, *content),
                None => factory::make_item(w, info.type_name),
            };
            id.map(EntityId::Item)
        }
        _ => {
            if let (Some(arg), Some(f)) = (info.arg, hooks.and_then(|h| h.construct_with_arg)) {
                return f(w, info.type_name, arg).map(EntityId::Item);
            }
            match w.types.find_type_by_full_name(info.type_name, false)? {
                EntityType::Item(t) => Some(EntityId::Item((t.construct?)(w))),
                EntityType::Mobile(t) => Some(EntityId::Mobile((t.construct?)(w))),
            }
        }
    }
}

fn entity_deleted(w: &World, e: EntityId) -> bool {
    match e {
        EntityId::Item(i) => w.item(i).is_none_or(|i| i.deleted()),
        EntityId::Mobile(m) => w.mobile(m).is_none_or(|m| m.deleted),
    }
}

/// `IEntity.Delete()`.
pub fn delete_entity(w: &mut World, e: EntityId) {
    match e {
        EntityId::Item(i) => {
            if let Some(t) = w.item(i).map(|i| i.ty) {
                (t.delete)(w, i);
            }
        }
        EntityId::Mobile(m) => {
            if let Some(t) = w.mobile(m).map(|m| m.ty) {
                (t.delete)(w, m);
            }
        }
    }
}

/// `GetDisplayEntity`: the entity the client is shown for entry `index`, made (and stored in the `DisplayCache`
/// container) when it is gone. `CanCacheDisplay` is false for every class, so no type table is kept.
pub fn get_display_entity(w: &mut World, vendor: MobileId, index: usize) -> Option<EntityId> {
    let cur = super::data(w, vendor)?.buy_info.get(index)?.display;
    if let Some(e) = cur
        && !entity_deleted(w, e)
    {
        return Some(e);
    }
    let info = super::data(w, vendor)?.buy_info.get(index)?.clone();
    let e = get_entity(w, &info);
    if let Some(e) = e {
        super::packs::display_cache_store(w, e);
    }
    super::data_mut(w, vendor)?.buy_info.get_mut(index)?.display = e;
    e
}

// ---- selling to the vendor ----

/// `GenericSellInfo`: `Dictionary<Type, int>` in insertion order, keyed by full class name.
#[derive(Clone, Debug, Default)]
pub struct SellInfo {
    pub table: IndexMap<&'static str, i32>,
}

impl SellInfo {
    /// `Add(type, price)`.
    pub fn add(&mut self, type_name: &'static str, price: i32) {
        self.table.insert(type_name, price);
    }

    /// `IsSellable` and `IsResellable` (the same test).
    pub fn is_sellable(&self, w: &World, item: ItemId) -> bool {
        w.item(item).is_some_and(|i| {
            !(i.ty.nontransferable)(w, item) && self.table.contains_key(i.ty.full_name)
        })
    }

    /// `GetSellPriceFor(item)`.
    pub fn get_sell_price_for(&self, w: &World, item: ItemId) -> i32 {
        let Some(it) = w.item(item) else { return 0 };
        let mut price = self.table.get(it.ty.full_name).copied().unwrap_or(0);

        if it.ty.is(&armor::BASE_ARMOR) {
            if let Some(a) = armor::data(w, item) {
                price = match a.quality {
                    armor::ArmorQuality::Low => (f64::from(price) * 0.60) as i32,
                    armor::ArmorQuality::Exceptional => (f64::from(price) * 1.25) as i32,
                    _ => price,
                };
                price += 100 * (a.durability as i32);
                price += 100 * (a.protection_level as i32);
                if price < 1 {
                    price = 1;
                }
            }
        } else if it.ty.is(&weapon::BASE_WEAPON) {
            if let Some(a) = weapon::data(w, item) {
                price = match a.quality {
                    weapon::WeaponQuality::Low => (f64::from(price) * 0.60) as i32,
                    weapon::WeaponQuality::Exceptional => (f64::from(price) * 1.25) as i32,
                    _ => price,
                };
                price += 100 * (a.durability_level as i32);
                price += 100 * (a.damage_level as i32);
                if price < 1 {
                    price = 1;
                }
            }
        } else if let Some(f) = w
            .resources
            .get::<super::VendorHooks>()
            .and_then(|h| h.beverage_is_empty_or_milk)
        {
            // BaseBeverage: price1 when empty or milk, price2 otherwise; Pitcher 3/5, bottle 3/3, Jug 6/6.
            if let Some(empty_or_milk) = f(w, item) {
                let (p1, p2) = match it.ty.full_name {
                    "Server.Items.Pitcher" => (3, 5),
                    "Server.Items.BeverageBottle" => (3, 3),
                    "Server.Items.Jug" => (6, 6),
                    _ => (price, price),
                };
                price = if empty_or_milk { p1 } else { p2 };
            }
        }
        price
    }

    /// `GetBuyPriceFor(item)`: `(int)(1.90 * GetSellPriceFor(item))`.
    pub fn get_buy_price_for(&self, w: &World, item: ItemId) -> i32 {
        (1.90 * f64::from(self.get_sell_price_for(w, item))) as i32
    }

    /// `GetNameFor(item)`.
    pub fn get_name_for(&self, w: &World, item: ItemId) -> String {
        match w.item(item) {
            Some(i) => i
                .name
                .clone()
                .unwrap_or_else(|| (i.ty.label_number)(w, item).to_string()),
            None => String::new(),
        }
    }

    /// `item.InTypeList(Types)`: the item's class is, or derives from, a key.
    pub fn in_type_list(&self, w: &World, item: ItemId) -> bool {
        let Some(i) = w.item(item) else { return false };
        let mut t = Some(i.ty);
        while let Some(ty) = t {
            if self.table.contains_key(ty.full_name) {
                return true;
            }
            t = ty.parent;
        }
        false
    }
}

/// What one `SBInfo` subclass instance holds: `BuyInfo` and `SellInfo`.
#[derive(Clone, Debug, Default)]
pub struct SbInstance {
    pub buy: Vec<BuyInfo>,
    pub sell: SellInfo,
}

/// An `SBInfo` class: `build` is `new SBAlchemist()` (the sell table is a field initializer, the buy list follows;
/// both may use the world's random generator and expansion).
pub struct SbInfo {
    pub name: &'static str,
    pub build: fn(&mut World, MobileId, &mut SbInstance),
}

impl SbInfo {
    pub fn new_for(&self, w: &mut World, vendor: MobileId) -> SbInstance {
        let mut sb = SbInstance::default();
        (self.build)(w, vendor, &mut sb);
        sb
    }
}
