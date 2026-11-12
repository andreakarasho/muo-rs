//! The faction mobiles: `BaseFactionGuard`, `Orders`, the ten guard classes, `FactionGuardAI` (its
//! movement and combat decisions), `BaseFactionVendor` and the five vendors, and `FactionWarHorse`.
//!
//! The guard classes are one table of [`GuardSpec`]s: a spec is the constructor of the class (body, stats,
//! resistances, skills, worn items, pack) in a fixed order, so the random calls always happen in that order.
//!
//! (the spell, bandage and potion half of `Think` is guard_ai.rs).

use muo_primitives::dotnet::InvariantCase;

use muo_content::creature::enums::{AIType, DEFAULT_RANGE_PERCEPTION, FightMode, OrderType};
use muo_content::creature::mount::{BASE_MOUNT, BASE_MOUNT_C, new_mount};
use muo_content::creature::setters::{
    new_creature, pack_item, set_damage, set_dex_range, set_int_range, set_resistance_range,
    set_skill_range, set_str_range,
};
use muo_content::creature::{BASE_CREATURE, BASE_CREATURE_C, CreatureType};
use muo_content::hooks::rnd;
use muo_content::vendor::{
    BASE_VENDOR, BASE_VENDOR_C, BASE_VENDOR_CLASS, BuyInfo, SbInfo, SbInstance, VendorClass,
};
use muo_io::{GenericReader, GenericWriter};
use muo_primitives::{DateTime, Direction, MessageType, Point3D, TimeSpan, utility};
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::item::{ITEM_BASE, LootType, base as item_base, ops};
use muo_server::layer::Layer;
use muo_server::map::MapId;
use muo_server::mobile::part1::{props_a, props_b};
use muo_server::mobile::part2::{items, messages as mm};
use muo_server::mobile::{MobileType, ResistanceType};
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::skills::SkillName;
use muo_server::timer;
use muo_server::type_registry::TypeRegistry;

use muo_content::ai::{ActionType, AiClass, base as ai_base, movement as ai_move, set_action};
use muo_content::mx;

use super::state::{read_faction_ref, read_town_ref, write_faction_ref, write_town_ref};
use super::tables::{FACTION_DEFS, TOWN_DEFS, VENDOR_DEFS};
use super::{FACET, find, st, town};

// ---- data ----

/// `ReactionType`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ReactionType {
    Ignore = 0,
    Warn = 1,
    Attack = 2,
}

/// `MovementType`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MovementType {
    Stand = 0,
    Patrol = 1,
    Follow = 2,
}

/// `Orders`.
#[derive(Clone, Debug)]
pub struct Orders {
    pub reactions: Vec<(Option<usize>, ReactionType)>,
    pub movement: MovementType,
    pub follow: Option<MobileId>,
}

impl Orders {
    fn new() -> Orders {
        Orders {
            reactions: Vec::new(),
            movement: MovementType::Patrol,
            follow: None,
        }
    }
}

/// What `BaseFactionGuard` adds to a creature.
pub struct GuardData {
    pub faction: Option<usize>,
    pub town: Option<usize>,
    pub orders: Orders,
    /// `m_OrdersEnd`.
    pub orders_end: DateTime,
    /// `GuardAI`.
    pub flags: u8,
}

/// `GuardAI` flags.
pub const BLESS: u8 = 0x01;
pub const CURSE: u8 = 0x02;
pub const MELEE: u8 = 0x04;
pub const MAGIC: u8 = 0x08;
pub const SMART: u8 = 0x10;

/// What `BaseFactionVendor` adds to a vendor.
#[derive(Default)]
pub struct FactionVendorData {
    pub faction: Option<usize>,
    pub town: Option<usize>,
}

/// `FactionWarHorse._faction`.
#[derive(Default)]
pub struct WarHorseData {
    pub faction: Option<usize>,
}

pub(super) fn gd(w: &World, m: MobileId) -> Option<&GuardData> {
    w.mobile(m)?.ext.get::<GuardData>()
}

fn gd_mut(w: &mut World, m: MobileId) -> Option<&mut GuardData> {
    w.mobile_mut(m)?.ext.get_mut::<GuardData>()
}

/// `m is BaseFactionGuard`.
pub fn is_guard(w: &World, m: MobileId) -> bool {
    gd(w, m).is_some()
}

/// `BaseFactionGuard.Faction`.
pub fn guard_faction(w: &World, m: MobileId) -> Option<usize> {
    gd(w, m)?.faction
}

/// `BaseFactionGuard.Town`.
pub fn guard_town(w: &World, m: MobileId) -> Option<usize> {
    gd(w, m)?.town
}

fn guard_unregister(w: &mut World, g: MobileId) {
    if let Some(t) = guard_town(w, g) {
        town::unregister_guard(w, t, g);
    }
}

fn guard_register(w: &mut World, g: MobileId) {
    if let Some(d) = gd(w, g)
        && let (Some(t), Some(_)) = (d.town, d.faction)
    {
        town::register_guard(w, t, g);
    }
}

/// `Faction` setter.
pub fn set_guard_faction(w: &mut World, g: MobileId, f: Option<usize>) {
    guard_unregister(w, g);
    if let Some(d) = gd_mut(w, g) {
        d.faction = f;
    }
    guard_register(w, g);
}

/// `Town` setter.
pub fn set_guard_town(w: &mut World, g: MobileId, t: Option<usize>) {
    guard_unregister(w, g);
    if let Some(d) = gd_mut(w, g) {
        d.town = t;
    }
    guard_register(w, g);
}

fn vd(w: &World, m: MobileId) -> Option<&FactionVendorData> {
    w.mobile(m)?.ext.get::<FactionVendorData>()
}

/// `BaseFactionVendor.Town`.
pub fn vendor_town(w: &World, m: MobileId) -> Option<usize> {
    vd(w, m)?.town
}

/// `BaseFactionVendor.Faction`.
pub fn vendor_faction(w: &World, m: MobileId) -> Option<usize> {
    vd(w, m)?.faction
}

fn vendor_register(w: &mut World, v: MobileId) {
    if let Some(d) = vd(w, v)
        && let (Some(t), Some(_)) = (d.town, d.faction)
    {
        town::register_vendor(w, t, v);
    }
}

fn vendor_unregister(w: &mut World, v: MobileId) {
    if let Some(t) = vendor_town(w, v) {
        town::unregister_vendor(w, t, v);
    }
}

pub fn set_vendor_town_faction(w: &mut World, v: MobileId, t: Option<usize>, f: Option<usize>) {
    vendor_unregister(w, v);
    if let Some(d) = w
        .mobile_mut(v)
        .and_then(|m| m.ext.get_mut::<FactionVendorData>())
    {
        d.town = t;
        d.faction = f;
    }
    vendor_register(w, v);
}

/// `entity.MoveToWorld(location, map); entity.Home = entity.Location`.
pub fn move_and_home(w: &mut World, m: MobileId, loc: Point3D, map: Option<MapId>) {
    if let Some(map) = map {
        muo_server::mobile::base::move_to_world(w, m, loc, map);
    }
    let here = w.mobile(m).map(|x| x.location).unwrap_or(loc);
    muo_content::creature::control::set_home(w, m, here);
}

/// `GuardList.Construct()`: a guard by class name, not yet in a town.
pub fn construct_guard(w: &mut World, type_name: &str) -> Option<MobileId> {
    muo_content::factory::make_mobile(w, type_name)
}

/// `VendorList.Construct(town, faction)`.
pub fn construct_vendor(w: &mut World, def: usize, t: usize, f: usize) -> Option<MobileId> {
    let v = muo_content::factory::make_mobile(w, VENDOR_DEFS[def].type_name)?;
    set_vendor_town_faction(w, v, Some(t), Some(f));
    Some(v)
}

// ---- items ----

#[derive(Clone, Copy)]
struct Wear {
    name: &'static str,
    hue: Option<i32>,
    immovable: bool,
    newbied: bool,
    layer: Option<Layer>,
}

