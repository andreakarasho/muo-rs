item("TapestryOfSosaria", |c| {
    c.weight = 1.0;
    c.label = 1062917;
    c.on_create(|i| {
        i.item_id = 0x234E;
        i.loot_type = "Blessed";
    });
});

item("RoseOfTrinsic", |c| {
    c.weight = 1.0;
    c.label = 1062913;
    c.on_create(|i| {
        i.item_id = 0x234D;
        i.loot_type = "Blessed";
    });
});

item("RoseOfTrinsicPetal", |c| {
    c.weight = 1.0;
    c.label = 1062926;
    c.on_create(|i| {
        i.item_id = 0x1021;
        i.stackable = true;
    });
});

item("HearthOfHomeFireDeed", |c| {
    c.label = 1062919;
});

item("ZoogiFungus", |c| {
    c.weight = 0.1;
});

item("PowderOfTranslocation", |c| {
    c.weight = 0.1;
});
