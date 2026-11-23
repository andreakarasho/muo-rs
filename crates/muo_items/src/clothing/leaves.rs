//! The concrete clothing classes that are tables. Classes with logic of their own are in `special.rs`.

use muo_server::item::ItemType;
use muo_server::type_registry::TypeRegistry;

use super::{dye_false, dye_refused, scissor_refused};
use muo_content::items::clothing::{
    BASE_CLOAK, BASE_CLOAK_CLASS, BASE_HAT, BASE_HAT_CLASS, BASE_MIDDLE_TORSO,
    BASE_MIDDLE_TORSO_CLASS, BASE_OUTER_LEGS, BASE_OUTER_LEGS_CLASS, BASE_OUTER_TORSO,
    BASE_OUTER_TORSO_CLASS, BASE_PANTS, BASE_PANTS_CLASS, BASE_SHIRT, BASE_SHIRT_CLASS, BASE_SHOES,
    BASE_SHOES_CLASS, BASE_WAIST, BASE_WAIST_CLASS,
};

clothing_leaf! {
    /// `Server.Items.CrimsonCincture`.
    CRIMSON_CINCTURE, CRIMSON_CINCTURE_CLASS, crimson_cincture_new = "Server.Items.CrimsonCincture" : HALF_APRON / HALF_APRON_CLASS, half_apron_new, hue 0, encoded false;
    @template;
}

clothing_leaf! {
    /// `Server.Items.DivineCountenance`.
    DIVINE_COUNTENANCE, DIVINE_COUNTENANCE_CLASS, divine_countenance_new = "Server.Items.DivineCountenance" : HORNED_TRIBAL_MASK / HORNED_TRIBAL_MASK_CLASS, horned_tribal_mask_new, hue 0, encoded false;
    @template;
}

clothing_leaf! {
    /// `Server.Items.HatOfTheMagi`.
    HAT_OF_THE_MAGI, HAT_OF_THE_MAGI_CLASS, hat_of_the_magi_new = "Server.Items.HatOfTheMagi" : WIZARDS_HAT / WIZARDS_HAT_CLASS, wizards_hat_new, hue 0, encoded false;
    @template;
}

clothing_leaf! {
    /// `Server.Items.HuntersHeaddress`.
    HUNTERS_HEADDRESS, HUNTERS_HEADDRESS_CLASS, hunters_headdress_new = "Server.Items.HuntersHeaddress" : DEER_MASK / DEER_MASK_CLASS, deer_mask_new, hue 0, encoded false;
    @template;
}

clothing_leaf! {
    /// `Server.Items.SpiritOfTheTotem`.
    SPIRIT_OF_THE_TOTEM, SPIRIT_OF_THE_TOTEM_CLASS, spirit_of_the_totem_new = "Server.Items.SpiritOfTheTotem" : BEAR_MASK / BEAR_MASK_CLASS, bear_mask_new, hue 0, encoded false;
    @template;
}

clothing_leaf! {
    /// `Server.Items.FurCape`.
    FUR_CAPE, FUR_CAPE_CLASS, fur_cape_new = "Server.Items.FurCape" : BASE_CLOAK / BASE_CLOAK_CLASS, crate::clothing::new_clothing_json, hue 0, encoded false;
}

clothing_leaf! {
    /// `Server.Items.Kasa`.
    KASA, KASA_CLASS, kasa_new = "Server.Items.Kasa" : BASE_HAT / BASE_HAT_CLASS, crate::clothing::new_clothing_json, hue 0, encoded false;
}

clothing_leaf! {
    /// `Server.Items.ClothNinjaHood`.
    CLOTH_NINJA_HOOD, CLOTH_NINJA_HOOD_CLASS, cloth_ninja_hood_new = "Server.Items.ClothNinjaHood" : BASE_HAT / BASE_HAT_CLASS, crate::clothing::new_clothing_json, hue 0, encoded false;
}

clothing_leaf! {
    /// `Server.Items.FlowerGarland`.
    FLOWER_GARLAND, FLOWER_GARLAND_CLASS, flower_garland_new = "Server.Items.FlowerGarland" : BASE_HAT / BASE_HAT_CLASS, crate::clothing::new_clothing_json, hue 0, encoded false;
}

clothing_leaf! {
    /// `Server.Items.FloppyHat`.
    FLOPPY_HAT, FLOPPY_HAT_CLASS, floppy_hat_new = "Server.Items.FloppyHat" : BASE_HAT / BASE_HAT_CLASS, crate::clothing::new_clothing_json, hue 0, encoded false;
}

