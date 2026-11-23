item("BulletinMessage", |c| {
    c.on_create(|i| {
        i.item_id = 0xEB0;
        i.movable = false;
    });
});
