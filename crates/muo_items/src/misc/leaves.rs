//! The Misc classes that are a constructor and constants. Classes with logic of their own are hand-written next to
//! this file.

use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::{ITEM_BASE, ItemType, ops};
use muo_server::type_registry::TypeRegistry;

use super::{hue, random_element, random_min_max};

/// `new GlobOfMonstreousInterredGrizzle()` of class `ty`.
pub fn glob_of_monstreous_interred_grizzle_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.GlobOfMonstreousInterredGrizzle")
}

item_leaf! {
    /// `Server.Items.GlobOfMonstreousInterredGrizzle`.
    GLOB_OF_MONSTREOUS_INTERRED_GRIZZLE = "Server.Items.GlobOfMonstreousInterredGrizzle" : ITEM_BASE, |w| glob_of_monstreous_interred_grizzle_new(w, &GLOB_OF_MONSTREOUS_INTERRED_GRIZZLE), encoded false;
    label_number: crate::class_props::label_number,
}

/// `new GrizzledSkullCollection()` of class `ty`.
pub fn grizzled_skull_collection_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.GrizzledSkullCollection")
}

item_leaf! {
    /// `Server.Items.GrizzledSkullCollection`.
    GRIZZLED_SKULL_COLLECTION = "Server.Items.GrizzledSkullCollection" : ITEM_BASE, |w| grizzled_skull_collection_new(w, &GRIZZLED_SKULL_COLLECTION), encoded false;
    label_number: crate::class_props::label_number,
}

/// `new MonsterousInterredGrizzleMaggots()` of class `ty`.
pub fn monsterous_interred_grizzle_maggots_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.MonsterousInterredGrizzleMaggots")
}

item_leaf! {
    /// `Server.Items.MonsterousInterredGrizzleMaggots`.
    MONSTEROUS_INTERRED_GRIZZLE_MAGGOTS = "Server.Items.MonsterousInterredGrizzleMaggots" : ITEM_BASE, |w| monsterous_interred_grizzle_maggots_new(w, &MONSTEROUS_INTERRED_GRIZZLE_MAGGOTS), encoded false;
    label_number: crate::class_props::label_number,
}

/// `new ResolvesBridle()` of class `ty`.
pub fn resolves_bridle_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.ResolvesBridle")
}

item_leaf! {
    /// `Server.Items.ResolvesBridle`.
    RESOLVES_BRIDLE = "Server.Items.ResolvesBridle" : ITEM_BASE, |w| resolves_bridle_new(w, &RESOLVES_BRIDLE), encoded false;
    label_number: crate::class_props::label_number,
}

/// `new TombstoneOfTheDamned()` of class `ty`.
pub fn tombstone_of_the_damned_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.TombstoneOfTheDamned")
}

item_leaf! {
    /// `Server.Items.TombstoneOfTheDamned`.
    TOMBSTONE_OF_THE_DAMNED = "Server.Items.TombstoneOfTheDamned" : ITEM_BASE, |w| tombstone_of_the_damned_new(w, &TOMBSTONE_OF_THE_DAMNED), encoded false;
    label_number: crate::class_props::label_number,
}

/// `new Beeswax(amount = 1)` of class `ty`.
pub fn beeswax_new(w: &mut World, ty: &'static ItemType, amount: i32) -> ItemId {
    let id = crate::simple::from_template(w, ty, "Server.Items.Beeswax");
    ops::set_amount(w, id, amount);
    id
}

item_leaf! {
    /// `Server.Items.Beeswax`.
    BEESWAX = "Server.Items.Beeswax" : ITEM_BASE, |w| beeswax_new(w, &BEESWAX, 1), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new AbscessTail()` of class `ty`.
pub fn abscess_tail_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.AbscessTail")
}

item_leaf! {
    /// `Server.Items.AbscessTail`.
    ABSCESS_TAIL = "Server.Items.AbscessTail" : ITEM_BASE, |w| abscess_tail_new(w, &ABSCESS_TAIL), encoded false;
    label_number: crate::class_props::label_number,
}

/// `new CoilsFang()` of class `ty`.
pub fn coils_fang_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.CoilsFang")
}

item_leaf! {
    /// `Server.Items.CoilsFang`.
    COILS_FANG = "Server.Items.CoilsFang" : ITEM_BASE, |w| coils_fang_new(w, &COILS_FANG), encoded false;
    label_number: crate::class_props::label_number,
}

/// `new EternallyCorruptTree()` of class `ty`.
pub fn eternally_corrupt_tree_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = crate::simple::from_template(w, ty, "Server.Items.EternallyCorruptTree");
    let v = random_min_max(w, 0x899, 0x8B0);
    hue(w, id, v);
    id
}

item_leaf! {
    /// `Server.Items.EternallyCorruptTree`.
    ETERNALLY_CORRUPT_TREE = "Server.Items.EternallyCorruptTree" : ITEM_BASE, |w| eternally_corrupt_tree_new(w, &ETERNALLY_CORRUPT_TREE), encoded false;
    label_number: crate::class_props::label_number,
}

/// `new HydraScale()` of class `ty`.
pub fn hydra_scale_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.HydraScale")
}

item_leaf! {
    /// `Server.Items.HydraScale`.
    HYDRA_SCALE = "Server.Items.HydraScale" : ITEM_BASE, |w| hydra_scale_new(w, &HYDRA_SCALE), encoded false;
    label_number: crate::class_props::label_number,
}

/// `new SalivasFeather()` of class `ty`.
pub fn salivas_feather_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.SalivasFeather")
}

item_leaf! {
    /// `Server.Items.SalivasFeather`.
    SALIVAS_FEATHER = "Server.Items.SalivasFeather" : ITEM_BASE, |w| salivas_feather_new(w, &SALIVAS_FEATHER), encoded false;
    label_number: crate::class_props::label_number,
}

/// `new SamplesOfCorruptedWater()` of class `ty`.
pub fn samples_of_corrupted_water_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.SamplesOfCorruptedWater")
}

item_leaf! {
    /// `Server.Items.SamplesOfCorruptedWater`.
    SAMPLES_OF_CORRUPTED_WATER = "Server.Items.SamplesOfCorruptedWater" : ITEM_BASE, |w| samples_of_corrupted_water_new(w, &SAMPLES_OF_CORRUPTED_WATER), encoded false;
    label_number: crate::class_props::label_number,
}

/// `new TaintedSeeds()` of class `ty`.
pub fn tainted_seeds_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.TaintedSeeds")
}

item_leaf! {
    /// `Server.Items.TaintedSeeds`.
    TAINTED_SEEDS = "Server.Items.TaintedSeeds" : ITEM_BASE, |w| tainted_seeds_new(w, &TAINTED_SEEDS), encoded false;
    label_number: crate::class_props::label_number,
}

/// `new ThorvaldsMedallion()` of class `ty`.
pub fn thorvalds_medallion_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.ThorvaldsMedallion")
}

item_leaf! {
    /// `Server.Items.ThorvaldsMedallion`.
    THORVALDS_MEDALLION = "Server.Items.ThorvaldsMedallion" : ITEM_BASE, |w| thorvalds_medallion_new(w, &THORVALDS_MEDALLION), encoded false;
    label_number: crate::class_props::label_number,
}

/// `new ThrashersTail()` of class `ty`.
pub fn thrashers_tail_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.ThrashersTail")
}

item_leaf! {
    /// `Server.Items.ThrashersTail`.
    THRASHERS_TAIL = "Server.Items.ThrashersTail" : ITEM_BASE, |w| thrashers_tail_new(w, &THRASHERS_TAIL), encoded false;
    label_number: crate::class_props::label_number,
}

/// `new BolaBall(amount = 1)` of class `ty`.
pub fn bola_ball_new(w: &mut World, ty: &'static ItemType, amount: i32) -> ItemId {
    let id = crate::simple::from_template(w, ty, "Server.Items.BolaBall");
    ops::set_amount(w, id, amount);
    hue(w, id, 0x8AC);
    id
}

item_leaf! {
    /// `Server.Items.BolaBall`.
    BOLA_BALL = "Server.Items.BolaBall" : ITEM_BASE, |w| bola_ball_new(w, &BOLA_BALL, 1), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new ExecutionersCap()` of class `ty`.
pub fn executioners_cap_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.ExecutionersCap")
}

