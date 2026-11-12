//! `Loot`: the item-class pools and the `Random*`/`Construct` pickers. A pool (`Type[]`) is a list of
//! item class full names; `Construct` makes one through the type registry (`Activator.CreateInstance`).

use muo_primitives::utility;
use muo_server::ids::ItemId;
use muo_server::world::World;

use crate::factory::make_item;
use crate::hooks::rnd;

/// `SpellbookType`, as far as `RandomScroll` looks at it.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SpellbookType {
    Invalid,
    Regular,
    Necromancer,
    Paladin,
    Arcanist,
    Mystic,
    Bushido,
    Ninjitsu,
    Spellweaving,
}

pub static PALADIN_SCROLL_TYPES: &[&str] = &[];

pub static SA_WEAPON_TYPES: &[&str] = &[
    "Server.Items.DiscMace",
    "Server.Items.GargishTalwar",
    "Server.Items.DualPointedSpear",
    "Server.Items.GlassStaff",
    "Server.Items.DualShortAxes",
    "Server.Items.GlassSword",
];
pub static SA_RANGED_WEAPON_TYPES: &[&str] = &[
    "Server.Items.Boomerang",
    "Server.Items.Cyclone",
    "Server.Items.SoulGlaive",
];
pub static ML_WEAPON_TYPES: &[&str] = &[
    "Server.Items.AssassinSpike",
    "Server.Items.DiamondMace",
    "Server.Items.ElvenMachete",
    "Server.Items.ElvenSpellblade",
    "Server.Items.Leafblade",
    "Server.Items.OrnateAxe",
    "Server.Items.RadiantScimitar",
    "Server.Items.RuneBlade",
    "Server.Items.WarCleaver",
    "Server.Items.WildStaff",
];
pub static ML_RANGED_WEAPON_TYPES: &[&str] = &[
    "Server.Items.ElvenCompositeLongbow",
    "Server.Items.MagicalShortbow",
];
pub static ML_ARMOR_TYPES: &[&str] = &[
    "Server.Items.Circlet",
    "Server.Items.GemmedCirclet",
    "Server.Items.LeafTonlet",
    "Server.Items.RavenHelm",
    "Server.Items.RoyalCirclet",
    "Server.Items.VultureHelm",
    "Server.Items.WingedHelm",
    "Server.Items.LeafArms",
    "Server.Items.LeafChest",
    "Server.Items.LeafGloves",
    "Server.Items.LeafGorget",
    "Server.Items.LeafLegs",
    "Server.Items.WoodlandArms",
    "Server.Items.WoodlandChest",
    "Server.Items.WoodlandGloves",
    "Server.Items.WoodlandGorget",
    "Server.Items.WoodlandLegs",
    "Server.Items.HideChest",
    "Server.Items.HideGloves",
    "Server.Items.HideGorget",
    "Server.Items.HidePants",
    "Server.Items.HidePauldrons",
];
pub static ML_CLOTHING_TYPES: &[&str] = &[
    "Server.Items.MaleElvenRobe",
    "Server.Items.FemaleElvenRobe",
    "Server.Items.ElvenPants",
    "Server.Items.ElvenShirt",
    "Server.Items.ElvenDarkShirt",
    "Server.Items.ElvenBoots",
    "Server.Items.VultureHelm",
    "Server.Items.WoodlandBelt",
];
pub static SE_WEAPON_TYPES: &[&str] = &[
    "Server.Items.Bokuto",
    "Server.Items.Daisho",
    "Server.Items.Kama",
    "Server.Items.Lajatang",
    "Server.Items.NoDachi",
    "Server.Items.Nunchaku",
    "Server.Items.Sai",
    "Server.Items.Tekagi",
    "Server.Items.Tessen",
    "Server.Items.Tetsubo",
    "Server.Items.Wakizashi",
];
pub static AOS_WEAPON_TYPES: &[&str] = &[
    "Server.Items.Scythe",
    "Server.Items.BoneHarvester",
    "Server.Items.Scepter",
    "Server.Items.BladedStaff",
    "Server.Items.Pike",
    "Server.Items.DoubleBladedStaff",
    "Server.Items.Lance",
    "Server.Items.CrescentBlade",
];
pub static WEAPON_TYPES: &[&str] = &[
    "Server.Items.Axe",
    "Server.Items.BattleAxe",
    "Server.Items.DoubleAxe",
    "Server.Items.ExecutionersAxe",
    "Server.Items.Hatchet",
    "Server.Items.LargeBattleAxe",
    "Server.Items.TwoHandedAxe",
    "Server.Items.WarAxe",
    "Server.Items.Club",
    "Server.Items.Mace",
    "Server.Items.Maul",
    "Server.Items.WarHammer",
    "Server.Items.WarMace",
    "Server.Items.Bardiche",
    "Server.Items.Halberd",
    "Server.Items.Spear",
    "Server.Items.ShortSpear",
    "Server.Items.Pitchfork",
    "Server.Items.WarFork",
    "Server.Items.BlackStaff",
    "Server.Items.GnarledStaff",
    "Server.Items.QuarterStaff",
    "Server.Items.Broadsword",
    "Server.Items.Cutlass",
    "Server.Items.Katana",
    "Server.Items.Kryss",
    "Server.Items.Longsword",
    "Server.Items.Scimitar",
    "Server.Items.VikingSword",
    "Server.Items.Pickaxe",
    "Server.Items.HammerPick",
    "Server.Items.ButcherKnife",
    "Server.Items.Cleaver",
    "Server.Items.Dagger",
    "Server.Items.SkinningKnife",
    "Server.Items.ShepherdsCrook",
];
pub static SE_RANGED_WEAPON_TYPES: &[&str] = &["Server.Items.Yumi"];
pub static AOS_RANGED_WEAPON_TYPES: &[&str] = &[
    "Server.Items.CompositeBow",
    "Server.Items.RepeatingCrossbow",
];
pub static RANGED_WEAPON_TYPES: &[&str] = &[
    "Server.Items.Bow",
    "Server.Items.Crossbow",
    "Server.Items.HeavyCrossbow",
];
pub static SE_ARMOR_TYPES: &[&str] = &[
    "Server.Items.ChainHatsuburi",
    "Server.Items.LeatherDo",
    "Server.Items.LeatherHaidate",
    "Server.Items.LeatherHiroSode",
    "Server.Items.LeatherJingasa",
    "Server.Items.LeatherMempo",
    "Server.Items.LeatherNinjaHood",
    "Server.Items.LeatherNinjaJacket",
    "Server.Items.LeatherNinjaMitts",
    "Server.Items.LeatherNinjaPants",
    "Server.Items.LeatherSuneate",
    "Server.Items.DecorativePlateKabuto",
    "Server.Items.HeavyPlateJingasa",
    "Server.Items.LightPlateJingasa",
    "Server.Items.PlateBattleKabuto",
    "Server.Items.PlateDo",
    "Server.Items.PlateHaidate",
    "Server.Items.PlateHatsuburi",
    "Server.Items.PlateHiroSode",
    "Server.Items.PlateMempo",
    "Server.Items.PlateSuneate",
    "Server.Items.SmallPlateJingasa",
    "Server.Items.StandardPlateKabuto",
    "Server.Items.StuddedDo",
    "Server.Items.StuddedHaidate",
    "Server.Items.StuddedHiroSode",
    "Server.Items.StuddedMempo",
    "Server.Items.StuddedSuneate",
];
pub static ARMOR_TYPES: &[&str] = &[
    "Server.Items.BoneArms",
    "Server.Items.BoneChest",
    "Server.Items.BoneGloves",
    "Server.Items.BoneLegs",
    "Server.Items.BoneHelm",
    "Server.Items.ChainChest",
    "Server.Items.ChainLegs",
    "Server.Items.ChainCoif",
    "Server.Items.Bascinet",
    "Server.Items.CloseHelm",
    "Server.Items.Helmet",
    "Server.Items.NorseHelm",
    "Server.Items.OrcHelm",
    "Server.Items.FemaleLeatherChest",
    "Server.Items.LeatherArms",
    "Server.Items.LeatherBustierArms",
    "Server.Items.LeatherChest",
    "Server.Items.LeatherGloves",
    "Server.Items.LeatherGorget",
    "Server.Items.LeatherLegs",
    "Server.Items.LeatherShorts",
    "Server.Items.LeatherSkirt",
    "Server.Items.LeatherCap",
    "Server.Items.FemalePlateChest",
    "Server.Items.PlateArms",
    "Server.Items.PlateChest",
    "Server.Items.PlateGloves",
    "Server.Items.PlateGorget",
    "Server.Items.PlateHelm",
    "Server.Items.PlateLegs",
    "Server.Items.RingmailArms",
    "Server.Items.RingmailChest",
    "Server.Items.RingmailGloves",
    "Server.Items.RingmailLegs",
    "Server.Items.FemaleStuddedChest",
    "Server.Items.StuddedArms",
    "Server.Items.StuddedBustierArms",
    "Server.Items.StuddedChest",
    "Server.Items.StuddedGloves",
    "Server.Items.StuddedGorget",
    "Server.Items.StuddedLegs",
];
pub static AOS_SHIELD_TYPES: &[&str] = &["Server.Items.ChaosShield", "Server.Items.OrderShield"];
pub static SHIELD_TYPES: &[&str] = &[
    "Server.Items.BronzeShield",
    "Server.Items.Buckler",
    "Server.Items.HeaterShield",
    "Server.Items.MetalShield",
    "Server.Items.MetalKiteShield",
    "Server.Items.WoodenKiteShield",
    "Server.Items.WoodenShield",
];
pub static GEM_TYPES: &[&str] = &[
    "Server.Items.Amber",
    "Server.Items.Amethyst",
    "Server.Items.Citrine",
    "Server.Items.Diamond",
    "Server.Items.Emerald",
    "Server.Items.Ruby",
    "Server.Items.Sapphire",
    "Server.Items.StarSapphire",
    "Server.Items.Tourmaline",
];
pub static JEWELRY_TYPES: &[&str] = &[
    "Server.Items.GoldRing",
    "Server.Items.GoldBracelet",
    "Server.Items.SilverRing",
    "Server.Items.SilverBracelet",
];
pub static REG_TYPES: &[&str] = &[
    "Server.Items.BlackPearl",
    "Server.Items.Bloodmoss",
    "Server.Items.Garlic",
    "Server.Items.Ginseng",
    "Server.Items.MandrakeRoot",
    "Server.Items.Nightshade",
    "Server.Items.SulfurousAsh",
    "Server.Items.SpidersSilk",
];
pub static NECRO_REG_TYPES: &[&str] = &[
    "Server.Items.BatWing",
    "Server.Items.GraveDust",
    "Server.Items.DaemonBlood",
    "Server.Items.NoxCrystal",
    "Server.Items.PigIron",
];
pub static POTION_TYPES: &[&str] = &[
    "Server.Items.AgilityPotion",
    "Server.Items.StrengthPotion",
    "Server.Items.RefreshPotion",
    "Server.Items.LesserCurePotion",
    "Server.Items.LesserHealPotion",
    "Server.Items.LesserPoisonPotion",
];
pub static SE_INSTRUMENT_TYPES: &[&str] = &["Server.Items.BambooFlute"];
pub static INSTRUMENT_TYPES: &[&str] = &[
    "Server.Items.Drums",
    "Server.Items.Harp",
    "Server.Items.LapHarp",
    "Server.Items.Lute",
    "Server.Items.Tambourine",
    "Server.Items.TambourineTassel",
];
pub static STATUE_TYPES: &[&str] = &[
    "Server.Items.StatueSouth",
    "Server.Items.StatueSouth2",
    "Server.Items.StatueNorth",
    "Server.Items.StatueWest",
    "Server.Items.StatueEast",
    "Server.Items.StatueEast2",
    "Server.Items.StatueSouthEast",
    "Server.Items.BustSouth",
    "Server.Items.BustEast",
];
pub static REGULAR_SCROLL_TYPES: &[&str] = &[
    "Server.Items.ReactiveArmorScroll",
    "Server.Items.ClumsyScroll",
    "Server.Items.CreateFoodScroll",
    "Server.Items.FeeblemindScroll",
    "Server.Items.HealScroll",
    "Server.Items.MagicArrowScroll",
    "Server.Items.NightSightScroll",
    "Server.Items.WeakenScroll",
    "Server.Items.AgilityScroll",
    "Server.Items.CunningScroll",
    "Server.Items.CureScroll",
    "Server.Items.HarmScroll",
    "Server.Items.MagicTrapScroll",
    "Server.Items.MagicUnTrapScroll",
    "Server.Items.ProtectionScroll",
    "Server.Items.StrengthScroll",
    "Server.Items.BlessScroll",
    "Server.Items.FireballScroll",
    "Server.Items.MagicLockScroll",
    "Server.Items.PoisonScroll",
    "Server.Items.TelekinesisScroll",
    "Server.Items.TeleportScroll",
    "Server.Items.UnlockScroll",
    "Server.Items.WallOfStoneScroll",
    "Server.Items.ArchCureScroll",
    "Server.Items.ArchProtectionScroll",
    "Server.Items.CurseScroll",
    "Server.Items.FireFieldScroll",
    "Server.Items.GreaterHealScroll",
    "Server.Items.LightningScroll",
    "Server.Items.ManaDrainScroll",
    "Server.Items.RecallScroll",
    "Server.Items.BladeSpiritsScroll",
    "Server.Items.DispelFieldScroll",
    "Server.Items.IncognitoScroll",
    "Server.Items.MagicReflectScroll",
    "Server.Items.MindBlastScroll",
    "Server.Items.ParalyzeScroll",
    "Server.Items.PoisonFieldScroll",
    "Server.Items.SummonCreatureScroll",
    "Server.Items.DispelScroll",
    "Server.Items.EnergyBoltScroll",
    "Server.Items.ExplosionScroll",
    "Server.Items.InvisibilityScroll",
    "Server.Items.MarkScroll",
    "Server.Items.MassCurseScroll",
    "Server.Items.ParalyzeFieldScroll",
    "Server.Items.RevealScroll",
    "Server.Items.ChainLightningScroll",
    "Server.Items.EnergyFieldScroll",
    "Server.Items.FlamestrikeScroll",
    "Server.Items.GateTravelScroll",
    "Server.Items.ManaVampireScroll",
    "Server.Items.MassDispelScroll",
    "Server.Items.MeteorSwarmScroll",
    "Server.Items.PolymorphScroll",
    "Server.Items.EarthquakeScroll",
    "Server.Items.EnergyVortexScroll",
    "Server.Items.ResurrectionScroll",
    "Server.Items.SummonAirElementalScroll",
    "Server.Items.SummonDaemonScroll",
    "Server.Items.SummonEarthElementalScroll",
    "Server.Items.SummonFireElementalScroll",
    "Server.Items.SummonWaterElementalScroll",
];
pub static NECROMANCY_SCROLL_TYPES: &[&str] = &[
    "Server.Items.AnimateDeadScroll",
    "Server.Items.BloodOathScroll",
    "Server.Items.CorpseSkinScroll",
    "Server.Items.CurseWeaponScroll",
    "Server.Items.EvilOmenScroll",
    "Server.Items.HorrificBeastScroll",
    "Server.Items.LichFormScroll",
    "Server.Items.MindRotScroll",
    "Server.Items.PainSpikeScroll",
    "Server.Items.PoisonStrikeScroll",
    "Server.Items.StrangleScroll",
    "Server.Items.SummonFamiliarScroll",
    "Server.Items.VampiricEmbraceScroll",
    "Server.Items.VengefulSpiritScroll",
    "Server.Items.WitherScroll",
    "Server.Items.WraithFormScroll",
];
pub static SE_NECROMANCY_SCROLL_TYPES: &[&str] = &[
    "Server.Items.AnimateDeadScroll",
    "Server.Items.BloodOathScroll",
    "Server.Items.CorpseSkinScroll",
    "Server.Items.CurseWeaponScroll",
    "Server.Items.EvilOmenScroll",
    "Server.Items.HorrificBeastScroll",
    "Server.Items.LichFormScroll",
    "Server.Items.MindRotScroll",
    "Server.Items.PainSpikeScroll",
    "Server.Items.PoisonStrikeScroll",
    "Server.Items.StrangleScroll",
    "Server.Items.SummonFamiliarScroll",
    "Server.Items.VampiricEmbraceScroll",
    "Server.Items.VengefulSpiritScroll",
    "Server.Items.WitherScroll",
    "Server.Items.WraithFormScroll",
    "Server.Items.ExorcismScroll",
];
pub static ARCANIST_SCROLL_TYPES: &[&str] = &[
    "Server.Items.ArcaneCircleScroll",
    "Server.Items.GiftOfRenewalScroll",
    "Server.Items.ImmolatingWeaponScroll",
    "Server.Items.AttuneWeaponScroll",
    "Server.Items.ThunderstormScroll",
    "Server.Items.NatureFuryScroll",
    "Server.Items.ReaperFormScroll",
    "Server.Items.WildfireScroll",
    "Server.Items.EssenceOfWindScroll",
    "Server.Items.DryadAllureScroll",
    "Server.Items.EtherealVoyageScroll",
    "Server.Items.WordOfDeathScroll",
    "Server.Items.GiftOfLifeScroll",
    "Server.Items.ArcaneEmpowermentScroll",
];
pub static GRIMMOCH_JOURNAL_TYPES: &[&str] = &[
    "Server.Items.GrimmochJournal1",
    "Server.Items.GrimmochJournal2",
    "Server.Items.GrimmochJournal3",
    "Server.Items.GrimmochJournal6",
    "Server.Items.GrimmochJournal7",
    "Server.Items.GrimmochJournal11",
    "Server.Items.GrimmochJournal14",
    "Server.Items.GrimmochJournal17",
    "Server.Items.GrimmochJournal23",
];
pub static LYSANDER_NOTEBOOK_TYPES: &[&str] = &[
    "Server.Items.LysanderNotebook1",
    "Server.Items.LysanderNotebook2",
    "Server.Items.LysanderNotebook3",
    "Server.Items.LysanderNotebook7",
    "Server.Items.LysanderNotebook8",
    "Server.Items.LysanderNotebook11",
];
pub static TAVARAS_JOURNAL_TYPES: &[&str] = &[
    "Server.Items.TavarasJournal1",
    "Server.Items.TavarasJournal2",
    "Server.Items.TavarasJournal3",
    "Server.Items.TavarasJournal6",
    "Server.Items.TavarasJournal7",
    "Server.Items.TavarasJournal8",
    "Server.Items.TavarasJournal9",
    "Server.Items.TavarasJournal11",
    "Server.Items.TavarasJournal14",
    "Server.Items.TavarasJournal16",
    "Server.Items.TavarasJournal16b",
    "Server.Items.TavarasJournal17",
    "Server.Items.TavarasJournal19",
];
pub static NEW_WAND_TYPES: &[&str] = &[
    "Server.Items.FireballWand",
    "Server.Items.LightningWand",
    "Server.Items.MagicArrowWand",
    "Server.Items.GreaterHealWand",
    "Server.Items.HarmWand",
    "Server.Items.HealWand",
];
pub static WAND_TYPES: &[&str] = &[
    "Server.Items.ClumsyWand",
    "Server.Items.FeebleWand",
    "Server.Items.ManaDrainWand",
    "Server.Items.WeaknessWand",
];
pub static OLD_WAND_TYPES: &[&str] = &["Server.Items.IDWand"];
pub static SE_CLOTHING_TYPES: &[&str] = &[
    "Server.Items.ClothNinjaJacket",
    "Server.Items.FemaleKimono",
    "Server.Items.Hakama",
    "Server.Items.HakamaShita",
    "Server.Items.JinBaori",
    "Server.Items.Kamishimo",
    "Server.Items.MaleKimono",
    "Server.Items.NinjaTabi",
    "Server.Items.Obi",
    "Server.Items.SamuraiTabi",
    "Server.Items.TattsukeHakama",
    "Server.Items.Waraji",
];
pub static AOS_CLOTHING_TYPES: &[&str] = &[
    "Server.Items.FurSarong",
    "Server.Items.FurCape",
    "Server.Items.FlowerGarland",
    "Server.Items.GildedDress",
    "Server.Items.FurBoots",
    "Server.Items.FormalShirt",
];
pub static CLOTHING_TYPES: &[&str] = &[
    "Server.Items.Cloak",
    "Server.Items.Bonnet",
    "Server.Items.Cap",
    "Server.Items.FeatheredHat",
    "Server.Items.FloppyHat",
    "Server.Items.JesterHat",
    "Server.Items.Surcoat",
    "Server.Items.SkullCap",
    "Server.Items.StrawHat",
    "Server.Items.TallStrawHat",
    "Server.Items.TricorneHat",
    "Server.Items.WideBrimHat",
    "Server.Items.WizardsHat",
    "Server.Items.BodySash",
    "Server.Items.Doublet",
    "Server.Items.Boots",
    "Server.Items.FullApron",
    "Server.Items.JesterSuit",
    "Server.Items.Sandals",
    "Server.Items.Tunic",
    "Server.Items.Shoes",
    "Server.Items.Shirt",
    "Server.Items.Kilt",
    "Server.Items.Skirt",
    "Server.Items.FancyShirt",
    "Server.Items.FancyDress",
    "Server.Items.ThighBoots",
    "Server.Items.LongPants",
    "Server.Items.PlainDress",
    "Server.Items.Robe",
    "Server.Items.ShortPants",
    "Server.Items.HalfApron",
];
pub static SE_HAT_TYPES: &[&str] = &["Server.Items.ClothNinjaHood", "Server.Items.Kasa"];
pub static AOS_HAT_TYPES: &[&str] = &[
    "Server.Items.FlowerGarland",
    "Server.Items.BearMask",
    "Server.Items.DeerMask",
];
pub static HAT_TYPES: &[&str] = &[
    "Server.Items.SkullCap",
    "Server.Items.Bandana",
    "Server.Items.FloppyHat",
    "Server.Items.Cap",
    "Server.Items.WideBrimHat",
    "Server.Items.StrawHat",
    "Server.Items.TallStrawHat",
    "Server.Items.WizardsHat",
    "Server.Items.Bonnet",
    "Server.Items.FeatheredHat",
    "Server.Items.TricorneHat",
    "Server.Items.JesterHat",
];
pub static LIBRARY_BOOK_TYPES: &[&str] = &[
    "Server.Items.GrammarOfOrcish",
    "Server.Items.CallToAnarchy",
    "Server.Items.ArmsAndWeaponsPrimer",
    "Server.Items.SongOfSamlethe",
    "Server.Items.TaleOfThreeTribes",
    "Server.Items.GuideToGuilds",
    "Server.Items.BirdsOfBritannia",
    "Server.Items.BritannianFlora",
    "Server.Items.ChildrenTalesVol2",
    "Server.Items.TalesOfVesperVol1",
    "Server.Items.DeceitDungeonOfHorror",
    "Server.Items.DimensionalTravel",
    "Server.Items.EthicalHedonism",
    "Server.Items.MyStory",
    "Server.Items.DiversityOfOurLand",
    "Server.Items.QuestOfVirtues",
    "Server.Items.RegardingLlamas",
    "Server.Items.TalkingToWisps",
    "Server.Items.TamingDragons",
    "Server.Items.BoldStranger",
    "Server.Items.BurningOfTrinsic",
    "Server.Items.TheFight",
    "Server.Items.LifeOfATravellingMinstrel",
    "Server.Items.MajorTradeAssociation",
    "Server.Items.RankingsOfTrades",
    "Server.Items.WildGirlOfTheForest",
    "Server.Items.TreatiseOnAlchemy",
    "Server.Items.VirtueBook",
];

