//! `TreasureChestLevel1` to `TreasureChestLevel4`: lockable chests that fill themselves in the
//! constructor and decay after a time rolled once per object (`_decayTime` is not serialized: a loaded chest rolls
//! again).

use muo_content::items::cont::{self, LOCKABLE_CONTAINER, TrapType, new_lockable};
use muo_content::items::gold::new_gold;
use muo_content::loot;
use muo_primitives::{Rectangle2D, TimeSpan};
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::{ItemType, container, ops};
use muo_server::type_registry::TypeRegistry;

use crate::simple::{random, random_bool, random_from};

/// `TreasureChestLevelN._decayTime`.
#[derive(Clone, Copy, Debug)]
struct ChestDecay(TimeSpan);

fn drop_item(w: &mut World, chest: ItemId, item: ItemId) {
    if let Some(ty) = w.item(chest).map(|i| i.ty) {
        (ty.drop_item)(w, chest, item);
    }
}

fn drop_opt(w: &mut World, chest: ItemId, item: Option<ItemId>) {
    if let Some(i) = item {
        drop_item(w, chest, i);
    }
}

fn amount(w: &mut World, item: Option<ItemId>, n: i32) -> Option<ItemId> {
    if let Some(i) = item {
        ops::set_amount(w, i, n);
    }
    item
}

/// The shared head of the four constructors, in order (the decay roll is a field initializer, so it comes
/// first): the appearance, `Movable`, the trap, `Locked` and the lock levels.
#[allow(clippy::too_many_arguments)] // the constructor's parameters, in order
fn begin(
    w: &mut World,
    ty: &'static ItemType,
    appearance: fn(&mut World, ItemId),
    trap: TrapType,
    power_mul: i32,
    power_min: i32,
    power_max: i32,
    skill: i32,
) -> ItemId {
    let decay = TimeSpan::from_mins(i64::from(random_from(w, 15, 60)));
    let id = new_lockable(w, ty, 0xE41);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(ChestDecay(decay));
    }
    appearance(w, id);
    ops::set_movable(w, id, false);
    let power = power_mul * random_from(w, power_min, power_max);
    if let Some(d) = cont::ext_mut(w, id) {
        d.trap_type = trap;
        d.trap_power = power;
    }
    cont::set_locked(w, id, true);
    let lock = skill - random_from(w, 1, 10);
    let max = skill + random_from(w, 1, 10);
    if let Some(d) = cont::ext_mut(w, id) {
        d.required_skill = skill;
        d.lock_level = lock;
        d.max_lock_level = max;
    }
    id
}

/// `SetChestAppearance()` of level 1: only the graphic.
fn appearance1(w: &mut World, id: ItemId) {
    let gid = match random(w, 6) {
        0 => 0xe3c,
        1 => 0xe3d,
        2 => 0xe3e,
        3 => 0xe3f,
        4 => 0x9a9,
        _ => 0xe7e,
    };
    crate::simple::item_id(w, id, gid);
}

/// `_chestAppearances` of levels 2 and 4 (crates, chests, a keg and a barrel).
const APPEARANCES_ALL: [(i32, i32); 14] = [
    (0xe3c, 0x44),
    (0xe3d, 0x44),
    (0xe3e, 0x44),
    (0xe3f, 0x44),
    (0x9a9, 0x44),
    (0xe7e, 0x44),
    (0xe42, 0x49),
    (0xe43, 0x49),
    (0x9ab, 0x4A),
    (0xe7c, 0x4A),
    (0xe40, 0x42),
    (0xe41, 0x42),
    (0xe7f, 0x3e),
    (0xe77, 0x3e),
];

fn set_appearance(w: &mut World, id: ItemId, table: &[(i32, i32)]) {
    let i = random(w, table.len() as i32) as usize;
    let (gid, gump) = table[i];
    crate::simple::item_id(w, id, gid);
    container::set_gump_id(w, id, gump);
}

fn appearance_all(w: &mut World, id: ItemId) {
    set_appearance(w, id, &APPEARANCES_ALL);
}

/// Level 3: the chests only.
fn appearance3(w: &mut World, id: ItemId) {
    set_appearance(w, id, &APPEARANCES_ALL[6..12]);
}

