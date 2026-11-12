//! Weapon classes of se; see weapons/mod.rs.

use super::*;

weapon_leaf! {
    /// `Server.Items.Bokuto`.
    BOKUTO, BOKUTO_CLASS, bokuto_new = "Server.Items.Bokuto" : BASE_SWORD / BASE_SWORD_CLASS, graphic 0x27A8, v0;
    flippable: Some((0x27A8, 0x27F3)),
    default_weight: Some(7.0),
    primary_ability: Some(Ability::Feint),
    secondary_ability: Some(Ability::NerveStrike),
    aos_str_req: 20,
    aos_min_damage: same(9),
    aos_max_damage: same(11),
    aos_speed: 53,
    ml_speed: 2.00,
    old_str_req: 20,
    old_min_damage: 9,
    old_max_damage: 11,
    old_speed: 53,
    def_hit_sound: 0x536,
    def_miss_sound: 0x23A,
    init_min_hits: 25,
    init_max_hits: 50,
}

weapon_leaf! {
    /// `Server.Items.Daisho`.
    DAISHO, DAISHO_CLASS, daisho_new = "Server.Items.Daisho" : BASE_SWORD / BASE_SWORD_CLASS, graphic 0x27A9, v0;
    @init |w, id| {
        ctor::set_layer(w, id, Layer::TwoHanded);
    };
    flippable: Some((0x27A9, 0x27F4)),
    default_weight: Some(8.0),
    primary_ability: Some(Ability::Feint),
    secondary_ability: Some(Ability::DoubleStrike),
    aos_str_req: 40,
    aos_min_damage: same(13),
    aos_max_damage: same(15),
    aos_speed: 40,
    ml_speed: 2.75,
    old_str_req: 40,
    old_min_damage: 13,
    old_max_damage: 15,
    old_speed: 40,
    def_hit_sound: 0x23B,
    def_miss_sound: 0x23A,
    init_min_hits: 45,
    init_max_hits: 65,
}

weapon_leaf! {
    /// `Server.Items.Kama`.
    KAMA, KAMA_CLASS, kama_new = "Server.Items.Kama" : BASE_KNIFE / BASE_KNIFE_CLASS, graphic 0x27AD, v0;
    @init |w, id| {
        ctor::set_layer(w, id, Layer::TwoHanded);
    };
    flippable: Some((0x27AD, 0x27F8)),
    default_weight: Some(7.0),
    primary_ability: Some(Ability::WhirlwindAttack),
    secondary_ability: Some(Ability::DefenseMastery),
    aos_str_req: 15,
    aos_min_damage: same(9),
    aos_max_damage: same(11),
    aos_speed: 55,
    ml_speed: 2.00,
    old_str_req: 15,
    old_min_damage: 9,
    old_max_damage: 11,
    old_speed: 55,
    def_hit_sound: 0x232,
    def_miss_sound: 0x238,
    init_min_hits: 35,
    init_max_hits: 60,
    def_skill: SkillName::Fencing,
    def_type: WeaponType::Piercing,
    def_animation: WeaponAnimation::Pierce1H,
}

weapon_leaf! {
    /// `Server.Items.Lajatang`.
    LAJATANG, LAJATANG_CLASS, lajatang_new = "Server.Items.Lajatang" : BASE_KNIFE / BASE_KNIFE_CLASS, graphic 0x27A7, v0;
    @init |w, id| {
        ctor::set_layer(w, id, Layer::TwoHanded);
    };
    flippable: Some((0x27A7, 0x27F2)),
    default_weight: Some(12.0),
    primary_ability: Some(Ability::DefenseMastery),
    secondary_ability: Some(Ability::FrenziedWhirlwind),
    aos_str_req: 65,
    aos_min_damage: same(16),
    aos_max_damage: same(18),
    aos_speed: 32,
    ml_speed: 3.50,
    old_str_req: 65,
    old_min_damage: 16,
    old_max_damage: 18,
    old_speed: 55,
    def_hit_sound: 0x232,
    def_miss_sound: 0x238,
    init_min_hits: 90,
    init_max_hits: 95,
    def_skill: SkillName::Fencing,
    def_type: WeaponType::Piercing,
    def_animation: WeaponAnimation::Pierce1H,
}

