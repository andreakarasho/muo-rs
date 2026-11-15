//! The New Haven Quest Rewards classes that are plain tables (a constructor and constant overrides); the classes with logic
//! are hand-written next to this file.

use crate::simple::weapon_art;
use muo_content::aos::{AosAttribute, AosWeaponAttribute, AttrKind};
use muo_content::items::set_attr;
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::{ItemType, LootType, ops};
use muo_server::type_registry::TypeRegistry;

use crate::armor::armor_leaf;
use crate::armor::leaves::*;
use crate::armor::special::*;
use crate::clothing::clothing_leaf;
use crate::clothing::leaves::*;
use crate::jewels::jewel_leaf;
use crate::jewels::leaves::*;
use crate::jewels::special::*;
use crate::weapons::*;
use muo_content::items::armor::ArmorClass;
use muo_content::items::leaves::{GOLD_RING, GOLD_RING_CLASS};
use muo_content::items::leaves::{LEATHER_CHEST, LEATHER_CHEST_CLASS};
use muo_content::items::leaves::{LONGSWORD, LONGSWORD_CLASS};

armor_leaf! {
    /// `Server.Items.ArmsOfArmstrong`.
    ARMS_OF_ARMSTRONG, ARMS_OF_ARMSTRONG_CLASS, arms_of_armstrong_new = "Server.Items.ArmsOfArmstrong" : LEATHER_ARMS / LEATHER_ARMS_CLASS, leather_arms_new, encoded true;
    @template;
}

jewel_leaf! {
    /// `Server.Items.BraceletOfResilience`.
    BRACELET_OF_RESILIENCE, BRACELET_OF_RESILIENCE_CLASS, bracelet_of_resilience_new = "Server.Items.BraceletOfResilience" : GOLD_BRACELET / GOLD_BRACELET_CLASS, gold_bracelet_new, encoded true;
    @template;
}

armor_leaf! {
    /// `Server.Items.BulwarkLeggings`.
    BULWARK_LEGGINGS, BULWARK_LEGGINGS_CLASS, bulwark_leggings_new = "Server.Items.BulwarkLeggings" : RINGMAIL_LEGS / RINGMAIL_LEGS_CLASS, ringmail_legs_new, encoded true;
    @template;
}

weapon_art! {
    /// `Server.Items.ChurchillsWarMace`.
    CHURCHILLS_WAR_MACE, CHURCHILLS_WAR_MACE_CLASS, churchills_war_mace_new = "Server.Items.ChurchillsWarMace" : WAR_MACE / WAR_MACE_CLASS, via war_mace_new, enc;
    @json;
}

jewel_leaf! {
    /// `Server.Items.ClaspOfConcentration`.
    CLASP_OF_CONCENTRATION, CLASP_OF_CONCENTRATION_CLASS, clasp_of_concentration_new = "Server.Items.ClaspOfConcentration" : SILVER_BRACELET / SILVER_BRACELET_CLASS, silver_bracelet_new, encoded true;
    @template;
}

weapon_art! {
    /// `Server.Items.EmberStaff`.
    EMBER_STAFF, EMBER_STAFF_CLASS, ember_staff_new = "Server.Items.EmberStaff" : QUARTER_STAFF / QUARTER_STAFF_CLASS, via quarter_staff_new, enc;
    @json;
}

armor_leaf! {
    /// `Server.Items.GlovesOfSafeguarding`.
    GLOVES_OF_SAFEGUARDING, GLOVES_OF_SAFEGUARDING_CLASS, gloves_of_safeguarding_new = "Server.Items.GlovesOfSafeguarding" : LEATHER_GLOVES / LEATHER_GLOVES_CLASS, leather_gloves_new, encoded true;
    @template;
}

armor_leaf! {
    /// `Server.Items.HealersTouch`.
    HEALERS_TOUCH, HEALERS_TOUCH_CLASS, healers_touch_new = "Server.Items.HealersTouch" : LEATHER_GLOVES / LEATHER_GLOVES_CLASS, leather_gloves_new, encoded true;
    @template;
}

weapon_art! {
    /// `Server.Items.Heartseeker`.
    HEARTSEEKER, HEARTSEEKER_CLASS, heartseeker_new = "Server.Items.Heartseeker" : COMPOSITE_BOW / COMPOSITE_BOW_CLASS, via composite_bow_new, enc;
    @json;
}

