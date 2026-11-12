//! The Cooking craft list (`InitCraftList`), generated: do not edit by hand.

use crate::craft::model::*;
use muo_content::core;
use muo_data::expansion::Expansion;
use muo_io::text::TextDefinition as Td;
use muo_server::World;

/// `InitCraftList()`.
pub fn init(w: &World, s: &mut CraftSystem) {
    let mut index;
    index = s.add_craft(
        "Server.Items.SackFlour",
        Td::of_number(1044495),
        Td::of_number(1024153),
        None,
        None,
        0.0,
        100.0,
        "Server.Items.WheatSheaf",
        Td::of_number(1044489),
        2,
        Td::of_number(1044490),
    );
    s.set_need_mill(index, true);
    index = s.add_craft(
        "Server.Items.Dough",
        Td::of_number(1044495),
        Td::of_number(1024157),
        None,
        None,
        0.0,
        100.0,
        "Server.Items.SackFlour",
        Td::of_number(1044468),
        1,
        Td::of_number(1044253),
    );
    s.add_res(
        index,
        "Server.Items.BaseBeverage",
        Td::of_number(1046458),
        1,
        Td::of_number(1044253),
    );
    index = s.add_craft(
        "Server.Items.SweetDough",
        Td::of_number(1044495),
        Td::of_number(1041340),
        None,
        None,
        0.0,
        100.0,
        "Server.Items.Dough",
        Td::of_number(1044469),
        1,
        Td::of_number(1044253),
    );
    s.add_res(
        index,
        "Server.Items.JarHoney",
        Td::of_number(1044472),
        1,
        Td::of_number(1044253),
    );
    index = s.add_craft(
        "Server.Items.CakeMix",
        Td::of_number(1044495),
        Td::of_number(1041002),
        None,
        None,
        0.0,
        100.0,
        "Server.Items.SackFlour",
        Td::of_number(1044468),
        1,
        Td::of_number(1044253),
    );
    s.add_res(
        index,
        "Server.Items.SweetDough",
        Td::of_number(1044475),
        1,
        Td::of_number(1044253),
    );
    index = s.add_craft(
        "Server.Items.CookieMix",
        Td::of_number(1044495),
        Td::of_number(1024159),
        None,
        None,
        0.0,
        100.0,
        "Server.Items.JarHoney",
        Td::of_number(1044472),
        1,
        Td::of_number(1044253),
    );
    s.add_res(
        index,
        "Server.Items.SweetDough",
        Td::of_number(1044475),
        1,
        Td::of_number(1044253),
    );
    if core::ml(w) {
        index = s.add_craft(
            "Server.Items.CocoaButter",
            Td::of_number(1044495),
            Td::of_number(1079998),
            None,
            None,
            0.0,
            100.0,
            "Server.Items.CocoaPulp",
            Td::of_number(1080530),
            1,
            Td::of_number(1044253),
        );
        s.set_item_hue(index, 0x457);
        s.set_needed_expansion(index, Expansion::ML);
        s.set_need_oven(index, true);
        index = s.add_craft(
            "Server.Items.CocoaLiquor",
            Td::of_number(1044495),
            Td::of_number(1079999),
            None,
            None,
            0.0,
            100.0,
            "Server.Items.CocoaPulp",
            Td::of_number(1080530),
            1,
            Td::of_number(1044253),
        );
        s.add_res(
            index,
            "Server.Items.EmptyPewterBowl",
            Td::of_number(1025629),
            1,
            Td::of_number(1044253),
        );
        s.set_item_hue(index, 0x46A);
        s.set_needed_expansion(index, Expansion::ML);
        s.set_need_oven(index, true);
    }
    index = s.add_craft(
        "Server.Items.UnbakedQuiche",
        Td::of_number(1044496),
        Td::of_number(1041339),
        None,
        None,
        0.0,
        100.0,
        "Server.Items.Dough",
        Td::of_number(1044469),
        1,
        Td::of_number(1044253),
    );
    s.add_res(
        index,
        "Server.Items.Eggs",
        Td::of_number(1044477),
        1,
        Td::of_number(1044253),
    );
    index = s.add_craft(
        "Server.Items.UnbakedMeatPie",
        Td::of_number(1044496),
        Td::of_number(1041338),
        None,
        None,
        0.0,
        100.0,
        "Server.Items.Dough",
        Td::of_number(1044469),
        1,
        Td::of_number(1044253),
    );
    s.add_res(
        index,
        "Server.Items.RawRibs",
        Td::of_number(1044482),
        1,
        Td::of_number(1044253),
    );
    index = s.add_craft(
        "Server.Items.UncookedSausagePizza",
        Td::of_number(1044496),
        Td::of_number(1041337),
        None,
        None,
        0.0,
        100.0,
        "Server.Items.Dough",
        Td::of_number(1044469),
        1,
        Td::of_number(1044253),
    );
    s.add_res(
        index,
        "Server.Items.Sausage",
        Td::of_number(1044483),
        1,
        Td::of_number(1044253),
    );
    index = s.add_craft(
        "Server.Items.UncookedCheesePizza",
        Td::of_number(1044496),
        Td::of_number(1041341),
        None,
        None,
        0.0,
        100.0,
        "Server.Items.Dough",
        Td::of_number(1044469),
        1,
        Td::of_number(1044253),
    );
    s.add_res(
        index,
        "Server.Items.CheeseWheel",
        Td::of_number(1044486),
        1,
        Td::of_number(1044253),
    );
    index = s.add_craft(
        "Server.Items.UnbakedFruitPie",
        Td::of_number(1044496),
        Td::of_number(1041334),
        None,
        None,
        0.0,
        100.0,
        "Server.Items.Dough",
        Td::of_number(1044469),
        1,
        Td::of_number(1044253),
    );
    s.add_res(
        index,
        "Server.Items.Pear",
        Td::of_number(1044481),
        1,
        Td::of_number(1044253),
    );
    index = s.add_craft(
        "Server.Items.UnbakedPeachCobbler",
        Td::of_number(1044496),
        Td::of_number(1041335),
        None,
        None,
        0.0,
        100.0,
        "Server.Items.Dough",
        Td::of_number(1044469),
        1,
        Td::of_number(1044253),
    );
    s.add_res(
        index,
        "Server.Items.Peach",
        Td::of_number(1044480),
        1,
        Td::of_number(1044253),
    );
    index = s.add_craft(
        "Server.Items.UnbakedApplePie",
        Td::of_number(1044496),
        Td::of_number(1041336),
        None,
        None,
        0.0,
        100.0,
        "Server.Items.Dough",
        Td::of_number(1044469),
        1,
        Td::of_number(1044253),
    );
    s.add_res(
        index,
        "Server.Items.Apple",
        Td::of_number(1044479),
        1,
        Td::of_number(1044253),
    );
    index = s.add_craft(
        "Server.Items.UnbakedPumpkinPie",
        Td::of_number(1044496),
        Td::of_number(1041342),
        None,
        None,
        0.0,
        100.0,
        "Server.Items.Dough",
        Td::of_number(1044469),
        1,
        Td::of_number(1044253),
    );
    s.add_res(
        index,
        "Server.Items.Pumpkin",
        Td::of_number(1044484),
        1,
        Td::of_number(1044253),
    );
    if core::se(w) {
        index = s.add_craft(
            "Server.Items.GreenTea",
            Td::of_number(1044496),
            Td::of_number(1030315),
            None,
            None,
            80.0,
            130.0,
            "Server.Items.GreenTeaBasket",
            Td::of_number(1030316),
            1,
            Td::of_number(1044253),
        );
        s.add_res(
            index,
            "Server.Items.BaseBeverage",
            Td::of_number(1046458),
            1,
            Td::of_number(1044253),
        );
        s.set_needed_expansion(index, Expansion::SE);
        s.set_need_oven(index, true);
        index = s.add_craft(
            "Server.Items.WasabiClumps",
            Td::of_number(1044496),
            Td::of_number(1029451),
            None,
            None,
            70.0,
            120.0,
            "Server.Items.BaseBeverage",
            Td::of_number(1046458),
            1,
            Td::of_number(1044253),
        );
        s.add_res(
            index,
            "Server.Items.WoodenBowlOfPeas",
            Td::of_number(1025633),
            3,
            Td::of_number(1044253),
        );
        s.set_needed_expansion(index, Expansion::SE);
        index = s.add_craft(
            "Server.Items.SushiRolls",
            Td::of_number(1044496),
            Td::of_number(1030303),
            None,
            None,
            90.0,
            120.0,
            "Server.Items.BaseBeverage",
            Td::of_number(1046458),
            1,
            Td::of_number(1044253),
        );
        s.add_res(
            index,
            "Server.Items.RawFishSteak",
            Td::of_number(1044476),
            10,
            Td::of_number(1044253),
        );
        s.set_needed_expansion(index, Expansion::SE);
        index = s.add_craft(
            "Server.Items.SushiPlatter",
            Td::of_number(1044496),
            Td::of_number(1030305),
            None,
            None,
            90.0,
            120.0,
            "Server.Items.BaseBeverage",
            Td::of_number(1046458),
            1,
            Td::of_number(1044253),
        );
        s.add_res(
            index,
            "Server.Items.RawFishSteak",
            Td::of_number(1044476),
            10,
            Td::of_number(1044253),
        );
        s.set_needed_expansion(index, Expansion::SE);
    }
    index = s.add_craft(
        "Server.Items.TribalPaint",
        Td::of_number(1044496),
        Td::of_number(1040000),
        None,
        None,
        if core::ml(w) { 55.0 } else { 80.0 },
        if core::ml(w) { 105.0 } else { 80.0 },
        "Server.Items.SackFlour",
        Td::of_number(1044468),
        1,
        Td::of_number(1044253),
    );
    s.add_res(
        index,
        "Server.Items.TribalBerry",
        Td::of_number(1046460),
        1,
        Td::of_number(1044253),
    );
    if core::se(w) {
        index = s.add_craft(
            "Server.Items.EggBomb",
            Td::of_number(1044496),
            Td::of_number(1030249),
            None,
            None,
            90.0,
            120.0,
            "Server.Items.Eggs",
            Td::of_number(1044477),
            1,
            Td::of_number(1044253),
        );
        s.add_res(
            index,
            "Server.Items.SackFlour",
            Td::of_number(1044468),
            3,
            Td::of_number(1044253),
        );
        s.set_needed_expansion(index, Expansion::SE);
    }
    index = s.add_craft(
        "Server.Items.BreadLoaf",
        Td::of_number(1044497),
        Td::of_number(1024156),
        None,
        None,
        0.0,
        100.0,
        "Server.Items.Dough",
        Td::of_number(1044469),
        1,
        Td::of_number(1044253),
    );
    s.set_need_oven(index, true);
    index = s.add_craft(
        "Server.Items.Cookies",
        Td::of_number(1044497),
        Td::of_number(1025643),
        None,
        None,
        0.0,
        100.0,
        "Server.Items.CookieMix",
        Td::of_number(1044474),
        1,
        Td::of_number(1044253),
    );
    s.set_need_oven(index, true);
    index = s.add_craft(
        "Server.Items.Cake",
        Td::of_number(1044497),
        Td::of_number(1022537),
        None,
        None,
        0.0,
        100.0,
        "Server.Items.CakeMix",
        Td::of_number(1044471),
        1,
        Td::of_number(1044253),
    );
    s.set_need_oven(index, true);
    index = s.add_craft(
        "Server.Items.Muffins",
        Td::of_number(1044497),
        Td::of_number(1022539),
        None,
        None,
        0.0,
        100.0,
        "Server.Items.SweetDough",
        Td::of_number(1044475),
        1,
        Td::of_number(1044253),
    );
    s.set_need_oven(index, true);
    index = s.add_craft(
        "Server.Items.Quiche",
        Td::of_number(1044497),
        Td::of_number(1041345),
        None,
        None,
        0.0,
        100.0,
        "Server.Items.UnbakedQuiche",
        Td::of_number(1044518),
        1,
        Td::of_number(1044253),
    );
    s.set_need_oven(index, true);
    index = s.add_craft(
        "Server.Items.MeatPie",
        Td::of_number(1044497),
        Td::of_number(1041347),
        None,
        None,
        0.0,
        100.0,
        "Server.Items.UnbakedMeatPie",
        Td::of_number(1044519),
        1,
        Td::of_number(1044253),
    );
    s.set_need_oven(index, true);
    index = s.add_craft(
        "Server.Items.SausagePizza",
        Td::of_number(1044497),
        Td::of_number(1044517),
        None,
        None,
        0.0,
        100.0,
        "Server.Items.UncookedSausagePizza",
        Td::of_number(1044520),
        1,
        Td::of_number(1044253),
    );
    s.set_need_oven(index, true);
    index = s.add_craft(
        "Server.Items.CheesePizza",
        Td::of_number(1044497),
        Td::of_number(1044516),
        None,
        None,
        0.0,
        100.0,
        "Server.Items.UncookedCheesePizza",
        Td::of_number(1044521),
        1,
        Td::of_number(1044253),
    );
    s.set_need_oven(index, true);
    index = s.add_craft(
        "Server.Items.FruitPie",
        Td::of_number(1044497),
        Td::of_number(1041346),
        None,
        None,
        0.0,
        100.0,
        "Server.Items.UnbakedFruitPie",
        Td::of_number(1044522),
        1,
        Td::of_number(1044253),
    );
    s.set_need_oven(index, true);
    index = s.add_craft(
        "Server.Items.PeachCobbler",
        Td::of_number(1044497),
        Td::of_number(1041344),
        None,
        None,
        0.0,
        100.0,
        "Server.Items.UnbakedPeachCobbler",
        Td::of_number(1044523),
        1,
        Td::of_number(1044253),
    );
    s.set_need_oven(index, true);
    index = s.add_craft(
        "Server.Items.ApplePie",
        Td::of_number(1044497),
        Td::of_number(1041343),
        None,
        None,
        0.0,
        100.0,
        "Server.Items.UnbakedApplePie",
        Td::of_number(1044524),
        1,
        Td::of_number(1044253),
    );
    s.set_need_oven(index, true);
    index = s.add_craft(
        "Server.Items.PumpkinPie",
        Td::of_number(1044497),
        Td::of_number(1041348),
        None,
        None,
        0.0,
        100.0,
        "Server.Items.UnbakedPumpkinPie",
        Td::of_number(1046461),
        1,
        Td::of_number(1044253),
    );
    s.set_need_oven(index, true);
    if core::se(w) {
        index = s.add_craft(
            "Server.Items.MisoSoup",
            Td::of_number(1044497),
            Td::of_number(1030317),
            None,
            None,
            60.0,
            110.0,
            "Server.Items.RawFishSteak",
            Td::of_number(1044476),
            1,
            Td::of_number(1044253),
        );
        s.add_res(
            index,
            "Server.Items.BaseBeverage",
            Td::of_number(1046458),
            1,
            Td::of_number(1044253),
        );
        s.set_needed_expansion(index, Expansion::SE);
        s.set_need_oven(index, true);
        index = s.add_craft(
            "Server.Items.WhiteMisoSoup",
            Td::of_number(1044497),
            Td::of_number(1030318),
            None,
            None,
            60.0,
            110.0,
            "Server.Items.RawFishSteak",
            Td::of_number(1044476),
            1,
            Td::of_number(1044253),
        );
        s.add_res(
            index,
            "Server.Items.BaseBeverage",
            Td::of_number(1046458),
            1,
            Td::of_number(1044253),
        );
        s.set_needed_expansion(index, Expansion::SE);
        s.set_need_oven(index, true);
        index = s.add_craft(
            "Server.Items.RedMisoSoup",
            Td::of_number(1044497),
            Td::of_number(1030319),
            None,
            None,
            60.0,
            110.0,
            "Server.Items.RawFishSteak",
            Td::of_number(1044476),
            1,
            Td::of_number(1044253),
        );
        s.add_res(
            index,
            "Server.Items.BaseBeverage",
            Td::of_number(1046458),
            1,
            Td::of_number(1044253),
        );
        s.set_needed_expansion(index, Expansion::SE);
        s.set_need_oven(index, true);
        index = s.add_craft(
            "Server.Items.AwaseMisoSoup",
            Td::of_number(1044497),
            Td::of_number(1030320),
            None,
            None,
            60.0,
            110.0,
            "Server.Items.RawFishSteak",
            Td::of_number(1044476),
            1,
            Td::of_number(1044253),
        );
        s.add_res(
            index,
            "Server.Items.BaseBeverage",
            Td::of_number(1046458),
            1,
            Td::of_number(1044253),
        );
        s.set_needed_expansion(index, Expansion::SE);
        s.set_need_oven(index, true);
    }
    index = s.add_craft(
        "Server.Items.CookedBird",
        Td::of_number(1044498),
        Td::of_number(1022487),
        None,
        None,
        0.0,
        100.0,
        "Server.Items.RawBird",
        Td::of_number(1044470),
        1,
        Td::of_number(1044253),
    );
    s.set_need_heat(index, true);
    s.set_use_all_res(index, true);
    index = s.add_craft(
        "Server.Items.ChickenLeg",
        Td::of_number(1044498),
        Td::of_number(1025640),
        None,
        None,
        0.0,
        100.0,
        "Server.Items.RawChickenLeg",
        Td::of_number(1044473),
        1,
        Td::of_number(1044253),
    );
    s.set_need_heat(index, true);
    s.set_use_all_res(index, true);
    index = s.add_craft(
        "Server.Items.FishSteak",
        Td::of_number(1044498),
        Td::of_number(1022427),
        None,
        None,
        0.0,
        100.0,
        "Server.Items.RawFishSteak",
        Td::of_number(1044476),
        1,
        Td::of_number(1044253),
    );
    s.set_need_heat(index, true);
    s.set_use_all_res(index, true);
    index = s.add_craft(
        "Server.Items.FriedEggs",
        Td::of_number(1044498),
        Td::of_number(1022486),
        None,
        None,
        0.0,
        100.0,
        "Server.Items.Eggs",
        Td::of_number(1044477),
        1,
        Td::of_number(1044253),
    );
    s.set_need_heat(index, true);
    s.set_use_all_res(index, true);
    index = s.add_craft(
        "Server.Items.LambLeg",
        Td::of_number(1044498),
        Td::of_number(1025642),
        None,
        None,
        0.0,
        100.0,
        "Server.Items.RawLambLeg",
        Td::of_number(1044478),
        1,
        Td::of_number(1044253),
    );
    s.set_need_heat(index, true);
    s.set_use_all_res(index, true);
    index = s.add_craft(
        "Server.Items.Ribs",
        Td::of_number(1044498),
        Td::of_number(1022546),
        None,
        None,
        0.0,
        100.0,
        "Server.Items.RawRibs",
        Td::of_number(1044485),
        1,
        Td::of_number(1044253),
    );
    s.set_need_heat(index, true);
    s.set_use_all_res(index, true);
    if core::ml(w) {
        index = s.add_craft(
            "Server.Items.DarkChocolate",
            Td::of_number(1080001),
            Td::of_number(1079994),
            None,
            None,
            15.0,
            100.0,
            "Server.Items.SackOfSugar",
            Td::of_number(1079997),
            1,
            Td::of_number(1044253),
        );
        s.add_res(
            index,
            "Server.Items.CocoaButter",
            Td::of_number(1079998),
            1,
            Td::of_number(1044253),
        );
        s.add_res(
            index,
            "Server.Items.CocoaLiquor",
            Td::of_number(1079999),
            1,
            Td::of_number(1044253),
        );
        s.set_item_hue(index, 0x465);
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.MilkChocolate",
            Td::of_number(1080001),
            Td::of_number(1079995),
            None,
            None,
            32.5,
            107.5,
            "Server.Items.SackOfSugar",
            Td::of_number(1079997),
            1,
            Td::of_number(1044253),
        );
        s.add_res(
            index,
            "Server.Items.CocoaButter",
            Td::of_number(1079998),
            1,
            Td::of_number(1044253),
        );
        s.add_res(
            index,
            "Server.Items.CocoaLiquor",
            Td::of_number(1079999),
            1,
            Td::of_number(1044253),
        );
        s.add_res(
            index,
            "Server.Items.BaseBeverage",
            Td::of_number(1022544),
            1,
            Td::of_number(1044253),
        );
        s.set_beverage_type(index, BeverageType::Milk);
        s.set_item_hue(index, 0x461);
        s.set_needed_expansion(index, Expansion::ML);
        index = s.add_craft(
            "Server.Items.WhiteChocolate",
            Td::of_number(1080001),
            Td::of_number(1079996),
            None,
            None,
            52.5,
            127.5,
            "Server.Items.SackOfSugar",
            Td::of_number(1079997),
            1,
            Td::of_number(1044253),
        );
        s.add_res(
            index,
            "Server.Items.CocoaButter",
            Td::of_number(1079998),
            1,
            Td::of_number(1044253),
        );
        s.add_res(
            index,
            "Server.Items.Vanilla",
            Td::of_number(1080000),
            1,
            Td::of_number(1044253),
        );
        s.add_res(
            index,
            "Server.Items.BaseBeverage",
            Td::of_number(1022544),
            1,
            Td::of_number(1044253),
        );
        s.set_beverage_type(index, BeverageType::Milk);
        s.set_item_hue(index, 0x47E);
        s.set_needed_expansion(index, Expansion::ML);
    }
}
