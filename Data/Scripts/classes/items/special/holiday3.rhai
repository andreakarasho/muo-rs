item("HolidayTimepiece", |c| {
    c.label = 1041113;
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0x1086;
        i.loot_type = "Blessed";
        i.layer = "Bracelet";
    });
});

item("WristWatch", |c| {
    c.label = 1041421;
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0x1086;
        i.loot_type = "Blessed";
        i.layer = "Bracelet";
    });
});

item("HolidayTree", |c| {
    c.label = 1041117;
    c.on_create(|i| {
        i.item_id = 1;
        i.movable = false;
    });
});

item("HolidayTreeDeed", |c| {
    c.weight = 1.0;
    c.label = 1041116;
    c.on_create(|i| {
        i.item_id = 0x14F0;
        i.hue = 0x488;
        i.loot_type = "Blessed";
    });
});

item("Ornament", |c| {
    c.label = 1041118;
});

item("TreeTrunk", |c| {
    c.label = 1041117;
});
