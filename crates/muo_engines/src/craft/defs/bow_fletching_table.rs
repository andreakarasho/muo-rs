//! The BowFletching craft list (`InitCraftList`), generated: do not edit by hand.

use crate::craft::model::*;
use muo_content::core;
use muo_data::expansion::Expansion;
use muo_io::text::TextDefinition as Td;
use muo_server::World;

/// `InitCraftList()`.
pub fn init(w: &World, s: &mut CraftSystem) {
    let mut index;
    s.add_craft(
        "Server.Items.Kindling",
        Td::of_number(1044457),
        Td::of_number(1023553),
        None,
        None,
        0.0,
        00.0,
        "Server.Items.Log",
        Td::of_number(1044041),
        1,
        Td::of_number(1044351),
    );
    index = s.add_craft(
        "Server.Items.Shaft",
        Td::of_number(1044457),
        Td::of_number(1027124),
        None,
        None,
        0.0,
        40.0,
        "Server.Items.Log",
        Td::of_number(1044041),
        1,
        Td::of_number(1044351),
    );
    s.set_use_all_res(index, true);
    index = s.add_craft(
        "Server.Items.Arrow",
        Td::of_number(1044565),
        Td::of_number(1023903),
        None,
        None,
        0.0,
        40.0,
        "Server.Items.Shaft",
        Td::of_number(1044560),
        1,
        Td::of_number(1044561),
    );
    s.add_res(
        index,
        "Server.Items.Feather",
        Td::of_number(1044562),
        1,
        Td::of_number(1044563),
    );
    s.set_use_all_res(index, true);
    index = s.add_craft(
        "Server.Items.Bolt",
        Td::of_number(1044565),
        Td::of_number(1027163),
        None,
        None,
        0.0,
        40.0,
        "Server.Items.Shaft",
        Td::of_number(1044560),
        1,
        Td::of_number(1044561),
    );
    s.add_res(
        index,
        "Server.Items.Feather",
        Td::of_number(1044562),
        1,
        Td::of_number(1044563),
    );
    s.set_use_all_res(index, true);
    if core::se(w) {
        index = s.add_craft(
            "Server.Items.FukiyaDarts",
            Td::of_number(1044565),
            Td::of_number(1030246),
            None,
            None,
            50.0,
            90.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            1,
            Td::of_number(1044351),
        );
        s.set_use_all_res(index, true);
        s.set_needed_expansion(index, Expansion::SE);
    }
    s.add_craft(
        "Server.Items.Bow",
        Td::of_number(1044566),
        Td::of_number(1025042),
        None,
        None,
        30.0,
        70.0,
        "Server.Items.Log",
        Td::of_number(1044041),
        7,
        Td::of_number(1044351),
    );
    s.add_craft(
        "Server.Items.Crossbow",
        Td::of_number(1044566),
        Td::of_number(1023919),
        None,
        None,
        60.0,
        100.0,
        "Server.Items.Log",
        Td::of_number(1044041),
        7,
        Td::of_number(1044351),
    );
    s.add_craft(
        "Server.Items.HeavyCrossbow",
        Td::of_number(1044566),
        Td::of_number(1025117),
        None,
        None,
        80.0,
        120.0,
        "Server.Items.Log",
        Td::of_number(1044041),
        10,
        Td::of_number(1044351),
    );
    if core::aos(w) {
        s.add_craft(
            "Server.Items.CompositeBow",
            Td::of_number(1044566),
            Td::of_number(1029922),
            None,
            None,
            70.0,
            110.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            7,
            Td::of_number(1044351),
        );
        s.add_craft(
            "Server.Items.RepeatingCrossbow",
            Td::of_number(1044566),
            Td::of_number(1029923),
            None,
            None,
            90.0,
            130.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            10,
            Td::of_number(1044351),
        );
    }
    if core::se(w) {
        index = s.add_craft(
            "Server.Items.Yumi",
            Td::of_number(1044566),
            Td::of_number(1030224),
            None,
            None,
            90.0,
            130.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            10,
            Td::of_number(1044351),
        );
        s.set_needed_expansion(index, Expansion::SE);
    }
    if core::ml(w) {
        index = s.add_craft(
            "Server.Items.BlightGrippedLongbow",
            Td::of_number(1044566),
            Td::of_number(1072907),
            None,
            None,
            75.0,
            125.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            20,
            Td::of_number(1044351),
        );
        s.add_res(
            index,
            "Server.Items.LardOfParoxysmus",
            Td::of_number(1032681),
            1,
            Td::of_number(1053098),
        );
        s.add_res(
            index,
            "Server.Items.Blight",
            Td::of_number(1032675),
            10,
            Td::of_number(1053098),
        );
        s.add_res(
            index,
            "Server.Items.Corruption",
            Td::of_number(1032676),
            10,
            Td::of_number(1053098),
        );
        s.add_rare_recipe(index, 200);
        s.force_non_exceptional(index);
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.SilvanisFeywoodBow",
            Td::of_number(1044566),
            Td::of_number(1072955),
            None,
            None,
            75.0,
            125.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            20,
            Td::of_number(1044351),
        );
        s.add_res(
            index,
            "Server.Items.LardOfParoxysmus",
            Td::of_number(1032681),
            1,
            Td::of_number(1053098),
        );
        s.add_res(
            index,
            "Server.Items.Scourge",
            Td::of_number(1032677),
            10,
            Td::of_number(1053098),
        );
        s.add_res(
            index,
            "Server.Items.Muculent",
            Td::of_number(1032680),
            10,
            Td::of_number(1053098),
        );
        s.add_rare_recipe(index, 202);
        s.force_non_exceptional(index);
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.TheNightReaper",
            Td::of_number(1044566),
            Td::of_number(1072912),
            None,
            None,
            75.0,
            125.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            10,
            Td::of_number(1044351),
        );
        s.add_res(
            index,
            "Server.Items.DreadHornMane",
            Td::of_number(1032682),
            1,
            Td::of_number(1053098),
        );
        s.add_res(
            index,
            "Server.Items.Blight",
            Td::of_number(1032675),
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
        s.add_rare_recipe(index, 204);
        s.force_non_exceptional(index);
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.BarbedLongbow",
            Td::of_number(1044566),
            Td::of_number(1073505),
            None,
            None,
            75.0,
            125.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            20,
            Td::of_number(1044351),
        );
        s.add_res(
            index,
            "Server.Items.FireRuby",
            Td::of_number(1026254),
            1,
            Td::of_number(1053098),
        );
        s.add_recipe(index, 205);
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.SlayerLongbow",
            Td::of_number(1044566),
            Td::of_number(1073506),
            None,
            None,
            75.0,
            125.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            20,
            Td::of_number(1044351),
        );
        s.add_res(
            index,
            "Server.Items.BrilliantAmber",
            Td::of_number(1026256),
            1,
            Td::of_number(1053098),
        );
        s.add_recipe(index, 206);
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.FrozenLongbow",
            Td::of_number(1044566),
            Td::of_number(1073507),
            None,
            None,
            75.0,
            125.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            20,
            Td::of_number(1044351),
        );
        s.add_res(
            index,
            "Server.Items.Turquoise",
            Td::of_number(1026250),
            1,
            Td::of_number(1053098),
        );
        s.add_recipe(index, 207);
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.LongbowOfMight",
            Td::of_number(1044566),
            Td::of_number(1073508),
            None,
            None,
            75.0,
            125.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            10,
            Td::of_number(1044351),
        );
        s.add_res(
            index,
            "Server.Items.BlueDiamond",
            Td::of_number(1026255),
            1,
            Td::of_number(1053098),
        );
        s.add_recipe(index, 208);
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.RangersShortbow",
            Td::of_number(1044566),
            Td::of_number(1073509),
            None,
            None,
            75.0,
            125.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            15,
            Td::of_number(1044351),
        );
        s.add_res(
            index,
            "Server.Items.PerfectEmerald",
            Td::of_number(1026251),
            1,
            Td::of_number(1053098),
        );
        s.add_recipe(index, 209);
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.LightweightShortbow",
            Td::of_number(1044566),
            Td::of_number(1073510),
            None,
            None,
            75.0,
            125.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            15,
            Td::of_number(1044351),
        );
        s.add_res(
            index,
            "Server.Items.WhitePearl",
            Td::of_number(1026253),
            1,
            Td::of_number(1053098),
        );
        s.add_recipe(index, 210);
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.MysticalShortbow",
            Td::of_number(1044566),
            Td::of_number(1073511),
            None,
            None,
            75.0,
            125.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            15,
            Td::of_number(1044351),
        );
        s.add_res(
            index,
            "Server.Items.EcruCitrine",
            Td::of_number(1026252),
            1,
            Td::of_number(1053098),
        );
        s.add_recipe(index, 211);
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.AssassinsShortbow",
            Td::of_number(1044566),
            Td::of_number(1073512),
            None,
            None,
            75.0,
            125.0,
            "Server.Items.Log",
            Td::of_number(1044041),
            15,
            Td::of_number(1044351),
        );
        s.add_res(
            index,
            "Server.Items.DarkSapphire",
            Td::of_number(1026249),
            1,
            Td::of_number(1053098),
        );
        s.add_recipe(index, 212);
        s.set_needed_expansion(index, Expansion::ML);
    }
    s.mark_option = true;
    s.repair = core::aos(w);
    s.set_sub_res("Server.Items.Log", Td::of_number(1072643));
    s.add_sub_res(
        "Server.Items.Log",
        Td::of_number(1072643),
        0.0,
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
