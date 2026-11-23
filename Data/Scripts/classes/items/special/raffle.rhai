item("HouseRaffleDeed", |c| {
    c.name = "a writ of lease";
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0x2830;
    });
});

item("HouseRaffleStone", |c| {
    c.name = "a house raffle stone";
    c.on_create(|i| {
        i.item_id = 0xEDD;
    });
});
