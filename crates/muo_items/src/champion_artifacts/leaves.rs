//! The Champion Artifacts classes that are plain tables (a constructor and constant overrides); the classes with logic
//! are hand-written next to this file.

use crate::simple::{hue, leaf, random, weapon_art};
use muo_content::aos::{AosArmorAttribute, AosAttribute, AosWeaponAttribute, AttrKind};
use muo_content::items::{same, set_attr};
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::{ITEM_BASE, ItemType, LootType, ops};
use muo_server::type_registry::TypeRegistry;

use crate::armor::armor_leaf;
use crate::armor::leaves::*;
use crate::clothing::clothing_leaf;
use crate::clothing::leaves::*;
use crate::clothing::special::*;
use crate::jewels::jewel_leaf;
use crate::jewels::leaves::*;
use crate::weapons::*;
use muo_content::items::armor::ArmorClass;
use muo_content::items::leaves::{KATANA, KATANA_CLASS};

/// `ArtifactLargeVase()`.
pub fn artifact_large_vase_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    ops::new_item_with_id(w, ty, 0x0B47)
}

leaf! {
    /// `Server.Items.ArtifactLargeVase`.
    ARTIFACT_LARGE_VASE = "Server.Items.ArtifactLargeVase" : ITEM_BASE, v0;
    construct: Some(|w| artifact_large_vase_new(w, &ARTIFACT_LARGE_VASE)),
}

/// `ArtifactVase()`.
pub fn artifact_vase_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    ops::new_item_with_id(w, ty, 0x0B48)
}

leaf! {
    /// `Server.Items.ArtifactVase`.
    ARTIFACT_VASE = "Server.Items.ArtifactVase" : ITEM_BASE, v0;
    construct: Some(|w| artifact_vase_new(w, &ARTIFACT_VASE)),
}

/// `DemonSkull()`.
pub fn demon_skull_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let t0 = random(w, 4);
    ops::new_item_with_id(w, ty, 0x224e + t0)
}

leaf! {
    /// `Server.Items.DemonSkull`.
    DEMON_SKULL = "Server.Items.DemonSkull" : ITEM_BASE, v0;
    construct: Some(|w| demon_skull_new(w, &DEMON_SKULL)),
}

/// `DirtPatch()`.
pub fn dirt_patch_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    ops::new_item_with_id(w, ty, 0x0913)
}

leaf! {
    /// `Server.Items.DirtPatch`.
    DIRT_PATCH = "Server.Items.DirtPatch" : ITEM_BASE, v0;
    construct: Some(|w| dirt_patch_new(w, &DIRT_PATCH)),
}

/// `EvilIdolSkull()`.
pub fn evil_idol_skull_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    ops::new_item_with_id(w, ty, 0x1F18)
}

leaf! {
    /// `Server.Items.EvilIdolSkull`.
    EVIL_IDOL_SKULL = "Server.Items.EvilIdolSkull" : ITEM_BASE, v0;
    construct: Some(|w| evil_idol_skull_new(w, &EVIL_IDOL_SKULL)),
    label_number: |_, _| 1095237,
}

/// `LavaTile()`.
pub fn lava_tile_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    ops::new_item_with_id(w, ty, 0x12EE)
}

leaf! {
    /// `Server.Items.LavaTile`.
    LAVA_TILE = "Server.Items.LavaTile" : ITEM_BASE, v0;
    construct: Some(|w| lava_tile_new(w, &LAVA_TILE)),
}

/// `SkullPole()`.
pub fn skull_pole_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    ops::new_item_with_id(w, ty, 0x2204)
}

leaf! {
    /// `Server.Items.SkullPole`.
    SKULL_POLE = "Server.Items.SkullPole" : ITEM_BASE, v0;
    construct: Some(|w| skull_pole_new(w, &SKULL_POLE)),
    default_weight: |_, _| 5.0,
}

/// `SwampTile()`.
pub fn swamp_tile_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    ops::new_item_with_id(w, ty, 0x320D)
}

