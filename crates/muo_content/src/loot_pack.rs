//! The loot tables (`LootPack`, `LootPackEntry`, `LootPackItem`, `LootPackDice`) and how
//! a creature's pack is rolled. The tables follow the code, in declaration order.

use muo_primitives::Point3D;
use muo_server::ids::{ItemId, MobileId};
use muo_server::mobile::part1::props_a;
use muo_server::world::World;

use crate::creature::data::cd;
use crate::factory::{item_is, make_item};
use crate::hooks::rnd;
use crate::loot::{self, SpellbookType};

/// `LootPackDice`: `Count`d`Sides` + `Bonus`.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct LootPackDice {
    pub count: i32,
    pub sides: i32,
    pub bonus: i32,
}

impl LootPackDice {
    pub const fn new(count: i32, sides: i32, bonus: i32) -> Self {
        Self {
            count,
            sides,
            bonus,
        }
    }

    /// `new LootPackDice("4d50+450")`; no `d` leaves everything 0.
    pub const fn parse(s: &str) -> Self {
        let b = s.as_bytes();
        let mut i = 0;
        while i < b.len() && b[i] != b'd' {
            i += 1;
        }
        if i >= b.len() {
            return Self::new(0, 0, 0);
        }
        let count = parse_int(b, 0, i);
        let start = i + 1;
        let mut j = start;
        let mut negative = true;
        while j < b.len() {
            if b[j] == b'+' {
                negative = false;
                break;
            }
            if b[j] == b'-' {
                break;
            }
            j += 1;
        }
        let sides = parse_int(b, start, j);
        if j >= b.len() {
            return Self::new(count, sides, 0);
        }
        let bonus = parse_int(b, j + 1, b.len());
        Self::new(count, sides, if negative { -bonus } else { bonus })
    }

    /// `Roll`.
    pub fn roll(&self, w: &mut World) -> i32 {
        let mut v = self.bonus;
        for _ in 0..self.count {
            v += rnd::from(w, 1, self.sides);
        }
        v
    }
}

const fn parse_int(b: &[u8], from: usize, to: usize) -> i32 {
    let mut v = 0i32;
    let mut i = from;
    while i < to {
        if b[i].is_ascii_digit() {
            v = v * 10 + (b[i] - b'0') as i32;
        }
        i += 1;
    }
    v
}

pub const fn dice(s: &str) -> LootPackDice {
    LootPackDice::parse(s)
}

/// `new LootPackDice(0, 0, quantity)`.
pub const fn qty(n: i32) -> LootPackDice {
    LootPackDice::new(0, 0, n)
}

/// `LootPackItem`: an item class (`Server.Items.<kind>`) and its weight.
#[derive(Copy, Clone, Debug)]
pub struct LootPackItem {
    pub kind: &'static str,
    pub chance: i32,
}

pub const fn item(kind: &'static str, chance: i32) -> LootPackItem {
    LootPackItem { kind, chance }
}

/// `LootPackEntry`.
#[derive(Copy, Clone, Debug)]
pub struct LootPackEntry {
    pub at_spawn_time: bool,
    pub items: &'static [LootPackItem],
    /// `(int)(100 * chance)`.
    pub chance: i32,
    pub quantity: LootPackDice,
    pub max_props: i32,
    pub min_intensity: i32,
    pub max_intensity: i32,
}

pub const fn entry(
    at_spawn: bool,
    items: &'static [LootPackItem],
    chance: f64,
    quantity: LootPackDice,
) -> LootPackEntry {
    entry_p(at_spawn, items, chance, quantity, 0, 0, 0)
}

pub const fn entry_p(
    at_spawn: bool,
    items: &'static [LootPackItem],
    chance: f64,
    quantity: LootPackDice,
    max_props: i32,
    min_intensity: i32,
    max_intensity: i32,
) -> LootPackEntry {
    LootPackEntry {
        at_spawn_time: at_spawn,
        items,
        chance: (100.0 * chance) as i32,
        quantity,
        max_props,
        min_intensity,
        max_intensity,
    }
}

/// `LootPack`.
pub struct LootPack {
    pub entries: &'static [LootPackEntry],
}

impl LootPack {
    pub const fn new(entries: &'static [LootPackEntry]) -> Self {
        Self { entries }
    }

    /// `Generate(from, cont, spawning, luckChance)`: rolls each entry into `cont`.
    pub fn generate(
        &'static self,
        w: &mut World,
        from: MobileId,
        cont: ItemId,
        spawning: bool,
        luck_chance: i32,
    ) {
        let mut check_luck = props_a::core_aos(w);
        for entry in self.entries {
            let mut should_add = entry.chance > rnd::below(w, 10000);
            if !should_add && check_luck {
                check_luck = false;
                if check_luck_chance(w, luck_chance) {
                    should_add = entry.chance > rnd::below(w, 10000);
                }
            }
            if !should_add {
                continue;
            }
            let Some(item) = entry.construct(w, from, luck_chance, spawning) else {
                continue;
            };
            let stackable = w.item(item).is_some_and(|i| i.stackable());
            if !stackable
                || !muo_server::item::container::try_drop_item(w, cont, Some(from), item, false)
            {
                muo_server::item::container::drop_item_default(w, cont, item);
            }
        }
    }
}

/// `CheckLuck(chance)`.
pub fn check_luck_chance(w: &mut World, chance: i32) -> bool {
    chance > rnd::below(w, 10000)
}

/// `GetLuckChance(killer, victim)`.
pub fn get_luck_chance(w: &mut World, killer: MobileId, victim: MobileId) -> i32 {
    if !props_a::core_aos(w) {
        return 0;
    }
    let mut luck = w.mobile(killer).map_or(0, |m| (m.ty.luck)(w, killer));
    luck += crate::player_mobile::hooks::honor_perfection_luck(w, killer, victim); // honoring this victim
    if luck < 0 {
        return 0;
    }
    if !w.core_se() && luck > 1200 {
        luck = 1200;
    }
    ((luck as f64).powf(1.0 / 1.8) * 100.0) as i32
}

