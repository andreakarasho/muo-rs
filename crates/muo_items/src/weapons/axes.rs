//! Weapon classes of axes; see weapons/mod.rs.

use super::*;

weapon_leaf! {
    /// `Server.Items.Axe`.
    AXE, AXE_CLASS, axe_new = "Server.Items.Axe" : BASE_AXE / BASE_AXE_CLASS, graphic 0xF49, v0;
    flippable: Some((0xF49, 0xF4A)),
    default_weight: Some(4.0),
    primary_ability: Some(Ability::CrushingBlow),
    secondary_ability: Some(Ability::Dismount),
}

weapon_leaf! {
    /// `Server.Items.BattleAxe`.
    BATTLE_AXE, BATTLE_AXE_CLASS, battle_axe_new = "Server.Items.BattleAxe" : BASE_AXE / BASE_AXE_CLASS, graphic 0xF47, v0;
    @json;
    flippable: Some((0xF47, 0xF48)),
    default_weight: Some(4.0),
    primary_ability: Some(Ability::BleedAttack),
    secondary_ability: Some(Ability::ConcussionBlow),
}

weapon_leaf! {
    /// `Server.Items.DoubleAxe`.
    DOUBLE_AXE, DOUBLE_AXE_CLASS, double_axe_new = "Server.Items.DoubleAxe" : BASE_AXE / BASE_AXE_CLASS, graphic 0xF4B, v0;
    flippable: Some((0xF4B, 0xF4C)),
    default_weight: Some(8.0),
    primary_ability: Some(Ability::DoubleStrike),
    secondary_ability: Some(Ability::WhirlwindAttack),
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
}

weapon_leaf! {
    /// `Server.Items.ExecutionersAxe`.
    EXECUTIONERS_AXE, EXECUTIONERS_AXE_CLASS, executioners_axe_new = "Server.Items.ExecutionersAxe" : BASE_AXE / BASE_AXE_CLASS, graphic 0xF45, v0;
    flippable: Some((0xF45, 0xF46)),
    default_weight: Some(8.0),
    primary_ability: Some(Ability::BleedAttack),
    secondary_ability: Some(Ability::MortalStrike),
}

weapon_leaf! {
    /// `Server.Items.GuardianAxe`.
    GUARDIAN_AXE, GUARDIAN_AXE_CLASS, guardian_axe_new = "Server.Items.GuardianAxe" : ORNATE_AXE / ORNATE_AXE_CLASS, via ornate_axe_new, enc;
    @json;
}

weapon_leaf! {
    /// `Server.Items.Hatchet`.
    HATCHET, HATCHET_CLASS, hatchet_new = "Server.Items.Hatchet" : BASE_AXE / BASE_AXE_CLASS, graphic 0xF43, v0;
    flippable: Some((0xF43, 0xF44)),
    default_weight: Some(4.0),
    primary_ability: Some(Ability::ArmorIgnore),
    secondary_ability: Some(Ability::Disarm),
}

weapon_leaf! {
    /// `Server.Items.HeavyOrnateAxe`.
    HEAVY_ORNATE_AXE, HEAVY_ORNATE_AXE_CLASS, heavy_ornate_axe_new = "Server.Items.HeavyOrnateAxe" : ORNATE_AXE / ORNATE_AXE_CLASS, via ornate_axe_new, enc;
    @json;
}

weapon_leaf! {
    /// `Server.Items.LargeBattleAxe`.
    LARGE_BATTLE_AXE, LARGE_BATTLE_AXE_CLASS, large_battle_axe_new = "Server.Items.LargeBattleAxe" : BASE_AXE / BASE_AXE_CLASS, graphic 0x13FB, v0;
    flippable: Some((0x13FB, 0x13FA)),
    default_weight: Some(6.0),
    primary_ability: Some(Ability::WhirlwindAttack),
    secondary_ability: Some(Ability::BleedAttack),
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
    def_animation: WeaponAnimation::Slash1H,
    on_craft: pickaxe_on_craft,
}

weapon_leaf! {
    /// `Server.Items.SingingAxe`.
    SINGING_AXE, SINGING_AXE_CLASS, singing_axe_new = "Server.Items.SingingAxe" : ORNATE_AXE / ORNATE_AXE_CLASS, via ornate_axe_new, enc;
    @json;
}

weapon_leaf! {
    /// `Server.Items.ThunderingAxe`.
    THUNDERING_AXE, THUNDERING_AXE_CLASS, thundering_axe_new = "Server.Items.ThunderingAxe" : ORNATE_AXE / ORNATE_AXE_CLASS, via ornate_axe_new, enc;
    @json;
}

weapon_leaf! {
    /// `Server.Items.TwoHandedAxe`.
    TWO_HANDED_AXE, TWO_HANDED_AXE_CLASS, two_handed_axe_new = "Server.Items.TwoHandedAxe" : BASE_AXE / BASE_AXE_CLASS, graphic 0x1443, v0;
    flippable: Some((0x1443, 0x1442)),
    default_weight: Some(8.0),
    primary_ability: Some(Ability::DoubleStrike),
    secondary_ability: Some(Ability::ShadowStrike),
}

weapon_leaf! {
    /// `Server.Items.WarAxe`.
    WAR_AXE, WAR_AXE_CLASS, war_axe_new = "Server.Items.WarAxe" : BASE_AXE / BASE_AXE_CLASS, graphic 0x13B0, v0;
    flippable: Some((0x13B0, 0x13AF)),
    default_weight: Some(8.0),
    primary_ability: Some(Ability::ArmorIgnore),
    secondary_ability: Some(Ability::BleedAttack),
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