fn ml(w: &World) -> bool {
    w.core_ml()
}
fn se(w: &World) -> bool {
    w.core_se()
}
fn aos(w: &World) -> bool {
    w.core_aos()
}
fn sa(w: &World) -> bool {
    w.expansion() >= muo_data::expansion::Expansion::SA
}

// ---- Construct --------------------------------------------------------------------------------------------

/// `Construct(type)`: `None` for an unknown class or a failed constructor.
pub fn construct(w: &mut World, name: &str) -> Option<ItemId> {
    make_item(w, name)
}

/// `Construct(types, index)`.
pub fn construct_index(w: &mut World, types: &[&str], index: i32) -> Option<ItemId> {
    if index >= 0 && (index as usize) < types.len() {
        construct(w, types[index as usize])
    } else {
        None
    }
}

/// `Construct(params Type[] types)`: a uniformly random element.
pub fn construct_any(w: &mut World, types: &[&str]) -> Option<ItemId> {
    let name = *utility::random_element(&mut w.rng, types)?;
    construct(w, name)
}

/// `Construct(params Type[][] types)`: one draw over the concatenation of all the lists.
pub fn construct_groups(w: &mut World, types: &[&[&str]]) -> Option<ItemId> {
    let total: usize = types.iter().map(|t| t.len()).sum();
    if total == 0 {
        return None;
    }
    let mut index = rnd::below(w, total as i32);
    for group in types {
        if index >= 0 && (index as usize) < group.len() {
            return construct(w, group[index as usize]);
        }
        index -= group.len() as i32;
    }
    None
}

