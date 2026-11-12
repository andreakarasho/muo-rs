//! Puzzle chests: `PuzzleChestSolution`, `PuzzleChestSolutionAndTime`, `PuzzleChest`
//! (`BaseTreasureChest`), its gumps, and `MetalGoldenPuzzleChest` / `StrongBoxPuzzle`.
//!
//! Quirks kept for save and behavior compatibility: `PuzzleChest`'s constructor allocates `_hints` after `base(itemID)` ran
//! `GenerateTreasure()`, so a new chest has three zero hints; a second wrong guess by the same player before the
//! hour cleanup hits `Dictionary.Add` of a present key, which fails, so nothing happens.

use indexmap::IndexMap;
use muo_content::gumps::base::{
    DynamicGumpLogic, DynamicGumpObj, RelayInfo, StaticGumpLogic, StaticGumpObj,
};
use muo_content::gumps::builder::{DynamicGumpBuilder, GumpStrings, StaticGumpBuilder};
use muo_content::gumps::send_gump_to_mobile;
use muo_content::hooks::rnd;
use muo_content::items::cont::{
    self, BASE_TREASURE_CHEST, LOCKABLE_CLASS, TREASURE_CLASS, TreasureLevel, new_treasure_chest,
};
use muo_content::items::{armor, clothing, jewel, weapon};
use muo_content::misc::aos_helpers::{self, DamageOptions, DamageSplit};
use muo_primitives::random::Rng;
use muo_primitives::{DateTime, TimeSpan};
use muo_server::World;

use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{ItemType, ops};
use muo_server::net_state::NetStateId;
use muo_server::packets::message::MessageType;
use muo_server::poison::PoisonId;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::skills::SkillName;
use muo_server::timer::{self, TimerHandle};
use muo_server::type_registry::TypeRegistry;

pub const LENGTH: usize = 5;
pub const HINTS_COUNT: usize = 3;
pub const CLEANUP_TIME: TimeSpan = TimeSpan::from_hours(1);

/// `PuzzleChestCylinder`.
pub mod cylinder {
    pub const NONE: i32 = 0xE73;
    pub const LIGHT_BLUE: i32 = 0x186F;
    pub const BLUE: i32 = 0x186A;
    pub const GREEN: i32 = 0x186B;
    pub const ORANGE: i32 = 0x186C;
    pub const PURPLE: i32 = 0x186D;
    pub const RED: i32 = 0x186E;
    pub const DARK_BLUE: i32 = 0x1869;
    pub const YELLOW: i32 = 0x1870;
}

/// `PuzzleChestSolution.RandomCylinder()`.
pub fn random_cylinder(w: &mut World) -> i32 {
    match rnd::below(w, 8) {
        0 => cylinder::LIGHT_BLUE,
        1 => cylinder::BLUE,
        2 => cylinder::GREEN,
        3 => cylinder::ORANGE,
        4 => cylinder::PURPLE,
        5 => cylinder::RED,
        6 => cylinder::DARK_BLUE,
        _ => cylinder::YELLOW,
    }
}

/// `new PuzzleChestSolution()`: five random cylinders.
fn random_solution(w: &mut World) -> Vec<i32> {
    (0..LENGTH).map(|_| random_cylinder(w)).collect()
}

/// `PuzzleChestSolution.Matches(solution, out cylinders, out colors)`: `(all matched, cylinders, colors)`.
pub fn matches(this: &[i32], solution: &[i32]) -> (bool, i32, i32) {
    let (mut cylinders, mut colors) = (0, 0);
    let mut src = vec![false; solution.len()];
    let mut dst = vec![false; solution.len()];
    for i in 0..this.len() {
        if this[i] == solution[i] {
            cylinders += 1;
            src[i] = true;
            dst[i] = true;
        }
    }
    for i in 0..this.len() {
        if src[i] {
            continue;
        }
        for j in 0..solution.len() {
            if this[i] == solution[j] && !dst[j] {
                colors += 1;
                dst[j] = true;
            }
        }
    }
    (cylinders as usize == this.len(), cylinders, colors)
}