item_leaf! {
    /// `Server.Items.ExecutionersCap`.
    EXECUTIONERS_CAP = "Server.Items.ExecutionersCap" : ITEM_BASE, |w| executioners_cap_new(w, &EXECUTIONERS_CAP), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new SmallFlask()` of class `ty`.
pub fn small_flask_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.SmallFlask")
}

item_leaf! {
    /// `Server.Items.SmallFlask`.
    SMALL_FLASK = "Server.Items.SmallFlask" : ITEM_BASE, |w| small_flask_new(w, &SMALL_FLASK), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new MediumFlask()` of class `ty`.
pub fn medium_flask_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.MediumFlask")
}

item_leaf! {
    /// `Server.Items.MediumFlask`.
    MEDIUM_FLASK = "Server.Items.MediumFlask" : ITEM_BASE, |w| medium_flask_new(w, &MEDIUM_FLASK), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new LargeFlask()` of class `ty`.
pub fn large_flask_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.LargeFlask")
}

item_leaf! {
    /// `Server.Items.LargeFlask`.
    LARGE_FLASK = "Server.Items.LargeFlask" : ITEM_BASE, |w| large_flask_new(w, &LARGE_FLASK), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new CurvedFlask()` of class `ty`.
pub fn curved_flask_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.CurvedFlask")
}

item_leaf! {
    /// `Server.Items.CurvedFlask`.
    CURVED_FLASK = "Server.Items.CurvedFlask" : ITEM_BASE, |w| curved_flask_new(w, &CURVED_FLASK), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new LongFlask()` of class `ty`.
pub fn long_flask_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.LongFlask")
}

item_leaf! {
    /// `Server.Items.LongFlask`.
    LONG_FLASK = "Server.Items.LongFlask" : ITEM_BASE, |w| long_flask_new(w, &LONG_FLASK), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new SpinningHourglass()` of class `ty`.
pub fn spinning_hourglass_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.SpinningHourglass")
}

item_leaf! {
    /// `Server.Items.SpinningHourglass`.
    SPINNING_HOURGLASS = "Server.Items.SpinningHourglass" : ITEM_BASE, |w| spinning_hourglass_new(w, &SPINNING_HOURGLASS), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new GreenBottle()` of class `ty`.
pub fn green_bottle_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.GreenBottle")
}

item_leaf! {
    /// `Server.Items.GreenBottle`.
    GREEN_BOTTLE = "Server.Items.GreenBottle" : ITEM_BASE, |w| green_bottle_new(w, &GREEN_BOTTLE), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new RedBottle()` of class `ty`.
pub fn red_bottle_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.RedBottle")
}

item_leaf! {
    /// `Server.Items.RedBottle`.
    RED_BOTTLE = "Server.Items.RedBottle" : ITEM_BASE, |w| red_bottle_new(w, &RED_BOTTLE), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new SmallBrownBottle()` of class `ty`.
pub fn small_brown_bottle_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.SmallBrownBottle")
}

item_leaf! {
    /// `Server.Items.SmallBrownBottle`.
    SMALL_BROWN_BOTTLE = "Server.Items.SmallBrownBottle" : ITEM_BASE, |w| small_brown_bottle_new(w, &SMALL_BROWN_BOTTLE), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new SmallGreenBottle()` of class `ty`.
pub fn small_green_bottle_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.SmallGreenBottle")
}

item_leaf! {
    /// `Server.Items.SmallGreenBottle`.
    SMALL_GREEN_BOTTLE = "Server.Items.SmallGreenBottle" : ITEM_BASE, |w| small_green_bottle_new(w, &SMALL_GREEN_BOTTLE), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new SmallVioletBottle()` of class `ty`.
pub fn small_violet_bottle_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.SmallVioletBottle")
}

item_leaf! {
    /// `Server.Items.SmallVioletBottle`.
    SMALL_VIOLET_BOTTLE = "Server.Items.SmallVioletBottle" : ITEM_BASE, |w| small_violet_bottle_new(w, &SMALL_VIOLET_BOTTLE), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new TinyYellowBottle()` of class `ty`.
pub fn tiny_yellow_bottle_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.TinyYellowBottle")
}

item_leaf! {
    /// `Server.Items.TinyYellowBottle`.
    TINY_YELLOW_BOTTLE = "Server.Items.TinyYellowBottle" : ITEM_BASE, |w| tiny_yellow_bottle_new(w, &TINY_YELLOW_BOTTLE), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new SmallBlueFlask()` of class `ty`.
pub fn small_blue_flask_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.SmallBlueFlask")
}

item_leaf! {
    /// `Server.Items.SmallBlueFlask`.
    SMALL_BLUE_FLASK = "Server.Items.SmallBlueFlask" : ITEM_BASE, |w| small_blue_flask_new(w, &SMALL_BLUE_FLASK), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new SmallYellowFlask()` of class `ty`.
pub fn small_yellow_flask_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.SmallYellowFlask")
}

item_leaf! {
    /// `Server.Items.SmallYellowFlask`.
    SMALL_YELLOW_FLASK = "Server.Items.SmallYellowFlask" : ITEM_BASE, |w| small_yellow_flask_new(w, &SMALL_YELLOW_FLASK), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new SmallRedFlask()` of class `ty`.
pub fn small_red_flask_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.SmallRedFlask")
}

item_leaf! {
    /// `Server.Items.SmallRedFlask`.
    SMALL_RED_FLASK = "Server.Items.SmallRedFlask" : ITEM_BASE, |w| small_red_flask_new(w, &SMALL_RED_FLASK), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new SmallEmptyFlask()` of class `ty`.
pub fn small_empty_flask_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.SmallEmptyFlask")
}

item_leaf! {
    /// `Server.Items.SmallEmptyFlask`.
    SMALL_EMPTY_FLASK = "Server.Items.SmallEmptyFlask" : ITEM_BASE, |w| small_empty_flask_new(w, &SMALL_EMPTY_FLASK), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new YellowBeaker()` of class `ty`.
pub fn yellow_beaker_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.YellowBeaker")
}

item_leaf! {
    /// `Server.Items.YellowBeaker`.
    YELLOW_BEAKER = "Server.Items.YellowBeaker" : ITEM_BASE, |w| yellow_beaker_new(w, &YELLOW_BEAKER), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new RedBeaker()` of class `ty`.
pub fn red_beaker_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.RedBeaker")
}