// ---- the pickers ----------------------------------------------------------------------------------------

pub fn random_wand(w: &mut World) -> Option<ItemId> {
    if ml(w) {
        construct_any(w, NEW_WAND_TYPES)
    } else if aos(w) {
        construct_groups(w, &[WAND_TYPES, NEW_WAND_TYPES])
    } else {
        construct_groups(w, &[OLD_WAND_TYPES, WAND_TYPES, NEW_WAND_TYPES])
    }
}

pub fn random_clothing(w: &mut World, in_tokuno: bool, is_mondain: bool) -> Option<ItemId> {
    if ml(w) && is_mondain {
        construct_groups(w, &[ML_CLOTHING_TYPES, AOS_CLOTHING_TYPES, CLOTHING_TYPES])
    } else if se(w) && in_tokuno {
        construct_groups(w, &[SE_CLOTHING_TYPES, AOS_CLOTHING_TYPES, CLOTHING_TYPES])
    } else if aos(w) {
        construct_groups(w, &[AOS_CLOTHING_TYPES, CLOTHING_TYPES])
    } else {
        construct_any(w, CLOTHING_TYPES)
    }
}

pub fn random_ranged_weapon(
    w: &mut World,
    in_tokuno: bool,
    is_mondain: bool,
    is_stygian: bool,
) -> Option<ItemId> {
    if sa(w) && is_stygian {
        construct_groups(
            w,
            &[
                SA_RANGED_WEAPON_TYPES,
                AOS_RANGED_WEAPON_TYPES,
                RANGED_WEAPON_TYPES,
            ],
        )
    } else if ml(w) && is_mondain {
        construct_groups(
            w,
            &[
                ML_RANGED_WEAPON_TYPES,
                AOS_RANGED_WEAPON_TYPES,
                RANGED_WEAPON_TYPES,
            ],
        )
    } else if se(w) && in_tokuno {
        construct_groups(
            w,
            &[
                SE_RANGED_WEAPON_TYPES,
                AOS_RANGED_WEAPON_TYPES,
                RANGED_WEAPON_TYPES,
            ],
        )
    } else if aos(w) {
        construct_groups(w, &[AOS_RANGED_WEAPON_TYPES, RANGED_WEAPON_TYPES])
    } else {
        construct_any(w, RANGED_WEAPON_TYPES)
    }
}

