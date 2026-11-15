//! Shared pieces of the `leaf_simple` folders (Resources, Food, Containers, Lights, Deeds, Body Parts, Farming,
//! PlantsFlowers, Traps, TreasureChests, the artifact folders, New Haven Quest Rewards and Construction): the leaf
//! macro and the constructor statements the leaves use.

use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::layer::Layer;
use muo_server::type_registry::TypeRegistry;

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

/// One concrete class that is "parent part + version 0": `NAME = "Server.Items.Foo" : PARENT, enc|v0;` then the
/// `construct:` line and one `field: value,` per constant `override` (`ItemType` fields).
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
                $crate::simple::write_version(out, leaf!(@enc $ver));
            },
            deserialize: |w, id, r| {
                ($parent.deserialize)(w, id, r)?;
                let _version = $crate::simple::read_version(r, leaf!(@enc $ver))?;
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

/// `ItemID = value`, through the class's own setter.
pub fn item_id(w: &mut World, id: ItemId, value: i32) {
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.set_item_id)(w, id, value);
    }
}

/// `Layer = value`, through the class's own setter.
pub fn layer(w: &mut World, id: ItemId, value: Layer) {
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.set_layer)(w, id, value);
    }
}

/// `new T()` of a class whose constructor is its template `name` (its graphic is the template's `itemId`), for
/// class `ty` (a derived class passes its own).
pub fn from_template(w: &mut World, ty: &'static muo_server::item::ItemType, name: &str) -> ItemId {
    let id = muo_server::item::ops::new_item_with_id(w, ty, 0);
    muo_content::templates::apply_item_as(w, id, name);
    id
}

/// Every generated class of the folders with its `[TypeAlias]` names, for the tests and the registry.
pub fn all() -> Vec<(&'static muo_server::item::ItemType, &'static [&'static str])> {
    let mut v = Vec::new();
    v.extend_from_slice(crate::body_parts::leaves::ALL);
    v.extend_from_slice(crate::champion_artifacts::leaves::ALL);
    v.extend_from_slice(crate::construction::leaves::ALL);
    v.extend_from_slice(crate::containers::leaves::ALL);
    v.extend_from_slice(crate::decoration_artifacts::leaves::ALL);
    v.extend_from_slice(crate::deeds::leaves::ALL);
    v.extend_from_slice(crate::farming::leaves::ALL);
    v.extend_from_slice(crate::food::leaves::ALL);
    v.extend_from_slice(crate::lights::leaves::ALL);
    v.extend_from_slice(crate::minor_artifacts::leaves::ALL);
    v.extend_from_slice(crate::new_haven_quest_rewards::leaves::ALL);
    v.extend_from_slice(crate::plants_flowers::leaves::ALL);
    v.extend_from_slice(crate::resources::leaves::ALL);
    v.extend_from_slice(crate::traps::leaves::ALL);
    v.extend_from_slice(crate::treasure_chests::leaves::ALL);
    v
}

/// Every folder's `IMPLEMENTS` rows (interfaces listed after a base class).
pub fn implements_tables()
-> impl Iterator<Item = (&'static muo_server::item::ItemType, &'static str)> {
    let tables: [&'static [(&'static muo_server::item::ItemType, &'static str)]; 15] = [
        crate::body_parts::leaves::IMPLEMENTS,
        crate::champion_artifacts::leaves::IMPLEMENTS,
        crate::construction::leaves::IMPLEMENTS,
        crate::containers::leaves::IMPLEMENTS,
        crate::decoration_artifacts::leaves::IMPLEMENTS,
        crate::deeds::leaves::IMPLEMENTS,
        crate::farming::leaves::IMPLEMENTS,
        crate::food::leaves::IMPLEMENTS,
        crate::lights::leaves::IMPLEMENTS,
        crate::minor_artifacts::leaves::IMPLEMENTS,
        crate::new_haven_quest_rewards::leaves::IMPLEMENTS,
        crate::plants_flowers::leaves::IMPLEMENTS,
        crate::resources::leaves::IMPLEMENTS,
        crate::traps::leaves::IMPLEMENTS,
        crate::treasure_chests::leaves::IMPLEMENTS,
    ];
    tables.into_iter().flatten().copied()
}

/// Every folder's `COMMODITIES` rows: (class, `IsDeedable`).
pub fn commodity_tables() -> impl Iterator<Item = (&'static muo_server::item::ItemType, bool)> {
    let tables: [&'static [(&'static muo_server::item::ItemType, bool)]; 15] = [
        crate::body_parts::leaves::COMMODITIES,
        crate::champion_artifacts::leaves::COMMODITIES,
        crate::construction::leaves::COMMODITIES,
        crate::containers::leaves::COMMODITIES,
        crate::decoration_artifacts::leaves::COMMODITIES,
        crate::deeds::leaves::COMMODITIES,
        crate::farming::leaves::COMMODITIES,
        crate::food::leaves::COMMODITIES,
        crate::lights::leaves::COMMODITIES,
        crate::minor_artifacts::leaves::COMMODITIES,
        crate::new_haven_quest_rewards::leaves::COMMODITIES,
        crate::plants_flowers::leaves::COMMODITIES,
        crate::resources::leaves::COMMODITIES,
        crate::traps::leaves::COMMODITIES,
        crate::treasure_chests::leaves::COMMODITIES,
    ];
    tables.into_iter().flatten().copied()
}

