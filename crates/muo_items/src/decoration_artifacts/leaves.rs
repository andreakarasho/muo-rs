//! The Decoration Artifacts classes that are plain tables (a constructor and constant overrides); the classes with logic
//! are hand-written next to this file.

use crate::decoration_artifacts::DecoClass;
use crate::simple::leaf;
use muo_content::items::cont::new_base_container;
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::ItemType;
use muo_server::type_registry::TypeRegistry;

use crate::decoration_artifacts::BASE_DECORATION_ARTIFACT;
use crate::decoration_artifacts::BASE_DECORATION_CONTAINER_ARTIFACT;

/// `AcademicBooksArtifact()`.
pub fn academic_books_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.AcademicBooksArtifact")
}

static ACADEMIC_BOOKS_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 8 };
leaf! {
    /// `Server.Items.AcademicBooksArtifact`.
    ACADEMIC_BOOKS_ARTIFACT = "Server.Items.AcademicBooksArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| academic_books_artifact_new(w, &ACADEMIC_BOOKS_ARTIFACT)),
    family: Some(&ACADEMIC_BOOKS_ARTIFACT_CLASS),
    label_number: crate::class_props::label_number,
}

/// `BackpackArtifact()`.
pub fn backpack_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    new_base_container(w, ty, 0x9B2)
}

static BACKPACK_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 5 };
leaf! {
    /// `Server.Items.BackpackArtifact`.
    BACKPACK_ARTIFACT = "Server.Items.BackpackArtifact" : BASE_DECORATION_CONTAINER_ARTIFACT, enc;
    construct: Some(|w| backpack_artifact_new(w, &BACKPACK_ARTIFACT)),
    family: Some(&BACKPACK_ARTIFACT_CLASS),
}

/// `BloodyWaterArtifact()`.
pub fn bloody_water_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.BloodyWaterArtifact")
}

static BLOODY_WATER_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 5 };
leaf! {
    /// `Server.Items.BloodyWaterArtifact`.
    BLOODY_WATER_ARTIFACT = "Server.Items.BloodyWaterArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| bloody_water_artifact_new(w, &BLOODY_WATER_ARTIFACT)),
    family: Some(&BLOODY_WATER_ARTIFACT_CLASS),
}

/// `BooksFaceDownArtifact()`.
pub fn books_face_down_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.BooksFaceDownArtifact")
}

static BOOKS_FACE_DOWN_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 3 };
leaf! {
    /// `Server.Items.BooksFaceDownArtifact`.
    BOOKS_FACE_DOWN_ARTIFACT = "Server.Items.BooksFaceDownArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| books_face_down_artifact_new(w, &BOOKS_FACE_DOWN_ARTIFACT)),
    family: Some(&BOOKS_FACE_DOWN_ARTIFACT_CLASS),
}

/// `BooksNorthArtifact()`.
pub fn books_north_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.BooksNorthArtifact")
}

static BOOKS_NORTH_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 3 };
leaf! {
    /// `Server.Items.BooksNorthArtifact`.
    BOOKS_NORTH_ARTIFACT = "Server.Items.BooksNorthArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| books_north_artifact_new(w, &BOOKS_NORTH_ARTIFACT)),
    family: Some(&BOOKS_NORTH_ARTIFACT_CLASS),
}

/// `BooksWestArtifact()`.
pub fn books_west_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.BooksWestArtifact")
}

static BOOKS_WEST_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 3 };
leaf! {
    /// `Server.Items.BooksWestArtifact`.
    BOOKS_WEST_ARTIFACT = "Server.Items.BooksWestArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| books_west_artifact_new(w, &BOOKS_WEST_ARTIFACT)),
    family: Some(&BOOKS_WEST_ARTIFACT_CLASS),
}

/// `BottleArtifact()`.
pub fn bottle_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.BottleArtifact")
}

static BOTTLE_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 1 };
leaf! {
    /// `Server.Items.BottleArtifact`.
    BOTTLE_ARTIFACT = "Server.Items.BottleArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| bottle_artifact_new(w, &BOTTLE_ARTIFACT)),
    family: Some(&BOTTLE_ARTIFACT_CLASS),
}

/// `BrazierArtifact()`.
pub fn brazier_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.BrazierArtifact")
}

static BRAZIER_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 2 };
leaf! {
    /// `Server.Items.BrazierArtifact`.
    BRAZIER_ARTIFACT = "Server.Items.BrazierArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| brazier_artifact_new(w, &BRAZIER_ARTIFACT)),
    family: Some(&BRAZIER_ARTIFACT_CLASS),
}

/// `CocoonArtifact()`.
pub fn cocoon_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.CocoonArtifact")
}

static COCOON_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 7 };
leaf! {
    /// `Server.Items.CocoonArtifact`.
    COCOON_ARTIFACT = "Server.Items.CocoonArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| cocoon_artifact_new(w, &COCOON_ARTIFACT)),
    family: Some(&COCOON_ARTIFACT_CLASS),
}

/// `DamagedBooksArtifact()`.
pub fn damaged_books_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.DamagedBooksArtifact")
}

static DAMAGED_BOOKS_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 1 };
leaf! {
    /// `Server.Items.DamagedBooksArtifact`.
    DAMAGED_BOOKS_ARTIFACT = "Server.Items.DamagedBooksArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| damaged_books_artifact_new(w, &DAMAGED_BOOKS_ARTIFACT)),
    family: Some(&DAMAGED_BOOKS_ARTIFACT_CLASS),
}

/// `EggCaseArtifact()`.
pub fn egg_case_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.EggCaseArtifact")
}

