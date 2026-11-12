//! The `SBInfo` tables: what each vendor buys and sells.

use super::sb_helpers::*;
use muo_content::hooks::rnd;
use muo_content::vendor::{BuyInfo, SbInfo, SbInstance};
use muo_primitives::utility;
use muo_server::ids::MobileId;
use muo_server::world::World;

/// `SBChainmailArmor`.
pub static SBCHAINMAIL_ARMOR: SbInfo = SbInfo {
    name: "SBChainmailArmor",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.ChainCoif", 6);
        sb.sell.add("Server.Items.ChainChest", 71);
        sb.sell.add("Server.Items.ChainLegs", 74);
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.ChainCoif",
            17,
            20,
            0x13BB,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.ChainChest",
            143,
            20,
            0x13BF,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.ChainLegs",
            149,
            20,
            0x13BE,
            0,
        ));
    },
};

/// `SBHelmetArmor`.
pub static SBHELMET_ARMOR: SbInfo = SbInfo {
    name: "SBHelmetArmor",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.Bascinet", 9);
        sb.sell.add("Server.Items.CloseHelm", 9);
        sb.sell.add("Server.Items.Helmet", 9);
        sb.sell.add("Server.Items.NorseHelm", 9);
        sb.sell.add("Server.Items.PlateHelm", 10);
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.PlateHelm",
            21,
            20,
            0x1412,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.CloseHelm",
            18,
            20,
            0x1408,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.CloseHelm",
            18,
            20,
            0x1409,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Helmet", 31, 20, 0x140A, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Helmet", 18, 20, 0x140B, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.NorseHelm",
            18,
            20,
            0x140E,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.NorseHelm",
            18,
            20,
            0x140F,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Bascinet",
            18,
            20,
            0x140C,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.PlateHelm",
            21,
            20,
            0x1419,
            0,
        ));
    },
};

/// `SBLeatherArmor`.
pub static SBLEATHER_ARMOR: SbInfo = SbInfo {
    name: "SBLeatherArmor",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.LeatherArms", 40);
        sb.sell.add("Server.Items.LeatherChest", 52);
        sb.sell.add("Server.Items.LeatherGloves", 30);
        sb.sell.add("Server.Items.LeatherGorget", 37);
        sb.sell.add("Server.Items.LeatherLegs", 40);
        sb.sell.add("Server.Items.LeatherCap", 5);
        sb.sell.add("Server.Items.FemaleLeatherChest", 18);
        sb.sell.add("Server.Items.FemaleStuddedChest", 25);
        sb.sell.add("Server.Items.LeatherShorts", 14);
        sb.sell.add("Server.Items.LeatherSkirt", 11);
        sb.sell.add("Server.Items.LeatherBustierArms", 11);
        sb.sell.add("Server.Items.StuddedBustierArms", 27);
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.LeatherArms",
            80,
            20,
            0x13CD,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.LeatherChest",
            101,
            20,
            0x13CC,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.LeatherGloves",
            60,
            20,
            0x13C6,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.LeatherGorget",
            74,
            20,
            0x13C7,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.LeatherLegs",
            80,
            20,
            0x13cb,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.LeatherCap",
            10,
            20,
            0x1DB9,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.FemaleLeatherChest",
            116,
            20,
            0x1C06,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.LeatherBustierArms",
            97,
            20,
            0x1C0A,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.LeatherShorts",
            86,
            20,
            0x1C00,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.LeatherSkirt",
            87,
            20,
            0x1C08,
            0,
        ));
    },
};

/// `SBMetalShields`.
pub static SBMETAL_SHIELDS: SbInfo = SbInfo {
    name: "SBMetalShields",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.Buckler", 25);
        sb.sell.add("Server.Items.BronzeShield", 33);
        sb.sell.add("Server.Items.MetalShield", 60);
        sb.sell.add("Server.Items.MetalKiteShield", 62);
        sb.sell.add("Server.Items.HeaterShield", 115);
        sb.sell.add("Server.Items.WoodenKiteShield", 35);
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.BronzeShield",
            66,
            20,
            0x1B72,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Buckler",
            50,
            20,
            0x1B73,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.MetalKiteShield",
            123,
            20,
            0x1B74,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.HeaterShield",
            231,
            20,
            0x1B76,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.WoodenKiteShield",
            70,
            20,
            0x1B78,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.MetalShield",
            121,
            20,
            0x1B7B,
            0,
        ));
    },
};

/// `SBPlateArmor`.
pub static SBPLATE_ARMOR: SbInfo = SbInfo {
    name: "SBPlateArmor",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.PlateArms", 94);
        sb.sell.add("Server.Items.PlateChest", 121);
        sb.sell.add("Server.Items.PlateGloves", 72);
        sb.sell.add("Server.Items.PlateGorget", 52);
        sb.sell.add("Server.Items.PlateLegs", 109);
        sb.sell.add("Server.Items.FemalePlateChest", 113);
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.PlateGorget",
            104,
            20,
            0x1413,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.PlateChest",
            243,
            20,
            0x1415,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.PlateLegs",
            218,
            20,
            0x1411,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.PlateArms",
            188,
            20,
            0x1410,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.PlateGloves",
            155,
            20,
            0x1414,
            0,
        ));
    },
};

/// `SBRingmailArmor`.
pub static SBRINGMAIL_ARMOR: SbInfo = SbInfo {
    name: "SBRingmailArmor",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.RingmailArms", 42);
        sb.sell.add("Server.Items.RingmailChest", 60);
        sb.sell.add("Server.Items.RingmailGloves", 26);
        sb.sell.add("Server.Items.RingmailLegs", 45);
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.RingmailChest",
            121,
            20,
            0x13ec,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.RingmailLegs",
            90,
            20,
            0x13F0,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.RingmailArms",
            85,
            20,
            0x13EE,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.RingmailGloves",
            93,
            20,
            0x13eb,
            0,
        ));
    },
};

/// `SBStuddedArmor`.
pub static SBSTUDDED_ARMOR: SbInfo = SbInfo {
    name: "SBStuddedArmor",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.StuddedArms", 43);
        sb.sell.add("Server.Items.StuddedChest", 64);
        sb.sell.add("Server.Items.StuddedGloves", 39);
        sb.sell.add("Server.Items.StuddedGorget", 36);
        sb.sell.add("Server.Items.StuddedLegs", 51);
        sb.sell.add("Server.Items.FemaleStuddedChest", 71);
        sb.sell.add("Server.Items.StuddedBustierArms", 60);
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.StuddedArms",
            87,
            20,
            0x13DC,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.StuddedChest",
            128,
            20,
            0x13DB,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.StuddedGloves",
            79,
            20,
            0x13D5,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.StuddedGorget",
            73,
            20,
            0x13D6,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.StuddedLegs",
            103,
            20,
            0x13DA,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.FemaleStuddedChest",
            142,
            20,
            0x1C02,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.StuddedBustierArms",
            120,
            20,
            0x1c0c,
            0,
        ));
    },
};

/// `SBWoodenShields`.
pub static SBWOODEN_SHIELDS: SbInfo = SbInfo {
    name: "SBWoodenShields",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.WoodenShield", 15);
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.WoodenShield",
            30,
            20,
            0x1B7A,
            0,
        ));
    },
};

/// `SBAlchemist`.
pub static SBALCHEMIST: SbInfo = SbInfo {
    name: "SBAlchemist",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.BlackPearl", 3);
        sb.sell.add("Server.Items.Bloodmoss", 3);
        sb.sell.add("Server.Items.MandrakeRoot", 2);
        sb.sell.add("Server.Items.Garlic", 2);
        sb.sell.add("Server.Items.Ginseng", 2);
        sb.sell.add("Server.Items.Nightshade", 2);
        sb.sell.add("Server.Items.SpidersSilk", 2);
        sb.sell.add("Server.Items.SulfurousAsh", 2);
        sb.sell.add("Server.Items.Bottle", 3);
        sb.sell.add("Server.Items.MortarPestle", 4);
        sb.sell.add("Server.Items.HairDye", 19);
        sb.sell.add("Server.Items.NightSightPotion", 7);
        sb.sell.add("Server.Items.AgilityPotion", 7);
        sb.sell.add("Server.Items.StrengthPotion", 7);
        sb.sell.add("Server.Items.RefreshPotion", 7);
        sb.sell.add("Server.Items.LesserCurePotion", 7);
        sb.sell.add("Server.Items.LesserHealPotion", 7);
        sb.sell.add("Server.Items.LesserPoisonPotion", 7);
        sb.sell.add("Server.Items.LesserExplosionPotion", 10);
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.RefreshPotion",
            15,
            10,
            0xF0B,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.AgilityPotion",
            15,
            10,
            0xF08,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.NightSightPotion",
            15,
            10,
            0xF06,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.LesserHealPotion",
            15,
            10,
            0xF0C,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.StrengthPotion",
            15,
            10,
            0xF09,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.LesserPoisonPotion",
            15,
            10,
            0xF0A,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.LesserCurePotion",
            15,
            10,
            0xF07,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.LesserExplosionPotion",
            21,
            10,
            0xF0D,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.MortarPestle",
            8,
            10,
            0xE9B,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.BlackPearl",
            5,
            20,
            0xF7A,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Bloodmoss",
            5,
            20,
            0xF7B,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Garlic", 3, 20, 0xF84, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Ginseng", 3, 20, 0xF85, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.MandrakeRoot",
            3,
            20,
            0xF86,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Nightshade",
            3,
            20,
            0xF88,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.SpidersSilk",
            3,
            20,
            0xF8D,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.SulfurousAsh",
            3,
            20,
            0xF8C,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Bottle", 5, 100, 0xF0E, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.HeatingStand",
            2,
            100,
            0x1849,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            Some("1041060"),
            "Server.Items.HairDye",
            37,
            10,
            0xEFF,
            0,
        ));
    },
};

/// `SBAnimalTrainer`.
pub static SBANIMAL_TRAINER: SbInfo = SbInfo {
    name: "SBAnimalTrainer",
    build: |w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.buy.push(BuyInfo::animal(
            1,
            Some("a cat"),
            "Server.Mobiles.Cat",
            132,
            10,
            201,
            0,
        ));
        sb.buy.push(BuyInfo::animal(
            1,
            Some("a dog"),
            "Server.Mobiles.Dog",
            170,
            10,
            217,
            0,
        ));
        sb.buy.push(BuyInfo::animal(
            1,
            Some("a horse"),
            "Server.Mobiles.Horse",
            550,
            10,
            204,
            0,
        ));
        sb.buy.push(BuyInfo::animal(
            1,
            Some("a pack horse"),
            "Server.Mobiles.PackHorse",
            631,
            10,
            291,
            0,
        ));
        sb.buy.push(BuyInfo::animal(
            1,
            Some("a pack llama"),
            "Server.Mobiles.PackLlama",
            565,
            10,
            292,
            0,
        ));
        sb.buy.push(BuyInfo::animal(
            1,
            Some("a rabbit"),
            "Server.Mobiles.Rabbit",
            106,
            10,
            205,
            0,
        ));
        if !w.core_aos() {
            sb.buy.push(BuyInfo::animal(
                1,
                Some("an eagle"),
                "Server.Mobiles.Eagle",
                402,
                10,
                5,
                0,
            ));
            sb.buy.push(BuyInfo::animal(
                1,
                Some("a brown bear"),
                "Server.Mobiles.BrownBear",
                855,
                10,
                211,
                0,
            ));
            sb.buy.push(BuyInfo::animal(
                1,
                Some("a grizzly bear"),
                "Server.Mobiles.GrizzlyBear",
                1767,
                10,
                212,
                0,
            ));
            sb.buy.push(BuyInfo::animal(
                1,
                Some("a panther"),
                "Server.Mobiles.Panther",
                1271,
                10,
                214,
                0,
            ));
            sb.buy.push(BuyInfo::animal(
                1,
                Some("a timber wolf"),
                "Server.Mobiles.TimberWolf",
                768,
                10,
                225,
                0,
            ));
            sb.buy.push(BuyInfo::animal(
                1,
                Some("a rat"),
                "Server.Mobiles.Rat",
                107,
                10,
                238,
                0,
            ));
        }
    },
};

/// `SBArchitect`.
pub static SBARCHITECT: SbInfo = SbInfo {
    name: "SBArchitect",
    build: |w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.InteriorDecorator", 5000);
        if w.core_aos() {
            sb.sell.add("Server.Items.HousePlacementTool", 301);
        }
        sb.buy.push(BuyInfo::new(
            Some("1041280"),
            "Server.Items.InteriorDecorator",
            10001,
            20,
            0xFC1,
            0,
        ));
        if w.core_aos() {
            sb.buy.push(BuyInfo::new(
                Some("1060651"),
                "Server.Items.HousePlacementTool",
                627,
                20,
                0x14F6,
                0,
            ));
        }
    },
};

/// `SBBaker`.
pub static SBBAKER: SbInfo = SbInfo {
    name: "SBBaker",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.BreadLoaf", 3);
        sb.sell.add("Server.Items.FrenchBread", 1);
        sb.sell.add("Server.Items.Cake", 5);
        sb.sell.add("Server.Items.Cookies", 3);
        sb.sell.add("Server.Items.Muffins", 2);
        sb.sell.add("Server.Items.CheesePizza", 4);
        sb.sell.add("Server.Items.ApplePie", 5);
        sb.sell.add("Server.Items.PeachCobbler", 5);
        sb.sell.add("Server.Items.Quiche", 6);
        sb.sell.add("Server.Items.Dough", 4);
        sb.sell.add("Server.Items.JarHoney", 1);
        sb.sell.add("Server.Items.Pitcher", 5);
        sb.sell.add("Server.Items.SackFlour", 1);
        sb.sell.add("Server.Items.Eggs", 1);
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.BreadLoaf",
            6,
            20,
            0x103B,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.BreadLoaf",
            5,
            20,
            0x103C,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.ApplePie",
            7,
            20,
            0x1041,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Cake", 13, 20, 0x9E9, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Muffins", 3, 20, 0x9EA, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.SackFlour",
            3,
            20,
            0x1039,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.FrenchBread",
            5,
            20,
            0x98C,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Cookies", 3, 20, 0x160b, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.CheesePizza",
            8,
            10,
            0x1040,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.JarHoney", 3, 20, 0x9ec, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.BowlFlour",
            7,
            20,
            0xA1E,
            0,
        ));
    },
};

/// `SBBanker`.
pub static SBBANKER: SbInfo = SbInfo {
    name: "SBBanker",
    build: |w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.buy.push(BuyInfo::new(
            Some("1041243"),
            "Server.Items.ContractOfEmployment",
            1252,
            20,
            0x14F0,
            0,
        ));
        if muo_content::vendor::new_vendor_system(w) {
            sb.buy.push(BuyInfo::new(
                Some("1062332"),
                "Server.Items.VendorRentalContract",
                1252,
                20,
                0x14F0,
                0x672,
            ));
        }
        sb.buy.push(BuyInfo::new(
            Some("1047016"),
            "Server.Items.CommodityDeed",
            5,
            20,
            0x14F0,
            0x47,
        ));
    },
};

/// `SBBard`.
pub static SBBARD: SbInfo = SbInfo {
    name: "SBBard",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.LapHarp", 10);
        sb.sell.add("Server.Items.Lute", 10);
        sb.sell.add("Server.Items.Drums", 10);
        sb.sell.add("Server.Items.Harp", 10);
        sb.sell.add("Server.Items.Tambourine", 10);
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Drums", 21, 10, 0x0E9C, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Tambourine",
            21,
            10,
            0x0E9E,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.LapHarp",
            21,
            10,
            0x0EB2,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Lute", 21, 10, 0x0EB3, 0));
    },
};

/// `SBBarkeeper`.
pub static SBBARKEEPER: SbInfo = SbInfo {
    name: "SBBarkeeper",
    build: |w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.WoodenBowlOfCarrots", 1);
        sb.sell.add("Server.Items.WoodenBowlOfCorn", 1);
        sb.sell.add("Server.Items.WoodenBowlOfLettuce", 1);
        sb.sell.add("Server.Items.WoodenBowlOfPeas", 1);
        sb.sell.add("Server.Items.EmptyPewterBowl", 1);
        sb.sell.add("Server.Items.PewterBowlOfCorn", 1);
        sb.sell.add("Server.Items.PewterBowlOfLettuce", 1);
        sb.sell.add("Server.Items.PewterBowlOfPeas", 1);
        sb.sell.add("Server.Items.PewterBowlOfPotatos", 1);
        sb.sell.add("Server.Items.WoodenBowlOfStew", 1);
        sb.sell.add("Server.Items.WoodenBowlOfTomatoSoup", 1);
        sb.sell.add("Server.Items.BeverageBottle", 3);
        sb.sell.add("Server.Items.Jug", 6);
        sb.sell.add("Server.Items.Pitcher", 5);
        sb.sell.add("Server.Items.GlassMug", 1);
        sb.sell.add("Server.Items.BreadLoaf", 3);
        sb.sell.add("Server.Items.CheeseWheel", 12);
        sb.sell.add("Server.Items.Ribs", 6);
        sb.sell.add("Server.Items.Peach", 1);
        sb.sell.add("Server.Items.Pear", 1);
        sb.sell.add("Server.Items.Grapes", 1);
        sb.sell.add("Server.Items.Apple", 1);
        sb.sell.add("Server.Items.Banana", 1);
        sb.sell.add("Server.Items.Candle", 3);
        sb.sell.add("Server.Items.Chessboard", 1);
        sb.sell.add("Server.Items.CheckerBoard", 1);
        sb.sell.add("Server.Items.Backgammon", 1);
        sb.sell.add("Server.Items.Dices", 1);
        sb.sell.add("Server.Items.ContractOfEmployment", 626);
        sb.buy.push(BuyInfo::beverage(
            None,
            "Server.Items.BeverageBottle",
            0,
            7,
            20,
            0x99F,
            0,
        ));
        sb.buy.push(BuyInfo::beverage(
            None,
            "Server.Items.BeverageBottle",
            4,
            7,
            20,
            0x9C7,
            0,
        ));
        sb.buy.push(BuyInfo::beverage(
            None,
            "Server.Items.BeverageBottle",
            2,
            7,
            20,
            0x99B,
            0,
        ));
        sb.buy.push(BuyInfo::beverage(
            None,
            "Server.Items.Jug",
            1,
            13,
            20,
            0x9C8,
            0,
        ));
        sb.buy.push(BuyInfo::beverage(
            None,
            "Server.Items.Pitcher",
            3,
            7,
            20,
            0x9F0,
            0,
        ));
        sb.buy.push(BuyInfo::beverage(
            None,
            "Server.Items.Pitcher",
            0,
            11,
            20,
            0x1F95,
            0,
        ));
        sb.buy.push(BuyInfo::beverage(
            None,
            "Server.Items.Pitcher",
            1,
            11,
            20,
            0x1F97,
            0,
        ));
        sb.buy.push(BuyInfo::beverage(
            None,
            "Server.Items.Pitcher",
            2,
            11,
            20,
            0x1F99,
            0,
        ));
        sb.buy.push(BuyInfo::beverage(
            None,
            "Server.Items.Pitcher",
            4,
            11,
            20,
            0x1F9B,
            0,
        ));
        sb.buy.push(BuyInfo::beverage(
            None,
            "Server.Items.Pitcher",
            5,
            11,
            20,
            0x1F9D,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.BreadLoaf",
            6,
            10,
            0x103B,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.CheeseWheel",
            21,
            10,
            0x97E,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.CookedBird",
            17,
            20,
            0x9B7,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.LambLeg", 8, 20, 0x160A, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.WoodenBowlOfCarrots",
            3,
            20,
            0x15F9,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.WoodenBowlOfCorn",
            3,
            20,
            0x15FA,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.WoodenBowlOfLettuce",
            3,
            20,
            0x15FB,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.WoodenBowlOfPeas",
            3,
            20,
            0x15FC,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.EmptyPewterBowl",
            2,
            20,
            0x15FD,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.PewterBowlOfCorn",
            3,
            20,
            0x15FE,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.PewterBowlOfLettuce",
            3,
            20,
            0x15FF,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.PewterBowlOfPeas",
            3,
            20,
            0x1600,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.PewterBowlOfPotatos",
            3,
            20,
            0x1601,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.WoodenBowlOfStew",
            3,
            20,
            0x1604,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.WoodenBowlOfTomatoSoup",
            3,
            20,
            0x1606,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.ApplePie",
            7,
            20,
            0x1041,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            Some("1016450"),
            "Server.Items.Chessboard",
            2,
            20,
            0xFA6,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            Some("1016449"),
            "Server.Items.CheckerBoard",
            2,
            20,
            0xFA6,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Backgammon",
            2,
            20,
            0xE1C,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Dices", 2, 20, 0xFA7, 0));
        sb.buy.push(BuyInfo::new(
            Some("1041243"),
            "Server.Items.ContractOfEmployment",
            1252,
            20,
            0x14F0,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            Some("a barkeep contract"),
            "Server.Items.BarkeepContract",
            1252,
            20,
            0x14F0,
            0,
        ));
        if muo_content::vendor::new_vendor_system(w) {
            sb.buy.push(BuyInfo::new(
                Some("1062332"),
                "Server.Items.VendorRentalContract",
                1252,
                20,
                0x14F0,
                0x672,
            ));
        }
    },
};

/// `SBBeekeeper`.
pub static SBBEEKEEPER: SbInfo = SbInfo {
    name: "SBBeekeeper",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.JarHoney", 1);
        sb.sell.add("Server.Items.Beeswax", 1);
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.JarHoney", 3, 20, 0x9EC, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Beeswax", 2, 20, 0x1422, 0));
    },
};

