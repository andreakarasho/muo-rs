//! Weapon classes of wooden; see weapons/mod.rs.

use super::*;

weapon_leaf! {
    /// `Server.Items.AncientWildStaff`.
    ANCIENT_WILD_STAFF, ANCIENT_WILD_STAFF_CLASS, ancient_wild_staff_new = "Server.Items.AncientWildStaff" : WILD_STAFF / WILD_STAFF_CLASS, via wild_staff_new, enc;
    @init |w, id| {
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::ResistPoisonBonus as u32, 5);
    };
    label_number: 1073550,
}

weapon_leaf! {
    /// `Server.Items.ArcanistsWildStaff`.
    ARCANISTS_WILD_STAFF, ARCANISTS_WILD_STAFF_CLASS, arcanists_wild_staff_new = "Server.Items.ArcanistsWildStaff" : WILD_STAFF / WILD_STAFF_CLASS, via wild_staff_new, enc;
    @init |w, id| {
        set_attr(w, id, AttrKind::Attributes, AosAttribute::BonusMana as u32, 3);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponDamage as u32, 3);
    };
    label_number: 1073549,
}

weapon_leaf! {
    /// `Server.Items.HardenedWildStaff`.
    HARDENED_WILD_STAFF, HARDENED_WILD_STAFF_CLASS, hardened_wild_staff_new = "Server.Items.HardenedWildStaff" : WILD_STAFF / WILD_STAFF_CLASS, via wild_staff_new, enc;
    @init |w, id| {
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponDamage as u32, 5);
    };
    label_number: 1073552,
}

weapon_leaf! {
    /// `Server.Items.ThornedWildStaff`.
    THORNED_WILD_STAFF, THORNED_WILD_STAFF_CLASS, thorned_wild_staff_new = "Server.Items.ThornedWildStaff" : WILD_STAFF / WILD_STAFF_CLASS, via wild_staff_new, enc;
    @init |w, id| {
        set_attr(w, id, AttrKind::Attributes, AosAttribute::ReflectPhysical as u32, 12);
    };
    label_number: 1073551,
}

pub fn register(types: &mut TypeRegistry) {
    types.register_item(&ANCIENT_WILD_STAFF, &[]);
    types.register_item(&ARCANISTS_WILD_STAFF, &[]);
    types.register_item(&HARDENED_WILD_STAFF, &[]);
    types.register_item(&THORNED_WILD_STAFF, &[]);
}
