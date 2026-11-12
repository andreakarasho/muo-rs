//! Bulk order rewards (the reward groups, point and gold tables of `SmithRewardCalculator` and
//! `TailorRewardCalculator`) and of the bulk order half of `BaseVendor.OnDragDrop`.
//!
//! A reward whose constructor would fail (a runic hammer, sewing kit, mining gloves or ancient smithy hammer
//! with a type out of range) constructs as null, as `Construct()` does; nothing is handed over for it.

use muo_content::hooks::rnd;
use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::skills::SkillName;

use super::gold_tables::{SMITH_GOLD, TAILOR_AOS_GOLD, TAILOR_OLD_GOLD};
use super::{data, is_large, is_tailor, mat};

/// `ConstructCallback`s of the smith and tailor calculators.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Reward {
    SturdyShovel,
    SturdyPickaxe,
    MiningGloves,
    GargoylesPickaxe,
    ProspectorsTool,
    PowderOfTemperament,
    RunicHammer,
    SmithPowerScroll,
    ColoredAnvil,
    AncientHammer,
    Cloth,
    Sandals,
    StretchedHide,
    RunicKit,
    Tapestry,
    TailorPowerScroll,
    BearRug,
    ClothingBlessDeed,
}

/// `RewardItem`.
#[derive(Copy, Clone, Debug)]
pub struct RewardItem {
    pub weight: i32,
    pub reward: Reward,
    pub ty: i32,
}

const fn ri(weight: i32, reward: Reward, ty: i32) -> RewardItem {
    RewardItem { weight, reward, ty }
}

/// `RewardGroup`.
pub struct RewardGroup {
    pub points: i32,
    pub items: &'static [RewardItem],
}

use Reward::*;

pub static SMITH_GROUPS: &[RewardGroup] = &[
    RewardGroup {
        points: 0,
        items: &[ri(1, SturdyShovel, 0)],
    },
    RewardGroup {
        points: 25,
        items: &[ri(1, SturdyPickaxe, 0)],
    },
    RewardGroup {
        points: 50,
        items: &[
            ri(45, SturdyShovel, 0),
            ri(45, SturdyPickaxe, 0),
            ri(10, MiningGloves, 1),
        ],
    },
    RewardGroup {
        points: 200,
        items: &[
            ri(45, GargoylesPickaxe, 0),
            ri(45, ProspectorsTool, 0),
            ri(10, MiningGloves, 3),
        ],
    },
    RewardGroup {
        points: 400,
        items: &[
            ri(2, GargoylesPickaxe, 0),
            ri(2, ProspectorsTool, 0),
            ri(1, PowderOfTemperament, 0),
        ],
    },
    RewardGroup {
        points: 450,
        items: &[ri(9, PowderOfTemperament, 0), ri(1, MiningGloves, 5)],
    },
    RewardGroup {
        points: 500,
        items: &[ri(1, RunicHammer, 1)],
    },
    RewardGroup {
        points: 550,
        items: &[ri(3, RunicHammer, 1), ri(2, RunicHammer, 2)],
    },
    RewardGroup {
        points: 600,
        items: &[ri(1, RunicHammer, 2)],
    },
    RewardGroup {
        points: 625,
        items: &[
            ri(3, RunicHammer, 2),
            ri(6, SmithPowerScroll, 5),
            ri(1, ColoredAnvil, 0),
        ],
    },
    RewardGroup {
        points: 650,
        items: &[ri(1, RunicHammer, 3)],
    },
    RewardGroup {
        points: 675,
        items: &[
            ri(1, ColoredAnvil, 0),
            ri(6, SmithPowerScroll, 10),
            ri(3, RunicHammer, 3),
        ],
    },
    RewardGroup {
        points: 700,
        items: &[ri(1, RunicHammer, 4)],
    },
    RewardGroup {
        points: 750,
        items: &[ri(1, AncientHammer, 10)],
    },
    RewardGroup {
        points: 800,
        items: &[ri(1, SmithPowerScroll, 15)],
    },
    RewardGroup {
        points: 850,
        items: &[ri(1, AncientHammer, 15)],
    },
    RewardGroup {
        points: 900,
        items: &[ri(1, SmithPowerScroll, 20)],
    },
    RewardGroup {
        points: 950,
        items: &[ri(1, RunicHammer, 5)],
    },
    RewardGroup {
        points: 1000,
        items: &[ri(1, AncientHammer, 30)],
    },
    RewardGroup {
        points: 1050,
        items: &[ri(1, RunicHammer, 6)],
    },
    RewardGroup {
        points: 1100,
        items: &[ri(1, AncientHammer, 60)],
    },
    RewardGroup {
        points: 1150,
        items: &[ri(1, RunicHammer, 7)],
    },
    RewardGroup {
        points: 1200,
        items: &[ri(1, RunicHammer, 8)],
    },
];

