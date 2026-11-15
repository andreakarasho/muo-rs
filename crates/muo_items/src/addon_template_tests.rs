//! The add-on, decoration and paired classes whose components, deed pairing and class tables moved to
//! `Data/Templates/Items` keep them, and the moved readers give the old values.

use muo_content::items::addon;
use muo_content::templates::{self, Kind, item_steps};
use muo_primitives::Point3D;
use muo_server::World;
use muo_server::item::ItemType;

/// `(class, key, is a prop)` written by the conversion.
const KEYS: &[(&str, &str, bool)] = &[
    ("Server.Items.AbbatoirAddon", "addonComponents", false),
    ("Server.Items.AbbatoirAddon", "addonDeed", true),
    ("Server.Items.AbbatoirDeed", "deedAddon", true),
    (
        "Server.Items.AlchemistTableEastAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.AlchemistTableEastAddon", "addonDeed", true),
    ("Server.Items.AlchemistTableEastDeed", "deedAddon", true),
    (
        "Server.Items.AlchemistTableSouthAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.AlchemistTableSouthAddon", "addonDeed", true),
    ("Server.Items.AlchemistTableSouthDeed", "deedAddon", true),
    ("Server.Items.AnkhNorth", "pairOffset", true),
    ("Server.Items.AnkhNorth+InternalItem", "pairOffset", true),
    ("Server.Items.AnkhWest", "pairOffset", true),
    ("Server.Items.AnkhWest+InternalItem", "pairOffset", true),
    ("Server.Items.AnvilEastAddon", "addonComponents", false),
    ("Server.Items.AnvilEastAddon", "addonDeed", true),
    ("Server.Items.AnvilEastDeed", "deedAddon", true),
    ("Server.Items.AnvilSouthAddon", "addonComponents", false),
    ("Server.Items.AnvilSouthAddon", "addonDeed", true),
    ("Server.Items.AnvilSouthDeed", "deedAddon", true),
    ("Server.Items.AppleTreeAddon", "addonComponents", false),
    ("Server.Items.AppleTreeAddon", "addonDeed", true),
    ("Server.Items.AppleTreeDeed", "deedAddon", true),
    ("Server.Items.AppleTrunkAddon", "addonComponents", false),
    ("Server.Items.AppleTrunkAddon", "addonDeed", true),
    ("Server.Items.AppleTrunkDeed", "deedAddon", true),
    (
        "Server.Items.ArcaneBookshelfEastAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.ArcaneBookshelfEastAddon", "addonDeed", true),
    (
        "Server.Items.ArcaneBookshelfEastAddon",
        "addonRetainDeedHue",
        true,
    ),
    ("Server.Items.ArcaneBookshelfEastDeed", "deedAddon", true),
    (
        "Server.Items.ArcaneBookshelfSouthAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.ArcaneBookshelfSouthAddon", "addonDeed", true),
    (
        "Server.Items.ArcaneBookshelfSouthAddon",
        "addonRetainDeedHue",
        true,
    ),
    ("Server.Items.ArcaneBookshelfSouthDeed", "deedAddon", true),
    ("Server.Items.ArcaneCircleAddon", "addonComponents", false),
    ("Server.Items.ArcaneCircleAddon", "addonDeed", true),
    ("Server.Items.ArcaneCircleDeed", "deedAddon", true),
    (
        "Server.Items.ArcanistStatueEastAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.ArcanistStatueEastAddon", "addonDeed", true),
    ("Server.Items.ArcanistStatueEastDeed", "deedAddon", true),
    (
        "Server.Items.ArcanistStatueSouthAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.ArcanistStatueSouthAddon", "addonDeed", true),
    ("Server.Items.ArcanistStatueSouthDeed", "deedAddon", true),
    ("Server.Items.ArcheryButteAddon", "addonComponents", false),
    ("Server.Items.ArcheryButteAddon", "addonDeed", true),
    ("Server.Items.ArcheryButteDeed", "deedAddon", true),
    (
        "Server.Items.AwesomeDisturbingPortraitAddon",
        "addonDeed",
        true,
    ),
    (
        "Server.Items.AwesomeDisturbingPortraitDeed",
        "deedAddon",
        true,
    ),
    ("Server.Items.BedOfNailsAddon", "addonDeed", true),
    ("Server.Items.BedOfNailsDeed", "deedAddon", true),
    ("Server.Items.BloodPentagram", "addonComponents", false),
    (
        "Server.Items.BlueDecorativeRugAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.BlueDecorativeRugAddon", "addonDeed", true),
    ("Server.Items.BlueDecorativeRugDeed", "deedAddon", true),
    ("Server.Items.BlueFancyRugAddon", "addonComponents", false),
    ("Server.Items.BlueFancyRugAddon", "addonDeed", true),
    ("Server.Items.BlueFancyRugDeed", "deedAddon", true),
    ("Server.Items.BluePlainRugAddon", "addonComponents", false),
    ("Server.Items.BluePlainRugAddon", "addonDeed", true),
    ("Server.Items.BluePlainRugDeed", "deedAddon", true),
    (
        "Server.Items.BoilingCauldronAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.BoilingCauldronAddon", "addonDeed", true),
    ("Server.Items.BoilingCauldronDeed", "deedAddon", true),
    ("Server.Items.BoneCouchAddon", "addonDeed", true),
    ("Server.Items.BoneCouchDeed", "deedAddon", true),
    ("Server.Items.BoneTableAddon", "addonComponents", false),
    ("Server.Items.BoneTableAddon", "addonDeed", true),
    ("Server.Items.BoneTableDeed", "deedAddon", true),
    ("Server.Items.BoneThroneAddon", "addonComponents", false),
    ("Server.Items.BoneThroneAddon", "addonDeed", true),
    ("Server.Items.BoneThroneDeed", "deedAddon", true),
    (
        "Server.Items.BrocadeGozaMatEastAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.BrocadeGozaMatEastAddon", "addonDeed", true),
    (
        "Server.Items.BrocadeGozaMatEastAddon",
        "addonRetainDeedHue",
        true,
    ),
    ("Server.Items.BrocadeGozaMatEastDeed", "deedAddon", true),
    (
        "Server.Items.BrocadeGozaMatSouthAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.BrocadeGozaMatSouthAddon", "addonDeed", true),
    (
        "Server.Items.BrocadeGozaMatSouthAddon",
        "addonRetainDeedHue",
        true,
    ),
    ("Server.Items.BrocadeGozaMatSouthDeed", "deedAddon", true),
    (
        "Server.Items.BrocadeSquareGozaMatEastAddon",
        "addonComponents",
        false,
    ),
    (
        "Server.Items.BrocadeSquareGozaMatEastAddon",
        "addonDeed",
        true,
    ),
    (
        "Server.Items.BrocadeSquareGozaMatEastAddon",
        "addonRetainDeedHue",
        true,
    ),
    (
        "Server.Items.BrocadeSquareGozaMatEastDeed",
        "deedAddon",
        true,
    ),
    (
        "Server.Items.BrocadeSquareGozaMatSouthAddon",
        "addonComponents",
        false,
    ),
    (
        "Server.Items.BrocadeSquareGozaMatSouthAddon",
        "addonDeed",
        true,
    ),
    (
        "Server.Items.BrocadeSquareGozaMatSouthAddon",
        "addonRetainDeedHue",
        true,
    ),
    (
        "Server.Items.BrocadeSquareGozaMatSouthDeed",
        "deedAddon",
        true,
    ),
    ("Server.Items.BrokenArmoireAddon", "addonComponents", false),
    ("Server.Items.BrokenArmoireAddon", "addonDeed", true),
    ("Server.Items.BrokenArmoireDeed", "deedAddon", true),
    ("Server.Items.BrokenBedAddon", "addonComponentsEast", true),
    ("Server.Items.BrokenBedAddon", "addonDeed", true),
    ("Server.Items.BrokenBedDeed", "deedAddon", true),
    ("Server.Items.BrokenBedDeed", "deedSelectionNumber", true),
    ("Server.Items.BrokenBookcaseAddon", "addonComponents", false),
    ("Server.Items.BrokenBookcaseAddon", "addonDeed", true),
    ("Server.Items.BrokenBookcaseDeed", "deedAddon", true),
    (
        "Server.Items.BrokenChestOfDrawersAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.BrokenChestOfDrawersAddon", "addonDeed", true),
    ("Server.Items.BrokenChestOfDrawersDeed", "deedAddon", true),
    (
        "Server.Items.BrokenCoveredChairAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.BrokenCoveredChairAddon", "addonDeed", true),
    ("Server.Items.BrokenCoveredChairDeed", "deedAddon", true),
    (
        "Server.Items.BrokenFallenChairAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.BrokenFallenChairAddon", "addonDeed", true),
    ("Server.Items.BrokenFallenChairDeed", "deedAddon", true),
    (
        "Server.Items.BrokenVanityAddon",
        "addonComponentsEast",
        true,
    ),
    ("Server.Items.BrokenVanityAddon", "addonDeed", true),
    ("Server.Items.BrokenVanityDeed", "deedAddon", true),
    ("Server.Items.BrokenVanityDeed", "deedSelectionNumber", true),
    (
        "Server.Items.BrownBearRugEastAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.BrownBearRugEastAddon", "addonDeed", true),
    ("Server.Items.BrownBearRugEastDeed", "deedAddon", true),
    (
        "Server.Items.BrownBearRugSouthAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.BrownBearRugSouthAddon", "addonDeed", true),
    ("Server.Items.BrownBearRugSouthDeed", "deedAddon", true),
    (
        "Server.Items.CherryBlossomTreeAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.CherryBlossomTreeAddon", "addonDeed", true),
    ("Server.Items.CherryBlossomTreeDeed", "deedAddon", true),
    (
        "Server.Items.CherryBlossomTrunkAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.CherryBlossomTrunkAddon", "addonDeed", true),
    ("Server.Items.CherryBlossomTrunkDeed", "deedAddon", true),
    (
        "Server.Items.CinnamonFancyRugAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.CinnamonFancyRugAddon", "addonDeed", true),
    ("Server.Items.CinnamonFancyRugDeed", "deedAddon", true),
    ("Server.Items.CreepyPortraitAddon", "addonDeed", true),
    ("Server.Items.CreepyPortraitDeed", "deedAddon", true),
    ("Server.Items.CurtainsAddon", "addonDeed", true),
    (
        "Server.Items.DarkFlowerTapestryEastAddon",
        "addonComponents",
        false,
    ),
    (
        "Server.Items.DarkFlowerTapestryEastAddon",
        "addonDeed",
        true,
    ),
    ("Server.Items.DarkFlowerTapestryEastDeed", "deedAddon", true),
    (
        "Server.Items.DarkFlowerTapestrySouthAddon",
        "addonComponents",
        false,
    ),
    (
        "Server.Items.DarkFlowerTapestrySouthAddon",
        "addonDeed",
        true,
    ),
    (
        "Server.Items.DarkFlowerTapestrySouthDeed",
        "deedAddon",
        true,
    ),
    ("Server.Items.DartBoardEastAddon", "addonComponents", false),
    ("Server.Items.DartBoardEastAddon", "addonDeed", true),
    ("Server.Items.DartBoardEastDeed", "deedAddon", true),
    ("Server.Items.DartBoardSouthAddon", "addonComponents", false),
    ("Server.Items.DartBoardSouthAddon", "addonDeed", true),
    ("Server.Items.DartBoardSouthDeed", "deedAddon", true),
    ("Server.Items.DecorativeDAxeNorth", "pairOffset", true),
    (
        "Server.Items.DecorativeDAxeNorth+InternalItem",
        "pairOffset",
        true,
    ),
    ("Server.Items.DecorativeDAxeWest", "pairOffset", true),
    (
        "Server.Items.DecorativeDAxeWest+InternalItem",
        "pairOffset",
        true,
    ),
    ("Server.Items.DecorativeSwordNorth", "pairOffset", true),
    (
        "Server.Items.DecorativeSwordNorth+InternalItem",
        "pairOffset",
        true,
    ),
    ("Server.Items.DecorativeSwordWest", "pairOffset", true),
    (
        "Server.Items.DecorativeSwordWest+InternalItem",
        "pairOffset",
        true,
    ),
    ("Server.Items.DisturbingPortraitAddon", "addonDeed", true),
    ("Server.Items.DisturbingPortraitDeed", "deedAddon", true),
    ("Server.Items.ElvenBedEastAddon", "addonComponents", false),
    ("Server.Items.ElvenBedEastAddon", "addonDeed", true),
    ("Server.Items.ElvenBedEastDeed", "deedAddon", true),
    ("Server.Items.ElvenBedSouthAddon", "addonComponents", false),
    ("Server.Items.ElvenBedSouthAddon", "addonDeed", true),
    ("Server.Items.ElvenBedSouthDeed", "deedAddon", true),
    (
        "Server.Items.ElvenDresserEastAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.ElvenDresserEastAddon", "addonDeed", true),
    (
        "Server.Items.ElvenDresserEastAddon",
        "addonRetainDeedHue",
        true,
    ),
    ("Server.Items.ElvenDresserEastDeed", "deedAddon", true),
    (
        "Server.Items.ElvenDresserSouthAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.ElvenDresserSouthAddon", "addonDeed", true),
    (
        "Server.Items.ElvenDresserSouthAddon",
        "addonRetainDeedHue",
        true,
    ),
    ("Server.Items.ElvenDresserSouthDeed", "deedAddon", true),
    ("Server.Items.ElvenForgeAddon", "addonComponents", false),
    ("Server.Items.ElvenForgeAddon", "addonDeed", true),
    ("Server.Items.ElvenForgeDeed", "deedAddon", true),
    (
        "Server.Items.ElvenLoveseatEastAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.ElvenLoveseatEastAddon", "addonDeed", true),
    ("Server.Items.ElvenLoveseatEastDeed", "deedAddon", true),
    (
        "Server.Items.ElvenLoveseatSouthAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.ElvenLoveseatSouthAddon", "addonDeed", true),
    ("Server.Items.ElvenLoveseatSouthDeed", "deedAddon", true),
    (
        "Server.Items.ElvenSpinningWheelEastAddon",
        "addonComponents",
        false,
    ),
    (
        "Server.Items.ElvenSpinningWheelEastAddon",
        "addonDeed",
        true,
    ),
    ("Server.Items.ElvenSpinningWheelEastDeed", "deedAddon", true),
    (
        "Server.Items.ElvenSpinningWheelSouthAddon",
        "addonComponents",
        false,
    ),
    (
        "Server.Items.ElvenSpinningWheelSouthAddon",
        "addonDeed",
        true,
    ),
    (
        "Server.Items.ElvenSpinningWheelSouthDeed",
        "deedAddon",
        true,
    ),
    ("Server.Items.ElvenStoveEastAddon", "addonComponents", false),
    ("Server.Items.ElvenStoveEastAddon", "addonDeed", true),
    ("Server.Items.ElvenStoveEastDeed", "deedAddon", true),
    (
        "Server.Items.ElvenStoveSouthAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.ElvenStoveSouthAddon", "addonDeed", true),
    ("Server.Items.ElvenStoveSouthDeed", "deedAddon", true),
    (
        "Server.Items.ElvenWashBasinEastAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.ElvenWashBasinEastAddon", "addonDeed", true),
    (
        "Server.Items.ElvenWashBasinEastAddon",
        "addonRetainDeedHue",
        true,
    ),
    ("Server.Items.ElvenWashBasinEastDeed", "deedAddon", true),
    (
        "Server.Items.ElvenWashBasinSouthAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.ElvenWashBasinSouthAddon", "addonDeed", true),
    (
        "Server.Items.ElvenWashBasinSouthAddon",
        "addonRetainDeedHue",
        true,
    ),
    ("Server.Items.ElvenWashBasinSouthDeed", "deedAddon", true),
    (
        "Server.Items.FancyElvenTableEastAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.FancyElvenTableEastAddon", "addonDeed", true),
    ("Server.Items.FancyElvenTableEastDeed", "deedAddon", true),
    (
        "Server.Items.FancyElvenTableSouthAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.FancyElvenTableSouthAddon", "addonDeed", true),
    ("Server.Items.FancyElvenTableSouthDeed", "deedAddon", true),
    ("Server.Items.FireColumnAddon", "addonComponents", false),
    ("Server.Items.FireColumnAddon", "addonShareHue", true),
    ("Server.Items.FlourMillEastAddon", "addonComponents", false),
    ("Server.Items.FlourMillEastAddon", "addonDeed", true),
    ("Server.Items.FlourMillEastDeed", "deedAddon", true),
    ("Server.Items.FlourMillSouthAddon", "addonComponents", false),
    ("Server.Items.FlourMillSouthAddon", "addonDeed", true),
    ("Server.Items.FlourMillSouthDeed", "deedAddon", true),
    ("Server.Items.FountainAddon", "addonComponents", false),
    ("Server.Items.FountainAddon", "addonDeed", true),
    ("Server.Items.FountainDeed", "deedAddon", true),
    ("Server.Items.GiantWeb1", "addonComponents", false),
    ("Server.Items.GiantWeb2", "addonComponents", false),
    ("Server.Items.GiantWeb3", "addonComponents", false),
    ("Server.Items.GiantWeb4", "addonComponents", false),
    ("Server.Items.GiantWeb5", "addonComponents", false),
    ("Server.Items.GiantWeb6", "addonComponents", false),
    (
        "Server.Items.GoldenDecorativeRugAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.GoldenDecorativeRugAddon", "addonDeed", true),
    ("Server.Items.GoldenDecorativeRugDeed", "deedAddon", true),
    ("Server.Items.GozaMatEastAddon", "addonComponents", false),
    ("Server.Items.GozaMatEastAddon", "addonDeed", true),
    ("Server.Items.GozaMatEastAddon", "addonRetainDeedHue", true),
    ("Server.Items.GozaMatEastDeed", "deedAddon", true),
    ("Server.Items.GozaMatSouthAddon", "addonComponents", false),
    ("Server.Items.GozaMatSouthAddon", "addonDeed", true),
    ("Server.Items.GozaMatSouthAddon", "addonRetainDeedHue", true),
    ("Server.Items.GozaMatSouthDeed", "deedAddon", true),
    (
        "Server.Items.GrayBrickFireplaceEastAddon",
        "addonComponents",
        false,
    ),
    (
        "Server.Items.GrayBrickFireplaceEastAddon",
        "addonDeed",
        true,
    ),
    ("Server.Items.GrayBrickFireplaceEastDeed", "deedAddon", true),
    (
        "Server.Items.GrayBrickFireplaceSouthAddon",
        "addonComponents",
        false,
    ),
    (
        "Server.Items.GrayBrickFireplaceSouthAddon",
        "addonDeed",
        true,
    ),
    (
        "Server.Items.GrayBrickFireplaceSouthDeed",
        "deedAddon",
        true,
    ),
    ("Server.Items.GuillotineAddon", "addonComponents", false),
    ("Server.Items.GuillotineAddon", "addonDeed", true),
    ("Server.Items.GuillotineDeed", "deedAddon", true),
    ("Server.Items.HangingAxesAddon", "addonComponentsEast", true),
    ("Server.Items.HangingAxesAddon", "addonDeed", true),
    ("Server.Items.HangingAxesDeed", "deedAddon", true),
    ("Server.Items.HangingAxesDeed", "deedSelectionNumber", true),
    (
        "Server.Items.HangingSwordsAddon",
        "addonComponentsEast",
        true,
    ),
    ("Server.Items.HangingSwordsAddon", "addonDeed", true),
    ("Server.Items.HangingSwordsDeed", "deedAddon", true),
    (
        "Server.Items.HangingSwordsDeed",
        "deedSelectionNumber",
        true,
    ),
    ("Server.Items.HaunterMirrorAddon", "addonComponents", false),
    ("Server.Items.HaunterMirrorAddon", "addonDeed", true),
    ("Server.Items.HaunterMirrorDeed", "deedAddon", true),
    ("Server.Items.HouseLadderAddon", "addonDeed", true),
    ("Server.Items.IronMaidenAddon", "addonComponents", false),
    ("Server.Items.IronMaidenAddon", "addonDeed", true),
    ("Server.Items.IronMaidenDeed", "deedAddon", true),
    ("Server.Items.LargeBedEastAddon", "addonComponents", false),
    ("Server.Items.LargeBedEastAddon", "addonDeed", true),
    ("Server.Items.LargeBedEastDeed", "deedAddon", true),
    ("Server.Items.LargeBedSouthAddon", "addonComponents", false),
    ("Server.Items.LargeBedSouthAddon", "addonDeed", true),
    ("Server.Items.LargeBedSouthDeed", "deedAddon", true),
    (
        "Server.Items.LargeFishingNetAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.LargeFishingNetAddon", "addonDeed", true),
    ("Server.Items.LargeFishingNetDeed", "deedAddon", true),
    ("Server.Items.LargeForgeEastAddon", "addonComponents", false),
    ("Server.Items.LargeForgeEastAddon", "addonDeed", true),
    ("Server.Items.LargeForgeEastDeed", "deedAddon", true),
    (
        "Server.Items.LargeForgeSouthAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.LargeForgeSouthAddon", "addonDeed", true),
    ("Server.Items.LargeForgeSouthDeed", "deedAddon", true),
    (
        "Server.Items.LargeStoneTableEastAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.LargeStoneTableEastAddon", "addonDeed", true),
    (
        "Server.Items.LargeStoneTableEastAddon",
        "addonRetainDeedHue",
        true,
    ),
    ("Server.Items.LargeStoneTableEastDeed", "deedAddon", true),
    (
        "Server.Items.LargeStoneTableSouthAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.LargeStoneTableSouthAddon", "addonDeed", true),
    (
        "Server.Items.LargeStoneTableSouthAddon",
        "addonRetainDeedHue",
        true,
    ),
    ("Server.Items.LargeStoneTableSouthDeed", "deedAddon", true),
    (
        "Server.Items.LightFlowerTapestryEastAddon",
        "addonComponents",
        false,
    ),
    (
        "Server.Items.LightFlowerTapestryEastAddon",
        "addonDeed",
        true,
    ),
    (
        "Server.Items.LightFlowerTapestryEastDeed",
        "deedAddon",
        true,
    ),
    (
        "Server.Items.LightFlowerTapestrySouthAddon",
        "addonComponents",
        false,
    ),
    (
        "Server.Items.LightFlowerTapestrySouthAddon",
        "addonDeed",
        true,
    ),
    (
        "Server.Items.LightFlowerTapestrySouthDeed",
        "deedAddon",
        true,
    ),
    ("Server.Items.LoomEastAddon", "addonComponents", false),
    ("Server.Items.LoomEastAddon", "addonDeed", true),
    ("Server.Items.LoomEastDeed", "deedAddon", true),
    ("Server.Items.LoomSouthAddon", "addonComponents", false),
    ("Server.Items.LoomSouthAddon", "addonDeed", true),
    ("Server.Items.LoomSouthDeed", "deedAddon", true),
    (
        "Server.Items.MediumStoneTableEastAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.MediumStoneTableEastAddon", "addonDeed", true),
    (
        "Server.Items.MediumStoneTableEastAddon",
        "addonRetainDeedHue",
        true,
    ),
    ("Server.Items.MediumStoneTableEastDeed", "deedAddon", true),
    (
        "Server.Items.MediumStoneTableSouthAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.MediumStoneTableSouthAddon", "addonDeed", true),
    (
        "Server.Items.MediumStoneTableSouthAddon",
        "addonRetainDeedHue",
        true,
    ),
    ("Server.Items.MediumStoneTableSouthDeed", "deedAddon", true),
    (
        "Server.Items.MediumStretchedHideEastAddon",
        "addonComponents",
        false,
    ),
    (
        "Server.Items.MediumStretchedHideEastAddon",
        "addonDeed",
        true,
    ),
    (
        "Server.Items.MediumStretchedHideEastDeed",
        "deedAddon",
        true,
    ),
    (
        "Server.Items.MediumStretchedHideSouthAddon",
        "addonComponents",
        false,
    ),
    (
        "Server.Items.MediumStretchedHideSouthAddon",
        "addonDeed",
        true,
    ),
    (
        "Server.Items.MediumStretchedHideSouthDeed",
        "deedAddon",
        true,
    ),
    (
        "Server.Items.MountedPixieBlueAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.MountedPixieBlueAddon", "addonDeed", true),
    ("Server.Items.MountedPixieBlueDeed", "deedAddon", true),
    (
        "Server.Items.MountedPixieGreenAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.MountedPixieGreenAddon", "addonDeed", true),
    ("Server.Items.MountedPixieGreenDeed", "deedAddon", true),
    (
        "Server.Items.MountedPixieLimeAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.MountedPixieLimeAddon", "addonDeed", true),
    ("Server.Items.MountedPixieLimeDeed", "deedAddon", true),
    (
        "Server.Items.MountedPixieOrangeAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.MountedPixieOrangeAddon", "addonDeed", true),
    ("Server.Items.MountedPixieOrangeDeed", "deedAddon", true),
    (
        "Server.Items.MountedPixieWhiteAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.MountedPixieWhiteAddon", "addonDeed", true),
    ("Server.Items.MountedPixieWhiteDeed", "deedAddon", true),
    (
        "Server.Items.OrnateElvenChestEastAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.OrnateElvenChestEastAddon", "addonDeed", true),
    (
        "Server.Items.OrnateElvenChestEastAddon",
        "addonRetainDeedHue",
        true,
    ),
    ("Server.Items.OrnateElvenChestEastDeed", "deedAddon", true),
    (
        "Server.Items.OrnateElvenChestSouthAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.OrnateElvenChestSouthAddon", "addonDeed", true),
    (
        "Server.Items.OrnateElvenChestSouthAddon",
        "addonRetainDeedHue",
        true,
    ),
    ("Server.Items.OrnateElvenChestSouthDeed", "deedAddon", true),
    (
        "Server.Items.OrnateElvenTableEastAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.OrnateElvenTableEastAddon", "addonDeed", true),
    ("Server.Items.OrnateElvenTableEastDeed", "deedAddon", true),
    (
        "Server.Items.OrnateElvenTableSouthAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.OrnateElvenTableSouthAddon", "addonDeed", true),
    ("Server.Items.OrnateElvenTableSouthDeed", "deedAddon", true),
    ("Server.Items.ParrotPerchAddon", "addonComponents", false),
    ("Server.Items.ParrotPerchAddon", "addonDeed", true),
    ("Server.Items.ParrotPerchDeed", "deedAddon", true),
    ("Server.Items.PeachTreeAddon", "addonComponents", false),
    ("Server.Items.PeachTreeAddon", "addonDeed", true),
    ("Server.Items.PeachTreeDeed", "deedAddon", true),
    ("Server.Items.PeachTrunkAddon", "addonComponents", false),
    ("Server.Items.PeachTrunkAddon", "addonDeed", true),
    ("Server.Items.PeachTrunkDeed", "deedAddon", true),
    ("Server.Items.PentagramAddon", "addonComponents", false),
    ("Server.Items.PentagramAddon", "addonDeed", true),
    ("Server.Items.PentagramDeed", "deedAddon", true),
    (
        "Server.Items.PickpocketDipEastAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.PickpocketDipEastAddon", "addonDeed", true),
    ("Server.Items.PickpocketDipEastDeed", "deedAddon", true),
    (
        "Server.Items.PickpocketDipSouthAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.PickpocketDipSouthAddon", "addonDeed", true),
    ("Server.Items.PickpocketDipSouthDeed", "deedAddon", true),
    ("Server.Items.PinkFancyRugAddon", "addonComponents", false),
    ("Server.Items.PinkFancyRugAddon", "addonDeed", true),
    ("Server.Items.PinkFancyRugDeed", "deedAddon", true),
    (
        "Server.Items.PolarBearRugEastAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.PolarBearRugEastAddon", "addonDeed", true),
    ("Server.Items.PolarBearRugEastDeed", "deedAddon", true),
    (
        "Server.Items.PolarBearRugSouthAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.PolarBearRugSouthAddon", "addonDeed", true),
    ("Server.Items.PolarBearRugSouthDeed", "deedAddon", true),
    ("Server.Items.PyramidAddon", "addonComponents", false),
    ("Server.Items.PyramidAddon", "addonShareHue", true),
    ("Server.Items.RedPlainRugAddon", "addonComponents", false),
    ("Server.Items.RedPlainRugAddon", "addonDeed", true),
    ("Server.Items.RedPlainRugDeed", "deedAddon", true),
    (
        "Server.Items.RejuvinationAnkhNorth",
        "addonComponents",
        false,
    ),
    (
        "Server.Items.RejuvinationAnkhWest",
        "addonComponents",
        false,
    ),
    (
        "Server.Items.SandstoneFireplaceEastAddon",
        "addonComponents",
        false,
    ),
    (
        "Server.Items.SandstoneFireplaceEastAddon",
        "addonDeed",
        true,
    ),
    ("Server.Items.SandstoneFireplaceEastDeed", "deedAddon", true),
    (
        "Server.Items.SandstoneFireplaceSouthAddon",
        "addonComponents",
        false,
    ),
    (
        "Server.Items.SandstoneFireplaceSouthAddon",
        "addonDeed",
        true,
    ),
    (
        "Server.Items.SandstoneFireplaceSouthDeed",
        "deedAddon",
        true,
    ),
    (
        "Server.Items.SandstoneFountainAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.ScarecrowAddon", "addonComponents", false),
    ("Server.Items.ScarecrowAddon", "addonDeed", true),
    ("Server.Items.ScarecrowDeed", "deedAddon", true),
    ("Server.Items.SerpentPillarAddon", "addonComponents", false),
    ("Server.Items.ShrineOfWisdomAddon", "addonComponents", false),
    ("Server.Items.SkullPileAddon", "addonComponents", false),
    ("Server.Items.SmallBedEastAddon", "addonComponents", false),
    ("Server.Items.SmallBedEastAddon", "addonDeed", true),
    ("Server.Items.SmallBedEastDeed", "deedAddon", true),
    ("Server.Items.SmallBedSouthAddon", "addonComponents", false),
    ("Server.Items.SmallBedSouthAddon", "addonDeed", true),
    ("Server.Items.SmallBedSouthDeed", "deedAddon", true),
    (
        "Server.Items.SmallFishingNetAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.SmallFishingNetAddon", "addonDeed", true),
    ("Server.Items.SmallFishingNetDeed", "deedAddon", true),
    ("Server.Items.SmallForgeAddon", "addonComponents", false),
    ("Server.Items.SmallForgeAddon", "addonDeed", true),
    ("Server.Items.SmallForgeDeed", "deedAddon", true),
    (
        "Server.Items.SmallStretchedHideEastAddon",
        "addonComponents",
        false,
    ),
    (
        "Server.Items.SmallStretchedHideEastAddon",
        "addonDeed",
        true,
    ),
    ("Server.Items.SmallStretchedHideEastDeed", "deedAddon", true),
    (
        "Server.Items.SmallStretchedHideSouthAddon",
        "addonComponents",
        false,
    ),
    (
        "Server.Items.SmallStretchedHideSouthAddon",
        "addonDeed",
        true,
    ),
    (
        "Server.Items.SmallStretchedHideSouthDeed",
        "deedAddon",
        true,
    ),
    (
        "Server.Items.SpinningWheelEastAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.SpinningWheelEastAddon", "addonDeed", true),
    ("Server.Items.SpinningWheelEastDeed", "deedAddon", true),
    (
        "Server.Items.SpinningWheelSouthAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.SpinningWheelSouthAddon", "addonDeed", true),
    ("Server.Items.SpinningWheelSouthDeed", "deedAddon", true),
    (
        "Server.Items.SquareGozaMatEastAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.SquareGozaMatEastAddon", "addonDeed", true),
    (
        "Server.Items.SquareGozaMatEastAddon",
        "addonRetainDeedHue",
        true,
    ),
    ("Server.Items.SquareGozaMatEastDeed", "deedAddon", true),
    (
        "Server.Items.SquareGozaMatSouthAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.SquareGozaMatSouthAddon", "addonDeed", true),
    (
        "Server.Items.SquareGozaMatSouthAddon",
        "addonRetainDeedHue",
        true,
    ),
    ("Server.Items.SquareGozaMatSouthDeed", "deedAddon", true),
    (
        "Server.Items.SquirrelStatueEastAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.SquirrelStatueEastAddon", "addonDeed", true),
    ("Server.Items.SquirrelStatueEastDeed", "deedAddon", true),
    (
        "Server.Items.SquirrelStatueSouthAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.SquirrelStatueSouthAddon", "addonDeed", true),
    ("Server.Items.SquirrelStatueSouthDeed", "deedAddon", true),
    (
        "Server.Items.StandingBrokenChairAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.StandingBrokenChairAddon", "addonDeed", true),
    ("Server.Items.StandingBrokenChairDeed", "deedAddon", true),
    (
        "Server.Items.StoneFireplaceEastAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.StoneFireplaceEastAddon", "addonDeed", true),
    ("Server.Items.StoneFireplaceEastDeed", "deedAddon", true),
    (
        "Server.Items.StoneFireplaceSouthAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.StoneFireplaceSouthAddon", "addonDeed", true),
    ("Server.Items.StoneFireplaceSouthDeed", "deedAddon", true),
    ("Server.Items.StoneFountainAddon", "addonComponents", false),
    ("Server.Items.StoneOvenEastAddon", "addonComponents", false),
    ("Server.Items.StoneOvenEastAddon", "addonDeed", true),
    ("Server.Items.StoneOvenEastDeed", "deedAddon", true),
    ("Server.Items.StoneOvenSouthAddon", "addonComponents", false),
    ("Server.Items.StoneOvenSouthAddon", "addonDeed", true),
    ("Server.Items.StoneOvenSouthDeed", "deedAddon", true),
    ("Server.Items.StoneStatueAddon", "addonComponentsEast", true),
    ("Server.Items.StoneStatueAddon", "addonDeed", true),
    ("Server.Items.StoneStatueDeed", "deedAddon", true),
    ("Server.Items.StoneStatueDeed", "deedSelectionNumber", true),
    (
        "Server.Items.SuitOfGoldArmorAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.SuitOfGoldArmorAddon", "addonDeed", true),
    ("Server.Items.SuitOfGoldArmorDeed", "deedAddon", true),
    (
        "Server.Items.SuitOfSilverArmorAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.SuitOfSilverArmorAddon", "addonDeed", true),
    ("Server.Items.SuitOfSilverArmorDeed", "deedAddon", true),
    (
        "Server.Items.TableWithBlueClothAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.TableWithBlueClothAddon", "addonDeed", true),
    ("Server.Items.TableWithBlueClothDeed", "deedAddon", true),
    (
        "Server.Items.TableWithOrangeClothAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.TableWithOrangeClothAddon", "addonDeed", true),
    ("Server.Items.TableWithOrangeClothDeed", "deedAddon", true),
    (
        "Server.Items.TableWithPurpleClothAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.TableWithPurpleClothAddon", "addonDeed", true),
    ("Server.Items.TableWithPurpleClothDeed", "deedAddon", true),
    (
        "Server.Items.TableWithRedClothAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.TableWithRedClothAddon", "addonDeed", true),
    ("Server.Items.TableWithRedClothDeed", "deedAddon", true),
    (
        "Server.Items.TallElvenBedEastAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.TallElvenBedEastAddon", "addonDeed", true),
    ("Server.Items.TallElvenBedEastDeed", "deedAddon", true),
    (
        "Server.Items.TallElvenBedSouthAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.TallElvenBedSouthAddon", "addonDeed", true),
    ("Server.Items.TallElvenBedSouthDeed", "deedAddon", true),
    ("Server.Items.Tapestry1N", "pairOffset", true),
    ("Server.Items.Tapestry1N+InternalItem", "pairOffset", true),
    ("Server.Items.Tapestry2N", "pairOffset", true),
    ("Server.Items.Tapestry2N+InternalItem", "pairOffset", true),
    ("Server.Items.Tapestry2W", "pairOffset", true),
    ("Server.Items.Tapestry2W+InternalItem", "pairOffset", true),
    ("Server.Items.Tapestry3N", "pairOffset", true),
    ("Server.Items.Tapestry3N+InternalItem", "pairOffset", true),
    ("Server.Items.Tapestry3W", "pairOffset", true),
    ("Server.Items.Tapestry3W+InternalItem", "pairOffset", true),
    ("Server.Items.Tapestry4N", "pairOffset", true),
    ("Server.Items.Tapestry4N+InternalItem", "pairOffset", true),
    ("Server.Items.Tapestry4W", "pairOffset", true),
    ("Server.Items.Tapestry4W+InternalItem", "pairOffset", true),
    ("Server.Items.Tapestry5N", "pairOffset", true),
    ("Server.Items.Tapestry5N+InternalItem", "pairOffset", true),
    ("Server.Items.Tapestry5W", "pairOffset", true),
    ("Server.Items.Tapestry5W+InternalItem", "pairOffset", true),
    ("Server.Items.Tapestry6N", "pairOffset", true),
    ("Server.Items.Tapestry6N+InternalItem", "pairOffset", true),
    ("Server.Items.Tapestry6W", "pairOffset", true),
    ("Server.Items.Tapestry6W+InternalItem", "pairOffset", true),
    ("Server.Items.Telescope", "addonComponents", false),
    (
        "Server.Items.TrainingDummyEastAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.TrainingDummyEastAddon", "addonDeed", true),
    ("Server.Items.TrainingDummyEastDeed", "deedAddon", true),
    (
        "Server.Items.TrainingDummySouthAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.TrainingDummySouthAddon", "addonDeed", true),
    ("Server.Items.TrainingDummySouthDeed", "deedAddon", true),
    ("Server.Items.UnmadeBedAddon", "addonComponentsEast", true),
    ("Server.Items.UnmadeBedAddon", "addonDeed", true),
    ("Server.Items.UnmadeBedDeed", "deedAddon", true),
    ("Server.Items.UnmadeBedDeed", "deedSelectionNumber", true),
    ("Server.Items.UnsettlingPortraitAddon", "addonDeed", true),
    ("Server.Items.UnsettlingPortraitDeed", "deedAddon", true),
    ("Server.Items.VanityAddon", "addonComponentsEast", true),
    ("Server.Items.VanityAddon", "addonDeed", true),
    ("Server.Items.VanityDeed", "deedAddon", true),
    ("Server.Items.VanityDeed", "deedSelectionNumber", true),
    ("Server.Items.WallTorchAddon", "addonComponents", false),
    ("Server.Items.WallTorchAddon", "addonDeed", true),
    ("Server.Items.WallTorchDeed", "deedAddon", true),
    (
        "Server.Items.WarriorStatueEastAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.WarriorStatueEastAddon", "addonDeed", true),
    ("Server.Items.WarriorStatueEastDeed", "deedAddon", true),
    (
        "Server.Items.WarriorStatueSouthAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.WarriorStatueSouthAddon", "addonDeed", true),
    ("Server.Items.WarriorStatueSouthDeed", "deedAddon", true),
    (
        "Server.Items.WaterTroughEastAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.WaterTroughEastAddon", "addonDeed", true),
    ("Server.Items.WaterTroughEastDeed", "deedAddon", true),
    (
        "Server.Items.WaterTroughSouthAddon",
        "addonComponents",
        false,
    ),
    ("Server.Items.WaterTroughSouthAddon", "addonDeed", true),
    ("Server.Items.WaterTroughSouthDeed", "deedAddon", true),
    ("Server.Items.WaterVatEast", "addonComponents", false),
    ("Server.Items.WaterVatSouth", "addonComponents", false),
    (
        "Server.Items.WoodenCoffinAddon",
        "addonComponentsEast",
        true,
    ),
    ("Server.Items.WoodenCoffinAddon", "addonDeed", true),
    ("Server.Items.WoodenCoffinDeed", "deedAddon", true),
    ("Server.Items.WoodenCoffinDeed", "deedSelectionNumber", true),
];

