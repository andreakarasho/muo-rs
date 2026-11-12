//! Constructor parts of monsters and animals (`crate::hand::<class>_ctorN`): the statements the tables cannot
//! express as data.

use muo_content::craft::CraftResource;
use muo_content::creature::data::cd_mut;
use muo_content::creature::enums::{AIType, FightMode};
use muo_content::creature::{mount, speeds};
use muo_content::factory;
use muo_content::items::{armor, weapon};
use muo_primitives::utility;
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{base as ibase, ops};
use muo_server::mobile::part2::items;
use muo_server::skills::{self, SkillName};

use crate::common::{add_new, pack_item, pack_new, pack_new_amount, rnd};

pub(crate) fn si(name: &str) -> String {
    format!("Server.Items.{name}")
}

/// `PackItem(new T())`.
pub(crate) fn pack(w: &mut World, id: MobileId, name: &str) {
    pack_new(w, id, &si(name));
}

/// `AddItem(new T())`.
pub(crate) fn add(w: &mut World, id: MobileId, name: &str) -> Option<ItemId> {
    add_new(w, id, &si(name))
}

/// `AddItem(new T { Hue = hue })` / `new T(hue)`.
pub(crate) fn add_hue(w: &mut World, id: MobileId, name: &str, hue: i32) -> Option<ItemId> {
    let it = factory::make_item(w, &si(name))?;
    ibase::set_hue(w, it, hue);
    items::add_item(w, id, it);
    Some(it)
}

/// `AddItem(new T(Utility.RandomNondyedHue()))`: the hue is rolled before the constructor.
pub(crate) fn add_nondyed(w: &mut World, id: MobileId, name: &str) {
    let hue = utility::random_nondyed_hue(&mut w.rng);
    add_hue(w, id, name, hue);
}

/// `AddItem(new T { Movable = false, Hue = hue })`.
fn add_fixed(w: &mut World, id: MobileId, name: &str, hue: Option<i32>) -> Option<ItemId> {
    let it = factory::make_item(w, &si(name))?;
    if let Some(h) = hue {
        ibase::set_hue(w, it, h);
    }
    ops::set_movable(w, it, false);
    items::add_item(w, id, it);
    Some(it)
}

/// `switch (Utility.Random(n)) { case i: PackItem(new names[i]()) }`; an index past the list packs nothing.
fn pack_one_of(w: &mut World, id: MobileId, n: i32, names: &[&str]) {
    let i = rnd::below(w, n) as usize;
    if let Some(name) = names.get(i) {
        pack(w, id, name);
    }
}

/// `Race.<name>`.
pub(crate) fn race_named(w: &World, name: &str) -> Option<muo_server::race::RaceId> {
    w.resources
        .get::<muo_server::race::Races>()
        .and_then(|r| r.try_parse(name))
}

/// `AI = ai`.
pub(crate) fn set_ai(w: &mut World, id: MobileId, ai: AIType) {
    let ai = if ai == AIType::AiUseDefault {
        muo_content::creature::data::read(w, id, ai, |d| d.default_ai)
    } else {
        ai
    };
    if let Some(d) = cd_mut(w, id) {
        d.current_ai = ai;
    }
    muo_content::ai::change_ai_type(w, id, ai);
}

/// `FightMode = mode`.
pub(crate) fn set_fight_mode(w: &mut World, id: MobileId, mode: FightMode) {
    if let Some(d) = cd_mut(w, id) {
        d.fight_mode = mode;
    }
}

/// `SetSpeed(active, passive)`.
fn set_speed(w: &mut World, id: MobileId, active: f64, passive: f64) {
    speeds::set_speed(w, id, active, passive, true);
}

// ---- armour and clothing packs ----

const PLATE_PIECES: [&str; 6] = [
    "PlateArms",
    "PlateChest",
    "PlateGloves",
    "PlateGorget",
    "PlateLegs",
    "PlateHelm",
];
const BONE_PIECES: [&str; 5] = [
    "BoneArms",
    "BoneChest",
    "BoneGloves",
    "BoneLegs",
    "BoneHelm",
];
const LEATHER_PIECES: [&str; 6] = [
    "LeatherGorget",
    "LeatherGloves",
    "LeatherArms",
    "LeatherLegs",
    "LeatherCap",
    "LeatherChest",
];

