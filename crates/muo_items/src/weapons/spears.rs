//! Weapon classes of spears; see weapons/mod.rs.

use super::*;

weapon_leaf! {
    /// `Server.Items.BladedStaff`.
    BLADED_STAFF, BLADED_STAFF_CLASS, bladed_staff_new = "Server.Items.BladedStaff" : BASE_SPEAR / BASE_SPEAR_CLASS, graphic 0x26BD, v0;
    flippable: Some((0x26BD, 0x26C7)),
    default_weight: Some(4.0),
    primary_ability: Some(Ability::ArmorIgnore),
    secondary_ability: Some(Ability::Dismount),
    aos_str_req: 40,
    aos_min_damage: same(14),
    aos_max_damage: same(16),
    aos_speed: 37,
    ml_speed: 3.00,
    old_str_req: 40,
    old_min_damage: 14,
    old_max_damage: 16,
    old_speed: 37,
    init_min_hits: 21,
    init_max_hits: 110,
    def_skill: SkillName::Swords,
}

weapon_leaf! {
    /// `Server.Items.DoubleBladedStaff`.
    DOUBLE_BLADED_STAFF, DOUBLE_BLADED_STAFF_CLASS, double_bladed_staff_new = "Server.Items.DoubleBladedStaff" : BASE_SPEAR / BASE_SPEAR_CLASS, graphic 0x26BF, v0;
    flippable: Some((0x26BF, 0x26C9)),
    default_weight: Some(2.0),
    primary_ability: Some(Ability::DoubleStrike),
    secondary_ability: Some(Ability::InfectiousStrike),
    aos_str_req: 50,
    aos_min_damage: same(12),
    aos_max_damage: same(13),
    aos_speed: 49,
    ml_speed: 2.25,
    old_str_req: 50,
    old_min_damage: 12,
    old_max_damage: 13,
    old_speed: 49,
    init_min_hits: 31,
    init_max_hits: 80,
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
    aos_str_req: 50,
    aos_min_damage: same(11),
    aos_max_damage: same(14),
    ml_speed: 2.25,
    init_min_hits: 31,
    init_max_hits: 80,
}

weapon_leaf! {
    /// `Server.Items.Pike`.
    PIKE, PIKE_CLASS, pike_new = "Server.Items.Pike" : BASE_SPEAR / BASE_SPEAR_CLASS, graphic 0x26BE, v0;
    flippable: Some((0x26BE, 0x26C8)),
    default_weight: Some(8.0),
    primary_ability: Some(Ability::ParalyzingBlow),
    secondary_ability: Some(Ability::InfectiousStrike),
    aos_str_req: 50,
    aos_min_damage: same(14),
    aos_max_damage: same(16),
    aos_speed: 37,
    ml_speed: 3.00,
    old_str_req: 50,
    old_min_damage: 14,
    old_max_damage: 16,
    old_speed: 37,
    init_min_hits: 31,
    init_max_hits: 110,
}

weapon_leaf! {
    /// `Server.Items.Pitchfork`.
    PITCHFORK, PITCHFORK_CLASS, pitchfork_new = "Server.Items.Pitchfork" : BASE_SPEAR / BASE_SPEAR_CLASS, graphic 0xE87, v0;
    flippable: Some((0xE87, 0xE88)),
    default_weight: Some(11.0),
    primary_ability: Some(Ability::BleedAttack),
    secondary_ability: Some(Ability::Dismount),
    aos_str_req: 55,
    aos_min_damage: same(13),
    aos_max_damage: same(14),
    aos_speed: 43,
    ml_speed: 2.50,
    old_str_req: 15,
    old_min_damage: 4,
    old_max_damage: 16,
    old_speed: 45,
    init_min_hits: 31,
    init_max_hits: 60,
}

weapon_leaf! {
    /// `Server.Items.ShortSpear`.
    SHORT_SPEAR, SHORT_SPEAR_CLASS, short_spear_new = "Server.Items.ShortSpear" : BASE_SPEAR / BASE_SPEAR_CLASS, graphic 0x1403, v0;
    flippable: Some((0x1403, 0x1402)),
    default_weight: Some(4.0),
    primary_ability: Some(Ability::ShadowStrike),
    secondary_ability: Some(Ability::MortalStrike),
    aos_str_req: 40,
    aos_min_damage: same(10),
    aos_max_damage: same(13),
    aos_speed: 55,
    ml_speed: 2.00,
    old_str_req: 15,
    old_min_damage: 4,
    old_max_damage: 32,
    old_speed: 50,
    init_min_hits: 31,
    init_max_hits: 70,
    def_animation: WeaponAnimation::Pierce1H,
}

weapon_leaf! {
    /// `Server.Items.Spear`.
    SPEAR, SPEAR_CLASS, spear_new = "Server.Items.Spear" : BASE_SPEAR / BASE_SPEAR_CLASS, graphic 0xF62, v0;
    flippable: Some((0xF62, 0xF63)),
    default_weight: Some(7.0),
    primary_ability: Some(Ability::ArmorIgnore),
    secondary_ability: Some(Ability::ParalyzingBlow),
    aos_str_req: 50,
    aos_min_damage: same(13),
    aos_max_damage: same(15),
    aos_speed: 42,
    ml_speed: 2.75,
    old_str_req: 30,
    old_min_damage: 2,
    old_max_damage: 36,
    old_speed: 46,
    init_min_hits: 31,
    init_max_hits: 80,
}

weapon_leaf! {
    /// `Server.Items.TribalSpear`.
    TRIBAL_SPEAR, TRIBAL_SPEAR_CLASS, tribal_spear_new = "Server.Items.TribalSpear" : BASE_SPEAR / BASE_SPEAR_CLASS, graphic 0xF62, v0;
    @init |w, id| {
        ctor::set_hue(w, id, 837);
    };
    @item {
        default_name: |_, _| Some("a tribal spear".to_string()),
    }
    flippable: Some((0xF62, 0xF63)),
    default_weight: Some(7.0),
    primary_ability: Some(Ability::ArmorIgnore),
    secondary_ability: Some(Ability::ParalyzingBlow),
    aos_str_req: 50,
    aos_min_damage: same(13),
    aos_max_damage: same(15),
    aos_speed: 42,
    ml_speed: 2.75,
    old_str_req: 30,
    old_min_damage: 2,
    old_max_damage: 36,
    old_speed: 46,
    init_min_hits: 31,
    init_max_hits: 80,
    virtual_damage_bonus: 25,
}

weapon_leaf! {
    /// `Server.Items.WarFork`.
    WAR_FORK, WAR_FORK_CLASS, war_fork_new = "Server.Items.WarFork" : BASE_SPEAR / BASE_SPEAR_CLASS, graphic 0x1405, v0;
    flippable: Some((0x1405, 0x1404)),
    default_weight: Some(9.0),
    primary_ability: Some(Ability::BleedAttack),
    secondary_ability: Some(Ability::Disarm),
    aos_str_req: 45,
    aos_min_damage: same(12),
    aos_max_damage: same(13),
    aos_speed: 43,
    ml_speed: 2.50,
    old_str_req: 35,
    old_min_damage: 4,
    old_max_damage: 32,
    old_speed: 45,
    def_hit_sound: 0x236,
    def_miss_sound: 0x238,
    init_min_hits: 31,
    init_max_hits: 110,
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
