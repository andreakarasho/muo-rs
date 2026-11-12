//! Weapon classes of axes; see weapons/mod.rs.

use super::*;

weapon_leaf! {
    /// `Server.Items.Axe`.
    AXE, AXE_CLASS, axe_new = "Server.Items.Axe" : BASE_AXE / BASE_AXE_CLASS, graphic 0xF49, v0;
    flippable: Some((0xF49, 0xF4A)),
    default_weight: Some(4.0),
    primary_ability: Some(Ability::CrushingBlow),
    secondary_ability: Some(Ability::Dismount),
    aos_str_req: 35,
    aos_min_damage: same(14),
    aos_max_damage: same(16),
    aos_speed: 37,
    ml_speed: 3.00,
    old_str_req: 35,
    old_min_damage: 6,
    old_max_damage: 33,
    old_speed: 37,
    init_min_hits: 31,
    init_max_hits: 110,
}

weapon_leaf! {
    /// `Server.Items.BattleAxe`.
    BATTLE_AXE, BATTLE_AXE_CLASS, battle_axe_new = "Server.Items.BattleAxe" : BASE_AXE / BASE_AXE_CLASS, graphic 0xF47, v0;
    @init |w, id| {
        ctor::set_layer(w, id, Layer::TwoHanded);
    };
    flippable: Some((0xF47, 0xF48)),
    default_weight: Some(4.0),
    primary_ability: Some(Ability::BleedAttack),
    secondary_ability: Some(Ability::ConcussionBlow),
    aos_str_req: 35,
    aos_min_damage: same(15),
    aos_max_damage: same(17),
    aos_speed: 31,
    ml_speed: 3.50,
    old_str_req: 40,
    old_min_damage: 6,
    old_max_damage: 38,
    old_speed: 30,
    init_min_hits: 31,
    init_max_hits: 70,
}

weapon_leaf! {
    /// `Server.Items.DoubleAxe`.
    DOUBLE_AXE, DOUBLE_AXE_CLASS, double_axe_new = "Server.Items.DoubleAxe" : BASE_AXE / BASE_AXE_CLASS, graphic 0xF4B, v0;
    flippable: Some((0xF4B, 0xF4C)),
    default_weight: Some(8.0),
    primary_ability: Some(Ability::DoubleStrike),
    secondary_ability: Some(Ability::WhirlwindAttack),
    aos_str_req: 45,
    aos_min_damage: same(15),
    aos_max_damage: same(17),
    aos_speed: 33,
    ml_speed: 3.25,
    old_str_req: 45,
    old_min_damage: 5,
    old_max_damage: 35,
    old_speed: 37,
    init_min_hits: 31,
    init_max_hits: 110,
}

weapon_leaf! {
    /// `Server.Items.DualShortAxes`.
    DUAL_SHORT_AXES, DUAL_SHORT_AXES_CLASS, dual_short_axes_new = "Server.Items.DualShortAxes" : BASE_AXE / BASE_AXE_CLASS, graphic 0x8FD, enc;
    @item {
        required_races: |_, _| ALLOW_GARGOYLES_ONLY,
    }
    flippable: Some((0x8FD, 0x4068)),
    primary_ability: Some(Ability::DoubleStrike),
    secondary_ability: Some(Ability::InfectiousStrike),
    aos_str_req: 35,
    aos_min_damage: same(14),
    aos_max_damage: same(17),
    ml_speed: 3.00,
    init_min_hits: 31,
    init_max_hits: 110,
}

weapon_leaf! {
    /// `Server.Items.ExecutionersAxe`.
    EXECUTIONERS_AXE, EXECUTIONERS_AXE_CLASS, executioners_axe_new = "Server.Items.ExecutionersAxe" : BASE_AXE / BASE_AXE_CLASS, graphic 0xF45, v0;
    flippable: Some((0xF45, 0xF46)),
    default_weight: Some(8.0),
    primary_ability: Some(Ability::BleedAttack),
    secondary_ability: Some(Ability::MortalStrike),
    aos_str_req: 40,
    aos_min_damage: same(15),
    aos_max_damage: same(17),
    aos_speed: 33,
    ml_speed: 3.25,
    old_str_req: 35,
    old_min_damage: 6,
    old_max_damage: 33,
    old_speed: 37,
    init_min_hits: 31,
    init_max_hits: 70,
}

