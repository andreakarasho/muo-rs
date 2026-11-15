//! Concrete weapon and armor classes: the pattern, shown on three real weapons and three real armors.
//! To add one, copy the nearest class below.
//!
//! A leaf is one `weapon_leaf!`/`armor_leaf!`/`clothing_leaf!`/`jewel_leaf!` call. Its fields are the class's `override`s, in declaration order,
//! under the Rust names of [`WeaponClass`]/[`ArmorClass`]; what the constructor does beyond
//! `base(itemID)` is the optional `@init |w, id| { .. };` closure, before the fields. The save part is generated: every
//! `[SerializationGenerator(0, false)]` leaf with no fields of its own is `parent part + version int 0`.

use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::type_registry::TypeRegistry;

use super::armor::{
    ArmorClass, ArmorMaterialType, ArmorMeditationAllowance, BASE_ARMOR, BASE_ARMOR_CLASS,
    BASE_SHIELD, BASE_SHIELD_CLASS, new_armor,
};
use super::clothing::{
    BASE_HAT, BASE_HAT_CLASS, BASE_SHIRT, BASE_SHIRT_CLASS, ClothingClass, new_clothing,
};
use super::jewel::{
    BASE_NECKLACE, BASE_NECKLACE_CLASS, BASE_RING, BASE_RING_CLASS, JewelClass, new_jewel,
};
use super::same;
use super::weapon::{
    Ability, BASE_RANGED, BASE_RANGED_CLASS, BASE_SWORD, BASE_SWORD_CLASS, WeaponAnimation,
    WeaponClass, new_weapon, set_resource,
};
use crate::craft::CraftResource;

