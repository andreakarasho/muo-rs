//! What the generated item classes (items_gen.rs) use: the `leaf!` macro (a copy of muo_items::simple's, which this
//! crate cannot depend on) and the constructor statements.

use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::layer::Layer;

pub use muo_server::item::base::add_quest_item_property;

/// The save version a class with no fields of its own writes: `[SerializationGenerator(0, false)]` writes it as an
/// int (`v0`), `[SerializationGenerator(0)]` as an encoded int (`enc`).
pub fn write_version(out: &mut dyn muo_io::GenericWriter, encoded: bool) {
    if encoded {
        out.write_encoded_int(0);
    } else {
        out.write_i32(0);
    }
}

/// Reads what [`write_version`] wrote.
pub fn read_version(r: &mut dyn muo_io::GenericReader, encoded: bool) -> muo_io::Result<i32> {
    if encoded {
        r.read_encoded_int()
    } else {
        r.read_i32()
    }
}

/// One concrete class that is "parent part + version 0".
macro_rules! leaf {
    (@enc enc) => { true };
    (@enc v0) => { false };
    (
        $(#[$m:meta])*
        $name:ident = $full:literal : $parent:ident, $ver:ident;
        $($f:ident : $v:expr,)*
    ) => {
        $(#[$m])*
        pub static $name: muo_server::item::ItemType = muo_server::item::ItemType {
            full_name: $full,
            parent: Some(&$parent),
            $($f: $v,)*
            serialize: |w, id, out| {
                ($parent.serialize)(w, id, out);
                $crate::ml_quests::item_support::write_version(out, leaf!(@enc $ver));
            },
            deserialize: |w, id, r| {
                ($parent.deserialize)(w, id, r)?;
                let _version = $crate::ml_quests::item_support::read_version(r, leaf!(@enc $ver))?;
                Ok(())
            },
            ..$parent
        };
    };
}
pub(crate) use leaf;

/// `Hue = value`, through the class's own setter.
pub fn hue(w: &mut World, id: ItemId, value: i32) {
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.set_hue)(w, id, value);
    }
}

/// `Layer = value`, through the class's own setter.
pub fn layer(w: &mut World, id: ItemId, value: Layer) {
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.set_layer)(w, id, value);
    }
}

/// `DropItem(new T(amount))` into the container `id`; the `(int amount)` constructor only sets `Amount`.
pub fn drop_new(w: &mut World, id: ItemId, name: &str, amount: Option<i32>) {
    let Some(item) = super::engine::make_item_by_name(w, name) else {
        return;
    };
    if let Some(a) = amount {
        muo_server::item::ops::set_amount(w, item, a);
    }
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.drop_item)(w, id, item);
    }
}
