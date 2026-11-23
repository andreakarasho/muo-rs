item("HolySword", |c| {
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1062921;
    c.on_create(|i| {
        i.hue = 0x482;
        i.loot_type = "Blessed";
        i.slayer = "Silver";
        i.attributes.weapon_damage = 40;
        i.weapon_attributes.self_repair = 10;
        i.weapon_attributes.lower_stat_req = 100;
        i.weapon_attributes.use_best_skill = 1;
    });
});

item("LeggingsOfEmbers", |c| {
    c.base_physical_resistance = 15;
    c.base_fire_resistance = 25;
    c.base_cold_resistance = 0;
    c.base_poison_resistance = 15;
    c.base_energy_resistance = 15;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1062911;
    c.on_create(|i| {
        i.hue = 0x2C;
        i.loot_type = "Blessed";
        i.armor_attributes.self_repair = 10;
        i.armor_attributes.mage_armor = 1;
        i.armor_attributes.lower_stat_req = 100;
    });
});

item("ShaminoCrossbow", |c| {
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1062915;
    c.on_create(|i| {
        i.hue = 0x504;
        i.loot_type = "Blessed";
        i.attributes.attack_chance = 15;
        i.attributes.weapon_damage = 40;
        i.weapon_attributes.self_repair = 10;
        i.weapon_attributes.lower_stat_req = 100;
    });
});

item("SamuraiHelm", |c| {
    c.base_physical_resistance = 15;
    c.base_fire_resistance = 10;
    c.base_cold_resistance = 10;
    c.base_poison_resistance = 15;
    c.base_energy_resistance = 10;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1062923;
    c.weight = 5.0;
    c.material_type = "Plate";
    c.graphic = 9068;
    c.on_create(|i| {
        i.loot_type = "Blessed";
        i.attributes.defend_chance = 15;
        i.armor_attributes.self_repair = 10;
        i.armor_attributes.lower_stat_req = 100;
        i.armor_attributes.mage_armor = 1;
    });
});

item("PlagueBeastGland", |c| {
    c.weight = 1.0;
    c.label = 1153759;
    c.on_create(|i| {
        i.item_id = 0x1CEF;
        i.hue = 6;
    });
});

item("WaterloggedBoots", |c| {
    c.label = 1074364;
    c.graphic = 5905;
    c.on_create(|i| {
        if coin() {
            i.item_id = 0x1711;
        } else {
            i.item_id = 0x170B;
        }
    });
});

item("CraftysFishingHat", |c| {
    c.label = 1074572;
    c.base_physical_resistance = 0;
    c.base_fire_resistance = 5;
    c.base_cold_resistance = 9;
    c.base_poison_resistance = 5;
    c.base_energy_resistance = 5;
    c.init_min_hits = 20;
    c.init_max_hits = 30;
    c.graphic = 5907;
});

item("HoodedShroudOfShadows", |c| {
    c.weight = 3.0;
    c.graphic = 9860;
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("JesterHatofChuckles", |c| {
    c.label = 1073256;
    c.weight = 1.0;
    c.base_physical_resistance = 12;
    c.base_fire_resistance = 12;
    c.base_cold_resistance = 12;
    c.base_poison_resistance = 12;
    c.base_energy_resistance = 12;
    c.init_min_hits = 100;
    c.init_max_hits = 100;
    c.graphic = 5916;
    c.on_create(|i| {
        i.attributes.luck = 150;
    });
});

item("LuckyNecklace", |c| {
    c.label = 1075239;
    c.graphic = 4232;
    c.hue = 1150;
    c.layer = "Neck";
    c.on_create(|i| {
        i.attributes.luck = 200;
        i.loot_type = "Blessed";
    });
});

item("MinotaurHedge", |c| {
    c.weight = 1.0;
    c.name = "minotaur hedge";
    c.on_create(|i| {
        switch random_below(4) {
            0 => {
                i.item_id = 0xC8F;
            },
            1 => {
                i.item_id = 0xC90;
            },
            2 => {
                i.item_id = 0xC91;
            },
            3 => {
                i.item_id = 0xC92;
            }
        }
    });
});

item("TormentedChains", |c| {
    c.weight = 1.0;
    c.on_create(|i| {
        switch random_below(2) {
            0 => {
                i.item_id = 0x1A07;
            },
            1 => {
                i.item_id = 0x1A08;
            }
        }
    });
});

item("RewardCake", |c| {
    c.weight = 1.0;
    c.label = 1049786;
    c.on_create(|i| {
        i.item_id = 0x9E9;
        i.stackable = false;
    });
});

item("EarringsOfProtection", |c| {
    c.layer = "Earrings";
    c.graphic = 4231;
});
