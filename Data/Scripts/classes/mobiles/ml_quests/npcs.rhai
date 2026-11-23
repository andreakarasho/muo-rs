mobile("Ben", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Ben";
    c.on_create(|m| {
        m.title = "the Apprentice Necromancer";
        m.body = 400;
        m.mlq_hue = 33789;
        m.mlq_hair_item_id = 8264;
        m.mlq_hair_hue = 1123;
        m.mlq_facial_hair_item_id = 8268;
        m.mlq_facial_hair_hue = 1123;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_equip = [
            #{ item: "Backpack" },
            #{ item: "Shoes", hue: 2305 },
            #{ item: "LongPants", hue: 443 },
            #{ item: "FancyShirt", hue: 1878 },
        ];
    });
});

mobile("Frederic", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "The Ghost of Frederic Smithson";
    c.on_create(|m| {
        m.body = 26;
        m.mlq_hue = 1109;
        m.mlq_frozen = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
    });
});

mobile("Leon", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Leon";
    c.on_create(|m| {
        m.title = "the Alchemist";
        m.mlq_race = "Human";
        m.body = 400;
        m.female = false;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.mlq_equip = [#{ item: "Backpack" }, #{ item: "Shoes", hue: 2305 }, #{ item: "Robe", hue: 1623 }];
    });
});

mobile("Andros", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Andros";
    c.on_create(|m| {
        m.title = "the Blacksmith";
        m.body = 400;
        m.mlq_hue = 33801;
        m.mlq_facial_hair_item_id = 8257;
        m.mlq_facial_hair_hue = 1118;
        m.mlq_hair_item_id = 8265;
        m.mlq_hair_hue = 1118;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_equip = [
            #{ item: "Backpack" },
            #{ item: "Boots", hue: 2305 },
            #{ item: "FancyShirt", hue: 1547 },
            #{ item: "LongPants", hue: 443 },
            #{ item: "FullApron", hue: 2305 },
            #{ item: "SmithHammer" },
        ];
    });
});

mobile("Kia", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Kia";
    c.on_create(|m| {
        m.title = "the student";
        m.mlq_race = "Human";
        m.body = 401;
        m.female = true;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.mlq_equip = [#{ item: "Backpack" }, #{ item: "Sandals", hue: 1801 }, #{ item: "Robe", hue: 1175 }];
    });
});

mobile("Emerillo", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Emerillo";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the cook";
        m.mlq_race = "Human";
        m.body = 400;
        m.female = false;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.mlq_equip = [
            #{ item: "Backpack" },
            #{ item: "Sandals", hue: "randomNeutralHue" },
            #{ item: "ShortPants", hue: "randomPinkHue" },
            #{ item: "Shirt" },
            #{ item: "HalfApron", hue: 2301 },
        ];
    });
});

mobile("Nythalia", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Mythalia";
    c.on_create(|m| {
        m.title = "the student";
        m.mlq_race = "Human";
        m.body = 401;
        m.female = true;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.mlq_equip = [
            #{ item: "Backpack" },
            #{ item: "Shoes", hue: "randomNeutralHue" },
            #{ item: "Robe", hue: #{ coin: [1175, 1176] } },
        ];
    });
});

mobile("Iosep", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Iosep";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the Exporter";
        m.body = 400;
        m.mlq_skin_hue = true;
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_equip = [
            #{ item: "FancyShirt", hue: "randomBlueHue" },
            #{ item: "LongPants", hue: 443 },
            #{ item: "Shoes", hue: "randomNeutralHue" },
            #{ item: "Backpack" },
        ];
    });
});

mobile("Aurelia", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Aurelia";
    c.on_create(|m| {
        m.title = "the Architect's Daughter";
        m.mlq_race = "Human";
        m.body = 401;
        m.female = true;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.mlq_equip = [
            #{ item: "Backpack" },
            #{ item: "Sandals", hue: "randomPinkHue" },
            #{ coin: [#{ item: "Kilt", hue: "randomPinkHue" }, #{ item: "Skirt", hue: "randomPinkHue" }] },
            #{ item: "FancyShirt", hue: "randomRedHue" },
        ];
    });
});

mobile("SkeletonOfSzandor", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Skeleton of Szandor";
    c.on_create(|m| {
        m.title = "the Late Architect";
        m.mlq_hue = 33778;
        m.body = 50;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
    });
});

mobile("Saril", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Saril";
    c.can_teach = true;
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the guard";
        m.mlq_race = "Elf";
        m.body = 606;
        m.female = true;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.skill("Meditation", 60.0, 80.0);
        m.skill("Focus", 60.0, 80.0);
        m.mlq_equip = [
            #{ item: "ElvenBoots" },
            #{ item: "WoodlandLegs" },
            #{ item: "WoodlandArms" },
            #{ item: "WoodlandBelt" },
            #{ item: "WingedHelm" },
            #{ item: "FemaleElvenPlateChest" },
            #{ item: "RadiantScimitar" },
        ];
    });
});

mobile("Cailla", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Cailla";
    c.can_teach = true;
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the guard";
        m.mlq_race = "Elf";
        m.body = 606;
        m.female = true;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.skill("Meditation", 60.0, 80.0);
        m.skill("Focus", 60.0, 80.0);
        m.mlq_equip = [
            #{ item: "ElvenBoots" },
            #{ item: "HidePants" },
            #{ item: "HidePauldrons" },
            #{ item: "HideGloves" },
            #{ item: "WoodlandBelt" },
            #{ item: "RavenHelm" },
            #{ item: "HideFemaleChest" },
            #{ item: "MagicalShortbow" },
        ];
    });
});

mobile("Tamm", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Tamm";
    c.can_teach = true;
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the guard";
        m.mlq_race = "Elf";
        m.body = 605;
        m.female = false;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.skill("Meditation", 60.0, 80.0);
        m.skill("Focus", 60.0, 80.0);
        m.mlq_equip = [
            #{ item: "ElvenBoots" },
            #{ item: "HidePants" },
            #{ item: "HidePauldrons" },
            #{ item: "WingedHelm" },
            #{ item: "HideChest" },
            #{ item: "ElvenCompositeLongbow" },
        ];
    });
});

mobile("Landy", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Landy";
    c.can_teach = true;
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the soil nurturer";
        m.mlq_race = "Elf";
        m.body = 605;
        m.female = false;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.skill("Meditation", 60.0, 80.0);
        m.skill("Focus", 60.0, 80.0);
        m.mlq_equip = [
            #{ item: "Sandals", hue: "randomYellowHue" },
            #{ item: "ShortPants", hue: "randomYellowHue" },
            #{ item: "Tunic", hue: "randomYellowHue" },
            #{ item: "LeafGloves", hue: "randomYellowHue" },
        ];
    });
});

mobile("Alejaha", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Elder Alejaha";
    c.can_teach = true;
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the wise";
        m.mlq_race = "Elf";
        m.body = 606;
        m.female = true;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.skill("Meditation", 60.0, 80.0);
        m.skill("Focus", 60.0, 80.0);
        m.mlq_equip = [
            #{ item: "Sandals", hue: "randomYellowHue" },
            #{ item: "ElvenShirt", hue: "randomYellowHue" },
            #{ item: "GemmedCirclet" },
            #{ item: "Cloak", hue: "randomBrightHue" },
            #{ coin: [#{ item: "Kilt", hue: 903 }, #{ item: "Skirt", hue: 903 }] },
        ];
    });
});

mobile("Mielan", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Mielan";
    c.can_teach = true;
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the arcanist";
        m.mlq_race = "Elf";
        m.body = 605;
        m.female = false;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.skill("Meditation", 60.0, 80.0);
        m.skill("Focus", 60.0, 80.0);
        m.mlq_equip = [
            #{ item: "ElvenBoots", hue: 2305 },
            #{ item: "ElvenShirt", hue: 86 },
            #{ item: "GemmedCirclet" },
            #{ item: "ElvenPants", hue: 2305 },
        ];
    });
});