static EGG_CASE_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 5 };
leaf! {
    /// `Server.Items.EggCaseArtifact`.
    EGG_CASE_ARTIFACT = "Server.Items.EggCaseArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| egg_case_artifact_new(w, &EGG_CASE_ARTIFACT)),
    family: Some(&EGG_CASE_ARTIFACT_CLASS),
}

/// `GruesomeStandardArtifact()`.
pub fn gruesome_standard_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.GruesomeStandardArtifact")
}

static GRUESOME_STANDARD_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 5 };
leaf! {
    /// `Server.Items.GruesomeStandardArtifact`.
    GRUESOME_STANDARD_ARTIFACT = "Server.Items.GruesomeStandardArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| gruesome_standard_artifact_new(w, &GRUESOME_STANDARD_ARTIFACT)),
    family: Some(&GRUESOME_STANDARD_ARTIFACT_CLASS),
}

/// `LampPostArtifact()`.
pub fn lamp_post_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.LampPostArtifact")
}

static LAMP_POST_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 3 };
leaf! {
    /// `Server.Items.LampPostArtifact`.
    LAMP_POST_ARTIFACT = "Server.Items.LampPostArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| lamp_post_artifact_new(w, &LAMP_POST_ARTIFACT)),
    family: Some(&LAMP_POST_ARTIFACT_CLASS),
}

/// `LeatherTunicArtifact()`.
pub fn leather_tunic_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.LeatherTunicArtifact")
}

static LEATHER_TUNIC_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 9 };
leaf! {
    /// `Server.Items.LeatherTunicArtifact`.
    LEATHER_TUNIC_ARTIFACT = "Server.Items.LeatherTunicArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| leather_tunic_artifact_new(w, &LEATHER_TUNIC_ARTIFACT)),
    family: Some(&LEATHER_TUNIC_ARTIFACT_CLASS),
}

/// `RockArtifact()`.
pub fn rock_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.RockArtifact")
}

static ROCK_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 1 };
leaf! {
    /// `Server.Items.RockArtifact`.
    ROCK_ARTIFACT = "Server.Items.RockArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| rock_artifact_new(w, &ROCK_ARTIFACT)),
    family: Some(&ROCK_ARTIFACT_CLASS),
}

/// `RuinedPaintingArtifact()`.
pub fn ruined_painting_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.RuinedPaintingArtifact")
}

static RUINED_PAINTING_ARTIFACT_CLASS: DecoClass = DecoClass {
    artifact_rarity: 12,
};
leaf! {
    /// `Server.Items.RuinedPaintingArtifact`.
    RUINED_PAINTING_ARTIFACT = "Server.Items.RuinedPaintingArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| ruined_painting_artifact_new(w, &RUINED_PAINTING_ARTIFACT)),
    family: Some(&RUINED_PAINTING_ARTIFACT_CLASS),
}

/// `SaddleArtifact()`.
pub fn saddle_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.SaddleArtifact")
}

static SADDLE_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 9 };
leaf! {
    /// `Server.Items.SaddleArtifact`.
    SADDLE_ARTIFACT = "Server.Items.SaddleArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| saddle_artifact_new(w, &SADDLE_ARTIFACT)),
    family: Some(&SADDLE_ARTIFACT_CLASS),
}

/// `SkinnedDeerArtifact()`.
pub fn skinned_deer_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.SkinnedDeerArtifact")
}

static SKINNED_DEER_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 8 };
leaf! {
    /// `Server.Items.SkinnedDeerArtifact`.
    SKINNED_DEER_ARTIFACT = "Server.Items.SkinnedDeerArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| skinned_deer_artifact_new(w, &SKINNED_DEER_ARTIFACT)),
    family: Some(&SKINNED_DEER_ARTIFACT_CLASS),
}

/// `SkinnedGoatArtifact()`.
pub fn skinned_goat_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.SkinnedGoatArtifact")
}

static SKINNED_GOAT_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 5 };
leaf! {
    /// `Server.Items.SkinnedGoatArtifact`.
    SKINNED_GOAT_ARTIFACT = "Server.Items.SkinnedGoatArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| skinned_goat_artifact_new(w, &SKINNED_GOAT_ARTIFACT)),
    family: Some(&SKINNED_GOAT_ARTIFACT_CLASS),
}

/// `SkullCandleArtifact()`.
pub fn skull_candle_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.SkullCandleArtifact")
}

static SKULL_CANDLE_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 1 };
leaf! {
    /// `Server.Items.SkullCandleArtifact`.
    SKULL_CANDLE_ARTIFACT = "Server.Items.SkullCandleArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| skull_candle_artifact_new(w, &SKULL_CANDLE_ARTIFACT)),
    family: Some(&SKULL_CANDLE_ARTIFACT_CLASS),
}

/// `StretchedHideArtifact()`.
pub fn stretched_hide_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.StretchedHideArtifact")
}

static STRETCHED_HIDE_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 2 };
leaf! {
    /// `Server.Items.StretchedHideArtifact`.
    STRETCHED_HIDE_ARTIFACT = "Server.Items.StretchedHideArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| stretched_hide_artifact_new(w, &STRETCHED_HIDE_ARTIFACT)),
    family: Some(&STRETCHED_HIDE_ARTIFACT_CLASS),
}

/// `StuddedLeggingsArtifact()`.
pub fn studded_leggings_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.StuddedLeggingsArtifact")
}

static STUDDED_LEGGINGS_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 5 };
leaf! {
    /// `Server.Items.StuddedLeggingsArtifact`.
    STUDDED_LEGGINGS_ARTIFACT = "Server.Items.StuddedLeggingsArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| studded_leggings_artifact_new(w, &STUDDED_LEGGINGS_ARTIFACT)),
    family: Some(&STUDDED_LEGGINGS_ARTIFACT_CLASS),
}

