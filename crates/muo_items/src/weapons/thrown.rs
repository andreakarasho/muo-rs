//! Weapon classes of thrown; see weapons/mod.rs.

use super::*;

weapon_leaf! {
    /// `Server.Items.AbyssReaver`.
    ABYSS_REAVER, ABYSS_REAVER_CLASS, abyss_reaver_new = "Server.Items.AbyssReaver" : CYCLONE / CYCLONE_CLASS, via cyclone_new, enc;
    @init |w, id| {
        let b = f64::from(random_min_max(w, 5, 10)); ctor::skill_bonus(w, id, 0, SkillName::Throwing, b);
        let v = random_min_max(w, 25, 35); set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponDamage as u32, v);
        ctor::set_slayer(w, id, SlayerName::Exorcism);
    };
    label_number: 1112694,
}

weapon_leaf! {
    /// `Server.Items.BansheesCall`.
    BANSHEES_CALL, BANSHEES_CALL_CLASS, banshees_call_new = "Server.Items.BansheesCall" : CYCLONE / CYCLONE_CLASS, via cyclone_new, enc;
    @init |w, id| {
        ctor::set_hue(w, id, 1266);
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::HitHarm as u32, 40);
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::HitLeechHits as u32, 45);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::BonusStr as u32, 5);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponSpeed as u32, 30);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponDamage as u32, 50);
        ctor::set_velocity(w, id, 35);
        set_attr(w, id, AttrKind::Element, AosElementAttribute::Cold as u32, 100);
    };
    label_number: 1113529,
}

weapon_leaf! {
    /// `Server.Items.Boomerang`.
    BOOMERANG, BOOMERANG_CLASS, boomerang_new = "Server.Items.Boomerang" : BASE_THROWN / BASE_THROWN_CLASS, graphic 0x8FF, enc;
    @init |w, id| {
        ctor::set_layer(w, id, Layer::OneHanded);
    };
    @item {
        required_races: |_, _| ALLOW_GARGOYLES_ONLY,
    }
    default_weight: Some(4.0),
    min_throw_range: 4,
    primary_ability: Some(Ability::MysticArc),
    secondary_ability: Some(Ability::ConcussionBlow),
}

weapon_leaf! {
    /// `Server.Items.Cyclone`.
    CYCLONE, CYCLONE_CLASS, cyclone_new = "Server.Items.Cyclone" : BASE_THROWN / BASE_THROWN_CLASS, graphic 0x901, enc;
    @init |w, id| {
        ctor::set_layer(w, id, Layer::OneHanded);
    };
    @item {
        required_races: |_, _| ALLOW_GARGOYLES_ONLY,
    }
    default_weight: Some(6.0),
    min_throw_range: 6,
    primary_ability: Some(Ability::MovingShot),
    secondary_ability: Some(Ability::InfusedThrow),
}

weapon_leaf! {
    /// `Server.Items.RaptorClaw`.
    RAPTOR_CLAW, RAPTOR_CLAW_CLASS, raptor_claw_new = "Server.Items.RaptorClaw" : BOOMERANG / BOOMERANG_CLASS, via boomerang_new, enc;
    @init |w, id| {
        ctor::set_hue(w, id, 53);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::AttackChance as u32, 12);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponSpeed as u32, 30);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponDamage as u32, 35);
        ctor::set_slayer(w, id, SlayerName::Silver);
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::HitLeechStam as u32, 40);
    };
    label_number: 1112394,
}

weapon_leaf! {
    /// `Server.Items.SoulGlaive`.
    SOUL_GLAIVE, SOUL_GLAIVE_CLASS, soul_glaive_new = "Server.Items.SoulGlaive" : BASE_THROWN / BASE_THROWN_CLASS, graphic 0x090A, enc;
    @init |w, id| {
        ctor::set_layer(w, id, Layer::OneHanded);
    };
    @item {
        required_races: |_, _| ALLOW_GARGOYLES_ONLY,
    }
    default_weight: Some(8.0),
    min_throw_range: 8,
    primary_ability: Some(Ability::ArmorIgnore),
    secondary_ability: Some(Ability::MortalStrike),
}

