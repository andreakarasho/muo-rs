//! The concrete jewel classes (not `BaseJewel` and the abstract
//! `BaseRing`, `BaseBracelet`, `BaseEarrings`, `BaseNecklace`, which are in `muo_content::items::jewel`), and
//! `Beads`. One `jewel_leaf!` call per class; `leaves.rs` holds the table classes.
//! `GoldRing` and `Necklace` are the samples of `muo_content::items::leaves` (registered there).

use muo_server::type_registry::TypeRegistry;

/// One concrete jewel class: `NAME, NAME_CLASS, name_new = "Server.Items.Foo" : PARENT / PARENT_CLASS, base,
/// encoded <bool>;` and then `@init`, `@ty { .. }` and the `field: value,` lines as in `clothing_leaf!`.
macro_rules! jewel_leaf {
    // `@template;`: the rest of the constructor is the class's item template (`Data/Templates/Items`).
    (
        $(#[$m:meta])*
        $name:ident, $class:ident, $ctor:ident = $full:literal : $parent:ident / $pclass:ident, $base:expr,
        encoded $enc:literal;
        @template;
        $($rest:tt)*
    ) => {
        $crate::jewels::jewel_leaf! {
            $(#[$m])*
            $name, $class, $ctor = $full : $parent / $pclass, $base, encoded $enc;
            @init |w, id| ::muo_content::templates::apply_item_as(w, id, $full);
            $($rest)*
        }
    };
    (
        $(#[$m:meta])*
        $name:ident, $class:ident, $ctor:ident = $full:literal : $parent:ident / $pclass:ident, $base:expr,
        encoded $enc:literal;
        $(@init $init:expr;)?
        $(@ty { $($tf:ident : $tv:expr,)* };)?
        $($f:ident : $v:expr,)*
    ) => {
        /// The `override`s of the class.
        pub static $class: ::muo_content::items::jewel::JewelClass = ::muo_content::items::jewel::JewelClass { $($f: $v,)* ..$pclass };

        /// `new T()` for class `ty` (a derived class passes its own).
        pub fn $ctor(w: &mut ::muo_server::World, ty: &'static ::muo_server::item::ItemType) -> ::muo_server::ids::ItemId {
            let base: fn(&mut ::muo_server::World, &'static ::muo_server::item::ItemType) -> ::muo_server::ids::ItemId = $base;
            let id = base(w, ty);
            $( let init: fn(&mut ::muo_server::World, ::muo_server::ids::ItemId) = $init; init(w, id); )?
            id
        }

        $(#[$m])*
        pub static $name: ::muo_server::item::ItemType = ::muo_server::item::ItemType {
            full_name: $full,
            parent: Some(&$parent),
            construct: Some(|w| $ctor(w, &$name)),
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
pub(crate) use jewel_leaf;

pub mod leaves;
pub mod special;

/// `new BaseJewel(itemID)` with the class's `graphic` prop as the item id.
pub(crate) fn new_jewel_json(
    w: &mut muo_server::World,
    ty: &'static muo_server::item::ItemType,
) -> muo_server::ids::ItemId {
    let g = muo_content::items::equip_props::graphic(w, ty);
    muo_content::items::jewel::new_jewel(w, ty, g)
}

/// Registers every jewel class of this crate.
pub fn register(types: &mut TypeRegistry) {
    leaves::register(types);
    special::register(types);
}
