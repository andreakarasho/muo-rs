item("HairRestylingDeed", |c| {
    c.weight = 1.0;
    c.label = 1041061;
    c.on_create(|i| {
        i.item_id = 0x14F0;
        i.loot_type = "Blessed";
    });
});

item("BarkeepContract", |c| {
    c.weight = 1.0;
    c.name = "a barkeep contract";
    c.on_create(|i| {
        i.item_id = 0x14F0;
        i.loot_type = "Blessed";
    });
});

item("DragonBardingDeed", |c| {
    c.weight = 1.0;
});