pub fn bone_knight_ctor1(w: &mut World, id: MobileId) {
    pack_one_of(w, id, 6, &PLATE_PIECES);
}
pub fn skeletal_knight_ctor1(w: &mut World, id: MobileId) {
    pack_one_of(w, id, 6, &PLATE_PIECES);
}
pub fn skeleton_ctor1(w: &mut World, id: MobileId) {
    pack_one_of(w, id, 5, &BONE_PIECES);
}
pub fn deathwatch_beetle_hatchling_ctor1(w: &mut World, id: MobileId) {
    if rnd::bool(w) {
        let r = muo_content::loot::random_reagent(w);
        if let Some(i) = r {
            ops::set_amount(w, i, 3);
        }
        pack_item(w, id, r);
    }
}
/// `switch (Utility.Random(12))` with cases 0-5 only.
pub fn deathwatch_beetle_hatchling_ctor2(w: &mut World, id: MobileId) {
    pack_one_of(w, id, 12, &LEATHER_PIECES);
}
pub fn deathwatch_beetle_ctor1(w: &mut World, id: MobileId) {
    pack_one_of(w, id, 12, &LEATHER_PIECES);
}
/// `if (Utility.RandomDouble() < .5) PackItem(Seed.RandomBonsaiSeed());`
pub fn deathwatch_beetle_ctor2(w: &mut World, id: MobileId) {
    if rnd::double(w) < 0.5 {
        let s = super::seed::random_bonsai_seed(w);
        pack_item(w, id, s);
    }
}
pub fn yomotsu_priest_ctor1(w: &mut World, id: MobileId) {
    pack_one_of(w, id, 3, &["LongPants", "ShortPants"]);
}
pub fn yomotsu_priest_ctor2(w: &mut World, id: MobileId) {
    pack_one_of(w, id, 6, &["Shoes", "Sandals", "Boots", "ThighBoots"]);
}
pub fn yomotsu_warrior_ctor1(w: &mut World, id: MobileId) {
    pack_one_of(w, id, 4, &["Shoes", "Sandals", "Boots", "ThighBoots"]);
}
pub fn ronin_ctor1(w: &mut World, id: MobileId) {
    let i = rnd::below(w, 4) as usize;
    add(
        w,
        id,
        [
            "LightPlateJingasa",
            "ChainHatsuburi",
            "DecorativePlateKabuto",
            "LeatherJingasa",
        ][i],
    );
}
pub fn ronin_ctor2(w: &mut World, id: MobileId) {
    let i = rnd::below(w, 3) as usize;
    add(
        w,
        id,
        ["StuddedHaidate", "LeatherSuneate", "PlateSuneate"][i],
    );
}
pub fn orc_captain_ctor1(w: &mut World, id: MobileId) {
    let i = rnd::below(w, 7);
    match i {
        0 => pack(w, id, "Arrow"),
        1 => pack(w, id, "Lockpick"),
        2 => pack(w, id, "Shaft"),
        3 => pack(w, id, "Ribs"),
        4 => pack(w, id, "Bandage"),
        5 => {
            let it = bottle(w, 4, "Server.Items.BeverageBottle"); // BeverageType.Wine
            pack_item(w, id, it);
        }
        _ => {
            let it = bottle(w, 1, "Server.Items.Jug"); // BeverageType.Cider
            pack_item(w, id, it);
        }
    }
}

/// `new BeverageBottle(type)` / `new Jug(type)`: the class's `(BeverageType)` constructor.
fn bottle(w: &mut World, kind: i64, class: &str) -> Option<ItemId> {
    factory::make_item_args(w, class, &[muo_content::props::Value::Enum(kind)])
        .or_else(|| factory::make_item(w, class))
}

pub fn kraken_ctor1(w: &mut World, id: MobileId) {
    let rope = factory::make_item(w, &si("Rope"));
    if let Some(r) = rope {
        ibase::set_item_id(w, r, 0x14F8);
    }
    pack_item(w, id, rope);
    if w.expansion() >= muo_data::expansion::Expansion::UOR {
        pack(w, id, "SpecialFishingNet");
    }
}
pub fn fire_steed_ctor1(w: &mut World, id: MobileId) {
    let n = rnd::min_max(w, 16, 30);
    pack_new_amount(w, id, &si("Ruby"), n);
}
pub fn wanderer_of_the_void_ctor1(w: &mut World, id: MobileId) {
    let count = rnd::min_max(w, 2, 3);
    for _ in 0..count {
        let m = make_treasure_map(w, 3);
        pack_item(w, id, m);
    }
}

/// `new TreasureMap(level, Map.Trammel)`.
fn make_treasure_map(w: &mut World, level: i64) -> Option<ItemId> {
    use muo_content::props::Value;
    factory::make_item_args(
        w,
        &si("TreasureMap"),
        &[
            Value::Int(level),
            Value::Map(Some(muo_server::map::MapId::TRAMMEL)),
        ],
    )
}

