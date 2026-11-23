item("BaseReagent", |c| {
    c.weight = 0.1;
});

item("BagOfReagents", |c| {
    c.reagents = [
        "BlackPearl",
        "Bloodmoss",
        "Garlic",
        "Ginseng",
        "MandrakeRoot",
        "Nightshade",
        "SulfurousAsh",
        "SpidersSilk",
    ];
    c.on_create(|i| {
        i.item_id = 0xE76;
    });
});

item("BatWing", |c| {
    c.on_create(|i| {
        i.item_id = 0xF78;
    });
});

item("BlackPearl", |c| {
    c.on_create(|i| {
        i.item_id = 0xF7A;
    });
});

item("Bloodmoss", |c| {
    c.on_create(|i| {
        i.item_id = 0xF7B;
    });
});

item("DaemonBlood", |c| {
    c.on_create(|i| {
        i.item_id = 0xF7D;
    });
});

item("DaemonBone", |c| {
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0xF80;
    });
});

item("DeadWood", |c| {
    c.on_create(|i| {
        i.item_id = 0xF90;
    });
});

item("DragonsBlood", |c| {
    c.on_create(|i| {
        i.item_id = 0x4077;
    });
});

item("Garlic", |c| {
    c.on_create(|i| {
        i.item_id = 0xF84;
    });
});

item("Ginseng", |c| {
    c.on_create(|i| {
        i.item_id = 0xF85;
    });
});

item("GraveDust", |c| {
    c.on_create(|i| {
        i.item_id = 0xF8F;
    });
});

item("MandrakeRoot", |c| {
    c.on_create(|i| {
        i.item_id = 0xF86;
    });
});

item("Nightshade", |c| {
    c.on_create(|i| {
        i.item_id = 0xF88;
    });
});

item("NoxCrystal", |c| {
    c.on_create(|i| {
        i.item_id = 0xF8E;
    });
});

item("PigIron", |c| {
    c.on_create(|i| {
        i.item_id = 0xF8A;
    });
});

item("SpidersSilk", |c| {
    c.on_create(|i| {
        i.item_id = 0xF8D;
    });
});

item("SulfurousAsh", |c| {
    c.on_create(|i| {
        i.item_id = 0xF8C;
    });
});

item("BagOfAllReagents", |c| {
    c.reagents = [
        "BlackPearl",
        "Bloodmoss",
        "Garlic",
        "Ginseng",
        "MandrakeRoot",
        "Nightshade",
        "SulfurousAsh",
        "SpidersSilk",
        "BatWing",
        "GraveDust",
        "DaemonBlood",
        "NoxCrystal",
        "PigIron",
    ];
});

item("BagOfNecroReagents", |c| {
    c.reagents = ["BatWing", "GraveDust", "DaemonBlood", "NoxCrystal", "PigIron"];
});

item("BagOfNecromancerReagents", |c| {
    c.reagents = ["BatWing", "GraveDust", "DaemonBlood", "NoxCrystal", "PigIron"];
});
