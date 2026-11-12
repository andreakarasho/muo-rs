//! Plant seeds the monsters pack, and the other constructor parts that pick a random
//! resource. `Seed` is made through the overload table of its class (`new Seed(PlantType, PlantHue)`): without the
//! plants engine registered nothing is packed (the random numbers are drawn all the same).

use muo_content::creature::setters::{add_loot, add_loot_n};
use muo_content::factory;
use muo_content::loot_pack;
use muo_content::props::Value;
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{base as ibase, ops};

use super::ctors::{pack, si};
use crate::common::{pack_item, rnd};

/// `PlantHue.Plain` = `0x1 | Crossable | Reproduces`.
const PLAIN: i64 = 0x1 | 0x400_0000 | 0x200_0000;

// `PlantType` members used by the seeds.
const CACTUS: i64 = 25;
const FLAX_FLOWERS: i64 = 26;
const FOXGLOVE_FLOWERS: i64 = 27;
const HOPS_EAST: i64 = 28;
const ORFLUER_FLOWERS: i64 = 29;
const CYPRESS_TWISTED: i64 = 30;
const HEDGE_SHORT: i64 = 31;
const JUNIPER_BUSH: i64 = 32;
const SNOWDROP_PATCH: i64 = 33;
const CATTAILS: i64 = 34;
const POPPY_PATCH: i64 = 35;
const SPIDER_TREE: i64 = 36;
const WATER_LILY: i64 = 37;
const CYPRESS_STRAIGHT: i64 = 38;
const HEDGE_TALL: i64 = 39;
const HOPS_SOUTH: i64 = 40;
const SUGAR_CANES: i64 = 41;
const COCOA_TREE: i64 = 42;
// Bonsai: CommonGreen 17, CommonPink 18, UncommonGreen 19, UncommonPink 20, RareGreen 21, RarePink 22, Exceptional 23, Exotic 24.

/// `PlantTypeInfo.RandomPeculiarGroup<n>()`.
fn peculiar_type(w: &mut World, group: i32) -> i64 {
    match group {
        1 => [
            CACTUS,
            FLAX_FLOWERS,
            FOXGLOVE_FLOWERS,
            HOPS_EAST,
            COCOA_TREE,
            ORFLUER_FLOWERS,
        ][rnd::below(w, 6) as usize],
        2 => [
            CYPRESS_TWISTED,
            HEDGE_SHORT,
            JUNIPER_BUSH,
            COCOA_TREE,
            SNOWDROP_PATCH,
        ][rnd::below(w, 5) as usize],
        3 => {
            [CATTAILS, POPPY_PATCH, SPIDER_TREE, COCOA_TREE, WATER_LILY][rnd::below(w, 5) as usize]
        }
        _ => [
            CYPRESS_STRAIGHT,
            HEDGE_TALL,
            HOPS_SOUTH,
            COCOA_TREE,
            SUGAR_CANES,
        ][rnd::below(w, 5) as usize],
    }
}

/// `PlantTypeInfo.RandomBonsai(increaseRatio)`.
fn bonsai_type(w: &mut World, ratio: f64) -> i64 {
    let k1 = if ratio >= 0.0 { ratio } else { 0.0 };
    let k2 = k1 * k1;
    let k3 = k2 * k1;
    let k4 = k3 * k1;
    let exp1 = k1 + 1.0;
    let exp2 = k2 + exp1;
    let exp3 = k3 + exp2;
    let exp4 = k4 + exp3;
    let rand = rnd::double(w);
    if rand < 0.5 / exp4 {
        17
    } else if rand < 1.0 / exp4 {
        18
    } else if rand < (k1 * 0.5 + 1.0) / exp4 {
        19
    } else if rand < exp1 / exp4 {
        20
    } else if rand < (k2 * 0.5 + exp1) / exp4 {
        21
    } else if rand < exp2 / exp4 {
        22
    } else if rand < exp3 / exp4 {
        23
    } else {
        24
    }
}

fn make_seed(w: &mut World, plant_type: i64) -> Option<ItemId> {
    factory::make_item_args(
        w,
        "Server.Engines.Plants.Seed",
        &[Value::Enum(plant_type), Value::Enum(PLAIN)],
    )
}

/// `Seed.RandomPeculiarSeed(group)`.
pub fn random_peculiar_seed(w: &mut World, group: i32) -> Option<ItemId> {
    let t = peculiar_type(w, group);
    make_seed(w, t)
}

/// `Seed.RandomBonsaiSeed()` (increase ratio 0.5).
pub fn random_bonsai_seed(w: &mut World) -> Option<ItemId> {
    let t = bonsai_type(w, 0.5);
    make_seed(w, t)
}

/// `if (Core.ML && Utility.RandomDouble() < .33) PackItem(Seed.RandomPeculiarSeed(group))`.
fn peculiar(w: &mut World, id: MobileId, group: i32) {
    if w.core_ml() && rnd::double(w) < 0.33 {
        let s = random_peculiar_seed(w, group);
        pack_item(w, id, s);
    }
}

