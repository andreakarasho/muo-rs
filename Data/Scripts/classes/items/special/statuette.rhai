item("MonsterStatuette", |c| {
    c.weight = 1.0;
});

item("HeritageToken", |c| {
    c.weight = 5.0;
    c.label = 1076596;
    c.on_create(|i| {
        i.item_id = 0x367A;
        i.loot_type = "Blessed";
    });
});

item("GrizzledMareStatuette", |c| {
    c.weight = 1.0;
    c.label = 1074475;
    c.on_create(|i| {
        i.item_id = 0x2617;
    });
});
