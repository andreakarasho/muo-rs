//! The concrete armor and shield classes that are tables. Classes with logic of their own are in `special.rs`.

use muo_content::aos::{AosArmorAttribute, AosAttribute, AosWeaponAttribute, AttrKind};
use muo_content::craft::CraftResource;
use muo_content::items::armor::{
    ArmorClass, ArmorMaterialType, ArmorMeditationAllowance, BASE_ARMOR, BASE_ARMOR_CLASS,
    BASE_SHIELD, BASE_SHIELD_CLASS, new_armor,
};
use muo_content::items::set_attr;
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::{ItemType, LootType, ops};
use muo_server::race::{ALLOW_ELVES_ONLY, ALLOW_GARGOYLES_ONLY};
use muo_server::skills::SkillName;
use muo_server::type_registry::TypeRegistry;

use super::special::*;
use super::{hue, item_id, max_and_hit_points, skill_bonus};
use muo_content::items::leaves::{CHAIN_CHEST, CHAIN_CHEST_CLASS, PLATE_CHEST, PLATE_CHEST_CLASS};

armor_leaf! {
    /// `Server.Items.ArmorOfFortune`.
    ARMOR_OF_FORTUNE, ARMOR_OF_FORTUNE_CLASS, armor_of_fortune_new = "Server.Items.ArmorOfFortune" : STUDDED_CHEST / STUDDED_CHEST_CLASS, studded_chest_new, encoded false;
    @init |w, id| {
        hue(w, id, 0x501);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::Luck as u32, 200);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::DefendChance as u32, 15);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::LowerRegCost as u32, 40);
        set_attr(w, id, AttrKind::Armor, AosArmorAttribute::MageArmor as u32, 1);
    };
    label_number: 1061098,
    artifact_rarity: 11,
    init_min_hits: 255,
    init_max_hits: 255,
}

armor_leaf! {
    /// `Server.Items.BrambleCoat`.
    BRAMBLE_COAT, BRAMBLE_COAT_CLASS, bramble_coat_new = "Server.Items.BrambleCoat" : WOODLAND_CHEST / WOODLAND_CHEST_CLASS, woodland_chest_new, encoded true;
    @init |w, id| {
        hue(w, id, 0x1);
        set_attr(w, id, AttrKind::Armor, AosArmorAttribute::SelfRepair as u32, 3);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::BonusHits as u32, 4);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::Luck as u32, 150);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::ReflectPhysical as u32, 25);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::DefendChance as u32, 15);
    };
    label_number: 1072925,
    base_physical_resistance: 10,
    base_fire_resistance: 8,
    base_cold_resistance: 7,
    base_poison_resistance: 8,
    base_energy_resistance: 7,
}

armor_leaf! {
    /// `Server.Items.IronwoodCrown`.
    IRONWOOD_CROWN, IRONWOOD_CROWN_CLASS, ironwood_crown_new = "Server.Items.IronwoodCrown" : RAVEN_HELM / RAVEN_HELM_CLASS, raven_helm_new, encoded true;
    @init |w, id| {
        hue(w, id, 0x1);
        set_attr(w, id, AttrKind::Armor, AosArmorAttribute::SelfRepair as u32, 3);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::BonusStr as u32, 5);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::BonusDex as u32, 5);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::BonusInt as u32, 5);
    };
    label_number: 1072924,
    base_physical_resistance: 10,
    base_fire_resistance: 6,
    base_cold_resistance: 7,
    base_poison_resistance: 7,
    base_energy_resistance: 10,
}

armor_leaf! {
    /// `Server.Items.SongWovenMantle`.
    SONG_WOVEN_MANTLE, SONG_WOVEN_MANTLE_CLASS, song_woven_mantle_new = "Server.Items.SongWovenMantle" : LEAF_ARMS / LEAF_ARMS_CLASS, leaf_arms_new, encoded true;
    @init |w, id| {
        hue(w, id, 0x493);
        skill_bonus(w, id, 0, SkillName::Musicianship, 10.0);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::Luck as u32, 100);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::DefendChance as u32, 5);
    };
    label_number: 1072931,
    base_physical_resistance: 14,
    base_cold_resistance: 14,
    base_energy_resistance: 16,
}

armor_leaf! {
    /// `Server.Items.SpellWovenBritches`.
    SPELL_WOVEN_BRITCHES, SPELL_WOVEN_BRITCHES_CLASS, spell_woven_britches_new = "Server.Items.SpellWovenBritches" : LEAF_LEGS / LEAF_LEGS_CLASS, leaf_legs_new, encoded true;
    @init |w, id| {
        hue(w, id, 0x487);
        skill_bonus(w, id, 0, SkillName::Meditation, 10.0);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::BonusInt as u32, 8);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::SpellDamage as u32, 10);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::LowerManaCost as u32, 10);
    };
    label_number: 1072929,
    base_fire_resistance: 15,
    base_poison_resistance: 16,
}

armor_leaf! {
    /// `Server.Items.StitchersMittens`.
    STITCHERS_MITTENS, STITCHERS_MITTENS_CLASS, stitchers_mittens_new = "Server.Items.StitchersMittens" : LEAF_GLOVES / LEAF_GLOVES_CLASS, leaf_gloves_new, encoded true;
    @init |w, id| {
        hue(w, id, 0x481);
        skill_bonus(w, id, 0, SkillName::Healing, 10.0);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::BonusDex as u32, 5);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::LowerRegCost as u32, 30);
    };
    label_number: 1072932,
    base_physical_resistance: 20,
    base_cold_resistance: 20,
}

armor_leaf! {
    /// `Server.Items.GauntletsOfNobility`.
    GAUNTLETS_OF_NOBILITY, GAUNTLETS_OF_NOBILITY_CLASS, gauntlets_of_nobility_new = "Server.Items.GauntletsOfNobility" : RINGMAIL_GLOVES / RINGMAIL_GLOVES_CLASS, ringmail_gloves_new, encoded false;
    @init |w, id| {
        hue(w, id, 0x4FE);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::BonusStr as u32, 8);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::Luck as u32, 100);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponDamage as u32, 20);
    };
    label_number: 1061092,
    artifact_rarity: 11,
    base_physical_resistance: 18,
    base_poison_resistance: 20,
    init_min_hits: 255,
    init_max_hits: 255,
}

armor_leaf! {
    /// `Server.Items.HelmOfInsight`.
    HELM_OF_INSIGHT, HELM_OF_INSIGHT_CLASS, helm_of_insight_new = "Server.Items.HelmOfInsight" : PLATE_HELM / PLATE_HELM_CLASS, plate_helm_new, encoded false;
    @init |w, id| {
        hue(w, id, 0x554);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::BonusInt as u32, 8);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::BonusMana as u32, 15);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::RegenMana as u32, 2);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::LowerManaCost as u32, 8);
    };
    label_number: 1061096,
    artifact_rarity: 11,
    base_energy_resistance: 17,
    init_min_hits: 255,
    init_max_hits: 255,
}

armor_leaf! {
    /// `Server.Items.HolyKnightsBreastplate`.
    HOLY_KNIGHTS_BREASTPLATE, HOLY_KNIGHTS_BREASTPLATE_CLASS, holy_knights_breastplate_new = "Server.Items.HolyKnightsBreastplate" : PLATE_CHEST / PLATE_CHEST_CLASS, plate_chest_new, encoded false;
    @init |w, id| {
        hue(w, id, 0x47E);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::BonusHits as u32, 10);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::ReflectPhysical as u32, 15);
    };
    label_number: 1061097,
    artifact_rarity: 11,
    base_physical_resistance: 35,
    init_min_hits: 255,
    init_max_hits: 255,
}

armor_leaf! {
    /// `Server.Items.InquisitorsResolution`.
    INQUISITORS_RESOLUTION, INQUISITORS_RESOLUTION_CLASS, inquisitors_resolution_new = "Server.Items.InquisitorsResolution" : PLATE_GLOVES / PLATE_GLOVES_CLASS, plate_gloves_new, encoded false;
    @init |w, id| {
        hue(w, id, 0x4F2);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::CastRecovery as u32, 3);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::LowerManaCost as u32, 8);
        set_attr(w, id, AttrKind::Armor, AosArmorAttribute::MageArmor as u32, 1);
    };
    label_number: 1060206,
    artifact_rarity: 10,
    base_cold_resistance: 22,
    base_energy_resistance: 17,
    init_min_hits: 255,
    init_max_hits: 255,
}

armor_leaf! {
    /// `Server.Items.JackalsCollar`.
    JACKALS_COLLAR, JACKALS_COLLAR_CLASS, jackals_collar_new = "Server.Items.JackalsCollar" : PLATE_GORGET / PLATE_GORGET_CLASS, plate_gorget_new, encoded false;
    @init |w, id| {
        hue(w, id, 0x6D1);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::BonusDex as u32, 15);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::RegenHits as u32, 2);
    };
    label_number: 1061594,
    artifact_rarity: 11,
    base_fire_resistance: 23,
    base_cold_resistance: 17,
    init_min_hits: 255,
    init_max_hits: 255,
}

armor_leaf! {
    /// `Server.Items.LeggingsOfBane`.
    LEGGINGS_OF_BANE, LEGGINGS_OF_BANE_CLASS, leggings_of_bane_new = "Server.Items.LeggingsOfBane" : CHAIN_LEGS / CHAIN_LEGS_CLASS, chain_legs_new, encoded false;
    @init |w, id| {
        hue(w, id, 0x4F5);
        set_attr(w, id, AttrKind::Armor, AosArmorAttribute::DurabilityBonus as u32, 100);
        max_and_hit_points(w, id, 255);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::BonusStam as u32, 8);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::AttackChance as u32, 20);
    };
    label_number: 1061100,
    artifact_rarity: 11,
    base_poison_resistance: 36,
    init_min_hits: 255,
    init_max_hits: 255,
}

armor_leaf! {
    /// `Server.Items.MidnightBracers`.
    MIDNIGHT_BRACERS, MIDNIGHT_BRACERS_CLASS, midnight_bracers_new = "Server.Items.MidnightBracers" : BONE_ARMS / BONE_ARMS_CLASS, bone_arms_new, encoded false;
    @init |w, id| {
        hue(w, id, 0x455);
        skill_bonus(w, id, 0, SkillName::Necromancy, 20.0);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::SpellDamage as u32, 10);
        set_attr(w, id, AttrKind::Armor, AosArmorAttribute::MageArmor as u32, 1);
    };
    label_number: 1061093,
    artifact_rarity: 11,
    base_physical_resistance: 23,
    init_min_hits: 255,
    init_max_hits: 255,
}

armor_leaf! {
    /// `Server.Items.OrnateCrownOfTheHarrower`.
    ORNATE_CROWN_OF_THE_HARROWER, ORNATE_CROWN_OF_THE_HARROWER_CLASS, ornate_crown_of_the_harrower_new = "Server.Items.OrnateCrownOfTheHarrower" : BONE_HELM / BONE_HELM_CLASS, bone_helm_new, encoded false;
    @init |w, id| {
        hue(w, id, 0x4F6);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::RegenHits as u32, 2);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::RegenStam as u32, 3);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponDamage as u32, 25);
    };
    label_number: 1061095,
    artifact_rarity: 11,
    base_poison_resistance: 17,
    init_min_hits: 255,
    init_max_hits: 255,
}

armor_leaf! {
    /// `Server.Items.ShadowDancerLeggings`.
    SHADOW_DANCER_LEGGINGS, SHADOW_DANCER_LEGGINGS_CLASS, shadow_dancer_leggings_new = "Server.Items.ShadowDancerLeggings" : LEATHER_LEGS / LEATHER_LEGS_CLASS, leather_legs_new, encoded false;
    @init |w, id| {
        item_id(w, id, 0x13D2);
        hue(w, id, 0x455);
        skill_bonus(w, id, 0, SkillName::Stealth, 20.0);
        skill_bonus(w, id, 1, SkillName::Stealing, 20.0);
    };
    label_number: 1061598,
    artifact_rarity: 11,
    base_physical_resistance: 17,
    base_poison_resistance: 18,
    base_energy_resistance: 18,
    init_min_hits: 255,
    init_max_hits: 255,
}

armor_leaf! {
    /// `Server.Items.TunicOfFire`.
    TUNIC_OF_FIRE, TUNIC_OF_FIRE_CLASS, tunic_of_fire_new = "Server.Items.TunicOfFire" : CHAIN_CHEST / CHAIN_CHEST_CLASS, chain_chest_new, encoded false;
    @init |w, id| {
        hue(w, id, 0x54F);
        set_attr(w, id, AttrKind::Armor, AosArmorAttribute::SelfRepair as u32, 5);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::NightSight as u32, 1);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::ReflectPhysical as u32, 15);
    };
    label_number: 1061099,
    artifact_rarity: 11,
    base_physical_resistance: 24,
    base_fire_resistance: 34,
    init_min_hits: 255,
    init_max_hits: 255,
}

armor_leaf! {
    /// `Server.Items.VoiceOfTheFallenKing`.
    VOICE_OF_THE_FALLEN_KING, VOICE_OF_THE_FALLEN_KING_CLASS, voice_of_the_fallen_king_new = "Server.Items.VoiceOfTheFallenKing" : LEATHER_GORGET / LEATHER_GORGET_CLASS, leather_gorget_new, encoded false;
    @init |w, id| {
        hue(w, id, 0x76D);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::BonusStr as u32, 8);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::RegenHits as u32, 5);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::RegenStam as u32, 3);
    };
    label_number: 1061094,
    artifact_rarity: 11,
    base_cold_resistance: 18,
    base_energy_resistance: 18,
    init_min_hits: 255,
    init_max_hits: 255,
}

armor_leaf! {
    /// `Server.Items.BoneArms`.
    BONE_ARMS, BONE_ARMS_CLASS, bone_arms_new = "Server.Items.BoneArms" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x144E), encoded false;
    flippable: Some((0x144e, 0x1453)),
    default_weight: Some(2.0),
    base_physical_resistance: 3,
    base_fire_resistance: 3,
    base_cold_resistance: 4,
    base_poison_resistance: 2,
    base_energy_resistance: 4,
    init_min_hits: 25,
    init_max_hits: 30,
    aos_str_req: 55,
    old_str_req: 40,
    old_dex_bonus: -2,
    armor_base: 30,
    revert_armor_base: Some(4),
    material_type: ArmorMaterialType::Bone,
    default_resource: CraftResource::RegularLeather,
}

armor_leaf! {
    /// `Server.Items.BoneChest`.
    BONE_CHEST, BONE_CHEST_CLASS, bone_chest_new = "Server.Items.BoneChest" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x144F), encoded false;
    flippable: Some((0x144f, 0x1454)),
    default_weight: Some(6.0),
    base_physical_resistance: 3,
    base_fire_resistance: 3,
    base_cold_resistance: 4,
    base_poison_resistance: 2,
    base_energy_resistance: 4,
    init_min_hits: 25,
    init_max_hits: 30,
    aos_str_req: 60,
    old_str_req: 40,
    old_dex_bonus: -6,
    armor_base: 30,
    revert_armor_base: Some(11),
    material_type: ArmorMaterialType::Bone,
    default_resource: CraftResource::RegularLeather,
}

armor_leaf! {
    /// `Server.Items.BoneGloves`.
    BONE_GLOVES, BONE_GLOVES_CLASS, bone_gloves_new = "Server.Items.BoneGloves" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x1450), encoded false;
    flippable: Some((0x1450, 0x1455)),
    default_weight: Some(2.0),
    base_physical_resistance: 3,
    base_fire_resistance: 3,
    base_cold_resistance: 4,
    base_poison_resistance: 2,
    base_energy_resistance: 4,
    init_min_hits: 25,
    init_max_hits: 30,
    aos_str_req: 55,
    old_str_req: 40,
    old_dex_bonus: -1,
    armor_base: 30,
    revert_armor_base: Some(2),
    material_type: ArmorMaterialType::Bone,
    default_resource: CraftResource::RegularLeather,
}

armor_leaf! {
    /// `Server.Items.BoneLegs`.
    BONE_LEGS, BONE_LEGS_CLASS, bone_legs_new = "Server.Items.BoneLegs" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x1452), encoded false;
    flippable: Some((0x1452, 0x1457)),
    default_weight: Some(3.0),
    base_physical_resistance: 3,
    base_fire_resistance: 3,
    base_cold_resistance: 4,
    base_poison_resistance: 2,
    base_energy_resistance: 4,
    init_min_hits: 25,
    init_max_hits: 30,
    aos_str_req: 55,
    old_str_req: 40,
    old_dex_bonus: -4,
    armor_base: 30,
    revert_armor_base: Some(7),
    material_type: ArmorMaterialType::Bone,
    default_resource: CraftResource::RegularLeather,
}

armor_leaf! {
    /// `Server.Items.ChainHatsuburi`.
    CHAIN_HATSUBURI, CHAIN_HATSUBURI_CLASS, chain_hatsuburi_new = "Server.Items.ChainHatsuburi" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x2774), encoded false;
    default_weight: Some(7.0),
    base_physical_resistance: 5,
    base_fire_resistance: 2,
    base_cold_resistance: 2,
    base_poison_resistance: 2,
    base_energy_resistance: 4,
    init_min_hits: 55,
    init_max_hits: 75,
    aos_str_req: 50,
    old_str_req: 50,
    armor_base: 3,
    material_type: ArmorMaterialType::Chainmail,
}

armor_leaf! {
    /// `Server.Items.ChainLegs`.
    CHAIN_LEGS, CHAIN_LEGS_CLASS, chain_legs_new = "Server.Items.ChainLegs" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x13BE), encoded false;
    flippable: Some((0x13be, 0x13c3)),
    default_weight: Some(7.0),
    base_physical_resistance: 4,
    base_fire_resistance: 4,
    base_cold_resistance: 4,
    base_poison_resistance: 1,
    base_energy_resistance: 2,
    init_min_hits: 45,
    init_max_hits: 60,
    aos_str_req: 60,
    old_str_req: 20,
    old_dex_bonus: -3,
    armor_base: 28,
    material_type: ArmorMaterialType::Chainmail,
}