/// `StuddedTunicArtifact()`.
pub fn studded_tunic_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.StuddedTunicArtifact")
}

static STUDDED_TUNIC_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 7 };
leaf! {
    /// `Server.Items.StuddedTunicArtifact`.
    STUDDED_TUNIC_ARTIFACT = "Server.Items.StuddedTunicArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| studded_tunic_artifact_new(w, &STUDDED_TUNIC_ARTIFACT)),
    family: Some(&STUDDED_TUNIC_ARTIFACT_CLASS),
}

/// `TarotCardsArtifact()`.
pub fn tarot_cards_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.TarotCardsArtifact")
}

static TAROT_CARDS_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 5 };
leaf! {
    /// `Server.Items.TarotCardsArtifact`.
    TAROT_CARDS_ARTIFACT = "Server.Items.TarotCardsArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| tarot_cards_artifact_new(w, &TAROT_CARDS_ARTIFACT)),
    family: Some(&TAROT_CARDS_ARTIFACT_CLASS),
}

/// `Basket1Artifact()`.
pub fn basket1_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    new_base_container(w, ty, 0x24DD)
}

static BASKET1_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 1 };
leaf! {
    /// `Server.Items.Basket1Artifact`.
    BASKET1_ARTIFACT = "Server.Items.Basket1Artifact" : BASE_DECORATION_CONTAINER_ARTIFACT, enc;
    construct: Some(|w| basket1_artifact_new(w, &BASKET1_ARTIFACT)),
    family: Some(&BASKET1_ARTIFACT_CLASS),
}

/// `Basket2Artifact()`.
pub fn basket2_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    new_base_container(w, ty, 0x24D7)
}

static BASKET2_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 1 };
leaf! {
    /// `Server.Items.Basket2Artifact`.
    BASKET2_ARTIFACT = "Server.Items.Basket2Artifact" : BASE_DECORATION_CONTAINER_ARTIFACT, enc;
    construct: Some(|w| basket2_artifact_new(w, &BASKET2_ARTIFACT)),
    family: Some(&BASKET2_ARTIFACT_CLASS),
}

/// `Basket3NorthArtifact()`.
pub fn basket3_north_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    new_base_container(w, ty, 0x24DA)
}

static BASKET3_NORTH_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 1 };
leaf! {
    /// `Server.Items.Basket3NorthArtifact`.
    BASKET3_NORTH_ARTIFACT = "Server.Items.Basket3NorthArtifact" : BASE_DECORATION_CONTAINER_ARTIFACT, enc;
    construct: Some(|w| basket3_north_artifact_new(w, &BASKET3_NORTH_ARTIFACT)),
    family: Some(&BASKET3_NORTH_ARTIFACT_CLASS),
}

/// `Basket3WestArtifact()`.
pub fn basket3_west_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    new_base_container(w, ty, 0x24D9)
}

static BASKET3_WEST_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 1 };
leaf! {
    /// `Server.Items.Basket3WestArtifact`.
    BASKET3_WEST_ARTIFACT = "Server.Items.Basket3WestArtifact" : BASE_DECORATION_CONTAINER_ARTIFACT, enc;
    construct: Some(|w| basket3_west_artifact_new(w, &BASKET3_WEST_ARTIFACT)),
    family: Some(&BASKET3_WEST_ARTIFACT_CLASS),
}

/// `Basket4Artifact()`.
pub fn basket4_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    new_base_container(w, ty, 0x24D8)
}

static BASKET4_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 2 };
leaf! {
    /// `Server.Items.Basket4Artifact`.
    BASKET4_ARTIFACT = "Server.Items.Basket4Artifact" : BASE_DECORATION_CONTAINER_ARTIFACT, enc;
    construct: Some(|w| basket4_artifact_new(w, &BASKET4_ARTIFACT)),
    family: Some(&BASKET4_ARTIFACT_CLASS),
}

/// `Basket5NorthArtifact()`.
pub fn basket5_north_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    new_base_container(w, ty, 0x24DB)
}

static BASKET5_NORTH_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 2 };
leaf! {
    /// `Server.Items.Basket5NorthArtifact`.
    BASKET5_NORTH_ARTIFACT = "Server.Items.Basket5NorthArtifact" : BASE_DECORATION_CONTAINER_ARTIFACT, enc;
    construct: Some(|w| basket5_north_artifact_new(w, &BASKET5_NORTH_ARTIFACT)),
    family: Some(&BASKET5_NORTH_ARTIFACT_CLASS),
}

/// `Basket5WestArtifact()`.
pub fn basket5_west_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    new_base_container(w, ty, 0x24DC)
}

static BASKET5_WEST_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 2 };
leaf! {
    /// `Server.Items.Basket5WestArtifact`.
    BASKET5_WEST_ARTIFACT = "Server.Items.Basket5WestArtifact" : BASE_DECORATION_CONTAINER_ARTIFACT, enc;
    construct: Some(|w| basket5_west_artifact_new(w, &BASKET5_WEST_ARTIFACT)),
    family: Some(&BASKET5_WEST_ARTIFACT_CLASS),
}

/// `Basket6Artifact()`.
pub fn basket6_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    new_base_container(w, ty, 0x24D5)
}

static BASKET6_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 2 };
leaf! {
    /// `Server.Items.Basket6Artifact`.
    BASKET6_ARTIFACT = "Server.Items.Basket6Artifact" : BASE_DECORATION_CONTAINER_ARTIFACT, enc;
    construct: Some(|w| basket6_artifact_new(w, &BASKET6_ARTIFACT)),
    family: Some(&BASKET6_ARTIFACT_CLASS),
}

