item("ArmsOfArmstrong", |c| {
    c.base_physical_resistance = 6;
    c.base_fire_resistance = 6;
    c.base_cold_resistance = 5;
    c.base_poison_resistance = 5;
    c.base_energy_resistance = 5;
    c.label = 1077675;
    c.on_create(|i| {
        i.loot_type = "Blessed";
        i.attributes.bonus_str = 3;
        i.attributes.regen_hits = 1;
    });
});

item("BulwarkLeggings", |c| {
    c.base_physical_resistance = 9;
    c.base_fire_resistance = 5;
    c.base_cold_resistance = 5;
    c.base_poison_resistance = 5;
    c.base_energy_resistance = 3;
    c.label = 1077727;
    c.on_create(|i| {
        i.loot_type = "Blessed";
        i.attributes.regen_stam = 1;
        i.attributes.regen_mana = 1;
    });
});

item("GlovesOfSafeguarding", |c| {
    c.base_physical_resistance = 6;
    c.base_fire_resistance = 6;
    c.base_cold_resistance = 5;
    c.base_poison_resistance = 5;
    c.base_energy_resistance = 5;
    c.label = 1077614;
    c.on_create(|i| {
        i.loot_type = "Blessed";
        i.attributes.bonus_stam = 3;
        i.attributes.regen_hits = 1;
    });
});

item("HealersTouch", |c| {
    c.base_physical_resistance = 6;
    c.base_fire_resistance = 6;
    c.base_cold_resistance = 5;
    c.base_poison_resistance = 5;
    c.base_energy_resistance = 5;
    c.label = 1077684;
    c.on_create(|i| {
        i.loot_type = "Blessed";
        i.attributes.bonus_stam = 3;
        i.attributes.reflect_physical = 5;
    });
});

item("TheDragonsTail", |c| {
    c.init_min_hits = 80;
    c.init_max_hits = 80;
    c.label = 1078015;
    c.on_create(|i| {
        i.loot_type = "Blessed";
        i.weapon_attributes.hit_leech_stam = 16;
        i.attributes.weapon_speed = 10;
        i.attributes.weapon_damage = 25;
    });
});

item("TunicOfGuarding", |c| {
    c.base_physical_resistance = 6;
    c.base_fire_resistance = 6;
    c.base_cold_resistance = 5;
    c.base_poison_resistance = 5;
    c.base_energy_resistance = 5;
    c.label = 1077693;
    c.on_create(|i| {
        i.loot_type = "Blessed";
        i.attributes.bonus_hits = 2;
        i.attributes.reflect_physical = 5;
    });
});

item("TwilightJacket", |c| {
    c.base_physical_resistance = 6;
    c.base_fire_resistance = 12;
    c.base_cold_resistance = 3;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 3;
    c.label = 1078183;
    c.on_create(|i| {
        i.loot_type = "Blessed";
        i.attributes.reflect_physical = 5;
    });
});

item("WalkersLeggings", |c| {
    c.base_physical_resistance = 10;
    c.base_fire_resistance = 6;
    c.base_cold_resistance = 6;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 3;
    c.label = 1078222;
    c.on_create(|i| {
        i.loot_type = "Blessed";
    });
});

item("RingOfTheSavant", |c| {
    c.label = 1077608;
    c.on_create(|i| {
        i.loot_type = "Blessed";
        i.attributes.bonus_int = 3;
        i.attributes.cast_recovery = 1;
        i.attributes.cast_speed = 1;
    });
});

item("PhilosophersHat", |c| {
    c.label = 1077602;
    c.base_physical_resistance = 5;
    c.base_fire_resistance = 5;
    c.base_cold_resistance = 9;
    c.base_poison_resistance = 5;
    c.base_energy_resistance = 5;
    c.on_create(|i| {
        i.loot_type = "Blessed";
        i.attributes.regen_mana = 1;
        i.attributes.lower_reg_cost = 7;
    });
});

item("ClaspOfConcentration", |c| {
    c.label = 1077695;
    c.on_create(|i| {
        i.loot_type = "Blessed";
        i.attributes.regen_stam = 2;
        i.attributes.regen_mana = 1;
        i.element_attributes.fire = 5;
        i.element_attributes.cold = 5;
    });
});

item("BraceletOfResilience", |c| {
    c.label = 1077627;
    c.on_create(|i| {
        i.loot_type = "Blessed";
        i.attributes.defend_chance = 5;
        i.element_attributes.fire = 5;
        i.element_attributes.cold = 5;
        i.element_attributes.poison = 5;
        i.element_attributes.energy = 5;
    });
});

item("ChurchillsWarMace", |c| {
    c.label = 1078062;
    c.on_create(|i| {
        i.loot_type = "Blessed";
        i.attributes.attack_chance = 5;
        i.attributes.weapon_speed = 10;
        i.attributes.weapon_damage = 25;
        i.weapon_attributes.lower_stat_req = 70;
    });
});

item("EmberStaff", |c| {
    c.label = 1077582;
    c.on_create(|i| {
        i.loot_type = "Blessed";
        i.weapon_attributes.hit_fireball = 15;
        i.weapon_attributes.mage_weapon = 10;
        i.weapon_attributes.lower_stat_req = 50;
        i.attributes.spell_channeling = 1;
        i.attributes.cast_speed = -1;
    });
});

item("Heartseeker", |c| {
    c.label = 1078210;
    c.on_create(|i| {
        i.loot_type = "Blessed";
        i.attributes.attack_chance = 5;
        i.attributes.weapon_speed = 10;
        i.attributes.weapon_damage = 25;
        i.weapon_attributes.lower_stat_req = 70;
    });
});

item("JocklesQuicksword", |c| {
    c.label = 1077666;
    c.on_create(|i| {
        i.loot_type = "Blessed";
        i.attributes.attack_chance = 5;
        i.attributes.weapon_speed = 10;
        i.attributes.weapon_damage = 25;
    });
});

item("RecarosRiposte", |c| {
    c.label = 1078195;
    c.on_create(|i| {
        i.loot_type = "Blessed";
        i.attributes.attack_chance = 5;
        i.attributes.weapon_speed = 10;
        i.attributes.weapon_damage = 25;
    });
});

item("SilverSerpentBlade", |c| {
    c.label = 1078163;
    c.on_create(|i| {
        i.loot_type = "Blessed";
        i.attributes.attack_chance = 5;
        i.attributes.weapon_speed = 10;
        i.attributes.weapon_damage = 25;
    });
});
