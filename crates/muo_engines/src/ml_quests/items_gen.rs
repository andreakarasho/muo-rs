//! The simple ML quest items: classes made only of constructor statements and plain overrides.

use super::item_support::*;
use muo_content::hooks::rnd;
use muo_content::items::cont::{BACKPACK, new_base_container};
use muo_primitives::utility;
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::{ITEM_BASE, ItemType, LootType, ops};
use muo_server::layer::Layer;
use muo_server::type_registry::TypeRegistry;

pub fn a_bauble_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0x23B);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
leaf! {
    /// `Server.Items.ABauble`.
    A_BAUBLE = "Server.Items.ABauble" : ITEM_BASE, v0;
    construct: Some(|w| a_bauble_new(w, &A_BAUBLE)),
    label_number: |_, _| 1073137,
}

pub fn alchemists_bandage_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0xE21);
    ops::set_loot_type(w, id, LootType::Blessed);
    hue(w, id, 0x482);
    id
}
leaf! {
    /// `Server.Items.AlchemistsBandage`.
    ALCHEMISTS_BANDAGE = "Server.Items.AlchemistsBandage" : ITEM_BASE, v0;
    construct: Some(|w| alchemists_bandage_new(w, &ALCHEMISTS_BANDAGE)),
    label_number: |_, _| 1075452,
    nontransferable: |_, _| true,
    add_name_properties: |w, id, list| { (ITEM_BASE.add_name_properties)(w, id, list); add_quest_item_property(w, id, list); },
}

pub fn andric_satchel_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = new_base_container(w, ty, 0xE75);
    layer(w, id, Layer::Backpack);
    let h = utility::random_bright_hue(&mut w.rng);
    hue(w, id, h);
    drop_new(w, id, "Feather", Some(10));
    drop_new(w, id, "FletcherTools", None);
    id
}
leaf! {
    /// `Server.Items.AndricSatchel`.
    ANDRIC_SATCHEL = "Server.Items.AndricSatchel" : BACKPACK, v0;
    construct: Some(|w| andric_satchel_new(w, &ANDRIC_SATCHEL)),
}

pub fn asandos_satchel_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = new_base_container(w, ty, 0xE75);
    layer(w, id, Layer::Backpack);
    let h = utility::random_bright_hue(&mut w.rng);
    hue(w, id, h);
    drop_new(w, id, "SackFlour", None);
    drop_new(w, id, "Skillet", None);
    id
}
leaf! {
    /// `Server.Items.AsandosSatchel`.
    ASANDOS_SATCHEL = "Server.Items.AsandosSatchel" : BACKPACK, v0;
    construct: Some(|w| asandos_satchel_new(w, &ASANDOS_SATCHEL)),
}

pub fn basin_of_crystal_clear_water_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0x1008);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
leaf! {
    /// `Server.Items.BasinOfCrystalClearWater`.
    BASIN_OF_CRYSTAL_CLEAR_WATER = "Server.Items.BasinOfCrystalClearWater" : ITEM_BASE, v0;
    construct: Some(|w| basin_of_crystal_clear_water_new(w, &BASIN_OF_CRYSTAL_CLEAR_WATER)),
    label_number: |_, _| 1075303,
    nontransferable: |_, _| true,
    add_name_properties: |w, id, list| { (ITEM_BASE.add_name_properties)(w, id, list); add_quest_item_property(w, id, list); },
}

pub fn brides_letter_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0x14ED);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
leaf! {
    /// `Server.Items.BridesLetter`.
    BRIDES_LETTER = "Server.Items.BridesLetter" : ITEM_BASE, v0;
    construct: Some(|w| brides_letter_new(w, &BRIDES_LETTER)),
    label_number: |_, _| 1075301,
    nontransferable: |_, _| true,
    add_name_properties: |w, id, list| { (ITEM_BASE.add_name_properties)(w, id, list); add_quest_item_property(w, id, list); },
}