/// `BowlArtifact()`.
pub fn bowl_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.BowlArtifact")
}

static BOWL_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 4 };
leaf! {
    /// `Server.Items.BowlArtifact`.
    BOWL_ARTIFACT = "Server.Items.BowlArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| bowl_artifact_new(w, &BOWL_ARTIFACT)),
    family: Some(&BOWL_ARTIFACT_CLASS),
}

/// `BowlsHorizontalArtifact()`.
pub fn bowls_horizontal_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.BowlsHorizontalArtifact")
}

static BOWLS_HORIZONTAL_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 4 };
leaf! {
    /// `Server.Items.BowlsHorizontalArtifact`.
    BOWLS_HORIZONTAL_ARTIFACT = "Server.Items.BowlsHorizontalArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| bowls_horizontal_artifact_new(w, &BOWLS_HORIZONTAL_ARTIFACT)),
    family: Some(&BOWLS_HORIZONTAL_ARTIFACT_CLASS),
}

/// `BowlsVerticalArtifact()`.
pub fn bowls_vertical_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.BowlsVerticalArtifact")
}

static BOWLS_VERTICAL_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 3 };
leaf! {
    /// `Server.Items.BowlsVerticalArtifact`.
    BOWLS_VERTICAL_ARTIFACT = "Server.Items.BowlsVerticalArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| bowls_vertical_artifact_new(w, &BOWLS_VERTICAL_ARTIFACT)),
    family: Some(&BOWLS_VERTICAL_ARTIFACT_CLASS),
}

/// `CupsArtifact()`.
pub fn cups_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.CupsArtifact")
}

static CUPS_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 4 };
leaf! {
    /// `Server.Items.CupsArtifact`.
    CUPS_ARTIFACT = "Server.Items.CupsArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| cups_artifact_new(w, &CUPS_ARTIFACT)),
    family: Some(&CUPS_ARTIFACT_CLASS),
}

/// `DolphinLeftArtifact()`.
pub fn dolphin_left_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.DolphinLeftArtifact")
}

static DOLPHIN_LEFT_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 8 };
leaf! {
    /// `Server.Items.DolphinLeftArtifact`.
    DOLPHIN_LEFT_ARTIFACT = "Server.Items.DolphinLeftArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| dolphin_left_artifact_new(w, &DOLPHIN_LEFT_ARTIFACT)),
    family: Some(&DOLPHIN_LEFT_ARTIFACT_CLASS),
}

/// `DolphinRightArtifact()`.
pub fn dolphin_right_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.DolphinRightArtifact")
}

static DOLPHIN_RIGHT_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 8 };
leaf! {
    /// `Server.Items.DolphinRightArtifact`.
    DOLPHIN_RIGHT_ARTIFACT = "Server.Items.DolphinRightArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| dolphin_right_artifact_new(w, &DOLPHIN_RIGHT_ARTIFACT)),
    family: Some(&DOLPHIN_RIGHT_ARTIFACT_CLASS),
}

/// `FanNorthArtifact()`.
pub fn fan_north_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.FanNorthArtifact")
}

static FAN_NORTH_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 3 };
leaf! {
    /// `Server.Items.FanNorthArtifact`.
    FAN_NORTH_ARTIFACT = "Server.Items.FanNorthArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| fan_north_artifact_new(w, &FAN_NORTH_ARTIFACT)),
    family: Some(&FAN_NORTH_ARTIFACT_CLASS),
}

/// `FanWestArtifact()`.
pub fn fan_west_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.FanWestArtifact")
}

static FAN_WEST_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 3 };
leaf! {
    /// `Server.Items.FanWestArtifact`.
    FAN_WEST_ARTIFACT = "Server.Items.FanWestArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| fan_west_artifact_new(w, &FAN_WEST_ARTIFACT)),
    family: Some(&FAN_WEST_ARTIFACT_CLASS),
}

/// `FlowersArtifact()`.
pub fn flowers_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.FlowersArtifact")
}

static FLOWERS_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 7 };
leaf! {
    /// `Server.Items.FlowersArtifact`.
    FLOWERS_ARTIFACT = "Server.Items.FlowersArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| flowers_artifact_new(w, &FLOWERS_ARTIFACT)),
    family: Some(&FLOWERS_ARTIFACT_CLASS),
}

/// `ManStatuetteEastArtifact()`.
pub fn man_statuette_east_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.ManStatuetteEastArtifact")
}

static MAN_STATUETTE_EAST_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 9 };
leaf! {
    /// `Server.Items.ManStatuetteEastArtifact`.
    MAN_STATUETTE_EAST_ARTIFACT = "Server.Items.ManStatuetteEastArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| man_statuette_east_artifact_new(w, &MAN_STATUETTE_EAST_ARTIFACT)),
    family: Some(&MAN_STATUETTE_EAST_ARTIFACT_CLASS),
}

/// `ManStatuetteSouthArtifact()`.
pub fn man_statuette_south_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.ManStatuetteSouthArtifact")
}

static MAN_STATUETTE_SOUTH_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 9 };
leaf! {
    /// `Server.Items.ManStatuetteSouthArtifact`.
    MAN_STATUETTE_SOUTH_ARTIFACT = "Server.Items.ManStatuetteSouthArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| man_statuette_south_artifact_new(w, &MAN_STATUETTE_SOUTH_ARTIFACT)),
    family: Some(&MAN_STATUETTE_SOUTH_ARTIFACT_CLASS),
}

/// `Painting1NorthArtifact()`.
pub fn painting1_north_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.Painting1NorthArtifact")
}