/// `PuzzleChestSolution.Serialize`.
fn write_solution(c: &[i32], out: &mut dyn muo_io::GenericWriter) {
    out.write_encoded_int(1);
    out.write_encoded_int(c.len() as i32);
    for v in c {
        out.write_enum(4, i64::from(*v));
    }
}

/// `PuzzleChestSolution.Deserialize` plus its `[AfterDeserialization]` (pad with random cylinders or cut to five).
fn read_solution(w: &mut World, r: &mut dyn muo_io::GenericReader) -> muo_io::Result<Vec<i32>> {
    let version = r.read_encoded_int()?;
    let len = r.read_encoded_int()?.max(0) as usize;
    let mut c = Vec::with_capacity(len.min(1024));
    for _ in 0..len {
        c.push(if version < 1 {
            r.read_i32()?
        } else {
            r.read_enum(4)? as i32
        });
    }
    // The constructor `new PuzzleChestSolution(chest)` rolled five cylinders before the read.
    let _ = random_solution(w);
    if c.len() > LENGTH {
        c.truncate(LENGTH);
    } else {
        while c.len() < LENGTH {
            let v = random_cylinder(w);
            c.push(v);
        }
    }
    Ok(c)
}

/// What a `PuzzleChest` adds to a treasure chest.
#[derive(Clone, Default)]
pub struct PuzzleData {
    pub solution: Vec<i32>,
    pub hints: Vec<i32>,
    /// `_guesses` (None is null): the guess and when it was made.
    pub guesses: Option<IndexMap<MobileId, (Vec<i32>, DateTime)>>,
    /// `_cleanupTimerToken`.
    pub cleanup: TimerHandle,
}

fn data(w: &World, id: ItemId) -> Option<&PuzzleData> {
    w.item(id)?.ext.get::<PuzzleData>()
}

fn data_mut(w: &mut World, id: ItemId) -> Option<&mut PuzzleData> {
    w.item_mut(id)?.ext.get_mut::<PuzzleData>()
}

fn ensure(w: &mut World, id: ItemId) {
    if data(w, id).is_none()
        && let Some(i) = w.item_mut(id)
    {
        i.ext.insert(PuzzleData::default());
    }
}

/// `Solution = value` (`OnSolutionChanged`: `InitHints()`).
fn set_solution(w: &mut World, id: ItemId, value: Vec<i32>) {
    ensure(w, id);
    if let Some(d) = data_mut(w, id) {
        if d.solution == value {
            return;
        }
        d.solution = value;
    }
    init_hints(w, id);
}

/// `InitHints()`: the last four cylinders shuffled, the first three kept.
fn init_hints(w: &mut World, id: ItemId) {
    let Some(mut c) = data(w, id).map(|d| d.solution.get(1..).unwrap_or(&[]).to_vec()) else {
        return;
    };
    w.rng.shuffle(&mut c[..]);
    if let Some(d) = data_mut(w, id) {
        d.hints = c.into_iter().take(HINTS_COUNT).collect();
    }
}

// ---- cleanup timer ----

fn start_cleanup_timer(w: &mut World, id: ItemId) {
    let Some(d) = data(w, id) else { return };
    if d.cleanup.running(w) || d.guesses.as_ref().is_none_or(|g| g.is_empty()) {
        return;
    }
    let h = timer::start(w, CLEANUP_TIME, CLEANUP_TIME, 0, move |w| {
        cleanup_guesses(w, id)
    });
    if let Some(d) = data_mut(w, id) {
        d.cleanup = h;
    }
}

fn stop_cleanup_timer(w: &mut World, id: ItemId) {
    let Some(mut h) = data(w, id).map(|d| d.cleanup) else {
        return;
    };
    h.cancel(w);
    if let Some(d) = data_mut(w, id) {
        d.cleanup = TimerHandle::default();
    }
}