pub fn completed_tuition_reimbursement_form_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0x14F0);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
leaf! {
    /// `Server.Items.CompletedTuitionReimbursementForm`.
    COMPLETED_TUITION_REIMBURSEMENT_FORM = "Server.Items.CompletedTuitionReimbursementForm" : ITEM_BASE, v0;
    construct: Some(|w| completed_tuition_reimbursement_form_new(w, &COMPLETED_TUITION_REIMBURSEMENT_FORM)),
    label_number: |_, _| 1074625,
    nontransferable: |_, _| true,
    add_name_properties: |w, id, list| { (ITEM_BASE.add_name_properties)(w, id, list); add_quest_item_property(w, id, list); },
}

pub fn dread_spider_silk_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0xDF8);
    ops::set_loot_type(w, id, LootType::Blessed);
    hue(w, id, 0x481);
    id
}
leaf! {
    /// `Server.Items.DreadSpiderSilk`.
    DREAD_SPIDER_SILK = "Server.Items.DreadSpiderSilk" : ITEM_BASE, v0;
    construct: Some(|w| dread_spider_silk_new(w, &DREAD_SPIDER_SILK)),
    label_number: |_, _| 1075319,
    nontransferable: |_, _| true,
    add_name_properties: |w, id, list| { (ITEM_BASE.add_name_properties)(w, id, list); add_quest_item_property(w, id, list); },
}

pub fn fragment_of_a_map_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0x14ED);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
leaf! {
    /// `Server.Items.FragmentOfAMap`.
    FRAGMENT_OF_A_MAP = "Server.Items.FragmentOfAMap" : ITEM_BASE, v0;
    construct: Some(|w| fragment_of_a_map_new(w, &FRAGMENT_OF_A_MAP)),
    label_number: |_, _| 1074533,
}

pub fn fragment_of_a_map_delivery_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0x14ED);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
leaf! {
    /// `Server.Items.FragmentOfAMapDelivery`.
    FRAGMENT_OF_A_MAP_DELIVERY = "Server.Items.FragmentOfAMapDelivery" : ITEM_BASE, v0;
    construct: Some(|w| fragment_of_a_map_delivery_new(w, &FRAGMENT_OF_A_MAP_DELIVERY)),
    label_number: |_, _| 1074533,
    nontransferable: |_, _| true,
    add_name_properties: |w, id, list| { (ITEM_BASE.add_name_properties)(w, id, list); add_quest_item_property(w, id, list); },
}

pub fn friend_of_the_library_token_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0x2F58);
    layer(w, id, Layer::Talisman);
    hue(w, id, 0x28A);
    id
}
leaf! {
    /// `Server.Items.FriendOfTheLibraryToken`.
    FRIEND_OF_THE_LIBRARY_TOKEN = "Server.Items.FriendOfTheLibraryToken" : ITEM_BASE, v0;
    construct: Some(|w| friend_of_the_library_token_new(w, &FRIEND_OF_THE_LIBRARY_TOKEN)),
    label_number: |_, _| 1073136,
}

pub fn friends_of_the_library_application_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0xEC0);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
leaf! {
    /// `Server.Items.FriendsOfTheLibraryApplication`.
    FRIENDS_OF_THE_LIBRARY_APPLICATION = "Server.Items.FriendsOfTheLibraryApplication" : ITEM_BASE, v0;
    construct: Some(|w| friends_of_the_library_application_new(w, &FRIENDS_OF_THE_LIBRARY_APPLICATION)),
    label_number: |_, _| 1073131,
    nontransferable: |_, _| true,
    add_name_properties: |w, id, list| { (ITEM_BASE.add_name_properties)(w, id, list); add_quest_item_property(w, id, list); },
}

pub fn gaman_horns_new(w: &mut World, ty: &'static ItemType, amount: i32) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0x1084);
    ops::set_loot_type(w, id, LootType::Blessed);
    ops::set_stackable(w, id, true);
    ops::set_amount(w, id, amount);
    hue(w, id, 0x395);
    id
}
leaf! {
    /// `Server.Items.GamanHorns`.
    GAMAN_HORNS = "Server.Items.GamanHorns" : ITEM_BASE, v0;
    construct: Some(|w| gaman_horns_new(w, &GAMAN_HORNS, 1)),
    label_number: |_, _| 1074557,
}