const fn imm(name: &'static str, hue: i32) -> Wear {
    Wear {
        name,
        hue: Some(hue),
        immovable: true,
        newbied: false,
        layer: None,
    }
}
const fn plain(name: &'static str) -> Wear {
    Wear {
        name,
        hue: None,
        immovable: false,
        newbied: false,
        layer: None,
    }
}
const fn newbied(name: &'static str, hue: Option<i32>) -> Wear {
    Wear {
        name,
        hue,
        immovable: false,
        newbied: true,
        layer: None,
    }
}
const SHROUD: &str = "Server.Item:0x204E";
const MOUNT: &str = "Server.Mobiles.VirtualMountItem";

fn it(name: &str) -> String {
    if name.starts_with("Server.") {
        name.to_owned()
    } else {
        format!("Server.Items.{name}")
    }
}

fn add_wear(w: &mut World, id: MobileId, wear: Wear) {
    let item = if wear.name == SHROUD {
        let i = ops::new_item_with_id(w, &ITEM_BASE, 0x204E);
        (w.item(i).expect("item").ty.set_layer)(w, i, Layer::OuterTorso);
        Some(i)
    } else if wear.name == MOUNT {
        Some(muo_content::creature::virtual_mount::new_virtual_mount_item(w, id))
    } else {
        muo_content::factory::make_item(w, &it(wear.name))
    };
    let Some(item) = item else { return };
    if let Some(l) = wear.layer {
        (w.item(item).expect("item").ty.set_layer)(w, item, l);
    }
    if let Some(h) = wear.hue {
        item_base::set_hue(w, item, h);
    }
    if wear.immovable {
        ops::set_movable(w, item, false);
    }
    if wear.newbied {
        ops::set_loot_type(w, item, LootType::Newbied);
    }
    items::add_item(w, id, item);
}

const STRONG: [&str; 13] = [
    "GreaterHealPotion",
    "GreaterHealPotion",
    "GreaterHealPotion",
    "GreaterCurePotion",
    "GreaterCurePotion",
    "GreaterCurePotion",
    "GreaterStrengthPotion",
    "GreaterStrengthPotion",
    "GreaterAgilityPotion",
    "GreaterAgilityPotion",
    "TotalRefreshPotion",
    "TotalRefreshPotion",
    "GreaterExplosionPotion",
];
const WEAK: [&str; 13] = [
    "HealPotion",
    "HealPotion",
    "HealPotion",
    "CurePotion",
    "CurePotion",
    "CurePotion",
    "StrengthPotion",
    "StrengthPotion",
    "AgilityPotion",
    "AgilityPotion",
    "RefreshPotion",
    "RefreshPotion",
    "ExplosionPotion",
];

/// `PackStrongPotions(min, max)` / `PackWeakPotions(min, max)`.
fn pack_potions(w: &mut World, id: MobileId, strong: bool, min: i32, max: i32) {
    let n = rnd::min_max(w, min, max);
    let list = if strong { &STRONG } else { &WEAK };
    for _ in 0..n {
        let i = rnd::below(w, list.len() as i32) as usize;
        let item = muo_content::factory::make_item(w, &it(list[i]));
        pack_item(w, id, item);
    }
}

// ---- guards ----

struct GuardSpec {
    title: &'static str,
    full_name: &'static str,
    female: bool,
    random_hair: bool,
    hue: Option<i32>,
    str_: (i32, i32),
    dex: (i32, i32),
    int: (i32, i32),
    damage: Option<(i32, i32)>,
    physical_100: bool,
    resist: (i32, i32),
    virtual_armor: i32,
    skills: &'static [(SkillName, f64, f64)],
    wear: &'static [Wear],
    /// Hair item, hair hue, facial hair item, facial hair hue.
    hair: Option<(i32, i32, i32, i32)>,
    bandages: (i32, i32),
    potions: (bool, i32, i32),
    flags: u8,
}

use SkillName as S;

const KNIGHT_SKILLS: &[(SkillName, f64, f64)] = &[
    (S::Swords, 100.0, 110.0),
    (S::Wrestling, 100.0, 110.0),
    (S::Tactics, 100.0, 110.0),
    (S::MagicResist, 100.0, 110.0),
    (S::Healing, 100.0, 110.0),
    (S::Anatomy, 100.0, 110.0),
    (S::Magery, 100.0, 110.0),
    (S::EvalInt, 100.0, 110.0),
    (S::Meditation, 100.0, 110.0),
];
const MAGE_SKILLS: &[(SkillName, f64, f64)] = &[
    (S::Macing, 110.0, 120.0),
    (S::Wrestling, 110.0, 120.0),
    (S::Tactics, 110.0, 120.0),
    (S::MagicResist, 110.0, 120.0),
    (S::Healing, 110.0, 120.0),
    (S::Anatomy, 110.0, 120.0),
    (S::Magery, 110.0, 120.0),
    (S::EvalInt, 110.0, 120.0),
    (S::Meditation, 110.0, 120.0),
];

