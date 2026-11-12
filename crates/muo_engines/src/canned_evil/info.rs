//! `ChampionSpawnInfo` and `ChampionSkullType`.

use muo_server::World;

/// `ChampionSpawnType`.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
#[repr(i32)]
pub enum ChampionSpawnType {
    #[default]
    Abyss = 0,
    Arachnid = 1,
    ColdBlood = 2,
    ForestLord = 3,
    VerminHorde = 4,
    UnholyTerror = 5,
    SleepingDragon = 6,
    Glade = 7,
    Corrupt = 8,
}

impl ChampionSpawnType {
    /// `(ChampionSpawnType)value`; an undefined number is `None`.
    pub fn from_i32(v: i32) -> Option<ChampionSpawnType> {
        use ChampionSpawnType::*;
        Some(match v {
            0 => Abyss,
            1 => Arachnid,
            2 => ColdBlood,
            3 => ForestLord,
            4 => VerminHorde,
            5 => UnholyTerror,
            6 => SleepingDragon,
            7 => Glade,
            8 => Corrupt,
            _ => return None,
        })
    }

    /// `ToString()`: the member name.
    pub fn name(self) -> &'static str {
        use ChampionSpawnType::*;
        match self {
            Abyss => "Abyss",
            Arachnid => "Arachnid",
            ColdBlood => "ColdBlood",
            ForestLord => "ForestLord",
            VerminHorde => "VerminHorde",
            UnholyTerror => "UnholyTerror",
            SleepingDragon => "SleepingDragon",
            Glade => "Glade",
            Corrupt => "Corrupt",
        }
    }
}

/// `ChampionSkullType`.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
#[repr(i32)]
pub enum ChampionSkullType {
    #[default]
    Power = 0,
    Enlightenment = 1,
    Venom = 2,
    Pain = 3,
    Greed = 4,
    Death = 5,
    None = 6,
}

impl ChampionSkullType {
    pub fn from_i32(v: i32) -> ChampionSkullType {
        use ChampionSkullType::*;
        match v {
            0 => Power,
            1 => Enlightenment,
            2 => Venom,
            3 => Pain,
            4 => Greed,
            5 => Death,
            _ => None,
        }
    }
}

/// `ChampionSpawnInfo`.
pub struct ChampionSpawnInfo {
    pub ty: ChampionSpawnType,
    /// The champion's class.
    pub champion: &'static str,
    /// `LevelNames[i].Number`.
    pub level_names: [i32; 3],
    /// `SpawnTypes`: the creature classes of each of the four levels.
    pub spawn_types: [&'static [&'static str]; 4],
}

const UNHOLY_L1_AOS: &[&str] = &[
    "Server.Mobiles.Bogle",
    "Server.Mobiles.Ghoul",
    "Server.Mobiles.Shade",
    "Server.Mobiles.Spectre",
    "Server.Mobiles.Wraith",
];
const UNHOLY_L1: &[&str] = &[
    "Server.Mobiles.Ghoul",
    "Server.Mobiles.Shade",
    "Server.Mobiles.Spectre",
    "Server.Mobiles.Wraith",
];

macro_rules! info {
    ($ty:ident, $champ:literal, [$a:literal, $b:literal, $c:literal], [$($l1:literal),+], [$($l2:literal),+], [$($l3:literal),+], [$($l4:literal),+]) => {
        ChampionSpawnInfo {
            ty: ChampionSpawnType::$ty,
            champion: concat!("Server.Mobiles.", $champ),
            level_names: [$a, $b, $c],
            spawn_types: [
                &[$(concat!("Server.Mobiles.", $l1)),+],
                &[$(concat!("Server.Mobiles.", $l2)),+],
                &[$(concat!("Server.Mobiles.", $l3)),+],
                &[$(concat!("Server.Mobiles.", $l4)),+],
            ],
        }
    };
}