pub fn gervis_satchel_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = new_base_container(w, ty, 0xE75);
    layer(w, id, Layer::Backpack);
    let h = utility::random_bright_hue(&mut w.rng);
    hue(w, id, h);
    drop_new(w, id, "IronIngot", Some(10));
    drop_new(w, id, "SmithHammer", None);
    id
}
leaf! {
    /// `Server.Items.GervisSatchel`.
    GERVIS_SATCHEL = "Server.Items.GervisSatchel" : BACKPACK, v0;
    construct: Some(|w| gervis_satchel_new(w, &GERVIS_SATCHEL)),
}

pub fn hargrove_satchel_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = new_base_container(w, ty, 0xE75);
    layer(w, id, Layer::Backpack);
    let h = utility::random_bright_hue(&mut w.rng);
    hue(w, id, h);
    drop_new(w, id, "Gold", Some(15));
    drop_new(w, id, "Hatchet", None);
    id
}
leaf! {
    /// `Server.Items.HargroveSatchel`.
    HARGROVE_SATCHEL = "Server.Items.HargroveSatchel" : BACKPACK, v0;
    construct: Some(|w| hargrove_satchel_new(w, &HARGROVE_SATCHEL)),
}

pub fn kirin_brains_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0x1CF0);
    ops::set_loot_type(w, id, LootType::Blessed);
    hue(w, id, 0xD7);
    id
}
leaf! {
    /// `Server.Items.KirinBrains`.
    KIRIN_BRAINS = "Server.Items.KirinBrains" : ITEM_BASE, v0;
    construct: Some(|w| kirin_brains_new(w, &KIRIN_BRAINS)),
    label_number: |_, _| 1074612,
}

pub fn lowel_satchel_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = new_base_container(w, ty, 0xE75);
    layer(w, id, Layer::Backpack);
    let h = utility::random_bright_hue(&mut w.rng);
    hue(w, id, h);
    drop_new(w, id, "Board", Some(10));
    drop_new(w, id, "DovetailSaw", None);
    id
}
leaf! {
    /// `Server.Items.LowelSatchel`.
    LOWEL_SATCHEL = "Server.Items.LowelSatchel" : BACKPACK, v0;
    construct: Some(|w| lowel_satchel_new(w, &LOWEL_SATCHEL)),
}

pub fn mirror_of_purification_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0x1008);
    ops::set_loot_type(w, id, LootType::Blessed);
    hue(w, id, 0x530);
    id
}
leaf! {
    /// `Server.Items.MirrorOfPurification`.
    MIRROR_OF_PURIFICATION = "Server.Items.MirrorOfPurification" : ITEM_BASE, v0;
    construct: Some(|w| mirror_of_purification_new(w, &MIRROR_OF_PURIFICATION)),
    label_number: |_, _| 1075304,
}

pub fn mugg_satchel_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = new_base_container(w, ty, 0xE75);
    layer(w, id, Layer::Backpack);
    let h = utility::random_bright_hue(&mut w.rng);
    hue(w, id, h);
    drop_new(w, id, "Pickaxe", None);
    drop_new(w, id, "Pickaxe", None);
    id
}
leaf! {
    /// `Server.Items.MuggSatchel`.
    MUGG_SATCHEL = "Server.Items.MuggSatchel" : BACKPACK, v0;
    construct: Some(|w| mugg_satchel_new(w, &MUGG_SATCHEL)),
}

pub fn notarized_application_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0x14EF);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
leaf! {
    /// `Server.Items.NotarizedApplication`.
    NOTARIZED_APPLICATION = "Server.Items.NotarizedApplication" : ITEM_BASE, v0;
    construct: Some(|w| notarized_application_new(w, &NOTARIZED_APPLICATION)),
    label_number: |_, _| 1073135,
    nontransferable: |_, _| true,
    add_name_properties: |w, id, list| { (ITEM_BASE.add_name_properties)(w, id, list); add_quest_item_property(w, id, list); },
}

pub fn official_sealing_wax_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0x1426);
    ops::set_loot_type(w, id, LootType::Blessed);
    hue(w, id, 0x84);
    id
}
leaf! {
    /// `Server.Items.OfficialSealingWax`.
    OFFICIAL_SEALING_WAX = "Server.Items.OfficialSealingWax" : ITEM_BASE, v0;
    construct: Some(|w| official_sealing_wax_new(w, &OFFICIAL_SEALING_WAX)),
    label_number: |_, _| 1072744,
    nontransferable: |_, _| true,
    add_name_properties: |w, id, list| { (ITEM_BASE.add_name_properties)(w, id, list); add_quest_item_property(w, id, list); },
}