weapon_leaf! {
    /// `Server.Items.GuardianAxe`.
    GUARDIAN_AXE, GUARDIAN_AXE_CLASS, guardian_axe_new = "Server.Items.GuardianAxe" : ORNATE_AXE / ORNATE_AXE_CLASS, via ornate_axe_new, enc;
    @init |w, id| {
        set_attr(w, id, AttrKind::Attributes, AosAttribute::BonusHits as u32, 4);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::RegenHits as u32, 1);
    };
    label_number: 1073545,
}

weapon_leaf! {
    /// `Server.Items.Hatchet`.
    HATCHET, HATCHET_CLASS, hatchet_new = "Server.Items.Hatchet" : BASE_AXE / BASE_AXE_CLASS, graphic 0xF43, v0;
    flippable: Some((0xF43, 0xF44)),
    default_weight: Some(4.0),
    primary_ability: Some(Ability::ArmorIgnore),
    secondary_ability: Some(Ability::Disarm),
    aos_str_req: 20,
    aos_min_damage: same(13),
    aos_max_damage: same(15),
    aos_speed: 41,
    ml_speed: 2.75,
    old_str_req: 15,
    old_min_damage: 2,
    old_max_damage: 17,
    old_speed: 40,
    init_min_hits: 31,
    init_max_hits: 80,
}

weapon_leaf! {
    /// `Server.Items.HeavyOrnateAxe`.
    HEAVY_ORNATE_AXE, HEAVY_ORNATE_AXE_CLASS, heavy_ornate_axe_new = "Server.Items.HeavyOrnateAxe" : ORNATE_AXE / ORNATE_AXE_CLASS, via ornate_axe_new, enc;
    @init |w, id| {
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponDamage as u32, 8);
    };
    label_number: 1073548,
}

weapon_leaf! {
    /// `Server.Items.LargeBattleAxe`.
    LARGE_BATTLE_AXE, LARGE_BATTLE_AXE_CLASS, large_battle_axe_new = "Server.Items.LargeBattleAxe" : BASE_AXE / BASE_AXE_CLASS, graphic 0x13FB, v0;
    flippable: Some((0x13FB, 0x13FA)),
    default_weight: Some(6.0),
    primary_ability: Some(Ability::WhirlwindAttack),
    secondary_ability: Some(Ability::BleedAttack),
    aos_str_req: 80,
    aos_min_damage: same(16),
    aos_max_damage: same(17),
    aos_speed: 29,
    ml_speed: 3.75,
    old_str_req: 40,
    old_min_damage: 6,
    old_max_damage: 38,
    old_speed: 30,
    init_min_hits: 31,
    init_max_hits: 70,
}

weapon_leaf! {
    /// `Server.Items.Pickaxe`.
    PICKAXE, PICKAXE_CLASS, pickaxe_new = "Server.Items.Pickaxe" : BASE_AXE / BASE_AXE_CLASS, graphic 0xE86, v0;
    @init |w, id| {
        ctor::set_uses_remaining(w, id, 50);
        ctor::set_show_uses_remaining(w, id, true);
    };
    flippable: Some((0xE86, 0xE85)),
    default_weight: Some(11.0),
    harvest_system: Some("Mining"),
    primary_ability: Some(Ability::DoubleStrike),
    secondary_ability: Some(Ability::Disarm),
    aos_str_req: 50,
    aos_min_damage: same(13),
    aos_max_damage: same(15),
    aos_speed: 35,
    ml_speed: 3.00,
    old_str_req: 25,
    old_min_damage: 1,
    old_max_damage: 15,
    old_speed: 35,
    init_min_hits: 31,
    init_max_hits: 60,
    def_animation: WeaponAnimation::Slash1H,
    on_craft: pickaxe_on_craft,
}