mobile("Ciala", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Ciala";
    c.can_teach = true;
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the arborist";
        m.mlq_race = "Elf";
        m.body = 606;
        m.female = true;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.skill("Meditation", 60.0, 80.0);
        m.skill("Focus", 60.0, 80.0);
        m.mlq_equip = [
            #{ item: "Skirt", hue: "randomBlueHue" },
            #{ item: "ElvenShirt", hue: "randomYellowHue" },
            #{ item: "RoyalCirclet" },
            #{
                coin: [#{ item: "Boots", hue: "randomYellowHue" }, #{ item: "ThighBoots", hue: "randomYellowHue" }],
            },
        ];
    });
});

mobile("Aniel", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Aniel";
    c.can_teach = true;
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the arborist";
        m.mlq_race = "Elf";
        m.body = 605;
        m.female = false;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.skill("Meditation", 60.0, 80.0);
        m.skill("Focus", 60.0, 80.0);
        m.mlq_equip = [
            #{ item: "ElvenPants", hue: 2305 },
            #{ item: "LeafChest" },
            #{ item: "HalfApron", hue: "randomYellowHue" },
            #{ item: "ElvenBoots", hue: 2305 },
        ];
    });
});

mobile("Aulan", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Aulan";
    c.can_teach = true;
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the expeditionist";
        m.mlq_race = "Elf";
        m.body = 605;
        m.female = false;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.skill("Meditation", 60.0, 80.0);
        m.skill("Focus", 60.0, 80.0);
        m.mlq_equip = [
            #{ item: "ElvenBoots", dye: "randomYellowHue" },
            #{ item: "ElvenPants", hue: "randomGreenHue" },
            #{ item: "Cloak", hue: "randomGreenHue" },
            #{ item: "Circlet" },
            #{ item: "HideChest", dye: "randomYellowHue" },
            #{ item: "HideGloves", dye: "randomYellowHue" },
        ];
    });
});

mobile("Brinnae", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Brinnae";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the wise";
        m.mlq_race = "Elf";
        m.body = 606;
        m.female = true;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.mlq_equip = [
            #{ item: "ElvenBoots" },
            #{ item: "FemaleLeafChest" },
            #{ item: "LeafArms" },
            #{ item: "HidePants" },
            #{ item: "ElvenCompositeLongbow" },
        ];
    });
});

mobile("Caelas", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Elder Caelas";
    c.can_teach = true;
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the wise";
        m.mlq_race = "Elf";
        m.body = 605;
        m.female = false;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.skill("Meditation", 60.0, 80.0);
        m.skill("Focus", 60.0, 80.0);
        m.mlq_equip = [
            #{ item: "ElvenBoots", hue: 443 },
            #{ item: "MaleElvenRobe", hue: 1161 },
            #{ item: "Cloak", hue: 1816 },
            #{ item: "RoyalCirclet" },
        ];
    });
});

mobile("Clehin", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Clehin";
    c.can_teach = true;
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the soil nurturer";
        m.mlq_race = "Elf";
        m.body = 606;
        m.female = true;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.skill("Meditation", 60.0, 80.0);
        m.skill("Focus", 60.0, 80.0);
        m.mlq_equip = [#{ item: "ElvenBoots" }, #{ item: "ElvenShirt" }, #{ item: "LeafTonlet" }];
    });
});

mobile("Cloorne", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Cloorne";
    c.can_teach = true;
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the expeditionist";
        m.mlq_race = "Elf";
        m.body = 605;
        m.female = false;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.skill("Meditation", 60.0, 80.0);
        m.skill("Focus", 60.0, 80.0);
        m.mlq_equip = [
            #{ item: "ElvenBoots", hue: 946 },
            #{ item: "RadiantScimitar" },
            #{ item: "WingedHelm" },
            #{ item: "WoodlandLegs", dye: 1866 },
            #{ item: "HideChest", dye: 1830 },
            #{ item: "LeafArms", dye: 1854 },
        ];
    });
});

mobile("Salaenih", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Salaenih";
    c.can_teach = true;
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the expeditionist";
        m.mlq_race = "Elf";
        m.body = 606;
        m.female = true;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.skill("Meditation", 60.0, 80.0);
        m.skill("Focus", 60.0, 80.0);
        m.mlq_equip = [
            #{ item: "ElvenBoots" },
            #{ item: "WarCleaver" },
            #{ item: "WoodlandBelt", dye: 1431 },
            #{ item: "VultureHelm", dye: 443 },
            #{ item: "WoodlandLegs", dye: 443 },
            #{ item: "WoodlandChest", dye: 443 },
            #{ item: "WoodlandArms", dye: 443 },
        ];
    });
});

mobile("Vilo", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Vilo";
    c.can_teach = true;
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the guard";
        m.mlq_race = "Elf";
        m.body = 605;
        m.female = false;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.skill("Meditation", 60.0, 80.0);
        m.skill("Focus", 60.0, 80.0);
        m.mlq_equip = [
            #{ item: "ElvenBoots", hue: 2305 },
            #{ item: "OrnateAxe" },
            #{ item: "WoodlandBelt", hue: 1426 },
            #{ item: "VultureHelm" },
            #{ item: "WoodlandLegs" },
            #{ item: "WoodlandChest" },
            #{ item: "WoodlandArms" },
            #{ item: "WoodlandGorget" },
        ];
    });
});

mobile("Tholef", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Tholef";
    c.can_teach = true;
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the grape tender";
        m.mlq_race = "Elf";
        m.body = 605;
        m.female = false;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.skill("Meditation", 60.0, 80.0);
        m.skill("Focus", 60.0, 80.0);
        m.mlq_equip = [
            #{ item: "Sandals", hue: 2305 },
            #{ item: "FullApron", hue: 1878 },
            #{ item: "ShortPants", hue: 652 },
            #{ item: "Shirt", hue: 652 },
            #{ item: "LeafArms", hue: 652 },
        ];
    });
});

mobile("Tillanil", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Tillanil";
    c.can_teach = true;
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the grape tender";
        m.mlq_race = "Elf";
        m.body = 606;
        m.female = true;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.skill("Meditation", 60.0, 80.0);
        m.skill("Focus", 60.0, 80.0);
        m.mlq_equip = [#{ item: "Sandals", hue: 443 }, #{ item: "Tunic", hue: 1881 }, #{ item: "ShortPants", hue: 33 }];
    });
});

mobile("Waelian", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Waelian";
    c.can_teach = true;
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the trinket weaver";
        m.mlq_race = "Elf";
        m.body = 605;
        m.female = false;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.skill("Meditation", 60.0, 80.0);
        m.skill("Focus", 60.0, 80.0);
        m.mlq_equip = [
            #{ item: "Shoes", hue: 2305 },
            #{ item: "SmithHammer" },
            #{ item: "LongPants", hue: 832 },
            #{ item: "GemmedCirclet" },
            #{ item: "LeafChest", hue: 836 },
        ];
    });
});

mobile("Sleen", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Sleen";
    c.can_teach = true;
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the trinket weaver";
        m.mlq_race = "Elf";
        m.body = 606;
        m.female = true;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.skill("Meditation", 60.0, 80.0);
        m.skill("Focus", 60.0, 80.0);
        m.mlq_equip = [
            #{ item: "ElvenBoots", hue: 2305 },
            #{ item: "SmithHammer" },
            #{ item: "Cloak", hue: 1882 },
            #{ item: "ElvenShirt" },
        ];
    });
});

mobile("Unoelil", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Unoelil";
    c.can_teach = true;
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the bark weaver";
        m.mlq_race = "Elf";
        m.body = 606;
        m.female = true;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.skill("Meditation", 60.0, 80.0);
        m.skill("Focus", 60.0, 80.0);
        m.mlq_equip = [
            #{ item: "ElvenBoots", hue: 443 },
            #{ item: "ShortPants", hue: 443 },
            #{ item: "Tunic", hue: 1613 },
        ];
    });
});

mobile("Anolly", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Anolly";
    c.can_teach = true;
    c.on_create(|m| {
        m.title = "the bark weaver";
        m.mlq_race = "Elf";
        m.body = 605;
        m.female = false;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.skill("Meditation", 60.0, 80.0);
        m.skill("Focus", 60.0, 80.0);
        m.mlq_equip = [
            #{ item: "Sandals", hue: 2305 },
            #{ item: "ShortPants", hue: 947 },
            #{ item: "FullApron", hue: 443 },
            #{ item: "SmithHammer" },
        ];
    });
});