pub fn portrait_of_the_bride_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0xE9F);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
leaf! {
    /// `Server.Items.PortraitOfTheBride`.
    PORTRAIT_OF_THE_BRIDE = "Server.Items.PortraitOfTheBride" : ITEM_BASE, v0;
    construct: Some(|w| portrait_of_the_bride_new(w, &PORTRAIT_OF_THE_BRIDE)),
    label_number: |_, _| 1075300,
    nontransferable: |_, _| true,
    add_name_properties: |w, id, list| { (ITEM_BASE.add_name_properties)(w, id, list); add_quest_item_property(w, id, list); },
}

pub fn reginas_letter_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0x14ED);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
leaf! {
    /// `Server.Items.ReginasLetter`.
    REGINAS_LETTER = "Server.Items.ReginasLetter" : ITEM_BASE, v0;
    construct: Some(|w| reginas_letter_new(w, &REGINAS_LETTER)),
    label_number: |_, _| 1075306,
    nontransferable: |_, _| true,
    add_name_properties: |w, id, list| { (ITEM_BASE.add_name_properties)(w, id, list); add_quest_item_property(w, id, list); },
}

pub fn sadrah_satchel_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = new_base_container(w, ty, 0xE75);
    layer(w, id, Layer::Backpack);
    let h = utility::random_bright_hue(&mut w.rng);
    hue(w, id, h);
    drop_new(w, id, "Bloodmoss", Some(10));
    drop_new(w, id, "MortarPestle", None);
    id
}
leaf! {
    /// `Server.Items.SadrahSatchel`.
    SADRAH_SATCHEL = "Server.Items.SadrahSatchel" : BACKPACK, v0;
    construct: Some(|w| sadrah_satchel_new(w, &SADRAH_SATCHEL)),
}

pub fn sap_of_sosaria_new(w: &mut World, ty: &'static ItemType, amount: i32) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0x1848);
    ops::set_loot_type(w, id, LootType::Blessed);
    ops::set_stackable(w, id, true);
    ops::set_amount(w, id, amount);
    id
}
leaf! {
    /// `Server.Items.SapOfSosaria`.
    SAP_OF_SOSARIA = "Server.Items.SapOfSosaria" : ITEM_BASE, v0;
    construct: Some(|w| sap_of_sosaria_new(w, &SAP_OF_SOSARIA, 1)),
    label_number: |_, _| 1074178,
}

pub fn sealed_notes_for_jamal_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0xEF9);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
leaf! {
    /// `Server.Items.SealedNotesForJamal`.
    SEALED_NOTES_FOR_JAMAL = "Server.Items.SealedNotesForJamal" : ITEM_BASE, v0;
    construct: Some(|w| sealed_notes_for_jamal_new(w, &SEALED_NOTES_FOR_JAMAL)),
    label_number: |_, _| 1074998,
    default_weight: |_, _| 1.0,
    nontransferable: |_, _| true,
    add_name_properties: |w, id, list| { (ITEM_BASE.add_name_properties)(w, id, list); add_quest_item_property(w, id, list); },
}

pub fn sealing_wax_order_addressed_to_petrus_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0xEBF);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
leaf! {
    /// `Server.Items.SealingWaxOrderAddressedToPetrus`.
    SEALING_WAX_ORDER_ADDRESSED_TO_PETRUS = "Server.Items.SealingWaxOrderAddressedToPetrus" : ITEM_BASE, v0;
    construct: Some(|w| sealing_wax_order_addressed_to_petrus_new(w, &SEALING_WAX_ORDER_ADDRESSED_TO_PETRUS)),
    label_number: |_, _| 1073132,
    nontransferable: |_, _| true,
    add_name_properties: |w, id, list| { (ITEM_BASE.add_name_properties)(w, id, list); add_quest_item_property(w, id, list); },
}