weapon_leaf! {
    /// `Server.Items.NoDachi`.
    NO_DACHI, NO_DACHI_CLASS, no_dachi_new = "Server.Items.NoDachi" : BASE_SWORD / BASE_SWORD_CLASS, graphic 0x27A2, v0;
    @init |w, id| {
        ctor::set_layer(w, id, Layer::TwoHanded);
    };
    flippable: Some((0x27A2, 0x27ED)),
    default_weight: Some(10.0),
    primary_ability: Some(Ability::CrushingBlow),
    secondary_ability: Some(Ability::RidingSwipe),
    aos_str_req: 40,
    aos_min_damage: same(16),
    aos_max_damage: same(18),
    aos_speed: 35,
    ml_speed: 3.50,
    old_str_req: 40,
    old_min_damage: 16,
    old_max_damage: 18,
    old_speed: 35,
    def_hit_sound: 0x23B,
    def_miss_sound: 0x23A,
    init_min_hits: 31,
    init_max_hits: 90,
}

weapon_leaf! {
    /// `Server.Items.Nunchaku`.
    NUNCHAKU, NUNCHAKU_CLASS, nunchaku_new = "Server.Items.Nunchaku" : BASE_BASHING / BASE_BASHING_CLASS, graphic 0x27AE, v0;
    flippable: Some((0x27AE, 0x27F9)),
    default_weight: Some(5.0),
    primary_ability: Some(Ability::Block),
    secondary_ability: Some(Ability::Feint),
    aos_str_req: 15,
    aos_min_damage: same(11),
    aos_max_damage: same(13),
    aos_speed: 47,
    ml_speed: 2.50,
    old_str_req: 15,
    old_min_damage: 11,
    old_max_damage: 13,
    old_speed: 47,
    def_hit_sound: 0x535,
    def_miss_sound: 0x239,
    init_min_hits: 40,
    init_max_hits: 55,
}

weapon_leaf! {
    /// `Server.Items.Sai`.
    SAI, SAI_CLASS, sai_new = "Server.Items.Sai" : BASE_KNIFE / BASE_KNIFE_CLASS, graphic 0x27AF, v0;
    @init |w, id| {
        ctor::set_layer(w, id, Layer::TwoHanded);
    };
    flippable: Some((0x27AF, 0x27FA)),
    default_weight: Some(7.0),
    primary_ability: Some(Ability::Block),
    secondary_ability: Some(Ability::ArmorPierce),
    aos_str_req: 15,
    aos_min_damage: same(9),
    aos_max_damage: same(11),
    aos_speed: 55,
    ml_speed: 2.00,
    old_str_req: 15,
    old_min_damage: 9,
    old_max_damage: 11,
    old_speed: 55,
    def_hit_sound: 0x23C,
    def_miss_sound: 0x232,
    init_min_hits: 55,
    init_max_hits: 60,
    def_skill: SkillName::Fencing,
    def_type: WeaponType::Piercing,
    def_animation: WeaponAnimation::Pierce1H,
}

weapon_leaf! {
    /// `Server.Items.Tekagi`.
    TEKAGI, TEKAGI_CLASS, tekagi_new = "Server.Items.Tekagi" : BASE_KNIFE / BASE_KNIFE_CLASS, graphic 0x27AB, v0;
    @init |w, id| {
        ctor::set_layer(w, id, Layer::TwoHanded);
    };
    flippable: Some((0x27AB, 0x27F6)),
    default_weight: Some(5.0),
    primary_ability: Some(Ability::DualWield),
    secondary_ability: Some(Ability::TalonStrike),
    aos_str_req: 10,
    aos_min_damage: same(10),
    aos_max_damage: same(12),
    aos_speed: 53,
    ml_speed: 2.00,
    old_str_req: 10,
    old_min_damage: 10,
    old_max_damage: 12,
    old_speed: 53,
    def_hit_sound: 0x238,
    def_miss_sound: 0x232,
    init_min_hits: 35,
    init_max_hits: 60,
    def_skill: SkillName::Fencing,
    def_type: WeaponType::Piercing,
    def_animation: WeaponAnimation::Pierce1H,
}

