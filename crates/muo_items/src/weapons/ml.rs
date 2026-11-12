//! Weapon classes of ml; see weapons/mod.rs.

use super::*;

weapon_leaf! {
    /// `Server.Items.AssassinSpike`.
    ASSASSIN_SPIKE, ASSASSIN_SPIKE_CLASS, assassin_spike_new = "Server.Items.AssassinSpike" : BASE_KNIFE / BASE_KNIFE_CLASS, graphic 0x2D21, enc;
    flippable: Some((0x2D21, 0x2D2D)),
    default_weight: Some(4.0),
    primary_ability: Some(Ability::InfectiousStrike),
    secondary_ability: Some(Ability::ShadowStrike),
    aos_str_req: 15,
    aos_min_damage: same(10),
    aos_max_damage: same(12),
    aos_speed: 50,
    ml_speed: 2.00,
    old_str_req: 15,
    old_min_damage: 10,
    old_max_damage: 12,
    old_speed: 50,
    def_miss_sound: 0x239,
    def_skill: SkillName::Fencing,
    init_min_hits: 30,
    init_max_hits: 60,
}

weapon_leaf! {
    /// `Server.Items.ButchersWarCleaver`.
    BUTCHERS_WAR_CLEAVER, BUTCHERS_WAR_CLEAVER_CLASS, butchers_war_cleaver_new = "Server.Items.ButchersWarCleaver" : WAR_CLEAVER / WAR_CLEAVER_CLASS, via war_cleaver_new, enc;
    @item {
        append_child_name_properties: butchers_war_cleaver_append_child_name_properties,
    }
    label_number: 1073526,
}

weapon_leaf! {
    /// `Server.Items.DiamondMace`.
    DIAMOND_MACE, DIAMOND_MACE_CLASS, diamond_mace_new = "Server.Items.DiamondMace" : BASE_BASHING / BASE_BASHING_CLASS, graphic 0x2D24, enc;
    flippable: Some((0x2D24, 0x2D30)),
    default_weight: Some(10.0),
    primary_ability: Some(Ability::ConcussionBlow),
    secondary_ability: Some(Ability::CrushingBlow),
    aos_str_req: 35,
    aos_min_damage: same(14),
    aos_max_damage: same(17),
    aos_speed: 37,
    ml_speed: 3.00,
    old_str_req: 35,
    old_min_damage: 14,
    old_max_damage: 17,
    old_speed: 37,
    init_min_hits: 30,
    init_max_hits: 60,
}

weapon_leaf! {
    /// `Server.Items.ElvenCompositeLongbow`.
    ELVEN_COMPOSITE_LONGBOW, ELVEN_COMPOSITE_LONGBOW_CLASS, elven_composite_longbow_new = "Server.Items.ElvenCompositeLongbow" : BASE_RANGED / BASE_RANGED_CLASS, graphic 0x2D1E, enc;
    @init |w, id| {
        set_resource(w, id, CraftResource::RegularWood);
    };
    flippable: Some((0x2D1E, 0x2D2A)),
    default_weight: Some(8.0),
    effect_id: 0xF42,
    ammo_type: Some("Server.Items.Arrow"),
    primary_ability: Some(Ability::ForceArrow),
    secondary_ability: Some(Ability::SerpentArrow),
    aos_str_req: 45,
    aos_min_damage: same(12),
    aos_max_damage: same(16),
    aos_speed: 27,
    ml_speed: 4.00,
    old_str_req: 45,
    old_min_damage: 12,
    old_max_damage: 16,
    old_speed: 27,
    def_max_range: 10,
    init_min_hits: 41,
    init_max_hits: 90,
    def_animation: WeaponAnimation::ShootBow,
}

weapon_leaf! {
    /// `Server.Items.ElvenMachete`.
    ELVEN_MACHETE, ELVEN_MACHETE_CLASS, elven_machete_new = "Server.Items.ElvenMachete" : BASE_SWORD / BASE_SWORD_CLASS, graphic 0x2D35, enc;
    flippable: Some((0x2D35, 0x2D29)),
    default_weight: Some(6.0),
    primary_ability: Some(Ability::DefenseMastery),
    secondary_ability: Some(Ability::Bladeweave),
    aos_str_req: 20,
    aos_min_damage: same(13),
    aos_max_damage: same(15),
    aos_speed: 41,
    ml_speed: 2.75,
    old_str_req: 20,
    old_min_damage: 13,
    old_max_damage: 15,
    old_speed: 41,
    def_hit_sound: 0x23B,
    def_miss_sound: 0x239,
    init_min_hits: 30,
    init_max_hits: 60,
}