mobile("Jusae", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Jusae";
    c.can_teach = true;
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the bowcrafter";
        m.mlq_race = "Elf";
        m.body = 605;
        m.female = false;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.skill("Meditation", 60.0, 80.0);
        m.skill("Focus", 60.0, 80.0);
        m.mlq_equip = [
            #{ item: "Sandals", hue: 2305 },
            #{ item: "ShortPants", hue: 1633 },
            #{ item: "MagicalShortbow" },
            #{ item: "HideChest", dye: 635 },
            #{ item: "HidePauldrons", dye: 638 },
        ];
    });
});

mobile("Cillitha", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Cillitha";
    c.can_teach = true;
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the bowcrafter";
        m.mlq_race = "Elf";
        m.body = 606;
        m.female = true;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.skill("Meditation", 60.0, 80.0);
        m.skill("Focus", 60.0, 80.0);
        m.mlq_equip = [#{ item: "ElvenBoots", hue: 2305 }, #{ item: "ElvenShirt", hue: 1841 }, #{ item: "LeafLegs" }];
    });
});

mobile("Lohn", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Lohn";
    c.can_teach = true;
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the metal weaver";
        m.mlq_race = "Elf";
        m.body = 605;
        m.female = false;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.skill("Meditation", 60.0, 80.0);
        m.skill("Focus", 60.0, 80.0);
        m.mlq_equip = [
            #{ item: "Shoes", hue: 2305 },
            #{ item: "LongPants", hue: 857 },
            #{ item: "SmithHammer" },
            #{ item: "GemmedCirclet" },
            #{ item: "LeafChest", hue: 857 },
        ];
    });
});

mobile("Olla", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Olla";
    c.can_teach = true;
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the metal weaver";
        m.mlq_race = "Elf";
        m.body = 606;
        m.female = true;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.skill("Meditation", 60.0, 80.0);
        m.skill("Focus", 60.0, 80.0);
        m.mlq_equip = [
            #{ item: "ElvenBoots" },
            #{ item: "LongPants", hue: 947 },
            #{ item: "SmithHammer" },
            #{ item: "FullApron", hue: 443 },
            #{ item: "ElvenShirt" },
        ];
    });
});

mobile("Thallary", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Thallary";
    c.can_teach = true;
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the cloth weaver";
        m.mlq_race = "Elf";
        m.body = 605;
        m.female = false;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.skill("Meditation", 60.0, 80.0);
        m.skill("Focus", 60.0, 80.0);
        m.mlq_equip = [
            #{ item: "Sandals", hue: 2305 },
            #{ item: "LongPants", hue: 1838 },
            #{ item: "Cloak", hue: 947 },
            #{ item: "FancyShirt", hue: 19 },
        ];
    });
});

mobile("Ahie", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Ahie";
    c.can_teach = true;
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the cloth weaver";
        m.mlq_race = "Elf";
        m.body = 606;
        m.female = true;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.skill("Meditation", 60.0, 80.0);
        m.skill("Focus", 60.0, 80.0);
        m.mlq_equip = [
            #{ item: "Boots", hue: 2305 },
            #{ item: "Skirt", hue: 28 },
            #{ item: "Cloak", hue: 98 },
            #{ item: "FancyShirt", hue: 1848 },
        ];
    });
});

mobile("Tyeelor", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Tyeelor";
    c.on_create(|m| {
        m.title = "the expeditionist";
        m.mlq_race = "Elf";
        m.body = 605;
        m.female = false;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.mlq_equip = [
            #{ item: "ElvenBoots", hue: 443 },
            #{ item: "WoodlandLegs", dye: 566 },
            #{ item: "WoodlandChest", dye: 566 },
            #{ item: "WoodlandArms", dye: 566 },
            #{ item: "VultureHelm", dye: 566 },
            #{ item: "WoodlandBelt", dye: 566 },
        ];
    });
});

mobile("Athailon", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Athailon";
    c.on_create(|m| {
        m.title = "the expeditionist";
        m.mlq_race = "Elf";
        m.body = 606;
        m.female = true;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.mlq_equip = [
            #{ item: "ElvenBoots", hue: 2305 },
            #{ item: "WoodlandBelt" },
            #{ item: "DiamondMace" },
            #{ item: "WoodlandLegs", dye: 946 },
            #{ item: "FemaleElvenPlateChest", dye: 946 },
            #{ item: "WoodlandArms", dye: 946 },
            #{ item: "WingedHelm", dye: 946 },
        ];
    });
});

mobile("ElderTaellia", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Elder Taellia";
    c.on_create(|m| {
        m.title = "the wise";
        m.mlq_race = "Elf";
        m.body = 606;
        m.female = true;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.mlq_equip = [
            #{ item: "ThighBoots", hue: 295 },
            #{ item: "FemaleElvenRobe", hue: "randomBrightHue" },
            #{ item: "MagicWand" },
            #{ item: "Circlet" },
        ];
    });
});

mobile("ElderMallew", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Elder Mallew";
    c.on_create(|m| {
        m.title = "the wise";
        m.mlq_race = "Elf";
        m.body = 605;
        m.female = false;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.mlq_equip = [
            #{ item: "ElvenBoots", hue: 443 },
            #{ item: "Cloak", hue: 946 },
            #{ item: "Circlet" },
            #{ item: "LeafTonlet", dye: 1348 },
            #{ item: "LeafChest", dye: 1336 },
            #{ item: "LeafArms", dye: 1320 },
        ];
    });
});

mobile("ElderAbbein", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Elder Abbein";
    c.on_create(|m| {
        m.title = "the wise";
        m.mlq_race = "Elf";
        m.body = 606;
        m.female = true;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.mlq_equip = [
            #{ item: "ElvenBoots", hue: 1836 },
            #{ item: "FemaleElvenRobe", hue: 2224 },
            #{ item: "RoyalCirclet" },
        ];
    });
});

mobile("ElderVicaie", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Elder Vicaie";
    c.on_create(|m| {
        m.title = "the wise";
        m.mlq_race = "Elf";
        m.body = 606;
        m.female = true;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.mlq_equip = [#{ item: "ElvenBoots" }, #{ item: "Tunic", hue: 1842 }, #{ item: "LeafLegs", hue: 946 }];
    });
});

mobile("ElderJothan", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Elder Jothan";
    c.on_create(|m| {
        m.title = "the wise";
        m.mlq_race = "Elf";
        m.body = 605;
        m.female = false;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.mlq_equip = [
            #{ item: "ThighBoots" },
            #{ item: "ElvenPants", hue: 1421 },
            #{ item: "ElvenShirt", hue: "randomYellowHue" },
            #{ item: "Cloak", hue: "randomBrightHue" },
            #{ item: "Circlet" },
        ];
    });
});

mobile("ElderAlethanian", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Elder Alethanian";
    c.on_create(|m| {
        m.title = "the wise";
        m.mlq_race = "Elf";
        m.body = 606;
        m.female = true;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.mlq_equip = [
            #{ item: "ElvenBoots" },
            #{ item: "HidePants" },
            #{ item: "HideFemaleChest" },
            #{ item: "HidePauldrons" },
            #{ item: "GemmedCirclet" },
        ];
    });
});

mobile("Rebinil", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Rebinil";
    c.on_create(|m| {
        m.title = "the healer";
        m.mlq_race = "Elf";
        m.body = 606;
        m.female = true;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.mlq_equip = [
            #{ item: "Sandals", hue: 1813 },
            #{ item: "FemaleElvenRobe", hue: 1858 },
            #{ item: "RoyalCirclet" },
        ];
    });
});

mobile("Aluniol", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Aluniol";
    c.on_create(|m| {
        m.title = "the healer";
        m.mlq_race = "Elf";
        m.body = 605;
        m.female = false;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.mlq_equip = [#{ item: "ElvenBoots", hue: 443 }, #{ item: "MaleElvenRobe", hue: 1150 }, #{ item: "WildStaff" }];
    });
});

mobile("Olaeni", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Olaeni";
    c.on_create(|m| {
        m.title = "the thaumaturgist";
        m.mlq_race = "Elf";
        m.body = 606;
        m.female = true;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.mlq_equip = [
            #{ item: "Shoes", hue: 1882 },
            #{ item: "FemaleElvenRobe", hue: 19 },
            #{ item: "MagicWand" },
            #{ item: "GemmedCirclet" },
        ];
    });
});

