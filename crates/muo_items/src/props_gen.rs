//! `[CommandProperty]` tables of classes whose members are fields of a data struct in `ext`.
#![allow(
    unused_imports,
    unused_variables,
    dead_code,
    redundant_semicolons,
    clippy::all
)]
use muo_content::aos::AosElementAttribute;
use muo_content::props::{self, Kind, PropDef, Value};
use muo_content::utilities::types::{Numeric, PropType};
use muo_primitives::{DateTime, Point2D, Point3D, Rectangle2D, TimeSpan};
use muo_server::ids::EntityId;
use muo_server::mobile::AccessLevel;
use muo_server::world::World;

const PLAYER: AccessLevel = AccessLevel::Player;
const COUNSELOR: AccessLevel = AccessLevel::Counselor;
const GM: AccessLevel = AccessLevel::GameMaster;
const SEER: AccessLevel = AccessLevel::Seer;
const ADMIN: AccessLevel = AccessLevel::Administrator;
const DEV: AccessLevel = AccessLevel::Developer;
const OWNER: AccessLevel = AccessLevel::Owner;
const BOOL: Kind = Kind::Plain(PropType::Bool);
const STR: Kind = Kind::Plain(PropType::Str);
const ENTITY: Kind = Kind::Plain(PropType::Entity);
const NOT_HERE: &str = "That object has no such state.";
const FACTION: Kind = Kind::Nullable(&Kind::CustomEnum(
    &[
        "Minax",
        "Council of Mages",
        "True Britannians",
        "Shadowlords",
    ],
    &[
        ("Council of Mages", 0),
        ("Minax", 1),
        ("Shadowlords", 2),
        ("True Britannians", 3),
    ],
));
const TOWN: Kind = Kind::Nullable(&Kind::CustomEnum(
    &[
        "Britain",
        "Magincia",
        "Minoc",
        "Moonglow",
        "Skara Brae",
        "Trinsic",
        "Vesper",
        "Yew",
    ],
    &[
        ("Britain", 0),
        ("Magincia", 1),
        ("Minoc", 2),
        ("Moonglow", 3),
        ("Skara Brae", 4),
        ("Trinsic", 5),
        ("Vesper", 6),
        ("Yew", 7),
    ],
));

const E_ACCESSLEVEL: &[(&str, i64)] = &[
    ("Player", 0),
    ("Counselor", 1),
    ("GameMaster", 2),
    ("Seer", 3),
    ("Administrator", 4),
    ("Developer", 5),
    ("Owner", 6),
];
const E_AOSELEMENTATTRIBUTE: &[(&str, i64)] = &[
    ("Physical", 1),
    ("Fire", 2),
    ("Cold", 4),
    ("Poison", 8),
    ("Energy", 16),
    ("Chaos", 32),
    ("Direct", 64),
];
const E_BEVERAGETYPE: &[(&str, i64)] = &[
    ("Ale", 0),
    ("Cider", 1),
    ("Liquor", 2),
    ("Milk", 3),
    ("Wine", 4),
    ("Water", 5),
];
const E_BOOKQUALITY: &[(&str, i64)] = &[("Regular", 0), ("Exceptional", 1)];
const E_CAMPFIRESTATUS: &[(&str, i64)] = &[("Burning", 0), ("Extinguishing", 1), ("Off", 2)];
const E_CANNONDIRECTION: &[(&str, i64)] = &[("North", 0), ("East", 1), ("South", 2), ("West", 3)];
const E_CLOTHINGQUALITY: &[(&str, i64)] = &[("Low", 0), ("Regular", 1), ("Exceptional", 2)];
const E_CRAFTRESOURCE: &[(&str, i64)] = &[
    ("None", 0),
    ("Iron", 1),
    ("DullCopper", 2),
    ("ShadowIron", 3),
    ("Copper", 4),
    ("Bronze", 5),
    ("Gold", 6),
    ("Agapite", 7),
    ("Verite", 8),
    ("Valorite", 9),
    ("RegularLeather", 101),
    ("SpinedLeather", 102),
    ("HornedLeather", 103),
    ("BarbedLeather", 104),
    ("RedScales", 201),
    ("YellowScales", 202),
    ("BlackScales", 203),
    ("GreenScales", 204),
    ("WhiteScales", 205),
    ("BlueScales", 206),
    ("RegularWood", 301),
    ("OakWood", 302),
    ("AshWood", 303),
    ("YewWood", 304),
    ("Heartwood", 305),
    ("Bloodwood", 306),
    ("Frostwood", 307),
];
const E_FILLABLECONTENTTYPE: &[(&str, i64)] = &[
    ("None", -1),
    ("Weaponsmith", 0),
    ("Provisioner", 1),
    ("Mage", 2),
    ("Alchemist", 3),
    ("Armorer", 4),
    ("ArtisanGuild", 5),
    ("Baker", 6),
    ("Bard", 7),
    ("Blacksmith", 8),
    ("Bowyer", 9),
    ("Butcher", 10),
    ("Carpenter", 11),
    ("Clothier", 12),
    ("Cobbler", 13),
    ("Docks", 14),
    ("Farm", 15),
    ("FighterGuild", 16),
    ("Guard", 17),
    ("Healer", 18),
    ("Herbalist", 19),
    ("Inn", 20),
    ("Jeweler", 21),
    ("Library", 22),
    ("Merchant", 23),
    ("Mill", 24),
    ("Mine", 25),
    ("Observatory", 26),
    ("Painter", 27),
    ("Ranger", 28),
    ("Stables", 29),
    ("Tanner", 30),
    ("Tavern", 31),
    ("ThiefGuild", 32),
    ("Tinker", 33),
    ("Veterinarian", 34),
];
const E_HEADTYPE: &[(&str, i64)] = &[("Regular", 0), ("Duel", 1), ("Tournament", 2)];
const E_HOUSERAFFLEEXPIREACTION: &[(&str, i64)] =
    &[("None", 0), ("HideStone", 1), ("DeleteStone", 2)];