armor_leaf! {
    /// `Server.Items.GargishClothArmsType1`.
    GARGISH_CLOTH_ARMS_TYPE1, GARGISH_CLOTH_ARMS_TYPE1_CLASS, gargish_cloth_arms_type1_new = "Server.Items.GargishClothArmsType1" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x404), encoded true;
    default_weight: Some(2.0),
    base_physical_resistance: 5,
    base_fire_resistance: 7,
    base_cold_resistance: 6,
    base_poison_resistance: 6,
    base_energy_resistance: 6,
    init_min_hits: 40,
    init_max_hits: 50,
    aos_str_req: 20,
    material_type: ArmorMaterialType::Cloth,
    default_resource: CraftResource::None,
    def_med_allowance: ArmorMeditationAllowance::All,
    required_races: Some(ALLOW_GARGOYLES_ONLY),
}

armor_leaf! {
    /// `Server.Items.GargishClothArmsType2`.
    GARGISH_CLOTH_ARMS_TYPE2, GARGISH_CLOTH_ARMS_TYPE2_CLASS, gargish_cloth_arms_type2_new = "Server.Items.GargishClothArmsType2" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x403), encoded true;
    default_weight: Some(2.0),
    base_physical_resistance: 5,
    base_fire_resistance: 7,
    base_cold_resistance: 6,
    base_poison_resistance: 6,
    base_energy_resistance: 6,
    init_min_hits: 40,
    init_max_hits: 50,
    aos_str_req: 20,
    material_type: ArmorMaterialType::Cloth,
    default_resource: CraftResource::None,
    def_med_allowance: ArmorMeditationAllowance::All,
    required_races: Some(ALLOW_GARGOYLES_ONLY),
}

armor_leaf! {
    /// `Server.Items.GargishClothChestType1`.
    GARGISH_CLOTH_CHEST_TYPE1, GARGISH_CLOTH_CHEST_TYPE1_CLASS, gargish_cloth_chest_type1_new = "Server.Items.GargishClothChestType1" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x406), encoded true;
    default_weight: Some(2.0),
    base_physical_resistance: 5,
    base_fire_resistance: 7,
    base_cold_resistance: 6,
    base_poison_resistance: 6,
    base_energy_resistance: 6,
    init_min_hits: 40,
    init_max_hits: 50,
    aos_str_req: 25,
    material_type: ArmorMaterialType::Cloth,
    default_resource: CraftResource::None,
    def_med_allowance: ArmorMeditationAllowance::All,
    required_races: Some(ALLOW_GARGOYLES_ONLY),
}

armor_leaf! {
    /// `Server.Items.GargishClothChestType2`.
    GARGISH_CLOTH_CHEST_TYPE2, GARGISH_CLOTH_CHEST_TYPE2_CLASS, gargish_cloth_chest_type2_new = "Server.Items.GargishClothChestType2" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x405), encoded true;
    default_weight: Some(2.0),
    base_physical_resistance: 5,
    base_fire_resistance: 7,
    base_cold_resistance: 6,
    base_poison_resistance: 6,
    base_energy_resistance: 6,
    init_min_hits: 40,
    init_max_hits: 50,
    aos_str_req: 25,
    material_type: ArmorMaterialType::Cloth,
    default_resource: CraftResource::None,
    def_med_allowance: ArmorMeditationAllowance::All,
    required_races: Some(ALLOW_GARGOYLES_ONLY),
}

armor_leaf! {
    /// `Server.Items.GargishClothKiltType1`.
    GARGISH_CLOTH_KILT_TYPE1, GARGISH_CLOTH_KILT_TYPE1_CLASS, gargish_cloth_kilt_type1_new = "Server.Items.GargishClothKiltType1" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x408), encoded true;
    default_weight: Some(2.0),
    base_physical_resistance: 5,
    base_fire_resistance: 7,
    base_cold_resistance: 6,
    base_poison_resistance: 6,
    base_energy_resistance: 6,
    init_min_hits: 40,
    init_max_hits: 50,
    aos_str_req: 20,
    material_type: ArmorMaterialType::Cloth,
    default_resource: CraftResource::None,
    def_med_allowance: ArmorMeditationAllowance::All,
    required_races: Some(ALLOW_GARGOYLES_ONLY),
}

armor_leaf! {
    /// `Server.Items.GargishClothKiltType2`.
    GARGISH_CLOTH_KILT_TYPE2, GARGISH_CLOTH_KILT_TYPE2_CLASS, gargish_cloth_kilt_type2_new = "Server.Items.GargishClothKiltType2" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x407), encoded true;
    default_weight: Some(2.0),
    base_physical_resistance: 5,
    base_fire_resistance: 7,
    base_cold_resistance: 6,
    base_poison_resistance: 6,
    base_energy_resistance: 6,
    init_min_hits: 40,
    init_max_hits: 50,
    aos_str_req: 20,
    material_type: ArmorMaterialType::Cloth,
    default_resource: CraftResource::None,
    def_med_allowance: ArmorMeditationAllowance::All,
    required_races: Some(ALLOW_GARGOYLES_ONLY),
}

armor_leaf! {
    /// `Server.Items.GargishClothLegsType1`.
    GARGISH_CLOTH_LEGS_TYPE1, GARGISH_CLOTH_LEGS_TYPE1_CLASS, gargish_cloth_legs_type1_new = "Server.Items.GargishClothLegsType1" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x40A), encoded true;
    default_weight: Some(2.0),
    base_physical_resistance: 5,
    base_fire_resistance: 7,
    base_cold_resistance: 6,
    base_poison_resistance: 6,
    base_energy_resistance: 6,
    init_min_hits: 40,
    init_max_hits: 50,
    aos_str_req: 20,
    material_type: ArmorMaterialType::Cloth,
    default_resource: CraftResource::None,
    def_med_allowance: ArmorMeditationAllowance::All,
    required_races: Some(ALLOW_GARGOYLES_ONLY),
}

armor_leaf! {
    /// `Server.Items.GargishClothLegsType2`.
    GARGISH_CLOTH_LEGS_TYPE2, GARGISH_CLOTH_LEGS_TYPE2_CLASS, gargish_cloth_legs_type2_new = "Server.Items.GargishClothLegsType2" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x409), encoded true;
    default_weight: Some(2.0),
    base_physical_resistance: 5,
    base_fire_resistance: 7,
    base_cold_resistance: 6,
    base_poison_resistance: 6,
    base_energy_resistance: 6,
    init_min_hits: 40,
    init_max_hits: 50,
    aos_str_req: 20,
    material_type: ArmorMaterialType::Cloth,
    default_resource: CraftResource::None,
    def_med_allowance: ArmorMeditationAllowance::All,
    required_races: Some(ALLOW_GARGOYLES_ONLY),
}

armor_leaf! {
    /// `Server.Items.DaemonArms`.
    DAEMON_ARMS, DAEMON_ARMS_CLASS, daemon_arms_new = "Server.Items.DaemonArms" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x144E), encoded false;
    @init |w, id| {
        hue(w, id, 0x648);
        set_attr(w, id, AttrKind::Armor, AosArmorAttribute::SelfRepair as u32, 1);
    };
    flippable: Some((0x144e, 0x1453)),
    default_weight: Some(2.0),
    base_physical_resistance: 6,
    base_fire_resistance: 6,
    base_cold_resistance: 7,
    base_poison_resistance: 5,
    base_energy_resistance: 7,
    init_min_hits: 255,
    init_max_hits: 255,
    aos_str_req: 55,
    old_str_req: 40,
    old_dex_bonus: -2,
    armor_base: 46,
    material_type: ArmorMaterialType::Bone,
    default_resource: CraftResource::RegularLeather,
    label_number: 1041371,
}

armor_leaf! {
    /// `Server.Items.DaemonChest`.
    DAEMON_CHEST, DAEMON_CHEST_CLASS, daemon_chest_new = "Server.Items.DaemonChest" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x144F), encoded false;
    @init |w, id| {
        hue(w, id, 0x648);
        set_attr(w, id, AttrKind::Armor, AosArmorAttribute::SelfRepair as u32, 1);
    };
    flippable: Some((0x144f, 0x1454)),
    default_weight: Some(6.0),
    base_physical_resistance: 6,
    base_fire_resistance: 6,
    base_cold_resistance: 7,
    base_poison_resistance: 5,
    base_energy_resistance: 7,
    init_min_hits: 255,
    init_max_hits: 255,
    aos_str_req: 60,
    old_str_req: 40,
    old_dex_bonus: -6,
    armor_base: 46,
    material_type: ArmorMaterialType::Bone,
    default_resource: CraftResource::RegularLeather,
    label_number: 1041372,
}

armor_leaf! {
    /// `Server.Items.DaemonGloves`.
    DAEMON_GLOVES, DAEMON_GLOVES_CLASS, daemon_gloves_new = "Server.Items.DaemonGloves" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x1450), encoded false;
    @init |w, id| {
        hue(w, id, 0x648);
        set_attr(w, id, AttrKind::Armor, AosArmorAttribute::SelfRepair as u32, 1);
    };
    flippable: Some((0x1450, 0x1455)),
    default_weight: Some(2.0),
    base_physical_resistance: 6,
    base_fire_resistance: 6,
    base_cold_resistance: 7,
    base_poison_resistance: 5,
    base_energy_resistance: 7,
    init_min_hits: 255,
    init_max_hits: 255,
    aos_str_req: 55,
    old_str_req: 40,
    old_dex_bonus: -1,
    armor_base: 46,
    material_type: ArmorMaterialType::Bone,
    default_resource: CraftResource::RegularLeather,
    label_number: 1041373,
}

armor_leaf! {
    /// `Server.Items.DaemonLegs`.
    DAEMON_LEGS, DAEMON_LEGS_CLASS, daemon_legs_new = "Server.Items.DaemonLegs" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x1452), encoded false;
    @init |w, id| {
        hue(w, id, 0x648);
        set_attr(w, id, AttrKind::Armor, AosArmorAttribute::SelfRepair as u32, 1);
    };
    flippable: Some((0x1452, 0x1457)),
    default_weight: Some(3.0),
    base_physical_resistance: 6,
    base_fire_resistance: 6,
    base_cold_resistance: 7,
    base_poison_resistance: 5,
    base_energy_resistance: 7,
    init_min_hits: 255,
    init_max_hits: 255,
    aos_str_req: 55,
    old_str_req: 40,
    old_dex_bonus: -4,
    armor_base: 46,
    material_type: ArmorMaterialType::Bone,
    default_resource: CraftResource::RegularLeather,
    label_number: 1041375,
}

armor_leaf! {
    /// `Server.Items.DragonArms`.
    DRAGON_ARMS, DRAGON_ARMS_CLASS, dragon_arms_new = "Server.Items.DragonArms" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x2657), encoded false;
    flippable: Some((0x2657, 0x2658)),
    default_weight: Some(5.0),
    base_physical_resistance: 3,
    base_fire_resistance: 3,
    base_cold_resistance: 3,
    base_poison_resistance: 3,
    base_energy_resistance: 3,
    init_min_hits: 55,
    init_max_hits: 75,
    aos_str_req: 75,
    old_str_req: 20,
    old_dex_bonus: -2,
    armor_base: 40,
    material_type: ArmorMaterialType::Dragon,
    default_resource: CraftResource::RedScales,
}

armor_leaf! {
    /// `Server.Items.DragonChest`.
    DRAGON_CHEST, DRAGON_CHEST_CLASS, dragon_chest_new = "Server.Items.DragonChest" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x2641), encoded false;
    flippable: Some((0x2641, 0x2642)),
    default_weight: Some(10.0),
    base_physical_resistance: 3,
    base_fire_resistance: 3,
    base_cold_resistance: 3,
    base_poison_resistance: 3,
    base_energy_resistance: 3,
    init_min_hits: 55,
    init_max_hits: 75,
    aos_str_req: 75,
    old_str_req: 60,
    old_dex_bonus: -8,
    armor_base: 40,
    material_type: ArmorMaterialType::Dragon,
    default_resource: CraftResource::RedScales,
}

armor_leaf! {
    /// `Server.Items.DragonGloves`.
    DRAGON_GLOVES, DRAGON_GLOVES_CLASS, dragon_gloves_new = "Server.Items.DragonGloves" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x2643), encoded false;
    flippable: Some((0x2643, 0x2644)),
    default_weight: Some(2.0),
    base_physical_resistance: 3,
    base_fire_resistance: 3,
    base_cold_resistance: 3,
    base_poison_resistance: 3,
    base_energy_resistance: 3,
    init_min_hits: 55,
    init_max_hits: 75,
    aos_str_req: 75,
    old_str_req: 30,
    old_dex_bonus: -2,
    armor_base: 40,
    material_type: ArmorMaterialType::Dragon,
    default_resource: CraftResource::RedScales,
}

armor_leaf! {
    /// `Server.Items.DragonHelm`.
    DRAGON_HELM, DRAGON_HELM_CLASS, dragon_helm_new = "Server.Items.DragonHelm" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x2645), encoded false;
    flippable: Some((0x2645, 0x2646)),
    default_weight: Some(5.0),
    base_physical_resistance: 3,
    base_fire_resistance: 3,
    base_cold_resistance: 3,
    base_poison_resistance: 3,
    base_energy_resistance: 3,
    init_min_hits: 55,
    init_max_hits: 75,
    aos_str_req: 75,
    old_str_req: 40,
    old_dex_bonus: -1,
    armor_base: 40,
    material_type: ArmorMaterialType::Dragon,
    default_resource: CraftResource::RedScales,
}

armor_leaf! {
    /// `Server.Items.DragonLegs`.
    DRAGON_LEGS, DRAGON_LEGS_CLASS, dragon_legs_new = "Server.Items.DragonLegs" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x2647), encoded false;
    flippable: Some((0x2647, 0x2648)),
    default_weight: Some(6.0),
    base_physical_resistance: 3,
    base_fire_resistance: 3,
    base_cold_resistance: 3,
    base_poison_resistance: 3,
    base_energy_resistance: 3,
    init_min_hits: 55,
    init_max_hits: 75,
    aos_str_req: 75,
    old_str_req: 60,
    old_dex_bonus: -6,
    armor_base: 40,
    material_type: ArmorMaterialType::Dragon,
    default_resource: CraftResource::RedScales,
}

armor_leaf! {
    /// `Server.Items.AnthropomorphistGlasses`.
    ANTHROPOMORPHIST_GLASSES, ANTHROPOMORPHIST_GLASSES_CLASS, anthropomorphist_glasses_new = "Server.Items.AnthropomorphistGlasses" : ELVEN_GLASSES / ELVEN_GLASSES_CLASS, elven_glasses_new, encoded false;
    @init |w, id| {
        set_attr(w, id, AttrKind::Attributes, AosAttribute::BonusHits as u32, 5);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::RegenMana as u32, 3);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::ReflectPhysical as u32, 20);
        hue(w, id, 0x80);
    };
    label_number: 1073379,
    base_physical_resistance: 5,
    base_fire_resistance: 5,
    base_cold_resistance: 10,
    base_poison_resistance: 20,
    base_energy_resistance: 20,
    init_min_hits: 255,
    init_max_hits: 255,
}

armor_leaf! {
    /// `Server.Items.ArtsGlasses`.
    ARTS_GLASSES, ARTS_GLASSES_CLASS, arts_glasses_new = "Server.Items.ArtsGlasses" : ELVEN_GLASSES / ELVEN_GLASSES_CLASS, elven_glasses_new, encoded false;
    @init |w, id| {
        set_attr(w, id, AttrKind::Attributes, AosAttribute::BonusStr as u32, 5);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::BonusInt as u32, 5);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::BonusHits as u32, 15);
        hue(w, id, 0x73);
    };
    label_number: 1073363,
    base_physical_resistance: 10,
    base_fire_resistance: 8,
    base_cold_resistance: 8,
    base_poison_resistance: 4,
    base_energy_resistance: 10,
    init_min_hits: 255,
    init_max_hits: 255,
}

armor_leaf! {
    /// `Server.Items.FoldedSteelGlasses`.
    FOLDED_STEEL_GLASSES, FOLDED_STEEL_GLASSES_CLASS, folded_steel_glasses_new = "Server.Items.FoldedSteelGlasses" : ELVEN_GLASSES / ELVEN_GLASSES_CLASS, elven_glasses_new, encoded false;
    @init |w, id| {
        set_attr(w, id, AttrKind::Attributes, AosAttribute::BonusStr as u32, 8);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::NightSight as u32, 1);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::DefendChance as u32, 15);
        hue(w, id, 0x47E);
    };
    label_number: 1073380,
    base_physical_resistance: 20,
    base_fire_resistance: 10,
    base_cold_resistance: 10,
    base_poison_resistance: 10,
    base_energy_resistance: 10,
    init_min_hits: 255,
    init_max_hits: 255,
}

armor_leaf! {
    /// `Server.Items.LightOfWayGlasses`.
    LIGHT_OF_WAY_GLASSES, LIGHT_OF_WAY_GLASSES_CLASS, light_of_way_glasses_new = "Server.Items.LightOfWayGlasses" : ELVEN_GLASSES / ELVEN_GLASSES_CLASS, elven_glasses_new, encoded false;
    @init |w, id| {
        set_attr(w, id, AttrKind::Attributes, AosAttribute::BonusStr as u32, 7);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::BonusInt as u32, 5);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponDamage as u32, 30);
        hue(w, id, 0x256);
    };
    label_number: 1073378,
    base_physical_resistance: 10,
    base_fire_resistance: 10,
    base_cold_resistance: 10,
    base_poison_resistance: 10,
    base_energy_resistance: 10,
    init_min_hits: 255,
    init_max_hits: 255,
}

armor_leaf! {
    /// `Server.Items.LyricalGlasses`.
    LYRICAL_GLASSES, LYRICAL_GLASSES_CLASS, lyrical_glasses_new = "Server.Items.LyricalGlasses" : ELVEN_GLASSES / ELVEN_GLASSES_CLASS, elven_glasses_new, encoded false;
    @init |w, id| {
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::HitLowerDefend as u32, 20);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::NightSight as u32, 1);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::ReflectPhysical as u32, 15);
        hue(w, id, 0x47F);
    };
    label_number: 1073382,
    base_physical_resistance: 10,
    base_fire_resistance: 10,
    base_cold_resistance: 10,
    base_poison_resistance: 10,
    base_energy_resistance: 10,
    init_min_hits: 255,
    init_max_hits: 255,
}

