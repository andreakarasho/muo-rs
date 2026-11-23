item("FarmableCabbage", |c| {
    c.on_create(|i| {
        i.item_id = 0xC7B;
    });
});

item("FarmableCarrot", |c| {
    c.on_create(|i| {
        i.item_id = 0xC76;
    });
});

item("FarmableCotton", |c| {
    c.on_create(|i| {
        switch random_below(4) {
            0 => {
                i.item_id = 0xC51;
            },
            1 => {
                i.item_id = 0xC52;
            },
            2 => {
                i.item_id = 0xC53;
            },
            3 => {
                i.item_id = 0xC54;
            }
        }
    });
});

item("FarmableFlax", |c| {
    c.on_create(|i| {
        switch random_below(3) {
            0 => {
                i.item_id = 0x1A99;
            },
            1 => {
                i.item_id = 0x1A9A;
            },
            2 => {
                i.item_id = 0x1A9B;
            }
        }
    });
});

item("FarmableLettuce", |c| {
    c.on_create(|i| {
        i.item_id = 0xCB6;
    });
});

item("FarmableOnion", |c| {
    c.on_create(|i| {
        i.item_id = 0xC6F;
    });
});

item("FarmablePumpkin", |c| {
    c.on_create(|i| {
        switch random_below(3) {
            0 => {
                i.item_id = 0xC5E;
            },
            1 => {
                i.item_id = 0xC5F;
            },
            2 => {
                i.item_id = 0xC60;
            }
        }
    });
});

item("FarmableTurnip", |c| {
    c.on_create(|i| {
        switch random_below(3) {
            0 => {
                i.item_id = 0xC61;
            },
            1 => {
                i.item_id = 0xC62;
            },
            2 => {
                i.item_id = 0xC63;
            }
        }
    });
});

item("FarmableWheat", |c| {
    c.on_create(|i| {
        switch random_below(4) {
            0 => {
                i.item_id = 0xC55;
            },
            1 => {
                i.item_id = 0xC56;
            },
            2 => {
                i.item_id = 0xC57;
            },
            3 => {
                i.item_id = 0xC58;
            }
        }
    });
});
