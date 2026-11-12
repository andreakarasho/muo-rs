//! The concrete jewel classes that are tables. Classes with logic of their own are in `special.rs`.

use muo_content::aos::{AosAttribute, AosElementAttribute, AttrKind};
use muo_content::items::set_attr;
use muo_server::item::ItemType;
use muo_server::skills::SkillName;
use muo_server::type_registry::TypeRegistry;

use super::special::*;
use crate::armor::{hue, skill_bonus};
use muo_content::items::jewel::{
    BASE_BRACELET, BASE_BRACELET_CLASS, BASE_EARRINGS, BASE_EARRINGS_CLASS, BASE_NECKLACE,
    BASE_NECKLACE_CLASS, BASE_RING, BASE_RING_CLASS, new_jewel,
};
use muo_content::items::leaves::{GOLD_RING, GOLD_RING_CLASS};

jewel_leaf! {
    /// `Server.Items.BraceletOfHealth`.
    BRACELET_OF_HEALTH, BRACELET_OF_HEALTH_CLASS, bracelet_of_health_new = "Server.Items.BraceletOfHealth" : GOLD_BRACELET / GOLD_BRACELET_CLASS, gold_bracelet_new, encoded false;
    @init |w, id| {
        hue(w, id, 0x21);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::BonusHits as u32, 5);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::RegenHits as u32, 10);
    };
    label_number: 1061103,
    artifact_rarity: 11,
}

jewel_leaf! {
    /// `Server.Items.EssenceOfBattle`.
    ESSENCE_OF_BATTLE, ESSENCE_OF_BATTLE_CLASS, essence_of_battle_new = "Server.Items.EssenceOfBattle" : GOLD_RING / GOLD_RING_CLASS, gold_ring_new, encoded true;
    @init |w, id| {
        hue(w, id, 0x550);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::BonusDex as u32, 7);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::BonusStr as u32, 7);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponDamage as u32, 30);
    };
    label_number: 1072935,
}

jewel_leaf! {
    /// `Server.Items.PendantOfTheMagi`.
    PENDANT_OF_THE_MAGI, PENDANT_OF_THE_MAGI_CLASS, pendant_of_the_magi_new = "Server.Items.PendantOfTheMagi" : GOLD_NECKLACE / GOLD_NECKLACE_CLASS, gold_necklace_new, encoded true;
    @init |w, id| {
        hue(w, id, 0x48D);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::BonusInt as u32, 10);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::RegenMana as u32, 3);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::SpellDamage as u32, 5);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::LowerManaCost as u32, 10);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::LowerRegCost as u32, 30);
    };
    label_number: 1072937,
}

jewel_leaf! {
    /// `Server.Items.ResilientBracer`.
    RESILIENT_BRACER, RESILIENT_BRACER_CLASS, resilient_bracer_new = "Server.Items.ResilientBracer" : GOLD_BRACELET / GOLD_BRACELET_CLASS, gold_bracelet_new, encoded true;
    @init |w, id| {
        hue(w, id, 0x488);
        skill_bonus(w, id, 0, SkillName::MagicResist, 15.0);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::BonusHits as u32, 5);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::RegenHits as u32, 2);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::DefendChance as u32, 10);
    };
    @ty {
        physical_resistance: |_, _| 20,
    };
    label_number: 1072933,
}

jewel_leaf! {
    /// `Server.Items.OrnamentOfTheMagician`.
    ORNAMENT_OF_THE_MAGICIAN, ORNAMENT_OF_THE_MAGICIAN_CLASS, ornament_of_the_magician_new = "Server.Items.OrnamentOfTheMagician" : GOLD_BRACELET / GOLD_BRACELET_CLASS, gold_bracelet_new, encoded false;
    @init |w, id| {
        hue(w, id, 0x554);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::CastRecovery as u32, 3);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::CastSpeed as u32, 2);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::LowerManaCost as u32, 10);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::LowerRegCost as u32, 20);
        set_attr(w, id, AttrKind::Element, AosElementAttribute::Energy as u32, 15);
    };
    label_number: 1061105,
    artifact_rarity: 11,
}

jewel_leaf! {
    /// `Server.Items.RingOfTheElements`.
    RING_OF_THE_ELEMENTS, RING_OF_THE_ELEMENTS_CLASS, ring_of_the_elements_new = "Server.Items.RingOfTheElements" : GOLD_RING / GOLD_RING_CLASS, gold_ring_new, encoded false;
    @init |w, id| {
        hue(w, id, 0x4E9);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::Luck as u32, 100);
        set_attr(w, id, AttrKind::Element, AosElementAttribute::Fire as u32, 16);
        set_attr(w, id, AttrKind::Element, AosElementAttribute::Cold as u32, 16);
        set_attr(w, id, AttrKind::Element, AosElementAttribute::Poison as u32, 16);
        set_attr(w, id, AttrKind::Element, AosElementAttribute::Energy as u32, 16);
    };
    label_number: 1061104,
    artifact_rarity: 11,
}

jewel_leaf! {
    /// `Server.Items.RingOfTheVile`.
    RING_OF_THE_VILE, RING_OF_THE_VILE_CLASS, ring_of_the_vile_new = "Server.Items.RingOfTheVile" : GOLD_RING / GOLD_RING_CLASS, gold_ring_new, encoded false;
    @init |w, id| {
        hue(w, id, 0x4F7);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::BonusDex as u32, 8);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::RegenStam as u32, 6);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::AttackChance as u32, 15);
        set_attr(w, id, AttrKind::Element, AosElementAttribute::Poison as u32, 20);
    };
    label_number: 1061102,
    artifact_rarity: 11,
}