static KNIGHT: GuardSpec = GuardSpec {
    title: "the knight",
    full_name: "Server.Factions.FactionKnight",
    female: false,
    random_hair: false,
    hue: None,
    str_: (126, 150),
    dex: (61, 85),
    int: (81, 95),
    damage: None,
    physical_100: true,
    resist: (30, 50),
    virtual_armor: 24,
    skills: KNIGHT_SKILLS,
    wear: &[
        imm("ChainChest", 2125),
        imm("ChainLegs", 2125),
        imm("ChainCoif", 2125),
        imm("PlateArms", 2125),
        imm("PlateGloves", 2125),
        imm("BodySash", 1254),
        imm("Kilt", 1254),
        imm("Sandals", 1254),
        newbied("Bardiche", None),
    ],
    hair: None,
    bandages: (30, 40),
    potions: (true, 6, 12),
    flags: MAGIC | MELEE | SMART | CURSE | BLESS,
};
static BERSERKER: GuardSpec = GuardSpec {
    title: "the berserker",
    full_name: "Server.Factions.FactionBerserker",
    female: false,
    random_hair: false,
    hue: None,
    str_: (126, 150),
    dex: (61, 85),
    int: (81, 95),
    damage: None,
    physical_100: true,
    resist: (30, 50),
    virtual_armor: 24,
    skills: KNIGHT_SKILLS,
    wear: &[
        imm("BodySash", 1645),
        imm("Kilt", 1645),
        imm("Sandals", 1645),
        newbied("DoubleAxe", None),
    ],
    hair: Some((0x2047, 0x29, 0x204B, 0x29)),
    bandages: (30, 40),
    potions: (true, 6, 12),
    flags: MELEE | CURSE | BLESS,
};
static DEATH_KNIGHT: GuardSpec = GuardSpec {
    title: "the death knight",
    full_name: "Server.Factions.FactionDeathKnight",
    female: false,
    random_hair: false,
    hue: Some(1),
    str_: (126, 150),
    dex: (61, 85),
    int: (81, 95),
    damage: None,
    physical_100: true,
    resist: (30, 50),
    virtual_armor: 24,
    skills: KNIGHT_SKILLS,
    wear: &[imm(SHROUD, 1109), newbied("ExecutionersAxe", Some(2211))],
    hair: None,
    bandages: (30, 40),
    potions: (true, 6, 12),
    flags: MELEE | CURSE | BLESS,
};
static DRAGOON: GuardSpec = GuardSpec {
    title: "the dragoon",
    full_name: "Server.Factions.FactionDragoon",
    female: false,
    random_hair: false,
    hue: None,
    str_: (151, 175),
    dex: (61, 85),
    int: (151, 175),
    damage: None,
    physical_100: false,
    resist: (40, 60),
    virtual_armor: 32,
    skills: MAGE_SKILLS,
    wear: &[
        imm("Cloak", 1645),
        imm("PlateChest", 1645),
        imm("PlateLegs", 1109),
        imm("PlateArms", 1109),
        imm("PlateGloves", 1109),
        imm("PlateGorget", 1109),
        imm("PlateHelm", 1109),
        newbied("WarHammer", None),
        imm(MOUNT, 1109),
    ],
    hair: None,
    bandages: (30, 40),
    potions: (true, 6, 12),
    flags: MAGIC | MELEE | SMART | BLESS | CURSE,
};
static HENCHMAN: GuardSpec = GuardSpec {
    title: "the henchman",
    full_name: "Server.Factions.FactionHenchman",
    female: false,
    random_hair: true,
    hue: None,
    str_: (91, 115),
    dex: (61, 85),
    int: (81, 95),
    damage: Some((10, 14)),
    physical_100: false,
    resist: (10, 30),
    virtual_armor: 8,
    skills: &[
        (S::Fencing, 80.0, 90.0),
        (S::Wrestling, 80.0, 90.0),
        (S::Tactics, 80.0, 90.0),
        (S::MagicResist, 80.0, 90.0),
        (S::Healing, 80.0, 90.0),
        (S::Anatomy, 80.0, 90.0),
    ],
    wear: &[
        plain("StuddedChest"),
        plain("StuddedLegs"),
        plain("StuddedArms"),
        plain("StuddedGloves"),
        plain("StuddedGorget"),
        plain("Boots"),
        newbied("Spear", None),
    ],
    hair: None,
    bandages: (10, 20),
    potions: (false, 1, 4),
    flags: MELEE,
};
static MERCENARY: GuardSpec = GuardSpec {
    title: "the mercenary",
    full_name: "Server.Factions.FactionMercenary",
    female: false,
    random_hair: true,
    hue: None,
    str_: (116, 125),
    dex: (61, 85),
    int: (81, 95),
    damage: None,
    physical_100: false,
    resist: (20, 40),
    virtual_armor: 16,
    skills: &[
        (S::Fencing, 90.0, 100.0),
        (S::Wrestling, 90.0, 100.0),
        (S::Tactics, 90.0, 100.0),
        (S::MagicResist, 90.0, 100.0),
        (S::Healing, 90.0, 100.0),
        (S::Anatomy, 90.0, 100.0),
    ],
    wear: &[
        plain("ChainChest"),
        plain("ChainLegs"),
        plain("RingmailArms"),
        plain("RingmailGloves"),
        plain("ChainCoif"),
        plain("Boots"),
        newbied("ShortSpear", None),
    ],
    hair: None,
    bandages: (20, 30),
    potions: (true, 3, 8),
    flags: MELEE | SMART,
};
static NECROMANCER: GuardSpec = GuardSpec {
    title: "the necromancer",
    full_name: "Server.Factions.FactionNecromancer",
    female: false,
    random_hair: false,
    hue: Some(1),
    str_: (151, 175),
    dex: (61, 85),
    int: (151, 175),
    damage: None,
    physical_100: false,
    resist: (40, 60),
    virtual_armor: 32,
    skills: MAGE_SKILLS,
    wear: &[imm(SHROUD, 1109), newbied("GnarledStaff", Some(2211))],
    hair: None,
    bandages: (30, 40),
    potions: (true, 6, 12),
    flags: MAGIC | SMART | BLESS | CURSE,
};
static PALADIN: GuardSpec = GuardSpec {
    title: "the paladin",
    full_name: "Server.Factions.FactionPaladin",
    female: false,
    random_hair: false,
    hue: None,
    str_: (151, 175),
    dex: (61, 85),
    int: (81, 95),
    damage: None,
    physical_100: false,
    resist: (40, 60),
    virtual_armor: 32,
    skills: &[
        (S::Swords, 110.0, 120.0),
        (S::Wrestling, 110.0, 120.0),
        (S::Tactics, 110.0, 120.0),
        (S::MagicResist, 110.0, 120.0),
        (S::Healing, 110.0, 120.0),
        (S::Anatomy, 110.0, 120.0),
        (S::Magery, 110.0, 120.0),
        (S::EvalInt, 110.0, 120.0),
        (S::Meditation, 110.0, 120.0),
    ],
    wear: &[
        imm("PlateChest", 2125),
        imm("PlateLegs", 2125),
        imm("PlateHelm", 2125),
        imm("PlateGorget", 2125),
        imm("PlateArms", 2125),
        imm("PlateGloves", 2125),
        imm("BodySash", 1254),
        imm("Cloak", 1254),
        newbied("Halberd", None),
        imm(MOUNT, 1254),
    ],
    hair: None,
    bandages: (30, 40),
    potions: (true, 6, 12),
    flags: MAGIC | MELEE | SMART | CURSE | BLESS,
};
static SORCERESS: GuardSpec = GuardSpec {
    title: "the sorceress",
    full_name: "Server.Factions.FactionSorceress",
    female: true,
    random_hair: false,
    hue: None,
    str_: (126, 150),
    dex: (61, 85),
    int: (126, 150),
    damage: None,
    physical_100: true,
    resist: (30, 50),
    virtual_armor: 24,
    skills: &[
        (S::Macing, 100.0, 110.0),
        (S::Wrestling, 100.0, 110.0),
        (S::Tactics, 100.0, 110.0),
        (S::MagicResist, 100.0, 110.0),
        (S::Healing, 100.0, 110.0),
        (S::Anatomy, 100.0, 110.0),
        (S::Magery, 100.0, 110.0),
        (S::EvalInt, 100.0, 110.0),
        (S::Meditation, 100.0, 110.0),
    ],
    wear: &[
        imm("WizardsHat", 1325),
        imm("Sandals", 1325),
        imm("LeatherGorget", 1325),
        imm("LeatherGloves", 1325),
        imm("LeatherLegs", 1325),
        imm("Skirt", 1325),
        imm("FemaleLeatherChest", 1325),
        newbied("QuarterStaff", Some(1310)),
    ],
    hair: None,
    bandages: (30, 40),
    potions: (true, 6, 12),
    flags: MAGIC | BLESS | CURSE,
};
static WIZARD: GuardSpec = GuardSpec {
    title: "the wizard",
    full_name: "Server.Factions.FactionWizard",
    female: false,
    random_hair: false,
    hue: None,
    str_: (151, 175),
    dex: (61, 85),
    int: (151, 175),
    damage: None,
    physical_100: true,
    resist: (40, 60),
    virtual_armor: 32,
    skills: MAGE_SKILLS,
    wear: &[
        imm("WizardsHat", 1325),
        imm("Sandals", 1325),
        imm("Robe", 1310),
        imm("LeatherGloves", 1325),
        newbied("GnarledStaff", Some(1310)),
    ],
    hair: None,
    bandages: (30, 40),
    potions: (true, 6, 12),
    flags: MAGIC | SMART | BLESS | CURSE,
};

/// The constructor of a guard class: `BaseFactionGuard(title)` then the class's own statements.
fn new_guard(w: &mut World, ty: &'static MobileType, spec: &'static GuardSpec) -> MobileId {
    let id = new_creature(
        w,
        ty,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(GuardData {
            faction: None,
            town: None,
            orders: Orders::new(),
            orders_end: DateTime::MIN,
            flags: spec.flags,
        });
    }
    props_a::set_title(w, id, Some(spec.title.to_owned()));
    if let Some(d) = muo_content::creature::cd_mut(w, id) {
        d.range_home = 6;
    }
    // `GenerateBody(isFemale, randomHair)`
    let skin = utility::random_from(&mut w.rng, 1002, 57) | 0x8000;
    item_base_hue(w, id, skin);
    let name = if spec.female {
        props_b::set_female(w, id, true);
        props_b::set_body(w, id, muo_primitives::Body(401));
        muo_content::misc::small::random_name(w, "female")
    } else {
        props_b::set_female(w, id, false);
        props_b::set_body(w, id, muo_primitives::Body(400));
        muo_content::misc::small::random_name(w, "male")
    };
    muo_server::mobile::base::set_name(w, id, Some(name));
    if spec.random_hair {
        muo_content::vendor::assign_random_hair_random_hue(w, id);
        let hue = w.mobile(id).map_or(0, |m| m.hair_hue);
        muo_content::vendor::assign_random_facial_hair(w, id, hue);
    }
    if let Some(h) = spec.hue {
        item_base_hue(w, id, h);
    }
    set_str_range(w, id, spec.str_.0, spec.str_.1);
    set_dex_range(w, id, spec.dex.0, spec.dex.1);
    set_int_range(w, id, spec.int.0, spec.int.1);
    if let Some((a, b)) = spec.damage {
        set_damage(w, id, a, b);
    }
    if spec.physical_100 {
        muo_content::creature::setters::set_damage_type(w, id, ResistanceType::Physical, 100);
    }
    for r in [
        ResistanceType::Physical,
        ResistanceType::Fire,
        ResistanceType::Cold,
        ResistanceType::Energy,
        ResistanceType::Poison,
    ] {
        set_resistance_range(w, id, r, spec.resist.0, spec.resist.1);
    }
    props_a::set_virtual_armor(w, id, spec.virtual_armor);
    for &(s, a, b) in spec.skills {
        set_skill_range(w, id, s, a, b);
    }
    for &wear in spec.wear {
        add_wear(w, id, wear);
    }
    if let Some((hi, hh, fi, fh)) = spec.hair {
        props_b::set_hair_item_id(w, id, hi);
        props_b::set_hair_hue(w, id, hh);
        props_b::set_facial_hair_item_id(w, id, fi);
        props_b::set_facial_hair_hue(w, id, fh);
    }
    let n = rnd::min_max(w, spec.bandages.0, spec.bandages.1);
    let bandage = muo_content::factory::make_item_amount(w, "Server.Items.Bandage", n);
    pack_item(w, id, bandage);
    pack_potions(w, id, spec.potions.0, spec.potions.1, spec.potions.2);
    id
}