mobile("Bolaevin", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Bolaevin";
    c.on_create(|m| {
        m.title = "the arcanist";
        m.mlq_race = "Elf";
        m.body = 605;
        m.female = false;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.mlq_equip = [
            #{ item: "ElvenBoots", hue: 946 },
            #{ item: "RoyalCirclet" },
            #{ item: "LeafChest" },
            #{ item: "LeafArms" },
            #{ item: "LeafLegs", hue: 443 },
        ];
    });
});

mobile("LorekeeperAneen", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Lorekeeper Aneen";
    c.on_create(|m| {
        m.title = "the keeper of tradition";
        m.mlq_race = "Elf";
        m.body = 605;
        m.female = false;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.mlq_equip = [#{ item: "Sandals", hue: 443 }, #{ item: "MaleElvenRobe", hue: 1167 }, #{ item: "MagicWand" }];
    });
});

mobile("Daelas", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Daelas";
    c.on_create(|m| {
        m.title = "the arborist";
        m.mlq_race = "Elf";
        m.body = 605;
        m.female = false;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.mlq_equip = [
            #{ item: "ElvenBoots", hue: 2305 },
            #{ item: "ElvenPants", hue: 2219 },
            #{ item: "LeafChest", dye: 2224 },
            #{ item: "LeafGloves", dye: 443 },
        ];
    });
});

mobile("Alelle", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Alelle";
    c.on_create(|m| {
        m.title = "the arborist";
        m.mlq_race = "Elf";
        m.body = 606;
        m.female = true;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.mlq_equip = [
            #{ item: "ElvenBoots", hue: 443 },
            #{ item: "FemaleLeafChest", dye: 58 },
            #{ item: "LeafLegs", dye: 1868 },
            #{ item: "LeafGloves", dye: 443 },
        ];
    });
});

mobile("LorekeeperNillaen", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Lorekeeper Nillaen";
    c.can_teach = true;
    c.on_create(|m| {
        m.title = "the keeper of tradition";
        m.mlq_race = "Elf";
        m.body = 606;
        m.female = true;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.skill("Meditation", 60.0, 80.0);
        m.skill("Focus", 60.0, 80.0);
        m.mlq_equip = [
            #{ item: "Shoes", hue: 443 },
            #{ item: "LongPants", hue: 507 },
            #{ item: "ElvenShirt" },
            #{ item: "GemmedCirclet" },
            #{ item: "BodySash", hue: 37 },
            #{ item: "BlackStaff" },
        ];
    });
});

mobile("LorekeeperRyal", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Lorekeeper Ryal";
    c.can_teach = true;
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the keeper of tradition";
        m.mlq_race = "Elf";
        m.body = 605;
        m.female = false;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.skill("Meditation", 60.0, 80.0);
        m.skill("Focus", 60.0, 80.0);
        m.mlq_equip = [
            #{ item: "ElvenBoots", hue: 443 },
            #{ item: "LeafTonlet" },
            #{ item: "ElvenShirt", hue: 733 },
            #{ item: "Cloak", hue: 537 },
            #{ item: "GnarledStaff" },
        ];
    });
});

mobile("Braen", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Braen";
    c.can_teach = true;
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the thaumaturgist";
        m.mlq_race = "Elf";
        m.body = 605;
        m.female = false;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.skill("Meditation", 60.0, 80.0);
        m.skill("Focus", 60.0, 80.0);
        m.mlq_equip = [#{ item: "Sandals", hue: 1812 }, #{ item: "MaleElvenRobe", hue: 1610 }, #{ item: "MagicWand" }];
    });
});

mobile("ElderAcob", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Elder Acob";
    c.can_teach = true;
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the wise";
        m.mlq_race = "Elf";
        m.body = 605;
        m.female = false;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.skill("Meditation", 60.0, 80.0);
        m.skill("Focus", 60.0, 80.0);
        m.mlq_equip = [
            #{ item: "ElvenBoots", hue: 1812 },
            #{ item: "ElvenShirt", hue: "randomBrightHue" },
            #{ item: "HidePants" },
        ];
    });
});

mobile("LorekeeperCalendor", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Lorekeeper Calendor";
    c.can_teach = true;
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the keeper of tradition";
        m.mlq_race = "Elf";
        m.body = 606;
        m.female = true;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.skill("Meditation", 60.0, 80.0);
        m.skill("Focus", 60.0, 80.0);
        m.mlq_equip = [
            #{ item: "ElvenBoots", hue: 1812 },
            #{ item: "ElvenShirt", hue: "randomOrangeHue" },
            #{ item: "Kilt", hue: "randomOrangeHue" },
            #{ item: "RoyalCirclet" },
        ];
    });
});

mobile("LorekeeperSiarra", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Lorekeeper Siarra";
    c.can_teach = true;
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the keeper of tradition";
        m.mlq_race = "Elf";
        m.body = 606;
        m.female = true;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.skill("Meditation", 60.0, 80.0);
        m.skill("Focus", 60.0, 80.0);
        m.mlq_equip = [
            #{ item: "Sandals", hue: 443 },
            #{ item: "ElvenShirt" },
            #{ item: "LeafTonlet" },
            #{ item: "GemmedCirclet" },
        ];
    });
});

mobile("Enigma", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Enigma";
    c.on_create(|m| {
        m.body = 788;
        m.sound = 0x3EE;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
    });
});

mobile("Sledge", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Sledge";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the Versatile";
        m.body = 400;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_equip = [
            #{ item: "Tunic", hue: "randomNeutralHue" },
            #{ item: "LongPants", hue: "randomBlueHue" },
            #{ item: "Cloak", hue: "randomBrightHue" },
            #{ item: "ElvenBoots", hue: "randomNeutralHue" },
            #{ item: "Backpack" },
        ];
    });
});

mobile("Patricus", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Patricus";
    c.on_create(|m| {
        m.title = "the Trader";
        m.body = 400;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_equip = [
            #{ item: "FancyShirt", hue: "randomNeutralHue" },
            #{ item: "LongPants", hue: "randomBrightHue" },
            #{ item: "Cloak", hue: 443 },
            #{ item: "Shoes", hue: "randomNeutralHue" },
            #{ item: "Backpack" },
        ];
    });
});

mobile("Belulah", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Belulah";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the scorned";
        m.female = true;
        m.body = 401;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.mlq_equip = [
            #{ item: "FancyShirt", hue: "randomBlueHue" },
            #{ item: "LongPants", hue: "randomNondyedHue" },
            #{ item: "Boots" },
        ];
    });
});

mobile("Evan", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Evan";
    c.on_create(|m| {
        m.title = "the Beggar";
        m.mlq_race = "Human";
        m.body = 400;
        m.female = false;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.mlq_equip = [#{ item: "Backpack" }, #{ item: "Doublet" }, #{ item: "ShortPants", hue: 1877 }];
    });
});

mobile("Regina", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Regina";
    c.on_create(|m| {
        m.title = "the Noble";
        m.mlq_race = "Human";
        m.body = 401;
        m.female = true;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.mlq_equip = [#{ item: "Backpack" }, #{ item: "GildedDress" }, #{ item: "Boots" }];
    });
});

mobile("Jelrice", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Jelrice";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the trader";
        m.mlq_race = "Human";
        m.body = 401;
        m.female = true;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.mlq_equip = [
            #{ item: "Shoes", hue: "randomNeutralHue" },
            #{ item: "Skirt", hue: "randomBlueHue" },
            #{ item: "FancyShirt", hue: "randomRedHue" },
        ];
    });
});

mobile("Yorus", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Yorus";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the tinker";
        m.mlq_race = "Human";
        m.body = 400;
        m.female = false;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.mlq_equip = [
            #{ item: "Shoes", hue: "randomNeutralHue" },
            #{ item: "LongPants", hue: "randomBlueHue" },
            #{ item: "FancyShirt", hue: "randomOrangeHue" },
            #{ item: "Cloak", hue: "randomBrightHue" },
        ];
    });
});

