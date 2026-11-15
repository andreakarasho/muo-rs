item("TaintedWool", |c| {
    c.weight = 4.0;
    c.on_create(|i| {
        i.item_id = 0x101F;
        i.stackable = true;
    });
});

item("BaseClothMaterial", |c| {
    c.weight = 1.0;
});

item("BigFish", |c| {
    c.label = 1041112;
    c.on_create(|i| {
        i.item_id = 0x9CC;
    });
});

item("Fish", |c| {
    c.weight = 1.0;
    c.on_create(|i| {
        switch random_below(4) {
            0 => {
                i.item_id = 0x9CC;
            },
            1 => {
                i.item_id = 0x9CD;
            },
            2 => {
                i.item_id = 0x9CE;
            },
            3 => {
                i.item_id = 0x9CF;
            }
        }
        i.stackable = true;
    });
});

item("BaseMagicFish", |c| {
    c.weight = 1.0;
});

item("PeculiarFish", |c| {
    c.label = 1041076;
    c.on_create(|i| {
        i.item_id = 0xDD6;
        i.hue = 0x42;
    });
});

item("Cloth", |c| {
    c.weight = 0.1;
    c.on_create(|i| {
        i.item_id = 0x1766;
        i.stackable = true;
    });
});

item("UncutCloth", |c| {
    c.weight = 0.1;
    c.on_create(|i| {
        i.item_id = 0x1767;
        i.stackable = true;
    });
});

item("BoltOfCloth", |c| {
    c.weight = 5.0;
    c.on_create(|i| {
        i.item_id = 0xF95;
        i.stackable = true;
    });
});

item("Bone", |c| {
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0xF7E;
        i.stackable = true;
    });
});

item("Bolt", |c| {
    c.weight = 0.1;
    c.on_create(|i| {
        i.item_id = 0x1BFB;
        i.stackable = true;
    });
});

item("Arrow", |c| {
    c.weight = 0.1;
    c.on_create(|i| {
        i.item_id = 0xF3F;
        i.stackable = true;
    });
});

item("Cotton", |c| {
    c.weight = 4.0;
    c.on_create(|i| {
        i.item_id = 0xDF9;
        i.stackable = true;
    });
});

item("Flax", |c| {
    c.weight = 1.0;
    c.label = 1026812;
    c.on_create(|i| {
        i.item_id = 0x1A9C;
        i.stackable = true;
    });
});

item("Wool", |c| {
    c.weight = 4.0;
    c.on_create(|i| {
        i.item_id = 0xDF8;
        i.stackable = true;
    });
});

item("DarkYarn", |c| {
    c.on_create(|i| {
        i.item_id = 0xE1D;
        i.stackable = true;
    });
});

item("LightYarn", |c| {
    c.on_create(|i| {
        i.item_id = 0xE1E;
        i.stackable = true;
    });
});

item("LightYarnUnraveled", |c| {
    c.on_create(|i| {
        i.item_id = 0xE1F;
        i.stackable = true;
    });
});

item("SpoolOfThread", |c| {
    c.on_create(|i| {
        i.item_id = 0xFA0;
        i.stackable = true;
    });
});

item("PrizedFish", |c| {
    c.label = 1041073;
    c.on_create(|i| {
        i.item_id = 0xDD6;
        i.hue = 0x33;
    });
});

item("WondrousFish", |c| {
    c.label = 1041074;
    c.on_create(|i| {
        i.item_id = 0xDD6;
        i.hue = 0x56;
    });
});

item("TrulyRareFish", |c| {
    c.label = 1041075;
    c.on_create(|i| {
        i.item_id = 0xDD6;
        i.hue = 0x4C;
    });
});
