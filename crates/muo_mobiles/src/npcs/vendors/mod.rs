//! Vendors: the NPC vendor classes (`npc.rs` tables, `hand.rs` with logic), the SBInfo tables
//! (`sb.rs`, `sb_helpers.rs`), over `muo_content::vendor` (BaseVendor, GenericBuyInfo, Banker statics).

/// One concrete vendor class that serializes as `BaseVendor part + version int 0`: the `MobileType`, its `CreatureType`
/// (with the `VendorClass` as `subclass`) and the constructor `$ctor` (`new_vendor` + the class's own statements).
macro_rules! vendor_leaf {
    ($name:ident, $class:ident, $vclass:ident = $full:literal, ctor $ctor:ident) => {
        vendor_leaf!($name, $class, $vclass = $full, ctor $ctor; m {});
    };
    ($name:ident, $class:ident, $vclass:ident = $full:literal, ctor $ctor:ident; m { $($mf:ident : $mv:expr,)* }) => {
        pub static $class: muo_content::creature::CreatureType = muo_content::creature::CreatureType {
            subclass: Some(&$vclass),
            ..muo_content::vendor::BASE_VENDOR_C
        };
        pub static $name: muo_server::mobile::MobileType = muo_server::mobile::MobileType {
            full_name: $full,
            parent: Some(&muo_content::vendor::BASE_VENDOR),
            construct: Some($ctor),
            family: Some(&$class),
            serialize: |w, id, out| {
                (muo_content::vendor::BASE_VENDOR.serialize)(w, id, out);
                out.write_i32(0);
            },
            deserialize: |w, id, r| {
                (muo_content::vendor::BASE_VENDOR.deserialize)(w, id, r)?;
                let _version = r.read_i32()?;
                Ok(())
            },
            $($mf: $mv,)*
            ..muo_content::vendor::BASE_VENDOR
        };
    };
}

pub mod barkeeper;
pub mod hairstylist;
pub mod hand;
pub mod npc;
pub mod player_vendor;
pub mod rental_contract;
pub mod rented;
pub mod sb;
pub mod sb_helpers;
pub mod stable;

use muo_server::ids::{ItemId, MobileId};
use muo_server::world::World;

/// `AddItem(new T(hue))` (the hue argument is read before the constructor runs) or `AddItem(new T())`.
pub fn add_item_hued(
    w: &mut World,
    id: MobileId,
    full_name: &str,
    hue: Option<i32>,
) -> Option<ItemId> {
    match hue {
        Some(h) => muo_content::vendor::add_hued(w, id, full_name, h),
        None => crate::common::add_new(w, id, full_name),
    }
}

/// `ReplaceEquipment(new T(hue))`: deletes what is on the item's layer, equips the new item or deletes it.
pub fn replace_equipment(w: &mut World, id: MobileId, full_name: &str, hue: Option<i32>) {
    let Some(item) = muo_content::factory::make_item(w, full_name) else {
        return;
    };
    if let Some(h) = hue {
        muo_server::item::base::set_hue(w, item, h);
    }
    replace_equipment_item(w, id, item);
}

/// `ReplaceEquipment(item)`.
pub fn replace_equipment_item(w: &mut World, id: MobileId, item: ItemId) {
    let layer = w.item(item).map(|i| i.layer);
    if let Some(layer) = layer
        && let Some(old) = muo_server::mobile::part2::items::find_item_on_layer(w, id, layer)
        && let Some(t) = w.item(old).map(|i| i.ty)
    {
        (t.delete)(w, old);
    }
    if !muo_server::mobile::base::equip_item(w, id, item)
        && let Some(t) = w.item(item).map(|i| i.ty)
    {
        (t.delete)(w, item);
    }
}

pub(crate) fn register(types: &mut muo_server::type_registry::TypeRegistry) {
    npc::register(types);
    hand::register(types);
    stable::register(types);
    player_vendor::register(types);
    rented::register(types);
    rental_contract::register(types);
    barkeeper::register(types);
    hairstylist::register(types);
}

#[cfg(test)]
pub(crate) fn all() -> Vec<&'static muo_server::mobile::MobileType> {
    let mut v = npc::all();
    v.extend(hand::all());
    v.extend(stable::all());
    v.extend(hairstylist::all());
    v
}
