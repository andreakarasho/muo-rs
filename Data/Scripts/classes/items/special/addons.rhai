item("BrokenArmoireComponent", |c| {
    c.label = 1076262;
});

item("BrokenArmoireDeed", |c| {
    c.label = 1076262;
    c.deed_addon = "Server.Items.BrokenArmoireAddon";
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("BrokenBedDeed", |c| {
    c.label = 1076263;
    c.deed_addon = "Server.Items.BrokenBedAddon";
    c.deed_selection_number = 1076749;
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("BrokenBookcaseComponent", |c| {
    c.label = 1076258;
});

item("BrokenBookcaseDeed", |c| {
    c.label = 1076258;
    c.deed_addon = "Server.Items.BrokenBookcaseAddon";
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("BrokenChestOfDrawersComponent", |c| {
    c.label = 1076261;
});

item("BrokenChestOfDrawersDeed", |c| {
    c.label = 1076261;
    c.deed_addon = "Server.Items.BrokenChestOfDrawersAddon";
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("BrokenCoveredChairComponent", |c| {
    c.label = 1076257;
});

item("BrokenCoveredChairDeed", |c| {
    c.label = 1076257;
    c.deed_addon = "Server.Items.BrokenCoveredChairAddon";
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("BrokenFallenChairComponent", |c| {
    c.label = 1076264;
});

item("BrokenFallenChairDeed", |c| {
    c.label = 1076264;
    c.deed_addon = "Server.Items.BrokenFallenChairAddon";
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("BrokenVanityDeed", |c| {
    c.label = 1076260;
    c.deed_addon = "Server.Items.BrokenVanityAddon";
    c.deed_selection_number = 1076747;
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("StandingBrokenChairComponent", |c| {
    c.label = 1076259;
});

item("StandingBrokenChairDeed", |c| {
    c.label = 1076259;
    c.deed_addon = "Server.Items.StandingBrokenChairAddon";
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("BoneCouchComponent", |c| {
    c.label = 1074477;
});

item("BoneTableDeed", |c| {
    c.label = 1074478;
    c.deed_addon = "Server.Items.BoneTableAddon";
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("BoneThroneComponent", |c| {
    c.label = 1074476;
});

item("BoneThroneDeed", |c| {
    c.label = 1074476;
    c.deed_addon = "Server.Items.BoneThroneAddon";
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("MountedPixieBlueComponent", |c| {
    c.label = 1074482;
});

item("MountedPixieBlueDeed", |c| {
    c.label = 1074482;
    c.deed_addon = "Server.Items.MountedPixieBlueAddon";
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("MountedPixieGreenComponent", |c| {
    c.label = 1074482;
});

item("MountedPixieGreenDeed", |c| {
    c.label = 1074482;
    c.deed_addon = "Server.Items.MountedPixieGreenAddon";
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("MountedPixieLimeComponent", |c| {
    c.label = 1074482;
});

item("MountedPixieLimeDeed", |c| {
    c.label = 1074482;
    c.deed_addon = "Server.Items.MountedPixieLimeAddon";
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("MountedPixieOrangeComponent", |c| {
    c.label = 1074482;
});

item("MountedPixieOrangeDeed", |c| {
    c.label = 1074482;
    c.deed_addon = "Server.Items.MountedPixieOrangeAddon";
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("MountedPixieWhiteComponent", |c| {
    c.label = 1074482;
});

item("MountedPixieWhiteDeed", |c| {
    c.label = 1074482;
    c.deed_addon = "Server.Items.MountedPixieWhiteAddon";
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("AppleTrunkDeed", |c| {
    c.label = 1076785;
    c.deed_addon = "Server.Items.AppleTrunkAddon";
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("BlueDecorativeRugDeed", |c| {
    c.label = 1076589;
    c.deed_addon = "Server.Items.BlueDecorativeRugAddon";
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("BlueFancyRugDeed", |c| {
    c.label = 1076273;
    c.deed_addon = "Server.Items.BlueFancyRugAddon";
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("BluePlainRugDeed", |c| {
    c.label = 1076585;
    c.deed_addon = "Server.Items.BluePlainRugAddon";
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("BoilingCauldronAddon", |c| {
    c.label = 1076267;
    c.gump_id = 9;
    c.drop_sound = 66;
    c.addon_deed = "Server.Items.BoilingCauldronDeed";
    c.on_create(|i| {
        i.item_id = 0x2068;
        i.addon_components = [[#{ itemId: 4012, label: 1076267 }, 0, 0, 0], [#{ itemId: 2416, label: 1076267 }, 0, 0, 8]];
    });
});

item("BoilingCauldronDeed", |c| {
    c.label = 1076267;
    c.deed_addon = "Server.Items.BoilingCauldronAddon";
});

item("CherryBlossomTreeDeed", |c| {
    c.label = 1076268;
    c.deed_addon = "Server.Items.CherryBlossomTreeAddon";
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("CherryBlossomTrunkDeed", |c| {
    c.label = 1076784;
    c.deed_addon = "Server.Items.CherryBlossomTrunkAddon";
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("CinnamonFancyRugDeed", |c| {
    c.label = 1076587;
    c.deed_addon = "Server.Items.CinnamonFancyRugAddon";
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("GoldenDecorativeRugDeed", |c| {
    c.label = 1076586;
    c.deed_addon = "Server.Items.GoldenDecorativeRugAddon";
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("GuillotineComponent", |c| {
    c.label = 1024656;
});

item("HangingAxesDeed", |c| {
    c.label = 1076271;
    c.deed_addon = "Server.Items.HangingAxesAddon";
    c.deed_selection_number = 1076745;
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("HangingSwordsDeed", |c| {
    c.label = 1076272;
    c.deed_addon = "Server.Items.HangingSwordsAddon";
    c.deed_selection_number = 1076746;
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("LargeFishingNetComponent", |c| {
    c.label = 1076285;
});

item("LargeFishingNetDeed", |c| {
    c.label = 1076285;
    c.deed_addon = "Server.Items.LargeFishingNetAddon";
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("PeachTrunkDeed", |c| {
    c.label = 1076786;
    c.deed_addon = "Server.Items.PeachTrunkAddon";
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("PinkFancyRugDeed", |c| {
    c.label = 1076590;
    c.deed_addon = "Server.Items.PinkFancyRugAddon";
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("RedPlainRugDeed", |c| {
    c.label = 1076588;
    c.deed_addon = "Server.Items.RedPlainRugAddon";
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("ScarecrowComponent", |c| {
    c.label = 1076608;
});

item("ScarecrowDeed", |c| {
    c.label = 1076608;
    c.deed_addon = "Server.Items.ScarecrowAddon";
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("SmallFishingNetComponent", |c| {
    c.label = 1076286;
});

item("SmallFishingNetDeed", |c| {
    c.label = 1076286;
    c.deed_addon = "Server.Items.SmallFishingNetAddon";
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("StoneStatueDeed", |c| {
    c.label = 1076284;
    c.deed_addon = "Server.Items.StoneStatueAddon";
    c.deed_selection_number = 1076579;
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("SuitOfGoldArmorComponent", |c| {
    c.label = 1076265;
});

item("SuitOfGoldArmorDeed", |c| {
    c.label = 1076265;
    c.deed_addon = "Server.Items.SuitOfGoldArmorAddon";
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("SuitOfSilverArmorComponent", |c| {
    c.label = 1076266;
});

item("SuitOfSilverArmorDeed", |c| {
    c.label = 1076266;
    c.deed_addon = "Server.Items.SuitOfSilverArmorAddon";
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("TableWithBlueClothDeed", |c| {
    c.label = 1076276;
    c.deed_addon = "Server.Items.TableWithBlueClothAddon";
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("TableWithOrangeClothDeed", |c| {
    c.label = 1076278;
    c.deed_addon = "Server.Items.TableWithOrangeClothAddon";
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("TableWithPurpleClothDeed", |c| {
    c.label = 1076275;
    c.deed_addon = "Server.Items.TableWithPurpleClothAddon";
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("TableWithRedClothDeed", |c| {
    c.label = 1076277;
    c.deed_addon = "Server.Items.TableWithRedClothAddon";
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("UnmadeBedDeed", |c| {
    c.label = 1076279;
    c.deed_addon = "Server.Items.UnmadeBedAddon";
    c.deed_selection_number = 1076580;
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("VanityAddon", |c| {
    c.label = 1074027;
    c.gump_id = 81;
    c.drop_sound = 66;
    c.addon_deed = "Server.Items.VanityDeed";
    c.addon_item_id_east = 2628;
    c.addon_components_east = [[2629, 0, -1, 0]];
    c.addon_item_id_south = 2620;
    c.addon_components_south = [[2621, -1, 0, 0]];
});

item("VanityDeed", |c| {
    c.label = 1074027;
    c.deed_addon = "Server.Items.VanityAddon";
    c.deed_selection_number = 1076744;
});

item("WallTorchDeed", |c| {
    c.label = 1076282;
    c.deed_addon = "Server.Items.WallTorchAddon";
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("WoodenCoffinComponent", |c| {
    c.label = 1076274;
});

item("WoodenCoffinDeed", |c| {
    c.label = 1076274;
    c.deed_addon = "Server.Items.WoodenCoffinAddon";
    c.deed_selection_number = 1076748;
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("AppleTrunkAddon", |c| {
    c.addon_deed = "Server.Items.AppleTrunkDeed";
    c.on_create(|i| {
        i.addon_components = [[#{ itemId: 3480, label: 1076785 }, 0, 0, 0]];
    });
});

item("BlueDecorativeRugAddon", |c| {
    c.addon_deed = "Server.Items.BlueDecorativeRugDeed";
    c.on_create(|i| {
        i.addon_components = [
            [#{ itemId: 2770, label: 1076589 }, 1, 1, 0],
            [#{ itemId: 2771, label: 1076589 }, -1, -1, 0],
            [#{ itemId: 2772, label: 1076589 }, -1, 1, 0],
            [#{ itemId: 2773, label: 1076589 }, 1, -1, 0],
            [#{ itemId: 2774, label: 1076589 }, -1, 0, 0],
            [#{ itemId: 2775, label: 1076589 }, 0, -1, 0],
            [#{ itemId: 2776, label: 1076589 }, 1, 0, 0],
            [#{ itemId: 2777, label: 1076589 }, 0, 1, 0],
            [#{ itemId: 2769, label: 1076589 }, 0, 0, 0],
        ];
    });
});

item("BlueFancyRugAddon", |c| {
    c.addon_deed = "Server.Items.BlueFancyRugDeed";
    c.on_create(|i| {
        i.addon_components = [
            [#{ itemId: 2754, label: 1076273 }, 1, 1, 0],
            [#{ itemId: 2755, label: 1076273 }, -1, -1, 0],
            [#{ itemId: 2756, label: 1076273 }, -1, 1, 0],
            [#{ itemId: 2757, label: 1076273 }, 1, -1, 0],
            [#{ itemId: 2806, label: 1076273 }, -1, 0, 0],
            [#{ itemId: 2807, label: 1076273 }, 0, -1, 0],
            [#{ itemId: 2808, label: 1076273 }, 1, 0, 0],
            [#{ itemId: 2809, label: 1076273 }, 0, 1, 0],
            [#{ itemId: 2810, label: 1076273 }, 0, 0, 0],
        ];
    });
});

item("BluePlainRugAddon", |c| {
    c.addon_deed = "Server.Items.BluePlainRugDeed";
    c.on_create(|i| {
        i.addon_components = [
            [#{ itemId: 2754, label: 1076585 }, 1, 1, 0],
            [#{ itemId: 2755, label: 1076585 }, -1, -1, 0],
            [#{ itemId: 2756, label: 1076585 }, -1, 1, 0],
            [#{ itemId: 2757, label: 1076585 }, 1, -1, 0],
            [#{ itemId: 2806, label: 1076585 }, -1, 0, 0],
            [#{ itemId: 2807, label: 1076585 }, 0, -1, 0],
            [#{ itemId: 2808, label: 1076585 }, 1, 0, 0],
            [#{ itemId: 2809, label: 1076585 }, 0, 1, 0],
            [#{ itemId: 2752, label: 1076585 }, 0, 0, 0],
        ];
    });
});

item("BoneTableAddon", |c| {
    c.addon_deed = "Server.Items.BoneTableDeed";
    c.on_create(|i| {
        i.addon_components = [[#{ itemId: 10844, label: 1074478 }, 0, 0, 0]];
    });
});

item("BoneThroneAddon", |c| {
    c.addon_deed = "Server.Items.BoneThroneDeed";
    c.on_create(|i| {
        i.addon_components = [[#{ itemId: 10840, type: "Server.Items.BoneThroneComponent" }, 0, 0, 0]];
    });
});

item("BrokenArmoireAddon", |c| {
    c.addon_deed = "Server.Items.BrokenArmoireDeed";
    c.on_create(|i| {
        i.addon_components = [[#{ itemId: 3090, type: "Server.Items.BrokenArmoireComponent" }, 0, 0, 0]];
    });
});

item("BrokenBedAddon", |c| {
    c.addon_deed = "Server.Items.BrokenBedDeed";
    c.addon_components_east = [
        [#{ itemId: 6293, label: 1076263 }, 0, 0, 0],
        [#{ itemId: 6292, label: 1076263 }, 0, 1, 0],
        [#{ itemId: 6295, label: 1076263 }, 1, 0, 0],
        [#{ itemId: 6294, label: 1076263 }, 1, 1, 0],
    ];
    c.addon_components_south = [
        [#{ itemId: 6297, label: 1076263 }, 0, 0, 0],
        [#{ itemId: 6296, label: 1076263 }, 1, 0, 0],
        [#{ itemId: 6299, label: 1076263 }, 0, 1, 0],
        [#{ itemId: 6298, label: 1076263 }, 1, 1, 0],
    ];
});

item("BrokenBookcaseAddon", |c| {
    c.addon_deed = "Server.Items.BrokenBookcaseDeed";
    c.on_create(|i| {
        i.addon_components = [[#{ itemId: 3092, type: "Server.Items.BrokenBookcaseComponent" }, 0, 0, 0]];
    });
});

item("BrokenChestOfDrawersAddon", |c| {
    c.addon_deed = "Server.Items.BrokenChestOfDrawersDeed";
    c.on_create(|i| {
        i.addon_components = [[#{ itemId: 3108, type: "Server.Items.BrokenChestOfDrawersComponent" }, 0, 0, 0]];
    });
});

item("BrokenCoveredChairAddon", |c| {
    c.addon_deed = "Server.Items.BrokenCoveredChairDeed";
    c.on_create(|i| {
        i.addon_components = [[#{ itemId: 3095, type: "Server.Items.BrokenCoveredChairComponent" }, 0, 0, 0]];
    });
});

item("BrokenFallenChairAddon", |c| {
    c.addon_deed = "Server.Items.BrokenFallenChairDeed";
    c.on_create(|i| {
        i.addon_components = [[#{ itemId: 3097, type: "Server.Items.BrokenFallenChairComponent" }, 0, 0, 0]];
    });
});

item("BrokenVanityAddon", |c| {
    c.addon_deed = "Server.Items.BrokenVanityDeed";
    c.addon_components_east = [[#{ itemId: 3104, label: 1076260 }, 0, 0, 0], [#{ itemId: 3105, label: 1076260 }, 0, -1, 0]];
    c.addon_components_south = [[#{ itemId: 3106, label: 1076260 }, 0, 0, 0], [#{ itemId: 3107, label: 1076260 }, -1, 0, 0]];
});

item("CherryBlossomTreeAddon", |c| {
    c.addon_deed = "Server.Items.CherryBlossomTreeDeed";
    c.on_create(|i| {
        i.addon_components = [[#{ itemId: 9966, label: 1076268 }, 0, 0, 0], [#{ itemId: 12578, label: 1076268 }, 0, 0, 0]];
    });
});

item("CherryBlossomTrunkAddon", |c| {
    c.addon_deed = "Server.Items.CherryBlossomTrunkDeed";
    c.on_create(|i| {
        i.addon_components = [[#{ itemId: 9966, label: 1076784 }, 0, 0, 0]];
    });
});

item("CinnamonFancyRugAddon", |c| {
    c.addon_deed = "Server.Items.CinnamonFancyRugDeed";
    c.on_create(|i| {
        i.addon_components = [
            [#{ itemId: 2787, label: 1076587 }, 1, 1, 0],
            [#{ itemId: 2788, label: 1076587 }, -1, -1, 0],
            [#{ itemId: 2789, label: 1076587 }, -1, 1, 0],
            [#{ itemId: 2790, label: 1076587 }, 1, -1, 0],
            [#{ itemId: 2791, label: 1076587 }, -1, 0, 0],
            [#{ itemId: 2792, label: 1076587 }, 0, -1, 0],
            [#{ itemId: 2793, label: 1076587 }, 1, 0, 0],
            [#{ itemId: 2794, label: 1076587 }, 0, 1, 0],
            [#{ itemId: 2795, label: 1076587 }, 0, 0, 0],
        ];
    });
});

item("GoldenDecorativeRugAddon", |c| {
    c.addon_deed = "Server.Items.GoldenDecorativeRugDeed";
    c.on_create(|i| {
        i.addon_components = [
            [#{ itemId: 2779, label: 1076586 }, 1, 1, 0],
            [#{ itemId: 2780, label: 1076586 }, -1, -1, 0],
            [#{ itemId: 2781, label: 1076586 }, -1, 1, 0],
            [#{ itemId: 2782, label: 1076586 }, 1, -1, 0],
            [#{ itemId: 2783, label: 1076586 }, -1, 0, 0],
            [#{ itemId: 2784, label: 1076586 }, 0, -1, 0],
            [#{ itemId: 2785, label: 1076586 }, 1, 0, 0],
            [#{ itemId: 2786, label: 1076586 }, 0, 1, 0],
            [#{ itemId: 2778, label: 1076586 }, 0, 0, 0],
        ];
    });
});

item("HangingAxesAddon", |c| {
    c.addon_deed = "Server.Items.HangingAxesDeed";
    c.addon_components_east = [[#{ itemId: 5482, label: 1076271 }, 0, 0, 0], [#{ itemId: 5483, label: 1076271 }, 0, -1, 0]];
    c.addon_components_south = [[#{ itemId: 5480, label: 1076271 }, 0, 0, 0], [#{ itemId: 5481, label: 1076271 }, 1, 0, 0]];
});

item("HangingSwordsAddon", |c| {
    c.addon_deed = "Server.Items.HangingSwordsDeed";
    c.addon_components_east = [[#{ itemId: 5478, label: 1076272 }, 0, 0, 0], [#{ itemId: 5479, label: 1076272 }, 0, -1, 0]];
    c.addon_components_south = [[#{ itemId: 5476, label: 1076272 }, 0, 0, 0], [#{ itemId: 5477, label: 1076272 }, 1, 0, 0]];
});

item("LargeFishingNetAddon", |c| {
    c.addon_deed = "Server.Items.LargeFishingNetDeed";
    c.on_create(|i| {
        i.addon_components = [[#{ itemId: 15758, type: "Server.Items.LargeFishingNetComponent" }, 0, 0, 0]];
    });
});

item("MountedPixieBlueAddon", |c| {
    c.addon_deed = "Server.Items.MountedPixieBlueDeed";
    c.on_create(|i| {
        i.addon_components = [[#{ itemId: 10869, type: "Server.Items.MountedPixieBlueComponent" }, 0, 0, 0]];
    });
});

item("MountedPixieGreenAddon", |c| {
    c.addon_deed = "Server.Items.MountedPixieGreenDeed";
    c.on_create(|i| {
        i.addon_components = [[#{ itemId: 10865, type: "Server.Items.MountedPixieGreenComponent" }, 0, 0, 0]];
    });
});

item("MountedPixieLimeAddon", |c| {
    c.addon_deed = "Server.Items.MountedPixieLimeDeed";
    c.on_create(|i| {
        i.addon_components = [[#{ itemId: 10871, type: "Server.Items.MountedPixieLimeComponent" }, 0, 0, 0]];
    });
});

item("MountedPixieOrangeAddon", |c| {
    c.addon_deed = "Server.Items.MountedPixieOrangeDeed";
    c.on_create(|i| {
        i.addon_components = [[#{ itemId: 10867, type: "Server.Items.MountedPixieOrangeComponent" }, 0, 0, 0]];
    });
});

item("MountedPixieWhiteAddon", |c| {
    c.addon_deed = "Server.Items.MountedPixieWhiteDeed";
    c.on_create(|i| {
        i.addon_components = [[#{ itemId: 10873, type: "Server.Items.MountedPixieWhiteComponent" }, 0, 0, 0]];
    });
});

item("PeachTrunkAddon", |c| {
    c.addon_deed = "Server.Items.PeachTrunkDeed";
    c.on_create(|i| {
        i.addon_components = [[#{ itemId: 3484, label: 1076786 }, 0, 0, 0]];
    });
});

item("PinkFancyRugAddon", |c| {
    c.addon_deed = "Server.Items.PinkFancyRugDeed";
    c.on_create(|i| {
        i.addon_components = [
            [#{ itemId: 2798, label: 1076590 }, 1, 1, 0],
            [#{ itemId: 2799, label: 1076590 }, -1, -1, 0],
            [#{ itemId: 2800, label: 1076590 }, -1, 1, 0],
            [#{ itemId: 2801, label: 1076590 }, 1, -1, 0],
            [#{ itemId: 2802, label: 1076590 }, -1, 0, 0],
            [#{ itemId: 2803, label: 1076590 }, 0, -1, 0],
            [#{ itemId: 2804, label: 1076590 }, 1, 0, 0],
            [#{ itemId: 2805, label: 1076590 }, 0, 1, 0],
            [#{ itemId: 2796, label: 1076590 }, 0, 0, 0],
        ];
    });
});

item("RedPlainRugAddon", |c| {
    c.addon_deed = "Server.Items.RedPlainRugDeed";
    c.on_create(|i| {
        i.addon_components = [
            [#{ itemId: 2761, label: 1076588 }, 1, 1, 0],
            [#{ itemId: 2762, label: 1076588 }, -1, -1, 0],
            [#{ itemId: 2763, label: 1076588 }, -1, 1, 0],
            [#{ itemId: 2764, label: 1076588 }, 1, -1, 0],
            [#{ itemId: 2765, label: 1076588 }, -1, 0, 0],
            [#{ itemId: 2766, label: 1076588 }, 0, -1, 0],
            [#{ itemId: 2767, label: 1076588 }, 1, 0, 0],
            [#{ itemId: 2768, label: 1076588 }, 0, 1, 0],
            [#{ itemId: 2758, label: 1076588 }, 0, 0, 0],
        ];
    });
});

item("ScarecrowAddon", |c| {
    c.addon_deed = "Server.Items.ScarecrowDeed";
    c.on_create(|i| {
        i.addon_components = [[#{ itemId: 7732, type: "Server.Items.ScarecrowComponent" }, 0, 0, 0]];
    });
});

item("SmallFishingNetAddon", |c| {
    c.addon_deed = "Server.Items.SmallFishingNetDeed";
    c.on_create(|i| {
        i.addon_components = [[#{ itemId: 7843, type: "Server.Items.SmallFishingNetComponent" }, 0, 0, 0]];
    });
});

item("StandingBrokenChairAddon", |c| {
    c.addon_deed = "Server.Items.StandingBrokenChairDeed";
    c.on_create(|i| {
        i.addon_components = [[#{ itemId: 3099, type: "Server.Items.StandingBrokenChairComponent" }, 0, 0, 0]];
    });
});

item("StoneStatueAddon", |c| {
    c.addon_deed = "Server.Items.StoneStatueDeed";
    c.addon_components_east = [
        [#{ itemId: 5022, label: 1076284 }, 0, 0, 0],
        [#{ itemId: 5023, label: 1076284 }, -1, 0, 0],
        [#{ itemId: 5024, label: 1076284 }, 0, -1, 0],
    ];
    c.addon_components_south = [
        [#{ itemId: 4767, label: 1076284 }, 0, 0, 0],
        [#{ itemId: 4768, label: 1076284 }, 0, -1, 0],
        [#{ itemId: 4769, label: 1076284 }, -1, 0, 0],
    ];
});

item("SuitOfGoldArmorAddon", |c| {
    c.addon_deed = "Server.Items.SuitOfGoldArmorDeed";
    c.on_create(|i| {
        i.addon_components = [[#{ itemId: 15786, type: "Server.Items.SuitOfGoldArmorComponent" }, 0, 0, 0]];
    });
});

item("SuitOfSilverArmorAddon", |c| {
    c.addon_deed = "Server.Items.SuitOfSilverArmorDeed";
    c.on_create(|i| {
        i.addon_components = [[#{ itemId: 15750, type: "Server.Items.SuitOfSilverArmorComponent" }, 0, 0, 0]];
    });
});

item("TableWithBlueClothAddon", |c| {
    c.addon_deed = "Server.Items.TableWithBlueClothDeed";
    c.on_create(|i| {
        i.addon_components = [[#{ itemId: 4492, label: 1076276 }, 0, 0, 0]];
    });
});

item("TableWithOrangeClothAddon", |c| {
    c.addon_deed = "Server.Items.TableWithOrangeClothDeed";
    c.on_create(|i| {
        i.addon_components = [[#{ itemId: 4494, label: 1076278 }, 0, 0, 0]];
    });
});

item("TableWithPurpleClothAddon", |c| {
    c.addon_deed = "Server.Items.TableWithPurpleClothDeed";
    c.on_create(|i| {
        i.addon_components = [[#{ itemId: 4491, label: 1076275 }, 0, 0, 0]];
    });
});

item("TableWithRedClothAddon", |c| {
    c.addon_deed = "Server.Items.TableWithRedClothDeed";
    c.on_create(|i| {
        i.addon_components = [[#{ itemId: 4493, label: 1076277 }, 0, 0, 0]];
    });
});

item("UnmadeBedAddon", |c| {
    c.addon_deed = "Server.Items.UnmadeBedDeed";
    c.addon_components_east = [
        [#{ itemId: 2700, label: 1076279 }, 0, 0, 0],
        [#{ itemId: 2701, label: 1076279 }, 0, -1, 0],
        [#{ itemId: 2704, label: 1076279 }, -1, 0, 0],
        [#{ itemId: 2705, label: 1076279 }, -1, -1, 0],
    ];
    c.addon_components_south = [
        [#{ itemId: 3504, label: 1076279 }, 0, 0, 0],
        [#{ itemId: 3505, label: 1076279 }, -1, 0, 0],
        [#{ itemId: 3508, label: 1076279 }, 0, -1, 0],
        [#{ itemId: 3509, label: 1076279 }, -1, -1, 0],
    ];
});

item("WallTorchAddon", |c| {
    c.addon_deed = "Server.Items.WallTorchDeed";
    c.on_create(|i| {
        i.addon_components = [[#{ itemId: 15768, type: "Server.Items.WallTorchComponent" }, 0, 0, 0]];
    });
});

item("WoodenCoffinAddon", |c| {
    c.addon_deed = "Server.Items.WoodenCoffinDeed";
    c.addon_components_east = [
        [#{ itemId: 7233, type: "Server.Items.WoodenCoffinComponent" }, 0, 0, 0],
        [#{ itemId: 7234, type: "Server.Items.WoodenCoffinComponent" }, 1, 0, 0],
        [#{ itemId: 7235, type: "Server.Items.WoodenCoffinComponent" }, 2, 0, 0],
    ];
    c.addon_components_south = [
        [#{ itemId: 7247, type: "Server.Items.WoodenCoffinComponent" }, 0, 0, 0],
        [#{ itemId: 7248, type: "Server.Items.WoodenCoffinComponent" }, 0, 1, 0],
        [#{ itemId: 7249, type: "Server.Items.WoodenCoffinComponent" }, 0, 2, 0],
    ];
});
