//! `FillableContentType` and the `FillableContent` of each (level, vendors that acquire it, weighted entries).

use muo_content::loot;

use super::fillable::{Content, Entry};

/// `FillableContentType` (`None` is -1).
pub const NAMES: &[&str] = &[
    "Weaponsmith",
    "Provisioner",
    "Mage",
    "Alchemist",
    "Armorer",
    "ArtisanGuild",
    "Baker",
    "Bard",
    "Blacksmith",
    "Bowyer",
    "Butcher",
    "Carpenter",
    "Clothier",
    "Cobbler",
    "Docks",
    "Farm",
    "FighterGuild",
    "Guard",
    "Healer",
    "Herbalist",
    "Inn",
    "Jeweler",
    "Library",
    "Merchant",
    "Mill",
    "Mine",
    "Observatory",
    "Painter",
    "Ranger",
    "Stables",
    "Tanner",
    "Tavern",
    "ThiefGuild",
    "Tinker",
    "Veterinarian",
];

/// `FillableContent.ContentTypes`, in the order of the enum.
pub static CONTENT_TYPES: &[Content] = &[
    Content {
        level: 2,
        vendors: &["Server.Mobiles.Weaponsmith"],
        entries: &[
            Entry::Loot {
                weight: 8,
                list: loot::WEAPON_TYPES,
                offset: 0,
                count: -1,
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Arrow"],
            },
        ],
    },
    Content {
        level: 1,
        vendors: &["Server.Mobiles.Provisioner"],
        entries: &[
            Entry::Types {
                weight: 1,
                types: &["Server.Items.CheeseWheel"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.CheeseWedge"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.CheeseSlice"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Eggs"],
            },
            Entry::Types {
                weight: 4,
                types: &["Server.Items.Fish"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.DirtyFrypan"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.DirtyPan"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.DirtyKettle"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.DirtySmallRoundPot"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.DirtyRoundPot"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.DirtySmallPot"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.DirtyPot"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Apple"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.Banana"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.Bananas"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.OpenCoconut"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.SplitCoconut"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Coconut"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Dates"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Grapes"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Lemon"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Lemons"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Lime"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Limes"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Peach"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Pear"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.SlabOfBacon"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.Bacon"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.RawFishSteak"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.FishSteak"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.CookedBird"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.RawBird"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.Ham"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.RawLambLeg"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.LambLeg"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Ribs"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.RawRibs"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.Sausage"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.RawChickenLeg"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.ChickenLeg"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Watermelon"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.SmallWatermelon"],
            },
            Entry::Types {
                weight: 3,
                types: &["Server.Items.Turnip"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.YellowGourd"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.GreenGourd"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.Pumpkin"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.SmallPumpkin"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.Onion"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.Lettuce"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.Squash"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.HoneydewMelon"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Carrot"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.Cantaloupe"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.Cabbage"],
            },
            Entry::Types {
                weight: 4,
                types: &["Server.Items.EarOfCorn"],
            },
        ],
    },
    Content {
        level: 2,
        vendors: &[
            "Server.Mobiles.Mage",
            "Server.Mobiles.HolyMage",
            "Server.Mobiles.MageGuildmaster",
        ],
        entries: &[
            Entry::Types {
                weight: 16,
                types: &["Server.Items.BlankScroll"],
            },
            Entry::Types {
                weight: 14,
                types: &["Server.Items.Spellbook"],
            },
            Entry::Loot {
                weight: 12,
                list: loot::REGULAR_SCROLL_TYPES,
                offset: 0,
                count: 8,
            },
            Entry::Loot {
                weight: 11,
                list: loot::REGULAR_SCROLL_TYPES,
                offset: 8,
                count: 8,
            },
            Entry::Loot {
                weight: 10,
                list: loot::REGULAR_SCROLL_TYPES,
                offset: 16,
                count: 8,
            },
            Entry::Loot {
                weight: 9,
                list: loot::REGULAR_SCROLL_TYPES,
                offset: 24,
                count: 8,
            },
            Entry::Loot {
                weight: 8,
                list: loot::REGULAR_SCROLL_TYPES,
                offset: 32,
                count: 8,
            },
            Entry::Loot {
                weight: 7,
                list: loot::REGULAR_SCROLL_TYPES,
                offset: 40,
                count: 8,
            },
            Entry::Loot {
                weight: 6,
                list: loot::REGULAR_SCROLL_TYPES,
                offset: 48,
                count: 8,
            },
            Entry::Loot {
                weight: 5,
                list: loot::REGULAR_SCROLL_TYPES,
                offset: 56,
                count: 8,
            },
        ],
    },
    Content {
        level: 1,
        vendors: &["Server.Mobiles.Alchemist"],
        entries: &[
            Entry::Types {
                weight: 1,
                types: &["Server.Items.NightSightPotion"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.LesserCurePotion"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.AgilityPotion"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.StrengthPotion"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.LesserPoisonPotion"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.RefreshPotion"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.LesserHealPotion"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.LesserExplosionPotion"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.MortarPestle"],
            },
        ],
    },
    Content {
        level: 2,
        vendors: &["Server.Mobiles.Armorer"],
        entries: &[
            Entry::Types {
                weight: 2,
                types: &["Server.Items.ChainCoif"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.PlateGorget"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.BronzeShield"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Buckler"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.MetalKiteShield"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.HeaterShield"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.WoodenShield"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.MetalShield"],
            },
        ],
    },
    Content {
        level: 1,
        vendors: &[],
        entries: &[
            Entry::Types {
                weight: 1,
                types: &["Server.Items.PaintsAndBrush"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.SledgeHammer"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.SmithHammer"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.Tongs"],
            },
            Entry::Types {
                weight: 4,
                types: &["Server.Items.Lockpick"],
            },
            Entry::Types {
                weight: 4,
                types: &["Server.Items.TinkerTools"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.MalletAndChisel"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.StatueEast2"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.StatueSouth"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.StatueSouthEast"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.StatueWest"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.StatueNorth"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.StatueEast"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.BustEast"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.BustSouth"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.BearMask"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.DeerMask"],
            },
            Entry::Types {
                weight: 4,
                types: &["Server.Items.OrcHelm"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.TribalMask"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.HornedTribalMask"],
            },
        ],
    },
    Content {
        level: 1,
        vendors: &["Server.Mobiles.Baker"],
        entries: &[
            Entry::Types {
                weight: 1,
                types: &["Server.Items.RollingPin"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.SackFlour"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.BreadLoaf"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.FrenchBread"],
            },
        ],
    },
    Content {
        level: 1,
        vendors: &["Server.Mobiles.Bard", "Server.Mobiles.BardGuildmaster"],
        entries: &[
            Entry::Types {
                weight: 1,
                types: &["Server.Items.LapHarp"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.Lute"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Drums"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Tambourine"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.TambourineTassel"],
            },
        ],
    },
    Content {
        level: 2,
        vendors: &[
            "Server.Mobiles.Blacksmith",
            "Server.Mobiles.BlacksmithGuildmaster",
        ],
        entries: &[
            Entry::Types {
                weight: 8,
                types: &["Server.Items.SmithHammer"],
            },
            Entry::Types {
                weight: 8,
                types: &["Server.Items.Tongs"],
            },
            Entry::Types {
                weight: 8,
                types: &["Server.Items.SledgeHammer"],
            },
            Entry::Types {
                weight: 8,
                types: &["Server.Items.IronIngot"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.IronWire"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.SilverWire"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.GoldWire"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.CopperWire"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.HorseShoes"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.ForgedMetal"],
            },
        ],
    },
    Content {
        level: 2,
        vendors: &["Server.Mobiles.Bowyer"],
        entries: &[
            Entry::Types {
                weight: 2,
                types: &["Server.Items.Bow"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.Crossbow"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Arrow"],
            },
        ],
    },
    Content {
        level: 1,
        vendors: &["Server.Mobiles.Butcher"],
        entries: &[
            Entry::Types {
                weight: 2,
                types: &["Server.Items.Cleaver"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.SlabOfBacon"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.Bacon"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.RawFishSteak"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.FishSteak"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.CookedBird"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.RawBird"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.Ham"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.RawLambLeg"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.LambLeg"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Ribs"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.RawRibs"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.Sausage"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.RawChickenLeg"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.ChickenLeg"],
            },
        ],
    },
    Content {
        level: 1,
        vendors: &[
            "Server.Mobiles.Carpenter",
            "Server.Mobiles.Architect",
            "Server.Mobiles.RealEstateBroker",
        ],
        entries: &[
            Entry::Types {
                weight: 1,
                types: &["Server.Items.ChiselsNorth"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.ChiselsWest"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.DovetailSaw"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.Hammer"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.MouldingPlane"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.Nails"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.JointingPlane"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.SmoothingPlane"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.Saw"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.DrawKnife"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Log"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Froe"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Inshave"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Scorp"],
            },
        ],
    },
    Content {
        level: 1,
        vendors: &[
            "Server.Mobiles.Tailor",
            "Server.Mobiles.Weaver",
            "Server.Mobiles.TailorGuildmaster",
        ],
        entries: &[
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Cotton"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Wool"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.DarkYarn"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.LightYarn"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.LightYarnUnraveled"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.SpoolOfThread"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Dyes"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.Leather"],
            },
        ],
    },
    Content {
        level: 1,
        vendors: &["Server.Mobiles.Cobbler"],
        entries: &[
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Boots"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.Shoes"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.Sandals"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.ThighBoots"],
            },
        ],
    },
    Content {
        level: 1,
        vendors: &[
            "Server.Mobiles.Fisherman",
            "Server.Mobiles.FisherGuildmaster",
        ],
        entries: &[
            Entry::Types {
                weight: 1,
                types: &["Server.Items.FishingPole"],
            },
            Entry::Types {
                weight: 4,
                types: &["Server.Items.Fish"],
            },
        ],
    },
    Content {
        level: 1,
        vendors: &["Server.Mobiles.Farmer", "Server.Mobiles.Rancher"],
        entries: &[
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Shirt"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.ShortPants"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Skirt"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.PlainDress"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Cap"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.Sandals"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.GnarledStaff"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.Pitchfork"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Bag"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Kindling"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Lettuce"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Onion"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Turnip"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Ham"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Bacon"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.RawLambLeg"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.SheafOfHay"],
            },
            Entry::Bvrge {
                weight: 1,
                ty: "Server.Items.Pitcher",
                content: "Milk",
            },
        ],
    },
    Content {
        level: 3,
        vendors: &["Server.Mobiles.WarriorGuildmaster"],
        entries: &[
            Entry::Loot {
                weight: 12,
                list: loot::ARMOR_TYPES,
                offset: 0,
                count: -1,
            },
            Entry::Loot {
                weight: 8,
                list: loot::WEAPON_TYPES,
                offset: 0,
                count: -1,
            },
            Entry::Loot {
                weight: 3,
                list: loot::SHIELD_TYPES,
                offset: 0,
                count: -1,
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Arrow"],
            },
        ],
    },
    Content {
        level: 3,
        vendors: &[],
        entries: &[
            Entry::Loot {
                weight: 12,
                list: loot::ARMOR_TYPES,
                offset: 0,
                count: -1,
            },
            Entry::Loot {
                weight: 8,
                list: loot::WEAPON_TYPES,
                offset: 0,
                count: -1,
            },
            Entry::Loot {
                weight: 3,
                list: loot::SHIELD_TYPES,
                offset: 0,
                count: -1,
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Arrow"],
            },
        ],
    },
    Content {
        level: 1,
        vendors: &["Server.Mobiles.Healer", "Server.Mobiles.HealerGuildmaster"],
        entries: &[
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Bandage"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.MortarPestle"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.LesserHealPotion"],
            },
        ],
    },
    Content {
        level: 1,
        vendors: &["Server.Mobiles.Herbalist"],
        entries: &[
            Entry::Types {
                weight: 10,
                types: &["Server.Items.Garlic"],
            },
            Entry::Types {
                weight: 10,
                types: &["Server.Items.Ginseng"],
            },
            Entry::Types {
                weight: 10,
                types: &["Server.Items.MandrakeRoot"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.DeadWood"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.WhiteDriedFlowers"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.GreenDriedFlowers"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.DriedOnions"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.DriedHerbs"],
            },
        ],
    },
    Content {
        level: 1,
        vendors: &[],
        entries: &[
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Candle"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Torch"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Lantern"],
            },
        ],
    },
    Content {
        level: 2,
        vendors: &["Server.Mobiles.Jeweler"],
        entries: &[
            Entry::Types {
                weight: 1,
                types: &["Server.Items.GoldRing"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.GoldBracelet"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.GoldEarrings"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.GoldNecklace"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.GoldBeadNecklace"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Necklace"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Beads"],
            },
            Entry::Loot {
                weight: 9,
                list: loot::GEM_TYPES,
                offset: 0,
                count: -1,
            },
        ],
    },
    Content {
        level: 1,
        vendors: &["Server.Mobiles.Scribe"],
        entries: &[
            Entry::Loot {
                weight: 8,
                list: loot::LIBRARY_BOOK_TYPES,
                offset: 0,
                count: -1,
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.RedBook"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.BlueBook"],
            },
        ],
    },
    Content {
        level: 1,
        vendors: &["Server.Mobiles.MerchantGuildmaster"],
        entries: &[
            Entry::Types {
                weight: 1,
                types: &["Server.Items.CheeseWheel"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.CheeseWedge"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.CheeseSlice"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Eggs"],
            },
            Entry::Types {
                weight: 4,
                types: &["Server.Items.Fish"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.RawFishSteak"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.FishSteak"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Apple"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.Banana"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.Bananas"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.OpenCoconut"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.SplitCoconut"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Coconut"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Dates"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Grapes"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Lemon"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Lemons"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Lime"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Limes"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Peach"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Pear"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.SlabOfBacon"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.Bacon"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.CookedBird"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.RawBird"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.Ham"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.RawLambLeg"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.LambLeg"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Ribs"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.RawRibs"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.Sausage"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.RawChickenLeg"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.ChickenLeg"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Watermelon"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.SmallWatermelon"],
            },
            Entry::Types {
                weight: 3,
                types: &["Server.Items.Turnip"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.YellowGourd"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.GreenGourd"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.Pumpkin"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.SmallPumpkin"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.Onion"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.Lettuce"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.Squash"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.HoneydewMelon"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Carrot"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.Cantaloupe"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.Cabbage"],
            },
            Entry::Types {
                weight: 4,
                types: &["Server.Items.EarOfCorn"],
            },
        ],
    },
    Content {
        level: 1,
        vendors: &[],
        entries: &[Entry::Types {
            weight: 1,
            types: &["Server.Items.SackFlour"],
        }],
    },
    Content {
        level: 1,
        vendors: &["Server.Mobiles.Miner"],
        entries: &[
            Entry::Types {
                weight: 2,
                types: &["Server.Items.Pickaxe"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.Shovel"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.IronIngot"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.ForgedMetal"],
            },
        ],
    },
    Content {
        level: 1,
        vendors: &[],
        entries: &[
            Entry::Types {
                weight: 2,
                types: &["Server.Items.Sextant"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.Clock"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Spyglass"],
            },
        ],
    },
    Content {
        level: 1,
        vendors: &[],
        entries: &[
            Entry::Types {
                weight: 1,
                types: &["Server.Items.PaintsAndBrush"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.PenAndInk"],
            },
        ],
    },
    Content {
        level: 2,
        vendors: &["Server.Mobiles.Ranger", "Server.Mobiles.RangerGuildmaster"],
        entries: &[
            Entry::Types {
                weight: 2,
                types: &["Server.Items.StuddedChest"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.StuddedLegs"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.StuddedArms"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.StuddedGloves"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.StuddedGorget"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.LeatherChest"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.LeatherLegs"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.LeatherArms"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.LeatherGloves"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.LeatherGorget"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.FeatheredHat"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.CloseHelm"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.TallStrawHat"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Bandana"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Cloak"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.Boots"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.ThighBoots"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.GnarledStaff"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Whip"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.Bow"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.Crossbow"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.HeavyCrossbow"],
            },
            Entry::Types {
                weight: 4,
                types: &["Server.Items.Arrow"],
            },
        ],
    },
    Content {
        level: 1,
        vendors: &[
            "Server.Mobiles.AnimalTrainer",
            "Server.Mobiles.GypsyAnimalTrainer",
        ],
        entries: &[Entry::Types {
            weight: 1,
            types: &["Server.Items.Carrot"],
        }],
    },
    Content {
        level: 2,
        vendors: &[
            "Server.Mobiles.Tanner",
            "Server.Mobiles.LeatherWorker",
            "Server.Mobiles.Furtrader",
        ],
        entries: &[
            Entry::Types {
                weight: 1,
                types: &["Server.Items.FeatheredHat"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.LeatherArms"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.LeatherLegs"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.LeatherChest"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.LeatherGloves"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.LeatherGorget"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.Leather"],
            },
        ],
    },
    Content {
        level: 1,
        vendors: &[
            "Server.Mobiles.TavernKeeper",
            "Server.Mobiles.Barkeeper",
            "Server.Mobiles.Waiter",
            "Server.Mobiles.Cook",
        ],
        entries: &[
            Entry::Bvrge {
                weight: 1,
                ty: "Server.Items.BeverageBottle",
                content: "Ale",
            },
            Entry::Bvrge {
                weight: 1,
                ty: "Server.Items.BeverageBottle",
                content: "Wine",
            },
            Entry::Bvrge {
                weight: 1,
                ty: "Server.Items.BeverageBottle",
                content: "Liquor",
            },
            Entry::Bvrge {
                weight: 1,
                ty: "Server.Items.Jug",
                content: "Cider",
            },
        ],
    },
    Content {
        level: 1,
        vendors: &["Server.Mobiles.Thief", "Server.Mobiles.ThiefGuildmaster"],
        entries: &[
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Lockpick"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.BearMask"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.DeerMask"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.TribalMask"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.HornedTribalMask"],
            },
            Entry::Types {
                weight: 4,
                types: &["Server.Items.OrcHelm"],
            },
        ],
    },
    Content {
        level: 1,
        vendors: &["Server.Mobiles.Tinker", "Server.Mobiles.TinkerGuildmaster"],
        entries: &[
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Lockpick"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.Clock"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.ClockParts"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.AxleGears"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.Gears"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.Hinge"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.Sextant"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.SextantParts"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.Axle"],
            },
            Entry::Types {
                weight: 2,
                types: &["Server.Items.Springs"],
            },
            Entry::Types {
                weight: 5,
                types: &["Server.Items.TinkerTools"],
            },
            Entry::Types {
                weight: 4,
                types: &["Server.Items.Key"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.DecoArrowShafts"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Lockpicks"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.ToolKit"],
            },
        ],
    },
    Content {
        level: 1,
        vendors: &["Server.Mobiles.Veterinarian"],
        entries: &[
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Bandage"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.MortarPestle"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.LesserHealPotion"],
            },
            Entry::Types {
                weight: 1,
                types: &["Server.Items.Carrot"],
            },
        ],
    },
];
