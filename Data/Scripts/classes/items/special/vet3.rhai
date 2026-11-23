item("AnkhOfSacrificeComponent", |c| {
    c.label = 1027772;
});

item("AnkhOfSacrificeDeed", |c| {
    c.label = 1080397;
});

item("MiningCartDeed", |c| {
    c.label = 1080385;
});

item("DecorativeShieldDeed", |c| {
    c.weight = 1.0;
    c.label = 1049771;
    c.on_create(|i| {
        i.item_id = 0x14F0;
        i.loot_type = "Blessed";
    });
});