leaf! {
    /// `Server.Items.SwampTile`.
    SWAMP_TILE = "Server.Items.SwampTile" : ITEM_BASE, v0;
    construct: Some(|w| swamp_tile_new(w, &SWAMP_TILE)),
}

/// `TatteredAncientMummyWrapping()`.
pub fn tattered_ancient_mummy_wrapping_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0xE21);
    hue(w, id, 0x909);
    id
}

leaf! {
    /// `Server.Items.TatteredAncientMummyWrapping`.
    TATTERED_ANCIENT_MUMMY_WRAPPING = "Server.Items.TatteredAncientMummyWrapping" : ITEM_BASE, v0;
    construct: Some(|w| tattered_ancient_mummy_wrapping_new(w, &TATTERED_ANCIENT_MUMMY_WRAPPING)),
    label_number: |_, _| 1094912,
}

/// `WaterTile()`.
pub fn water_tile_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    ops::new_item_with_id(w, ty, 0x346E)
}

leaf! {
    /// `Server.Items.WaterTile`.
    WATER_TILE = "Server.Items.WaterTile" : ITEM_BASE, v0;
    construct: Some(|w| water_tile_new(w, &WATER_TILE)),
}

/// `WindSpirit()`.
pub fn wind_spirit_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    ops::new_item_with_id(w, ty, 0x1F1F)
}

leaf! {
    /// `Server.Items.WindSpirit`.
    WIND_SPIRIT = "Server.Items.WindSpirit" : ITEM_BASE, v0;
    construct: Some(|w| wind_spirit_new(w, &WIND_SPIRIT)),
    label_number: |_, _| 1094925,
}

clothing_leaf! {
    /// `Server.Items.ANecromancerShroud`.
    A_NECROMANCER_SHROUD, A_NECROMANCER_SHROUD_CLASS, a_necromancer_shroud_new = "Server.Items.ANecromancerShroud" : ROBE / ROBE_CLASS, robe_new, hue 0, encoded false;
    @init |w, id| {
        hue(w, id, 0x455);
    };
    label_number: 1094913,
    base_cold_resistance: 5,
    init_min_hits: same(150),
    init_max_hits: same(150),
    can_fortify: false,
}

weapon_art! {
    /// `Server.Items.BraveKnightOfTheBritannia`.
    BRAVE_KNIGHT_OF_THE_BRITANNIA, BRAVE_KNIGHT_OF_THE_BRITANNIA_CLASS, brave_knight_of_the_britannia_new = "Server.Items.BraveKnightOfTheBritannia" : KATANA / KATANA_CLASS, via katana_new, v0;
    @init |w, id| {
        hue(w, id, 0x47e);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponSpeed as u32, 30);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponDamage as u32, 35);
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::HitLeechStam as u32, 48);
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::HitHarm as u32, 26);
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::HitLeechHits as u32, 22);
    };
    label_number: 1094909,
    init_min_hits: 150,
    init_max_hits: 150,
    can_fortify: false,
    get_damage_types: |_, _, _| muo_content::items::weapon_combat::DamageTypes { phys: 0, fire: 40, cold: 30, pois: 10, nrgy: 20, chaos: 0, direct: 0 },
}

jewel_leaf! {
    /// `Server.Items.DjinnisRing`.
    DJINNIS_RING, DJINNIS_RING_CLASS, djinnis_ring_new = "Server.Items.DjinnisRing" : SILVER_RING / SILVER_RING_CLASS, silver_ring_new, encoded false;
    @init |w, id| {
        set_attr(w, id, AttrKind::Attributes, AosAttribute::BonusInt as u32, 5);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::SpellDamage as u32, 10);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::CastSpeed as u32, 2);
    };
    label_number: 1094927,
    init_min_hits: 150,
    init_max_hits: 150,
}