const E_HOUSERAFFLESTATE: &[(&str, i64)] = &[("Inactive", 0), ("Active", 1), ("Completed", 2)];
const E_INSTRUMENTQUALITY: &[(&str, i64)] = &[("Low", 0), ("Regular", 1), ("Exceptional", 2)];
const E_MINIHOUSETYPE: &[(&str, i64)] = &[
    ("StoneAndPlaster", 0),
    ("FieldStone", 1),
    ("SmallBrick", 2),
    ("Wooden", 3),
    ("WoodAndPlaster", 4),
    ("ThatchedRoof", 5),
    ("Brick", 6),
    ("TwoStoryWoodAndPlaster", 7),
    ("TwoStoryStoneAndPlaster", 8),
    ("Tower", 9),
    ("SmallStoneKeep", 10),
    ("Castle", 11),
    ("LargeHouseWithPatio", 12),
    ("MarbleHouseWithPatio", 13),
    ("SmallStoneTower", 14),
    ("TwoStoryLogCabin", 15),
    ("TwoStoryVilla", 16),
    ("SandstoneHouseWithPatio", 17),
    ("SmallStoneWorkshop", 18),
    ("SmallMarbleWorkshop", 19),
    ("MalasMountainPass", 20),
    ("ChurchAtNight", 21),
];
const E_MININGCARTTYPE: &[(&str, i64)] = &[
    ("OreSouth", 100),
    ("OreEast", 101),
    ("GemSouth", 102),
    ("GemEast", 103),
];
const E_MONSTERSTATUETTETYPE: &[(&str, i64)] = &[
    ("Crocodile", 0),
    ("Daemon", 1),
    ("Dragon", 2),
    ("EarthElemental", 3),
    ("Ettin", 4),
    ("Gargoyle", 5),
    ("Gorilla", 6),
    ("Lich", 7),
    ("Lizardman", 8),
    ("Ogre", 9),
    ("Orc", 10),
    ("Ratman", 11),
    ("Skeleton", 12),
    ("Troll", 13),
    ("Cow", 14),
    ("Zombie", 15),
    ("Llama", 16),
    ("Ophidian", 17),
    ("Reaper", 18),
    ("Mongbat", 19),
    ("Gazer", 20),
    ("FireElemental", 21),
    ("Wolf", 22),
    ("PhillipsWoodenSteed", 23),
    ("Seahorse", 24),
    ("Harrower", 25),
    ("Efreet", 26),
    ("Slime", 27),
    ("PlagueBeast", 28),
    ("RedDeath", 29),
    ("Spider", 30),
    ("OphidianArchMage", 31),
    ("OphidianWarrior", 32),
    ("OphidianKnight", 33),
    ("OphidianMage", 34),
    ("DreadHorn", 35),
    ("Minotaur", 36),
    ("BlackCat", 37),
    ("HalloweenGhoul", 38),
    ("Santa", 39),
];
const E_MOONSTONETYPE: &[(&str, i64)] = &[("Felucca", 0), ("Trammel", 1)];
const E_POTIONEFFECT: &[(&str, i64)] = &[
    ("Nightsight", 0),
    ("CureLesser", 1),
    ("Cure", 2),
    ("CureGreater", 3),
    ("Agility", 4),
    ("AgilityGreater", 5),
    ("Strength", 6),
    ("StrengthGreater", 7),
    ("PoisonLesser", 8),
    ("Poison", 9),
    ("PoisonGreater", 10),
    ("PoisonDeadly", 11),
    ("Refresh", 12),
    ("RefreshTotal", 13),
    ("HealLesser", 14),
    ("Heal", 15),
    ("HealGreater", 16),
    ("ExplosionLesser", 17),
    ("Explosion", 18),
    ("ExplosionGreater", 19),
    ("Conflagration", 20),
    ("ConflagrationGreater", 21),
    ("MaskOfDeath", 22),
    ("MaskOfDeathGreater", 23),
    ("ConfusionBlast", 24),
    ("ConfusionBlastGreater", 25),
    ("Invisibility", 26),
    ("Parasitic", 27),
    ("Darkglow", 28),
    ("FlintsPungentBrew", 29),
];
const E_SECURELEVEL: &[(&str, i64)] = &[
    ("Owner", 0),
    ("CoOwners", 1),
    ("Friends", 2),
    ("Anyone", 3),
    ("Guild", 4),
];
const E_SKILLNAME: &[(&str, i64)] = &[
    ("Alchemy", 0),
    ("Anatomy", 1),
    ("AnimalLore", 2),
    ("ItemID", 3),
    ("ArmsLore", 4),
    ("Parry", 5),
    ("Begging", 6),
    ("Blacksmith", 7),
    ("Fletching", 8),
    ("Peacemaking", 9),
    ("Camping", 10),
    ("Carpentry", 11),
    ("Cartography", 12),
    ("Cooking", 13),
    ("DetectHidden", 14),
    ("Discordance", 15),
    ("EvalInt", 16),
    ("Healing", 17),
    ("Fishing", 18),
    ("Forensics", 19),
    ("Herding", 20),
    ("Hiding", 21),
    ("Provocation", 22),
    ("Inscribe", 23),
    ("Lockpicking", 24),
    ("Magery", 25),
    ("MagicResist", 26),
    ("Tactics", 27),
    ("Snooping", 28),
    ("Musicianship", 29),
    ("Poisoning", 30),
    ("Archery", 31),
    ("SpiritSpeak", 32),
    ("Stealing", 33),
    ("Tailoring", 34),
    ("AnimalTaming", 35),
    ("TasteID", 36),
    ("Tinkering", 37),
    ("Tracking", 38),
    ("Veterinary", 39),
    ("Swords", 40),
    ("Macing", 41),
    ("Fencing", 42),
    ("Wrestling", 43),
    ("Lumberjacking", 44),
    ("Mining", 45),
    ("Meditation", 46),
    ("Stealth", 47),
    ("RemoveTrap", 48),
    ("Necromancy", 49),
    ("Focus", 50),
    ("Chivalry", 51),
    ("Bushido", 52),
    ("Ninjitsu", 53),
    ("Spellweaving", 54),
    ("Mysticism", 55),
    ("Imbuing", 56),
    ("Throwing", 57),
];
const E_SLAYERNAME: &[(&str, i64)] = &[
    ("None", 0),
    ("Silver", 1),
    ("OrcSlaying", 2),
    ("TrollSlaughter", 3),
    ("OgreTrashing", 4),
    ("Repond", 5),
    ("DragonSlaying", 6),
    ("Terathan", 7),
    ("SnakesBane", 8),
    ("LizardmanSlaughter", 9),
    ("ReptilianDeath", 10),
    ("DaemonDismissal", 11),
    ("GargoylesFoe", 12),
    ("BalronDamnation", 13),
    ("Exorcism", 14),
    ("Ophidian", 15),
    ("SpidersDeath", 16),
    ("ScorpionsBane", 17),
    ("ArachnidDoom", 18),
    ("FlameDousing", 19),
    ("WaterDissipation", 20),
    ("Vacuum", 21),
    ("ElementalHealth", 22),
    ("EarthShatter", 23),
    ("BloodDrinking", 24),
    ("SummerWind", 25),
    ("ElementalBan", 26),
    ("Fey", 27),
];
const E_SNOWGLOBETYPEONE: &[(&str, i64)] = &[
    ("Britain", 0),
    ("Moonglow", 1),
    ("Minoc", 2),
    ("Magincia", 3),
    ("BuccaneersDen", 4),
    ("Trinsic", 5),
    ("Yew", 6),
    ("SkaraBrae", 7),
    ("Jhelom", 8),
    ("Nujelm", 9),
    ("Papua", 10),
    ("Delucia", 11),
    ("Cove", 12),
    ("Ocllo", 13),
    ("SerpentsHold", 14),
    ("EmpathAbbey", 15),
    ("TheLycaeum", 16),
    ("Vesper", 17),
    ("Wind", 18),
];
const E_TALISMANREMOVAL: &[(&str, i64)] = &[
    ("None", 0),
    ("Ward", 390),
    ("Damage", 404),
    ("Curse", 407),
    ("Wildfire", 2843),
];
const E_TALISMANSLAYERNAME: &[(&str, i64)] = &[
    ("None", 0),
    ("Bear", 1),
    ("Vermin", 2),
    ("Bat", 3),
    ("Mage", 4),
    ("Beetle", 5),
    ("Bird", 6),
    ("Ice", 7),
    ("Flame", 8),
    ("Bovine", 9),
];
const E_WANDEFFECT: &[(&str, i64)] = &[
    ("Clumsiness", 0),
    ("Identification", 1),
    ("Healing", 2),
    ("Feeblemindedness", 3),
    ("Weakness", 4),
    ("MagicArrow", 5),
    ("Harming", 6),
    ("Fireball", 7),
    ("GreaterHealing", 8),
    ("Lightning", 9),
    ("ManaDraining", 10),
];
static SERVER_ITEMS_FUKIYA: &[PropDef] = &[
    PropDef {
        name: "UsesRemaining",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::skill_items::ninja::NinjaData>())
                .map_or(Value::Null, |d| {
                    let x = &d.uses_remaining;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            crate::skill_items::ninja::set_uses_remaining(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "Poison",
        kind: Kind::Poison,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::skill_items::ninja::NinjaData>())
                .map_or(Value::Null, |d| {
                    let x = &d.poison;
                    Value::Poison(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match v {
                Value::Poison(p) => p,
                _ => None,
            };
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::skill_items::ninja::NinjaData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.poison == n {
                return Ok(());
            }
            d.poison = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "PoisonCharges",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::skill_items::ninja::NinjaData>())
                .map_or(Value::Null, |d| {
                    let x = &d.poison_charges;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::skill_items::ninja::NinjaData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.poison_charges == n {
                return Ok(());
            }
            d.poison_charges = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
];

static SERVER_ITEMS_TROPHYADDON: &[PropDef] = &[
    PropDef {
        name: "Hunter",
        kind: STR,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::skill_items::trophy::TrophyData>())
                .map_or(Value::Null, |d| {
                    let x = &d.hunter;
                    x.clone().map_or(Value::Null, Value::Str)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.string();
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::skill_items::trophy::TrophyData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.hunter == n {
                return Ok(());
            }
            d.hunter = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "AnimalWeight",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::skill_items::trophy::TrophyData>())
                .map_or(Value::Null, |d| {
                    let x = &d.animal_weight;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::skill_items::trophy::TrophyData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.animal_weight == n {
                return Ok(());
            }
            d.animal_weight = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "WestId",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::skill_items::trophy::TrophyData>())
                .map_or(Value::Null, |d| {
                    let x = &d.west_id;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::skill_items::trophy::TrophyData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.west_id == n {
                return Ok(());
            }
            d.west_id = n;
            Ok(())
        }),
    },
    PropDef {
        name: "NorthId",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::skill_items::trophy::TrophyData>())
                .map_or(Value::Null, |d| {
                    let x = &d.north_id;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::skill_items::trophy::TrophyData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.north_id == n {
                return Ok(());
            }
            d.north_id = n;
            Ok(())
        }),
    },
    PropDef {
        name: "DeedNumber",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::skill_items::trophy::TrophyData>())
                .map_or(Value::Null, |d| {
                    let x = &d.deed_number;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::skill_items::trophy::TrophyData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.deed_number == n {
                return Ok(());
            }
            d.deed_number = n;
            Ok(())
        }),
    },
    PropDef {
        name: "AddonNumber",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::skill_items::trophy::TrophyData>())
                .map_or(Value::Null, |d| {
                    let x = &d.addon_number;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::skill_items::trophy::TrophyData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.addon_number == n {
                return Ok(());
            }
            d.addon_number = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
];

static SERVER_ITEMS_TROPHYDEED: &[PropDef] = &[
    PropDef {
        name: "Hunter",
        kind: STR,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::skill_items::trophy::TrophyData>())
                .map_or(Value::Null, |d| {
                    let x = &d.hunter;
                    x.clone().map_or(Value::Null, Value::Str)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.string();
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::skill_items::trophy::TrophyData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.hunter == n {
                return Ok(());
            }
            d.hunter = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "AnimalWeight",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::skill_items::trophy::TrophyData>())
                .map_or(Value::Null, |d| {
                    let x = &d.animal_weight;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::skill_items::trophy::TrophyData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.animal_weight == n {
                return Ok(());
            }
            d.animal_weight = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "WestId",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::skill_items::trophy::TrophyData>())
                .map_or(Value::Null, |d| {
                    let x = &d.west_id;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::skill_items::trophy::TrophyData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.west_id == n {
                return Ok(());
            }
            d.west_id = n;
            Ok(())
        }),
    },
    PropDef {
        name: "NorthId",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::skill_items::trophy::TrophyData>())
                .map_or(Value::Null, |d| {
                    let x = &d.north_id;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::skill_items::trophy::TrophyData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.north_id == n {
                return Ok(());
            }
            d.north_id = n;
            Ok(())
        }),
    },
    PropDef {
        name: "DeedNumber",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::skill_items::trophy::TrophyData>())
                .map_or(Value::Null, |d| {
                    let x = &d.deed_number;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::skill_items::trophy::TrophyData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.deed_number == n {
                return Ok(());
            }
            d.deed_number = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "AddonNumber",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::skill_items::trophy::TrophyData>())
                .map_or(Value::Null, |d| {
                    let x = &d.addon_number;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::skill_items::trophy::TrophyData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.addon_number == n {
                return Ok(());
            }
            d.addon_number = n;
            Ok(())
        }),
    },
];

static SERVER_ITEMS_FURNITUREDYETUB: &[PropDef] = &[PropDef {
    name: "IsRewardItem",
    kind: BOOL,
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| o.ext.get::<crate::skill_items::dyes::TubData>())
            .map_or(Value::Null, |d| {
                let x = &d.is_reward;
                Value::Bool(*x)
            }),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.bool();
        let Some(d) = w
            .item_mut(i)
            .and_then(|o| o.ext.get_mut::<crate::skill_items::dyes::TubData>())
        else {
            return Err(NOT_HERE.to_string());
        };
        if d.is_reward == n {
            return Ok(());
        }
        d.is_reward = n;
        Ok(())
    }),
}];

static SERVER_ITEMS_BASEQUIVER: &[PropDef] = &[
    PropDef {
        name: "LowerAmmoCost",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<muo_content::items::ext_bags::QuiverData>())
                .map_or(Value::Null, |d| {
                    let x = &d.lower_ammo_cost;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<muo_content::items::ext_bags::QuiverData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.lower_ammo_cost == n {
                return Ok(());
            }
            d.lower_ammo_cost = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "WeightReduction",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<muo_content::items::ext_bags::QuiverData>())
                .map_or(Value::Null, |d| {
                    let x = &d.weight_reduction;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<muo_content::items::ext_bags::QuiverData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.weight_reduction == n {
                return Ok(());
            }
            d.weight_reduction = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "DamageIncrease",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<muo_content::items::ext_bags::QuiverData>())
                .map_or(Value::Null, |d| {
                    let x = &d.damage_increase;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<muo_content::items::ext_bags::QuiverData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.damage_increase == n {
                return Ok(());
            }
            d.damage_increase = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "Crafter",
        kind: STR,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<muo_content::items::ext_bags::QuiverData>())
                .map_or(Value::Null, |d| {
                    let x = &d.crafter;
                    x.clone().map_or(Value::Null, Value::Str)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.string();
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<muo_content::items::ext_bags::QuiverData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.crafter == n {
                return Ok(());
            }
            d.crafter = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "Quality",
        kind: Kind::Plain(PropType::Enum(E_CLOTHINGQUALITY)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<muo_content::items::ext_bags::QuiverData>())
                .map_or(Value::Null, |d| {
                    let x = &d.quality;
                    Value::Enum(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<muo_content::items::ext_bags::QuiverData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.quality == n {
                return Ok(());
            }
            d.quality = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "Capacity",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<muo_content::items::ext_bags::QuiverData>())
                .map_or(Value::Null, |d| {
                    let x = &d.capacity;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<muo_content::items::ext_bags::QuiverData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.capacity == n {
                return Ok(());
            }
            d.capacity = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
];

static SERVER_ITEMS_BASETALISMAN: &[PropDef] = &[
    PropDef {
        name: "Removal",
        kind: Kind::Plain(PropType::Enum(E_TALISMANREMOVAL)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<muo_content::items::ext_bags::TalismanData>())
                .map_or(Value::Null, |d| {
                    let x = &d.removal;
                    Value::Enum(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w.item_mut(i).and_then(|o| {
                o.ext
                    .get_mut::<muo_content::items::ext_bags::TalismanData>()
            }) else {
                return Err(NOT_HERE.to_string());
            };
            if d.removal == n {
                return Ok(());
            }
            d.removal = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "Skill",
        kind: Kind::Plain(PropType::Enum(E_SKILLNAME)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<muo_content::items::ext_bags::TalismanData>())
                .map_or(Value::Null, |d| {
                    let x = &d.skill;
                    Value::Enum(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w.item_mut(i).and_then(|o| {
                o.ext
                    .get_mut::<muo_content::items::ext_bags::TalismanData>()
            }) else {
                return Err(NOT_HERE.to_string());
            };
            if d.skill == n {
                return Ok(());
            }
            d.skill = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "SuccessBonus",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<muo_content::items::ext_bags::TalismanData>())
                .map_or(Value::Null, |d| {
                    let x = &d.success_bonus;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w.item_mut(i).and_then(|o| {
                o.ext
                    .get_mut::<muo_content::items::ext_bags::TalismanData>()
            }) else {
                return Err(NOT_HERE.to_string());
            };
            if d.success_bonus == n {
                return Ok(());
            }
            d.success_bonus = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "ExceptionalBonus",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<muo_content::items::ext_bags::TalismanData>())
                .map_or(Value::Null, |d| {
                    let x = &d.exceptional_bonus;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w.item_mut(i).and_then(|o| {
                o.ext
                    .get_mut::<muo_content::items::ext_bags::TalismanData>()
            }) else {
                return Err(NOT_HERE.to_string());
            };
            if d.exceptional_bonus == n {
                return Ok(());
            }
            d.exceptional_bonus = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "MaxCharges",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<muo_content::items::ext_bags::TalismanData>())
                .map_or(Value::Null, |d| {
                    let x = &d.max_charges;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w.item_mut(i).and_then(|o| {
                o.ext
                    .get_mut::<muo_content::items::ext_bags::TalismanData>()
            }) else {
                return Err(NOT_HERE.to_string());
            };
            if d.max_charges == n {
                return Ok(());
            }
            d.max_charges = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "MaxChargeTime",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<muo_content::items::ext_bags::TalismanData>())
                .map_or(Value::Null, |d| {
                    let x = &d.max_charge_time;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w.item_mut(i).and_then(|o| {
                o.ext
                    .get_mut::<muo_content::items::ext_bags::TalismanData>()
            }) else {
                return Err(NOT_HERE.to_string());
            };
            if d.max_charge_time == n {
                return Ok(());
            }
            d.max_charge_time = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "Blessed",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<muo_content::items::ext_bags::TalismanData>())
                .map_or(Value::Null, |d| {
                    let x = &d.blessed;
                    Value::Bool(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.bool();
            let Some(d) = w.item_mut(i).and_then(|o| {
                o.ext
                    .get_mut::<muo_content::items::ext_bags::TalismanData>()
            }) else {
                return Err(NOT_HERE.to_string());
            };
            if d.blessed == n {
                return Ok(());
            }
            d.blessed = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "Slayer",
        kind: Kind::Plain(PropType::Enum(E_TALISMANSLAYERNAME)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<muo_content::items::ext_bags::TalismanData>())
                .map_or(Value::Null, |d| {
                    let x = &d.slayer;
                    Value::Enum(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w.item_mut(i).and_then(|o| {
                o.ext
                    .get_mut::<muo_content::items::ext_bags::TalismanData>()
            }) else {
                return Err(NOT_HERE.to_string());
            };
            if d.slayer == n {
                return Ok(());
            }
            d.slayer = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "Charges",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<muo_content::items::ext_bags::TalismanData>())
                .map_or(Value::Null, |d| {
                    let x = &d.charges;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            crate::talismans::set_charges(w, i, n);
            Ok(())
        }),
    },
];

static SERVER_ITEMS_SPELLBOOK: &[PropDef] = &[
    PropDef {
        name: "Quality",
        kind: Kind::Plain(PropType::Enum(E_BOOKQUALITY)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<muo_content::items::ext_bags::SpellbookData>())
                .map_or(Value::Null, |d| {
                    let x = &d.quality;
                    Value::Enum(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w.item_mut(i).and_then(|o| {
                o.ext
                    .get_mut::<muo_content::items::ext_bags::SpellbookData>()
            }) else {
                return Err(NOT_HERE.to_string());
            };
            if d.quality == n {
                return Ok(());
            }
            d.quality = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "EngravedText",
        kind: STR,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<muo_content::items::ext_bags::SpellbookData>())
                .map_or(Value::Null, |d| {
                    let x = &d.engraved_text;
                    x.clone().map_or(Value::Null, Value::Str)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.string();
            let Some(d) = w.item_mut(i).and_then(|o| {
                o.ext
                    .get_mut::<muo_content::items::ext_bags::SpellbookData>()
            }) else {
                return Err(NOT_HERE.to_string());
            };
            if d.engraved_text == n {
                return Ok(());
            }
            d.engraved_text = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "Crafter",
        kind: STR,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<muo_content::items::ext_bags::SpellbookData>())
                .map_or(Value::Null, |d| {
                    let x = &d.crafter;
                    x.clone().map_or(Value::Null, Value::Str)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.string();
            let Some(d) = w.item_mut(i).and_then(|o| {
                o.ext
                    .get_mut::<muo_content::items::ext_bags::SpellbookData>()
            }) else {
                return Err(NOT_HERE.to_string());
            };
            if d.crafter == n {
                return Ok(());
            }
            d.crafter = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "Slayer",
        kind: Kind::Plain(PropType::Enum(E_SLAYERNAME)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<muo_content::items::ext_bags::SpellbookData>())
                .map_or(Value::Null, |d| {
                    let x = &d.slayer;
                    Value::Enum(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w.item_mut(i).and_then(|o| {
                o.ext
                    .get_mut::<muo_content::items::ext_bags::SpellbookData>()
            }) else {
                return Err(NOT_HERE.to_string());
            };
            if d.slayer == n {
                return Ok(());
            }
            d.slayer = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "Slayer2",
        kind: Kind::Plain(PropType::Enum(E_SLAYERNAME)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<muo_content::items::ext_bags::SpellbookData>())
                .map_or(Value::Null, |d| {
                    let x = &d.slayer2;
                    Value::Enum(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w.item_mut(i).and_then(|o| {
                o.ext
                    .get_mut::<muo_content::items::ext_bags::SpellbookData>()
            }) else {
                return Err(NOT_HERE.to_string());
            };
            if d.slayer2 == n {
                return Ok(());
            }
            d.slayer2 = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "SpellCount",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<muo_content::items::ext_bags::SpellbookData>())
                .map_or(Value::Null, |d| {
                    let x = &d.spell_count;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w.item_mut(i).and_then(|o| {
                o.ext
                    .get_mut::<muo_content::items::ext_bags::SpellbookData>()
            }) else {
                return Err(NOT_HERE.to_string());
            };
            if d.spell_count == n {
                return Ok(());
            }
            d.spell_count = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "Content",
        kind: Kind::Plain(PropType::Numeric(Numeric::ULong)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<muo_content::items::ext_bags::SpellbookData>())
                .map_or(Value::Null, |d| {
                    let x = &d.content;
                    Value::Int(*x as i64)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as u64;
            crate::skill_items::spellbook::set_content(w, i, n);
            Ok(())
        }),
    },
];

static SERVER_ITEMS_TREASUREMAP: &[PropDef] = &[
    PropDef {
        name: "CompletedBy",
        kind: Kind::Plain(PropType::EntityOf("Mobile")),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::maps::MapItemData>())
                .map_or(Value::Null, |d| {
                    d.treasure.as_ref().map_or(Value::Null, |t| {
                        Value::Entity(t.completed_by.map(EntityId::Mobile))
                    })
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::maps::MapItemData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if let Some(t) = d.treasure.as_mut() {
                t.completed_by = v.mobile();
            }
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "Level",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::maps::MapItemData>())
                .map_or(Value::Null, |d| {
                    d.treasure
                        .as_ref()
                        .map_or(Value::Null, |t| Value::Int(i64::from(t.level)))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::maps::MapItemData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if let Some(t) = d.treasure.as_mut() {
                t.level = v.i32();
            }
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "Completed",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::maps::MapItemData>())
                .map_or(Value::Null, |d| {
                    d.treasure
                        .as_ref()
                        .map_or(Value::Null, |t| Value::Bool(t.completed))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::maps::MapItemData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if let Some(t) = d.treasure.as_mut() {
                t.completed = v.bool();
            }
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "Decoder",
        kind: Kind::Plain(PropType::EntityOf("Mobile")),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::maps::MapItemData>())
                .map_or(Value::Null, |d| {
                    d.treasure.as_ref().map_or(Value::Null, |t| {
                        Value::Entity(t.decoder.map(EntityId::Mobile))
                    })
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::maps::MapItemData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if let Some(t) = d.treasure.as_mut() {
                t.decoder = v.mobile();
            }
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "ChestMap",
        kind: Kind::Map,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::maps::MapItemData>())
                .map_or(Value::Null, |d| {
                    d.treasure
                        .as_ref()
                        .map_or(Value::Null, |t| Value::Map(t.chest_map))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::maps::MapItemData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if let Some(t) = d.treasure.as_mut() {
                t.chest_map = v.map();
            }
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
];

static SERVER_ITEMS_AQUARIUM: &[PropDef] = &[
    PropDef {
        name: "LiveCreatures",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::special::aquarium::AquariumData>())
                .map_or(Value::Null, |d| {
                    let x = &d.live_creatures;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::special::aquarium::AquariumData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.live_creatures == n {
                return Ok(());
            }
            d.live_creatures = n;
            Ok(())
        }),
    },
    PropDef {
        name: "VacationLeft",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::special::aquarium::AquariumData>())
                .map_or(Value::Null, |d| {
                    let x = &d.vacation_left;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            crate::special::aquarium::set_vacation_left(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "RewardAvailable",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::special::aquarium::AquariumData>())
                .map_or(Value::Null, |d| {
                    let x = &d.reward_available;
                    Value::Bool(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.bool();
            crate::special::aquarium::set_reward_available(w, i, n);
            Ok(())
        }),
    },
];

static SERVER_ITEMS_HEAD: &[PropDef] = &[
    PropDef {
        name: "PlayerName",
        kind: STR,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::body_parts::parts::HeadData>())
                .map_or(Value::Null, |d| {
                    let x = &d.player_name;
                    x.clone().map_or(Value::Null, Value::Str)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.string();
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::body_parts::parts::HeadData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.player_name == n {
                return Ok(());
            }
            d.player_name = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "HeadType",
        kind: Kind::Plain(PropType::Enum(E_HEADTYPE)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::body_parts::parts::HeadData>())
                .map_or(Value::Null, |d| {
                    let x = &d.head_type;
                    Value::Enum(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::body_parts::parts::HeadData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.head_type == n {
                return Ok(());
            }
            d.head_type = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "BountyTarget",
        kind: Kind::Plain(PropType::EntityOf("PlayerMobile")),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::body_parts::parts::HeadData>())
                .map_or(Value::Null, |d| {
                    let x = &d.bounty_target;
                    Value::Entity(x.map(EntityId::Mobile))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.mobile();
            crate::body_parts::parts::set_bounty_target(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "CarvedTime",
        kind: Kind::DateTime,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::body_parts::parts::HeadData>())
                .map_or(Value::Null, |d| {
                    let x = &d.carved_time;
                    Value::DateTime(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match v {
                Value::DateTime(t) => t,
                _ => DateTime::MIN,
            };
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::body_parts::parts::HeadData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.carved_time == n {
                return Ok(());
            }
            d.carved_time = n;
            Ok(())
        }),
    },
];

static SERVER_ITEMS_BASEBOOK: &[PropDef] = &[
    PropDef {
        name: "Level",
        kind: Kind::Plain(PropType::Enum(E_SECURELEVEL)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::books::BookData>())
                .map_or(Value::Null, |d| {
                    let x = &d.level;
                    Value::Enum(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::books::BookData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.level == n {
                return Ok(());
            }
            d.level = n;
            Ok(())
        }),
    },
    PropDef {
        name: "Title",
        kind: STR,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::books::BookData>())
                .map_or(Value::Null, |d| {
                    let x = &d.title;
                    x.clone().map_or(Value::Null, Value::Str)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.string();
            crate::books::set_title(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "Author",
        kind: STR,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::books::BookData>())
                .map_or(Value::Null, |d| {
                    let x = &d.author;
                    x.clone().map_or(Value::Null, Value::Str)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.string();
            crate::books::set_author(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "Writable",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::books::BookData>())
                .map_or(Value::Null, |d| {
                    let x = &d.writable;
                    Value::Bool(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.bool();
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::books::BookData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.writable == n {
                return Ok(());
            }
            d.writable = n;
            Ok(())
        }),
    },
];

static SERVER_ITEMS_BASEBULLETINBOARD: &[PropDef] = &[PropDef {
    name: "BoardName",
    kind: STR,
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| o.ext.get::<crate::bulletin::BoardData>())
            .map_or(Value::Null, |d| {
                let x = &d.board_name;
                x.clone().map_or(Value::Null, Value::Str)
            }),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.string();
        let Some(d) = w
            .item_mut(i)
            .and_then(|o| o.ext.get_mut::<crate::bulletin::BoardData>())
        else {
            return Err(NOT_HERE.to_string());
        };
        if d.board_name == n {
            return Ok(());
        }
        d.board_name = n;
        Ok(())
    }),
}];

static SERVER_ITEMS_BULLETINMESSAGE: &[PropDef] = &[
    PropDef {
        name: "Poster",
        kind: Kind::Plain(PropType::EntityOf("Mobile")),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::bulletin::MessageData>())
                .map_or(Value::Null, |d| {
                    let x = &d.poster;
                    Value::Entity(x.map(EntityId::Mobile))
                }),
            _ => Value::Null,
        },
        set: None,
    },
    PropDef {
        name: "Subject",
        kind: STR,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::bulletin::MessageData>())
                .map_or(Value::Null, |d| {
                    let x = &d.subject;
                    x.clone().map_or(Value::Null, Value::Str)
                }),
            _ => Value::Null,
        },
        set: None,
    },
    PropDef {
        name: "Time",
        kind: Kind::DateTime,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::bulletin::MessageData>())
                .map_or(Value::Null, |d| {
                    let x = &d.time;
                    Value::DateTime(*x)
                }),
            _ => Value::Null,
        },
        set: None,
    },
    PropDef {
        name: "Thread",
        kind: Kind::Plain(PropType::EntityOf("BulletinMessage")),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::bulletin::MessageData>())
                .map_or(Value::Null, |d| {
                    let x = &d.thread;
                    Value::Entity(x.map(EntityId::Item))
                }),
            _ => Value::Null,
        },
        set: None,
    },
    PropDef {
        name: "PostedName",
        kind: STR,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::bulletin::MessageData>())
                .map_or(Value::Null, |d| {
                    let x = &d.posted_name;
                    x.clone().map_or(Value::Null, Value::Str)
                }),
            _ => Value::Null,
        },
        set: None,
    },
];

static SERVER_ITEMS_TREASUREMAPCHEST: &[PropDef] = &[
    PropDef {
        name: "Temporary",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::last::chests::TmcData>())
                .map_or(Value::Null, |d| {
                    let x = &d.temporary;
                    Value::Bool(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.bool();
            crate::last::chests::set_temporary(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "Owner",
        kind: Kind::Plain(PropType::EntityOf("Mobile")),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::last::chests::TmcData>())
                .map_or(Value::Null, |d| {
                    let x = &d.owner;
                    Value::Entity(x.map(EntityId::Mobile))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.mobile();
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::last::chests::TmcData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.owner == n {
                return Ok(());
            }
            d.owner = n;
            Ok(())
        }),
    },
    PropDef {
        name: "Level",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::last::chests::TmcData>())
                .map_or(Value::Null, |d| {
                    let x = &d.level;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::last::chests::TmcData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.level == n {
                return Ok(());
            }
            d.level = n;
            Ok(())
        }),
    },
    PropDef {
        name: "Lifted",
        kind: Kind::Text,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::last::chests::TmcData>())
                .map_or(Value::Null, |d| {
                    let x = &d.lifted;
                    if x.is_some() {
                        Value::Text("System.Collections.Generic.HashSet`1[Server.Item]".to_string())
                    } else {
                        Value::Null
                    }
                }),
            _ => Value::Null,
        },
        set: None,
    },
];

static SERVER_ITEMS_BASEBEVERAGE: &[PropDef] = &[
    PropDef {
        name: "Poison",
        kind: Kind::Poison,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::food::beverage::BevData>())
                .map_or(Value::Null, |d| {
                    let x = &d.poison;
                    Value::Poison(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match v {
                Value::Poison(p) => p,
                _ => None,
            };
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::food::beverage::BevData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.poison == n {
                return Ok(());
            }
            d.poison = n;
            Ok(())
        }),
    },
    PropDef {
        name: "Poisoner",
        kind: Kind::Plain(PropType::EntityOf("Mobile")),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::food::beverage::BevData>())
                .map_or(Value::Null, |d| {
                    let x = &d.poisoner;
                    Value::Entity(x.map(EntityId::Mobile))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.mobile();
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::food::beverage::BevData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.poisoner == n {
                return Ok(());
            }
            d.poisoner = n;
            Ok(())
        }),
    },
    PropDef {
        name: "Content",
        kind: Kind::Plain(PropType::Enum(E_BEVERAGETYPE)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::food::beverage::BevData>())
                .map_or(Value::Null, |d| {
                    let x = &d.content;
                    Value::Enum(*x as i64)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = crate::food::beverage::BeverageType::from_i32(v.i64() as i32);
            crate::food::beverage::set_content(w, i, n);
            Ok(())
        }),
    },
];

static SERVER_ITEMS_ENDLESSDECANTER: &[PropDef] = &[
    PropDef {
        name: "Linked",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::food::beverage::BevData>())
                .map_or(Value::Null, |d| {
                    let x = &d.linked;
                    Value::Bool(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.bool();
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::food::beverage::BevData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.linked == n {
                return Ok(());
            }
            d.linked = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "LinkLocation",
        kind: Kind::Plain(PropType::IPoint3D),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::food::beverage::BevData>())
                .map_or(Value::Null, |d| {
                    let x = &d.link_location;
                    Value::Point3D(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.point();
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::food::beverage::BevData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.link_location == n {
                return Ok(());
            }
            d.link_location = n;
            Ok(())
        }),
    },
    PropDef {
        name: "LinkMap",
        kind: Kind::Map,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::food::beverage::BevData>())
                .map_or(Value::Null, |d| {
                    let x = &d.link_map;
                    Value::Map(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.map();
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::food::beverage::BevData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.link_map == n {
                return Ok(());
            }
            d.link_map = n;
            Ok(())
        }),
    },
];

static SERVER_ITEMS_BASEBOARD: &[PropDef] = &[PropDef {
    name: "Level",
    kind: Kind::Plain(PropType::Enum(E_SECURELEVEL)),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| o.ext.get::<crate::games::BoardData>())
            .map_or(Value::Null, |d| {
                let x = &d.level;
                Value::Enum(i64::from(*x))
            }),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.i64() as i32;
        let Some(d) = w
            .item_mut(i)
            .and_then(|o| o.ext.get_mut::<crate::games::BoardData>())
        else {
            return Err(NOT_HERE.to_string());
        };
        if d.level == n {
            return Ok(());
        }
        d.level = n;
        Ok(())
    }),
}];

static SERVER_ITEMS_GUILDSTONE: &[PropDef] = &[
    PropDef {
        name: "GuildName",
        kind: STR,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::guilds::StoneData>())
                .map_or(Value::Null, |d| {
                    let x = &d.guild_name;
                    x.clone().map_or(Value::Null, Value::Str)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.string();
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::guilds::StoneData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.guild_name == n {
                return Ok(());
            }
            d.guild_name = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "GuildAbbrev",
        kind: STR,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::guilds::StoneData>())
                .map_or(Value::Null, |d| {
                    let x = &d.guild_abbrev;
                    x.clone().map_or(Value::Null, Value::Str)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.string();
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::guilds::StoneData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.guild_abbrev == n {
                return Ok(());
            }
            d.guild_abbrev = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
];

static SERVER_ITEMS_GUILDSTONEDEED: &[PropDef] = &[
    PropDef {
        name: "GuildName",
        kind: STR,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::guilds::StoneData>())
                .map_or(Value::Null, |d| {
                    let x = &d.guild_name;
                    x.clone().map_or(Value::Null, Value::Str)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.string();
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::guilds::StoneData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.guild_name == n {
                return Ok(());
            }
            d.guild_name = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "GuildAbbrev",
        kind: STR,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::guilds::StoneData>())
                .map_or(Value::Null, |d| {
                    let x = &d.guild_abbrev;
                    x.clone().map_or(Value::Null, Value::Str)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.string();
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::guilds::StoneData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.guild_abbrev == n {
                return Ok(());
            }
            d.guild_abbrev = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
];

static SERVER_ITEMS_BASELIGHT: &[PropDef] = &[
    PropDef {
        name: "BurntOut",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::lights::base::LightData>())
                .map_or(Value::Null, |d| {
                    let x = &d.burnt_out;
                    Value::Bool(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.bool();
            crate::lights::base::set_burnt_out(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "Protected",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::lights::base::LightData>())
                .map_or(Value::Null, |d| {
                    let x = &d.protected;
                    Value::Bool(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.bool();
            crate::lights::base::set_protected(w, i, n);
            Ok(())
        }),
    },
];

static SERVER_ITEMS_MAPITEM: &[PropDef] = &[
    PropDef {
        name: "Bounds",
        kind: Kind::Rect2D,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::maps::MapItemData>())
                .map_or(Value::Null, |d| {
                    let x = &d.bounds;
                    Value::Rect(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match v {
                Value::Rect(r) => r,
                _ => Rectangle2D::default(),
            };
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::maps::MapItemData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.bounds == n {
                return Ok(());
            }
            d.bounds = n;
            Ok(())
        }),
    },
    PropDef {
        name: "Width",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::maps::MapItemData>())
                .map_or(Value::Null, |d| {
                    let x = &d.width;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::maps::MapItemData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.width == n {
                return Ok(());
            }
            d.width = n;
            Ok(())
        }),
    },
    PropDef {
        name: "Height",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::maps::MapItemData>())
                .map_or(Value::Null, |d| {
                    let x = &d.height;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::maps::MapItemData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.height == n {
                return Ok(());
            }
            d.height = n;
            Ok(())
        }),
    },
    PropDef {
        name: "Protected",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::maps::MapItemData>())
                .map_or(Value::Null, |d| {
                    let x = &d.protected;
                    Value::Bool(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.bool();
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::maps::MapItemData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.protected == n {
                return Ok(());
            }
            d.protected = n;
            Ok(())
        }),
    },
    PropDef {
        name: "Pins",
        kind: Kind::Text,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::maps::MapItemData>())
                .map_or(Value::Null, |d| {
                    let x = &d.pins;
                    Value::Text("System.Collections.Generic.List`1[Server.Point2D]".to_string())
                }),
            _ => Value::Null,
        },
        set: None,
    },
    PropDef {
        name: "Facet",
        kind: Kind::Map,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::maps::MapItemData>())
                .map_or(Value::Null, |d| {
                    let x = &d.facet;
                    Value::Map(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.map();
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::maps::MapItemData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.facet == n {
                return Ok(());
            }
            d.facet = n;
            Ok(())
        }),
    },
];

static SERVER_ITEMS_MORPHITEM: &[PropDef] = &[
    PropDef {
        name: "InactiveItemId",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::misc::basic::MorphData>())
                .map_or(Value::Null, |d| {
                    let x = &d.inactive_item_id;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::misc::basic::MorphData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.inactive_item_id == n {
                return Ok(());
            }
            d.inactive_item_id = n;
            Ok(())
        }),
    },
    PropDef {
        name: "ActiveItemId",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::misc::basic::MorphData>())
                .map_or(Value::Null, |d| {
                    let x = &d.active_item_id;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::misc::basic::MorphData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.active_item_id == n {
                return Ok(());
            }
            d.active_item_id = n;
            Ok(())
        }),
    },
];

static SERVER_ITEMS_LOCALIZEDSTATIC: &[PropDef] = &[PropDef {
    name: "Number",
    kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| o.ext.get::<crate::misc::basic::LocalizedStaticData>())
            .map_or(Value::Null, |d| {
                let x = &d.number;
                Value::Int(i64::from(*x))
            }),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.i64() as i32;
        let Some(d) = w
            .item_mut(i)
            .and_then(|o| o.ext.get_mut::<crate::misc::basic::LocalizedStaticData>())
        else {
            return Err(NOT_HERE.to_string());
        };
        if d.number == n {
            return Ok(());
        }
        d.number = n;
        muo_server::item::ops::invalidate_properties(w, i);
        Ok(())
    }),
}];

static SERVER_ITEMS_MESSAGEINABOTTLE: &[PropDef] = &[PropDef {
    name: "Level",
    kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| o.ext.get::<crate::skill_items::fishing::SosData>())
            .map_or(Value::Null, |d| {
                let x = &d.level;
                Value::Int(i64::from(*x))
            }),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.i64() as i32;
        crate::skill_items::fishing::set_level(w, i, n);
        Ok(())
    }),
}];

static SERVER_ITEMS_SOS: &[PropDef] = &[
    PropDef {
        name: "TargetMap",
        kind: Kind::Map,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::skill_items::fishing::SosData>())
                .map_or(Value::Null, |d| {
                    let x = &d.target_map;
                    Value::Map(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.map();
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::skill_items::fishing::SosData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.target_map == n {
                return Ok(());
            }
            d.target_map = n;
            Ok(())
        }),
    },
    PropDef {
        name: "TargetLocation",
        kind: Kind::Plain(PropType::IPoint3D),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::skill_items::fishing::SosData>())
                .map_or(Value::Null, |d| {
                    let x = &d.target_location;
                    Value::Point3D(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.point();
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::skill_items::fishing::SosData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.target_location == n {
                return Ok(());
            }
            d.target_location = n;
            Ok(())
        }),
    },
    PropDef {
        name: "MessageIndex",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::skill_items::fishing::SosData>())
                .map_or(Value::Null, |d| {
                    let x = &d.message_index;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::skill_items::fishing::SosData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.message_index == n {
                return Ok(());
            }
            d.message_index = n;
            Ok(())
        }),
    },
    PropDef {
        name: "Level",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::skill_items::fishing::SosData>())
                .map_or(Value::Null, |d| {
                    let x = &d.level;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            crate::skill_items::fishing::set_level(w, i, n);
            Ok(())
        }),
    },
];

static SERVER_ITEMS_RUNEBOOK: &[PropDef] = &[
    PropDef {
        name: "Quality",
        kind: Kind::Plain(PropType::Enum(E_BOOKQUALITY)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::skill_items::runebook::BookData>())
                .map_or(Value::Null, |d| {
                    let x = &d.quality;
                    Value::Enum(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::skill_items::runebook::BookData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.quality == n {
                return Ok(());
            }
            d.quality = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "Crafter",
        kind: STR,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::skill_items::runebook::BookData>())
                .map_or(Value::Null, |d| {
                    let x = &d.crafter;
                    x.clone().map_or(Value::Null, Value::Str)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.string();
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::skill_items::runebook::BookData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.crafter == n {
                return Ok(());
            }
            d.crafter = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "Level",
        kind: Kind::Plain(PropType::Enum(E_SECURELEVEL)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::skill_items::runebook::BookData>())
                .map_or(Value::Null, |d| {
                    let x = &d.level;
                    Value::Enum(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::skill_items::runebook::BookData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.level == n {
                return Ok(());
            }
            d.level = n;
            Ok(())
        }),
    },
    PropDef {
        name: "Description",
        kind: STR,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::skill_items::runebook::BookData>())
                .map_or(Value::Null, |d| {
                    let x = &d.description;
                    x.clone().map_or(Value::Null, Value::Str)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.string();
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::skill_items::runebook::BookData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.description == n {
                return Ok(());
            }
            d.description = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "CurCharges",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::skill_items::runebook::BookData>())
                .map_or(Value::Null, |d| {
                    let x = &d.cur_charges;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            crate::skill_items::runebook::set_cur_charges(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "MaxCharges",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::skill_items::runebook::BookData>())
                .map_or(Value::Null, |d| {
                    let x = &d.max_charges;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            crate::skill_items::runebook::set_max_charges(w, i, n);
            Ok(())
        }),
    },
];

static SERVER_ITEMS_RECALLRUNE: &[PropDef] = &[
    PropDef {
        name: "Description",
        kind: STR,
        read: COUNSELOR,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::skill_items::runebook::RuneData>())
                .map_or(Value::Null, |d| {
                    let x = &d.description;
                    x.clone().map_or(Value::Null, Value::Str)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.string();
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::skill_items::runebook::RuneData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.description == n {
                return Ok(());
            }
            d.description = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "Target",
        kind: Kind::Plain(PropType::IPoint3D),
        read: COUNSELOR,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::skill_items::runebook::RuneData>())
                .map_or(Value::Null, |d| {
                    let x = &d.target;
                    Value::Point3D(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.point();
            crate::skill_items::runebook::set_target(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "Marked",
        kind: BOOL,
        read: COUNSELOR,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::skill_items::runebook::RuneData>())
                .map_or(Value::Null, |d| {
                    let x = &d.marked;
                    Value::Bool(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.bool();
            crate::skill_items::runebook::set_marked(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "TargetMap",
        kind: Kind::Map,
        read: COUNSELOR,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::skill_items::runebook::RuneData>())
                .map_or(Value::Null, |d| {
                    let x = &d.target_map;
                    Value::Map(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.map();
            crate::skill_items::runebook::set_target_map(w, i, n);
            Ok(())
        }),
    },
];

static SERVER_ITEMS_BASEINSTRUMENT: &[PropDef] = &[
    PropDef {
        name: "Crafter",
        kind: STR,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| {
                    o.ext
                        .get::<crate::skill_items::instruments::InstrumentData>()
                })
                .map_or(Value::Null, |d| {
                    let x = &d.crafter;
                    x.clone().map_or(Value::Null, Value::Str)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.string();
            let Some(d) = w.item_mut(i).and_then(|o| {
                o.ext
                    .get_mut::<crate::skill_items::instruments::InstrumentData>()
            }) else {
                return Err(NOT_HERE.to_string());
            };
            if d.crafter == n {
                return Ok(());
            }
            d.crafter = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "Slayer",
        kind: Kind::Plain(PropType::Enum(E_SLAYERNAME)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| {
                    o.ext
                        .get::<crate::skill_items::instruments::InstrumentData>()
                })
                .map_or(Value::Null, |d| {
                    let x = &d.slayer;
                    Value::Enum(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w.item_mut(i).and_then(|o| {
                o.ext
                    .get_mut::<crate::skill_items::instruments::InstrumentData>()
            }) else {
                return Err(NOT_HERE.to_string());
            };
            if d.slayer == n {
                return Ok(());
            }
            d.slayer = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "Slayer2",
        kind: Kind::Plain(PropType::Enum(E_SLAYERNAME)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| {
                    o.ext
                        .get::<crate::skill_items::instruments::InstrumentData>()
                })
                .map_or(Value::Null, |d| {
                    let x = &d.slayer2;
                    Value::Enum(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w.item_mut(i).and_then(|o| {
                o.ext
                    .get_mut::<crate::skill_items::instruments::InstrumentData>()
            }) else {
                return Err(NOT_HERE.to_string());
            };
            if d.slayer2 == n {
                return Ok(());
            }
            d.slayer2 = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "SuccessSound",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| {
                    o.ext
                        .get::<crate::skill_items::instruments::InstrumentData>()
                })
                .map_or(Value::Null, |d| {
                    let x = &d.success_sound;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w.item_mut(i).and_then(|o| {
                o.ext
                    .get_mut::<crate::skill_items::instruments::InstrumentData>()
            }) else {
                return Err(NOT_HERE.to_string());
            };
            if d.success_sound == n {
                return Ok(());
            }
            d.success_sound = n;
            Ok(())
        }),
    },
    PropDef {
        name: "FailureSound",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| {
                    o.ext
                        .get::<crate::skill_items::instruments::InstrumentData>()
                })
                .map_or(Value::Null, |d| {
                    let x = &d.failure_sound;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w.item_mut(i).and_then(|o| {
                o.ext
                    .get_mut::<crate::skill_items::instruments::InstrumentData>()
            }) else {
                return Err(NOT_HERE.to_string());
            };
            if d.failure_sound == n {
                return Ok(());
            }
            d.failure_sound = n;
            Ok(())
        }),
    },
];

static SERVER_ITEMS_DYETUB: &[PropDef] = &[PropDef {
    name: "Level",
    kind: Kind::Plain(PropType::Enum(E_SECURELEVEL)),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| o.ext.get::<crate::skill_items::dyes::TubData>())
            .map_or(Value::Null, |d| {
                let x = &d.level;
                Value::Enum(i64::from(*x))
            }),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.i64() as i32;
        let Some(d) = w
            .item_mut(i)
            .and_then(|o| o.ext.get_mut::<crate::skill_items::dyes::TubData>())
        else {
            return Err(NOT_HERE.to_string());
        };
        if d.level == n {
            return Ok(());
        }
        d.level = n;
        Ok(())
    }),
}];

static SERVER_ITEMS_SOULSTONE: &[PropDef] = &[
    PropDef {
        name: "LastUserName",
        kind: STR,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::special::soulstone::SoulData>())
                .map_or(Value::Null, |d| {
                    let x = &d.last_user_name;
                    x.clone().map_or(Value::Null, Value::Str)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.string();
            crate::special::soulstone::set_last_user_name(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "Level",
        kind: Kind::Plain(PropType::Enum(E_SECURELEVEL)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::special::soulstone::SoulData>())
                .map_or(Value::Null, |d| {
                    let x = &d.level;
                    Value::Enum(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::special::soulstone::SoulData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.level == n {
                return Ok(());
            }
            d.level = n;
            Ok(())
        }),
    },
    PropDef {
        name: "Account",
        kind: STR,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::special::soulstone::SoulData>())
                .map_or(Value::Null, |d| {
                    let x = &d.account;
                    x.clone().map_or(Value::Null, Value::Str)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.string();
            crate::special::soulstone::set_account(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "Skill",
        kind: Kind::Plain(PropType::Enum(E_SKILLNAME)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::special::soulstone::SoulData>())
                .map_or(Value::Null, |d| {
                    let x = &d.skill;
                    Value::Enum(*x as i64)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match muo_server::skills::SkillName::from_i32(v.i64() as i32) {
                Some(e) => e,
                None => return Err("That is not a member of the enum.".to_string()),
            };
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::special::soulstone::SoulData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.skill == n {
                return Ok(());
            }
            d.skill = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "SkillValue",
        kind: Kind::Plain(PropType::Double),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::special::soulstone::SoulData>())
                .map_or(Value::Null, |d| {
                    let x = &d.skill_value;
                    Value::Float(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.f64();
            crate::special::soulstone::set_skill_value(w, i, n);
            Ok(())
        }),
    },
];

static SERVER_ITEMS_HOUSERAFFLEDEED: &[PropDef] = &[
    PropDef {
        name: "Stone",
        kind: Kind::Plain(PropType::EntityOf("HouseRaffleStone")),
        read: GM,
        write: SEER,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::special::raffle::DeedData>())
                .map_or(Value::Null, |d| {
                    let x = &d.stone;
                    Value::Entity(x.map(EntityId::Item))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match v.entity() {
                Some(EntityId::Item(i)) => Some(i),
                _ => None,
            };
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::special::raffle::DeedData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.stone == n {
                return Ok(());
            }
            d.stone = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "PlotLocation",
        kind: Kind::Plain(PropType::IPoint3D),
        read: GM,
        write: SEER,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::special::raffle::DeedData>())
                .map_or(Value::Null, |d| {
                    let x = &d.plot_location;
                    Value::Point3D(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.point();
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::special::raffle::DeedData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.plot_location == n {
                return Ok(());
            }
            d.plot_location = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "PlotFacet",
        kind: Kind::Map,
        read: GM,
        write: SEER,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::special::raffle::DeedData>())
                .map_or(Value::Null, |d| {
                    let x = &d.plot_facet;
                    Value::Map(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.map();
            crate::special::raffle::set_plot_facet(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "AwardedTo",
        kind: Kind::Plain(PropType::EntityOf("Mobile")),
        read: GM,
        write: SEER,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::special::raffle::DeedData>())
                .map_or(Value::Null, |d| {
                    let x = &d.awarded_to;
                    Value::Entity(x.map(EntityId::Mobile))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.mobile();
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::special::raffle::DeedData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.awarded_to == n {
                return Ok(());
            }
            d.awarded_to = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
];

static SERVER_ITEMS_HOUSERAFFLESTONE: &[PropDef] = &[
    PropDef {
        name: "ExpireAction",
        kind: Kind::Plain(PropType::Enum(E_HOUSERAFFLEEXPIREACTION)),
        read: GM,
        write: SEER,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::special::raffle::StoneData>())
                .map_or(Value::Null, |d| {
                    let x = &d.expire_action;
                    Value::Enum(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::special::raffle::StoneData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.expire_action == n {
                return Ok(());
            }
            d.expire_action = n;
            Ok(())
        }),
    },
    PropDef {
        name: "Deed",
        kind: Kind::Plain(PropType::EntityOf("HouseRaffleDeed")),
        read: GM,
        write: SEER,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::special::raffle::StoneData>())
                .map_or(Value::Null, |d| {
                    let x = &d.deed;
                    Value::Entity(x.map(EntityId::Item))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match v.entity() {
                Some(EntityId::Item(i)) => Some(i),
                _ => None,
            };
            crate::special::raffle::set_deed(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "Winner",
        kind: Kind::Plain(PropType::EntityOf("Mobile")),
        read: GM,
        write: SEER,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::special::raffle::StoneData>())
                .map_or(Value::Null, |d| {
                    let x = &d.winner;
                    Value::Entity(x.map(EntityId::Mobile))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.mobile();
            crate::special::raffle::set_winner(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "Started",
        kind: Kind::DateTime,
        read: GM,
        write: SEER,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::special::raffle::StoneData>())
                .map_or(Value::Null, |d| {
                    let x = &d.started;
                    Value::DateTime(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match v {
                Value::DateTime(t) => t,
                _ => DateTime::MIN,
            };
            crate::special::raffle::set_started(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "Duration",
        kind: Kind::TimeSpan,
        read: GM,
        write: SEER,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::special::raffle::StoneData>())
                .map_or(Value::Null, |d| {
                    let x = &d.duration;
                    Value::TimeSpan(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match v {
                Value::TimeSpan(t) => t,
                _ => TimeSpan::ZERO,
            };
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::special::raffle::StoneData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.duration == n {
                return Ok(());
            }
            d.duration = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "PlotBounds",
        kind: Kind::Rect2D,
        read: GM,
        write: SEER,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::special::raffle::StoneData>())
                .map_or(Value::Null, |d| {
                    let x = &d.plot_bounds;
                    Value::Rect(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match v {
                Value::Rect(r) => r,
                _ => Rectangle2D::default(),
            };
            crate::special::raffle::set_plot_bounds(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "PlotFacet",
        kind: Kind::Map,
        read: GM,
        write: SEER,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::special::raffle::StoneData>())
                .map_or(Value::Null, |d| {
                    let x = &d.plot_facet;
                    Value::Map(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.map();
            crate::special::raffle::set_plot_facet(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "TicketPrice",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: SEER,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::special::raffle::StoneData>())
                .map_or(Value::Null, |d| {
                    let x = &d.ticket_price;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            crate::special::raffle::set_ticket_price(w, i, n);
            Ok(())
        }),
    },
];

static SERVER_ITEMS_MININGCART: &[PropDef] = &[PropDef {
    name: "CartType",
    kind: Kind::Plain(PropType::Enum(E_MININGCARTTYPE)),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| o.ext.get::<crate::special::vet3::CartData>())
            .map_or(Value::Null, |d| {
                let x = &d.cart_type;
                Value::Enum(i64::from(*x))
            }),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.i64() as i32;
        let Some(d) = w
            .item_mut(i)
            .and_then(|o| o.ext.get_mut::<crate::special::vet3::CartData>())
        else {
            return Err(NOT_HERE.to_string());
        };
        if d.cart_type == n {
            return Ok(());
        }
        d.cart_type = n;
        Ok(())
    }),
}];

static SERVER_ITEMS_MININGCARTDEED: &[PropDef] = &[
    PropDef {
        name: "Gems",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::special::vet3::CartData>())
                .map_or(Value::Null, |d| {
                    let x = &d.gems;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::special::vet3::CartData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.gems == n {
                return Ok(());
            }
            d.gems = n;
            Ok(())
        }),
    },
    PropDef {
        name: "Ore",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::special::vet3::CartData>())
                .map_or(Value::Null, |d| {
                    let x = &d.ore;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::special::vet3::CartData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.ore == n {
                return Ok(());
            }
            d.ore = n;
            Ok(())
        }),
    },
];

static SERVER_ITEMS_TREESTUMP: &[PropDef] = &[
    PropDef {
        name: "Logs",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::special::vet::StumpData>())
                .map_or(Value::Null, |d| {
                    let x = &d.logs;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::special::vet::StumpData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.logs == n {
                return Ok(());
            }
            d.logs = n;
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "LastChecked",
        kind: Kind::DateTime,
        read: GM,
        write: ADMIN,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::special::vet::StumpData>())
                .map_or(Value::Null, |d| {
                    let x = &d.last_checked;
                    Value::DateTime(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match v {
                Value::DateTime(t) => t,
                _ => DateTime::MIN,
            };
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::special::vet::StumpData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.last_checked == n {
                return Ok(());
            }
            d.last_checked = n;
            Ok(())
        }),
    },
];

static SERVER_ITEMS_TREESTUMPDEED: &[PropDef] = &[PropDef {
    name: "Logs",
    kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| o.ext.get::<crate::special::vet::DeedState>())
            .map_or(Value::Null, |d| {
                let x = &d.logs;
                Value::Int(i64::from(*x))
            }),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.i64() as i32;
        let Some(d) = w
            .item_mut(i)
            .and_then(|o| o.ext.get_mut::<crate::special::vet::DeedState>())
        else {
            return Err(NOT_HERE.to_string());
        };
        if d.logs == n {
            return Ok(());
        }
        d.logs = n;
        muo_server::item::ops::invalidate_properties(w, i);
        Ok(())
    }),
}];

static SERVER_ITEMS_MORPHITEM_36: &[PropDef] = &[
    PropDef {
        name: "OutsideRange",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::misc::basic::MorphData>())
                .map_or(Value::Null, |d| {
                    let x = &d.outside_range;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            if let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::misc::basic::MorphData>())
            {
                if d.outside_range != n {
                    d.outside_range = n.clamp(0, 18);
                }
            };
            Ok(())
        }),
    },
    PropDef {
        name: "InsideRange",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::misc::basic::MorphData>())
                .map_or(Value::Null, |d| {
                    let x = &d.inside_range;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            if let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::misc::basic::MorphData>())
            {
                if d.inside_range != n {
                    d.inside_range = n.clamp(0, 18);
                }
            };
            Ok(())
        }),
    },
    PropDef {
        name: "CurrentRange",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| {
                    o.ext.get::<crate::misc::basic::MorphData>().map(|d| {
                        Value::Int(i64::from(if o.item_id == d.inactive_item_id {
                            d.inside_range
                        } else {
                            d.outside_range
                        }))
                    })
                })
                .unwrap_or(Value::Null),
            _ => Value::Null,
        },
        set: None,
    },
];

static SERVER_ITEMS_FLOURMILLEASTADDON: &[PropDef] = &[
    PropDef {
        name: "HasFlour",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::addons::devices::MillData>())
                .map_or(Value::Null, |d| Value::Bool(d.cur_flour > 0)),
            _ => Value::Null,
        },
        set: None,
    },
    PropDef {
        name: "IsFull",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::addons::devices::MillData>())
                .map_or(Value::Null, |d| {
                    Value::Bool(d.cur_flour >= crate::addons::devices::MILL_MAX_FLOUR)
                }),
            _ => Value::Null,
        },
        set: None,
    },
    PropDef {
        name: "IsWorking",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::addons::devices::MillData>())
                .map_or(Value::Null, |d| {
                    let x = &d.working;
                    Value::Bool(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.bool();
            if let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::addons::devices::MillData>())
            {
                d.working = n;
            };
            Ok(())
        }),
    },
    PropDef {
        name: "MaxFlour",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::addons::devices::MillData>())
                .map_or(Value::Null, |d| {
                    Value::Int(crate::addons::devices::MILL_MAX_FLOUR as i64)
                }),
            _ => Value::Null,
        },
        set: None,
    },
    PropDef {
        name: "CurFlour",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::addons::devices::MillData>())
                .map_or(Value::Null, |d| {
                    let x = &d.cur_flour;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            crate::addons::devices::flour_mill_set_cur_flour(w, i, n);
            Ok(())
        }),
    },
];

static SERVER_ITEMS_SACKFLOUR: &[PropDef] = &[PropDef {
    name: "Quantity",
    kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| o.ext.get::<crate::food::cooking::FlourData>())
            .map_or(Value::Null, |d| {
                let x = &d.quantity;
                Value::Int(i64::from(*x))
            }),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.i64() as i32;
        crate::food::cooking::set_flour_quantity(w, i, n);
        Ok(())
    }),
}];

static SERVER_ITEMS_COOKABLEFOOD: &[PropDef] = &[PropDef {
    name: "CookingLevel",
    kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| o.ext.get::<crate::food::cooking::CookData>())
            .map_or(Value::Null, |d| {
                let x = &d.level;
                Value::Int(i64::from(*x))
            }),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.i64() as i32;
        if let Some(d) = w
            .item_mut(i)
            .and_then(|o| o.ext.get_mut::<crate::food::cooking::CookData>())
        {
            d.level = n;
        };
        Ok(())
    }),
}];

static SERVER_ITEMS_PICKPOCKETDIP: &[PropDef] = &[
    PropDef {
        name: "MinSkill",
        kind: Kind::Plain(PropType::Double),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::addons::devices::PracticeData>())
                .map_or(Value::Null, |d| {
                    let x = &d.min_skill;
                    Value::Float(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.f64();
            if let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::addons::devices::PracticeData>())
            {
                d.min_skill = n;
            };
            Ok(())
        }),
    },
    PropDef {
        name: "MaxSkill",
        kind: Kind::Plain(PropType::Double),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::addons::devices::PracticeData>())
                .map_or(Value::Null, |d| {
                    let x = &d.max_skill;
                    Value::Float(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.f64();
            if let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::addons::devices::PracticeData>())
            {
                d.max_skill = n;
            };
            Ok(())
        }),
    },
    PropDef {
        name: "Swinging",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::addons::devices::PracticeData>())
                .map_or(Value::Null, |d| Value::Bool(d.timer.is_some())),
            _ => Value::Null,
        },
        set: None,
    },
];

static SERVER_ITEMS_ARCHERYBUTTE: &[PropDef] = &[
    PropDef {
        name: "MinSkill",
        kind: Kind::Plain(PropType::Double),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::addons::devices::ButteData>())
                .map_or(Value::Null, |d| {
                    let x = &d.min_skill;
                    Value::Float(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.f64();
            if let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::addons::devices::ButteData>())
            {
                d.min_skill = n;
            };
            Ok(())
        }),
    },
    PropDef {
        name: "MaxSkill",
        kind: Kind::Plain(PropType::Double),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::addons::devices::ButteData>())
                .map_or(Value::Null, |d| {
                    let x = &d.max_skill;
                    Value::Float(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.f64();
            if let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::addons::devices::ButteData>())
            {
                d.max_skill = n;
            };
            Ok(())
        }),
    },
    PropDef {
        name: "Arrows",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::addons::devices::ButteData>())
                .map_or(Value::Null, |d| {
                    let x = &d.arrows;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            if let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::addons::devices::ButteData>())
            {
                d.arrows = n;
            };
            Ok(())
        }),
    },
    PropDef {
        name: "Bolts",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::addons::devices::ButteData>())
                .map_or(Value::Null, |d| {
                    let x = &d.bolts;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            if let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::addons::devices::ButteData>())
            {
                d.bolts = n;
            };
            Ok(())
        }),
    },
    PropDef {
        name: "LastUse",
        kind: Kind::DateTime,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::addons::devices::ButteData>())
                .map_or(Value::Null, |d| {
                    Value::DateTime(d.last_use.unwrap_or(DateTime::MIN))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match v {
                Value::DateTime(t) => t,
                _ => DateTime::MIN,
            };
            if let Some(d) = crate::addons::devices::butte_mut(w, i) {
                d.last_use = Some(n);
            };
            Ok(())
        }),
    },
    PropDef {
        name: "FacingEast",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => Value::Bool(w.item(i).is_some_and(|o| o.item_id == 0x100A)),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.bool();
            {
                let ty = w.item(i).map(|o| o.ty);
                if let Some(ty) = ty {
                    (ty.set_item_id)(w, i, if n { 0x100A } else { 0x100B });
                }
            };
            Ok(())
        }),
    },
];

static SERVER_ITEMS_SHTELECOMPONENT: &[PropDef] = &[
    PropDef {
        name: "TeleOffset",
        kind: Kind::Plain(PropType::IPoint3D),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::addons::hive::TeleData>())
                .map_or(Value::Null, |d| {
                    let x = &d.offset;
                    Value::Point3D(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.point();
            if let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::addons::hive::TeleData>())
            {
                d.offset = n;
            };
            Ok(())
        }),
    },
    PropDef {
        name: "Active",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::addons::hive::TeleData>())
                .map_or(Value::Null, |d| {
                    let x = &d.active;
                    Value::Bool(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.bool();
            crate::addons::hive::sh_tele_set_active(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "TeleDest",
        kind: Kind::Plain(PropType::EntityOf("SHTeleComponent")),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => Value::Entity(
                w.item(i)
                    .and_then(|o| o.ext.get::<crate::addons::hive::TeleData>())
                    .and_then(|d| d.dest)
                    .map(EntityId::Item),
            ),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match v.entity() {
                Some(EntityId::Item(t)) => Some(t),
                _ => None,
            };
            crate::addons::hive::sh_tele_set_dest(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "TelePoint",
        kind: Kind::Plain(PropType::IPoint3D),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => Value::Point3D(crate::addons::hive::sh_tele_point(w, i)),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.point();
            {
                let loc = w.item(i).map_or_else(Point3D::default, |o| o.location);
                if let Some(d) = w
                    .item_mut(i)
                    .and_then(|o| o.ext.get_mut::<crate::addons::hive::TeleData>())
                {
                    d.offset = Point3D::new(n.x - loc.x, n.y - loc.y, n.z - loc.z);
                }
            };
            Ok(())
        }),
    },
];

static SERVER_ITEMS_SHTELEPORTER: &[PropDef] = &[
    PropDef {
        name: "External",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::addons::hive::TeleporterData>())
                .map_or(Value::Null, |d| {
                    let x = &d.external;
                    Value::Bool(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.bool();
            if let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::addons::hive::TeleporterData>())
            {
                d.external = n;
            };
            Ok(())
        }),
    },
    PropDef {
        name: "UpTele",
        kind: Kind::Plain(PropType::EntityOf("SHTeleComponent")),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::addons::hive::TeleporterData>())
                .map_or(Value::Null, |d| {
                    let x = &d.up;
                    Value::Entity(x.map(EntityId::Item))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match v.entity() {
                Some(EntityId::Item(i)) => Some(i),
                _ => None,
            };
            if let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::addons::hive::TeleporterData>())
            {
                d.up = n;
            };
            Ok(())
        }),
    },
    PropDef {
        name: "RightTele",
        kind: Kind::Plain(PropType::EntityOf("SHTeleComponent")),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::addons::hive::TeleporterData>())
                .map_or(Value::Null, |d| {
                    let x = &d.right;
                    Value::Entity(x.map(EntityId::Item))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match v.entity() {
                Some(EntityId::Item(i)) => Some(i),
                _ => None,
            };
            if let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::addons::hive::TeleporterData>())
            {
                d.right = n;
            };
            Ok(())
        }),
    },
    PropDef {
        name: "DownTele",
        kind: Kind::Plain(PropType::EntityOf("SHTeleComponent")),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::addons::hive::TeleporterData>())
                .map_or(Value::Null, |d| {
                    let x = &d.down;
                    Value::Entity(x.map(EntityId::Item))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match v.entity() {
                Some(EntityId::Item(i)) => Some(i),
                _ => None,
            };
            if let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::addons::hive::TeleporterData>())
            {
                d.down = n;
            };
            Ok(())
        }),
    },
    PropDef {
        name: "LeftTele",
        kind: Kind::Plain(PropType::EntityOf("SHTeleComponent")),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::addons::hive::TeleporterData>())
                .map_or(Value::Null, |d| {
                    let x = &d.left;
                    Value::Entity(x.map(EntityId::Item))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match v.entity() {
                Some(EntityId::Item(i)) => Some(i),
                _ => None,
            };
            if let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::addons::hive::TeleporterData>())
            {
                d.left = n;
            };
            Ok(())
        }),
    },
];

static SERVER_ITEMS_BASEWINDCHIMES: &[PropDef] = &[PropDef {
    name: "TurnedOn",
    kind: BOOL,
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| o.ext.get::<crate::misc::gadgets::ChimesData>())
            .map_or(Value::Null, |d| {
                let x = &d.turned_on;
                Value::Bool(*x)
            }),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.bool();
        crate::misc::gadgets::chimes_set_on(w, i, n);
        Ok(())
    }),
}];

static SERVER_ITEMS_MOONGATE: &[PropDef] = &[
    PropDef {
        name: "Target",
        kind: Kind::Plain(PropType::IPoint3D),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::misc::moongates::MoongateData>())
                .map_or(Value::Null, |d| {
                    let x = &d.target;
                    Value::Point3D(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.point();
            if let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::misc::moongates::MoongateData>())
            {
                d.target = n;
            };
            Ok(())
        }),
    },
    PropDef {
        name: "TargetMap",
        kind: Kind::Map,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::misc::moongates::MoongateData>())
                .map_or(Value::Null, |d| {
                    let x = &d.target_map;
                    Value::Map(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.map();
            if let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::misc::moongates::MoongateData>())
            {
                d.target_map = n;
            };
            Ok(())
        }),
    },
    PropDef {
        name: "Dispellable",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::misc::moongates::MoongateData>())
                .map_or(Value::Null, |d| {
                    let x = &d.dispellable;
                    Value::Bool(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.bool();
            crate::misc::moongates::set_dispellable(w, i, n);
            Ok(())
        }),
    },
];

static SERVER_ITEMS_CONFIRMATIONMOONGATE: &[PropDef] = &[
    PropDef {
        name: "GumpWidth",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::misc::moongates::ConfirmationData>())
                .map_or(Value::Null, |d| {
                    let x = &d.gump_width;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            if let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::misc::moongates::ConfirmationData>())
            {
                d.gump_width = n;
            };
            Ok(())
        }),
    },
    PropDef {
        name: "GumpHeight",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::misc::moongates::ConfirmationData>())
                .map_or(Value::Null, |d| {
                    let x = &d.gump_height;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            if let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::misc::moongates::ConfirmationData>())
            {
                d.gump_height = n;
            };
            Ok(())
        }),
    },
    PropDef {
        name: "TitleColor",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::misc::moongates::ConfirmationData>())
                .map_or(Value::Null, |d| {
                    let x = &d.title_color;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            if let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::misc::moongates::ConfirmationData>())
            {
                d.title_color = n;
            };
            Ok(())
        }),
    },
    PropDef {
        name: "MessageColor",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::misc::moongates::ConfirmationData>())
                .map_or(Value::Null, |d| {
                    let x = &d.message_color;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            if let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::misc::moongates::ConfirmationData>())
            {
                d.message_color = n;
            };
            Ok(())
        }),
    },
    PropDef {
        name: "TitleNumber",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::misc::moongates::ConfirmationData>())
                .map_or(Value::Null, |d| {
                    let x = &d.title_number;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            if let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::misc::moongates::ConfirmationData>())
            {
                d.title_number = n;
            };
            Ok(())
        }),
    },
];

static SERVER_ITEMS_MOONSTONE: &[PropDef] = &[PropDef {
    name: "Type",
    kind: Kind::Plain(PropType::Enum(E_MOONSTONETYPE)),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => Value::Enum(
            w.item(i)
                .and_then(|o| o.ext.get::<i32>())
                .map_or(0, |t| i64::from(*t)),
        ),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.i64() as i32;
        {
            if let Some(t) = w.item_mut(i).and_then(|o| o.ext.get_mut::<i32>()) {
                *t = n;
            }
            muo_server::item::ops::invalidate_properties(w, i);
        };
        Ok(())
    }),
}];

static SERVER_ITEMS_POTIONKEG: &[PropDef] = &[
    PropDef {
        name: "Type",
        kind: Kind::Plain(PropType::Enum(E_POTIONEFFECT)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::skill_items::potion_keg::KegData>())
                .map_or(Value::Null, |d| Value::Enum(i64::from(d.effect))),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            if let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::skill_items::potion_keg::KegData>())
            {
                d.effect = n;
            }
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "Held",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::skill_items::potion_keg::KegData>())
                .map_or(Value::Null, |d| {
                    let x = &d.held;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            crate::skill_items::potion_keg::set_held(w, i, n);
            Ok(())
        }),
    },
];

static SERVER_ITEMS_MONSTERSTATUETTE: &[PropDef] = &[
    PropDef {
        name: "TurnedOn",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::special::statuette::StatuetteData>())
                .map_or(Value::Null, |d| Value::Bool(d.turned_on)),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.bool();
            crate::special::statuette::set_turned_on(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "IsRewardItem",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::special::statuette::StatuetteData>())
                .map_or(Value::Null, |d| {
                    let x = &d.is_reward_item;
                    Value::Bool(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.bool();
            crate::special::statuette::set_is_reward_item(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "Type",
        kind: Kind::Plain(PropType::Enum(E_MONSTERSTATUETTETYPE)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::special::statuette::StatuetteData>())
                .map_or(Value::Null, |d| Value::Enum(i64::from(d.kind))),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            crate::special::statuette::set_type(w, i, n);
            Ok(())
        }),
    },
];

static SERVER_ITEMS_SPECIALSCROLL: &[PropDef] = &[
    PropDef {
        name: "Skill",
        kind: Kind::Plain(PropType::Enum(E_SKILLNAME)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::special::scrolls::ScrollData>())
                .map_or(Value::Null, |d| Value::Enum(d.skill as i64)),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            {
                if let Some(s) = muo_server::skills::SkillName::from_i32(n) {
                    if let Some(d) = w
                        .item_mut(i)
                        .and_then(|o| o.ext.get_mut::<crate::special::scrolls::ScrollData>())
                    {
                        d.skill = s;
                    }
                }
                muo_server::item::ops::invalidate_properties(w, i);
            };
            Ok(())
        }),
    },
    PropDef {
        name: "Value",
        kind: Kind::Plain(PropType::Double),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::special::scrolls::ScrollData>())
                .map_or(Value::Null, |d| {
                    let x = &d.value;
                    Value::Float(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.f64();
            if let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::special::scrolls::ScrollData>())
            {
                d.value = n;
            }
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
];

static SERVER_ITEMS_HOLIDAYBELL: &[PropDef] = &[PropDef {
    name: "SoundID",
    kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| o.ext.get::<crate::special::holiday::BellData>())
            .map_or(Value::Null, |d| {
                let x = &d.sound_id;
                Value::Int(i64::from(*x))
            }),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.i64() as i32;
        if let Some(d) = w
            .item_mut(i)
            .and_then(|o| o.ext.get_mut::<crate::special::holiday::BellData>())
        {
            d.sound_id = n;
        }
        muo_server::item::ops::invalidate_properties(w, i);
        Ok(())
    }),
}];

static SERVER_ITEMS_SNOWGLOBEONE: &[PropDef] = &[PropDef {
    name: "Place",
    kind: Kind::Plain(PropType::Enum(E_SNOWGLOBETYPEONE)),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => Value::Enum(i64::from(crate::special::holiday::place(w, i))),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.i64() as i32;
        crate::special::holiday::set_place(w, i, n);
        Ok(())
    }),
}];

static SERVER_ITEMS_SNOWMAN: &[PropDef] = &[PropDef {
    name: "Title",
    kind: STR,
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => crate::special::holiday::title(w, i).map_or(Value::Null, Value::Str),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.string();
        crate::special::holiday::set_snowman_title(w, i, n);
        Ok(())
    }),
}];

static SERVER_ITEMS_DECEITBRAZIER: &[PropDef] = &[
    PropDef {
        name: "SpawnRange",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::misc::gadgets::BrazierData>())
                .map_or(Value::Null, |d| {
                    let x = &d.spawn_range;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            if let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::misc::gadgets::BrazierData>())
            {
                d.spawn_range = n;
            };
            Ok(())
        }),
    },
    PropDef {
        name: "NextSpawnDelay",
        kind: Kind::TimeSpan,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::misc::gadgets::BrazierData>())
                .map_or(Value::Null, |d| {
                    let x = &d.next_spawn_delay;
                    Value::TimeSpan(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match v {
                Value::TimeSpan(t) => t,
                _ => TimeSpan::ZERO,
            };
            if let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::misc::gadgets::BrazierData>())
            {
                d.next_spawn_delay = n;
            };
            Ok(())
        }),
    },
    PropDef {
        name: "NextSpawn",
        kind: Kind::DateTime,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::misc::gadgets::BrazierData>())
                .map_or(Value::Null, |d| {
                    let x = &d.next_spawn;
                    Value::DateTime(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match v {
                Value::DateTime(t) => t,
                _ => DateTime::MIN,
            };
            if let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::misc::gadgets::BrazierData>())
            {
                d.next_spawn = n;
            };
            Ok(())
        }),
    },
];

static SERVER_ITEMS_SERPENTPILLAR: &[PropDef] = &[
    PropDef {
        name: "Active",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::misc::gadgets::PillarData>())
                .map_or(Value::Null, |d| {
                    let x = &d.active;
                    Value::Bool(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.bool();
            if let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::misc::gadgets::PillarData>())
            {
                d.active = n;
            };
            Ok(())
        }),
    },
    PropDef {
        name: "Word",
        kind: STR,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::misc::gadgets::PillarData>())
                .map_or(Value::Null, |d| {
                    let x = &d.word;
                    x.clone().map_or(Value::Null, Value::Str)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.string();
            if let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::misc::gadgets::PillarData>())
            {
                d.word = n;
            };
            Ok(())
        }),
    },
    PropDef {
        name: "Destination",
        kind: Kind::Rect2D,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::misc::gadgets::PillarData>())
                .map_or(Value::Null, |d| {
                    let x = &d.destination;
                    Value::Rect(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match v {
                Value::Rect(r) => r,
                _ => Rectangle2D::default(),
            };
            if let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::misc::gadgets::PillarData>())
            {
                d.destination = n;
            };
            Ok(())
        }),
    },
];