fn cleanup_guesses(w: &mut World, id: ItemId) {
    let now = w.now;
    let Some(d) = data_mut(w, id) else { return };
    let Some(g) = d.guesses.as_mut() else {
        stop_cleanup_timer(w, id);
        return;
    };
    if g.is_empty() {
        stop_cleanup_timer(w, id);
        return;
    }
    g.retain(|_, (_, when)| now - *when <= CLEANUP_TIME);
    if g.is_empty() {
        d.guesses = None;
        stop_cleanup_timer(w, id);
    }
}

// ---- the chest ----

fn check_locked(w: &mut World, id: ItemId, from: MobileId) -> bool {
    if !cont::locked(w, id) {
        return false;
    }
    let solution = data(w, id)
        .and_then(|d| d.guesses.as_ref())
        .and_then(|g| g.get(&from))
        .map_or_else(|| vec![cylinder::NONE; LENGTH], |(c, _)| c.clone());
    display_puzzle(w, from, id, solution, 0);
    true
}

/// `new PuzzleGump(from, chest, solution)` and `from.SendGump(..)`.
fn display_puzzle(w: &mut World, from: MobileId, chest: ItemId, solution: Vec<i32>, check: usize) {
    let lockpicking = muo_server::skills::base(w, from, SkillName::Lockpicking);
    let d = data(w, chest).cloned().unwrap_or_default();
    let last_guess = d
        .guesses
        .as_ref()
        .and_then(|g| g.get(&from))
        .map(|(c, _)| c.clone());
    let g = PuzzleGump {
        chest,
        solution,
        check,
        lockpicking,
        last_guess,
        chest_solution: d.solution,
        hints: d.hints,
    };
    send_gump_to_mobile(w, from, Box::new(DynamicGumpObj::new(50, 50, g)), true);
}

/// `SubmitSolution(m, solution)`.
fn submit_solution(w: &mut World, id: ItemId, m: MobileId, solution: &[i32]) {
    let Some(d) = data(w, id) else { return };
    let (ok, cylinders, colors) = matches(solution, &d.solution);
    if ok {
        puzzle_lock_pick(w, id, m);
        if let Some(ty) = w.item(id).map(|i| i.ty) {
            (ty.display_to)(w, id, m); // DisplayTo(m)
        }
    } else {
        let now = w.now;
        let present = data(w, id)
            .and_then(|d| d.guesses.as_ref())
            .is_some_and(|g| g.contains_key(&m));
        if present {
            return; // `Dictionary.Add` of a present key fails
        }
        if let Some(d) = data_mut(w, id) {
            d.guesses
                .get_or_insert_with(IndexMap::new)
                .insert(m, (solution.to_vec(), now));
        }
        start_cleanup_timer(w, id);
        send_gump_to_mobile(
            w,
            m,
            Box::new(StaticGumpObj::new(50, 50, StatusGump { cylinders, colors })),
            true,
        );
        do_damage(w, m);
    }
}

/// `DoDamage(to)`.
fn do_damage(w: &mut World, to: MobileId) {
    let (loc, map) = w
        .mobile(to)
        .map_or((muo_primitives::Point3D::ZERO, None), |m| {
            (m.location, m.map)
        });
    let overhead = |w: &mut World, hue: i32, number: i32| {
        muo_server::item::deps::local_overhead_message(w, to, MessageType::REGULAR, hue, number);
    };
    let aos = |w: &mut World, split: DamageSplit| {
        let dmg = rnd::min_max(w, 10, 40);
        aos_helpers::damage(w, to, Some(to), dmg, split, DamageOptions::default());
    };
    match rnd::below(w, 4) {
        0 => {
            muo_server::effects::send_location_effect(w, loc, map, 0x113A, 20, 10, 0, 0);
            muo_content::mx::play_sound(w, to, 0x231);
            overhead(w, 0x44, 1010523); // A toxic vapor envelops thee.
            muo_server::mobile::base::apply_poison(w, to, Some(to), PoisonId(1));
        }
        1 => {
            muo_server::effects::send_location_effect(w, loc, map, 0x3709, 30, 10, 0, 0);
            muo_content::mx::play_sound(w, to, 0x54);
            overhead(w, 0xEE, 1010524); // Searing heat scorches thy skin.
            aos(
                w,
                DamageSplit {
                    phys: 0,
                    fire: 100,
                    cold: 0,
                    pois: 0,
                    nrgy: 0,
                },
            );
        }
        2 => {
            muo_content::mx::play_sound(w, to, 0x223);
            overhead(w, 0x62, 1010525); // Pain lances through thee from a sharp metal blade.
            aos(
                w,
                DamageSplit {
                    phys: 100,
                    fire: 0,
                    cold: 0,
                    pois: 0,
                    nrgy: 0,
                },
            );
        }
        _ => {
            muo_server::mobile::part2::effects::bolt_effect(w, to, 0);
            overhead(w, 0xDA, 1010526); // Lightning arcs through thy body.
            aos(
                w,
                DamageSplit {
                    phys: 0,
                    fire: 0,
                    cold: 0,
                    pois: 0,
                    nrgy: 100,
                },
            );
        }
    }
}