/// `if (Utility.RandomDouble() < chance) PackItem(Seed.RandomBonsaiSeed())`.
fn bonsai(w: &mut World, id: MobileId, chance: f64) {
    if rnd::double(w) < chance {
        let s = random_bonsai_seed(w);
        pack_item(w, id, s);
    }
}

pub fn ant_lion_ctor1(w: &mut World, id: MobileId) {
    peculiar(w, id, 2);
}
pub fn mummy_ctor1(w: &mut World, id: MobileId) {
    peculiar(w, id, 2);
}
pub fn serpentine_dragon_ctor1(w: &mut World, id: MobileId) {
    peculiar(w, id, 2);
}
pub fn terathan_warrior_ctor1(w: &mut World, id: MobileId) {
    peculiar(w, id, 3);
}
pub fn titan_ctor1(w: &mut World, id: MobileId) {
    peculiar(w, id, 1);
}
pub fn tsuki_wolf_ctor1(w: &mut World, id: MobileId) {
    peculiar(w, id, 1);
}
pub fn fan_dancer_ctor1(w: &mut World, id: MobileId) {
    bonsai(w, id, 0.33);
}
pub fn oni_ctor1(w: &mut World, id: MobileId) {
    bonsai(w, id, 0.33);
}
pub fn rune_beetle_ctor1(w: &mut World, id: MobileId) {
    bonsai(w, id, 0.25);
}
pub fn yomotsu_elder_ctor1(w: &mut World, id: MobileId) {
    bonsai(w, id, 0.25);
}
pub fn yomotsu_priest_ctor3(w: &mut World, id: MobileId) {
    bonsai(w, id, 0.25);
}
pub fn yomotsu_warrior_ctor2(w: &mut World, id: MobileId) {
    bonsai(w, id, 0.25);
}

/// An ore pile of a random one of four kinds, 1-10 of them, in the ant lion's ore picture.
pub fn ant_lion_ctor2(w: &mut World, id: MobileId) {
    let name = match rnd::below(w, 4) {
        0 => "DullCopperOre",
        1 => "ShadowIronOre",
        2 => "CopperOre",
        _ => "BronzeOre",
    };
    let pile = factory::make_item(w, &si(name));
    let amount = rnd::min_max(w, 1, 10);
    if let Some(p) = pile {
        ops::set_amount(w, p, amount);
        ibase::set_item_id(w, p, 0x19B9);
    }
    pack_item(w, id, pile);
}

/// `AddLoot(Rich); AddLoot(Average); AddLoot(Gems);` and 2%: one of five daemon armor pieces.
pub fn efreet_generate_loot(w: &mut World, id: MobileId) {
    add_loot(w, id, loot_pack::rich(w));
    add_loot(w, id, loot_pack::average(w));
    add_loot(w, id, &loot_pack::GEMS);
    if rnd::double(w) < 0.02 {
        let i = rnd::below(w, 5) as usize;
        pack(
            w,
            id,
            [
                "DaemonArms",
                "DaemonChest",
                "DaemonGloves",
                "DaemonLegs",
                "DaemonHelm",
            ][i],
        );
    }
}

/// `new SavageRidgeback().Rider = this;`
pub fn savage_rider_ctor1(w: &mut World, id: MobileId) {
    if let Some(m) = factory::make_mobile(w, "Server.Mobiles.SavageRidgeback") {
        muo_content::creature::mount::set_rider(w, m, Some(id));
    }
}

#[allow(unused_imports)]
use add_loot_n as _;

pub fn wisp_ctor1(w: &mut World, id: MobileId) {
    peculiar(w, id, 3);
}

/// `Hue = DefaultHue;` (the Twisted Weald changelings).
fn default_hue(w: &mut World, id: MobileId) {
    let h = muo_content::creature::ctype(w, id).map_or(0, |c| (c.default_hue)(w, id));
    crate::common::hue(w, id, h);
}
pub fn guile_ctor1(w: &mut World, id: MobileId) {
    default_hue(w, id);
}
pub fn irk_ctor1(w: &mut World, id: MobileId) {
    default_hue(w, id);
}
pub fn spite_ctor1(w: &mut World, id: MobileId) {
    default_hue(w, id);
}

/// `if (Utility.RandomDouble() < .33) PackItem(Seed.RandomBonsaiSeed())`.
pub fn bonsai_third(w: &mut World, id: MobileId) {
    bonsai(w, id, 0.33);
}
/// `if (Core.ML && Utility.RandomDouble() < .33) PackItem(Seed.RandomPeculiarSeed(group))`.
pub fn peculiar_third(w: &mut World, id: MobileId, group: i32) {
    peculiar(w, id, group);
}
/// `PackItem(Seed.RandomPeculiarSeed(group))`.
pub fn pack_peculiar(w: &mut World, id: MobileId, group: i32) {
    let s = random_peculiar_seed(w, group);
    pack_item(w, id, s);
}