fn world() -> World {
    let mut w = crate::test_world();
    muo_content::items::addon::register(&mut w.types);
    crate::addons::register(&mut w.types);
    crate::special::register(&mut w.types);
    crate::construction::register(&mut w.types);
    w
}

#[test]
fn converted_classes_keep_their_template_keys() {
    let mut w = crate::test_world();
    let t = templates::table(&mut w);
    for &(class, key, prop) in KEYS {
        let tmpl = t
            .get(Kind::Item, class)
            .unwrap_or_else(|| panic!("{class}: no template"));
        if prop {
            assert!(tmpl.props.contains_key(key), "{class}: no prop {key}");
        } else {
            assert!(
                tmpl.item_steps
                    .iter()
                    .any(|s| matches!(s, item_steps::Step::Ext(n, _, _) if *n == key)),
                "{class}: no key {key}"
            );
        }
    }
}

fn make(w: &mut World, t: &'static ItemType) -> muo_server::ids::ItemId {
    (t.construct.unwrap())(w)
}

fn comps(w: &World, a: muo_server::ids::ItemId) -> Vec<(i32, Point3D)> {
    addon::components(w, a)
        .into_iter()
        .map(|c| (w.item(c).unwrap().item_id, addon::component_offset(w, c)))
        .collect()
}

