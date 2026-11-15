//! Concrete weapon and armor classes: the pattern, shown on three real weapons and three real armors.
//! To add one, copy the nearest class below.
//!
//! A leaf is one `weapon_leaf!`/`armor_leaf!`/`clothing_leaf!`/`jewel_leaf!` call. Its data is its item template
//! (`Data/Templates/Items/samples/leaves.json`): the `graphic` prop is what the constructor passes to `base(itemID)`,
//! the other props are the constant `override`s (the class table field in camelCase), and the template's keys are the
//! rest of the constructor. Fields of the call are `override`s that are code (fn pointers of [`WeaponClass`],
//! [`ArmorClass`], ..). The save part is generated: every `[SerializationGenerator(0, false)]` leaf with no fields of
//! its own is `parent part + version int 0`.

use muo_server::type_registry::TypeRegistry;

use super::armor::{
    ArmorClass, BASE_ARMOR, BASE_ARMOR_CLASS, BASE_SHIELD, BASE_SHIELD_CLASS, new_armor,
};
use super::clothing::{
    BASE_HAT, BASE_HAT_CLASS, BASE_SHIRT, BASE_SHIRT_CLASS, ClothingClass, new_clothing,
};
use super::jewel::{
    BASE_NECKLACE, BASE_NECKLACE_CLASS, BASE_RING, BASE_RING_CLASS, JewelClass, new_jewel,
};
use super::weapon::{
    BASE_RANGED, BASE_RANGED_CLASS, BASE_SWORD, BASE_SWORD_CLASS, WeaponClass, new_weapon,
};

/// One concrete weapon class.
macro_rules! weapon_leaf {
    (
        $(#[$m:meta])*
        $name:ident, $class:ident = $full:literal : $parent:ident / $pclass:ident;
        $($f:ident : $v:expr,)*
    ) => {
        /// The `override`s of the class.
        pub static $class: WeaponClass = WeaponClass { $($f: $v,)* ..$pclass };

        $(#[$m])*
        pub static $name: muo_server::item::ItemType = muo_server::item::ItemType {
            full_name: $full,
            parent: Some(&$parent),
            construct: Some(|w| {
                let g = super::equip_props::graphic(w, &$name);
                let id = new_weapon(w, &$name, g);
                crate::templates::apply_item_as(w, id, $full);
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
        $name:ident, $class:ident = $full:literal : $parent:ident / $pclass:ident;
        $($f:ident : $v:expr,)*
    ) => {
        /// The `override`s of the class.
        pub static $class: ArmorClass = ArmorClass { $($f: $v,)* ..$pclass };

        $(#[$m])*
        pub static $name: muo_server::item::ItemType = muo_server::item::ItemType {
            full_name: $full,
            parent: Some(&$parent),
            construct: Some(|w| {
                let g = super::equip_props::graphic(w, &$name);
                let id = new_armor(w, &$name, g);
                crate::templates::apply_item_as(w, id, $full);
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
    LONGSWORD, LONGSWORD_CLASS = "Server.Items.Longsword" : BASE_SWORD / BASE_SWORD_CLASS;
}

weapon_leaf! {
    /// `Server.Items.Katana`.
    KATANA, KATANA_CLASS = "Server.Items.Katana" : BASE_SWORD / BASE_SWORD_CLASS;
}

weapon_leaf! {
    /// `Server.Items.Bow` (its template sets the layer and the resource).
    BOW, BOW_CLASS = "Server.Items.Bow" : BASE_RANGED / BASE_RANGED_CLASS;
}

/// One concrete clothing class; the constructor's `hue` argument defaults to 0 (what `[Constructible]` makes), the layer comes from the abstract parent.
macro_rules! clothing_leaf {
    (
        $(#[$m:meta])*
        $name:ident, $class:ident = $full:literal : $parent:ident / $pclass:ident;
        $($f:ident : $v:expr,)*
    ) => {
        /// The `override`s of the class.
        pub static $class: ClothingClass = ClothingClass { $($f: $v,)* ..$pclass };

        $(#[$m])*
        pub static $name: muo_server::item::ItemType = muo_server::item::ItemType {
            full_name: $full,
            parent: Some(&$parent),
            construct: Some(|w| {
                let g = super::equip_props::graphic(w, &$name);
                let id = new_clothing(w, &$name, g, 0);
                crate::templates::apply_item_as(w, id, $full);
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
        $name:ident, $class:ident = $full:literal : $parent:ident / $pclass:ident;
        $($f:ident : $v:expr,)*
    ) => {
        /// The `override`s of the class.
        pub static $class: JewelClass = JewelClass { $($f: $v,)* ..$pclass };

        $(#[$m])*
        pub static $name: muo_server::item::ItemType = muo_server::item::ItemType {
            full_name: $full,
            parent: Some(&$parent),
            construct: Some(|w| {
                let g = super::equip_props::graphic(w, &$name);
                let id = new_jewel(w, &$name, g);
                crate::templates::apply_item_as(w, id, $full);
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
    BUCKLER, BUCKLER_CLASS = "Server.Items.Buckler" : BASE_SHIELD / BASE_SHIELD_CLASS;
}

clothing_leaf! {
    /// `Server.Items.Cap` (a `BaseHat`; the resistances are its own).
    CAP, CAP_CLASS = "Server.Items.Cap" : BASE_HAT / BASE_HAT_CLASS;
}

clothing_leaf! {
    /// `Server.Items.FancyShirt`.
    FANCY_SHIRT, FANCY_SHIRT_CLASS = "Server.Items.FancyShirt" : BASE_SHIRT / BASE_SHIRT_CLASS;
}

jewel_leaf! {
    /// `Server.Items.GoldRing`.
    GOLD_RING, GOLD_RING_CLASS = "Server.Items.GoldRing" : BASE_RING / BASE_RING_CLASS;
}

jewel_leaf! {
    /// `Server.Items.Necklace`.
    NECKLACE, NECKLACE_CLASS = "Server.Items.Necklace" : BASE_NECKLACE / BASE_NECKLACE_CLASS;
}

armor_leaf! {
    /// `Server.Items.PlateChest`.
    PLATE_CHEST, PLATE_CHEST_CLASS = "Server.Items.PlateChest" : BASE_ARMOR / BASE_ARMOR_CLASS;
}

armor_leaf! {
    /// `Server.Items.LeatherChest`.
    LEATHER_CHEST, LEATHER_CHEST_CLASS = "Server.Items.LeatherChest" : BASE_ARMOR / BASE_ARMOR_CLASS;
}

armor_leaf! {
    /// `Server.Items.ChainChest`.
    CHAIN_CHEST, CHAIN_CHEST_CLASS = "Server.Items.ChainChest" : BASE_ARMOR / BASE_ARMOR_CLASS;
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