/// One concrete weapon class. `$id` is the graphic the constructor passes to `base(..)`.
macro_rules! weapon_leaf {
    (
        $(#[$m:meta])*
        $name:ident, $class:ident = $full:literal : $parent:ident / $pclass:ident, graphic $gid:expr;
        $(@init $init:expr;)?
        $($f:ident : $v:expr,)*
    ) => {
        /// The `override`s of the class.
        pub static $class: WeaponClass = WeaponClass { $($f: $v,)* ..$pclass };

        $(#[$m])*
        pub static $name: muo_server::item::ItemType = muo_server::item::ItemType {
            full_name: $full,
            parent: Some(&$parent),
            construct: Some(|w| {
                let id = new_weapon(w, &$name, $gid);
                $( let init: fn(&mut World, ItemId) = $init; init(w, id); )?
                id
            }),
            family: Some(&$class),
            serialize: |w, id, out| {
                ($parent.serialize)(w, id, out);
                out.write_i32(0); // [SerializationGenerator(0, false)]: version only
            },
            deserialize: |w, id, r| {
                ($parent.deserialize)(w, id, r)?;
                let _version = r.read_i32()?;
                Ok(())
            },
            ..$parent
        };
    };
}

/// One concrete armor class (clothing and jewels get the same shape).
macro_rules! armor_leaf {
    (
        $(#[$m:meta])*
        $name:ident, $class:ident = $full:literal : $parent:ident / $pclass:ident, graphic $gid:expr;
        $(@init $init:expr;)?
        $($f:ident : $v:expr,)*
    ) => {
        /// The `override`s of the class.
        pub static $class: ArmorClass = ArmorClass { $($f: $v,)* ..$pclass };

        $(#[$m])*
        pub static $name: muo_server::item::ItemType = muo_server::item::ItemType {
            full_name: $full,
            parent: Some(&$parent),
            construct: Some(|w| {
                let id = new_armor(w, &$name, $gid);
                $( let init: fn(&mut World, ItemId) = $init; init(w, id); )?
                id
            }),
            family: Some(&$class),
            serialize: |w, id, out| {
                ($parent.serialize)(w, id, out);
                out.write_i32(0);
            },
            deserialize: |w, id, r| {
                ($parent.deserialize)(w, id, r)?;
                let _version = r.read_i32()?;
                Ok(())
            },
            ..$parent
        };
    };
}

weapon_leaf! {
    /// `Server.Items.Longsword`.
    LONGSWORD, LONGSWORD_CLASS = "Server.Items.Longsword" : BASE_SWORD / BASE_SWORD_CLASS, graphic 0xF61;
    default_weight: Some(7.0),
    primary_ability: Some(Ability::ArmorIgnore),
    secondary_ability: Some(Ability::ConcussionBlow),
}

weapon_leaf! {
    /// `Server.Items.Katana`.
    KATANA, KATANA_CLASS = "Server.Items.Katana" : BASE_SWORD / BASE_SWORD_CLASS, graphic 0x13FF;
    default_weight: Some(6.0),
    primary_ability: Some(Ability::DoubleStrike),
    secondary_ability: Some(Ability::ArmorIgnore),
}

weapon_leaf! {
    /// `Server.Items.Bow` (its constructor sets the layer and the resource).
    BOW, BOW_CLASS = "Server.Items.Bow" : BASE_RANGED / BASE_RANGED_CLASS, graphic 0x13B2;
    @init |w, id| {
        // Layer = Layer.TwoHanded; Resource = CraftResource.RegularWood;
        if let Some(ty) = w.item(id).map(|i| i.ty) {
            (ty.set_layer)(w, id, muo_server::layer::Layer::TwoHanded);
        }
        set_resource(w, id, CraftResource::RegularWood);
    };
    default_weight: Some(6.0),
    effect_id: 0xF42,
    ammo_type: Some("Server.Items.Arrow"),
    primary_ability: Some(Ability::ParalyzingBlow),
    secondary_ability: Some(Ability::MortalStrike),
    def_animation: WeaponAnimation::ShootBow,
}

/// One concrete clothing class. `$gid` is the graphic the constructor passes to `base(..)`; the constructor's
/// `hue` argument defaults to 0 (what `[Constructible]` makes), the layer comes from the abstract parent.
macro_rules! clothing_leaf {
    (
        $(#[$m:meta])*
        $name:ident, $class:ident = $full:literal : $parent:ident / $pclass:ident, graphic $gid:expr;
        $(@init $init:expr;)?
        $($f:ident : $v:expr,)*
    ) => {
        /// The `override`s of the class.
        pub static $class: ClothingClass = ClothingClass { $($f: $v,)* ..$pclass };

        $(#[$m])*
        pub static $name: muo_server::item::ItemType = muo_server::item::ItemType {
            full_name: $full,
            parent: Some(&$parent),
            construct: Some(|w| {
                let id = new_clothing(w, &$name, $gid, 0);
                $( let init: fn(&mut World, ItemId) = $init; init(w, id); )?
                id
            }),
            family: Some(&$class),
            serialize: |w, id, out| {
                ($parent.serialize)(w, id, out);
                out.write_i32(0);
            },
            deserialize: |w, id, r| {
                ($parent.deserialize)(w, id, r)?;
                let _version = r.read_i32()?;
                Ok(())
            },
            ..$parent
        };
    };
}

/// One concrete jewel class; the layer and the gem name base come from the abstract parent.
macro_rules! jewel_leaf {
    (
        $(#[$m:meta])*
        $name:ident, $class:ident = $full:literal : $parent:ident / $pclass:ident, graphic $gid:expr;
        $(@init $init:expr;)?
        $($f:ident : $v:expr,)*
    ) => {
        /// The `override`s of the class.
        pub static $class: JewelClass = JewelClass { $($f: $v,)* ..$pclass };

        $(#[$m])*
        pub static $name: muo_server::item::ItemType = muo_server::item::ItemType {
            full_name: $full,
            parent: Some(&$parent),
            construct: Some(|w| {
                let id = new_jewel(w, &$name, $gid);
                $( let init: fn(&mut World, ItemId) = $init; init(w, id); )?
                id
            }),
            family: Some(&$class),
            serialize: |w, id, out| {
                ($parent.serialize)(w, id, out);
                out.write_i32(0);
            },
            deserialize: |w, id, r| {
                ($parent.deserialize)(w, id, r)?;
                let _version = r.read_i32()?;
                Ok(())
            },
            ..$parent
        };
    };
}

armor_leaf! {
    /// `Server.Items.Buckler`, the sample shield (`BaseShield` replaces `ArmorRating` and `OnHit`).
    BUCKLER, BUCKLER_CLASS = "Server.Items.Buckler" : BASE_SHIELD / BASE_SHIELD_CLASS, graphic 0x1B73;
    default_weight: Some(5.0),
}

clothing_leaf! {
    /// `Server.Items.Cap` (a `BaseHat`; the resistances are its own).
    CAP, CAP_CLASS = "Server.Items.Cap" : BASE_HAT / BASE_HAT_CLASS, graphic 0x1715;
    default_weight: Some(1.0),
    base_physical_resistance: 0,
    base_fire_resistance: 5,
    base_cold_resistance: 9,
    base_poison_resistance: 5,
    base_energy_resistance: 5,
    init_min_hits: same(20),
    init_max_hits: same(30),
}

clothing_leaf! {
    /// `Server.Items.FancyShirt`.
    FANCY_SHIRT, FANCY_SHIRT_CLASS = "Server.Items.FancyShirt" : BASE_SHIRT / BASE_SHIRT_CLASS, graphic 0x1EFD;
    default_weight: Some(2.0),
}

jewel_leaf! {
    /// `Server.Items.GoldRing`.
    GOLD_RING, GOLD_RING_CLASS = "Server.Items.GoldRing" : BASE_RING / BASE_RING_CLASS, graphic 0x108A;
    default_weight: Some(0.1),
}

jewel_leaf! {
    /// `Server.Items.Necklace`.
    NECKLACE, NECKLACE_CLASS = "Server.Items.Necklace" : BASE_NECKLACE / BASE_NECKLACE_CLASS, graphic 0x1085;
    default_weight: Some(0.1),
}

armor_leaf! {
    /// `Server.Items.PlateChest`.
    PLATE_CHEST, PLATE_CHEST_CLASS = "Server.Items.PlateChest" : BASE_ARMOR / BASE_ARMOR_CLASS, graphic 0x1415;
    default_weight: Some(10.0),
    material_type: ArmorMaterialType::Plate,
}

armor_leaf! {
    /// `Server.Items.LeatherChest`.
    LEATHER_CHEST, LEATHER_CHEST_CLASS = "Server.Items.LeatherChest" : BASE_ARMOR / BASE_ARMOR_CLASS, graphic 0x13CC;
    default_weight: Some(6.0),
    material_type: ArmorMaterialType::Leather,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::All,
}

armor_leaf! {
    /// `Server.Items.ChainChest`.
    CHAIN_CHEST, CHAIN_CHEST_CLASS = "Server.Items.ChainChest" : BASE_ARMOR / BASE_ARMOR_CLASS, graphic 0x13BF;
    default_weight: Some(7.0),
    material_type: ArmorMaterialType::Chainmail,
}

/// Registers the leaves above (and the pattern for the rest: one line per class, in the leaf's file order).
pub fn register(types: &mut TypeRegistry) {
    for t in [
        &LONGSWORD,
        &KATANA,
        &BOW,
        &PLATE_CHEST,
        &LEATHER_CHEST,
        &CHAIN_CHEST,
        &BUCKLER,
        &CAP,
        &FANCY_SHIRT,
        &GOLD_RING,
        &NECKLACE,
    ] {
        types.register_item(t, &[]);
    }
    super::weapon::register(types);
}
