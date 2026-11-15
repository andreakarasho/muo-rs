item("TreasureChestDirt", |c| {
    c.on_create(|i| {
        i.item_id = 0x912;
        i.movable = false;
    });
});