/// `GetRandomAOSStats(out attributeCount, out min, out max)`.
fn random_aos_stats(w: &mut World) -> (i32, i32, i32) {
    let r = rnd::below(w, 15);
    if r < 1 {
        (rnd::min_max(w, 2, 6), 20, 70)
    } else if r < 3 {
        (rnd::min_max(w, 2, 4), 20, 50)
    } else if r < 6 {
        (rnd::min_max(w, 2, 3), 20, 40)
    } else if r < 10 {
        (rnd::min_max(w, 1, 2), 10, 30)
    } else {
        (1, 10, 20)
    }
}

fn generate_treasure(w: &mut World, id: ItemId) {
    use muo_content::items::runic;
    let drop = |w: &mut World, item: ItemId| {
        if let Some(ty) = w.item(id).map(|i| i.ty) {
            (ty.drop_item)(w, id, item);
        }
    };
    ensure(w, id);
    let amount = rnd::min_max(w, 600, 900);
    let gold = muo_content::items::gold::new_gold(w, amount);
    drop(w, gold);

    let mut gems: Vec<ItemId> = Vec::new();
    for _ in 0..9 {
        let Some(gem) = muo_content::loot::random_gem(w) else {
            continue;
        };
        let gem_type = w.item(gem).map(|i| i.ty.full_name);
        let existing = gems
            .iter()
            .copied()
            .find(|g| w.item(*g).map(|i| i.ty.full_name) == gem_type);
        if let Some(g) = existing {
            let a = w.item(g).map_or(1, |i| i.amount);
            ops::set_amount(w, g, a + 1);
            if let Some(ty) = w.item(gem).map(|i| i.ty) {
                (ty.delete)(w, gem);
            }
        } else {
            gems.push(gem);
        }
    }
    for g in gems {
        drop(w, g);
    }

    if rnd::double(w) < 0.2
        && let Some(bag) = muo_content::factory::make_item(w, "Server.Items.BagOfReagents")
    {
        drop(w, bag);
    }

    let aos = muo_content::core::aos(w);
    for _ in 0..2 {
        let item = if aos {
            muo_content::loot::random_armor_or_shield_or_weapon_or_jewelry(w, false, false)
        } else {
            muo_content::loot::random_armor_or_shield_or_weapon(w, false, false)
        };
        let Some(item) = item else { continue };
        if weapon::is_weapon(w, item) {
            if aos {
                let (count, min, max) = random_aos_stats(w);
                runic::apply_to_weapon(w, item, false, 0, count, min, max);
            } else {
                let d = rnd::below(w, 6);
                weapon::set_damage_level(
                    w,
                    item,
                    weapon::WeaponDamageLevel::from_i32(d)
                        .unwrap_or(weapon::WeaponDamageLevel::Regular),
                );
                let a = rnd::below(w, 6);
                weapon::set_accuracy_level(
                    w,
                    item,
                    weapon::WeaponAccuracyLevel::from_i32(a)
                        .unwrap_or(weapon::WeaponAccuracyLevel::Regular),
                );
                let u = rnd::below(w, 6);
                weapon::set_durability_level(
                    w,
                    item,
                    weapon::WeaponDurabilityLevel::from_i32(u)
                        .unwrap_or(weapon::WeaponDurabilityLevel::Regular),
                );
            }
            drop(w, item);
        } else if armor::is_armor(w, item) {
            if aos {
                let (count, min, max) = random_aos_stats(w);
                runic::apply_to_armor(w, item, false, 0, count, min, max);
            } else {
                let p = rnd::below(w, 6);
                armor::set_protection_level(
                    w,
                    item,
                    armor::ArmorProtectionLevel::from_i32(p)
                        .unwrap_or(armor::ArmorProtectionLevel::Regular),
                );
                let u = rnd::below(w, 6);
                armor::set_durability_level(
                    w,
                    item,
                    armor::ArmorDurabilityLevel::from_i32(u)
                        .unwrap_or(armor::ArmorDurabilityLevel::Regular),
                );
            }
            drop(w, item);
        } else if w.item(item).is_some_and(|i| i.ty.is(&clothing::BASE_HAT)) {
            if aos {
                let (count, min, max) = random_aos_stats(w);
                runic::apply_to_hat(w, item, false, 0, count, min, max);
            }
            drop(w, item);
        } else if jewel::is_jewel(w, item) {
            let (count, min, max) = random_aos_stats(w);
            runic::apply_to_jewel(w, item, false, 0, count, min, max);
            drop(w, item);
        }
    }

    let s = random_solution(w);
    set_solution(w, id, s);
}