jewel_leaf! {
    /// `Server.Items.GoldBracelet`.
    GOLD_BRACELET, GOLD_BRACELET_CLASS, gold_bracelet_new = "Server.Items.GoldBracelet" : BASE_BRACELET / BASE_BRACELET_CLASS, |w, ty| new_jewel(w, ty, 0x1086), encoded false;
    default_weight: Some(0.1),
}

jewel_leaf! {
    /// `Server.Items.SilverBracelet`.
    SILVER_BRACELET, SILVER_BRACELET_CLASS, silver_bracelet_new = "Server.Items.SilverBracelet" : BASE_BRACELET / BASE_BRACELET_CLASS, |w, ty| new_jewel(w, ty, 0x1F06), encoded false;
    default_weight: Some(0.1),
}

jewel_leaf! {
    /// `Server.Items.GoldEarrings`.
    GOLD_EARRINGS, GOLD_EARRINGS_CLASS, gold_earrings_new = "Server.Items.GoldEarrings" : BASE_EARRINGS / BASE_EARRINGS_CLASS, |w, ty| new_jewel(w, ty, 0x1087), encoded false;
    default_weight: Some(0.1),
}

jewel_leaf! {
    /// `Server.Items.SilverEarrings`.
    SILVER_EARRINGS, SILVER_EARRINGS_CLASS, silver_earrings_new = "Server.Items.SilverEarrings" : BASE_EARRINGS / BASE_EARRINGS_CLASS, |w, ty| new_jewel(w, ty, 0x1F07), encoded false;
    default_weight: Some(0.1),
}

jewel_leaf! {
    /// `Server.Items.GoldNecklace`.
    GOLD_NECKLACE, GOLD_NECKLACE_CLASS, gold_necklace_new = "Server.Items.GoldNecklace" : BASE_NECKLACE / BASE_NECKLACE_CLASS, |w, ty| new_jewel(w, ty, 0x1088), encoded false;
    default_weight: Some(0.1),
}

jewel_leaf! {
    /// `Server.Items.GoldBeadNecklace`.
    GOLD_BEAD_NECKLACE, GOLD_BEAD_NECKLACE_CLASS, gold_bead_necklace_new = "Server.Items.GoldBeadNecklace" : BASE_NECKLACE / BASE_NECKLACE_CLASS, |w, ty| new_jewel(w, ty, 0x1089), encoded false;
    default_weight: Some(0.1),
}

jewel_leaf! {
    /// `Server.Items.SilverNecklace`.
    SILVER_NECKLACE, SILVER_NECKLACE_CLASS, silver_necklace_new = "Server.Items.SilverNecklace" : BASE_NECKLACE / BASE_NECKLACE_CLASS, |w, ty| new_jewel(w, ty, 0x1F08), encoded false;
    default_weight: Some(0.1),
}

jewel_leaf! {
    /// `Server.Items.SilverBeadNecklace`.
    SILVER_BEAD_NECKLACE, SILVER_BEAD_NECKLACE_CLASS, silver_bead_necklace_new = "Server.Items.SilverBeadNecklace" : BASE_NECKLACE / BASE_NECKLACE_CLASS, |w, ty| new_jewel(w, ty, 0x1F05), encoded false;
    default_weight: Some(0.1),
}

jewel_leaf! {
    /// `Server.Items.SilverRing`.
    SILVER_RING, SILVER_RING_CLASS, silver_ring_new = "Server.Items.SilverRing" : BASE_RING / BASE_RING_CLASS, |w, ty| new_jewel(w, ty, 0x1F09), encoded false;
    default_weight: Some(0.1),
}

jewel_leaf! {
    /// `Server.Items.WeddingRing`.
    WEDDING_RING, WEDDING_RING_CLASS, wedding_ring_new = "Server.Items.WeddingRing" : BASE_RING / BASE_RING_CLASS, |w, ty| new_jewel(w, ty, 0x108a), encoded false;
    @ty {
        default_name: |_, _| Some("wedding ring".to_string()),
    };
    default_weight: Some(0.1),
}

/// Every class of this file, for `register` and the tests.
pub static ALL: &[(&ItemType, &[&str])] = &[
    (&BRACELET_OF_HEALTH, &[]),
    (&ESSENCE_OF_BATTLE, &[]),
    (&PENDANT_OF_THE_MAGI, &[]),
    (&RESILIENT_BRACER, &[]),
    (&ORNAMENT_OF_THE_MAGICIAN, &[]),
    (&RING_OF_THE_ELEMENTS, &[]),
    (&RING_OF_THE_VILE, &[]),
    (&GOLD_BRACELET, &[]),
    (&SILVER_BRACELET, &[]),
    (&GOLD_EARRINGS, &[]),
    (&SILVER_EARRINGS, &[]),
    (&GOLD_NECKLACE, &[]),
    (&GOLD_BEAD_NECKLACE, &[]),
    (&SILVER_NECKLACE, &[]),
    (&SILVER_BEAD_NECKLACE, &[]),
    (&SILVER_RING, &[]),
    (&WEDDING_RING, &[]),
];

/// Registers the classes of this file.
pub fn register(types: &mut TypeRegistry) {
    for (t, aliases) in ALL {
        types.register_item(t, aliases);
    }
}
