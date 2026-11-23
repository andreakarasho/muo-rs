item("AbbatoirDeed", |c| {
    c.label = 1044329;
    c.deed_addon = "Server.Items.AbbatoirAddon";
});

item("AlchemistTableEastDeed", |c| {
    c.label = 1073397;
    c.deed_addon = "Server.Items.AlchemistTableEastAddon";
});

item("AlchemistTableSouthDeed", |c| {
    c.label = 1073396;
    c.deed_addon = "Server.Items.AlchemistTableSouthAddon";
});

item("AnvilEastDeed", |c| {
    c.label = 1044333;
    c.deed_addon = "Server.Items.AnvilEastAddon";
});

item("AnvilSouthDeed", |c| {
    c.label = 1044334;
    c.deed_addon = "Server.Items.AnvilSouthAddon";
});

item("ArcaneBookshelfEastAddon", |c| {
    c.gump_id = 263;
    c.drop_sound = 66;
    c.addon_retain_deed_hue = true;
    c.addon_deed = "Server.Items.ArcaneBookshelfEastDeed";
    c.on_create(|i| {
        i.item_id = 0x3084;
        i.addon_components = [[12421, -1, 0, 0]];
    });
});

item("ArcaneBookshelfEastDeed", |c| {
    c.label = 1073371;
    c.deed_addon = "Server.Items.ArcaneBookshelfEastAddon";
});

