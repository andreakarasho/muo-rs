//! The 8th anniversary `DupresShield`, `QuiverOfInfinity` and talismans.
//!
//! FountainOfLife and Dawn's Music Box are in `anniv2`.

use muo_content::aos::{AosAttribute, AttrKind};
use muo_content::items::armor::{ArmorClass, BASE_SHIELD, BASE_SHIELD_CLASS};
use muo_content::items::set_attr;
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, LootType, ops};
use muo_server::layer::Layer;
use muo_server::type_registry::TypeRegistry;

use crate::armor::armor_leaf;

armor_leaf! {
    /// `Server.Items.DupresShield`.
    DUPRES_SHIELD, DUPRES_SHIELD_CLASS, dupres_shield_new = "Server.Items.DupresShield" : BASE_SHIELD / BASE_SHIELD_CLASS, crate::armor::new_armor_json, encoded true;
    @template;
}

/// `QuiverOfInfinity()`.
pub fn quiver_of_infinity_new(w: &mut World) -> ItemId {
    let id = crate::quivers::new_quiver(w, &QUIVER_OF_INFINITY, 0x2B02);
    ops::set_loot_type(w, id, LootType::Blessed);
    crate::quivers::set_weight_reduction(w, id, 30);
    crate::quivers::set_lower_ammo_cost(w, id, 20);
    set_attr(
        w,
        id,
        AttrKind::Attributes,
        AosAttribute::DefendChance as u32,
        5,
    );
    id
}

/// `Server.Items.QuiverOfInfinity`.
pub static QUIVER_OF_INFINITY: ItemType = ItemType {
    full_name: "Server.Items.QuiverOfInfinity",
    parent: Some(&crate::quivers::BASE_QUIVER),
    construct: Some(quiver_of_infinity_new),
    default_weight: crate::class_props::default_weight,
    label_number: crate::class_props::label_number, // Quiver of Infinity
    serialize: |w, id, out| {
        (crate::quivers::BASE_QUIVER.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (crate::quivers::BASE_QUIVER.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        Ok(())
    },
    ..crate::quivers::BASE_QUIVER
};

// ---- the form talismans ----

/// `TalismanForm`: the cliloc of each form's name.
pub struct FormClass {
    pub form: i32,
}

/// `BaseFormTalisman()`.
pub fn base_form_talisman_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0x2F59);
    ops::set_loot_type(w, id, LootType::Blessed);
    (ty.set_layer)(w, id, Layer::Talisman);
    id
}

fn form_of(w: &World, id: ItemId) -> i32 {
    w.item(id)
        .and_then(|i| i.ty.family::<FormClass>())
        .map_or(1031671, |c| c.form) // Squirrel
}

/// `Server.Items.BaseFormTalisman`.
pub static BASE_FORM_TALISMAN: ItemType = ItemType {
    full_name: "Server.Items.BaseFormTalisman",
    parent: Some(&ITEM_BASE),
    construct: None, // no [Constructible]
    default_weight: crate::class_props::default_weight,
    add_name_property: |w, id, list| list.add_str(1075200, &format!("#{}", form_of(w, id))),
    on_removed: |w, id, parent| {
        (ITEM_BASE.on_removed)(w, id, parent);
        if let muo_server::ids::EntityId::Mobile(m) = parent {
            muo_content::player_mobile::hooks::animal_form_remove_context(w, m);
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        Ok(())
    },
    ..ITEM_BASE
};

macro_rules! form_talisman {
    ($($(#[$m:meta])* $name:ident, $class:ident = $full:literal, $form:expr;)*) => {$(
        static $class: FormClass = FormClass { form: $form };
        $(#[$m])*
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&BASE_FORM_TALISMAN),
            construct: Some(|w| base_form_talisman_new(w, &$name)),
            family: Some(&$class),
            serialize: |w, id, out| {
                (BASE_FORM_TALISMAN.serialize)(w, id, out);
                out.write_encoded_int(0);
            },
            deserialize: |w, id, r| {
                (BASE_FORM_TALISMAN.deserialize)(w, id, r)?;
                let _version = r.read_encoded_int()?;
                Ok(())
            },
            ..BASE_FORM_TALISMAN
        };
    )*};
}

form_talisman! {
    /// `Server.Items.FerretFormTalisman`.
    FERRET_FORM_TALISMAN, FERRET_CLASS = "Server.Items.FerretFormTalisman", 1031672;
    /// `Server.Items.SquirrelFormTalisman`.
    SQUIRREL_FORM_TALISMAN, SQUIRREL_CLASS = "Server.Items.SquirrelFormTalisman", 1031671;
    /// `Server.Items.CuSidheFormTalisman`.
    CU_SIDHE_FORM_TALISMAN, CU_SIDHE_CLASS = "Server.Items.CuSidheFormTalisman", 1031670;
    /// `Server.Items.ReptalonFormTalisman`.
    REPTALON_FORM_TALISMAN, REPTALON_CLASS = "Server.Items.ReptalonFormTalisman", 1075202;
}

/// `BaseFormTalisman.EntryEnabled(m, type)`: the talisman worn must be the form's own.
pub fn entry_enabled(w: &World, m: MobileId, type_name: &str) -> bool {
    let want = match type_name {
        "Squirrel" => &SQUIRREL_FORM_TALISMAN,
        "Ferret" => &FERRET_FORM_TALISMAN,
        "CuSidhe" => &CU_SIDHE_FORM_TALISMAN,
        "Reptalon" => &REPTALON_FORM_TALISMAN,
        _ => return true,
    };
    muo_server::mobile::part2::items::find_item_on_layer(w, m, Layer::Talisman)
        .and_then(|i| w.item(i))
        .is_some_and(|i| i.ty.is(want))
}

pub fn register(types: &mut TypeRegistry) {
    for t in [
        &DUPRES_SHIELD,
        &QUIVER_OF_INFINITY,
        &BASE_FORM_TALISMAN,
        &FERRET_FORM_TALISMAN,
        &SQUIRREL_FORM_TALISMAN,
        &CU_SIDHE_FORM_TALISMAN,
        &REPTALON_FORM_TALISMAN,
    ] {
        types.register_item(t, &[]);
    }
}