static SERVER_ITEMS_WARNINGITEM: &[PropDef] = &[PropDef {
    name: "Range",
    kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| o.ext.get::<crate::misc::gadgets::WarningData>())
            .map_or(Value::Null, |d| {
                let x = &d.range;
                Value::Int(i64::from(*x))
            }),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.i64() as i32;
        if let Some(d) = w
            .item_mut(i)
            .and_then(|o| o.ext.get_mut::<crate::misc::gadgets::WarningData>())
        {
            if d.range != n {
                d.range = n.min(18);
            }
        };
        Ok(())
    }),
}];

static SERVER_ITEMS_BASEWAND: &[PropDef] = &[
    PropDef {
        name: "WandEffect",
        kind: Kind::Plain(PropType::Enum(E_WANDEFFECT)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => Value::Enum(
                muo_content::items::weapon::data(w, i).map_or(0, |d| i64::from(d.wand_effect)),
            ),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            {
                if let Some(d) = muo_content::items::weapon::data_mut(w, i) {
                    d.wand_effect = n;
                }
                muo_server::item::ops::invalidate_properties(w, i);
            };
            Ok(())
        }),
    },
    PropDef {
        name: "Charges",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => Value::Int(i64::from(crate::wands::charges(w, i))),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            crate::wands::set_charges(w, i, n);
            Ok(())
        }),
    },
];

