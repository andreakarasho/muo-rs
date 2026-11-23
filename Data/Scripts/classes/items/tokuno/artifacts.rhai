item("Stormgrip", |c| {
    c.label = 1070970;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.base_physical_resistance = 10;
    c.base_cold_resistance = 18;
    c.base_energy_resistance = 18;
    c.on_create(|i| {
        i.attributes.bonus_int = 8;
        i.attributes.luck = 125;
        i.attributes.weapon_damage = 25;
    });
});

item("RuneBeetleCarapace", |c| {
    c.label = 1070968;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.base_cold_resistance = 14;
    c.base_energy_resistance = 14;
    c.on_create(|i| {
        i.attributes.bonus_mana = 10;
        i.attributes.regen_mana = 3;
        i.attributes.lower_mana_cost = 15;
        i.armor_attributes.lower_stat_req = 100;
        i.armor_attributes.mage_armor = 1;
    });
});

item("KasaOfTheRajin", |c| {
    c.label = 1070969;
    c.base_physical_resistance = 12;
    c.base_fire_resistance = 17;
    c.base_cold_resistance = 21;
    c.base_poison_resistance = 17;
    c.base_energy_resistance = 17;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.on_create(|i| {
        i.attributes.spell_damage = 12;
    });
});

item("LeurociansMempoOfFortune", |c| {
    c.label = 1071460;
    c.base_physical_resistance = 15;
    c.base_fire_resistance = 10;
    c.base_cold_resistance = 10;
    c.base_poison_resistance = 10;
    c.base_energy_resistance = 15;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.on_create(|i| {
        i.loot_type = "Regular";
        i.hue = 0x501;
        i.attributes.luck = 300;
        i.attributes.regen_mana = 1;
    });
});

item("LegsOfStability", |c| {
    c.label = 1070925;
    c.base_physical_resistance = 20;
    c.base_poison_resistance = 18;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.on_create(|i| {
        i.attributes.bonus_stam = 5;
        i.armor_attributes.self_repair = 3;
        i.armor_attributes.lower_stat_req = 100;
        i.armor_attributes.mage_armor = 1;
    });
});

item("GlovesOfTheSun", |c| {
    c.label = 1070924;
    c.base_fire_resistance = 24;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.on_create(|i| {
        i.attributes.regen_hits = 2;
        i.attributes.night_sight = 1;
        i.attributes.lower_mana_cost = 5;
        i.attributes.lower_reg_cost = 18;
    });
});

item("DaimyosHelm", |c| {
    c.label = 1070920;
    c.base_cold_resistance = 10;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.on_create(|i| {
        i.armor_attributes.lower_stat_req = 100;
        i.armor_attributes.mage_armor = 1;
        i.armor_attributes.self_repair = 3;
        i.attributes.weapon_speed = 10;
    });
});

item("BlackLotusHood", |c| {
    c.label = 1070919;
    c.base_physical_resistance = 0;
    c.base_fire_resistance = 11;
    c.base_cold_resistance = 15;
    c.base_poison_resistance = 11;
    c.base_energy_resistance = 11;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.on_create(|i| {
        i.attributes.lower_mana_cost = 6;
        i.attributes.attack_chance = 6;
        i.armor_attributes.self_repair = 5;
    });
});

item("ArmsOfTacticalExcellence", |c| {
    c.label = 1070921;
    c.base_fire_resistance = 9;
    c.base_cold_resistance = 13;
    c.base_poison_resistance = 8;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.on_create(|i| {
        i.attributes.bonus_dex = 5;
        i.skill_bonuses = [["Tactics", 12.0]];
    });
});

item("AncientSamuraiDo", |c| {
    c.label = 1070926;
    c.base_physical_resistance = 15;
    c.base_fire_resistance = 12;
    c.base_cold_resistance = 10;
    c.base_poison_resistance = 11;
    c.base_energy_resistance = 8;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.on_create(|i| {
        i.armor_attributes.lower_stat_req = 100;
        i.armor_attributes.mage_armor = 1;
        i.skill_bonuses = [["Parry", 10.0]];
    });
});