clothing_leaf! {
    /// `Server.Items.WideBrimHat`.
    WIDE_BRIM_HAT, WIDE_BRIM_HAT_CLASS, wide_brim_hat_new = "Server.Items.WideBrimHat" : BASE_HAT / BASE_HAT_CLASS, crate::clothing::new_clothing_json, hue 0, encoded false;
}

clothing_leaf! {
    /// `Server.Items.SkullCap`.
    SKULL_CAP, SKULL_CAP_CLASS, skull_cap_new = "Server.Items.SkullCap" : BASE_HAT / BASE_HAT_CLASS, crate::clothing::new_clothing_json, hue 0, encoded false;
}

clothing_leaf! {
    /// `Server.Items.Bandana`.
    BANDANA, BANDANA_CLASS, bandana_new = "Server.Items.Bandana" : BASE_HAT / BASE_HAT_CLASS, crate::clothing::new_clothing_json, hue 0, encoded false;
}

clothing_leaf! {
    /// `Server.Items.BearMask`.
    BEAR_MASK, BEAR_MASK_CLASS, bear_mask_new = "Server.Items.BearMask" : BASE_HAT / BASE_HAT_CLASS, crate::clothing::new_clothing_json, hue 0, encoded false;
    dye: dye_refused,
}

clothing_leaf! {
    /// `Server.Items.DeerMask`.
    DEER_MASK, DEER_MASK_CLASS, deer_mask_new = "Server.Items.DeerMask" : BASE_HAT / BASE_HAT_CLASS, crate::clothing::new_clothing_json, hue 0, encoded false;
    dye: dye_refused,
}

clothing_leaf! {
    /// `Server.Items.HornedTribalMask`.
    HORNED_TRIBAL_MASK, HORNED_TRIBAL_MASK_CLASS, horned_tribal_mask_new = "Server.Items.HornedTribalMask" : BASE_HAT / BASE_HAT_CLASS, crate::clothing::new_clothing_json, hue 0, encoded false;
    dye: dye_refused,
}

clothing_leaf! {
    /// `Server.Items.TribalMask`.
    TRIBAL_MASK, TRIBAL_MASK_CLASS, tribal_mask_new = "Server.Items.TribalMask" : BASE_HAT / BASE_HAT_CLASS, crate::clothing::new_clothing_json, hue 0, encoded false;
    dye: dye_refused,
}

clothing_leaf! {
    /// `Server.Items.TallStrawHat`.
    TALL_STRAW_HAT, TALL_STRAW_HAT_CLASS, tall_straw_hat_new = "Server.Items.TallStrawHat" : BASE_HAT / BASE_HAT_CLASS, crate::clothing::new_clothing_json, hue 0, encoded false;
}

clothing_leaf! {
    /// `Server.Items.StrawHat`.
    STRAW_HAT, STRAW_HAT_CLASS, straw_hat_new = "Server.Items.StrawHat" : BASE_HAT / BASE_HAT_CLASS, crate::clothing::new_clothing_json, hue 0, encoded false;
}

clothing_leaf! {
    /// `Server.Items.WizardsHat`.
    WIZARDS_HAT, WIZARDS_HAT_CLASS, wizards_hat_new = "Server.Items.WizardsHat" : BASE_HAT / BASE_HAT_CLASS, crate::clothing::new_clothing_json, hue 0, encoded false;
}

clothing_leaf! {
    /// `Server.Items.MagicWizardsHat`.
    MAGIC_WIZARDS_HAT, MAGIC_WIZARDS_HAT_CLASS, magic_wizards_hat_new = "Server.Items.MagicWizardsHat" : BASE_HAT / BASE_HAT_CLASS, crate::clothing::new_clothing_json, hue 0, encoded false;
}

clothing_leaf! {
    /// `Server.Items.Bonnet`.
    BONNET, BONNET_CLASS, bonnet_new = "Server.Items.Bonnet" : BASE_HAT / BASE_HAT_CLASS, crate::clothing::new_clothing_json, hue 0, encoded false;
}

clothing_leaf! {
    /// `Server.Items.FeatheredHat`.
    FEATHERED_HAT, FEATHERED_HAT_CLASS, feathered_hat_new = "Server.Items.FeatheredHat" : BASE_HAT / BASE_HAT_CLASS, crate::clothing::new_clothing_json, hue 0, encoded false;
}