/// `SBBlacksmith`.
pub static SBBLACKSMITH: SbInfo = SbInfo {
    name: "SBBlacksmith",
    build: |w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.Tongs", 7);
        sb.sell.add("Server.Items.IronIngot", 4);
        sb.sell.add("Server.Items.Buckler", 25);
        sb.sell.add("Server.Items.BronzeShield", 33);
        sb.sell.add("Server.Items.MetalShield", 60);
        sb.sell.add("Server.Items.MetalKiteShield", 62);
        sb.sell.add("Server.Items.HeaterShield", 115);
        sb.sell.add("Server.Items.WoodenKiteShield", 35);
        sb.sell.add("Server.Items.WoodenShield", 15);
        sb.sell.add("Server.Items.PlateArms", 94);
        sb.sell.add("Server.Items.PlateChest", 121);
        sb.sell.add("Server.Items.PlateGloves", 72);
        sb.sell.add("Server.Items.PlateGorget", 52);
        sb.sell.add("Server.Items.PlateLegs", 109);
        sb.sell.add("Server.Items.FemalePlateChest", 113);
        sb.sell.add("Server.Items.FemaleLeatherChest", 18);
        sb.sell.add("Server.Items.FemaleStuddedChest", 25);
        sb.sell.add("Server.Items.LeatherShorts", 14);
        sb.sell.add("Server.Items.LeatherSkirt", 11);
        sb.sell.add("Server.Items.LeatherBustierArms", 11);
        sb.sell.add("Server.Items.StuddedBustierArms", 27);
        sb.sell.add("Server.Items.Bascinet", 9);
        sb.sell.add("Server.Items.CloseHelm", 9);
        sb.sell.add("Server.Items.Helmet", 9);
        sb.sell.add("Server.Items.NorseHelm", 9);
        sb.sell.add("Server.Items.PlateHelm", 10);
        sb.sell.add("Server.Items.ChainCoif", 6);
        sb.sell.add("Server.Items.ChainChest", 71);
        sb.sell.add("Server.Items.ChainLegs", 74);
        sb.sell.add("Server.Items.RingmailArms", 42);
        sb.sell.add("Server.Items.RingmailChest", 60);
        sb.sell.add("Server.Items.RingmailGloves", 26);
        sb.sell.add("Server.Items.RingmailLegs", 45);
        sb.sell.add("Server.Items.BattleAxe", 13);
        sb.sell.add("Server.Items.DoubleAxe", 26);
        sb.sell.add("Server.Items.ExecutionersAxe", 15);
        sb.sell.add("Server.Items.LargeBattleAxe", 16);
        sb.sell.add("Server.Items.Pickaxe", 11);
        sb.sell.add("Server.Items.TwoHandedAxe", 16);
        sb.sell.add("Server.Items.WarAxe", 14);
        sb.sell.add("Server.Items.Axe", 20);
        sb.sell.add("Server.Items.Bardiche", 30);
        sb.sell.add("Server.Items.Halberd", 21);
        sb.sell.add("Server.Items.ButcherKnife", 7);
        sb.sell.add("Server.Items.Cleaver", 7);
        sb.sell.add("Server.Items.Dagger", 10);
        sb.sell.add("Server.Items.SkinningKnife", 7);
        sb.sell.add("Server.Items.Club", 8);
        sb.sell.add("Server.Items.HammerPick", 13);
        sb.sell.add("Server.Items.Mace", 14);
        sb.sell.add("Server.Items.Maul", 10);
        sb.sell.add("Server.Items.WarHammer", 12);
        sb.sell.add("Server.Items.WarMace", 15);
        sb.sell.add("Server.Items.HeavyCrossbow", 27);
        sb.sell.add("Server.Items.Bow", 17);
        sb.sell.add("Server.Items.Crossbow", 23);
        if w.core_aos() {
            sb.sell.add("Server.Items.CompositeBow", 25);
            sb.sell.add("Server.Items.RepeatingCrossbow", 28);
            sb.sell.add("Server.Items.Scepter", 20);
            sb.sell.add("Server.Items.BladedStaff", 20);
            sb.sell.add("Server.Items.Scythe", 19);
            sb.sell.add("Server.Items.BoneHarvester", 17);
            sb.sell.add("Server.Items.Scepter", 18);
            sb.sell.add("Server.Items.BladedStaff", 16);
            sb.sell.add("Server.Items.Pike", 19);
            sb.sell.add("Server.Items.DoubleBladedStaff", 17);
            sb.sell.add("Server.Items.Lance", 17);
            sb.sell.add("Server.Items.CrescentBlade", 18);
        }
        sb.sell.add("Server.Items.Spear", 15);
        sb.sell.add("Server.Items.Pitchfork", 9);
        sb.sell.add("Server.Items.ShortSpear", 11);
        sb.sell.add("Server.Items.BlackStaff", 11);
        sb.sell.add("Server.Items.GnarledStaff", 8);
        sb.sell.add("Server.Items.QuarterStaff", 9);
        sb.sell.add("Server.Items.ShepherdsCrook", 10);
        sb.sell.add("Server.Items.SmithHammer", 10);
        sb.sell.add("Server.Items.Broadsword", 17);
        sb.sell.add("Server.Items.Cutlass", 12);
        sb.sell.add("Server.Items.Katana", 16);
        sb.sell.add("Server.Items.Kryss", 16);
        sb.sell.add("Server.Items.Longsword", 27);
        sb.sell.add("Server.Items.Scimitar", 18);
        sb.sell.add("Server.Items.ThinLongsword", 13);
        sb.sell.add("Server.Items.VikingSword", 27);
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.IronIngot",
            5,
            16,
            0x1BF2,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Tongs", 13, 14, 0xFBB, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.BronzeShield",
            66,
            20,
            0x1B72,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Buckler",
            50,
            20,
            0x1B73,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.MetalKiteShield",
            123,
            20,
            0x1B74,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.HeaterShield",
            231,
            20,
            0x1B76,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.WoodenKiteShield",
            70,
            20,
            0x1B78,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.MetalShield",
            121,
            20,
            0x1B7B,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.WoodenShield",
            30,
            20,
            0x1B7A,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.PlateGorget",
            104,
            20,
            0x1413,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.PlateChest",
            243,
            20,
            0x1415,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.PlateLegs",
            218,
            20,
            0x1411,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.PlateArms",
            188,
            20,
            0x1410,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.PlateGloves",
            155,
            20,
            0x1414,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.PlateHelm",
            21,
            20,
            0x1412,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.CloseHelm",
            18,
            20,
            0x1408,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.CloseHelm",
            18,
            20,
            0x1409,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Helmet", 31, 20, 0x140A, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Helmet", 18, 20, 0x140B, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.NorseHelm",
            18,
            20,
            0x140E,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.NorseHelm",
            18,
            20,
            0x140F,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Bascinet",
            18,
            20,
            0x140C,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.PlateHelm",
            21,
            20,
            0x1419,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.ChainCoif",
            17,
            20,
            0x13BB,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.ChainChest",
            143,
            20,
            0x13BF,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.ChainLegs",
            149,
            20,
            0x13BE,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.RingmailChest",
            121,
            20,
            0x13ec,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.RingmailLegs",
            90,
            20,
            0x13F0,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.RingmailArms",
            85,
            20,
            0x13EE,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.RingmailGloves",
            93,
            20,
            0x13eb,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.ExecutionersAxe",
            30,
            20,
            0xF45,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Bardiche",
            60,
            20,
            0xF4D,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.BattleAxe",
            26,
            20,
            0xF47,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.TwoHandedAxe",
            32,
            20,
            0x1443,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Bow", 35, 20, 0x13B2, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.ButcherKnife",
            14,
            20,
            0x13F6,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Crossbow",
            46,
            20,
            0xF50,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.HeavyCrossbow",
            55,
            20,
            0x13FD,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Cutlass",
            24,
            20,
            0x1441,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Dagger", 21, 20, 0xF52, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Halberd",
            42,
            20,
            0x143E,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.HammerPick",
            26,
            20,
            0x143D,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Katana", 33, 20, 0x13FF, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Kryss", 32, 20, 0x1401, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Broadsword",
            35,
            20,
            0xF5E,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Longsword",
            55,
            20,
            0xF61,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.ThinLongsword",
            27,
            20,
            0x13B8,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.VikingSword",
            55,
            20,
            0x13B9,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Cleaver", 15, 20, 0xEC3, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Axe", 40, 20, 0xF49, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.DoubleAxe",
            52,
            20,
            0xF4B,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Pickaxe", 22, 20, 0xE86, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Pitchfork",
            19,
            20,
            0xE87,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Scimitar",
            36,
            20,
            0x13B6,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.SkinningKnife",
            14,
            20,
            0xEC4,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.LargeBattleAxe",
            33,
            20,
            0x13FB,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.WarAxe", 29, 20, 0x13B0, 0));
        if w.core_aos() {
            sb.buy.push(BuyInfo::new(
                None,
                "Server.Items.BoneHarvester",
                35,
                20,
                0x26BB,
                0,
            ));
            sb.buy.push(BuyInfo::new(
                None,
                "Server.Items.CrescentBlade",
                37,
                20,
                0x26C1,
                0,
            ));
            sb.buy.push(BuyInfo::new(
                None,
                "Server.Items.DoubleBladedStaff",
                35,
                20,
                0x26BF,
                0,
            ));
            sb.buy
                .push(BuyInfo::new(None, "Server.Items.Lance", 34, 20, 0x26C0, 0));
            sb.buy
                .push(BuyInfo::new(None, "Server.Items.Pike", 39, 20, 0x26BE, 0));
            sb.buy
                .push(BuyInfo::new(None, "Server.Items.Scythe", 39, 20, 0x26BA, 0));
            sb.buy.push(BuyInfo::new(
                None,
                "Server.Items.CompositeBow",
                50,
                20,
                0x26C2,
                0,
            ));
            sb.buy.push(BuyInfo::new(
                None,
                "Server.Items.RepeatingCrossbow",
                57,
                20,
                0x26C3,
                0,
            ));
        }
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.BlackStaff",
            22,
            20,
            0xDF1,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Club", 16, 20, 0x13B4, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.GnarledStaff",
            16,
            20,
            0x13F8,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Mace", 28, 20, 0xF5C, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Maul", 21, 20, 0x143B, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.QuarterStaff",
            19,
            20,
            0xE89,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.ShepherdsCrook",
            20,
            20,
            0xE81,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.SmithHammer",
            21,
            20,
            0x13E3,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.ShortSpear",
            23,
            20,
            0x1403,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Spear", 31, 20, 0xF62, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.WarHammer",
            25,
            20,
            0x1439,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.WarMace",
            31,
            20,
            0x1407,
            0,
        ));
        if w.core_aos() {
            sb.buy.push(BuyInfo::new(
                None,
                "Server.Items.Scepter",
                39,
                20,
                0x26BC,
                0,
            ));
            sb.buy.push(BuyInfo::new(
                None,
                "Server.Items.BladedStaff",
                40,
                20,
                0x26BD,
                0,
            ));
        }
    },
};

/// `SBBowyer`.
pub static SBBOWYER: SbInfo = SbInfo {
    name: "SBBowyer",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.FletcherTools", 1);
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.FletcherTools",
            2,
            20,
            0x1022,
            0,
        ));
    },
};

/// `SBButcher`.
pub static SBBUTCHER: SbInfo = SbInfo {
    name: "SBButcher",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.RawRibs", 8);
        sb.sell.add("Server.Items.RawLambLeg", 4);
        sb.sell.add("Server.Items.RawChickenLeg", 3);
        sb.sell.add("Server.Items.RawBird", 4);
        sb.sell.add("Server.Items.Bacon", 3);
        sb.sell.add("Server.Items.Sausage", 9);
        sb.sell.add("Server.Items.Ham", 13);
        sb.sell.add("Server.Items.ButcherKnife", 7);
        sb.sell.add("Server.Items.Cleaver", 7);
        sb.sell.add("Server.Items.SkinningKnife", 7);
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Bacon", 7, 20, 0x979, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Ham", 26, 20, 0x9C9, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Sausage", 18, 20, 0x9C0, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.RawChickenLeg",
            6,
            20,
            0x1607,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.RawBird", 9, 20, 0x9B9, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.RawLambLeg",
            9,
            20,
            0x1609,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.RawRibs", 16, 20, 0x9F1, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.ButcherKnife",
            13,
            20,
            0x13F6,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Cleaver", 13, 20, 0xEC3, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.SkinningKnife",
            13,
            20,
            0xEC4,
            0,
        ));
    },
};

/// `SBCarpenter`.
pub static SBCARPENTER: SbInfo = SbInfo {
    name: "SBCarpenter",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.WoodenBox", 7);
        sb.sell.add("Server.Items.SmallCrate", 5);
        sb.sell.add("Server.Items.MediumCrate", 6);
        sb.sell.add("Server.Items.LargeCrate", 7);
        sb.sell.add("Server.Items.WoodenChest", 15);
        sb.sell.add("Server.Items.LargeTable", 10);
        sb.sell.add("Server.Items.Nightstand", 7);
        sb.sell.add("Server.Items.YewWoodTable", 10);
        sb.sell.add("Server.Items.Throne", 24);
        sb.sell.add("Server.Items.WoodenThrone", 6);
        sb.sell.add("Server.Items.Stool", 6);
        sb.sell.add("Server.Items.FootStool", 6);
        sb.sell.add("Server.Items.FancyWoodenChairCushion", 12);
        sb.sell.add("Server.Items.WoodenChairCushion", 10);
        sb.sell.add("Server.Items.WoodenChair", 8);
        sb.sell.add("Server.Items.BambooChair", 6);
        sb.sell.add("Server.Items.WoodenBench", 6);
        sb.sell.add("Server.Items.Saw", 9);
        sb.sell.add("Server.Items.Scorp", 6);
        sb.sell.add("Server.Items.SmoothingPlane", 6);
        sb.sell.add("Server.Items.DrawKnife", 6);
        sb.sell.add("Server.Items.Froe", 6);
        sb.sell.add("Server.Items.Hammer", 14);
        sb.sell.add("Server.Items.Inshave", 6);
        sb.sell.add("Server.Items.JointingPlane", 6);
        sb.sell.add("Server.Items.MouldingPlane", 6);
        sb.sell.add("Server.Items.DovetailSaw", 7);
        sb.sell.add("Server.Items.Board", 2);
        sb.sell.add("Server.Items.Axle", 1);
        sb.sell.add("Server.Items.Club", 13);
        sb.sell.add("Server.Items.Lute", 10);
        sb.sell.add("Server.Items.LapHarp", 10);
        sb.sell.add("Server.Items.Tambourine", 10);
        sb.sell.add("Server.Items.Drums", 10);
        sb.sell.add("Server.Items.Log", 1);
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Nails", 3, 20, 0x102E, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Axle", 2, 20, 0x105B, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Board", 3, 20, 0x1BD7, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.DrawKnife",
            10,
            20,
            0x10E4,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Froe", 10, 20, 0x10E5, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Scorp", 10, 20, 0x10E7, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Inshave",
            10,
            20,
            0x10E6,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.DovetailSaw",
            12,
            20,
            0x1028,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Saw", 15, 20, 0x1034, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Hammer", 17, 20, 0x102A, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.MouldingPlane",
            11,
            20,
            0x102C,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.SmoothingPlane",
            10,
            20,
            0x1032,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.JointingPlane",
            11,
            20,
            0x1030,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Drums", 21, 20, 0xE9C, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Tambourine",
            21,
            20,
            0xE9D,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.LapHarp", 21, 20, 0xEB2, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Lute", 21, 20, 0xEB3, 0));
    },
};

