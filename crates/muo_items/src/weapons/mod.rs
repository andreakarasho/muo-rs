//! The concrete weapon classes (the
//! abstract bases are in `muo_content::items::weapon`). One `weapon_leaf!` call per class.
//! The classes with logic of their own are in `special.rs`.
//!
//! `Longsword`, `Katana` and `Bow` are the three samples in `muo_content::items::leaves` (registered there);
//! `samples.rs` has their constructors for the classes derived from them.

pub use muo_content::items::leaves::{
    BOW, BOW_CLASS, KATANA, KATANA_CLASS, LONGSWORD, LONGSWORD_CLASS,
};
use muo_content::items::weapon::{
    BASE_AXE, BASE_AXE_CLASS, BASE_BASHING, BASE_BASHING_CLASS, BASE_KNIFE, BASE_KNIFE_CLASS,
    BASE_POLE_ARM, BASE_POLE_ARM_CLASS, BASE_RANGED, BASE_RANGED_CLASS, BASE_SPEAR,
    BASE_SPEAR_CLASS, BASE_STAFF, BASE_STAFF_CLASS, BASE_SWORD, BASE_SWORD_CLASS, BASE_THROWN,
    BASE_THROWN_CLASS, WeaponClass, new_weapon,
};
use muo_primitives::utility::random;
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::ItemType;
use muo_server::layer::Layer;
use muo_server::skills::SkillName;
use muo_server::type_registry::TypeRegistry;

fn random_min_max(w: &mut World, min: i32, max: i32) -> i32 {
    muo_content::items::random_min_max(w, min, max)
}

/// Runs the constructor of the class a leaf derives from, for the leaf's own type.
macro_rules! ctor_call {
    (graphic, $w:ident, $ty:expr, $arg:expr) => {
        new_weapon($w, $ty, $arg)
    };
    (json, $w:ident, $ty:expr, $arg:expr) => {{
        let g = muo_content::items::equip_props::graphic($w, $ty);
        new_weapon($w, $ty, g)
    }};
    (via, $w:ident, $ty:expr, $arg:expr) => {
        $arg($w, $ty)
    };
}

macro_rules! ver_write {
    (v0, $out:ident) => {
        $out.write_i32(0)
    };
    (enc, $out:ident) => {
        $out.write_encoded_int(0)
    };
}

macro_rules! ver_read {
    (v0, $r:ident) => {
        $r.read_i32()?
    };
    (enc, $r:ident) => {
        $r.read_encoded_int()?
    };
}

/// One concrete weapon class: `NAME, NAME_CLASS, ctor_fn = "Full.Name" : PARENT / PARENT_CLASS, <how the
/// constructor starts>, <version encoding of the generated serializer>;`, then the optional `@init` (the
/// constructor body after `base(..)`), the optional `@item { .. }` (`ItemType` overrides) and the `WeaponClass`
/// overrides in declaration order.
///
/// `graphic X` is `: base(X)` of an abstract base; `via f` runs the constructor `f` of the parent leaf (an implicit
/// `base()`), both for the leaf's own type (so a virtual the constructor reads is the leaf's). `v0` is
/// `[SerializationGenerator(0, false)]` (the version as an int), `enc` is `[SerializationGenerator(0)]` (an
/// encoded int). `@json;` in place of `@init` replays the class's own template (`Data/Templates/Items`).
macro_rules! weapon_leaf {
    (
        $(#[$m:meta])*
        $name:ident, $class:ident, $ctor:ident = $full:literal : $parent:ident / $pclass:ident,
        $how:ident $arg:expr, $ver:ident;
        @json;
        $($rest:tt)*
    ) => {
        weapon_leaf! {
            $(#[$m])*
            $name, $class, $ctor = $full : $parent / $pclass, $how $arg, $ver;
            @init |w, id| muo_content::templates::apply_item_as(w, id, $full);
            $($rest)*
        }
    };
    (
        $(#[$m:meta])*
        $name:ident, $class:ident, $ctor:ident = $full:literal : $parent:ident / $pclass:ident,
        $how:ident $arg:expr, $ver:ident;
        $(@init $init:expr;)?
        $(@item { $($tf:ident : $tv:expr,)* })?
        $($f:ident : $v:expr,)*
    ) => {
        /// The `override`s of the class.
        pub static $class: WeaponClass = WeaponClass { $($f: $v,)* ..$pclass };

        /// The constructor, run for the dynamic type `ty` (a derived class passes its own).
        pub fn $ctor(w: &mut World, ty: &'static ItemType) -> ItemId {
            let id = ctor_call!($how, w, ty, $arg);
            $( let init: fn(&mut World, ItemId) = $init; init(w, id); )?
            id
        }

        $(#[$m])*
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&$parent),
            construct: Some(|w| $ctor(w, &$name)),
            family: Some(&$class),
            serialize: |w, id, out| {
                ($parent.serialize)(w, id, out);
                ver_write!($ver, out);
            },
            deserialize: |w, id, r| {
                ($parent.deserialize)(w, id, r)?;
                let _version = ver_read!($ver, r);
                Ok(())
            },
            $($($tf: $tv,)*)?
            ..$parent
        };
    };
}

pub mod artifacts;
pub mod axes;
pub mod ctor;
pub mod knives;
pub mod maces;
pub mod ml;
pub mod pickaxes;
pub mod polearms;
pub mod ranged;
pub mod samples;
pub mod se;
pub mod spears;
pub mod special;
pub mod staves;
pub mod swords;
pub mod thrown;
pub mod wooden;

pub use artifacts::*;
pub use axes::*;
pub use knives::*;
pub use maces::*;
pub use ml::*;
pub use polearms::*;
pub use ranged::*;
pub use samples::*;
pub use se::*;
pub use spears::*;
pub use special::*;
pub use staves::*;
pub use swords::*;
pub use thrown::*;
pub use wooden::*;

/// Registers every class of this directory (not the three samples, which `muo_content` registers).
pub fn register(types: &mut TypeRegistry) {
    artifacts::register(types);
    axes::register(types);
    knives::register(types);
    maces::register(types);
    ml::register(types);
    pickaxes::register(types);
    polearms::register(types);
    ranged::register(types);
    se::register(types);
    spears::register(types);
    special::register(types);
    staves::register(types);
    swords::register(types);
    thrown::register(types);
    wooden::register(types);
}

#[cfg(test)]
mod template_tests;
#[cfg(test)]
mod tests;
