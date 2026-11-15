//! Weapon classes of knives; see weapons/mod.rs.

use super::*;

weapon_leaf! {
    /// `Server.Items.ButcherKnife`.
    BUTCHER_KNIFE, BUTCHER_KNIFE_CLASS, butcher_knife_new = "Server.Items.ButcherKnife" : BASE_KNIFE / BASE_KNIFE_CLASS, graphic 0x13F6, v0;
    flippable: Some((0x13F6, 0x13F7)),
    default_weight: Some(1.0),
    primary_ability: Some(Ability::InfectiousStrike),
    secondary_ability: Some(Ability::Disarm),
}

weapon_leaf! {
    /// `Server.Items.Cleaver`.
    CLEAVER, CLEAVER_CLASS, cleaver_new = "Server.Items.Cleaver" : BASE_KNIFE / BASE_KNIFE_CLASS, graphic 0xEC3, v0;
    flippable: Some((0xEC3, 0xEC2)),
    default_weight: Some(2.0),
    primary_ability: Some(Ability::BleedAttack),
    secondary_ability: Some(Ability::InfectiousStrike),
}

weapon_leaf! {
    /// `Server.Items.Dagger`.
    DAGGER, DAGGER_CLASS, dagger_new = "Server.Items.Dagger" : BASE_KNIFE / BASE_KNIFE_CLASS, graphic 0xF52, v0;
    flippable: Some((0xF52, 0xF51)),
    default_weight: Some(1.0),
    primary_ability: Some(Ability::InfectiousStrike),
    secondary_ability: Some(Ability::ShadowStrike),
    def_skill: SkillName::Fencing,
    def_type: WeaponType::Piercing,
    def_animation: WeaponAnimation::Pierce1H,
}

weapon_leaf! {
    /// `Server.Items.SkinningKnife`.
    SKINNING_KNIFE, SKINNING_KNIFE_CLASS, skinning_knife_new = "Server.Items.SkinningKnife" : BASE_KNIFE / BASE_KNIFE_CLASS, graphic 0xEC4, v0;
    flippable: Some((0xEC4, 0xEC5)),
    default_weight: Some(1.0),
    primary_ability: Some(Ability::ShadowStrike),
    secondary_ability: Some(Ability::Disarm),
}

pub fn register(types: &mut TypeRegistry) {
    types.register_item(&BUTCHER_KNIFE, &[]);
    types.register_item(&CLEAVER, &[]);
    types.register_item(&DAGGER, &[]);
    types.register_item(&SKINNING_KNIFE, &[]);
}
