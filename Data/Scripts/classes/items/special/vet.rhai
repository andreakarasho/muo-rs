item("BloodyPentagramComponent", |c| {
    c.label = 1080279;
});

item("BloodyPentagramDeed", |c| {
    c.label = 1080384;
});

item("StoneAnkhComponent", |c| {
    c.weight = 1.0;
});

item("StoneAnkhDeed", |c| {
    c.label = 1049773;
});

item("MinotaurStatueDeed", |c| {
    c.label = 1080409;
});

item("TreeStumpDeed", |c| {
    c.label = 1080406;
});

item("RewardBrazier", |c| {
    c.weight = 10.0;
});

item("RewardBrazierDeed", |c| {
    c.weight = 1.0;
    c.label = 1080527;
    c.on_create(|i| {
        i.item_id = 0x14F0;
    });
});

item("RewardPottedCactus", |c| {
    c.weight = 5.0;
});

item("PottedCactusDeed", |c| {
    c.weight = 1.0;
    c.label = 1080407;
    c.on_create(|i| {
        i.item_id = 0x14F0;
    });
});