pub fn headless_one_ctor1(w: &mut World, id: MobileId) {
    let i = rnd::below(w, 6) as usize;
    let part = [
        "Head", "Torso", "RightArm", "LeftArm", "RightLeg", "LeftLeg",
    ][i];
    add(w, id, part);
}
pub fn nightmare_ctor1(w: &mut World, id: MobileId) {
    let (body, item) = match rnd::below(w, 3) {
        0 => (116, 16039),
        1 => (178, 16041),
        _ => (179, 16055),
    };
    crate::common::body(w, id, body);
    mount::set_item_id(w, id, item);
}
pub fn bird_ctor1(w: &mut World, id: MobileId) {
    if rnd::bool(w) {
        crate::common::hue(w, id, 0x901);
        let n = match rnd::below(w, 3) {
            0 => "a crow",
            2 => "a raven",
            _ => "a magpie",
        };
        crate::common::name(w, id, Some(n.to_owned()));
    } else {
        let h = rnd::from(w, 2101, 30);
        crate::common::hue(w, id, h);
        crate::common::name_random(w, id, "bird");
    }
}
pub fn red_death_ctor1(w: &mut World, id: MobileId) {
    set_ai(w, id, AIType::AiMelee);
}
pub fn red_death_ctor2(w: &mut World, id: MobileId) {
    set_fight_mode(w, id, FightMode::Closest);
}
pub fn grobu_ctor1(w: &mut World, id: MobileId) {
    set_ai(w, id, AIType::AiMelee);
}
pub fn grobu_ctor2(w: &mut World, id: MobileId) {
    set_fight_mode(w, id, FightMode::Closest);
}
pub fn pyre_ctor1(w: &mut World, id: MobileId) {
    set_fight_mode(w, id, FightMode::Closest);
}
pub fn chaos_dragoon_ctor1(w: &mut World, id: MobileId) {
    set_speed(w, id, 0.15, 0.4);
}
pub fn chaos_dragoon_elite_ctor1(w: &mut World, id: MobileId) {
    set_speed(w, id, 0.15, 0.4);
}
pub fn corrupted_soul_ctor1(w: &mut World, id: MobileId) {
    set_speed(w, id, 0.25, 2.5);
}

pub fn master_theophilus_ctor1(w: &mut World, id: MobileId) {
    crate::npcs::vendors::replace_equipment(w, id, &si("Shoes"), Some(0x537));
}
pub fn master_theophilus_ctor2(w: &mut World, id: MobileId) {
    crate::npcs::vendors::replace_equipment(w, id, &si("Robe"), Some(0x452));
}
pub fn master_theophilus_ctor3(w: &mut World, id: MobileId) {
    for _ in 0..2 {
        if rnd::bool(w) {
            let i = rnd::min_max(w, 5, 9);
            muo_content::creature::packing::pack_necro_scroll(w, id, i);
        } else {
            muo_content::creature::packing::pack_scroll_range(w, id, 4, 7);
        }
    }
}

pub fn elf_brigand_ctor1(w: &mut World, id: MobileId) {
    let elf = race_named(w, "Elf");
    muo_server::mobile::part1::props_a::set_race(w, id, elf);
}
pub fn elf_brigand_ctor2(w: &mut World, id: MobileId) {
    // `Race.RandomSkinHue()`
    let ops = w.mobile(id).and_then(|m| m.race).and_then(|r| {
        w.resources
            .get::<muo_server::race::Races>()?
            .get(r)
            .map(|r| r.ops)
    });
    if let Some(ops) = ops {
        let h = (ops.random_skin_hue)(&mut w.rng);
        crate::common::hue(w, id, h);
    }
}
pub fn elf_brigand_ctor3(w: &mut World, id: MobileId) {
    let female = rnd::bool(w);
    muo_server::mobile::part1::props_b::set_female(w, id, female);
    if female {
        crate::common::body(w, id, 0x25E);
        crate::common::name_random(w, id, "female elf brigand");
        if rnd::below(w, 2) == 0 {
            add_nondyed(w, id, "Skirt");
        } else {
            add_nondyed(w, id, "Kilt");
        }
    } else {
        crate::common::body(w, id, 0x25D);
        crate::common::name_random(w, id, "male elf brigand");
        add_nondyed(w, id, "ShortPants");
    }
}
pub fn elf_brigand_ctor4(w: &mut World, id: MobileId) {
    add_nondyed(w, id, "Shirt");
}