pub fn random_weapon(
    w: &mut World,
    in_tokuno: bool,
    is_mondain: bool,
    is_stygian: bool,
) -> Option<ItemId> {
    if sa(w) && is_stygian {
        construct_groups(w, &[SA_WEAPON_TYPES, AOS_WEAPON_TYPES, WEAPON_TYPES])
    } else if ml(w) && is_mondain {
        construct_groups(w, &[ML_WEAPON_TYPES, AOS_WEAPON_TYPES, WEAPON_TYPES])
    } else if se(w) && in_tokuno {
        construct_groups(w, &[SE_WEAPON_TYPES, AOS_WEAPON_TYPES, WEAPON_TYPES])
    } else if aos(w) {
        construct_groups(w, &[AOS_WEAPON_TYPES, WEAPON_TYPES])
    } else {
        construct_any(w, WEAPON_TYPES)
    }
}

pub fn random_weapon_or_jewelry(
    w: &mut World,
    in_tokuno: bool,
    is_mondain: bool,
) -> Option<ItemId> {
    if ml(w) && is_mondain {
        construct_groups(
            w,
            &[
                ML_WEAPON_TYPES,
                AOS_WEAPON_TYPES,
                WEAPON_TYPES,
                JEWELRY_TYPES,
            ],
        )
    } else if se(w) && in_tokuno {
        construct_groups(
            w,
            &[
                SE_WEAPON_TYPES,
                AOS_WEAPON_TYPES,
                WEAPON_TYPES,
                JEWELRY_TYPES,
            ],
        )
    } else if aos(w) {
        construct_groups(w, &[AOS_WEAPON_TYPES, WEAPON_TYPES, JEWELRY_TYPES])
    } else {
        construct_groups(w, &[WEAPON_TYPES, JEWELRY_TYPES])
    }
}

