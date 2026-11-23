item("Head", |c| {
    c.on_create(|i| {
        i.item_id = 0x1DA0;
    });
});

item("BonePile", |c| {
    c.weight = 10.0;
    c.on_create(|i| {
        switch random_below(8) {
            0 => {
                i.item_id = 0x1B09;
            },
            1 => {
                i.item_id = 0x1B0A;
            },
            2 => {
                i.item_id = 0x1B0B;
            },
            3 => {
                i.item_id = 0x1B0C;
            },
            4 => {
                i.item_id = 0x1B0D;
            },
            5 => {
                i.item_id = 0x1B0E;
            },
            6 => {
                i.item_id = 0x1B0F;
            },
            7 => {
                i.item_id = 0x1B10;
            }
        }
        i.stackable = false;
    });
});

item("RibCage", |c| {
    c.weight = 5.0;
    c.on_create(|i| {
        switch random_below(2) {
            0 => {
                i.item_id = 0x1B17;
            },
            1 => {
                i.item_id = 0x1B18;
            }
        }
        i.stackable = false;
    });
});
