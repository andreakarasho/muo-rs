//! The Resources classes that are plain tables (a constructor and constant overrides); the classes with logic
//! are hand-written next to this file.

use crate::simple::leaf;
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::{ITEM_BASE, ItemType, ops};
use muo_server::type_registry::TypeRegistry;

/// `Foo(int amountFrom, int amountTo)` of the resources that have it: the class template, then its script's
/// `on_spawn(e, amountFrom, amountTo)` sets `Amount = Utility.RandomMinMax(amountFrom, amountTo)`.
pub fn amount_range_new(w: &mut World, ty: &'static ItemType, from: i32, to: i32) -> ItemId {
    use muo_server::script_hooks::ScriptArg;
    let id = ops::new_item_with_id(w, ty, 0);
    let args = [
        ScriptArg::Int(i64::from(from)),
        ScriptArg::Int(i64::from(to)),
    ];
    muo_content::templates::apply_item_with(w, id, ty.full_name, &args);
    id
}

/// `Feather(i32 amount)`.
pub fn feather_new(w: &mut World, ty: &'static ItemType, amount: i32) -> ItemId {
    let id = crate::simple::from_template(w, ty, "Server.Items.Feather");
    ops::set_amount(w, id, amount);
    id
}

leaf! {
    /// `Server.Items.Feather`.
    FEATHER = "Server.Items.Feather" : ITEM_BASE, v0;
    construct: Some(|w| feather_new(w, &FEATHER, 1)),
    default_weight: crate::class_props::default_weight,
}

/// `Shaft(i32 amount)`.
pub fn shaft_new(w: &mut World, ty: &'static ItemType, amount: i32) -> ItemId {
    let id = crate::simple::from_template(w, ty, "Server.Items.Shaft");
    ops::set_amount(w, id, amount);
    id
}

leaf! {
    /// `Server.Items.Shaft`.
    SHAFT = "Server.Items.Shaft" : ITEM_BASE, v0;
    construct: Some(|w| shaft_new(w, &SHAFT, 1)),
    default_weight: crate::class_props::default_weight,
}

/// `BarkFragment(i32 amount)`.
pub fn bark_fragment_new(w: &mut World, ty: &'static ItemType, amount: i32) -> ItemId {
    let id = crate::simple::from_template(w, ty, "Server.Items.BarkFragment");
    ops::set_amount(w, id, amount);
    id
}

leaf! {
    /// `Server.Items.BarkFragment`.
    BARK_FRAGMENT = "Server.Items.BarkFragment" : ITEM_BASE, v0;
    construct: Some(|w| bark_fragment_new(w, &BARK_FRAGMENT, 1)),
}

/// `Blight(i32 amount)`.
pub fn blight_new(w: &mut World, ty: &'static ItemType, amount: i32) -> ItemId {
    let id = crate::simple::from_template(w, ty, "Server.Items.Blight");
    ops::set_amount(w, id, amount);
    id
}

leaf! {
    /// `Server.Items.Blight`.
    BLIGHT = "Server.Items.Blight" : ITEM_BASE, v0;
    construct: Some(|w| blight_new(w, &BLIGHT, 1)),
}

/// `BlueDiamond(i32 amount)`.
pub fn blue_diamond_new(w: &mut World, ty: &'static ItemType, amount: i32) -> ItemId {
    let id = crate::simple::from_template(w, ty, "Server.Items.BlueDiamond");
    ops::set_amount(w, id, amount);
    id
}

leaf! {
    /// `Server.Items.BlueDiamond`.
    BLUE_DIAMOND = "Server.Items.BlueDiamond" : ITEM_BASE, v0;
    construct: Some(|w| blue_diamond_new(w, &BLUE_DIAMOND, 1)),
}

/// `BrilliantAmber(i32 amount)`.
pub fn brilliant_amber_new(w: &mut World, ty: &'static ItemType, amount: i32) -> ItemId {
    let id = crate::simple::from_template(w, ty, "Server.Items.BrilliantAmber");
    ops::set_amount(w, id, amount);
    id
}

leaf! {
    /// `Server.Items.BrilliantAmber`.
    BRILLIANT_AMBER = "Server.Items.BrilliantAmber" : ITEM_BASE, v0;
    construct: Some(|w| brilliant_amber_new(w, &BRILLIANT_AMBER, 1)),
}

/// `CapturedEssence(i32 amount)`.
pub fn captured_essence_new(w: &mut World, ty: &'static ItemType, amount: i32) -> ItemId {
    let id = crate::simple::from_template(w, ty, "Server.Items.CapturedEssence");
    ops::set_amount(w, id, amount);
    id
}