pub fn random_jewelry(w: &mut World) -> Option<ItemId> {
    construct_any(w, JEWELRY_TYPES)
}

pub fn random_armor(w: &mut World, in_tokuno: bool, is_mondain: bool) -> Option<ItemId> {
    if ml(w) && is_mondain {
        construct_groups(w, &[ML_ARMOR_TYPES, ARMOR_TYPES])
    } else if se(w) && in_tokuno {
        construct_groups(w, &[SE_ARMOR_TYPES, ARMOR_TYPES])
    } else {
        construct_any(w, ARMOR_TYPES)
    }
}

pub fn random_hat(w: &mut World, in_tokuno: bool) -> Option<ItemId> {
    if se(w) && in_tokuno {
        construct_groups(w, &[SE_HAT_TYPES, AOS_HAT_TYPES, HAT_TYPES])
    } else if aos(w) {
        construct_groups(w, &[AOS_HAT_TYPES, HAT_TYPES])
    } else {
        construct_any(w, HAT_TYPES)
    }
}

pub fn random_armor_or_hat(w: &mut World, in_tokuno: bool, is_mondain: bool) -> Option<ItemId> {
    if ml(w) && is_mondain {
        construct_groups(w, &[ML_ARMOR_TYPES, ARMOR_TYPES, AOS_HAT_TYPES, HAT_TYPES])
    } else if se(w) && in_tokuno {
        construct_groups(
            w,
            &[
                SE_ARMOR_TYPES,
                ARMOR_TYPES,
                SE_HAT_TYPES,
                AOS_HAT_TYPES,
                HAT_TYPES,
            ],
        )
    } else if aos(w) {
        construct_groups(w, &[ARMOR_TYPES, AOS_HAT_TYPES, HAT_TYPES])
    } else {
        construct_groups(w, &[ARMOR_TYPES, HAT_TYPES])
    }
}