pub fn severed_elf_ears_new(w: &mut World, ty: &'static ItemType, amount: i32) -> ItemId {
    let gid = [0x312D, 0x312E][rnd::below(w, 2) as usize];
    let id = ops::new_item_with_id(w, ty, gid);
    ops::set_stackable(w, id, true);
    ops::set_amount(w, id, amount);
    id
}
leaf! {
    /// `Server.Items.SeveredElfEars`.
    SEVERED_ELF_EARS = "Server.Items.SeveredElfEars" : ITEM_BASE, v0;
    construct: Some(|w| severed_elf_ears_new(w, &SEVERED_ELF_EARS, 1)),
}

pub fn severed_human_ears_new(w: &mut World, ty: &'static ItemType, amount: i32) -> ItemId {
    let gid = [0x312F, 0x3130][rnd::below(w, 2) as usize];
    let id = ops::new_item_with_id(w, ty, gid);
    ops::set_stackable(w, id, true);
    ops::set_amount(w, id, amount);
    id
}
leaf! {
    /// `Server.Items.SeveredHumanEars`.
    SEVERED_HUMAN_EARS = "Server.Items.SeveredHumanEars" : ITEM_BASE, v0;
    construct: Some(|w| severed_human_ears_new(w, &SEVERED_HUMAN_EARS, 1)),
}

pub fn signed_tuition_reimbursement_form_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0x14F0);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
leaf! {
    /// `Server.Items.SignedTuitionReimbursementForm`.
    SIGNED_TUITION_REIMBURSEMENT_FORM = "Server.Items.SignedTuitionReimbursementForm" : ITEM_BASE, v0;
    construct: Some(|w| signed_tuition_reimbursement_form_new(w, &SIGNED_TUITION_REIMBURSEMENT_FORM)),
    label_number: |_, _| 1074614,
    nontransferable: |_, _| true,
    add_name_properties: |w, id, list| { (ITEM_BASE.add_name_properties)(w, id, list); add_quest_item_property(w, id, list); },
}

pub fn special_treat_for_drithen_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0x21B);
    ops::set_loot_type(w, id, LootType::Blessed);
    hue(w, id, 0x489);
    id
}
leaf! {
    /// `Server.Items.SpecialTreatForDrithen`.
    SPECIAL_TREAT_FOR_DRITHEN = "Server.Items.SpecialTreatForDrithen" : ITEM_BASE, v0;
    construct: Some(|w| special_treat_for_drithen_new(w, &SPECIAL_TREAT_FOR_DRITHEN)),
    label_number: |_, _| 1074517,
}

pub fn spirit_bottle_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0xEFB);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
leaf! {
    /// `Server.Items.SpiritBottle`.
    SPIRIT_BOTTLE = "Server.Items.SpiritBottle" : ITEM_BASE, v0;
    construct: Some(|w| spirit_bottle_new(w, &SPIRIT_BOTTLE)),
    label_number: |_, _| 1075283,
    nontransferable: |_, _| true,
    add_name_properties: |w, id, list| { (ITEM_BASE.add_name_properties)(w, id, list); add_quest_item_property(w, id, list); },
}

pub fn stout_whip_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0x166F);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
leaf! {
    /// `Server.Items.StoutWhip`.
    STOUT_WHIP = "Server.Items.StoutWhip" : ITEM_BASE, v0;
    construct: Some(|w| stout_whip_new(w, &STOUT_WHIP)),
    label_number: |_, _| 1074812,
}

pub fn tainted_tree_sample_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0xDE2);
    ops::set_loot_type(w, id, LootType::Blessed);
    hue(w, id, 0x9D);
    id
}
leaf! {
    /// `Server.Items.TaintedTreeSample`.
    TAINTED_TREE_SAMPLE = "Server.Items.TaintedTreeSample" : ITEM_BASE, v0;
    construct: Some(|w| tainted_tree_sample_new(w, &TAINTED_TREE_SAMPLE)),
    label_number: |_, _| 1074997,
    nontransferable: |_, _| true,
    add_name_properties: |w, id, list| { (ITEM_BASE.add_name_properties)(w, id, list); add_quest_item_property(w, id, list); },
}