/// `GetLuckChanceForKiller(dead)`: the luck of whoever did the most damage among those with looting rights.
pub fn get_luck_chance_for_killer(w: &mut World, dead: MobileId) -> i32 {
    let hits_max = w.mobile(dead).map_or(0, |m| (m.ty.hits_max)(w, dead));
    let list = crate::creature::death::get_looting_rights(w, dead, hits_max);
    let mut highest: Option<&crate::creature::death::DamageStore> = None;
    for ds in &list {
        if ds.has_right && highest.is_none_or(|h| ds.damage > h.damage) {
            highest = Some(ds);
        }
    }
    match highest {
        None => 0,
        Some(h) => get_luck_chance(w, h.mobile, dead),
    }
}

/// `MondainsLegacy.IsMLRegion(region)`.
pub fn is_ml_region(w: &World, region: muo_server::region::RegionId) -> bool {
    [
        "Twisted Weald",
        "Sanctuary",
        "The Prism of Light",
        "The Citadel",
        "Bedlam",
        "Blighted Grove",
        "The Painted Caves",
        "The Palace of Paroxysmus",
        "Labyrinth",
    ]
    .iter()
    .any(|n| muo_server::region::is_part_of_name(w, region, n, false))
}

/// Where a creature stands, for `LootPackEntry.IsInTokuno/IsMondain/IsStygian`.
fn flavor(w: &World, m: MobileId) -> (bool, bool, bool) {
    let Some(mob) = w.mobile(m) else {
        return (false, false, false);
    };
    let (map, region) = (mob.map, mob.region);
    let part_of = |name: &str| muo_server::region::is_part_of_name(w, region, name, false);
    let tokuno = part_of("Fan Dancer's Dojo")
        || part_of("Yomotsu Mines")
        || map == Some(muo_server::map::MapId::TOKUNO);
    let mondain = is_ml_region(w, region);
    let stygian = map == Some(muo_server::map::MapId::TER_MUR);
    (tokuno, mondain, stygian)
}

impl LootPackEntry {
    /// `Construct(from, luckChance, spawning)`.
    pub fn construct(
        &self,
        w: &mut World,
        from: MobileId,
        luck_chance: i32,
        spawning: bool,
    ) -> Option<ItemId> {
        if self.at_spawn_time != spawning {
            return None;
        }
        let total: i32 = self.items.iter().map(|i| i.chance).sum();
        let mut roll = rnd::below(w, total);
        for it in self.items {
            if roll < it.chance {
                let (tokuno, mondain, stygian) = flavor(w, from);
                let made = it.construct(w, tokuno, mondain, stygian);
                return self.mutate(w, from, luck_chance, made);
            }
            roll -= it.chance;
        }
        None
    }

    fn random_old_bonus(&self, w: &mut World) -> i32 {
        let mut r = rnd::min_max(w, self.min_intensity, self.max_intensity);
        if r < 50 {
            return 1;
        }
        r -= 50;
        if r < 25 {
            return 2;
        }
        r -= 25;
        if r < 14 {
            return 3;
        }
        r -= 14;
        if r < 8 {
            return 4;
        }
        5
    }

    /// `GetBonusProperties`.
    pub fn bonus_properties(&self, w: &mut World) -> i32 {
        let (mut p0, mut p1, mut p2, mut p3, mut p4, mut p5) = (0, 0, 0, 0, 0, 0);
        match self.max_props {
            1 => (p0, p1) = (3, 1),
            2 => (p0, p1, p2) = (6, 3, 1),
            3 => (p0, p1, p2, p3) = (10, 6, 3, 1),
            4 => (p0, p1, p2, p3, p4) = (16, 12, 6, 5, 1),
            5 => (p0, p1, p2, p3, p4, p5) = (30, 25, 20, 15, 9, 1),
            _ => {}
        }
        let pc = p0 + p1 + p2 + p3 + p4 + p5;
        let mut r = rnd::below(w, pc);
        if r < p5 {
            return 5;
        }
        r -= p5;
        if r < p4 {
            return 4;
        }
        r -= p4;
        if r < p3 {
            return 3;
        }
        r -= p3;
        if r < p2 {
            return 2;
        }
        if r - p2 < p1 { 1 } else { 0 }
    }