armor_leaf! {
    /// `Server.Items.MaceShieldGlasses`.
    MACE_SHIELD_GLASSES, MACE_SHIELD_GLASSES_CLASS, mace_shield_glasses_new = "Server.Items.MaceShieldGlasses" : ELVEN_GLASSES / ELVEN_GLASSES_CLASS, elven_glasses_new, encoded false;
    @init |w, id| {
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::HitLowerDefend as u32, 30);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::BonusStr as u32, 10);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::BonusDex as u32, 5);
        hue(w, id, 0x1DD);
    };
    label_number: 1073381,
    base_physical_resistance: 25,
    base_fire_resistance: 10,
    base_cold_resistance: 10,
    base_poison_resistance: 10,
    base_energy_resistance: 10,
    init_min_hits: 255,
    init_max_hits: 255,
}

armor_leaf! {
    /// `Server.Items.MaritimeGlasses`.
    MARITIME_GLASSES, MARITIME_GLASSES_CLASS, maritime_glasses_new = "Server.Items.MaritimeGlasses" : ELVEN_GLASSES / ELVEN_GLASSES_CLASS, elven_glasses_new, encoded false;
    @init |w, id| {
        set_attr(w, id, AttrKind::Attributes, AosAttribute::Luck as u32, 150);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::NightSight as u32, 1);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::ReflectPhysical as u32, 20);
        hue(w, id, 0x581);
    };
    label_number: 1073364,
    base_physical_resistance: 3,
    base_fire_resistance: 4,
    base_cold_resistance: 30,
    base_poison_resistance: 5,
    base_energy_resistance: 3,
    init_min_hits: 255,
    init_max_hits: 255,
}

armor_leaf! {
    /// `Server.Items.NecromanticGlasses`.
    NECROMANTIC_GLASSES, NECROMANTIC_GLASSES_CLASS, necromantic_glasses_new = "Server.Items.NecromanticGlasses" : ELVEN_GLASSES / ELVEN_GLASSES_CLASS, elven_glasses_new, encoded false;
    @init |w, id| {
        set_attr(w, id, AttrKind::Attributes, AosAttribute::LowerManaCost as u32, 15);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::LowerRegCost as u32, 30);
        hue(w, id, 0x22D);
    };
    label_number: 1073377,
    base_physical_resistance: 0,
    base_fire_resistance: 0,
    base_cold_resistance: 0,
    base_poison_resistance: 0,
    base_energy_resistance: 0,
    init_min_hits: 255,
    init_max_hits: 255,
}

armor_leaf! {
    /// `Server.Items.PoisonedGlasses`.
    POISONED_GLASSES, POISONED_GLASSES_CLASS, poisoned_glasses_new = "Server.Items.PoisonedGlasses" : ELVEN_GLASSES / ELVEN_GLASSES_CLASS, elven_glasses_new, encoded false;
    @init |w, id| {
        set_attr(w, id, AttrKind::Attributes, AosAttribute::BonusStam as u32, 3);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::RegenStam as u32, 4);
        hue(w, id, 0x113);
    };
    label_number: 1073376,
    base_physical_resistance: 10,
    base_fire_resistance: 10,
    base_cold_resistance: 10,
    base_poison_resistance: 30,
    base_energy_resistance: 10,
    init_min_hits: 255,
    init_max_hits: 255,
}

armor_leaf! {
    /// `Server.Items.TradeGlasses`.
    TRADE_GLASSES, TRADE_GLASSES_CLASS, trade_glasses_new = "Server.Items.TradeGlasses" : ELVEN_GLASSES / ELVEN_GLASSES_CLASS, elven_glasses_new, encoded false;
    @init |w, id| {
        set_attr(w, id, AttrKind::Attributes, AosAttribute::BonusStr as u32, 10);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::BonusInt as u32, 10);
    };
    label_number: 1073362,
    base_physical_resistance: 10,
    base_fire_resistance: 10,
    base_cold_resistance: 10,
    base_poison_resistance: 10,
    base_energy_resistance: 10,
    init_min_hits: 255,
    init_max_hits: 255,
}

armor_leaf! {
    /// `Server.Items.TreasureTrinketGlasses`.
    TREASURE_TRINKET_GLASSES, TREASURE_TRINKET_GLASSES_CLASS, treasure_trinket_glasses_new = "Server.Items.TreasureTrinketGlasses" : ELVEN_GLASSES / ELVEN_GLASSES_CLASS, elven_glasses_new, encoded false;
    @init |w, id| {
        set_attr(w, id, AttrKind::Attributes, AosAttribute::BonusInt as u32, 10);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::BonusHits as u32, 5);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::SpellDamage as u32, 10);
        hue(w, id, 0x1C2);
    };
    label_number: 1073373,
    base_physical_resistance: 10,
    base_fire_resistance: 10,
    base_cold_resistance: 10,
    base_poison_resistance: 10,
    base_energy_resistance: 10,
    init_min_hits: 255,
    init_max_hits: 255,
}

armor_leaf! {
    /// `Server.Items.WizardsGlasses`.
    WIZARDS_GLASSES, WIZARDS_GLASSES_CLASS, wizards_glasses_new = "Server.Items.WizardsGlasses" : ELVEN_GLASSES / ELVEN_GLASSES_CLASS, elven_glasses_new, encoded false;
    @init |w, id| {
        set_attr(w, id, AttrKind::Attributes, AosAttribute::BonusMana as u32, 10);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::RegenMana as u32, 3);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::SpellDamage as u32, 15);
        hue(w, id, 0x2B0);
    };
    label_number: 1073374,
    base_physical_resistance: 5,
    base_fire_resistance: 5,
    base_cold_resistance: 5,
    base_poison_resistance: 5,
    base_energy_resistance: 5,
    init_min_hits: 255,
    init_max_hits: 255,
}

armor_leaf! {
    /// `Server.Items.Bascinet`.
    BASCINET, BASCINET_CLASS, bascinet_new = "Server.Items.Bascinet" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x140C), encoded false;
    default_weight: Some(5.0),
    base_physical_resistance: 7,
    base_fire_resistance: 2,
    base_cold_resistance: 2,
    base_poison_resistance: 2,
    base_energy_resistance: 2,
    init_min_hits: 40,
    init_max_hits: 50,
    aos_str_req: 40,
    old_str_req: 10,
    armor_base: 18,
    material_type: ArmorMaterialType::Plate,
}

armor_leaf! {
    /// `Server.Items.BoneHelm`.
    BONE_HELM, BONE_HELM_CLASS, bone_helm_new = "Server.Items.BoneHelm" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x1451), encoded false;
    flippable: Some((0x1451, 0x1456)),
    default_weight: Some(3.0),
    base_physical_resistance: 3,
    base_fire_resistance: 3,
    base_cold_resistance: 4,
    base_poison_resistance: 2,
    base_energy_resistance: 4,
    init_min_hits: 25,
    init_max_hits: 30,
    aos_str_req: 20,
    old_str_req: 40,
    armor_base: 30,
    material_type: ArmorMaterialType::Bone,
}

armor_leaf! {
    /// `Server.Items.ChainCoif`.
    CHAIN_COIF, CHAIN_COIF_CLASS, chain_coif_new = "Server.Items.ChainCoif" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x13BB), encoded false;
    flippable: Some((0x13BB, 0x13C0)),
    default_weight: Some(1.0),
    base_physical_resistance: 4,
    base_fire_resistance: 4,
    base_cold_resistance: 4,
    base_poison_resistance: 1,
    base_energy_resistance: 2,
    init_min_hits: 35,
    init_max_hits: 60,
    aos_str_req: 60,
    old_str_req: 20,
    armor_base: 28,
    material_type: ArmorMaterialType::Chainmail,
}

armor_leaf! {
    /// `Server.Items.Circlet`.
    CIRCLET, CIRCLET_CLASS, circlet_new = "Server.Items.Circlet" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x2B6E), encoded true;
    flippable: Some((0x2B6E, 0x3165)),
    default_weight: Some(2.0),
    base_physical_resistance: 1,
    base_fire_resistance: 5,
    base_cold_resistance: 2,
    base_poison_resistance: 2,
    base_energy_resistance: 5,
    init_min_hits: 50,
    init_max_hits: 65,
    aos_str_req: 10,
    old_str_req: 10,
    armor_base: 30,
    material_type: ArmorMaterialType::Plate,
    def_med_allowance: ArmorMeditationAllowance::All,
    required_races: Some(ALLOW_ELVES_ONLY),
}

armor_leaf! {
    /// `Server.Items.CloseHelm`.
    CLOSE_HELM, CLOSE_HELM_CLASS, close_helm_new = "Server.Items.CloseHelm" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x1408), encoded false;
    default_weight: Some(5.0),
    base_physical_resistance: 3,
    base_fire_resistance: 3,
    base_cold_resistance: 3,
    base_poison_resistance: 3,
    base_energy_resistance: 3,
    init_min_hits: 45,
    init_max_hits: 60,
    aos_str_req: 55,
    old_str_req: 40,
    armor_base: 30,
    material_type: ArmorMaterialType::Plate,
}

armor_leaf! {
    /// `Server.Items.DaemonHelm`.
    DAEMON_HELM, DAEMON_HELM_CLASS, daemon_helm_new = "Server.Items.DaemonHelm" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x1451), encoded false;
    @init |w, id| {
        hue(w, id, 0x648);
        set_attr(w, id, AttrKind::Armor, AosArmorAttribute::SelfRepair as u32, 1);
    };
    flippable: Some((0x1451, 0x1456)),
    default_weight: Some(3.0),
    base_physical_resistance: 6,
    base_fire_resistance: 6,
    base_cold_resistance: 7,
    base_poison_resistance: 5,
    base_energy_resistance: 7,
    init_min_hits: 255,
    init_max_hits: 255,
    aos_str_req: 20,
    old_str_req: 40,
    armor_base: 46,
    material_type: ArmorMaterialType::Bone,
    default_resource: CraftResource::RegularLeather,
    label_number: 1041374,
}

armor_leaf! {
    /// `Server.Items.GemmedCirclet`.
    GEMMED_CIRCLET, GEMMED_CIRCLET_CLASS, gemmed_circlet_new = "Server.Items.GemmedCirclet" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x2B70), encoded true;
    flippable: Some((0x2B70, 0x3167)),
    default_weight: Some(2.0),
    required_races: Some(ALLOW_ELVES_ONLY),
    base_physical_resistance: 1,
    base_fire_resistance: 5,
    base_cold_resistance: 2,
    base_poison_resistance: 2,
    base_energy_resistance: 5,
    init_min_hits: 20,
    init_max_hits: 35,
    aos_str_req: 10,
    old_str_req: 10,
    armor_base: 30,
    material_type: ArmorMaterialType::Plate,
    def_med_allowance: ArmorMeditationAllowance::All,
}

armor_leaf! {
    /// `Server.Items.Helmet`.
    HELMET, HELMET_CLASS, helmet_new = "Server.Items.Helmet" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x140A), encoded false;
    default_weight: Some(5.0),
    base_physical_resistance: 2,
    base_fire_resistance: 4,
    base_cold_resistance: 4,
    base_poison_resistance: 3,
    base_energy_resistance: 2,
    init_min_hits: 45,
    init_max_hits: 60,
    aos_str_req: 45,
    old_str_req: 40,
    armor_base: 30,
    material_type: ArmorMaterialType::Plate,
}

armor_leaf! {
    /// `Server.Items.LeatherCap`.
    LEATHER_CAP, LEATHER_CAP_CLASS, leather_cap_new = "Server.Items.LeatherCap" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x1DB9), encoded false;
    flippable: Some((0x1db9, 0x1dba)),
    default_weight: Some(2.0),
    base_physical_resistance: 2,
    base_fire_resistance: 4,
    base_cold_resistance: 3,
    base_poison_resistance: 3,
    base_energy_resistance: 3,
    init_min_hits: 30,
    init_max_hits: 40,
    aos_str_req: 20,
    old_str_req: 15,
    armor_base: 13,
    material_type: ArmorMaterialType::Leather,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::All,
}

armor_leaf! {
    /// `Server.Items.NorseHelm`.
    NORSE_HELM, NORSE_HELM_CLASS, norse_helm_new = "Server.Items.NorseHelm" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x140E), encoded false;
    default_weight: Some(5.0),
    base_physical_resistance: 4,
    base_fire_resistance: 1,
    base_cold_resistance: 4,
    base_poison_resistance: 4,
    base_energy_resistance: 2,
    init_min_hits: 45,
    init_max_hits: 60,
    aos_str_req: 55,
    old_str_req: 40,
    armor_base: 30,
    material_type: ArmorMaterialType::Plate,
}

armor_leaf! {
    /// `Server.Items.OrcHelm`.
    ORC_HELM, ORC_HELM_CLASS, orc_helm_new = "Server.Items.OrcHelm" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x1F0B), encoded false;
    base_physical_resistance: 3,
    base_fire_resistance: 1,
    base_cold_resistance: 3,
    base_poison_resistance: 3,
    base_energy_resistance: 5,
    init_min_hits: 30,
    init_max_hits: 50,
    aos_str_req: 30,
    old_str_req: 10,
    armor_base: 20,
    default_weight: Some(5.0),
    material_type: ArmorMaterialType::Leather,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::None,
}

armor_leaf! {
    /// `Server.Items.PlateHelm`.
    PLATE_HELM, PLATE_HELM_CLASS, plate_helm_new = "Server.Items.PlateHelm" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x1412), encoded false;
    default_weight: Some(5.0),
    base_physical_resistance: 5,
    base_fire_resistance: 3,
    base_cold_resistance: 2,
    base_poison_resistance: 3,
    base_energy_resistance: 2,
    init_min_hits: 50,
    init_max_hits: 65,
    aos_str_req: 80,
    old_str_req: 40,
    old_dex_bonus: -1,
    armor_base: 40,
    material_type: ArmorMaterialType::Plate,
}

armor_leaf! {
    /// `Server.Items.RavenHelm`.
    RAVEN_HELM, RAVEN_HELM_CLASS, raven_helm_new = "Server.Items.RavenHelm" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x2B71), encoded true;
    flippable: Some((0x2B71, 0x3168)),
    default_weight: Some(5.0),
    required_races: Some(ALLOW_ELVES_ONLY),
    base_physical_resistance: 5,
    base_fire_resistance: 1,
    base_cold_resistance: 2,
    base_poison_resistance: 2,
    base_energy_resistance: 5,
    init_min_hits: 50,
    init_max_hits: 65,
    aos_str_req: 25,
    old_str_req: 25,
    armor_base: 40,
    material_type: ArmorMaterialType::Plate,
}

armor_leaf! {
    /// `Server.Items.RoyalCirclet`.
    ROYAL_CIRCLET, ROYAL_CIRCLET_CLASS, royal_circlet_new = "Server.Items.RoyalCirclet" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x2B6F), encoded true;
    flippable: Some((0x2B6F, 0x3166)),
    default_weight: Some(2.0),
    required_races: Some(ALLOW_ELVES_ONLY),
    base_physical_resistance: 1,
    base_fire_resistance: 5,
    base_cold_resistance: 2,
    base_poison_resistance: 2,
    base_energy_resistance: 5,
    init_min_hits: 20,
    init_max_hits: 35,
    aos_str_req: 10,
    old_str_req: 10,
    armor_base: 30,
    material_type: ArmorMaterialType::Plate,
    def_med_allowance: ArmorMeditationAllowance::All,
}

armor_leaf! {
    /// `Server.Items.VultureHelm`.
    VULTURE_HELM, VULTURE_HELM_CLASS, vulture_helm_new = "Server.Items.VultureHelm" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x2B72), encoded true;
    flippable: Some((0x2B72, 0x3169)),
    default_weight: Some(5.0),
    required_races: Some(ALLOW_ELVES_ONLY),
    base_physical_resistance: 5,
    base_fire_resistance: 1,
    base_cold_resistance: 2,
    base_poison_resistance: 2,
    base_energy_resistance: 5,
    init_min_hits: 50,
    init_max_hits: 65,
    aos_str_req: 25,
    old_str_req: 25,
    armor_base: 40,
    material_type: ArmorMaterialType::Plate,
}

armor_leaf! {
    /// `Server.Items.WingedHelm`.
    WINGED_HELM, WINGED_HELM_CLASS, winged_helm_new = "Server.Items.WingedHelm" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x2B73), encoded true;
    flippable: Some((0x2B73, 0x316A)),
    default_weight: Some(5.0),
    required_races: Some(ALLOW_ELVES_ONLY),
    base_physical_resistance: 5,
    base_fire_resistance: 1,
    base_cold_resistance: 2,
    base_poison_resistance: 2,
    base_energy_resistance: 5,
    init_min_hits: 45,
    init_max_hits: 55,
    aos_str_req: 25,
    old_str_req: 25,
    armor_base: 40,
    material_type: ArmorMaterialType::Plate,
}

armor_leaf! {
    /// `Server.Items.FemaleLeafChest`.
    FEMALE_LEAF_CHEST, FEMALE_LEAF_CHEST_CLASS, female_leaf_chest_new = "Server.Items.FemaleLeafChest" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x2FCB), encoded true;
    flippable: Some((0x2FCB, 0x3181)),
    default_weight: Some(2.0),
    base_physical_resistance: 2,
    base_fire_resistance: 3,
    base_cold_resistance: 2,
    base_poison_resistance: 4,
    base_energy_resistance: 4,
    init_min_hits: 30,
    init_max_hits: 40,
    aos_str_req: 20,
    old_str_req: 20,
    armor_base: 13,
    material_type: ArmorMaterialType::Leather,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::All,
    allow_male_wearer: false,
}

armor_leaf! {
    /// `Server.Items.FemaleLeatherChest`.
    FEMALE_LEATHER_CHEST, FEMALE_LEATHER_CHEST_CLASS, female_leather_chest_new = "Server.Items.FemaleLeatherChest" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x1C06), encoded false;
    flippable: Some((0x1c06, 0x1c07)),
    default_weight: Some(1.0),
    base_physical_resistance: 2,
    base_fire_resistance: 4,
    base_cold_resistance: 3,
    base_poison_resistance: 3,
    base_energy_resistance: 3,
    init_min_hits: 30,
    init_max_hits: 40,
    aos_str_req: 25,
    old_str_req: 15,
    armor_base: 13,
    material_type: ArmorMaterialType::Leather,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::All,
    allow_male_wearer: false,
}

