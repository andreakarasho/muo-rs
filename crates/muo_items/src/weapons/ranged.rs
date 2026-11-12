//! Weapon classes of ranged; see weapons/mod.rs.

use super::*;

weapon_leaf! {
    /// `Server.Items.AssassinsShortbow`.
    ASSASSINS_SHORTBOW, ASSASSINS_SHORTBOW_CLASS, assassins_shortbow_new = "Server.Items.AssassinsShortbow" : MAGICAL_SHORTBOW / MAGICAL_SHORTBOW_CLASS, via magical_shortbow_new, enc;
    @init |w, id| {
        set_attr(w, id, AttrKind::Attributes, AosAttribute::AttackChance as u32, 3);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponDamage as u32, 4);
    };
    label_number: 1073512,
}

weapon_leaf! {
    /// `Server.Items.BarbedLongbow`.
    BARBED_LONGBOW, BARBED_LONGBOW_CLASS, barbed_longbow_new = "Server.Items.BarbedLongbow" : ELVEN_COMPOSITE_LONGBOW / ELVEN_COMPOSITE_LONGBOW_CLASS, via elven_composite_longbow_new, enc;
    @init |w, id| {
        set_attr(w, id, AttrKind::Attributes, AosAttribute::ReflectPhysical as u32, 12);
    };
    label_number: 1073505,
}

weapon_leaf! {
    /// `Server.Items.CompositeBow`.
    COMPOSITE_BOW, COMPOSITE_BOW_CLASS, composite_bow_new = "Server.Items.CompositeBow" : BASE_RANGED / BASE_RANGED_CLASS, graphic 0x26C2, v0;
    @init |w, id| {
        set_resource(w, id, CraftResource::RegularWood);
    };
    flippable: Some((0x26C2, 0x26CC)),
    default_weight: Some(5.0),
    effect_id: 0xF42,
    ammo_type: Some("Server.Items.Arrow"),
    primary_ability: Some(Ability::ArmorIgnore),
    secondary_ability: Some(Ability::MovingShot),
    aos_str_req: 45,
    aos_min_damage: ml(15, 13),
    aos_max_damage: same(17),
    aos_speed: 25,
    ml_speed: 4.00,
    old_str_req: 45,
    old_min_damage: 15,
    old_max_damage: 17,
    old_speed: 25,
    def_max_range: 10,
    init_min_hits: 31,
    init_max_hits: 70,
    def_animation: WeaponAnimation::ShootBow,
}

weapon_leaf! {
    /// `Server.Items.Crossbow`.
    CROSSBOW, CROSSBOW_CLASS, crossbow_new = "Server.Items.Crossbow" : BASE_RANGED / BASE_RANGED_CLASS, graphic 0xF50, v0;
    @init |w, id| {
        ctor::set_layer(w, id, Layer::TwoHanded);
        set_resource(w, id, CraftResource::RegularWood);
    };
    flippable: Some((0xF50, 0xF4F)),
    default_weight: Some(7.0),
    effect_id: 0x1BFE,
    ammo_type: Some("Server.Items.Bolt"),
    primary_ability: Some(Ability::ConcussionBlow),
    secondary_ability: Some(Ability::MortalStrike),
    aos_str_req: 35,
    aos_min_damage: same(18),
    aos_max_damage: ml(20, 22),
    aos_speed: 24,
    ml_speed: 4.50,
    old_str_req: 30,
    old_min_damage: 8,
    old_max_damage: 43,
    old_speed: 18,
    def_max_range: 8,
    init_min_hits: 31,
    init_max_hits: 80,
}

weapon_leaf! {
    /// `Server.Items.FrozenLongbow`.
    FROZEN_LONGBOW, FROZEN_LONGBOW_CLASS, frozen_longbow_new = "Server.Items.FrozenLongbow" : ELVEN_COMPOSITE_LONGBOW / ELVEN_COMPOSITE_LONGBOW_CLASS, via elven_composite_longbow_new, enc;
    @init |w, id| {
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponSpeed as u32, -5);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::DefendChance as u32, 10);
    };
    label_number: 1073507,
}

weapon_leaf! {
    /// `Server.Items.HeavyCrossbow`.
    HEAVY_CROSSBOW, HEAVY_CROSSBOW_CLASS, heavy_crossbow_new = "Server.Items.HeavyCrossbow" : BASE_RANGED / BASE_RANGED_CLASS, graphic 0x13FD, v0;
    @init |w, id| {
        ctor::set_layer(w, id, Layer::TwoHanded);
        set_resource(w, id, CraftResource::RegularWood);
    };
    flippable: Some((0x13FD, 0x13FC)),
    default_weight: Some(9.0),
    effect_id: 0x1BFE,
    ammo_type: Some("Server.Items.Bolt"),
    primary_ability: Some(Ability::MovingShot),
    secondary_ability: Some(Ability::Dismount),
    aos_str_req: 80,
    aos_min_damage: ml(19, 20),
    aos_max_damage: ml(20, 24),
    aos_speed: 22,
    ml_speed: 5.00,
    old_str_req: 40,
    old_min_damage: 11,
    old_max_damage: 56,
    old_speed: 10,
    def_max_range: 8,
    init_min_hits: 31,
    init_max_hits: 100,
}