clothing_leaf! {
    /// `Server.Items.TricorneHat`.
    TRICORNE_HAT, TRICORNE_HAT_CLASS, tricorne_hat_new = "Server.Items.TricorneHat" : BASE_HAT / BASE_HAT_CLASS, crate::clothing::new_clothing_json, hue 0, encoded false;
}

clothing_leaf! {
    /// `Server.Items.JesterHat`.
    JESTER_HAT, JESTER_HAT_CLASS, jester_hat_new = "Server.Items.JesterHat" : BASE_HAT / BASE_HAT_CLASS, crate::clothing::new_clothing_json, hue 0, encoded false;
}

clothing_leaf! {
    /// `Server.Items.BodySash`.
    BODY_SASH, BODY_SASH_CLASS, body_sash_new = "Server.Items.BodySash" : BASE_MIDDLE_TORSO / BASE_MIDDLE_TORSO_CLASS, crate::clothing::new_clothing_json, hue 0, encoded false;
}

clothing_leaf! {
    /// `Server.Items.FullApron`.
    FULL_APRON, FULL_APRON_CLASS, full_apron_new = "Server.Items.FullApron" : BASE_MIDDLE_TORSO / BASE_MIDDLE_TORSO_CLASS, crate::clothing::new_clothing_json, hue 0, encoded false;
}

clothing_leaf! {
    /// `Server.Items.Doublet`.
    DOUBLET, DOUBLET_CLASS, doublet_new = "Server.Items.Doublet" : BASE_MIDDLE_TORSO / BASE_MIDDLE_TORSO_CLASS, crate::clothing::new_clothing_json, hue 0, encoded false;
}

clothing_leaf! {
    /// `Server.Items.Surcoat`.
    SURCOAT, SURCOAT_CLASS, surcoat_new = "Server.Items.Surcoat" : BASE_MIDDLE_TORSO / BASE_MIDDLE_TORSO_CLASS, crate::clothing::new_clothing_json, hue 0, encoded false;
}

clothing_leaf! {
    /// `Server.Items.Tunic`.
    TUNIC, TUNIC_CLASS, tunic_new = "Server.Items.Tunic" : BASE_MIDDLE_TORSO / BASE_MIDDLE_TORSO_CLASS, crate::clothing::new_clothing_json, hue 0, encoded false;
}

clothing_leaf! {
    /// `Server.Items.FormalShirt`.
    FORMAL_SHIRT, FORMAL_SHIRT_CLASS, formal_shirt_new = "Server.Items.FormalShirt" : BASE_MIDDLE_TORSO / BASE_MIDDLE_TORSO_CLASS, crate::clothing::new_clothing_json, hue 0, encoded false;
}

clothing_leaf! {
    /// `Server.Items.JesterSuit`.
    JESTER_SUIT, JESTER_SUIT_CLASS, jester_suit_new = "Server.Items.JesterSuit" : BASE_MIDDLE_TORSO / BASE_MIDDLE_TORSO_CLASS, crate::clothing::new_clothing_json, hue 0, encoded false;
}

clothing_leaf! {
    /// `Server.Items.JinBaori`.
    JIN_BAORI, JIN_BAORI_CLASS, jin_baori_new = "Server.Items.JinBaori" : BASE_MIDDLE_TORSO / BASE_MIDDLE_TORSO_CLASS, crate::clothing::new_clothing_json, hue 0, encoded false;
}

clothing_leaf! {
    /// `Server.Items.FurSarong`.
    FUR_SARONG, FUR_SARONG_CLASS, fur_sarong_new = "Server.Items.FurSarong" : BASE_OUTER_LEGS / BASE_OUTER_LEGS_CLASS, crate::clothing::new_clothing_json, hue 0, encoded false;
}

clothing_leaf! {
    /// `Server.Items.Skirt`.
    SKIRT, SKIRT_CLASS, skirt_new = "Server.Items.Skirt" : BASE_OUTER_LEGS / BASE_OUTER_LEGS_CLASS, crate::clothing::new_clothing_json, hue 0, encoded false;
}

clothing_leaf! {
    /// `Server.Items.Kilt`.
    KILT, KILT_CLASS, kilt_new = "Server.Items.Kilt" : BASE_OUTER_LEGS / BASE_OUTER_LEGS_CLASS, crate::clothing::new_clothing_json, hue 0, encoded false;
}

clothing_leaf! {
    /// `Server.Items.Hakama`.
    HAKAMA, HAKAMA_CLASS, hakama_new = "Server.Items.Hakama" : BASE_OUTER_LEGS / BASE_OUTER_LEGS_CLASS, crate::clothing::new_clothing_json, hue 0, encoded false;
}