static PAINTING1_NORTH_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 4 };
leaf! {
    /// `Server.Items.Painting1NorthArtifact`.
    PAINTING1_NORTH_ARTIFACT = "Server.Items.Painting1NorthArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| painting1_north_artifact_new(w, &PAINTING1_NORTH_ARTIFACT)),
    family: Some(&PAINTING1_NORTH_ARTIFACT_CLASS),
}

/// `Painting1WestArtifact()`.
pub fn painting1_west_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.Painting1WestArtifact")
}

static PAINTING1_WEST_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 4 };
leaf! {
    /// `Server.Items.Painting1WestArtifact`.
    PAINTING1_WEST_ARTIFACT = "Server.Items.Painting1WestArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| painting1_west_artifact_new(w, &PAINTING1_WEST_ARTIFACT)),
    family: Some(&PAINTING1_WEST_ARTIFACT_CLASS),
}

/// `Painting2NorthArtifact()`.
pub fn painting2_north_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.Painting2NorthArtifact")
}

static PAINTING2_NORTH_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 4 };
leaf! {
    /// `Server.Items.Painting2NorthArtifact`.
    PAINTING2_NORTH_ARTIFACT = "Server.Items.Painting2NorthArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| painting2_north_artifact_new(w, &PAINTING2_NORTH_ARTIFACT)),
    family: Some(&PAINTING2_NORTH_ARTIFACT_CLASS),
}

/// `Painting2WestArtifact()`.
pub fn painting2_west_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.Painting2WestArtifact")
}

static PAINTING2_WEST_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 4 };
leaf! {
    /// `Server.Items.Painting2WestArtifact`.
    PAINTING2_WEST_ARTIFACT = "Server.Items.Painting2WestArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| painting2_west_artifact_new(w, &PAINTING2_WEST_ARTIFACT)),
    family: Some(&PAINTING2_WEST_ARTIFACT_CLASS),
}

/// `Painting3Artifact()`.
pub fn painting3_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.Painting3Artifact")
}

static PAINTING3_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 5 };
leaf! {
    /// `Server.Items.Painting3Artifact`.
    PAINTING3_ARTIFACT = "Server.Items.Painting3Artifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| painting3_artifact_new(w, &PAINTING3_ARTIFACT)),
    family: Some(&PAINTING3_ARTIFACT_CLASS),
}

/// `Painting4NorthArtifact()`.
pub fn painting4_north_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.Painting4NorthArtifact")
}

static PAINTING4_NORTH_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 6 };
leaf! {
    /// `Server.Items.Painting4NorthArtifact`.
    PAINTING4_NORTH_ARTIFACT = "Server.Items.Painting4NorthArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| painting4_north_artifact_new(w, &PAINTING4_NORTH_ARTIFACT)),
    family: Some(&PAINTING4_NORTH_ARTIFACT_CLASS),
}

/// `Painting4WestArtifact()`.
pub fn painting4_west_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.Painting4WestArtifact")
}

static PAINTING4_WEST_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 6 };
leaf! {
    /// `Server.Items.Painting4WestArtifact`.
    PAINTING4_WEST_ARTIFACT = "Server.Items.Painting4WestArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| painting4_west_artifact_new(w, &PAINTING4_WEST_ARTIFACT)),
    family: Some(&PAINTING4_WEST_ARTIFACT_CLASS),
}

/// `Painting5NorthArtifact()`.
pub fn painting5_north_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.Painting5NorthArtifact")
}

static PAINTING5_NORTH_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 8 };
leaf! {
    /// `Server.Items.Painting5NorthArtifact`.
    PAINTING5_NORTH_ARTIFACT = "Server.Items.Painting5NorthArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| painting5_north_artifact_new(w, &PAINTING5_NORTH_ARTIFACT)),
    family: Some(&PAINTING5_NORTH_ARTIFACT_CLASS),
}

/// `Painting5WestArtifact()`.
pub fn painting5_west_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.Painting5WestArtifact")
}

static PAINTING5_WEST_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 8 };
leaf! {
    /// `Server.Items.Painting5WestArtifact`.
    PAINTING5_WEST_ARTIFACT = "Server.Items.Painting5WestArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| painting5_west_artifact_new(w, &PAINTING5_WEST_ARTIFACT)),
    family: Some(&PAINTING5_WEST_ARTIFACT_CLASS),
}

/// `Painting6NorthArtifact()`.
pub fn painting6_north_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.Painting6NorthArtifact")
}

static PAINTING6_NORTH_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 9 };
leaf! {
    /// `Server.Items.Painting6NorthArtifact`.
    PAINTING6_NORTH_ARTIFACT = "Server.Items.Painting6NorthArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| painting6_north_artifact_new(w, &PAINTING6_NORTH_ARTIFACT)),
    family: Some(&PAINTING6_NORTH_ARTIFACT_CLASS),
}

/// `Painting6WestArtifact()`.
pub fn painting6_west_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.Painting6WestArtifact")
}

static PAINTING6_WEST_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 9 };
leaf! {
    /// `Server.Items.Painting6WestArtifact`.
    PAINTING6_WEST_ARTIFACT = "Server.Items.Painting6WestArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| painting6_west_artifact_new(w, &PAINTING6_WEST_ARTIFACT)),
    family: Some(&PAINTING6_WEST_ARTIFACT_CLASS),
}

/// `SakeArtifact()`.
pub fn sake_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.SakeArtifact")
}

static SAKE_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 4 };
leaf! {
    /// `Server.Items.SakeArtifact`.
    SAKE_ARTIFACT = "Server.Items.SakeArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| sake_artifact_new(w, &SAKE_ARTIFACT)),
    family: Some(&SAKE_ARTIFACT_CLASS),
}

