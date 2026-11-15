//! Weapon classes of spears; see weapons/mod.rs.

use super::*;

weapon_leaf! {
    /// `Server.Items.BladedStaff`.
    BLADED_STAFF, BLADED_STAFF_CLASS, bladed_staff_new = "Server.Items.BladedStaff" : BASE_SPEAR / BASE_SPEAR_CLASS, json (), v0;
}

weapon_leaf! {
    /// `Server.Items.DoubleBladedStaff`.
    DOUBLE_BLADED_STAFF, DOUBLE_BLADED_STAFF_CLASS, double_bladed_staff_new = "Server.Items.DoubleBladedStaff" : BASE_SPEAR / BASE_SPEAR_CLASS, json (), v0;
}

weapon_leaf! {
    /// `Server.Items.DualPointedSpear`.
    DUAL_POINTED_SPEAR, DUAL_POINTED_SPEAR_CLASS, dual_pointed_spear_new = "Server.Items.DualPointedSpear" : BASE_SPEAR / BASE_SPEAR_CLASS, json (), enc;
}

weapon_leaf! {
    /// `Server.Items.Pike`.
    PIKE, PIKE_CLASS, pike_new = "Server.Items.Pike" : BASE_SPEAR / BASE_SPEAR_CLASS, json (), v0;
}

weapon_leaf! {
    /// `Server.Items.Pitchfork`.
    PITCHFORK, PITCHFORK_CLASS, pitchfork_new = "Server.Items.Pitchfork" : BASE_SPEAR / BASE_SPEAR_CLASS, json (), v0;
}

weapon_leaf! {
    /// `Server.Items.ShortSpear`.
    SHORT_SPEAR, SHORT_SPEAR_CLASS, short_spear_new = "Server.Items.ShortSpear" : BASE_SPEAR / BASE_SPEAR_CLASS, json (), v0;
}

weapon_leaf! {
    /// `Server.Items.Spear`.
    SPEAR, SPEAR_CLASS, spear_new = "Server.Items.Spear" : BASE_SPEAR / BASE_SPEAR_CLASS, json (), v0;
}

weapon_leaf! {
    /// `Server.Items.TribalSpear`.
    TRIBAL_SPEAR, TRIBAL_SPEAR_CLASS, tribal_spear_new = "Server.Items.TribalSpear" : BASE_SPEAR / BASE_SPEAR_CLASS, json (), v0;
    @json;
    @item {
        default_name: |_, _| Some("a tribal spear".to_string()),
    }
}

weapon_leaf! {
    /// `Server.Items.WarFork`.
    WAR_FORK, WAR_FORK_CLASS, war_fork_new = "Server.Items.WarFork" : BASE_SPEAR / BASE_SPEAR_CLASS, json (), v0;
}

pub fn register(types: &mut TypeRegistry) {
    types.register_item(&BLADED_STAFF, &[]);
    types.register_item(&DOUBLE_BLADED_STAFF, &[]);
    types.register_item(&DUAL_POINTED_SPEAR, &[]);
    types.register_item(&PIKE, &[]);
    types.register_item(&PITCHFORK, &[]);
    types.register_item(&SHORT_SPEAR, &[]);
    types.register_item(&SPEAR, &[]);
    types.register_item(&TRIBAL_SPEAR, &[]);
    types.register_item(&WAR_FORK, &[]);
}