leaf! {
    /// `Server.Items.CapturedEssence`.
    CAPTURED_ESSENCE = "Server.Items.CapturedEssence" : ITEM_BASE, v0;
    construct: Some(|w| captured_essence_new(w, &CAPTURED_ESSENCE, 1)),
}

/// `Corruption(i32 amount)`.
pub fn corruption_new(w: &mut World, ty: &'static ItemType, amount: i32) -> ItemId {
    let id = crate::simple::from_template(w, ty, "Server.Items.Corruption");
    ops::set_amount(w, id, amount);
    id
}

leaf! {
    /// `Server.Items.Corruption`.
    CORRUPTION = "Server.Items.Corruption" : ITEM_BASE, v0;
    construct: Some(|w| corruption_new(w, &CORRUPTION, 1)),
}

/// `DarkSapphire(i32 amount)`.
pub fn dark_sapphire_new(w: &mut World, ty: &'static ItemType, amount: i32) -> ItemId {
    let id = crate::simple::from_template(w, ty, "Server.Items.DarkSapphire");
    ops::set_amount(w, id, amount);
    id
}

leaf! {
    /// `Server.Items.DarkSapphire`.
    DARK_SAPPHIRE = "Server.Items.DarkSapphire" : ITEM_BASE, v0;
    construct: Some(|w| dark_sapphire_new(w, &DARK_SAPPHIRE, 1)),
}

/// `DiseasedBark(i32 amount)`.
pub fn diseased_bark_new(w: &mut World, ty: &'static ItemType, amount: i32) -> ItemId {
    let id = crate::simple::from_template(w, ty, "Server.Items.DiseasedBark");
    ops::set_amount(w, id, amount);
    id
}

leaf! {
    /// `Server.Items.DiseasedBark`.
    DISEASED_BARK = "Server.Items.DiseasedBark" : ITEM_BASE, v0;
    construct: Some(|w| diseased_bark_new(w, &DISEASED_BARK, 1)),
}

/// `DreadHornMane(i32 amount)`.
pub fn dread_horn_mane_new(w: &mut World, ty: &'static ItemType, amount: i32) -> ItemId {
    let id = crate::simple::from_template(w, ty, "Server.Items.DreadHornMane");
    ops::set_amount(w, id, amount);
    id
}

leaf! {
    /// `Server.Items.DreadHornMane`.
    DREAD_HORN_MANE = "Server.Items.DreadHornMane" : ITEM_BASE, v0;
    construct: Some(|w| dread_horn_mane_new(w, &DREAD_HORN_MANE, 1)),
}

/// `EcruCitrine(i32 amount)`.
pub fn ecru_citrine_new(w: &mut World, ty: &'static ItemType, amount: i32) -> ItemId {
    let id = crate::simple::from_template(w, ty, "Server.Items.EcruCitrine");
    ops::set_amount(w, id, amount);
    id
}

leaf! {
    /// `Server.Items.EcruCitrine`.
    ECRU_CITRINE = "Server.Items.EcruCitrine" : ITEM_BASE, v0;
    construct: Some(|w| ecru_citrine_new(w, &ECRU_CITRINE, 1)),
}

/// `EyeOfTheTravesty(i32 amount)`.
pub fn eye_of_the_travesty_new(w: &mut World, ty: &'static ItemType, amount: i32) -> ItemId {
    let id = crate::simple::from_template(w, ty, "Server.Items.EyeOfTheTravesty");
    ops::set_amount(w, id, amount);
    id
}

leaf! {
    /// `Server.Items.EyeOfTheTravesty`.
    EYE_OF_THE_TRAVESTY = "Server.Items.EyeOfTheTravesty" : ITEM_BASE, v0;
    construct: Some(|w| eye_of_the_travesty_new(w, &EYE_OF_THE_TRAVESTY, 1)),
}

/// `FireRuby(i32 amount)`.
pub fn fire_ruby_new(w: &mut World, ty: &'static ItemType, amount: i32) -> ItemId {
    let id = crate::simple::from_template(w, ty, "Server.Items.FireRuby");
    ops::set_amount(w, id, amount);
    id
}

leaf! {
    /// `Server.Items.FireRuby`.
    FIRE_RUBY = "Server.Items.FireRuby" : ITEM_BASE, v0;
    construct: Some(|w| fire_ruby_new(w, &FIRE_RUBY, 1)),
}

/// `GrizzledBones(i32 amount)`.
pub fn grizzled_bones_new(w: &mut World, ty: &'static ItemType, amount: i32) -> ItemId {
    let id = crate::simple::from_template(w, ty, "Server.Items.GrizzledBones");
    ops::set_amount(w, id, amount);
    id
}

