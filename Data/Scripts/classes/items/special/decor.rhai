item("CurtainsComponent", |c| {
    c.label = 1076280;
});

item("CurtainsDeed", |c| {
    c.label = 1076280;
});

item("HouseLadderDeed", |c| {
    c.label = 1076287;
});

item("BedOfNailsComponent", |c| {
    c.label = 1074801;
});

item("HaunterMirrorComponent", |c| {
    c.label = 1074800;
});

item("CreepyPortraitComponent", |c| {
    c.label = 1074481;
});

item("AwesomeDisturbingPortraitComponent", |c| {
    c.label = 1074479;
});

item("SacrificialAltarAddon", |c| {
    c.label = 1074818;
    c.max_weight = 0;
    c.gump_id = 263;
    c.drop_sound = 66;
});

item("SacrificialAltarDeed", |c| {
    c.label = 1074818;
});

item("GingerBreadHouseDeed", |c| {
    c.weight = 1.0;
    c.label = 1077394;
});

item("GuillotineAddon", |c| {
    c.addon_deed = "Server.Items.GuillotineDeed";
    c.on_create(|i| {
        i.addon_components = [[#{ itemId: 4702, type: "Server.Items.GuillotineComponent" }, 0, 0, 0]];
    });
});

item("IronMaidenAddon", |c| {
    c.addon_deed = "Server.Items.IronMaidenDeed";
    c.on_create(|i| {
        i.addon_components = [[#{ itemId: 4681, label: 1076288 }, 0, 0, 0]];
    });
});

item("FountainAddon", |c| {
    c.addon_deed = "Server.Items.FountainDeed";
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

item("HaunterMirrorAddon", |c| {
    c.addon_deed = "Server.Items.HaunterMirrorDeed";
    c.on_create(|i| {
        i.addon_components = [[#{ itemId: 10875, type: "Server.Items.HaunterMirrorComponent" }, 0, 0, 0]];
    });
});

item("GuillotineDeed", |c| {
    c.label = 1024656;
    c.deed_addon = "Server.Items.GuillotineAddon";
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("IronMaidenDeed", |c| {
    c.label = 1076288;
    c.deed_addon = "Server.Items.IronMaidenAddon";
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("FountainDeed", |c| {
    c.label = 1076283;
    c.deed_addon = "Server.Items.FountainAddon";
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("BedOfNailsDeed", |c| {
    c.label = 1074801;
    c.deed_addon = "Server.Items.BedOfNailsAddon";
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("BoneCouchDeed", |c| {
    c.label = 1074477;
    c.deed_addon = "Server.Items.BoneCouchAddon";
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("HaunterMirrorDeed", |c| {
    c.label = 1074800;
    c.deed_addon = "Server.Items.HaunterMirrorAddon";
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("CurtainsAddon", |c| {
    c.addon_deed = "Server.Items.CurtainsDeed";
});

item("HouseLadderAddon", |c| {
    c.addon_deed = "Server.Items.HouseLadderDeed";
});

item("BedOfNailsAddon", |c| {
    c.addon_deed = "Server.Items.BedOfNailsDeed";
});

item("BoneCouchAddon", |c| {
    c.addon_deed = "Server.Items.BoneCouchDeed";
});

item("AppleTreeAddon", |c| {
    c.addon_deed = "Server.Items.AppleTreeDeed";
    c.on_create(|i| {
        i.addon_components = [[#{ itemId: 3480, label: 1076269 }, 0, 0, 0], [#{ itemId: 12580, label: 1076269 }, 0, 0, 0]];
    });
});

item("AppleTreeDeed", |c| {
    c.label = 1076269;
    c.deed_addon = "Server.Items.AppleTreeAddon";
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("PeachTreeAddon", |c| {
    c.addon_deed = "Server.Items.PeachTreeDeed";
    c.on_create(|i| {
        i.addon_components = [[#{ itemId: 3484, label: 1076270 }, 0, 0, 0], [#{ itemId: 12579, label: 1076270 }, 0, 0, 0]];
    });
});

item("PeachTreeDeed", |c| {
    c.label = 1076270;
    c.deed_addon = "Server.Items.PeachTreeAddon";
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("CreepyPortraitAddon", |c| {
    c.addon_deed = "Server.Items.CreepyPortraitDeed";
});

item("CreepyPortraitDeed", |c| {
    c.label = 1074481;
    c.deed_addon = "Server.Items.CreepyPortraitAddon";
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("DisturbingPortraitAddon", |c| {
    c.addon_deed = "Server.Items.DisturbingPortraitDeed";
});

item("DisturbingPortraitDeed", |c| {
    c.label = 1074479;
    c.deed_addon = "Server.Items.DisturbingPortraitAddon";
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("UnsettlingPortraitAddon", |c| {
    c.addon_deed = "Server.Items.UnsettlingPortraitDeed";
});

item("UnsettlingPortraitDeed", |c| {
    c.label = 1074480;
    c.deed_addon = "Server.Items.UnsettlingPortraitAddon";
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("AwesomeDisturbingPortraitAddon", |c| {
    c.addon_deed = "Server.Items.AwesomeDisturbingPortraitDeed";
});

item("AwesomeDisturbingPortraitDeed", |c| {
    c.label = 1074479;
    c.deed_addon = "Server.Items.AwesomeDisturbingPortraitAddon";
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});