/// `SBCobbler`.
pub static SBCOBBLER: SbInfo = SbInfo {
    name: "SBCobbler",
    build: |w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.Shoes", 4);
        sb.sell.add("Server.Items.Boots", 5);
        sb.sell.add("Server.Items.ThighBoots", 7);
        sb.sell.add("Server.Items.Sandals", 2);
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.ThighBoots",
            15,
            20,
            0x1711,
            utility::random_neutral_hue(&mut w.rng),
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Shoes",
            8,
            20,
            0x170f,
            utility::random_neutral_hue(&mut w.rng),
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Boots",
            10,
            20,
            0x170b,
            utility::random_neutral_hue(&mut w.rng),
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Sandals",
            5,
            20,
            0x170d,
            utility::random_neutral_hue(&mut w.rng),
        ));
    },
};

/// `SBCook`.
pub static SBCOOK: SbInfo = SbInfo {
    name: "SBCook",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.CheeseWheel", 12);
        sb.sell.add("Server.Items.CookedBird", 8);
        sb.sell.add("Server.Items.RoastPig", 53);
        sb.sell.add("Server.Items.Cake", 5);
        sb.sell.add("Server.Items.JarHoney", 1);
        sb.sell.add("Server.Items.SackFlour", 1);
        sb.sell.add("Server.Items.BreadLoaf", 2);
        sb.sell.add("Server.Items.ChickenLeg", 3);
        sb.sell.add("Server.Items.LambLeg", 4);
        sb.sell.add("Server.Items.Skillet", 1);
        sb.sell.add("Server.Items.FlourSifter", 1);
        sb.sell.add("Server.Items.RollingPin", 1);
        sb.sell.add("Server.Items.Muffins", 1);
        sb.sell.add("Server.Items.ApplePie", 3);
        sb.sell.add("Server.Items.WoodenBowlOfCarrots", 1);
        sb.sell.add("Server.Items.WoodenBowlOfCorn", 1);
        sb.sell.add("Server.Items.WoodenBowlOfLettuce", 1);
        sb.sell.add("Server.Items.WoodenBowlOfPeas", 1);
        sb.sell.add("Server.Items.EmptyPewterBowl", 1);
        sb.sell.add("Server.Items.PewterBowlOfCorn", 1);
        sb.sell.add("Server.Items.PewterBowlOfLettuce", 1);
        sb.sell.add("Server.Items.PewterBowlOfPeas", 1);
        sb.sell.add("Server.Items.PewterBowlOfPotatos", 1);
        sb.sell.add("Server.Items.WoodenBowlOfStew", 1);
        sb.sell.add("Server.Items.WoodenBowlOfTomatoSoup", 1);
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.BreadLoaf",
            5,
            20,
            0x103B,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.BreadLoaf",
            5,
            20,
            0x103C,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.ApplePie",
            7,
            20,
            0x1041,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Cake", 13, 20, 0x9E9, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Muffins", 3, 20, 0x9EA, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.CheeseWheel",
            21,
            10,
            0x97E,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.CookedBird",
            17,
            20,
            0x9B7,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.LambLeg", 8, 20, 0x160A, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.ChickenLeg",
            5,
            20,
            0x1608,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.WoodenBowlOfCarrots",
            3,
            20,
            0x15F9,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.WoodenBowlOfCorn",
            3,
            20,
            0x15FA,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.WoodenBowlOfLettuce",
            3,
            20,
            0x15FB,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.WoodenBowlOfPeas",
            3,
            20,
            0x15FC,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.EmptyPewterBowl",
            2,
            20,
            0x15FD,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.PewterBowlOfCorn",
            3,
            20,
            0x15FE,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.PewterBowlOfLettuce",
            3,
            20,
            0x15FF,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.PewterBowlOfPeas",
            3,
            20,
            0x1600,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.PewterBowlOfPotatos",
            3,
            20,
            0x1601,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.WoodenBowlOfStew",
            3,
            20,
            0x1604,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.WoodenBowlOfTomatoSoup",
            3,
            20,
            0x1606,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.RoastPig",
            106,
            20,
            0x9BB,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.SackFlour",
            3,
            20,
            0x1039,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.JarHoney", 3, 20, 0x9EC, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.RollingPin",
            2,
            20,
            0x1043,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.FlourSifter",
            2,
            20,
            0x103E,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            Some("1044567"),
            "Server.Items.Skillet",
            3,
            20,
            0x97F,
            0,
        ));
    },
};

/// `SBFarmer`.
pub static SBFARMER: SbInfo = SbInfo {
    name: "SBFarmer",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.Pitcher", 5);
        sb.sell.add("Server.Items.Eggs", 1);
        sb.sell.add("Server.Items.Apple", 1);
        sb.sell.add("Server.Items.Grapes", 1);
        sb.sell.add("Server.Items.Watermelon", 3);
        sb.sell.add("Server.Items.YellowGourd", 1);
        sb.sell.add("Server.Items.GreenGourd", 1);
        sb.sell.add("Server.Items.Pumpkin", 5);
        sb.sell.add("Server.Items.Onion", 1);
        sb.sell.add("Server.Items.Lettuce", 2);
        sb.sell.add("Server.Items.Squash", 1);
        sb.sell.add("Server.Items.Carrot", 1);
        sb.sell.add("Server.Items.HoneydewMelon", 3);
        sb.sell.add("Server.Items.Cantaloupe", 3);
        sb.sell.add("Server.Items.Cabbage", 2);
        sb.sell.add("Server.Items.Lemon", 1);
        sb.sell.add("Server.Items.Lime", 1);
        sb.sell.add("Server.Items.Peach", 1);
        sb.sell.add("Server.Items.Pear", 1);
        sb.sell.add("Server.Items.SheafOfHay", 1);
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Cabbage", 5, 20, 0xC7B, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Cantaloupe",
            6,
            20,
            0xC79,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Carrot", 3, 20, 0xC78, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.HoneydewMelon",
            7,
            20,
            0xC74,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Squash", 3, 20, 0xC72, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Lettuce", 5, 20, 0xC70, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Onion", 3, 20, 0xC6D, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Pumpkin", 11, 20, 0xC6A, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.GreenGourd",
            3,
            20,
            0xC66,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.YellowGourd",
            3,
            20,
            0xC64,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Watermelon",
            7,
            20,
            0xC5C,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Eggs", 3, 20, 0x9B5, 0));
        sb.buy.push(BuyInfo::beverage(
            None,
            "Server.Items.Pitcher",
            3,
            7,
            20,
            0x9AD,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Peach", 3, 20, 0x9D2, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Pear", 3, 20, 0x994, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Lemon", 3, 20, 0x1728, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Lime", 3, 20, 0x172A, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Grapes", 3, 20, 0x9D1, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Apple", 3, 20, 0x9D0, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.SheafOfHay",
            2,
            20,
            0xF36,
            0,
        ));
    },
};

/// `SBFisherman`.
pub static SBFISHERMAN: SbInfo = SbInfo {
    name: "SBFisherman",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.RawFishSteak", 1);
        sb.sell.add("Server.Items.Fish", 1);
        sb.sell.add("Server.Items.FishingPole", 7);
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.RawFishSteak",
            3,
            20,
            0x97A,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Fish", 6, 80, 0x9CC, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Fish", 6, 80, 0x9CD, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Fish", 6, 80, 0x9CE, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Fish", 6, 80, 0x9CF, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.FishingPole",
            15,
            20,
            0xDC0,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.AquariumFishNet",
            250,
            20,
            0xDC8,
            0x240,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.AquariumFood",
            62,
            20,
            0xEFC,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.FishBowl",
            6312,
            20,
            0x241C,
            0x482,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.VacationWafer",
            67,
            20,
            0x971,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.AquariumNorthDeed",
            250002,
            20,
            0x14F0,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.AquariumEastDeed",
            250002,
            20,
            0x14F0,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.NewAquariumBook",
            15,
            20,
            0xFF2,
            0,
        ));
    },
};

/// `SBFortuneTeller`.
pub static SBFORTUNE_TELLER: SbInfo = SbInfo {
    name: "SBFortuneTeller",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.Bandage", 1);
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Bandage", 5, 20, 0xE21, 0));
    },
};

/// `SBFurtrader`.
pub static SBFURTRADER: SbInfo = SbInfo {
    name: "SBFurtrader",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.Hides", 2);
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Hides", 3, 40, 0x1079, 0));
    },
};

/// `SBGlassblower`.
pub static SBGLASSBLOWER: SbInfo = SbInfo {
    name: "SBGlassblower",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.BlackPearl", 3);
        sb.sell.add("Server.Items.Bloodmoss", 3);
        sb.sell.add("Server.Items.MandrakeRoot", 2);
        sb.sell.add("Server.Items.Garlic", 2);
        sb.sell.add("Server.Items.Ginseng", 2);
        sb.sell.add("Server.Items.Nightshade", 2);
        sb.sell.add("Server.Items.SpidersSilk", 2);
        sb.sell.add("Server.Items.SulfurousAsh", 2);
        sb.sell.add("Server.Items.Bottle", 3);
        sb.sell.add("Server.Items.MortarPestle", 4);
        sb.sell.add("Server.Items.NightSightPotion", 7);
        sb.sell.add("Server.Items.AgilityPotion", 7);
        sb.sell.add("Server.Items.StrengthPotion", 7);
        sb.sell.add("Server.Items.RefreshPotion", 7);
        sb.sell.add("Server.Items.LesserCurePotion", 7);
        sb.sell.add("Server.Items.LesserHealPotion", 7);
        sb.sell.add("Server.Items.LesserPoisonPotion", 7);
        sb.sell.add("Server.Items.LesserExplosionPotion", 10);
        sb.sell.add("Server.Items.GlassblowingBook", 5000);
        sb.sell.add("Server.Items.SandMiningBook", 5000);
        sb.sell.add("Server.Items.Blowpipe", 10);
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.RefreshPotion",
            15,
            10,
            0xF0B,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.AgilityPotion",
            15,
            10,
            0xF08,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.NightSightPotion",
            15,
            10,
            0xF06,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.LesserHealPotion",
            15,
            10,
            0xF0C,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.StrengthPotion",
            15,
            10,
            0xF09,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.LesserPoisonPotion",
            15,
            10,
            0xF0A,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.LesserCurePotion",
            15,
            10,
            0xF07,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.LesserExplosionPotion",
            21,
            10,
            0xF0D,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.MortarPestle",
            8,
            10,
            0xE9B,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.BlackPearl",
            5,
            20,
            0xF7A,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Bloodmoss",
            5,
            20,
            0xF7B,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Garlic", 3, 20, 0xF84, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Ginseng", 3, 20, 0xF85, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.MandrakeRoot",
            3,
            20,
            0xF86,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Nightshade",
            3,
            20,
            0xF88,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.SpidersSilk",
            3,
            20,
            0xF8D,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.SulfurousAsh",
            3,
            20,
            0xF8C,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Bottle", 5, 100, 0xF0E, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.HeatingStand",
            2,
            100,
            0x1849,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            Some("Crafting Glass With Glassblowing"),
            "Server.Items.GlassblowingBook",
            10637,
            30,
            0xFF4,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            Some("Finding Glass-Quality Sand"),
            "Server.Items.SandMiningBook",
            10637,
            30,
            0xFF4,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            Some("1044608"),
            "Server.Items.Blowpipe",
            21,
            100,
            0xE8A,
            0x3B9,
        ));
    },
};

/// `SBHairStylist`.
pub static SBHAIR_STYLIST: SbInfo = SbInfo {
    name: "SBHairStylist",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.HairDye", 30);
        sb.sell.add("Server.Items.SpecialBeardDye", 250000);
        sb.sell.add("Server.Items.SpecialHairDye", 250000);
        sb.buy.push(BuyInfo::new(
            Some("special beard dye"),
            "Server.Items.SpecialBeardDye",
            500000,
            20,
            0xE26,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            Some("special hair dye"),
            "Server.Items.SpecialHairDye",
            500000,
            20,
            0xE26,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            Some("1041060"),
            "Server.Items.HairDye",
            60,
            20,
            0xEFF,
            0,
        ));
    },
};

/// `SBHealer`.
pub static SBHEALER: SbInfo = SbInfo {
    name: "SBHealer",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.Bandage", 1);
        sb.sell.add("Server.Items.LesserHealPotion", 7);
        sb.sell.add("Server.Items.RefreshPotion", 7);
        sb.sell.add("Server.Items.Garlic", 2);
        sb.sell.add("Server.Items.Ginseng", 2);
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Bandage", 5, 20, 0xE21, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.LesserHealPotion",
            15,
            20,
            0xF0C,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Ginseng", 3, 20, 0xF85, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Garlic", 3, 20, 0xF84, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.RefreshPotion",
            15,
            20,
            0xF0B,
            0,
        ));
    },
};

/// `SBHerbalist`.
pub static SBHERBALIST: SbInfo = SbInfo {
    name: "SBHerbalist",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.Bloodmoss", 3);
        sb.sell.add("Server.Items.MandrakeRoot", 2);
        sb.sell.add("Server.Items.Garlic", 2);
        sb.sell.add("Server.Items.Ginseng", 2);
        sb.sell.add("Server.Items.Nightshade", 2);
        sb.sell.add("Server.Items.Bottle", 3);
        sb.sell.add("Server.Items.MortarPestle", 4);
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Ginseng", 3, 20, 0xF85, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Garlic", 3, 20, 0xF84, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.MandrakeRoot",
            3,
            20,
            0xF86,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Nightshade",
            3,
            20,
            0xF88,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Bloodmoss",
            5,
            20,
            0xF7B,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.MortarPestle",
            8,
            20,
            0xE9B,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Bottle", 5, 20, 0xF0E, 0));
    },
};

/// `SBHolyMage`.
pub static SBHOLY_MAGE: SbInfo = SbInfo {
    name: "SBHolyMage",
    build: |w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.BlackPearl", 3);
        sb.sell.add("Server.Items.Bloodmoss", 3);
        sb.sell.add("Server.Items.MandrakeRoot", 2);
        sb.sell.add("Server.Items.Garlic", 2);
        sb.sell.add("Server.Items.Ginseng", 2);
        sb.sell.add("Server.Items.Nightshade", 2);
        sb.sell.add("Server.Items.SpidersSilk", 2);
        sb.sell.add("Server.Items.SulfurousAsh", 2);
        sb.sell.add("Server.Items.RecallRune", 8);
        sb.sell.add("Server.Items.Spellbook", 9);
        sb.sell.add("Server.Items.BlankScroll", 3);
        sb.sell.add("Server.Items.NightSightPotion", 7);
        sb.sell.add("Server.Items.AgilityPotion", 7);
        sb.sell.add("Server.Items.StrengthPotion", 7);
        sb.sell.add("Server.Items.RefreshPotion", 7);
        sb.sell.add("Server.Items.LesserCurePotion", 7);
        sb.sell.add("Server.Items.LesserHealPotion", 7);
        scroll_sell(sb, 2);
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Spellbook",
            18,
            10,
            0xEFA,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.ScribesPen",
            8,
            10,
            0xFBF,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.BlankScroll",
            5,
            20,
            0x0E34,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            Some("1041072"),
            "Server.Items.MagicWizardsHat",
            11,
            10,
            0x1718,
            utility::random_dyed_hue(&mut w.rng),
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.RecallRune",
            15,
            10,
            0x1f14,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.RefreshPotion",
            15,
            20,
            0xF0B,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.AgilityPotion",
            15,
            20,
            0xF08,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.NightSightPotion",
            15,
            20,
            0xF06,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.LesserHealPotion",
            15,
            20,
            0xF0C,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.StrengthPotion",
            15,
            20,
            0xF09,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.LesserCurePotion",
            15,
            20,
            0xF07,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.BlackPearl",
            5,
            20,
            0xF7A,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Bloodmoss",
            5,
            20,
            0xF7B,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Garlic", 3, 20, 0xF84, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Ginseng", 3, 20, 0xF85, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.MandrakeRoot",
            3,
            20,
            0xF86,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Nightshade",
            3,
            20,
            0xF88,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.SpidersSilk",
            3,
            20,
            0xF8D,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.SulfurousAsh",
            3,
            20,
            0xF8C,
            0,
        ));
        scroll_buy(w, sb, 8);
    },
};

/// `SBHouseDeed`.
pub static SBHOUSE_DEED: SbInfo = SbInfo {
    name: "SBHouseDeed",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.buy.push(BuyInfo::new(
            Some("deed to a stone-and-plaster house"),
            "Server.Multis.Deeds.StonePlasterHouseDeed",
            43800,
            20,
            0x14F0,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            Some("deed to a field stone house"),
            "Server.Multis.Deeds.FieldStoneHouseDeed",
            43800,
            20,
            0x14F0,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            Some("deed to a small brick house"),
            "Server.Multis.Deeds.SmallBrickHouseDeed",
            43800,
            20,
            0x14F0,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            Some("deed to a wooden house"),
            "Server.Multis.Deeds.WoodHouseDeed",
            43800,
            20,
            0x14F0,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            Some("deed to a wood-and-plaster house"),
            "Server.Multis.Deeds.WoodPlasterHouseDeed",
            43800,
            20,
            0x14F0,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            Some("deed to a thatched-roof cottage"),
            "Server.Multis.Deeds.ThatchedRoofCottageDeed",
            43800,
            20,
            0x14F0,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            Some("deed to a brick house"),
            "Server.Multis.Deeds.BrickHouseDeed",
            144500,
            20,
            0x14F0,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            Some("deed to a two-story wood-and-plaster house"),
            "Server.Multis.Deeds.TwoStoryWoodPlasterHouseDeed",
            192400,
            20,
            0x14F0,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            Some("deed to a tower"),
            "Server.Multis.Deeds.TowerDeed",
            433200,
            20,
            0x14F0,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            Some("deed to a small stone keep"),
            "Server.Multis.Deeds.KeepDeed",
            665200,
            20,
            0x14F0,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            Some("deed to a castle"),
            "Server.Multis.Deeds.CastleDeed",
            1022800,
            20,
            0x14F0,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            Some("deed to a large house with patio"),
            "Server.Multis.Deeds.LargePatioDeed",
            152800,
            20,
            0x14F0,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            Some("deed to a marble house with patio"),
            "Server.Multis.Deeds.LargeMarbleDeed",
            192000,
            20,
            0x14F0,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            Some("deed to a small stone tower"),
            "Server.Multis.Deeds.SmallTowerDeed",
            88500,
            20,
            0x14F0,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            Some("deed to a two story log cabin"),
            "Server.Multis.Deeds.LogCabinDeed",
            97800,
            20,
            0x14F0,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            Some("deed to a sandstone house with patio"),
            "Server.Multis.Deeds.SandstonePatioDeed",
            90900,
            20,
            0x14F0,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            Some("deed to a two story villa"),
            "Server.Multis.Deeds.VillaDeed",
            136500,
            20,
            0x14F0,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            Some("deed to a small stone workshop"),
            "Server.Multis.Deeds.StoneWorkshopDeed",
            60600,
            20,
            0x14F0,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            Some("deed to a small marble workshop"),
            "Server.Multis.Deeds.MarbleWorkshopDeed",
            63000,
            20,
            0x14F0,
            0,
        ));
    },
};