armor_leaf! {
    /// `Server.Items.GargishLeatherArmsType1`.
    GARGISH_LEATHER_ARMS_TYPE1, GARGISH_LEATHER_ARMS_TYPE1_CLASS, gargish_leather_arms_type1_new = "Server.Items.GargishLeatherArmsType1" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x301), encoded false;
    default_weight: Some(4.0),
    required_races: Some(ALLOW_GARGOYLES_ONLY),
    base_physical_resistance: 5,
    base_fire_resistance: 6,
    base_cold_resistance: 7,
    base_poison_resistance: 6,
    base_energy_resistance: 6,
    init_min_hits: 30,
    init_max_hits: 50,
    aos_str_req: 25,
    old_str_req: 25,
    armor_base: 13,
    material_type: ArmorMaterialType::Leather,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::All,
}

armor_leaf! {
    /// `Server.Items.GargishLeatherArmsType2`.
    GARGISH_LEATHER_ARMS_TYPE2, GARGISH_LEATHER_ARMS_TYPE2_CLASS, gargish_leather_arms_type2_new = "Server.Items.GargishLeatherArmsType2" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x302), encoded false;
    default_weight: Some(4.0),
    required_races: Some(ALLOW_GARGOYLES_ONLY),
    base_physical_resistance: 5,
    base_fire_resistance: 6,
    base_cold_resistance: 7,
    base_poison_resistance: 6,
    base_energy_resistance: 6,
    init_min_hits: 30,
    init_max_hits: 50,
    aos_str_req: 25,
    old_str_req: 25,
    armor_base: 13,
    material_type: ArmorMaterialType::Leather,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::All,
}

armor_leaf! {
    /// `Server.Items.GargishLeatherChestType1`.
    GARGISH_LEATHER_CHEST_TYPE1, GARGISH_LEATHER_CHEST_TYPE1_CLASS, gargish_leather_chest_type1_new = "Server.Items.GargishLeatherChestType1" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x0304), encoded false;
    default_weight: Some(4.0),
    required_races: Some(ALLOW_GARGOYLES_ONLY),
    base_physical_resistance: 5,
    base_fire_resistance: 6,
    base_cold_resistance: 7,
    base_poison_resistance: 6,
    base_energy_resistance: 6,
    init_min_hits: 30,
    init_max_hits: 50,
    aos_str_req: 25,
    old_str_req: 25,
    armor_base: 13,
    material_type: ArmorMaterialType::Leather,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::All,
}

armor_leaf! {
    /// `Server.Items.GargishLeatherChestType2`.
    GARGISH_LEATHER_CHEST_TYPE2, GARGISH_LEATHER_CHEST_TYPE2_CLASS, gargish_leather_chest_type2_new = "Server.Items.GargishLeatherChestType2" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x303), encoded false;
    default_weight: Some(4.0),
    required_races: Some(ALLOW_GARGOYLES_ONLY),
    base_physical_resistance: 5,
    base_fire_resistance: 6,
    base_cold_resistance: 7,
    base_poison_resistance: 6,
    base_energy_resistance: 6,
    init_min_hits: 30,
    init_max_hits: 50,
    aos_str_req: 25,
    old_str_req: 25,
    armor_base: 13,
    material_type: ArmorMaterialType::Leather,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::All,
}

armor_leaf! {
    /// `Server.Items.GargishLeatherKiltType1`.
    GARGISH_LEATHER_KILT_TYPE1, GARGISH_LEATHER_KILT_TYPE1_CLASS, gargish_leather_kilt_type1_new = "Server.Items.GargishLeatherKiltType1" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x311), encoded false;
    default_weight: Some(4.0),
    required_races: Some(ALLOW_GARGOYLES_ONLY),
    base_physical_resistance: 5,
    base_fire_resistance: 6,
    base_cold_resistance: 7,
    base_poison_resistance: 6,
    base_energy_resistance: 6,
    init_min_hits: 30,
    init_max_hits: 50,
    aos_str_req: 25,
    old_str_req: 25,
    armor_base: 13,
    material_type: ArmorMaterialType::Leather,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::All,
}

armor_leaf! {
    /// `Server.Items.GargishLeatherKiltType2`.
    GARGISH_LEATHER_KILT_TYPE2, GARGISH_LEATHER_KILT_TYPE2_CLASS, gargish_leather_kilt_type2_new = "Server.Items.GargishLeatherKiltType2" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x310), encoded false;
    default_weight: Some(4.0),
    required_races: Some(ALLOW_GARGOYLES_ONLY),
    base_physical_resistance: 5,
    base_fire_resistance: 6,
    base_cold_resistance: 7,
    base_poison_resistance: 6,
    base_energy_resistance: 6,
    init_min_hits: 30,
    init_max_hits: 50,
    aos_str_req: 25,
    old_str_req: 25,
    armor_base: 13,
    material_type: ArmorMaterialType::Leather,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::All,
}

armor_leaf! {
    /// `Server.Items.GargishLeatherLegsType1`.
    GARGISH_LEATHER_LEGS_TYPE1, GARGISH_LEATHER_LEGS_TYPE1_CLASS, gargish_leather_legs_type1_new = "Server.Items.GargishLeatherLegsType1" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x305), encoded false;
    default_weight: Some(4.0),
    required_races: Some(ALLOW_GARGOYLES_ONLY),
    base_physical_resistance: 5,
    base_fire_resistance: 6,
    base_cold_resistance: 7,
    base_poison_resistance: 6,
    base_energy_resistance: 6,
    init_min_hits: 30,
    init_max_hits: 50,
    aos_str_req: 25,
    old_str_req: 25,
    armor_base: 13,
    material_type: ArmorMaterialType::Leather,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::All,
}

armor_leaf! {
    /// `Server.Items.GargishLeatherLegsType2`.
    GARGISH_LEATHER_LEGS_TYPE2, GARGISH_LEATHER_LEGS_TYPE2_CLASS, gargish_leather_legs_type2_new = "Server.Items.GargishLeatherLegsType2" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x306), encoded false;
    default_weight: Some(4.0),
    required_races: Some(ALLOW_GARGOYLES_ONLY),
    base_physical_resistance: 5,
    base_fire_resistance: 6,
    base_cold_resistance: 7,
    base_poison_resistance: 6,
    base_energy_resistance: 6,
    init_min_hits: 30,
    init_max_hits: 50,
    aos_str_req: 25,
    old_str_req: 25,
    armor_base: 13,
    material_type: ArmorMaterialType::Leather,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::All,
}

armor_leaf! {
    /// `Server.Items.GargishLeatherWingArmor`.
    GARGISH_LEATHER_WING_ARMOR, GARGISH_LEATHER_WING_ARMOR_CLASS, gargish_leather_wing_armor_new = "Server.Items.GargishLeatherWingArmor" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x457E), encoded false;
    @ty {
        physical_resistance: |_, _| 0,
        fire_resistance: |_, _| 0,
        cold_resistance: |_, _| 0,
        poison_resistance: |_, _| 0,
        energy_resistance: |_, _| 0,
    };
    flippable: Some((0x457E, 0x457F)),
    default_weight: Some(2.0),
    required_races: Some(ALLOW_GARGOYLES_ONLY),
    aos_str_req: 10,
    old_str_req: 10,
    armor_base: 0,
    material_type: ArmorMaterialType::Leather,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::All,
}

armor_leaf! {
    /// `Server.Items.LeafArms`.
    LEAF_ARMS, LEAF_ARMS_CLASS, leaf_arms_new = "Server.Items.LeafArms" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x2FC8), encoded true;
    flippable: Some((0x2FC8, 0x317E)),
    default_weight: Some(2.0),
    required_races: Some(ALLOW_ELVES_ONLY),
    base_physical_resistance: 2,
    base_fire_resistance: 3,
    base_cold_resistance: 2,
    base_poison_resistance: 4,
    base_energy_resistance: 4,
    init_min_hits: 30,
    init_max_hits: 40,
    aos_str_req: 15,
    old_str_req: 15,
    armor_base: 13,
    material_type: ArmorMaterialType::Leather,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::All,
}

armor_leaf! {
    /// `Server.Items.LeafChest`.
    LEAF_CHEST, LEAF_CHEST_CLASS, leaf_chest_new = "Server.Items.LeafChest" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x2FC5), encoded true;
    flippable: Some((0x2FC5, 0x317B)),
    default_weight: Some(2.0),
    required_races: Some(ALLOW_ELVES_ONLY),
    base_physical_resistance: 2,
    base_fire_resistance: 3,
    base_cold_resistance: 2,
    base_poison_resistance: 4,
    base_energy_resistance: 4,
    init_min_hits: 30,
    init_max_hits: 40,
    aos_str_req: 20,
    old_str_req: 20,
    armor_base: 13,
    material_type: ArmorMaterialType::Leather,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::All,
}

armor_leaf! {
    /// `Server.Items.LeafGorget`.
    LEAF_GORGET, LEAF_GORGET_CLASS, leaf_gorget_new = "Server.Items.LeafGorget" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x2FC7), encoded true;
    default_weight: Some(2.0),
    required_races: Some(ALLOW_ELVES_ONLY),
    base_physical_resistance: 2,
    base_fire_resistance: 3,
    base_cold_resistance: 2,
    base_poison_resistance: 4,
    base_energy_resistance: 4,
    init_min_hits: 30,
    init_max_hits: 40,
    aos_str_req: 10,
    old_str_req: 10,
    armor_base: 13,
    material_type: ArmorMaterialType::Leather,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::All,
}

armor_leaf! {
    /// `Server.Items.LeafLegs`.
    LEAF_LEGS, LEAF_LEGS_CLASS, leaf_legs_new = "Server.Items.LeafLegs" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x2FC9), encoded true;
    flippable: Some((0x2FC9, 0x317F)),
    default_weight: Some(2.0),
    required_races: Some(ALLOW_ELVES_ONLY),
    base_physical_resistance: 2,
    base_fire_resistance: 3,
    base_cold_resistance: 2,
    base_poison_resistance: 4,
    base_energy_resistance: 4,
    init_min_hits: 30,
    init_max_hits: 40,
    aos_str_req: 20,
    old_str_req: 20,
    armor_base: 13,
    material_type: ArmorMaterialType::Leather,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::All,
}

armor_leaf! {
    /// `Server.Items.LeafTonlet`.
    LEAF_TONLET, LEAF_TONLET_CLASS, leaf_tonlet_new = "Server.Items.LeafTonlet" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x2FCA), encoded true;
    flippable: Some((0x2FCA, 0x3180)),
    default_weight: Some(2.0),
    required_races: Some(ALLOW_ELVES_ONLY),
    base_physical_resistance: 2,
    base_fire_resistance: 3,
    base_cold_resistance: 2,
    base_poison_resistance: 4,
    base_energy_resistance: 4,
    init_min_hits: 30,
    init_max_hits: 40,
    aos_str_req: 10,
    old_str_req: 10,
    armor_base: 13,
    material_type: ArmorMaterialType::Leather,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::All,
}

armor_leaf! {
    /// `Server.Items.LeatherArms`.
    LEATHER_ARMS, LEATHER_ARMS_CLASS, leather_arms_new = "Server.Items.LeatherArms" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x13CD), encoded false;
    flippable: Some((0x13cd, 0x13c5)),
    default_weight: Some(2.0),
    base_physical_resistance: 2,
    base_fire_resistance: 4,
    base_cold_resistance: 3,
    base_poison_resistance: 3,
    base_energy_resistance: 3,
    init_min_hits: 30,
    init_max_hits: 40,
    aos_str_req: 20,
    old_str_req: 15,
    armor_base: 13,
    material_type: ArmorMaterialType::Leather,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::All,
}

armor_leaf! {
    /// `Server.Items.LeatherBustierArms`.
    LEATHER_BUSTIER_ARMS, LEATHER_BUSTIER_ARMS_CLASS, leather_bustier_arms_new = "Server.Items.LeatherBustierArms" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x1C0A), encoded false;
    flippable: Some((0x1c0a, 0x1c0b)),
    default_weight: Some(1.0),
    base_physical_resistance: 2,
    base_fire_resistance: 4,
    base_cold_resistance: 3,
    base_poison_resistance: 3,
    base_energy_resistance: 3,
    init_min_hits: 30,
    init_max_hits: 40,
    aos_str_req: 20,
    old_str_req: 15,
    armor_base: 13,
    material_type: ArmorMaterialType::Leather,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::All,
    allow_male_wearer: false,
}

armor_leaf! {
    /// `Server.Items.LeatherDo`.
    LEATHER_DO, LEATHER_DO_CLASS, leather_do_new = "Server.Items.LeatherDo" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x27C6), encoded false;
    default_weight: Some(6.0),
    base_physical_resistance: 2,
    base_fire_resistance: 4,
    base_cold_resistance: 3,
    base_poison_resistance: 3,
    base_energy_resistance: 3,
    init_min_hits: 35,
    init_max_hits: 45,
    aos_str_req: 40,
    old_str_req: 40,
    armor_base: 3,
    material_type: ArmorMaterialType::Leather,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::All,
}

armor_leaf! {
    /// `Server.Items.LeatherGorget`.
    LEATHER_GORGET, LEATHER_GORGET_CLASS, leather_gorget_new = "Server.Items.LeatherGorget" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x13C7), encoded false;
    default_weight: Some(1.0),
    base_physical_resistance: 2,
    base_fire_resistance: 4,
    base_cold_resistance: 3,
    base_poison_resistance: 3,
    base_energy_resistance: 3,
    init_min_hits: 30,
    init_max_hits: 40,
    aos_str_req: 20,
    old_str_req: 10,
    armor_base: 13,
    material_type: ArmorMaterialType::Leather,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::All,
}

armor_leaf! {
    /// `Server.Items.LeatherHaidate`.
    LEATHER_HAIDATE, LEATHER_HAIDATE_CLASS, leather_haidate_new = "Server.Items.LeatherHaidate" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x278A), encoded false;
    default_weight: Some(4.0),
    base_physical_resistance: 2,
    base_fire_resistance: 4,
    base_cold_resistance: 3,
    base_poison_resistance: 3,
    base_energy_resistance: 3,
    init_min_hits: 30,
    init_max_hits: 40,
    aos_str_req: 20,
    old_str_req: 20,
    armor_base: 3,
    material_type: ArmorMaterialType::Leather,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::All,
}

armor_leaf! {
    /// `Server.Items.LeatherHiroSode`.
    LEATHER_HIRO_SODE, LEATHER_HIRO_SODE_CLASS, leather_hiro_sode_new = "Server.Items.LeatherHiroSode" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x277E), encoded false;
    default_weight: Some(1.0),
    base_physical_resistance: 2,
    base_fire_resistance: 4,
    base_cold_resistance: 3,
    base_poison_resistance: 3,
    base_energy_resistance: 3,
    init_min_hits: 40,
    init_max_hits: 50,
    aos_str_req: 25,
    old_str_req: 25,
    armor_base: 3,
    material_type: ArmorMaterialType::Leather,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::All,
}

armor_leaf! {
    /// `Server.Items.LeatherJingasa`.
    LEATHER_JINGASA, LEATHER_JINGASA_CLASS, leather_jingasa_new = "Server.Items.LeatherJingasa" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x2776), encoded false;
    default_weight: Some(3.0),
    base_physical_resistance: 4,
    base_fire_resistance: 3,
    base_cold_resistance: 3,
    base_poison_resistance: 2,
    base_energy_resistance: 3,
    init_min_hits: 20,
    init_max_hits: 30,
    aos_str_req: 25,
    old_str_req: 25,
    armor_base: 3,
    material_type: ArmorMaterialType::Leather,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::All,
}

armor_leaf! {
    /// `Server.Items.LeatherLegs`.
    LEATHER_LEGS, LEATHER_LEGS_CLASS, leather_legs_new = "Server.Items.LeatherLegs" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x13CB), encoded false;
    flippable: Some((0x13cb, 0x13d2)),
    default_weight: Some(4.0),
    base_physical_resistance: 2,
    base_fire_resistance: 4,
    base_cold_resistance: 3,
    base_poison_resistance: 3,
    base_energy_resistance: 3,
    init_min_hits: 30,
    init_max_hits: 40,
    aos_str_req: 20,
    old_str_req: 10,
    armor_base: 13,
    material_type: ArmorMaterialType::Leather,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::All,
}

armor_leaf! {
    /// `Server.Items.LeatherMempo`.
    LEATHER_MEMPO, LEATHER_MEMPO_CLASS, leather_mempo_new = "Server.Items.LeatherMempo" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x277A), encoded false;
    default_weight: Some(2.0),
    base_physical_resistance: 2,
    base_fire_resistance: 4,
    base_cold_resistance: 3,
    base_poison_resistance: 3,
    base_energy_resistance: 3,
    init_min_hits: 35,
    init_max_hits: 40,
    aos_str_req: 30,
    old_str_req: 30,
    armor_base: 3,
    material_type: ArmorMaterialType::Leather,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::All,
}

armor_leaf! {
    /// `Server.Items.LeatherNinjaHood`.
    LEATHER_NINJA_HOOD, LEATHER_NINJA_HOOD_CLASS, leather_ninja_hood_new = "Server.Items.LeatherNinjaHood" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x278E), encoded false;
    default_weight: Some(2.0),
    base_physical_resistance: 2,
    base_fire_resistance: 3,
    base_cold_resistance: 3,
    base_poison_resistance: 3,
    base_energy_resistance: 4,
    init_min_hits: 25,
    init_max_hits: 45,
    aos_str_req: 10,
    old_str_req: 10,
    armor_base: 3,
    material_type: ArmorMaterialType::Leather,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::All,
}

armor_leaf! {
    /// `Server.Items.LeatherNinjaJacket`.
    LEATHER_NINJA_JACKET, LEATHER_NINJA_JACKET_CLASS, leather_ninja_jacket_new = "Server.Items.LeatherNinjaJacket" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x2793), encoded false;
    default_weight: Some(5.0),
    base_physical_resistance: 2,
    base_fire_resistance: 4,
    base_cold_resistance: 3,
    base_poison_resistance: 3,
    base_energy_resistance: 3,
    init_min_hits: 55,
    init_max_hits: 65,
    aos_str_req: 10,
    old_str_req: 10,
    armor_base: 3,
    material_type: ArmorMaterialType::Leather,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::All,
}