leaf! {
    /// `Server.Items.GrizzledBones`.
    GRIZZLED_BONES = "Server.Items.GrizzledBones" : ITEM_BASE, v0;
    construct: Some(|w| grizzled_bones_new(w, &GRIZZLED_BONES, 1)),
}

/// `LardOfParoxysmus(i32 amount)`.
pub fn lard_of_paroxysmus_new(w: &mut World, ty: &'static ItemType, amount: i32) -> ItemId {
    let id = crate::simple::from_template(w, ty, "Server.Items.LardOfParoxysmus");
    ops::set_amount(w, id, amount);
    id
}

leaf! {
    /// `Server.Items.LardOfParoxysmus`.
    LARD_OF_PAROXYSMUS = "Server.Items.LardOfParoxysmus" : ITEM_BASE, v0;
    construct: Some(|w| lard_of_paroxysmus_new(w, &LARD_OF_PAROXYSMUS, 1)),
}

/// `LuminescentFungi(i32 amount)`.
pub fn luminescent_fungi_new(w: &mut World, ty: &'static ItemType, amount: i32) -> ItemId {
    let id = crate::simple::from_template(w, ty, "Server.Items.LuminescentFungi");
    ops::set_amount(w, id, amount);
    id
}

leaf! {
    /// `Server.Items.LuminescentFungi`.
    LUMINESCENT_FUNGI = "Server.Items.LuminescentFungi" : ITEM_BASE, v0;
    construct: Some(|w| luminescent_fungi_new(w, &LUMINESCENT_FUNGI, 1)),
}

/// `Muculent(i32 amount)`.
pub fn muculent_new(w: &mut World, ty: &'static ItemType, amount: i32) -> ItemId {
    let id = crate::simple::from_template(w, ty, "Server.Items.Muculent");
    ops::set_amount(w, id, amount);
    id
}

leaf! {
    /// `Server.Items.Muculent`.
    MUCULENT = "Server.Items.Muculent" : ITEM_BASE, v0;
    construct: Some(|w| muculent_new(w, &MUCULENT, 1)),
}

/// `ParasiticPlant(i32 amount)`.
pub fn parasitic_plant_new(w: &mut World, ty: &'static ItemType, amount: i32) -> ItemId {
    let id = crate::simple::from_template(w, ty, "Server.Items.ParasiticPlant");
    ops::set_amount(w, id, amount);
    id
}

leaf! {
    /// `Server.Items.ParasiticPlant`.
    PARASITIC_PLANT = "Server.Items.ParasiticPlant" : ITEM_BASE, v0;
    construct: Some(|w| parasitic_plant_new(w, &PARASITIC_PLANT, 1)),
}

/// `PerfectEmerald(i32 amount)`.
pub fn perfect_emerald_new(w: &mut World, ty: &'static ItemType, amount: i32) -> ItemId {
    let id = crate::simple::from_template(w, ty, "Server.Items.PerfectEmerald");
    ops::set_amount(w, id, amount);
    id
}

leaf! {
    /// `Server.Items.PerfectEmerald`.
    PERFECT_EMERALD = "Server.Items.PerfectEmerald" : ITEM_BASE, v0;
    construct: Some(|w| perfect_emerald_new(w, &PERFECT_EMERALD, 1)),
}

/// `PristineDreadHorn()`.
pub fn pristine_dread_horn_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.PristineDreadHorn")
}

leaf! {
    /// `Server.Items.PristineDreadHorn`.
    PRISTINE_DREAD_HORN = "Server.Items.PristineDreadHorn" : ITEM_BASE, v0;
    construct: Some(|w| pristine_dread_horn_new(w, &PRISTINE_DREAD_HORN)),
}

/// `Putrefication(i32 amount)`.
pub fn putrefication_new(w: &mut World, ty: &'static ItemType, amount: i32) -> ItemId {
    let id = crate::simple::from_template(w, ty, "Server.Items.Putrefication");
    ops::set_amount(w, id, amount);
    id
}

leaf! {
    /// `Server.Items.Putrefication`.
    PUTREFICATION = "Server.Items.Putrefication" : ITEM_BASE, v0;
    construct: Some(|w| putrefication_new(w, &PUTREFICATION, 1)),
}

/// `Scourge(i32 amount)`.
pub fn scourge_new(w: &mut World, ty: &'static ItemType, amount: i32) -> ItemId {
    let id = crate::simple::from_template(w, ty, "Server.Items.Scourge");
    ops::set_amount(w, id, amount);
    id
}