    /// `Mutate(from, luckChance, item)`.
    pub fn mutate(
        &self,
        w: &mut World,
        from: MobileId,
        luck_chance: i32,
        item: Option<ItemId>,
    ) -> Option<ItemId> {
        let item = item?;
        let hooks = w.resources.get::<LootHooks>().map(|h| {
            (
                h.apply_aos_attributes,
                h.apply_old_weapon,
                h.apply_old_armor,
                h.apply_instrument,
            )
        });
        let (apply_aos, apply_old_weapon, apply_old_armor, apply_instrument) =
            hooks.unwrap_or_default();
        let is_weapon = item_is(w, item, "Server.Items.BaseWeapon");
        if is_weapon && rnd::below(w, 100) < 1 {
            muo_server::item::base::delete(w, item);
            return make_item(w, "Server.Items.FireHorn");
        }
        let magic = is_weapon
            || item_is(w, item, "Server.Items.BaseArmor")
            || item_is(w, item, "Server.Items.BaseJewel")
            || item_is(w, item, "Server.Items.BaseHat");
        if magic {
            if props_a::core_aos(w) {
                let mut bonus = self.bonus_properties(w);
                if bonus < self.max_props && check_luck_chance(w, luck_chance) {
                    bonus += 1;
                }
                let mut props = 1 + bonus;
                // Make sure we're not spawning items with 6 properties.
                if props > self.max_props {
                    props = self.max_props;
                }
                if let Some(f) = apply_aos {
                    f(
                        w,
                        item,
                        luck_chance,
                        props,
                        self.min_intensity,
                        self.max_intensity,
                    );
                }
            } else if is_weapon {
                // The five rolls and their order are fixed (the random sequence depends on it); the item task
                // applies them.
                let acc = if rnd::below(w, 100) < 80 {
                    Some(self.random_old_bonus(w))
                } else {
                    None
                };
                let dmg = if rnd::below(w, 100) < 60 {
                    Some(self.random_old_bonus(w))
                } else {
                    None
                };
                let dur = if rnd::below(w, 100) < 40 {
                    Some(self.random_old_bonus(w))
                } else {
                    None
                };
                let silver = rnd::below(w, 100) < 5;
                // The levels are tested first: the roll is only drawn when no level was set.
                let loot_slayer = !silver
                    && acc.is_none()
                    && dmg.is_none()
                    && dur.is_none()
                    && rnd::below(w, 100) < 5;
                if let Some(f) = apply_old_weapon {
                    f(
                        w,
                        item,
                        OldWeaponRolls {
                            acc,
                            dmg,
                            dur,
                            silver,
                            loot_slayer,
                            from,
                        },
                    );
                }
            } else if item_is(w, item, "Server.Items.BaseArmor") {
                let prot = if rnd::below(w, 100) < 80 {
                    Some(self.random_old_bonus(w))
                } else {
                    None
                };
                let dur = if rnd::below(w, 100) < 40 {
                    Some(self.random_old_bonus(w))
                } else {
                    None
                };
                if let Some(f) = apply_old_armor {
                    f(w, item, prot, dur);
                }
            }
        } else if item_is(w, item, "Server.Items.BaseInstrument") {
            // The slayer roll (`BaseRunicTool.GetRandomSlayer` / `SlayerGroup.GetLootSlayerType`) is the hook set in
            // muo_items; with none the instrument is deleted, as for `SlayerName.None`.
            let ok = apply_instrument.is_some_and(|f| f(w, item, from));
            if !ok {
                muo_server::item::base::delete(w, item);
                return None;
            }
        }
        let stackable = w.item(item).is_some_and(|i| i.stackable());
        if stackable {
            // We set the amount, even if it can be negative so we get an error log
            let amount = self.quantity.roll(w);
            muo_server::item::ops::set_amount(w, item, amount);
            if amount <= 0 {
                muo_server::item::base::delete(w, item);
                return None;
            }
        }
        Some(item)
    }
}

/// The rolls of the pre-AOS weapon branch of `Mutate`.
#[derive(Copy, Clone, Debug)]
pub struct OldWeaponRolls {
    pub acc: Option<i32>,
    pub dmg: Option<i32>,
    pub dur: Option<i32>,
    /// `Slayer = SlayerName.Silver`.
    pub silver: bool,
    /// The 5% `SlayerGroup.GetLootSlayerType(from.GetType())` branch, valid only if no level or slayer was set.
    pub loot_slayer: bool,
    pub from: MobileId,
}

type ApplyAosAttributesFn = fn(&mut World, ItemId, i32, i32, i32, i32);
type ApplyOldArmorFn = fn(&mut World, ItemId, Option<i32>, Option<i32>);
/// The item-side writes `Mutate` makes (the items task sets these in its `register`).
#[derive(Default)]
pub struct LootHooks {
    /// `BaseRunicTool.ApplyAttributesTo(item, false, luckChance, props, min, max)` for weapon/armor/jewel/hat.
    pub apply_aos_attributes: Option<ApplyAosAttributesFn>,
    pub apply_old_weapon: Option<fn(&mut World, ItemId, OldWeaponRolls)>,
    pub apply_old_armor: Option<ApplyOldArmorFn>,
    /// Gives an instrument its slayer; false when `SlayerName.None`.
    pub apply_instrument: Option<fn(&mut World, ItemId, MobileId) -> bool>,
}

impl LootPackItem {
    /// `RandomScroll(index, minCircle, maxCircle)`.
    fn random_scroll(
        w: &mut World,
        index: usize,
        min_circle: i32,
        max_circle: i32,
    ) -> Option<ItemId> {
        const BLANK: &[&str] = &["Server.Items.BlankScroll"];
        let necro: [&[&str]; 3] = [
            &[
                "Server.Items.AnimateDeadScroll",
                "Server.Items.BloodOathScroll",
                "Server.Items.CorpseSkinScroll",
                "Server.Items.CurseWeaponScroll",
                "Server.Items.EvilOmenScroll",
                "Server.Items.HorrificBeastScroll",
                "Server.Items.MindRotScroll",
                "Server.Items.PainSpikeScroll",
                "Server.Items.SummonFamiliarScroll",
                "Server.Items.WraithFormScroll",
            ],
            &[
                "Server.Items.LichFormScroll",
                "Server.Items.PoisonStrikeScroll",
                "Server.Items.StrangleScroll",
                "Server.Items.WitherScroll",
            ],
            // High (the SE list adds Exorcism; picked once at startup from Core.SE)
            &[
                "Server.Items.VengefulSpiritScroll",
                "Server.Items.VampiricEmbraceScroll",
                "Server.Items.ExorcismScroll",
            ],
        ];
        let (min_circle, max_circle) = (min_circle - 1, max_circle - 1);
        let mut scroll_count = (max_circle - min_circle + 1) * 8;
        if index == 0 {
            scroll_count += BLANK.len() as i32;
        }
        let aos = w.core_aos();
        let necro_len = if index == 2 && !w.core_se() {
            2
        } else {
            necro[index].len()
        } as i32;
        if aos {
            scroll_count += necro_len;
        }
        let mut r = rnd::below(w, scroll_count);
        if index == 0 && r < BLANK.len() as i32 {
            return loot::construct_any(w, BLANK);
        }
        if index == 0 {
            r -= BLANK.len() as i32;
        }
        // `rnd < m_NecroTypes.Length` compares with the number of lists (3), not their length.
        if aos && r < necro.len() as i32 {
            let list = &necro[index][..necro_len as usize];
            return loot::construct_any(w, list);
        }
        loot::random_scroll(
            w,
            min_circle * 8,
            max_circle * 8 + 7,
            SpellbookType::Regular,
        )
    }