weapon_leaf! {
    /// `Server.Items.ElvenSpellblade`.
    ELVEN_SPELLBLADE, ELVEN_SPELLBLADE_CLASS, elven_spellblade_new = "Server.Items.ElvenSpellblade" : BASE_KNIFE / BASE_KNIFE_CLASS, graphic 0x2D20, enc;
    @init |w, id| {
        ctor::set_layer(w, id, Layer::TwoHanded);
    };
    flippable: Some((0x2D20, 0x2D2C)),
    default_weight: Some(5.0),
    primary_ability: Some(Ability::PsychicAttack),
    secondary_ability: Some(Ability::BleedAttack),
    aos_str_req: 35,
    aos_min_damage: same(12),
    aos_max_damage: same(14),
    aos_speed: 44,
    ml_speed: 2.50,
    old_str_req: 35,
    old_min_damage: 12,
    old_max_damage: 14,
    old_speed: 44,
    def_miss_sound: 0x239,
    init_min_hits: 30,
    init_max_hits: 60,
}

weapon_leaf! {
    /// `Server.Items.Leafblade`.
    LEAFBLADE, LEAFBLADE_CLASS, leafblade_new = "Server.Items.Leafblade" : BASE_KNIFE / BASE_KNIFE_CLASS, graphic 0x2D22, enc;
    flippable: Some((0x2D22, 0x2D2E)),
    default_weight: Some(8.0),
    primary_ability: Some(Ability::Feint),
    secondary_ability: Some(Ability::ArmorIgnore),
    aos_str_req: 20,
    aos_min_damage: same(13),
    aos_max_damage: same(15),
    aos_speed: 42,
    ml_speed: 2.75,
    old_str_req: 20,
    old_min_damage: 13,
    old_max_damage: 15,
    old_speed: 42,
    def_miss_sound: 0x239,
    def_skill: SkillName::Fencing,
    init_min_hits: 30,
    init_max_hits: 60,
}

weapon_leaf! {
    /// `Server.Items.MagicalShortbow`.
    MAGICAL_SHORTBOW, MAGICAL_SHORTBOW_CLASS, magical_shortbow_new = "Server.Items.MagicalShortbow" : BASE_RANGED / BASE_RANGED_CLASS, graphic 0x2D2B, enc;
    @init |w, id| {
        set_resource(w, id, CraftResource::RegularWood);
    };
    flippable: Some((0x2D2B, 0x2D1F)),
    default_weight: Some(6.0),
    effect_id: 0xF42,
    ammo_type: Some("Server.Items.Arrow"),
    primary_ability: Some(Ability::LightningArrow),
    secondary_ability: Some(Ability::PsychicAttack),
    aos_str_req: 45,
    aos_min_damage: same(9),
    aos_max_damage: same(13),
    aos_speed: 38,
    ml_speed: 3.00,
    old_str_req: 45,
    old_min_damage: 9,
    old_max_damage: 13,
    old_speed: 38,
    def_max_range: 10,
    init_min_hits: 41,
    init_max_hits: 90,
}

weapon_leaf! {
    /// `Server.Items.OrnateAxe`.
    ORNATE_AXE, ORNATE_AXE_CLASS, ornate_axe_new = "Server.Items.OrnateAxe" : BASE_AXE / BASE_AXE_CLASS, graphic 0x2D28, enc;
    @init |w, id| {
        ctor::set_layer(w, id, Layer::TwoHanded);
    };
    flippable: Some((0x2D28, 0x2D34)),
    default_weight: Some(12.0),
    primary_ability: Some(Ability::Disarm),
    secondary_ability: Some(Ability::CrushingBlow),
    aos_str_req: 45,
    aos_min_damage: same(18),
    aos_max_damage: same(20),
    aos_speed: 26,
    ml_speed: 3.50,
    old_str_req: 45,
    old_min_damage: 18,
    old_max_damage: 20,
    old_speed: 26,
    def_miss_sound: 0x239,
    init_min_hits: 30,
    init_max_hits: 60,
}