pub fn random_shield(w: &mut World) -> Option<ItemId> {
    if aos(w) {
        construct_groups(w, &[AOS_SHIELD_TYPES, SHIELD_TYPES])
    } else {
        construct_any(w, SHIELD_TYPES)
    }
}

pub fn random_armor_or_shield(w: &mut World, in_tokuno: bool, is_mondain: bool) -> Option<ItemId> {
    if ml(w) && is_mondain {
        construct_groups(
            w,
            &[ML_ARMOR_TYPES, ARMOR_TYPES, AOS_SHIELD_TYPES, SHIELD_TYPES],
        )
    } else if se(w) && in_tokuno {
        construct_groups(
            w,
            &[SE_ARMOR_TYPES, ARMOR_TYPES, AOS_SHIELD_TYPES, SHIELD_TYPES],
        )
    } else if aos(w) {
        construct_groups(w, &[ARMOR_TYPES, AOS_SHIELD_TYPES, SHIELD_TYPES])
    } else {
        construct_groups(w, &[ARMOR_TYPES, SHIELD_TYPES])
    }
}

pub fn random_armor_or_shield_or_jewelry(
    w: &mut World,
    in_tokuno: bool,
    is_mondain: bool,
) -> Option<ItemId> {
    if ml(w) && is_mondain {
        construct_groups(
            w,
            &[
                ML_ARMOR_TYPES,
                ARMOR_TYPES,
                AOS_HAT_TYPES,
                HAT_TYPES,
                AOS_SHIELD_TYPES,
                SHIELD_TYPES,
                JEWELRY_TYPES,
            ],
        )
    } else if se(w) && in_tokuno {
        construct_groups(
            w,
            &[
                SE_ARMOR_TYPES,
                ARMOR_TYPES,
                SE_HAT_TYPES,
                AOS_HAT_TYPES,
                HAT_TYPES,
                AOS_SHIELD_TYPES,
                SHIELD_TYPES,
                JEWELRY_TYPES,
            ],
        )
    } else if aos(w) {
        construct_groups(
            w,
            &[
                ARMOR_TYPES,
                AOS_HAT_TYPES,
                HAT_TYPES,
                AOS_SHIELD_TYPES,
                SHIELD_TYPES,
                JEWELRY_TYPES,
            ],
        )
    } else {
        construct_groups(w, &[ARMOR_TYPES, HAT_TYPES, SHIELD_TYPES, JEWELRY_TYPES])
    }
}

