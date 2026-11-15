//! Weapon classes of se; see weapons/mod.rs.

use super::*;

weapon_leaf! {
    /// `Server.Items.Bokuto`.
    BOKUTO, BOKUTO_CLASS, bokuto_new = "Server.Items.Bokuto" : BASE_SWORD / BASE_SWORD_CLASS, graphic 0x27A8, v0;
    flippable: Some((0x27A8, 0x27F3)),
    default_weight: Some(7.0),
    primary_ability: Some(Ability::Feint),
    secondary_ability: Some(Ability::NerveStrike),
}

weapon_leaf! {
    /// `Server.Items.Daisho`.
    DAISHO, DAISHO_CLASS, daisho_new = "Server.Items.Daisho" : BASE_SWORD / BASE_SWORD_CLASS, graphic 0x27A9, v0;
    @json;
    flippable: Some((0x27A9, 0x27F4)),
    default_weight: Some(8.0),
    primary_ability: Some(Ability::Feint),
    secondary_ability: Some(Ability::DoubleStrike),
}

weapon_leaf! {
    /// `Server.Items.Kama`.
    KAMA, KAMA_CLASS, kama_new = "Server.Items.Kama" : BASE_KNIFE / BASE_KNIFE_CLASS, graphic 0x27AD, v0;
    @json;
    flippable: Some((0x27AD, 0x27F8)),
    default_weight: Some(7.0),
    primary_ability: Some(Ability::WhirlwindAttack),
    secondary_ability: Some(Ability::DefenseMastery),
    def_skill: SkillName::Fencing,
    def_type: WeaponType::Piercing,
    def_animation: WeaponAnimation::Pierce1H,
}

weapon_leaf! {
    /// `Server.Items.Lajatang`.
    LAJATANG, LAJATANG_CLASS, lajatang_new = "Server.Items.Lajatang" : BASE_KNIFE / BASE_KNIFE_CLASS, graphic 0x27A7, v0;
    @json;
    flippable: Some((0x27A7, 0x27F2)),
    default_weight: Some(12.0),
    primary_ability: Some(Ability::DefenseMastery),
    secondary_ability: Some(Ability::FrenziedWhirlwind),
    def_skill: SkillName::Fencing,
    def_type: WeaponType::Piercing,
    def_animation: WeaponAnimation::Pierce1H,
}

weapon_leaf! {
    /// `Server.Items.NoDachi`.
    NO_DACHI, NO_DACHI_CLASS, no_dachi_new = "Server.Items.NoDachi" : BASE_SWORD / BASE_SWORD_CLASS, graphic 0x27A2, v0;
    @json;
    flippable: Some((0x27A2, 0x27ED)),
    default_weight: Some(10.0),
    primary_ability: Some(Ability::CrushingBlow),
    secondary_ability: Some(Ability::RidingSwipe),
}

weapon_leaf! {
    /// `Server.Items.Nunchaku`.
    NUNCHAKU, NUNCHAKU_CLASS, nunchaku_new = "Server.Items.Nunchaku" : BASE_BASHING / BASE_BASHING_CLASS, graphic 0x27AE, v0;
    flippable: Some((0x27AE, 0x27F9)),
    default_weight: Some(5.0),
    primary_ability: Some(Ability::Block),
    secondary_ability: Some(Ability::Feint),
}

weapon_leaf! {
    /// `Server.Items.Sai`.
    SAI, SAI_CLASS, sai_new = "Server.Items.Sai" : BASE_KNIFE / BASE_KNIFE_CLASS, graphic 0x27AF, v0;
    @json;
    flippable: Some((0x27AF, 0x27FA)),
    default_weight: Some(7.0),
    primary_ability: Some(Ability::Block),
    secondary_ability: Some(Ability::ArmorPierce),
    def_skill: SkillName::Fencing,
    def_type: WeaponType::Piercing,
    def_animation: WeaponAnimation::Pierce1H,
}

weapon_leaf! {
    /// `Server.Items.Tekagi`.
    TEKAGI, TEKAGI_CLASS, tekagi_new = "Server.Items.Tekagi" : BASE_KNIFE / BASE_KNIFE_CLASS, graphic 0x27AB, v0;
    @json;
    flippable: Some((0x27AB, 0x27F6)),
    default_weight: Some(5.0),
    primary_ability: Some(Ability::DualWield),
    secondary_ability: Some(Ability::TalonStrike),
    def_skill: SkillName::Fencing,
    def_type: WeaponType::Piercing,
    def_animation: WeaponAnimation::Pierce1H,
}

weapon_leaf! {
    /// `Server.Items.Tessen`.
    TESSEN, TESSEN_CLASS, tessen_new = "Server.Items.Tessen" : BASE_BASHING / BASE_BASHING_CLASS, graphic 0x27A3, v0;
    @json;
    flippable: Some((0x27A3, 0x27EE)),
    default_weight: Some(6.0),
    primary_ability: Some(Ability::Feint),
    secondary_ability: Some(Ability::Block),
    def_animation: WeaponAnimation::Bash2H,
}

weapon_leaf! {
    /// `Server.Items.Tetsubo`.
    TETSUBO, TETSUBO_CLASS, tetsubo_new = "Server.Items.Tetsubo" : BASE_BASHING / BASE_BASHING_CLASS, graphic 0x27A6, v0;
    @json;
    flippable: Some((0x27A6, 0x27F1)),
    default_weight: Some(8.0),
    primary_ability: Some(Ability::FrenziedWhirlwind),
    secondary_ability: Some(Ability::CrushingBlow),
    def_animation: WeaponAnimation::Bash2H,
}

weapon_leaf! {
    /// `Server.Items.Wakizashi`.
    WAKIZASHI, WAKIZASHI_CLASS, wakizashi_new = "Server.Items.Wakizashi" : BASE_SWORD / BASE_SWORD_CLASS, graphic 0x27A4, v0;
    @json;
    flippable: Some((0x27A4, 0x27EF)),
    default_weight: Some(5.0),
    primary_ability: Some(Ability::FrenziedWhirlwind),
    secondary_ability: Some(Ability::DoubleStrike),
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