/// `Mobile.Hue = value`.
fn item_base_hue(w: &mut World, id: MobileId, hue: i32) {
    muo_server::mobile::base::set_hue(w, id, hue);
}

/// `Orders.GetReaction(faction)`: the stored reaction, adding the default one.
fn get_reaction(w: &mut World, g: MobileId, f: Option<usize>) -> ReactionType {
    let own = guard_faction(w, g);
    let d = gd_mut(w, g).expect("guard");
    if let Some(&(_, t)) = d.orders.reactions.iter().find(|(rf, _)| *rf == f) {
        return t;
    }
    let t = if f.is_none() || f == own {
        ReactionType::Ignore
    } else {
        ReactionType::Attack
    };
    d.orders.reactions.push((f, t));
    t
}

fn set_reaction(w: &mut World, g: MobileId, f: Option<usize>, t: ReactionType) {
    get_reaction(w, g, f);
    if let Some(e) =
        gd_mut(w, g).and_then(|d| d.orders.reactions.iter_mut().find(|(rf, _)| *rf == f))
    {
        e.1 = t;
    }
}

/// `BaseFactionGuard.IsEnemy(m)`.
fn guard_is_enemy(w: &mut World, g: MobileId, m: MobileId) -> bool {
    let our = guard_faction(w, g);
    let mut theirs = find(w, m);
    if theirs.is_none() {
        theirs = guard_faction(w, m);
    }
    if let (Some(o), Some(t)) = (our, theirs)
        && o != t
    {
        if get_reaction(w, g, Some(t)) == ReactionType::Attack {
            return true;
        }
        let aggressed: Vec<MobileId> = w.mobile(m).map_or_else(Vec::new, |x| {
            x.aggressed.iter().map(|a| a.defender).collect()
        });
        for d in aggressed {
            if guard_faction(w, d) == Some(o) {
                return true;
            }
        }
    }
    false
}

fn guard_on_movement(w: &mut World, g: MobileId, m: MobileId, old: Point3D) {
    (BASE_CREATURE.on_movement)(w, g, m, old);
    let player = w.mobile(m).is_some_and(|x| x.player);
    if !(player && muo_server::mobile::base::alive(w, m)) {
        return;
    }
    let loc = w.mobile(m).map(|x| x.location).unwrap_or_default();
    if !(muo_server::mobile::base::in_range_point3d(w, g, loc, 10)
        && !muo_server::mobile::base::in_range_point3d(w, g, old, 10)
        && muo_server::mobile::part1::combat::in_los(w, g, m))
    {
        return;
    }
    let mf = find(w, m);
    if get_reaction(w, g, mf) != ReactionType::Warn {
        return;
    }
    let d = mx::get_direction_to(w, g, m);
    muo_server::mobile::part1::props_b::set_direction(w, g, d);
    let name = muo_server::mobile::base::name(w, m).unwrap_or_default();
    let line = match rnd::below(w, 6) {
        0 => format!(
            "I warn you, {name}, you would do well to leave this area before someone shows you the world of gray."
        ),
        1 => format!(
            "It would be wise to leave this area, {name}, lest your head become my commanders' trophy."
        ),
        2 => format!(
            "You are bold, {name}, for one of the meager {}. Leave now, lest you be taught the taste of dirt.",
            mf.map_or("civilians", |f| FACTION_DEFS[f].friendly_name)
        ),
        3 => format!("Your presence here is an insult, {name}. Be gone now, knave."),
        4 => format!("Dost thou wish to be hung by your toes, {name}? Nay? Then come no closer."),
        _ => format!("Hey, {name}. Yeah, you. Get out of here before I beat you with a stick."),
    };
    mm::say_text(w, g, &line);
}

fn guard_handles_on_speech(w: &World, g: MobileId, from: MobileId) -> bool {
    let loc = w.mobile(from).map(|x| x.location).unwrap_or_default();
    muo_server::mobile::base::in_range_point3d(w, g, loc, 12)
        || (BASE_CREATURE.handles_on_speech)(w, g, from)
}

/// `ChangeReaction(faction, type)`.
fn change_reaction(w: &mut World, g: MobileId, f: Option<usize>, t: ReactionType) {
    match f {
        None => match t {
            ReactionType::Ignore => mm::say_number(w, g, 1005179, ""), // Civilians will now be ignored.
            ReactionType::Warn => mm::say_number(w, g, 1005180, ""), // Civilians will now be warned of their impending deaths.
            ReactionType::Attack => return,
        },
        Some(f) => {
            let d = FACTION_DEFS[f];
            let n = match t {
                ReactionType::Ignore => d.guard_ignore,
                ReactionType::Warn => d.guard_warn,
                ReactionType::Attack => d.guard_attack,
            };
            if n > 0 {
                mm::say_number(w, g, n, "");
            }
        }
    }
    set_reaction(w, g, f, t);
}

fn guard_on_speech(w: &mut World, g: MobileId, e: &mut muo_server::events::SpeechEventArgs) {
    (BASE_CREATURE.on_speech)(w, g, e);
    let from = e.mobile;
    let loc = w.mobile(from).map(|x| x.location).unwrap_or_default();
    if e.handled
        || !muo_server::mobile::base::in_range_point3d(w, g, loc, 12)
        || !muo_server::mobile::base::alive(w, from)
    {
        return;
    }
    let speech = e.speech.lower_inv();
    let town_of_guard = guard_town(w, g);
    let here = town_of_region(w, g);
    let sheriff = town_of_guard.is_some_and(|t| town::is_sheriff(w, t, from));
    let now = w.now;
    if e.keywords.contains(&0xE6)
        && (speech == "orders"
            || muo_server::mobile::base::name(w, g)
                .is_some_and(|n| speech.starts_with(&n.lower_inv())))
    {
        if !sheriff {
            mm::say_number(w, g, 1042189, ""); // I don't work for you!
        } else if here == town_of_guard {
            mm::say_number(w, g, 1042180, ""); // Your orders, sire?
            if let Some(d) = gd_mut(w, g) {
                d.orders_end = now + TimeSpan::from_secs(10);
            }
        }
    } else if gd(w, g).is_some_and(|d| now < d.orders_end) {
        if !sheriff || here != town_of_guard {
            return;
        }
        if let Some(d) = gd_mut(w, g) {
            d.orders_end = now + TimeSpan::from_secs(10);
        }
        let new_type = if speech.contains("attack") {
            Some(ReactionType::Attack)
        } else if speech.contains("warn") {
            Some(ReactionType::Warn)
        } else if speech.contains("ignore") {
            Some(ReactionType::Ignore)
        } else {
            None
        };
        let mut understood = true;
        if let Some(nt) = new_type {
            understood = false;
            if speech.contains("civil") {
                change_reaction(w, g, None, nt);
                understood = true;
            }
            let own = guard_faction(w, g);
            for (f, def) in FACTION_DEFS.iter().enumerate() {
                if Some(f) != own && speech.contains(def.keyword) {
                    change_reaction(w, g, Some(f), nt);
                    understood = true;
                }
            }
        } else if speech.contains("patrol") {
            let loc = w.mobile(g).map(|x| x.location).unwrap_or_default();
            muo_content::creature::control::set_home(w, g, loc);
            if let Some(c) = muo_content::creature::cd_mut(w, g) {
                c.range_home = 6;
            }
            mx::set_combatant(w, g, None);
            if let Some(d) = gd_mut(w, g) {
                d.orders.movement = MovementType::Patrol;
            }
            mm::say_number(w, g, 1005146, ""); // This spot looks like it needs protection!  I shall guard it with my life.
        } else if speech.contains("follow") {
            let loc = w.mobile(g).map(|x| x.location).unwrap_or_default();
            muo_content::creature::control::set_home(w, g, loc);
            if let Some(c) = muo_content::creature::cd_mut(w, g) {
                c.range_home = 6;
            }
            mx::set_combatant(w, g, None);
            if let Some(d) = gd_mut(w, g) {
                d.orders.follow = Some(from);
                d.orders.movement = MovementType::Follow;
            }
            mm::say_number(w, g, 1005144, ""); // Yes, Sire.
        } else {
            understood = false;
        }
        if !understood {
            mm::say_number(w, g, 1042183, ""); // I'm sorry, I don't understand your orders...
        }
    }
}

