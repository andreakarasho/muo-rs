item("Server.Items.BaseBagBall", |c| {
    c.weight = 1.0;
});

item("BankCheck", |c| {
    c.label = 1041361;
    c.weight = 1.0;
});

item("Key", |c| {
    c.weight = 1.0;
});

item("KeyRing", |c| {
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0x1011;
    });
});

item("FruitBasket", |c| {
    c.weight = 2.0;
    c.on_create(|i| {
        i.item_id = 0x993;
        i.food_fill_factor = 5;
        i.stackable = false;
    });
});

item("ResGate", |c| {
    c.name = "a resurrection gate";
    c.on_create(|i| {
        i.item_id = 0xF6C;
        i.movable = false;
        i.hue = 0x2D1;
        i.light = "Circle300";
    });
});

item("GamblingStone", |c| {
    c.name = "a gambling stone";
    c.on_create(|i| {
        i.item_id = 0xED4;
        i.movable = false;
        i.hue = 0x56;
    });
});

item("Backpack", |c| {
    c.weight = 3.0;
    c.on_create(|i| {
        i.item_id = 0xE75;
        i.layer = "Backpack";
    });
});

item("StrongBackpack", |c| {
    c.weight = 13.0;
    c.on_create(|i| {
        i.item_id = 0xE75;
        i.layer = "Backpack";
    });
});

item("CreatureBackpack", |c| {
    c.weight = 3.0;
    c.on_create(|i| {
        i.item_id = 0xE75;
        i.layer = "Backpack";
        i.hue = 5;
    });
});

item("Pouch", |c| {
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0xE79;
    });
});

item("SmallBagBall", |c| {
    c.on_create(|i| {
        i.item_id = 0x2256;
    });
});

item("Bag", |c| {
    c.weight = 2.0;
    c.on_create(|i| {
        i.item_id = 0xE76;
    });
});

item("Barrel", |c| {
    c.weight = 25.0;
    c.on_create(|i| {
        i.item_id = 0xE77;
    });
});

item("Keg", |c| {
    c.weight = 15.0;
    c.on_create(|i| {
        i.item_id = 0xE7F;
    });
});

item("PicnicBasket", |c| {
    c.weight = 2.0;
    c.on_create(|i| {
        i.item_id = 0xE7A;
    });
});

item("Basket", |c| {
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0x990;
    });
});

item("RarewoodChest", |c| {
    c.on_create(|i| {
        i.item_id = 0x2DF1;
    });
});

item("WoodenFootLocker", |c| {
    c.on_create(|i| {
        i.item_id = 0x2811;
        i.set = #{ GumpID: 267 };
    });
});

item("LargeBagBall", |c| {
    c.on_create(|i| {
        i.item_id = 0x2257;
    });
});

item("DecorativeBox", |c| {
    c.on_create(|i| {
        i.item_id = 0x2DF4;
    });
});

item("WoodenBox", |c| {
    c.weight = 4.0;
    c.on_create(|i| {
        i.item_id = 0x9AA;
    });
});

item("SmallCrate", |c| {
    c.weight = 2.0;
    c.on_create(|i| {
        i.item_id = 0x9A9;
    });
});

item("MediumCrate", |c| {
    c.weight = 2.0;
    c.on_create(|i| {
        i.item_id = 0xE3F;
    });
});

item("LargeCrate", |c| {
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0xE3D;
    });
});

item("MetalBox", |c| {
    c.on_create(|i| {
        i.item_id = 0x9A8;
    });
});

item("MetalChest", |c| {
    c.on_create(|i| {
        i.item_id = 0x9AB;
    });
});

item("MetalGoldenChest", |c| {
    c.on_create(|i| {
        i.item_id = 0xE41;
    });
});

item("WoodenChest", |c| {
    c.weight = 2.0;
    c.on_create(|i| {
        i.item_id = 0xE43;
    });
});

item("PlainWoodenChest", |c| {
    c.on_create(|i| {
        i.item_id = 0x280B;
    });
});

item("OrnateWoodenChest", |c| {
    c.on_create(|i| {
        i.item_id = 0x280D;
    });
});

item("GildedWoodenChest", |c| {
    c.on_create(|i| {
        i.item_id = 0x280F;
    });
});

item("FinishedWoodenChest", |c| {
    c.on_create(|i| {
        i.item_id = 0x2813;
    });
});