weapon_leaf! {
    /// `Server.Items.JukaBow`.
    JUKA_BOW, JUKA_BOW_CLASS, juka_bow_new = "Server.Items.JukaBow" : BOW / BOW_CLASS, via bow_new, v0;
    @init |w, id| {
        set_resource(w, id, CraftResource::RegularWood);
    };
    @item {
        on_double_click: juka_bow_on_double_click,
    }
    flippable: Some((0x13B2, 0x13B1)),
    aos_str_req: 80,
    aos_dex_req: 80,
    old_str_req: 80,
    old_dex_req: 80,
}

weapon_leaf! {
    /// `Server.Items.LightweightShortbow`.
    LIGHTWEIGHT_SHORTBOW, LIGHTWEIGHT_SHORTBOW_CLASS, lightweight_shortbow_new = "Server.Items.LightweightShortbow" : MAGICAL_SHORTBOW / MAGICAL_SHORTBOW_CLASS, via magical_shortbow_new, enc;
    @init |w, id| {
        ctor::set_balanced(w, id, true);
    };
    label_number: 1073510,
}

weapon_leaf! {
    /// `Server.Items.LongbowOfMight`.
    LONGBOW_OF_MIGHT, LONGBOW_OF_MIGHT_CLASS, longbow_of_might_new = "Server.Items.LongbowOfMight" : ELVEN_COMPOSITE_LONGBOW / ELVEN_COMPOSITE_LONGBOW_CLASS, via elven_composite_longbow_new, enc;
    @init |w, id| {
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponDamage as u32, 5);
    };
    label_number: 1073508,
}

weapon_leaf! {
    /// `Server.Items.MysticalShortbow`.
    MYSTICAL_SHORTBOW, MYSTICAL_SHORTBOW_CLASS, mystical_shortbow_new = "Server.Items.MysticalShortbow" : MAGICAL_SHORTBOW / MAGICAL_SHORTBOW_CLASS, via magical_shortbow_new, enc;
    @init |w, id| {
        set_attr(w, id, AttrKind::Attributes, AosAttribute::SpellChanneling as u32, 1);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::CastSpeed as u32, -1);
    };
    label_number: 1073511,
}

weapon_leaf! {
    /// `Server.Items.RangersShortbow`.
    RANGERS_SHORTBOW, RANGERS_SHORTBOW_CLASS, rangers_shortbow_new = "Server.Items.RangersShortbow" : MAGICAL_SHORTBOW / MAGICAL_SHORTBOW_CLASS, via magical_shortbow_new, enc;
    @init |w, id| {
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponSpeed as u32, 5);
    };
    label_number: 1073509,
}

weapon_leaf! {
    /// `Server.Items.RepeatingCrossbow`.
    REPEATING_CROSSBOW, REPEATING_CROSSBOW_CLASS, repeating_crossbow_new = "Server.Items.RepeatingCrossbow" : BASE_RANGED / BASE_RANGED_CLASS, graphic 0x26C3, v0;
    @init |w, id| {
        set_resource(w, id, CraftResource::RegularWood);
    };
    flippable: Some((0x26C3, 0x26CD)),
    default_weight: Some(6.0),
    effect_id: 0x1BFE,
    ammo_type: Some("Server.Items.Bolt"),
    primary_ability: Some(Ability::DoubleStrike),
    secondary_ability: Some(Ability::MovingShot),
    aos_str_req: 30,
    aos_min_damage: ml(10, 8),
    aos_max_damage: same(12),
    aos_speed: 41,
    ml_speed: 2.75,
    old_str_req: 30,
    old_min_damage: 10,
    old_max_damage: 12,
    old_speed: 41,
    def_max_range: 7,
    init_min_hits: 31,
    init_max_hits: 80,
}

weapon_leaf! {
    /// `Server.Items.SlayerLongbow`.
    SLAYER_LONGBOW, SLAYER_LONGBOW_CLASS, slayer_longbow_new = "Server.Items.SlayerLongbow" : ELVEN_COMPOSITE_LONGBOW / ELVEN_COMPOSITE_LONGBOW_CLASS, via elven_composite_longbow_new, enc;
    @init |w, id| {
        let v = random_min_max(w, 1, 27); ctor::set_slayer2(w, id, v);
    };
    label_number: 1073506,
}

pub fn register(types: &mut TypeRegistry) {
    types.register_item(&ASSASSINS_SHORTBOW, &[]);
    types.register_item(&BARBED_LONGBOW, &[]);
    types.register_item(&COMPOSITE_BOW, &[]);
    types.register_item(&CROSSBOW, &[]);
    types.register_item(&FROZEN_LONGBOW, &[]);
    types.register_item(&HEAVY_CROSSBOW, &[]);
    types.register_item(&JUKA_BOW, &[]);
    types.register_item(&LIGHTWEIGHT_SHORTBOW, &[]);
    types.register_item(&LONGBOW_OF_MIGHT, &[]);
    types.register_item(&MYSTICAL_SHORTBOW, &[]);
    types.register_item(&RANGERS_SHORTBOW, &[]);
    types.register_item(&REPEATING_CROSSBOW, &[]);
    types.register_item(&SLAYER_LONGBOW, &[]);
}
