//! Weapon classes of axes; see weapons/mod.rs.

use super::*;

weapon_leaf! {
    /// `Server.Items.Axe`.
    AXE, AXE_CLASS, axe_new = "Server.Items.Axe" : BASE_AXE / BASE_AXE_CLASS, json (), v0;
}

weapon_leaf! {
    /// `Server.Items.BattleAxe`.
    BATTLE_AXE, BATTLE_AXE_CLASS, battle_axe_new = "Server.Items.BattleAxe" : BASE_AXE / BASE_AXE_CLASS, json (), v0;
    @json;
}

weapon_leaf! {
    /// `Server.Items.DoubleAxe`.
    DOUBLE_AXE, DOUBLE_AXE_CLASS, double_axe_new = "Server.Items.DoubleAxe" : BASE_AXE / BASE_AXE_CLASS, json (), v0;
}

weapon_leaf! {
    /// `Server.Items.DualShortAxes`.
    DUAL_SHORT_AXES, DUAL_SHORT_AXES_CLASS, dual_short_axes_new = "Server.Items.DualShortAxes" : BASE_AXE / BASE_AXE_CLASS, json (), enc;
}

weapon_leaf! {
    /// `Server.Items.ExecutionersAxe`.
    EXECUTIONERS_AXE, EXECUTIONERS_AXE_CLASS, executioners_axe_new = "Server.Items.ExecutionersAxe" : BASE_AXE / BASE_AXE_CLASS, json (), v0;
}

weapon_leaf! {
    /// `Server.Items.GuardianAxe`.
    GUARDIAN_AXE, GUARDIAN_AXE_CLASS, guardian_axe_new = "Server.Items.GuardianAxe" : ORNATE_AXE / ORNATE_AXE_CLASS, via ornate_axe_new, enc;
    @json;
}

weapon_leaf! {
    /// `Server.Items.Hatchet`.
    HATCHET, HATCHET_CLASS, hatchet_new = "Server.Items.Hatchet" : BASE_AXE / BASE_AXE_CLASS, json (), v0;
}

weapon_leaf! {
    /// `Server.Items.HeavyOrnateAxe`.
    HEAVY_ORNATE_AXE, HEAVY_ORNATE_AXE_CLASS, heavy_ornate_axe_new = "Server.Items.HeavyOrnateAxe" : ORNATE_AXE / ORNATE_AXE_CLASS, via ornate_axe_new, enc;
    @json;
}

weapon_leaf! {
    /// `Server.Items.LargeBattleAxe`.
    LARGE_BATTLE_AXE, LARGE_BATTLE_AXE_CLASS, large_battle_axe_new = "Server.Items.LargeBattleAxe" : BASE_AXE / BASE_AXE_CLASS, json (), v0;
}

weapon_leaf! {
    /// `Server.Items.Pickaxe`.
    PICKAXE, PICKAXE_CLASS, pickaxe_new = "Server.Items.Pickaxe" : BASE_AXE / BASE_AXE_CLASS, json (), v0;
    @json;
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
    TWO_HANDED_AXE, TWO_HANDED_AXE_CLASS, two_handed_axe_new = "Server.Items.TwoHandedAxe" : BASE_AXE / BASE_AXE_CLASS, json (), v0;
}

weapon_leaf! {
    /// `Server.Items.WarAxe`.
    WAR_AXE, WAR_AXE_CLASS, war_axe_new = "Server.Items.WarAxe" : BASE_AXE / BASE_AXE_CLASS, json (), v0;
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