clothing_leaf! {
    /// `Server.Items.GildedDress`.
    GILDED_DRESS, GILDED_DRESS_CLASS, gilded_dress_new = "Server.Items.GildedDress" : BASE_OUTER_TORSO / BASE_OUTER_TORSO_CLASS, crate::clothing::new_clothing_json, hue 0, encoded false;
}

clothing_leaf! {
    /// `Server.Items.FancyDress`.
    FANCY_DRESS, FANCY_DRESS_CLASS, fancy_dress_new = "Server.Items.FancyDress" : BASE_OUTER_TORSO / BASE_OUTER_TORSO_CLASS, crate::clothing::new_clothing_json, hue 0, encoded false;
}

clothing_leaf! {
    /// `Server.Items.MonkRobe`.
    MONK_ROBE, MONK_ROBE_CLASS, monk_robe_new = "Server.Items.MonkRobe" : BASE_OUTER_TORSO / BASE_OUTER_TORSO_CLASS, crate::clothing::new_clothing_json, hue 0x21E, encoded false;
    @template;
    dye: dye_refused,
}

clothing_leaf! {
    /// `Server.Items.PlainDress`.
    PLAIN_DRESS, PLAIN_DRESS_CLASS, plain_dress_new = "Server.Items.PlainDress" : BASE_OUTER_TORSO / BASE_OUTER_TORSO_CLASS, crate::clothing::new_clothing_json, hue 0, encoded false;
}

clothing_leaf! {
    /// `Server.Items.Kamishimo`.
    KAMISHIMO, KAMISHIMO_CLASS, kamishimo_new = "Server.Items.Kamishimo" : BASE_OUTER_TORSO / BASE_OUTER_TORSO_CLASS, crate::clothing::new_clothing_json, hue 0, encoded false;
}

clothing_leaf! {
    /// `Server.Items.HakamaShita`.
    HAKAMA_SHITA, HAKAMA_SHITA_CLASS, hakama_shita_new = "Server.Items.HakamaShita" : BASE_OUTER_TORSO / BASE_OUTER_TORSO_CLASS, crate::clothing::new_clothing_json, hue 0, encoded false;
}

clothing_leaf! {
    /// `Server.Items.MaleKimono`.
    MALE_KIMONO, MALE_KIMONO_CLASS, male_kimono_new = "Server.Items.MaleKimono" : BASE_OUTER_TORSO / BASE_OUTER_TORSO_CLASS, crate::clothing::new_clothing_json, hue 0, encoded false;
}

clothing_leaf! {
    /// `Server.Items.FemaleKimono`.
    FEMALE_KIMONO, FEMALE_KIMONO_CLASS, female_kimono_new = "Server.Items.FemaleKimono" : BASE_OUTER_TORSO / BASE_OUTER_TORSO_CLASS, crate::clothing::new_clothing_json, hue 0, encoded false;
}

clothing_leaf! {
    /// `Server.Items.MaleElvenRobe`.
    MALE_ELVEN_ROBE, MALE_ELVEN_ROBE_CLASS, male_elven_robe_new = "Server.Items.MaleElvenRobe" : BASE_OUTER_TORSO / BASE_OUTER_TORSO_CLASS, crate::clothing::new_clothing_json, hue 0, encoded true;
}

clothing_leaf! {
    /// `Server.Items.FemaleElvenRobe`.
    FEMALE_ELVEN_ROBE, FEMALE_ELVEN_ROBE_CLASS, female_elven_robe_new = "Server.Items.FemaleElvenRobe" : BASE_OUTER_TORSO / BASE_OUTER_TORSO_CLASS, crate::clothing::new_clothing_json, hue 0, encoded true;
}

clothing_leaf! {
    /// `Server.Items.ShortPants`.
    SHORT_PANTS, SHORT_PANTS_CLASS, short_pants_new = "Server.Items.ShortPants" : BASE_PANTS / BASE_PANTS_CLASS, crate::clothing::new_clothing_json, hue 0, encoded false;
}

clothing_leaf! {
    /// `Server.Items.LongPants`.
    LONG_PANTS, LONG_PANTS_CLASS, long_pants_new = "Server.Items.LongPants" : BASE_PANTS / BASE_PANTS_CLASS, crate::clothing::new_clothing_json, hue 0, encoded false;
}