weapon_leaf! {
    /// `Server.Items.SingingAxe`.
    SINGING_AXE, SINGING_AXE_CLASS, singing_axe_new = "Server.Items.SingingAxe" : ORNATE_AXE / ORNATE_AXE_CLASS, via ornate_axe_new, enc;
    @init |w, id| {
        ctor::skill_bonus(w, id, 0, SkillName::Musicianship, 5.0);
    };
    label_number: 1073546,
}

weapon_leaf! {
    /// `Server.Items.ThunderingAxe`.
    THUNDERING_AXE, THUNDERING_AXE_CLASS, thundering_axe_new = "Server.Items.ThunderingAxe" : ORNATE_AXE / ORNATE_AXE_CLASS, via ornate_axe_new, enc;
    @init |w, id| {
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::HitLightning as u32, 10);
    };
    label_number: 1073547,
}

weapon_leaf! {
    /// `Server.Items.TwoHandedAxe`.
    TWO_HANDED_AXE, TWO_HANDED_AXE_CLASS, two_handed_axe_new = "Server.Items.TwoHandedAxe" : BASE_AXE / BASE_AXE_CLASS, graphic 0x1443, v0;
    flippable: Some((0x1443, 0x1442)),
    default_weight: Some(8.0),
    primary_ability: Some(Ability::DoubleStrike),
    secondary_ability: Some(Ability::ShadowStrike),
    aos_str_req: 40,
    aos_min_damage: same(16),
    aos_max_damage: same(17),
    aos_speed: 31,
    ml_speed: 3.50,
    old_str_req: 35,
    old_min_damage: 5,
    old_max_damage: 39,
    old_speed: 30,
    init_min_hits: 31,
    init_max_hits: 90,
}

weapon_leaf! {
    /// `Server.Items.WarAxe`.
    WAR_AXE, WAR_AXE_CLASS, war_axe_new = "Server.Items.WarAxe" : BASE_AXE / BASE_AXE_CLASS, graphic 0x13B0, v0;
    flippable: Some((0x13B0, 0x13AF)),
    default_weight: Some(8.0),
    primary_ability: Some(Ability::ArmorIgnore),
    secondary_ability: Some(Ability::BleedAttack),
    aos_str_req: 35,
    aos_min_damage: same(14),
    aos_max_damage: same(15),
    aos_speed: 33,
    ml_speed: 3.25,
    old_str_req: 35,
    old_min_damage: 9,
    old_max_damage: 27,
    old_speed: 40,
    def_hit_sound: 0x233,
    def_miss_sound: 0x239,
    init_min_hits: 31,
    init_max_hits: 80,
    def_skill: SkillName::Macing,
    def_type: WeaponType::Bashing,
    def_animation: WeaponAnimation::Bash1H,
    harvest_system: None,
}

pub fn register(types: &mut TypeRegistry) {
    types.register_item(&AXE, &[]);
    types.register_item(&BATTLE_AXE, &[]);
    types.register_item(&DOUBLE_AXE, &[]);
    types.register_item(&DUAL_SHORT_AXES, &[]);
    types.register_item(&EXECUTIONERS_AXE, &[]);
    types.register_item(&GUARDIAN_AXE, &[]);
    types.register_item(&HATCHET, &[]);
    types.register_item(&HEAVY_ORNATE_AXE, &[]);
    types.register_item(&LARGE_BATTLE_AXE, &[]);
    types.register_item(&PICKAXE, &[]);
    types.register_item(&SINGING_AXE, &[]);
    types.register_item(&THUNDERING_AXE, &[]);
    types.register_item(&TWO_HANDED_AXE, &[]);
    types.register_item(&WAR_AXE, &[]);
}