weapon_leaf! {
    /// `Server.Items.StoneSlithClaw`.
    STONE_SLITH_CLAW, STONE_SLITH_CLAW_CLASS, stone_slith_claw_new = "Server.Items.StoneSlithClaw" : CYCLONE / CYCLONE_CLASS, via cyclone_new, enc;
    @init |w, id| {
        ctor::set_hue(w, id, 1150);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponSpeed as u32, 25);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponDamage as u32, 45);
        ctor::set_slayer(w, id, SlayerName::DaemonDismissal);
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::HitHarm as u32, 40);
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::HitLowerDefend as u32, 40);
    };
    label_number: 1112393,
}

weapon_leaf! {
    /// `Server.Items.StormCaller`.
    STORM_CALLER, STORM_CALLER_CLASS, storm_caller_new = "Server.Items.StormCaller" : BOOMERANG / BOOMERANG_CLASS, via boomerang_new, enc;
    @init |w, id| {
        ctor::set_hue(w, id, 456);
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::HitLightning as u32, 40);
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::HitLowerDefend as u32, 30);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::BonusStr as u32, 5);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponSpeed as u32, 30);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponDamage as u32, 40);
        set_attr(w, id, AttrKind::Element, AosElementAttribute::Physical as u32, 20);
        set_attr(w, id, AttrKind::Element, AosElementAttribute::Fire as u32, 20);
        set_attr(w, id, AttrKind::Element, AosElementAttribute::Cold as u32, 20);
        set_attr(w, id, AttrKind::Element, AosElementAttribute::Poison as u32, 20);
        set_attr(w, id, AttrKind::Element, AosElementAttribute::Energy as u32, 20);
    };
    label_number: 1113530,
}

weapon_leaf! {
    /// `Server.Items.ValkyriesGlaive`.
    VALKYRIES_GLAIVE, VALKYRIES_GLAIVE_CLASS, valkyries_glaive_new = "Server.Items.ValkyriesGlaive" : SOUL_GLAIVE / SOUL_GLAIVE_CLASS, via soul_glaive_new, enc;
    @init |w, id| {
        ctor::set_hue(w, id, 1651);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::SpellChanneling as u32, 1);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::BonusStr as u32, 5);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponSpeed as u32, 20);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponDamage as u32, 20);
        ctor::set_slayer(w, id, SlayerName::Silver);
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::HitFireball as u32, 40);
    };
    label_number: 1113531,
    artifact_rarity: 5,
}

weapon_leaf! {
    /// `Server.Items.WindOfCorruption`.
    WIND_OF_CORRUPTION, WIND_OF_CORRUPTION_CLASS, wind_of_corruption_new = "Server.Items.WindOfCorruption" : CYCLONE / CYCLONE_CLASS, via cyclone_new, enc;
    @init |w, id| {
        ctor::set_hue(w, id, 1171);
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::HitLeechStam as u32, 40);
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::HitLowerDefend as u32, 40);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponSpeed as u32, 30);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponDamage as u32, 50);
        ctor::set_slayer(w, id, SlayerName::Fey);
        set_attr(w, id, AttrKind::Element, AosElementAttribute::Chaos as u32, 100);
    };
    label_number: 1150358,
}

pub fn register(types: &mut TypeRegistry) {
    types.register_item(&ABYSS_REAVER, &[]);
    types.register_item(&BANSHEES_CALL, &[]);
    types.register_item(&BOOMERANG, &[]);
    types.register_item(&CYCLONE, &[]);
    types.register_item(&RAPTOR_CLAW, &[]);
    types.register_item(&SOUL_GLAIVE, &[]);
    types.register_item(&STONE_SLITH_CLAW, &[]);
    types.register_item(&STORM_CALLER, &[]);
    types.register_item(&VALKYRIES_GLAIVE, &[]);
    types.register_item(&WIND_OF_CORRUPTION, &[]);
}
