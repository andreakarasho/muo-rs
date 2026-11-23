item("SpecialScroll", |c| {
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0x14F0;
        i.loot_type = "Cursed";
    });
});

item("ScrollofAlacrity", |c| {
    c.label = 1078604;
});

item("ScrollofTranscendence", |c| {
    c.label = 1094934;
});
