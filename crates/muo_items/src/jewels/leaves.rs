//! The concrete jewel classes that are tables. Classes with logic of their own are in `special.rs`.

use muo_server::item::ItemType;
use muo_server::type_registry::TypeRegistry;

use super::special::*;
use muo_content::items::jewel::{
    BASE_BRACELET, BASE_BRACELET_CLASS, BASE_EARRINGS, BASE_EARRINGS_CLASS, BASE_NECKLACE,
    BASE_NECKLACE_CLASS, BASE_RING, BASE_RING_CLASS,
};
use muo_content::items::leaves::{GOLD_RING, GOLD_RING_CLASS};

jewel_leaf! {
    /// `Server.Items.BraceletOfHealth`.
    BRACELET_OF_HEALTH, BRACELET_OF_HEALTH_CLASS, bracelet_of_health_new = "Server.Items.BraceletOfHealth" : GOLD_BRACELET / GOLD_BRACELET_CLASS, gold_bracelet_new, encoded false;
    @template;
}

jewel_leaf! {
    /// `Server.Items.EssenceOfBattle`.
    ESSENCE_OF_BATTLE, ESSENCE_OF_BATTLE_CLASS, essence_of_battle_new = "Server.Items.EssenceOfBattle" : GOLD_RING / GOLD_RING_CLASS, gold_ring_new, encoded true;
    @template;
}

jewel_leaf! {
    /// `Server.Items.PendantOfTheMagi`.
    PENDANT_OF_THE_MAGI, PENDANT_OF_THE_MAGI_CLASS, pendant_of_the_magi_new = "Server.Items.PendantOfTheMagi" : GOLD_NECKLACE / GOLD_NECKLACE_CLASS, gold_necklace_new, encoded true;
    @template;
}

jewel_leaf! {
    /// `Server.Items.ResilientBracer`.
    RESILIENT_BRACER, RESILIENT_BRACER_CLASS, resilient_bracer_new = "Server.Items.ResilientBracer" : GOLD_BRACELET / GOLD_BRACELET_CLASS, gold_bracelet_new, encoded true;
    @template;
    @ty {
        physical_resistance: |_, _| 20,
    };
}

jewel_leaf! {
    /// `Server.Items.OrnamentOfTheMagician`.
    ORNAMENT_OF_THE_MAGICIAN, ORNAMENT_OF_THE_MAGICIAN_CLASS, ornament_of_the_magician_new = "Server.Items.OrnamentOfTheMagician" : GOLD_BRACELET / GOLD_BRACELET_CLASS, gold_bracelet_new, encoded false;
    @template;
}

jewel_leaf! {
    /// `Server.Items.RingOfTheElements`.
    RING_OF_THE_ELEMENTS, RING_OF_THE_ELEMENTS_CLASS, ring_of_the_elements_new = "Server.Items.RingOfTheElements" : GOLD_RING / GOLD_RING_CLASS, gold_ring_new, encoded false;
    @template;
}

jewel_leaf! {
    /// `Server.Items.RingOfTheVile`.
    RING_OF_THE_VILE, RING_OF_THE_VILE_CLASS, ring_of_the_vile_new = "Server.Items.RingOfTheVile" : GOLD_RING / GOLD_RING_CLASS, gold_ring_new, encoded false;
    @template;
}

jewel_leaf! {
    /// `Server.Items.GoldBracelet`.
    GOLD_BRACELET, GOLD_BRACELET_CLASS, gold_bracelet_new = "Server.Items.GoldBracelet" : BASE_BRACELET / BASE_BRACELET_CLASS, crate::jewels::new_jewel_json, encoded false;
}

jewel_leaf! {
    /// `Server.Items.SilverBracelet`.
    SILVER_BRACELET, SILVER_BRACELET_CLASS, silver_bracelet_new = "Server.Items.SilverBracelet" : BASE_BRACELET / BASE_BRACELET_CLASS, crate::jewels::new_jewel_json, encoded false;
}

jewel_leaf! {
    /// `Server.Items.GoldEarrings`.
    GOLD_EARRINGS, GOLD_EARRINGS_CLASS, gold_earrings_new = "Server.Items.GoldEarrings" : BASE_EARRINGS / BASE_EARRINGS_CLASS, crate::jewels::new_jewel_json, encoded false;
}

jewel_leaf! {
    /// `Server.Items.SilverEarrings`.
    SILVER_EARRINGS, SILVER_EARRINGS_CLASS, silver_earrings_new = "Server.Items.SilverEarrings" : BASE_EARRINGS / BASE_EARRINGS_CLASS, crate::jewels::new_jewel_json, encoded false;
}

jewel_leaf! {
    /// `Server.Items.GoldNecklace`.
    GOLD_NECKLACE, GOLD_NECKLACE_CLASS, gold_necklace_new = "Server.Items.GoldNecklace" : BASE_NECKLACE / BASE_NECKLACE_CLASS, crate::jewels::new_jewel_json, encoded false;
}

jewel_leaf! {
    /// `Server.Items.GoldBeadNecklace`.
    GOLD_BEAD_NECKLACE, GOLD_BEAD_NECKLACE_CLASS, gold_bead_necklace_new = "Server.Items.GoldBeadNecklace" : BASE_NECKLACE / BASE_NECKLACE_CLASS, crate::jewels::new_jewel_json, encoded false;
}

jewel_leaf! {
    /// `Server.Items.SilverNecklace`.
    SILVER_NECKLACE, SILVER_NECKLACE_CLASS, silver_necklace_new = "Server.Items.SilverNecklace" : BASE_NECKLACE / BASE_NECKLACE_CLASS, crate::jewels::new_jewel_json, encoded false;
}

jewel_leaf! {
    /// `Server.Items.SilverBeadNecklace`.
    SILVER_BEAD_NECKLACE, SILVER_BEAD_NECKLACE_CLASS, silver_bead_necklace_new = "Server.Items.SilverBeadNecklace" : BASE_NECKLACE / BASE_NECKLACE_CLASS, crate::jewels::new_jewel_json, encoded false;
}

jewel_leaf! {
    /// `Server.Items.SilverRing`.
    SILVER_RING, SILVER_RING_CLASS, silver_ring_new = "Server.Items.SilverRing" : BASE_RING / BASE_RING_CLASS, crate::jewels::new_jewel_json, encoded false;
}

jewel_leaf! {
    /// `Server.Items.WeddingRing`.
    WEDDING_RING, WEDDING_RING_CLASS, wedding_ring_new = "Server.Items.WeddingRing" : BASE_RING / BASE_RING_CLASS, crate::jewels::new_jewel_json, encoded false;
    @ty {
        default_name: |_, _| Some("wedding ring".to_string()),
    };
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