// ---- PuzzleGump ----

struct PuzzleGump {
    chest: ItemId,
    solution: Vec<i32>,
    check: usize,
    lockpicking: f64,
    last_guess: Option<Vec<i32>>,
    chest_solution: Vec<i32>,
    hints: Vec<i32>,
}

fn add_cylinder(b: &mut DynamicGumpBuilder, x: i32, y: i32, c: i32) {
    if c != cylinder::NONE {
        b.add_item(x, y, c, 0);
    } else {
        b.add_item(x + 9, y, c, 0);
    }
}

fn add_left_cylinder_button(b: &mut DynamicGumpBuilder, x: i32, y: i32, c: i32, button: i32) {
    b.add_background(x, y, 30, 30, 0x13EC);
    add_cylinder(b, x - 7, y + 10, c);
    b.add_button(x + 38, y + 9, 0x13A8, 0x4B9, button);
}

fn add_right_cylinder_button(b: &mut DynamicGumpBuilder, x: i32, y: i32, c: i32, button: i32) {
    b.add_background(x, y, 30, 30, 0x13EC);
    add_cylinder(b, x - 7, y + 10, c);
    b.add_button(x - 26, y + 9, 0x13A8, 0x4B9, button);
}

fn add_pedestal(b: &mut DynamicGumpBuilder, x: i32, y: i32, c: i32, switch_id: i32, initial: bool) {
    b.add_item(x, y, 0xB10, 0);
    b.add_item(x - 23, y + 12, 0xB12, 0);
    b.add_item(x + 23, y + 12, 0xB13, 0);
    b.add_item(x, y + 23, 0xB11, 0);
    if c != cylinder::NONE {
        b.add_item(x, y + 2, 0x51A, 0);
        add_cylinder(b, x - 1, y + 19, c);
    } else {
        b.add_item(x, y + 2, 0x521, 0);
    }
    b.add_radio(x + 7, y + 65, 0x867, 0x86A, initial, switch_id);
}

impl DynamicGumpLogic for PuzzleGump {
    const TYPE_NAME: &'static str = "Server.Items.PuzzleChest+PuzzleGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.set_no_dispose();
        b.add_background(25, 0, 500, 410, 0x53);
        b.add_image(62, 20, 0x67, 0);
        b.add_html_localized(80, 36, 110, 70, 1018309, true, false); // A Puzzle Lock
        // Correctly choose the sequence of cylinders needed to open the latch. ...
        b.add_html_localized(214, 26, 270, 90, 1018310, true, true);

