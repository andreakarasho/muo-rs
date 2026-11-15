//! The simple ML quest items: classes made only of constructor statements and plain overrides.

use super::item_support::*;
use muo_content::items::cont::BACKPACK;
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::{ITEM_BASE, ItemType, ops};
use muo_server::type_registry::TypeRegistry;

leaf! {
    /// `Server.Items.ABauble`.
    A_BAUBLE = "Server.Items.ABauble" : ITEM_BASE, v0;
    @template template_new;
}

leaf! {
    /// `Server.Items.AlchemistsBandage`.
    ALCHEMISTS_BANDAGE = "Server.Items.AlchemistsBandage" : ITEM_BASE, v0;
    @template template_new;
    add_name_properties: |w, id, list| { (ITEM_BASE.add_name_properties)(w, id, list); add_quest_item_property(w, id, list); },
}

leaf! {
    /// `Server.Items.AndricSatchel`.
    ANDRIC_SATCHEL = "Server.Items.AndricSatchel" : BACKPACK, v0;
    @template template_container;
}

leaf! {
    /// `Server.Items.AsandosSatchel`.
    ASANDOS_SATCHEL = "Server.Items.AsandosSatchel" : BACKPACK, v0;
    @template template_container;
}

leaf! {
    /// `Server.Items.BasinOfCrystalClearWater`.
    BASIN_OF_CRYSTAL_CLEAR_WATER = "Server.Items.BasinOfCrystalClearWater" : ITEM_BASE, v0;
    @template template_new;
    add_name_properties: |w, id, list| { (ITEM_BASE.add_name_properties)(w, id, list); add_quest_item_property(w, id, list); },
}

leaf! {
    /// `Server.Items.BridesLetter`.
    BRIDES_LETTER = "Server.Items.BridesLetter" : ITEM_BASE, v0;
    @template template_new;
    add_name_properties: |w, id, list| { (ITEM_BASE.add_name_properties)(w, id, list); add_quest_item_property(w, id, list); },
}

leaf! {
    /// `Server.Items.CompletedTuitionReimbursementForm`.
    COMPLETED_TUITION_REIMBURSEMENT_FORM = "Server.Items.CompletedTuitionReimbursementForm" : ITEM_BASE, v0;
    @template template_new;
    add_name_properties: |w, id, list| { (ITEM_BASE.add_name_properties)(w, id, list); add_quest_item_property(w, id, list); },
}

leaf! {
    /// `Server.Items.DreadSpiderSilk`.
    DREAD_SPIDER_SILK = "Server.Items.DreadSpiderSilk" : ITEM_BASE, v0;
    @template template_new;
    add_name_properties: |w, id, list| { (ITEM_BASE.add_name_properties)(w, id, list); add_quest_item_property(w, id, list); },
}

leaf! {
    /// `Server.Items.FragmentOfAMap`.
    FRAGMENT_OF_A_MAP = "Server.Items.FragmentOfAMap" : ITEM_BASE, v0;
    @template template_new;
}

leaf! {
    /// `Server.Items.FragmentOfAMapDelivery`.
    FRAGMENT_OF_A_MAP_DELIVERY = "Server.Items.FragmentOfAMapDelivery" : ITEM_BASE, v0;
    @template template_new;
    add_name_properties: |w, id, list| { (ITEM_BASE.add_name_properties)(w, id, list); add_quest_item_property(w, id, list); },
}

leaf! {
    /// `Server.Items.FriendOfTheLibraryToken`.
    FRIEND_OF_THE_LIBRARY_TOKEN = "Server.Items.FriendOfTheLibraryToken" : ITEM_BASE, v0;
    @template template_new;
}

leaf! {
    /// `Server.Items.FriendsOfTheLibraryApplication`.
    FRIENDS_OF_THE_LIBRARY_APPLICATION = "Server.Items.FriendsOfTheLibraryApplication" : ITEM_BASE, v0;
    @template template_new;
    add_name_properties: |w, id, list| { (ITEM_BASE.add_name_properties)(w, id, list); add_quest_item_property(w, id, list); },
}