fn new1(w: &mut World) -> ItemId {
    let id = begin(
        w,
        &TREASURE_CHEST_LEVEL1,
        appearance1,
        TrapType::DartTrap,
        1,
        1,
        25,
        57,
    );
    // Utility.Random(30, 100): from 30, count 100
    let n = random_from(w, 30, 100);
    let g = new_gold(w, n);
    drop_item(w, id, g);
    if random_bool(w) {
        let gems = loot::random_gem(w);
        let n = random_from(w, 1, 3);
        let gems = amount(w, gems, n);
        drop_opt(w, id, gems);
    }
    if random_bool(w) {
        let i = loot::random_weapon(w, false, false, false);
        drop_opt(w, id, i);
    }
    if random_bool(w) {
        let i = loot::random_armor_or_shield(w, false, false);
        drop_opt(w, id, i);
    }
    if random_bool(w) {
        let i = loot::random_clothing(w, false, false);
        drop_opt(w, id, i);
    }
    if random_bool(w) {
        let i = loot::random_jewelry(w);
        drop_opt(w, id, i);
    }
    id
}

fn new2(w: &mut World) -> ItemId {
    let id = begin(
        w,
        &TREASURE_CHEST_LEVEL2,
        appearance_all,
        TrapType::ExplosionTrap,
        2,
        1,
        25,
        72,
    );
    let n = random_from(w, 70, 100);
    let g = new_gold(w, n);
    drop_item(w, id, g);
    let mut i = random(w, 3);
    while i > 0 {
        let r = loot::random_reagent(w);
        let n = random_from(w, 1, 2);
        let r = amount(w, r, n);
        drop_opt(w, id, r);
        i -= 1;
    }
    if random_bool(w) {
        let s = loot::random_scroll(w, 0, 39, loot::SpellbookType::Regular);
        let n = random_from(w, 1, 8);
        let s = amount(w, s, n);
        drop_opt(w, id, s);
    }
    if random_bool(w) {
        let p = loot::random_potion(w);
        drop_opt(w, id, p);
    }
    if random_bool(w) {
        let gems = loot::random_gem(w);
        let n = random_from(w, 1, 6);
        let gems = amount(w, gems, n);
        drop_opt(w, id, gems);
    }
    id
}

/// The wand, equipment, clothing and jewelry loops that close the level 3 (`n` = 3) and level 4 (`n` = 4) constructors.
/// The equipment's enchantment levels are `Utility.Random(n)`.
fn magic_tail(w: &mut World, id: ItemId, n: i32) {
    use muo_content::items::{armor, weapon};
    let mut i = random(w, n);
    while i > 0 {
        let wand = loot::random_wand(w);
        drop_opt(w, id, wand);
        i -= 1;
    }
    let mut i = random(w, n);
    while i > 0 {
        let item = loot::random_armor_or_shield_or_weapon(w, false, false);
        if let Some(it) = item {
            if weapon::is_weapon(w, it) {
                let (d, a, du) = (random(w, n), random(w, n), random(w, n));
                weapon::set_damage_level(
                    w,
                    it,
                    weapon::WeaponDamageLevel::from_i32(d)
                        .unwrap_or(weapon::WeaponDamageLevel::Regular),
                );
                weapon::set_accuracy_level(
                    w,
                    it,
                    weapon::WeaponAccuracyLevel::from_i32(a)
                        .unwrap_or(weapon::WeaponAccuracyLevel::Regular),
                );
                weapon::set_durability_level(
                    w,
                    it,
                    weapon::WeaponDurabilityLevel::from_i32(du)
                        .unwrap_or(weapon::WeaponDurabilityLevel::Regular),
                );
                weapon::set_quality(w, it, weapon::WeaponQuality::Regular);
            } else if armor::is_armor(w, it) {
                let (p, du) = (random(w, n), random(w, n));
                armor::set_protection_level(
                    w,
                    it,
                    armor::ArmorProtectionLevel::from_i32(p)
                        .unwrap_or(armor::ArmorProtectionLevel::Regular),
                );
                armor::set_durability_level(
                    w,
                    it,
                    armor::ArmorDurabilityLevel::from_i32(du)
                        .unwrap_or(armor::ArmorDurabilityLevel::Regular),
                );
                armor::set_quality(w, it, armor::ArmorQuality::Regular);
            }
        }
        drop_opt(w, id, item);
        i -= 1;
    }
    let mut i = random(w, 3);
    while i > 0 {
        let c = loot::random_clothing(w, false, false);
        drop_opt(w, id, c);
        i -= 1;
    }
    let mut i = random(w, 3);
    while i > 0 {
        let j = loot::random_jewelry(w);
        drop_opt(w, id, j);
        i -= 1;
    }
}

