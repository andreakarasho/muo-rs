//! Weapon classes of staves; see weapons/mod.rs.

use super::*;

weapon_leaf! {
    /// `Server.Items.BlackStaff`.
    BLACK_STAFF, BLACK_STAFF_CLASS, black_staff_new = "Server.Items.BlackStaff" : BASE_STAFF / BASE_STAFF_CLASS, graphic 0xDF0, v0;
    flippable: Some((0xDF1, 0xDF0)),
    default_weight: Some(6.0),
    primary_ability: Some(Ability::WhirlwindAttack),
    secondary_ability: Some(Ability::ParalyzingBlow),
}

// ChylothStaff (namespace Server.Engines.Quests.Doom)
weapon_leaf! {
    /// `Server.Engines.Quests.Doom.ChylothStaff`.
    CHYLOTH_STAFF, CHYLOTH_STAFF_CLASS, chyloth_staff_new = "Server.Engines.Quests.Doom.ChylothStaff" : BLACK_STAFF / BLACK_STAFF_CLASS, via black_staff_new, v0;
    @json;
}

weapon_leaf! {
    /// `Server.Items.GlacialStaff`.
    GLACIAL_STAFF, GLACIAL_STAFF_CLASS, glacial_staff_new = "Server.Items.GlacialStaff" : BLACK_STAFF / BLACK_STAFF_CLASS, via black_staff_new, v0;
    @init |w, id| {
        ctor::set_hue(w, id, 0x480);
        let v = 5 * random_min_max(w, 1, 5); set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::HitHarm as u32, v);
        let v = random_min_max(w, 5, 10); set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::MageWeapon as u32, v);
        let v = 20 + 5 * random_min_max(w, 0, 6); set_attr(w, id, AttrKind::Element, AosElementAttribute::Cold as u32, v);
    };
}

weapon_leaf! {
    /// `Server.Items.GlassStaff`.
    GLASS_STAFF, GLASS_STAFF_CLASS, glass_staff_new = "Server.Items.GlassStaff" : BASE_STAFF / BASE_STAFF_CLASS, graphic 0x905, enc;
    @init |w, id| {
        set_resource(w, id, CraftResource::None);
    };
    @item {
        required_races: |_, _| ALLOW_GARGOYLES_ONLY,
    }
    flippable: Some((0x905, 0x4070)),
    default_weight: Some(4.0),
    primary_ability: Some(Ability::DoubleStrike),
    secondary_ability: Some(Ability::MortalStrike),
}

weapon_leaf! {
    /// `Server.Items.GnarledStaff`.
    GNARLED_STAFF, GNARLED_STAFF_CLASS, gnarled_staff_new = "Server.Items.GnarledStaff" : BASE_STAFF / BASE_STAFF_CLASS, graphic 0x13F8, v0;
    @init |w, id| {
        set_resource(w, id, CraftResource::RegularWood);
    };
    flippable: Some((0x13F8, 0x13F9)),
    default_weight: Some(3.0),
    primary_ability: Some(Ability::ConcussionBlow),
    secondary_ability: Some(Ability::ForceOfNature),
}

weapon_leaf! {
    /// `Server.Items.QuarterStaff`.
    QUARTER_STAFF, QUARTER_STAFF_CLASS, quarter_staff_new = "Server.Items.QuarterStaff" : BASE_STAFF / BASE_STAFF_CLASS, graphic 0xE89, v0;
    @init |w, id| {
        set_resource(w, id, CraftResource::RegularWood);
    };
    flippable: Some((0xE89, 0xE8A)),
    default_weight: Some(4.0),
    primary_ability: Some(Ability::DoubleStrike),
    secondary_ability: Some(Ability::ConcussionBlow),
}

weapon_leaf! {
    /// `Server.Items.SerpentstoneStaff`.
    SERPENTSTONE_STAFF, SERPENTSTONE_STAFF_CLASS, serpentstone_staff_new = "Server.Items.SerpentstoneStaff" : BASE_STAFF / BASE_STAFF_CLASS, graphic 0x906, enc;
    @init |w, id| {
        set_resource(w, id, CraftResource::RegularWood);
    };
    @item {
        required_races: |_, _| ALLOW_GARGOYLES_ONLY,
    }
    flippable: Some((0x906, 0x406F)),
    primary_ability: Some(Ability::CrushingBlow),
    secondary_ability: Some(Ability::Dismount),
}

weapon_leaf! {
    /// `Server.Items.ShepherdsCrook`.
    SHEPHERDS_CROOK, SHEPHERDS_CROOK_CLASS, shepherds_crook_new = "Server.Items.ShepherdsCrook" : BASE_STAFF / BASE_STAFF_CLASS, graphic 0xE81, v0;
    @init |w, id| {
        set_resource(w, id, CraftResource::RegularWood);
    };
    @item {
        on_double_click: shepherds_crook_on_double_click,
    }
    flippable: Some((0xE81, 0xE82)),
    default_weight: Some(4.0),
    primary_ability: Some(Ability::CrushingBlow),
    secondary_ability: Some(Ability::Disarm),
}

pub fn register(types: &mut TypeRegistry) {
    types.register_item(&BLACK_STAFF, &[]);
    types.register_item(&GLACIAL_STAFF, &[]);
    types.register_item(&CHYLOTH_STAFF, &[]);
    types.register_item(&GLASS_STAFF, &[]);
    types.register_item(&GNARLED_STAFF, &[]);
    types.register_item(&QUARTER_STAFF, &[]);
    types.register_item(&SERPENTSTONE_STAFF, &["Server.Items.SerpentStoneStaff"]);
    types.register_item(&SHEPHERDS_CROOK, &[]);
}
