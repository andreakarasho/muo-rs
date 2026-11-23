item("FishingPole", |c| {
    c.weight = 8.0;
    c.on_create(|i| {
        i.item_id = 0xDC0;
    });
});

item("MessageInABottle", |c| {
    c.weight = 1.0;
    c.label = 1041080;
});

item("SOS", |c| {
    c.weight = 1.0;
});

item("SpecialFishingNet", |c| {
    c.weight = 1.0;
    c.label = 1041079;
    c.on_create(|i| {
        i.item_id = 0xDCA;
    });
});

item("FabledFishingNet", |c| {
    c.label = 1063451;
    c.on_create(|i| {
        i.item_id = 0xDCA;
    });
});
