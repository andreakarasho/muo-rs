item("GuildDeed", |c| {
    c.weight = 1.0;
    c.label = 1041055;
    c.on_create(|i| {
        i.item_id = 0x14F0;
    });
});

item("GuildTeleporter", |c| {
    c.weight = 1.0;
    c.label = 1041054;
    c.on_create(|i| {
        i.item_id = 0x1869;
        i.loot_type = "Blessed";
    });
});

item("Guildstone", |c| {
    c.label = 1041429;
});

item("GuildstoneDeed", |c| {
    c.weight = 1.0;
    c.label = 1041233;
    c.on_create(|i| {
        i.item_id = 0x14F0;
    });
});
