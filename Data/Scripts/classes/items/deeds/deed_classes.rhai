item("CommodityDeed", |c| {
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0x14F0;
        i.hue = 0x47;
    });
});

item("NameChangeDeed", |c| {
    c.name = "a name change deed";
    c.on_create(|i| {
        i.item_id = 0x14F0;
        i.loot_type = "Blessed";
    });
});

item("ClothingBlessDeed", |c| {
    c.weight = 1.0;
    c.name = "a clothing bless deed";
    c.on_create(|i| {
        i.item_id = 0x14F0;
        i.loot_type = "Blessed";
    });
});

item("NewPlayerTicket", |c| {
    c.weight = 1.0;
    c.label = 1062094;
    c.on_create(|i| {
        i.item_id = 0x14EF;
        i.loot_type = "Blessed";
    });
});
