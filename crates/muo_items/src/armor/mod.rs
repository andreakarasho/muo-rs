//! The concrete armor and shield classes (not `BaseArmor` and
//! `BaseShield`, which are in `muo_content::items::armor`). One `armor_leaf!` call per class.
//! `leaves.rs` holds the table classes; the classes with logic of their own
//! (`ElvenGlasses`, the arcane gloves, `ChaosShield`/`OrderShield`, `MetalKiteShield.Dye`) are in `special.rs`.
//!
//! `PlateChest`, `LeatherChest` and `ChainChest` are the samples of `muo_content::items::leaves` (registered
//! there); `special.rs` has their constructors for the classes derived from them.

use muo_content::aos::Attrs;
use muo_content::items::armor::{self};
use muo_content::items::set_attr;
use muo_io::{GenericReader, GenericWriter, Result};
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::skills::SkillName;
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

/// One concrete armor class: `NAME, NAME_CLASS, name_new = "Server.Items.Foo" : PARENT / PARENT_CLASS, base,
/// encoded <bool>;` where `base` is what the constructor calls first (`new_armor(w, ty, graphic)` or the
/// parent's `parent_new`), then `@init` (the rest of the constructor, in order), `@ty { .. }` (overrides that
/// are code and replace a vtable entry) and one `field: value,` per constant `override`.
/// The `construct` of an armor class: the parameterless constructor, or `none` for a class whose only `[Constructible]`
/// constructor takes arguments.
macro_rules! armor_construct {
    ($ctor:ident, $name:ident) => {
        Some(|w| $ctor(w, &$name))
    };
    ($ctor:ident, $name:ident, none) => {
        None
    };
}

macro_rules! armor_leaf {
    // `@template;`: the rest of the constructor is the class's item template (`Data/Templates/Items`).
    (
        $(#[$m:meta])*
        $name:ident, $class:ident, $ctor:ident = $full:literal : $parent:ident / $pclass:ident, $base:expr, encoded $enc:literal;
        @template;
        $($rest:tt)*
    ) => {
        $crate::armor::armor_leaf! {
            $(#[$m])*
            $name, $class, $ctor = $full : $parent / $pclass, $base, encoded $enc;
            @init |w, id| ::muo_content::templates::apply_item_as(w, id, $full);
            $($rest)*
        }
    };
    (
        $(#[$m:meta])*
        $name:ident, $class:ident, $ctor:ident = $full:literal : $parent:ident / $pclass:ident, $base:expr, encoded $enc:literal;
        $(@init $init:expr;)?
        $(@no_construct $nc:ident;)?
        $(@ty { $($tf:ident : $tv:expr,)* };)?
        $($f:ident : $v:expr,)*
    ) => {
        /// The `override`s of the class.
        pub static $class: ArmorClass = ArmorClass { $($f: $v,)* ..$pclass };

        /// `new T()` for class `ty` (a derived class passes its own).
        pub fn $ctor(w: &mut World, ty: &'static ItemType) -> ItemId {
            let base: fn(&mut World, &'static ItemType) -> ItemId = $base;
            let id = base(w, ty);
            $( let init: fn(&mut World, ItemId) = $init; init(w, id); )?
            id
        }

        $(#[$m])*
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&$parent),
            construct: $crate::armor::armor_construct!($ctor, $name $(, $nc)?),
            family: Some(&$class),
            $($($tf: $tv,)*)?
            serialize: |w, id, out| {
                ($parent.serialize)(w, id, out);
                $crate::armor::write_version(out, $enc);
            },
            deserialize: |w, id, r| {
                ($parent.deserialize)(w, id, r)?;
                let _version = $crate::armor::read_version(r, $enc)?;
                Ok(())
            },
            ..$parent
        };
    };
}
pub(crate) use armor_construct;
pub(crate) use armor_leaf;

pub mod leaves;
pub mod special;

#[cfg(test)]
mod template_tests;
#[cfg(test)]
mod tests;

// ---- constructor statements ----

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

/// `HitPoints = MaxHitPoints = value`. (`MaxHitPoints` is a plain serialized field; `HitPoints` ignores a write
/// while the maximum is 0 and is capped by it.)
pub(crate) fn max_and_hit_points(w: &mut World, id: ItemId, value: i32) {
    if let Some(d) = armor::data_mut(w, id) {
        d.max_hit_points = value;
    }
    armor::set_hit_points(w, id, value);
}

/// `SkillBonuses.SetValues(index, skill, bonus)`.
pub(crate) fn skill_bonus(w: &mut World, id: ItemId, index: u32, skill: SkillName, bonus: f64) {
    set_attr(
        w,
        id,
        muo_content::aos::AttrKind::SkillBonuses,
        1 << index,
        Attrs::skill_bonus_value(skill, bonus),
    );
}

/// Registers every armor and shield class of this crate.
pub fn register(types: &mut TypeRegistry) {
    leaves::register(types);
    special::register(types);
    // `BaseShield` is concrete (the shields derive from it): it is in the serialization table too.
    types.register_item(&muo_content::items::armor::BASE_SHIELD, &[]);
}