pub fn tuition_reimbursement_form_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0xE3A);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}
leaf! {
    /// `Server.Items.TuitionReimbursementForm`.
    TUITION_REIMBURSEMENT_FORM = "Server.Items.TuitionReimbursementForm" : ITEM_BASE, v0;
    construct: Some(|w| tuition_reimbursement_form_new(w, &TUITION_REIMBURSEMENT_FORM)),
    label_number: |_, _| 1074610,
    nontransferable: |_, _| true,
    add_name_properties: |w, id, list| { (ITEM_BASE.add_name_properties)(w, id, list); add_quest_item_property(w, id, list); },
}

pub fn unicorn_ribs_new(w: &mut World, ty: &'static ItemType, amount: i32) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0x9F1);
    ops::set_loot_type(w, id, LootType::Blessed);
    hue(w, id, 0x14B);
    ops::set_stackable(w, id, true);
    ops::set_amount(w, id, amount);
    id
}
leaf! {
    /// `Server.Items.UnicornRibs`.
    UNICORN_RIBS = "Server.Items.UnicornRibs" : ITEM_BASE, v0;
    construct: Some(|w| unicorn_ribs_new(w, &UNICORN_RIBS, 1)),
    label_number: |_, _| 1074611,
}

/// `[Flippable(a, b)]` of the classes (the flip command reads it).
pub static FLIPPABLE: &[(&ItemType, i32, i32)] = &[
    (&SEVERED_ELF_EARS, 0x312D, 0x312E),
    (&SEVERED_HUMAN_EARS, 0x312F, 0x3130),
];

/// Registers the generated classes.
pub fn register(types: &mut TypeRegistry) {
    types.register_item(&A_BAUBLE, &[]);
    types.register_item(&ALCHEMISTS_BANDAGE, &[]);
    types.register_item(&ANDRIC_SATCHEL, &[]);
    types.register_item(&ASANDOS_SATCHEL, &[]);
    types.register_item(&BASIN_OF_CRYSTAL_CLEAR_WATER, &[]);
    types.register_item(&BRIDES_LETTER, &[]);
    types.register_item(&COMPLETED_TUITION_REIMBURSEMENT_FORM, &[]);
    types.register_item(&DREAD_SPIDER_SILK, &[]);
    types.register_item(&FRAGMENT_OF_A_MAP, &[]);
    types.register_item(&FRAGMENT_OF_A_MAP_DELIVERY, &[]);
    types.register_item(&FRIEND_OF_THE_LIBRARY_TOKEN, &[]);
    types.register_item(&FRIENDS_OF_THE_LIBRARY_APPLICATION, &[]);
    types.register_item(&GAMAN_HORNS, &[]);
    types.register_item(&GERVIS_SATCHEL, &[]);
    types.register_item(&HARGROVE_SATCHEL, &[]);
    types.register_item(&KIRIN_BRAINS, &[]);
    types.register_item(&LOWEL_SATCHEL, &[]);
    types.register_item(&MIRROR_OF_PURIFICATION, &[]);
    types.register_item(&MUGG_SATCHEL, &[]);
    types.register_item(&NOTARIZED_APPLICATION, &[]);
    types.register_item(&OFFICIAL_SEALING_WAX, &[]);
    types.register_item(&PORTRAIT_OF_THE_BRIDE, &[]);
    types.register_item(&REGINAS_LETTER, &[]);
    types.register_item(&SADRAH_SATCHEL, &[]);
    types.register_item(&SAP_OF_SOSARIA, &[]);
    types.register_item(&SEALED_NOTES_FOR_JAMAL, &[]);
    types.register_item(&SEALING_WAX_ORDER_ADDRESSED_TO_PETRUS, &[]);
    types.register_item(&SEVERED_ELF_EARS, &[]);
    types.register_item(&SEVERED_HUMAN_EARS, &[]);
    types.register_item(&SIGNED_TUITION_REIMBURSEMENT_FORM, &[]);
    types.register_item(&SPECIAL_TREAT_FOR_DRITHEN, &[]);
    types.register_item(&SPIRIT_BOTTLE, &[]);
    types.register_item(&STOUT_WHIP, &[]);
    types.register_item(&TAINTED_TREE_SAMPLE, &[]);
    types.register_item(&TUITION_REIMBURSEMENT_FORM, &[]);
    types.register_item(&UNICORN_RIBS, &[]);
}