static SERVER_ITEMS_FIREWORKSWAND: &[PropDef] = &[PropDef {
    name: "Charges",
    kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => Value::Int(i64::from(crate::weapons::special::charges(w, i))),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.i64() as i32;
        crate::weapons::special::set_charges(w, i, n);
        Ok(())
    }),
}];

static SERVER_ITEMS_MINIHOUSEADDON: &[PropDef] = &[PropDef {
    name: "Type",
    kind: Kind::Plain(PropType::Enum(E_MINIHOUSETYPE)),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => Value::Enum(i64::from(crate::special::vet2::mini_type(w, i))),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.i64() as i32;
        crate::special::vet2::set_mini_addon_type(w, i, n);
        Ok(())
    }),
}];

static SERVER_ITEMS_MINIHOUSEDEED: &[PropDef] = &[PropDef {
    name: "Type",
    kind: Kind::Plain(PropType::Enum(E_MINIHOUSETYPE)),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => Value::Enum(i64::from(crate::special::vet2::mini_type(w, i))),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.i64() as i32;
        {
            crate::special::vet2::set_mini_type_raw(w, i, n);
            muo_server::item::ops::invalidate_properties(w, i);
        };
        Ok(())
    }),
}];

static SERVER_ITEMS_TREASUREMAP_61: &[PropDef] = &[PropDef {
    name: "ChestLocation",
    kind: Kind::Plain(PropType::IPoint2D),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => {
            let p = crate::maps::treasure::treasure(w, i).chest_location;
            Value::Point3D(Point3D::new(p.x, p.y, 0))
        }
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.point();
        if let Some(d) = crate::maps::treasure::treasure_mut(w, i) {
            d.chest_location = Point2D::new(n.x, n.y);
        };
        Ok(())
    }),
}];