weapon_art! {
    /// `Server.Items.JocklesQuicksword`.
    JOCKLES_QUICKSWORD, JOCKLES_QUICKSWORD_CLASS, jockles_quicksword_new = "Server.Items.JocklesQuicksword" : LONGSWORD / LONGSWORD_CLASS, via longsword_new, enc;
    @json;
}

clothing_leaf! {
    /// `Server.Items.PhilosophersHat`.
    PHILOSOPHERS_HAT, PHILOSOPHERS_HAT_CLASS, philosophers_hat_new = "Server.Items.PhilosophersHat" : WIZARDS_HAT / WIZARDS_HAT_CLASS, wizards_hat_new, hue 0, encoded true;
    @template;
    base_physical_resistance: 5,
    base_fire_resistance: 5,
    base_cold_resistance: 9,
    base_poison_resistance: 5,
    base_energy_resistance: 5,
}

weapon_art! {
    /// `Server.Items.RecarosRiposte`.
    RECAROS_RIPOSTE, RECAROS_RIPOSTE_CLASS, recaros_riposte_new = "Server.Items.RecarosRiposte" : WAR_FORK / WAR_FORK_CLASS, via war_fork_new, enc;
    @json;
}

jewel_leaf! {
    /// `Server.Items.RingOfTheSavant`.
    RING_OF_THE_SAVANT, RING_OF_THE_SAVANT_CLASS, ring_of_the_savant_new = "Server.Items.RingOfTheSavant" : GOLD_RING / GOLD_RING_CLASS, gold_ring_new, encoded true;
    @template;
}

weapon_art! {
    /// `Server.Items.SilverSerpentBlade`.
    SILVER_SERPENT_BLADE, SILVER_SERPENT_BLADE_CLASS, silver_serpent_blade_new = "Server.Items.SilverSerpentBlade" : KRYSS / KRYSS_CLASS, via kryss_new, enc;
    @json;
}

weapon_art! {
    /// `Server.Items.TheDragonsTail`.
    THE_DRAGONS_TAIL, THE_DRAGONS_TAIL_CLASS, the_dragons_tail_new = "Server.Items.TheDragonsTail" : NO_DACHI / NO_DACHI_CLASS, via no_dachi_new, enc;
    @json;
}

armor_leaf! {
    /// `Server.Items.TunicOfGuarding`.
    TUNIC_OF_GUARDING, TUNIC_OF_GUARDING_CLASS, tunic_of_guarding_new = "Server.Items.TunicOfGuarding" : LEATHER_CHEST / LEATHER_CHEST_CLASS, leather_chest_new, encoded true;
    @template;
}

armor_leaf! {
    /// `Server.Items.TwilightJacket`.
    TWILIGHT_JACKET, TWILIGHT_JACKET_CLASS, twilight_jacket_new = "Server.Items.TwilightJacket" : LEATHER_NINJA_JACKET / LEATHER_NINJA_JACKET_CLASS, leather_ninja_jacket_new, encoded true;
    @template;
}

armor_leaf! {
    /// `Server.Items.WalkersLeggings`.
    WALKERS_LEGGINGS, WALKERS_LEGGINGS_CLASS, walkers_leggings_new = "Server.Items.WalkersLeggings" : LEATHER_NINJA_PANTS / LEATHER_NINJA_PANTS_CLASS, leather_ninja_pants_new, encoded true;
    @template;
}

/// Every class of this file: the type and its `[TypeAlias]` names.
pub static ALL: &[(&ItemType, &[&str])] = &[
    (&ARMS_OF_ARMSTRONG, &[]),
    (&BRACELET_OF_RESILIENCE, &[]),
    (&BULWARK_LEGGINGS, &[]),
    (&CHURCHILLS_WAR_MACE, &[]),
    (&CLASP_OF_CONCENTRATION, &[]),
    (&EMBER_STAFF, &[]),
    (&GLOVES_OF_SAFEGUARDING, &[]),
    (&HEALERS_TOUCH, &[]),
    (&HEARTSEEKER, &[]),
    (&JOCKLES_QUICKSWORD, &[]),
    (&PHILOSOPHERS_HAT, &[]),
    (&RECAROS_RIPOSTE, &[]),
    (&RING_OF_THE_SAVANT, &[]),
    (&SILVER_SERPENT_BLADE, &[]),
    (&THE_DRAGONS_TAIL, &[]),
    (&TUNIC_OF_GUARDING, &[]),
    (&TWILIGHT_JACKET, &[]),
    (&WALKERS_LEGGINGS, &[]),
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