        add_left_cylinder_button(b, 62, 130, cylinder::LIGHT_BLUE, 10);
        add_left_cylinder_button(b, 62, 180, cylinder::BLUE, 11);
        add_left_cylinder_button(b, 62, 230, cylinder::GREEN, 12);
        add_left_cylinder_button(b, 62, 280, cylinder::ORANGE, 13);
        add_right_cylinder_button(b, 451, 130, cylinder::PURPLE, 14);
        add_right_cylinder_button(b, 451, 180, cylinder::RED, 15);
        add_right_cylinder_button(b, 451, 230, cylinder::DARK_BLUE, 16);
        add_right_cylinder_button(b, 451, 280, cylinder::YELLOW, 17);

        let hint = |i: usize| self.hints.get(i).copied().unwrap_or(0);
        let lp = self.lockpicking;
        if lp >= 60.0 {
            b.add_html_localized(160, 125, 230, 24, 1018308, false, false); // Lockpicking hint:
            b.add_background(159, 150, 230, 95, 0x13EC);
            if lp >= 80.0 {
                b.add_html_localized(165, 157, 200, 40, 1018312, false, false); // In the first slot:
                add_cylinder(
                    b,
                    350,
                    165,
                    self.chest_solution.first().copied().unwrap_or(0),
                );
                b.add_html_localized(165, 197, 200, 40, 1018313, false, false); // Used in unknown slot:
                add_cylinder(b, 350, 200, hint(0));
                if lp >= 90.0 {
                    add_cylinder(b, 350, 212, hint(1));
                }
                if lp >= 100.0 {
                    add_cylinder(b, 350, 224, hint(2));
                }
            } else {
                b.add_html_localized(165, 157, 200, 40, 1018313, false, false);
                add_cylinder(b, 350, 160, hint(0));
                if lp >= 70.0 {
                    add_cylinder(b, 350, 172, hint(1));
                }
            }
        }

        if let Some(last) = &self.last_guess {
            b.add_html_localized(127, 249, 170, 20, 1018311, false, false); // Thy previous guess:
            b.add_background(290, 247, 115, 25, 0x13EC);
            for (i, x) in [281, 303, 325, 347, 369].into_iter().enumerate() {
                add_cylinder(b, x, 254, last.get(i).copied().unwrap_or(0));
            }
        }

        for (i, x) in [140, 195, 250, 305, 360].into_iter().enumerate() {
            add_pedestal(
                b,
                x,
                270,
                self.solution.get(i).copied().unwrap_or(0),
                i as i32,
                self.check == i,
            );
        }
        b.add_button(258, 370, 0xFA5, 0xFA7, 1);
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(from) = w.net_states.get(&sender).and_then(|n| n.mobile) else {
            return;
        };
        let chest = self.chest;
        if w.item(chest).is_none_or(|i| i.deleted())
            || info.button_id == 0
            || !muo_server::mobile::part2::stats::check_alive(w, from, true)
        {
            return;
        }
        let player = w
            .mobile(from)
            .is_some_and(|m| m.access_level == muo_server::mobile::AccessLevel::Player);
        if player {
            let same_map = w.mobile(from).map(|m| m.map) == w.item(chest).map(|i| i.map);
            let near =
                muo_content::mx::in_range_point(w, from, ops::get_world_location(w, chest), 2);
            if !same_map || !near {
                muo_server::item::deps::local_overhead_message(
                    w,
                    from,
                    MessageType::REGULAR,
                    0x3B2,
                    500446,
                );
                return;
            }
        }
        if info.button_id == 1 {
            let s = self.solution.clone();
            submit_solution(w, chest, from, &s);
            return;
        }
        let Some(&pedestal) = info.switches.first() else {
            return;
        };
        if pedestal < 0 || pedestal as usize >= self.solution.len() {
            return;
        }
        let c = match info.button_id {
            10 => cylinder::LIGHT_BLUE,
            11 => cylinder::BLUE,
            12 => cylinder::GREEN,
            13 => cylinder::ORANGE,
            14 => cylinder::PURPLE,
            15 => cylinder::RED,
            16 => cylinder::DARK_BLUE,
            17 => cylinder::YELLOW,
            _ => cylinder::NONE,
        };
        if c == cylinder::NONE {
            return;
        }
        self.solution[pedestal as usize] = c;
        display_puzzle(w, from, chest, self.solution.clone(), pedestal as usize);
    }
}

