//! The `BaseCreature` `Pack*` helpers leaf `GenerateLoot` methods call.

use muo_server::ids::{ItemId, MobileId};
use muo_server::world::World;

use super::setters::pack_item;
use crate::factory::item_is;
use crate::hooks::rnd;
use crate::loot::{self, SpellbookType};

type ApplyAosAttributesFn = fn(&mut World, ItemId, i32, i32, i32);
type SetWeaponLevelsFn = fn(&mut World, ItemId, bool, i32, i32, i32);
/// The item-side writes of `PackArmor`/`PackWeapon`/`PackSlayer` (the items task sets these in its `register`).
#[derive(Default)]
pub struct PackHooks {
    /// `BaseRunicTool.ApplyAttributesTo(item, attributeCount, min, max)` for armor, weapon and jewelry.
    pub apply_aos_attributes: Option<ApplyAosAttributesFn>,
    /// `armor.ProtectionLevel = ...; armor.Durability = ...`.
    pub set_armor_levels: Option<fn(&mut World, ItemId, i32, i32)>,
    /// `weapon.Slayer = Silver (if silver); DamageLevel, AccuracyLevel, DurabilityLevel`.
    pub set_weapon_levels: Option<SetWeaponLevelsFn>,
    /// `item.Slayer = SlayerGroup.GetLootSlayerType(creatureType)` for an instrument or weapon.
    pub set_loot_slayer: Option<fn(&mut World, ItemId, MobileId)>,
}

#[allow(clippy::type_complexity)] // a callback type, spelled out
fn hooks(
    w: &World,
) -> (
    Option<fn(&mut World, ItemId, i32, i32, i32)>,
    Option<fn(&mut World, ItemId, i32, i32)>,
    Option<fn(&mut World, ItemId, bool, i32, i32, i32)>,
    Option<fn(&mut World, ItemId, MobileId)>,
) {
    w.resources
        .get::<PackHooks>()
        .map_or((None, None, None, None), |h| {
            (
                h.apply_aos_attributes,
                h.set_armor_levels,
                h.set_weapon_levels,
                h.set_loot_slayer,
            )
        })
}

/// `GetRandomAOSStats`: (attributeCount, min, max).
pub fn get_random_aos_stats(w: &mut World, min_level: i32, max_level: i32) -> (i32, i32, i32) {
    let v = random_min_max_scaled(w, min_level, max_level);
    if v >= 5 {
        (rnd::min_max(w, 2, 6), 20, 70)
    } else if v == 4 {
        (rnd::min_max(w, 2, 4), 20, 50)
    } else if v == 3 {
        (rnd::min_max(w, 2, 3), 20, 40)
    } else if v == 2 {
        (rnd::min_max(w, 1, 2), 10, 30)
    } else {
        (1, 10, 20)
    }
}

/// `RandomMinMaxScaled`: lower levels are likelier (the weights are the squares of the remaining count).
pub fn random_min_max_scaled(w: &mut World, mut min: i32, mut max: i32) -> i32 {
    if min == max {
        return min;
    }
    if min > max {
        std::mem::swap(&mut min, &mut max);
    }
    let count = max - min + 1;
    let mut total = 0;
    let mut to_add = count;
    for _ in 0..count {
        total += to_add * to_add;
        to_add -= 1;
    }
    let mut rand = rnd::below(w, total);
    to_add = count;
    let mut val = min;
    for _ in 0..count {
        rand -= to_add * to_add;
        if rand < 0 {
            break;
        }
        to_add -= 1;
        val += 1;
    }
    val
}

/// `PackArmor`(minLevel, maxLevel, chance)
pub fn pack_armor(
    w: &mut World,
    id: MobileId,
    min_level: i32,
    max_level: i32,
    chance: f64,
) -> bool {
    if chance <= rnd::double(w) {
        return false;
    }
    let (min_level, max_level) = (min_level.clamp(0, 5), max_level.clamp(0, 5));
    let (aos_hook, armor_levels, _, _) = hooks(w);
    if w.core_aos() {
        let Some(item) = loot::random_armor_or_shield_or_jewelry(w, false, false) else {
            return false;
        };
        let (count, min, max) = get_random_aos_stats(w, min_level, max_level);
        if (item_is(w, item, "Server.Items.BaseArmor")
            || item_is(w, item, "Server.Items.BaseJewel"))
            && let Some(f) = aos_hook
        {
            f(w, item, count, min, max);
        }
        pack_item(w, id, Some(item));
    } else {
        let Some(armor) = loot::random_armor_or_shield(w, false, false) else {
            return false;
        };
        let prot = random_min_max_scaled(w, min_level, max_level);
        let dur = random_min_max_scaled(w, min_level, max_level);
        if let Some(f) = armor_levels {
            f(w, armor, prot, dur);
        }
        pack_item(w, id, Some(armor));
    }
    true
}