armor_leaf! {
    /// `Server.Items.GuantletsOfAnger`.
    GUANTLETS_OF_ANGER, GUANTLETS_OF_ANGER_CLASS, guantlets_of_anger_new = "Server.Items.GuantletsOfAnger" : PLATE_GLOVES / PLATE_GLOVES_CLASS, plate_gloves_new, encoded false;
    @init |w, id| {
        hue(w, id, 0x29b);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::BonusHits as u32, 8);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::RegenHits as u32, 2);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::DefendChance as u32, 10);
    };
    label_number: 1094902,
    base_physical_resistance: 4,
    base_fire_resistance: 4,
    base_cold_resistance: 5,
    base_poison_resistance: 6,
    base_energy_resistance: 5,
    init_min_hits: 150,
    init_max_hits: 150,
    can_fortify: false,
}

clothing_leaf! {
    /// `Server.Items.LieutenantOfTheBritannianRoyalGuard`.
    LIEUTENANT_OF_THE_BRITANNIAN_ROYAL_GUARD, LIEUTENANT_OF_THE_BRITANNIAN_ROYAL_GUARD_CLASS, lieutenant_of_the_britannian_royal_guard_new = "Server.Items.LieutenantOfTheBritannianRoyalGuard" : BODY_SASH / BODY_SASH_CLASS, body_sash_new, hue 0, encoded false;
    @init |w, id| {
        hue(w, id, 0xe8);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::BonusInt as u32, 5);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::RegenMana as u32, 2);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::LowerRegCost as u32, 10);
    };
    label_number: 1094910,
    init_min_hits: same(150),
    init_max_hits: same(150),
    can_fortify: false,
}

weapon_art! {
    /// `Server.Items.OblivionsNeedle`.
    OBLIVIONS_NEEDLE, OBLIVIONS_NEEDLE_CLASS, oblivions_needle_new = "Server.Items.OblivionsNeedle" : DAGGER / DAGGER_CLASS, via dagger_new, v0;
    @init |w, id| {
        set_attr(w, id, AttrKind::Attributes, AosAttribute::BonusStam as u32, 20);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::AttackChance as u32, 20);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::DefendChance as u32, -20);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponDamage as u32, 40);
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::HitLeechStam as u32, 50);
    };
    label_number: 1094916,
    init_min_hits: 150,
    init_max_hits: 150,
    can_fortify: false,
}

weapon_art! {
    /// `Server.Items.RoyalGuardSurvivalKnife`.
    ROYAL_GUARD_SURVIVAL_KNIFE, ROYAL_GUARD_SURVIVAL_KNIFE_CLASS, royal_guard_survival_knife_new = "Server.Items.RoyalGuardSurvivalKnife" : SKINNING_KNIFE / SKINNING_KNIFE_CLASS, via skinning_knife_new, v0;
    @init |w, id| {
        set_attr(w, id, AttrKind::Attributes, AosAttribute::SpellChanneling as u32, 1);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::Luck as u32, 140);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::EnhancePotions as u32, 25);
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::UseBestSkill as u32, 1);
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::LowerStatReq as u32, 50);
    };
    label_number: 1094918,
    init_min_hits: 150,
    init_max_hits: 150,
    can_fortify: false,
}

clothing_leaf! {
    /// `Server.Items.SamaritanRobe`.
    SAMARITAN_ROBE, SAMARITAN_ROBE_CLASS, samaritan_robe_new = "Server.Items.SamaritanRobe" : ROBE / ROBE_CLASS, robe_new, hue 0, encoded false;
    @init |w, id| {
        hue(w, id, 0x2a3);
    };
    label_number: 1094926,
    base_physical_resistance: 5,
    init_min_hits: same(150),
    init_max_hits: same(150),
    can_fortify: false,
}