item("Log", |c| {
    c.resource = "RegularWood";
});

item("Board", |c| {
    c.resource = "RegularWood";
});

item("IronIngot", |c| {
    c.resource = "Iron";
});

item("DullCopperIngot", |c| {
    c.resource = "DullCopper";
});

item("ShadowIronIngot", |c| {
    c.resource = "ShadowIron";
});

item("CopperIngot", |c| {
    c.resource = "Copper";
});

item("BronzeIngot", |c| {
    c.resource = "Bronze";
});

item("GoldIngot", |c| {
    c.resource = "Gold";
});

item("AgapiteIngot", |c| {
    c.resource = "Agapite";
});

item("VeriteIngot", |c| {
    c.resource = "Verite";
});

item("ValoriteIngot", |c| {
    c.resource = "Valorite";
});

item("IronOre", |c| {
    c.resource = "Iron";
});

item("DullCopperOre", |c| {
    c.resource = "DullCopper";
});

item("ShadowIronOre", |c| {
    c.resource = "ShadowIron";
});

item("CopperOre", |c| {
    c.resource = "Copper";
});

item("BronzeOre", |c| {
    c.resource = "Bronze";
});

item("GoldOre", |c| {
    c.resource = "Gold";
});

item("AgapiteOre", |c| {
    c.resource = "Agapite";
});

item("VeriteOre", |c| {
    c.resource = "Verite";
});

item("ValoriteOre", |c| {
    c.resource = "Valorite";
});

item("HeartwoodLog", |c| {
    c.resource = "Heartwood";
});

item("BloodwoodLog", |c| {
    c.resource = "Bloodwood";
});

item("FrostwoodLog", |c| {
    c.resource = "Frostwood";
});

item("OakLog", |c| {
    c.resource = "OakWood";
});

item("AshLog", |c| {
    c.resource = "AshWood";
});

item("YewLog", |c| {
    c.resource = "YewWood";
});

item("HeartwoodBoard", |c| {
    c.resource = "Heartwood";
});

item("BloodwoodBoard", |c| {
    c.resource = "Bloodwood";
});

item("FrostwoodBoard", |c| {
    c.resource = "Frostwood";
});

item("OakBoard", |c| {
    c.resource = "OakWood";
});

item("AshBoard", |c| {
    c.resource = "AshWood";
});

item("YewBoard", |c| {
    c.resource = "YewWood";
});

item("Leather", |c| {
    c.resource = "RegularLeather";
});

item("SpinedLeather", |c| {
    c.resource = "SpinedLeather";
});

item("HornedLeather", |c| {
    c.resource = "HornedLeather";
});

item("BarbedLeather", |c| {
    c.resource = "BarbedLeather";
});

item("Hides", |c| {
    c.resource = "RegularLeather";
});

item("SpinedHides", |c| {
    c.resource = "SpinedLeather";
});

item("HornedHides", |c| {
    c.resource = "HornedLeather";
});

item("BarbedHides", |c| {
    c.resource = "BarbedLeather";
});

item("RedScales", |c| {
    c.resource = "RedScales";
});

item("YellowScales", |c| {
    c.resource = "YellowScales";
});

item("BlackScales", |c| {
    c.resource = "BlackScales";
});

item("GreenScales", |c| {
    c.resource = "GreenScales";
});

item("WhiteScales", |c| {
    c.resource = "WhiteScales";
});

item("BlueScales", |c| {
    c.resource = "BlueScales";
});

item("Granite", |c| {
    c.resource = "Iron";
});

item("DullCopperGranite", |c| {
    c.resource = "DullCopper";
});

item("ShadowIronGranite", |c| {
    c.resource = "ShadowIron";
});

item("CopperGranite", |c| {
    c.resource = "Copper";
});

item("BronzeGranite", |c| {
    c.resource = "Bronze";
});

item("GoldGranite", |c| {
    c.resource = "Gold";
});

item("AgapiteGranite", |c| {
    c.resource = "Agapite";
});

item("VeriteGranite", |c| {
    c.resource = "Verite";
});

item("ValoriteGranite", |c| {
    c.resource = "Valorite";
});

item("Gold", |c| {
    c.on_create(|i| {
        i.item_id = 0xEED;
        i.stackable = true;
    });
});