/// `SBInnKeeper`.
pub static SBINN_KEEPER: SbInfo = SbInfo {
    name: "SBInnKeeper",
    build: |w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.BeverageBottle", 3);
        sb.sell.add("Server.Items.Jug", 6);
        sb.sell.add("Server.Items.Pitcher", 5);
        sb.sell.add("Server.Items.GlassMug", 1);
        sb.sell.add("Server.Items.BreadLoaf", 3);
        sb.sell.add("Server.Items.CheeseWheel", 12);
        sb.sell.add("Server.Items.Ribs", 6);
        sb.sell.add("Server.Items.Peach", 1);
        sb.sell.add("Server.Items.Pear", 1);
        sb.sell.add("Server.Items.Grapes", 1);
        sb.sell.add("Server.Items.Apple", 1);
        sb.sell.add("Server.Items.Banana", 1);
        sb.sell.add("Server.Items.Torch", 3);
        sb.sell.add("Server.Items.Candle", 3);
        sb.sell.add("Server.Items.Chessboard", 1);
        sb.sell.add("Server.Items.CheckerBoard", 1);
        sb.sell.add("Server.Items.Backgammon", 1);
        sb.sell.add("Server.Items.Dices", 1);
        sb.sell.add("Server.Items.ContractOfEmployment", 626);
        sb.sell.add("Server.Items.Beeswax", 1);
        sb.sell.add("Server.Items.WoodenBowlOfCarrots", 1);
        sb.sell.add("Server.Items.WoodenBowlOfCorn", 1);
        sb.sell.add("Server.Items.WoodenBowlOfLettuce", 1);
        sb.sell.add("Server.Items.WoodenBowlOfPeas", 1);
        sb.sell.add("Server.Items.EmptyPewterBowl", 1);
        sb.sell.add("Server.Items.PewterBowlOfCorn", 1);
        sb.sell.add("Server.Items.PewterBowlOfLettuce", 1);
        sb.sell.add("Server.Items.PewterBowlOfPeas", 1);
        sb.sell.add("Server.Items.PewterBowlOfPotatos", 1);
        sb.sell.add("Server.Items.WoodenBowlOfStew", 1);
        sb.sell.add("Server.Items.WoodenBowlOfTomatoSoup", 1);
        sb.buy.push(BuyInfo::beverage(
            None,
            "Server.Items.BeverageBottle",
            0,
            7,
            20,
            0x99F,
            0,
        ));
        sb.buy.push(BuyInfo::beverage(
            None,
            "Server.Items.BeverageBottle",
            4,
            7,
            20,
            0x9C7,
            0,
        ));
        sb.buy.push(BuyInfo::beverage(
            None,
            "Server.Items.BeverageBottle",
            2,
            7,
            20,
            0x99B,
            0,
        ));
        sb.buy.push(BuyInfo::beverage(
            None,
            "Server.Items.Jug",
            1,
            13,
            20,
            0x9C8,
            0,
        ));
        sb.buy.push(BuyInfo::beverage(
            None,
            "Server.Items.Pitcher",
            3,
            7,
            20,
            0x9F0,
            0,
        ));
        sb.buy.push(BuyInfo::beverage(
            None,
            "Server.Items.Pitcher",
            0,
            11,
            20,
            0x1F95,
            0,
        ));
        sb.buy.push(BuyInfo::beverage(
            None,
            "Server.Items.Pitcher",
            1,
            11,
            20,
            0x1F97,
            0,
        ));
        sb.buy.push(BuyInfo::beverage(
            None,
            "Server.Items.Pitcher",
            2,
            11,
            20,
            0x1F99,
            0,
        ));
        sb.buy.push(BuyInfo::beverage(
            None,
            "Server.Items.Pitcher",
            4,
            11,
            20,
            0x1F9B,
            0,
        ));
        sb.buy.push(BuyInfo::beverage(
            None,
            "Server.Items.Pitcher",
            5,
            11,
            20,
            0x1F9D,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.BreadLoaf",
            6,
            10,
            0x103B,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.CheeseWheel",
            21,
            10,
            0x97E,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.CookedBird",
            17,
            20,
            0x9B7,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.LambLeg", 8, 20, 0x160A, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.ChickenLeg",
            5,
            20,
            0x1608,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Ribs", 7, 20, 0x9F2, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.WoodenBowlOfCarrots",
            3,
            20,
            0x15F9,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.WoodenBowlOfCorn",
            3,
            20,
            0x15FA,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.WoodenBowlOfLettuce",
            3,
            20,
            0x15FB,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.WoodenBowlOfPeas",
            3,
            20,
            0x15FC,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.EmptyPewterBowl",
            2,
            20,
            0x15FD,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.PewterBowlOfCorn",
            3,
            20,
            0x15FE,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.PewterBowlOfLettuce",
            3,
            20,
            0x15FF,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.PewterBowlOfPeas",
            3,
            20,
            0x1600,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.PewterBowlOfPotatos",
            3,
            20,
            0x1601,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.WoodenBowlOfStew",
            3,
            20,
            0x1604,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.WoodenBowlOfTomatoSoup",
            3,
            20,
            0x1606,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.ApplePie",
            7,
            20,
            0x1041,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Peach", 3, 20, 0x9D2, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Pear", 3, 20, 0x994, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Grapes", 3, 20, 0x9D1, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Apple", 3, 20, 0x9D0, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Banana", 2, 20, 0x171F, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Torch", 7, 20, 0xF6B, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Candle", 6, 20, 0xA28, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Beeswax", 1, 20, 0x1422, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Backpack",
            15,
            20,
            0x9B2,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            Some("1016450"),
            "Server.Items.Chessboard",
            2,
            20,
            0xFA6,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            Some("1016449"),
            "Server.Items.CheckerBoard",
            2,
            20,
            0xFA6,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Backgammon",
            2,
            20,
            0xE1C,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Dices", 2, 20, 0xFA7, 0));
        sb.buy.push(BuyInfo::new(
            Some("1041243"),
            "Server.Items.ContractOfEmployment",
            1252,
            20,
            0x14F0,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            Some("a barkeep contract"),
            "Server.Items.BarkeepContract",
            1252,
            20,
            0x14F0,
            0,
        ));
        if muo_content::vendor::new_vendor_system(w) {
            sb.buy.push(BuyInfo::new(
                Some("1062332"),
                "Server.Items.VendorRentalContract",
                1252,
                20,
                0x14F0,
                0x672,
            ));
        }
    },
};

/// `SBJewel`.
pub static SBJEWEL: SbInfo = SbInfo {
    name: "SBJewel",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.Amber", 25);
        sb.sell.add("Server.Items.Amethyst", 50);
        sb.sell.add("Server.Items.Citrine", 25);
        sb.sell.add("Server.Items.Diamond", 100);
        sb.sell.add("Server.Items.Emerald", 50);
        sb.sell.add("Server.Items.Ruby", 37);
        sb.sell.add("Server.Items.Sapphire", 50);
        sb.sell.add("Server.Items.StarSapphire", 62);
        sb.sell.add("Server.Items.Tourmaline", 47);
        sb.sell.add("Server.Items.GoldRing", 13);
        sb.sell.add("Server.Items.SilverRing", 10);
        sb.sell.add("Server.Items.Necklace", 13);
        sb.sell.add("Server.Items.GoldNecklace", 13);
        sb.sell.add("Server.Items.GoldBeadNecklace", 13);
        sb.sell.add("Server.Items.SilverNecklace", 10);
        sb.sell.add("Server.Items.SilverBeadNecklace", 10);
        sb.sell.add("Server.Items.Beads", 13);
        sb.sell.add("Server.Items.GoldBracelet", 13);
        sb.sell.add("Server.Items.SilverBracelet", 10);
        sb.sell.add("Server.Items.GoldEarrings", 13);
        sb.sell.add("Server.Items.SilverEarrings", 10);
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.GoldRing",
            27,
            20,
            0x108A,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Necklace",
            26,
            20,
            0x1085,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.GoldNecklace",
            27,
            20,
            0x1088,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.GoldBeadNecklace",
            27,
            20,
            0x1089,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Beads", 27, 20, 0x108B, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.GoldBracelet",
            27,
            20,
            0x1086,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.GoldEarrings",
            27,
            20,
            0x1087,
            0,
        ));
        sb.buy.push(
            BuyInfo::new(
                Some("1060740"),
                "Server.Items.BroadcastCrystal",
                68,
                20,
                0x1ED0,
                0,
            )
            .with_arg(500),
        );
        sb.buy.push(
            BuyInfo::new(
                Some("1060740"),
                "Server.Items.BroadcastCrystal",
                131,
                20,
                0x1ED0,
                0,
            )
            .with_arg(1000),
        );
        sb.buy.push(
            BuyInfo::new(
                Some("1060740"),
                "Server.Items.BroadcastCrystal",
                256,
                20,
                0x1ED0,
                0,
            )
            .with_arg(2000),
        );
        sb.buy.push(BuyInfo::new(
            Some("1060740"),
            "Server.Items.ReceiverCrystal",
            6,
            20,
            0x1ED0,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.StarSapphire",
            125,
            20,
            0xF21,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Emerald",
            100,
            20,
            0xF10,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Sapphire",
            100,
            20,
            0xF19,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Ruby", 75, 20, 0xF13, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Citrine", 50, 20, 0xF15, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Amethyst",
            100,
            20,
            0xF16,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Tourmaline",
            75,
            20,
            0xF2D,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Amber", 50, 20, 0xF25, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Diamond",
            200,
            20,
            0xF26,
            0,
        ));
    },
};

/// `SBKeeperOfChivalry`.
pub static SBKEEPER_OF_CHIVALRY: SbInfo = SbInfo {
    name: "SBKeeperOfChivalry",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.BookOfChivalry",
            140,
            20,
            0x2252,
            0,
        ));
    },
};

/// `SBLeatherWorker`.
pub static SBLEATHER_WORKER: SbInfo = SbInfo {
    name: "SBLeatherWorker",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.Hides", 2);
        sb.sell.add("Server.Items.ThighBoots", 7);
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Hides", 4, 999, 0x1078, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.ThighBoots",
            56,
            10,
            0x1711,
            0,
        ));
    },
};

/// `SBMage`.
pub static SBMAGE: SbInfo = SbInfo {
    name: "SBMage",
    build: |w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.WizardsHat", 15);
        sb.sell.add("Server.Items.BlackPearl", 3);
        sb.sell.add("Server.Items.Bloodmoss", 4);
        sb.sell.add("Server.Items.MandrakeRoot", 2);
        sb.sell.add("Server.Items.Garlic", 2);
        sb.sell.add("Server.Items.Ginseng", 2);
        sb.sell.add("Server.Items.Nightshade", 2);
        sb.sell.add("Server.Items.SpidersSilk", 2);
        sb.sell.add("Server.Items.SulfurousAsh", 2);
        if w.core_aos() {
            sb.sell.add("Server.Items.BatWing", 1);
            sb.sell.add("Server.Items.DaemonBlood", 3);
            sb.sell.add("Server.Items.PigIron", 2);
            sb.sell.add("Server.Items.NoxCrystal", 3);
            sb.sell.add("Server.Items.GraveDust", 1);
        }
        sb.sell.add("Server.Items.RecallRune", 13);
        sb.sell.add("Server.Items.Spellbook", 9);
        scroll_sell(sb, 2);
        if w.core_se() {
            sb.sell.add("Server.Items.ExorcismScroll", 3);
            sb.sell.add("Server.Items.AnimateDeadScroll", 8);
            sb.sell.add("Server.Items.BloodOathScroll", 8);
            sb.sell.add("Server.Items.CorpseSkinScroll", 8);
            sb.sell.add("Server.Items.CurseWeaponScroll", 8);
            sb.sell.add("Server.Items.EvilOmenScroll", 8);
            sb.sell.add("Server.Items.PainSpikeScroll", 8);
            sb.sell.add("Server.Items.SummonFamiliarScroll", 8);
            sb.sell.add("Server.Items.HorrificBeastScroll", 8);
            sb.sell.add("Server.Items.MindRotScroll", 10);
            sb.sell.add("Server.Items.PoisonStrikeScroll", 10);
            sb.sell.add("Server.Items.WraithFormScroll", 15);
            sb.sell.add("Server.Items.LichFormScroll", 16);
            sb.sell.add("Server.Items.StrangleScroll", 16);
            sb.sell.add("Server.Items.WitherScroll", 16);
            sb.sell.add("Server.Items.VampiricEmbraceScroll", 20);
            sb.sell.add("Server.Items.VengefulSpiritScroll", 20);
        }
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Spellbook",
            18,
            10,
            0xEFA,
            0,
        ));
        if w.core_aos() {
            sb.buy.push(BuyInfo::new(
                None,
                "Server.Items.NecromancerSpellbook",
                115,
                10,
                0x2253,
                0,
            ));
        }
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.ScribesPen",
            8,
            10,
            0xFBF,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.BlankScroll",
            5,
            20,
            0x0E34,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            Some("1041072"),
            "Server.Items.MagicWizardsHat",
            11,
            10,
            0x1718,
            utility::random_dyed_hue(&mut w.rng),
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.RecallRune",
            15,
            10,
            0x1F14,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.RefreshPotion",
            15,
            10,
            0xF0B,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.AgilityPotion",
            15,
            10,
            0xF08,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.NightSightPotion",
            15,
            10,
            0xF06,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.LesserHealPotion",
            15,
            10,
            0xF0C,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.StrengthPotion",
            15,
            10,
            0xF09,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.LesserPoisonPotion",
            15,
            10,
            0xF0A,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.LesserCurePotion",
            15,
            10,
            0xF07,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.LesserExplosionPotion",
            21,
            10,
            0xF0D,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.BlackPearl",
            5,
            20,
            0xF7A,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Bloodmoss",
            5,
            20,
            0xF7B,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Garlic", 3, 20, 0xF84, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Ginseng", 3, 20, 0xF85, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.MandrakeRoot",
            3,
            20,
            0xF86,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Nightshade",
            3,
            20,
            0xF88,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.SpidersSilk",
            3,
            20,
            0xF8D,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.SulfurousAsh",
            3,
            20,
            0xF8C,
            0,
        ));
        if w.core_aos() {
            sb.buy
                .push(BuyInfo::new(None, "Server.Items.BatWing", 3, 999, 0xF78, 0));
            sb.buy.push(BuyInfo::new(
                None,
                "Server.Items.DaemonBlood",
                6,
                999,
                0xF7D,
                0,
            ));
            sb.buy
                .push(BuyInfo::new(None, "Server.Items.PigIron", 5, 999, 0xF8A, 0));
            sb.buy.push(BuyInfo::new(
                None,
                "Server.Items.NoxCrystal",
                6,
                999,
                0xF8E,
                0,
            ));
            sb.buy.push(BuyInfo::new(
                None,
                "Server.Items.GraveDust",
                3,
                999,
                0xF8F,
                0,
            ));
        }
        scroll_buy(w, sb, 24);
    },
};

/// `SBMapmaker`.
pub static SBMAPMAKER: SbInfo = SbInfo {
    name: "SBMapmaker",
    build: |w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.BlankScroll", 3);
        sb.sell.add("Server.Items.MapmakersPen", 4);
        sb.sell.add("Server.Items.BlankMap", 2);
        sb.sell.add("Server.Items.CityMap", 3);
        sb.sell.add("Server.Items.LocalMap", 3);
        sb.sell.add("Server.Items.WorldMap", 3);
        sb.sell.add("Server.Items.PresetMapEntry", 3);
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.BlankMap",
            5,
            40,
            0x14EC,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.MapmakersPen",
            8,
            20,
            0x0FBF,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.BlankScroll",
            12,
            40,
            0xEF3,
            0,
        ));
        preset_maps(w, sb);
    },
};

/// `SBMiller`.
pub static SBMILLER: SbInfo = SbInfo {
    name: "SBMiller",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.SackFlour", 1);
        sb.sell.add("Server.Items.SheafOfHay", 1);
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.SackFlour",
            3,
            20,
            0x1039,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.SheafOfHay",
            2,
            20,
            0xF36,
            0,
        ));
    },
};

/// `SBMiner`.
pub static SBMINER: SbInfo = SbInfo {
    name: "SBMiner",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.Pickaxe", 12);
        sb.sell.add("Server.Items.Shovel", 6);
        sb.sell.add("Server.Items.Lantern", 1);
        sb.sell.add("Server.Items.Torch", 3);
        sb.sell.add("Server.Items.Bag", 3);
        sb.sell.add("Server.Items.Candle", 3);
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Bag", 6, 20, 0xE76, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Candle", 6, 10, 0xA28, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Torch", 8, 10, 0xF6B, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Lantern", 2, 10, 0xA25, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Pickaxe", 25, 10, 0xE86, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Shovel", 12, 10, 0xF39, 0));
    },
};

/// `SBMonk`.
pub static SBMONK: SbInfo = SbInfo {
    name: "SBMonk",
    build: |w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        if w.core_aos() {
            sb.buy.push(BuyInfo::new(
                None,
                "Server.Items.MonkRobe",
                136,
                20,
                0x2687,
                0x21E,
            ));
        }
    },
};

/// `SBNinja`.
pub static SBNINJA: SbInfo = SbInfo {
    name: "SBNinja",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.BookOfNinjitsu",
            335,
            20,
            0x23A0,
            0,
        ));
    },
};