pub static TAILOR_GROUPS: &[RewardGroup] = &[
    RewardGroup {
        points: 0,
        items: &[ri(1, Cloth, 0)],
    },
    RewardGroup {
        points: 50,
        items: &[ri(1, Cloth, 1)],
    },
    RewardGroup {
        points: 100,
        items: &[ri(1, Cloth, 2)],
    },
    RewardGroup {
        points: 150,
        items: &[ri(9, Cloth, 3), ri(1, Sandals, 0)],
    },
    RewardGroup {
        points: 200,
        items: &[ri(4, Cloth, 4), ri(1, Sandals, 0)],
    },
    RewardGroup {
        points: 300,
        items: &[ri(1, StretchedHide, 0)],
    },
    RewardGroup {
        points: 350,
        items: &[ri(1, RunicKit, 1)],
    },
    RewardGroup {
        points: 400,
        items: &[ri(2, TailorPowerScroll, 5), ri(3, Tapestry, 0)],
    },
    RewardGroup {
        points: 450,
        items: &[ri(1, BearRug, 0)],
    },
    RewardGroup {
        points: 500,
        items: &[ri(1, TailorPowerScroll, 10)],
    },
    RewardGroup {
        points: 550,
        items: &[ri(1, ClothingBlessDeed, 0)],
    },
    RewardGroup {
        points: 575,
        items: &[ri(1, TailorPowerScroll, 15)],
    },
    RewardGroup {
        points: 600,
        items: &[ri(1, RunicKit, 2)],
    },
    RewardGroup {
        points: 650,
        items: &[ri(1, TailorPowerScroll, 20)],
    },
    RewardGroup {
        points: 700,
        items: &[ri(1, RunicKit, 3)],
    },
];

/// `RewardType`s of the smith calculator: points and the exact class names.
static SMITH_TYPES: &[(i32, &[&str])] = &[
    (
        200,
        &[
            "RingmailGloves",
            "RingmailChest",
            "RingmailArms",
            "RingmailLegs",
        ],
    ),
    (300, &["ChainCoif", "ChainLegs", "ChainChest"]),
    (
        400,
        &[
            "PlateArms",
            "PlateLegs",
            "PlateHelm",
            "PlateGorget",
            "PlateGloves",
            "PlateChest",
        ],
    ),
    (200, &["Bardiche", "Halberd"]),
    (300, &["Dagger", "ShortSpear", "Spear", "WarFork", "Kryss"]),
    (
        350,
        &[
            "Axe",
            "BattleAxe",
            "DoubleAxe",
            "ExecutionersAxe",
            "LargeBattleAxe",
            "TwoHandedAxe",
        ],
    ),
    (
        350,
        &[
            "Broadsword",
            "Cutlass",
            "Katana",
            "Longsword",
            "Scimitar",
            "VikingSword",
        ],
    ),
    (
        350,
        &[
            "WarAxe",
            "HammerPick",
            "Mace",
            "Maul",
            "WarHammer",
            "WarMace",
        ],
    ),
];

fn type_is(full: Option<&str>, short: &str) -> bool {
    full.is_some_and(|f| f.strip_prefix("Server.Items.") == Some(short))
}

fn type_index(types: &[(i32, &[&str])], ty: Option<&str>) -> Option<usize> {
    types
        .iter()
        .position(|(_, names)| names.iter().any(|n| type_is(ty, n)))
}