/// `Town.FromRegion(mobile.Region)`.
fn town_of_region(w: &World, m: MobileId) -> Option<usize> {
    town::from_region(w, props_b::region(w, m))
}

fn guard_get_properties(
    w: &World,
    g: MobileId,
    list: &mut muo_server::property_list::PropertyList,
) {
    (BASE_CREATURE.get_properties)(w, g, list);
    if let Some(f) = guard_faction(w, g)
        && w.mobile(g).and_then(|m| m.map) == Some(FACET)
    {
        list.add_str(1060846, &super::number_text(FACTION_DEFS[f].prop_name)); // Guard: ~1_val~
    }
}

fn guard_on_single_click(w: &mut World, g: MobileId, from: MobileId) {
    if let Some(f) = guard_faction(w, g)
        && w.mobile(g).and_then(|m| m.map) == Some(FACET)
    {
        let text = format!("(Guard, {})", FACTION_DEFS[f].friendly_name);
        let hue = if find(w, from) == Some(f) { 98 } else { 38 };
        let ns = w.mobile(from).and_then(|m| m.net_state);
        mm::private_overhead_message(w, g, MessageType::LABEL, hue, true, &text, ns);
    }
    (BASE_CREATURE.on_single_click)(w, g, from);
}

/// `BaseFactionGuard.OnDeath(c)`: no corpse is left.
fn guard_on_death(w: &mut World, g: MobileId, c: Option<muo_server::ids::ItemId>) {
    (BASE_CREATURE.on_death)(w, g, c);
    if let Some(c) = c
        && let Some(t) = w.item(c).map(|i| i.ty)
    {
        (t.delete)(w, c);
    }
}

fn guard_on_after_delete(w: &mut World, g: MobileId) {
    (BASE_CREATURE.on_after_delete)(w, g);
    guard_unregister(w, g);
}

fn guard_serialize(w: &World, g: MobileId, out: &mut dyn GenericWriter) {
    (BASE_CREATURE.serialize)(w, g, out);
    out.write_i32(0); // version
    let Some(d) = gd(w, g) else { return };
    write_faction_ref(out, d.faction);
    write_town_ref(out, d.town);
    // Orders.Serialize
    out.write_encoded_int(1);
    out.write_mobile(w, d.orders.follow);
    out.write_encoded_int(d.orders.reactions.len() as i32);
    for &(f, t) in &d.orders.reactions {
        out.write_encoded_int(0); // Reaction version
        write_faction_ref(out, f);
        out.write_encoded_int(t as i32);
    }
    out.write_encoded_int(d.orders.movement as i32);
}

fn guard_deserialize(
    w: &mut World,
    g: MobileId,
    r: &mut dyn GenericReader,
    flags: u8,
) -> muo_io::Result<()> {
    (BASE_CREATURE.deserialize)(w, g, r)?;
    let _version = r.read_i32()?;
    let faction = read_faction_ref(r)?;
    let town_ = read_town_ref(r)?;
    let mut orders = Orders::new();
    let ov = r.read_encoded_int()?;
    if ov == 1 {
        orders.follow = r.read_mobile(w)?;
    }
    if ov == 0 || ov == 1 {
        let n = r.read_encoded_int()?;
        for _ in 0..n {
            let _rv = r.read_encoded_int()?;
            let f = read_faction_ref(r)?;
            let t = match r.read_encoded_int()? {
                1 => ReactionType::Warn,
                2 => ReactionType::Attack,
                _ => ReactionType::Ignore,
            };
            orders.reactions.push((f, t));
        }
        orders.movement = match r.read_encoded_int()? {
            0 => MovementType::Stand,
            2 => MovementType::Follow,
            _ => MovementType::Patrol,
        };
    }
    if let Some(m) = w.mobile_mut(g) {
        m.ext.insert(GuardData {
            faction,
            town: town_,
            orders,
            orders_end: DateTime::MIN,
            flags,
        });
    }
    // `Timer.StartTimer(Register)`
    timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
        guard_register(w, g)
    });
    Ok(())
}

// ---- FactionGuardAI ----

/// `FactionGuardAI.RunTo(m)`.
fn run_to(w: &mut World, g: MobileId, m: MobileId) {
    let range_fight = muo_content::creature::cd(w, g).map_or(1, |d| d.range_fight);
    let loc = w.mobile(m).map(|x| x.location).unwrap_or_default();
    if !muo_server::mobile::base::in_range_point3d(w, g, loc, range_fight) {
        if !ai_move::move_to(w, g, m, 1) {
            on_failed_move(w, g);
        }
    } else if muo_server::mobile::base::in_range_point3d(w, g, loc, range_fight - 1) {
        run_from(w, g, m);
    }
}

/// `RunFrom(m)`.
fn run_from(w: &mut World, g: MobileId, m: MobileId) {
    let d = mx::get_direction_to(w, g, m);
    run(
        w,
        g,
        Direction::from_bits_truncate((d.bits().wrapping_sub(4)) & Direction::MASK.bits()),
    );
}

/// `Run(d)`.
fn run(w: &mut World, g: MobileId, d: Direction) {
    let (paralyzed, frozen) = w
        .mobile(g)
        .map_or((false, false), |m| (m.paralyzed, m.frozen));
    let casting = w
        .mobile(g)
        .and_then(|m| m.spell)
        .is_some_and(|sp| muo_content::spells::spell::is_casting(w, sp));
    let no_moves =
        muo_content::creature::vtable::ctype(w, g).is_some_and(|c| (c.disallow_all_moves)(w, g));
    if casting || paralyzed || frozen || no_moves {
        return;
    }
    let dir = d | Direction::RUNNING;
    muo_server::mobile::part1::props_b::set_direction(w, g, dir);
    if !ai_move::do_move(w, g, dir, true) {
        on_failed_move(w, g);
    }
}

/// `OnFailedMove()`.
fn on_failed_move(w: &mut World, g: MobileId) {
    let (rp, mode) = muo_content::creature::cd(w, g)
        .map_or((0, FightMode::None), |d| (d.range_perception, d.fight_mode));
    if ai_base::acquire_focus_mob(w, g, rp, mode, false, false, true) {
        let f = muo_content::creature::cd(w, g).and_then(|d| d.focus_mob);
        mx::set_combatant(w, g, f);
        set_action(w, g, ActionType::Combat);
    } else {
        ai_base::debug_say(w, g, "I am stuck");
    }
}

