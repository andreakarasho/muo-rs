//! The Minor Artifacts classes that are plain tables (a constructor and constant overrides); the classes with logic
//! are hand-written next to this file.

use crate::armor::skill_bonus;
use crate::simple::{hue, item_id, leaf, weapon_art};
use muo_content::aos::{AosAttribute, AosWeaponAttribute, AttrKind};
use muo_content::items::{same, set_attr};
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::{ITEM_BASE, ItemType};
use muo_server::skills::SkillName;
use muo_server::type_registry::TypeRegistry;

use crate::armor::armor_leaf;
use crate::armor::leaves::*;
use crate::armor::special::*;
use crate::clothing::clothing_leaf;
use crate::clothing::leaves::*;
use crate::jewels::jewel_leaf;
use crate::jewels::leaves::*;
use crate::weapons::*;
use muo_content::items::armor::ArmorClass;
use muo_content::items::leaves::{BOW, BOW_CLASS};
use muo_content::items::leaves::{PLATE_CHEST, PLATE_CHEST_CLASS};
use muo_content::items::slayer::SlayerName;

jewel_leaf! {
    /// `Server.Items.AlchemistsBauble`.
    ALCHEMISTS_BAUBLE, ALCHEMISTS_BAUBLE_CLASS, alchemists_bauble_new = "Server.Items.AlchemistsBauble" : GOLD_BRACELET / GOLD_BRACELET_CLASS, gold_bracelet_new, encoded false;
    @template;
}

weapon_art! {
    /// `Server.Items.ArcticDeathDealer`.
    ARCTIC_DEATH_DEALER, ARCTIC_DEATH_DEALER_CLASS, arctic_death_dealer_new = "Server.Items.ArcticDeathDealer" : WAR_MACE / WAR_MACE_CLASS, via war_mace_new, v0;
    @json;
}

weapon_art! {
    /// `Server.Items.BlazeOfDeath`.
    BLAZE_OF_DEATH, BLAZE_OF_DEATH_CLASS, blaze_of_death_new = "Server.Items.BlazeOfDeath" : HALBERD / HALBERD_CLASS, via halberd_new, v0;
    @json;
}

weapon_art! {
    /// `Server.Items.BowOfTheJukaKing`.
    BOW_OF_THE_JUKA_KING, BOW_OF_THE_JUKA_KING_CLASS, bow_of_the_juka_king_new = "Server.Items.BowOfTheJukaKing" : BOW / BOW_CLASS, via bow_new, v0;
    @json;
}

/// `CandelabraOfSouls()`.
pub fn candelabra_of_souls_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.CandelabraOfSouls")
}

leaf! {
    /// `Server.Items.CandelabraOfSouls`.
    CANDELABRA_OF_SOULS = "Server.Items.CandelabraOfSouls" : ITEM_BASE, v0;
    construct: Some(|w| candelabra_of_souls_new(w, &CANDELABRA_OF_SOULS)),
}

weapon_art! {
    /// `Server.Items.CaptainQuacklebushsCutlass`.
    CAPTAIN_QUACKLEBUSHS_CUTLASS, CAPTAIN_QUACKLEBUSHS_CUTLASS_CLASS, captain_quacklebushs_cutlass_new = "Server.Items.CaptainQuacklebushsCutlass" : CUTLASS / CUTLASS_CLASS, via cutlass_new, v0;
    @json;
}

weapon_art! {
    /// `Server.Items.CavortingClub`.
    CAVORTING_CLUB, CAVORTING_CLUB_CLASS, cavorting_club_new = "Server.Items.CavortingClub" : CLUB / CLUB_CLASS, via club_new, v0;
    @json;
}

weapon_art! {
    /// `Server.Items.ColdBlood`.
    COLD_BLOOD, COLD_BLOOD_CLASS, cold_blood_new = "Server.Items.ColdBlood" : CLEAVER / CLEAVER_CLASS, via cleaver_new, v0;
    @json;
}

weapon_art! {
    /// `Server.Items.EnchantedTitanLegBone`.
    ENCHANTED_TITAN_LEG_BONE, ENCHANTED_TITAN_LEG_BONE_CLASS, enchanted_titan_leg_bone_new = "Server.Items.EnchantedTitanLegBone" : SHORT_SPEAR / SHORT_SPEAR_CLASS, via short_spear_new, v0;
    @json;
}

/// `GhostShipAnchor()`.
pub fn ghost_ship_anchor_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.GhostShipAnchor")
}

leaf! {
    /// `Server.Items.GhostShipAnchor`.
    GHOST_SHIP_ANCHOR = "Server.Items.GhostShipAnchor" : ITEM_BASE, v0;
    construct: Some(|w| ghost_ship_anchor_new(w, &GHOST_SHIP_ANCHOR)),
    label_number: |_, _| 1070816,
}