item("ArcaneBookshelfSouthAddon", |c| {
    c.gump_id = 263;
    c.drop_sound = 66;
    c.label = 1032420;
    c.addon_retain_deed_hue = true;
    c.addon_deed = "Server.Items.ArcaneBookshelfSouthDeed";
    c.on_create(|i| {
        i.item_id = 0x2DEF;
        i.addon_components = [[#{ itemId: 8612, label: 1031760 }, -1, 0, 0]];
    });
});

item("ArcaneBookshelfSouthDeed", |c| {
    c.label = 1072871;
    c.deed_addon = "Server.Items.ArcaneBookshelfSouthAddon";
});

item("ArcaneCircleDeed", |c| {
    c.label = 1072703;
    c.deed_addon = "Server.Items.ArcaneCircleAddon";
});

item("ArcanistStatueEastDeed", |c| {
    c.label = 1072886;
    c.deed_addon = "Server.Items.ArcanistStatueEastAddon";
});

item("ArcanistStatueSouthDeed", |c| {
    c.label = 1072885;
    c.deed_addon = "Server.Items.ArcanistStatueSouthAddon";
});

item("BrownBearRugEastDeed", |c| {
    c.label = 1049397;
    c.deed_addon = "Server.Items.BrownBearRugEastAddon";
});

item("BrownBearRugSouthDeed", |c| {
    c.label = 1049398;
    c.deed_addon = "Server.Items.BrownBearRugSouthAddon";
});

item("PolarBearRugEastDeed", |c| {
    c.label = 1049399;
    c.deed_addon = "Server.Items.PolarBearRugEastAddon";
});

item("PolarBearRugSouthDeed", |c| {
    c.label = 1049400;
    c.deed_addon = "Server.Items.PolarBearRugSouthAddon";
});

item("ElvenBedEastDeed", |c| {
    c.label = 1072861;
    c.deed_addon = "Server.Items.ElvenBedEastAddon";
});

item("ElvenBedSouthDeed", |c| {
    c.label = 1072860;
    c.deed_addon = "Server.Items.ElvenBedSouthAddon";
});

item("ElvenDresserEastAddon", |c| {
    c.gump_id = 81;
    c.drop_sound = 66;
    c.addon_retain_deed_hue = true;
    c.addon_deed = "Server.Items.ElvenDresserEastDeed";
    c.on_create(|i| {
        i.item_id = 0x30E4;
        i.addon_components = [[12515, 0, -1, 0]];
    });
});

item("ElvenDresserEastDeed", |c| {
    c.label = 1073388;
    c.deed_addon = "Server.Items.ElvenDresserEastAddon";
});

item("ElvenDresserSouthAddon", |c| {
    c.gump_id = 81;
    c.drop_sound = 66;
    c.addon_retain_deed_hue = true;
    c.addon_deed = "Server.Items.ElvenDresserSouthDeed";
    c.on_create(|i| {
        i.item_id = 0x30E6;
        i.addon_components = [[12517, -1, 0, 0]];
    });
});

item("ElvenDresserSouthDeed", |c| {
    c.label = 1072864;
    c.deed_addon = "Server.Items.ElvenDresserSouthAddon";
});

item("ElvenForgeAddon", |c| {
    c.label = 1072875;
    c.addon_deed = "Server.Items.ElvenForgeDeed";
    c.on_create(|i| {
        i.addon_components = [[11736, 0, 0, 0]];
    });
});

item("ElvenForgeDeed", |c| {
    c.label = 1072875;
    c.deed_addon = "Server.Items.ElvenForgeAddon";
});

item("ElvenLoveseatEastDeed", |c| {
    c.label = 1073372;
    c.deed_addon = "Server.Items.ElvenLoveseatEastAddon";
});

item("ElvenLoveseatSouthDeed", |c| {
    c.label = 1072867;
    c.deed_addon = "Server.Items.ElvenLoveseatSouthAddon";
});

item("ElvenStoveEastDeed", |c| {
    c.label = 1073395;
    c.deed_addon = "Server.Items.ElvenStoveEastAddon";
});

item("ElvenStoveSouthDeed", |c| {
    c.label = 1073394;
    c.deed_addon = "Server.Items.ElvenStoveSouthAddon";
});

item("ElvenWashBasinEastAddon", |c| {
    c.gump_id = 260;
    c.drop_sound = 66;
    c.addon_retain_deed_hue = true;
    c.addon_deed = "Server.Items.ElvenWashBasinEastDeed";
    c.on_create(|i| {
        i.item_id = 0x30E0;
        i.addon_components = [[12511, 0, -1, 0]];
    });
});

item("ElvenWashBasinEastDeed", |c| {
    c.label = 1073387;
    c.deed_addon = "Server.Items.ElvenWashBasinEastAddon";
});

item("ElvenWashBasinSouthAddon", |c| {
    c.gump_id = 260;
    c.drop_sound = 66;
    c.addon_retain_deed_hue = true;
    c.addon_deed = "Server.Items.ElvenWashBasinSouthDeed";
    c.on_create(|i| {
        i.item_id = 0x30E2;
        i.addon_components = [[12513, -1, 0, 0]];
    });
});

item("ElvenWashBasinSouthDeed", |c| {
    c.label = 1072865;
    c.deed_addon = "Server.Items.ElvenWashBasinSouthAddon";
});

item("FancyElvenTableEastDeed", |c| {
    c.label = 1073386;
    c.deed_addon = "Server.Items.FancyElvenTableEastAddon";
});

item("FancyElvenTableSouthDeed", |c| {
    c.label = 1073385;
    c.deed_addon = "Server.Items.FancyElvenTableSouthAddon";
});

item("LightFlowerTapestryEastDeed", |c| {
    c.label = 1049393;
    c.deed_addon = "Server.Items.LightFlowerTapestryEastAddon";
});

item("LightFlowerTapestrySouthDeed", |c| {
    c.label = 1049394;
    c.deed_addon = "Server.Items.LightFlowerTapestrySouthAddon";
});

item("DarkFlowerTapestryEastDeed", |c| {
    c.label = 1049395;
    c.deed_addon = "Server.Items.DarkFlowerTapestryEastAddon";
});

item("DarkFlowerTapestrySouthDeed", |c| {
    c.label = 1049396;
    c.deed_addon = "Server.Items.DarkFlowerTapestrySouthAddon";
});

item("GozaMatEastDeed", |c| {
    c.label = 1030404;
    c.deed_addon = "Server.Items.GozaMatEastAddon";
});

item("GozaMatSouthDeed", |c| {
    c.label = 1030405;
    c.deed_addon = "Server.Items.GozaMatSouthAddon";
});

item("SquareGozaMatEastAddon", |c| {
    c.label = 1030688;
    c.addon_retain_deed_hue = true;
    c.addon_deed = "Server.Items.SquareGozaMatEastDeed";
    c.on_create(|i| {
        i.addon_components = [[#{ itemId: 10408, label: 1030688 }, 0, 0, 0]];
    });
});

item("SquareGozaMatEastDeed", |c| {
    c.label = 1030407;
    c.deed_addon = "Server.Items.SquareGozaMatEastAddon";
});

item("SquareGozaMatSouthDeed", |c| {
    c.label = 1030406;
    c.deed_addon = "Server.Items.SquareGozaMatSouthAddon";
});

item("BrocadeGozaMatEastDeed", |c| {
    c.label = 1030408;
    c.deed_addon = "Server.Items.BrocadeGozaMatEastAddon";
});

item("BrocadeGozaMatSouthDeed", |c| {
    c.label = 1030409;
    c.deed_addon = "Server.Items.BrocadeGozaMatSouthAddon";
});

item("BrocadeSquareGozaMatEastDeed", |c| {
    c.label = 1030411;
    c.deed_addon = "Server.Items.BrocadeSquareGozaMatEastAddon";
});

item("BrocadeSquareGozaMatSouthDeed", |c| {
    c.label = 1030410;
    c.deed_addon = "Server.Items.BrocadeSquareGozaMatSouthAddon";
});

item("GrayBrickFireplaceEastDeed", |c| {
    c.label = 1061846;
    c.deed_addon = "Server.Items.GrayBrickFireplaceEastAddon";
});

item("GrayBrickFireplaceSouthDeed", |c| {
    c.label = 1061847;
    c.deed_addon = "Server.Items.GrayBrickFireplaceSouthAddon";
});

item("LargeBedEastDeed", |c| {
    c.label = 1044324;
    c.deed_addon = "Server.Items.LargeBedEastAddon";
});

item("LargeBedSouthDeed", |c| {
    c.label = 1044323;
    c.deed_addon = "Server.Items.LargeBedSouthAddon";
});

item("LargeForgeEastDeed", |c| {
    c.label = 1044331;
    c.deed_addon = "Server.Items.LargeForgeEastAddon";
});

item("LargeForgeSouthDeed", |c| {
    c.label = 1044332;
    c.deed_addon = "Server.Items.LargeForgeSouthAddon";
});

item("LargeStoneTableEastDeed", |c| {
    c.label = 1044511;
    c.deed_addon = "Server.Items.LargeStoneTableEastAddon";
});

item("LargeStoneTableSouthDeed", |c| {
    c.label = 1044512;
    c.deed_addon = "Server.Items.LargeStoneTableSouthAddon";
});

item("MediumStoneTableEastDeed", |c| {
    c.label = 1044508;
    c.deed_addon = "Server.Items.MediumStoneTableEastAddon";
});

item("MediumStoneTableSouthDeed", |c| {
    c.label = 1044509;
    c.deed_addon = "Server.Items.MediumStoneTableSouthAddon";
});

item("OrnateElvenChestEastAddon", |c| {
    c.gump_id = 268;
    c.drop_sound = 66;
    c.addon_retain_deed_hue = true;
    c.addon_deed = "Server.Items.OrnateElvenChestEastDeed";
    c.on_create(|i| {
        i.item_id = 0x309A;
        i.addon_components = [[12443, 0, -1, 0]];
    });
});

item("OrnateElvenChestEastDeed", |c| {
    c.label = 1073383;
    c.deed_addon = "Server.Items.OrnateElvenChestEastAddon";
});

item("OrnateElvenChestSouthAddon", |c| {
    c.gump_id = 268;
    c.drop_sound = 66;
    c.addon_retain_deed_hue = true;
    c.addon_deed = "Server.Items.OrnateElvenChestSouthDeed";
    c.on_create(|i| {
        i.item_id = 0x3098;
        i.addon_components = [[12441, -1, 0, 0]];
    });
});

item("OrnateElvenChestSouthDeed", |c| {
    c.label = 1072862;
    c.deed_addon = "Server.Items.OrnateElvenChestSouthAddon";
});

item("OrnateElvenTableEastDeed", |c| {
    c.label = 1073384;
    c.deed_addon = "Server.Items.OrnateElvenTableEastAddon";
});

item("OrnateElvenTableSouthDeed", |c| {
    c.label = 1072869;
    c.deed_addon = "Server.Items.OrnateElvenTableSouthAddon";
});

item("ParrotPerchDeed", |c| {
    c.label = 1072617;
    c.deed_addon = "Server.Items.ParrotPerchAddon";
});

item("PentagramDeed", |c| {
    c.label = 1044328;
    c.deed_addon = "Server.Items.PentagramAddon";
});

item("SandstoneFireplaceEastDeed", |c| {
    c.label = 1061844;
    c.deed_addon = "Server.Items.SandstoneFireplaceEastAddon";
});

item("SandstoneFireplaceSouthDeed", |c| {
    c.label = 1061845;
    c.deed_addon = "Server.Items.SandstoneFireplaceSouthAddon";
});

item("ShrineOfWisdomComponent", |c| {
    c.label = 1062046;
});

item("SmallBedEastDeed", |c| {
    c.label = 1044322;
    c.deed_addon = "Server.Items.SmallBedEastAddon";
});

item("SmallBedSouthDeed", |c| {
    c.label = 1044321;
    c.deed_addon = "Server.Items.SmallBedSouthAddon";
});

item("SmallForgeDeed", |c| {
    c.label = 1044330;
    c.deed_addon = "Server.Items.SmallForgeAddon";
});

item("SquirrelStatueEastDeed", |c| {
    c.label = 1073398;
    c.deed_addon = "Server.Items.SquirrelStatueEastAddon";
});

item("SquirrelStatueSouthDeed", |c| {
    c.label = 1072884;
    c.deed_addon = "Server.Items.SquirrelStatueSouthAddon";
});

item("StoneFireplaceEastDeed", |c| {
    c.label = 1061848;
    c.deed_addon = "Server.Items.StoneFireplaceEastAddon";
});

item("StoneFireplaceSouthDeed", |c| {
    c.label = 1061849;
    c.deed_addon = "Server.Items.StoneFireplaceSouthAddon";
});

item("StoneOvenEastDeed", |c| {
    c.label = 1044345;
    c.deed_addon = "Server.Items.StoneOvenEastAddon";
});

item("StoneOvenSouthDeed", |c| {
    c.label = 1044346;
    c.deed_addon = "Server.Items.StoneOvenSouthAddon";
});

item("SmallStretchedHideEastDeed", |c| {
    c.label = 1049401;
    c.deed_addon = "Server.Items.SmallStretchedHideEastAddon";
});

item("SmallStretchedHideSouthDeed", |c| {
    c.label = 1049402;
    c.deed_addon = "Server.Items.SmallStretchedHideSouthAddon";
});

item("MediumStretchedHideEastDeed", |c| {
    c.label = 1049403;
    c.deed_addon = "Server.Items.MediumStretchedHideEastAddon";
});

item("MediumStretchedHideSouthDeed", |c| {
    c.label = 1049404;
    c.deed_addon = "Server.Items.MediumStretchedHideSouthAddon";
});

item("TallElvenBedEastDeed", |c| {
    c.label = 1072859;
    c.deed_addon = "Server.Items.TallElvenBedEastAddon";
});

item("TallElvenBedSouthDeed", |c| {
    c.label = 1072858;
    c.deed_addon = "Server.Items.TallElvenBedSouthAddon";
});

item("WarriorStatueEastDeed", |c| {
    c.label = 1072888;
    c.deed_addon = "Server.Items.WarriorStatueEastAddon";
});

item("WarriorStatueSouthDeed", |c| {
    c.label = 1072887;
    c.deed_addon = "Server.Items.WarriorStatueSouthAddon";
});

item("AbbatoirAddon", |c| {
    c.addon_deed = "Server.Items.AbbatoirDeed";
    c.on_create(|i| {
        i.addon_components = [
            [4622, -1, -1, 0],
            [4623, 0, -1, 0],
            [4624, 1, -1, 0],
            [4629, -1, 0, 0],
            [4630, 0, 0, 0],
            [4625, 1, 0, 0],
            [4628, -1, 1, 0],
            [4627, 0, 1, 0],
            [4626, 1, 1, 0],
        ];
    });
});

item("AlchemistTableEastAddon", |c| {
    c.addon_deed = "Server.Items.AlchemistTableEastDeed";
    c.on_create(|i| {
        i.addon_components = [[11731, 0, 0, 0]];
    });
});

item("AlchemistTableSouthAddon", |c| {
    c.addon_deed = "Server.Items.AlchemistTableSouthDeed";
    c.on_create(|i| {
        i.addon_components = [[11732, 0, 0, 0]];
    });
});

item("AnvilEastAddon", |c| {
    c.addon_deed = "Server.Items.AnvilEastDeed";
    c.on_create(|i| {
        i.addon_components = [[#{ itemId: 4015, type: "Server.Items.AnvilComponent" }, 0, 0, 0]];
    });
});

item("AnvilSouthAddon", |c| {
    c.addon_deed = "Server.Items.AnvilSouthDeed";
    c.on_create(|i| {
        i.addon_components = [[#{ itemId: 4016, type: "Server.Items.AnvilComponent" }, 0, 0, 0]];
    });
});

item("ArcaneCircleAddon", |c| {
    c.addon_deed = "Server.Items.ArcaneCircleDeed";
    c.on_create(|i| {
        i.addon_components = [
            [12419, -1, -1, 0],
            [12416, -1, 0, 0],
            [12418, 0, -1, 0],
            [12417, 1, -1, 0],
            [12413, -1, 1, 0],
            [12415, 0, 0, 0],
            [12414, 1, 0, 0],
            [12412, 0, 1, 0],
            [12411, 1, 1, 0],
        ];
    });
});

item("ArcanistStatueEastAddon", |c| {
    c.addon_deed = "Server.Items.ArcanistStatueEastDeed";
    c.on_create(|i| {
        i.addon_components = [[11534, 0, 0, 0]];
    });
});

item("ArcanistStatueSouthAddon", |c| {
    c.addon_deed = "Server.Items.ArcanistStatueSouthDeed";
    c.on_create(|i| {
        i.addon_components = [[11535, 0, 0, 0]];
    });
});

item("BloodPentagram", |c| {
    c.on_create(|i| {
        i.addon_components = [
            [7417, 0, 1, 0],
            [7416, 0, 2, 0],
            [7415, 0, 3, 0],
            [7414, 0, 4, 0],
            [7413, 0, 5, 0],
            [7419, 1, 0, 0],
            [7418, 1, 1, 0],
            [7433, 1, 2, 0],
            [7432, 1, 3, 0],
            [7431, 1, 4, 0],
            [7412, 1, 5, 0],
            [7420, 2, 0, 0],
            [7434, 2, 1, 0],
            [7441, 2, 2, 0],
            [7440, 2, 3, 0],
            [7430, 2, 4, 0],
            [7411, 2, 5, 0],
            [7421, 3, 0, 0],
            [7435, 3, 1, 0],
            [7442, 3, 2, 0],
            [7439, 3, 3, 0],
            [7429, 3, 4, 0],
            [7410, 3, 5, 0],
            [7422, 4, 0, 0],
            [7436, 4, 1, 0],
            [7437, 4, 2, 0],
            [7438, 4, 3, 0],
            [7428, 4, 4, 0],
            [7409, 4, 5, 0],
            [7423, 5, 0, 0],
            [7424, 5, 1, 0],
            [7425, 5, 2, 0],
            [7426, 5, 3, 0],
            [7427, 5, 4, 0],
        ];
    });
});

item("BrocadeGozaMatEastAddon", |c| {
    c.addon_retain_deed_hue = true;
    c.addon_deed = "Server.Items.BrocadeGozaMatEastDeed";
    c.on_create(|i| {
        i.addon_components = [[#{ itemId: 10411, label: 1030688 }, 0, 0, 0], [#{ itemId: 10410, label: 1030688 }, 1, 0, 0]];
    });
});

item("BrocadeGozaMatSouthAddon", |c| {
    c.addon_retain_deed_hue = true;
    c.addon_deed = "Server.Items.BrocadeGozaMatSouthDeed";
    c.on_create(|i| {
        i.addon_components = [[#{ itemId: 10413, label: 1030688 }, 0, 0, 0], [#{ itemId: 10412, label: 1030688 }, 0, 1, 0]];
    });
});

item("BrocadeSquareGozaMatEastAddon", |c| {
    c.addon_retain_deed_hue = true;
    c.addon_deed = "Server.Items.BrocadeSquareGozaMatEastDeed";
    c.on_create(|i| {
        i.addon_components = [[#{ itemId: 10414, label: 1030688 }, 0, 0, 0]];
    });
});

item("BrocadeSquareGozaMatSouthAddon", |c| {
    c.addon_retain_deed_hue = true;
    c.addon_deed = "Server.Items.BrocadeSquareGozaMatSouthDeed";
    c.on_create(|i| {
        i.addon_components = [[#{ itemId: 10415, label: 1030688 }, 0, 0, 0]];
    });
});

item("BrownBearRugEastAddon", |c| {
    c.addon_deed = "Server.Items.BrownBearRugEastDeed";
    c.on_create(|i| {
        i.addon_components = [
            [7744, 1, 1, 0],
            [7745, 1, 0, 0],
            [7746, 1, -1, 0],
            [7747, 0, -1, 0],
            [7748, 0, 0, 0],
            [7749, 0, 1, 0],
            [7750, -1, 1, 0],
            [7751, -1, 0, 0],
            [7752, -1, -1, 0],
        ];
    });
});

item("BrownBearRugSouthAddon", |c| {
    c.addon_deed = "Server.Items.BrownBearRugSouthDeed";
    c.on_create(|i| {
        i.addon_components = [
            [7734, 1, 1, 0],
            [7735, 0, 1, 0],
            [7736, -1, 1, 0],
            [7737, -1, 0, 0],
            [7738, 0, 0, 0],
            [7739, 1, 0, 0],
            [7740, 1, -1, 0],
            [7741, 0, -1, 0],
            [7742, -1, -1, 0],
        ];
    });
});

item("DarkFlowerTapestryEastAddon", |c| {
    c.addon_deed = "Server.Items.DarkFlowerTapestryEastDeed";
    c.on_create(|i| {
        i.addon_components = [[4064, 0, 0, 0], [4063, 0, 1, 0]];
    });
});

item("DarkFlowerTapestrySouthAddon", |c| {
    c.addon_deed = "Server.Items.DarkFlowerTapestrySouthDeed";
    c.on_create(|i| {
        i.addon_components = [[4061, 0, 0, 0], [4062, 1, 0, 0]];
    });
});

item("ElvenBedEastAddon", |c| {
    c.addon_deed = "Server.Items.ElvenBedEastDeed";
    c.on_create(|i| {
        i.addon_components = [[12365, 0, 0, 0], [12364, 1, 0, 0]];
    });
});

item("ElvenBedSouthAddon", |c| {
    c.addon_deed = "Server.Items.ElvenBedSouthDeed";
    c.on_create(|i| {
        i.addon_components = [[12368, 0, 0, 0], [12369, 0, -1, 0]];
    });
});

item("ElvenLoveseatEastAddon", |c| {
    c.addon_deed = "Server.Items.ElvenLoveseatEastDeed";
    c.on_create(|i| {
        i.addon_components = [[12426, 0, 0, 0], [12427, 0, -1, 0]];
    });
});

item("ElvenLoveseatSouthAddon", |c| {
    c.addon_deed = "Server.Items.ElvenLoveseatSouthDeed";
    c.on_create(|i| {
        i.addon_components = [[12424, 0, 0, 0], [12425, -1, 0, 0]];
    });
});

item("ElvenStoveEastAddon", |c| {
    c.addon_deed = "Server.Items.ElvenStoveEastDeed";
    c.on_create(|i| {
        i.addon_components = [[11739, 0, 0, 0]];
    });
});

item("ElvenStoveSouthAddon", |c| {
    c.addon_deed = "Server.Items.ElvenStoveSouthDeed";
    c.on_create(|i| {
        i.addon_components = [[11740, 0, 0, 0]];
    });
});

item("FancyElvenTableEastAddon", |c| {
    c.addon_deed = "Server.Items.FancyElvenTableEastDeed";
    c.on_create(|i| {
        i.addon_components = [[12436, -1, 0, 0], [12435, 0, 0, 0], [12434, 1, 0, 0]];
    });
});

item("FancyElvenTableSouthAddon", |c| {
    c.addon_deed = "Server.Items.FancyElvenTableSouthDeed";
    c.on_create(|i| {
        i.addon_components = [[12437, 0, 1, 0], [12438, 0, 0, 0], [12439, 0, -1, 0]];
    });
});

item("FireColumnAddon", |c| {
    c.addon_share_hue = false;
    c.addon_components_bloody = [
        [4651, -2, 0, 0],
        [4654, 0, -2, 0],
        [4653, -1, 1, 0],
        [4655, 1, -1, 0],
        [4653, 0, 1, 0],
        [4650, 1, 0, 0],
        [4651, 2, -1, 0],
        [4651, 0, 2, 0],
        [4654, 1, 1, 0],
    ];
    c.on_create(|i| {
        i.addon_components = [
            [933, 0, 0, 0],
            [933, 0, 0, 5],
            [933, 0, 0, 10],
            [933, 0, 0, 15],
            [6587, 0, 0, 21],
            [6571, 0, 0, 23],
        ];
    });
});

item("GiantWeb1", |c| {
    c.on_create(|i| {
        i.addon_components = [[4280, 4, -4, 0], [4281, 3, -3, 0], [4282, 2, -2, 0], [4283, 1, -1, 0], [4284, 0, 0, 0]];
    });
});

item("GiantWeb2", |c| {
    c.on_create(|i| {
        i.addon_components = [[4285, 0, 0, 0], [4286, 1, -1, 0], [4287, 2, -2, 0], [4288, 3, -3, 0], [4289, 4, -4, 0]];
    });
});

item("GiantWeb3", |c| {
    c.on_create(|i| {
        i.addon_components = [[4290, 0, 0, 0], [4291, 1, -1, 0], [4292, 2, -2, 0], [4293, 3, -3, 0]];
    });
});

item("GiantWeb4", |c| {
    c.on_create(|i| {
        i.addon_components = [[4294, 3, -3, 0], [4295, 2, -2, 0], [4296, 1, -1, 0], [4297, 0, 0, 0]];
    });
});

item("GiantWeb5", |c| {
    c.on_create(|i| {
        i.addon_components = [[4298, 0, 0, 0], [4299, 1, -1, 0], [4300, 2, -2, 0], [4301, 3, -3, 0]];
    });
});

item("GiantWeb6", |c| {
    c.on_create(|i| {
        i.addon_components = [[4302, 3, -3, 0], [4303, 2, -2, 0], [4304, 1, -1, 0], [4305, 0, 0, 0]];
    });
});

item("GozaMatEastAddon", |c| {
    c.addon_retain_deed_hue = true;
    c.addon_deed = "Server.Items.GozaMatEastDeed";
    c.on_create(|i| {
        i.addon_components = [[#{ itemId: 10404, label: 1030688 }, 1, 0, 0], [#{ itemId: 10405, label: 1030688 }, 0, 0, 0]];
    });
});

item("GozaMatSouthAddon", |c| {
    c.addon_retain_deed_hue = true;
    c.addon_deed = "Server.Items.GozaMatSouthDeed";
    c.on_create(|i| {
        i.addon_components = [[#{ itemId: 10406, label: 1030688 }, 0, 1, 0], [#{ itemId: 10407, label: 1030688 }, 0, 0, 0]];
    });
});

item("GrayBrickFireplaceEastAddon", |c| {
    c.addon_deed = "Server.Items.GrayBrickFireplaceEastDeed";
    c.on_create(|i| {
        i.addon_components = [[2365, 0, 0, 0], [2359, 0, 1, 0]];
    });
});

item("GrayBrickFireplaceSouthAddon", |c| {
    c.addon_deed = "Server.Items.GrayBrickFireplaceSouthDeed";
    c.on_create(|i| {
        i.addon_components = [[2379, -1, 0, 0], [2373, 0, 0, 0]];
    });
});

item("LargeBedEastAddon", |c| {
    c.addon_deed = "Server.Items.LargeBedEastDeed";
    c.on_create(|i| {
        i.addon_components = [[2685, 0, 0, 0], [2684, 0, 1, 0], [2681, 1, 0, 0], [2680, 1, 1, 0]];
    });
});

item("LargeBedSouthAddon", |c| {
    c.addon_deed = "Server.Items.LargeBedSouthDeed";
    c.on_create(|i| {
        i.addon_components = [[2691, 0, 0, 0], [2687, 0, 1, 0], [2690, 1, 0, 0], [2686, 1, 1, 0]];
    });
});

item("LargeForgeEastAddon", |c| {
    c.addon_deed = "Server.Items.LargeForgeEastDeed";
    c.on_create(|i| {
        i.addon_components = [
            [#{ itemId: 6534, type: "Server.Items.ForgeComponent" }, 0, 0, 0],
            [#{ itemId: 6538, type: "Server.Items.ForgeComponent" }, 0, 1, 0],
            [#{ itemId: 6550, type: "Server.Items.ForgeComponent" }, 0, 2, 0],
            [#{ itemId: 6546, type: "Server.Items.ForgeComponent" }, 0, 3, 0],
        ];
    });
});

item("LargeForgeSouthAddon", |c| {
    c.addon_deed = "Server.Items.LargeForgeSouthDeed";
    c.on_create(|i| {
        i.addon_components = [
            [#{ itemId: 6522, type: "Server.Items.ForgeComponent" }, 0, 0, 0],
            [#{ itemId: 6526, type: "Server.Items.ForgeComponent" }, 1, 0, 0],
            [#{ itemId: 6562, type: "Server.Items.ForgeComponent" }, 2, 0, 0],
            [#{ itemId: 6558, type: "Server.Items.ForgeComponent" }, 3, 0, 0],
        ];
    });
});

item("LargeStoneTableEastAddon", |c| {
    c.addon_retain_deed_hue = true;
    c.addon_deed = "Server.Items.LargeStoneTableEastDeed";
    c.on_create(|i| {
        i.addon_components = [[4610, 0, 0, 0], [4611, 0, 1, 0], [4609, 0, 2, 0]];
    });
});

item("LargeStoneTableSouthAddon", |c| {
    c.addon_retain_deed_hue = true;
    c.addon_deed = "Server.Items.LargeStoneTableSouthDeed";
    c.on_create(|i| {
        i.addon_components = [[4613, 0, 0, 0], [4614, 1, 0, 0], [4612, 2, 0, 0]];
    });
});

item("LightFlowerTapestryEastAddon", |c| {
    c.addon_deed = "Server.Items.LightFlowerTapestryEastDeed";
    c.on_create(|i| {
        i.addon_components = [[4060, 0, 0, 0], [4059, 0, 1, 0]];
    });
});

item("LightFlowerTapestrySouthAddon", |c| {
    c.addon_deed = "Server.Items.LightFlowerTapestrySouthDeed";
    c.on_create(|i| {
        i.addon_components = [[4057, 0, 0, 0], [4058, 1, 0, 0]];
    });
});

item("MediumStoneTableEastAddon", |c| {
    c.addon_retain_deed_hue = true;
    c.addon_deed = "Server.Items.MediumStoneTableEastDeed";
    c.on_create(|i| {
        i.addon_components = [[4610, 0, 0, 0], [4609, 0, 1, 0]];
    });
});

item("MediumStoneTableSouthAddon", |c| {
    c.addon_retain_deed_hue = true;
    c.addon_deed = "Server.Items.MediumStoneTableSouthDeed";
    c.on_create(|i| {
        i.addon_components = [[4613, 0, 0, 0], [4612, 1, 0, 0]];
    });
});

item("MediumStretchedHideEastAddon", |c| {
    c.addon_deed = "Server.Items.MediumStretchedHideEastDeed";
    c.on_create(|i| {
        i.addon_components = [[4203, 0, 0, 0]];
    });
});

item("MediumStretchedHideSouthAddon", |c| {
    c.addon_deed = "Server.Items.MediumStretchedHideSouthDeed";
    c.on_create(|i| {
        i.addon_components = [[4220, 0, 0, 0]];
    });
});

item("OrnateElvenTableEastAddon", |c| {
    c.addon_deed = "Server.Items.OrnateElvenTableEastDeed";
    c.on_create(|i| {
        i.addon_components = [[12430, -1, 0, 0], [12429, 0, 0, 0], [12428, 1, 0, 0]];
    });
});

item("OrnateElvenTableSouthAddon", |c| {
    c.addon_deed = "Server.Items.OrnateElvenTableSouthDeed";
    c.on_create(|i| {
        i.addon_components = [[12431, 0, 1, 0], [12432, 0, 0, 0], [12433, 0, -1, 0]];
    });
});

item("ParrotPerchAddon", |c| {
    c.addon_deed = "Server.Items.ParrotPerchDeed";
    c.on_create(|i| {
        i.addon_components = [[12276, 0, 0, 0]];
    });
});

item("PentagramAddon", |c| {
    c.addon_deed = "Server.Items.PentagramDeed";
    c.on_create(|i| {
        i.addon_components = [
            [4071, -1, -1, 0],
            [4072, 0, -1, 0],
            [4075, 1, -1, 0],
            [4070, -1, 0, 0],
            [4074, 0, 0, 0],
            [4078, 1, 0, 0],
            [4073, -1, 1, 0],
            [4076, 0, 1, 0],
            [4077, 1, 1, 0],
        ];
    });
});

item("PolarBearRugEastAddon", |c| {
    c.addon_deed = "Server.Items.PolarBearRugEastDeed";
    c.on_create(|i| {
        i.addon_components = [
            [7763, 1, 1, 0],
            [7764, 1, 0, 0],
            [7765, 1, -1, 0],
            [7766, 0, -1, 0],
            [7767, 0, 0, 0],
            [7768, 0, 1, 0],
            [7769, -1, 1, 0],
            [7770, -1, 0, 0],
            [7771, -1, -1, 0],
        ];
    });
});

item("PolarBearRugSouthAddon", |c| {
    c.addon_deed = "Server.Items.PolarBearRugSouthDeed";
    c.on_create(|i| {
        i.addon_components = [
            [7753, 1, 1, 0],
            [7754, 0, 1, 0],
            [7755, -1, 1, 0],
            [7756, -1, 0, 0],
            [7757, 0, 0, 0],
            [7758, 1, 0, 0],
            [7759, 1, -1, 0],
            [7760, 0, -1, 0],
            [7761, -1, -1, 0],
        ];
    });
});

item("PyramidAddon", |c| {
    c.addon_share_hue = false;
    c.on_create(|i| {
        i.addon_components = [
            [1006, 0, 0, 5],
            [1011, -1, -1, 5],
            [1012, 1, 1, 5],
            [1013, 1, -1, 5],
            [1014, -1, 1, 5],
            [1011, -2, -2, 0],
            [1012, 2, 2, 0],
            [1013, 2, -2, 0],
            [1014, -2, 2, 0],
            [1007, -1, 2, 0],
            [1008, 2, -1, 0],
            [1009, -1, -2, 0],
            [1010, -2, -1, 0],
            [1007, 0, 2, 0],
            [1008, 2, 0, 0],
            [1009, 0, -2, 0],
            [1010, -2, 0, 0],
            [1007, 1, 2, 0],
            [1008, 2, 1, 0],
            [1009, 1, -2, 0],
            [1010, -2, 1, 0],
            [1007, 0, 1, 5],
            [1008, 1, 0, 5],
            [1009, 0, -1, 5],
            [1010, -1, 0, 5],
        ];
    });
});

item("SandstoneFireplaceEastAddon", |c| {
    c.addon_deed = "Server.Items.SandstoneFireplaceEastDeed";
    c.on_create(|i| {
        i.addon_components = [[1161, 0, 0, 0], [1141, 0, 1, 0]];
    });
});

item("SandstoneFireplaceSouthAddon", |c| {
    c.addon_deed = "Server.Items.SandstoneFireplaceSouthDeed";
    c.on_create(|i| {
        i.addon_components = [[1154, -1, 0, 0], [1147, 0, 0, 0]];
    });
});

item("SandstoneFountainAddon", |c| {
    c.on_create(|i| {
        i.addon_components = [
            [6595, -2, 1, 0],
            [6596, -1, 1, 0],
            [6597, 0, 1, 0],
            [6598, 1, 1, 0],
            [6599, 1, 0, 0],
            [6600, 1, -1, 0],
            [6601, 1, -2, 0],
            [6602, 0, -2, 0],
            [6603, 0, -1, 0],
            [6604, 0, 0, 0],
            [6605, -1, 0, 0],
            [6606, -2, 0, 0],
            [6607, -2, -1, 0],
            [6608, -1, -1, 0],
            [6609, -1, -2, 0],
            [6611, -2, -2, 0],
        ];
    });
});

item("SerpentPillarAddon", |c| {
    c.on_create(|i| {
        i.addon_components = [
            [9020, -2, 1, 0],
            [9021, -2, 0, 0],
            [9022, -2, -1, 0],
            [9023, -2, -2, 0],
            [9024, -1, -2, 0],
            [9025, 0, -2, 0],
            [9026, 1, -2, 0],
            [9027, -1, 1, 0],
            [9028, -1, 0, 0],
            [9029, -1, -1, 0],
            [9030, 0, 1, 0],
            [9031, 0, 0, 0],
            [9032, 0, -1, 0],
            [9033, 1, 0, 0],
            [9034, 1, -1, 0],
        ];
    });
});

item("ShrineOfWisdomAddon", |c| {
    c.on_create(|i| {
        i.addon_components = [
            [#{ itemId: 5315, type: "Server.Items.ShrineOfWisdomComponent" }, 0, 0, 0],
            [#{ itemId: 5318, type: "Server.Items.ShrineOfWisdomComponent" }, 1, 0, 0],
            [#{ itemId: 5332, type: "Server.Items.ShrineOfWisdomComponent" }, 0, 1, 0],
            [#{ itemId: 5333, type: "Server.Items.ShrineOfWisdomComponent" }, 1, 1, 0],
        ];
        i.hue = 0x47E;
    });
});

item("SkullPileAddon", |c| {
    c.on_create(|i| {
        i.addon_components = [
            [6872, 1, 1, 0],
            [6873, 0, 1, 0],
            [6874, -1, 1, 0],
            [6875, 0, 0, 0],
            [6876, 1, 0, 0],
            [6877, 1, -1, 0],
            [6878, 2, -1, 0],
            [6879, 2, 0, 0],
        ];
    });
});

item("SmallBedEastAddon", |c| {
    c.addon_deed = "Server.Items.SmallBedEastDeed";
    c.on_create(|i| {
        i.addon_components = [[2653, 0, 0, 0], [2658, 1, 0, 0]];
    });
});

item("SmallBedSouthAddon", |c| {
    c.addon_deed = "Server.Items.SmallBedSouthDeed";
    c.on_create(|i| {
        i.addon_components = [[2659, 0, 0, 0], [2652, 0, 1, 0]];
    });
});

item("SmallForgeAddon", |c| {
    c.addon_deed = "Server.Items.SmallForgeDeed";
    c.on_create(|i| {
        i.addon_components = [[#{ itemId: 4017, type: "Server.Items.ForgeComponent" }, 0, 0, 0]];
    });
});

item("SmallStretchedHideEastAddon", |c| {
    c.addon_deed = "Server.Items.SmallStretchedHideEastDeed";
    c.on_create(|i| {
        i.addon_components = [[4201, 0, 0, 0]];
    });
});

item("SmallStretchedHideSouthAddon", |c| {
    c.addon_deed = "Server.Items.SmallStretchedHideSouthDeed";
    c.on_create(|i| {
        i.addon_components = [[4218, 0, 0, 0]];
    });
});

item("SquareGozaMatSouthAddon", |c| {
    c.addon_retain_deed_hue = true;
    c.addon_deed = "Server.Items.SquareGozaMatSouthDeed";
    c.on_create(|i| {
        i.addon_components = [[#{ itemId: 10409, label: 1030688 }, 0, 0, 0]];
    });
});

item("SquirrelStatueEastAddon", |c| {
    c.addon_deed = "Server.Items.SquirrelStatueEastDeed";
    c.on_create(|i| {
        i.addon_components = [[11536, 0, 0, 0]];
    });
});

item("SquirrelStatueSouthAddon", |c| {
    c.addon_deed = "Server.Items.SquirrelStatueSouthDeed";
    c.on_create(|i| {
        i.addon_components = [[11537, 0, 0, 0]];
    });
});

item("StoneFireplaceEastAddon", |c| {
    c.addon_deed = "Server.Items.StoneFireplaceEastDeed";
    c.on_create(|i| {
        i.addon_components = [[2393, 0, 0, 0], [2387, 0, 1, 0]];
    });
});

item("StoneFireplaceSouthAddon", |c| {
    c.addon_deed = "Server.Items.StoneFireplaceSouthDeed";
    c.on_create(|i| {
        i.addon_components = [[2407, -1, 0, 0], [2401, 0, 0, 0]];
    });
});

item("StoneFountainAddon", |c| {
    c.on_create(|i| {
        i.addon_components = [
            [5937, -2, 1, 0],
            [5938, -1, 1, 0],
            [5939, 0, 1, 0],
            [5940, 1, 1, 0],
            [5941, 1, 0, 0],
            [5942, 1, -1, 0],
            [5943, 1, -2, 0],
            [5944, 0, -2, 0],
            [5945, 0, -1, 0],
            [5946, 0, 0, 0],
            [5947, -1, 0, 0],
            [5948, -2, 0, 0],
            [5949, -2, -1, 0],
            [5950, -1, -1, 0],
            [5951, -1, -2, 0],
            [5953, -2, -2, 0],
        ];
    });
});

item("StoneOvenEastAddon", |c| {
    c.addon_deed = "Server.Items.StoneOvenEastDeed";
    c.on_create(|i| {
        i.addon_components = [[2348, 0, 0, 0], [2347, 0, 1, 0]];
    });
});

item("StoneOvenSouthAddon", |c| {
    c.addon_deed = "Server.Items.StoneOvenSouthDeed";
    c.on_create(|i| {
        i.addon_components = [[2353, -1, 0, 0], [2352, 0, 0, 0]];
    });
});

item("TallElvenBedEastAddon", |c| {
    c.addon_deed = "Server.Items.TallElvenBedEastDeed";
    c.on_create(|i| {
        i.addon_components = [[12372, 0, 0, 0], [12371, 1, 0, 0], [12373, 2, -1, 0], [12370, 2, 0, 0]];
    });
});

item("TallElvenBedSouthAddon", |c| {
    c.addon_deed = "Server.Items.TallElvenBedSouthDeed";
    c.on_create(|i| {
        i.addon_components = [[12376, 0, 0, 0], [12375, -1, 1, 0], [12377, 0, -1, 0], [12374, 0, 1, 0]];
    });
});

item("Telescope", |c| {
    c.on_create(|i| {
        i.addon_components = [
            [5268, 0, 5, 0],
            [5211, 0, 6, 0],
            [5210, 0, 7, 0],
            [5269, 1, 4, 0],
            [5212, 1, 7, 0],
            [5213, 1, 8, 0],
            [5270, 2, 3, 0],
            [5273, 2, 4, 0],
            [5262, 2, 6, 0],
            [5267, 2, 7, 0],
            [5266, 2, 8, 0],
            [5214, 2, 9, 0],
            [5209, 2, 10, 0],
            [5271, 3, 2, 0],
            [5215, 3, 9, 0],
            [5217, 3, 10, 0],
            [5274, 4, 1, 0],
            [5272, 4, 2, 0],
            [5263, 4, 4, 0],
            [5261, 4, 6, 0],
            [5256, 4, 8, 0],
            [5216, 4, 9, 0],
            [5218, 4, 10, 0],
            [5245, 5, 0, 0],
            [5264, 5, 4, 0],
            [5259, 5, 5, 0],
            [5258, 5, 6, 0],
            [5254, 5, 7, 0],
            [5253, 5, 8, 0],
            [5244, 6, 0, 0],
            [5265, 6, 4, 0],
            [5260, 6, 5, 0],
            [5257, 6, 6, 0],
            [5255, 6, 7, 0],
            [5252, 6, 8, 0],
            [5219, 6, 10, 0],
            [5243, 7, 0, 0],
            [5247, 7, 3, 0],
            [5248, 7, 4, 0],
            [5250, 7, 5, 0],
            [5225, 7, 6, 0],
            [5224, 7, 7, 0],
            [5221, 7, 8, 0],
            [5220, 7, 9, 0],
            [5242, 8, 0, 0],
            [5241, 8, 1, 0],
            [5239, 8, 2, 0],
            [5246, 8, 3, 0],
            [5249, 8, 4, 0],
            [5251, 8, 5, 0],
            [5226, 8, 6, 0],
            [5223, 8, 7, 0],
            [5222, 8, 8, 0],
            [5240, 9, 1, 0],
            [5237, 9, 2, 0],
            [5236, 9, 3, 0],
            [5231, 9, 4, 0],
            [5230, 9, 5, 0],
            [5229, 9, 6, 0],
            [5227, 9, 7, 0],
            [5238, 10, 2, 0],
            [5235, 10, 3, 0],
            [5232, 10, 4, 0],
            [5233, 10, 5, 0],
            [5234, 10, 6, 0],
        ];
    });
});

item("WarriorStatueEastAddon", |c| {
    c.addon_deed = "Server.Items.WarriorStatueEastDeed";
    c.on_create(|i| {
        i.addon_components = [[11538, 0, 0, 0]];
    });
});

item("WarriorStatueSouthAddon", |c| {
    c.addon_deed = "Server.Items.WarriorStatueSouthDeed";
    c.on_create(|i| {
        i.addon_components = [[11539, 0, 0, 0]];
    });
});

item("WaterVatEast", |c| {
    c.on_create(|i| {
        i.addon_components = [
            [5464, 0, 0, 0],
            [5342, -1, 1, 0],
            [5458, 0, 1, 0],
            [5343, 1, -1, 0],
            [5460, 1, 0, 0],
            [5465, 1, 1, 0],
            [5456, 1, 3, 0],
            [5461, 3, 1, 0],
            [5335, 2, 2, 0],
            [8612, 2, -1, 0],
            [8612, 3, 0, 0],
            [8612, -1, 2, 0],
            [8612, 0, 3, 0],
        ];
    });
});

item("WaterVatSouth", |c| {
    c.on_create(|i| {
        i.addon_components = [
            [5464, 0, 0, 0],
            [5342, -1, 1, 0],
            [5458, 0, 1, 0],
            [5343, 1, -1, 0],
            [5460, 1, 0, 0],
            [5465, 1, 1, 0],
            [5457, 1, 3, 0],
            [5462, 3, 1, 0],
            [5335, 2, 2, 0],
            [8612, 2, -1, 0],
            [8612, 3, 0, 0],
            [8612, -1, 2, 0],
            [8612, 0, 3, 0],
        ];
    });
});
