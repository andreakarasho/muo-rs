item("CannonAddonComponent", |c| {
    c.label = 1076157;
});

item("CannonDeed", |c| {
    c.label = 1076195;
});

item("WallBannerDeed", |c| {
    c.label = 1080549;
});

item("WeaponEngravingTool", |c| {
    c.weight = 1.0;
    c.label = 1076158;
    c.on_create(|i| {
        i.item_id = 0x32F8;
        i.loot_type = "Blessed";
    });
});