    /// `Construct(inTokuno, isMondain, isStygian)`.
    pub fn construct(
        &self,
        w: &mut World,
        in_tokuno: bool,
        is_mondain: bool,
        is_stygian: bool,
    ) -> Option<ItemId> {
        match self.kind {
            "BaseRanged" => loot::random_ranged_weapon(w, in_tokuno, is_mondain, is_stygian),
            "BaseWeapon" => loot::random_weapon(w, in_tokuno, is_mondain, is_stygian),
            "BaseArmor" => loot::random_armor_or_hat(w, in_tokuno, is_mondain),
            "BaseShield" => loot::random_shield(w),
            "BaseJewel" => {
                if w.core_aos() {
                    loot::random_jewelry(w)
                } else {
                    loot::random_armor_or_shield_or_weapon(w, false, false)
                }
            }
            "BaseInstrument" => loot::random_instrument(w),
            "Amber" => loot::random_gem(w),
            "ClumsyScroll" => Self::random_scroll(w, 0, 1, 3),
            "ArchCureScroll" => Self::random_scroll(w, 1, 4, 7),
            "SummonAirElementalScroll" => Self::random_scroll(w, 2, 8, 8),
            other => make_item(w, &format!("Server.Items.{other}")),
        }
    }
}

// ---- era-dependent selectors --------------------------------------------------------------------------------

