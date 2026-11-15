item("Server.Multis.Deeds.HouseDeed", |c| {
    c.on_create(|i| {
        i.item_id = 0x14F0;
        i.loot_type = "Newbied";
    });
});

item("StonePlasterHouseDeed", |c| {
    c.inherits = "Server.Multis.Deeds.HouseDeed";
    c.multi_id = 100;
    c.offset = [0, 4, 0];
    c.label = 1041211;
});

item("FieldStoneHouseDeed", |c| {
    c.inherits = "Server.Multis.Deeds.HouseDeed";
    c.multi_id = 102;
    c.offset = [0, 4, 0];
    c.label = 1041212;
});

item("SmallBrickHouseDeed", |c| {
    c.inherits = "Server.Multis.Deeds.HouseDeed";
    c.multi_id = 104;
    c.offset = [0, 4, 0];
    c.label = 1041213;
});

item("WoodHouseDeed", |c| {
    c.inherits = "Server.Multis.Deeds.HouseDeed";
    c.multi_id = 106;
    c.offset = [0, 4, 0];
    c.label = 1041214;
});

item("WoodPlasterHouseDeed", |c| {
    c.inherits = "Server.Multis.Deeds.HouseDeed";
    c.multi_id = 108;
    c.offset = [0, 4, 0];
    c.label = 1041215;
});

item("ThatchedRoofCottageDeed", |c| {
    c.inherits = "Server.Multis.Deeds.HouseDeed";
    c.multi_id = 110;
    c.offset = [0, 4, 0];
    c.label = 1041216;
});

item("BrickHouseDeed", |c| {
    c.inherits = "Server.Multis.Deeds.HouseDeed";
    c.multi_id = 116;
    c.offset = [-1, 7, 0];
    c.label = 1041219;
});

item("TwoStoryWoodPlasterHouseDeed", |c| {
    c.inherits = "Server.Multis.Deeds.HouseDeed";
    c.multi_id = 118;
    c.offset = [-3, 7, 0];
    c.label = 1041220;
});

item("TwoStoryStonePlasterHouseDeed", |c| {
    c.inherits = "Server.Multis.Deeds.HouseDeed";
    c.multi_id = 120;
    c.offset = [-3, 7, 0];
    c.label = 1041221;
});

item("TowerDeed", |c| {
    c.inherits = "Server.Multis.Deeds.HouseDeed";
    c.multi_id = 122;
    c.offset = [0, 7, 0];
    c.label = 1041222;
});

item("KeepDeed", |c| {
    c.inherits = "Server.Multis.Deeds.HouseDeed";
    c.multi_id = 124;
    c.offset = [0, 11, 0];
    c.label = 1041223;
});

item("CastleDeed", |c| {
    c.inherits = "Server.Multis.Deeds.HouseDeed";
    c.multi_id = 126;
    c.offset = [0, 16, 0];
    c.label = 1041224;
});

item("LargePatioDeed", |c| {
    c.inherits = "Server.Multis.Deeds.HouseDeed";
    c.multi_id = 140;
    c.offset = [-4, 7, 0];
    c.label = 1041231;
});

item("LargeMarbleDeed", |c| {
    c.inherits = "Server.Multis.Deeds.HouseDeed";
    c.multi_id = 150;
    c.offset = [-4, 7, 0];
    c.label = 1041236;
});

item("SmallTowerDeed", |c| {
    c.inherits = "Server.Multis.Deeds.HouseDeed";
    c.multi_id = 152;
    c.offset = [3, 4, 0];
    c.label = 1041237;
});

item("LogCabinDeed", |c| {
    c.inherits = "Server.Multis.Deeds.HouseDeed";
    c.multi_id = 154;
    c.offset = [1, 6, 0];
    c.label = 1041238;
});

item("SandstonePatioDeed", |c| {
    c.inherits = "Server.Multis.Deeds.HouseDeed";
    c.multi_id = 156;
    c.offset = [-1, 4, 0];
    c.label = 1041239;
});

item("VillaDeed", |c| {
    c.inherits = "Server.Multis.Deeds.HouseDeed";
    c.multi_id = 158;
    c.offset = [3, 6, 0];
    c.label = 1041240;
});

item("StoneWorkshopDeed", |c| {
    c.inherits = "Server.Multis.Deeds.HouseDeed";
    c.multi_id = 160;
    c.offset = [-1, 4, 0];
    c.label = 1041241;
});

item("MarbleWorkshopDeed", |c| {
    c.inherits = "Server.Multis.Deeds.HouseDeed";
    c.multi_id = 162;
    c.offset = [-1, 4, 0];
    c.label = 1041242;
});

item("Server.Multis.BaseBoatDeed", |c| {
    c.on_create(|i| {
        i.item_id = 0x14F2;
        if era("AOS") {} else {
            i.loot_type = "Newbied";
        }
    });
});

item("SmallBoatDeed", |c| {
    c.inherits = "Server.Multis.BaseBoatDeed";
    c.multi_id = 0;
    c.offset = [0, 0, 0];
    c.label = 1041205;
});

item("SmallDragonBoatDeed", |c| {
    c.inherits = "Server.Multis.BaseBoatDeed";
    c.multi_id = 4;
    c.offset = [0, 0, 0];
    c.label = 1041206;
});

item("MediumBoatDeed", |c| {
    c.inherits = "Server.Multis.BaseBoatDeed";
    c.multi_id = 8;
    c.offset = [0, 0, 0];
    c.label = 1041207;
});

item("MediumDragonBoatDeed", |c| {
    c.inherits = "Server.Multis.BaseBoatDeed";
    c.multi_id = 12;
    c.offset = [0, 0, 0];
    c.label = 1041208;
});

item("LargeBoatDeed", |c| {
    c.inherits = "Server.Multis.BaseBoatDeed";
    c.multi_id = 16;
    c.offset = [0, -1, 0];
    c.label = 1041209;
});

item("LargeDragonBoatDeed", |c| {
    c.inherits = "Server.Multis.BaseBoatDeed";
    c.multi_id = 20;
    c.offset = [0, -1, 0];
    c.label = 1041210;
});

item("HousePlacementTool", |c| {
    c.weight = 3.0;
    c.label = 1060651;
    c.on_create(|i| {
        i.item_id = 0x14F6;
        i.loot_type = "Blessed";
    });
});
