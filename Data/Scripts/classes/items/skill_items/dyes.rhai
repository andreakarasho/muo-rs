item("DyeTub", |c| {
    c.weight = 10.0;
    c.on_create(|i| {
        i.item_id = 0xFAB;
    });
});

item("Dyes", |c| {
    c.weight = 3.0;
    c.on_create(|i| {
        i.item_id = 0xFA9;
    });
});

item("Scissors", |c| {
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0xF9F;
    });
});

item("BlackDyeTub", |c| {
    c.label = 0;
    c.on_create(|i| {
        i.dyed_hue = 1;
        i.hue = 1;
        i.redyable = false;
    });
});

item("BlazeDyeTub", |c| {
    c.label = 0;
    c.on_create(|i| {
        i.dyed_hue = 1161;
        i.hue = 0x489;
        i.redyable = false;
    });
});

item("FurnitureDyeTub", |c| {
    c.label = 1041246;
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("LeatherDyeTub", |c| {
    c.label = 1041284;
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("MetallicClothDyetub", |c| {
    c.label = 1152920;
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("MetallicLeatherDyeTub", |c| {
    c.label = 1153495;
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("RewardBlackDyeTub", |c| {
    c.label = 1006008;
    c.on_create(|i| {
        i.dyed_hue = 1;
        i.hue = 1;
        i.redyable = false;
        i.loot_type = "Blessed";
    });
});

item("RunebookDyeTub", |c| {
    c.label = 1049740;
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("SpecialDyeTub", |c| {
    c.label = 1041285;
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("StatuetteDyeTub", |c| {
    c.label = 1049741;
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("WhiteClothDyeTub", |c| {
    c.label = 1149984;
    c.on_create(|i| {
        i.dyed_hue = 2498;
        i.hue = 0x9C2;
    });
});

item("WhiteLeatherDyeTub", |c| {
    c.label = 1149900;
    c.on_create(|i| {
        i.dyed_hue = 2498;
        i.hue = 0x9C2;
        i.loot_type = "Blessed";
    });
});