item_leaf! {
    /// `Server.Items.RedBeaker`.
    RED_BEAKER = "Server.Items.RedBeaker" : ITEM_BASE, |w| red_beaker_new(w, &RED_BEAKER), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new BlueBeaker()` of class `ty`.
pub fn blue_beaker_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.BlueBeaker")
}

item_leaf! {
    /// `Server.Items.BlueBeaker`.
    BLUE_BEAKER = "Server.Items.BlueBeaker" : ITEM_BASE, |w| blue_beaker_new(w, &BLUE_BEAKER), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new GreenBeaker()` of class `ty`.
pub fn green_beaker_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.GreenBeaker")
}

item_leaf! {
    /// `Server.Items.GreenBeaker`.
    GREEN_BEAKER = "Server.Items.GreenBeaker" : ITEM_BASE, |w| green_beaker_new(w, &GREEN_BEAKER), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new EmptyCurvedFlaskW()` of class `ty`.
pub fn empty_curved_flask_w_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.EmptyCurvedFlaskW")
}

item_leaf! {
    /// `Server.Items.EmptyCurvedFlaskW`.
    EMPTY_CURVED_FLASK_W = "Server.Items.EmptyCurvedFlaskW" : ITEM_BASE, |w| empty_curved_flask_w_new(w, &EMPTY_CURVED_FLASK_W), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new RedCurvedFlask()` of class `ty`.
pub fn red_curved_flask_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.RedCurvedFlask")
}

item_leaf! {
    /// `Server.Items.RedCurvedFlask`.
    RED_CURVED_FLASK = "Server.Items.RedCurvedFlask" : ITEM_BASE, |w| red_curved_flask_new(w, &RED_CURVED_FLASK), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new LtBlueCurvedFlask()` of class `ty`.
pub fn lt_blue_curved_flask_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.LtBlueCurvedFlask")
}

item_leaf! {
    /// `Server.Items.LtBlueCurvedFlask`.
    LT_BLUE_CURVED_FLASK = "Server.Items.LtBlueCurvedFlask" : ITEM_BASE, |w| lt_blue_curved_flask_new(w, &LT_BLUE_CURVED_FLASK), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new EmptyCurvedFlaskE()` of class `ty`.
pub fn empty_curved_flask_e_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.EmptyCurvedFlaskE")
}

item_leaf! {
    /// `Server.Items.EmptyCurvedFlaskE`.
    EMPTY_CURVED_FLASK_E = "Server.Items.EmptyCurvedFlaskE" : ITEM_BASE, |w| empty_curved_flask_e_new(w, &EMPTY_CURVED_FLASK_E), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new BlueCurvedFlask()` of class `ty`.
pub fn blue_curved_flask_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.BlueCurvedFlask")
}

item_leaf! {
    /// `Server.Items.BlueCurvedFlask`.
    BLUE_CURVED_FLASK = "Server.Items.BlueCurvedFlask" : ITEM_BASE, |w| blue_curved_flask_new(w, &BLUE_CURVED_FLASK), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new GreenCurvedFlask()` of class `ty`.
pub fn green_curved_flask_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.GreenCurvedFlask")
}

item_leaf! {
    /// `Server.Items.GreenCurvedFlask`.
    GREEN_CURVED_FLASK = "Server.Items.GreenCurvedFlask" : ITEM_BASE, |w| green_curved_flask_new(w, &GREEN_CURVED_FLASK), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new RedRibbedFlask()` of class `ty`.
pub fn red_ribbed_flask_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.RedRibbedFlask")
}

item_leaf! {
    /// `Server.Items.RedRibbedFlask`.
    RED_RIBBED_FLASK = "Server.Items.RedRibbedFlask" : ITEM_BASE, |w| red_ribbed_flask_new(w, &RED_RIBBED_FLASK), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new VioletRibbedFlask()` of class `ty`.
pub fn violet_ribbed_flask_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.VioletRibbedFlask")
}

item_leaf! {
    /// `Server.Items.VioletRibbedFlask`.
    VIOLET_RIBBED_FLASK = "Server.Items.VioletRibbedFlask" : ITEM_BASE, |w| violet_ribbed_flask_new(w, &VIOLET_RIBBED_FLASK), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new EmptyRibbedFlask()` of class `ty`.
pub fn empty_ribbed_flask_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.EmptyRibbedFlask")
}

item_leaf! {
    /// `Server.Items.EmptyRibbedFlask`.
    EMPTY_RIBBED_FLASK = "Server.Items.EmptyRibbedFlask" : ITEM_BASE, |w| empty_ribbed_flask_new(w, &EMPTY_RIBBED_FLASK), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new LargeYellowFlask()` of class `ty`.
pub fn large_yellow_flask_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.LargeYellowFlask")
}

item_leaf! {
    /// `Server.Items.LargeYellowFlask`.
    LARGE_YELLOW_FLASK = "Server.Items.LargeYellowFlask" : ITEM_BASE, |w| large_yellow_flask_new(w, &LARGE_YELLOW_FLASK), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new LargeVioletFlask()` of class `ty`.
pub fn large_violet_flask_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.LargeVioletFlask")
}

item_leaf! {
    /// `Server.Items.LargeVioletFlask`.
    LARGE_VIOLET_FLASK = "Server.Items.LargeVioletFlask" : ITEM_BASE, |w| large_violet_flask_new(w, &LARGE_VIOLET_FLASK), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new LargeEmptyFlask()` of class `ty`.
pub fn large_empty_flask_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.LargeEmptyFlask")
}

item_leaf! {
    /// `Server.Items.LargeEmptyFlask`.
    LARGE_EMPTY_FLASK = "Server.Items.LargeEmptyFlask" : ITEM_BASE, |w| large_empty_flask_new(w, &LARGE_EMPTY_FLASK), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new AniRedRibbedFlask()` of class `ty`.
pub fn ani_red_ribbed_flask_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.AniRedRibbedFlask")
}

item_leaf! {
    /// `Server.Items.AniRedRibbedFlask`.
    ANI_RED_RIBBED_FLASK = "Server.Items.AniRedRibbedFlask" : ITEM_BASE, |w| ani_red_ribbed_flask_new(w, &ANI_RED_RIBBED_FLASK), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new AniLargeVioletFlask()` of class `ty`.
pub fn ani_large_violet_flask_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.AniLargeVioletFlask")
}

item_leaf! {
    /// `Server.Items.AniLargeVioletFlask`.
    ANI_LARGE_VIOLET_FLASK = "Server.Items.AniLargeVioletFlask" : ITEM_BASE, |w| ani_large_violet_flask_new(w, &ANI_LARGE_VIOLET_FLASK), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new AniSmallBlueFlask()` of class `ty`.
pub fn ani_small_blue_flask_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.AniSmallBlueFlask")
}

item_leaf! {
    /// `Server.Items.AniSmallBlueFlask`.
    ANI_SMALL_BLUE_FLASK = "Server.Items.AniSmallBlueFlask" : ITEM_BASE, |w| ani_small_blue_flask_new(w, &ANI_SMALL_BLUE_FLASK), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new SmallBlueBottle()` of class `ty`.
pub fn small_blue_bottle_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.SmallBlueBottle")
}