item("AncientFarmersKasa", |c| {
    c.label = 1070922;
    c.base_cold_resistance = 19;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.on_create(|i| {
        i.attributes.bonus_str = 5;
        i.attributes.bonus_stam = 5;
        i.attributes.regen_stam = 5;
        i.skill_bonuses = [["AnimalLore", 5.0]];
    });
});

item("DemonForks", |c| {
    c.label = 1070917;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.on_create(|i| {
        i.weapon_attributes.resist_fire_bonus = 10;
        i.weapon_attributes.resist_poison_bonus = 10;
        i.attributes.reflect_physical = 10;
        i.attributes.weapon_damage = 35;
        i.attributes.defend_chance = 10;
    });
});

item("DragonNunchaku", |c| {
    c.label = 1070914;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.on_create(|i| {
        i.weapon_attributes.resist_fire_bonus = 5;
        i.weapon_attributes.self_repair = 3;
        i.weapon_attributes.hit_fireball = 50;
        i.attributes.weapon_damage = 40;
        i.attributes.weapon_speed = 20;
    });
});

item("Exiler", |c| {
    c.label = 1070913;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.damage_types = #{ nrgy: 100 };
    c.on_create(|i| {
        i.weapon_attributes.hit_dispel = 33;
        i.tokuno_slayer = "Exorcism";
        i.attributes.weapon_damage = 40;
        i.attributes.weapon_speed = 20;
    });
});

item("HanzosBow", |c| {
    c.label = 1070918;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.on_create(|i| {
        i.weapon_attributes.hit_leech_hits = 40;
        i.weapon_attributes.self_repair = 3;
        i.attributes.weapon_damage = 50;
        i.skill_bonuses = [["Ninjitsu", 10.0]];
    });
});

item("PeasantsBokuto", |c| {
    c.label = 1070912;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.on_create(|i| {
        i.weapon_attributes.self_repair = 3;
        i.weapon_attributes.hit_lower_defend = 30;
        i.attributes.weapon_damage = 35;
        i.attributes.weapon_speed = 10;
        i.tokuno_slayer = "SnakesBane";
    });
});

item("PilferedDancerFans", |c| {
    c.label = 1070916;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.on_create(|i| {
        i.attributes.weapon_damage = 20;
        i.attributes.weapon_speed = 20;
        i.attributes.cast_recovery = 2;
        i.attributes.defend_chance = 5;
        i.attributes.spell_channeling = 1;
    });
});

item("TheDestroyer", |c| {
    c.label = 1070915;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.on_create(|i| {
        i.weapon_attributes.hit_leech_stam = 40;
        i.attributes.bonus_str = 6;
        i.attributes.attack_chance = 10;
        i.attributes.weapon_damage = 50;
    });
});

item("DarkenedSky", |c| {
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1070966;
    c.damage_types = #{ cold: 50, nrgy: 50 };
    c.on_create(|i| {
        i.weapon_attributes.hit_lightning = 60;
        i.attributes.weapon_speed = 25;
        i.attributes.weapon_damage = 50;
    });
});

item("SwordOfTheStampede", |c| {
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1070964;
    c.damage_types = #{ cold: 100 };
    c.on_create(|i| {
        i.weapon_attributes.hit_harm = 100;
        i.attributes.attack_chance = 10;
        i.attributes.weapon_damage = 60;
    });
});

item("SwordsOfProsperity", |c| {
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1070963;
    c.damage_types = #{ fire: 100 };
    c.on_create(|i| {
        i.weapon_attributes.mage_weapon = 30;
        i.attributes.spell_channeling = 1;
        i.attributes.cast_speed = 1;
        i.attributes.luck = 200;
    });
});

item("TheHorselord", |c| {
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1070967;
    c.on_create(|i| {
        i.attributes.bonus_dex = 5;
        i.attributes.regen_mana = 1;
        i.attributes.luck = 125;
        i.attributes.weapon_damage = 50;
        i.tokuno_slayer = "ElementalBan";
        i.tokuno_slayer2 = "ReptilianDeath";
    });
});

item("WindsEdge", |c| {
    c.label = 1070965;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.damage_types = #{ nrgy: 100 };
    c.on_create(|i| {
        i.weapon_attributes.hit_leech_mana = 40;
        i.attributes.weapon_damage = 50;
        i.attributes.weapon_speed = 50;
        i.attributes.defend_chance = 10;
    });
});