armor_leaf! {
    /// `Server.Items.LeatherNinjaMitts`.
    LEATHER_NINJA_MITTS, LEATHER_NINJA_MITTS_CLASS, leather_ninja_mitts_new = "Server.Items.LeatherNinjaMitts" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x2792), encoded false;
    default_weight: Some(2.0),
    base_physical_resistance: 2,
    base_fire_resistance: 4,
    base_cold_resistance: 3,
    base_poison_resistance: 3,
    base_energy_resistance: 3,
    init_min_hits: 25,
    init_max_hits: 25,
    aos_str_req: 10,
    old_str_req: 10,
    armor_base: 3,
    material_type: ArmorMaterialType::Leather,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::All,
}

armor_leaf! {
    /// `Server.Items.LeatherNinjaPants`.
    LEATHER_NINJA_PANTS, LEATHER_NINJA_PANTS_CLASS, leather_ninja_pants_new = "Server.Items.LeatherNinjaPants" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x2791), encoded false;
    default_weight: Some(3.0),
    base_physical_resistance: 2,
    base_fire_resistance: 4,
    base_cold_resistance: 3,
    base_poison_resistance: 3,
    base_energy_resistance: 3,
    init_min_hits: 40,
    init_max_hits: 50,
    aos_str_req: 10,
    old_str_req: 10,
    armor_base: 3,
    material_type: ArmorMaterialType::Leather,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::All,
}

armor_leaf! {
    /// `Server.Items.LeatherShorts`.
    LEATHER_SHORTS, LEATHER_SHORTS_CLASS, leather_shorts_new = "Server.Items.LeatherShorts" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x1C00), encoded false;
    flippable: Some((0x1c00, 0x1c01)),
    default_weight: Some(3.0),
    base_physical_resistance: 2,
    base_fire_resistance: 4,
    base_cold_resistance: 3,
    base_poison_resistance: 3,
    base_energy_resistance: 3,
    init_min_hits: 30,
    init_max_hits: 40,
    aos_str_req: 20,
    old_str_req: 10,
    armor_base: 13,
    material_type: ArmorMaterialType::Leather,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::All,
    allow_male_wearer: false,
}

armor_leaf! {
    /// `Server.Items.LeatherSkirt`.
    LEATHER_SKIRT, LEATHER_SKIRT_CLASS, leather_skirt_new = "Server.Items.LeatherSkirt" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x1C08), encoded false;
    flippable: Some((0x1c08, 0x1c09)),
    default_weight: Some(1.0),
    base_physical_resistance: 2,
    base_fire_resistance: 4,
    base_cold_resistance: 3,
    base_poison_resistance: 3,
    base_energy_resistance: 3,
    init_min_hits: 30,
    init_max_hits: 40,
    aos_str_req: 20,
    old_str_req: 10,
    armor_base: 13,
    material_type: ArmorMaterialType::Leather,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::All,
    allow_male_wearer: false,
}

armor_leaf! {
    /// `Server.Items.LeatherSuneate`.
    LEATHER_SUNEATE, LEATHER_SUNEATE_CLASS, leather_suneate_new = "Server.Items.LeatherSuneate" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x2786), encoded false;
    default_weight: Some(4.0),
    base_physical_resistance: 2,
    base_fire_resistance: 4,
    base_cold_resistance: 3,
    base_poison_resistance: 3,
    base_energy_resistance: 3,
    init_min_hits: 25,
    init_max_hits: 40,
    aos_str_req: 20,
    old_str_req: 20,
    armor_base: 3,
    material_type: ArmorMaterialType::Leather,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::All,
}

armor_leaf! {
    /// `Server.Items.DecorativePlateKabuto`.
    DECORATIVE_PLATE_KABUTO, DECORATIVE_PLATE_KABUTO_CLASS, decorative_plate_kabuto_new = "Server.Items.DecorativePlateKabuto" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x2778), encoded false;
    default_weight: Some(6.0),
    base_physical_resistance: 6,
    base_fire_resistance: 2,
    base_cold_resistance: 2,
    base_poison_resistance: 2,
    base_energy_resistance: 3,
    init_min_hits: 55,
    init_max_hits: 75,
    aos_str_req: 70,
    old_str_req: 70,
    armor_base: 3,
    material_type: ArmorMaterialType::Plate,
}

armor_leaf! {
    /// `Server.Items.FemalePlateChest`.
    FEMALE_PLATE_CHEST, FEMALE_PLATE_CHEST_CLASS, female_plate_chest_new = "Server.Items.FemalePlateChest" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x1C04), encoded false;
    flippable: Some((0x1c04, 0x1c05)),
    default_weight: Some(4.0),
    base_physical_resistance: 5,
    base_fire_resistance: 3,
    base_cold_resistance: 2,
    base_poison_resistance: 3,
    base_energy_resistance: 2,
    init_min_hits: 50,
    init_max_hits: 65,
    aos_str_req: 95,
    old_str_req: 45,
    old_dex_bonus: -5,
    allow_male_wearer: false,
    armor_base: 30,
    material_type: ArmorMaterialType::Plate,
}

armor_leaf! {
    /// `Server.Items.FemaleElvenPlateChest`.
    FEMALE_ELVEN_PLATE_CHEST, FEMALE_ELVEN_PLATE_CHEST_CLASS, female_elven_plate_chest_new = "Server.Items.FemaleElvenPlateChest" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x2B6D), encoded true;
    flippable: Some((0x2B6D, 0x3164)),
    default_weight: Some(8.0),
    base_physical_resistance: 5,
    base_fire_resistance: 3,
    base_cold_resistance: 2,
    base_poison_resistance: 3,
    base_energy_resistance: 2,
    init_min_hits: 50,
    init_max_hits: 65,
    aos_str_req: 95,
    old_str_req: 95,
    allow_male_wearer: false,
    armor_base: 30,
    material_type: ArmorMaterialType::Wood,
}

armor_leaf! {
    /// `Server.Items.HeavyPlateJingasa`.
    HEAVY_PLATE_JINGASA, HEAVY_PLATE_JINGASA_CLASS, heavy_plate_jingasa_new = "Server.Items.HeavyPlateJingasa" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x2777), encoded false;
    default_weight: Some(5.0),
    base_physical_resistance: 7,
    base_fire_resistance: 2,
    base_cold_resistance: 2,
    base_poison_resistance: 2,
    base_energy_resistance: 2,
    init_min_hits: 50,
    init_max_hits: 70,
    aos_str_req: 55,
    old_str_req: 55,
    armor_base: 4,
    material_type: ArmorMaterialType::Plate,
}

armor_leaf! {
    /// `Server.Items.LightPlateJingasa`.
    LIGHT_PLATE_JINGASA, LIGHT_PLATE_JINGASA_CLASS, light_plate_jingasa_new = "Server.Items.LightPlateJingasa" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x2781), encoded false;
    default_weight: Some(5.0),
    base_physical_resistance: 7,
    base_fire_resistance: 2,
    base_cold_resistance: 2,
    base_poison_resistance: 2,
    base_energy_resistance: 2,
    init_min_hits: 55,
    init_max_hits: 60,
    aos_str_req: 55,
    old_str_req: 55,
    armor_base: 4,
    material_type: ArmorMaterialType::Plate,
}

armor_leaf! {
    /// `Server.Items.PlateArms`.
    PLATE_ARMS, PLATE_ARMS_CLASS, plate_arms_new = "Server.Items.PlateArms" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x1410), encoded false;
    flippable: Some((0x1410, 0x1417)),
    default_weight: Some(5.0),
    base_physical_resistance: 5,
    base_fire_resistance: 3,
    base_cold_resistance: 2,
    base_poison_resistance: 3,
    base_energy_resistance: 2,
    init_min_hits: 50,
    init_max_hits: 65,
    aos_str_req: 80,
    old_str_req: 40,
    old_dex_bonus: -2,
    armor_base: 40,
    material_type: ArmorMaterialType::Plate,
}

armor_leaf! {
    /// `Server.Items.PlateBattleKabuto`.
    PLATE_BATTLE_KABUTO, PLATE_BATTLE_KABUTO_CLASS, plate_battle_kabuto_new = "Server.Items.PlateBattleKabuto" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x2785), encoded false;
    default_weight: Some(6.0),
    base_physical_resistance: 6,
    base_fire_resistance: 2,
    base_cold_resistance: 2,
    base_poison_resistance: 2,
    base_energy_resistance: 3,
    init_min_hits: 60,
    init_max_hits: 65,
    aos_str_req: 70,
    old_str_req: 70,
    armor_base: 3,
    material_type: ArmorMaterialType::Plate,
}

armor_leaf! {
    /// `Server.Items.PlateDo`.
    PLATE_DO, PLATE_DO_CLASS, plate_do_new = "Server.Items.PlateDo" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x277D), encoded false;
    default_weight: Some(10.0),
    base_physical_resistance: 5,
    base_fire_resistance: 3,
    base_cold_resistance: 2,
    base_poison_resistance: 3,
    base_energy_resistance: 2,
    init_min_hits: 60,
    init_max_hits: 70,
    aos_str_req: 85,
    old_str_req: 85,
    armor_base: 3,
    material_type: ArmorMaterialType::Plate,
}

armor_leaf! {
    /// `Server.Items.PlateGloves`.
    PLATE_GLOVES, PLATE_GLOVES_CLASS, plate_gloves_new = "Server.Items.PlateGloves" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x1414), encoded false;
    flippable: Some((0x1414, 0x1418)),
    default_weight: Some(2.0),
    base_physical_resistance: 5,
    base_fire_resistance: 3,
    base_cold_resistance: 2,
    base_poison_resistance: 3,
    base_energy_resistance: 2,
    init_min_hits: 50,
    init_max_hits: 65,
    aos_str_req: 70,
    old_str_req: 30,
    old_dex_bonus: -2,
    armor_base: 40,
    material_type: ArmorMaterialType::Plate,
}

armor_leaf! {
    /// `Server.Items.PlateGorget`.
    PLATE_GORGET, PLATE_GORGET_CLASS, plate_gorget_new = "Server.Items.PlateGorget" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x1413), encoded false;
    default_weight: Some(2.0),
    base_physical_resistance: 5,
    base_fire_resistance: 3,
    base_cold_resistance: 2,
    base_poison_resistance: 3,
    base_energy_resistance: 2,
    init_min_hits: 50,
    init_max_hits: 65,
    aos_str_req: 45,
    old_str_req: 30,
    old_dex_bonus: -1,
    armor_base: 40,
    material_type: ArmorMaterialType::Plate,
}

armor_leaf! {
    /// `Server.Items.PlateHaidate`.
    PLATE_HAIDATE, PLATE_HAIDATE_CLASS, plate_haidate_new = "Server.Items.PlateHaidate" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x278D), encoded false;
    default_weight: Some(7.0),
    base_physical_resistance: 5,
    base_fire_resistance: 3,
    base_cold_resistance: 2,
    base_poison_resistance: 3,
    base_energy_resistance: 2,
    init_min_hits: 55,
    init_max_hits: 65,
    aos_str_req: 80,
    old_str_req: 80,
    armor_base: 3,
    material_type: ArmorMaterialType::Plate,
}

armor_leaf! {
    /// `Server.Items.PlateHatsuburi`.
    PLATE_HATSUBURI, PLATE_HATSUBURI_CLASS, plate_hatsuburi_new = "Server.Items.PlateHatsuburi" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x2775), encoded false;
    default_weight: Some(5.0),
    base_physical_resistance: 5,
    base_fire_resistance: 3,
    base_cold_resistance: 2,
    base_poison_resistance: 2,
    base_energy_resistance: 3,
    init_min_hits: 55,
    init_max_hits: 75,
    aos_str_req: 65,
    old_str_req: 65,
    armor_base: 4,
    material_type: ArmorMaterialType::Plate,
}

armor_leaf! {
    /// `Server.Items.PlateHiroSode`.
    PLATE_HIRO_SODE, PLATE_HIRO_SODE_CLASS, plate_hiro_sode_new = "Server.Items.PlateHiroSode" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x2780), encoded false;
    default_weight: Some(3.0),
    base_physical_resistance: 5,
    base_fire_resistance: 3,
    base_cold_resistance: 2,
    base_poison_resistance: 3,
    base_energy_resistance: 2,
    init_min_hits: 55,
    init_max_hits: 75,
    aos_str_req: 75,
    old_str_req: 75,
    armor_base: 3,
    material_type: ArmorMaterialType::Plate,
}

armor_leaf! {
    /// `Server.Items.PlateLegs`.
    PLATE_LEGS, PLATE_LEGS_CLASS, plate_legs_new = "Server.Items.PlateLegs" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x1411), encoded false;
    flippable: Some((0x1411, 0x141a)),
    default_weight: Some(7.0),
    base_physical_resistance: 5,
    base_fire_resistance: 3,
    base_cold_resistance: 2,
    base_poison_resistance: 3,
    base_energy_resistance: 2,
    init_min_hits: 50,
    init_max_hits: 65,
    aos_str_req: 90,
    old_str_req: 60,
    old_dex_bonus: -6,
    armor_base: 40,
    material_type: ArmorMaterialType::Plate,
}

armor_leaf! {
    /// `Server.Items.PlateMempo`.
    PLATE_MEMPO, PLATE_MEMPO_CLASS, plate_mempo_new = "Server.Items.PlateMempo" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x2779), encoded false;
    default_weight: Some(3.0),
    base_physical_resistance: 5,
    base_fire_resistance: 3,
    base_cold_resistance: 2,
    base_poison_resistance: 3,
    base_energy_resistance: 2,
    init_min_hits: 60,
    init_max_hits: 70,
    aos_str_req: 50,
    old_str_req: 50,
    armor_base: 4,
    material_type: ArmorMaterialType::Plate,
}

armor_leaf! {
    /// `Server.Items.PlateSuneate`.
    PLATE_SUNEATE, PLATE_SUNEATE_CLASS, plate_suneate_new = "Server.Items.PlateSuneate" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x2788), encoded false;
    default_weight: Some(7.0),
    base_physical_resistance: 5,
    base_fire_resistance: 3,
    base_cold_resistance: 2,
    base_poison_resistance: 3,
    base_energy_resistance: 2,
    init_min_hits: 55,
    init_max_hits: 65,
    aos_str_req: 80,
    old_str_req: 80,
    armor_base: 3,
    material_type: ArmorMaterialType::Plate,
}

armor_leaf! {
    /// `Server.Items.SmallPlateJingasa`.
    SMALL_PLATE_JINGASA, SMALL_PLATE_JINGASA_CLASS, small_plate_jingasa_new = "Server.Items.SmallPlateJingasa" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x2784), encoded false;
    default_weight: Some(5.0),
    base_physical_resistance: 7,
    base_fire_resistance: 2,
    base_cold_resistance: 2,
    base_poison_resistance: 2,
    base_energy_resistance: 2,
    init_min_hits: 55,
    init_max_hits: 60,
    aos_str_req: 55,
    old_str_req: 55,
    armor_base: 4,
    material_type: ArmorMaterialType::Plate,
}

armor_leaf! {
    /// `Server.Items.StandardPlateKabuto`.
    STANDARD_PLATE_KABUTO, STANDARD_PLATE_KABUTO_CLASS, standard_plate_kabuto_new = "Server.Items.StandardPlateKabuto" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x2789), encoded false;
    default_weight: Some(6.0),
    base_physical_resistance: 6,
    base_fire_resistance: 2,
    base_cold_resistance: 2,
    base_poison_resistance: 2,
    base_energy_resistance: 3,
    init_min_hits: 60,
    init_max_hits: 65,
    aos_str_req: 70,
    old_str_req: 70,
    armor_base: 3,
    material_type: ArmorMaterialType::Plate,
}

armor_leaf! {
    /// `Server.Items.WoodlandArms`.
    WOODLAND_ARMS, WOODLAND_ARMS_CLASS, woodland_arms_new = "Server.Items.WoodlandArms" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x2B6C), encoded true;
    flippable: Some((0x2B6C, 0x3163)),
    default_weight: Some(5.0),
    base_physical_resistance: 5,
    base_fire_resistance: 3,
    base_cold_resistance: 2,
    base_poison_resistance: 3,
    base_energy_resistance: 2,
    init_min_hits: 50,
    init_max_hits: 65,
    aos_str_req: 80,
    old_str_req: 80,
    armor_base: 40,
    material_type: ArmorMaterialType::Wood,
    required_races: Some(ALLOW_ELVES_ONLY),
}

armor_leaf! {
    /// `Server.Items.WoodlandChest`.
    WOODLAND_CHEST, WOODLAND_CHEST_CLASS, woodland_chest_new = "Server.Items.WoodlandChest" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x2B67), encoded true;
    flippable: Some((0x2B67, 0x315E)),
    default_weight: Some(8.0),
    base_physical_resistance: 5,
    base_fire_resistance: 3,
    base_cold_resistance: 2,
    base_poison_resistance: 3,
    base_energy_resistance: 2,
    init_min_hits: 50,
    init_max_hits: 65,
    aos_str_req: 95,
    old_str_req: 95,
    armor_base: 40,
    material_type: ArmorMaterialType::Wood,
    required_races: Some(ALLOW_ELVES_ONLY),
}

armor_leaf! {
    /// `Server.Items.WoodlandGloves`.
    WOODLAND_GLOVES, WOODLAND_GLOVES_CLASS, woodland_gloves_new = "Server.Items.WoodlandGloves" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x2B6A), encoded true;
    flippable: Some((0x2B6A, 0x3161)),
    default_weight: Some(2.0),
    base_physical_resistance: 5,
    base_fire_resistance: 3,
    base_cold_resistance: 2,
    base_poison_resistance: 3,
    base_energy_resistance: 2,
    init_min_hits: 50,
    init_max_hits: 65,
    aos_str_req: 70,
    old_str_req: 70,
    armor_base: 40,
    material_type: ArmorMaterialType::Wood,
    required_races: Some(ALLOW_ELVES_ONLY),
}

armor_leaf! {
    /// `Server.Items.WoodlandGorget`.
    WOODLAND_GORGET, WOODLAND_GORGET_CLASS, woodland_gorget_new = "Server.Items.WoodlandGorget" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x2B69), encoded true;
    flippable: Some((0x2B69, 0x3160)),
    base_physical_resistance: 5,
    base_fire_resistance: 3,
    base_cold_resistance: 2,
    base_poison_resistance: 3,
    base_energy_resistance: 2,
    init_min_hits: 50,
    init_max_hits: 65,
    aos_str_req: 45,
    old_str_req: 45,
    armor_base: 40,
    material_type: ArmorMaterialType::Wood,
    required_races: Some(ALLOW_ELVES_ONLY),
}