leaf! {
    /// `Server.Items.Scourge`.
    SCOURGE = "Server.Items.Scourge" : ITEM_BASE, v0;
    construct: Some(|w| scourge_new(w, &SCOURGE, 1)),
}

/// `SwitchItem(i32 amount)`.
pub fn switch_item_new(w: &mut World, ty: &'static ItemType, amount: i32) -> ItemId {
    let id = crate::simple::from_template(w, ty, "Server.Items.SwitchItem");
    ops::set_amount(w, id, amount);
    id
}

leaf! {
    /// `Server.Items.SwitchItem`.
    SWITCH_ITEM = "Server.Items.SwitchItem" : ITEM_BASE, v0;
    construct: Some(|w| switch_item_new(w, &SWITCH_ITEM, 1)),
}

/// `Taint(i32 amount)`.
pub fn taint_new(w: &mut World, ty: &'static ItemType, amount: i32) -> ItemId {
    let id = crate::simple::from_template(w, ty, "Server.Items.Taint");
    ops::set_amount(w, id, amount);
    id
}

leaf! {
    /// `Server.Items.Taint`.
    TAINT = "Server.Items.Taint" : ITEM_BASE, v0;
    construct: Some(|w| taint_new(w, &TAINT, 1)),
}

/// `Turquoise(i32 amount)`.
pub fn turquoise_new(w: &mut World, ty: &'static ItemType, amount: i32) -> ItemId {
    let id = crate::simple::from_template(w, ty, "Server.Items.Turquoise");
    ops::set_amount(w, id, amount);
    id
}

leaf! {
    /// `Server.Items.Turquoise`.
    TURQUOISE = "Server.Items.Turquoise" : ITEM_BASE, v0;
    construct: Some(|w| turquoise_new(w, &TURQUOISE, 1)),
}

/// `WhitePearl(i32 amount)`.
pub fn white_pearl_new(w: &mut World, ty: &'static ItemType, amount: i32) -> ItemId {
    let id = crate::simple::from_template(w, ty, "Server.Items.WhitePearl");
    ops::set_amount(w, id, amount);
    id
}

leaf! {
    /// `Server.Items.WhitePearl`.
    WHITE_PEARL = "Server.Items.WhitePearl" : ITEM_BASE, v0;
    construct: Some(|w| white_pearl_new(w, &WHITE_PEARL, 1)),
}

/// Every class of this file: the type and its `[TypeAlias]` names.
pub static ALL: &[(&ItemType, &[&str])] = &[
    (&FEATHER, &[]),
    (&SHAFT, &[]),
    (&BARK_FRAGMENT, &[]),
    (&BLIGHT, &[]),
    (&BLUE_DIAMOND, &[]),
    (&BRILLIANT_AMBER, &[]),
    (&CAPTURED_ESSENCE, &[]),
    (&CORRUPTION, &[]),
    (&DARK_SAPPHIRE, &[]),
    (&DISEASED_BARK, &[]),
    (&DREAD_HORN_MANE, &[]),
    (&ECRU_CITRINE, &[]),
    (&EYE_OF_THE_TRAVESTY, &[]),
    (&FIRE_RUBY, &[]),
    (&GRIZZLED_BONES, &[]),
    (&LARD_OF_PAROXYSMUS, &[]),
    (&LUMINESCENT_FUNGI, &[]),
    (&MUCULENT, &[]),
    (&PARASITIC_PLANT, &[]),
    (&PERFECT_EMERALD, &[]),
    (&PRISTINE_DREAD_HORN, &[]),
    (&PUTREFICATION, &[]),
    (&SCOURGE, &[]),
    (&SWITCH_ITEM, &[]),
    (&TAINT, &[]),
    (&TURQUOISE, &[]),
    (&WHITE_PEARL, &[]),
];

/// `[Flippable(a, b)]` of the classes (the flip command reads it).
pub static FLIPPABLE: &[(&ItemType, i32, i32)] = &[(&PRISTINE_DREAD_HORN, 0x315A, 0x315B)];

/// The interfaces a class lists after its base class (`is ICommodity` and the like); none has a consumer yet.
pub static IMPLEMENTS: &[(&ItemType, &str)] = &[(&FEATHER, "ICommodity"), (&SHAFT, "ICommodity")];

/// `ICommodity` members of a class: (type, `IsDeedable`); `DescriptionNumber` is `LabelNumber`.
pub static COMMODITIES: &[(&ItemType, bool)] = &[(&FEATHER, true), (&SHAFT, true)];

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