clothing_leaf! {
    /// `Server.Items.TattsukeHakama`.
    TATTSUKE_HAKAMA, TATTSUKE_HAKAMA_CLASS, tattsuke_hakama_new = "Server.Items.TattsukeHakama" : BASE_PANTS / BASE_PANTS_CLASS, crate::clothing::new_clothing_json, hue 0, encoded false;
}

clothing_leaf! {
    /// `Server.Items.ElvenPants`.
    ELVEN_PANTS, ELVEN_PANTS_CLASS, elven_pants_new = "Server.Items.ElvenPants" : BASE_PANTS / BASE_PANTS_CLASS, crate::clothing::new_clothing_json, hue 0, encoded true;
}

clothing_leaf! {
    /// `Server.Items.Shirt`.
    SHIRT, SHIRT_CLASS, shirt_new = "Server.Items.Shirt" : BASE_SHIRT / BASE_SHIRT_CLASS, crate::clothing::new_clothing_json, hue 0, encoded false;
}

clothing_leaf! {
    /// `Server.Items.ClothNinjaJacket`.
    CLOTH_NINJA_JACKET, CLOTH_NINJA_JACKET_CLASS, cloth_ninja_jacket_new = "Server.Items.ClothNinjaJacket" : BASE_SHIRT / BASE_SHIRT_CLASS, crate::clothing::new_clothing_json, hue 0, encoded false;
    @template;
}

clothing_leaf! {
    /// `Server.Items.ElvenShirt`.
    ELVEN_SHIRT, ELVEN_SHIRT_CLASS, elven_shirt_new = "Server.Items.ElvenShirt" : BASE_SHIRT / BASE_SHIRT_CLASS, crate::clothing::new_clothing_json, hue 0, encoded true;
}

clothing_leaf! {
    /// `Server.Items.ElvenDarkShirt`.
    ELVEN_DARK_SHIRT, ELVEN_DARK_SHIRT_CLASS, elven_dark_shirt_new = "Server.Items.ElvenDarkShirt" : BASE_SHIRT / BASE_SHIRT_CLASS, crate::clothing::new_clothing_json, hue 0, encoded true;
}

clothing_leaf! {
    /// `Server.Items.FurBoots`.
    FUR_BOOTS, FUR_BOOTS_CLASS, fur_boots_new = "Server.Items.FurBoots" : BASE_SHOES / BASE_SHOES_CLASS, crate::clothing::new_clothing_json, hue 0, encoded false;
}

clothing_leaf! {
    /// `Server.Items.Boots`.
    BOOTS, BOOTS_CLASS, boots_new = "Server.Items.Boots" : BASE_SHOES / BASE_SHOES_CLASS, crate::clothing::new_clothing_json, hue 0, encoded false;
}

clothing_leaf! {
    /// `Server.Items.Shoes`.
    SHOES, SHOES_CLASS, shoes_new = "Server.Items.Shoes" : BASE_SHOES / BASE_SHOES_CLASS, crate::clothing::new_clothing_json, hue 0, encoded false;
}

clothing_leaf! {
    /// `Server.Items.Sandals`.
    SANDALS, SANDALS_CLASS, sandals_new = "Server.Items.Sandals" : BASE_SHOES / BASE_SHOES_CLASS, crate::clothing::new_clothing_json, hue 0, encoded false;
    dye: dye_false,
}

clothing_leaf! {
    /// `Server.Items.NinjaTabi`.
    NINJA_TABI, NINJA_TABI_CLASS, ninja_tabi_new = "Server.Items.NinjaTabi" : BASE_SHOES / BASE_SHOES_CLASS, crate::clothing::new_clothing_json, hue 0, encoded false;
}

clothing_leaf! {
    /// `Server.Items.SamuraiTabi`.
    SAMURAI_TABI, SAMURAI_TABI_CLASS, samurai_tabi_new = "Server.Items.SamuraiTabi" : BASE_SHOES / BASE_SHOES_CLASS, crate::clothing::new_clothing_json, hue 0, encoded false;
}

clothing_leaf! {
    /// `Server.Items.Waraji`.
    WARAJI, WARAJI_CLASS, waraji_new = "Server.Items.Waraji" : BASE_SHOES / BASE_SHOES_CLASS, crate::clothing::new_clothing_json, hue 0, encoded false;
}

clothing_leaf! {
    /// `Server.Items.ElvenBoots`.
    ELVEN_BOOTS, ELVEN_BOOTS_CLASS, elven_boots_new = "Server.Items.ElvenBoots" : BASE_SHOES / BASE_SHOES_CLASS, crate::clothing::new_clothing_json, hue 0, encoded true;
    dye: dye_false,
}

