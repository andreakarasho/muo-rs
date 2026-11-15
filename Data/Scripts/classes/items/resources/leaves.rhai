item("PristineDreadHorn", |c| {
    c.on_create(|i| {
        i.item_id = 0x315A;
    });
});

item("Feather", |c| {
    c.weight = 0.1;
    c.on_create(|i| {
        i.item_id = 0x1BD1;
        i.stackable = true;
    });
});

item("Shaft", |c| {
    c.weight = 0.1;
    c.on_create(|i| {
        i.item_id = 0x1BD4;
        i.stackable = true;
    });
});

item("BarkFragment", |c| {
    c.script = "content/items/amount_range.rhai";
    c.on_create(|i| {
        i.item_id = 0x318F;
        i.stackable = true;
    });
});

item("Blight", |c| {
    c.on_create(|i| {
        i.item_id = 0x3183;
        i.stackable = true;
    });
});

item("BlueDiamond", |c| {
    c.script = "content/items/amount_range.rhai";
    c.on_create(|i| {
        i.item_id = 0x3198;
        i.stackable = true;
    });
});

item("BrilliantAmber", |c| {
    c.script = "content/items/amount_range.rhai";
    c.on_create(|i| {
        i.item_id = 0x3199;
        i.stackable = true;
    });
});

item("CapturedEssence", |c| {
    c.on_create(|i| {
        i.item_id = 0x318E;
        i.stackable = true;
    });
});

item("Corruption", |c| {
    c.script = "content/items/amount_range.rhai";
    c.on_create(|i| {
        i.item_id = 0x3184;
        i.stackable = true;
    });
});

item("DarkSapphire", |c| {
    c.script = "content/items/amount_range.rhai";
    c.on_create(|i| {
        i.item_id = 0x3192;
        i.stackable = true;
    });
});

item("DiseasedBark", |c| {
    c.script = "content/items/amount_range.rhai";
    c.on_create(|i| {
        i.item_id = 0x318B;
        i.stackable = true;
    });
});

item("DreadHornMane", |c| {
    c.script = "content/items/amount_range.rhai";
    c.on_create(|i| {
        i.item_id = 0x318A;
        i.stackable = true;
    });
});

item("EcruCitrine", |c| {
    c.script = "content/items/amount_range.rhai";
    c.on_create(|i| {
        i.item_id = 0x3195;
        i.stackable = true;
    });
});

item("EyeOfTheTravesty", |c| {
    c.script = "content/items/amount_range.rhai";
    c.on_create(|i| {
        i.item_id = 0x318D;
        i.stackable = true;
    });
});

item("FireRuby", |c| {
    c.script = "content/items/amount_range.rhai";
    c.on_create(|i| {
        i.item_id = 0x3197;
        i.stackable = true;
    });
});

item("GrizzledBones", |c| {
    c.script = "content/items/amount_range.rhai";
    c.on_create(|i| {
        i.item_id = 0x318C;
        i.stackable = true;
    });
});

item("LardOfParoxysmus", |c| {
    c.script = "content/items/amount_range.rhai";
    c.on_create(|i| {
        i.item_id = 0x3189;
        i.stackable = true;
    });
});

item("LuminescentFungi", |c| {
    c.on_create(|i| {
        i.item_id = 0x3191;
        i.stackable = true;
    });
});

item("Muculent", |c| {
    c.script = "content/items/amount_range.rhai";
    c.on_create(|i| {
        i.item_id = 0x3188;
        i.stackable = true;
    });
});

item("ParasiticPlant", |c| {
    c.script = "content/items/amount_range.rhai";
    c.on_create(|i| {
        i.item_id = 0x3190;
        i.stackable = true;
    });
});

item("PerfectEmerald", |c| {
    c.script = "content/items/amount_range.rhai";
    c.on_create(|i| {
        i.item_id = 0x3194;
        i.stackable = true;
    });
});

item("Putrefication", |c| {
    c.script = "content/items/amount_range.rhai";
    c.on_create(|i| {
        i.item_id = 0x3186;
        i.hue = 0x373;
        i.stackable = true;
    });
});

item("Scourge", |c| {
    c.script = "content/items/amount_range.rhai";
    c.on_create(|i| {
        i.item_id = 0x3185;
        i.hue = 0x96;
        i.stackable = true;
    });
});

item("SwitchItem", |c| {
    c.script = "content/items/amount_range.rhai";
    c.on_create(|i| {
        i.item_id = 0x2F5F;
        i.stackable = true;
    });
});

item("Taint", |c| {
    c.script = "content/items/amount_range.rhai";
    c.on_create(|i| {
        i.item_id = 0x3187;
        i.hue = 0x2DB;
        i.stackable = true;
    });
});

item("Turquoise", |c| {
    c.script = "content/items/amount_range.rhai";
    c.on_create(|i| {
        i.item_id = 0x3193;
        i.stackable = true;
    });
});

item("WhitePearl", |c| {
    c.on_create(|i| {
        i.item_id = 0x3196;
        i.stackable = true;
    });
});