static SERVER_ITEMS_HOUSERAFFLESTONE_62: &[PropDef] = &[
    PropDef {
        name: "CurrentState",
        kind: Kind::Plain(PropType::Enum(E_HOUSERAFFLESTATE)),
        read: GM,
        write: SEER,
        get: |w, e| match e {
            EntityId::Item(i) => Value::Enum(
                crate::special::raffle::stone(w, i).map_or(0, |d| i64::from(d.current_state)),
            ),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            crate::special::raffle::set_state(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "IsExpired",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => Value::Bool(crate::special::raffle::is_expired(w, i)),
            _ => Value::Null,
        },
        set: None,
    },
];

static SERVER_ITEMS_HOUSERAFFLEDEED_63: &[PropDef] = &[PropDef {
    name: "IsExpired",
    kind: BOOL,
    read: GM,
    write: SEER,
    get: |w, e| match e {
        EntityId::Item(i) => Value::Bool(crate::special::raffle::deed_is_expired(w, i)),
        _ => Value::Null,
    },
    set: None,
}];

static SERVER_ITEMS_LOCALIZEDSIGN: &[PropDef] = &[PropDef {
    name: "Number",
    kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| o.ext.get::<crate::construction::walls::NumberData>())
            .map_or(Value::Null, |d| {
                let x = &d.number;
                Value::Int(i64::from(*x))
            }),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.i64() as i32;
        if let Some(d) = w
            .item_mut(i)
            .and_then(|o| o.ext.get_mut::<crate::construction::walls::NumberData>())
        {
            d.number = n;
        }
        muo_server::item::ops::invalidate_properties(w, i);
        Ok(())
    }),
}];