/// `SBPlayerBarkeeper`.
pub static SBPLAYER_BARKEEPER: SbInfo = SbInfo {
    name: "SBPlayerBarkeeper",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.buy.push(BuyInfo::beverage(
            None,
            "Server.Items.BeverageBottle",
            0,
            7,
            20,
            0x99F,
            0,
        ));
        sb.buy.push(BuyInfo::beverage(
            None,
            "Server.Items.BeverageBottle",
            4,
            7,
            20,
            0x9C7,
            0,
        ));
        sb.buy.push(BuyInfo::beverage(
            None,
            "Server.Items.BeverageBottle",
            2,
            7,
            20,
            0x99B,
            0,
        ));
        sb.buy.push(BuyInfo::beverage(
            None,
            "Server.Items.Jug",
            1,
            13,
            20,
            0x9C8,
            0,
        ));
        sb.buy.push(BuyInfo::beverage(
            None,
            "Server.Items.Pitcher",
            3,
            7,
            20,
            0x9F0,
            0,
        ));
        sb.buy.push(BuyInfo::beverage(
            None,
            "Server.Items.Pitcher",
            0,
            11,
            20,
            0x1F95,
            0,
        ));
        sb.buy.push(BuyInfo::beverage(
            None,
            "Server.Items.Pitcher",
            1,
            11,
            20,
            0x1F97,
            0,
        ));
        sb.buy.push(BuyInfo::beverage(
            None,
            "Server.Items.Pitcher",
            2,
            11,
            20,
            0x1F99,
            0,
        ));
        sb.buy.push(BuyInfo::beverage(
            None,
            "Server.Items.Pitcher",
            4,
            11,
            20,
            0x1F9B,
            0,
        ));
        sb.buy.push(BuyInfo::beverage(
            None,
            "Server.Items.Pitcher",
            5,
            11,
            20,
            0x1F9D,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            Some("1016450"),
            "Server.Items.Chessboard",
            2,
            20,
            0xFA6,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            Some("1016449"),
            "Server.Items.CheckerBoard",
            2,
            20,
            0xFA6,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Backgammon",
            2,
            20,
            0xE1C,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Dices", 2, 20, 0xFA7, 0));
    },
};

/// `SBProvisioner`.
pub static SBPROVISIONER: SbInfo = SbInfo {
    name: "SBProvisioner",
    build: |w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.Arrow", 1);
        sb.sell.add("Server.Items.Bolt", 2);
        sb.sell.add("Server.Items.Backpack", 7);
        sb.sell.add("Server.Items.Pouch", 3);
        sb.sell.add("Server.Items.Bag", 3);
        sb.sell.add("Server.Items.Candle", 3);
        sb.sell.add("Server.Items.Torch", 4);
        sb.sell.add("Server.Items.Lantern", 1);
        sb.sell.add("Server.Items.OilFlask", 7);
        sb.sell.add("Server.Items.Lockpick", 6);
        sb.sell.add("Server.Items.FloppyHat", 3);
        sb.sell.add("Server.Items.WideBrimHat", 4);
        sb.sell.add("Server.Items.Cap", 5);
        sb.sell.add("Server.Items.TallStrawHat", 4);
        sb.sell.add("Server.Items.StrawHat", 3);
        sb.sell.add("Server.Items.WizardsHat", 5);
        sb.sell.add("Server.Items.LeatherCap", 5);
        sb.sell.add("Server.Items.FeatheredHat", 5);
        sb.sell.add("Server.Items.TricorneHat", 4);
        sb.sell.add("Server.Items.Bandana", 3);
        sb.sell.add("Server.Items.SkullCap", 3);
        sb.sell.add("Server.Items.Bottle", 3);
        sb.sell.add("Server.Items.RedBook", 7);
        sb.sell.add("Server.Items.BlueBook", 7);
        sb.sell.add("Server.Items.TanBook", 7);
        sb.sell.add("Server.Items.WoodenBox", 7);
        sb.sell.add("Server.Items.Kindling", 1);
        sb.sell.add("Server.Items.HairDye", 30);
        sb.sell.add("Server.Items.Chessboard", 1);
        sb.sell.add("Server.Items.CheckerBoard", 1);
        sb.sell.add("Server.Items.Backgammon", 1);
        sb.sell.add("Server.Items.Dices", 1);
        sb.sell.add("Server.Items.Beeswax", 1);
        sb.sell.add("Server.Items.Amber", 25);
        sb.sell.add("Server.Items.Amethyst", 50);
        sb.sell.add("Server.Items.Citrine", 25);
        sb.sell.add("Server.Items.Diamond", 100);
        sb.sell.add("Server.Items.Emerald", 50);
        sb.sell.add("Server.Items.Ruby", 37);
        sb.sell.add("Server.Items.Sapphire", 50);
        sb.sell.add("Server.Items.StarSapphire", 62);
        sb.sell.add("Server.Items.Tourmaline", 47);
        sb.sell.add("Server.Items.GoldRing", 13);
        sb.sell.add("Server.Items.SilverRing", 10);
        sb.sell.add("Server.Items.Necklace", 13);
        sb.sell.add("Server.Items.GoldNecklace", 13);
        sb.sell.add("Server.Items.GoldBeadNecklace", 13);
        sb.sell.add("Server.Items.SilverNecklace", 10);
        sb.sell.add("Server.Items.SilverBeadNecklace", 10);
        sb.sell.add("Server.Items.Beads", 13);
        sb.sell.add("Server.Items.GoldBracelet", 13);
        sb.sell.add("Server.Items.SilverBracelet", 10);
        sb.sell.add("Server.Items.GoldEarrings", 13);
        sb.sell.add("Server.Items.SilverEarrings", 10);
        if !muo_content::misc::guild::new_guild_system(w) {
            sb.sell.add("Server.Items.GuildDeed", 6225);
        }
        if w.core_ml() {
            sb.buy.push(BuyInfo::new(
                Some("1079931"),
                "Server.Items.SalvageBag",
                1255,
                20,
                0xE76,
                utility::random_blue_hue(&mut w.rng),
            ));
        }
        sb.buy.push(BuyInfo::new(
            Some("1060834"),
            "Server.Engines.Plants.PlantBowl",
            2,
            20,
            0x15FD,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Arrow", 2, 20, 0xF3F, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Bolt", 5, 20, 0x1BFB, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Backpack",
            15,
            20,
            0x9B2,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Pouch", 6, 20, 0xE79, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Bag", 6, 20, 0xE76, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Candle", 6, 20, 0xA28, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Torch", 8, 20, 0xF6B, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Lantern", 2, 20, 0xA25, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.OilFlask",
            9,
            20,
            0x1C18,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Lockpick",
            12,
            20,
            0x14FC,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.FloppyHat",
            7,
            20,
            0x1713,
            utility::random_dyed_hue(&mut w.rng),
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.WideBrimHat",
            8,
            20,
            0x1714,
            utility::random_dyed_hue(&mut w.rng),
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Cap",
            10,
            20,
            0x1715,
            utility::random_dyed_hue(&mut w.rng),
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.TallStrawHat",
            8,
            20,
            0x1716,
            utility::random_dyed_hue(&mut w.rng),
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.StrawHat",
            7,
            20,
            0x1717,
            utility::random_dyed_hue(&mut w.rng),
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.WizardsHat",
            11,
            20,
            0x1718,
            utility::random_dyed_hue(&mut w.rng),
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.LeatherCap",
            10,
            20,
            0x1DB9,
            utility::random_dyed_hue(&mut w.rng),
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.FeatheredHat",
            10,
            20,
            0x171A,
            utility::random_dyed_hue(&mut w.rng),
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.TricorneHat",
            8,
            20,
            0x171B,
            utility::random_dyed_hue(&mut w.rng),
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Bandana",
            6,
            20,
            0x1540,
            utility::random_dyed_hue(&mut w.rng),
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.SkullCap",
            7,
            20,
            0x1544,
            utility::random_dyed_hue(&mut w.rng),
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.BreadLoaf",
            6,
            10,
            0x103B,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.LambLeg", 8, 20, 0x160A, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.ChickenLeg",
            5,
            20,
            0x1608,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.CookedBird",
            17,
            20,
            0x9B7,
            0,
        ));
        sb.buy.push(BuyInfo::beverage(
            None,
            "Server.Items.BeverageBottle",
            0,
            7,
            20,
            0x99F,
            0,
        ));
        sb.buy.push(BuyInfo::beverage(
            None,
            "Server.Items.BeverageBottle",
            4,
            7,
            20,
            0x9C7,
            0,
        ));
        sb.buy.push(BuyInfo::beverage(
            None,
            "Server.Items.BeverageBottle",
            2,
            7,
            20,
            0x99B,
            0,
        ));
        sb.buy.push(BuyInfo::beverage(
            None,
            "Server.Items.Jug",
            1,
            13,
            20,
            0x9C8,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Pear", 3, 20, 0x994, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Apple", 3, 20, 0x9D0, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Beeswax", 1, 20, 0x1422, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Garlic", 3, 20, 0xF84, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Ginseng", 3, 20, 0xF85, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Bottle", 5, 20, 0xF0E, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.RedBook", 15, 20, 0xFF1, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.BlueBook",
            15,
            20,
            0xFF2,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.TanBook", 15, 20, 0xFF0, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.WoodenBox",
            14,
            20,
            0xE7D,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Key", 2, 20, 0x100E, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Bedroll", 5, 20, 0xA59, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Kindling", 2, 20, 0xDE1, 0));
        sb.buy.push(BuyInfo::new(
            Some("1041205"),
            "Server.Multis.SmallBoatDeed",
            10177,
            20,
            0x14F2,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            Some("1041060"),
            "Server.Items.HairDye",
            60,
            20,
            0xEFF,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            Some("1016450"),
            "Server.Items.Chessboard",
            2,
            20,
            0xFA6,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            Some("1016449"),
            "Server.Items.CheckerBoard",
            2,
            20,
            0xFA6,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Backgammon",
            2,
            20,
            0xE1C,
            0,
        ));
        if w.core_aos() {
            sb.buy.push(BuyInfo::new(
                None,
                "Server.Engines.Mahjong.MahjongGame",
                6,
                20,
                0xFAA,
                0,
            ));
        }
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Dices", 2, 20, 0xFA7, 0));
        if w.core_aos() {
            sb.buy.push(BuyInfo::new(
                None,
                "Server.Items.SmallBagBall",
                3,
                20,
                0x2256,
                0,
            ));
            sb.buy.push(BuyInfo::new(
                None,
                "Server.Items.LargeBagBall",
                3,
                20,
                0x2257,
                0,
            ));
        }
        if !muo_content::misc::guild::new_guild_system(w) {
            sb.buy.push(BuyInfo::new(
                Some("1041055"),
                "Server.Items.GuildDeed",
                12450,
                20,
                0x14F0,
                0,
            ));
        }
    },
};

/// `SBRancher`.
pub static SBRANCHER: SbInfo = SbInfo {
    name: "SBRancher",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.buy.push(BuyInfo::animal(
            1,
            Some("a pack horse"),
            "Server.Mobiles.PackHorse",
            631,
            10,
            291,
            0,
        ));
    },
};

/// `SBRanger`.
pub static SBRANGER: SbInfo = SbInfo {
    name: "SBRanger",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.buy.push(BuyInfo::animal(
            1,
            Some("a cat"),
            "Server.Mobiles.Cat",
            138,
            20,
            201,
            0,
        ));
        sb.buy.push(BuyInfo::animal(
            1,
            Some("a dog"),
            "Server.Mobiles.Dog",
            181,
            20,
            217,
            0,
        ));
        sb.buy.push(BuyInfo::animal(
            1,
            Some("a pack llama"),
            "Server.Mobiles.PackLlama",
            491,
            20,
            292,
            0,
        ));
        sb.buy.push(BuyInfo::animal(
            1,
            Some("a pack horse"),
            "Server.Mobiles.PackHorse",
            606,
            20,
            291,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Bandage", 5, 20, 0xE21, 0));
    },
};

/// `SBRealEstateBroker`.
pub static SBREAL_ESTATE_BROKER: SbInfo = SbInfo {
    name: "SBRealEstateBroker",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.ScribesPen", 4);
        sb.sell.add("Server.Items.BlankScroll", 2);
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.BlankScroll",
            5,
            20,
            0x0E34,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.ScribesPen",
            8,
            20,
            0xFBF,
            0,
        ));
    },
};

/// `SBSECook`.
pub static SBSECOOK: SbInfo = SbInfo {
    name: "SBSECook",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.Wasabi", 1);
        sb.sell.add("Server.Items.BentoBox", 3);
        sb.sell.add("Server.Items.GreenTea", 1);
        sb.sell.add("Server.Items.SushiRolls", 1);
        sb.sell.add("Server.Items.SushiPlatter", 2);
        sb.sell.add("Server.Items.MisoSoup", 1);
        sb.sell.add("Server.Items.RedMisoSoup", 1);
        sb.sell.add("Server.Items.WhiteMisoSoup", 1);
        sb.sell.add("Server.Items.AwaseMisoSoup", 1);
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Wasabi", 2, 20, 0x24E8, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Wasabi", 2, 20, 0x24E9, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.SushiRolls",
            3,
            20,
            0x283E,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.SushiPlatter",
            3,
            20,
            0x2840,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.GreenTea",
            3,
            20,
            0x284C,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.MisoSoup",
            3,
            20,
            0x284D,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.WhiteMisoSoup",
            3,
            20,
            0x284E,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.RedMisoSoup",
            3,
            20,
            0x284F,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.AwaseMisoSoup",
            3,
            20,
            0x2850,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.BentoBox",
            6,
            20,
            0x2836,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.BentoBox",
            6,
            20,
            0x2837,
            0,
        ));
    },
};

/// `SBSEHats`.
pub static SBSEHATS: SbInfo = SbInfo {
    name: "SBSEHats",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.Kasa", 15);
        sb.sell.add("Server.Items.LeatherJingasa", 5);
        sb.sell.add("Server.Items.ClothNinjaHood", 16);
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Kasa", 31, 20, 0x2798, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.LeatherJingasa",
            11,
            20,
            0x2776,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.ClothNinjaHood",
            33,
            20,
            0x278F,
            0,
        ));
    },
};

/// `SBSamurai`.
pub static SBSAMURAI: SbInfo = SbInfo {
    name: "SBSamurai",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.BookOfBushido",
            280,
            20,
            0x238C,
            0,
        ));
    },
};

/// `SBScribe`.
pub static SBSCRIBE: SbInfo = SbInfo {
    name: "SBScribe",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.ScribesPen", 4);
        sb.sell.add("Server.Items.BrownBook", 7);
        sb.sell.add("Server.Items.TanBook", 7);
        sb.sell.add("Server.Items.BlueBook", 7);
        sb.sell.add("Server.Items.BlankScroll", 3);
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.ScribesPen",
            8,
            20,
            0xFBF,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.BlankScroll",
            5,
            999,
            0x0E34,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.ScribesPen",
            8,
            20,
            0xFC0,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.BrownBook",
            15,
            10,
            0xFEF,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.TanBook", 15, 10, 0xFF0, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.BlueBook",
            15,
            10,
            0xFF2,
            0,
        ));
    },
};

/// `SBShipwright`.
pub static SBSHIPWRIGHT: SbInfo = SbInfo {
    name: "SBShipwright",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.buy.push(BuyInfo::new(
            Some("1041205"),
            "Server.Multis.SmallBoatDeed",
            10177,
            20,
            0x14F2,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            Some("1041206"),
            "Server.Multis.SmallDragonBoatDeed",
            10177,
            20,
            0x14F2,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            Some("1041207"),
            "Server.Multis.MediumBoatDeed",
            11552,
            20,
            0x14F2,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            Some("1041208"),
            "Server.Multis.MediumDragonBoatDeed",
            11552,
            20,
            0x14F2,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            Some("1041209"),
            "Server.Multis.LargeBoatDeed",
            12927,
            20,
            0x14F2,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            Some("1041210"),
            "Server.Multis.LargeDragonBoatDeed",
            12927,
            20,
            0x14F2,
            0,
        ));
    },
};

/// `SBSmithTools`.
pub static SBSMITH_TOOLS: SbInfo = SbInfo {
    name: "SBSmithTools",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.Tongs", 7);
        sb.sell.add("Server.Items.IronIngot", 4);
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.IronIngot",
            5,
            16,
            0x1BF2,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Tongs", 13, 14, 0xFBB, 0));
    },
};

/// `SBStoneCrafter`.
pub static SBSTONE_CRAFTER: SbInfo = SbInfo {
    name: "SBStoneCrafter",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.MasonryBook", 5000);
        sb.sell.add("Server.Items.StoneMiningBook", 5000);
        sb.sell.add("Server.Items.MalletAndChisel", 1);
        sb.sell.add("Server.Items.WoodenBox", 7);
        sb.sell.add("Server.Items.SmallCrate", 5);
        sb.sell.add("Server.Items.MediumCrate", 6);
        sb.sell.add("Server.Items.LargeCrate", 7);
        sb.sell.add("Server.Items.WoodenChest", 15);
        sb.sell.add("Server.Items.LargeTable", 10);
        sb.sell.add("Server.Items.Nightstand", 7);
        sb.sell.add("Server.Items.YewWoodTable", 10);
        sb.sell.add("Server.Items.Throne", 24);
        sb.sell.add("Server.Items.WoodenThrone", 6);
        sb.sell.add("Server.Items.Stool", 6);
        sb.sell.add("Server.Items.FootStool", 6);
        sb.sell.add("Server.Items.FancyWoodenChairCushion", 12);
        sb.sell.add("Server.Items.WoodenChairCushion", 10);
        sb.sell.add("Server.Items.WoodenChair", 8);
        sb.sell.add("Server.Items.BambooChair", 6);
        sb.sell.add("Server.Items.WoodenBench", 6);
        sb.sell.add("Server.Items.Saw", 9);
        sb.sell.add("Server.Items.Scorp", 6);
        sb.sell.add("Server.Items.SmoothingPlane", 6);
        sb.sell.add("Server.Items.DrawKnife", 6);
        sb.sell.add("Server.Items.Froe", 6);
        sb.sell.add("Server.Items.Hammer", 14);
        sb.sell.add("Server.Items.Inshave", 6);
        sb.sell.add("Server.Items.JointingPlane", 6);
        sb.sell.add("Server.Items.MouldingPlane", 6);
        sb.sell.add("Server.Items.DovetailSaw", 7);
        sb.sell.add("Server.Items.Board", 2);
        sb.sell.add("Server.Items.Axle", 1);
        sb.sell.add("Server.Items.WoodenShield", 31);
        sb.sell.add("Server.Items.BlackStaff", 24);
        sb.sell.add("Server.Items.GnarledStaff", 12);
        sb.sell.add("Server.Items.QuarterStaff", 15);
        sb.sell.add("Server.Items.ShepherdsCrook", 12);
        sb.sell.add("Server.Items.Club", 13);
        sb.sell.add("Server.Items.Log", 1);
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Nails", 3, 20, 0x102E, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Axle", 2, 20, 0x105B, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Board", 3, 20, 0x1BD7, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.DrawKnife",
            10,
            20,
            0x10E4,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Froe", 10, 20, 0x10E5, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Scorp", 10, 20, 0x10E7, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Inshave",
            10,
            20,
            0x10E6,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.DovetailSaw",
            12,
            20,
            0x1028,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Saw", 15, 20, 0x1034, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Hammer", 17, 20, 0x102A, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.MouldingPlane",
            11,
            20,
            0x102C,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.SmoothingPlane",
            10,
            20,
            0x1032,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.JointingPlane",
            11,
            20,
            0x1030,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            Some("Making Valuables With Stonecrafting"),
            "Server.Items.MasonryBook",
            10625,
            10,
            0xFBE,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            Some("Mining For Quality Stone"),
            "Server.Items.StoneMiningBook",
            10625,
            10,
            0xFBE,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            Some("1044515"),
            "Server.Items.MalletAndChisel",
            3,
            50,
            0x12B3,
            0,
        ));
    },
};