#[test]
fn direction_deeds_build_the_side_their_east_says() {
    let mut w = world();
    let deed = make(&mut w, &crate::special::addons::BROKEN_BED_DEED);
    assert_eq!(addon::deed_selection_number(&w, deed), Some(1076749));
    for (east, first) in [(true, 6293), (false, 6297)] {
        let a = crate::special::addons::broken_bed_addon_new(&mut w, east);
        assert_eq!(comps(&w, a)[0], (first, Point3D::new(0, 0, 0)));
        assert_eq!(comps(&w, a).len(), 4);
    }
    let v = crate::special::addons::vanity_addon_new(&mut w, true);
    assert_eq!(w.item(v).unwrap().item_id, 0xA44);
}

#[test]
fn hued_deeds_and_bloody_columns() {
    let mut w = world();
    let deed = make(&mut w, &crate::addons::leaves::GOZA_MAT_EAST_DEED);
    (w.item(deed).unwrap().ty.set_hue)(&mut w, deed, 33);
    let a = (addon::deed_class(&w, deed).addon)(&mut w, deed);
    assert_eq!(w.item(a).unwrap().hue, 33);
    assert!(addon::retain_deed_hue(&w, a));
    let col = crate::addons::leaves::fire_column_addon_new(&mut w, true);
    assert_eq!(comps(&w, col).len(), 15);
    assert!(!addon::share_hue(&w, col));
}

#[test]
fn pair_halves_keep_their_offsets() {
    use crate::construction::paired as p;
    let mut w = world();
    for (t, dx, dy) in [
        (&p::TAPESTRY1N, 1, 0),
        (&p::ANKH_WEST, 0, 1),
        (&p::DECORATIVE_SWORD_WEST, 0, -1),
    ] {
        let id = make(&mut w, t);
        let other = p::other(&w, id).unwrap();
        (w.item(id).unwrap().ty.set_location)(&mut w, id, Point3D::new(100, 100, 0));
        assert_eq!(
            w.item(other).unwrap().location,
            Point3D::new(100 + dx, 100 + dy, 0),
            "{}",
            t.full_name
        );
    }
}