pub fn random_armor_or_shield_or_weapon(
    w: &mut World,
    in_tokuno: bool,
    is_mondain: bool,
) -> Option<ItemId> {
    if ml(w) && is_mondain {
        construct_groups(
            w,
            &[
                ML_WEAPON_TYPES,
                AOS_WEAPON_TYPES,
                WEAPON_TYPES,
                ML_RANGED_WEAPON_TYPES,
                AOS_RANGED_WEAPON_TYPES,
                RANGED_WEAPON_TYPES,
                ML_ARMOR_TYPES,
                ARMOR_TYPES,
                AOS_HAT_TYPES,
                HAT_TYPES,
                AOS_SHIELD_TYPES,
                SHIELD_TYPES,
            ],
        )
    } else if se(w) && in_tokuno {
        construct_groups(
            w,
            &[
                SE_WEAPON_TYPES,
                AOS_WEAPON_TYPES,
                WEAPON_TYPES,
                SE_RANGED_WEAPON_TYPES,
                AOS_RANGED_WEAPON_TYPES,
                RANGED_WEAPON_TYPES,
                SE_ARMOR_TYPES,
                ARMOR_TYPES,
                SE_HAT_TYPES,
                AOS_HAT_TYPES,
                HAT_TYPES,
                AOS_SHIELD_TYPES,
                SHIELD_TYPES,
            ],
        )
    } else if aos(w) {
        construct_groups(
            w,
            &[
                AOS_WEAPON_TYPES,
                WEAPON_TYPES,
                AOS_RANGED_WEAPON_TYPES,
                RANGED_WEAPON_TYPES,
                ARMOR_TYPES,
                AOS_HAT_TYPES,
                HAT_TYPES,
                AOS_SHIELD_TYPES,
                SHIELD_TYPES,
            ],
        )
    } else {
        construct_groups(
            w,
            &[
                WEAPON_TYPES,
                RANGED_WEAPON_TYPES,
                ARMOR_TYPES,
                HAT_TYPES,
                SHIELD_TYPES,
            ],
        )
    }
}

pub fn random_armor_or_shield_or_weapon_or_jewelry(
    w: &mut World,
    in_tokuno: bool,
    is_mondain: bool,
) -> Option<ItemId> {
    if ml(w) && is_mondain {
        construct_groups(
            w,
            &[
                ML_WEAPON_TYPES,
                AOS_WEAPON_TYPES,
                WEAPON_TYPES,
                ML_RANGED_WEAPON_TYPES,
                AOS_RANGED_WEAPON_TYPES,
                RANGED_WEAPON_TYPES,
                ML_ARMOR_TYPES,
                ARMOR_TYPES,
                AOS_HAT_TYPES,
                HAT_TYPES,
                AOS_SHIELD_TYPES,
                SHIELD_TYPES,
                JEWELRY_TYPES,
            ],
        )
    } else if se(w) && in_tokuno {
        construct_groups(
            w,
            &[
                SE_WEAPON_TYPES,
                AOS_WEAPON_TYPES,
                WEAPON_TYPES,
                SE_RANGED_WEAPON_TYPES,
                AOS_RANGED_WEAPON_TYPES,
                RANGED_WEAPON_TYPES,
                SE_ARMOR_TYPES,
                ARMOR_TYPES,
                SE_HAT_TYPES,
                AOS_HAT_TYPES,
                HAT_TYPES,
                AOS_SHIELD_TYPES,
                SHIELD_TYPES,
                JEWELRY_TYPES,
            ],
        )
    } else if aos(w) {
        construct_groups(
            w,
            &[
                AOS_WEAPON_TYPES,
                WEAPON_TYPES,
                AOS_RANGED_WEAPON_TYPES,
                RANGED_WEAPON_TYPES,
                ARMOR_TYPES,
                AOS_HAT_TYPES,
                HAT_TYPES,
                AOS_SHIELD_TYPES,
                SHIELD_TYPES,
                JEWELRY_TYPES,
            ],
        )
    } else {
        construct_groups(
            w,
            &[
                WEAPON_TYPES,
                RANGED_WEAPON_TYPES,
                ARMOR_TYPES,
                HAT_TYPES,
                SHIELD_TYPES,
                JEWELRY_TYPES,
            ],
        )
    }
}