/// `SBTailor`.
pub static SBTAILOR: SbInfo = SbInfo {
    name: "SBTailor",
    build: |w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.Scissors", 6);
        sb.sell.add("Server.Items.SewingKit", 1);
        sb.sell.add("Server.Items.Dyes", 4);
        sb.sell.add("Server.Items.DyeTub", 4);
        sb.sell.add("Server.Items.BoltOfCloth", 50);
        sb.sell.add("Server.Items.FancyShirt", 10);
        sb.sell.add("Server.Items.Shirt", 6);
        sb.sell.add("Server.Items.ShortPants", 3);
        sb.sell.add("Server.Items.LongPants", 5);
        sb.sell.add("Server.Items.Cloak", 4);
        sb.sell.add("Server.Items.FancyDress", 12);
        sb.sell.add("Server.Items.Robe", 9);
        sb.sell.add("Server.Items.PlainDress", 7);
        sb.sell.add("Server.Items.Skirt", 5);
        sb.sell.add("Server.Items.Kilt", 5);
        sb.sell.add("Server.Items.Doublet", 7);
        sb.sell.add("Server.Items.Tunic", 9);
        sb.sell.add("Server.Items.JesterSuit", 13);
        sb.sell.add("Server.Items.FullApron", 5);
        sb.sell.add("Server.Items.HalfApron", 5);
        sb.sell.add("Server.Items.JesterHat", 6);
        sb.sell.add("Server.Items.FloppyHat", 3);
        sb.sell.add("Server.Items.WideBrimHat", 4);
        sb.sell.add("Server.Items.Cap", 5);
        sb.sell.add("Server.Items.SkullCap", 3);
        sb.sell.add("Server.Items.Bandana", 3);
        sb.sell.add("Server.Items.TallStrawHat", 4);
        sb.sell.add("Server.Items.StrawHat", 4);
        sb.sell.add("Server.Items.WizardsHat", 5);
        sb.sell.add("Server.Items.Bonnet", 4);
        sb.sell.add("Server.Items.FeatheredHat", 5);
        sb.sell.add("Server.Items.TricorneHat", 4);
        sb.sell.add("Server.Items.SpoolOfThread", 9);
        sb.sell.add("Server.Items.Flax", 51);
        sb.sell.add("Server.Items.Cotton", 51);
        sb.sell.add("Server.Items.Wool", 31);
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.SewingKit",
            3,
            20,
            0xF9D,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Scissors",
            11,
            20,
            0xF9F,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.DyeTub", 8, 20, 0xFAB, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Dyes", 8, 20, 0xFA9, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Shirt", 12, 20, 0x1517, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.ShortPants",
            7,
            20,
            0x152E,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.FancyShirt",
            21,
            20,
            0x1EFD,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.LongPants",
            10,
            20,
            0x1539,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.FancyDress",
            26,
            20,
            0x1EFF,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.PlainDress",
            13,
            20,
            0x1F01,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Kilt", 11, 20, 0x1537, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Kilt",
            11,
            20,
            0x1537,
            utility::random_dyed_hue(&mut w.rng),
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.HalfApron",
            10,
            20,
            0x153b,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Robe", 18, 20, 0x1F03, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Cloak", 8, 20, 0x1515, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Cloak", 8, 20, 0x1515, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Doublet",
            13,
            20,
            0x1F7B,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Tunic", 18, 20, 0x1FA1, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.JesterSuit",
            26,
            20,
            0x1F9F,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.JesterHat",
            12,
            20,
            0x171C,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.FloppyHat",
            7,
            20,
            0x1713,
            utility::random_dyed_hue(&mut w.rng),
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.WideBrimHat",
            8,
            20,
            0x1714,
            utility::random_dyed_hue(&mut w.rng),
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Cap",
            10,
            20,
            0x1715,
            utility::random_dyed_hue(&mut w.rng),
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.TallStrawHat",
            8,
            20,
            0x1716,
            utility::random_dyed_hue(&mut w.rng),
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.StrawHat",
            7,
            20,
            0x1717,
            utility::random_dyed_hue(&mut w.rng),
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.WizardsHat",
            11,
            20,
            0x1718,
            utility::random_dyed_hue(&mut w.rng),
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.LeatherCap",
            10,
            20,
            0x1DB9,
            utility::random_dyed_hue(&mut w.rng),
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.FeatheredHat",
            10,
            20,
            0x171A,
            utility::random_dyed_hue(&mut w.rng),
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.TricorneHat",
            8,
            20,
            0x171B,
            utility::random_dyed_hue(&mut w.rng),
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Bandana",
            6,
            20,
            0x1540,
            utility::random_dyed_hue(&mut w.rng),
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.SkullCap",
            7,
            20,
            0x1544,
            utility::random_dyed_hue(&mut w.rng),
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.BoltOfCloth",
            100,
            20,
            0xf95,
            utility::random_dyed_hue(&mut w.rng),
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Cloth",
            2,
            20,
            0x1766,
            utility::random_dyed_hue(&mut w.rng),
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.UncutCloth",
            2,
            20,
            0x1767,
            utility::random_dyed_hue(&mut w.rng),
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Cotton", 102, 20, 0xDF9, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Wool", 62, 20, 0xDF8, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Flax", 102, 20, 0x1A9C, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.SpoolOfThread",
            18,
            20,
            0xFA0,
            0,
        ));
    },
};

/// `SBTanner`.
pub static SBTANNER: SbInfo = SbInfo {
    name: "SBTanner",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.Bag", 3);
        sb.sell.add("Server.Items.Pouch", 3);
        sb.sell.add("Server.Items.Backpack", 7);
        sb.sell.add("Server.Items.Leather", 5);
        sb.sell.add("Server.Items.SkinningKnife", 7);
        sb.sell.add("Server.Items.LeatherArms", 40);
        sb.sell.add("Server.Items.LeatherChest", 52);
        sb.sell.add("Server.Items.LeatherGloves", 30);
        sb.sell.add("Server.Items.LeatherGorget", 37);
        sb.sell.add("Server.Items.LeatherLegs", 40);
        sb.sell.add("Server.Items.LeatherCap", 5);
        sb.sell.add("Server.Items.StuddedArms", 43);
        sb.sell.add("Server.Items.StuddedChest", 37);
        sb.sell.add("Server.Items.StuddedGloves", 39);
        sb.sell.add("Server.Items.StuddedGorget", 22);
        sb.sell.add("Server.Items.StuddedLegs", 33);
        sb.sell.add("Server.Items.FemaleStuddedChest", 31);
        sb.sell.add("Server.Items.StuddedBustierArms", 23);
        sb.sell.add("Server.Items.FemalePlateChest", 103);
        sb.sell.add("Server.Items.FemaleLeatherChest", 18);
        sb.sell.add("Server.Items.LeatherBustierArms", 12);
        sb.sell.add("Server.Items.LeatherShorts", 14);
        sb.sell.add("Server.Items.LeatherSkirt", 12);
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.LeatherArms",
            80,
            20,
            0x13CD,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.LeatherChest",
            101,
            20,
            0x13CC,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.LeatherGloves",
            60,
            20,
            0x13C6,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.LeatherGorget",
            74,
            20,
            0x13C7,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.LeatherLegs",
            80,
            20,
            0x13cb,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.LeatherCap",
            10,
            20,
            0x1DB9,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.StuddedGorget",
            50,
            20,
            0x13D6,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.StuddedArms",
            57,
            20,
            0x13DC,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.StuddedChest",
            75,
            20,
            0x13DB,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.StuddedLegs",
            67,
            20,
            0x13DA,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.StuddedGloves",
            45,
            20,
            0x13D5,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.FemaleStuddedChest",
            62,
            20,
            0x1C02,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.FemalePlateChest",
            207,
            20,
            0x1C04,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.FemaleLeatherChest",
            116,
            20,
            0x1C06,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.LeatherShorts",
            86,
            20,
            0x1C00,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.LeatherSkirt",
            87,
            20,
            0x1C08,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.LeatherBustierArms",
            97,
            20,
            0x1C0A,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.LeatherBustierArms",
            97,
            20,
            0x1C0B,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.StuddedBustierArms",
            97,
            20,
            0x1C0C,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.StuddedBustierArms",
            97,
            20,
            0x1C0D,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Bag", 6, 20, 0xE76, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Pouch", 6, 20, 0xE79, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Backpack",
            15,
            20,
            0x9B2,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Leather", 6, 20, 0x1081, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.SkinningKnife",
            15,
            20,
            0xEC4,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            Some("1041279"),
            "Server.Items.TaxidermyKit",
            100000,
            20,
            0x1EBA,
            0,
        ));
    },
};

/// `SBTavernKeeper`.
pub static SBTAVERN_KEEPER: SbInfo = SbInfo {
    name: "SBTavernKeeper",
    build: |w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.WoodenBowlOfCarrots", 1);
        sb.sell.add("Server.Items.WoodenBowlOfCorn", 1);
        sb.sell.add("Server.Items.WoodenBowlOfLettuce", 1);
        sb.sell.add("Server.Items.WoodenBowlOfPeas", 1);
        sb.sell.add("Server.Items.EmptyPewterBowl", 1);
        sb.sell.add("Server.Items.PewterBowlOfCorn", 1);
        sb.sell.add("Server.Items.PewterBowlOfLettuce", 1);
        sb.sell.add("Server.Items.PewterBowlOfPeas", 1);
        sb.sell.add("Server.Items.PewterBowlOfPotatos", 1);
        sb.sell.add("Server.Items.WoodenBowlOfStew", 1);
        sb.sell.add("Server.Items.WoodenBowlOfTomatoSoup", 1);
        sb.sell.add("Server.Items.BeverageBottle", 3);
        sb.sell.add("Server.Items.Jug", 6);
        sb.sell.add("Server.Items.Pitcher", 5);
        sb.sell.add("Server.Items.GlassMug", 1);
        sb.sell.add("Server.Items.BreadLoaf", 3);
        sb.sell.add("Server.Items.CheeseWheel", 12);
        sb.sell.add("Server.Items.Ribs", 6);
        sb.sell.add("Server.Items.Peach", 1);
        sb.sell.add("Server.Items.Pear", 1);
        sb.sell.add("Server.Items.Grapes", 1);
        sb.sell.add("Server.Items.Apple", 1);
        sb.sell.add("Server.Items.Banana", 1);
        sb.sell.add("Server.Items.Candle", 3);
        sb.sell.add("Server.Items.Chessboard", 1);
        sb.sell.add("Server.Items.CheckerBoard", 1);
        sb.sell.add("Server.Items.Backgammon", 1);
        sb.sell.add("Server.Items.Dices", 1);
        sb.sell.add("Server.Items.ContractOfEmployment", 626);
        sb.buy.push(BuyInfo::beverage(
            None,
            "Server.Items.BeverageBottle",
            0,
            7,
            20,
            0x99F,
            0,
        ));
        sb.buy.push(BuyInfo::beverage(
            None,
            "Server.Items.BeverageBottle",
            4,
            7,
            20,
            0x9C7,
            0,
        ));
        sb.buy.push(BuyInfo::beverage(
            None,
            "Server.Items.BeverageBottle",
            2,
            7,
            20,
            0x99B,
            0,
        ));
        sb.buy.push(BuyInfo::beverage(
            None,
            "Server.Items.Jug",
            1,
            13,
            20,
            0x9C8,
            0,
        ));
        sb.buy.push(BuyInfo::beverage(
            None,
            "Server.Items.Pitcher",
            3,
            7,
            20,
            0x9F0,
            0,
        ));
        sb.buy.push(BuyInfo::beverage(
            None,
            "Server.Items.Pitcher",
            0,
            11,
            20,
            0x1F95,
            0,
        ));
        sb.buy.push(BuyInfo::beverage(
            None,
            "Server.Items.Pitcher",
            1,
            11,
            20,
            0x1F97,
            0,
        ));
        sb.buy.push(BuyInfo::beverage(
            None,
            "Server.Items.Pitcher",
            2,
            11,
            20,
            0x1F99,
            0,
        ));
        sb.buy.push(BuyInfo::beverage(
            None,
            "Server.Items.Pitcher",
            4,
            11,
            20,
            0x1F9B,
            0,
        ));
        sb.buy.push(BuyInfo::beverage(
            None,
            "Server.Items.Pitcher",
            5,
            11,
            20,
            0x1F9D,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.BreadLoaf",
            6,
            10,
            0x103B,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.CheeseWheel",
            21,
            10,
            0x97E,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.CookedBird",
            17,
            20,
            0x9B7,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.LambLeg", 8, 20, 0x160A, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.ChickenLeg",
            5,
            20,
            0x1608,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Ribs", 7, 20, 0x9F2, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.WoodenBowlOfCarrots",
            3,
            20,
            0x15F9,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.WoodenBowlOfCorn",
            3,
            20,
            0x15FA,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.WoodenBowlOfLettuce",
            3,
            20,
            0x15FB,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.WoodenBowlOfPeas",
            3,
            20,
            0x15FC,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.EmptyPewterBowl",
            2,
            20,
            0x15FD,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.PewterBowlOfCorn",
            3,
            20,
            0x15FE,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.PewterBowlOfLettuce",
            3,
            20,
            0x15FF,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.PewterBowlOfPeas",
            3,
            20,
            0x1600,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.PewterBowlOfPotatos",
            3,
            20,
            0x1601,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.WoodenBowlOfStew",
            3,
            20,
            0x1604,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.WoodenBowlOfTomatoSoup",
            3,
            20,
            0x1606,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.ApplePie",
            7,
            20,
            0x1041,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            Some("1016450"),
            "Server.Items.Chessboard",
            2,
            20,
            0xFA6,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            Some("1016449"),
            "Server.Items.CheckerBoard",
            2,
            20,
            0xFA6,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Backgammon",
            2,
            20,
            0xE1C,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Dices", 2, 20, 0xFA7, 0));
        sb.buy.push(BuyInfo::new(
            Some("1041243"),
            "Server.Items.ContractOfEmployment",
            1252,
            20,
            0x14F0,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            Some("a barkeep contract"),
            "Server.Items.BarkeepContract",
            1252,
            20,
            0x14F0,
            0,
        ));
        if muo_content::vendor::new_vendor_system(w) {
            sb.buy.push(BuyInfo::new(
                Some("1062332"),
                "Server.Items.VendorRentalContract",
                1252,
                20,
                0x14F0,
                0x672,
            ));
        }
    },
};

/// `SBThief`.
pub static SBTHIEF: SbInfo = SbInfo {
    name: "SBThief",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.Backpack", 7);
        sb.sell.add("Server.Items.Pouch", 3);
        sb.sell.add("Server.Items.Torch", 3);
        sb.sell.add("Server.Items.Lantern", 1);
        sb.sell.add("Server.Items.Lockpick", 6);
        sb.sell.add("Server.Items.WoodenBox", 7);
        sb.sell.add("Server.Items.HairDye", 19);
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Backpack",
            15,
            20,
            0x9B2,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Pouch", 6, 20, 0xE79, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Torch", 8, 20, 0xF6B, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Lantern", 2, 20, 0xA25, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Lockpick",
            12,
            20,
            0x14FC,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.WoodenBox",
            14,
            20,
            0x9AA,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Key", 2, 20, 0x100E, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.HairDye", 37, 20, 0xEFF, 0));
    },
};

/// `SBTinker`.
pub static SBTINKER: SbInfo = SbInfo {
    name: "SBTinker",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.Drums", 10);
        sb.sell.add("Server.Items.Tambourine", 10);
        sb.sell.add("Server.Items.LapHarp", 10);
        sb.sell.add("Server.Items.Lute", 10);
        sb.sell.add("Server.Items.Shovel", 6);
        sb.sell.add("Server.Items.SewingKit", 1);
        sb.sell.add("Server.Items.Scissors", 6);
        sb.sell.add("Server.Items.Tongs", 7);
        sb.sell.add("Server.Items.Key", 1);
        sb.sell.add("Server.Items.DovetailSaw", 6);
        sb.sell.add("Server.Items.MouldingPlane", 6);
        sb.sell.add("Server.Items.Nails", 1);
        sb.sell.add("Server.Items.JointingPlane", 6);
        sb.sell.add("Server.Items.SmoothingPlane", 6);
        sb.sell.add("Server.Items.Saw", 7);
        sb.sell.add("Server.Items.Clock", 11);
        sb.sell.add("Server.Items.ClockParts", 1);
        sb.sell.add("Server.Items.AxleGears", 1);
        sb.sell.add("Server.Items.Gears", 1);
        sb.sell.add("Server.Items.Hinge", 1);
        sb.sell.add("Server.Items.Sextant", 6);
        sb.sell.add("Server.Items.SextantParts", 2);
        sb.sell.add("Server.Items.Axle", 1);
        sb.sell.add("Server.Items.Springs", 1);
        sb.sell.add("Server.Items.DrawKnife", 5);
        sb.sell.add("Server.Items.Froe", 5);
        sb.sell.add("Server.Items.Inshave", 5);
        sb.sell.add("Server.Items.Scorp", 5);
        sb.sell.add("Server.Items.Lockpick", 6);
        sb.sell.add("Server.Items.TinkerTools", 3);
        sb.sell.add("Server.Items.Board", 1);
        sb.sell.add("Server.Items.Log", 1);
        sb.sell.add("Server.Items.Pickaxe", 16);
        sb.sell.add("Server.Items.Hammer", 3);
        sb.sell.add("Server.Items.SmithHammer", 11);
        sb.sell.add("Server.Items.ButcherKnife", 6);
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Clock", 22, 20, 0x104B, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Nails", 3, 20, 0x102E, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.ClockParts",
            3,
            20,
            0x104F,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.AxleGears",
            3,
            20,
            0x1051,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Gears", 2, 20, 0x1053, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Hinge", 2, 20, 0x1055, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Sextant",
            13,
            20,
            0x1057,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.SextantParts",
            5,
            20,
            0x1059,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Axle", 2, 20, 0x105B, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Springs", 3, 20, 0x105D, 0));
        sb.buy.push(BuyInfo::new(
            Some("1024111"),
            "Server.Items.Key",
            8,
            20,
            0x100F,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            Some("1024112"),
            "Server.Items.Key",
            8,
            20,
            0x1010,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            Some("1024115"),
            "Server.Items.Key",
            8,
            20,
            0x1013,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.KeyRing", 8, 20, 0x1010, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Lockpick",
            12,
            20,
            0x14FC,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.TinkersTools",
            7,
            20,
            0x1EBC,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Board", 3, 20, 0x1BD7, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.IronIngot",
            5,
            16,
            0x1BF2,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.SewingKit",
            3,
            20,
            0xF9D,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.DrawKnife",
            10,
            20,
            0x10E4,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Froe", 10, 20, 0x10E5, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Scorp", 10, 20, 0x10E7, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Inshave",
            10,
            20,
            0x10E6,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.ButcherKnife",
            13,
            20,
            0x13F6,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Scissors",
            11,
            20,
            0xF9F,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Tongs", 13, 14, 0xFBB, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.DovetailSaw",
            12,
            20,
            0x1028,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Saw", 15, 20, 0x1034, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Hammer", 17, 20, 0x102A, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.SmithHammer",
            23,
            20,
            0x13E3,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Shovel", 12, 20, 0xF39, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.MouldingPlane",
            11,
            20,
            0x102C,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.JointingPlane",
            10,
            20,
            0x1030,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.SmoothingPlane",
            11,
            20,
            0x1032,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Pickaxe", 25, 20, 0xE86, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Drums", 21, 20, 0x0E9C, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Tambourine",
            21,
            20,
            0x0E9E,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.LapHarp",
            21,
            20,
            0x0EB2,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Lute", 21, 20, 0x0EB3, 0));
    },
};

