//! The concrete clothing classes (not `BaseClothing` and the abstract
//! `BaseHat`, `BaseShirt`, .. which are in `muo_content::items::clothing`). One `clothing_leaf!` call per class.
//! `leaves.rs` holds the table classes; the classes
//! with logic or state of their own (the arcane cloak, robe and boots, the reward clothes, `DeathRobe`,
//! `OrcishKinMask`, `SavageMask`) are in `special.rs`. `Cap` and `FancyShirt` are the samples of
//! `muo_content::items::leaves` (registered there); `special.rs` has their constructors.

use muo_io::{GenericReader, GenericWriter, Result};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::deps;
use muo_server::type_registry::TypeRegistry;

/// The save version a class with no fields of its own writes: `[SerializationGenerator(0, false)]` writes it as an
/// int, `[SerializationGenerator(0)]` as an encoded int.
pub fn write_version(out: &mut dyn GenericWriter, encoded: bool) {
    if encoded {
        out.write_encoded_int(0);
    } else {
        out.write_i32(0);
    }
}

/// Reads what [`write_version`] wrote.
pub fn read_version(r: &mut dyn GenericReader, encoded: bool) -> Result<i32> {
    if encoded {
        r.read_encoded_int()
    } else {
        r.read_i32()
    }
}

/// One concrete clothing class: `NAME, NAME_CLASS, name_new = "Server.Items.Foo" : PARENT / PARENT_CLASS, base,
/// hue <default>, encoded <bool>;` where `base` is what the constructor calls first (`new_clothing(w, ty,
/// graphic, hue)` or the parent's `parent_new`), `hue` the default of the constructor's `int hue = ..`; then
/// `@init` (the rest of the constructor, in order), `@ty { .. }` (overrides that replace an `ItemType` entry) and
/// one `field: value,` per constant `override`.
macro_rules! clothing_leaf {
    // `@template;`: the rest of the constructor is the class's item template (`Data/Scripts/classes/items`).
    (
        $(#[$m:meta])*
        $name:ident, $class:ident, $ctor:ident = $full:literal : $parent:ident / $pclass:ident, $base:expr,
        hue $dh:expr, encoded $enc:literal;
        @template;
        $($rest:tt)*
    ) => {
        $crate::clothing::clothing_leaf! {
            $(#[$m])*
            $name, $class, $ctor = $full : $parent / $pclass, $base, hue $dh, encoded $enc;
            @init |w, id| ::muo_content::templates::apply_item_as(w, id, $full);
            $($rest)*
        }
    };
    (
        $(#[$m:meta])*
        $name:ident, $class:ident, $ctor:ident = $full:literal : $parent:ident / $pclass:ident, $base:expr,
        hue $dh:expr, encoded $enc:literal;
        $(@init $init:expr;)?
        $(@ty { $($tf:ident : $tv:expr,)* };)?
        $($f:ident : $v:expr,)*
    ) => {
        /// The `override`s of the class.
        pub static $class: ::muo_content::items::clothing::ClothingClass = ::muo_content::items::clothing::ClothingClass { $($f: $v,)* ..$pclass };

        /// `new T(hue)` for class `ty` (a derived class passes its own).
        pub fn $ctor(w: &mut ::muo_server::World, ty: &'static ::muo_server::item::ItemType, hue: i32) -> ::muo_server::ids::ItemId {
            let base: fn(&mut ::muo_server::World, &'static ::muo_server::item::ItemType, i32) -> ::muo_server::ids::ItemId = $base;
            let id = base(w, ty, hue);
            $( let init: fn(&mut ::muo_server::World, ::muo_server::ids::ItemId) = $init; init(w, id); )?
            id
        }

        $(#[$m])*
        pub static $name: ::muo_server::item::ItemType = ::muo_server::item::ItemType {
            full_name: $full,
            parent: Some(&$parent),
            construct: Some(|w| $ctor(w, &$name, $dh)),
            family: Some(&$class),
            $($($tf: $tv,)*)?
            serialize: |w, id, out| {
                ($parent.serialize)(w, id, out);
                $crate::clothing::write_version(out, $enc);
            },
            deserialize: |w, id, r| {
                ($parent.deserialize)(w, id, r)?;
                let _version = $crate::clothing::read_version(r, $enc)?;
                Ok(())
            },
            ..$parent
        };
    };
}
pub(crate) use clothing_leaf;

pub mod leaves;
pub mod special;

/// `Dye(from, sender)` that refuses with `sender.FailMessage`.
/// `new BaseClothing(itemID, hue)` with the class's `graphic` prop as the item id.
pub(crate) fn new_clothing_json(
    w: &mut World,
    ty: &'static muo_server::item::ItemType,
    hue: i32,
) -> ItemId {
    let g = muo_content::items::equip_props::graphic(w, ty);
    muo_content::items::clothing::new_clothing(w, ty, g, hue)
}

pub(crate) fn dye_refused(w: &mut World, _id: ItemId, from: MobileId, tub: ItemId) -> bool {
    let n = muo_content::items::clothing::tub_fail_message(w, tub);
    deps::send_localized_message(w, from, n);
    false
}

/// `Dye(from, sender) => false`.
pub(crate) fn dye_false(_w: &mut World, _id: ItemId, _from: MobileId, _tub: ItemId) -> bool {
    false
}

/// `Scissor(from, scissors)` that only says no.
pub(crate) fn scissor_refused(w: &mut World, _id: ItemId, from: MobileId) -> bool {
    deps::send_localized_message(w, from, 502440); // Scissors can not be used on that to produce anything.
    false
}

/// Registers every clothing class of this crate.
pub fn register(types: &mut TypeRegistry) {
    leaves::register(types);
    special::register(types);
}
