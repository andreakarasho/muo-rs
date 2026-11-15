//! The special classes that are tables over an existing family (weapons, armor, jewels, clothing) or small
//! hand-written classes: Gifts, AoS Promotional, 11th Year promo (EarringsOfProtection), Community Collection, ML,
//! Mutation Core (PlagueBeastGland) and RewardCake. The first group are
//! plain tables; the rest have code of their own.
//!
//! EarringBoxSet is in `boxes.rs`, HeritageToken and the statuettes in `statuette.rs`.

use crate::armor::leaves::*;
use crate::armor::{armor_leaf, hue};
use crate::clothing::clothing_leaf;
use crate::jewels::jewel_leaf;
use crate::simple::{leaf, random_from, weapon_art};
use crate::weapons::*;
use muo_content::aos::{AosAttribute, AosElementAttribute, AosWeaponAttribute, AttrKind};
use muo_content::items::armor::{
    ArmorClass, ArmorMaterialType, BASE_ARMOR, BASE_ARMOR_CLASS, new_armor,
};
use muo_content::items::clothing::{
    BASE_HAT, BASE_HAT_CLASS, BASE_OUTER_TORSO, BASE_OUTER_TORSO_CLASS, new_clothing,
};
use muo_content::items::jewel::{BASE_JEWEL, BASE_JEWEL_CLASS, JewelClass, new_jewel};
use muo_content::items::slayer::SlayerName;
use muo_content::items::{same, set_attr};

use muo_primitives::MessageType;
use muo_primitives::utility;
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::{ITEM_BASE, ItemType, LootType, deps, ops};
use muo_server::layer::Layer;
use muo_server::type_registry::TypeRegistry;

weapon_art! {
    /// `Server.Items.HolySword`.
    HOLY_SWORD, HOLY_SWORD_CLASS, holy_sword_new = "Server.Items.HolySword" : LONGSWORD / LONGSWORD_CLASS, via longsword_new, enc;
    @json;
}

armor_leaf! {
    /// `Server.Items.LeggingsOfEmbers`.
    LEGGINGS_OF_EMBERS, LEGGINGS_OF_EMBERS_CLASS, leggings_of_embers_new = "Server.Items.LeggingsOfEmbers" : PLATE_LEGS / PLATE_LEGS_CLASS, plate_legs_new, encoded true;
    @template;
}

weapon_art! {
    /// `Server.Items.ShaminoCrossbow`.
    SHAMINO_CROSSBOW, SHAMINO_CROSSBOW_CLASS, shamino_crossbow_new = "Server.Items.ShaminoCrossbow" : REPEATING_CROSSBOW / REPEATING_CROSSBOW_CLASS, via repeating_crossbow_new, enc;
    @json;
}

/// `MinotaurHedge()`.
pub fn minotaur_hedge_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let t0 = random_from(w, 3215, 4);
    ops::new_item_with_id(w, ty, t0)
}

leaf! {
    /// `Server.Items.MinotaurHedge`.
    MINOTAUR_HEDGE = "Server.Items.MinotaurHedge" : ITEM_BASE, v0;
    construct: Some(|w| minotaur_hedge_new(w, &MINOTAUR_HEDGE)),
    default_weight: |_, _| 1.0,
    default_name: |_, _| Some("minotaur hedge".to_string()),
}

/// `TormentedChains()`.
pub fn tormented_chains_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let t0 = random_from(w, 6663, 2);
    ops::new_item_with_id(w, ty, t0)
}

leaf! {
    /// `Server.Items.TormentedChains`.
    TORMENTED_CHAINS = "Server.Items.TormentedChains" : ITEM_BASE, v0;
    construct: Some(|w| tormented_chains_new(w, &TORMENTED_CHAINS)),
    default_weight: |_, _| 1.0,
}

/// `PlagueBeastGland()`.
pub fn plague_beast_gland_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.PlagueBeastGland")
}

leaf! {
    /// `Server.Items.PlagueBeastGland`.
    PLAGUE_BEAST_GLAND = "Server.Items.PlagueBeastGland" : ITEM_BASE, enc;
    construct: Some(|w| plague_beast_gland_new(w, &PLAGUE_BEAST_GLAND)),
    default_weight: |_, _| 1.0,
    label_number: |_, _| 1153759,
}

// ---- SamuraiHelm: [Flippable(0x236C, 0x236D)], BaseArmor ----
armor_leaf! {
    /// `Server.Items.SamuraiHelm`.
    SAMURAI_HELM, SAMURAI_HELM_CLASS, samurai_helm_new = "Server.Items.SamuraiHelm" : BASE_ARMOR / BASE_ARMOR_CLASS, |w, ty| new_armor(w, ty, 0x236C), encoded true;
    @template;
    flippable: Some((0x236C, 0x236D)),
    default_weight: Some(5.0),
    material_type: ArmorMaterialType::Plate,
}