/// `PackWeapon`(minLevel, maxLevel, chance)
pub fn pack_weapon(
    w: &mut World,
    id: MobileId,
    min_level: i32,
    max_level: i32,
    chance: f64,
) -> bool {
    if chance <= rnd::double(w) {
        return false;
    }
    let (min_level, max_level) = (min_level.clamp(0, 5), max_level.clamp(0, 5));
    let (aos_hook, _, weapon_levels, _) = hooks(w);
    if w.core_aos() {
        let Some(item) = loot::random_weapon_or_jewelry(w, false, false) else {
            return false;
        };
        let (count, min, max) = get_random_aos_stats(w, min_level, max_level);
        if (item_is(w, item, "Server.Items.BaseWeapon")
            || item_is(w, item, "Server.Items.BaseJewel"))
            && let Some(f) = aos_hook
        {
            f(w, item, count, min, max);
        }
        pack_item(w, id, Some(item));
    } else {
        let Some(weapon) = loot::random_weapon(w, false, false, false) else {
            return false;
        };
        let silver = rnd::double(w) < 0.05;
        let dmg = random_min_max_scaled(w, min_level, max_level);
        let acc = random_min_max_scaled(w, min_level, max_level);
        let dur = random_min_max_scaled(w, min_level, max_level);
        if let Some(f) = weapon_levels {
            f(w, weapon, silver, dmg, acc, dur);
        }
        pack_item(w, id, Some(weapon));
    }
    true
}

/// `PackMagicItems`(minLevel, maxLevel, armorChance, weaponChance)
pub fn pack_magic_items(
    w: &mut World,
    id: MobileId,
    min_level: i32,
    max_level: i32,
    armor_chance: f64,
    weapon_chance: f64,
) {
    if !pack_armor(w, id, min_level, max_level, armor_chance) {
        pack_weapon(w, id, min_level, max_level, weapon_chance);
    }
}

/// `PackSlayer`(chance)
pub fn pack_slayer(w: &mut World, id: MobileId, chance: f64) -> bool {
    if chance <= rnd::double(w) {
        return false;
    }
    let (_, _, _, slayer) = hooks(w);
    if rnd::bool(w) {
        if let Some(instrument) = loot::random_instrument(w) {
            if let Some(f) = slayer {
                f(w, instrument, id);
            }
            pack_item(w, id, Some(instrument));
        }
    } else if !w.core_aos()
        && let Some(weapon) = loot::random_weapon(w, false, false, false)
    {
        if let Some(f) = slayer {
            f(w, weapon, id);
        }
        pack_item(w, id, Some(weapon));
    }
    true
}

/// `PackArcaneScroll`()
pub fn pack_arcane_scroll(w: &mut World, id: MobileId) {
    if !w.core_ml() {
        return;
    }
    let it = loot::construct_any(w, loot::ARCANIST_SCROLL_TYPES);
    pack_item(w, id, it);
}

pub fn pack_arcane_scroll_n(w: &mut World, id: MobileId, amount: i32) {
    for _ in 0..amount {
        pack_arcane_scroll(w, id);
    }
}

pub fn pack_arcane_scroll_range(w: &mut World, id: MobileId, min: i32, max: i32) {
    let n = rnd::min_max(w, min, max);
    pack_arcane_scroll_n(w, id, n);
}

/// `PackArcanceScroll`(chance) (sic)
pub fn pack_arcane_scroll_chance(w: &mut World, id: MobileId, chance: f64) {
    if !w.core_ml() || chance <= rnd::double(w) {
        return;
    }
    let it = loot::construct_any(w, loot::ARCANIST_SCROLL_TYPES);
    pack_item(w, id, it);
}

/// `PackPotion`
pub fn pack_potion(w: &mut World, id: MobileId) {
    let it = loot::random_potion(w);
    pack_item(w, id, it);
}