// ---- StatusGump ----

struct StatusGump {
    cylinders: i32,
    colors: i32,
}

impl StaticGumpLogic for StatusGump {
    const TYPE_NAME: &'static str = "Server.Items.PuzzleChest+StatusGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.add_background(15, 250, 305, 163, 0x53);
        b.add_background(28, 265, 280, 133, 0xBB8);
        b.add_html_localized(35, 271, 270, 24, 1018314, false, false); // Thou hast failed to solve the puzzle!
        b.add_html_localized(35, 297, 250, 24, 1018315, false, false); // Correctly placed colors:
        b.add_label_placeholder(285, 297, 0x44, "cylinders");
        b.add_html_localized(35, 323, 250, 24, 1018316, false, false); // Used colors in wrong slots:
        b.add_label_placeholder(285, 323, 0x44, "colors");
        b.add_button(152, 369, 0xFA5, 0xFA7, 0); // Close
    }

    fn build_strings(&self, s: &mut GumpStrings) {
        s.set_string_slot("cylinders", &format!("{}", self.cylinders));
        s.set_string_slot("colors", &format!("{}", self.colors));
    }
}

// ---- classes ----

/// `PuzzleChest.LockPick(from)`: the base, then stop the timer and forget the guesses.
fn puzzle_lock_pick(w: &mut World, id: ItemId, from: MobileId) {
    (LOCKABLE_CLASS.lock_pick)(w, id, from);
    stop_cleanup_timer(w, id);
    if let Some(d) = data_mut(w, id) {
        d.guesses = None;
    }
}

pub static PUZZLE_CHEST_CLASS: cont::ContClass = cont::ContClass {
    lock_pick: puzzle_lock_pick,
    check_locked,
    set_lock_level: |w, id| {
        if let Some(d) = cont::ext_mut(w, id) {
            d.lock_level = 0; // ILockpickable.CannotPick: can't be unlocked
        }
    },
    generate_treasure,
    ..TREASURE_CLASS
};

/// `Server.Items.PuzzleChest` (abstract), `[SerializationGenerator(1)]`.
pub static PUZZLE_CHEST: ItemType = ItemType {
    full_name: "Server.Items.PuzzleChest",
    parent: Some(&BASE_TREASURE_CHEST),
    construct: None,
    family: Some(&PUZZLE_CHEST_CLASS),
    default_name: |_, _| None,
    on_delete: |w, id| {
        stop_cleanup_timer(w, id);
        (BASE_TREASURE_CHEST.on_delete)(w, id);
    },
    serialize: |w, id, out| {
        (BASE_TREASURE_CHEST.serialize)(w, id, out);
        let d = data(w, id).cloned().unwrap_or_default();
        out.write_encoded_int(1);
        write_solution(&d.solution, out);
        out.write_encoded_int(d.hints.len() as i32);
        for h in &d.hints {
            out.write_enum(4, i64::from(*h));
        }
        match &d.guesses {
            Some(g) => {
                out.write_bool(true);
                // `Tidy()`: guesses of deleted players go.
                let live: Vec<_> = g
                    .iter()
                    .filter(|(m, _)| w.mobile(**m).is_some_and(|m| !m.deleted))
                    .collect();
                out.write_encoded_int(live.len() as i32);
                for (m, (c, when)) in live {
                    out.write_mobile(w, Some(*m));
                    out.write_bool(true);
                    write_solution(c, out);
                    out.write_encoded_int(1);
                    out.write_anchored_time_ticks(when.ticks());
                }
            }
            None => out.write_bool(false),
        }
    },
    deserialize: |w, id, r| {
        (BASE_TREASURE_CHEST.deserialize)(w, id, r)?;
        let version = r.read_encoded_int()?;
        ensure(w, id);
        let mut d = PuzzleData::default();
        if version < 1 {
            // `Deserialize(reader, version)`: hints are raw ints, the guesses are not nullable.
            d.solution = read_solution(w, r)?;
            let len = r.read_encoded_int()?.max(0) as usize;
            d.hints = vec![0; HINTS_COUNT];
            for i in 0..len {
                let c = r.read_i32()?;
                if len == HINTS_COUNT {
                    d.hints[i] = c;
                }
            }
            let count = r.read_encoded_int()?;
            if count > 0 {
                let mut g = IndexMap::new();
                for _ in 0..count {
                    let m = r.read_mobile(w)?;
                    let (c, when) = read_guess(w, r)?;
                    if let Some(m) = m {
                        g.insert(m, (c, when));
                    }
                }
                d.guesses = Some(g);
            }
        } else {
            d.solution = read_solution(w, r)?;
            let len = r.read_encoded_int()?.max(0) as usize;
            for _ in 0..len {
                d.hints.push(r.read_enum(4)? as i32);
            }
            if r.read_bool()? {
                let count = r.read_encoded_int()?;
                let mut g = IndexMap::new();
                for _ in 0..count {
                    let m = r.read_mobile(w)?;
                    let guess = if r.read_bool()? {
                        Some(read_guess(w, r)?)
                    } else {
                        None
                    };
                    if let (Some(m), Some(guess)) = (m, guess) {
                        g.insert(m, guess);
                    }
                }
                d.guesses = Some(g);
            }
        }
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(d);
        }
        // [AfterDeserialization]
        if data(w, id).is_some_and(|d| d.hints.len() != HINTS_COUNT) {
            init_hints(w, id);
        }
        start_cleanup_timer(w, id);
        Ok(())
    },
    ..BASE_TREASURE_CHEST
};