clothing_leaf! {
    /// `Server.Items.AcidProofRobe`.
    ACID_PROOF_ROBE, ACID_PROOF_ROBE_CLASS, acid_proof_robe_new = "Server.Items.AcidProofRobe" : ROBE / ROBE_CLASS, robe_new, hue 0, encoded false;
    @init |w, id| {
        hue(w, id, 0x455);
        ops::set_loot_type(w, id, LootType::Blessed);
    };
    label_number: 1095236,
    base_fire_resistance: 4,
    init_min_hits: same(150),
    init_max_hits: same(150),
    can_fortify: false,
}

weapon_art! {
    /// `Server.Items.Calm`.
    CALM, CALM_CLASS, calm_new = "Server.Items.Calm" : HALBERD / HALBERD_CLASS, via halberd_new, v0;
    @init |w, id| {
        hue(w, id, 0x2cb);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::SpellChanneling as u32, 1);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponSpeed as u32, 20);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponDamage as u32, 50);
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::HitLeechMana as u32, 100);
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::UseBestSkill as u32, 1);
    };
    label_number: 1094915,
    init_min_hits: 150,
    init_max_hits: 150,
    can_fortify: false,
}

clothing_leaf! {
    /// `Server.Items.CrownOfTalKeesh`.
    CROWN_OF_TAL_KEESH, CROWN_OF_TAL_KEESH_CLASS, crown_of_tal_keesh_new = "Server.Items.CrownOfTalKeesh" : BANDANA / BANDANA_CLASS, bandana_new, hue 0, encoded false;
    @init |w, id| {
        hue(w, id, 0x4F2);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::BonusInt as u32, 8);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::RegenMana as u32, 4);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::SpellDamage as u32, 10);
    };
    label_number: 1094903,
    base_physical_resistance: 0,
    base_fire_resistance: 5,
    base_cold_resistance: 9,
    base_poison_resistance: 20,
    base_energy_resistance: 20,
    init_min_hits: same(150),
    init_max_hits: same(150),
    can_fortify: false,
}

weapon_art! {
    /// `Server.Items.FangOfRactus`.
    FANG_OF_RACTUS, FANG_OF_RACTUS_CLASS, fang_of_ractus_new = "Server.Items.FangOfRactus" : KRYSS / KRYSS_CLASS, via kryss_new, v0;
    @init |w, id| {
        hue(w, id, 0x117);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::SpellChanneling as u32, 1);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::AttackChance as u32, 5);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::DefendChance as u32, 5);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponDamage as u32, 35);
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::HitPoisonArea as u32, 20);
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::ResistPoisonBonus as u32, 15);
    };
    label_number: 1094892,
    init_min_hits: 150,
    init_max_hits: 150,
    can_fortify: false,
}

armor_leaf! {
    /// `Server.Items.GladiatorsCollar`.
    GLADIATORS_COLLAR, GLADIATORS_COLLAR_CLASS, gladiators_collar_new = "Server.Items.GladiatorsCollar" : PLATE_GORGET / PLATE_GORGET_CLASS, plate_gorget_new, encoded false;
    @init |w, id| {
        hue(w, id, 0x26d);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::BonusHits as u32, 10);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::AttackChance as u32, 10);
        set_attr(w, id, AttrKind::Armor, AosArmorAttribute::MageArmor as u32, 1);
    };
    label_number: 1094917,
    base_physical_resistance: 18,
    base_fire_resistance: 18,
    base_cold_resistance: 17,
    base_poison_resistance: 18,
    base_energy_resistance: 16,
    init_min_hits: 150,
    init_max_hits: 150,
    can_fortify: false,
}

weapon_art! {
    /// `Server.Items.Pacify`.
    PACIFY, PACIFY_CLASS, pacify_new = "Server.Items.Pacify" : PIKE / PIKE_CLASS, via pike_new, v0;
    @init |w, id| {
        hue(w, id, 0x835);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::SpellChanneling as u32, 1);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::AttackChance as u32, 10);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponSpeed as u32, 20);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponDamage as u32, 50);
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::HitLeechMana as u32, 100);
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::UseBestSkill as u32, 1);
    };
    label_number: 1094929,
    init_min_hits: 150,
    init_max_hits: 150,
    can_fortify: false,
}