weapon_leaf! {
    /// `Server.Items.RadiantScimitar`.
    RADIANT_SCIMITAR, RADIANT_SCIMITAR_CLASS, radiant_scimitar_new = "Server.Items.RadiantScimitar" : BASE_SWORD / BASE_SWORD_CLASS, graphic 0x2D33, enc;
    flippable: Some((0x2D33, 0x2D27)),
    default_weight: Some(9.0),
    primary_ability: Some(Ability::WhirlwindAttack),
    secondary_ability: Some(Ability::Bladeweave),
    aos_str_req: 20,
    aos_min_damage: same(12),
    aos_max_damage: same(14),
    aos_speed: 43,
    ml_speed: 2.50,
    old_str_req: 20,
    old_min_damage: 12,
    old_max_damage: 14,
    old_speed: 43,
    def_hit_sound: 0x23B,
    def_miss_sound: 0x239,
    init_min_hits: 30,
    init_max_hits: 60,
}

weapon_leaf! {
    /// `Server.Items.RuneBlade`.
    RUNE_BLADE, RUNE_BLADE_CLASS, rune_blade_new = "Server.Items.RuneBlade" : BASE_SWORD / BASE_SWORD_CLASS, graphic 0x2D32, enc;
    @init |w, id| {
        ctor::set_layer(w, id, Layer::TwoHanded);
    };
    flippable: Some((0x2D32, 0x2D26)),
    default_weight: Some(7.0),
    primary_ability: Some(Ability::Disarm),
    secondary_ability: Some(Ability::Bladeweave),
    aos_str_req: 30,
    aos_min_damage: same(15),
    aos_max_damage: same(17),
    aos_speed: 35,
    ml_speed: 3.00,
    old_str_req: 30,
    old_min_damage: 15,
    old_max_damage: 17,
    old_speed: 35,
    def_hit_sound: 0x23B,
    def_miss_sound: 0x239,
    init_min_hits: 30,
    init_max_hits: 60,
}

weapon_leaf! {
    /// `Server.Items.WarCleaver`.
    WAR_CLEAVER, WAR_CLEAVER_CLASS, war_cleaver_new = "Server.Items.WarCleaver" : BASE_KNIFE / BASE_KNIFE_CLASS, graphic 0x2D2F, enc;
    flippable: Some((0x2D2F, 0x2D23)),
    default_weight: Some(10.0),
    primary_ability: Some(Ability::Disarm),
    secondary_ability: Some(Ability::Bladeweave),
    aos_str_req: 15,
    aos_min_damage: same(9),
    aos_max_damage: same(11),
    aos_speed: 48,
    ml_speed: 2.25,
    old_str_req: 15,
    old_min_damage: 9,
    old_max_damage: 11,
    old_speed: 48,
    def_hit_sound: 0x23B,
    def_miss_sound: 0x239,
    init_min_hits: 30,
    init_max_hits: 60,
    def_skill: SkillName::Fencing,
    def_type: WeaponType::Piercing,
    def_animation: WeaponAnimation::Pierce1H,
}

weapon_leaf! {
    /// `Server.Items.WildStaff`.
    WILD_STAFF, WILD_STAFF_CLASS, wild_staff_new = "Server.Items.WildStaff" : BASE_STAFF / BASE_STAFF_CLASS, graphic 0x2D25, enc;
    @init |w, id| {
        set_resource(w, id, CraftResource::RegularWood);
    };
    flippable: Some((0x2D25, 0x2D31)),
    default_weight: Some(8.0),
    primary_ability: Some(Ability::Block),
    secondary_ability: Some(Ability::ForceOfNature),
    aos_str_req: 15,
    aos_min_damage: same(10),
    aos_max_damage: same(12),
    aos_speed: 48,
    ml_speed: 2.25,
    old_str_req: 15,
    old_min_damage: 10,
    old_max_damage: 12,
    old_speed: 48,
    init_min_hits: 30,
    init_max_hits: 60,
}

pub fn register(types: &mut TypeRegistry) {
    types.register_item(&ASSASSIN_SPIKE, &[]);
    types.register_item(&BUTCHERS_WAR_CLEAVER, &[]);
    types.register_item(&DIAMOND_MACE, &[]);
    types.register_item(&ELVEN_COMPOSITE_LONGBOW, &[]);
    types.register_item(&ELVEN_MACHETE, &[]);
    types.register_item(&ELVEN_SPELLBLADE, &[]);
    types.register_item(&LEAFBLADE, &[]);
    types.register_item(&MAGICAL_SHORTBOW, &[]);
    types.register_item(&ORNATE_AXE, &[]);
    types.register_item(&RADIANT_SCIMITAR, &[]);
    types.register_item(&RUNE_BLADE, &[]);
    types.register_item(&WAR_CLEAVER, &[]);
    types.register_item(&WILD_STAFF, &[]);
}