/// `ChampionSpawnInfo.Table`. Level 1 of `UnholyTerror` is the AOS list; [`spawn_types`] swaps it for the pre-AOS one.
pub static TABLE: [ChampionSpawnInfo; 9] = [
    info!(
        Abyss,
        "Semidar",
        [1113118, 1113107, 1113096],
        ["GreaterMongbat", "Imp"],
        ["Gargoyle", "Harpy"],
        ["FireGargoyle", "StoneGargoyle"],
        ["Daemon", "Succubus"]
    ),
    info!(
        Arachnid,
        "Mephitis",
        [1113117, 1113106, 1113095],
        ["Scorpion", "GiantSpider"],
        ["TerathanDrone", "TerathanWarrior"],
        ["DreadSpider", "TerathanMatriarch"],
        ["PoisonElemental", "TerathanAvenger"]
    ),
    info!(
        ColdBlood,
        "Rikktor",
        [1113115, 1113104, 1113093],
        ["Lizardman", "GiantSerpent"],
        ["LavaLizard", "OphidianWarrior"],
        ["Drake", "OphidianArchmage"],
        ["Dragon", "OphidianKnight"]
    ),
    info!(
        ForestLord,
        "LordOaks",
        [1113116, 1113105, 1113094],
        ["Pixie", "ShadowWisp"],
        ["Kirin", "Wisp"],
        ["Centaur", "Unicorn"],
        ["EtherealWarrior", "SerpentineDragon"]
    ),
    info!(
        VerminHorde,
        "Barracoon",
        [1113119, 1113108, 1113097],
        ["GiantRat", "Slime"],
        ["DireWolf", "Ratman"],
        ["HellHound", "RatmanMage"],
        ["RatmanArcher", "SilverSerpent"]
    ),
    ChampionSpawnInfo {
        ty: ChampionSpawnType::UnholyTerror,
        champion: "Server.Mobiles.Neira",
        level_names: [1113120, 1113109, 1113098],
        spawn_types: [
            UNHOLY_L1_AOS,
            &[
                "Server.Mobiles.BoneMagi",
                "Server.Mobiles.Mummy",
                "Server.Mobiles.SkeletalMage",
            ],
            &[
                "Server.Mobiles.BoneKnight",
                "Server.Mobiles.Lich",
                "Server.Mobiles.SkeletalKnight",
            ],
            &["Server.Mobiles.LichLord", "Server.Mobiles.RottingCorpse"],
        ],
    },
    info!(
        SleepingDragon,
        "Serado",
        [1113121, 1113110, 1113099],
        ["DeathwatchBeetleHatchling", "Lizardman"],
        ["DeathwatchBeetle", "Kappa"],
        ["LesserHiryu", "RevenantLion"],
        ["Hiryu", "Oni"]
    ),
    info!(
        Glade,
        "Twaulo",
        [1113123, 1113112, 1113101],
        ["Pixie", "ShadowWisp"],
        ["Centaur", "MLDryad"],
        ["Satyr", "CuSidhe"],
        ["FeralTreefellow", "RagingGrizzlyBear"]
    ),
    info!(
        Corrupt,
        "Ilhenir",
        [1113122, 1113111, 1113100],
        ["PlagueSpawn", "Bogling"],
        ["PlagueBeast", "BogThing"],
        ["PlagueBeastLord", "InterredGrizzle"],
        ["FetidEssence", "PestilentBandage"]
    ),
];

/// `GetInfo(type)`: an unknown value is row 0.
pub fn get_info(ty: i32) -> &'static ChampionSpawnInfo {
    let v = usize::try_from(ty)
        .ok()
        .filter(|v| *v < TABLE.len())
        .unwrap_or(0);
    &TABLE[v]
}

/// `GetInfo(type).SpawnTypes[level]` (the table is built at startup with `Core.AOS`).
pub fn spawn_types(w: &World, ty: i32, level: usize) -> &'static [&'static str] {
    let info = get_info(ty);
    if info.ty == ChampionSpawnType::UnholyTerror && level == 0 && !w.core_aos() {
        return UNHOLY_L1;
    }
    info.spawn_types[level]
}
