//! Weapon classes of knives; see weapons/mod.rs.

use super::*;

weapon_leaf! {
    /// `Server.Items.ButcherKnife`.
    BUTCHER_KNIFE, BUTCHER_KNIFE_CLASS, butcher_knife_new = "Server.Items.ButcherKnife" : BASE_KNIFE / BASE_KNIFE_CLASS, json (), v0;
}

weapon_leaf! {
    /// `Server.Items.Cleaver`.
    CLEAVER, CLEAVER_CLASS, cleaver_new = "Server.Items.Cleaver" : BASE_KNIFE / BASE_KNIFE_CLASS, json (), v0;
}

weapon_leaf! {
    /// `Server.Items.Dagger`.
    DAGGER, DAGGER_CLASS, dagger_new = "Server.Items.Dagger" : BASE_KNIFE / BASE_KNIFE_CLASS, json (), v0;
}

weapon_leaf! {
    /// `Server.Items.SkinningKnife`.
    SKINNING_KNIFE, SKINNING_KNIFE_CLASS, skinning_knife_new = "Server.Items.SkinningKnife" : BASE_KNIFE / BASE_KNIFE_CLASS, json (), v0;
}

pub fn register(types: &mut TypeRegistry) {
    types.register_item(&BUTCHER_KNIFE, &[]);
    types.register_item(&CLEAVER, &[]);
    types.register_item(&DAGGER, &[]);
    types.register_item(&SKINNING_KNIFE, &[]);
}