item_leaf! {
    /// `Server.Items.SmallBlueBottle`.
    SMALL_BLUE_BOTTLE = "Server.Items.SmallBlueBottle" : ITEM_BASE, |w| small_blue_bottle_new(w, &SMALL_BLUE_BOTTLE), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new SmallGreenBottle2()` of class `ty`.
pub fn small_green_bottle2_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.SmallGreenBottle2")
}

item_leaf! {
    /// `Server.Items.SmallGreenBottle2`.
    SMALL_GREEN_BOTTLE2 = "Server.Items.SmallGreenBottle2" : ITEM_BASE, |w| small_green_bottle2_new(w, &SMALL_GREEN_BOTTLE2), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new EmptyVialsWRack()` of class `ty`.
pub fn empty_vials_w_rack_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.EmptyVialsWRack")
}

item_leaf! {
    /// `Server.Items.EmptyVialsWRack`.
    EMPTY_VIALS_W_RACK = "Server.Items.EmptyVialsWRack" : ITEM_BASE, |w| empty_vials_w_rack_new(w, &EMPTY_VIALS_W_RACK), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new FullVialsWRack()` of class `ty`.
pub fn full_vials_w_rack_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.FullVialsWRack")
}

item_leaf! {
    /// `Server.Items.FullVialsWRack`.
    FULL_VIALS_W_RACK = "Server.Items.FullVialsWRack" : ITEM_BASE, |w| full_vials_w_rack_new(w, &FULL_VIALS_W_RACK), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new EmptyVial()` of class `ty`.
pub fn empty_vial_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.EmptyVial")
}

item_leaf! {
    /// `Server.Items.EmptyVial`.
    EMPTY_VIAL = "Server.Items.EmptyVial" : ITEM_BASE, |w| empty_vial_new(w, &EMPTY_VIAL), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new HourglassAni()` of class `ty`.
pub fn hourglass_ani_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.HourglassAni")
}

item_leaf! {
    /// `Server.Items.HourglassAni`.
    HOURGLASS_ANI = "Server.Items.HourglassAni" : ITEM_BASE, |w| hourglass_ani_new(w, &HOURGLASS_ANI), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new Hourglass()` of class `ty`.
pub fn hourglass_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.Hourglass")
}

item_leaf! {
    /// `Server.Items.Hourglass`.
    HOURGLASS = "Server.Items.Hourglass" : ITEM_BASE, |w| hourglass_new(w, &HOURGLASS), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new TinyRedBottle()` of class `ty`.
pub fn tiny_red_bottle_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.TinyRedBottle")
}

item_leaf! {
    /// `Server.Items.TinyRedBottle`.
    TINY_RED_BOTTLE = "Server.Items.TinyRedBottle" : ITEM_BASE, |w| tiny_red_bottle_new(w, &TINY_RED_BOTTLE), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new HoveringWisp()` of class `ty`.
pub fn hovering_wisp_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.HoveringWisp")
}

item_leaf! {
    /// `Server.Items.HoveringWisp`.
    HOVERING_WISP = "Server.Items.HoveringWisp" : ITEM_BASE, |w| hovering_wisp_new(w, &HOVERING_WISP), encoded false;
    label_number: crate::class_props::label_number,
}

/// `new MinotaurArtifact()` of class `ty`.
pub fn minotaur_artifact_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.MinotaurArtifact")
}

item_leaf! {
    /// `Server.Items.MinotaurArtifact`.
    MINOTAUR_ARTIFACT = "Server.Items.MinotaurArtifact" : ITEM_BASE, |w| minotaur_artifact_new(w, &MINOTAUR_ARTIFACT), encoded false;
    label_number: crate::class_props::label_number,
    default_weight: |w, id| if w.item(id).is_some_and(|i| (i.ty.item_id)(w, id) == 0x9ED) { 30.0 } else { 5.0 },
}

/// `new OrigamiButterfly()` of class `ty`.
pub fn origami_butterfly_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.OrigamiButterfly")
}

item_leaf! {
    /// `Server.Items.OrigamiButterfly`.
    ORIGAMI_BUTTERFLY = "Server.Items.OrigamiButterfly" : ITEM_BASE, |w| origami_butterfly_new(w, &ORIGAMI_BUTTERFLY), encoded true;
    label_number: crate::class_props::label_number,
}

/// `new OrigamiSwan()` of class `ty`.
pub fn origami_swan_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.OrigamiSwan")
}

item_leaf! {
    /// `Server.Items.OrigamiSwan`.
    ORIGAMI_SWAN = "Server.Items.OrigamiSwan" : ITEM_BASE, |w| origami_swan_new(w, &ORIGAMI_SWAN), encoded true;
    label_number: crate::class_props::label_number,
}

/// `new OrigamiFrog()` of class `ty`.
pub fn origami_frog_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.OrigamiFrog")
}

item_leaf! {
    /// `Server.Items.OrigamiFrog`.
    ORIGAMI_FROG = "Server.Items.OrigamiFrog" : ITEM_BASE, |w| origami_frog_new(w, &ORIGAMI_FROG), encoded true;
    label_number: crate::class_props::label_number,
}

/// `new OrigamiShape()` of class `ty`.
pub fn origami_shape_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.OrigamiShape")
}

item_leaf! {
    /// `Server.Items.OrigamiShape`.
    ORIGAMI_SHAPE = "Server.Items.OrigamiShape" : ITEM_BASE, |w| origami_shape_new(w, &ORIGAMI_SHAPE), encoded true;
    label_number: crate::class_props::label_number,
}

/// `new OrigamiSongbird()` of class `ty`.
pub fn origami_songbird_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.OrigamiSongbird")
}

item_leaf! {
    /// `Server.Items.OrigamiSongbird`.
    ORIGAMI_SONGBIRD = "Server.Items.OrigamiSongbird" : ITEM_BASE, |w| origami_songbird_new(w, &ORIGAMI_SONGBIRD), encoded true;
    label_number: crate::class_props::label_number,
}

/// `new OrigamiFish()` of class `ty`.
pub fn origami_fish_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.OrigamiFish")
}

item_leaf! {
    /// `Server.Items.OrigamiFish`.
    ORIGAMI_FISH = "Server.Items.OrigamiFish" : ITEM_BASE, |w| origami_fish_new(w, &ORIGAMI_FISH), encoded true;
    label_number: crate::class_props::label_number,
}

/// `new GrobusFur()` of class `ty`.
pub fn grobus_fur_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.GrobusFur")
}

item_leaf! {
    /// `Server.Items.GrobusFur`.
    GROBUS_FUR = "Server.Items.GrobusFur" : ITEM_BASE, |w| grobus_fur_new(w, &GROBUS_FUR), encoded false;
    label_number: crate::class_props::label_number,
}

/// `new PrimitiveFetish()` of class `ty`.
pub fn primitive_fetish_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.PrimitiveFetish")
}

item_leaf! {
    /// `Server.Items.PrimitiveFetish`.
    PRIMITIVE_FETISH = "Server.Items.PrimitiveFetish" : ITEM_BASE, |w| primitive_fetish_new(w, &PRIMITIVE_FETISH), encoded false;
    label_number: crate::class_props::label_number,
}

/// `new AcidProofRope()` of class `ty`.
pub fn acid_proof_rope_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.AcidProofRope")
}

item_leaf! {
    /// `Server.Items.AcidProofRope`.
    ACID_PROOF_ROPE = "Server.Items.AcidProofRope" : ITEM_BASE, |w| acid_proof_rope_new(w, &ACID_PROOF_ROPE), encoded false;
    label_number: crate::class_props::label_number,
}

/// `new ParoxysmusCorrodedStein()` of class `ty`.
pub fn paroxysmus_corroded_stein_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.ParoxysmusCorrodedStein")
}