pub fn gaman_horns_new(w: &mut World, ty: &'static ItemType, amount: i32) -> ItemId {
    let id = template_new(w, ty);
    ops::set_amount(w, id, amount);
    id
}
leaf! {
    /// `Server.Items.GamanHorns`.
    GAMAN_HORNS = "Server.Items.GamanHorns" : ITEM_BASE, v0;
    @template amount gaman_horns_new;
}

leaf! {
    /// `Server.Items.GervisSatchel`.
    GERVIS_SATCHEL = "Server.Items.GervisSatchel" : BACKPACK, v0;
    @template template_container;
}

leaf! {
    /// `Server.Items.HargroveSatchel`.
    HARGROVE_SATCHEL = "Server.Items.HargroveSatchel" : BACKPACK, v0;
    @template template_container;
}

leaf! {
    /// `Server.Items.KirinBrains`.
    KIRIN_BRAINS = "Server.Items.KirinBrains" : ITEM_BASE, v0;
    @template template_new;
}

leaf! {
    /// `Server.Items.LowelSatchel`.
    LOWEL_SATCHEL = "Server.Items.LowelSatchel" : BACKPACK, v0;
    @template template_container;
}

leaf! {
    /// `Server.Items.MirrorOfPurification`.
    MIRROR_OF_PURIFICATION = "Server.Items.MirrorOfPurification" : ITEM_BASE, v0;
    @template template_new;
}

leaf! {
    /// `Server.Items.MuggSatchel`.
    MUGG_SATCHEL = "Server.Items.MuggSatchel" : BACKPACK, v0;
    @template template_container;
}

leaf! {
    /// `Server.Items.NotarizedApplication`.
    NOTARIZED_APPLICATION = "Server.Items.NotarizedApplication" : ITEM_BASE, v0;
    @template template_new;
    add_name_properties: |w, id, list| { (ITEM_BASE.add_name_properties)(w, id, list); add_quest_item_property(w, id, list); },
}

leaf! {
    /// `Server.Items.OfficialSealingWax`.
    OFFICIAL_SEALING_WAX = "Server.Items.OfficialSealingWax" : ITEM_BASE, v0;
    @template template_new;
    add_name_properties: |w, id, list| { (ITEM_BASE.add_name_properties)(w, id, list); add_quest_item_property(w, id, list); },
}

leaf! {
    /// `Server.Items.PortraitOfTheBride`.
    PORTRAIT_OF_THE_BRIDE = "Server.Items.PortraitOfTheBride" : ITEM_BASE, v0;
    @template template_new;
    add_name_properties: |w, id, list| { (ITEM_BASE.add_name_properties)(w, id, list); add_quest_item_property(w, id, list); },
}

leaf! {
    /// `Server.Items.ReginasLetter`.
    REGINAS_LETTER = "Server.Items.ReginasLetter" : ITEM_BASE, v0;
    @template template_new;
    add_name_properties: |w, id, list| { (ITEM_BASE.add_name_properties)(w, id, list); add_quest_item_property(w, id, list); },
}

leaf! {
    /// `Server.Items.SadrahSatchel`.
    SADRAH_SATCHEL = "Server.Items.SadrahSatchel" : BACKPACK, v0;
    @template template_container;
}

pub fn sap_of_sosaria_new(w: &mut World, ty: &'static ItemType, amount: i32) -> ItemId {
    let id = template_new(w, ty);
    ops::set_amount(w, id, amount);
    id
}
leaf! {
    /// `Server.Items.SapOfSosaria`.
    SAP_OF_SOSARIA = "Server.Items.SapOfSosaria" : ITEM_BASE, v0;
    @template amount sap_of_sosaria_new;
}