/// Every folder's `[Furniture]` classes.
pub fn furniture_tables() -> impl Iterator<Item = &'static muo_server::item::ItemType> {
    let tables: [&'static [&'static muo_server::item::ItemType]; 15] = [
        crate::body_parts::leaves::FURNITURE,
        crate::champion_artifacts::leaves::FURNITURE,
        crate::construction::leaves::FURNITURE,
        crate::containers::leaves::FURNITURE,
        crate::decoration_artifacts::leaves::FURNITURE,
        crate::deeds::leaves::FURNITURE,
        crate::farming::leaves::FURNITURE,
        crate::food::leaves::FURNITURE,
        crate::lights::leaves::FURNITURE,
        crate::minor_artifacts::leaves::FURNITURE,
        crate::new_haven_quest_rewards::leaves::FURNITURE,
        crate::plants_flowers::leaves::FURNITURE,
        crate::resources::leaves::FURNITURE,
        crate::traps::leaves::FURNITURE,
        crate::treasure_chests::leaves::FURNITURE,
    ];
    tables.into_iter().flatten().copied()
}

#[cfg(test)]
#[path = "simple_tests.rs"]
mod tests;

/// Registers every class of the folders.
pub fn register(types: &mut TypeRegistry) {
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "Server.Items.ICommodity.Configure",
        crate::commodity::configure,
    );
    crate::body_parts::register(types);
    crate::champion_artifacts::register(types);
    crate::construction::register(types);
    crate::containers::register(types);
    crate::decoration_artifacts::register(types);
    crate::deeds::register(types);
    crate::farming::register(types);
    crate::food::register(types);
    crate::lights::register(types);
    crate::minor_artifacts::register(types);
    crate::new_haven_quest_rewards::register(types);
    crate::plants_flowers::register(types);
    crate::resources::register(types);
    crate::traps::register(types);
    crate::treasure_chests::register(types);
}

// ---- Utility calls of constructors ----

/// `Utility.Random(count)`.
pub fn random(w: &mut World, count: i32) -> i32 {
    muo_primitives::utility::random(&mut w.rng, count)
}

/// `Utility.Random(from, count)`.
pub fn random_from(w: &mut World, from: i32, count: i32) -> i32 {
    muo_primitives::utility::random_from(&mut w.rng, from, count)
}

/// `Utility.RandomMinMax(min, max)`.
pub fn random_min_max(w: &mut World, min: i32, max: i32) -> i32 {
    muo_primitives::utility::random_min_max(&mut w.rng, min, max)
}

/// `Utility.RandomBool()`.
pub fn random_bool(w: &mut World) -> bool {
    muo_primitives::utility::random_bool(&mut w.rng)
}

/// One concrete weapon class derived from a weapon leaf (an artifact): the same shape as `weapon_leaf!` of
/// `weapons/mod.rs` (which is private to that module), for the `via` constructor form only:
/// `NAME, NAME_CLASS, ctor = "Full.Name" : PARENT / PARENT_CLASS, via parent_new, v0|enc;` then `@init |w, id| {..};`
/// and the `WeaponClass` overrides in declaration order.
macro_rules! weapon_art {
    (@enc enc) => { true };
    (@enc v0) => { false };
    (
        $(#[$m:meta])*
        $name:ident, $class:ident, $ctor:ident = $full:literal : $parent:ident / $pclass:ident,
        via $pnew:ident, $ver:ident;
        $(@init $init:expr;)?
        $($f:ident : $v:expr,)*
    ) => {
        /// The `override`s of the class.
        pub static $class: muo_content::items::weapon::WeaponClass =
            muo_content::items::weapon::WeaponClass { $($f: $v,)* ..$pclass };

        /// The constructor, run for the dynamic type `ty` (a derived class passes its own).
        pub fn $ctor(w: &mut World, ty: &'static muo_server::item::ItemType) -> ItemId {
            let id = $pnew(w, ty);
            $( let init: fn(&mut World, ItemId) = $init; init(w, id); )?
            id
        }

        $(#[$m])*
        pub static $name: muo_server::item::ItemType = muo_server::item::ItemType {
            full_name: $full,
            parent: Some(&$parent),
            construct: Some(|w| $ctor(w, &$name)),
            family: Some(&$class),
            serialize: |w, id, out| {
                ($parent.serialize)(w, id, out);
                $crate::simple::write_version(out, weapon_art!(@enc $ver));
            },
            deserialize: |w, id, r| {
                ($parent.deserialize)(w, id, r)?;
                let _version = $crate::simple::read_version(r, weapon_art!(@enc $ver))?;
                Ok(())
            },
            ..$parent
        };
    };
}
pub(crate) use weapon_art;