/// `Sculpture1Artifact()`.
pub fn sculpture1_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.Sculpture1Artifact")
}

static SCULPTURE1_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 3 };
leaf! {
    /// `Server.Items.Sculpture1Artifact`.
    SCULPTURE1_ARTIFACT = "Server.Items.Sculpture1Artifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| sculpture1_artifact_new(w, &SCULPTURE1_ARTIFACT)),
    family: Some(&SCULPTURE1_ARTIFACT_CLASS),
}

/// `Sculpture2Artifact()`.
pub fn sculpture2_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.Sculpture2Artifact")
}

static SCULPTURE2_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 3 };
leaf! {
    /// `Server.Items.Sculpture2Artifact`.
    SCULPTURE2_ARTIFACT = "Server.Items.Sculpture2Artifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| sculpture2_artifact_new(w, &SCULPTURE2_ARTIFACT)),
    family: Some(&SCULPTURE2_ARTIFACT_CLASS),
}

/// `SwordDisplay1NorthArtifact()`.
pub fn sword_display1_north_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.SwordDisplay1NorthArtifact")
}

static SWORD_DISPLAY1_NORTH_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 5 };
leaf! {
    /// `Server.Items.SwordDisplay1NorthArtifact`.
    SWORD_DISPLAY1_NORTH_ARTIFACT = "Server.Items.SwordDisplay1NorthArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| sword_display1_north_artifact_new(w, &SWORD_DISPLAY1_NORTH_ARTIFACT)),
    family: Some(&SWORD_DISPLAY1_NORTH_ARTIFACT_CLASS),
}

/// `SwordDisplay1WestArtifact()`.
pub fn sword_display1_west_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.SwordDisplay1WestArtifact")
}

static SWORD_DISPLAY1_WEST_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 5 };
leaf! {
    /// `Server.Items.SwordDisplay1WestArtifact`.
    SWORD_DISPLAY1_WEST_ARTIFACT = "Server.Items.SwordDisplay1WestArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| sword_display1_west_artifact_new(w, &SWORD_DISPLAY1_WEST_ARTIFACT)),
    family: Some(&SWORD_DISPLAY1_WEST_ARTIFACT_CLASS),
}

/// `SwordDisplay2NorthArtifact()`.
pub fn sword_display2_north_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.SwordDisplay2NorthArtifact")
}

static SWORD_DISPLAY2_NORTH_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 6 };
leaf! {
    /// `Server.Items.SwordDisplay2NorthArtifact`.
    SWORD_DISPLAY2_NORTH_ARTIFACT = "Server.Items.SwordDisplay2NorthArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| sword_display2_north_artifact_new(w, &SWORD_DISPLAY2_NORTH_ARTIFACT)),
    family: Some(&SWORD_DISPLAY2_NORTH_ARTIFACT_CLASS),
}

/// `SwordDisplay2WestArtifact()`.
pub fn sword_display2_west_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.SwordDisplay2WestArtifact")
}

static SWORD_DISPLAY2_WEST_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 6 };
leaf! {
    /// `Server.Items.SwordDisplay2WestArtifact`.
    SWORD_DISPLAY2_WEST_ARTIFACT = "Server.Items.SwordDisplay2WestArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| sword_display2_west_artifact_new(w, &SWORD_DISPLAY2_WEST_ARTIFACT)),
    family: Some(&SWORD_DISPLAY2_WEST_ARTIFACT_CLASS),
}

/// `SwordDisplay3EastArtifact()`.
pub fn sword_display3_east_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.SwordDisplay3EastArtifact")
}

static SWORD_DISPLAY3_EAST_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 8 };
leaf! {
    /// `Server.Items.SwordDisplay3EastArtifact`.
    SWORD_DISPLAY3_EAST_ARTIFACT = "Server.Items.SwordDisplay3EastArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| sword_display3_east_artifact_new(w, &SWORD_DISPLAY3_EAST_ARTIFACT)),
    family: Some(&SWORD_DISPLAY3_EAST_ARTIFACT_CLASS),
}

/// `SwordDisplay3SouthArtifact()`.
pub fn sword_display3_south_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.SwordDisplay3SouthArtifact")
}

static SWORD_DISPLAY3_SOUTH_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 8 };
leaf! {
    /// `Server.Items.SwordDisplay3SouthArtifact`.
    SWORD_DISPLAY3_SOUTH_ARTIFACT = "Server.Items.SwordDisplay3SouthArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| sword_display3_south_artifact_new(w, &SWORD_DISPLAY3_SOUTH_ARTIFACT)),
    family: Some(&SWORD_DISPLAY3_SOUTH_ARTIFACT_CLASS),
}

/// `SwordDisplay4NorthArtifact()`.
pub fn sword_display4_north_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.SwordDisplay4NorthArtifact")
}

static SWORD_DISPLAY4_NORTH_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 9 };
leaf! {
    /// `Server.Items.SwordDisplay4NorthArtifact`.
    SWORD_DISPLAY4_NORTH_ARTIFACT = "Server.Items.SwordDisplay4NorthArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| sword_display4_north_artifact_new(w, &SWORD_DISPLAY4_NORTH_ARTIFACT)),
    family: Some(&SWORD_DISPLAY4_NORTH_ARTIFACT_CLASS),
}

/// `SwordDisplay4WestArtifact()`.
pub fn sword_display4_west_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.SwordDisplay4WestArtifact")
}

static SWORD_DISPLAY4_WEST_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 8 };
leaf! {
    /// `Server.Items.SwordDisplay4WestArtifact`.
    SWORD_DISPLAY4_WEST_ARTIFACT = "Server.Items.SwordDisplay4WestArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| sword_display4_west_artifact_new(w, &SWORD_DISPLAY4_WEST_ARTIFACT)),
    family: Some(&SWORD_DISPLAY4_WEST_ARTIFACT_CLASS),
}