static SERVER_ITEMS_SUBTEXTSIGN: &[PropDef] = &[PropDef {
    name: "SubText",
    kind: STR,
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| o.ext.get::<crate::construction::walls::SubtextData>())
            .map_or(Value::Null, |d| {
                let x = &d.text;
                x.clone().map_or(Value::Null, Value::Str)
            }),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.string();
        if let Some(d) = w
            .item_mut(i)
            .and_then(|o| o.ext.get_mut::<crate::construction::walls::SubtextData>())
        {
            d.text = n;
        }
        muo_server::item::ops::invalidate_properties(w, i);
        Ok(())
    }),
}];

static SERVER_ITEMS_STRONGBOX: &[PropDef] = &[
    PropDef {
        name: "Owner",
        kind: Kind::Plain(PropType::EntityOf("Mobile")),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::containers::strong_box::BoxData>())
                .map_or(Value::Null, |d| {
                    let x = &d.owner;
                    Value::Entity(x.map(EntityId::Mobile))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.mobile();
            if let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::containers::strong_box::BoxData>())
            {
                d.owner = n;
            }
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "House",
        kind: Kind::Plain(PropType::EntityOf("BaseHouse")),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::containers::strong_box::BoxData>())
                .map_or(Value::Null, |d| {
                    let x = &d.house;
                    Value::Entity(x.map(EntityId::Item))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match v.entity() {
                Some(EntityId::Item(i)) => Some(i),
                _ => None,
            };
            if let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::containers::strong_box::BoxData>())
            {
                d.house = n;
            }
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
];

static SERVER_ITEMS_TREASUREMAPCHEST_67: &[PropDef] = &[
    PropDef {
        name: "ExpireTimer",
        kind: Kind::Text,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::last::chests::TmcData>())
                .filter(|d| d.expire != muo_server::timer::TimerHandle::default())
                .map_or(Value::Null, |_| {
                    Value::Text("DelayCallTimer[Server.Item.Delete]".to_string())
                }),
            _ => Value::Null,
        },
        set: None,
    },
    PropDef {
        name: "DeleteTime",
        kind: Kind::DateTime,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => {
                match w
                    .item(i)
                    .and_then(|o| o.ext.get::<crate::last::chests::TmcData>())
                    .map(|d| d.expire)
                {
                    Some(h) if h != muo_server::timer::TimerHandle::default() => {
                        Value::DateTime(h.next(w))
                    }
                    Some(_) => Value::DateTime(DateTime::MIN),
                    None => Value::Null,
                }
            }
            _ => Value::Null,
        },
        set: None,
    },
];

static SERVER_ITEMS_SOS_68: &[PropDef] = &[PropDef {
    name: "IsAncient",
    kind: BOOL,
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| o.ext.get::<crate::skill_items::fishing::SosData>())
            .map_or(Value::Null, |d| Value::Bool(d.level >= 4)),
        _ => Value::Null,
    },
    set: None,
}];

static SERVER_ITEMS_CAMPFIRE: &[PropDef] = &[PropDef {
    name: "Status",
    kind: Kind::Plain(PropType::Enum(E_CAMPFIRESTATUS)),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => Value::Enum(crate::skill_items::camping::status(w, i) as i64),
        _ => Value::Null,
    },
    set: None,
}];

static SERVER_ITEMS_BASEINSTRUMENT_70: &[PropDef] = &[
    PropDef {
        name: "ReplenishesCharges",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| {
                    o.ext
                        .get::<crate::skill_items::instruments::InstrumentData>()
                })
                .map_or(Value::Null, |d| {
                    let x = &d.replenishes_charges;
                    Value::Bool(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.bool();
            {
                let now = w.now;
                if let Some(d) = w.item_mut(i).and_then(|o| {
                    o.ext
                        .get_mut::<crate::skill_items::instruments::InstrumentData>()
                }) {
                    if n != d.replenishes_charges && n {
                        d.last_replenished = now;
                    }
                    d.replenishes_charges = n;
                }
            };
            Ok(())
        }),
    },
    PropDef {
        name: "LastReplenished",
        kind: Kind::DateTime,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| {
                    o.ext
                        .get::<crate::skill_items::instruments::InstrumentData>()
                })
                .map_or(Value::Null, |d| {
                    let x = &d.last_replenished;
                    Value::DateTime(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match v {
                Value::DateTime(t) => t,
                _ => DateTime::MIN,
            };
            {
                let changed = crate::skill_items::instruments::data(w, i)
                    .is_some_and(|d| d.last_replenished != n);
                if changed {
                    if let Some(d) = w.item_mut(i).and_then(|o| {
                        o.ext
                            .get_mut::<crate::skill_items::instruments::InstrumentData>()
                    }) {
                        d.last_replenished = n;
                    }
                    crate::skill_items::instruments::check_replenish_uses(w, i, true);
                }
            };
            Ok(())
        }),
    },
    PropDef {
        name: "Quality",
        kind: Kind::Plain(PropType::Enum(E_INSTRUMENTQUALITY)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => Value::Enum(
                crate::skill_items::instruments::data(w, i).map_or(0, |d| i64::from(d.quality)),
            ),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            crate::skill_items::instruments::set_quality(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "UsesRemaining",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => Value::Int(i64::from(
                crate::skill_items::instruments::uses_remaining(w, i),
            )),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            crate::skill_items::instruments::set_uses_remaining(w, i, n);
            Ok(())
        }),
    },
];

static SERVER_ITEMS_BASERUNICTOOL: &[PropDef] = &[PropDef {
    name: "Resource",
    kind: Kind::Plain(PropType::Enum(E_CRAFTRESOURCE)),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => Value::Enum(crate::skill_items::runic_tools::resource(w, i) as i64),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.i64() as i32;
        match muo_content::craft::CraftResource::from_i32(n) {
            Some(r) => crate::skill_items::runic_tools::set_resource(w, i, r),
            None => return Err("That is not a member of the enum.".to_string()),
        };
        Ok(())
    }),
}];

static SERVER_ITEMS_BASEBEVERAGE_72: &[PropDef] = &[
    PropDef {
        name: "IsEmpty",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => Value::Bool(crate::food::beverage::is_empty(w, i)),
            _ => Value::Null,
        },
        set: None,
    },
    PropDef {
        name: "ContainsAlcohol",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => Value::Bool(crate::food::beverage::contains_alcohol(w, i)),
            _ => Value::Null,
        },
        set: None,
    },
    PropDef {
        name: "IsFull",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => Value::Bool(crate::food::beverage::is_full(w, i)),
            _ => Value::Null,
        },
        set: None,
    },
    PropDef {
        name: "Quantity",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => Value::Int(i64::from(crate::food::beverage::quantity(w, i))),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            crate::food::beverage::set_quantity(w, i, n);
            Ok(())
        }),
    },
];

static SERVER_ITEMS_BASELIGHT_73: &[PropDef] = &[
    PropDef {
        name: "Burning",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => Value::Bool(crate::lights::base::burning(w, i)),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.bool();
            crate::lights::base::set_burning(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "Duration",
        kind: Kind::TimeSpan,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => Value::TimeSpan(crate::lights::base::duration(w, i)),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match v {
                Value::TimeSpan(t) => t,
                _ => TimeSpan::ZERO,
            };
            crate::lights::base::set_duration(w, i, n);
            Ok(())
        }),
    },
];

static SERVER_ITEMS_SOULSTONE_74: &[PropDef] = &[
    PropDef {
        name: "ActiveItemID",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::special::soulstone::SoulData>())
                .map_or(Value::Null, |d| {
                    let x = &d.active_item_id;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            crate::special::soulstone::set_active_item_id(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "InactiveItemID",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::special::soulstone::SoulData>())
                .map_or(Value::Null, |d| {
                    let x = &d.inactive_item_id;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            crate::special::soulstone::set_inactive_item_id(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "IsEmpty",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::special::soulstone::SoulData>())
                .map_or(Value::Null, |d| Value::Bool(d.skill_value <= 0.0)),
            _ => Value::Null,
        },
        set: None,
    },
];

static SERVER_ITEMS_SOULSTONEFRAGMENT: &[PropDef] = &[PropDef {
    name: "UsesRemaining",
    kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| o.ext.get::<crate::special::soulstone::FragmentData>())
            .map_or(Value::Null, |d| {
                let x = &d.uses_remaining;
                Value::Int(i64::from(*x))
            }),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.i64() as i32;
        if let Some(d) = w
            .item_mut(i)
            .and_then(|o| o.ext.get_mut::<crate::special::soulstone::FragmentData>())
        {
            d.uses_remaining = n;
        }
        muo_server::item::ops::invalidate_properties(w, i);
        Ok(())
    }),
}];

static SERVER_ITEMS_REDSOULSTONE: &[PropDef] = &[PropDef {
    name: "IsRewardItem",
    kind: BOOL,
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| o.ext.get::<crate::special::soulstone::RedData>())
            .map_or(Value::Null, |d| {
                let x = &d.is_reward_item;
                Value::Bool(*x)
            }),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.bool();
        if let Some(d) = w
            .item_mut(i)
            .and_then(|o| o.ext.get_mut::<crate::special::soulstone::RedData>())
        {
            d.is_reward_item = n;
        }
        muo_server::item::ops::invalidate_properties(w, i);
        Ok(())
    }),
}];

static SERVER_ITEMS_RUNEBOOK_77: &[PropDef] = &[PropDef {
    name: "NextUse",
    kind: Kind::DateTime,
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| o.ext.get::<crate::skill_items::runebook::BookData>())
            .map_or(Value::Null, |d| {
                let x = &d.next_use;
                Value::DateTime(*x)
            }),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = match v {
            Value::DateTime(t) => t,
            _ => DateTime::MIN,
        };
        if let Some(d) = w
            .item_mut(i)
            .and_then(|o| o.ext.get_mut::<crate::skill_items::runebook::BookData>())
        {
            d.next_use = n;
        };
        Ok(())
    }),
}];

static SERVER_ITEMS_RECALLRUNE_78: &[PropDef] = &[PropDef {
    name: "House",
    kind: Kind::Plain(PropType::EntityOf("BaseHouse")),
    read: COUNSELOR,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => {
            let h = crate::skill_items::runebook::rune(w, i).and_then(|d| d.house);
            if h.is_some_and(|h| w.item(h).is_none_or(|x| x.deleted())) {
                crate::skill_items::runebook::set_house(w, i, None);
            }
            Value::Entity(
                crate::skill_items::runebook::rune(w, i)
                    .and_then(|d| d.house)
                    .map(EntityId::Item),
            )
        }
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = match v.entity() {
            Some(EntityId::Item(h)) => Some(h),
            _ => None,
        };
        crate::skill_items::runebook::set_house(w, i, n);
        Ok(())
    }),
}];

static SERVER_ITEMS_CLOAK: &[PropDef] = &[
    PropDef {
        name: "CurArcaneCharges",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => muo_content::items::clothing::data(w, i)
                .map_or(Value::Null, |d| Value::Int(i64::from(d.arcane_charges.0))),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            crate::clothing::special::set_cur_arcane_charges(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "MaxArcaneCharges",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => muo_content::items::clothing::data(w, i)
                .map_or(Value::Null, |d| Value::Int(i64::from(d.arcane_charges.1))),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            crate::clothing::special::set_max_arcane_charges(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "IsArcane",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => Value::Bool(crate::clothing::special::is_arcane(w, i)),
            _ => Value::Null,
        },
        set: None,
    },
];

static SERVER_ITEMS_ROBE: &[PropDef] = &[
    PropDef {
        name: "CurArcaneCharges",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => muo_content::items::clothing::data(w, i)
                .map_or(Value::Null, |d| Value::Int(i64::from(d.arcane_charges.0))),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            crate::clothing::special::set_cur_arcane_charges(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "MaxArcaneCharges",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => muo_content::items::clothing::data(w, i)
                .map_or(Value::Null, |d| Value::Int(i64::from(d.arcane_charges.1))),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            crate::clothing::special::set_max_arcane_charges(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "IsArcane",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => Value::Bool(crate::clothing::special::is_arcane(w, i)),
            _ => Value::Null,
        },
        set: None,
    },
];

static SERVER_ITEMS_THIGHBOOTS: &[PropDef] = &[
    PropDef {
        name: "CurArcaneCharges",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => muo_content::items::clothing::data(w, i)
                .map_or(Value::Null, |d| Value::Int(i64::from(d.arcane_charges.0))),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            crate::clothing::special::set_cur_arcane_charges(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "MaxArcaneCharges",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => muo_content::items::clothing::data(w, i)
                .map_or(Value::Null, |d| Value::Int(i64::from(d.arcane_charges.1))),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            crate::clothing::special::set_max_arcane_charges(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "IsArcane",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => Value::Bool(crate::clothing::special::is_arcane(w, i)),
            _ => Value::Null,
        },
        set: None,
    },
];

static SERVER_ITEMS_LEAFGLOVES: &[PropDef] = &[
    PropDef {
        name: "CurArcaneCharges",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => muo_content::items::armor::data(w, i)
                .map_or(Value::Null, |d| Value::Int(i64::from(d.arcane_charges.0))),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            crate::armor::special::set_cur_arcane_charges(w, i, n, 0x2FC6);
            Ok(())
        }),
    },
    PropDef {
        name: "MaxArcaneCharges",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => muo_content::items::armor::data(w, i)
                .map_or(Value::Null, |d| Value::Int(i64::from(d.arcane_charges.1))),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            crate::armor::special::set_max_arcane_charges(w, i, n, 0x2FC6);
            Ok(())
        }),
    },
    PropDef {
        name: "IsArcane",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => Value::Bool(crate::armor::special::is_arcane(w, i)),
            _ => Value::Null,
        },
        set: None,
    },
];