armor_leaf! {
    /// `Server.Items.WoodlandLegs`.
    WOODLAND_LEGS, WOODLAND_LEGS_CLASS, woodland_legs_new = "Server.Items.WoodlandLegs" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x2B6B), encoded true;
    flippable: Some((0x2B6B, 0x3162)),
    default_weight: Some(8.0),
    base_physical_resistance: 5,
    base_fire_resistance: 3,
    base_cold_resistance: 2,
    base_poison_resistance: 3,
    base_energy_resistance: 2,
    init_min_hits: 50,
    init_max_hits: 65,
    aos_str_req: 90,
    old_str_req: 90,
    armor_base: 40,
    material_type: ArmorMaterialType::Wood,
    required_races: Some(ALLOW_ELVES_ONLY),
}

armor_leaf! {
    /// `Server.Items.RangerArms`.
    RANGER_ARMS, RANGER_ARMS_CLASS, ranger_arms_new = "Server.Items.RangerArms" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x13DC), encoded false;
    @init |w, id| {
        hue(w, id, 0x59C);
    };
    flippable: Some((0x13dc, 0x13d4)),
    default_weight: Some(4.0),
    base_physical_resistance: 2,
    base_fire_resistance: 4,
    base_cold_resistance: 3,
    base_poison_resistance: 3,
    base_energy_resistance: 4,
    init_min_hits: 35,
    init_max_hits: 45,
    aos_str_req: 25,
    old_str_req: 25,
    armor_base: 16,
    material_type: ArmorMaterialType::Studded,
    default_resource: CraftResource::RegularLeather,
    label_number: 1041493,
}

armor_leaf! {
    /// `Server.Items.RangerChest`.
    RANGER_CHEST, RANGER_CHEST_CLASS, ranger_chest_new = "Server.Items.RangerChest" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x13DB), encoded false;
    @init |w, id| {
        hue(w, id, 0x59C);
    };
    flippable: Some((0x13db, 0x13e2)),
    default_weight: Some(8.0),
    base_physical_resistance: 2,
    base_fire_resistance: 4,
    base_cold_resistance: 3,
    base_poison_resistance: 3,
    base_energy_resistance: 4,
    init_min_hits: 35,
    init_max_hits: 45,
    aos_str_req: 35,
    old_str_req: 35,
    armor_base: 16,
    material_type: ArmorMaterialType::Studded,
    default_resource: CraftResource::RegularLeather,
    label_number: 1041497,
}

armor_leaf! {
    /// `Server.Items.RangerGloves`.
    RANGER_GLOVES, RANGER_GLOVES_CLASS, ranger_gloves_new = "Server.Items.RangerGloves" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x13D5), encoded false;
    @init |w, id| {
        hue(w, id, 0x59C);
    };
    flippable: Some((0x13d5, 0x13dd)),
    default_weight: Some(1.0),
    base_physical_resistance: 2,
    base_fire_resistance: 4,
    base_cold_resistance: 3,
    base_poison_resistance: 3,
    base_energy_resistance: 4,
    init_min_hits: 35,
    init_max_hits: 45,
    aos_str_req: 25,
    old_str_req: 25,
    armor_base: 16,
    material_type: ArmorMaterialType::Studded,
    default_resource: CraftResource::RegularLeather,
    label_number: 1041494,
}

armor_leaf! {
    /// `Server.Items.RangerGorget`.
    RANGER_GORGET, RANGER_GORGET_CLASS, ranger_gorget_new = "Server.Items.RangerGorget" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x13D6), encoded false;
    @init |w, id| {
        hue(w, id, 0x59C);
    };
    default_weight: Some(1.0),
    base_physical_resistance: 2,
    base_fire_resistance: 4,
    base_cold_resistance: 3,
    base_poison_resistance: 3,
    base_energy_resistance: 4,
    init_min_hits: 35,
    init_max_hits: 45,
    aos_str_req: 25,
    old_str_req: 25,
    armor_base: 16,
    material_type: ArmorMaterialType::Studded,
    default_resource: CraftResource::RegularLeather,
    label_number: 1041495,
}

armor_leaf! {
    /// `Server.Items.RangerLegs`.
    RANGER_LEGS, RANGER_LEGS_CLASS, ranger_legs_new = "Server.Items.RangerLegs" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x13DA), encoded false;
    @init |w, id| {
        hue(w, id, 0x59C);
    };
    flippable: Some((0x13da, 0x13e1)),
    default_weight: Some(3.0),
    base_physical_resistance: 2,
    base_fire_resistance: 4,
    base_cold_resistance: 3,
    base_poison_resistance: 3,
    base_energy_resistance: 4,
    init_min_hits: 35,
    init_max_hits: 45,
    aos_str_req: 30,
    old_str_req: 35,
    armor_base: 16,
    material_type: ArmorMaterialType::Studded,
    default_resource: CraftResource::RegularLeather,
    label_number: 1041496,
}

armor_leaf! {
    /// `Server.Items.RingmailArms`.
    RINGMAIL_ARMS, RINGMAIL_ARMS_CLASS, ringmail_arms_new = "Server.Items.RingmailArms" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x13EE), encoded false;
    flippable: Some((0x13ee, 0x13ef)),
    default_weight: Some(15.0),
    base_physical_resistance: 3,
    base_fire_resistance: 3,
    base_cold_resistance: 1,
    base_poison_resistance: 5,
    base_energy_resistance: 3,
    init_min_hits: 40,
    init_max_hits: 50,
    aos_str_req: 40,
    old_str_req: 20,
    old_dex_bonus: -1,
    armor_base: 22,
    material_type: ArmorMaterialType::Ringmail,
}

armor_leaf! {
    /// `Server.Items.RingmailChest`.
    RINGMAIL_CHEST, RINGMAIL_CHEST_CLASS, ringmail_chest_new = "Server.Items.RingmailChest" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x13EC), encoded false;
    flippable: Some((0x13ec, 0x13ed)),
    default_weight: Some(15.0),
    base_physical_resistance: 3,
    base_fire_resistance: 3,
    base_cold_resistance: 1,
    base_poison_resistance: 5,
    base_energy_resistance: 3,
    init_min_hits: 40,
    init_max_hits: 50,
    aos_str_req: 40,
    old_str_req: 20,
    old_dex_bonus: -2,
    armor_base: 22,
    material_type: ArmorMaterialType::Ringmail,
}

armor_leaf! {
    /// `Server.Items.RingmailGloves`.
    RINGMAIL_GLOVES, RINGMAIL_GLOVES_CLASS, ringmail_gloves_new = "Server.Items.RingmailGloves" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x13EB), encoded false;
    flippable: Some((0x13eb, 0x13f2)),
    default_weight: Some(2.0),
    base_physical_resistance: 3,
    base_fire_resistance: 3,
    base_cold_resistance: 1,
    base_poison_resistance: 5,
    base_energy_resistance: 3,
    init_min_hits: 40,
    init_max_hits: 50,
    aos_str_req: 40,
    old_str_req: 20,
    old_dex_bonus: -1,
    armor_base: 22,
    material_type: ArmorMaterialType::Ringmail,
}

armor_leaf! {
    /// `Server.Items.RingmailLegs`.
    RINGMAIL_LEGS, RINGMAIL_LEGS_CLASS, ringmail_legs_new = "Server.Items.RingmailLegs" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x13F0), encoded false;
    flippable: Some((0x13f0, 0x13f1)),
    default_weight: Some(15.0),
    base_physical_resistance: 3,
    base_fire_resistance: 3,
    base_cold_resistance: 1,
    base_poison_resistance: 5,
    base_energy_resistance: 3,
    init_min_hits: 40,
    init_max_hits: 50,
    aos_str_req: 40,
    old_str_req: 20,
    old_dex_bonus: -1,
    armor_base: 22,
    material_type: ArmorMaterialType::Ringmail,
}

armor_leaf! {
    /// `Server.Items.SpiritualityHelm`.
    SPIRITUALITY_HELM, SPIRITUALITY_HELM_CLASS, spirituality_helm_new = "Server.Items.SpiritualityHelm" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x2B10), encoded true;
    @init |w, id| {
        ops::set_loot_type(w, id, LootType::Blessed);
        ops::set_weight(w, id, 6.0);
        hue(w, id, 0x226);
        set_attr(w, id, AttrKind::Armor, AosArmorAttribute::SelfRepair as u32, 5);
    };
    flippable: Some((0x2B10, 0x2B11)),
    label_number: 1075237,
    base_physical_resistance: 13,
    base_fire_resistance: 13,
    base_cold_resistance: 12,
    base_poison_resistance: 14,
    base_energy_resistance: 13,
    init_min_hits: 255,
    init_max_hits: 255,
    aos_str_req: 25,
    material_type: ArmorMaterialType::Plate,
}

armor_leaf! {
    /// `Server.Items.ValorGauntlets`.
    VALOR_GAUNTLETS, VALOR_GAUNTLETS_CLASS, valor_gauntlets_new = "Server.Items.ValorGauntlets" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x2B0C), encoded true;
    @init |w, id| {
        ops::set_loot_type(w, id, LootType::Blessed);
        ops::set_weight(w, id, 4.0);
        hue(w, id, 0x226);
        set_attr(w, id, AttrKind::Armor, AosArmorAttribute::SelfRepair as u32, 5);
    };
    flippable: Some((0x2B0C, 0x2B0D)),
    label_number: 1075238,
    base_physical_resistance: 11,
    base_fire_resistance: 11,
    base_cold_resistance: 13,
    base_poison_resistance: 14,
    base_energy_resistance: 10,
    init_min_hits: 255,
    init_max_hits: 255,
    aos_str_req: 50,
    material_type: ArmorMaterialType::Plate,
}

armor_leaf! {
    /// `Server.Items.GargishStoneArmsType1`.
    GARGISH_STONE_ARMS_TYPE1, GARGISH_STONE_ARMS_TYPE1_CLASS, gargish_stone_arms_type1_new = "Server.Items.GargishStoneArmsType1" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x284), encoded false;
    default_weight: Some(10.0),
    required_races: Some(ALLOW_GARGOYLES_ONLY),
    base_physical_resistance: 6,
    base_fire_resistance: 6,
    base_cold_resistance: 4,
    base_poison_resistance: 8,
    base_energy_resistance: 6,
    init_min_hits: 40,
    init_max_hits: 50,
    aos_str_req: 40,
    old_str_req: 40,
    armor_base: 40,
    material_type: ArmorMaterialType::Stone,
}

armor_leaf! {
    /// `Server.Items.GargishStoneArmsType2`.
    GARGISH_STONE_ARMS_TYPE2, GARGISH_STONE_ARMS_TYPE2_CLASS, gargish_stone_arms_type2_new = "Server.Items.GargishStoneArmsType2" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x283), encoded false;
    default_weight: Some(10.0),
    required_races: Some(ALLOW_GARGOYLES_ONLY),
    base_physical_resistance: 6,
    base_fire_resistance: 6,
    base_cold_resistance: 4,
    base_poison_resistance: 8,
    base_energy_resistance: 6,
    init_min_hits: 40,
    init_max_hits: 50,
    aos_str_req: 40,
    old_str_req: 40,
    armor_base: 40,
    material_type: ArmorMaterialType::Stone,
}

armor_leaf! {
    /// `Server.Items.GargishStoneChestType1`.
    GARGISH_STONE_CHEST_TYPE1, GARGISH_STONE_CHEST_TYPE1_CLASS, gargish_stone_chest_type1_new = "Server.Items.GargishStoneChestType1" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x286), encoded false;
    default_weight: Some(15.0),
    required_races: Some(ALLOW_GARGOYLES_ONLY),
    base_physical_resistance: 6,
    base_fire_resistance: 6,
    base_cold_resistance: 4,
    base_poison_resistance: 8,
    base_energy_resistance: 6,
    init_min_hits: 40,
    init_max_hits: 50,
    aos_str_req: 40,
    old_str_req: 40,
    armor_base: 40,
    material_type: ArmorMaterialType::Stone,
}

armor_leaf! {
    /// `Server.Items.GargishStoneChestType2`.
    GARGISH_STONE_CHEST_TYPE2, GARGISH_STONE_CHEST_TYPE2_CLASS, gargish_stone_chest_type2_new = "Server.Items.GargishStoneChestType2" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x285), encoded false;
    default_weight: Some(15.0),
    required_races: Some(ALLOW_GARGOYLES_ONLY),
    base_physical_resistance: 6,
    base_fire_resistance: 6,
    base_cold_resistance: 4,
    base_poison_resistance: 8,
    base_energy_resistance: 6,
    init_min_hits: 40,
    init_max_hits: 50,
    aos_str_req: 40,
    old_str_req: 40,
    armor_base: 40,
    material_type: ArmorMaterialType::Stone,
}

armor_leaf! {
    /// `Server.Items.GargishStoneKiltType1`.
    GARGISH_STONE_KILT_TYPE1, GARGISH_STONE_KILT_TYPE1_CLASS, gargish_stone_kilt_type1_new = "Server.Items.GargishStoneKiltType1" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x288), encoded false;
    default_weight: Some(10.0),
    required_races: Some(ALLOW_GARGOYLES_ONLY),
    base_physical_resistance: 6,
    base_fire_resistance: 6,
    base_cold_resistance: 4,
    base_poison_resistance: 8,
    base_energy_resistance: 6,
    init_min_hits: 40,
    init_max_hits: 50,
    aos_str_req: 40,
    old_str_req: 40,
    armor_base: 40,
    material_type: ArmorMaterialType::Stone,
}

armor_leaf! {
    /// `Server.Items.GargishStoneKiltType2`.
    GARGISH_STONE_KILT_TYPE2, GARGISH_STONE_KILT_TYPE2_CLASS, gargish_stone_kilt_type2_new = "Server.Items.GargishStoneKiltType2" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x287), encoded false;
    default_weight: Some(10.0),
    required_races: Some(ALLOW_GARGOYLES_ONLY),
    base_physical_resistance: 6,
    base_fire_resistance: 6,
    base_cold_resistance: 4,
    base_poison_resistance: 8,
    base_energy_resistance: 6,
    init_min_hits: 40,
    init_max_hits: 50,
    aos_str_req: 40,
    old_str_req: 40,
    armor_base: 40,
    material_type: ArmorMaterialType::Stone,
}

armor_leaf! {
    /// `Server.Items.GargishStoneLegsType1`.
    GARGISH_STONE_LEGS_TYPE1, GARGISH_STONE_LEGS_TYPE1_CLASS, gargish_stone_legs_type1_new = "Server.Items.GargishStoneLegsType1" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x28A), encoded false;
    default_weight: Some(15.0),
    required_races: Some(ALLOW_GARGOYLES_ONLY),
    base_physical_resistance: 6,
    base_fire_resistance: 6,
    base_cold_resistance: 4,
    base_poison_resistance: 8,
    base_energy_resistance: 6,
    init_min_hits: 40,
    init_max_hits: 50,
    aos_str_req: 40,
    old_str_req: 40,
    armor_base: 40,
    material_type: ArmorMaterialType::Stone,
}

armor_leaf! {
    /// `Server.Items.GargishStoneLegsType2`.
    GARGISH_STONE_LEGS_TYPE2, GARGISH_STONE_LEGS_TYPE2_CLASS, gargish_stone_legs_type2_new = "Server.Items.GargishStoneLegsType2" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x289), encoded false;
    default_weight: Some(15.0),
    required_races: Some(ALLOW_GARGOYLES_ONLY),
    base_physical_resistance: 6,
    base_fire_resistance: 6,
    base_cold_resistance: 4,
    base_poison_resistance: 8,
    base_energy_resistance: 6,
    init_min_hits: 40,
    init_max_hits: 50,
    aos_str_req: 40,
    old_str_req: 40,
    armor_base: 40,
    material_type: ArmorMaterialType::Stone,
}

armor_leaf! {
    /// `Server.Items.FemaleStuddedChest`.
    FEMALE_STUDDED_CHEST, FEMALE_STUDDED_CHEST_CLASS, female_studded_chest_new = "Server.Items.FemaleStuddedChest" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x1C02), encoded false;
    flippable: Some((0x1c02, 0x1c03)),
    default_weight: Some(6.0),
    base_physical_resistance: 2,
    base_fire_resistance: 4,
    base_cold_resistance: 3,
    base_poison_resistance: 3,
    base_energy_resistance: 4,
    init_min_hits: 35,
    init_max_hits: 45,
    aos_str_req: 35,
    old_str_req: 35,
    armor_base: 16,
    material_type: ArmorMaterialType::Studded,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::Half,
    allow_male_wearer: false,
}

armor_leaf! {
    /// `Server.Items.GargishStuddedArmsType1`.
    GARGISH_STUDDED_ARMS_TYPE1, GARGISH_STUDDED_ARMS_TYPE1_CLASS, gargish_studded_arms_type1_new = "Server.Items.GargishStuddedArmsType1" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x284), encoded false;
    default_weight: Some(10.0),
    required_races: Some(ALLOW_GARGOYLES_ONLY),
    base_physical_resistance: 6,
    base_fire_resistance: 6,
    base_cold_resistance: 4,
    base_poison_resistance: 8,
    base_energy_resistance: 6,
    init_min_hits: 40,
    init_max_hits: 50,
    aos_str_req: 40,
    old_str_req: 40,
    material_type: ArmorMaterialType::Studded,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::Half,
}

armor_leaf! {
    /// `Server.Items.GargishStuddedArmsType2`.
    GARGISH_STUDDED_ARMS_TYPE2, GARGISH_STUDDED_ARMS_TYPE2_CLASS, gargish_studded_arms_type2_new = "Server.Items.GargishStuddedArmsType2" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x283), encoded false;
    default_weight: Some(10.0),
    required_races: Some(ALLOW_GARGOYLES_ONLY),
    base_physical_resistance: 6,
    base_fire_resistance: 6,
    base_cold_resistance: 4,
    base_poison_resistance: 8,
    base_energy_resistance: 6,
    init_min_hits: 40,
    init_max_hits: 50,
    aos_str_req: 40,
    old_str_req: 40,
    armor_base: 16,
    material_type: ArmorMaterialType::Studded,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::Half,
}