fn new3(w: &mut World) -> ItemId {
    let id = begin(
        w,
        &TREASURE_CHEST_LEVEL3,
        appearance3,
        TrapType::PoisonTrap,
        3,
        1,
        25,
        84,
    );
    let n = random_from(w, 180, 240);
    let g = new_gold(w, n);
    drop_item(w, id, g);
    let mut i = random(w, 2);
    while i >= 0 {
        let r = loot::random_reagent(w);
        let n = random_from(w, 1, 9);
        let r = amount(w, r, n);
        drop_opt(w, id, r);
        i -= 1;
    }
    let mut i = random(w, 3);
    while i > 0 {
        let s = loot::random_scroll(w, 0, 47, loot::SpellbookType::Regular);
        let n = random_from(w, 1, 12);
        let s = amount(w, s, n);
        drop_opt(w, id, s);
        i -= 1;
    }
    let mut i = random(w, 3);
    while i > 0 {
        let p = loot::random_potion(w);
        drop_opt(w, id, p);
        i -= 1;
    }
    let mut i = random(w, 3);
    while i > 0 {
        let gems = loot::random_gem(w);
        let n = random_from(w, 1, 9);
        let gems = amount(w, gems, n);
        drop_opt(w, id, gems);
        i -= 1;
    }
    magic_tail(w, id, 3);
    id
}

fn new4(w: &mut World) -> ItemId {
    let id = begin(
        w,
        &TREASURE_CHEST_LEVEL4,
        appearance_all,
        TrapType::ExplosionTrap,
        4,
        10,
        25,
        92,
    );
    let n = random_from(w, 200, 400);
    let g = new_gold(w, n);
    drop_item(w, id, g);
    let mut i = random(w, 4);
    while i > 0 {
        let r = loot::random_reagent(w);
        let r = amount(w, r, 12);
        drop_opt(w, id, r);
        i -= 1;
    }
    let mut i = random(w, 4);
    while i > 0 {
        let s = loot::random_scroll(w, 0, 47, loot::SpellbookType::Regular);
        let s = amount(w, s, 16);
        drop_opt(w, id, s);
        i -= 1;
    }
    let n = random_from(w, 1, 4);
    if let Some(b) = muo_content::factory::make_item_amount(w, "Server.Items.BlankScroll", n) {
        drop_item(w, id, b);
    }
    let mut i = random(w, 4);
    while i > 0 {
        let p = loot::random_potion(w);
        drop_opt(w, id, p);
        i -= 1;
    }
    let mut i = random(w, 4);
    while i > 0 {
        let gems = loot::random_gem(w);
        let gems = amount(w, gems, 12);
        drop_opt(w, id, gems);
        i -= 1;
    }
    magic_tail(w, id, 4);
    id
}

macro_rules! chests {
    ($($name:ident = $full:literal, $ctor:ident;)*) => {$(
        #[doc = concat!("`", $full, "`.")]
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&LOCKABLE_CONTAINER),
            construct: Some($ctor),
            decays: |_, _| true,
            is_deco_container: |_, _| false,
            decay_time: |w, id| w.item(id).and_then(|i| i.ext.get::<ChestDecay>()).map_or(TimeSpan::ZERO, |d| d.0),
            default_gump_id: crate::class_props::default_gump_id,
            default_drop_sound: crate::class_props::default_drop_sound,
            bounds: crate::class_props::bounds,
            serialize: |w, id, out| {
                (LOCKABLE_CONTAINER.serialize)(w, id, out);
                out.write_i32(0);
            },
            deserialize: |w, id, r| {
                (LOCKABLE_CONTAINER.deserialize)(w, id, r)?;
                let _version = r.read_i32()?;
                // the field initializer rolls on every load
                let decay = TimeSpan::from_mins(i64::from(random_from(w, 15, 60)));
                if let Some(i) = w.item_mut(id) {
                    i.ext.insert(ChestDecay(decay));
                }
                Ok(())
            },
            ..LOCKABLE_CONTAINER
        };
    )*};
}