/// `FactionGuardAI.Think()`.
fn guard_think(w: &mut World, g: MobileId) -> bool {
    if mx::deleted(w, g) {
        return false;
    }
    let (rp, mode) = muo_content::creature::cd(w, g)
        .map_or((0, FightMode::None), |d| (d.range_perception, d.fight_mode));
    let mut combatant = mx::combatant(w, g);
    let bad = match combatant {
        None => true,
        Some(c) => {
            mx::deleted(w, c)
                || !mx::alive(w, c)
                || w.mobile(c).is_some_and(|m| (m.ty.is_dead_bonded_pet)(w, c))
                || !muo_server::mobile::base::can_see_mobile(w, g, c)
                || !mx::can_be_harmful_quiet(w, g, c)
                || mx::map(w, c) != mx::map(w, g)
        }
    };
    if bad {
        if ai_base::acquire_focus_mob(w, g, rp, mode, false, false, true) {
            combatant = muo_content::creature::cd(w, g).and_then(|d| d.focus_mob);
            mx::set_combatant(w, g, combatant);
            if let Some(d) = muo_content::creature::cd_mut(w, g) {
                d.focus_mob = None;
            }
        } else {
            combatant = None;
            mx::set_combatant(w, g, None);
        }
    }
    if let Some(c) = combatant {
        let loc = w.mobile(c).map(|x| x.location).unwrap_or_default();
        if !mx::in_los(w, g, c) || !muo_server::mobile::base::in_range_point3d(w, g, loc, 12) {
            if ai_base::acquire_focus_mob(w, g, rp, mode, false, false, true) {
                combatant = muo_content::creature::cd(w, g).and_then(|d| d.focus_mob);
                mx::set_combatant(w, g, combatant);
                if let Some(d) = muo_content::creature::cd_mut(w, g) {
                    d.focus_mob = None;
                }
            } else if !muo_server::mobile::base::in_range_point3d(w, g, loc, 36) {
                combatant = None;
                mx::set_combatant(w, g, None);
            }
        }
    }

    let dispel = super::guard_ai::before_movement(w, g, combatant);
    let movement = gd(w, g).map_or(MovementType::Stand, |d| d.orders.movement);
    if let Some(d) = dispel {
        if muo_content::ai::action(w, g) != Some(ActionType::Combat) {
            set_action(w, g, ActionType::Combat);
        }
        mx::set_warmode(w, g, true);
        run_from(w, g, d);
    } else if let Some(c) = combatant {
        if muo_content::ai::action(w, g) != Some(ActionType::Combat) {
            set_action(w, g, ActionType::Combat);
        }
        mx::set_warmode(w, g, true);
        run_to(w, g, c);
    } else if movement != MovementType::Stand {
        let mut to_follow = None;
        if let Some(t) = guard_town(w, g)
            && movement == MovementType::Follow
        {
            to_follow = gd(w, g)
                .and_then(|d| d.orders.follow)
                .or_else(|| st(w).and_then(|s| s.towns[t].sheriff));
        }
        let follow_ok = to_follow.filter(|&f| {
            mx::map(w, f) == mx::map(w, g)
                && {
                    let loc = w.mobile(g).map(|x| x.location).unwrap_or_default();
                    muo_server::mobile::base::in_range_point3d(w, f, loc, rp * 3)
                }
                && town_of_region(w, f) == guard_town(w, g)
        });
        if let Some(f) = follow_ok {
            if muo_content::ai::action(w, g) != Some(ActionType::Combat) {
                set_action(w, g, ActionType::Combat);
            }
            muo_content::creature::speeds::set_current_speed_to_active(w, g);
            mx::set_warmode(w, g, true);
            run_to(w, g, f);
        } else {
            if muo_content::ai::action(w, g) != Some(ActionType::Wander) {
                set_action(w, g, ActionType::Wander);
            }
            muo_content::creature::speeds::set_current_speed_to_passive(w, g);
            mx::set_warmode(w, g, false);
            ai_move::walk_random_in_home(w, g, 2, 2, 1);
        }
    } else {
        if muo_content::ai::action(w, g) != Some(ActionType::Wander) {
            set_action(w, g, ActionType::Wander);
        }
        mx::set_warmode(w, g, false);
    }
    super::guard_ai::spells(w, g, combatant, dispel);
    true
}

static FACTION_GUARD_AI: AiClass = AiClass {
    name: "FactionGuardAI",
    think: guard_think,
    ..ai_base::BASE_AI
};

// ---- guard classes ----

macro_rules! guard_class {
    ($name:ident, $class:ident, $ctor:ident, $spec:ident) => {
        fn $ctor(w: &mut World) -> MobileId {
            new_guard(w, &$name, &$spec)
        }
        static $class: CreatureType = CreatureType {
            bard_immune: |_, _| true,
            reacquire_delay: |_, _| TimeSpan::from_secs(2),
            forced_ai: |_, _| Some(&FACTION_GUARD_AI),
            is_enemy: guard_is_enemy,
            ..BASE_CREATURE_C
        };
        pub static $name: MobileType = MobileType {
            full_name: $spec.full_name,
            parent: Some(&BASE_CREATURE),
            construct: Some($ctor),
            family: Some(&$class),
            serialize: |w, id, out| {
                guard_serialize(w, id, out);
                out.write_i32(0);
            },
            deserialize: |w, id, r| {
                guard_deserialize(w, id, r, $spec.flags)?;
                let _version = r.read_i32()?;
                Ok(())
            },
            on_movement: guard_on_movement,
            handles_on_speech: guard_handles_on_speech,
            on_speech: guard_on_speech,
            get_properties: guard_get_properties,
            on_single_click: guard_on_single_click,
            on_after_delete: guard_on_after_delete,
            click_title: |_, _| false,
            on_death: guard_on_death,
            ..BASE_CREATURE
        };
    };
}

guard_class!(FACTION_KNIGHT, FACTION_KNIGHT_C, knight_new, KNIGHT);
guard_class!(
    FACTION_BERSERKER,
    FACTION_BERSERKER_C,
    berserker_new,
    BERSERKER
);
guard_class!(
    FACTION_DEATH_KNIGHT,
    FACTION_DEATH_KNIGHT_C,
    death_knight_new,
    DEATH_KNIGHT
);
guard_class!(FACTION_DRAGOON, FACTION_DRAGOON_C, dragoon_new, DRAGOON);
guard_class!(FACTION_HENCHMAN, FACTION_HENCHMAN_C, henchman_new, HENCHMAN);
guard_class!(
    FACTION_MERCENARY,
    FACTION_MERCENARY_C,
    mercenary_new,
    MERCENARY
);
guard_class!(
    FACTION_NECROMANCER,
    FACTION_NECROMANCER_C,
    necromancer_new,
    NECROMANCER
);
guard_class!(FACTION_PALADIN, FACTION_PALADIN_C, paladin_new, PALADIN);
guard_class!(
    FACTION_SORCERESS,
    FACTION_SORCERESS_C,
    sorceress_new,
    SORCERESS
);
guard_class!(FACTION_WIZARD, FACTION_WIZARD_C, wizard_new, WIZARD);

// ---- vendors ----

fn vendor_init(w: &mut World, v: MobileId) {
    props_a::set_frozen(w, v, true);
    if let Some(m) = w.mobile_mut(v) {
        m.cant_walk = true;
    }
    if let Some(d) = muo_content::creature::cd_mut(w, v) {
        d.range_home = 0;
    }
    props_b::set_female(w, v, false);
    props_b::set_body(w, v, muo_primitives::Body(400));
    let name = muo_content::misc::small::random_name(w, "male");
    muo_server::mobile::base::set_name(w, v, Some(name));
    if let Some(m) = w.mobile_mut(v) {
        m.ext.insert(FactionVendorData::default());
    }
}

fn buy_items(
    items: &'static [(&'static str, i32, i32)],
    repeat: usize,
) -> impl Fn(&mut World, MobileId, &mut SbInstance) {
    move |_w, _v, sb| {
        for _ in 0..repeat {
            for &(ty, price, item_id) in items {
                sb.buy.push(BuyInfo::new(None, ty, price, 20, item_id, 0));
            }
        }
    }
}