/// `SBVagabond`.
pub static SBVAGABOND: SbInfo = SbInfo {
    name: "SBVagabond",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.Board", 1);
        sb.sell.add("Server.Items.IronIngot", 3);
        sb.sell.add("Server.Items.Amber", 25);
        sb.sell.add("Server.Items.Amethyst", 50);
        sb.sell.add("Server.Items.Citrine", 25);
        sb.sell.add("Server.Items.Diamond", 100);
        sb.sell.add("Server.Items.Emerald", 50);
        sb.sell.add("Server.Items.Ruby", 37);
        sb.sell.add("Server.Items.Sapphire", 50);
        sb.sell.add("Server.Items.StarSapphire", 62);
        sb.sell.add("Server.Items.Tourmaline", 47);
        sb.sell.add("Server.Items.GoldRing", 13);
        sb.sell.add("Server.Items.SilverRing", 10);
        sb.sell.add("Server.Items.Necklace", 13);
        sb.sell.add("Server.Items.GoldNecklace", 13);
        sb.sell.add("Server.Items.GoldBeadNecklace", 13);
        sb.sell.add("Server.Items.SilverNecklace", 10);
        sb.sell.add("Server.Items.SilverBeadNecklace", 10);
        sb.sell.add("Server.Items.Beads", 13);
        sb.sell.add("Server.Items.GoldBracelet", 13);
        sb.sell.add("Server.Items.SilverBracelet", 10);
        sb.sell.add("Server.Items.GoldEarrings", 13);
        sb.sell.add("Server.Items.SilverEarrings", 10);
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.GoldRing",
            27,
            20,
            0x108A,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Necklace",
            26,
            20,
            0x1085,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.GoldNecklace",
            27,
            20,
            0x1088,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.GoldBeadNecklace",
            27,
            20,
            0x1089,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Beads", 27, 20, 0x108B, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.GoldBracelet",
            27,
            20,
            0x1086,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.GoldEarrings",
            27,
            20,
            0x1087,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Board", 3, 20, 0x1BD7, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.IronIngot",
            6,
            20,
            0x1BF2,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.StarSapphire",
            125,
            20,
            0xF21,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Emerald",
            100,
            20,
            0xF10,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Sapphire",
            100,
            20,
            0xF19,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Ruby", 75, 20, 0xF13, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Citrine", 50, 20, 0xF15, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Amethyst",
            100,
            20,
            0xF16,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Tourmaline",
            75,
            20,
            0xF2D,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Amber", 50, 20, 0xF25, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Diamond",
            200,
            20,
            0xF26,
            0,
        ));
    },
};

/// `SBVarietyDealer`.
pub static SBVARIETY_DEALER: SbInfo = SbInfo {
    name: "SBVarietyDealer",
    build: |w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.Bandage", 1);
        sb.sell.add("Server.Items.BlankScroll", 3);
        sb.sell.add("Server.Items.NightSightPotion", 7);
        sb.sell.add("Server.Items.AgilityPotion", 7);
        sb.sell.add("Server.Items.StrengthPotion", 7);
        sb.sell.add("Server.Items.RefreshPotion", 7);
        sb.sell.add("Server.Items.LesserCurePotion", 7);
        sb.sell.add("Server.Items.LesserHealPotion", 7);
        sb.sell.add("Server.Items.LesserPoisonPotion", 7);
        sb.sell.add("Server.Items.LesserExplosionPotion", 10);
        sb.sell.add("Server.Items.Bolt", 3);
        sb.sell.add("Server.Items.Arrow", 2);
        sb.sell.add("Server.Items.BlackPearl", 3);
        sb.sell.add("Server.Items.Bloodmoss", 3);
        sb.sell.add("Server.Items.MandrakeRoot", 2);
        sb.sell.add("Server.Items.Garlic", 2);
        sb.sell.add("Server.Items.Ginseng", 2);
        sb.sell.add("Server.Items.Nightshade", 2);
        sb.sell.add("Server.Items.SpidersSilk", 2);
        sb.sell.add("Server.Items.SulfurousAsh", 2);
        sb.sell.add("Server.Items.BreadLoaf", 3);
        sb.sell.add("Server.Items.Backpack", 7);
        sb.sell.add("Server.Items.RecallRune", 8);
        sb.sell.add("Server.Items.Spellbook", 9);
        sb.sell.add("Server.Items.BlankScroll", 3);
        if w.core_aos() {
            sb.sell.add("Server.Items.BatWing", 2);
            sb.sell.add("Server.Items.GraveDust", 2);
            sb.sell.add("Server.Items.DaemonBlood", 3);
            sb.sell.add("Server.Items.NoxCrystal", 3);
            sb.sell.add("Server.Items.PigIron", 3);
        }
        scroll_sell(sb, 5);
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Bandage", 5, 20, 0xE21, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.BlankScroll",
            5,
            999,
            0x0E34,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.NightSightPotion",
            15,
            10,
            0xF06,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.AgilityPotion",
            15,
            10,
            0xF08,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.StrengthPotion",
            15,
            10,
            0xF09,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.RefreshPotion",
            15,
            10,
            0xF0B,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.LesserCurePotion",
            15,
            10,
            0xF07,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.LesserHealPotion",
            15,
            10,
            0xF0C,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.LesserPoisonPotion",
            15,
            10,
            0xF0A,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.LesserExplosionPotion",
            21,
            10,
            0xF0D,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Bolt",
            6,
            rnd::from(w, 30, 60),
            0x1BFB,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Arrow",
            3,
            rnd::from(w, 30, 60),
            0xF3F,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.BlackPearl",
            5,
            999,
            0xF7A,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Bloodmoss",
            5,
            999,
            0xF7B,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.MandrakeRoot",
            3,
            999,
            0xF86,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Garlic", 3, 999, 0xF84, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Ginseng", 3, 999, 0xF85, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Nightshade",
            3,
            999,
            0xF88,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.SpidersSilk",
            3,
            999,
            0xF8D,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.SulfurousAsh",
            3,
            999,
            0xF8C,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.BreadLoaf",
            7,
            10,
            0x103B,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Backpack",
            15,
            20,
            0x9B2,
            0,
        ));
        scroll_buy(w, sb, 24);
        if w.core_aos() {
            sb.buy
                .push(BuyInfo::new(None, "Server.Items.BatWing", 3, 999, 0xF78, 0));
            sb.buy.push(BuyInfo::new(
                None,
                "Server.Items.GraveDust",
                3,
                999,
                0xF8F,
                0,
            ));
            sb.buy.push(BuyInfo::new(
                None,
                "Server.Items.DaemonBlood",
                6,
                999,
                0xF7D,
                0,
            ));
            sb.buy.push(BuyInfo::new(
                None,
                "Server.Items.NoxCrystal",
                6,
                999,
                0xF8E,
                0,
            ));
            sb.buy
                .push(BuyInfo::new(None, "Server.Items.PigIron", 5, 999, 0xF8A, 0));
            sb.buy.push(BuyInfo::new(
                None,
                "Server.Items.NecromancerSpellbook",
                115,
                10,
                0x2253,
                0,
            ));
        }
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.RecallRune",
            15,
            10,
            0x1f14,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Spellbook",
            18,
            10,
            0xEFA,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            Some("1041072"),
            "Server.Items.MagicWizardsHat",
            11,
            10,
            0x1718,
            0,
        ));
    },
};

/// `SBVeterinarian`.
pub static SBVETERINARIAN: SbInfo = SbInfo {
    name: "SBVeterinarian",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.Bandage", 1);
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Bandage", 6, 20, 0xE21, 0));
        sb.buy.push(BuyInfo::animal(
            1,
            Some("a pack horse"),
            "Server.Mobiles.PackHorse",
            616,
            10,
            291,
            0,
        ));
        sb.buy.push(BuyInfo::animal(
            1,
            Some("a pack llama"),
            "Server.Mobiles.PackLlama",
            523,
            10,
            292,
            0,
        ));
        sb.buy.push(BuyInfo::animal(
            1,
            Some("a dog"),
            "Server.Mobiles.Dog",
            158,
            10,
            217,
            0,
        ));
        sb.buy.push(BuyInfo::animal(
            1,
            Some("a cat"),
            "Server.Mobiles.Cat",
            131,
            10,
            201,
            0,
        ));
    },
};

/// `SBWaiter`.
pub static SBWAITER: SbInfo = SbInfo {
    name: "SBWaiter",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.buy.push(BuyInfo::beverage(
            None,
            "Server.Items.BeverageBottle",
            0,
            7,
            20,
            0x99F,
            0,
        ));
        sb.buy.push(BuyInfo::beverage(
            None,
            "Server.Items.BeverageBottle",
            4,
            7,
            20,
            0x9C7,
            0,
        ));
        sb.buy.push(BuyInfo::beverage(
            None,
            "Server.Items.BeverageBottle",
            2,
            7,
            20,
            0x99B,
            0,
        ));
        sb.buy.push(BuyInfo::beverage(
            None,
            "Server.Items.Jug",
            1,
            13,
            20,
            0x9C8,
            0,
        ));
        sb.buy.push(BuyInfo::beverage(
            None,
            "Server.Items.Pitcher",
            3,
            7,
            20,
            0x9F0,
            0,
        ));
        sb.buy.push(BuyInfo::beverage(
            None,
            "Server.Items.Pitcher",
            0,
            11,
            20,
            0x1F95,
            0,
        ));
        sb.buy.push(BuyInfo::beverage(
            None,
            "Server.Items.Pitcher",
            1,
            11,
            20,
            0x1F97,
            0,
        ));
        sb.buy.push(BuyInfo::beverage(
            None,
            "Server.Items.Pitcher",
            2,
            11,
            20,
            0x1F99,
            0,
        ));
        sb.buy.push(BuyInfo::beverage(
            None,
            "Server.Items.Pitcher",
            4,
            11,
            20,
            0x1F9B,
            0,
        ));
        sb.buy.push(BuyInfo::beverage(
            None,
            "Server.Items.Pitcher",
            5,
            11,
            20,
            0x1F9D,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.BreadLoaf",
            6,
            10,
            0x103B,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.CheeseWheel",
            21,
            10,
            0x97E,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.CookedBird",
            17,
            20,
            0x9B7,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.LambLeg", 8, 20, 0x160A, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.WoodenBowlOfCarrots",
            3,
            20,
            0x15F9,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.WoodenBowlOfCorn",
            3,
            20,
            0x15FA,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.WoodenBowlOfLettuce",
            3,
            20,
            0x15FB,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.WoodenBowlOfPeas",
            3,
            20,
            0x15FC,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.EmptyPewterBowl",
            2,
            20,
            0x15FD,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.PewterBowlOfCorn",
            3,
            20,
            0x15FE,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.PewterBowlOfLettuce",
            3,
            20,
            0x15FF,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.PewterBowlOfPeas",
            3,
            20,
            0x1600,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.PewterBowlOfPotatos",
            3,
            20,
            0x1601,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.WoodenBowlOfStew",
            3,
            20,
            0x1604,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.WoodenBowlOfTomatoSoup",
            3,
            20,
            0x1606,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.ApplePie",
            7,
            20,
            0x1041,
            0,
        ));
    },
};

/// `SBWeaponSmith`.
pub static SBWEAPON_SMITH: SbInfo = SbInfo {
    name: "SBWeaponSmith",
    build: |w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.BattleAxe", 13);
        sb.sell.add("Server.Items.DoubleAxe", 26);
        sb.sell.add("Server.Items.ExecutionersAxe", 15);
        sb.sell.add("Server.Items.LargeBattleAxe", 16);
        sb.sell.add("Server.Items.Pickaxe", 11);
        sb.sell.add("Server.Items.TwoHandedAxe", 16);
        sb.sell.add("Server.Items.WarAxe", 14);
        sb.sell.add("Server.Items.Axe", 20);
        sb.sell.add("Server.Items.Bardiche", 30);
        sb.sell.add("Server.Items.Halberd", 21);
        sb.sell.add("Server.Items.ButcherKnife", 7);
        sb.sell.add("Server.Items.Cleaver", 7);
        sb.sell.add("Server.Items.Dagger", 10);
        sb.sell.add("Server.Items.SkinningKnife", 7);
        sb.sell.add("Server.Items.Club", 8);
        sb.sell.add("Server.Items.HammerPick", 13);
        sb.sell.add("Server.Items.Mace", 14);
        sb.sell.add("Server.Items.Maul", 10);
        sb.sell.add("Server.Items.WarHammer", 12);
        sb.sell.add("Server.Items.WarMace", 15);
        sb.sell.add("Server.Items.HeavyCrossbow", 27);
        sb.sell.add("Server.Items.Bow", 17);
        sb.sell.add("Server.Items.Crossbow", 23);
        if w.core_aos() {
            sb.sell.add("Server.Items.CompositeBow", 25);
            sb.sell.add("Server.Items.RepeatingCrossbow", 28);
            sb.sell.add("Server.Items.Scepter", 20);
            sb.sell.add("Server.Items.BladedStaff", 20);
            sb.sell.add("Server.Items.Scythe", 19);
            sb.sell.add("Server.Items.BoneHarvester", 17);
            sb.sell.add("Server.Items.Scepter", 18);
            sb.sell.add("Server.Items.BladedStaff", 16);
            sb.sell.add("Server.Items.Pike", 19);
            sb.sell.add("Server.Items.DoubleBladedStaff", 17);
            sb.sell.add("Server.Items.Lance", 17);
            sb.sell.add("Server.Items.CrescentBlade", 18);
        }
        sb.sell.add("Server.Items.Spear", 15);
        sb.sell.add("Server.Items.Pitchfork", 9);
        sb.sell.add("Server.Items.ShortSpear", 11);
        sb.sell.add("Server.Items.BlackStaff", 11);
        sb.sell.add("Server.Items.GnarledStaff", 8);
        sb.sell.add("Server.Items.QuarterStaff", 9);
        sb.sell.add("Server.Items.ShepherdsCrook", 10);
        sb.sell.add("Server.Items.SmithHammer", 10);
        sb.sell.add("Server.Items.Broadsword", 17);
        sb.sell.add("Server.Items.Cutlass", 12);
        sb.sell.add("Server.Items.Katana", 16);
        sb.sell.add("Server.Items.Kryss", 16);
        sb.sell.add("Server.Items.Longsword", 27);
        sb.sell.add("Server.Items.Scimitar", 18);
        sb.sell.add("Server.Items.ThinLongsword", 13);
        sb.sell.add("Server.Items.VikingSword", 27);
        sb.sell.add("Server.Items.Hatchet", 13);
        sb.sell.add("Server.Items.WarFork", 16);
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.BlackStaff",
            22,
            20,
            0xDF1,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Club", 16, 20, 0x13B4, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.GnarledStaff",
            16,
            20,
            0x13F8,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Mace", 28, 20, 0xF5C, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Maul", 21, 20, 0x143B, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.QuarterStaff",
            19,
            20,
            0xE89,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.ShepherdsCrook",
            20,
            20,
            0xE81,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.SmithHammer",
            21,
            20,
            0x13E3,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.ShortSpear",
            23,
            20,
            0x1403,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Spear", 31, 20, 0xF62, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.WarHammer",
            25,
            20,
            0x1439,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.WarMace",
            31,
            20,
            0x1407,
            0,
        ));
        if w.core_aos() {
            sb.buy.push(BuyInfo::new(
                None,
                "Server.Items.Scepter",
                39,
                20,
                0x26BC,
                0,
            ));
            sb.buy.push(BuyInfo::new(
                None,
                "Server.Items.BladedStaff",
                40,
                20,
                0x26BD,
                0,
            ));
        }
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Hatchet", 25, 20, 0xF44, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Hatchet", 27, 20, 0xF43, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.WarFork",
            32,
            20,
            0x1405,
            0,
        ));
        if rnd::below(w, 3) == 0 {
            sb.buy.push(BuyInfo::new(
                None,
                "Server.Items.ExecutionersAxe",
                30,
                20,
                0xF45,
                0,
            ));
            sb.buy.push(BuyInfo::new(
                None,
                "Server.Items.Bardiche",
                60,
                20,
                0xF4D,
                0,
            ));
            sb.buy.push(BuyInfo::new(
                None,
                "Server.Items.BattleAxe",
                26,
                20,
                0xF47,
                0,
            ));
            sb.buy.push(BuyInfo::new(
                None,
                "Server.Items.TwoHandedAxe",
                32,
                20,
                0x1443,
                0,
            ));
            sb.buy
                .push(BuyInfo::new(None, "Server.Items.Bow", 35, 20, 0x13B2, 0));
            sb.buy.push(BuyInfo::new(
                None,
                "Server.Items.ButcherKnife",
                14,
                20,
                0x13F6,
                0,
            ));
            sb.buy.push(BuyInfo::new(
                None,
                "Server.Items.Crossbow",
                46,
                20,
                0xF50,
                0,
            ));
            sb.buy.push(BuyInfo::new(
                None,
                "Server.Items.HeavyCrossbow",
                55,
                20,
                0x13FD,
                0,
            ));
            sb.buy.push(BuyInfo::new(
                None,
                "Server.Items.Cutlass",
                24,
                20,
                0x1441,
                0,
            ));
            sb.buy
                .push(BuyInfo::new(None, "Server.Items.Dagger", 21, 20, 0xF52, 0));
            sb.buy.push(BuyInfo::new(
                None,
                "Server.Items.Halberd",
                42,
                20,
                0x143E,
                0,
            ));
            sb.buy.push(BuyInfo::new(
                None,
                "Server.Items.HammerPick",
                26,
                20,
                0x143D,
                0,
            ));
            sb.buy
                .push(BuyInfo::new(None, "Server.Items.Katana", 33, 20, 0x13FF, 0));
            sb.buy
                .push(BuyInfo::new(None, "Server.Items.Kryss", 32, 20, 0x1401, 0));
            sb.buy.push(BuyInfo::new(
                None,
                "Server.Items.Broadsword",
                35,
                20,
                0xF5E,
                0,
            ));
            sb.buy.push(BuyInfo::new(
                None,
                "Server.Items.Longsword",
                55,
                20,
                0xF61,
                0,
            ));
            sb.buy.push(BuyInfo::new(
                None,
                "Server.Items.ThinLongsword",
                27,
                20,
                0x13B8,
                0,
            ));
            sb.buy.push(BuyInfo::new(
                None,
                "Server.Items.VikingSword",
                55,
                20,
                0x13B9,
                0,
            ));
            sb.buy
                .push(BuyInfo::new(None, "Server.Items.Cleaver", 15, 20, 0xEC3, 0));
            sb.buy
                .push(BuyInfo::new(None, "Server.Items.Axe", 40, 20, 0xF49, 0));
            sb.buy.push(BuyInfo::new(
                None,
                "Server.Items.DoubleAxe",
                52,
                20,
                0xF4B,
                0,
            ));
            sb.buy
                .push(BuyInfo::new(None, "Server.Items.Pickaxe", 22, 20, 0xE86, 0));
            sb.buy.push(BuyInfo::new(
                None,
                "Server.Items.Pitchfork",
                19,
                20,
                0xE87,
                0,
            ));
            sb.buy.push(BuyInfo::new(
                None,
                "Server.Items.Scimitar",
                36,
                20,
                0x13B6,
                0,
            ));
            sb.buy.push(BuyInfo::new(
                None,
                "Server.Items.SkinningKnife",
                14,
                20,
                0xEC4,
                0,
            ));
            sb.buy.push(BuyInfo::new(
                None,
                "Server.Items.LargeBattleAxe",
                33,
                20,
                0x13FB,
                0,
            ));
            sb.buy
                .push(BuyInfo::new(None, "Server.Items.WarAxe", 29, 20, 0x13B0, 0));
            if w.core_aos() {
                sb.buy.push(BuyInfo::new(
                    None,
                    "Server.Items.BoneHarvester",
                    35,
                    20,
                    0x26BB,
                    0,
                ));
                sb.buy.push(BuyInfo::new(
                    None,
                    "Server.Items.CrescentBlade",
                    37,
                    20,
                    0x26C1,
                    0,
                ));
                sb.buy.push(BuyInfo::new(
                    None,
                    "Server.Items.DoubleBladedStaff",
                    35,
                    20,
                    0x26BF,
                    0,
                ));
                sb.buy
                    .push(BuyInfo::new(None, "Server.Items.Lance", 34, 20, 0x26C0, 0));
                sb.buy
                    .push(BuyInfo::new(None, "Server.Items.Pike", 39, 20, 0x26BE, 0));
                sb.buy
                    .push(BuyInfo::new(None, "Server.Items.Scythe", 39, 20, 0x26BA, 0));
                sb.buy.push(BuyInfo::new(
                    None,
                    "Server.Items.CompositeBow",
                    50,
                    20,
                    0x26C2,
                    0,
                ));
                sb.buy.push(BuyInfo::new(
                    None,
                    "Server.Items.RepeatingCrossbow",
                    57,
                    20,
                    0x26C3,
                    0,
                ));
            }
        }
    },
};

