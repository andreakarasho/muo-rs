//! `BaseWaterContainer`, `Bucket`, `WaterBarrel` and `WaterTub`.
//!
//! `BaseWaterContainer` is an abstract `Container` holding `IHasQuantity`'s `Quantity`; a full one has no contents and
//! cannot be opened or picked up. It is a family here: [`WaterClass`] holds what the three leaves override.
//!
//! The crafting engine's `QuantityApi` (`muo_engines::craft::state`) reads these (and the beverages and the flour
//! sack) as `IHasQuantity`.

use muo_primitives::Point3D;
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::container::{self, CONTAINER};
use muo_server::item::{ItemType, ops};
use muo_server::property_list::PropertyList;
use muo_server::type_registry::TypeRegistry;

/// What a leaf overrides: `EmptyItemId`, `FullItemId`, `MaxQuantity`.
pub struct WaterClass {
    pub empty_item_id: i32,
    pub full_item_id: i32,
    pub max_quantity: i32,
}

/// `BaseWaterContainer._quantity`.
#[derive(Clone, Copy, Debug, Default)]
struct WaterData {
    quantity: i32,
}

fn class(w: &World, id: ItemId) -> Option<&'static WaterClass> {
    w.item(id)?.ty.family::<WaterClass>()
}

fn quantity_raw(w: &World, id: ItemId) -> i32 {
    w.item(id)
        .and_then(|i| i.ext.get::<WaterData>())
        .map_or(0, |d| d.quantity)
}

/// `Quantity` getter.
pub fn quantity(w: &World, id: ItemId) -> i32 {
    quantity_raw(w, id)
}

/// `IsEmpty`.
pub fn is_empty(w: &World, id: ItemId) -> bool {
    quantity_raw(w, id) <= 0
}

/// `IsFull`.
pub fn is_full(w: &World, id: ItemId) -> bool {
    class(w, id).is_some_and(|c| quantity_raw(w, id) >= c.max_quantity)
}

/// `Quantity = value`.
pub fn set_quantity(w: &mut World, id: ItemId, value: i32) {
    let Some(c) = class(w, id) else { return };
    if value == quantity_raw(w, id) {
        return;
    }
    let q = value.clamp(0, c.max_quantity);
    if let Some(d) = w.item_mut(id).and_then(|i| i.ext.get_mut::<WaterData>()) {
        d.quantity = q;
    }
    let empty = is_empty(w, id);
    let movable = !ops::is_locked_down(w, id) && empty;
    ops::set_movable(w, id, movable);
    let ty = w.item(id).map(|i| i.ty).expect("item");
    (ty.set_item_id)(
        w,
        id,
        if empty {
            c.empty_item_id
        } else {
            c.full_item_id
        },
    );
    if !empty {
        let at = ops::root_parent(w, id).and_then(|r| match r {
            EntityId::Item(i) => w.item(i).map(|x| (x.location, x.map)),
            EntityId::Mobile(m) => w.mobile(m).map(|x| (x.location, x.map)),
        });
        if let Some((loc, Some(map))) = at
            && map != muo_server::map::MapId::INTERNAL
        {
            ops::move_to_world_map(w, id, loc, Some(map));
        }
    }
    ops::invalidate_properties(w, id);
}

fn label_full(w: &mut World, id: ItemId, from: MobileId) {
    match w.item(id).and_then(|i| i.name.clone()) {
        None => {
            let ty = w.item(id).map(|i| i.ty).expect("item");
            let n = (ty.label_number)(w, id);
            ops::label_to_number(w, id, from, n, "");
        }
        Some(name) => ops::label_to(w, id, from, &name),
    }
}

/// `new T(filled)`: `base(filled ? FullItemId : EmptyItemId, filled)`.
fn new_water(w: &mut World, ty: &'static ItemType, filled: bool) -> ItemId {
    let c = ty.family::<WaterClass>().expect("water container class");
    let id = container::new_container(
        w,
        ty,
        if filled {
            c.full_item_id
        } else {
            c.empty_item_id
        },
    );
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(WaterData {
            quantity: if filled { c.max_quantity } else { 0 },
        });
    }
    id
}