chests! {
    TREASURE_CHEST_LEVEL1 = "Server.Items.TreasureChestLevel1", new1;
    TREASURE_CHEST_LEVEL2 = "Server.Items.TreasureChestLevel2", new2;
    TREASURE_CHEST_LEVEL3 = "Server.Items.TreasureChestLevel3", new3;
    TREASURE_CHEST_LEVEL4 = "Server.Items.TreasureChestLevel4", new4;
}

/// Registers the classes.
pub fn register(types: &mut TypeRegistry) {
    for t in [
        &TREASURE_CHEST_LEVEL1,
        &TREASURE_CHEST_LEVEL2,
        &TREASURE_CHEST_LEVEL3,
        &TREASURE_CHEST_LEVEL4,
    ] {
        types.register_item(t, &[]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use muo_io::{SaveReader, SaveWriter};

    #[test]
    fn level_one_chest_is_locked_trapped_and_holds_gold() {
        for _ in 0..10 {
            let mut w = World::new();
            let id = (TREASURE_CHEST_LEVEL1.construct.unwrap())(&mut w);
            let d = cont::ext(&w, id).unwrap().clone();
            assert_eq!(
                (d.trap_type, d.required_skill, d.locked),
                (TrapType::DartTrap, 57, true)
            );
            assert!((1..=25).contains(&d.trap_power));
            assert!((47..=56).contains(&d.lock_level) && (58..=67).contains(&d.max_lock_level));
            assert!(!w.item(id).unwrap().movable());
            assert!(!w.item(id).unwrap().items.is_empty());
            let t = (TREASURE_CHEST_LEVEL1.decay_time)(&w, id);
            assert!((15..75).contains(&((t.ticks() / 600_000_000) as i32))); // Utility.Random(15, 60): 60 values from 15
        }
    }

    #[test]
    fn deeper_chests_construct_and_round_trip() {
        for t in [
            &TREASURE_CHEST_LEVEL2,
            &TREASURE_CHEST_LEVEL3,
            &TREASURE_CHEST_LEVEL4,
        ] {
            let mut w = World::new();
            let id = (t.construct.unwrap())(&mut w);
            assert!(cont::locked(&w, id));
            let mut out = SaveWriter::new(true);
            (t.serialize)(&w, id, &mut out);
            let bytes = out.into_vec();
            let id2 = w.add_item(muo_server::item::Item::new_for_load(
                muo_primitives::Serial::new(0x4000_0100),
                t,
            ));
            let mut r = SaveReader::new(&bytes);
            (t.deserialize)(&mut w, id2, &mut r).unwrap();
            assert_eq!(r.remaining(), 0);
        }
    }

    // DecayScheduler reads ScheduledDecayTime repeatedly and needs the reads to agree, so the random interval must
    // be rolled once per chest rather than per read.
    #[test]
    fn scheduled_decay_time_is_stable_across_reads() {
        for t in [
            &TREASURE_CHEST_LEVEL1,
            &TREASURE_CHEST_LEVEL2,
            &TREASURE_CHEST_LEVEL3,
            &TREASURE_CHEST_LEVEL4,
        ] {
            let mut w = muo_server::map::test_support::world();
            let id = (t.construct.unwrap())(&mut w);
            ops::move_to_world_map(
                &mut w,
                id,
                muo_primitives::Point3D::new(150, 150, 0),
                Some(muo_server::map::MapId::FELUCCA),
            );
            let expected = ops::scheduled_decay_time(&w, id);
            for _ in 0..100 {
                assert_eq!(ops::scheduled_decay_time(&w, id), expected);
            }
        }
    }
}