/// `SBWeaver`.
pub static SBWEAVER: SbInfo = SbInfo {
    name: "SBWeaver",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.Scissors", 6);
        sb.sell.add("Server.Items.Dyes", 4);
        sb.sell.add("Server.Items.DyeTub", 4);
        sb.sell.add("Server.Items.UncutCloth", 1);
        sb.sell.add("Server.Items.BoltOfCloth", 50);
        sb.sell.add("Server.Items.LightYarnUnraveled", 9);
        sb.sell.add("Server.Items.LightYarn", 9);
        sb.sell.add("Server.Items.DarkYarn", 9);
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Dyes", 8, 20, 0xFA9, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.DyeTub", 8, 20, 0xFAB, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.UncutCloth",
            3,
            20,
            0x1761,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.UncutCloth",
            3,
            20,
            0x1762,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.UncutCloth",
            3,
            20,
            0x1763,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.UncutCloth",
            3,
            20,
            0x1764,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.BoltOfCloth",
            100,
            20,
            0xF9B,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.BoltOfCloth",
            100,
            20,
            0xF9C,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.BoltOfCloth",
            100,
            20,
            0xf96,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.BoltOfCloth",
            100,
            20,
            0xf97,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.DarkYarn",
            18,
            20,
            0xE1D,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.LightYarn",
            18,
            20,
            0xE1E,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.LightYarnUnraveled",
            18,
            20,
            0xE1F,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Scissors",
            11,
            20,
            0xF9F,
            0,
        ));
    },
};

/// `SBSEArmor`.
pub static SBSEARMOR: SbInfo = SbInfo {
    name: "SBSEArmor",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.PlateHatsuburi", 38);
        sb.sell.add("Server.Items.HeavyPlateJingasa", 38);
        sb.sell.add("Server.Items.DecorativePlateKabuto", 47);
        sb.sell.add("Server.Items.PlateDo", 155);
        sb.sell.add("Server.Items.PlateHiroSode", 111);
        sb.sell.add("Server.Items.PlateSuneate", 112);
        sb.sell.add("Server.Items.PlateHaidate", 117);
        sb.sell.add("Server.Items.ChainHatsuburi", 38);
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.PlateHatsuburi",
            76,
            20,
            0x2775,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.HeavyPlateJingasa",
            76,
            20,
            0x2777,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.DecorativePlateKabuto",
            95,
            20,
            0x2778,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.PlateDo",
            310,
            20,
            0x277D,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.PlateHiroSode",
            222,
            20,
            0x2780,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.PlateSuneate",
            224,
            20,
            0x2788,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.PlateHaidate",
            235,
            20,
            0x278D,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.ChainHatsuburi",
            76,
            20,
            0x2774,
            0,
        ));
    },
};

/// `SBSEBowyer`.
pub static SBSEBOWYER: SbInfo = SbInfo {
    name: "SBSEBowyer",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.Yumi", 26);
        sb.sell.add("Server.Items.Fukiya", 10);
        sb.sell.add("Server.Items.Nunchaku", 17);
        sb.sell.add("Server.Items.FukiyaDarts", 1);
        sb.sell.add("Server.Items.Bokuto", 10);
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Yumi", 53, 20, 0x27A5, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Fukiya", 20, 20, 0x27AA, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Nunchaku",
            35,
            20,
            0x27AE,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.FukiyaDarts",
            3,
            20,
            0x2806,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Bokuto", 21, 20, 0x27A8, 0));
    },
};

/// `SBSECarpenter`.
pub static SBSECARPENTER: SbInfo = SbInfo {
    name: "SBSECarpenter",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.Tetsubo", 21);
        sb.sell.add("Server.Items.Fukiya", 10);
        sb.sell.add("Server.Items.BambooFlute", 10);
        sb.sell.add("Server.Items.Bokuto", 10);
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Bokuto", 21, 20, 0x27A8, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Tetsubo",
            43,
            20,
            0x27A6,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Fukiya", 20, 20, 0x27AA, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.BambooFlute",
            21,
            20,
            0x2805,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.BambooFlute",
            21,
            20,
            0x2805,
            0,
        ));
    },
};

/// `SBSEFood`.
pub static SBSEFOOD: SbInfo = SbInfo {
    name: "SBSEFood",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.Wasabi", 1);
        sb.sell.add("Server.Items.BentoBox", 3);
        sb.sell.add("Server.Items.GreenTeaBasket", 1);
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Wasabi", 2, 20, 0x24E8, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Wasabi", 2, 20, 0x24E9, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.BentoBox",
            6,
            20,
            0x2836,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.BentoBox",
            6,
            20,
            0x2837,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.GreenTeaBasket",
            2,
            20,
            0x284B,
            0,
        ));
    },
};

/// `SBSELeatherArmor`.
pub static SBSELEATHER_ARMOR: SbInfo = SbInfo {
    name: "SBSELeatherArmor",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.LeatherJingasa", 5);
        sb.sell.add("Server.Items.LeatherDo", 42);
        sb.sell.add("Server.Items.LeatherHiroSode", 23);
        sb.sell.add("Server.Items.LeatherSuneate", 26);
        sb.sell.add("Server.Items.LeatherHaidate", 28);
        sb.sell.add("Server.Items.LeatherNinjaPants", 25);
        sb.sell.add("Server.Items.LeatherNinjaJacket", 26);
        sb.sell.add("Server.Items.StuddedMempo", 28);
        sb.sell.add("Server.Items.StuddedDo", 66);
        sb.sell.add("Server.Items.StuddedHiroSode", 32);
        sb.sell.add("Server.Items.StuddedSuneate", 40);
        sb.sell.add("Server.Items.StuddedHaidate", 37);
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.LeatherJingasa",
            11,
            20,
            0x2776,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.LeatherDo",
            87,
            20,
            0x277B,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.LeatherHiroSode",
            49,
            20,
            0x277E,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.LeatherSuneate",
            55,
            20,
            0x2786,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.LeatherHaidate",
            54,
            20,
            0x278A,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.LeatherNinjaPants",
            49,
            20,
            0x2791,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.LeatherNinjaJacket",
            51,
            20,
            0x2793,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.StuddedMempo",
            61,
            20,
            0x279D,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.StuddedDo",
            130,
            20,
            0x277C,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.StuddedHiroSode",
            73,
            20,
            0x277F,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.StuddedSuneate",
            78,
            20,
            0x2787,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.StuddedHaidate",
            76,
            20,
            0x278B,
            0,
        ));
    },
};

/// `SBSEWeapons`.
pub static SBSEWEAPONS: SbInfo = SbInfo {
    name: "SBSEWeapons",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.NoDachi", 41);
        sb.sell.add("Server.Items.Tessen", 41);
        sb.sell.add("Server.Items.Wakizashi", 19);
        sb.sell.add("Server.Items.Tetsubo", 21);
        sb.sell.add("Server.Items.Lajatang", 54);
        sb.sell.add("Server.Items.Daisho", 33);
        sb.sell.add("Server.Items.Tekagi", 22);
        sb.sell.add("Server.Items.Shuriken", 9);
        sb.sell.add("Server.Items.Kama", 30);
        sb.sell.add("Server.Items.Sai", 28);
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.NoDachi",
            82,
            20,
            0x27A2,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Tessen", 83, 20, 0x27A3, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Wakizashi",
            38,
            20,
            0x27A4,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Tetsubo",
            43,
            20,
            0x27A6,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Lajatang",
            108,
            20,
            0x27A7,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Daisho", 66, 20, 0x27A9, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Tekagi", 55, 20, 0x27AB, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Shuriken",
            18,
            20,
            0x27AC,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Kama", 61, 20, 0x27AD, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Sai", 56, 20, 0x27AF, 0));
    },
};

/// `SBAxeWeapon`.
pub static SBAXE_WEAPON: SbInfo = SbInfo {
    name: "SBAxeWeapon",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.BattleAxe", 13);
        sb.sell.add("Server.Items.DoubleAxe", 26);
        sb.sell.add("Server.Items.ExecutionersAxe", 15);
        sb.sell.add("Server.Items.LargeBattleAxe", 16);
        sb.sell.add("Server.Items.Pickaxe", 11);
        sb.sell.add("Server.Items.TwoHandedAxe", 16);
        sb.sell.add("Server.Items.WarAxe", 14);
        sb.sell.add("Server.Items.Axe", 20);
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.ExecutionersAxe",
            30,
            20,
            0xF45,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.BattleAxe",
            26,
            20,
            0xF47,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.TwoHandedAxe",
            32,
            20,
            0x1443,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Axe", 40, 20, 0xF49, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.DoubleAxe",
            52,
            20,
            0xF4B,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Pickaxe", 22, 20, 0xE86, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.LargeBattleAxe",
            33,
            20,
            0x13FB,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.WarAxe", 29, 20, 0x13B0, 0));
    },
};

/// `SBKnifeWeapon`.
pub static SBKNIFE_WEAPON: SbInfo = SbInfo {
    name: "SBKnifeWeapon",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.ButcherKnife", 7);
        sb.sell.add("Server.Items.Cleaver", 7);
        sb.sell.add("Server.Items.Dagger", 10);
        sb.sell.add("Server.Items.SkinningKnife", 7);
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.ButcherKnife",
            14,
            20,
            0x13F6,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Dagger", 21, 20, 0xF52, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Cleaver", 15, 20, 0xEC3, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.SkinningKnife",
            14,
            20,
            0xEC4,
            0,
        ));
    },
};

/// `SBMaceWeapon`.
pub static SBMACE_WEAPON: SbInfo = SbInfo {
    name: "SBMaceWeapon",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.Club", 8);
        sb.sell.add("Server.Items.HammerPick", 13);
        sb.sell.add("Server.Items.Mace", 14);
        sb.sell.add("Server.Items.Maul", 10);
        sb.sell.add("Server.Items.WarHammer", 12);
        sb.sell.add("Server.Items.WarMace", 15);
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.HammerPick",
            26,
            20,
            0x143D,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Club", 16, 20, 0x13B4, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Mace", 28, 20, 0xF5C, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Maul", 21, 20, 0x143B, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.WarHammer",
            25,
            20,
            0x1439,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.WarMace",
            31,
            20,
            0x1407,
            0,
        ));
    },
};

/// `SBPoleArmWeapon`.
pub static SBPOLE_ARM_WEAPON: SbInfo = SbInfo {
    name: "SBPoleArmWeapon",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.Bardiche", 30);
        sb.sell.add("Server.Items.Halberd", 21);
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Bardiche",
            60,
            20,
            0xF4D,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Halberd",
            42,
            20,
            0x143E,
            0,
        ));
    },
};

/// `SBRangedWeapon`.
pub static SBRANGED_WEAPON: SbInfo = SbInfo {
    name: "SBRangedWeapon",
    build: |w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.Bolt", 1);
        sb.sell.add("Server.Items.Arrow", 1);
        sb.sell.add("Server.Items.Shaft", 1);
        sb.sell.add("Server.Items.Feather", 1);
        sb.sell.add("Server.Items.HeavyCrossbow", 27);
        sb.sell.add("Server.Items.Bow", 17);
        sb.sell.add("Server.Items.Crossbow", 25);
        if w.core_aos() {
            sb.sell.add("Server.Items.CompositeBow", 23);
            sb.sell.add("Server.Items.RepeatingCrossbow", 22);
        }
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Crossbow",
            55,
            20,
            0xF50,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.HeavyCrossbow",
            55,
            20,
            0x13FD,
            0,
        ));
        if w.core_aos() {
            sb.buy.push(BuyInfo::new(
                None,
                "Server.Items.RepeatingCrossbow",
                46,
                20,
                0x26C3,
                0,
            ));
            sb.buy.push(BuyInfo::new(
                None,
                "Server.Items.CompositeBow",
                45,
                20,
                0x26C2,
                0,
            ));
        }
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Bolt",
            3,
            rnd::from(w, 30, 60),
            0x1BFB,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Bow", 40, 20, 0x13B2, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Arrow",
            2,
            rnd::from(w, 30, 60),
            0xF3F,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Feather",
            2,
            rnd::from(w, 30, 60),
            0x1BD1,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Shaft",
            3,
            rnd::from(w, 30, 60),
            0x1BD4,
            0,
        ));
    },
};

/// `SBSpearForkWeapon`.
pub static SBSPEAR_FORK_WEAPON: SbInfo = SbInfo {
    name: "SBSpearForkWeapon",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.Spear", 15);
        sb.sell.add("Server.Items.Pitchfork", 9);
        sb.sell.add("Server.Items.ShortSpear", 11);
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Pitchfork",
            19,
            20,
            0xE87,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.ShortSpear",
            23,
            20,
            0x1403,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Spear", 31, 20, 0xF62, 0));
    },
};

/// `SBStavesWeapon`.
pub static SBSTAVES_WEAPON: SbInfo = SbInfo {
    name: "SBStavesWeapon",
    build: |_w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.BlackStaff", 11);
        sb.sell.add("Server.Items.GnarledStaff", 8);
        sb.sell.add("Server.Items.QuarterStaff", 9);
        sb.sell.add("Server.Items.ShepherdsCrook", 10);
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.BlackStaff",
            22,
            20,
            0xDF1,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.GnarledStaff",
            16,
            20,
            0x13F8,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.QuarterStaff",
            19,
            20,
            0xE89,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.ShepherdsCrook",
            20,
            20,
            0xE81,
            0,
        ));
    },
};

/// `SBSwordWeapon`.
pub static SBSWORD_WEAPON: SbInfo = SbInfo {
    name: "SBSwordWeapon",
    build: |w: &mut World, _vendor: MobileId, sb: &mut SbInstance| {
        sb.sell.add("Server.Items.Broadsword", 17);
        sb.sell.add("Server.Items.Cutlass", 12);
        sb.sell.add("Server.Items.Katana", 16);
        sb.sell.add("Server.Items.Kryss", 16);
        sb.sell.add("Server.Items.Longsword", 27);
        sb.sell.add("Server.Items.Scimitar", 18);
        sb.sell.add("Server.Items.ThinLongsword", 13);
        sb.sell.add("Server.Items.VikingSword", 27);
        if w.core_aos() {
            sb.sell.add("Server.Items.Scythe", 19);
            sb.sell.add("Server.Items.BoneHarvester", 17);
            sb.sell.add("Server.Items.Scepter", 18);
            sb.sell.add("Server.Items.BladedStaff", 16);
            sb.sell.add("Server.Items.Pike", 19);
            sb.sell.add("Server.Items.DoubleBladedStaff", 17);
            sb.sell.add("Server.Items.Lance", 17);
            sb.sell.add("Server.Items.CrescentBlade", 18);
        }
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Cutlass",
            24,
            20,
            0x1441,
            0,
        ));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Katana", 33, 20, 0x13FF, 0));
        sb.buy
            .push(BuyInfo::new(None, "Server.Items.Kryss", 32, 20, 0x1401, 0));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Broadsword",
            35,
            20,
            0xF5E,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Longsword",
            55,
            20,
            0xF61,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.ThinLongsword",
            27,
            20,
            0x13B8,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.VikingSword",
            55,
            20,
            0x13B9,
            0,
        ));
        sb.buy.push(BuyInfo::new(
            None,
            "Server.Items.Scimitar",
            36,
            20,
            0x13B6,
            0,
        ));
        if w.core_aos() {
            sb.buy.push(BuyInfo::new(
                None,
                "Server.Items.BoneHarvester",
                35,
                20,
                0x26BB,
                0,
            ));
            sb.buy.push(BuyInfo::new(
                None,
                "Server.Items.CrescentBlade",
                37,
                20,
                0x26C1,
                0,
            ));
            sb.buy.push(BuyInfo::new(
                None,
                "Server.Items.DoubleBladedStaff",
                35,
                20,
                0x26BF,
                0,
            ));
            sb.buy
                .push(BuyInfo::new(None, "Server.Items.Lance", 34, 20, 0x26C0, 0));
            sb.buy
                .push(BuyInfo::new(None, "Server.Items.Pike", 39, 20, 0x26BE, 0));
            sb.buy
                .push(BuyInfo::new(None, "Server.Items.Scythe", 39, 20, 0x26BA, 0));
        }
    },
};
