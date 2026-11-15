//! Weapon classes of polearms; see weapons/mod.rs.

use super::*;

weapon_leaf! {
    /// `Server.Items.Bardiche`.
    BARDICHE, BARDICHE_CLASS, bardiche_new = "Server.Items.Bardiche" : BASE_POLE_ARM / BASE_POLE_ARM_CLASS, json (), v0;
}

weapon_leaf! {
    /// `Server.Items.Halberd`.
    HALBERD, HALBERD_CLASS, halberd_new = "Server.Items.Halberd" : BASE_POLE_ARM / BASE_POLE_ARM_CLASS, json (), v0;
}

weapon_leaf! {
    /// `Server.Items.Scythe`.
    SCYTHE, SCYTHE_CLASS, scythe_new = "Server.Items.Scythe" : BASE_POLE_ARM / BASE_POLE_ARM_CLASS, json (), v0;
}

pub fn register(types: &mut TypeRegistry) {
    types.register_item(&BARDICHE, &[]);
    types.register_item(&HALBERD, &[]);
    types.register_item(&SCYTHE, &[]);
}