armor_leaf! {
    /// `Server.Items.GlovesOfThePugilist`.
    GLOVES_OF_THE_PUGILIST, GLOVES_OF_THE_PUGILIST_CLASS, gloves_of_the_pugilist_new = "Server.Items.GlovesOfThePugilist" : LEATHER_GLOVES / LEATHER_GLOVES_CLASS, leather_gloves_new, encoded false;
    @template;
}

/// `GoldBricks()`.
pub fn gold_bricks_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.GoldBricks")
}

leaf! {
    /// `Server.Items.GoldBricks`.
    GOLD_BRICKS = "Server.Items.GoldBricks" : ITEM_BASE, v0;
    construct: Some(|w| gold_bricks_new(w, &GOLD_BRICKS)),
    label_number: |_, _| 1063489,
}

armor_leaf! {
    /// `Server.Items.HeartOfTheLion`.
    HEART_OF_THE_LION, HEART_OF_THE_LION_CLASS, heart_of_the_lion_new = "Server.Items.HeartOfTheLion" : PLATE_CHEST / PLATE_CHEST_CLASS, plate_chest_new, encoded false;
    @template;
}

weapon_art! {
    /// `Server.Items.LunaLance`.
    LUNA_LANCE, LUNA_LANCE_CLASS, luna_lance_new = "Server.Items.LunaLance" : LANCE / LANCE_CLASS, via lance_new, v0;
    @json;
}

weapon_art! {
    /// `Server.Items.BladeDance`.
    BLADE_DANCE, BLADE_DANCE_CLASS, blade_dance_new = "Server.Items.BladeDance" : RUNE_BLADE / RUNE_BLADE_CLASS, via rune_blade_new, enc;
    @json;
}

weapon_art! {
    /// `Server.Items.Bonesmasher`.
    BONESMASHER, BONESMASHER_CLASS, bonesmasher_new = "Server.Items.Bonesmasher" : DIAMOND_MACE / DIAMOND_MACE_CLASS, via diamond_mace_new, enc;
    @json;
}

weapon_art! {
    /// `Server.Items.Boomstick`.
    BOOMSTICK, BOOMSTICK_CLASS, boomstick_new = "Server.Items.Boomstick" : WILD_STAFF / WILD_STAFF_CLASS, via wild_staff_new, enc;
    @json;
}

armor_leaf! {
    /// `Server.Items.BrightsightLenses`.
    BRIGHTSIGHT_LENSES, BRIGHTSIGHT_LENSES_CLASS, brightsight_lenses_new = "Server.Items.BrightsightLenses" : ELVEN_GLASSES / ELVEN_GLASSES_CLASS, elven_glasses_new, encoded true;
    @template;
}

weapon_art! {
    /// `Server.Items.FleshRipper`.
    FLESH_RIPPER, FLESH_RIPPER_CLASS, flesh_ripper_new = "Server.Items.FleshRipper" : ASSASSIN_SPIKE / ASSASSIN_SPIKE_CLASS, via assassin_spike_new, enc;
    @json;
}

armor_leaf! {
    /// `Server.Items.HelmOfSwiftness`.
    HELM_OF_SWIFTNESS, HELM_OF_SWIFTNESS_CLASS, helm_of_swiftness_new = "Server.Items.HelmOfSwiftness" : WINGED_HELM / WINGED_HELM_CLASS, winged_helm_new, encoded true;
    @template;
}

weapon_art! {
    /// `Server.Items.MelisandesCorrodedHatchet`.
    MELISANDES_CORRODED_HATCHET, MELISANDES_CORRODED_HATCHET_CLASS, melisandes_corroded_hatchet_new = "Server.Items.MelisandesCorrodedHatchet" : HATCHET / HATCHET_CLASS, via hatchet_new, enc;
    @json;
}

weapon_art! {
    /// `Server.Items.RaedsGlory`.
    RAEDS_GLORY, RAEDS_GLORY_CLASS, raeds_glory_new = "Server.Items.RaedsGlory" : WAR_CLEAVER / WAR_CLEAVER_CLASS, via war_cleaver_new, enc;
    @json;
}

weapon_art! {
    /// `Server.Items.RighteousAnger`.
    RIGHTEOUS_ANGER, RIGHTEOUS_ANGER_CLASS, righteous_anger_new = "Server.Items.RighteousAnger" : ELVEN_MACHETE / ELVEN_MACHETE_CLASS, via elven_machete_new, enc;
    @json;
}

weapon_art! {
    /// `Server.Items.SoulSeeker`.
    SOUL_SEEKER, SOUL_SEEKER_CLASS, soul_seeker_new = "Server.Items.SoulSeeker" : RADIANT_SCIMITAR / RADIANT_SCIMITAR_CLASS, via radiant_scimitar_new, enc;
    @json;
}

weapon_art! {
    /// `Server.Items.TalonBite`.
    TALON_BITE, TALON_BITE_CLASS, talon_bite_new = "Server.Items.TalonBite" : ORNATE_AXE / ORNATE_AXE_CLASS, via ornate_axe_new, enc;
    @json;
}