macro_rules! water_leaf {
    ($(#[$m:meta])* $name:ident, $class:ident = $full:literal, $empty:expr, $full_id:expr, $max:expr $(, label $label:expr)?) => {
        pub static $class: WaterClass = WaterClass { empty_item_id: $empty, full_item_id: $full_id, max_quantity: $max };
        $(#[$m])*
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&CONTAINER),
            construct: Some(|w| new_water(w, &$name, false)),
            family: Some(&$class),
            $(label_number: |_, _| $label,)?
            default_gump_id: |_, _| 0x3e,
            on_double_click: |w, id, from| {
                if is_empty(w, id) {
                    (CONTAINER.on_double_click)(w, id, from);
                }
            },
            on_single_click: |w, id, from| {
                if is_empty(w, id) {
                    (CONTAINER.on_single_click)(w, id, from);
                } else {
                    label_full(w, id, from);
                }
            },
            on_aos_single_click: |w, id, from| {
                if is_empty(w, id) {
                    (CONTAINER.on_aos_single_click)(w, id, from);
                } else {
                    label_full(w, id, from);
                }
            },
            get_properties: |w, id, list: &mut PropertyList| {
                if is_empty(w, id) {
                    (CONTAINER.get_properties)(w, id, list);
                }
            },
            on_drag_drop_into: |w, id, from, item, p: Point3D| {
                is_empty(w, id) && (CONTAINER.on_drag_drop_into)(w, id, from, item, p)
            },
            serialize: |w, id, out| {
                (CONTAINER.serialize)(w, id, out);
                out.write_i32(0); // BaseWaterContainer's version
                out.write_i32(quantity_raw(w, id));
                out.write_i32(0); // the leaf's own version
            },
            deserialize: |w, id, r| {
                (CONTAINER.deserialize)(w, id, r)?;
                let _version = r.read_i32()?;
                let q = r.read_i32()?;
                if let Some(i) = w.item_mut(id) {
                    i.ext.insert(WaterData { quantity: q });
                }
                let _leaf_version = r.read_i32()?;
                Ok(())
            },
            ..CONTAINER
        };
    };
}

water_leaf!(
    /// `Server.Items.Bucket`.
    BUCKET,
    BUCKET_CLASS = "Server.Items.Bucket",
    0x14e0,
    0x2004,
    25
);
water_leaf!(
    /// `Server.Items.WaterBarrel`.
    WATER_BARREL, WATER_BARREL_CLASS = "Server.Items.WaterBarrel", 0xe77, 0x154d, 100, label 1025453 // water barrel
);
water_leaf!(
    /// `Server.Items.Tub`.
    TUB,
    TUB_CLASS = "Server.Items.Tub",
    0xe83,
    0xe7b,
    50
);

/// `new Bucket(filled)`.
pub fn new_bucket(w: &mut World, filled: bool) -> ItemId {
    new_water(w, &BUCKET, filled)
}

/// `new WaterBarrel(filled)`.
pub fn new_water_barrel(w: &mut World, filled: bool) -> ItemId {
    new_water(w, &WATER_BARREL, filled)
}

/// `new Tub(filled)`.
pub fn new_tub(w: &mut World, filled: bool) -> ItemId {
    new_water(w, &TUB, filled)
}

pub static ALL: &[&ItemType] = &[&BUCKET, &WATER_BARREL, &TUB];

/// Registers the classes of this file.
pub fn register(types: &mut TypeRegistry) {
    for t in ALL {
        types.register_item(t, &[]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use muo_io::{SaveReader, SaveWriter};
    use muo_server::item::Item;

    #[test]
    fn water_container_quantity_and_save_bytes() {
        let mut w = crate::test_world();
        let id = new_bucket(&mut w, true);
        assert_eq!((quantity(&w, id), is_full(&w, id)), (25, true));
        assert_eq!((BUCKET.item_id)(&w, id), 0x2004);
        set_quantity(&mut w, id, 99); // clamped to MaxQuantity
        assert_eq!(quantity(&w, id), 25);
        set_quantity(&mut w, id, 0);
        assert!(is_empty(&w, id));
        assert_eq!((BUCKET.item_id)(&w, id), 0x14e0);
        assert!(w.item(id).unwrap().movable());

        set_quantity(&mut w, id, 7);
        let mut out = SaveWriter::new(true);
        (BUCKET.serialize)(&w, id, &mut out);
        let bytes = out.into_vec();
        assert_eq!(
            &bytes[bytes.len() - 12..],
            &[0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0]
        );

        let mut w2 = crate::test_world();
        let id2 = w2.add_item(Item::new_for_load(
            muo_primitives::Serial::new(0x4000_0002),
            &BUCKET,
        ));
        let mut r = SaveReader::new(&bytes);
        (BUCKET.deserialize)(&mut w2, id2, &mut r).unwrap();
        assert_eq!(r.remaining(), 0);
        assert_eq!(quantity(&w2, id2), 7);
    }
}