/// `SwordDisplay5NorthArtifact()`.
pub fn sword_display5_north_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.SwordDisplay5NorthArtifact")
}

static SWORD_DISPLAY5_NORTH_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 9 };
leaf! {
    /// `Server.Items.SwordDisplay5NorthArtifact`.
    SWORD_DISPLAY5_NORTH_ARTIFACT = "Server.Items.SwordDisplay5NorthArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| sword_display5_north_artifact_new(w, &SWORD_DISPLAY5_NORTH_ARTIFACT)),
    family: Some(&SWORD_DISPLAY5_NORTH_ARTIFACT_CLASS),
}

/// `SwordDisplay5WestArtifact()`.
pub fn sword_display5_west_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.SwordDisplay5WestArtifact")
}

static SWORD_DISPLAY5_WEST_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 9 };
leaf! {
    /// `Server.Items.SwordDisplay5WestArtifact`.
    SWORD_DISPLAY5_WEST_ARTIFACT = "Server.Items.SwordDisplay5WestArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| sword_display5_west_artifact_new(w, &SWORD_DISPLAY5_WEST_ARTIFACT)),
    family: Some(&SWORD_DISPLAY5_WEST_ARTIFACT_CLASS),
}

/// `TeapotNorthArtifact()`.
pub fn teapot_north_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.TeapotNorthArtifact")
}

static TEAPOT_NORTH_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 3 };
leaf! {
    /// `Server.Items.TeapotNorthArtifact`.
    TEAPOT_NORTH_ARTIFACT = "Server.Items.TeapotNorthArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| teapot_north_artifact_new(w, &TEAPOT_NORTH_ARTIFACT)),
    family: Some(&TEAPOT_NORTH_ARTIFACT_CLASS),
}

/// `TeapotWestArtifact()`.
pub fn teapot_west_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.TeapotWestArtifact")
}

static TEAPOT_WEST_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 3 };
leaf! {
    /// `Server.Items.TeapotWestArtifact`.
    TEAPOT_WEST_ARTIFACT = "Server.Items.TeapotWestArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| teapot_west_artifact_new(w, &TEAPOT_WEST_ARTIFACT)),
    family: Some(&TEAPOT_WEST_ARTIFACT_CLASS),
}

/// `TripleFanNorthArtifact()`.
pub fn triple_fan_north_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.TripleFanNorthArtifact")
}

static TRIPLE_FAN_NORTH_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 4 };
leaf! {
    /// `Server.Items.TripleFanNorthArtifact`.
    TRIPLE_FAN_NORTH_ARTIFACT = "Server.Items.TripleFanNorthArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| triple_fan_north_artifact_new(w, &TRIPLE_FAN_NORTH_ARTIFACT)),
    family: Some(&TRIPLE_FAN_NORTH_ARTIFACT_CLASS),
}

/// `TripleFanWestArtifact()`.
pub fn triple_fan_west_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.TripleFanWestArtifact")
}

static TRIPLE_FAN_WEST_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 4 };
leaf! {
    /// `Server.Items.TripleFanWestArtifact`.
    TRIPLE_FAN_WEST_ARTIFACT = "Server.Items.TripleFanWestArtifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| triple_fan_west_artifact_new(w, &TRIPLE_FAN_WEST_ARTIFACT)),
    family: Some(&TRIPLE_FAN_WEST_ARTIFACT_CLASS),
}

/// `Urn1Artifact()`.
pub fn urn1_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.Urn1Artifact")
}

static URN1_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 3 };
leaf! {
    /// `Server.Items.Urn1Artifact`.
    URN1_ARTIFACT = "Server.Items.Urn1Artifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| urn1_artifact_new(w, &URN1_ARTIFACT)),
    family: Some(&URN1_ARTIFACT_CLASS),
}

/// `Urn2Artifact()`.
pub fn urn2_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.Urn2Artifact")
}

static URN2_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 3 };
leaf! {
    /// `Server.Items.Urn2Artifact`.
    URN2_ARTIFACT = "Server.Items.Urn2Artifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| urn2_artifact_new(w, &URN2_ARTIFACT)),
    family: Some(&URN2_ARTIFACT_CLASS),
}

/// `ZenRock1Artifact()`.
pub fn zen_rock1_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.ZenRock1Artifact")
}

static ZEN_ROCK1_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 2 };
leaf! {
    /// `Server.Items.ZenRock1Artifact`.
    ZEN_ROCK1_ARTIFACT = "Server.Items.ZenRock1Artifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| zen_rock1_artifact_new(w, &ZEN_ROCK1_ARTIFACT)),
    family: Some(&ZEN_ROCK1_ARTIFACT_CLASS),
}

/// `ZenRock2Artifact()`.
pub fn zen_rock2_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.ZenRock2Artifact")
}

static ZEN_ROCK2_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 3 };
leaf! {
    /// `Server.Items.ZenRock2Artifact`.
    ZEN_ROCK2_ARTIFACT = "Server.Items.ZenRock2Artifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| zen_rock2_artifact_new(w, &ZEN_ROCK2_ARTIFACT)),
    family: Some(&ZEN_ROCK2_ARTIFACT_CLASS),
}

/// `ZenRock3Artifact()`.
pub fn zen_rock3_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.ZenRock3Artifact")
}

