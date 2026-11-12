//! Weapon classes of polearms; see weapons/mod.rs.

use super::*;

weapon_leaf! {
    /// `Server.Items.Bardiche`.
    BARDICHE, BARDICHE_CLASS, bardiche_new = "Server.Items.Bardiche" : BASE_POLE_ARM / BASE_POLE_ARM_CLASS, graphic 0xF4D, v0;
    flippable: Some((0xF4D, 0xF4E)),
    default_weight: Some(7.0),
    primary_ability: Some(Ability::ParalyzingBlow),
    secondary_ability: Some(Ability::Dismount),
    aos_str_req: 45,
    aos_min_damage: same(17),
    aos_max_damage: same(18),
    aos_speed: 28,
    ml_speed: 3.75,
    old_str_req: 40,
    old_min_damage: 5,
    old_max_damage: 43,
    old_speed: 26,
    init_min_hits: 31,
    init_max_hits: 100,
}

weapon_leaf! {
    /// `Server.Items.Halberd`.
    HALBERD, HALBERD_CLASS, halberd_new = "Server.Items.Halberd" : BASE_POLE_ARM / BASE_POLE_ARM_CLASS, graphic 0x143E, v0;
    flippable: Some((0x143E, 0x143F)),
    default_weight: Some(16.0),
    primary_ability: Some(Ability::WhirlwindAttack),
    secondary_ability: Some(Ability::ConcussionBlow),
    aos_str_req: 95,
    aos_min_damage: same(18),
    aos_max_damage: same(19),
    aos_speed: 25,
    ml_speed: 4.25,
    old_str_req: 45,
    old_min_damage: 5,
    old_max_damage: 49,
    old_speed: 25,
    init_min_hits: 31,
    init_max_hits: 80,
}

weapon_leaf! {
    /// `Server.Items.Scythe`.
    SCYTHE, SCYTHE_CLASS, scythe_new = "Server.Items.Scythe" : BASE_POLE_ARM / BASE_POLE_ARM_CLASS, graphic 0x26BA, v0;
    flippable: Some((0x26BA, 0x26C4)),
    default_weight: Some(5.0),
    primary_ability: Some(Ability::BleedAttack),
    secondary_ability: Some(Ability::ParalyzingBlow),
    aos_str_req: 45,
    aos_min_damage: same(15),
    aos_max_damage: same(18),
    aos_speed: 32,
    ml_speed: 3.50,
    old_str_req: 45,
    old_min_damage: 15,
    old_max_damage: 18,
    old_speed: 32,
    init_min_hits: 31,
    init_max_hits: 100,
    harvest_system: None,
}

pub fn register(types: &mut TypeRegistry) {
    types.register_item(&BARDICHE, &[]);
    types.register_item(&HALBERD, &[]);
    types.register_item(&SCYTHE, &[]);
}