mobile("Drithen", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.can_teach = true;
    c.name = "Drithen";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the Fierce";
        m.mlq_race = "Human";
        m.body = 400;
        m.female = false;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_equip = [
            #{ item: "Backpack" },
            #{ item: "ElvenBoots", hue: "randomNeutralHue" },
            #{ item: "LongPants", hue: "randomBlueHue" },
            #{ item: "Tunic", hue: "randomNeutralHue" },
            #{ item: "Cloak", hue: "randomBrightHue" },
        ];
        m.skill("Focus", 60.0, 80.0);
    });
});

mobile("Aernya", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Aernya";
    c.on_create(|m| {
        m.title = "the Mistress of Admissions";
        m.mlq_race = "Human";
        m.body = 401;
        m.female = true;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.mlq_equip = [
            #{ item: "Backpack" },
            #{ item: "Sandals", hue: "randomNeutralHue" },
            #{ item: "Skirt", hue: #{ coin: [1, 0] } },
            #{ item: "Cloak", hue: "randomBrightHue" },
            #{ item: "FancyShirt", hue: #{ coin: [946, 947] } },
        ];
    });
});

mobile("Gorrow", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Gorrow";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the Mayor";
        m.mlq_race = "Human";
        m.body = 400;
        m.female = false;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_equip = [
            #{ item: "Backpack" },
            #{ item: "Shoes", hue: 443 },
            #{ item: "Tunic", hue: "randomNeutralHue" },
            #{ item: "LongPants", hue: 2305 },
            #{ item: "Cloak", hue: "randomRedHue" },
        ];
    });
});

mobile("MasterGnosos", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.can_teach = true;
    c.name = "Master Gnosos";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the necromancer";
        m.mlq_race = "Human";
        m.body = 400;
        m.female = false;
        m.mlq_hue = 33768;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_hair_item_id = 8265;
        m.mlq_facial_hair_item_id = 8267;
        m.mlq_equip = [#{ item: "Backpack" }, #{ item: "Shoes", hue: 1157 }, #{ item: "Robe", hue: 1175 }];
        m.skill("EvalInt", 60.0, 80.0);
        m.skill("Inscribe", 60.0, 80.0);
        m.skill("MagicResist", 60.0, 80.0);
        m.skill("SpiritSpeak", 60.0, 80.0);
        m.skill("Meditation", 60.0, 80.0);
        m.skill("Necromancy", 60.0, 80.0);
    });
});

mobile("Dimethro", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.can_teach = true;
    c.name = "Dimethro";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the Wrestling Instructor";
        m.body = 400;
        m.mlq_hue = 33770;
        m.mlq_hair_item_id = 8253;
        m.mlq_hair_hue = 1109;
        m.mlq_facial_hair_item_id = 8269;
        m.mlq_facial_hair_hue = 1109;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.skill("EvalInt", 120.0);
        m.skill("Inscribe", 120.0);
        m.skill("Magery", 120.0);
        m.skill("MagicResist", 120.0);
        m.skill("Wrestling", 120.0);
        m.skill("Meditation", 120.0);
        m.mlq_equip = [
            #{ item: "Backpack" },
            #{ item: "Sandals", hue: 1109 },
            #{ item: "BodySash", hue: 1109 },
            #{ item: "LongPants", hue: 1109 },
        ];
    });
});

mobile("Churchill", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.can_teach = true;
    c.name = "Churchill";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the Mace Fighting Instructor";
        m.body = 400;
        m.mlq_hue = 33770;
        m.mlq_hair_item_id = 8252;
        m.mlq_hair_hue = 1109;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.skill("Anatomy", 120.0);
        m.skill("Parry", 120.0);
        m.skill("Healing", 120.0);
        m.skill("Tactics", 120.0);
        m.skill("Macing", 120.0);
        m.skill("Focus", 120.0);
        m.mlq_equip = [
            #{ item: "Backpack" },
            #{ item: "OrderShield" },
            #{ item: "WarMace" },
            #{ item: "PlateLegs", dye: 2406 },
            #{ item: "PlateGloves", dye: 2406 },
            #{ item: "PlateGorget", dye: 2406 },
            #{ item: "PlateChest", dye: 2406 },
            #{ item: "PlateArms", dye: 2406 },
        ];
    });
});

mobile("Recaro", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.can_teach = true;
    c.name = "Recaro";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the Fencer Instructor";
        m.body = 400;
        m.mlq_hue = 33770;
        m.mlq_hair_item_id = 8252;
        m.mlq_hair_hue = 1109;
        m.mlq_facial_hair_item_id = 8269;
        m.mlq_facial_hair_hue = 1109;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.skill("Anatomy", 120.0);
        m.skill("Parry", 120.0);
        m.skill("Healing", 120.0);
        m.skill("Tactics", 120.0);
        m.skill("Fencing", 120.0);
        m.skill("Focus", 120.0);
        m.mlq_equip = [
            #{ item: "Backpack" },
            #{ item: "Shoes", hue: 1109 },
            #{ item: "WarFork" },
            #{ item: "StuddedLegs", dye: 1109 },
            #{ item: "StuddedGloves", dye: 1109 },
            #{ item: "StuddedGorget", dye: 1109 },
            #{ item: "StuddedChest", dye: 1109 },
            #{ item: "StuddedArms", dye: 1109 },
        ];
    });
});

mobile("AldenArmstrong", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.can_teach = true;
    c.name = "Alden Armstrong";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the Tactics Instructor";
        m.body = 400;
        m.mlq_hue = 33770;
        m.mlq_hair_item_id = 8251;
        m.mlq_hair_hue = 1102;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.skill("Anatomy", 120.0);
        m.skill("Parry", 120.0);
        m.skill("Healing", 120.0);
        m.skill("Tactics", 120.0);
        m.skill("Swords", 120.0);
        m.skill("Focus", 120.0);
        m.mlq_equip = [
            #{ item: "Backpack" },
            #{ item: "Shoes" },
            #{ item: "StuddedLegs" },
            #{ item: "StuddedGloves" },
            #{ item: "StuddedGorget" },
            #{ item: "StuddedChest" },
            #{ item: "StuddedArms" },
            #{ item: "Katana" },
        ];
    });
});

mobile("Jockles", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.can_teach = true;
    c.name = "Jockles";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the Swordsmanship Instructor";
        m.body = 400;
        m.mlq_hue = 33786;
        m.mlq_hair_item_id = 8252;
        m.mlq_hair_hue = 2215;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.skill("Anatomy", 120.0);
        m.skill("Parry", 120.0);
        m.skill("Healing", 120.0);
        m.skill("Tactics", 120.0);
        m.skill("Swords", 120.0);
        m.skill("Focus", 120.0);
        m.mlq_equip = [
            #{ item: "Backpack" },
            #{ item: "Broadsword" },
            #{ item: "PlateChest" },
            #{ item: "PlateLegs" },
            #{ item: "PlateGloves" },
            #{ item: "PlateArms" },
            #{ item: "PlateGorget" },
            #{ item: "OrderShield" },
        ];
    });
});

mobile("TylAriadne", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.can_teach = true;
    c.name = "Tyl Ariadne";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the Parrying Instructor";
        m.body = 400;
        m.mlq_hue = 33652;
        m.mlq_hair_item_id = 0;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.skill("Anatomy", 120.0);
        m.skill("Parry", 120.0);
        m.skill("Healing", 120.0);
        m.skill("Tactics", 120.0);
        m.skill("Swords", 120.0);
        m.skill("Meditation", 120.0);
        m.skill("Focus", 120.0);
        m.mlq_equip = [
            #{ item: "Backpack" },
            #{ item: "ElvenBoots", hue: 2413 },
            #{ item: "StuddedLegs", dye: 2413 },
            #{ item: "StuddedGloves", dye: 2413 },
            #{ item: "StuddedGorget", dye: 2413 },
            #{ item: "StuddedChest", dye: 2413 },
            #{ item: "StuddedArms", dye: 2413 },
            #{ item: "DiamondMace", dye: 2413 },
        ];
    });
});

mobile("Alefian", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.can_teach = true;
    c.name = "Alefian";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the Resisting Spells Instructor";
        m.body = 400;
        m.mlq_hue = 33770;
        m.mlq_hair_item_id = 8253;
        m.mlq_hair_hue = 1111;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.skill("EvalInt", 120.0);
        m.skill("Inscribe", 120.0);
        m.skill("Magery", 120.0);
        m.skill("MagicResist", 120.0);
        m.skill("Wrestling", 120.0);
        m.skill("Meditation", 120.0);
        m.mlq_equip = [#{ item: "Backpack" }, #{ item: "Robe" }, #{ item: "Sandals" }];
    });
});

