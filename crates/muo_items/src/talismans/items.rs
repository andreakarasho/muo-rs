//! The four plain items that are talisman reagents and `RunedSwitch`, which
//! recharges a talisman. Graphics, weights and labels are their templates (`Data/Scripts/classes/items/talismans`).

use std::rc::Rc;

use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, deps, ops};
use muo_server::target::{self, TargetFlags, Targeted};
use muo_server::type_registry::TypeRegistry;

macro_rules! reagent_leaves {
    ($($(#[$m:meta])* $name:ident = $full:literal;)*) => {
        $(
            $(#[$m])*
            pub static $name: ItemType = ItemType {
                full_name: $full,
                parent: Some(&ITEM_BASE),
                construct: Some(|w| crate::simple::from_template(w, &$name, $full)),
                default_weight: crate::class_props::default_weight,
                label_number: crate::class_props::label_number,
                serialize: |w, id, out| {
                    (ITEM_BASE.serialize)(w, id, out);
                    out.write_i32(0); // [SerializationGenerator(0, false)]
                },
                deserialize: |w, id, r| {
                    (ITEM_BASE.deserialize)(w, id, r)?;
                    let _version = r.read_i32()?;
                    Ok(())
                },
                ..ITEM_BASE
            };
        )*
    };
}

reagent_leaves! {
    /// `Server.Items.EnchantedSwitch`.
    ENCHANTED_SWITCH = "Server.Items.EnchantedSwitch"; // enchanted switch
    /// `Server.Items.HollowPrism`.
    HOLLOW_PRISM = "Server.Items.HollowPrism"; // hollow prism
    /// `Server.Items.JeweledFiligree`.
    JEWELED_FILIGREE = "Server.Items.JeweledFiligree"; // jeweled filigree
    /// `Server.Items.RunedPrism`.
    RUNED_PRISM = "Server.Items.RunedPrism"; // runed prism
}

fn on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    let pack = muo_server::mobile::part1::props_b::backpack(w, from);
    if ops::is_child_of(w, id, pack.map(EntityId::Item)) {
        deps::send_localized_message(w, from, 1075101); // Please select an item to recharge.
        let mut t = target::Target::new(w, 0, false, TargetFlags::NONE);
        t.handlers.on_target = Some(Rc::new(move |w: &mut World, from, targeted| {
            on_target(w, id, from, targeted)
        }));
        let tid = target::add_target(w, t);
        target::set_mobile_target(w, from, Some(tid));
    } else {
        deps::send_localized_message(w, from, 1060640); // The item must be in your backpack to use it.
    }
}

/// `InternalTarget.OnTarget`.
fn on_target(w: &mut World, switch: ItemId, from: MobileId, o: Targeted) {
    if ops::is_deleted(w, switch) {
        return;
    }
    match o {
        Targeted::Entity(EntityId::Item(t)) if super::is_talisman(w, t) => {
            let (charges, max) = super::data(w, t).map_or((0, 0), |d| (d.charges, d.max_charges));
            if charges == 0 {
                super::set_charges(w, t, max);
                if let Some(ty) = w.item(switch).map(|i| i.ty) {
                    (ty.delete)(w, switch);
                }
                deps::send_localized_message(w, from, 1075100); // The item has been recharged.
            } else {
                // You cannot recharge that item until all of its current charges have been used.
                deps::send_localized_message(w, from, 1075099);
            }
        }
        _ => deps::send_localized_message(w, from, 1046439), // That is not a valid target.
    }
}

/// `Server.Items.RunedSwitch`.
pub static RUNED_SWITCH: ItemType = ItemType {
    full_name: "Server.Items.RunedSwitch",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| crate::simple::from_template(w, &RUNED_SWITCH, "Server.Items.RunedSwitch")),
    default_weight: crate::class_props::default_weight,
    label_number: crate::class_props::label_number, // runed switch
    on_double_click,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..ITEM_BASE
};

/// Registers the classes of this file.
pub fn register(types: &mut TypeRegistry) {
    for t in [
        &ENCHANTED_SWITCH,
        &HOLLOW_PRISM,
        &JEWELED_FILIGREE,
        &RUNED_PRISM,
        &RUNED_SWITCH,
    ] {
        types.register_item(t, &[]);
    }
}
