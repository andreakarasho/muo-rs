item("HangingSkeletonDeed", |c| {
    c.weight = 1.0;
    c.label = 1049772;
    c.on_create(|i| {
        i.item_id = 0x14F0;
        i.loot_type = "Blessed";
    });
});