clothing_leaf! {
    /// `Server.Items.HalfApron`.
    HALF_APRON, HALF_APRON_CLASS, half_apron_new = "Server.Items.HalfApron" : BASE_WAIST / BASE_WAIST_CLASS, crate::clothing::new_clothing_json, hue 0, encoded false;
}

clothing_leaf! {
    /// `Server.Items.Obi`.
    OBI, OBI_CLASS, obi_new = "Server.Items.Obi" : BASE_WAIST / BASE_WAIST_CLASS, crate::clothing::new_clothing_json, hue 0, encoded false;
}

clothing_leaf! {
    /// `Server.Items.WoodlandBelt`.
    WOODLAND_BELT, WOODLAND_BELT_CLASS, woodland_belt_new = "Server.Items.WoodlandBelt" : BASE_WAIST / BASE_WAIST_CLASS, crate::clothing::new_clothing_json, hue 0, encoded true;
    dye: dye_refused,
    scissor: scissor_refused,
}

/// Every class of this file, for `register` and the tests.
pub static ALL: &[(&ItemType, &[&str])] = &[
    (&CRIMSON_CINCTURE, &[]),
    (&DIVINE_COUNTENANCE, &[]),
    (&HAT_OF_THE_MAGI, &[]),
    (&HUNTERS_HEADDRESS, &[]),
    (&SPIRIT_OF_THE_TOTEM, &[]),
    (&FUR_CAPE, &[]),
    (&KASA, &[]),
    (&CLOTH_NINJA_HOOD, &[]),
    (&FLOWER_GARLAND, &[]),
    (&FLOPPY_HAT, &[]),
    (&WIDE_BRIM_HAT, &[]),
    (&SKULL_CAP, &[]),
    (&BANDANA, &[]),
    (&BEAR_MASK, &[]),
    (&DEER_MASK, &[]),
    (&HORNED_TRIBAL_MASK, &[]),
    (&TRIBAL_MASK, &[]),
    (&TALL_STRAW_HAT, &[]),
    (&STRAW_HAT, &[]),
    (&WIZARDS_HAT, &[]),
    (&MAGIC_WIZARDS_HAT, &[]),
    (&BONNET, &[]),
    (&FEATHERED_HAT, &[]),
    (&TRICORNE_HAT, &[]),
    (&JESTER_HAT, &[]),
    (&BODY_SASH, &[]),
    (&FULL_APRON, &[]),
    (&DOUBLET, &[]),
    (&SURCOAT, &[]),
    (&TUNIC, &[]),
    (&FORMAL_SHIRT, &[]),
    (&JESTER_SUIT, &[]),
    (&JIN_BAORI, &[]),
    (&FUR_SARONG, &[]),
    (&SKIRT, &[]),
    (&KILT, &[]),
    (&HAKAMA, &[]),
    (&GILDED_DRESS, &[]),
    (&FANCY_DRESS, &[]),
    (&MONK_ROBE, &[]),
    (&PLAIN_DRESS, &[]),
    (&KAMISHIMO, &[]),
    (&HAKAMA_SHITA, &[]),
    (&MALE_KIMONO, &[]),
    (&FEMALE_KIMONO, &[]),
    (&MALE_ELVEN_ROBE, &[]),
    (&FEMALE_ELVEN_ROBE, &[]),
    (&SHORT_PANTS, &[]),
    (&LONG_PANTS, &[]),
    (&TATTSUKE_HAKAMA, &[]),
    (&ELVEN_PANTS, &[]),
    (&SHIRT, &[]),
    (&CLOTH_NINJA_JACKET, &[]),
    (&ELVEN_SHIRT, &[]),
    (&ELVEN_DARK_SHIRT, &[]),
    (&FUR_BOOTS, &[]),
    (&BOOTS, &[]),
    (&SHOES, &[]),
    (&SANDALS, &[]),
    (&NINJA_TABI, &[]),
    (&SAMURAI_TABI, &[]),
    (&WARAJI, &[]),
    (&ELVEN_BOOTS, &[]),
    (&HALF_APRON, &[]),
    (&OBI, &[]),
    (&WOODLAND_BELT, &[]),
];

/// Registers the classes of this file.
pub fn register(types: &mut TypeRegistry) {
    for (t, aliases) in ALL {
        types.register_item(t, aliases);
    }
}