pub fn random_chest_of_heirlooms_content(w: &mut World) -> Option<ItemId> {
    construct_groups(
        w,
        &[
            SE_ARMOR_TYPES,
            SE_HAT_TYPES,
            SE_WEAPON_TYPES,
            SE_RANGED_WEAPON_TYPES,
            JEWELRY_TYPES,
        ],
    )
}

pub fn random_gem(w: &mut World) -> Option<ItemId> {
    construct_any(w, GEM_TYPES)
}
pub fn random_reagent(w: &mut World) -> Option<ItemId> {
    construct_any(w, REG_TYPES)
}
pub fn random_necromancy_reagent(w: &mut World) -> Option<ItemId> {
    construct_any(w, NECRO_REG_TYPES)
}
pub fn random_possible_reagent(w: &mut World) -> Option<ItemId> {
    if aos(w) {
        construct_groups(w, &[REG_TYPES, NECRO_REG_TYPES])
    } else {
        construct_any(w, REG_TYPES)
    }
}
pub fn random_potion(w: &mut World) -> Option<ItemId> {
    construct_any(w, POTION_TYPES)
}
pub fn random_instrument(w: &mut World) -> Option<ItemId> {
    if se(w) {
        construct_groups(w, &[SE_INSTRUMENT_TYPES, INSTRUMENT_TYPES])
    } else {
        construct_any(w, INSTRUMENT_TYPES)
    }
}
pub fn random_statue(w: &mut World) -> Option<ItemId> {
    construct_any(w, STATUE_TYPES)
}

/// `RandomScroll(minIndex, maxIndex, type)`.
pub fn random_scroll(
    w: &mut World,
    min_index: i32,
    max_index: i32,
    ty: SpellbookType,
) -> Option<ItemId> {
    let types: &[&str] = match ty {
        SpellbookType::Regular => REGULAR_SCROLL_TYPES,
        SpellbookType::Necromancer => {
            if se(w) {
                SE_NECROMANCY_SCROLL_TYPES
            } else {
                NECROMANCY_SCROLL_TYPES
            }
        }
        SpellbookType::Paladin => PALADIN_SCROLL_TYPES,
        SpellbookType::Arcanist => ARCANIST_SCROLL_TYPES,
        _ => REGULAR_SCROLL_TYPES,
    };
    let i = rnd::min_max(w, min_index, max_index);
    construct_index(w, types, i)
}

pub fn random_grimmoch_journal(w: &mut World) -> Option<ItemId> {
    construct_any(w, GRIMMOCH_JOURNAL_TYPES)
}
pub fn random_lysander_notebook(w: &mut World) -> Option<ItemId> {
    construct_any(w, LYSANDER_NOTEBOOK_TYPES)
}
pub fn random_tavaras_journal(w: &mut World) -> Option<ItemId> {
    construct_any(w, TAVARAS_JOURNAL_TYPES)
}
pub fn random_library_book(w: &mut World) -> Option<ItemId> {
    construct_any(w, LIBRARY_BOOK_TYPES)
}

/// `Loot.RandomTalisman()`: a `BaseTalisman` rolled by `muo_items` (through `ContentHooks::random_talisman`).
pub fn random_talisman(w: &mut World) -> Option<ItemId> {
    let f = crate::hooks::hooks(w).and_then(|h| h.random_talisman)?;
    Some(f(w))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pools_have_the_expected_sizes() {
        assert_eq!(GEM_TYPES.len(), 9);
        assert_eq!(REGULAR_SCROLL_TYPES.len(), 64);
        assert_eq!(NECROMANCY_SCROLL_TYPES.len(), 16);
        assert_eq!(SE_NECROMANCY_SCROLL_TYPES.len(), 17);
        assert_eq!(ARCANIST_SCROLL_TYPES.len(), 14);
        assert!(PALADIN_SCROLL_TYPES.is_empty());
        assert_eq!(WEAPON_TYPES.len(), 36);
        assert_eq!(ARMOR_TYPES.len(), 41);
        assert!(GEM_TYPES.iter().all(|t| t.starts_with("Server.Items.")));
    }

    #[test]
    fn construct_over_an_unknown_class_is_none() {
        let mut w = World::new();
        assert_eq!(construct(&mut w, "Server.Items.NoSuchThing"), None);
        assert_eq!(
            construct_any(&mut w, GEM_TYPES),
            None,
            "no classes registered in a bare world"
        );
        assert_eq!(construct_groups(&mut w, &[&[], &[]]), None);
        assert_eq!(construct_index(&mut w, GEM_TYPES, 99), None);
    }

    #[test]
    fn scrolls_of_a_circle_index_by_eight() {
        // RandomScroll(minCircle * 8, maxCircle * 8 + 7) over the regular list: the first scroll of circle 2.
        assert_eq!(REGULAR_SCROLL_TYPES[8], "Server.Items.AgilityScroll");
        assert_eq!(REGULAR_SCROLL_TYPES[0], "Server.Items.ReactiveArmorScroll");
    }
}