mobile("Gustar", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.can_teach = true;
    c.name = "Gustar";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the Meditation Instructor";
        m.body = 400;
        m.mlq_hue = 33781;
        m.mlq_hair_item_id = 8251;
        m.mlq_hair_hue = 1109;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.skill("EvalInt", 120.0);
        m.skill("Inscribe", 120.0);
        m.skill("Magery", 120.0);
        m.skill("MagicResist", 120.0);
        m.skill("Wrestling", 120.0);
        m.skill("Meditation", 120.0);
        m.mlq_equip = [#{ item: "Backpack" }, #{ item: "GustarShroud" }, #{ item: "Sandals" }];
    });
});

mobile("Jillian", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.can_teach = true;
    c.name = "Jillian";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the Inscription Instructor";
        m.body = 401;
        m.female = true;
        m.mlq_hue = 33770;
        m.mlq_hair_item_id = 8253;
        m.mlq_hair_hue = 1109;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.skill("EvalInt", 120.0);
        m.skill("Inscribe", 120.0);
        m.skill("Magery", 120.0);
        m.skill("MagicResist", 120.0);
        m.skill("Wrestling", 120.0);
        m.skill("Meditation", 120.0);
        m.mlq_equip = [#{ item: "Backpack" }, #{ item: "Robe", hue: 1145 }, #{ item: "Sandals" }];
    });
});

mobile("Kaelynna", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.can_teach = true;
    c.name = "Kaelynna";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the Magery Instructor";
        m.body = 401;
        m.female = true;
        m.mlq_hue = 33770;
        m.mlq_hair_item_id = 8252;
        m.mlq_hair_hue = 1149;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.skill("EvalInt", 120.0);
        m.skill("Inscribe", 120.0);
        m.skill("Magery", 120.0);
        m.skill("MagicResist", 120.0);
        m.skill("Wrestling", 120.0);
        m.skill("Meditation", 120.0);
        m.mlq_equip = [#{ item: "Backpack" }, #{ item: "Robe", hue: 1426 }, #{ item: "Sandals" }];
    });
});

mobile("Mithneral", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.can_teach = true;
    c.name = "Mithneral";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the Evaluating Intelligence Instructor";
        m.body = 400;
        m.mlq_hue = 33770;
        m.mlq_hair_item_id = 8252;
        m.mlq_hair_hue = 1109;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.skill("EvalInt", 120.0);
        m.skill("Inscribe", 120.0);
        m.skill("Magery", 120.0);
        m.skill("MagicResist", 120.0);
        m.skill("Wrestling", 120.0);
        m.skill("Meditation", 120.0);
        m.mlq_equip = [#{ item: "Backpack" }, #{ item: "Sandals" }, #{ item: "GustarShroud", hue: 1308 }];
    });
});

mobile("AndreasVesalius", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.can_teach = true;
    c.name = "Andreas Vesalius";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the Anatomy Instructor";
        m.body = 400;
        m.mlq_hue = 33772;
        m.mlq_hair_item_id = 8252;
        m.mlq_hair_hue = 1143;
        m.mlq_facial_hair_item_id = 8254;
        m.mlq_facial_hair_hue = 1143;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.skill("Anatomy", 120.0);
        m.skill("Parry", 120.0);
        m.skill("Healing", 120.0);
        m.skill("Tactics", 120.0);
        m.skill("Swords", 120.0);
        m.skill("Focus", 120.0);
        m.mlq_equip = [
            #{ item: "Backpack" },
            #{ item: "Boots" },
            #{ item: "BlackStaff" },
            #{ item: "LongPants" },
            #{ item: "Tunic", hue: 1645 },
        ];
    });
});

mobile("Avicenna", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.can_teach = true;
    c.name = "Avicenna";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the Healing Instructor";
        m.body = 400;
        m.mlq_hue = 33770;
        m.mlq_hair_item_id = 8251;
        m.mlq_hair_hue = 1143;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.skill("Anatomy", 120.0);
        m.skill("Parry", 120.0);
        m.skill("Healing", 120.0);
        m.skill("Tactics", 120.0);
        m.skill("Swords", 120.0);
        m.skill("Focus", 120.0);
        m.mlq_equip = [#{ item: "Backpack" }, #{ item: "Robe", hue: 1645 }, #{ item: "Boots" }, #{ item: "GnarledStaff" }];
    });
});

mobile("SarsmeaSmythe", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.can_teach = true;
    c.name = "Sarsmea Smythe";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the Focus Instructor";
        m.body = 401;
        m.female = true;
        m.mlq_hue = 33770;
        m.mlq_hair_item_id = 8252;
        m.mlq_hair_hue = 1110;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.skill("Anatomy", 120.0);
        m.skill("Parry", 120.0);
        m.skill("Healing", 120.0);
        m.skill("Tactics", 120.0);
        m.skill("Swords", 120.0);
        m.skill("Focus", 120.0);
        m.mlq_equip = [
            #{ item: "Backpack" },
            #{ item: "ThighBoots" },
            #{ item: "StuddedGorget" },
            #{ item: "LeatherLegs" },
            #{ item: "FemaleLeatherChest" },
            #{ item: "StuddedGloves" },
            #{ item: "LeatherNinjaBelt" },
            #{ item: "LightPlateJingasa" },
        ];
    });
});

mobile("Chiyo", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.can_teach = true;
    c.name = "Chiyo";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the Hiding Instructor";
        m.body = 247;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.skill("Hiding", 120.0);
        m.skill("Tactics", 120.0);
        m.skill("Tracking", 120.0);
        m.skill("Fencing", 120.0);
        m.skill("Stealth", 120.0);
        m.skill("Ninjitsu", 120.0);
    });
});

mobile("Jun", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.can_teach = true;
    c.name = "Jun";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the Stealth Instructor";
        m.body = 400;
        m.mlq_hue = 33795;
        m.mlq_hair_item_id = 8251;
        m.mlq_hair_hue = 1109;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.skill("Hiding", 120.0);
        m.skill("Tactics", 120.0);
        m.skill("Tracking", 120.0);
        m.skill("Fencing", 120.0);
        m.skill("Stealth", 120.0);
        m.skill("Ninjitsu", 120.0);
        m.mlq_equip = [
            #{ item: "Backpack" },
            #{ item: "SamuraiTabi" },
            #{ item: "LeatherNinjaPants" },
            #{ item: "LeatherNinjaMitts" },
            #{ item: "LeatherNinjaHood" },
            #{ item: "LeatherNinjaJacket" },
            #{ item: "LeatherNinjaBelt" },
        ];
    });
});

mobile("Walker", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.can_teach = true;
    c.name = "Walker";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the Tracking Instructor";
        m.body = 400;
        m.mlq_hue = 33770;
        m.mlq_hair_item_id = 8251;
        m.mlq_hair_hue = 1149;
        m.mlq_facial_hair_item_id = 8267;
        m.mlq_facial_hair_hue = 1149;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.skill("Hiding", 120.0);
        m.skill("Tactics", 120.0);
        m.skill("Tracking", 120.0);
        m.skill("Fencing", 120.0);
        m.skill("Wrestling", 120.0);
        m.skill("Stealth", 120.0);
        m.skill("Ninjitsu", 120.0);
        m.mlq_equip = [
            #{ item: "Backpack" },
            #{ item: "Boots", hue: 1109 },
            #{ item: "LongPants", hue: 1109 },
            #{ item: "FancyShirt", hue: 1149 },
            #{ item: "FloppyHat", hue: 1109 },
        ];
    });
});