item_leaf! {
    /// `Server.Items.ParoxysmusCorrodedStein`.
    PAROXYSMUS_CORRODED_STEIN = "Server.Items.ParoxysmusCorrodedStein" : ITEM_BASE, |w| paroxysmus_corroded_stein_new(w, &PAROXYSMUS_CORRODED_STEIN), encoded false;
    label_number: crate::class_props::label_number,
}

/// `new ParoxysmusDinner()` of class `ty`.
pub fn paroxysmus_dinner_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.ParoxysmusDinner")
}

item_leaf! {
    /// `Server.Items.ParoxysmusDinner`.
    PAROXYSMUS_DINNER = "Server.Items.ParoxysmusDinner" : ITEM_BASE, |w| paroxysmus_dinner_new(w, &PAROXYSMUS_DINNER), encoded false;
    label_number: crate::class_props::label_number,
}

/// `new StringOfPartsOfParoxysmusVictims()` of class `ty`.
pub fn string_of_parts_of_paroxysmus_victims_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.StringOfPartsOfParoxysmusVictims")
}

item_leaf! {
    /// `Server.Items.StringOfPartsOfParoxysmusVictims`.
    STRING_OF_PARTS_OF_PAROXYSMUS_VICTIMS = "Server.Items.StringOfPartsOfParoxysmusVictims" : ITEM_BASE, |w| string_of_parts_of_paroxysmus_victims_new(w, &STRING_OF_PARTS_OF_PAROXYSMUS_VICTIMS), encoded false;
    label_number: crate::class_props::label_number,
}

/// `new SweatOfParoxysmus()` of class `ty`.
pub fn sweat_of_paroxysmus_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.SweatOfParoxysmus")
}

item_leaf! {
    /// `Server.Items.SweatOfParoxysmus`.
    SWEAT_OF_PAROXYSMUS = "Server.Items.SweatOfParoxysmus" : ITEM_BASE, |w| sweat_of_paroxysmus_new(w, &SWEAT_OF_PAROXYSMUS), encoded false;
    label_number: crate::class_props::label_number,
}

/// `new CrystallineFragments()` of class `ty`.
pub fn crystalline_fragments_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.CrystallineFragments")
}

item_leaf! {
    /// `Server.Items.CrystallineFragments`.
    CRYSTALLINE_FRAGMENTS = "Server.Items.CrystallineFragments" : ITEM_BASE, |w| crystalline_fragments_new(w, &CRYSTALLINE_FRAGMENTS), encoded false;
    label_number: crate::class_props::label_number,
}

/// `new IcyHeart()` of class `ty`.
pub fn icy_heart_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.IcyHeart")
}

item_leaf! {
    /// `Server.Items.IcyHeart`.
    ICY_HEART = "Server.Items.IcyHeart" : ITEM_BASE, |w| icy_heart_new(w, &ICY_HEART), encoded false;
    label_number: crate::class_props::label_number,
}

/// `new LuckyDagger()` of class `ty`.
pub fn lucky_dagger_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.LuckyDagger")
}

item_leaf! {
    /// `Server.Items.LuckyDagger`.
    LUCKY_DAGGER = "Server.Items.LuckyDagger" : ITEM_BASE, |w| lucky_dagger_new(w, &LUCKY_DAGGER), encoded false;
    label_number: crate::class_props::label_number,
}

/// `new ProtectorsEssence()` of class `ty`.
pub fn protectors_essence_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.ProtectorsEssence")
}

item_leaf! {
    /// `Server.Items.ProtectorsEssence`.
    PROTECTORS_ESSENCE = "Server.Items.ProtectorsEssence" : ITEM_BASE, |w| protectors_essence_new(w, &PROTECTORS_ESSENCE), encoded false;
    label_number: crate::class_props::label_number,
}

const ITEMIDS: [i32; 36] = [
    0x2206, 0x2207, 0x2208, 0x2209, 0x220A, 0x220B, 0x220C, 0x220D, 0x220E, 0x2210, 0x2211, 0x2212,
    0x2213, 0x2214, 0x2215, 0x2216, 0x2217, 0x2218, 0x221A, 0x221B, 0x221C, 0x221D, 0x221E, 0x221F,
    0x2220, 0x2221, 0x2222, 0x2224, 0x2225, 0x2226, 0x2227, 0x2228, 0x2229, 0x222A, 0x222B, 0x222C,
];

/// `new ShimmeringCrystals()` of class `ty`.
pub fn shimmering_crystals_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let graphic = random_element(w, &ITEMIDS);
    ops::new_item_with_id(w, ty, graphic)
}

item_leaf! {
    /// `Server.Items.ShimmeringCrystals`.
    SHIMMERING_CRYSTALS = "Server.Items.ShimmeringCrystals" : ITEM_BASE, |w| shimmering_crystals_new(w, &SHIMMERING_CRYSTALS), encoded false;
    label_number: crate::class_props::label_number,
    force_show_properties: |_, _| true,
}

/// `new Rope(amount = 1)` of class `ty`.
pub fn rope_new(w: &mut World, ty: &'static ItemType, amount: i32) -> ItemId {
    let id = crate::simple::from_template(w, ty, "Server.Items.Rope");
    ops::set_amount(w, id, amount);
    id
}

