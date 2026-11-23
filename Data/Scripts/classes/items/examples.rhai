item("Examples.HealingStone", |c| {
    c.extends = "Item";
    c.script = "examples/healing_stone.rhai";
    c.on_create(|i| {
        i.item_id = 0x1363;
        i.hue = 0x481;
        i.name = "a healing stone";
        i.movable = false;
    });
});

item("Examples.RandomBlade", |c| {
    c.extends = "Longsword";
    c.script = "examples/random_blade.rhai";
    c.on_create(|i| {
        i.hue = 0x489;
        i.attributes.luck = 50;
    });
});
