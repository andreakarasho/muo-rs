item("SpecialBeardDye", |c| {
    c.weight = 1.0;
    c.label = 1041087;
    c.on_create(|i| {
        i.item_id = 0xE26;
        i.loot_type = "Newbied";
    });
});

item("HairDye", |c| {
    c.weight = 1.0;
    c.label = 1041060;
});

item("SpecialHairDye", |c| {
    c.weight = 1.0;
    c.label = 1074402;
});

item("MelisandesHairDye", |c| {
    c.label = 1041088;
    c.on_create(|i| {
        i.item_id = 0xEFF;
    });
});

item("PromotionalToken", |c| {
    c.weight = 5.0;
    c.label = 1070997;
    c.on_create(|i| {
        i.item_id = 0x2AAA;
        i.loot_type = "Blessed";
        i.light = "Circle300";
    });
});

item("ContractOfEmployment", |c| {
    c.weight = 1.0;
    c.label = 1041243;
});
