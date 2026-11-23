//! The concrete Misc item classes (not `Corpse`, `Gold`, `BankCheck`, `Key`,
//! `KeyRing`: those are `muo_content::items`). `leaves.rs` holds the plain classes
//! (a class that is one constructor, constant overrides and a version-0 serializer); the classes with logic are
//! hand-written in the other modules.

use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::type_registry::TypeRegistry;

/// The save version a class with no fields of its own writes: `[SerializationGenerator(0, false)]` writes it as an
/// int, `[SerializationGenerator(0)]` as an encoded int.
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

/// One concrete `Item` subclass that is "parent part + version 0": `NAME = "Server.Items.Foo" : PARENT, |w| ctor,
/// encoded <bool>;` then one `field: value,` per constant `override`.
macro_rules! item_leaf {
    (
        $(#[$m:meta])*
        $name:ident = $full:literal : $parent:ident, |$w:ident| $ctor:expr, encoded $enc:literal;
        $($f:ident : $v:expr,)*
    ) => {
        $(#[$m])*
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&$parent),
            construct: Some(|$w| $ctor),
            $($f: $v,)*
            serialize: |w, id, out| {
                ($parent.serialize)(w, id, out);
                $crate::misc::write_version(out, $enc);
            },
            deserialize: |w, id, r| {
                ($parent.deserialize)(w, id, r)?;
                let _version = $crate::misc::read_version(r, $enc)?;
                Ok(())
            },
            ..$parent
        };
    };
}

/// A class with its own `serialize`/`deserialize` or constructor: the fields are given, the rest is the parent's.
macro_rules! item_leaf_raw {
    (
        $(#[$m:meta])*
        $name:ident = $full:literal : $parent:ident;
        $($f:ident : $v:expr,)*
    ) => {
        $(#[$m])*
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&$parent),
            $($f: $v,)*
            ..$parent
        };
    };
}
pub(crate) use item_leaf_raw;

pub mod basic;
pub mod decorator;
pub mod dyes;
pub mod gadgets;
pub mod leaves;
pub mod machines;
pub mod moongates;
pub mod player_bb;
pub mod small;
pub mod teleporter;

#[cfg(test)]
pub(crate) mod tests;

// ---- constructor statements and Utility calls ----

/// `Hue = value`.
pub(crate) fn hue(w: &mut World, id: ItemId, value: i32) {
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.set_hue)(w, id, value);
    }
}

/// `ItemID = value`.
pub(crate) fn item_id(w: &mut World, id: ItemId, value: i32) {
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.set_item_id)(w, id, value);
    }
}

/// `Utility.RandomMinMax(min, max)`.
pub(crate) fn random_min_max(w: &mut World, min: i32, max: i32) -> i32 {
    muo_primitives::utility::random_min_max(&mut w.rng, min, max)
}

/// `Utility.Random(count)`.
pub(crate) fn random(w: &mut World, count: i32) -> i32 {
    muo_primitives::utility::random(&mut w.rng, count)
}

/// `Utility.RandomList(..)`.
pub(crate) fn random_list(w: &mut World, list: &[i32]) -> i32 {
    list[random(w, list.len() as i32) as usize]
}

/// `array.RandomElement()`.
pub(crate) fn random_element(w: &mut World, list: &[i32]) -> i32 {
    muo_primitives::utility::random_element(&mut w.rng, list)
        .copied()
        .unwrap_or(0)
}

/// Registers every class of this folder.
pub fn register(types: &mut TypeRegistry) {
    leaves::register(types);
    basic::register(types);
    gadgets::register(types);
    dyes::register(types);
    decorator::register(types);
    machines::register(types);
    moongates::register(types);
    player_bb::register(types);
    small::register(types);
    teleporter::register(types);
}