pub fn guardian_ctor1(w: &mut World, id: MobileId) {
    if let Some(m) = factory::make_mobile(w, "Server.Mobiles.ForestOstard") {
        mount::set_rider(w, m, Some(id));
    }
}
pub fn guardian_ctor2(w: &mut World, id: MobileId) {
    for p in [
        "PlateChest",
        "PlateArms",
        "PlateGloves",
        "PlateGorget",
        "PlateLegs",
        "PlateHelm",
    ] {
        add_hue(w, id, p, 0x966);
    }
    if let Some(bow) = factory::make_item(w, &si("Bow")) {
        ops::set_movable(w, bow, false);
        let name = muo_server::mobile::part1::props_b::name(w, id);
        if let Some(d) = weapon::data_mut(w, bow) {
            d.crafter = name;
        }
        weapon::set_quality(w, bow, weapon::WeaponQuality::Exceptional);
        items::add_item(w, id, bow);
    }
    pack_new_amount(w, id, &si("Arrow"), 250);
    crate::common::pack_gold_range(w, id, 250, 500);
    for s in [
        SkillName::Anatomy,
        SkillName::Tactics,
        SkillName::Archery,
        SkillName::MagicResist,
    ] {
        skills::set_base(w, id, s, 120.0);
    }
    skills::set_base(w, id, SkillName::DetectHidden, 100.0);
}

pub fn khaldun_summoner_ctor1(w: &mut World, id: MobileId) {
    add_hue(w, id, "LeatherGloves", 0x66D);
    add_hue(w, id, "BoneHelm", 0x835);
    add_hue(w, id, "Necklace", 0x66D);
    add_hue(w, id, "Cloak", 0x66D);
    add_hue(w, id, "Kilt", 0x66D);
    add_hue(w, id, "Sandals", 0x66D);
}
pub fn khaldun_zealot_ctor1(w: &mut World, id: MobileId) {
    add_fixed(w, id, "VikingSword", Some(0x835));
    add_fixed(w, id, "MetalShield", Some(0x835));
    for p in [
        "BoneHelm",
        "BoneArms",
        "BoneGloves",
        "BoneChest",
        "BoneLegs",
    ] {
        add_hue(w, id, p, 0x835);
    }
    add(w, id, "Boots");
}

pub fn spectral_armour_ctor1(w: &mut World, id: MobileId) {
    add_fixed(w, id, "Buckler", Some(0x835));
}
pub fn spectral_armour_ctor2(w: &mut World, id: MobileId) {
    add_hue(w, id, "ChainCoif", 0x835);
}
pub fn spectral_armour_ctor3(w: &mut World, id: MobileId) {
    add_hue(w, id, "PlateGloves", 0x835);
}

pub fn cursed_ctor1(w: &mut World, id: MobileId) {
    use crate::common::*;
    let weapon = muo_content::loot::random_weapon(w, false, false, false);
    if let Some(wp) = weapon {
        ops::set_movable(w, wp, false);
        items::add_item(w, id, wp);
    }
    set_str_range(w, id, 91, 100);
    set_dex_range(w, id, 86, 95);
    set_int_range(w, id, 61, 70);
    set_hits_range(w, id, 91, 120);
    set_damage(w, id, 5, 13);
    set_resistance_range(w, id, ResistanceType::Physical, 15, 25);
    set_resistance_range(w, id, ResistanceType::Fire, 5, 10);
    set_resistance_range(w, id, ResistanceType::Cold, 25, 35);
    set_resistance_range(w, id, ResistanceType::Poison, 5, 10);
    set_resistance_range(w, id, ResistanceType::Energy, 5, 10);
    set_skill_range(w, id, SkillName::Fencing, 46.0, 77.5);
    set_skill_range(w, id, SkillName::Macing, 35.0, 57.5);
    set_skill_range(w, id, SkillName::MagicResist, 53.5, 62.5);
    set_skill_range(w, id, SkillName::Swords, 55.0, 77.5);
    set_skill_range(w, id, SkillName::Tactics, 60.0, 82.5);
    set_skill_range(w, id, SkillName::Poisoning, 60.0, 82.5);
    fame(w, id, 1000);
    karma(w, id, -2000);
}