weapon_art! {
    /// `Server.Items.NightsKiss`.
    NIGHTS_KISS, NIGHTS_KISS_CLASS, nights_kiss_new = "Server.Items.NightsKiss" : DAGGER / DAGGER_CLASS, via dagger_new, v0;
    @json;
}

weapon_art! {
    /// `Server.Items.NoxRangersHeavyCrossbow`.
    NOX_RANGERS_HEAVY_CROSSBOW, NOX_RANGERS_HEAVY_CROSSBOW_CLASS, nox_rangers_heavy_crossbow_new = "Server.Items.NoxRangersHeavyCrossbow" : HEAVY_CROSSBOW / HEAVY_CROSSBOW_CLASS, via heavy_crossbow_new, v0;
    @json;
}

armor_leaf! {
    /// `Server.Items.OrcishVisage`.
    ORCISH_VISAGE, ORCISH_VISAGE_CLASS, orcish_visage_new = "Server.Items.OrcishVisage" : ORC_HELM / ORC_HELM_CLASS, orc_helm_new, encoded false;
    @template;
}

weapon_art! {
    /// `Server.Items.PixieSwatter`.
    PIXIE_SWATTER, PIXIE_SWATTER_CLASS, pixie_swatter_new = "Server.Items.PixieSwatter" : SCEPTER / SCEPTER_CLASS, via scepter_new, v0;
    @json;
}

clothing_leaf! {
    /// `Server.Items.PolarBearMask`.
    POLAR_BEAR_MASK, POLAR_BEAR_MASK_CLASS, polar_bear_mask_new = "Server.Items.PolarBearMask" : BEAR_MASK / BEAR_MASK_CLASS, bear_mask_new, hue 0, encoded false;
    @template;
}

/// `ShipModelOfTheHMSCape()`.
pub fn ship_model_of_the_hms_cape_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.ShipModelOfTheHMSCape")
}

leaf! {
    /// `Server.Items.ShipModelOfTheHMSCape`.
    SHIP_MODEL_OF_THE_HMS_CAPE = "Server.Items.ShipModelOfTheHMSCape" : ITEM_BASE, v0;
    construct: Some(|w| ship_model_of_the_hms_cape_new(w, &SHIP_MODEL_OF_THE_HMS_CAPE)),
    label_number: |_, _| 1063476,
}

weapon_art! {
    /// `Server.Items.StaffOfPower`.
    STAFF_OF_POWER, STAFF_OF_POWER_CLASS, staff_of_power_new = "Server.Items.StaffOfPower" : BLACK_STAFF / BLACK_STAFF_CLASS, via black_staff_new, v0;
    @json;
}

armor_leaf! {
    /// `Server.Items.VioletCourage`.
    VIOLET_COURAGE, VIOLET_COURAGE_CLASS, violet_courage_new = "Server.Items.VioletCourage" : FEMALE_PLATE_CHEST / FEMALE_PLATE_CHEST_CLASS, female_plate_chest_new, encoded false;
    @template;
}

/// Every class of this file: the type and its `[TypeAlias]` names.
pub static ALL: &[(&ItemType, &[&str])] = &[
    (&ALCHEMISTS_BAUBLE, &[]),
    (&ARCTIC_DEATH_DEALER, &[]),
    (&BLAZE_OF_DEATH, &[]),
    (&BOW_OF_THE_JUKA_KING, &[]),
    (&CANDELABRA_OF_SOULS, &[]),
    (&CAPTAIN_QUACKLEBUSHS_CUTLASS, &[]),
    (&CAVORTING_CLUB, &[]),
    (&COLD_BLOOD, &[]),
    (&ENCHANTED_TITAN_LEG_BONE, &[]),
    (&GHOST_SHIP_ANCHOR, &[]),
    (&GLOVES_OF_THE_PUGILIST, &[]),
    (&GOLD_BRICKS, &[]),
    (&HEART_OF_THE_LION, &[]),
    (&LUNA_LANCE, &[]),
    (&BLADE_DANCE, &[]),
    (&BONESMASHER, &[]),
    (&BOOMSTICK, &[]),
    (&BRIGHTSIGHT_LENSES, &[]),
    (&FLESH_RIPPER, &[]),
    (&HELM_OF_SWIFTNESS, &[]),
    (&MELISANDES_CORRODED_HATCHET, &[]),
    (&RAEDS_GLORY, &[]),
    (&RIGHTEOUS_ANGER, &[]),
    (&SOUL_SEEKER, &[]),
    (&TALON_BITE, &[]),
    (&NIGHTS_KISS, &[]),
    (&NOX_RANGERS_HEAVY_CROSSBOW, &[]),
    (&ORCISH_VISAGE, &[]),
    (&PIXIE_SWATTER, &[]),
    (&POLAR_BEAR_MASK, &[]),
    (&SHIP_MODEL_OF_THE_HMS_CAPE, &[]),
    (&STAFF_OF_POWER, &[]),
    (&VIOLET_COURAGE, &[]),
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