static SB_BOTTLE: SbInfo = SbInfo {
    name: "SBFactionBottle",
    build: |w, v, sb| buy_items(&[("Server.Items.Bottle", 5, 0xF0E)], 5)(w, v, sb),
};
static SB_BOARD: SbInfo = SbInfo {
    name: "SBFactionBoard",
    build: |w, v, sb| buy_items(&[("Server.Items.Board", 3, 0x1BD7)], 5)(w, v, sb),
};
static SB_ORE: SbInfo = SbInfo {
    name: "SBFactionOre",
    build: |_w, _v, sb| {
        for _ in 0..5 {
            // `new object[] { true }`: a fixed-size IronOre.
            sb.buy
                .push(BuyInfo::new(None, "Server.Items.IronOre", 16, 20, 0x19B8, 0).with_arg(1));
        }
    },
};
static SB_REAGENT: SbInfo = SbInfo {
    name: "SBFactionReagent",
    build: |w, v, sb| {
        buy_items(
            &[
                ("Server.Items.BlackPearl", 5, 0xF7A),
                ("Server.Items.Bloodmoss", 5, 0xF7B),
                ("Server.Items.MandrakeRoot", 3, 0xF86),
                ("Server.Items.Garlic", 3, 0xF84),
                ("Server.Items.Ginseng", 3, 0xF85),
                ("Server.Items.Nightshade", 3, 0xF88),
                ("Server.Items.SpidersSilk", 3, 0xF8D),
                ("Server.Items.SulfurousAsh", 3, 0xF8C),
            ],
            2,
        )(w, v, sb)
    },
};

fn shoe_random(w: &mut World, _v: MobileId) -> muo_content::vendor::ShoeType {
    if rnd::bool(w) {
        muo_content::vendor::ShoeType::Shoes
    } else {
        muo_content::vendor::ShoeType::Sandals
    }
}

fn add_hued(w: &mut World, v: MobileId, name: &str, hue: Option<i32>) {
    if let Some(i) = muo_content::factory::make_item(w, name) {
        if let Some(h) = hue {
            item_base::set_hue(w, i, h);
        }
        items::add_item(w, v, i);
    }
}

macro_rules! vendor_class {
    ($name:ident, $class:ident, $vclass:ident, $full:literal, $ctor:ident, $title:literal, $init:expr) => {
        fn $ctor(w: &mut World) -> MobileId {
            let id = muo_content::vendor::new_vendor(w, &$name, Some($title));
            vendor_init(w, id);
            let init: fn(&mut World, MobileId) = $init;
            init(w, id);
            id
        }
        pub static $class: CreatureType = CreatureType {
            subclass: Some(&$vclass),
            ..BASE_VENDOR_C
        };
        pub static $name: MobileType = MobileType {
            full_name: $full,
            parent: Some(&BASE_VENDOR),
            construct: None, // `FactionVendor(town, faction)` has no parameterless constructor
            make: Some($ctor),
            family: Some(&$class),
            serialize: |w, id, out| {
                (BASE_VENDOR.serialize)(w, id, out);
                out.write_i32(0); // generated version
                faction_vendor_serialize(w, id, out);
            },
            deserialize: |w, id, r| {
                (BASE_VENDOR.deserialize)(w, id, r)?;
                let _version = r.read_i32()?;
                faction_vendor_deserialize(w, id, r)
            },
            on_after_delete: |w, id| {
                (BASE_VENDOR.on_after_delete)(w, id);
                vendor_unregister(w, id);
            },
            // `BaseFactionVendor.OnMoveOver`: from ML on anyone may walk through a faction vendor.
            on_move_over: |w, id, m| {
                muo_content::core::ml(w) || (BASE_VENDOR.on_move_over)(w, id, m)
            },
            ..BASE_VENDOR
        };
    };
}

fn faction_vendor_serialize(w: &World, id: MobileId, out: &mut dyn GenericWriter) {
    out.write_i32(0); // BaseFactionVendor version
    write_town_ref(out, vendor_town(w, id));
    write_faction_ref(out, vendor_faction(w, id));
}

fn faction_vendor_deserialize(
    w: &mut World,
    id: MobileId,
    r: &mut dyn GenericReader,
) -> muo_io::Result<()> {
    let version = r.read_i32()?;
    let mut d = FactionVendorData::default();
    if version == 0 {
        d.town = read_town_ref(r)?;
        d.faction = read_faction_ref(r)?;
    }
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(FactionVendorData::default());
    }
    set_vendor_town_faction(w, id, d.town, d.faction);
    props_a::set_frozen(w, id, true);
    Ok(())
}

static BOTTLE_V: VendorClass = VendorClass {
    init_sb_info: |w, id| vec![SB_BOTTLE.new_for(w, id)],
    shoe_type: shoe_random,
    init_outfit: |w, id| {
        muo_content::vendor::init_outfit(w, id);
        let h = utility::random_pink_hue(&mut w.rng);
        add_hued(w, id, "Server.Items.Robe", Some(h));
    },
    check_vendor_access: |_, _, _| true,
    ..BASE_VENDOR_CLASS
};
static BOARD_V: VendorClass = VendorClass {
    init_sb_info: |w, id| vec![SB_BOARD.new_for(w, id)],
    init_outfit: |w, id| {
        muo_content::vendor::init_outfit(w, id);
        add_hued(w, id, "Server.Items.HalfApron", None);
    },
    check_vendor_access: |_, _, _| true,
    ..BASE_VENDOR_CLASS
};
static ORE_V: VendorClass = VendorClass {
    init_sb_info: |w, id| vec![SB_ORE.new_for(w, id)],
    init_outfit: |w, id| {
        muo_content::vendor::init_outfit(w, id);
        add_hued(w, id, "Server.Items.HalfApron", None);
    },
    check_vendor_access: |_, _, _| true,
    ..BASE_VENDOR_CLASS
};
static REAGENT_V: VendorClass = VendorClass {
    init_sb_info: |w, id| vec![SB_REAGENT.new_for(w, id)],
    shoe_type: shoe_random,
    init_outfit: |w, id| {
        muo_content::vendor::init_outfit(w, id);
        let h = utility::random_blue_hue(&mut w.rng);
        add_hued(w, id, "Server.Items.Robe", Some(h));
        add_hued(w, id, "Server.Items.GnarledStaff", None);
    },
    check_vendor_access: |_, _, _| true,
    ..BASE_VENDOR_CLASS
};
static HORSE_V: VendorClass = VendorClass {
    init_sb_info: |_, _| Vec::new(),
    shoe_type: |w, id| {
        if w.mobile(id).is_some_and(|m| m.female) {
            muo_content::vendor::ShoeType::ThighBoots
        } else {
            muo_content::vendor::ShoeType::Boots
        }
    },
    get_shoe_hue: |_, _| 0,
    init_outfit: |w, id| {
        muo_content::vendor::init_outfit(w, id);
        if rnd::bool(w) {
            add_hued(w, id, "Server.Items.QuarterStaff", None);
        } else {
            add_hued(w, id, "Server.Items.ShepherdsCrook", None);
        }
    },
    vendor_buy: |w, id, from| {
        let f = vendor_faction(w, id);
        let Some(f) = f.filter(|_| find(w, from) == f) else {
            let ns = w.mobile(from).and_then(|m| m.net_state);
            // You are not in my faction, I cannot sell you a horse!
            mm::private_overhead_message_number(
                w,
                id,
                MessageType::REGULAR,
                0x3B2,
                1042201,
                "",
                ns,
            );
            return;
        };
        if super::items::faction_gump_exists(w, from) {
            mm::send_localized_message(w, from, 1042160, "", 0x3B2); // You already have a faction menu open.
        } else if muo_content::player_mobile::data(w, from).is_some() {
            super::gumps::send_horse_breeder_gump(w, from, f);
        }
    },
    vendor_sell: |_, _, _| {},
    on_buy_items: |_, _, _, _| false,
    on_sell_items: |_, _, _, _| false,
    check_vendor_access: |_, _, _| true,
    ..BASE_VENDOR_CLASS
};

fn set_skills(w: &mut World, id: MobileId, list: &[(SkillName, f64, f64)]) {
    for &(s, a, b) in list {
        set_skill_range(w, id, s, a, b);
    }
}