armor_leaf! {
    /// `Server.Items.GargishStuddedChestType1`.
    GARGISH_STUDDED_CHEST_TYPE1, GARGISH_STUDDED_CHEST_TYPE1_CLASS, gargish_studded_chest_type1_new = "Server.Items.GargishStuddedChestType1" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x286), encoded false;
    default_weight: Some(15.0),
    required_races: Some(ALLOW_GARGOYLES_ONLY),
    base_physical_resistance: 6,
    base_fire_resistance: 6,
    base_cold_resistance: 4,
    base_poison_resistance: 8,
    base_energy_resistance: 6,
    init_min_hits: 40,
    init_max_hits: 50,
    aos_str_req: 40,
    old_str_req: 40,
    armor_base: 16,
    material_type: ArmorMaterialType::Studded,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::Half,
}

armor_leaf! {
    /// `Server.Items.GargishStuddedChestType2`.
    GARGISH_STUDDED_CHEST_TYPE2, GARGISH_STUDDED_CHEST_TYPE2_CLASS, gargish_studded_chest_type2_new = "Server.Items.GargishStuddedChestType2" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x285), encoded false;
    default_weight: Some(15.0),
    required_races: Some(ALLOW_GARGOYLES_ONLY),
    base_physical_resistance: 6,
    base_fire_resistance: 6,
    base_cold_resistance: 4,
    base_poison_resistance: 8,
    base_energy_resistance: 6,
    init_min_hits: 40,
    init_max_hits: 50,
    aos_str_req: 40,
    old_str_req: 40,
    armor_base: 16,
    material_type: ArmorMaterialType::Studded,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::Half,
}

armor_leaf! {
    /// `Server.Items.GargishStuddedKiltType1`.
    GARGISH_STUDDED_KILT_TYPE1, GARGISH_STUDDED_KILT_TYPE1_CLASS, gargish_studded_kilt_type1_new = "Server.Items.GargishStuddedKiltType1" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x288), encoded false;
    default_weight: Some(10.0),
    required_races: Some(ALLOW_GARGOYLES_ONLY),
    base_physical_resistance: 6,
    base_fire_resistance: 6,
    base_cold_resistance: 4,
    base_poison_resistance: 8,
    base_energy_resistance: 6,
    init_min_hits: 40,
    init_max_hits: 50,
    aos_str_req: 40,
    old_str_req: 40,
    armor_base: 16,
    material_type: ArmorMaterialType::Studded,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::Half,
}

armor_leaf! {
    /// `Server.Items.GargishStuddedKiltType2`.
    GARGISH_STUDDED_KILT_TYPE2, GARGISH_STUDDED_KILT_TYPE2_CLASS, gargish_studded_kilt_type2_new = "Server.Items.GargishStuddedKiltType2" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x287), encoded false;
    default_weight: Some(10.0),
    required_races: Some(ALLOW_GARGOYLES_ONLY),
    base_physical_resistance: 6,
    base_fire_resistance: 6,
    base_cold_resistance: 4,
    base_poison_resistance: 8,
    base_energy_resistance: 6,
    init_min_hits: 40,
    init_max_hits: 50,
    aos_str_req: 40,
    old_str_req: 40,
    armor_base: 16,
    material_type: ArmorMaterialType::Studded,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::Half,
}

armor_leaf! {
    /// `Server.Items.GargishStuddedLegsType1`.
    GARGISH_STUDDED_LEGS_TYPE1, GARGISH_STUDDED_LEGS_TYPE1_CLASS, gargish_studded_legs_type1_new = "Server.Items.GargishStuddedLegsType1" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x28A), encoded false;
    default_weight: Some(15.0),
    required_races: Some(ALLOW_GARGOYLES_ONLY),
    base_physical_resistance: 6,
    base_fire_resistance: 6,
    base_cold_resistance: 4,
    base_poison_resistance: 8,
    base_energy_resistance: 6,
    init_min_hits: 40,
    init_max_hits: 50,
    aos_str_req: 40,
    old_str_req: 40,
    armor_base: 16,
    material_type: ArmorMaterialType::Studded,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::Half,
}

armor_leaf! {
    /// `Server.Items.GargishStuddedLegsType2`.
    GARGISH_STUDDED_LEGS_TYPE2, GARGISH_STUDDED_LEGS_TYPE2_CLASS, gargish_studded_legs_type2_new = "Server.Items.GargishStuddedLegsType2" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x289), encoded false;
    default_weight: Some(15.0),
    required_races: Some(ALLOW_GARGOYLES_ONLY),
    base_physical_resistance: 6,
    base_fire_resistance: 6,
    base_cold_resistance: 4,
    base_poison_resistance: 8,
    base_energy_resistance: 6,
    init_min_hits: 40,
    init_max_hits: 50,
    aos_str_req: 40,
    old_str_req: 40,
    armor_base: 16,
    material_type: ArmorMaterialType::Studded,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::Half,
}

armor_leaf! {
    /// `Server.Items.HideChest`.
    HIDE_CHEST, HIDE_CHEST_CLASS, hide_chest_new = "Server.Items.HideChest" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x2B74), encoded true;
    flippable: Some((0x2B74, 0x316B)),
    default_weight: Some(6.0),
    required_races: Some(ALLOW_ELVES_ONLY),
    base_physical_resistance: 3,
    base_fire_resistance: 3,
    base_cold_resistance: 4,
    base_poison_resistance: 3,
    base_energy_resistance: 2,
    init_min_hits: 35,
    init_max_hits: 45,
    aos_str_req: 25,
    old_str_req: 25,
    armor_base: 15,
    material_type: ArmorMaterialType::Studded,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::Half,
}

armor_leaf! {
    /// `Server.Items.HideFemaleChest`.
    HIDE_FEMALE_CHEST, HIDE_FEMALE_CHEST_CLASS, hide_female_chest_new = "Server.Items.HideFemaleChest" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x2B79), encoded true;
    flippable: Some((0x2B79, 0x3170)),
    default_weight: Some(6.0),
    required_races: Some(ALLOW_ELVES_ONLY),
    base_physical_resistance: 3,
    base_fire_resistance: 3,
    base_cold_resistance: 4,
    base_poison_resistance: 3,
    base_energy_resistance: 2,
    init_min_hits: 35,
    init_max_hits: 45,
    aos_str_req: 35,
    old_str_req: 35,
    armor_base: 15,
    material_type: ArmorMaterialType::Studded,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::Half,
    allow_male_wearer: false,
}

armor_leaf! {
    /// `Server.Items.HideGloves`.
    HIDE_GLOVES, HIDE_GLOVES_CLASS, hide_gloves_new = "Server.Items.HideGloves" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x2B75), encoded true;
    flippable: Some((0x2B75, 0x316C)),
    default_weight: Some(2.0),
    required_races: Some(ALLOW_ELVES_ONLY),
    base_physical_resistance: 3,
    base_fire_resistance: 3,
    base_cold_resistance: 4,
    base_poison_resistance: 3,
    base_energy_resistance: 2,
    init_min_hits: 35,
    init_max_hits: 45,
    aos_str_req: 15,
    old_str_req: 15,
    armor_base: 15,
    material_type: ArmorMaterialType::Studded,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::Half,
}

armor_leaf! {
    /// `Server.Items.HideGorget`.
    HIDE_GORGET, HIDE_GORGET_CLASS, hide_gorget_new = "Server.Items.HideGorget" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x2B76), encoded false;
    flippable: Some((0x2B76, 0x316D)),
    default_weight: Some(3.0),
    required_races: Some(ALLOW_ELVES_ONLY),
    base_physical_resistance: 3,
    base_fire_resistance: 3,
    base_cold_resistance: 4,
    base_poison_resistance: 3,
    base_energy_resistance: 2,
    init_min_hits: 35,
    init_max_hits: 45,
    aos_str_req: 15,
    old_str_req: 15,
    armor_base: 15,
    material_type: ArmorMaterialType::Studded,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::Half,
}

armor_leaf! {
    /// `Server.Items.HidePants`.
    HIDE_PANTS, HIDE_PANTS_CLASS, hide_pants_new = "Server.Items.HidePants" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x2B78), encoded true;
    flippable: Some((0x2B78, 0x316F)),
    default_weight: Some(5.0),
    required_races: Some(ALLOW_ELVES_ONLY),
    base_physical_resistance: 3,
    base_fire_resistance: 3,
    base_cold_resistance: 4,
    base_poison_resistance: 3,
    base_energy_resistance: 2,
    init_min_hits: 35,
    init_max_hits: 45,
    aos_str_req: 25,
    old_str_req: 25,
    armor_base: 15,
    material_type: ArmorMaterialType::Studded,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::Half,
}

armor_leaf! {
    /// `Server.Items.HidePauldrons`.
    HIDE_PAULDRONS, HIDE_PAULDRONS_CLASS, hide_pauldrons_new = "Server.Items.HidePauldrons" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x2B77), encoded true;
    flippable: Some((0x2B77, 0x316E)),
    default_weight: Some(4.0),
    required_races: Some(ALLOW_ELVES_ONLY),
    base_physical_resistance: 3,
    base_fire_resistance: 3,
    base_cold_resistance: 4,
    base_poison_resistance: 3,
    base_energy_resistance: 2,
    init_min_hits: 35,
    init_max_hits: 45,
    aos_str_req: 20,
    old_str_req: 20,
    armor_base: 15,
    material_type: ArmorMaterialType::Studded,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::Half,
}

armor_leaf! {
    /// `Server.Items.StuddedArms`.
    STUDDED_ARMS, STUDDED_ARMS_CLASS, studded_arms_new = "Server.Items.StuddedArms" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x13DC), encoded false;
    flippable: Some((0x13dc, 0x13d4)),
    default_weight: Some(4.0),
    base_physical_resistance: 2,
    base_fire_resistance: 4,
    base_cold_resistance: 3,
    base_poison_resistance: 3,
    base_energy_resistance: 4,
    init_min_hits: 35,
    init_max_hits: 45,
    aos_str_req: 25,
    old_str_req: 25,
    armor_base: 16,
    material_type: ArmorMaterialType::Studded,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::Half,
}

armor_leaf! {
    /// `Server.Items.StuddedBustierArms`.
    STUDDED_BUSTIER_ARMS, STUDDED_BUSTIER_ARMS_CLASS, studded_bustier_arms_new = "Server.Items.StuddedBustierArms" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x1C0C), encoded false;
    flippable: Some((0x1c0c, 0x1c0d)),
    default_weight: Some(1.0),
    base_physical_resistance: 2,
    base_fire_resistance: 4,
    base_cold_resistance: 3,
    base_poison_resistance: 3,
    base_energy_resistance: 4,
    init_min_hits: 35,
    init_max_hits: 45,
    aos_str_req: 35,
    old_str_req: 35,
    armor_base: 16,
    material_type: ArmorMaterialType::Studded,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::Half,
    allow_male_wearer: false,
}

armor_leaf! {
    /// `Server.Items.StuddedChest`.
    STUDDED_CHEST, STUDDED_CHEST_CLASS, studded_chest_new = "Server.Items.StuddedChest" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x13DB), encoded false;
    flippable: Some((0x13db, 0x13e2)),
    default_weight: Some(8.0),
    base_physical_resistance: 2,
    base_fire_resistance: 4,
    base_cold_resistance: 3,
    base_poison_resistance: 3,
    base_energy_resistance: 4,
    init_min_hits: 35,
    init_max_hits: 45,
    aos_str_req: 35,
    old_str_req: 35,
    armor_base: 16,
    material_type: ArmorMaterialType::Studded,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::Half,
}

armor_leaf! {
    /// `Server.Items.StuddedDo`.
    STUDDED_DO, STUDDED_DO_CLASS, studded_do_new = "Server.Items.StuddedDo" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x27C7), encoded false;
    default_weight: Some(8.0),
    base_physical_resistance: 2,
    base_fire_resistance: 4,
    base_cold_resistance: 3,
    base_poison_resistance: 3,
    base_energy_resistance: 4,
    init_min_hits: 40,
    init_max_hits: 50,
    aos_str_req: 55,
    old_str_req: 55,
    armor_base: 3,
    material_type: ArmorMaterialType::Studded,
    default_resource: CraftResource::RegularLeather,
}

armor_leaf! {
    /// `Server.Items.StuddedGloves`.
    STUDDED_GLOVES, STUDDED_GLOVES_CLASS, studded_gloves_new = "Server.Items.StuddedGloves" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x13D5), encoded false;
    flippable: Some((0x13d5, 0x13dd)),
    default_weight: Some(1.0),
    base_physical_resistance: 2,
    base_fire_resistance: 4,
    base_cold_resistance: 3,
    base_poison_resistance: 3,
    base_energy_resistance: 4,
    init_min_hits: 35,
    init_max_hits: 45,
    aos_str_req: 25,
    old_str_req: 25,
    armor_base: 16,
    material_type: ArmorMaterialType::Studded,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::Half,
}

armor_leaf! {
    /// `Server.Items.StuddedGorget`.
    STUDDED_GORGET, STUDDED_GORGET_CLASS, studded_gorget_new = "Server.Items.StuddedGorget" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x13D6), encoded false;
    default_weight: Some(1.0),
    base_physical_resistance: 2,
    base_fire_resistance: 4,
    base_cold_resistance: 3,
    base_poison_resistance: 3,
    base_energy_resistance: 4,
    init_min_hits: 35,
    init_max_hits: 45,
    aos_str_req: 25,
    old_str_req: 25,
    armor_base: 16,
    material_type: ArmorMaterialType::Studded,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::Half,
}

armor_leaf! {
    /// `Server.Items.StuddedHaidate`.
    STUDDED_HAIDATE, STUDDED_HAIDATE_CLASS, studded_haidate_new = "Server.Items.StuddedHaidate" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x278B), encoded false;
    default_weight: Some(5.0),
    base_physical_resistance: 2,
    base_fire_resistance: 4,
    base_cold_resistance: 3,
    base_poison_resistance: 3,
    base_energy_resistance: 4,
    init_min_hits: 35,
    init_max_hits: 45,
    aos_str_req: 30,
    old_str_req: 30,
    armor_base: 3,
    material_type: ArmorMaterialType::Studded,
    default_resource: CraftResource::RegularLeather,
}

armor_leaf! {
    /// `Server.Items.StuddedHiroSode`.
    STUDDED_HIRO_SODE, STUDDED_HIRO_SODE_CLASS, studded_hiro_sode_new = "Server.Items.StuddedHiroSode" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x277F), encoded false;
    default_weight: Some(1.0),
    base_physical_resistance: 2,
    base_fire_resistance: 4,
    base_cold_resistance: 3,
    base_poison_resistance: 3,
    base_energy_resistance: 4,
    init_min_hits: 45,
    init_max_hits: 55,
    aos_str_req: 30,
    old_str_req: 30,
    armor_base: 3,
    material_type: ArmorMaterialType::Studded,
    default_resource: CraftResource::RegularLeather,
}

armor_leaf! {
    /// `Server.Items.StuddedLegs`.
    STUDDED_LEGS, STUDDED_LEGS_CLASS, studded_legs_new = "Server.Items.StuddedLegs" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x13DA), encoded false;
    flippable: Some((0x13da, 0x13e1)),
    default_weight: Some(5.0),
    base_physical_resistance: 2,
    base_fire_resistance: 4,
    base_cold_resistance: 3,
    base_poison_resistance: 3,
    base_energy_resistance: 4,
    init_min_hits: 35,
    init_max_hits: 45,
    aos_str_req: 30,
    old_str_req: 35,
    armor_base: 16,
    material_type: ArmorMaterialType::Studded,
    default_resource: CraftResource::RegularLeather,
    def_med_allowance: ArmorMeditationAllowance::Half,
}

armor_leaf! {
    /// `Server.Items.StuddedMempo`.
    STUDDED_MEMPO, STUDDED_MEMPO_CLASS, studded_mempo_new = "Server.Items.StuddedMempo" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x279D), encoded false;
    default_weight: Some(2.0),
    base_physical_resistance: 2,
    base_fire_resistance: 4,
    base_cold_resistance: 3,
    base_poison_resistance: 3,
    base_energy_resistance: 3,
    init_min_hits: 30,
    init_max_hits: 40,
    aos_str_req: 30,
    old_str_req: 30,
    armor_base: 3,
    material_type: ArmorMaterialType::Studded,
    default_resource: CraftResource::RegularLeather,
}

armor_leaf! {
    /// `Server.Items.StuddedSuneate`.
    STUDDED_SUNEATE, STUDDED_SUNEATE_CLASS, studded_suneate_new = "Server.Items.StuddedSuneate" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x27D2), encoded false;
    default_weight: Some(5.0),
    base_physical_resistance: 2,
    base_fire_resistance: 4,
    base_cold_resistance: 3,
    base_poison_resistance: 3,
    base_energy_resistance: 4,
    init_min_hits: 35,
    init_max_hits: 50,
    aos_str_req: 30,
    old_str_req: 30,
    armor_base: 3,
    material_type: ArmorMaterialType::Studded,
    default_resource: CraftResource::RegularLeather,
}

armor_leaf! {
    /// `Server.Items.Aegis`.
    AEGIS, AEGIS_CLASS, aegis_new = "Server.Items.Aegis" : HEATER_SHIELD / HEATER_SHIELD_CLASS, heater_shield_new, encoded false;
    @init |w, id| {
        hue(w, id, 0x47E);
        set_attr(w, id, AttrKind::Armor, AosArmorAttribute::SelfRepair as u32, 5);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::ReflectPhysical as u32, 15);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::DefendChance as u32, 15);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::LowerManaCost as u32, 8);
    };
    label_number: 1061602,
    artifact_rarity: 11,
    base_physical_resistance: 15,
    init_min_hits: 255,
    init_max_hits: 255,
}

armor_leaf! {
    /// `Server.Items.ArcaneShield`.
    ARCANE_SHIELD, ARCANE_SHIELD_CLASS, arcane_shield_new = "Server.Items.ArcaneShield" : WOODEN_KITE_SHIELD / WOODEN_KITE_SHIELD_CLASS, wooden_kite_shield_new, encoded false;
    @init |w, id| {
        item_id(w, id, 0x1B78);
        hue(w, id, 0x556);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::NightSight as u32, 1);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::SpellChanneling as u32, 1);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::DefendChance as u32, 15);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::CastSpeed as u32, 1);
    };
    label_number: 1061101,
    artifact_rarity: 11,
    init_min_hits: 255,
    init_max_hits: 255,
}