// ---- LuckyNecklace: BaseJewel(0x1088, Layer.Neck), Hue => 1150 ----
jewel_leaf! {
    /// `Server.Items.LuckyNecklace`.
    LUCKY_NECKLACE, LUCKY_NECKLACE_CLASS, lucky_necklace_new = "Server.Items.LuckyNecklace" : BASE_JEWEL / BASE_JEWEL_CLASS, |w, ty| new_jewel(w, ty, 0x1088), encoded false;
    @template;
    @ty { hue: |_, _| 1150, };
    layer: Layer::Neck,
}

// ---- JesterHatofChuckles: BaseHat(0x171C, hue) ----
clothing_leaf! {
    /// `Server.Items.JesterHatofChuckles`.
    JESTER_HAT_OF_CHUCKLES, JESTER_HAT_OF_CHUCKLES_CLASS, jester_hat_of_chuckles_new = "Server.Items.JesterHatofChuckles" : BASE_HAT / BASE_HAT_CLASS, |w, ty, hue| new_clothing(w, ty, 0x171C, hue), hue 0, encoded false;
    @template;
    default_weight: Some(1.0),
    base_physical_resistance: 12,
    base_fire_resistance: 12,
    base_cold_resistance: 12,
    base_poison_resistance: 12,
    base_energy_resistance: 12,
    init_min_hits: same(100),
    init_max_hits: same(100),
}

// ---- HoodedShroudOfShadows: [Flippable(0x2684, 0x2683)], BaseOuterTorso(0x2684, hue = 0x455) ----
clothing_leaf! {
    /// `Server.Items.HoodedShroudOfShadows`.
    HOODED_SHROUD_OF_SHADOWS, HOODED_SHROUD_OF_SHADOWS_CLASS, hooded_shroud_of_shadows_new = "Server.Items.HoodedShroudOfShadows" : BASE_OUTER_TORSO / BASE_OUTER_TORSO_CLASS, |w, ty, hue| new_clothing(w, ty, 0x2684, hue), hue 0x455, encoded false;
    @template;
    flippable: Some((0x2684, 0x2683)),
    default_weight: Some(3.0),
    dye: crate::clothing::dye_refused,
}

// ---- EarringsOfProtection ----

/// `EarringsOfProtection._attribute`.
#[derive(Clone, Copy, Debug)]
pub struct EarringData {
    pub attribute: AosElementAttribute,
}

/// `GetItemData(element, label)`.
fn earring_item_data(element: AosElementAttribute, label: bool) -> i32 {
    let (l, h) = match element {
        AosElementAttribute::Physical => (1071091, 0),
        AosElementAttribute::Fire => (1071092, 0x4ec),
        AosElementAttribute::Cold => (1071093, 0x4f2),
        AosElementAttribute::Poison => (1071094, 0x4f8),
        AosElementAttribute::Energy => (1071095, 0x4fe),
        _ => return -1,
    };
    if label { l } else { h }
}

fn earring_attribute(w: &World, id: ItemId) -> AosElementAttribute {
    w.item(id)
        .and_then(|i| i.ext.get::<EarringData>())
        .map_or(AosElementAttribute::Physical, |d| d.attribute)
}

/// `EarringsOfProtection.GetTypes(value)`.
pub fn earrings_get_types(value: i32) -> AosElementAttribute {
    match value {
        0 => AosElementAttribute::Physical,
        1 => AosElementAttribute::Fire,
        2 => AosElementAttribute::Cold,
        3 => AosElementAttribute::Poison,
        _ => AosElementAttribute::Energy,
    }
}

/// `EarringsOfProtection(element)`.
pub fn earrings_of_protection_new(w: &mut World, element: AosElementAttribute) -> ItemId {
    let id = new_jewel(w, &EARRINGS_OF_PROTECTION, 0x1087);
    set_attr(w, id, AttrKind::Element, element as u32, 2);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(EarringData { attribute: element });
    }
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}

/// `EarringsOfProtection()`: `this(RandomType())`.
pub fn earrings_of_protection_random(w: &mut World) -> ItemId {
    let n = utility::random(&mut w.rng, 5);
    earrings_of_protection_new(w, earrings_get_types(n))
}

/// The class table (`Layer.Earrings` is the `BaseJewel(itemID, layer)` argument).
pub static EARRINGS_OF_PROTECTION_CLASS: JewelClass = JewelClass {
    layer: Layer::Earrings,
    ..BASE_JEWEL_CLASS
};

/// `Server.Items.EarringsOfProtection`: version 0 (int), then `_attribute` as an enum (int).
pub static EARRINGS_OF_PROTECTION: ItemType = ItemType {
    full_name: "Server.Items.EarringsOfProtection",
    parent: Some(&BASE_JEWEL),
    construct: Some(earrings_of_protection_random),
    family: Some(&EARRINGS_OF_PROTECTION_CLASS),
    label_number: |w, id| earring_item_data(earring_attribute(w, id), true),
    hue: |w, id| earring_item_data(earring_attribute(w, id), false),
    serialize: |w, id, out| {
        (BASE_JEWEL.serialize)(w, id, out);
        out.write_i32(0);
        out.write_enum(4, earring_attribute(w, id) as i64);
    },
    deserialize: |w, id, r| {
        (BASE_JEWEL.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let attribute = match r.read_enum(4)? as i32 {
            0x2 => AosElementAttribute::Fire,
            0x4 => AosElementAttribute::Cold,
            0x8 => AosElementAttribute::Poison,
            0x10 => AosElementAttribute::Energy,
            0x20 => AosElementAttribute::Chaos,
            0x40 => AosElementAttribute::Direct,
            _ => AosElementAttribute::Physical, // 0x1; a number with no name has no value here
        };
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(EarringData { attribute });
        }
        Ok(())
    },
    ..BASE_JEWEL
};