pub fn juka_lord_ctor1(w: &mut World, id: MobileId) {
    use muo_server::item::container::drop_item_default;
    let pack_item_id = factory::make_item(w, &si("Backpack"));
    if let Some(p) = pack_item_id {
        for (name, lo, hi) in [
            ("Arrow", 25, 35),
            ("Arrow", 25, 35),
            ("Bandage", 5, 15),
            ("Bandage", 5, 15),
        ] {
            let n = rnd::min_max(w, lo, hi);
            if let Some(i) = factory::make_item_amount(w, &si(name), n) {
                drop_item_default(w, p, i);
            }
        }
        if let Some(g) = muo_content::loot::random_gem(w) {
            drop_item_default(w, p, g);
        }
        if let Some(g) = factory::make_item(w, &si("ArcaneGem")) {
            drop_item_default(w, p, g);
        }
    }
    pack_item(w, id, pack_item_id);
    add(w, id, "JukaBow");
}

pub fn pack_horse_ctor1(w: &mut World, id: MobileId) {
    super::pack_animal::replace_backpack(w, id);
}
pub fn pack_llama_ctor1(w: &mut World, id: MobileId) {
    super::pack_animal::replace_backpack(w, id);
}

// ---- the dragoons ----

fn random_scale_resource(w: &mut World) -> CraftResource {
    match rnd::below(w, 6) {
        0 => CraftResource::BlackScales,
        1 => CraftResource::RedScales,
        2 => CraftResource::BlueScales,
        3 => CraftResource::YellowScales,
        4 => CraftResource::GreenScales,
        _ => CraftResource::WhiteScales,
    }
}

fn scales_name(res: CraftResource) -> &'static str {
    match res {
        CraftResource::BlackScales => "BlackScales",
        CraftResource::RedScales => "RedScales",
        CraftResource::BlueScales => "BlueScales",
        CraftResource::YellowScales => "YellowScales",
        CraftResource::GreenScales => "GreenScales",
        _ => "WhiteScales",
    }
}

/// `AddItem(new Dragon<piece> { Resource = res, Movable = false })`.
fn add_dragon_piece(w: &mut World, id: MobileId, name: &str, res: CraftResource) {
    if let Some(it) = add_fixed(w, id, name, None) {
        armor::set_resource(w, it, res);
    }
}

fn add_random_melee(w: &mut World, id: MobileId) {
    let name = match rnd::below(w, 3) {
        0 => "Kryss",
        1 => "Broadsword",
        _ => "Katana",
    };
    add_fixed(w, id, name, None);
}

fn add_scales(w: &mut World, id: MobileId, res: CraftResource) {
    let amount = rnd::min_max(w, 1, 3);
    if let Some(it) = factory::make_item_amount(w, &si(scales_name(res)), amount) {
        items::add_item(w, id, it);
    }
}

pub fn chaos_dragoon_ctor2(w: &mut World, id: MobileId) {
    let res = random_scale_resource(w);
    add_random_melee(w, id);
    for p in [
        "DragonHelm",
        "DragonChest",
        "DragonArms",
        "DragonGloves",
        "DragonLegs",
    ] {
        add_dragon_piece(w, id, p, res);
    }
    add_fixed(w, id, "ChaosShield", None);
    add(w, id, "Shirt");
    add(w, id, "Boots");
    add_scales(w, id, res);
    if let Some(m) = factory::make_mobile(w, "Server.Mobiles.SwampDragon") {
        mount::set_rider(w, m, Some(id));
    }
}

pub fn chaos_dragoon_elite_ctor2(w: &mut World, id: MobileId) {
    let res = random_scale_resource(w);
    add_random_melee(w, id);
    for p in [
        "DragonChest",
        "DragonLegs",
        "DragonArms",
        "DragonGloves",
        "DragonHelm",
    ] {
        add_dragon_piece(w, id, p, res);
    }
    add_fixed(w, id, "ChaosShield", None);
    add_hue(w, id, "Boots", 0x455);
    let hue = rnd::from(w, 2401, 30); // Utility.RandomMetalHue()
    add_hue(w, id, "Shirt", hue);
    add_scales(w, id, res);
    let barding = match rnd::below(w, 9) {
        0 => CraftResource::DullCopper,
        1 => CraftResource::ShadowIron,
        2 => CraftResource::Copper,
        3 => CraftResource::Bronze,
        4 => CraftResource::Gold,
        5 => CraftResource::Agapite,
        6 => CraftResource::Verite,
        7 => CraftResource::Valorite,
        _ => CraftResource::Iron,
    };
    if let Some(m) = factory::make_mobile(w, "Server.Mobiles.SwampDragon") {
        crate::classes::swamp_dragon::equip_barding(w, m, barding);
        mount::set_rider(w, m, Some(id));
    }
}