weapon_art! {
    /// `Server.Items.Quell`.
    QUELL, QUELL_CLASS, quell_new = "Server.Items.Quell" : BARDICHE / BARDICHE_CLASS, via bardiche_new, v0;
    @init |w, id| {
        hue(w, id, 0x225);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::SpellChanneling as u32, 1);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponSpeed as u32, 20);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponDamage as u32, 50);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::AttackChance as u32, 10);
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::HitLeechMana as u32, 100);
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::UseBestSkill as u32, 1);
    };
    label_number: 1094928,
    init_min_hits: 150,
    init_max_hits: 150,
    can_fortify: false,
}

weapon_art! {
    /// `Server.Items.Subdue`.
    SUBDUE, SUBDUE_CLASS, subdue_new = "Server.Items.Subdue" : SCYTHE / SCYTHE_CLASS, via scythe_new, v0;
    @init |w, id| {
        hue(w, id, 0x2cb);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::SpellChanneling as u32, 1);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponSpeed as u32, 20);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponDamage as u32, 50);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::AttackChance as u32, 10);
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::HitLeechMana as u32, 100);
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::UseBestSkill as u32, 1);
    };
    label_number: 1094930,
    init_min_hits: 150,
    init_max_hits: 150,
    can_fortify: false,
}

/// Every class of this file: the type and its `[TypeAlias]` names.
pub static ALL: &[(&ItemType, &[&str])] = &[
    (&ARTIFACT_LARGE_VASE, &[]),
    (&ARTIFACT_VASE, &[]),
    (&DEMON_SKULL, &[]),
    (&DIRT_PATCH, &[]),
    (&EVIL_IDOL_SKULL, &[]),
    (&LAVA_TILE, &[]),
    (&SKULL_POLE, &[]),
    (&SWAMP_TILE, &[]),
    (&TATTERED_ANCIENT_MUMMY_WRAPPING, &[]),
    (&WATER_TILE, &[]),
    (&WIND_SPIRIT, &[]),
    (&A_NECROMANCER_SHROUD, &[]),
    (&BRAVE_KNIGHT_OF_THE_BRITANNIA, &[]),
    (&DJINNIS_RING, &[]),
    (&GUANTLETS_OF_ANGER, &[]),
    (&LIEUTENANT_OF_THE_BRITANNIAN_ROYAL_GUARD, &[]),
    (&OBLIVIONS_NEEDLE, &[]),
    (&ROYAL_GUARD_SURVIVAL_KNIFE, &[]),
    (&SAMARITAN_ROBE, &[]),
    (&ACID_PROOF_ROBE, &[]),
    (&CALM, &[]),
    (&CROWN_OF_TAL_KEESH, &[]),
    (&FANG_OF_RACTUS, &[]),
    (&GLADIATORS_COLLAR, &[]),
    (&PACIFY, &[]),
    (&QUELL, &[]),
    (&SUBDUE, &[]),
];

/// `[Flippable(a, b)]` of the classes (the flip command reads it).
pub static FLIPPABLE: &[(&ItemType, i32, i32)] = &[];

/// The interfaces a class lists after its base class (`is ICommodity` and the like); none has a consumer yet.
pub static IMPLEMENTS: &[(&ItemType, &str)] = &[];

/// `ICommodity` members of a class: (type, `IsDeedable`); `DescriptionNumber` is `LabelNumber`.
pub static COMMODITIES: &[(&ItemType, bool)] = &[];

/// Classes with `[Flippable]` or `[DynamicFlipping]` and no graphics pair (the flip is the class itself).
pub static OWN_FLIP: &[&ItemType] = &[];

/// Classes with `[Furniture]`.
pub static FURNITURE: &[&ItemType] = &[];

/// Registers the classes of this file.
pub fn register(types: &mut TypeRegistry) {
    for (t, aliases) in ALL {
        types.register_item(t, aliases);
    }
}