mobile("JacobWaltz", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.can_teach = true;
    c.name = "Jacob Waltz";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the Miner Instructor";
        m.body = 400;
        m.mlq_hue = 33770;
        m.mlq_hair_item_id = 8264;
        m.mlq_hair_hue = 1102;
        m.mlq_facial_hair_item_id = 8269;
        m.mlq_facial_hair_hue = 1102;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.skill("ArmsLore", 120.0);
        m.skill("Blacksmith", 120.0);
        m.skill("Magery", 120.0);
        m.skill("Tactics", 120.0);
        m.skill("Tinkering", 120.0);
        m.skill("Swords", 120.0);
        m.skill("Mining", 120.0);
        m.mlq_equip = [
            #{ item: "Backpack" },
            #{ item: "Pickaxe" },
            #{ item: "Boots" },
            #{ item: "WideBrimHat", hue: 2406 },
            #{ item: "ShortPants", hue: 880 },
            #{ item: "Shirt", hue: 2406 },
            #{ item: "HalfApron", hue: 443 },
        ];
    });
});

mobile("Andric", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.can_teach = true;
    c.name = "Andric";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the archer trainer";
        m.mlq_race = "Human";
        m.body = 400;
        m.female = false;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.skill("Archery", 60.0, 80.0);
        m.mlq_equip = [
            #{ item: "Backpack" },
            #{ item: "LeatherChest", dye: 443 },
            #{ item: "LeatherLegs", dye: 1709 },
            #{ item: "LeatherArms", dye: 1709 },
            #{ item: "LeatherGloves", dye: 443 },
            #{ item: "Boots", hue: 443 },
            #{ item: "CompositeBow" },
        ];
    });
});

mobile("Kashiel", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.name = "Kashiel";
    c.is_invulnerable = true;
    c.on_create(|m| {
        m.title = "the archer";
        m.mlq_race = "Human";
        m.body = 401;
        m.female = true;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.mlq_equip = [
            #{ item: "Backpack" },
            #{ item: "LeatherChest", dye: 443 },
            #{ item: "LeatherLegs", dye: 2305 },
            #{ item: "LeatherArms", dye: 2305 },
            #{ item: "LeatherGloves", dye: 443 },
            #{ item: "Boots", hue: 443 },
            #{ item: "CompositeBow" },
        ];
    });
});

mobile("Asandos", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Asandos";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the chef";
        m.mlq_race = "Human";
        m.body = 400;
        m.female = false;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.mlq_equip = [
            #{ item: "Backpack" },
            #{ item: "Boots", hue: 2305 },
            #{ item: "ShortPants" },
            #{ item: "Shirt" },
            #{ item: "Cap" },
            #{ item: "HalfApron", hue: 40 },
        ];
    });
});

mobile("Clairesse", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Clarisse";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the servant";
        m.mlq_race = "Human";
        m.body = 401;
        m.female = true;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.mlq_equip = [
            #{ item: "Backpack" },
            #{ item: "Shoes", hue: "randomNeutralHue" },
            #{ item: "PlainDress", hue: 969 },
        ];
    });
});

mobile("Gervis", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.can_teach = true;
    c.name = "Gervis";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the blacksmith trainer";
        m.mlq_race = "Human";
        m.body = 400;
        m.female = false;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.skill("Blacksmith", 60.0, 80.0);
        m.mlq_equip = [
            #{ item: "Backpack" },
            #{ item: "Boots", hue: 947 },
            #{ item: "ShortPants", hue: 443 },
            #{ item: "Doublet", hue: 1618 },
            #{ item: "SmithHammer" },
            #{ item: "LeatherGloves", hue: 946 },
        ];
    });
});

mobile("Mugg", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Mugg";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the miner";
        m.mlq_race = "Human";
        m.body = 400;
        m.female = false;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.mlq_equip = [
            #{ item: "Backpack" },
            #{ item: "Boots", hue: 2305 },
            #{ item: "ShortPants", hue: 947 },
            #{ item: "Shirt", hue: 555 },
            #{ item: "HalfApron", hue: 1521 },
            #{ item: "SkullCap", hue: 375 },
            #{ item: "Pickaxe" },
        ];
    });
});

mobile("Lowel", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Lowel";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the carpenter";
        m.mlq_race = "Human";
        m.body = 400;
        m.female = false;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.mlq_equip = [
            #{ item: "Backpack" },
            #{ item: "Boots", hue: 1347 },
            #{ item: "ShortPants", hue: 1880 },
            #{ item: "FancyShirt", hue: 1338 },
            #{ item: "HalfApron", hue: 1746 },
        ];
    });
});

mobile("Lyle", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Lyle";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the mage";
        m.mlq_race = "Human";
        m.body = 400;
        m.female = false;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.mlq_equip = [#{ item: "Backpack" }, #{ item: "Robe", hue: 765 }, #{ item: "ThighBoots" }];
    });
});

mobile("Nibbet", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.name = "Nibbet";
    c.is_invulnerable = true;
    c.on_create(|m| {
        m.title = "the tinker";
        m.mlq_race = "Human";
        m.body = 400;
        m.female = false;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.mlq_equip = [
            #{ item: "Backpack" },
            #{ item: "Boots", hue: 1425 },
            #{ item: "ShortPants", hue: 248 },
            #{ item: "Shirt", hue: 45 },
            #{ item: "FullApron", hue: 648 },
        ];
    });
});

mobile("Norton", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Norton";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the fisher";
        m.mlq_race = "Human";
        m.body = 400;
        m.female = false;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.mlq_equip = [
            #{ item: "Backpack" },
            #{ item: "ThighBoots" },
            #{ item: "LongPants", hue: 1730 },
            #{ item: "Shirt", hue: 285 },
        ];
    });
});

mobile("Sadrah", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Sadrah";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the courier";
        m.mlq_race = "Human";
        m.body = 401;
        m.female = true;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.mlq_equip = [
            #{ item: "Backpack" },
            #{ item: "Boots", hue: 2305 },
            #{ item: "Skirt", hue: 82 },
            #{ item: "Shirt", hue: 295 },
            #{ item: "Cloak", hue: 101 },
            #{ item: "Longsword" },
        ];
    });
});

mobile("Hargrove", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.name = "Hargrove";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the Lumberjack";
        m.mlq_race = "Human";
        m.body = 400;
        m.female = false;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.mlq_equip = [
            #{ item: "Backpack" },
            #{ item: "Boots", hue: 2305 },
            #{ item: "StuddedLegs" },
            #{ item: "Shirt", hue: 648 },
            #{ item: "Bandana", hue: 32 },
            #{ item: "BattleAxe" },
            #{ item: "PlateGloves", hue: 542 },
        ];
    });
});

mobile("Beotham", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.can_teach = true;
    c.name = "Beotham";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the bowcrafter";
        m.mlq_race = "Elf";
        m.body = 605;
        m.female = false;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.skill("Meditation", 60.0, 80.0);
        m.skill("Focus", 60.0, 80.0);
        m.mlq_equip = [
            #{ item: "Sandals", hue: 2305 },
            #{ item: "ShortPants", hue: 1314 },
            #{ item: "FancyShirt", hue: 1301 },
            #{ item: "LeafGloves", hue: 2305 },
        ];
    });
});

mobile("Danoel", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.can_teach = true;
    c.name = "Danoel";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the metal weaver";
        m.mlq_race = "Elf";
        m.body = 605;
        m.female = false;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.skill("Meditation", 60.0, 80.0);
        m.skill("Focus", 60.0, 80.0);
        m.mlq_equip = [
            #{ item: "ElvenBoots", hue: 2305 },
            #{ item: "ElvenPants", hue: 902 },
            #{ item: "ElvenShirt", hue: 1887 },
            #{ item: "FullApron", hue: 443 },
            #{ item: "RoyalCirclet" },
            #{ item: "SmithHammer" },
        ];
    });
});

mobile("Tallinin", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.can_teach = true;
    c.name = "Tallinin";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the cloth weaver";
        m.mlq_race = "Elf";
        m.body = 606;
        m.female = true;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.skill("Meditation", 60.0, 80.0);
        m.skill("Focus", 60.0, 80.0);
        m.mlq_equip = [#{ item: "ElvenBoots", hue: 2305 }, #{ item: "Tunic", hue: 55 }, #{ item: "Cloak", hue: 1845 }];
    });
});

mobile("Tiana", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.can_teach = true;
    c.name = "Tiana";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the guard";
        m.mlq_race = "Elf";
        m.body = 606;
        m.female = true;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.skill("Meditation", 60.0, 80.0);
        m.skill("Focus", 60.0, 80.0);
        m.mlq_equip = [
            #{ item: "ElvenBoots" },
            #{ item: "HidePants" },
            #{ item: "HideFemaleChest" },
            #{ item: "HidePauldrons" },
            #{ item: "WoodlandBelt", dye: 1651 },
            #{ item: "RavenHelm", dye: 1091 },
        ];
    });
});

