//! Weapon classes of spears; see weapons/mod.rs.

use super::*;

weapon_leaf! {
    /// `Server.Items.BladedStaff`.
    BLADED_STAFF, BLADED_STAFF_CLASS, bladed_staff_new = "Server.Items.BladedStaff" : BASE_SPEAR / BASE_SPEAR_CLASS, graphic 0x26BD, v0;
    flippable: Some((0x26BD, 0x26C7)),
    default_weight: Some(4.0),
    primary_ability: Some(Ability::ArmorIgnore),
    secondary_ability: Some(Ability::Dismount),
    def_skill: SkillName::Swords,
}

weapon_leaf! {
    /// `Server.Items.DoubleBladedStaff`.
    DOUBLE_BLADED_STAFF, DOUBLE_BLADED_STAFF_CLASS, double_bladed_staff_new = "Server.Items.DoubleBladedStaff" : BASE_SPEAR / BASE_SPEAR_CLASS, graphic 0x26BF, v0;
    flippable: Some((0x26BF, 0x26C9)),
    default_weight: Some(2.0),
    primary_ability: Some(Ability::DoubleStrike),
    secondary_ability: Some(Ability::InfectiousStrike),
}

weapon_leaf! {
    /// `Server.Items.DualPointedSpear`.
    DUAL_POINTED_SPEAR, DUAL_POINTED_SPEAR_CLASS, dual_pointed_spear_new = "Server.Items.DualPointedSpear" : BASE_SPEAR / BASE_SPEAR_CLASS, graphic 0x904, enc;
    @item {
        required_races: |_, _| ALLOW_GARGOYLES_ONLY,
    }
    flippable: Some((0x904, 0x406D)),
    primary_ability: Some(Ability::DoubleStrike),
    secondary_ability: Some(Ability::Disarm),
}

weapon_leaf! {
    /// `Server.Items.Pike`.
    PIKE, PIKE_CLASS, pike_new = "Server.Items.Pike" : BASE_SPEAR / BASE_SPEAR_CLASS, graphic 0x26BE, v0;
    flippable: Some((0x26BE, 0x26C8)),
    default_weight: Some(8.0),
    primary_ability: Some(Ability::ParalyzingBlow),
    secondary_ability: Some(Ability::InfectiousStrike),
}

weapon_leaf! {
    /// `Server.Items.Pitchfork`.
    PITCHFORK, PITCHFORK_CLASS, pitchfork_new = "Server.Items.Pitchfork" : BASE_SPEAR / BASE_SPEAR_CLASS, graphic 0xE87, v0;
    flippable: Some((0xE87, 0xE88)),
    default_weight: Some(11.0),
    primary_ability: Some(Ability::BleedAttack),
    secondary_ability: Some(Ability::Dismount),
}

weapon_leaf! {
    /// `Server.Items.ShortSpear`.
    SHORT_SPEAR, SHORT_SPEAR_CLASS, short_spear_new = "Server.Items.ShortSpear" : BASE_SPEAR / BASE_SPEAR_CLASS, graphic 0x1403, v0;
    flippable: Some((0x1403, 0x1402)),
    default_weight: Some(4.0),
    primary_ability: Some(Ability::ShadowStrike),
    secondary_ability: Some(Ability::MortalStrike),
    def_animation: WeaponAnimation::Pierce1H,
}

weapon_leaf! {
    /// `Server.Items.Spear`.
    SPEAR, SPEAR_CLASS, spear_new = "Server.Items.Spear" : BASE_SPEAR / BASE_SPEAR_CLASS, graphic 0xF62, v0;
    flippable: Some((0xF62, 0xF63)),
    default_weight: Some(7.0),
    primary_ability: Some(Ability::ArmorIgnore),
    secondary_ability: Some(Ability::ParalyzingBlow),
}

weapon_leaf! {
    /// `Server.Items.TribalSpear`.
    TRIBAL_SPEAR, TRIBAL_SPEAR_CLASS, tribal_spear_new = "Server.Items.TribalSpear" : BASE_SPEAR / BASE_SPEAR_CLASS, graphic 0xF62, v0;
    @json;
    @item {
        default_name: |_, _| Some("a tribal spear".to_string()),
    }
    flippable: Some((0xF62, 0xF63)),
    default_weight: Some(7.0),
    primary_ability: Some(Ability::ArmorIgnore),
    secondary_ability: Some(Ability::ParalyzingBlow),
    virtual_damage_bonus: 25,
}

weapon_leaf! {
    /// `Server.Items.WarFork`.
    WAR_FORK, WAR_FORK_CLASS, war_fork_new = "Server.Items.WarFork" : BASE_SPEAR / BASE_SPEAR_CLASS, graphic 0x1405, v0;
    flippable: Some((0x1405, 0x1404)),
    default_weight: Some(9.0),
    primary_ability: Some(Ability::BleedAttack),
    secondary_ability: Some(Ability::Disarm),
    def_animation: WeaponAnimation::Pierce1H,
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
