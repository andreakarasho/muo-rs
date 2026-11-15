//! Weapon classes of polearms; see weapons/mod.rs.

use super::*;

weapon_leaf! {
    /// `Server.Items.Bardiche`.
    BARDICHE, BARDICHE_CLASS, bardiche_new = "Server.Items.Bardiche" : BASE_POLE_ARM / BASE_POLE_ARM_CLASS, graphic 0xF4D, v0;
    flippable: Some((0xF4D, 0xF4E)),
    default_weight: Some(7.0),
    primary_ability: Some(Ability::ParalyzingBlow),
    secondary_ability: Some(Ability::Dismount),
}

weapon_leaf! {
    /// `Server.Items.Halberd`.
    HALBERD, HALBERD_CLASS, halberd_new = "Server.Items.Halberd" : BASE_POLE_ARM / BASE_POLE_ARM_CLASS, graphic 0x143E, v0;
    flippable: Some((0x143E, 0x143F)),
    default_weight: Some(16.0),
    primary_ability: Some(Ability::WhirlwindAttack),
    secondary_ability: Some(Ability::ConcussionBlow),
}

weapon_leaf! {
    /// `Server.Items.Scythe`.
    SCYTHE, SCYTHE_CLASS, scythe_new = "Server.Items.Scythe" : BASE_POLE_ARM / BASE_POLE_ARM_CLASS, graphic 0x26BA, v0;
    flippable: Some((0x26BA, 0x26C4)),
    default_weight: Some(5.0),
    primary_ability: Some(Ability::BleedAttack),
    secondary_ability: Some(Ability::ParalyzingBlow),
    harvest_system: None,
}

pub fn register(types: &mut TypeRegistry) {
    types.register_item(&BARDICHE, &[]);
    types.register_item(&HALBERD, &[]);
    types.register_item(&SCYTHE, &[]);
}