mobile("LorekeeperOolua", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.can_teach = true;
    c.name = "Lorekeeper Oolua";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the keeper of tradition";
        m.mlq_race = "Elf";
        m.body = 606;
        m.female = true;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.skill("Meditation", 60.0, 80.0);
        m.skill("Focus", 60.0, 80.0);
        m.mlq_equip = [
            #{ item: "ElvenBoots", hue: 1882 },
            #{ item: "Skirt", hue: "randomBrightHue" },
            #{ item: "FancyShirt", hue: 1858 },
            #{ item: "Cloak", hue: 443 },
            #{ item: "WildStaff" },
        ];
    });
});

mobile("LorekeeperRollarn", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.can_teach = true;
    c.name = "Lorekeeper Rollarn";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the keeper of tradition";
        m.mlq_race = "Elf";
        m.body = 605;
        m.female = false;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.skill("Meditation", 60.0, 80.0);
        m.skill("Focus", 60.0, 80.0);
        m.mlq_equip = [
            #{ item: "Sandals", hue: 443 },
            #{ item: "Cloak", hue: 662 },
            #{ item: "Circlet" },
            #{ item: "LeafChest" },
            #{ item: "LeafLegs", hue: 1818 },
        ];
    });
});

mobile("Dallid", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.can_teach = true;
    c.name = "Dallid";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the cook";
        m.mlq_race = "Elf";
        m.body = 605;
        m.female = false;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.skill("Meditation", 60.0, 80.0);
        m.skill("Focus", 60.0, 80.0);
        m.mlq_equip = [
            #{ item: "Boots", hue: 2305 },
            #{ item: "ShortPants", hue: 1852 },
            #{ item: "Shirt", hue: 1860 },
            #{ item: "FullApron", hue: 446 },
            #{ item: "Cleaver" },
        ];
    });
});

mobile("Canir", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.can_teach = true;
    c.name = "Canir";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the thaumaturgist";
        m.mlq_race = "Elf";
        m.body = 606;
        m.female = true;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.skill("Meditation", 60.0, 80.0);
        m.skill("Focus", 60.0, 80.0);
        m.mlq_equip = [
            #{ item: "Sandals", hue: 443 },
            #{ item: "FemaleElvenRobe", hue: 1447 },
            #{ item: "GemmedCirclet" },
            #{ item: "MagicWand" },
        ];
    });
});

mobile("Yellienir", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.name = "Yellienir";
    c.is_invulnerable = true;
    c.on_create(|m| {
        m.title = "the bark weaver";
        m.mlq_race = "Elf";
        m.body = 606;
        m.female = true;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.mlq_equip = [
            #{ item: "ElvenBoots" },
            #{ item: "LeafTonlet" },
            #{ item: "FemaleLeafChest" },
            #{ item: "LeafArms" },
            #{ item: "Cloak", hue: 946 },
        ];
    });
});

mobile("ElderOnallan", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.can_teach = true;
    c.name = "Elder Onallan";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the wise";
        m.mlq_race = "Elf";
        m.body = 605;
        m.female = false;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.skill("Meditation", 60.0, 80.0);
        m.skill("Focus", 60.0, 80.0);
        m.mlq_equip = [#{ item: "Shoes", hue: 1833 }, #{ item: "WildStaff" }, #{ item: "Cloak", hue: 1614 }];
    });
});

mobile("Aeluva", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.can_teach = true;
    c.name = "Aeluva";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the arcanist";
        m.mlq_race = "Elf";
        m.female = true;
        m.body = 606;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.skill("Meditation", 60.0, 80.0);
        m.skill("Focus", 60.0, 80.0);
        m.mlq_equip = [
            #{ item: "ElvenShirt" },
            #{ item: "Kilt", hue: "randomNondyedHue" },
            #{ item: "ElvenBoots" },
            #{ item: "Circlet" },
        ];
    });
});

mobile("Koole", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.can_teach = true;
    c.name = "Koole";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the arcanist";
        m.mlq_race = "Elf";
        m.body = 605;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.skill("Meditation", 60.0, 80.0);
        m.skill("Focus", 60.0, 80.0);
        m.mlq_equip = [
            #{ item: "LeafChest", dye: 443 },
            #{ item: "LeafArms", dye: 443 },
            #{ item: "LeafTonlet" },
            #{ item: "ThighBoots", hue: "randomAnimalHue" },
            #{ item: "RoyalCirclet" },
        ];
    });
});

mobile("Synaeva", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.can_teach = true;
    c.name = "Synaeva";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the arcanist";
        m.mlq_race = "Elf";
        m.female = true;
        m.body = 606;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.skill("Meditation", 60.0, 80.0);
        m.skill("Focus", 60.0, 80.0);
        m.mlq_equip = [
            #{ item: "RavenHelm", hue: "randomGreenHue" },
            #{ item: "FemaleLeafChest" },
            #{ item: "LeafArms" },
            #{ item: "LeafTonlet" },
            #{ item: "ElvenBoots" },
            #{ item: "WildStaff" },
        ];
    });
});

mobile("ElderBrae", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.can_teach = true;
    c.name = "Elder Brae";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the wise";
        m.mlq_race = "Elf";
        m.female = true;
        m.body = 606;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.skill("Meditation", 60.0, 80.0);
        m.skill("Focus", 60.0, 80.0);
        m.mlq_equip = [
            #{ item: "GemmedCirclet" },
            #{ item: "FemaleElvenRobe", hue: "randomBrightHue" },
            #{ item: "ElvenBoots", hue: "randomAnimalHue" },
        ];
    });
});

mobile("LorekeeperBroolol", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.is_invulnerable = true;
    c.can_teach = true;
    c.name = "Lorekeeper Broolol";
    c.can_shout = true;
    c.on_create(|m| {
        m.title = "the keeper of tradition";
        m.mlq_race = "Elf";
        m.body = 606;
        m.female = true;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.skill("Meditation", 60.0, 80.0);
        m.skill("Focus", 60.0, 80.0);
        m.mlq_equip = [#{ item: "ElvenBoots", hue: 1805 }, #{ item: "FemaleElvenRobe", hue: 58 }, #{ item: "WildStaff" }];
    });
});

mobile("Emilio", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.name = "Emilio";
    c.is_invulnerable = true;
    c.on_create(|m| {
        m.title = "the Tortured Artist";
        m.mlq_race = "Human";
        m.body = 400;
        m.female = false;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.mlq_equip = [
            #{ item: "Backpack" },
            #{ item: "Sandals", hue: 1835 },
            #{ item: "LongPants", hue: 1317 },
            #{ item: "FancyShirt", hue: 1343 },
            #{ item: "FloppyHat", hue: 1420 },
            #{ item: "BodySash", hue: 28 },
        ];
    });
});

mobile("Thalia", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.name = "Thalia";
    c.is_invulnerable = true;
    c.on_create(|m| {
        m.title = "the Bride";
        m.mlq_race = "Human";
        m.body = 401;
        m.female = true;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
        m.mlq_equip = [#{ item: "Backpack" }, #{ item: "Sandals", hue: 2301 }, #{ item: "FancyDress", hue: 2301 }];
    });
});

mobile("Jamal", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.script = "content/engines/jamal.rhai";
    c.is_invulnerable = true;
    c.name = "Jamal";
    c.on_create(|m| {
        m.title = "the Fisherman";
        m.body = 400;
        m.mlq_skin_hue = true;
        m.mlq_init_stats = [100, 100, 25];
        m.mlq_random_hair = true;
    });
});

mobile("GrandpaCharley", |c| {
    c.ai = "Vendor";
    c.fight_mode = "None";
    c.range_perception = 2;
    c.range_fight = 1;
    c.script = "content/engines/grandpa_charley.rhai";
    c.is_invulnerable = true;
    c.name = "Grandpa Charley";
    c.can_teach = true;
    c.on_create(|m| {
        m.title = "the farmer";
        m.body = 400;
        m.mlq_skin_hue = true;
        m.mlq_speed = [0.5, 2.0];
        m.mlq_init_stats = [100, 100, 25];
    });
});