// ---- RewardCake ----

/// `Server.Items.RewardCake`.
pub static REWARD_CAKE: ItemType = ItemType {
    full_name: "Server.Items.RewardCake",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let id = ops::new_item_with_id(w, &REWARD_CAKE, 0x9e9);
        ops::set_stackable(w, id, false);
        const HUES: [i32; 16] = [
            0x135, 0xcd, 0x38, 0x3b, 0x42, 0x4f, 0x11e, 0x60, 0x317, 0x10, 0x136, 0x1f9, 0x1a,
            0xeb, 0x86, 0x2e,
        ];
        let h = *utility::random_element(&mut w.rng, &HUES).unwrap_or(&0x135);
        super::hue(w, id, h);
        ops::set_loot_type(w, id, LootType::Blessed);
        id
    }),
    default_weight: |_, _| 1.0,
    label_number: |_, _| 1049786, // Happy Birthday!  ...
    display_loot_type: |_, _| false,
    on_double_click: |w, id, from| {
        let p = ops::get_world_location(w, id);
        if !deps::mobile_in_range(w, from, p, 1) {
            // I can't reach that.
            muo_server::mobile::part2::messages::local_overhead_message_number(
                w,
                from,
                MessageType::REGULAR,
                906,
                1019045,
                "",
            );
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..ITEM_BASE
};

/// Every generated class.
pub static ALL: &[(&ItemType, &[&str])] = &[
    (&HOLY_SWORD, &[]),
    (&LEGGINGS_OF_EMBERS, &[]),
    (&SHAMINO_CROSSBOW, &[]),
    (&MINOTAUR_HEDGE, &[]),
    (&TORMENTED_CHAINS, &[]),
    (&PLAGUE_BEAST_GLAND, &[]),
];

/// Registers the classes of this file.
pub fn register(types: &mut TypeRegistry) {
    for (t, aliases) in ALL {
        types.register_item(t, aliases);
    }
    types.register_item(&SAMURAI_HELM, &[]);
    types.register_item(&LUCKY_NECKLACE, &[]);
    types.register_item(&JESTER_HAT_OF_CHUCKLES, &[]);
    types.register_item(&HOODED_SHROUD_OF_SHADOWS, &[]);
    types.register_item(&EARRINGS_OF_PROTECTION, &[]);
    types.register_item(&REWARD_CAKE, &[]);
}

#[cfg(test)]
mod leaves_tests {
    use super::*;
    use muo_io::{SaveReader, SaveWriter};
    use muo_server::item::Item;

    fn classes() -> Vec<&'static ItemType> {
        let mut v: Vec<&'static ItemType> = ALL.iter().map(|(t, _)| *t).collect();
        v.extend([
            &SAMURAI_HELM,
            &LUCKY_NECKLACE,
            &JESTER_HAT_OF_CHUCKLES,
            &HOODED_SHROUD_OF_SHADOWS,
            &EARRINGS_OF_PROTECTION,
            &REWARD_CAKE,
        ]);
        v
    }

    fn save(w: &World, id: ItemId) -> Vec<u8> {
        let ty = w.item(id).unwrap().ty;
        let mut out = SaveWriter::new(true);
        (ty.serialize)(w, id, &mut out);
        out.into_vec()
    }

    #[test]
    fn special_leaf_classes_construct_and_round_trip() {
        for t in classes() {
            let mut w = World::new();
            let id = (t.construct.unwrap())(&mut w);
            assert_eq!(w.item(id).unwrap().ty.full_name, t.full_name);
            let bytes = save(&w, id);
            let mut w2 = World::new();
            let id2 = w2.add_item(Item::new_for_load(
                muo_primitives::Serial::new(0x4000_0001),
                t,
            ));
            let mut r = SaveReader::new(&bytes);
            (t.deserialize)(&mut w2, id2, &mut r)
                .unwrap_or_else(|e| panic!("{}: {e}", t.full_name));
            assert_eq!(r.remaining(), 0, "{} is read to its end", t.full_name);
            assert_eq!(save(&w2, id2), bytes, "{}", t.full_name);
        }
    }

    #[test]
    fn earrings_of_protection_follow_their_element() {
        let mut w = World::new();
        let id = earrings_of_protection_new(&mut w, AosElementAttribute::Cold);
        let ty = w.item(id).unwrap().ty;
        assert_eq!((ty.label_number)(&w, id), 1071093);
        assert_eq!((ty.hue)(&w, id), 0x4f2);
        // the last byte is the enum, an encoded int
        let bytes = save(&w, id);
        assert_eq!(&bytes[bytes.len() - 1..], &[4]); // an int enum is an encoded int
        assert_eq!(earring_item_data(AosElementAttribute::Chaos, true), -1);
    }
}