static SERVER_ITEMS_LEATHERGLOVES: &[PropDef] = &[
    PropDef {
        name: "CurArcaneCharges",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => muo_content::items::armor::data(w, i)
                .map_or(Value::Null, |d| Value::Int(i64::from(d.arcane_charges.0))),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            crate::armor::special::set_cur_arcane_charges(w, i, n, 0x13C6);
            Ok(())
        }),
    },
    PropDef {
        name: "MaxArcaneCharges",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => muo_content::items::armor::data(w, i)
                .map_or(Value::Null, |d| Value::Int(i64::from(d.arcane_charges.1))),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            crate::armor::special::set_max_arcane_charges(w, i, n, 0x13C6);
            Ok(())
        }),
    },
    PropDef {
        name: "IsArcane",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => Value::Bool(crate::armor::special::is_arcane(w, i)),
            _ => Value::Null,
        },
        set: None,
    },
];

static SERVER_ITEMS_CANDELABRA: &[PropDef] = &[PropDef {
    name: "IsShipwreckedItem",
    kind: BOOL,
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => Value::Bool(
            w.item(i)
                .and_then(|o| o.ext.get::<crate::last::lights::Shipwrecked>())
                .is_some_and(|s| s.0),
        ),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.bool();
        crate::last::lights::set_shipwrecked(w, i, n);
        Ok(())
    }),
}];

static SERVER_ITEMS_BASEBOOK_85: &[PropDef] = &[PropDef {
    name: "PagesCount",
    kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| o.ext.get::<crate::books::BookData>())
            .map_or(Value::Null, |d| Value::Int(d.pages.len() as i64)),
        _ => Value::Null,
    },
    set: None,
}];

static SERVER_ITEMS_RECIPESCROLL: &[PropDef] = &[PropDef {
    name: "RecipeID",
    kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => Value::Int(i64::from(crate::skill_items::misc::recipe_id(w, i))),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.i64() as i32;
        {
            if let Some(o) = w.item_mut(i) {
                o.ext.insert(crate::skill_items::misc::RecipeData(n));
            }
            muo_server::item::ops::invalidate_properties(w, i);
        };
        Ok(())
    }),
}];

static SERVER_ITEMS_POWDEROFTEMPERAMENT: &[PropDef] = &[PropDef {
    name: "UsesRemaining",
    kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => Value::Int(i64::from(crate::special::rewards2::uses(w, i))),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.i64() as i32;
        crate::special::rewards2::set_powder_uses(w, i, n);
        Ok(())
    }),
}];

static SERVER_ITEMS_ANCIENTSMITHYHAMMER: &[PropDef] = &[PropDef {
    name: "Bonus",
    kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => {
            Value::Int(i64::from(crate::special::rewards2::bonus_data(w, i).bonus))
        }
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.i64() as i32;
        crate::special::rewards2::set_hammer_bonus(w, i, n);
        Ok(())
    }),
}];

static SERVER_ITEMS_BASEGLOVESOFMINING: &[PropDef] = &[PropDef {
    name: "Bonus",
    kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => {
            Value::Int(i64::from(crate::special::rewards2::bonus_data(w, i).bonus))
        }
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.i64() as i32;
        crate::special::rewards2::set_gloves_bonus(w, i, n);
        Ok(())
    }),
}];

static SERVER_ITEMS_CURTAINSCOMPONENT: &[PropDef] = &[PropDef {
    name: "ClosedId",
    kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => Value::Int(i64::from(crate::special::decor::closed_id(w, i))),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.i64() as i32;
        crate::special::decor::set_closed_id(w, i, n);
        Ok(())
    }),
}];

static SERVER_ITEMS_BASEFRUITTREEADDON: &[PropDef] = &[PropDef {
    name: "Fruits",
    kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => Value::Int(i64::from(crate::special::decor::fruits(w, i))),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.i64() as i32;
        crate::special::decor::set_fruits(w, i, n);
        Ok(())
    }),
}];

static SERVER_ITEMS_PLAGUEBEASTMUTATIONCORE: &[PropDef] = &[PropDef {
    name: "Cut",
    kind: BOOL,
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| o.ext.get::<crate::special::mutation::Pb>())
            .map_or(Value::Null, |d| {
                let x = &d.cut;
                Value::Bool(*x)
            }),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.bool();
        if let Some(d) = w
            .item_mut(i)
            .and_then(|o| o.ext.get_mut::<crate::special::mutation::Pb>())
        {
            d.cut = n;
        };
        Ok(())
    }),
}];

static SERVER_ITEMS_BALLOFSUMMONING: &[PropDef] = &[
    PropDef {
        name: "MaxCharges",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => Value::Int(20),
            _ => Value::Null,
        },
        set: None,
    },
    PropDef {
        name: "MaxRecharges",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => Value::Int(255),
            _ => Value::Null,
        },
        set: None,
    },
];

static SERVER_ITEMS_BAGOFSENDING: &[PropDef] = &[PropDef {
    name: "MaxRecharges",
    kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => Value::Int(255),
        _ => Value::Null,
    },
    set: None,
}];

static SERVER_ITEMS_VALENTINESCARD: &[PropDef] = &[
    PropDef {
        name: "From",
        kind: STR,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::special::boxes::CardData>())
                .map_or(Value::Null, |d| {
                    d.from.clone().map_or(Value::Null, Value::Str)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.string();
            if let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::special::boxes::CardData>())
            {
                d.from = n;
            }
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "To",
        kind: STR,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::special::boxes::CardData>())
                .map_or(Value::Null, |d| {
                    d.to.clone().map_or(Value::Null, Value::Str)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.string();
            if let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::special::boxes::CardData>())
            {
                d.to = n;
            }
            muo_server::item::ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
];

static SERVER_ITEMS_CANNONADDON: &[PropDef] = &[PropDef {
    name: "CannonDirection",
    kind: Kind::Plain(PropType::Enum(E_CANNONDIRECTION)),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| o.ext.get::<crate::special::vet5::CannonData>())
            .map_or(Value::Null, |d| Value::Enum(i64::from(d.direction))),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.i64() as i32;
        if let Some(d) = w
            .item_mut(i)
            .and_then(|o| o.ext.get_mut::<crate::special::vet5::CannonData>())
        {
            d.direction = n;
        };
        Ok(())
    }),
}];

static SERVER_ITEMS_CANNONDEED: &[PropDef] = &[PropDef {
    name: "Charges",
    kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => Value::Int(i64::from(crate::special::vet5::charges(w, i))),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.i64() as i32;
        crate::special::vet5::set_charges(w, i, n);
        Ok(())
    }),
}];

static SERVER_ITEMS_JUKABOW: &[PropDef] = &[PropDef {
    name: "IsModified",
    kind: BOOL,
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => Value::Bool(w.item(i).is_some_and(|o| o.hue == 0x453)),
        _ => Value::Null,
    },
    set: None,
}];

static SERVER_ITEMS_TOWERLANTERNARTIFACT: &[PropDef] = &[PropDef {
    name: "IsOn",
    kind: BOOL,
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => Value::Bool(w.item(i).is_some_and(|o| o.item_id == 0x24BF)),
        _ => Value::Null,
    },
    set: None,
}];

static SERVER_ITEMS_EARRINGSOFPROTECTION: &[PropDef] = &[PropDef {
    name: "Attribute",
    kind: Kind::Plain(PropType::Enum(E_AOSELEMENTATTRIBUTE)),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| o.ext.get::<crate::special::leaves::EarringData>())
            .map_or(Value::Null, |d| {
                Value::Enum(i64::from((d.attribute as u32).trailing_zeros()))
            }),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.i64() as u32;
        if n < 7 {
            if let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::special::leaves::EarringData>())
            {
                d.attribute = [
                    AosElementAttribute::Physical,
                    AosElementAttribute::Fire,
                    AosElementAttribute::Cold,
                    AosElementAttribute::Poison,
                    AosElementAttribute::Energy,
                    AosElementAttribute::Chaos,
                    AosElementAttribute::Direct,
                ][n as usize];
            }
        };
        Ok(())
    }),
}];

static SERVER_ITEMS_ROSEOFTRINSIC: &[PropDef] = &[
    PropDef {
        name: "Level",
        kind: Kind::Plain(PropType::Enum(E_SECURELEVEL)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::special::gifts::RoseData>())
                .map_or(Value::Null, |d| Value::Enum(i64::from(d.level))),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            if let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::special::gifts::RoseData>())
            {
                d.level = n;
            };
            Ok(())
        }),
    },
    PropDef {
        name: "Petals",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::special::gifts::RoseData>())
                .map_or(Value::Null, |d| {
                    let x = &d.petals;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            crate::special::gifts::set_petals(w, i, n);
            Ok(())
        }),
    },
];

static SERVER_ITEMS_TAPESTRYOFSOSARIA: &[PropDef] = &[PropDef {
    name: "Level",
    kind: Kind::Plain(PropType::Enum(E_SECURELEVEL)),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => Value::Enum(i64::from(crate::special::gifts::level(w, i))),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.i64() as i32;
        if let Some(o) = w.item_mut(i) {
            o.ext.insert(crate::special::gifts::Secure(n));
        };
        Ok(())
    }),
}];

static SERVER_ITEMS_MININGCART_103: &[PropDef] = &[
    PropDef {
        name: "Gems",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => {
                crate::special::vet3::give_resources(w, i);
                Value::Int(
                    w.item(i)
                        .and_then(|o| o.ext.get::<crate::special::vet3::CartData>())
                        .map_or(0, |d| i64::from(d.gems)),
                )
            }
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            if let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::special::vet3::CartData>())
            {
                d.gems = n;
            };
            Ok(())
        }),
    },
    PropDef {
        name: "Ore",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => {
                crate::special::vet3::give_resources(w, i);
                Value::Int(
                    w.item(i)
                        .and_then(|o| o.ext.get::<crate::special::vet3::CartData>())
                        .map_or(0, |d| i64::from(d.ore)),
                )
            }
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            if let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::special::vet3::CartData>())
            {
                d.ore = n;
            };
            Ok(())
        }),
    },
];

static SERVER_ITEMS_COMMODITYDEEDBOX: &[PropDef] = &[PropDef {
    name: "IsRewardItem",
    kind: BOOL,
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => Value::Bool(crate::special::boxes::reward_flag(w, i)),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.bool();
        crate::special::boxes::set_commodity_box_reward(w, i, n);
        Ok(())
    }),
}];

static SERVER_ITEMS_FILLABLECONTAINER: &[PropDef] = &[
    PropDef {
        name: "ContentType",
        kind: Kind::Plain(PropType::Enum(E_FILLABLECONTENTTYPE)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => {
                Value::Enum(i64::from(crate::containers::fillable::content_type(w, i)))
            }
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            crate::containers::fillable::set_content_type(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "NextRespawnTime",
        kind: Kind::DateTime,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => {
                match w
                    .item(i)
                    .and_then(|o| o.ext.get::<crate::containers::fillable::FillData>())
                    .and_then(|d| d.timer)
                {
                    Some(t) if t.running(w) => Value::DateTime(t.next(w)),
                    _ => Value::DateTime(DateTime::MIN),
                }
            }
            _ => Value::Null,
        },
        set: None,
    },
];

static SERVER_ITEMS_COMMODITYDEED: &[PropDef] = &[PropDef {
    name: "Commodity",
    kind: Kind::Plain(PropType::EntityOf("Item")),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => {
            Value::Entity(crate::deeds::deed_classes::commodity_of(w, i).map(EntityId::Item))
        }
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = match v.entity() {
            Some(EntityId::Item(c)) => Some(c),
            _ => None,
        };
        crate::deeds::deed_classes::set_commodity_field(w, i, n);
        Ok(())
    }),
}];

static SERVER_ITEMS_DRAGONBARDINGDEED: &[PropDef] = &[
    PropDef {
        name: "CraftedBy",
        kind: STR,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::last::deeds::BardingData>())
                .map_or(Value::Null, |d| {
                    let x = &d.crafted_by;
                    x.clone().map_or(Value::Null, Value::Str)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.string();
            {
                let mut d = crate::last::deeds::bd(w, i);
                d.crafted_by = n;
                crate::last::deeds::set_bd(w, i, d);
            };
            Ok(())
        }),
    },
    PropDef {
        name: "Exceptional",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::last::deeds::BardingData>())
                .map_or(Value::Null, |d| {
                    let x = &d.exceptional;
                    Value::Bool(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.bool();
            {
                let mut d = crate::last::deeds::bd(w, i);
                d.exceptional = n;
                crate::last::deeds::set_bd(w, i, d);
            };
            Ok(())
        }),
    },
    PropDef {
        name: "Resource",
        kind: Kind::Plain(PropType::Enum(E_CRAFTRESOURCE)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::last::deeds::BardingData>())
                .map_or(Value::Null, |d| Value::Enum(d.resource as i64)),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            match muo_content::craft::CraftResource::from_i32(n) {
                Some(r) => crate::last::deeds::set_resource(w, i, r),
                None => return Err("That is not a member of the enum.".to_string()),
            };
            Ok(())
        }),
    },
];

static SERVER_ITEMS_BASEPLAYERBB: &[PropDef] = &[
    PropDef {
        name: "Level",
        kind: Kind::Plain(PropType::Enum(E_SECURELEVEL)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::misc::player_bb::BbData>())
                .map_or(Value::Null, |d| Value::Enum(i64::from(d.level))),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            if let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::misc::player_bb::BbData>())
            {
                d.level = n;
            };
            Ok(())
        }),
    },
    PropDef {
        name: "Title",
        kind: STR,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::misc::player_bb::BbData>())
                .map_or(Value::Null, |d| {
                    let x = &d.title;
                    x.clone().map_or(Value::Null, Value::Str)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.string();
            if let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::misc::player_bb::BbData>())
            {
                d.title = n;
            };
            Ok(())
        }),
    },
];

static SERVER_ITEMS_BROADCASTCRYSTAL: &[PropDef] = &[PropDef {
    name: "Receivers",
    kind: Kind::Text,
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| o.ext.get::<crate::misc::machines::BroadcastData>())
            .map_or(Value::Null, |d| {
                Value::Text(
                    "System.Collections.Generic.List`1[Server.Items.ReceiverCrystal]".to_string(),
                )
            }),
        _ => Value::Null,
    },
    set: None,
}];

static SERVER_ITEMS_MARKCONTAINER: &[PropDef] = &[PropDef {
    name: "NextRelock",
    kind: Kind::DateTime,
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| o.ext.get::<crate::last::mark::MarkData>())
            .map_or(Value::Null, |d| {
                Value::DateTime(if d.relock == muo_server::timer::TimerHandle::default() {
                    DateTime::MIN
                } else {
                    d.relock.next(w)
                })
            }),
        _ => Value::Null,
    },
    set: None,
}];

static SERVER_SPELLS_SEVENTH_GATETRAVELMOONGATE: &[PropDef] = &[PropDef {
    name: "LinkedGate",
    kind: Kind::Plain(PropType::EntityOf("Moongate")),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => Value::Entity(
            w.item(i)
                .and_then(|o| o.ext.get::<crate::misc::moongates::LinkedGate>())
                .and_then(|l| l.0)
                .map(EntityId::Item),
        ),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = match v.entity() {
            Some(EntityId::Item(c)) => Some(c),
            _ => None,
        };
        if let Some(o) = w.item_mut(i) {
            o.ext.insert(crate::misc::moongates::LinkedGate(n));
        };
        Ok(())
    }),
}];

static SERVER_ITEMS_PROJECTEDITEM: &[PropDef] = &[PropDef {
    name: "MinimumVisible",
    kind: Kind::Plain(PropType::Enum(E_ACCESSLEVEL)),
    read: DEV,
    write: DEV,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| o.ext.get::<crate::misc::machines::ProjectedData>())
            .map_or(Value::Null, |d| Value::Enum(d.minimum_visible as i64)),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.i64() as i32;
        match muo_server::mobile::AccessLevel::from_i32(n) {
            Some(a) => {
                if let Some(d) = w
                    .item_mut(i)
                    .and_then(|o| o.ext.get_mut::<crate::misc::machines::ProjectedData>())
                {
                    d.minimum_visible = a;
                }
            }
            None => return Err("That is not a member of the enum.".to_string()),
        };
        Ok(())
    }),
}];

static SERVER_MOBILES_PLAYERMOBILE: &[PropDef] = &[PropDef {
    name: "ToothAche",
    kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Mobile(m) => Value::Int(i64::from(crate::special::holiday::get_tooth_ache(w, m))),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Mobile(m) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.i64() as i32;
        crate::special::holiday::set_tooth_ache(w, m, n);
        Ok(())
    }),
}];

static SERVER_ITEMS_WAYPOINT: &[PropDef] = &[PropDef {
    name: "NextPoint",
    kind: Kind::Plain(PropType::EntityOf("WayPoint")),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => {
            Value::Entity(crate::misc::basic::way_point_next(w, i).map(EntityId::Item))
        }
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = match v.entity() {
            Some(EntityId::Item(c))
                if w.item(c)
                    .is_some_and(|x| x.ty.full_name == "Server.Items.WayPoint") =>
            {
                Some(c)
            }
            Some(_) => return Err("not a WayPoint".to_string()),
            None => None,
        };
        crate::misc::basic::way_point_set_next(w, i, n);
        Ok(())
    }),
}];