vendor_class!(
    FACTION_BOTTLE_VENDOR,
    FACTION_BOTTLE_VENDOR_C,
    BOTTLE_V,
    "Server.Factions.FactionBottleVendor",
    bottle_new,
    "the Bottle Seller",
    |w, id| set_skills(
        w,
        id,
        &[(S::Alchemy, 85.0, 100.0), (S::TasteID, 65.0, 88.0)]
    )
);
vendor_class!(
    FACTION_BOARD_VENDOR,
    FACTION_BOARD_VENDOR_C,
    BOARD_V,
    "Server.Factions.FactionBoardVendor",
    board_new,
    "the LumberMan",
    |w, id| set_skills(
        w,
        id,
        &[(S::Carpentry, 85.0, 100.0), (S::Lumberjacking, 60.0, 83.0)]
    )
);
vendor_class!(
    FACTION_ORE_VENDOR,
    FACTION_ORE_VENDOR_C,
    ORE_V,
    "Server.Factions.FactionOreVendor",
    ore_new,
    "the Ore Man",
    |w, id| set_skills(
        w,
        id,
        &[(S::Carpentry, 85.0, 100.0), (S::Lumberjacking, 60.0, 83.0)]
    )
);
vendor_class!(
    FACTION_REAGENT_VENDOR,
    FACTION_REAGENT_VENDOR_C,
    REAGENT_V,
    "Server.Factions.FactionReagentVendor",
    reagent_new,
    "the Reagent Man",
    |w, id| set_skills(
        w,
        id,
        &[
            (S::EvalInt, 65.0, 88.0),
            (S::Inscribe, 60.0, 83.0),
            (S::Magery, 64.0, 100.0),
            (S::Meditation, 60.0, 83.0),
            (S::MagicResist, 65.0, 88.0),
            (S::Wrestling, 36.0, 68.0),
        ]
    )
);
vendor_class!(
    FACTION_HORSE_VENDOR,
    FACTION_HORSE_VENDOR_C,
    HORSE_V,
    "Server.Factions.FactionHorseVendor",
    horse_vendor_new,
    "the Horse Breeder",
    |w, id| set_skills(
        w,
        id,
        &[
            (S::AnimalLore, 64.0, 100.0),
            (S::AnimalTaming, 90.0, 100.0),
            (S::Veterinary, 65.0, 88.0)
        ]
    )
);

// ---- FactionWarHorse ----

pub fn war_horse_set_faction(w: &mut World, id: MobileId, f: Option<usize>) {
    if let Some(d) = w
        .mobile_mut(id)
        .and_then(|m| m.ext.get_mut::<WarHorseData>())
    {
        d.faction = f;
    }
    props_b::set_body(
        w,
        id,
        muo_primitives::Body(f.map_or(0xE2, |f| FACTION_DEFS[f].war_horse_body)),
    );
    muo_content::creature::mount::set_item_id(
        w,
        id,
        f.map_or(0x3EA0, |f| FACTION_DEFS[f].war_horse_item),
    );
}

/// `new FactionWarHorse(faction)`.
pub fn new_war_horse(w: &mut World, f: Option<usize>) -> Option<MobileId> {
    let id = muo_content::factory::make_mobile(w, "Server.Factions.FactionWarHorse")?;
    war_horse_set_faction(w, id, f);
    Some(id)
}

fn war_horse_construct(w: &mut World) -> MobileId {
    let id = new_mount(
        w,
        &FACTION_WAR_HORSE,
        0xE2,
        0x3EA0,
        AIType::AiMelee,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(WarHorseData::default());
        m.base_sound_id = 0xA8;
    }
    muo_content::creature::setters::set_str(w, id, 400);
    muo_content::creature::setters::set_dex(w, id, 125);
    set_int_range(w, id, 51, 55);
    muo_content::creature::setters::set_hits(w, id, 240);
    muo_content::creature::setters::set_mana(w, id, 0);
    set_damage(w, id, 5, 8);
    muo_content::creature::setters::set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 40, 50);
    for r in [
        ResistanceType::Fire,
        ResistanceType::Cold,
        ResistanceType::Poison,
        ResistanceType::Energy,
    ] {
        set_resistance_range(w, id, r, 30, 40);
    }
    set_skill_range(w, id, S::MagicResist, 25.1, 30.0);
    set_skill_range(w, id, S::Tactics, 29.3, 44.0);
    set_skill_range(w, id, S::Wrestling, 29.3, 44.0);
    props_a::set_fame(w, id, 300);
    props_a::set_karma(w, id, 300);
    if let Some(d) = muo_content::creature::cd_mut(w, id) {
        d.tamable_raw = true;
        d.control_slots = 1;
    }
    war_horse_set_faction(w, id, None);
    id
}

static FACTION_WAR_HORSE_C: CreatureType = CreatureType {
    default_name: |_, _| Some("a war horse"),
    corpse_name: |_, _| Some("a war horse corpse"),
    steps_max: |_, _| 6400,
    favorite_food: |_, _| {
        muo_content::creature::enums::FoodType::FRUITS_AND_VEGGIES
            | muo_content::creature::enums::FoodType::GRAINS_AND_HAY
    },
    ..BASE_MOUNT_C
};

pub static FACTION_WAR_HORSE: MobileType = MobileType {
    full_name: "Server.Factions.FactionWarHorse",
    parent: Some(&BASE_MOUNT),
    construct: Some(war_horse_construct),
    family: Some(&FACTION_WAR_HORSE_C),
    serialize: |w, id, out| {
        (BASE_MOUNT.serialize)(w, id, out);
        out.write_i32(0);
        write_faction_ref(
            out,
            w.mobile(id)
                .and_then(|m| m.ext.get::<WarHorseData>())
                .and_then(|d| d.faction),
        );
    },
    deserialize: |w, id, r| {
        (BASE_MOUNT.deserialize)(w, id, r)?;
        if let Some(m) = w.mobile_mut(id) {
            m.ext.insert(WarHorseData::default());
        }
        let version = r.read_i32()?;
        if version == 0 {
            let f = read_faction_ref(r)?;
            war_horse_set_faction(w, id, f);
        }
        Ok(())
    },
    on_double_click: |w, id, from| {
        let own = w
            .mobile(id)
            .and_then(|m| m.ext.get::<WarHorseData>())
            .and_then(|d| d.faction);
        let msg = |w: &mut World, n: i32| mm::send_localized_message(w, from, n, "", 0x3B2);
        match super::player_state(w, from) {
            None => msg(w, 1010366), // You cannot mount a faction war horse!
            Some((pf, _)) if Some(pf) != own => msg(w, 1010367), // You cannot ride an opposing faction's war horse!
            Some((pf, p)) if FACTION_DEFS[pf].ranks[p.rank].rank < 2 => {
                msg(w, 1010368); // You must achieve a faction rank of at least two before riding a war horse!
            }
            Some(_) => (BASE_MOUNT.on_double_click)(w, id, from),
        }
    },
    ..BASE_MOUNT
};

// ---- setup ----

/// Nothing runs at `Initialize`: guards register themselves when they load.
pub fn initialize(_w: &mut World) {
    let _ = OrderType::Follow;
}

pub fn register(types: &mut TypeRegistry) {
    for t in [
        &FACTION_KNIGHT,
        &FACTION_BERSERKER,
        &FACTION_DEATH_KNIGHT,
        &FACTION_DRAGOON,
        &FACTION_HENCHMAN,
        &FACTION_MERCENARY,
        &FACTION_NECROMANCER,
        &FACTION_PALADIN,
        &FACTION_SORCERESS,
        &FACTION_WIZARD,
    ] {
        types.register_mobile(t, &[]);
    }
    for t in [
        &FACTION_BOTTLE_VENDOR,
        &FACTION_BOARD_VENDOR,
        &FACTION_ORE_VENDOR,
        &FACTION_REAGENT_VENDOR,
        &FACTION_HORSE_VENDOR,
        &FACTION_WAR_HORSE,
    ] {
        types.register_mobile(t, &[]);
    }
    let _ = TOWN_DEFS.len();
}