armor_leaf! {
    /// `Server.Items.BronzeShield`.
    BRONZE_SHIELD, BRONZE_SHIELD_CLASS, bronze_shield_new = "Server.Items.BronzeShield" : BASE_SHIELD / BASE_SHIELD_CLASS, |w, ty| new_armor(w, ty, 0x1B72), encoded false;
    default_weight: Some(6.0),
    base_physical_resistance: 0,
    base_fire_resistance: 0,
    base_cold_resistance: 1,
    base_poison_resistance: 0,
    base_energy_resistance: 0,
    init_min_hits: 25,
    init_max_hits: 30,
    aos_str_req: 35,
    armor_base: 10,
}

armor_leaf! {
    /// `Server.Items.Buckler`.
    BUCKLER, BUCKLER_CLASS, buckler_new = "Server.Items.Buckler" : BASE_SHIELD / BASE_SHIELD_CLASS, |w, ty| new_armor(w, ty, 0x1B73), encoded false;
    default_weight: Some(5.0),
    base_physical_resistance: 0,
    base_fire_resistance: 0,
    base_cold_resistance: 0,
    base_poison_resistance: 1,
    base_energy_resistance: 0,
    init_min_hits: 40,
    init_max_hits: 50,
    aos_str_req: 20,
    armor_base: 7,
}

armor_leaf! {
    /// `Server.Items.GargishWoodenShield`.
    GARGISH_WOODEN_SHIELD, GARGISH_WOODEN_SHIELD_CLASS, gargish_wooden_shield_new = "Server.Items.GargishWoodenShield" : BASE_SHIELD / BASE_SHIELD_CLASS, |w, ty| new_armor(w, ty, 0x4200), encoded true;
    flippable: Some((0x4200, 0x4207)),
    default_weight: Some(5.0),
    base_physical_resistance: 0,
    base_fire_resistance: 0,
    base_cold_resistance: 0,
    base_poison_resistance: 0,
    base_energy_resistance: 1,
    init_min_hits: 20,
    init_max_hits: 25,
    aos_str_req: 20,
    required_races: Some(ALLOW_GARGOYLES_ONLY),
    material_type: ArmorMaterialType::Wood,
}

armor_leaf! {
    /// `Server.Items.HeaterShield`.
    HEATER_SHIELD, HEATER_SHIELD_CLASS, heater_shield_new = "Server.Items.HeaterShield" : BASE_SHIELD / BASE_SHIELD_CLASS, |w, ty| new_armor(w, ty, 0x1B76), encoded false;
    default_weight: Some(8.0),
    base_physical_resistance: 0,
    base_fire_resistance: 1,
    base_cold_resistance: 0,
    base_poison_resistance: 0,
    base_energy_resistance: 0,
    init_min_hits: 50,
    init_max_hits: 65,
    aos_str_req: 90,
    armor_base: 23,
}

armor_leaf! {
    /// `Server.Items.MetalKiteShield`.
    METAL_KITE_SHIELD, METAL_KITE_SHIELD_CLASS, metal_kite_shield_new = "Server.Items.MetalKiteShield" : BASE_SHIELD / BASE_SHIELD_CLASS, |w, ty| new_armor(w, ty, 0x1B74), encoded false;
    default_weight: Some(7.0),
    base_physical_resistance: 0,
    base_fire_resistance: 0,
    base_cold_resistance: 0,
    base_poison_resistance: 0,
    base_energy_resistance: 1,
    init_min_hits: 45,
    init_max_hits: 60,
    aos_str_req: 45,
    armor_base: 16,
}

armor_leaf! {
    /// `Server.Items.MetalShield`.
    METAL_SHIELD, METAL_SHIELD_CLASS, metal_shield_new = "Server.Items.MetalShield" : BASE_SHIELD / BASE_SHIELD_CLASS, |w, ty| new_armor(w, ty, 0x1B7B), encoded false;
    default_weight: Some(6.0),
    base_physical_resistance: 0,
    base_fire_resistance: 1,
    base_cold_resistance: 0,
    base_poison_resistance: 0,
    base_energy_resistance: 0,
    init_min_hits: 50,
    init_max_hits: 65,
    aos_str_req: 45,
    armor_base: 11,
}

armor_leaf! {
    /// `Server.Items.WoodenKiteShield`.
    WOODEN_KITE_SHIELD, WOODEN_KITE_SHIELD_CLASS, wooden_kite_shield_new = "Server.Items.WoodenKiteShield" : BASE_SHIELD / BASE_SHIELD_CLASS, |w, ty| new_armor(w, ty, 0x1B79), encoded false;
    default_weight: Some(5.0),
    base_physical_resistance: 0,
    base_fire_resistance: 0,
    base_cold_resistance: 0,
    base_poison_resistance: 0,
    base_energy_resistance: 1,
    init_min_hits: 50,
    init_max_hits: 65,
    aos_str_req: 20,
    armor_base: 12,
    material_type: ArmorMaterialType::Wood,
}

armor_leaf! {
    /// `Server.Items.WoodenShield`.
    WOODEN_SHIELD, WOODEN_SHIELD_CLASS, wooden_shield_new = "Server.Items.WoodenShield" : BASE_SHIELD / BASE_SHIELD_CLASS, |w, ty| new_armor(w, ty, 0x1B7A), encoded false;
    default_weight: Some(5.0),
    base_physical_resistance: 0,
    base_fire_resistance: 0,
    base_cold_resistance: 0,
    base_poison_resistance: 0,
    base_energy_resistance: 1,
    init_min_hits: 20,
    init_max_hits: 25,
    aos_str_req: 20,
    armor_base: 8,
    material_type: ArmorMaterialType::Wood,
}

/// Every class of this file, for `register` and the tests.
pub static ALL: &[(&ItemType, &[&str])] = &[
    (&ARMOR_OF_FORTUNE, &[]),
    (&BRAMBLE_COAT, &[]),
    (&IRONWOOD_CROWN, &[]),
    (&SONG_WOVEN_MANTLE, &[]),
    (&SPELL_WOVEN_BRITCHES, &[]),
    (&STITCHERS_MITTENS, &[]),
    (&GAUNTLETS_OF_NOBILITY, &[]),
    (&HELM_OF_INSIGHT, &[]),
    (&HOLY_KNIGHTS_BREASTPLATE, &[]),
    (&INQUISITORS_RESOLUTION, &[]),
    (&JACKALS_COLLAR, &[]),
    (&LEGGINGS_OF_BANE, &[]),
    (&MIDNIGHT_BRACERS, &[]),
    (&ORNATE_CROWN_OF_THE_HARROWER, &[]),
    (&SHADOW_DANCER_LEGGINGS, &[]),
    (&TUNIC_OF_FIRE, &[]),
    (&VOICE_OF_THE_FALLEN_KING, &[]),
    (&BONE_ARMS, &[]),
    (&BONE_CHEST, &[]),
    (&BONE_GLOVES, &[]),
    (&BONE_LEGS, &[]),
    (&CHAIN_HATSUBURI, &[]),
    (&CHAIN_LEGS, &[]),
    (
        &GARGISH_CLOTH_ARMS_TYPE1,
        &[
            "Server.Items.GargishClothArms",
            "Server.Items.GargishClothArmsArmor",
        ],
    ),
    (
        &GARGISH_CLOTH_ARMS_TYPE2,
        &[
            "Server.Items.FemaleGargishClothArms",
            "Server.Items.FemaleGargishClothArmsArmor",
        ],
    ),
    (
        &GARGISH_CLOTH_CHEST_TYPE1,
        &[
            "Server.Items.GargishClothChest",
            "Server.Items.GargishClothChestArmor",
        ],
    ),
    (
        &GARGISH_CLOTH_CHEST_TYPE2,
        &[
            "Server.Items.FemaleGargishClothChest",
            "Server.Items.FemaleGargishClothChestArmor",
        ],
    ),
    (
        &GARGISH_CLOTH_KILT_TYPE1,
        &[
            "Server.Items.GargishClothKilt",
            "Server.Items.GargishClothKiltArmor",
        ],
    ),
    (&GARGISH_CLOTH_KILT_TYPE2, &[]),
    (
        &GARGISH_CLOTH_LEGS_TYPE1,
        &[
            "Server.Items.GargishClothLegs",
            "Server.Items.GargishClothLegsArmor",
        ],
    ),
    (
        &GARGISH_CLOTH_LEGS_TYPE2,
        &[
            "Server.Items.FemaleGargishClothLegs",
            "Server.Items.FemaleGargishClothLegsArmor",
        ],
    ),
    (&DAEMON_ARMS, &[]),
    (&DAEMON_CHEST, &[]),
    (&DAEMON_GLOVES, &[]),
    (&DAEMON_LEGS, &[]),
    (&DRAGON_ARMS, &[]),
    (&DRAGON_CHEST, &[]),
    (&DRAGON_GLOVES, &[]),
    (&DRAGON_HELM, &[]),
    (&DRAGON_LEGS, &[]),
    (&ANTHROPOMORPHIST_GLASSES, &[]),
    (&ARTS_GLASSES, &[]),
    (&FOLDED_STEEL_GLASSES, &[]),
    (&LIGHT_OF_WAY_GLASSES, &[]),
    (&LYRICAL_GLASSES, &[]),
    (&MACE_SHIELD_GLASSES, &[]),
    (&MARITIME_GLASSES, &[]),
    (&NECROMANTIC_GLASSES, &[]),
    (&POISONED_GLASSES, &[]),
    (&TRADE_GLASSES, &[]),
    (&TREASURE_TRINKET_GLASSES, &[]),
    (&WIZARDS_GLASSES, &[]),
    (&BASCINET, &[]),
    (&BONE_HELM, &[]),
    (&CHAIN_COIF, &[]),
    (&CIRCLET, &[]),
    (&CLOSE_HELM, &[]),
    (&DAEMON_HELM, &[]),
    (&GEMMED_CIRCLET, &[]),
    (&HELMET, &[]),
    (&LEATHER_CAP, &[]),
    (&NORSE_HELM, &[]),
    (&ORC_HELM, &[]),
    (&PLATE_HELM, &[]),
    (&RAVEN_HELM, &[]),
    (&ROYAL_CIRCLET, &[]),
    (&VULTURE_HELM, &[]),
    (&WINGED_HELM, &[]),
    (&FEMALE_LEAF_CHEST, &[]),
    (&FEMALE_LEATHER_CHEST, &[]),
    (
        &GARGISH_LEATHER_ARMS_TYPE1,
        &["Server.Items.GargishLeatherArms"],
    ),
    (
        &GARGISH_LEATHER_ARMS_TYPE2,
        &["Server.Items.FemaleGargishLeatherArms"],
    ),
    (
        &GARGISH_LEATHER_CHEST_TYPE1,
        &["Server.Items.GargishLeatherChest"],
    ),
    (
        &GARGISH_LEATHER_CHEST_TYPE2,
        &["Server.Items.FemaleGargishLeatherChest"],
    ),
    (
        &GARGISH_LEATHER_KILT_TYPE1,
        &["Server.Items.GargishLeatherKilt"],
    ),
    (
        &GARGISH_LEATHER_KILT_TYPE2,
        &["Server.Items.FemaleGargishLeatherKilt"],
    ),
    (
        &GARGISH_LEATHER_LEGS_TYPE1,
        &["Server.Items.GargishLeatherLegs"],
    ),
    (
        &GARGISH_LEATHER_LEGS_TYPE2,
        &["Server.Items.FemaleGargishLeatherLegs"],
    ),
    (&GARGISH_LEATHER_WING_ARMOR, &[]),
    (&LEAF_ARMS, &[]),
    (&LEAF_CHEST, &[]),
    (&LEAF_GORGET, &[]),
    (&LEAF_LEGS, &[]),
    (&LEAF_TONLET, &[]),
    (&LEATHER_ARMS, &[]),
    (&LEATHER_BUSTIER_ARMS, &[]),
    (&LEATHER_DO, &[]),
    (&LEATHER_GORGET, &[]),
    (&LEATHER_HAIDATE, &[]),
    (&LEATHER_HIRO_SODE, &[]),
    (&LEATHER_JINGASA, &[]),
    (&LEATHER_LEGS, &[]),
    (&LEATHER_MEMPO, &[]),
    (&LEATHER_NINJA_HOOD, &[]),
    (&LEATHER_NINJA_JACKET, &[]),
    (&LEATHER_NINJA_MITTS, &[]),
    (&LEATHER_NINJA_PANTS, &[]),
    (&LEATHER_SHORTS, &[]),
    (&LEATHER_SKIRT, &[]),
    (&LEATHER_SUNEATE, &[]),
    (&DECORATIVE_PLATE_KABUTO, &[]),
    (&FEMALE_PLATE_CHEST, &[]),
    (&FEMALE_ELVEN_PLATE_CHEST, &[]),
    (&HEAVY_PLATE_JINGASA, &[]),
    (&LIGHT_PLATE_JINGASA, &[]),
    (&PLATE_ARMS, &[]),
    (&PLATE_BATTLE_KABUTO, &[]),
    (&PLATE_DO, &[]),
    (&PLATE_GLOVES, &[]),
    (&PLATE_GORGET, &[]),
    (&PLATE_HAIDATE, &[]),
    (&PLATE_HATSUBURI, &[]),
    (&PLATE_HIRO_SODE, &[]),
    (&PLATE_LEGS, &[]),
    (&PLATE_MEMPO, &[]),
    (&PLATE_SUNEATE, &[]),
    (&SMALL_PLATE_JINGASA, &[]),
    (&STANDARD_PLATE_KABUTO, &[]),
    (&WOODLAND_ARMS, &[]),
    (&WOODLAND_CHEST, &[]),
    (&WOODLAND_GLOVES, &[]),
    (&WOODLAND_GORGET, &[]),
    (&WOODLAND_LEGS, &[]),
    (&RANGER_ARMS, &[]),
    (&RANGER_CHEST, &[]),
    (&RANGER_GLOVES, &[]),
    (&RANGER_GORGET, &[]),
    (&RANGER_LEGS, &[]),
    (&RINGMAIL_ARMS, &[]),
    (&RINGMAIL_CHEST, &[]),
    (&RINGMAIL_GLOVES, &[]),
    (&RINGMAIL_LEGS, &[]),
    (&SPIRITUALITY_HELM, &[]),
    (&VALOR_GAUNTLETS, &[]),
    (
        &GARGISH_STONE_ARMS_TYPE1,
        &["Server.Items.GargishStoneArms"],
    ),
    (
        &GARGISH_STONE_ARMS_TYPE2,
        &["Server.Items.FemaleGargishStoneArms"],
    ),
    (
        &GARGISH_STONE_CHEST_TYPE1,
        &["Server.Items.GargishStoneChest"],
    ),
    (
        &GARGISH_STONE_CHEST_TYPE2,
        &["Server.Items.FemaleGargishStoneChest"],
    ),
    (
        &GARGISH_STONE_KILT_TYPE1,
        &["Server.Items.GargishStoneKilt"],
    ),
    (
        &GARGISH_STONE_KILT_TYPE2,
        &["Server.Items.FemaleGargishStoneKilt"],
    ),
    (
        &GARGISH_STONE_LEGS_TYPE1,
        &["Server.Items.GargishStoneLegs"],
    ),
    (
        &GARGISH_STONE_LEGS_TYPE2,
        &["Server.Items.FemaleGargishStoneLegs"],
    ),
    (&FEMALE_STUDDED_CHEST, &[]),
    (
        &GARGISH_STUDDED_ARMS_TYPE1,
        &["Server.Items.GargishStuddedArms"],
    ),
    (
        &GARGISH_STUDDED_ARMS_TYPE2,
        &["Server.Items.FemaleGargishStuddedArms"],
    ),
    (
        &GARGISH_STUDDED_CHEST_TYPE1,
        &["Server.Items.GargishStuddedChest"],
    ),
    (
        &GARGISH_STUDDED_CHEST_TYPE2,
        &["Server.Items.FemaleGargishStuddedChest"],
    ),
    (
        &GARGISH_STUDDED_KILT_TYPE1,
        &["Server.Items.GargishStuddedKilt"],
    ),
    (
        &GARGISH_STUDDED_KILT_TYPE2,
        &["Server.Items.FemaleGargishStuddedKilt"],
    ),
    (
        &GARGISH_STUDDED_LEGS_TYPE1,
        &["Server.Items.GargishStuddedLegs"],
    ),
    (
        &GARGISH_STUDDED_LEGS_TYPE2,
        &["Server.Items.FemaleGargishStuddedLegs"],
    ),
    (&HIDE_CHEST, &[]),
    (&HIDE_FEMALE_CHEST, &[]),
    (&HIDE_GLOVES, &[]),
    (&HIDE_GORGET, &[]),
    (&HIDE_PANTS, &[]),
    (&HIDE_PAULDRONS, &[]),
    (&STUDDED_ARMS, &[]),
    (&STUDDED_BUSTIER_ARMS, &[]),
    (&STUDDED_CHEST, &[]),
    (&STUDDED_DO, &[]),
    (&STUDDED_GLOVES, &[]),
    (&STUDDED_GORGET, &[]),
    (&STUDDED_HAIDATE, &[]),
    (&STUDDED_HIRO_SODE, &[]),
    (&STUDDED_LEGS, &[]),
    (&STUDDED_MEMPO, &[]),
    (&STUDDED_SUNEATE, &[]),
    (&AEGIS, &[]),
    (&ARCANE_SHIELD, &[]),
    (&BRONZE_SHIELD, &[]),
    (&BUCKLER, &[]),
    (&GARGISH_WOODEN_SHIELD, &[]),
    (&HEATER_SHIELD, &[]),
    (&METAL_KITE_SHIELD, &[]),
    (&METAL_SHIELD, &[]),
    (&WOODEN_KITE_SHIELD, &[]),
    (&WOODEN_SHIELD, &[]),
];

/// Registers the classes of this file.
pub fn register(types: &mut TypeRegistry) {
    for (t, aliases) in ALL {
        types.register_item(t, aliases);
    }
}