static ZEN_ROCK3_ARTIFACT_CLASS: DecoClass = DecoClass { artifact_rarity: 3 };
leaf! {
    /// `Server.Items.ZenRock3Artifact`.
    ZEN_ROCK3_ARTIFACT = "Server.Items.ZenRock3Artifact" : BASE_DECORATION_ARTIFACT, enc;
    construct: Some(|w| zen_rock3_artifact_new(w, &ZEN_ROCK3_ARTIFACT)),
    family: Some(&ZEN_ROCK3_ARTIFACT_CLASS),
}

/// Every class of this file: the type and its `[TypeAlias]` names.
pub static ALL: &[(&ItemType, &[&str])] = &[
    (&ACADEMIC_BOOKS_ARTIFACT, &[]),
    (&BACKPACK_ARTIFACT, &[]),
    (&BLOODY_WATER_ARTIFACT, &[]),
    (&BOOKS_FACE_DOWN_ARTIFACT, &[]),
    (&BOOKS_NORTH_ARTIFACT, &[]),
    (&BOOKS_WEST_ARTIFACT, &[]),
    (&BOTTLE_ARTIFACT, &[]),
    (&BRAZIER_ARTIFACT, &[]),
    (&COCOON_ARTIFACT, &[]),
    (&DAMAGED_BOOKS_ARTIFACT, &[]),
    (&EGG_CASE_ARTIFACT, &[]),
    (&GRUESOME_STANDARD_ARTIFACT, &[]),
    (&LAMP_POST_ARTIFACT, &[]),
    (&LEATHER_TUNIC_ARTIFACT, &[]),
    (&ROCK_ARTIFACT, &[]),
    (&RUINED_PAINTING_ARTIFACT, &[]),
    (&SADDLE_ARTIFACT, &[]),
    (&SKINNED_DEER_ARTIFACT, &[]),
    (&SKINNED_GOAT_ARTIFACT, &[]),
    (&SKULL_CANDLE_ARTIFACT, &[]),
    (&STRETCHED_HIDE_ARTIFACT, &[]),
    (&STUDDED_LEGGINGS_ARTIFACT, &[]),
    (&STUDDED_TUNIC_ARTIFACT, &[]),
    (&TAROT_CARDS_ARTIFACT, &[]),
    (&BASKET1_ARTIFACT, &[]),
    (&BASKET2_ARTIFACT, &[]),
    (&BASKET3_NORTH_ARTIFACT, &[]),
    (&BASKET3_WEST_ARTIFACT, &[]),
    (&BASKET4_ARTIFACT, &[]),
    (&BASKET5_NORTH_ARTIFACT, &[]),
    (&BASKET5_WEST_ARTIFACT, &[]),
    (&BASKET6_ARTIFACT, &[]),
    (&BOWL_ARTIFACT, &[]),
    (&BOWLS_HORIZONTAL_ARTIFACT, &[]),
    (&BOWLS_VERTICAL_ARTIFACT, &[]),
    (&CUPS_ARTIFACT, &[]),
    (&DOLPHIN_LEFT_ARTIFACT, &[]),
    (&DOLPHIN_RIGHT_ARTIFACT, &[]),
    (&FAN_NORTH_ARTIFACT, &[]),
    (&FAN_WEST_ARTIFACT, &[]),
    (&FLOWERS_ARTIFACT, &[]),
    (&MAN_STATUETTE_EAST_ARTIFACT, &[]),
    (&MAN_STATUETTE_SOUTH_ARTIFACT, &[]),
    (&PAINTING1_NORTH_ARTIFACT, &[]),
    (&PAINTING1_WEST_ARTIFACT, &[]),
    (&PAINTING2_NORTH_ARTIFACT, &[]),
    (&PAINTING2_WEST_ARTIFACT, &[]),
    (&PAINTING3_ARTIFACT, &[]),
    (&PAINTING4_NORTH_ARTIFACT, &[]),
    (&PAINTING4_WEST_ARTIFACT, &[]),
    (&PAINTING5_NORTH_ARTIFACT, &[]),
    (&PAINTING5_WEST_ARTIFACT, &[]),
    (&PAINTING6_NORTH_ARTIFACT, &[]),
    (&PAINTING6_WEST_ARTIFACT, &[]),
    (&SAKE_ARTIFACT, &[]),
    (&SCULPTURE1_ARTIFACT, &[]),
    (&SCULPTURE2_ARTIFACT, &[]),
    (&SWORD_DISPLAY1_NORTH_ARTIFACT, &[]),
    (&SWORD_DISPLAY1_WEST_ARTIFACT, &[]),
    (&SWORD_DISPLAY2_NORTH_ARTIFACT, &[]),
    (&SWORD_DISPLAY2_WEST_ARTIFACT, &[]),
    (&SWORD_DISPLAY3_EAST_ARTIFACT, &[]),
    (&SWORD_DISPLAY3_SOUTH_ARTIFACT, &[]),
    (&SWORD_DISPLAY4_NORTH_ARTIFACT, &[]),
    (&SWORD_DISPLAY4_WEST_ARTIFACT, &[]),
    (&SWORD_DISPLAY5_NORTH_ARTIFACT, &[]),
    (&SWORD_DISPLAY5_WEST_ARTIFACT, &[]),
    (&TEAPOT_NORTH_ARTIFACT, &[]),
    (&TEAPOT_WEST_ARTIFACT, &[]),
    (&TRIPLE_FAN_NORTH_ARTIFACT, &[]),
    (&TRIPLE_FAN_WEST_ARTIFACT, &[]),
    (&URN1_ARTIFACT, &[]),
    (&URN2_ARTIFACT, &[]),
    (&ZEN_ROCK1_ARTIFACT, &[]),
    (&ZEN_ROCK2_ARTIFACT, &[]),
    (&ZEN_ROCK3_ARTIFACT, &[]),
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