leaf! {
    /// `Server.Items.SealedNotesForJamal`.
    SEALED_NOTES_FOR_JAMAL = "Server.Items.SealedNotesForJamal" : ITEM_BASE, v0;
    @template template_new;
    default_weight: |_, _| 1.0,
    add_name_properties: |w, id, list| { (ITEM_BASE.add_name_properties)(w, id, list); add_quest_item_property(w, id, list); },
}

leaf! {
    /// `Server.Items.SealingWaxOrderAddressedToPetrus`.
    SEALING_WAX_ORDER_ADDRESSED_TO_PETRUS = "Server.Items.SealingWaxOrderAddressedToPetrus" : ITEM_BASE, v0;
    @template template_new;
    add_name_properties: |w, id, list| { (ITEM_BASE.add_name_properties)(w, id, list); add_quest_item_property(w, id, list); },
}

pub fn severed_elf_ears_new(w: &mut World, ty: &'static ItemType, amount: i32) -> ItemId {
    let id = template_new(w, ty);
    ops::set_amount(w, id, amount);
    id
}
leaf! {
    /// `Server.Items.SeveredElfEars`.
    SEVERED_ELF_EARS = "Server.Items.SeveredElfEars" : ITEM_BASE, v0;
    @template amount severed_elf_ears_new;
}

pub fn severed_human_ears_new(w: &mut World, ty: &'static ItemType, amount: i32) -> ItemId {
    let id = template_new(w, ty);
    ops::set_amount(w, id, amount);
    id
}
leaf! {
    /// `Server.Items.SeveredHumanEars`.
    SEVERED_HUMAN_EARS = "Server.Items.SeveredHumanEars" : ITEM_BASE, v0;
    @template amount severed_human_ears_new;
}

leaf! {
    /// `Server.Items.SignedTuitionReimbursementForm`.
    SIGNED_TUITION_REIMBURSEMENT_FORM = "Server.Items.SignedTuitionReimbursementForm" : ITEM_BASE, v0;
    @template template_new;
    add_name_properties: |w, id, list| { (ITEM_BASE.add_name_properties)(w, id, list); add_quest_item_property(w, id, list); },
}

leaf! {
    /// `Server.Items.SpecialTreatForDrithen`.
    SPECIAL_TREAT_FOR_DRITHEN = "Server.Items.SpecialTreatForDrithen" : ITEM_BASE, v0;
    @template template_new;
}

leaf! {
    /// `Server.Items.SpiritBottle`.
    SPIRIT_BOTTLE = "Server.Items.SpiritBottle" : ITEM_BASE, v0;
    @template template_new;
    add_name_properties: |w, id, list| { (ITEM_BASE.add_name_properties)(w, id, list); add_quest_item_property(w, id, list); },
}

leaf! {
    /// `Server.Items.StoutWhip`.
    STOUT_WHIP = "Server.Items.StoutWhip" : ITEM_BASE, v0;
    @template template_new;
}

leaf! {
    /// `Server.Items.TaintedTreeSample`.
    TAINTED_TREE_SAMPLE = "Server.Items.TaintedTreeSample" : ITEM_BASE, v0;
    @template template_new;
    add_name_properties: |w, id, list| { (ITEM_BASE.add_name_properties)(w, id, list); add_quest_item_property(w, id, list); },
}

leaf! {
    /// `Server.Items.TuitionReimbursementForm`.
    TUITION_REIMBURSEMENT_FORM = "Server.Items.TuitionReimbursementForm" : ITEM_BASE, v0;
    @template template_new;
    add_name_properties: |w, id, list| { (ITEM_BASE.add_name_properties)(w, id, list); add_quest_item_property(w, id, list); },
}

pub fn unicorn_ribs_new(w: &mut World, ty: &'static ItemType, amount: i32) -> ItemId {
    let id = template_new(w, ty);
    ops::set_amount(w, id, amount);
    id
}
leaf! {
    /// `Server.Items.UnicornRibs`.
    UNICORN_RIBS = "Server.Items.UnicornRibs" : ITEM_BASE, v0;
    @template amount unicorn_ribs_new;
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