/// `new PuzzleChestSolutionAndTime(chest).Deserialize(reader)`.
fn read_guess(
    w: &mut World,
    r: &mut dyn muo_io::GenericReader,
) -> muo_io::Result<(Vec<i32>, DateTime)> {
    let c = read_solution(w, r)?;
    let version = r.read_encoded_int()?;
    let when = if version == 0 {
        r.read_delta_time_ticks(DateTime::utc_now().ticks())?
    } else {
        r.read_anchored_time_ticks()?
    };
    Ok((c, DateTime::from_ticks(when)))
}

macro_rules! puzzle_leaf {
    ($(#[$m:meta])* $name:ident = $full:literal, $gid:expr) => {
        $(#[$m])*
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&PUZZLE_CHEST),
            construct: Some(|w| {
                let id = new_treasure_chest(w, &$name, $gid, TreasureLevel::Level2);
                // `_hints = new PuzzleChestCylinder[HintsCount]` runs after `base(itemID)`.
                if let Some(d) = data_mut(w, id) {
                    d.hints = vec![0; HINTS_COUNT];
                }
                id
            }),
            serialize: |w, id, out| {
                (PUZZLE_CHEST.serialize)(w, id, out);
                out.write_encoded_int(0);
            },
            deserialize: |w, id, r| {
                (PUZZLE_CHEST.deserialize)(w, id, r)?;
                let _version = r.read_encoded_int()?;
                Ok(())
            },
            ..PUZZLE_CHEST
        };
    };
}

puzzle_leaf!(
    /// `Server.Items.MetalGoldenPuzzleChest` `[Flippable(0xE41, 0xE40)]`.
    METAL_GOLDEN_PUZZLE_CHEST = "Server.Items.MetalGoldenPuzzleChest", 0xE41
);
puzzle_leaf!(
    /// `Server.Items.StrongBoxPuzzle` `[Flippable(0xE80, 0x9A8)]`.
    STRONG_BOX_PUZZLE = "Server.Items.StrongBoxPuzzle", 0xE80
);

pub fn register(types: &mut TypeRegistry) {
    types.register_item(&PUZZLE_CHEST, &[]);
    types.register_item(&METAL_GOLDEN_PUZZLE_CHEST, &[]);
    types.register_item(&STRONG_BOX_PUZZLE, &[]);
}