/// `PackNecroScroll`(index)
pub fn pack_necro_scroll(w: &mut World, id: MobileId, index: i32) {
    if !w.core_aos() || rnd::double(w) < 0.95 {
        return;
    }
    let it = loot::construct_index(w, loot::NECROMANCY_SCROLL_TYPES, index);
    pack_item(w, id, it);
}

/// `PackScroll`(circle)
pub fn pack_scroll(w: &mut World, id: MobileId, circle: i32) {
    let min = (circle - 1) * 8;
    let it = loot::random_scroll(w, min, min + 7, SpellbookType::Regular);
    pack_item(w, id, it);
}

pub fn pack_scroll_range(w: &mut World, id: MobileId, min_circle: i32, max_circle: i32) {
    let c = rnd::min_max(w, min_circle, max_circle);
    pack_scroll(w, id, c);
}

fn pack_stack(w: &mut World, id: MobileId, item: Option<ItemId>, amount: i32) {
    if let Some(it) = item {
        muo_server::item::ops::set_amount(w, it, amount);
    }
    pack_item(w, id, item);
}

/// `PackGem`(amount)
pub fn pack_gem(w: &mut World, id: MobileId, amount: i32) {
    if amount <= 0 {
        return;
    }
    let gem = loot::random_gem(w);
    pack_stack(w, id, gem, amount);
}

pub fn pack_gem_range(w: &mut World, id: MobileId, min: i32, max: i32) {
    let n = rnd::min_max(w, min, max);
    pack_gem(w, id, n);
}

/// `PackReg`(amount)
pub fn pack_reg(w: &mut World, id: MobileId, amount: i32) {
    if amount <= 0 {
        return;
    }
    let reg = loot::random_reagent(w);
    pack_stack(w, id, reg, amount);
}

pub fn pack_reg_range(w: &mut World, id: MobileId, min: i32, max: i32) {
    let n = rnd::min_max(w, min, max);
    pack_reg(w, id, n);
}

/// `PackNecroReg`()
pub fn pack_necro_reg(w: &mut World, id: MobileId) {
    if !w.core_aos() {
        return;
    }
    let it = loot::random_necromancy_reagent(w);
    pack_item(w, id, it);
}

pub fn pack_necro_reg_n(w: &mut World, id: MobileId, amount: i32) {
    for _ in 0..amount {
        pack_necro_reg(w, id);
    }
}

pub fn pack_necro_reg_range(w: &mut World, id: MobileId, min: i32, max: i32) {
    let n = rnd::min_max(w, min, max);
    pack_necro_reg_n(w, id, n);
}

/// `PackStatue`()
pub fn pack_statue(w: &mut World, id: MobileId) {
    let it = loot::random_statue(w);
    pack_item(w, id, it);
}

pub fn pack_statue_n(w: &mut World, id: MobileId, amount: i32) {
    for _ in 0..amount {
        pack_statue(w, id);
    }
}

pub fn pack_statue_range(w: &mut World, id: MobileId, min: i32, max: i32) {
    let n = rnd::min_max(w, min, max);
    pack_statue_n(w, id, n);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scaled_roll_stays_in_range_and_favors_the_low_end() {
        let mut w = World::new();
        let mut counts = [0u32; 5];
        for _ in 0..5500 {
            let v = random_min_max_scaled(&mut w, 1, 5);
            assert!((1..=5).contains(&v));
            counts[(v - 1) as usize] += 1;
        }
        // 25/55, 16/55, 9/55, 4/55, 1/55 of 5500, with slack.
        assert!(
            counts[0] > counts[1]
                && counts[1] > counts[2]
                && counts[2] > counts[3]
                && counts[3] > counts[4]
        );
        assert_eq!(random_min_max_scaled(&mut w, 3, 3), 3);
        let swapped = random_min_max_scaled(&mut w, 5, 1);
        assert!((1..=5).contains(&swapped));
    }

    #[test]
    fn aos_stats_bands() {
        let mut w = World::new();
        assert_eq!(get_random_aos_stats(&mut w, 0, 0), (1, 10, 20));
        let (c, min, max) = get_random_aos_stats(&mut w, 5, 5);
        assert!((2..=6).contains(&c) && (min, max) == (20, 70));
        let (c, min, max) = get_random_aos_stats(&mut w, 2, 2);
        assert!((1..=2).contains(&c) && (min, max) == (10, 30));
    }
}