static SERVER_ITEMS_BIGFISH: &[PropDef] = &[PropDef {
    name: "Fisher",
    kind: Kind::Plain(PropType::EntityOf("Mobile")),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => {
            Value::Entity(crate::resources::cloth::fisher(w, i).map(EntityId::Mobile))
        }
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.mobile();
        {
            if let Some(o) = w.item_mut(i) {
                o.ext
                    .insert(crate::resources::cloth::FishData { fisher: n });
            }
            muo_server::item::ops::invalidate_properties(w, i);
        };
        Ok(())
    }),
}];

static SERVER_ITEMS_SPECIALFISHINGNET: &[PropDef] = &[PropDef {
    name: "InUse",
    kind: BOOL,
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => Value::Bool(
            w.item(i)
                .and_then(|o| o.ext.get::<crate::skill_items::fishing::NetData>())
                .is_some_and(|d| d.in_use),
        ),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.bool();
        crate::skill_items::fishing::set_in_use_pub(w, i, n);
        Ok(())
    }),
}];

static SERVER_ITEMS_BUCKET: &[PropDef] = &[
    PropDef {
        name: "IsEmpty",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => Value::Bool(crate::special::water::is_empty(w, i)),
            _ => Value::Null,
        },
        set: None,
    },
    PropDef {
        name: "IsFull",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => Value::Bool(crate::special::water::is_full(w, i)),
            _ => Value::Null,
        },
        set: None,
    },
    PropDef {
        name: "Quantity",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => Value::Int(i64::from(crate::special::water::quantity(w, i))),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            crate::special::water::set_quantity(w, i, n);
            Ok(())
        }),
    },
];

pub fn configure(w: &mut World) {
    props::add(w, "Server.Items.Fukiya", SERVER_ITEMS_FUKIYA);
    props::add(w, "Server.Items.FukiyaDarts", SERVER_ITEMS_FUKIYA);
    props::add(w, "Server.Items.LeatherNinjaBelt", SERVER_ITEMS_FUKIYA);
    props::add(w, "Server.Items.Shuriken", SERVER_ITEMS_FUKIYA);
    props::add(w, "Server.Items.TrophyAddon", SERVER_ITEMS_TROPHYADDON);
    props::add(w, "Server.Items.TrophyDeed", SERVER_ITEMS_TROPHYDEED);
    props::add(
        w,
        "Server.Items.FurnitureDyeTub",
        SERVER_ITEMS_FURNITUREDYETUB,
    );
    props::add(
        w,
        "Server.Items.LeatherDyeTub",
        SERVER_ITEMS_FURNITUREDYETUB,
    );
    props::add(
        w,
        "Server.Items.RewardBlackDyeTub",
        SERVER_ITEMS_FURNITUREDYETUB,
    );
    props::add(
        w,
        "Server.Items.RunebookDyeTub",
        SERVER_ITEMS_FURNITUREDYETUB,
    );
    props::add(
        w,
        "Server.Items.SpecialDyeTub",
        SERVER_ITEMS_FURNITUREDYETUB,
    );
    props::add(
        w,
        "Server.Items.StatuetteDyeTub",
        SERVER_ITEMS_FURNITUREDYETUB,
    );
    props::add(w, "Server.Items.BaseQuiver", SERVER_ITEMS_BASEQUIVER);
    props::add(w, "Server.Items.BaseTalisman", SERVER_ITEMS_BASETALISMAN);
    props::add(w, "Server.Items.Spellbook", SERVER_ITEMS_SPELLBOOK);
    props::add(w, "Server.Items.TreasureMap", SERVER_ITEMS_TREASUREMAP);
    props::add(w, "Server.Items.Aquarium", SERVER_ITEMS_AQUARIUM);
    props::add(w, "Server.Items.Head", SERVER_ITEMS_HEAD);
    props::add(w, "Server.Items.BaseBook", SERVER_ITEMS_BASEBOOK);
    props::add(
        w,
        "Server.Items.BaseBulletinBoard",
        SERVER_ITEMS_BASEBULLETINBOARD,
    );
    props::add(
        w,
        "Server.Items.BulletinMessage",
        SERVER_ITEMS_BULLETINMESSAGE,
    );
    props::add(
        w,
        "Server.Items.TreasureMapChest",
        SERVER_ITEMS_TREASUREMAPCHEST,
    );
    props::add(w, "Server.Items.BaseBeverage", SERVER_ITEMS_BASEBEVERAGE);
    props::add(
        w,
        "Server.Items.EndlessDecanter",
        SERVER_ITEMS_ENDLESSDECANTER,
    );
    props::add(w, "Server.Items.BaseBoard", SERVER_ITEMS_BASEBOARD);
    props::add(w, "Server.Items.Guildstone", SERVER_ITEMS_GUILDSTONE);
    props::add(
        w,
        "Server.Items.GuildstoneDeed",
        SERVER_ITEMS_GUILDSTONEDEED,
    );
    props::add(w, "Server.Items.BaseLight", SERVER_ITEMS_BASELIGHT);
    props::add(w, "Server.Items.MapItem", SERVER_ITEMS_MAPITEM);
    props::add(w, "Server.Items.MorphItem", SERVER_ITEMS_MORPHITEM);
    props::add(
        w,
        "Server.Items.LocalizedStatic",
        SERVER_ITEMS_LOCALIZEDSTATIC,
    );
    props::add(
        w,
        "Server.Items.MessageInABottle",
        SERVER_ITEMS_MESSAGEINABOTTLE,
    );
    props::add(w, "Server.Items.SOS", SERVER_ITEMS_SOS);
    props::add(w, "Server.Items.Runebook", SERVER_ITEMS_RUNEBOOK);
    props::add(w, "Server.Items.RecallRune", SERVER_ITEMS_RECALLRUNE);
    props::add(
        w,
        "Server.Items.BaseInstrument",
        SERVER_ITEMS_BASEINSTRUMENT,
    );
    props::add(w, "Server.Items.DyeTub", SERVER_ITEMS_DYETUB);
    props::add(w, "Server.Items.SoulStone", SERVER_ITEMS_SOULSTONE);
    props::add(
        w,
        "Server.Items.HouseRaffleDeed",
        SERVER_ITEMS_HOUSERAFFLEDEED,
    );
    props::add(
        w,
        "Server.Items.HouseRaffleStone",
        SERVER_ITEMS_HOUSERAFFLESTONE,
    );
    props::add(w, "Server.Items.MiningCart", SERVER_ITEMS_MININGCART);
    props::add(
        w,
        "Server.Items.MiningCartDeed",
        SERVER_ITEMS_MININGCARTDEED,
    );
    props::add(w, "Server.Items.TreeStump", SERVER_ITEMS_TREESTUMP);
    props::add(w, "Server.Items.TreeStumpDeed", SERVER_ITEMS_TREESTUMPDEED);
    props::add(w, "Server.Items.MorphItem", SERVER_ITEMS_MORPHITEM_36);
    props::add(
        w,
        "Server.Items.FlourMillEastAddon",
        SERVER_ITEMS_FLOURMILLEASTADDON,
    );
    props::add(
        w,
        "Server.Items.FlourMillSouthAddon",
        SERVER_ITEMS_FLOURMILLEASTADDON,
    );
    props::add(w, "Server.Items.SackFlour", SERVER_ITEMS_SACKFLOUR);
    props::add(w, "Server.Items.CookableFood", SERVER_ITEMS_COOKABLEFOOD);
    props::add(w, "Server.Items.PickpocketDip", SERVER_ITEMS_PICKPOCKETDIP);
    props::add(w, "Server.Items.TrainingDummy", SERVER_ITEMS_PICKPOCKETDIP);
    props::add(w, "Server.Items.ArcheryButte", SERVER_ITEMS_ARCHERYBUTTE);
    props::add(
        w,
        "Server.Items.SHTeleComponent",
        SERVER_ITEMS_SHTELECOMPONENT,
    );
    props::add(w, "Server.Items.SHTeleporter", SERVER_ITEMS_SHTELEPORTER);
    props::add(
        w,
        "Server.Items.BaseWindChimes",
        SERVER_ITEMS_BASEWINDCHIMES,
    );
    props::add(w, "Server.Items.Moongate", SERVER_ITEMS_MOONGATE);
    props::add(
        w,
        "Server.Items.ConfirmationMoongate",
        SERVER_ITEMS_CONFIRMATIONMOONGATE,
    );
    props::add(w, "Server.Items.Moonstone", SERVER_ITEMS_MOONSTONE);
    props::add(w, "Server.Items.PotionKeg", SERVER_ITEMS_POTIONKEG);
    props::add(
        w,
        "Server.Items.MonsterStatuette",
        SERVER_ITEMS_MONSTERSTATUETTE,
    );
    props::add(w, "Server.Items.SpecialScroll", SERVER_ITEMS_SPECIALSCROLL);
    props::add(w, "Server.Items.HolidayBell", SERVER_ITEMS_HOLIDAYBELL);
    props::add(w, "Server.Items.SnowGlobeOne", SERVER_ITEMS_SNOWGLOBEONE);
    props::add(w, "Server.Items.SnowGlobeTwo", SERVER_ITEMS_SNOWGLOBEONE);
    props::add(w, "Server.Items.SnowGlobeThree", SERVER_ITEMS_SNOWGLOBEONE);
    props::add(w, "Server.Items.Snowman", SERVER_ITEMS_SNOWMAN);
    props::add(w, "Server.Items.DeceitBrazier", SERVER_ITEMS_DECEITBRAZIER);
    props::add(w, "Server.Items.SerpentPillar", SERVER_ITEMS_SERPENTPILLAR);
    props::add(w, "Server.Items.WarningItem", SERVER_ITEMS_WARNINGITEM);
    props::add(w, "Server.Items.BaseWand", SERVER_ITEMS_BASEWAND);
    props::add(w, "Server.Items.FireworksWand", SERVER_ITEMS_FIREWORKSWAND);
    props::add(
        w,
        "Server.Items.MiniHouseAddon",
        SERVER_ITEMS_MINIHOUSEADDON,
    );
    props::add(w, "Server.Items.MiniHouseDeed", SERVER_ITEMS_MINIHOUSEDEED);
    props::add(w, "Server.Items.TreasureMap", SERVER_ITEMS_TREASUREMAP_61);
    props::add(
        w,
        "Server.Items.HouseRaffleStone",
        SERVER_ITEMS_HOUSERAFFLESTONE_62,
    );
    props::add(
        w,
        "Server.Items.HouseRaffleDeed",
        SERVER_ITEMS_HOUSERAFFLEDEED_63,
    );
    props::add(w, "Server.Items.LocalizedSign", SERVER_ITEMS_LOCALIZEDSIGN);
    props::add(w, "Server.Items.SubtextSign", SERVER_ITEMS_SUBTEXTSIGN);
    props::add(w, "Server.Items.StrongBox", SERVER_ITEMS_STRONGBOX);
    props::add(
        w,
        "Server.Items.TreasureMapChest",
        SERVER_ITEMS_TREASUREMAPCHEST_67,
    );
    props::add(w, "Server.Items.SOS", SERVER_ITEMS_SOS_68);
    props::add(w, "Server.Items.Campfire", SERVER_ITEMS_CAMPFIRE);
    props::add(
        w,
        "Server.Items.BaseInstrument",
        SERVER_ITEMS_BASEINSTRUMENT_70,
    );
    props::add(w, "Server.Items.BaseRunicTool", SERVER_ITEMS_BASERUNICTOOL);
    props::add(w, "Server.Items.BaseBeverage", SERVER_ITEMS_BASEBEVERAGE_72);
    props::add(w, "Server.Items.BaseLight", SERVER_ITEMS_BASELIGHT_73);
    props::add(w, "Server.Items.SoulStone", SERVER_ITEMS_SOULSTONE_74);
    props::add(
        w,
        "Server.Items.SoulstoneFragment",
        SERVER_ITEMS_SOULSTONEFRAGMENT,
    );
    props::add(w, "Server.Items.RedSoulstone", SERVER_ITEMS_REDSOULSTONE);
    props::add(w, "Server.Items.Runebook", SERVER_ITEMS_RUNEBOOK_77);
    props::add(w, "Server.Items.RecallRune", SERVER_ITEMS_RECALLRUNE_78);
    props::add(w, "Server.Items.Cloak", SERVER_ITEMS_CLOAK);
    props::add(w, "Server.Items.Robe", SERVER_ITEMS_ROBE);
    props::add(w, "Server.Items.ThighBoots", SERVER_ITEMS_THIGHBOOTS);
    props::add(w, "Server.Items.LeafGloves", SERVER_ITEMS_LEAFGLOVES);
    props::add(w, "Server.Items.LeatherGloves", SERVER_ITEMS_LEATHERGLOVES);
    props::add(w, "Server.Items.Candelabra", SERVER_ITEMS_CANDELABRA);
    props::add(w, "Server.Items.BaseBook", SERVER_ITEMS_BASEBOOK_85);
    props::add(w, "Server.Items.RecipeScroll", SERVER_ITEMS_RECIPESCROLL);
    props::add(
        w,
        "Server.Items.PowderOfTemperament",
        SERVER_ITEMS_POWDEROFTEMPERAMENT,
    );
    props::add(
        w,
        "Server.Items.AncientSmithyHammer",
        SERVER_ITEMS_ANCIENTSMITHYHAMMER,
    );
    props::add(
        w,
        "Server.Items.BaseGlovesOfMining",
        SERVER_ITEMS_BASEGLOVESOFMINING,
    );
    props::add(
        w,
        "Server.Items.CurtainsComponent",
        SERVER_ITEMS_CURTAINSCOMPONENT,
    );
    props::add(
        w,
        "Server.Items.BaseFruitTreeAddon",
        SERVER_ITEMS_BASEFRUITTREEADDON,
    );
    props::add(
        w,
        "Server.Items.PlagueBeastMutationCore",
        SERVER_ITEMS_PLAGUEBEASTMUTATIONCORE,
    );
    props::add(
        w,
        "Server.Items.BallOfSummoning",
        SERVER_ITEMS_BALLOFSUMMONING,
    );
    props::add(
        w,
        "Server.Items.BraceletOfBinding",
        SERVER_ITEMS_BALLOFSUMMONING,
    );
    props::add(w, "Server.Items.BagOfSending", SERVER_ITEMS_BAGOFSENDING);
    props::add(
        w,
        "Server.Items.ValentinesCard",
        SERVER_ITEMS_VALENTINESCARD,
    );
    props::add(w, "Server.Items.CannonAddon", SERVER_ITEMS_CANNONADDON);
    props::add(w, "Server.Items.CannonDeed", SERVER_ITEMS_CANNONDEED);
    props::add(w, "Server.Items.JukaBow", SERVER_ITEMS_JUKABOW);
    props::add(
        w,
        "Server.Items.TowerLanternArtifact",
        SERVER_ITEMS_TOWERLANTERNARTIFACT,
    );
    props::add(
        w,
        "Server.Items.EarringsOfProtection",
        SERVER_ITEMS_EARRINGSOFPROTECTION,
    );
    props::add(w, "Server.Items.RoseOfTrinsic", SERVER_ITEMS_ROSEOFTRINSIC);
    props::add(
        w,
        "Server.Items.TapestryOfSosaria",
        SERVER_ITEMS_TAPESTRYOFSOSARIA,
    );
    props::add(w, "Server.Items.MiningCart", SERVER_ITEMS_MININGCART_103);
    props::add(
        w,
        "Server.Items.CommodityDeedBox",
        SERVER_ITEMS_COMMODITYDEEDBOX,
    );
    props::add(
        w,
        "Server.Items.FillableContainer",
        SERVER_ITEMS_FILLABLECONTAINER,
    );
    props::add(w, "Server.Items.CommodityDeed", SERVER_ITEMS_COMMODITYDEED);
    props::add(
        w,
        "Server.Items.DragonBardingDeed",
        SERVER_ITEMS_DRAGONBARDINGDEED,
    );
    props::add(w, "Server.Items.BasePlayerBB", SERVER_ITEMS_BASEPLAYERBB);
    props::add(
        w,
        "Server.Items.BroadcastCrystal",
        SERVER_ITEMS_BROADCASTCRYSTAL,
    );
    props::add(w, "Server.Items.MarkContainer", SERVER_ITEMS_MARKCONTAINER);
    props::add(
        w,
        "Server.Spells.Seventh.GateTravelMoongate",
        SERVER_SPELLS_SEVENTH_GATETRAVELMOONGATE,
    );
    props::add(w, "Server.Items.ProjectedItem", SERVER_ITEMS_PROJECTEDITEM);
    props::add(
        w,
        "Server.Mobiles.PlayerMobile",
        SERVER_MOBILES_PLAYERMOBILE,
    );
    props::add(w, "Server.Items.WayPoint", SERVER_ITEMS_WAYPOINT);
    props::add(w, "Server.Items.BigFish", SERVER_ITEMS_BIGFISH);
    props::add(
        w,
        "Server.Items.SpecialFishingNet",
        SERVER_ITEMS_SPECIALFISHINGNET,
    );
    props::add(w, "Server.Items.Bucket", SERVER_ITEMS_BUCKET);
    props::add(w, "Server.Items.WaterBarrel", SERVER_ITEMS_BUCKET);
    props::add(w, "Server.Items.Tub", SERVER_ITEMS_BUCKET);
}
