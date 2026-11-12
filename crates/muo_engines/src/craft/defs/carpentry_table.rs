//! The Carpentry craft list (`InitCraftList`), generated: do not edit by hand.

use crate::craft::model::*;
use muo_content::core;
use muo_data::expansion::Expansion;
use muo_io::text::TextDefinition as Td;
use muo_server::World;
use muo_server::skills::SkillName;

/// `InitCraftList()`.
pub fn init(w: &World, s: &mut CraftSystem) {
    let mut index;
    if matches!(core::expansion(w), Expansion::AOS | Expansion::SE) {
        index = s.add_craft(
            "Server.Items.Board",
            Td::of_number(1044294),
            Td::of_number(1027127),
            None,
            None,
            0.0,
            0.0,
            "Server.Items.Log",
            Td::of_number(1044466),
            1,
            Td::of_number(1044465),
        );
        s.set_use_all_res(index, true);
    }
    s.add_craft(
        "Server.Items.BarrelStaves",
        Td::of_number(1044294),
        Td::of_number(1027857),
        None,
        None,
        00.0,
        25.0,
        "Server.Items.Log",
        Td::of_number(1044041),
        5,
        Td::of_number(1044351),
    );
    s.add_craft(
        "Server.Items.BarrelLid",
        Td::of_number(1044294),
        Td::of_number(1027608),
        None,
        None,
        11.0,
        36.0,
        "Server.Items.Log",
        Td::of_number(1044041),
        4,
        Td::of_number(1044351),
    );
    s.add_craft(
        "Server.Items.ShortMusicStand",
        Td::of_number(1044294),
        Td::of_number(1044313),
        None,
        None,
        78.9,
        103.9,
        "Server.Items.Log",
        Td::of_number(1044041),
        15,
        Td::of_number(1044351),
    );
    s.add_craft(
        "Server.Items.TallMusicStand",
        Td::of_number(1044294),
        Td::of_number(1044315),
        None,
        None,
        81.5,
        106.5,
        "Server.Items.Log",
        Td::of_number(1044041),
        20,
        Td::of_number(1044351),
    );
    s.add_craft(
        "Server.Items.Easel",
        Td::of_number(1044294),
        Td::of_number(1044317),
        None,
        None,
        86.8,
        111.8,
        "Server.Items.Log",
        Td::of_number(1044041),
        20,
        Td::of_number(1044351),
    );
    if core::se(w) {
        index = s.add_craft(
            "Server.Items.RedHangingLantern",
            Td::of_number(1044294),
            Td::of_number(1029412),
            None,
            None,
            65.0,
            90.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            5,
            Td::of_number(1044351),
        );
        s.add_res(
            index,
            "Server.Items.BlankScroll",
            Td::of_number(1044377),
            10,
            Td::of_number(1044378),
        );
        s.set_needed_expansion(index, Expansion::SE);
        index = s.add_craft(
            "Server.Items.WhiteHangingLantern",
            Td::of_number(1044294),
            Td::of_number(1029416),
            None,
            None,
            65.0,
            90.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            5,
            Td::of_number(1044351),
        );
        s.add_res(
            index,
            "Server.Items.BlankScroll",
            Td::of_number(1044377),
            10,
            Td::of_number(1044378),
        );
        s.set_needed_expansion(index, Expansion::SE);
        index = s.add_craft(
            "Server.Items.ShojiScreen",
            Td::of_number(1044294),
            Td::of_number(1029423),
            None,
            None,
            80.0,
            105.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            75,
            Td::of_number(1044351),
        );
        s.add_skill(index, SkillName::Tailoring, 50.0, 55.0);
        s.add_res(
            index,
            "Server.Items.Cloth",
            Td::of_number(1044286),
            60,
            Td::of_number(1044287),
        );
        s.set_needed_expansion(index, Expansion::SE);
        index = s.add_craft(
            "Server.Items.BambooScreen",
            Td::of_number(1044294),
            Td::of_number(1029428),
            None,
            None,
            80.0,
            105.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            75,
            Td::of_number(1044351),
        );
        s.add_skill(index, SkillName::Tailoring, 50.0, 55.0);
        s.add_res(
            index,
            "Server.Items.Cloth",
            Td::of_number(1044286),
            60,
            Td::of_number(1044287),
        );
        s.set_needed_expansion(index, Expansion::SE);
    }
    if core::aos(w) {
        index = s.add_craft(
            "Server.Items.FishingPole",
            Td::of_number(1044294),
            Td::of_number(1023519),
            None,
            None,
            68.4,
            93.4,
            "Server.Items.Log",
            Td::of_number(1044041),
            5,
            Td::of_number(1044351),
        );
        s.add_skill(index, SkillName::Tailoring, 40.0, 45.0);
        s.add_res(
            index,
            "Server.Items.Cloth",
            Td::of_number(1044286),
            5,
            Td::of_number(1044287),
        );
    }
    if core::ml(w) {
        index = s.add_craft(
            "Server.Items.RunedSwitch",
            Td::of_number(1044294),
            Td::of_number(1072896),
            None,
            None,
            70.0,
            120.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            2,
            Td::of_number(1044351),
        );
        s.add_res(
            index,
            "Server.Items.EnchantedSwitch",
            Td::of_number(1072893),
            1,
            Td::of_number(1053098),
        );
        s.add_res(
            index,
            "Server.Items.RunedPrism",
            Td::of_number(1073465),
            1,
            Td::of_number(1053098),
        );
        s.add_res(
            index,
            "Server.Items.JeweledFiligree",
            Td::of_number(1072894),
            1,
            Td::of_number(1053098),
        );
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.ArcanistStatueSouthDeed",
            Td::of_number(1044294),
            Td::of_number(1072885),
            None,
            None,
            0.0,
            35.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            250,
            Td::of_number(1044351),
        );
        s.force_non_exceptional(index);
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.ArcanistStatueEastDeed",
            Td::of_number(1044294),
            Td::of_number(1072886),
            None,
            None,
            0.0,
            35.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            250,
            Td::of_number(1044351),
        );
        s.force_non_exceptional(index);
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.WarriorStatueSouthDeed",
            Td::of_number(1044294),
            Td::of_number(1072887),
            None,
            None,
            0.0,
            35.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            250,
            Td::of_number(1044351),
        );
        s.add_recipe(index, 300);
        s.force_non_exceptional(index);
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.WarriorStatueEastDeed",
            Td::of_number(1044294),
            Td::of_number(1072888),
            None,
            None,
            0.0,
            35.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            250,
            Td::of_number(1044351),
        );
        s.add_recipe(index, 301);
        s.force_non_exceptional(index);
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.SquirrelStatueSouthDeed",
            Td::of_number(1044294),
            Td::of_number(1072884),
            None,
            None,
            0.0,
            35.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            250,
            Td::of_number(1044351),
        );
        s.add_recipe(index, 302);
        s.force_non_exceptional(index);
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.SquirrelStatueEastDeed",
            Td::of_number(1044294),
            Td::of_number(1073398),
            None,
            None,
            0.0,
            35.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            250,
            Td::of_number(1044351),
        );
        s.add_recipe(index, 303);
        s.force_non_exceptional(index);
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.GiantReplicaAcorn",
            Td::of_number(1044294),
            Td::of_number(1072889),
            None,
            None,
            80.0,
            105.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            35,
            Td::of_number(1044351),
        );
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.MountedDreadHorn",
            Td::of_number(1044294),
            Td::of_number(1032632),
            None,
            None,
            90.0,
            115.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            50,
            Td::of_number(1044351),
        );
        s.add_res(
            index,
            "Server.Items.PristineDreadHorn",
            Td::of_number(1032634),
            1,
            Td::of_number(1053098),
        );
        s.force_non_exceptional(index);
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.AcidProofRope",
            Td::of_number(1044294),
            Td::of_number(1074886),
            None,
            None,
            80.0,
            130.0,
            "Server.Items.GreaterStrengthPotion",
            Td::of_number(1073466),
            2,
            Td::of_number(1044253),
        );
        s.add_res(
            index,
            "Server.Items.ProtectionScroll",
            Td::of_number(1044395),
            1,
            Td::of_number(1053098),
        );
        s.add_res(
            index,
            "Server.Items.SwitchItem",
            Td::of_number(1032127),
            1,
            Td::of_number(1053098),
        );
        s.add_rare_recipe(index, 304);
        s.force_non_exceptional(index);
        s.set_needed_expansion(index, Expansion::ML);
    }
    s.add_craft(
        "Server.Items.FootStool",
        Td::of_number(1044291),
        Td::of_number(1022910),
        None,
        None,
        11.0,
        36.0,
        "Server.Items.Log",
        Td::of_number(1044041),
        9,
        Td::of_number(1044351),
    );
    s.add_craft(
        "Server.Items.Stool",
        Td::of_number(1044291),
        Td::of_number(1022602),
        None,
        None,
        11.0,
        36.0,
        "Server.Items.Log",
        Td::of_number(1044041),
        9,
        Td::of_number(1044351),
    );
    s.add_craft(
        "Server.Items.BambooChair",
        Td::of_number(1044291),
        Td::of_number(1044300),
        None,
        None,
        21.0,
        46.0,
        "Server.Items.Log",
        Td::of_number(1044041),
        13,
        Td::of_number(1044351),
    );
    s.add_craft(
        "Server.Items.WoodenChair",
        Td::of_number(1044291),
        Td::of_number(1044301),
        None,
        None,
        21.0,
        46.0,
        "Server.Items.Log",
        Td::of_number(1044041),
        13,
        Td::of_number(1044351),
    );
    s.add_craft(
        "Server.Items.FancyWoodenChairCushion",
        Td::of_number(1044291),
        Td::of_number(1044302),
        None,
        None,
        42.1,
        67.1,
        "Server.Items.Log",
        Td::of_number(1044041),
        15,
        Td::of_number(1044351),
    );
    s.add_craft(
        "Server.Items.WoodenChairCushion",
        Td::of_number(1044291),
        Td::of_number(1044303),
        None,
        None,
        42.1,
        67.1,
        "Server.Items.Log",
        Td::of_number(1044041),
        13,
        Td::of_number(1044351),
    );
    s.add_craft(
        "Server.Items.WoodenBench",
        Td::of_number(1044291),
        Td::of_number(1022860),
        None,
        None,
        52.6,
        77.6,
        "Server.Items.Log",
        Td::of_number(1044041),
        17,
        Td::of_number(1044351),
    );
    s.add_craft(
        "Server.Items.WoodenThrone",
        Td::of_number(1044291),
        Td::of_number(1044304),
        None,
        None,
        52.6,
        77.6,
        "Server.Items.Log",
        Td::of_number(1044041),
        17,
        Td::of_number(1044351),
    );
    s.add_craft(
        "Server.Items.Throne",
        Td::of_number(1044291),
        Td::of_number(1044305),
        None,
        None,
        73.6,
        98.6,
        "Server.Items.Log",
        Td::of_number(1044041),
        19,
        Td::of_number(1044351),
    );
    s.add_craft(
        "Server.Items.Nightstand",
        Td::of_number(1044291),
        Td::of_number(1044306),
        None,
        None,
        42.1,
        67.1,
        "Server.Items.Log",
        Td::of_number(1044041),
        17,
        Td::of_number(1044351),
    );
    s.add_craft(
        "Server.Items.WritingTable",
        Td::of_number(1044291),
        Td::of_number(1022890),
        None,
        None,
        63.1,
        88.1,
        "Server.Items.Log",
        Td::of_number(1044041),
        17,
        Td::of_number(1044351),
    );
    s.add_craft(
        "Server.Items.YewWoodTable",
        Td::of_number(1044291),
        Td::of_number(1044307),
        None,
        None,
        63.1,
        88.1,
        "Server.Items.Log",
        Td::of_number(1044041),
        23,
        Td::of_number(1044351),
    );
    s.add_craft(
        "Server.Items.LargeTable",
        Td::of_number(1044291),
        Td::of_number(1044308),
        None,
        None,
        84.2,
        109.2,
        "Server.Items.Log",
        Td::of_number(1044041),
        27,
        Td::of_number(1044351),
    );
    if core::se(w) {
        index = s.add_craft(
            "Server.Items.ElegantLowTable",
            Td::of_number(1044291),
            Td::of_number(1030265),
            None,
            None,
            80.0,
            105.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            35,
            Td::of_number(1044351),
        );
        s.set_needed_expansion(index, Expansion::SE);
        index = s.add_craft(
            "Server.Items.PlainLowTable",
            Td::of_number(1044291),
            Td::of_number(1030266),
            None,
            None,
            80.0,
            105.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            35,
            Td::of_number(1044351),
        );
        s.set_needed_expansion(index, Expansion::SE);
    }
    if core::ml(w) {
        index = s.add_craft(
            "Server.Items.OrnateElvenTableSouthDeed",
            Td::of_number(1044291),
            Td::of_number(1072869),
            None,
            None,
            85.0,
            110.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            60,
            Td::of_number(1044351),
        );
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.OrnateElvenTableEastDeed",
            Td::of_number(1044291),
            Td::of_number(1073384),
            None,
            None,
            85.0,
            110.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            60,
            Td::of_number(1044351),
        );
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.FancyElvenTableSouthDeed",
            Td::of_number(1044291),
            Td::of_number(1073385),
            None,
            None,
            80.0,
            105.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            50,
            Td::of_number(1044351),
        );
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.FancyElvenTableEastDeed",
            Td::of_number(1044291),
            Td::of_number(1073386),
            None,
            None,
            80.0,
            105.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            50,
            Td::of_number(1044351),
        );
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.ElvenPodium",
            Td::of_number(1044291),
            Td::of_number(1073399),
            None,
            None,
            80.0,
            105.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            20,
            Td::of_number(1044351),
        );
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.OrnateElvenChair",
            Td::of_number(1044291),
            Td::of_number(1072870),
            None,
            None,
            80.0,
            105.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            30,
            Td::of_number(1044351),
        );
        s.add_recipe(index, 305);
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.BigElvenChair",
            Td::of_number(1044291),
            Td::of_number(1072872),
            None,
            None,
            85.0,
            110.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            40,
            Td::of_number(1044351),
        );
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.ElvenReadingChair",
            Td::of_number(1044291),
            Td::of_number(1072873),
            None,
            None,
            80.0,
            105.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            30,
            Td::of_number(1044351),
        );
        s.set_needed_expansion(index, Expansion::ML);
    }
    s.add_craft(
        "Server.Items.WoodenBox",
        Td::of_number(1044292),
        Td::of_number(1023709),
        None,
        None,
        21.0,
        46.0,
        "Server.Items.Log",
        Td::of_number(1044041),
        10,
        Td::of_number(1044351),
    );
    s.add_craft(
        "Server.Items.SmallCrate",
        Td::of_number(1044292),
        Td::of_number(1044309),
        None,
        None,
        10.0,
        35.0,
        "Server.Items.Log",
        Td::of_number(1044041),
        8,
        Td::of_number(1044351),
    );
    s.add_craft(
        "Server.Items.MediumCrate",
        Td::of_number(1044292),
        Td::of_number(1044310),
        None,
        None,
        31.0,
        56.0,
        "Server.Items.Log",
        Td::of_number(1044041),
        15,
        Td::of_number(1044351),
    );
    s.add_craft(
        "Server.Items.LargeCrate",
        Td::of_number(1044292),
        Td::of_number(1044311),
        None,
        None,
        47.3,
        72.3,
        "Server.Items.Log",
        Td::of_number(1044041),
        18,
        Td::of_number(1044351),
    );
    s.add_craft(
        "Server.Items.WoodenChest",
        Td::of_number(1044292),
        Td::of_number(1023650),
        None,
        None,
        73.6,
        98.6,
        "Server.Items.Log",
        Td::of_number(1044041),
        20,
        Td::of_number(1044351),
    );
    s.add_craft(
        "Server.Items.EmptyBookcase",
        Td::of_number(1044292),
        Td::of_number(1022718),
        None,
        None,
        31.5,
        56.5,
        "Server.Items.Log",
        Td::of_number(1044041),
        25,
        Td::of_number(1044351),
    );
    s.add_craft(
        "Server.Items.FancyArmoire",
        Td::of_number(1044292),
        Td::of_number(1044312),
        None,
        None,
        84.2,
        109.2,
        "Server.Items.Log",
        Td::of_number(1044041),
        35,
        Td::of_number(1044351),
    );
    s.add_craft(
        "Server.Items.Armoire",
        Td::of_number(1044292),
        Td::of_number(1022643),
        None,
        None,
        84.2,
        109.2,
        "Server.Items.Log",
        Td::of_number(1044041),
        35,
        Td::of_number(1044351),
    );
    if core::se(w) {
        index = s.add_craft(
            "Server.Items.PlainWoodenChest",
            Td::of_number(1044292),
            Td::of_number(1030251),
            None,
            None,
            90.0,
            115.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            30,
            Td::of_number(1044351),
        );
        s.set_needed_expansion(index, Expansion::SE);
        index = s.add_craft(
            "Server.Items.OrnateWoodenChest",
            Td::of_number(1044292),
            Td::of_number(1030253),
            None,
            None,
            90.0,
            115.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            30,
            Td::of_number(1044351),
        );
        s.set_needed_expansion(index, Expansion::SE);
        index = s.add_craft(
            "Server.Items.GildedWoodenChest",
            Td::of_number(1044292),
            Td::of_number(1030255),
            None,
            None,
            90.0,
            115.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            30,
            Td::of_number(1044351),
        );
        s.set_needed_expansion(index, Expansion::SE);
        index = s.add_craft(
            "Server.Items.WoodenFootLocker",
            Td::of_number(1044292),
            Td::of_number(1030257),
            None,
            None,
            90.0,
            115.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            30,
            Td::of_number(1044351),
        );
        s.set_needed_expansion(index, Expansion::SE);
        index = s.add_craft(
            "Server.Items.FinishedWoodenChest",
            Td::of_number(1044292),
            Td::of_number(1030259),
            None,
            None,
            90.0,
            115.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            30,
            Td::of_number(1044351),
        );
        s.set_needed_expansion(index, Expansion::SE);
        index = s.add_craft(
            "Server.Items.TallCabinet",
            Td::of_number(1044292),
            Td::of_number(1030261),
            None,
            None,
            90.0,
            115.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            35,
            Td::of_number(1044351),
        );
        s.set_needed_expansion(index, Expansion::SE);
        index = s.add_craft(
            "Server.Items.ShortCabinet",
            Td::of_number(1044292),
            Td::of_number(1030263),
            None,
            None,
            90.0,
            115.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            35,
            Td::of_number(1044351),
        );
        s.set_needed_expansion(index, Expansion::SE);
        index = s.add_craft(
            "Server.Items.RedArmoire",
            Td::of_number(1044292),
            Td::of_number(1030328),
            None,
            None,
            90.0,
            115.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            40,
            Td::of_number(1044351),
        );
        s.set_needed_expansion(index, Expansion::SE);
        index = s.add_craft(
            "Server.Items.ElegantArmoire",
            Td::of_number(1044292),
            Td::of_number(1030330),
            None,
            None,
            90.0,
            115.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            40,
            Td::of_number(1044351),
        );
        s.set_needed_expansion(index, Expansion::SE);
        index = s.add_craft(
            "Server.Items.MapleArmoire",
            Td::of_number(1044292),
            Td::of_number(1030332),
            None,
            None,
            90.0,
            115.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            40,
            Td::of_number(1044351),
        );
        s.set_needed_expansion(index, Expansion::SE);
        index = s.add_craft(
            "Server.Items.CherryArmoire",
            Td::of_number(1044292),
            Td::of_number(1030334),
            None,
            None,
            90.0,
            115.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            40,
            Td::of_number(1044351),
        );
        s.set_needed_expansion(index, Expansion::SE);
    }
    index = s.add_craft(
        "Server.Items.Keg",
        Td::of_number(1044292),
        Td::of_number(1023711),
        None,
        None,
        57.8,
        82.8,
        "Server.Items.BarrelStaves",
        Td::of_number(1044288),
        3,
        Td::of_number(1044253),
    );
    s.add_res(
        index,
        "Server.Items.BarrelHoops",
        Td::of_number(1044289),
        1,
        Td::of_number(1044253),
    );
    s.add_res(
        index,
        "Server.Items.BarrelLid",
        Td::of_number(1044251),
        1,
        Td::of_number(1044253),
    );
    if core::ml(w) {
        index = s.add_craft(
            "Server.Items.ArcaneBookshelfSouthDeed",
            Td::of_number(1044292),
            Td::of_number(1072871),
            None,
            None,
            94.7,
            119.7,
            "Server.Items.Log",
            Td::of_number(1044041),
            80,
            Td::of_number(1044351),
        );
        s.add_recipe(index, 306);
        s.force_non_exceptional(index);
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.ArcaneBookshelfEastDeed",
            Td::of_number(1044292),
            Td::of_number(1073371),
            None,
            None,
            94.7,
            119.7,
            "Server.Items.Log",
            Td::of_number(1044041),
            80,
            Td::of_number(1044351),
        );
        s.add_recipe(index, 307);
        s.force_non_exceptional(index);
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.OrnateElvenChestSouthDeed",
            Td::of_number(1044292),
            Td::of_number(1072862),
            None,
            None,
            94.7,
            119.7,
            "Server.Items.Log",
            Td::of_number(1044041),
            40,
            Td::of_number(1044351),
        );
        s.add_recipe(index, 308);
        s.force_non_exceptional(index);
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.OrnateElvenChestEastDeed",
            Td::of_number(1044292),
            Td::of_number(1073383),
            None,
            None,
            94.7,
            119.7,
            "Server.Items.Log",
            Td::of_number(1044041),
            40,
            Td::of_number(1044351),
        );
        s.add_recipe(index, 309);
        s.force_non_exceptional(index);
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.ElvenWashBasinSouthDeed",
            Td::of_number(1044292),
            Td::of_number(1072865),
            None,
            None,
            70.0,
            95.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            40,
            Td::of_number(1044351),
        );
        s.force_non_exceptional(index);
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.ElvenWashBasinEastDeed",
            Td::of_number(1044292),
            Td::of_number(1072865),
            None,
            None,
            70.0,
            95.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            40,
            Td::of_number(1044351),
        );
        s.force_non_exceptional(index);
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.ElvenDresserSouthDeed",
            Td::of_number(1044292),
            Td::of_number(1072864),
            None,
            None,
            75.0,
            100.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            45,
            Td::of_number(1044351),
        );
        s.add_recipe(index, 310);
        s.force_non_exceptional(index);
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.ElvenDresserEastDeed",
            Td::of_number(1044292),
            Td::of_number(1073388),
            None,
            None,
            75.0,
            100.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            45,
            Td::of_number(1044351),
        );
        s.add_recipe(index, 311);
        s.force_non_exceptional(index);
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.FancyElvenArmoire",
            Td::of_number(1044292),
            Td::of_number(1072866),
            None,
            None,
            80.0,
            105.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            60,
            Td::of_number(1044351),
        );
        s.add_recipe(index, 312);
        s.force_non_exceptional(index);
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.SimpleElvenArmoire",
            Td::of_number(1044292),
            Td::of_number(1073401),
            None,
            None,
            80.0,
            105.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            60,
            Td::of_number(1044351),
        );
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.RarewoodChest",
            Td::of_number(1044292),
            Td::of_number(1073402),
            None,
            None,
            80.0,
            105.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            30,
            Td::of_number(1044351),
        );
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.DecorativeBox",
            Td::of_number(1044292),
            Td::of_number(1073403),
            None,
            None,
            80.0,
            105.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            25,
            Td::of_number(1044351),
        );
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.FullBookcase",
            Td::of_number(1044292),
            Td::of_number(1071213),
            None,
            None,
            60.0,
            85.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            25,
            Td::of_number(1044351),
        );
        s.add_res(
            index,
            "Server.Items.AcademicBooksArtifact",
            Td::of_number(1071202),
            1,
            Td::of_number(1044253),
        );
        s.set_needed_expansion(index, Expansion::ML);
    }
    s.add_craft(
        "Server.Items.ShepherdsCrook",
        Td::of_number(if core::ml(w) { 1044566 } else { 1044295 }),
        Td::of_number(1023713),
        None,
        None,
        78.9,
        103.9,
        "Server.Items.Log",
        Td::of_number(1044041),
        7,
        Td::of_number(1044351),
    );
    s.add_craft(
        "Server.Items.QuarterStaff",
        Td::of_number(if core::ml(w) { 1044566 } else { 1044295 }),
        Td::of_number(1023721),
        None,
        None,
        73.6,
        98.6,
        "Server.Items.Log",
        Td::of_number(1044041),
        6,
        Td::of_number(1044351),
    );
    s.add_craft(
        "Server.Items.GnarledStaff",
        Td::of_number(if core::ml(w) { 1044566 } else { 1044295 }),
        Td::of_number(1025112),
        None,
        None,
        78.9,
        103.9,
        "Server.Items.Log",
        Td::of_number(1044041),
        7,
        Td::of_number(1044351),
    );
    if !core::aos(w) {
        index = s.add_craft(
            "Server.Items.FishingPole",
            Td::of_number(if core::ml(w) { 1044294 } else { 1044295 }),
            Td::of_number(1023519),
            None,
            None,
            68.4,
            93.4,
            "Server.Items.Log",
            Td::of_number(1044041),
            5,
            Td::of_number(1044351),
        );
        s.add_skill(index, SkillName::Tailoring, 40.0, 45.0);
        s.add_res(
            index,
            "Server.Items.Cloth",
            Td::of_number(1044286),
            5,
            Td::of_number(1044287),
        );
    }
    if core::se(w) {
        index = s.add_craft(
            "Server.Items.Bokuto",
            Td::of_number(if core::ml(w) { 1044566 } else { 1044295 }),
            Td::of_number(1030227),
            None,
            None,
            70.0,
            95.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            6,
            Td::of_number(1044351),
        );
        s.set_needed_expansion(index, Expansion::SE);
        index = s.add_craft(
            "Server.Items.Fukiya",
            Td::of_number(if core::ml(w) { 1044566 } else { 1044295 }),
            Td::of_number(1030229),
            None,
            None,
            60.0,
            85.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            6,
            Td::of_number(1044351),
        );
        s.set_needed_expansion(index, Expansion::SE);
        index = s.add_craft(
            "Server.Items.Tetsubo",
            Td::of_number(if core::ml(w) { 1044566 } else { 1044295 }),
            Td::of_number(1030225),
            None,
            None,
            80.0,
            140.3,
            "Server.Items.Log",
            Td::of_number(1044041),
            10,
            Td::of_number(1044351),
        );
        s.set_needed_expansion(index, Expansion::SE);
    }
    if core::ml(w) {
        index = s.add_craft(
            "Server.Items.WildStaff",
            Td::of_number(1044566),
            Td::of_number(1031557),
            None,
            None,
            63.8,
            113.8,
            "Server.Items.Log",
            Td::of_number(1044041),
            16,
            Td::of_number(1044351),
        );
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.PhantomStaff",
            Td::of_number(1044566),
            Td::of_number(1072919),
            None,
            None,
            90.0,
            130.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            16,
            Td::of_number(1044351),
        );
        s.add_res(
            index,
            "Server.Items.DiseasedBark",
            Td::of_number(1032683),
            1,
            Td::of_number(1053098),
        );
        s.add_res(
            index,
            "Server.Items.Putrefication",
            Td::of_number(1032678),
            10,
            Td::of_number(1053098),
        );
        s.add_res(
            index,
            "Server.Items.Taint",
            Td::of_number(1032679),
            10,
            Td::of_number(1053098),
        );
        s.add_rare_recipe(index, 313);
        s.force_non_exceptional(index);
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.ArcanistsWildStaff",
            Td::of_number(1044566),
            Td::of_number(1073549),
            None,
            None,
            63.8,
            113.8,
            "Server.Items.Log",
            Td::of_number(1044041),
            16,
            Td::of_number(1044351),
        );
        s.add_res(
            index,
            "Server.Items.WhitePearl",
            Td::of_number(1026253),
            1,
            Td::of_number(1053098),
        );
        s.add_recipe(index, 314);
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.AncientWildStaff",
            Td::of_number(1044566),
            Td::of_number(1073550),
            None,
            None,
            63.8,
            113.8,
            "Server.Items.Log",
            Td::of_number(1044041),
            16,
            Td::of_number(1044351),
        );
        s.add_res(
            index,
            "Server.Items.PerfectEmerald",
            Td::of_number(1026251),
            1,
            Td::of_number(1053098),
        );
        s.add_recipe(index, 315);
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.ThornedWildStaff",
            Td::of_number(1044566),
            Td::of_number(1073551),
            None,
            None,
            63.8,
            113.8,
            "Server.Items.Log",
            Td::of_number(1044041),
            16,
            Td::of_number(1044351),
        );
        s.add_res(
            index,
            "Server.Items.FireRuby",
            Td::of_number(1026254),
            1,
            Td::of_number(1053098),
        );
        s.add_recipe(index, 316);
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.HardenedWildStaff",
            Td::of_number(1044566),
            Td::of_number(1073552),
            None,
            None,
            63.8,
            113.8,
            "Server.Items.Log",
            Td::of_number(1044041),
            16,
            Td::of_number(1044351),
        );
        s.add_res(
            index,
            "Server.Items.Turquoise",
            Td::of_number(1026250),
            1,
            Td::of_number(1053098),
        );
        s.add_recipe(index, 317);
        s.set_needed_expansion(index, Expansion::ML);
    }
    s.add_craft(
        "Server.Items.WoodenShield",
        Td::of_number(if core::ml(w) { 1062760 } else { 1044295 }),
        Td::of_number(1027034),
        None,
        None,
        52.6,
        77.6,
        "Server.Items.Log",
        Td::of_number(1044041),
        9,
        Td::of_number(1044351),
    );
    if core::ml(w) {
        index = s.add_craft(
            "Server.Items.WoodlandChest",
            Td::of_number(1062760),
            Td::of_number(1031111),
            None,
            None,
            90.0,
            115.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            20,
            Td::of_number(1044351),
        );
        s.add_res(
            index,
            "Server.Items.BarkFragment",
            Td::of_number(1032687),
            6,
            Td::of_number(1053098),
        );
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.WoodlandArms",
            Td::of_number(1062760),
            Td::of_number(1031116),
            None,
            None,
            80.0,
            105.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            15,
            Td::of_number(1044351),
        );
        s.add_res(
            index,
            "Server.Items.BarkFragment",
            Td::of_number(1032687),
            4,
            Td::of_number(1053098),
        );
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.WoodlandGloves",
            Td::of_number(1062760),
            Td::of_number(1031114),
            None,
            None,
            85.0,
            110.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            15,
            Td::of_number(1044351),
        );
        s.add_res(
            index,
            "Server.Items.BarkFragment",
            Td::of_number(1032687),
            4,
            Td::of_number(1053098),
        );
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.WoodlandLegs",
            Td::of_number(1062760),
            Td::of_number(1031115),
            None,
            None,
            85.0,
            110.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            15,
            Td::of_number(1044351),
        );
        s.add_res(
            index,
            "Server.Items.BarkFragment",
            Td::of_number(1032687),
            4,
            Td::of_number(1053098),
        );
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.WoodlandGorget",
            Td::of_number(1062760),
            Td::of_number(1031113),
            None,
            None,
            85.0,
            110.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            15,
            Td::of_number(1044351),
        );
        s.add_res(
            index,
            "Server.Items.BarkFragment",
            Td::of_number(1032687),
            4,
            Td::of_number(1053098),
        );
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.RavenHelm",
            Td::of_number(1062760),
            Td::of_number(1031121),
            None,
            None,
            65.0,
            115.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            10,
            Td::of_number(1044351),
        );
        s.add_res(
            index,
            "Server.Items.BarkFragment",
            Td::of_number(1032687),
            4,
            Td::of_number(1053098),
        );
        s.add_res(
            index,
            "Server.Items.Feather",
            Td::of_number(1027123),
            25,
            Td::of_number(1053098),
        );
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.VultureHelm",
            Td::of_number(1062760),
            Td::of_number(1031122),
            None,
            None,
            63.9,
            113.9,
            "Server.Items.Log",
            Td::of_number(1044041),
            10,
            Td::of_number(1044351),
        );
        s.add_res(
            index,
            "Server.Items.BarkFragment",
            Td::of_number(1032687),
            4,
            Td::of_number(1053098),
        );
        s.add_res(
            index,
            "Server.Items.Feather",
            Td::of_number(1027123),
            25,
            Td::of_number(1053098),
        );
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.WingedHelm",
            Td::of_number(1062760),
            Td::of_number(1031123),
            None,
            None,
            58.4,
            108.4,
            "Server.Items.Log",
            Td::of_number(1044041),
            10,
            Td::of_number(1044351),
        );
        s.add_res(
            index,
            "Server.Items.BarkFragment",
            Td::of_number(1032687),
            4,
            Td::of_number(1053098),
        );
        s.add_res(
            index,
            "Server.Items.Feather",
            Td::of_number(1027123),
            60,
            Td::of_number(1053098),
        );
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.IronwoodCrown",
            Td::of_number(1062760),
            Td::of_number(1072924),
            None,
            None,
            85.0,
            120.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            10,
            Td::of_number(1044351),
        );
        s.add_res(
            index,
            "Server.Items.DiseasedBark",
            Td::of_number(1032683),
            1,
            Td::of_number(1053098),
        );
        s.add_res(
            index,
            "Server.Items.Corruption",
            Td::of_number(1032676),
            10,
            Td::of_number(1053098),
        );
        s.add_res(
            index,
            "Server.Items.Putrefication",
            Td::of_number(1032678),
            10,
            Td::of_number(1053098),
        );
        s.add_rare_recipe(index, 318);
        s.force_non_exceptional(index);
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.BrambleCoat",
            Td::of_number(1062760),
            Td::of_number(1072925),
            None,
            None,
            85.0,
            120.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            10,
            Td::of_number(1044351),
        );
        s.add_res(
            index,
            "Server.Items.DiseasedBark",
            Td::of_number(1032683),
            1,
            Td::of_number(1053098),
        );
        s.add_res(
            index,
            "Server.Items.Taint",
            Td::of_number(1032679),
            10,
            Td::of_number(1053098),
        );
        s.add_res(
            index,
            "Server.Items.Scourge",
            Td::of_number(1032677),
            10,
            Td::of_number(1053098),
        );
        s.add_rare_recipe(index, 319);
        s.force_non_exceptional(index);
        s.set_needed_expansion(index, Expansion::ML);
    }
    index = s.add_craft(
        "Server.Items.LapHarp",
        Td::of_number(1044293),
        Td::of_number(1023762),
        None,
        None,
        63.1,
        88.1,
        "Server.Items.Log",
        Td::of_number(1044041),
        20,
        Td::of_number(1044351),
    );
    s.add_skill(index, SkillName::Musicianship, 45.0, 50.0);
    s.add_res(
        index,
        "Server.Items.Cloth",
        Td::of_number(1044286),
        10,
        Td::of_number(1044287),
    );
    index = s.add_craft(
        "Server.Items.Harp",
        Td::of_number(1044293),
        Td::of_number(1023761),
        None,
        None,
        78.9,
        103.9,
        "Server.Items.Log",
        Td::of_number(1044041),
        35,
        Td::of_number(1044351),
    );
    s.add_skill(index, SkillName::Musicianship, 45.0, 50.0);
    s.add_res(
        index,
        "Server.Items.Cloth",
        Td::of_number(1044286),
        15,
        Td::of_number(1044287),
    );
    index = s.add_craft(
        "Server.Items.Drums",
        Td::of_number(1044293),
        Td::of_number(1023740),
        None,
        None,
        57.8,
        82.8,
        "Server.Items.Log",
        Td::of_number(1044041),
        20,
        Td::of_number(1044351),
    );
    s.add_skill(index, SkillName::Musicianship, 45.0, 50.0);
    s.add_res(
        index,
        "Server.Items.Cloth",
        Td::of_number(1044286),
        10,
        Td::of_number(1044287),
    );
    index = s.add_craft(
        "Server.Items.Lute",
        Td::of_number(1044293),
        Td::of_number(1023763),
        None,
        None,
        68.4,
        93.4,
        "Server.Items.Log",
        Td::of_number(1044041),
        25,
        Td::of_number(1044351),
    );
    s.add_skill(index, SkillName::Musicianship, 45.0, 50.0);
    s.add_res(
        index,
        "Server.Items.Cloth",
        Td::of_number(1044286),
        10,
        Td::of_number(1044287),
    );
    index = s.add_craft(
        "Server.Items.Tambourine",
        Td::of_number(1044293),
        Td::of_number(1023741),
        None,
        None,
        57.8,
        82.8,
        "Server.Items.Log",
        Td::of_number(1044041),
        15,
        Td::of_number(1044351),
    );
    s.add_skill(index, SkillName::Musicianship, 45.0, 50.0);
    s.add_res(
        index,
        "Server.Items.Cloth",
        Td::of_number(1044286),
        10,
        Td::of_number(1044287),
    );
    index = s.add_craft(
        "Server.Items.TambourineTassel",
        Td::of_number(1044293),
        Td::of_number(1044320),
        None,
        None,
        57.8,
        82.8,
        "Server.Items.Log",
        Td::of_number(1044041),
        15,
        Td::of_number(1044351),
    );
    s.add_skill(index, SkillName::Musicianship, 45.0, 50.0);
    s.add_res(
        index,
        "Server.Items.Cloth",
        Td::of_number(1044286),
        15,
        Td::of_number(1044287),
    );
    if core::se(w) {
        index = s.add_craft(
            "Server.Items.BambooFlute",
            Td::of_number(1044293),
            Td::of_number(1030247),
            None,
            None,
            80.0,
            105.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            15,
            Td::of_number(1044351),
        );
        s.add_skill(index, SkillName::Musicianship, 45.0, 50.0);
        s.set_needed_expansion(index, Expansion::SE);
    }
    if core::aos(w) {
        s.add_craft(
            "Server.Items.PlayerBBEast",
            Td::of_number(1044290),
            Td::of_number(1062420),
            None,
            None,
            85.0,
            110.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            50,
            Td::of_number(1044351),
        );
        s.add_craft(
            "Server.Items.PlayerBBSouth",
            Td::of_number(1044290),
            Td::of_number(1062421),
            None,
            None,
            85.0,
            110.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            50,
            Td::of_number(1044351),
        );
    }
    if core::ml(w) {
        index = s.add_craft(
            "Server.Items.ParrotPerchDeed",
            Td::of_number(1044290),
            Td::of_number(1072617),
            None,
            None,
            50.0,
            85.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            100,
            Td::of_number(1044351),
        );
        s.force_non_exceptional(index);
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.ArcaneCircleDeed",
            Td::of_number(1044290),
            Td::of_number(1072703),
            None,
            None,
            94.7,
            119.7,
            "Server.Items.Log",
            Td::of_number(1044041),
            100,
            Td::of_number(1044351),
        );
        s.add_res(
            index,
            "Server.Items.BlueDiamond",
            Td::of_number(1026255),
            2,
            Td::of_number(1053098),
        );
        s.add_res(
            index,
            "Server.Items.PerfectEmerald",
            Td::of_number(1026251),
            2,
            Td::of_number(1053098),
        );
        s.add_res(
            index,
            "Server.Items.FireRuby",
            Td::of_number(1026254),
            2,
            Td::of_number(1053098),
        );
        s.force_non_exceptional(index);
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.TallElvenBedSouthDeed",
            Td::of_number(1044290),
            Td::of_number(1072858),
            None,
            None,
            94.7,
            119.7,
            "Server.Items.Log",
            Td::of_number(1044041),
            200,
            Td::of_number(1044351),
        );
        s.add_skill(index, SkillName::Tailoring, 75.0, 80.0);
        s.add_res(
            index,
            "Server.Items.Cloth",
            Td::of_number(1044286),
            100,
            Td::of_number(1044287),
        );
        s.add_recipe(index, 320);
        s.force_non_exceptional(index);
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.TallElvenBedEastDeed",
            Td::of_number(1044290),
            Td::of_number(1072859),
            None,
            None,
            94.7,
            119.7,
            "Server.Items.Log",
            Td::of_number(1044041),
            200,
            Td::of_number(1044351),
        );
        s.add_skill(index, SkillName::Tailoring, 75.0, 80.0);
        s.add_res(
            index,
            "Server.Items.Cloth",
            Td::of_number(1044286),
            100,
            Td::of_number(1044287),
        );
        s.add_recipe(index, 321);
        s.force_non_exceptional(index);
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.ElvenBedSouthDeed",
            Td::of_number(1044290),
            Td::of_number(1072860),
            None,
            None,
            94.7,
            119.7,
            "Server.Items.Log",
            Td::of_number(1044041),
            100,
            Td::of_number(1044351),
        );
        s.add_res(
            index,
            "Server.Items.Cloth",
            Td::of_number(1044286),
            100,
            Td::of_number(1044287),
        );
        s.force_non_exceptional(index);
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.ElvenBedEastDeed",
            Td::of_number(1044290),
            Td::of_number(1072861),
            None,
            None,
            94.7,
            119.7,
            "Server.Items.Log",
            Td::of_number(1044041),
            100,
            Td::of_number(1044351),
        );
        s.add_res(
            index,
            "Server.Items.Cloth",
            Td::of_number(1044286),
            100,
            Td::of_number(1044287),
        );
        s.force_non_exceptional(index);
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.ElvenLoveseatSouthDeed",
            Td::of_number(1044290),
            Td::of_number(1072867),
            None,
            None,
            80.0,
            105.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            50,
            Td::of_number(1044351),
        );
        s.force_non_exceptional(index);
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.ElvenLoveseatEastDeed",
            Td::of_number(1044290),
            Td::of_number(1073372),
            None,
            None,
            80.0,
            105.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            50,
            Td::of_number(1044351),
        );
        s.force_non_exceptional(index);
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.AlchemistTableSouthDeed",
            Td::of_number(1044290),
            Td::of_number(1073396),
            None,
            None,
            85.0,
            110.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            70,
            Td::of_number(1044351),
        );
        s.force_non_exceptional(index);
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.AlchemistTableEastDeed",
            Td::of_number(1044290),
            Td::of_number(1073397),
            None,
            None,
            85.0,
            110.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            70,
            Td::of_number(1044351),
        );
        s.force_non_exceptional(index);
        s.set_needed_expansion(index, Expansion::ML);
    }
    index = s.add_craft(
        "Server.Items.SmallBedSouthDeed",
        Td::of_number(1044290),
        Td::of_number(1044321),
        None,
        None,
        94.7,
        119.8,
        "Server.Items.Log",
        Td::of_number(1044041),
        100,
        Td::of_number(1044351),
    );
    s.add_skill(index, SkillName::Tailoring, 75.0, 80.0);
    s.add_res(
        index,
        "Server.Items.Cloth",
        Td::of_number(1044286),
        100,
        Td::of_number(1044287),
    );
    index = s.add_craft(
        "Server.Items.SmallBedEastDeed",
        Td::of_number(1044290),
        Td::of_number(1044322),
        None,
        None,
        94.7,
        119.8,
        "Server.Items.Log",
        Td::of_number(1044041),
        100,
        Td::of_number(1044351),
    );
    s.add_skill(index, SkillName::Tailoring, 75.0, 80.0);
    s.add_res(
        index,
        "Server.Items.Cloth",
        Td::of_number(1044286),
        100,
        Td::of_number(1044287),
    );
    index = s.add_craft(
        "Server.Items.LargeBedSouthDeed",
        Td::of_number(1044290),
        Td::of_number(1044323),
        None,
        None,
        94.7,
        119.8,
        "Server.Items.Log",
        Td::of_number(1044041),
        150,
        Td::of_number(1044351),
    );
    s.add_skill(index, SkillName::Tailoring, 75.0, 80.0);
    s.add_res(
        index,
        "Server.Items.Cloth",
        Td::of_number(1044286),
        150,
        Td::of_number(1044287),
    );
    index = s.add_craft(
        "Server.Items.LargeBedEastDeed",
        Td::of_number(1044290),
        Td::of_number(1044324),
        None,
        None,
        94.7,
        119.8,
        "Server.Items.Log",
        Td::of_number(1044041),
        150,
        Td::of_number(1044351),
    );
    s.add_skill(index, SkillName::Tailoring, 75.0, 80.0);
    s.add_res(
        index,
        "Server.Items.Cloth",
        Td::of_number(1044286),
        150,
        Td::of_number(1044287),
    );
    s.add_craft(
        "Server.Items.DartBoardSouthDeed",
        Td::of_number(1044290),
        Td::of_number(1044325),
        None,
        None,
        15.7,
        40.7,
        "Server.Items.Log",
        Td::of_number(1044041),
        5,
        Td::of_number(1044351),
    );
    s.add_craft(
        "Server.Items.DartBoardEastDeed",
        Td::of_number(1044290),
        Td::of_number(1044326),
        None,
        None,
        15.7,
        40.7,
        "Server.Items.Log",
        Td::of_number(1044041),
        5,
        Td::of_number(1044351),
    );
    s.add_craft(
        "Server.Items.BallotBoxDeed",
        Td::of_number(1044290),
        Td::of_number(1044327),
        None,
        None,
        47.3,
        72.3,
        "Server.Items.Log",
        Td::of_number(1044041),
        5,
        Td::of_number(1044351),
    );
    index = s.add_craft(
        "Server.Items.PentagramDeed",
        Td::of_number(1044290),
        Td::of_number(1044328),
        None,
        None,
        100.0,
        125.0,
        "Server.Items.Log",
        Td::of_number(1044041),
        100,
        Td::of_number(1044351),
    );
    s.add_skill(index, SkillName::Magery, 75.0, 80.0);
    s.add_res(
        index,
        "Server.Items.IronIngot",
        Td::of_number(1044036),
        40,
        Td::of_number(1044037),
    );
    index = s.add_craft(
        "Server.Items.AbbatoirDeed",
        Td::of_number(1044290),
        Td::of_number(1044329),
        None,
        None,
        100.0,
        125.0,
        "Server.Items.Log",
        Td::of_number(1044041),
        100,
        Td::of_number(1044351),
    );
    s.add_skill(index, SkillName::Magery, 50.0, 55.0);
    s.add_res(
        index,
        "Server.Items.IronIngot",
        Td::of_number(1044036),
        40,
        Td::of_number(1044037),
    );
    if core::ml(w) {
        index = s.add_craft(
            "Server.Items.ElvenForgeDeed",
            Td::of_number(1044296),
            Td::of_number(1072875),
            None,
            None,
            94.7,
            119.7,
            "Server.Items.Log",
            Td::of_number(1044041),
            200,
            Td::of_number(1044351),
        );
        s.force_non_exceptional(index);
        s.set_needed_expansion(index, Expansion::ML);
    }
    index = s.add_craft(
        "Server.Items.SmallForgeDeed",
        Td::of_number(1044296),
        Td::of_number(1044330),
        None,
        None,
        73.6,
        98.6,
        "Server.Items.Log",
        Td::of_number(1044041),
        5,
        Td::of_number(1044351),
    );
    s.add_skill(index, SkillName::Blacksmith, 75.0, 80.0);
    s.add_res(
        index,
        "Server.Items.IronIngot",
        Td::of_number(1044036),
        75,
        Td::of_number(1044037),
    );
    index = s.add_craft(
        "Server.Items.LargeForgeEastDeed",
        Td::of_number(1044296),
        Td::of_number(1044331),
        None,
        None,
        78.9,
        103.9,
        "Server.Items.Log",
        Td::of_number(1044041),
        5,
        Td::of_number(1044351),
    );
    s.add_skill(index, SkillName::Blacksmith, 80.0, 85.0);
    s.add_res(
        index,
        "Server.Items.IronIngot",
        Td::of_number(1044036),
        100,
        Td::of_number(1044037),
    );
    index = s.add_craft(
        "Server.Items.LargeForgeSouthDeed",
        Td::of_number(1044296),
        Td::of_number(1044332),
        None,
        None,
        78.9,
        103.9,
        "Server.Items.Log",
        Td::of_number(1044041),
        5,
        Td::of_number(1044351),
    );
    s.add_skill(index, SkillName::Blacksmith, 80.0, 85.0);
    s.add_res(
        index,
        "Server.Items.IronIngot",
        Td::of_number(1044036),
        100,
        Td::of_number(1044037),
    );
    index = s.add_craft(
        "Server.Items.AnvilEastDeed",
        Td::of_number(1044296),
        Td::of_number(1044333),
        None,
        None,
        73.6,
        98.6,
        "Server.Items.Log",
        Td::of_number(1044041),
        5,
        Td::of_number(1044351),
    );
    s.add_skill(index, SkillName::Blacksmith, 75.0, 80.0);
    s.add_res(
        index,
        "Server.Items.IronIngot",
        Td::of_number(1044036),
        150,
        Td::of_number(1044037),
    );
    index = s.add_craft(
        "Server.Items.AnvilSouthDeed",
        Td::of_number(1044296),
        Td::of_number(1044334),
        None,
        None,
        73.6,
        98.6,
        "Server.Items.Log",
        Td::of_number(1044041),
        5,
        Td::of_number(1044351),
    );
    s.add_skill(index, SkillName::Blacksmith, 75.0, 80.0);
    s.add_res(
        index,
        "Server.Items.IronIngot",
        Td::of_number(1044036),
        150,
        Td::of_number(1044037),
    );
    index = s.add_craft(
        "Server.Items.TrainingDummyEastDeed",
        Td::of_number(1044297),
        Td::of_number(1044335),
        None,
        None,
        68.4,
        93.4,
        "Server.Items.Log",
        Td::of_number(1044041),
        55,
        Td::of_number(1044351),
    );
    s.add_skill(index, SkillName::Tailoring, 50.0, 55.0);
    s.add_res(
        index,
        "Server.Items.Cloth",
        Td::of_number(1044286),
        60,
        Td::of_number(1044287),
    );
    index = s.add_craft(
        "Server.Items.TrainingDummySouthDeed",
        Td::of_number(1044297),
        Td::of_number(1044336),
        None,
        None,
        68.4,
        93.4,
        "Server.Items.Log",
        Td::of_number(1044041),
        55,
        Td::of_number(1044351),
    );
    s.add_skill(index, SkillName::Tailoring, 50.0, 55.0);
    s.add_res(
        index,
        "Server.Items.Cloth",
        Td::of_number(1044286),
        60,
        Td::of_number(1044287),
    );
    index = s.add_craft(
        "Server.Items.PickpocketDipEastDeed",
        Td::of_number(1044297),
        Td::of_number(1044337),
        None,
        None,
        73.6,
        98.6,
        "Server.Items.Log",
        Td::of_number(1044041),
        65,
        Td::of_number(1044351),
    );
    s.add_skill(index, SkillName::Tailoring, 50.0, 55.0);
    s.add_res(
        index,
        "Server.Items.Cloth",
        Td::of_number(1044286),
        60,
        Td::of_number(1044287),
    );
    index = s.add_craft(
        "Server.Items.PickpocketDipSouthDeed",
        Td::of_number(1044297),
        Td::of_number(1044338),
        None,
        None,
        73.6,
        98.6,
        "Server.Items.Log",
        Td::of_number(1044041),
        65,
        Td::of_number(1044351),
    );
    s.add_skill(index, SkillName::Tailoring, 50.0, 55.0);
    s.add_res(
        index,
        "Server.Items.Cloth",
        Td::of_number(1044286),
        60,
        Td::of_number(1044287),
    );
    index = s.add_craft(
        "Server.Items.Dressform",
        Td::of_number(1044298),
        Td::of_number(1044339),
        None,
        None,
        63.1,
        88.1,
        "Server.Items.Log",
        Td::of_number(1044041),
        25,
        Td::of_number(1044351),
    );
    s.add_skill(index, SkillName::Tailoring, 65.0, 70.0);
    s.add_res(
        index,
        "Server.Items.Cloth",
        Td::of_number(1044286),
        10,
        Td::of_number(1044287),
    );
    if core::ml(w) {
        index = s.add_craft(
            "Server.Items.ElvenSpinningWheelEastDeed",
            Td::of_number(1044298),
            Td::of_number(1073393),
            None,
            None,
            75.0,
            100.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            60,
            Td::of_number(1044351),
        );
        s.add_skill(index, SkillName::Tailoring, 65.0, 85.0);
        s.add_res(
            index,
            "Server.Items.Cloth",
            Td::of_number(1044286),
            40,
            Td::of_number(1044287),
        );
        s.force_non_exceptional(index);
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.ElvenSpinningWheelSouthDeed",
            Td::of_number(1044298),
            Td::of_number(1072878),
            None,
            None,
            75.0,
            100.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            60,
            Td::of_number(1044351),
        );
        s.add_skill(index, SkillName::Tailoring, 65.0, 85.0);
        s.add_res(
            index,
            "Server.Items.Cloth",
            Td::of_number(1044286),
            40,
            Td::of_number(1044287),
        );
        s.force_non_exceptional(index);
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.ElvenStoveSouthDeed",
            Td::of_number(1044298),
            Td::of_number(1073394),
            None,
            None,
            85.0,
            110.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            80,
            Td::of_number(1044351),
        );
        s.force_non_exceptional(index);
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.ElvenStoveEastDeed",
            Td::of_number(1044298),
            Td::of_number(1073395),
            None,
            None,
            85.0,
            110.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            80,
            Td::of_number(1044351),
        );
        s.force_non_exceptional(index);
        s.set_needed_expansion(index, Expansion::ML);
    }
    index = s.add_craft(
        "Server.Items.SpinningWheelEastDeed",
        Td::of_number(1044298),
        Td::of_number(1044341),
        None,
        None,
        73.6,
        98.6,
        "Server.Items.Log",
        Td::of_number(1044041),
        75,
        Td::of_number(1044351),
    );
    s.add_skill(index, SkillName::Tailoring, 65.0, 70.0);
    s.add_res(
        index,
        "Server.Items.Cloth",
        Td::of_number(1044286),
        25,
        Td::of_number(1044287),
    );
    index = s.add_craft(
        "Server.Items.SpinningWheelSouthDeed",
        Td::of_number(1044298),
        Td::of_number(1044342),
        None,
        None,
        73.6,
        98.6,
        "Server.Items.Log",
        Td::of_number(1044041),
        75,
        Td::of_number(1044351),
    );
    s.add_skill(index, SkillName::Tailoring, 65.0, 70.0);
    s.add_res(
        index,
        "Server.Items.Cloth",
        Td::of_number(1044286),
        25,
        Td::of_number(1044287),
    );
    index = s.add_craft(
        "Server.Items.LoomEastDeed",
        Td::of_number(1044298),
        Td::of_number(1044343),
        None,
        None,
        84.2,
        109.2,
        "Server.Items.Log",
        Td::of_number(1044041),
        85,
        Td::of_number(1044351),
    );
    s.add_skill(index, SkillName::Tailoring, 65.0, 70.0);
    s.add_res(
        index,
        "Server.Items.Cloth",
        Td::of_number(1044286),
        25,
        Td::of_number(1044287),
    );
    index = s.add_craft(
        "Server.Items.LoomSouthDeed",
        Td::of_number(1044298),
        Td::of_number(1044344),
        None,
        None,
        84.2,
        109.2,
        "Server.Items.Log",
        Td::of_number(1044041),
        85,
        Td::of_number(1044351),
    );
    s.add_skill(index, SkillName::Tailoring, 65.0, 70.0);
    s.add_res(
        index,
        "Server.Items.Cloth",
        Td::of_number(1044286),
        25,
        Td::of_number(1044287),
    );
    index = s.add_craft(
        "Server.Items.StoneOvenEastDeed",
        Td::of_number(if core::ml(w) { 1044298 } else { 1044299 }),
        Td::of_number(1044345),
        None,
        None,
        68.4,
        93.4,
        "Server.Items.Log",
        Td::of_number(1044041),
        85,
        Td::of_number(1044351),
    );
    s.add_skill(index, SkillName::Tinkering, 50.0, 55.0);
    s.add_res(
        index,
        "Server.Items.IronIngot",
        Td::of_number(1044036),
        125,
        Td::of_number(1044037),
    );
    index = s.add_craft(
        "Server.Items.StoneOvenSouthDeed",
        Td::of_number(if core::ml(w) { 1044298 } else { 1044299 }),
        Td::of_number(1044346),
        None,
        None,
        68.4,
        93.4,
        "Server.Items.Log",
        Td::of_number(1044041),
        85,
        Td::of_number(1044351),
    );
    s.add_skill(index, SkillName::Tinkering, 50.0, 55.0);
    s.add_res(
        index,
        "Server.Items.IronIngot",
        Td::of_number(1044036),
        125,
        Td::of_number(1044037),
    );
    index = s.add_craft(
        "Server.Items.FlourMillEastDeed",
        Td::of_number(if core::ml(w) { 1044298 } else { 1044299 }),
        Td::of_number(1044347),
        None,
        None,
        94.7,
        119.7,
        "Server.Items.Log",
        Td::of_number(1044041),
        100,
        Td::of_number(1044351),
    );
    s.add_skill(index, SkillName::Tinkering, 50.0, 55.0);
    s.add_res(
        index,
        "Server.Items.IronIngot",
        Td::of_number(1044036),
        50,
        Td::of_number(1044037),
    );
    index = s.add_craft(
        "Server.Items.FlourMillSouthDeed",
        Td::of_number(if core::ml(w) { 1044298 } else { 1044299 }),
        Td::of_number(1044348),
        None,
        None,
        94.7,
        119.7,
        "Server.Items.Log",
        Td::of_number(1044041),
        100,
        Td::of_number(1044351),
    );
    s.add_skill(index, SkillName::Tinkering, 50.0, 55.0);
    s.add_res(
        index,
        "Server.Items.IronIngot",
        Td::of_number(1044036),
        50,
        Td::of_number(1044037),
    );
    s.add_craft(
        "Server.Items.WaterTroughEastDeed",
        Td::of_number(if core::ml(w) { 1044298 } else { 1044299 }),
        Td::of_number(1044349),
        None,
        None,
        94.7,
        119.7,
        "Server.Items.Log",
        Td::of_number(1044041),
        150,
        Td::of_number(1044351),
    );
    s.add_craft(
        "Server.Items.WaterTroughSouthDeed",
        Td::of_number(if core::ml(w) { 1044298 } else { 1044299 }),
        Td::of_number(1044350),
        None,
        None,
        94.7,
        119.7,
        "Server.Items.Log",
        Td::of_number(1044041),
        150,
        Td::of_number(1044351),
    );
    s.mark_option = true;
    s.repair = core::aos(w);
    s.set_sub_res("Server.Items.Log", Td::of_number(1072643));
    s.add_sub_res(
        "Server.Items.Log",
        Td::of_number(1072643),
        00.0,
        1044041,
        Td::of_number(1072652),
    );
    s.add_sub_res(
        "Server.Items.OakLog",
        Td::of_number(1072644),
        65.0,
        1044041,
        Td::of_number(1072652),
    );
    s.add_sub_res(
        "Server.Items.AshLog",
        Td::of_number(1072645),
        80.0,
        1044041,
        Td::of_number(1072652),
    );
    s.add_sub_res(
        "Server.Items.YewLog",
        Td::of_number(1072646),
        95.0,
        1044041,
        Td::of_number(1072652),
    );
    s.add_sub_res(
        "Server.Items.HeartwoodLog",
        Td::of_number(1072647),
        100.0,
        1044041,
        Td::of_number(1072652),
    );
    s.add_sub_res(
        "Server.Items.BloodwoodLog",
        Td::of_number(1072648),
        100.0,
        1044041,
        Td::of_number(1072652),
    );
    s.add_sub_res(
        "Server.Items.FrostwoodLog",
        Td::of_number(1072649),
        100.0,
        1044041,
        Td::of_number(1072652),
    );
}