item_leaf! {
    /// `Server.Items.Rope`.
    ROPE = "Server.Items.Rope" : ITEM_BASE, |w| rope_new(w, &ROPE, 1), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new IronWire(amount = 1)` of class `ty`.
pub fn iron_wire_new(w: &mut World, ty: &'static ItemType, amount: i32) -> ItemId {
    let id = crate::simple::from_template(w, ty, "Server.Items.IronWire");
    ops::set_amount(w, id, amount);
    id
}

item_leaf! {
    /// `Server.Items.IronWire`.
    IRON_WIRE = "Server.Items.IronWire" : ITEM_BASE, |w| iron_wire_new(w, &IRON_WIRE, 1), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new SilverWire(amount = 1)` of class `ty`.
pub fn silver_wire_new(w: &mut World, ty: &'static ItemType, amount: i32) -> ItemId {
    let id = crate::simple::from_template(w, ty, "Server.Items.SilverWire");
    ops::set_amount(w, id, amount);
    id
}

item_leaf! {
    /// `Server.Items.SilverWire`.
    SILVER_WIRE = "Server.Items.SilverWire" : ITEM_BASE, |w| silver_wire_new(w, &SILVER_WIRE, 1), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new GoldWire(amount = 1)` of class `ty`.
pub fn gold_wire_new(w: &mut World, ty: &'static ItemType, amount: i32) -> ItemId {
    let id = crate::simple::from_template(w, ty, "Server.Items.GoldWire");
    ops::set_amount(w, id, amount);
    id
}

item_leaf! {
    /// `Server.Items.GoldWire`.
    GOLD_WIRE = "Server.Items.GoldWire" : ITEM_BASE, |w| gold_wire_new(w, &GOLD_WIRE, 1), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new CopperWire(amount = 1)` of class `ty`.
pub fn copper_wire_new(w: &mut World, ty: &'static ItemType, amount: i32) -> ItemId {
    let id = crate::simple::from_template(w, ty, "Server.Items.CopperWire");
    ops::set_amount(w, id, amount);
    id
}

item_leaf! {
    /// `Server.Items.CopperWire`.
    COPPER_WIRE = "Server.Items.CopperWire" : ITEM_BASE, |w| copper_wire_new(w, &COPPER_WIRE, 1), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new WhiteDriedFlowers(amount = 1)` of class `ty`.
pub fn white_dried_flowers_new(w: &mut World, ty: &'static ItemType, amount: i32) -> ItemId {
    let id = crate::simple::from_template(w, ty, "Server.Items.WhiteDriedFlowers");
    ops::set_amount(w, id, amount);
    id
}

item_leaf! {
    /// `Server.Items.WhiteDriedFlowers`.
    WHITE_DRIED_FLOWERS = "Server.Items.WhiteDriedFlowers" : ITEM_BASE, |w| white_dried_flowers_new(w, &WHITE_DRIED_FLOWERS, 1), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new GreenDriedFlowers(amount = 1)` of class `ty`.
pub fn green_dried_flowers_new(w: &mut World, ty: &'static ItemType, amount: i32) -> ItemId {
    let id = crate::simple::from_template(w, ty, "Server.Items.GreenDriedFlowers");
    ops::set_amount(w, id, amount);
    id
}

item_leaf! {
    /// `Server.Items.GreenDriedFlowers`.
    GREEN_DRIED_FLOWERS = "Server.Items.GreenDriedFlowers" : ITEM_BASE, |w| green_dried_flowers_new(w, &GREEN_DRIED_FLOWERS, 1), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new DriedOnions(amount = 1)` of class `ty`.
pub fn dried_onions_new(w: &mut World, ty: &'static ItemType, amount: i32) -> ItemId {
    let id = crate::simple::from_template(w, ty, "Server.Items.DriedOnions");
    ops::set_amount(w, id, amount);
    id
}

item_leaf! {
    /// `Server.Items.DriedOnions`.
    DRIED_ONIONS = "Server.Items.DriedOnions" : ITEM_BASE, |w| dried_onions_new(w, &DRIED_ONIONS, 1), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new DriedHerbs(amount = 1)` of class `ty`.
pub fn dried_herbs_new(w: &mut World, ty: &'static ItemType, amount: i32) -> ItemId {
    let id = crate::simple::from_template(w, ty, "Server.Items.DriedHerbs");
    ops::set_amount(w, id, amount);
    id
}

item_leaf! {
    /// `Server.Items.DriedHerbs`.
    DRIED_HERBS = "Server.Items.DriedHerbs" : ITEM_BASE, |w| dried_herbs_new(w, &DRIED_HERBS, 1), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new HorseShoes()` of class `ty`.
pub fn horse_shoes_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.HorseShoes")
}

item_leaf! {
    /// `Server.Items.HorseShoes`.
    HORSE_SHOES = "Server.Items.HorseShoes" : ITEM_BASE, |w| horse_shoes_new(w, &HORSE_SHOES), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new ForgedMetal()` of class `ty`.
pub fn forged_metal_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.ForgedMetal")
}

item_leaf! {
    /// `Server.Items.ForgedMetal`.
    FORGED_METAL = "Server.Items.ForgedMetal" : ITEM_BASE, |w| forged_metal_new(w, &FORGED_METAL), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new Whip()` of class `ty`.
pub fn whip_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.Whip")
}

item_leaf! {
    /// `Server.Items.Whip`.
    WHIP = "Server.Items.Whip" : ITEM_BASE, |w| whip_new(w, &WHIP), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new PaintsAndBrush()` of class `ty`.
pub fn paints_and_brush_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.PaintsAndBrush")
}

item_leaf! {
    /// `Server.Items.PaintsAndBrush`.
    PAINTS_AND_BRUSH = "Server.Items.PaintsAndBrush" : ITEM_BASE, |w| paints_and_brush_new(w, &PAINTS_AND_BRUSH), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new PenAndInk()` of class `ty`.
pub fn pen_and_ink_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.PenAndInk")
}

item_leaf! {
    /// `Server.Items.PenAndInk`.
    PEN_AND_INK = "Server.Items.PenAndInk" : ITEM_BASE, |w| pen_and_ink_new(w, &PEN_AND_INK), encoded false;
    default_weight: crate::class_props::default_weight,
}

/// `new ChiselsNorth()` of class `ty`.
pub fn chisels_north_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.ChiselsNorth")
}

item_leaf! {
    /// `Server.Items.ChiselsNorth`.
    CHISELS_NORTH = "Server.Items.ChiselsNorth" : ITEM_BASE, |w| chisels_north_new(w, &CHISELS_NORTH), encoded true;
    default_weight: crate::class_props::default_weight,
}

/// `new ChiselsWest()` of class `ty`.
pub fn chisels_west_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.ChiselsWest")
}

item_leaf! {
    /// `Server.Items.ChiselsWest`.
    CHISELS_WEST = "Server.Items.ChiselsWest" : ITEM_BASE, |w| chisels_west_new(w, &CHISELS_WEST), encoded true;
    default_weight: crate::class_props::default_weight,
}

/// `new DirtyPan()` of class `ty`.
pub fn dirty_pan_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.DirtyPan")
}

item_leaf! {
    /// `Server.Items.DirtyPan`.
    DIRTY_PAN = "Server.Items.DirtyPan" : ITEM_BASE, |w| dirty_pan_new(w, &DIRTY_PAN), encoded true;
    default_weight: crate::class_props::default_weight,
}

/// `new DirtySmallRoundPot()` of class `ty`.
pub fn dirty_small_round_pot_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.DirtySmallRoundPot")
}

item_leaf! {
    /// `Server.Items.DirtySmallRoundPot`.
    DIRTY_SMALL_ROUND_POT = "Server.Items.DirtySmallRoundPot" : ITEM_BASE, |w| dirty_small_round_pot_new(w, &DIRTY_SMALL_ROUND_POT), encoded true;
    default_weight: crate::class_props::default_weight,
}

/// `new DirtyPot()` of class `ty`.
pub fn dirty_pot_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.DirtyPot")
}

item_leaf! {
    /// `Server.Items.DirtyPot`.
    DIRTY_POT = "Server.Items.DirtyPot" : ITEM_BASE, |w| dirty_pot_new(w, &DIRTY_POT), encoded true;
    default_weight: crate::class_props::default_weight,
}

/// `new DirtyRoundPot()` of class `ty`.
pub fn dirty_round_pot_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.DirtyRoundPot")
}

item_leaf! {
    /// `Server.Items.DirtyRoundPot`.
    DIRTY_ROUND_POT = "Server.Items.DirtyRoundPot" : ITEM_BASE, |w| dirty_round_pot_new(w, &DIRTY_ROUND_POT), encoded true;
    default_weight: crate::class_props::default_weight,
}

/// `new DirtyFrypan()` of class `ty`.
pub fn dirty_frypan_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.DirtyFrypan")
}

item_leaf! {
    /// `Server.Items.DirtyFrypan`.
    DIRTY_FRYPAN = "Server.Items.DirtyFrypan" : ITEM_BASE, |w| dirty_frypan_new(w, &DIRTY_FRYPAN), encoded true;
    default_weight: crate::class_props::default_weight,
}

/// `new DirtySmallPot()` of class `ty`.
pub fn dirty_small_pot_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.DirtySmallPot")
}

item_leaf! {
    /// `Server.Items.DirtySmallPot`.
    DIRTY_SMALL_POT = "Server.Items.DirtySmallPot" : ITEM_BASE, |w| dirty_small_pot_new(w, &DIRTY_SMALL_POT), encoded true;
    default_weight: crate::class_props::default_weight,
}

/// `new DirtyKettle()` of class `ty`.
pub fn dirty_kettle_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.DirtyKettle")
}

item_leaf! {
    /// `Server.Items.DirtyKettle`.
    DIRTY_KETTLE = "Server.Items.DirtyKettle" : ITEM_BASE, |w| dirty_kettle_new(w, &DIRTY_KETTLE), encoded true;
    default_weight: crate::class_props::default_weight,
}

/// `new SwarmOfFlies()` of class `ty`.
pub fn swarm_of_flies_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.SwarmOfFlies")
}

item_leaf! {
    /// `Server.Items.SwarmOfFlies`.
    SWARM_OF_FLIES = "Server.Items.SwarmOfFlies" : ITEM_BASE, |w| swarm_of_flies_new(w, &SWARM_OF_FLIES), encoded false;
    default_name: crate::class_props::default_name,
}

/// `new DragonFlameSectBadge()` of class `ty`.
pub fn dragon_flame_sect_badge_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.DragonFlameSectBadge")
}

item_leaf! {
    /// `Server.Items.DragonFlameSectBadge`.
    DRAGON_FLAME_SECT_BADGE = "Server.Items.DragonFlameSectBadge" : ITEM_BASE, |w| dragon_flame_sect_badge_new(w, &DRAGON_FLAME_SECT_BADGE), encoded false;
    label_number: crate::class_props::label_number,
}

/// `new OrdersFromMinax()` of class `ty`.
pub fn orders_from_minax_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.OrdersFromMinax")
}

item_leaf! {
    /// `Server.Items.OrdersFromMinax`.
    ORDERS_FROM_MINAX = "Server.Items.OrdersFromMinax" : ITEM_BASE, |w| orders_from_minax_new(w, &ORDERS_FROM_MINAX), encoded false;
    label_number: crate::class_props::label_number,
}

/// `new SerpentFangSectBadge()` of class `ty`.
pub fn serpent_fang_sect_badge_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.SerpentFangSectBadge")
}

item_leaf! {
    /// `Server.Items.SerpentFangSectBadge`.
    SERPENT_FANG_SECT_BADGE = "Server.Items.SerpentFangSectBadge" : ITEM_BASE, |w| serpent_fang_sect_badge_new(w, &SERPENT_FANG_SECT_BADGE), encoded false;
    label_number: crate::class_props::label_number,
}

/// `new TigerClawSectBadge()` of class `ty`.
pub fn tiger_claw_sect_badge_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.TigerClawSectBadge")
}

item_leaf! {
    /// `Server.Items.TigerClawSectBadge`.
    TIGER_CLAW_SECT_BADGE = "Server.Items.TigerClawSectBadge" : ITEM_BASE, |w| tiger_claw_sect_badge_new(w, &TIGER_CLAW_SECT_BADGE), encoded false;
    label_number: crate::class_props::label_number,
}

/// `new TravestysCollectionOfShells()` of class `ty`.
pub fn travestys_collection_of_shells_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.TravestysCollectionOfShells")
}

item_leaf! {
    /// `Server.Items.TravestysCollectionOfShells`.
    TRAVESTYS_COLLECTION_OF_SHELLS = "Server.Items.TravestysCollectionOfShells" : ITEM_BASE, |w| travestys_collection_of_shells_new(w, &TRAVESTYS_COLLECTION_OF_SHELLS), encoded false;
    label_number: crate::class_props::label_number,
}

/// `new TravestysFineTeakwoodTray()` of class `ty`.
pub fn travestys_fine_teakwood_tray_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.TravestysFineTeakwoodTray")
}

item_leaf! {
    /// `Server.Items.TravestysFineTeakwoodTray`.
    TRAVESTYS_FINE_TEAKWOOD_TRAY = "Server.Items.TravestysFineTeakwoodTray" : ITEM_BASE, |w| travestys_fine_teakwood_tray_new(w, &TRAVESTYS_FINE_TEAKWOOD_TRAY), encoded false;
    label_number: crate::class_props::label_number,
}

/// `new TravestysSushiPreparations()` of class `ty`.
pub fn travestys_sushi_preparations_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.TravestysSushiPreparations")
}

item_leaf! {
    /// `Server.Items.TravestysSushiPreparations`.
    TRAVESTYS_SUSHI_PREPARATIONS = "Server.Items.TravestysSushiPreparations" : ITEM_BASE, |w| travestys_sushi_preparations_new(w, &TRAVESTYS_SUSHI_PREPARATIONS), encoded false;
    label_number: crate::class_props::label_number,
}

/// `new TribalBerry(amount = 1)` of class `ty`.
pub fn tribal_berry_new(w: &mut World, ty: &'static ItemType, amount: i32) -> ItemId {
    let id = crate::simple::from_template(w, ty, "Server.Items.TribalBerry");
    ops::set_amount(w, id, amount);
    hue(w, id, 6);
    id
}

item_leaf! {
    /// `Server.Items.TribalBerry`.
    TRIBAL_BERRY = "Server.Items.TribalBerry" : ITEM_BASE, |w| tribal_berry_new(w, &TRIBAL_BERRY, 1), encoded false;
    default_weight: crate::class_props::default_weight,
    label_number: crate::class_props::label_number,
}

/// `new HornOfTheDreadhorn()` of class `ty`.
pub fn horn_of_the_dreadhorn_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.HornOfTheDreadhorn")
}

item_leaf! {
    /// `Server.Items.HornOfTheDreadhorn`.
    HORN_OF_THE_DREADHORN = "Server.Items.HornOfTheDreadhorn" : ITEM_BASE, |w| horn_of_the_dreadhorn_new(w, &HORN_OF_THE_DREADHORN), encoded false;
    label_number: crate::class_props::label_number,
}

/// `new MangledHeadOfDreadhorn()` of class `ty`.
pub fn mangled_head_of_dreadhorn_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.MangledHeadOfDreadhorn")
}

item_leaf! {
    /// `Server.Items.MangledHeadOfDreadhorn`.
    MANGLED_HEAD_OF_DREADHORN = "Server.Items.MangledHeadOfDreadhorn" : ITEM_BASE, |w| mangled_head_of_dreadhorn_new(w, &MANGLED_HEAD_OF_DREADHORN), encoded false;
    label_number: crate::class_props::label_number,
}

/// `new TaintedMushroom()` of class `ty`.
pub fn tainted_mushroom_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.TaintedMushroom")
}

item_leaf! {
    /// `Server.Items.TaintedMushroom`.
    TAINTED_MUSHROOM = "Server.Items.TaintedMushroom" : ITEM_BASE, |w| tainted_mushroom_new(w, &TAINTED_MUSHROOM), encoded false;
    label_number: crate::class_props::label_number,
    force_show_properties: |_, _| true,
}

/// Every class of this file, for `register` and the tests.
pub static ALL: &[(&ItemType, &[&str])] = &[
    (&GLOB_OF_MONSTREOUS_INTERRED_GRIZZLE, &[]),
    (&GRIZZLED_SKULL_COLLECTION, &[]),
    (&MONSTEROUS_INTERRED_GRIZZLE_MAGGOTS, &[]),
    (&RESOLVES_BRIDLE, &[]),
    (&TOMBSTONE_OF_THE_DAMNED, &[]),
    (&BEESWAX, &[]),
    (&ABSCESS_TAIL, &[]),
    (&COILS_FANG, &[]),
    (&ETERNALLY_CORRUPT_TREE, &[]),
    (&HYDRA_SCALE, &[]),
    (&SALIVAS_FEATHER, &[]),
    (&SAMPLES_OF_CORRUPTED_WATER, &[]),
    (&TAINTED_SEEDS, &[]),
    (&THORVALDS_MEDALLION, &[]),
    (&THRASHERS_TAIL, &[]),
    (&BOLA_BALL, &[]),
    (&EXECUTIONERS_CAP, &[]),
    (&SMALL_FLASK, &[]),
    (&MEDIUM_FLASK, &[]),
    (&LARGE_FLASK, &[]),
    (&CURVED_FLASK, &[]),
    (&LONG_FLASK, &[]),
    (&SPINNING_HOURGLASS, &[]),
    (&GREEN_BOTTLE, &[]),
    (&RED_BOTTLE, &[]),
    (&SMALL_BROWN_BOTTLE, &[]),
    (&SMALL_GREEN_BOTTLE, &[]),
    (&SMALL_VIOLET_BOTTLE, &[]),
    (&TINY_YELLOW_BOTTLE, &[]),
    (&SMALL_BLUE_FLASK, &[]),
    (&SMALL_YELLOW_FLASK, &[]),
    (&SMALL_RED_FLASK, &[]),
    (&SMALL_EMPTY_FLASK, &[]),
    (&YELLOW_BEAKER, &[]),
    (&RED_BEAKER, &[]),
    (&BLUE_BEAKER, &[]),
    (&GREEN_BEAKER, &[]),
    (&EMPTY_CURVED_FLASK_W, &[]),
    (&RED_CURVED_FLASK, &[]),
    (&LT_BLUE_CURVED_FLASK, &[]),
    (&EMPTY_CURVED_FLASK_E, &[]),
    (&BLUE_CURVED_FLASK, &[]),
    (&GREEN_CURVED_FLASK, &[]),
    (&RED_RIBBED_FLASK, &[]),
    (&VIOLET_RIBBED_FLASK, &[]),
    (&EMPTY_RIBBED_FLASK, &[]),
    (&LARGE_YELLOW_FLASK, &[]),
    (&LARGE_VIOLET_FLASK, &[]),
    (&LARGE_EMPTY_FLASK, &[]),
    (&ANI_RED_RIBBED_FLASK, &[]),
    (&ANI_LARGE_VIOLET_FLASK, &[]),
    (&ANI_SMALL_BLUE_FLASK, &[]),
    (&SMALL_BLUE_BOTTLE, &[]),
    (&SMALL_GREEN_BOTTLE2, &[]),
    (&EMPTY_VIALS_W_RACK, &[]),
    (&FULL_VIALS_W_RACK, &[]),
    (&EMPTY_VIAL, &[]),
    (&HOURGLASS_ANI, &[]),
    (&HOURGLASS, &[]),
    (&TINY_RED_BOTTLE, &[]),
    (&HOVERING_WISP, &[]),
    (&MINOTAUR_ARTIFACT, &[]),
    (&ORIGAMI_BUTTERFLY, &[]),
    (&ORIGAMI_SWAN, &[]),
    (&ORIGAMI_FROG, &[]),
    (&ORIGAMI_SHAPE, &[]),
    (&ORIGAMI_SONGBIRD, &[]),
    (&ORIGAMI_FISH, &[]),
    (&GROBUS_FUR, &[]),
    (&PRIMITIVE_FETISH, &[]),
    (&ACID_PROOF_ROPE, &[]),
    (&PAROXYSMUS_CORRODED_STEIN, &[]),
    (&PAROXYSMUS_DINNER, &[]),
    (&STRING_OF_PARTS_OF_PAROXYSMUS_VICTIMS, &[]),
    (&SWEAT_OF_PAROXYSMUS, &[]),
    (&CRYSTALLINE_FRAGMENTS, &[]),
    (&ICY_HEART, &[]),
    (&LUCKY_DAGGER, &[]),
    (&PROTECTORS_ESSENCE, &[]),
    (&SHIMMERING_CRYSTALS, &[]),
    (&ROPE, &[]),
    (&IRON_WIRE, &[]),
    (&SILVER_WIRE, &[]),
    (&GOLD_WIRE, &[]),
    (&COPPER_WIRE, &[]),
    (&WHITE_DRIED_FLOWERS, &[]),
    (&GREEN_DRIED_FLOWERS, &[]),
    (&DRIED_ONIONS, &[]),
    (&DRIED_HERBS, &[]),
    (&HORSE_SHOES, &[]),
    (&FORGED_METAL, &[]),
    (&WHIP, &[]),
    (&PAINTS_AND_BRUSH, &[]),
    (&PEN_AND_INK, &[]),
    (&CHISELS_NORTH, &[]),
    (&CHISELS_WEST, &[]),
    (&DIRTY_PAN, &[]),
    (&DIRTY_SMALL_ROUND_POT, &[]),
    (&DIRTY_POT, &[]),
    (&DIRTY_ROUND_POT, &[]),
    (&DIRTY_FRYPAN, &[]),
    (&DIRTY_SMALL_POT, &[]),
    (&DIRTY_KETTLE, &[]),
    (&SWARM_OF_FLIES, &[]),
    (&DRAGON_FLAME_SECT_BADGE, &[]),
    (&ORDERS_FROM_MINAX, &[]),
    (&SERPENT_FANG_SECT_BADGE, &[]),
    (&TIGER_CLAW_SECT_BADGE, &[]),
    (&TRAVESTYS_COLLECTION_OF_SHELLS, &[]),
    (&TRAVESTYS_FINE_TEAKWOOD_TRAY, &[]),
    (&TRAVESTYS_SUSHI_PREPARATIONS, &[]),
    (&TRIBAL_BERRY, &[]),
    (&HORN_OF_THE_DREADHORN, &[]),
    (&MANGLED_HEAD_OF_DREADHORN, &[]),
    (&TAINTED_MUSHROOM, &[]),
];

/// `[Flippable(..)]` of the classes: data for the flip command.
pub static FLIPPABLE: &[(&ItemType, &[i32])] = &[
    (&SMALL_FLASK, &[0x182E, 0x182F, 0x1830, 0x1831]),
    (&MEDIUM_FLASK, &[0x182A, 0x182B, 0x182C, 0x182D]),
    (&LARGE_FLASK, &[0x183B, 0x183C, 0x183D]),
    (
        &CURVED_FLASK,
        &[0x1832, 0x1833, 0x1834, 0x1835, 0x1836, 0x1837],
    ),
    (&LONG_FLASK, &[0x1838, 0x1839, 0x183A]),
    (&SPINNING_HOURGLASS, &[0x1810, 0x1811]),
    (&EMPTY_VIALS_W_RACK, &[0x185B, 0x185C]),
    (&FULL_VIALS_W_RACK, &[0x185D, 0x185E]),
    (&HORN_OF_THE_DREADHORN, &[0x315C, 0x315D]),
    (&MANGLED_HEAD_OF_DREADHORN, &[0x3156, 0x3157]),
];

/// Registers the classes of this file.
pub fn register(types: &mut TypeRegistry) {
    for (t, aliases) in ALL {
        types.register_item(t, aliases);
    }
}
