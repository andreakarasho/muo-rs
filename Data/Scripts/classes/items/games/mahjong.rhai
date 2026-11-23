item("MahjongGame", |c| {
    c.weight = 5.0;
    c.on_create(|i| {
        i.item_id = 0xFAA;
    });
});
