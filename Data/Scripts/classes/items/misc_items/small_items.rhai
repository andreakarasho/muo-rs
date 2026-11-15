item("Vines", |c| {
    c.weight = 1.0;
});

item("TowerLanternArtifact", |c| {
    c.rarity = 3;
    c.on_create(|i| {
        i.item_id = 0x24C0;
        i.light = "Circle225";
    });
});