macro_rules! era_pack {
    ($(#[$m:meta])* $name:ident, $se:ident, $aos:ident, $old:ident) => {
        $(#[$m])*
        pub fn $name(w: &World) -> &'static LootPack {
            if w.core_se() { &$se } else if w.core_aos() { &$aos } else { &$old }
        }
    };
}

era_pack!(poor, SE_POOR, AOS_POOR, OLD_POOR);
era_pack!(meager, SE_MEAGER, AOS_MEAGER, OLD_MEAGER);
era_pack!(average, SE_AVERAGE, AOS_AVERAGE, OLD_AVERAGE);
era_pack!(rich, SE_RICH, AOS_RICH, OLD_RICH);
era_pack!(
    filthy_rich,
    SE_FILTHY_RICH,
    AOS_FILTHY_RICH,
    OLD_FILTHY_RICH
);
era_pack!(ultra_rich, SE_ULTRA_RICH, AOS_ULTRA_RICH, OLD_ULTRA_RICH);
era_pack!(super_boss, SE_SUPER_BOSS, AOS_SUPER_BOSS, OLD_SUPER_BOSS);

#[allow(dead_code)]
fn _unused(w: &World, m: MobileId, _: Point3D) -> bool {
    cd(w, m).is_some()
}

// ---- generated tables ---------------------------------------------------------------------------------------

pub static GOLD: &[LootPackItem] = &[item("Gold", 1)];
pub static INSTRUMENTS: &[LootPackItem] = &[item("BaseInstrument", 1)];
pub static LOW_SCROLL_ITEMS: &[LootPackItem] = &[item("ClumsyScroll", 1)];
pub static MED_SCROLL_ITEMS: &[LootPackItem] = &[item("ArchCureScroll", 1)];
pub static HIGH_SCROLL_ITEMS: &[LootPackItem] = &[item("SummonAirElementalScroll", 1)];
pub static GEM_ITEMS: &[LootPackItem] = &[item("Amber", 1)];
pub static POTION_ITEMS: &[LootPackItem] = &[
    item("AgilityPotion", 1),
    item("StrengthPotion", 1),
    item("RefreshPotion", 1),
    item("LesserCurePotion", 1),
    item("LesserHealPotion", 1),
    item("LesserPoisonPotion", 1),
];
pub static OLD_MAGIC_ITEMS: &[LootPackItem] = &[
    item("BaseJewel", 1),
    item("BaseArmor", 4),
    item("BaseWeapon", 3),
    item("BaseRanged", 1),
    item("BaseShield", 1),
];
pub static AOS_MAGIC_ITEMS_RICH_TYPE1: &[LootPackItem] = &[
    item("BaseWeapon", 211),
    item("BaseRanged", 53),
    item("BaseArmor", 303),
    item("BaseShield", 39),
    item("BaseJewel", 158),
];
pub static AOS_MAGIC_ITEMS_POOR: &[LootPackItem] = &[
    item("BaseWeapon", 3),
    item("BaseRanged", 1),
    item("BaseArmor", 4),
    item("BaseShield", 1),
    item("BaseJewel", 2),
];
pub static AOS_MAGIC_ITEMS_MEAGER_TYPE1: &[LootPackItem] = &[
    item("BaseWeapon", 56),
    item("BaseRanged", 14),
    item("BaseArmor", 81),
    item("BaseShield", 11),
    item("BaseJewel", 42),
];
pub static AOS_MAGIC_ITEMS_MEAGER_TYPE2: &[LootPackItem] = &[
    item("BaseWeapon", 28),
    item("BaseRanged", 7),
    item("BaseArmor", 40),
    item("BaseShield", 5),
    item("BaseJewel", 21),
];
pub static AOS_MAGIC_ITEMS_AVERAGE_TYPE1: &[LootPackItem] = &[
    item("BaseWeapon", 90),
    item("BaseRanged", 23),
    item("BaseArmor", 130),
    item("BaseShield", 17),
    item("BaseJewel", 68),
];
pub static AOS_MAGIC_ITEMS_AVERAGE_TYPE2: &[LootPackItem] = &[
    item("BaseWeapon", 54),
    item("BaseRanged", 13),
    item("BaseArmor", 77),
    item("BaseShield", 10),
    item("BaseJewel", 40),
];
pub static AOS_MAGIC_ITEMS_RICH_TYPE2: &[LootPackItem] = &[
    item("BaseWeapon", 170),
    item("BaseRanged", 43),
    item("BaseArmor", 245),
    item("BaseShield", 32),
    item("BaseJewel", 128),
];
pub static AOS_MAGIC_ITEMS_FILTHY_RICH_TYPE1: &[LootPackItem] = &[
    item("BaseWeapon", 219),
    item("BaseRanged", 55),
    item("BaseArmor", 315),
    item("BaseShield", 41),
    item("BaseJewel", 164),
];
pub static AOS_MAGIC_ITEMS_FILTHY_RICH_TYPE2: &[LootPackItem] = &[
    item("BaseWeapon", 239),
    item("BaseRanged", 60),
    item("BaseArmor", 343),
    item("BaseShield", 90),
    item("BaseJewel", 45),
];
pub static AOS_MAGIC_ITEMS_ULTRA_RICH: &[LootPackItem] = &[
    item("BaseWeapon", 276),
    item("BaseRanged", 69),
    item("BaseArmor", 397),
    item("BaseShield", 52),
    item("BaseJewel", 207),
];
pub static LOW_SCROLLS: LootPack = LootPack::new(&[entry(false, LOW_SCROLL_ITEMS, 100.00, qty(1))]);
pub static MED_SCROLLS: LootPack = LootPack::new(&[entry(false, MED_SCROLL_ITEMS, 100.00, qty(1))]);
pub static HIGH_SCROLLS: LootPack =
    LootPack::new(&[entry(false, HIGH_SCROLL_ITEMS, 100.00, qty(1))]);
pub static GEMS: LootPack = LootPack::new(&[entry(false, GEM_ITEMS, 100.00, qty(1))]);
pub static POTIONS: LootPack = LootPack::new(&[entry(false, POTION_ITEMS, 100.00, qty(1))]);
pub static ML_RICH: LootPack = LootPack::new(&[
    entry(true, GOLD, 100.00, dice("4d50+450")),
    entry_p(false, AOS_MAGIC_ITEMS_RICH_TYPE1, 100.00, qty(1), 3, 0, 75),
    entry_p(false, AOS_MAGIC_ITEMS_RICH_TYPE1, 80.00, qty(1), 3, 0, 75),
    entry_p(false, AOS_MAGIC_ITEMS_RICH_TYPE1, 60.00, qty(1), 5, 0, 100),
    entry(false, INSTRUMENTS, 1.00, qty(1)),
]);
pub static SE_POOR: LootPack = LootPack::new(&[
    entry(true, GOLD, 100.00, dice("2d10+20")),
    entry_p(false, AOS_MAGIC_ITEMS_POOR, 1.00, qty(1), 5, 0, 100),
    entry(false, INSTRUMENTS, 0.02, qty(1)),
]);
pub static SE_MEAGER: LootPack = LootPack::new(&[
    entry(true, GOLD, 100.00, dice("4d10+40")),
    entry_p(false, AOS_MAGIC_ITEMS_MEAGER_TYPE1, 20.40, qty(1), 2, 0, 50),
    entry_p(
        false,
        AOS_MAGIC_ITEMS_MEAGER_TYPE2,
        10.20,
        qty(1),
        5,
        0,
        100,
    ),
    entry(false, INSTRUMENTS, 0.10, qty(1)),
]);
pub static SE_AVERAGE: LootPack = LootPack::new(&[
    entry(true, GOLD, 100.00, dice("8d10+100")),
    entry_p(
        false,
        AOS_MAGIC_ITEMS_AVERAGE_TYPE1,
        32.80,
        qty(1),
        3,
        0,
        50,
    ),
    entry_p(
        false,
        AOS_MAGIC_ITEMS_AVERAGE_TYPE1,
        32.80,
        qty(1),
        4,
        0,
        75,
    ),
    entry_p(
        false,
        AOS_MAGIC_ITEMS_AVERAGE_TYPE2,
        19.50,
        qty(1),
        5,
        0,
        100,
    ),
    entry(false, INSTRUMENTS, 0.40, qty(1)),
]);
pub static SE_RICH: LootPack = LootPack::new(&[
    entry(true, GOLD, 100.00, dice("15d10+225")),
    entry_p(false, AOS_MAGIC_ITEMS_RICH_TYPE1, 76.30, qty(1), 4, 0, 75),
    entry_p(false, AOS_MAGIC_ITEMS_RICH_TYPE1, 76.30, qty(1), 4, 0, 75),
    entry_p(false, AOS_MAGIC_ITEMS_RICH_TYPE2, 61.70, qty(1), 5, 0, 100),
    entry(false, INSTRUMENTS, 1.00, qty(1)),
]);
pub static SE_FILTHY_RICH: LootPack = LootPack::new(&[
    entry(true, GOLD, 100.00, dice("3d100+400")),
    entry_p(
        false,
        AOS_MAGIC_ITEMS_FILTHY_RICH_TYPE1,
        79.50,
        qty(1),
        5,
        0,
        100,
    ),
    entry_p(
        false,
        AOS_MAGIC_ITEMS_FILTHY_RICH_TYPE1,
        79.50,
        qty(1),
        5,
        0,
        100,
    ),
    entry_p(
        false,
        AOS_MAGIC_ITEMS_FILTHY_RICH_TYPE2,
        77.60,
        qty(1),
        5,
        25,
        100,
    ),
    entry(false, INSTRUMENTS, 2.00, qty(1)),
]);
pub static SE_ULTRA_RICH: LootPack = LootPack::new(&[
    entry(true, GOLD, 100.00, dice("6d100+600")),
    entry_p(
        false,
        AOS_MAGIC_ITEMS_ULTRA_RICH,
        100.00,
        qty(1),
        5,
        25,
        100,
    ),
    entry_p(
        false,
        AOS_MAGIC_ITEMS_ULTRA_RICH,
        100.00,
        qty(1),
        5,
        25,
        100,
    ),
    entry_p(
        false,
        AOS_MAGIC_ITEMS_ULTRA_RICH,
        100.00,
        qty(1),
        5,
        25,
        100,
    ),
    entry_p(
        false,
        AOS_MAGIC_ITEMS_ULTRA_RICH,
        100.00,
        qty(1),
        5,
        25,
        100,
    ),
    entry_p(
        false,
        AOS_MAGIC_ITEMS_ULTRA_RICH,
        100.00,
        qty(1),
        5,
        25,
        100,
    ),
    entry_p(
        false,
        AOS_MAGIC_ITEMS_ULTRA_RICH,
        100.00,
        qty(1),
        5,
        33,
        100,
    ),
    entry(false, INSTRUMENTS, 2.00, qty(1)),
]);
pub static SE_SUPER_BOSS: LootPack = LootPack::new(&[
    entry(true, GOLD, 100.00, dice("10d100+800")),
    entry_p(
        false,
        AOS_MAGIC_ITEMS_ULTRA_RICH,
        100.00,
        qty(1),
        5,
        25,
        100,
    ),
    entry_p(
        false,
        AOS_MAGIC_ITEMS_ULTRA_RICH,
        100.00,
        qty(1),
        5,
        25,
        100,
    ),
    entry_p(
        false,
        AOS_MAGIC_ITEMS_ULTRA_RICH,
        100.00,
        qty(1),
        5,
        25,
        100,
    ),
    entry_p(
        false,
        AOS_MAGIC_ITEMS_ULTRA_RICH,
        100.00,
        qty(1),
        5,
        25,
        100,
    ),
    entry_p(
        false,
        AOS_MAGIC_ITEMS_ULTRA_RICH,
        100.00,
        qty(1),
        5,
        33,
        100,
    ),
    entry_p(
        false,
        AOS_MAGIC_ITEMS_ULTRA_RICH,
        100.00,
        qty(1),
        5,
        33,
        100,
    ),
    entry_p(
        false,
        AOS_MAGIC_ITEMS_ULTRA_RICH,
        100.00,
        qty(1),
        5,
        33,
        100,
    ),
    entry_p(
        false,
        AOS_MAGIC_ITEMS_ULTRA_RICH,
        100.00,
        qty(1),
        5,
        33,
        100,
    ),
    entry_p(
        false,
        AOS_MAGIC_ITEMS_ULTRA_RICH,
        100.00,
        qty(1),
        5,
        50,
        100,
    ),
    entry_p(
        false,
        AOS_MAGIC_ITEMS_ULTRA_RICH,
        100.00,
        qty(1),
        5,
        50,
        100,
    ),
    entry(false, INSTRUMENTS, 2.00, qty(1)),
]);
pub static AOS_POOR: LootPack = LootPack::new(&[
    entry(true, GOLD, 100.00, dice("1d10+10")),
    entry_p(false, AOS_MAGIC_ITEMS_POOR, 0.02, qty(1), 5, 0, 90),
    entry(false, INSTRUMENTS, 0.02, qty(1)),
]);
pub static AOS_MEAGER: LootPack = LootPack::new(&[
    entry(true, GOLD, 100.00, dice("3d10+20")),
    entry_p(false, AOS_MAGIC_ITEMS_MEAGER_TYPE1, 1.00, qty(1), 2, 0, 10),
    entry_p(false, AOS_MAGIC_ITEMS_MEAGER_TYPE2, 0.20, qty(1), 5, 0, 90),
    entry(false, INSTRUMENTS, 0.10, qty(1)),
]);
pub static AOS_AVERAGE: LootPack = LootPack::new(&[
    entry(true, GOLD, 100.00, dice("5d10+50")),
    entry_p(false, AOS_MAGIC_ITEMS_AVERAGE_TYPE1, 5.00, qty(1), 4, 0, 20),
    entry_p(false, AOS_MAGIC_ITEMS_AVERAGE_TYPE1, 2.00, qty(1), 3, 0, 50),
    entry_p(false, AOS_MAGIC_ITEMS_AVERAGE_TYPE2, 0.50, qty(1), 5, 0, 90),
    entry(false, INSTRUMENTS, 0.40, qty(1)),
]);
pub static AOS_RICH: LootPack = LootPack::new(&[
    entry(true, GOLD, 100.00, dice("10d10+150")),
    entry_p(false, AOS_MAGIC_ITEMS_RICH_TYPE1, 20.00, qty(1), 4, 0, 40),
    entry_p(false, AOS_MAGIC_ITEMS_RICH_TYPE1, 10.00, qty(1), 5, 0, 60),
    entry_p(false, AOS_MAGIC_ITEMS_RICH_TYPE2, 1.00, qty(1), 5, 0, 90),
    entry(false, INSTRUMENTS, 1.00, qty(1)),
]);
pub static AOS_FILTHY_RICH: LootPack = LootPack::new(&[
    entry(true, GOLD, 100.00, dice("2d100+200")),
    entry_p(
        false,
        AOS_MAGIC_ITEMS_FILTHY_RICH_TYPE1,
        33.00,
        qty(1),
        4,
        0,
        50,
    ),
    entry_p(
        false,
        AOS_MAGIC_ITEMS_FILTHY_RICH_TYPE1,
        33.00,
        qty(1),
        4,
        0,
        60,
    ),
    entry_p(
        false,
        AOS_MAGIC_ITEMS_FILTHY_RICH_TYPE2,
        20.00,
        qty(1),
        5,
        0,
        75,
    ),
    entry_p(
        false,
        AOS_MAGIC_ITEMS_FILTHY_RICH_TYPE2,
        5.00,
        qty(1),
        5,
        0,
        100,
    ),
    entry(false, INSTRUMENTS, 2.00, qty(1)),
]);
pub static AOS_ULTRA_RICH: LootPack = LootPack::new(&[
    entry(true, GOLD, 100.00, dice("5d100+500")),
    entry_p(
        false,
        AOS_MAGIC_ITEMS_ULTRA_RICH,
        100.00,
        qty(1),
        5,
        25,
        100,
    ),
    entry_p(
        false,
        AOS_MAGIC_ITEMS_ULTRA_RICH,
        100.00,
        qty(1),
        5,
        25,
        100,
    ),
    entry_p(
        false,
        AOS_MAGIC_ITEMS_ULTRA_RICH,
        100.00,
        qty(1),
        5,
        25,
        100,
    ),
    entry_p(
        false,
        AOS_MAGIC_ITEMS_ULTRA_RICH,
        100.00,
        qty(1),
        5,
        25,
        100,
    ),
    entry_p(
        false,
        AOS_MAGIC_ITEMS_ULTRA_RICH,
        100.00,
        qty(1),
        5,
        25,
        100,
    ),
    entry_p(
        false,
        AOS_MAGIC_ITEMS_ULTRA_RICH,
        100.00,
        qty(1),
        5,
        35,
        100,
    ),
    entry(false, INSTRUMENTS, 2.00, qty(1)),
]);
pub static AOS_SUPER_BOSS: LootPack = LootPack::new(&[
    entry(true, GOLD, 100.00, dice("5d100+500")),
    entry_p(
        false,
        AOS_MAGIC_ITEMS_ULTRA_RICH,
        100.00,
        qty(1),
        5,
        25,
        100,
    ),
    entry_p(
        false,
        AOS_MAGIC_ITEMS_ULTRA_RICH,
        100.00,
        qty(1),
        5,
        25,
        100,
    ),
    entry_p(
        false,
        AOS_MAGIC_ITEMS_ULTRA_RICH,
        100.00,
        qty(1),
        5,
        25,
        100,
    ),
    entry_p(
        false,
        AOS_MAGIC_ITEMS_ULTRA_RICH,
        100.00,
        qty(1),
        5,
        25,
        100,
    ),
    entry_p(
        false,
        AOS_MAGIC_ITEMS_ULTRA_RICH,
        100.00,
        qty(1),
        5,
        33,
        100,
    ),
    entry_p(
        false,
        AOS_MAGIC_ITEMS_ULTRA_RICH,
        100.00,
        qty(1),
        5,
        33,
        100,
    ),
    entry_p(
        false,
        AOS_MAGIC_ITEMS_ULTRA_RICH,
        100.00,
        qty(1),
        5,
        33,
        100,
    ),
    entry_p(
        false,
        AOS_MAGIC_ITEMS_ULTRA_RICH,
        100.00,
        qty(1),
        5,
        33,
        100,
    ),
    entry_p(
        false,
        AOS_MAGIC_ITEMS_ULTRA_RICH,
        100.00,
        qty(1),
        5,
        50,
        100,
    ),
    entry_p(
        false,
        AOS_MAGIC_ITEMS_ULTRA_RICH,
        100.00,
        qty(1),
        5,
        50,
        100,
    ),
    entry(false, INSTRUMENTS, 2.00, qty(1)),
]);
pub static OLD_POOR: LootPack = LootPack::new(&[
    entry(true, GOLD, 100.00, dice("1d25")),
    entry(false, INSTRUMENTS, 0.02, qty(1)),
]);
pub static OLD_MEAGER: LootPack = LootPack::new(&[
    entry(true, GOLD, 100.00, dice("5d10+25")),
    entry(false, INSTRUMENTS, 0.10, qty(1)),
    entry_p(false, OLD_MAGIC_ITEMS, 1.00, qty(1), 1, 0, 60),
    entry_p(false, OLD_MAGIC_ITEMS, 0.20, qty(1), 1, 10, 70),
]);
pub static OLD_AVERAGE: LootPack = LootPack::new(&[
    entry(true, GOLD, 100.00, dice("10d10+50")),
    entry(false, INSTRUMENTS, 0.40, qty(1)),
    entry_p(false, OLD_MAGIC_ITEMS, 5.00, qty(1), 1, 20, 80),
    entry_p(false, OLD_MAGIC_ITEMS, 2.00, qty(1), 1, 30, 90),
    entry_p(false, OLD_MAGIC_ITEMS, 0.50, qty(1), 1, 40, 100),
]);
pub static OLD_RICH: LootPack = LootPack::new(&[
    entry(true, GOLD, 100.00, dice("10d10+250")),
    entry(false, INSTRUMENTS, 1.00, qty(1)),
    entry_p(false, OLD_MAGIC_ITEMS, 20.00, qty(1), 1, 60, 100),
    entry_p(false, OLD_MAGIC_ITEMS, 10.00, qty(1), 1, 65, 100),
    entry_p(false, OLD_MAGIC_ITEMS, 1.00, qty(1), 1, 70, 100),
]);
pub static OLD_FILTHY_RICH: LootPack = LootPack::new(&[
    entry(true, GOLD, 100.00, dice("2d125+400")),
    entry(false, INSTRUMENTS, 2.00, qty(1)),
    entry_p(false, OLD_MAGIC_ITEMS, 33.00, qty(1), 1, 50, 100),
    entry_p(false, OLD_MAGIC_ITEMS, 33.00, qty(1), 1, 60, 100),
    entry_p(false, OLD_MAGIC_ITEMS, 20.00, qty(1), 1, 70, 100),
    entry_p(false, OLD_MAGIC_ITEMS, 5.00, qty(1), 1, 80, 100),
]);
pub static OLD_ULTRA_RICH: LootPack = LootPack::new(&[
    entry(true, GOLD, 100.00, dice("5d100+500")),
    entry(false, INSTRUMENTS, 2.00, qty(1)),
    entry_p(false, OLD_MAGIC_ITEMS, 100.00, qty(1), 1, 40, 100),
    entry_p(false, OLD_MAGIC_ITEMS, 100.00, qty(1), 1, 40, 100),
    entry_p(false, OLD_MAGIC_ITEMS, 100.00, qty(1), 1, 50, 100),
    entry_p(false, OLD_MAGIC_ITEMS, 100.00, qty(1), 1, 50, 100),
    entry_p(false, OLD_MAGIC_ITEMS, 100.00, qty(1), 1, 60, 100),
    entry_p(false, OLD_MAGIC_ITEMS, 100.00, qty(1), 1, 60, 100),
]);
pub static OLD_SUPER_BOSS: LootPack = LootPack::new(&[
    entry(true, GOLD, 100.00, dice("5d100+500")),
    entry(false, INSTRUMENTS, 2.00, qty(1)),
    entry_p(false, OLD_MAGIC_ITEMS, 100.00, qty(1), 1, 40, 100),
    entry_p(false, OLD_MAGIC_ITEMS, 100.00, qty(1), 1, 40, 100),
    entry_p(false, OLD_MAGIC_ITEMS, 100.00, qty(1), 1, 40, 100),
    entry_p(false, OLD_MAGIC_ITEMS, 100.00, qty(1), 1, 50, 100),
    entry_p(false, OLD_MAGIC_ITEMS, 100.00, qty(1), 1, 50, 100),
    entry_p(false, OLD_MAGIC_ITEMS, 100.00, qty(1), 1, 50, 100),
    entry_p(false, OLD_MAGIC_ITEMS, 100.00, qty(1), 1, 60, 100),
    entry_p(false, OLD_MAGIC_ITEMS, 100.00, qty(1), 1, 60, 100),
    entry_p(false, OLD_MAGIC_ITEMS, 100.00, qty(1), 1, 60, 100),
    entry_p(false, OLD_MAGIC_ITEMS, 100.00, qty(1), 1, 70, 100),
]);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dice_parse_from_their_text_form() {
        assert_eq!(dice("4d50+450"), LootPackDice::new(4, 50, 450));
        assert_eq!(dice("1d25"), LootPackDice::new(1, 25, 0));
        assert_eq!(dice("2d10-5"), LootPackDice::new(2, 10, -5));
        assert_eq!(dice("15d10+225"), LootPackDice::new(15, 10, 225));
        assert_eq!(qty(3), LootPackDice::new(0, 0, 3));
    }

    #[test]
    fn dice_roll_stays_in_range() {
        let mut w = World::new();
        let d = dice("2d10+20");
        for _ in 0..200 {
            let r = d.roll(&mut w);
            assert!((22..=40).contains(&r), "{r}");
        }
        assert_eq!(qty(7).roll(&mut w), 7);
    }

    #[test]
    fn entry_chance_is_hundredths_of_a_percent() {
        assert_eq!(entry(false, GOLD, 100.00, qty(1)).chance, 10000);
        assert_eq!(entry(false, INSTRUMENTS, 0.02, qty(1)).chance, 2);
        assert_eq!(
            entry(false, INSTRUMENTS, 32.80, qty(1)).chance,
            3279,
            "(int)(100 * 32.8) truncates"
        );
    }

    #[test]
    fn tables_have_the_expected_entries() {
        // SeRich: Gold at spawn time "15d10+225" is entry 0.
        assert!(SE_RICH.entries[0].at_spawn_time);
        assert_eq!(SE_RICH.entries[0].quantity, LootPackDice::new(15, 10, 225));
        assert_eq!(OLD_SUPER_BOSS.entries.len(), 12);
        assert_eq!(
            AOS_MAGIC_ITEMS_ULTRA_RICH
                .iter()
                .map(|i| i.chance)
                .sum::<i32>(),
            276 + 69 + 397 + 52 + 207
        );
        assert_eq!(ML_RICH.entries[2].max_props, 3);
        assert_eq!(
            (
                ML_RICH.entries[3].min_intensity,
                ML_RICH.entries[3].max_intensity
            ),
            (0, 100)
        );
        assert_eq!(POTION_ITEMS.len(), 6);
    }

    #[test]
    fn luck_chance_formula() {
        let mut w = World::new();
        let id = w.new_mobile(&muo_server::mobile::MOBILE_BASE);
        // Mobile.Luck is 0: (int)(0^(1/1.8) * 100) = 0, and pre-AOS is 0 regardless.
        assert_eq!(get_luck_chance(&mut w, id, id), 0);
        assert!(!check_luck_chance(&mut w, 0));
        assert!(check_luck_chance(&mut w, 10001));
    }

    #[test]
    fn bonus_properties_follow_max_props_weights() {
        let mut w = World::new();
        let e = entry_p(false, GOLD, 100.0, qty(1), 1, 0, 0);
        for _ in 0..100 {
            assert!((0..=1).contains(&e.bonus_properties(&mut w)));
        }
        let e = entry_p(false, GOLD, 100.0, qty(1), 0, 0, 0);
        // MaxProps 0: no weights, `Utility.Random(0)` gives 0 and the result is 0.
        assert_eq!(e.bonus_properties(&mut w), 0);
    }
}