/// `LookupRewards(points)`.
fn lookup(groups: &'static [RewardGroup], points: i32) -> &'static RewardGroup {
    for g in groups[1..].iter().rev() {
        if points >= g.points {
            return g;
        }
    }
    &groups[0]
}

/// What a deed hands to the calculators: `(quantity, exceptional, material, itemCount, type)`.
fn facts(w: &World, id: ItemId) -> (i32, bool, i32, usize, Option<&'static str>) {
    let d = data(w, id).cloned().unwrap_or_default();
    if is_large(w, id) {
        (
            d.amount_max,
            d.require_exceptional,
            d.material,
            d.entries.len(),
            d.entries.first().and_then(|e| e.details.type_name),
        )
    } else {
        (
            d.amount_max,
            d.require_exceptional,
            d.material,
            1,
            d.type_name,
        )
    }
}

fn quantity_points(quantity: i32) -> i32 {
    match quantity {
        10 => 10,
        15 => 25,
        20 => 50,
        _ => 0,
    }
}

/// `ComputePoints(bod)`.
pub fn compute_points(w: &World, id: ItemId) -> i32 {
    let (quantity, exceptional, material, count, ty) = facts(w, id);
    let mut points = quantity_points(quantity);
    if is_tailor(w, id) {
        if exceptional {
            points += 100;
        }
        points += match count {
            4 => 300,
            5 => 400,
            6 => 500,
            _ => 0,
        };
        points += match material {
            mat::SPINED => 50,
            mat::HORNED => 100,
            mat::BARBED => 150,
            _ => 0,
        };
    } else {
        if exceptional {
            points += 200;
        }
        if count > 1
            && let Some(i) = type_index(SMITH_TYPES, ty)
        {
            points += SMITH_TYPES[i].0;
        }
        if (mat::DULL_COPPER..=mat::VALORITE).contains(&material) {
            points += 200 + 50 * (material - mat::DULL_COPPER);
        }
    }
    points
}

/// `ComputeFame(bod)`.
pub fn compute_fame(w: &World, id: ItemId) -> i32 {
    let points = compute_points(w, id) / 50;
    points * points
}

fn quan_index(quantity: i32) -> usize {
    match quantity {
        20 => 2,
        15 => 1,
        _ => 0,
    }
}

/// `ComputeGold(bod)`.
pub fn compute_gold(w: &mut World, id: ItemId) -> i32 {
    let (quantity, exceptional, material, count, ty) = facts(w, id);
    let gold = if is_tailor(w, id) {
        let table = if muo_content::core::aos(w) {
            &TAILOR_AOS_GOLD
        } else {
            &TAILOR_OLD_GOLD
        };
        let type_index = match count {
            6 => 3,
            5 => 2,
            4 => 1,
            _ => 0,
        } * 2
            + usize::from(exceptional);
        let mtrl = match material {
            mat::BARBED => 3,
            mat::HORNED => 2,
            mat::SPINED => 1,
            _ => 0,
        };
        table[type_index][quan_index(quantity)][mtrl]
    } else {
        // Item count of 1 means it's a small BOD; the loop looks at the first seven reward types only.
        let mut type_idx = if count == 1 {
            None
        } else {
            Some(
                SMITH_TYPES
                    .iter()
                    .take(7)
                    .position(|(_, n)| n.iter().any(|x| type_is(ty, x)))
                    .unwrap_or(7),
            )
        };
        if let Some(t) = type_idx.as_mut() {
            // Types 5, 6 and 7 are Large Weapon BODs with the same rewards.
            if *t > 5 {
                *t = 5;
            }
        }
        let mut type_index = type_idx.map_or(0, |t| (t + 1) * 2);
        if exceptional {
            type_index += 1;
        }
        let mtrl = if (mat::DULL_COPPER..=mat::VALORITE).contains(&material) {
            (1 + (material - mat::DULL_COPPER)) as usize
        } else {
            0
        };
        SMITH_GOLD[type_index][quan_index(quantity)][mtrl]
    };
    let (min, max) = (gold * 9 / 10, gold * 10 / 9);
    rnd::min_max(w, min, max)
}

/// `ComputeRewards(full)`.
pub fn compute_rewards(w: &mut World, id: ItemId, full: bool) -> Vec<RewardItem> {
    let groups = if is_tailor(w, id) {
        TAILOR_GROUPS
    } else {
        SMITH_GROUPS
    };
    let group = lookup(groups, compute_points(w, id));
    if full {
        return group.items.to_vec();
    }
    // `AcquireItem`
    match group.items.len() {
        0 => Vec::new(),
        1 => vec![group.items[0]],
        _ => {
            let total: i32 = group.items.iter().map(|i| i.weight).sum();
            let mut r = rnd::below(w, total);
            for item in group.items {
                if r < item.weight {
                    return vec![*item];
                }
                r -= item.weight;
            }
            Vec::new()
        }
    }
}

const CLOTH_HUES: [[i32; 4]; 5] = [
    [0x483, 0x48C, 0x488, 0x48A],
    [0x495, 0x48B, 0x486, 0x485],
    [0x48D, 0x490, 0x48E, 0x491],
    [0x48F, 0x494, 0x484, 0x497],
    [0x489, 0x47F, 0x482, 0x47E],
];
const SANDAL_HUES: [i32; 8] = [0x489, 0x47F, 0x482, 0x47E, 0x48F, 0x494, 0x484, 0x497];

fn make(w: &mut World, short: &str) -> Option<ItemId> {
    muo_content::factory::make_item(w, &format!("Server.Items.{short}"))
}

fn pick4(w: &mut World, names: [&str; 4]) -> Option<ItemId> {
    // `Utility.Random(4) switch { 1 => b, 2 => c, 3 => d, _ => a }`
    let n = match rnd::below(w, 4) {
        1 => names[1],
        2 => names[2],
        3 => names[3],
        _ => names[0],
    };
    make(w, n)
}

/// `RewardItem.Construct()`: null when the constructor would throw.
pub fn construct(w: &mut World, r: &RewardItem) -> Option<ItemId> {
    match r.reward {
        SturdyShovel => make(w, "SturdyShovel"),
        SturdyPickaxe => make(w, "SturdyPickaxe"),
        GargoylesPickaxe => make(w, "GargoylesPickaxe"),
        ProspectorsTool => make(w, "ProspectorsTool"),
        PowderOfTemperament => make(w, "PowderOfTemperament"),
        ColoredAnvil => make(w, "ColoredAnvil"),
        ClothingBlessDeed => make(w, "ClothingBlessDeed"),
        MiningGloves => match r.ty {
            1 => Some(muo_items::special::rewards2::leather_gloves_of_mining_new_bonus(w, 1)),
            3 => Some(muo_items::special::rewards2::studded_gloves_of_mining_new_bonus(w, 3)),
            5 => Some(muo_items::special::rewards2::ringmail_gloves_of_mining_new_bonus(w, 5)),
            _ => None,
        },
        // `new RunicHammer(CraftResource.Iron + type, Core.AOS ? 55 - type * 5 : 50)`
        RunicHammer => (1..=8)
            .contains(&r.ty)
            .then(|| {
                let uses = if muo_content::core::aos(w) {
                    55 - r.ty * 5
                } else {
                    50
                };
                let args = [
                    muo_content::props::Value::Int(i64::from(1 + r.ty)),
                    muo_content::props::Value::Int(i64::from(uses)),
                ];
                muo_content::factory::make_item_args(w, "Server.Items.RunicHammer", &args)
            })
            .flatten(),
        // `new RunicSewingKit(CraftResource.RegularLeather + type, 60 - type * 15)`
        RunicKit => (1..=3)
            .contains(&r.ty)
            .then(|| {
                let args = [
                    muo_content::props::Value::Int(i64::from(101 + r.ty)),
                    muo_content::props::Value::Int(i64::from(60 - r.ty * 15)),
                ];
                muo_content::factory::make_item_args(w, "Server.Items.RunicSewingKit", &args)
            })
            .flatten(),
        // `new AncientSmithyHammer(type)`: the type is the bonus; it has no parameterless constructor.
        AncientHammer => matches!(r.ty, 10 | 15 | 30 | 60)
            .then(|| {
                let args = [muo_content::props::Value::Int(i64::from(r.ty))];
                muo_content::factory::make_item_args(w, "Server.Items.AncientSmithyHammer", &args)
            })
            .flatten(),
        SmithPowerScroll | TailorPowerScroll => {
            if !matches!(r.ty, 5 | 10 | 15 | 20) {
                return None;
            }
            let skill = if r.reward == SmithPowerScroll {
                SkillName::Blacksmith
            } else {
                SkillName::Tailoring
            };
            Some(muo_items::special::scrolls::power_scroll_new(
                w,
                skill,
                f64::from(100 + r.ty),
            ))
        }
        Cloth => {
            let row = usize::try_from(r.ty)
                .ok()
                .filter(|&t| t < CLOTH_HUES.len())?;
            let cloth = make(w, "UncutCloth")?;
            muo_server::item::ops::set_amount(w, cloth, 100);
            let hue = CLOTH_HUES[row][rnd::below(w, 4) as usize];
            let ty = w.item(cloth)?.ty;
            (ty.set_hue)(w, cloth, hue);
            Some(cloth)
        }
        Sandals => {
            let hue = SANDAL_HUES[rnd::below(w, 8) as usize];
            let s = make(w, "Sandals")?;
            let ty = w.item(s)?.ty;
            (ty.set_hue)(w, s, hue);
            Some(s)
        }
        StretchedHide => pick4(
            w,
            [
                "SmallStretchedHideEastDeed",
                "SmallStretchedHideSouthDeed",
                "MediumStretchedHideEastDeed",
                "MediumStretchedHideSouthDeed",
            ],
        ),
        Tapestry => pick4(
            w,
            [
                "LightFlowerTapestryEastDeed",
                "LightFlowerTapestrySouthDeed",
                "DarkFlowerTapestryEastDeed",
                "DarkFlowerTapestrySouthDeed",
            ],
        ),
        BearRug => pick4(
            w,
            [
                "BrownBearRugEastDeed",
                "BrownBearRugSouthDeed",
                "PolarBearRugEastDeed",
                "PolarBearRugSouthDeed",
            ],
        ),
    }
}

/// The bulk order part of `BaseVendor.OnDragDrop(from, dropped)`: `None` when `dropped` is not a deed (or bulk
/// orders are off) and the base drop applies, else whether the drop was taken.
pub fn vendor_turn_in(
    w: &mut World,
    vendor: MobileId,
    from: MobileId,
    dropped: ItemId,
) -> Option<bool> {
    use muo_server::mobile::part2::speech::say_to_number;
    let enabled = w
        .resources
        .get::<crate::feature_flags::ContentFeatureFlags>()
        .is_none_or(|c| c.bulk_orders);
    if !enabled || !super::is_bod(w, dropped) {
        return None;
    }
    let ml = muo_content::core::ml(w);
    let now = w.now;
    let is_player = muo_content::player_mobile::data(w, from).is_some();
    if ml
        && muo_content::player_mobile::data(w, from).is_some_and(|d| d.next_bod_turn_in_time > now)
    {
        say_to_number(w, vendor, from, 1079976, ""); // You'll have to wait a few seconds while I inspect the last order.
        return Some(false);
    }
    let valid = muo_content::vendor::class(w, vendor)
        .is_some_and(|c| (c.is_valid_bulk_order)(w, vendor, dropped));
    if !valid {
        say_to_number(w, vendor, from, 1045130, ""); // That order is for some other shopkeeper.
        return Some(false);
    }
    if !super::complete(w, dropped) {
        say_to_number(w, vendor, from, 1045131, ""); // You have not completed the order yet.
        return Some(false);
    }
    let (reward, gold, fame) = super::get_rewards(w, dropped);
    muo_server::mobile::part2::effects::send_sound(w, from, 0x3D);
    say_to_number(w, vendor, from, 1045132, ""); // Thank you so much!  Here is a reward for your effort.
    if let Some(r) = reward {
        muo_server::mobile::part2::items::add_to_backpack(w, from, r);
    }
    if gold > 1000 {
        let c = muo_content::items::gold::new_bank_check(w, gold);
        muo_server::mobile::part2::items::add_to_backpack(w, from, c);
    } else if gold > 0 {
        let g = muo_content::items::gold::new_gold(w, gold);
        muo_server::mobile::part2::items::add_to_backpack(w, from, g);
    }
    muo_content::misc::titles::award_fame(w, from, fame, true);
    if let Some(c) = muo_content::vendor::class(w, vendor) {
        (c.on_successful_bulk_order_receive)(w, vendor, from);
    }
    if ml && is_player {
        let until = w.now + TimeSpan::from_secs(10);
        if let Some(d) = muo_content::player_mobile::data_mut(w, from) {
            d.next_bod_turn_in_time = until;
        }
    }
    if let Some(ty) = w.item(dropped).map(|i| i.ty) {
        (ty.delete)(w, dropped);
    }
    Some(true)
}