weapon_leaf! {
    /// `Server.Items.Tessen`.
    TESSEN, TESSEN_CLASS, tessen_new = "Server.Items.Tessen" : BASE_BASHING / BASE_BASHING_CLASS, graphic 0x27A3, v0;
    @init |w, id| {
        ctor::set_layer(w, id, Layer::TwoHanded);
    };
    flippable: Some((0x27A3, 0x27EE)),
    default_weight: Some(6.0),
    primary_ability: Some(Ability::Feint),
    secondary_ability: Some(Ability::Block),
    aos_str_req: 10,
    aos_min_damage: same(10),
    aos_max_damage: same(12),
    aos_speed: 50,
    ml_speed: 2.00,
    old_str_req: 10,
    old_min_damage: 10,
    old_max_damage: 12,
    old_speed: 50,
    def_hit_sound: 0x232,
    def_miss_sound: 0x238,
    init_min_hits: 55,
    init_max_hits: 60,
    def_animation: WeaponAnimation::Bash2H,
}

weapon_leaf! {
    /// `Server.Items.Tetsubo`.
    TETSUBO, TETSUBO_CLASS, tetsubo_new = "Server.Items.Tetsubo" : BASE_BASHING / BASE_BASHING_CLASS, graphic 0x27A6, v0;
    @init |w, id| {
        ctor::set_layer(w, id, Layer::TwoHanded);
    };
    flippable: Some((0x27A6, 0x27F1)),
    default_weight: Some(8.0),
    primary_ability: Some(Ability::FrenziedWhirlwind),
    secondary_ability: Some(Ability::CrushingBlow),
    aos_str_req: 35,
    aos_min_damage: same(12),
    aos_max_damage: same(14),
    aos_speed: 45,
    ml_speed: 2.50,
    old_str_req: 35,
    old_min_damage: 12,
    old_max_damage: 14,
    old_speed: 45,
    def_hit_sound: 0x233,
    def_miss_sound: 0x238,
    init_min_hits: 60,
    init_max_hits: 65,
    def_animation: WeaponAnimation::Bash2H,
}

weapon_leaf! {
    /// `Server.Items.Wakizashi`.
    WAKIZASHI, WAKIZASHI_CLASS, wakizashi_new = "Server.Items.Wakizashi" : BASE_SWORD / BASE_SWORD_CLASS, graphic 0x27A4, v0;
    @init |w, id| {
        ctor::set_layer(w, id, Layer::OneHanded);
    };
    flippable: Some((0x27A4, 0x27EF)),
    default_weight: Some(5.0),
    primary_ability: Some(Ability::FrenziedWhirlwind),
    secondary_ability: Some(Ability::DoubleStrike),
    aos_str_req: 20,
    aos_min_damage: same(11),
    aos_max_damage: same(13),
    aos_speed: 44,
    ml_speed: 2.50,
    old_str_req: 20,
    old_min_damage: 11,
    old_max_damage: 13,
    old_speed: 44,
    def_hit_sound: 0x23B,
    def_miss_sound: 0x23A,
    init_min_hits: 45,
    init_max_hits: 50,
}

weapon_leaf! {
    /// `Server.Items.Yumi`.
    YUMI, YUMI_CLASS, yumi_new = "Server.Items.Yumi" : BASE_RANGED / BASE_RANGED_CLASS, graphic 0x27A5, v0;
    @init |w, id| {
        ctor::set_layer(w, id, Layer::TwoHanded);
        set_resource(w, id, CraftResource::RegularWood);
    };
    flippable: Some((0x27A5, 0x27F0)),
    default_weight: Some(9.0),
    effect_id: 0xF42,
    ammo_type: Some("Server.Items.Arrow"),
    primary_ability: Some(Ability::ArmorPierce),
    secondary_ability: Some(Ability::DoubleShot),
    aos_str_req: 35,
    aos_min_damage: ml(18, 16),
    aos_max_damage: same(20),
    aos_speed: 25,
    ml_speed: 4.5,
    old_str_req: 35,
    old_min_damage: 18,
    old_max_damage: 20,
    old_speed: 25,
    def_max_range: 10,
    init_min_hits: 55,
    init_max_hits: 60,
    def_animation: WeaponAnimation::ShootBow,
}

pub fn register(types: &mut TypeRegistry) {
    types.register_item(&BOKUTO, &[]);
    types.register_item(&DAISHO, &[]);
    types.register_item(&KAMA, &[]);
    types.register_item(&LAJATANG, &[]);
    types.register_item(&NO_DACHI, &[]);
    types.register_item(&NUNCHAKU, &[]);
    types.register_item(&SAI, &[]);
    types.register_item(&TEKAGI, &[]);
    types.register_item(&TESSEN, &[]);
    types.register_item(&TETSUBO, &[]);
    types.register_item(&WAKIZASHI, &[]);
    types.register_item(&YUMI, &[]);
}
