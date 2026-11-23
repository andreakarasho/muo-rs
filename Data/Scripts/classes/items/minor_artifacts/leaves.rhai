item("ArcticDeathDealer", |c| {
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1063481;
    c.damage_types = #{ phys: 50, cold: 50 };
    c.on_create(|i| {
        i.hue = 0x480;
        i.weapon_attributes.hit_harm = 33;
        i.weapon_attributes.hit_lower_attack = 40;
        i.weapon_attributes.resist_cold_bonus = 10;
        i.attributes.weapon_speed = 20;
        i.attributes.weapon_damage = 40;
    });
});

item("BlazeOfDeath", |c| {
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1063486;
    c.damage_types = #{ phys: 50, fire: 50 };
    c.on_create(|i| {
        i.hue = 0x501;
        i.weapon_attributes.hit_fire_area = 50;
        i.weapon_attributes.hit_fireball = 50;
        i.weapon_attributes.resist_fire_bonus = 10;
        i.weapon_attributes.lower_stat_req = 100;
        i.attributes.weapon_speed = 25;
        i.attributes.weapon_damage = 35;
    });
});

item("BowOfTheJukaKing", |c| {
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1070636;
    c.on_create(|i| {
        i.hue = 0x460;
        i.weapon_attributes.hit_magic_arrow = 25;
        i.slayer = "ReptilianDeath";
        i.attributes.attack_chance = 15;
        i.attributes.weapon_damage = 40;
    });
});

item("CaptainQuacklebushsCutlass", |c| {
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1063474;
    c.on_create(|i| {
        i.hue = 0x66C;
        i.attributes.bonus_dex = 5;
        i.attributes.attack_chance = 10;
        i.attributes.weapon_speed = 20;
        i.attributes.weapon_damage = 50;
        i.weapon_attributes.use_best_skill = 1;
    });
});

item("CavortingClub", |c| {
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1063472;
    c.on_create(|i| {
        i.hue = 0x593;
        i.weapon_attributes.self_repair = 3;
        i.weapon_attributes.resist_fire_bonus = 8;
        i.weapon_attributes.resist_cold_bonus = 8;
        i.weapon_attributes.resist_poison_bonus = 8;
        i.weapon_attributes.resist_energy_bonus = 8;
        i.attributes.weapon_speed = 25;
        i.attributes.weapon_damage = 35;
    });
});

item("ColdBlood", |c| {
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1070818;
    c.damage_types = #{ cold: 100 };
    c.on_create(|i| {
        i.hue = 0x4F2;
        i.attributes.weapon_speed = 40;
        i.attributes.bonus_hits = 6;
        i.attributes.bonus_stam = 6;
        i.attributes.bonus_mana = 6;
    });
});

item("EnchantedTitanLegBone", |c| {
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1063482;
    c.on_create(|i| {
        i.hue = 0x8A5;
        i.weapon_attributes.hit_lower_defend = 40;
        i.weapon_attributes.hit_lightning = 40;
        i.weapon_attributes.resist_physical_bonus = 10;
        i.attributes.attack_chance = 10;
        i.attributes.weapon_damage = 20;
    });
});

item("GlovesOfThePugilist", |c| {
    c.base_physical_resistance = 18;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1070690;
    c.on_create(|i| {
        i.hue = 0x6D1;
        i.skill_bonuses = [["Wrestling", 10.0]];
        i.attributes.bonus_dex = 8;
        i.attributes.weapon_damage = 15;
    });
});

item("HeartOfTheLion", |c| {
    c.base_physical_resistance = 15;
    c.base_fire_resistance = 10;
    c.base_cold_resistance = 10;
    c.base_poison_resistance = 10;
    c.base_energy_resistance = 10;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1070817;
    c.on_create(|i| {
        i.hue = 0x501;
        i.attributes.luck = 95;
        i.attributes.defend_chance = 15;
        i.armor_attributes.lower_stat_req = 100;
        i.armor_attributes.mage_armor = 1;
    });
});

item("LunaLance", |c| {
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1063469;
    c.on_create(|i| {
        i.hue = 0x47E;
        i.skill_bonuses = [["Chivalry", 10.0]];
        i.attributes.bonus_str = 5;
        i.attributes.weapon_speed = 20;
        i.attributes.weapon_damage = 35;
        i.weapon_attributes.use_best_skill = 1;
    });
});

item("BladeDance", |c| {
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1075033;
    c.on_create(|i| {
        i.hue = 0x66C;
        i.attributes.bonus_mana = 8;
        i.attributes.spell_channeling = 1;
        i.attributes.weapon_damage = 30;
        i.weapon_attributes.hit_leech_mana = 20;
        i.weapon_attributes.use_best_skill = 1;
    });
});

item("Bonesmasher", |c| {
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1075030;
    c.on_create(|i| {
        i.item_id = 0x2D30;
        i.hue = 0x482;
        i.skill_bonuses = [["Macing", 10.0]];
        i.weapon_attributes.hit_leech_mana = 40;
        i.weapon_attributes.self_repair = 2;
    });
});

item("Boomstick", |c| {
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1075032;
    c.damage_types = #{ chaos: 100 };
    c.on_create(|i| {
        i.hue = 0x25;
        i.attributes.spell_channeling = 1;
        i.attributes.regen_mana = 3;
        i.attributes.cast_speed = 1;
        i.attributes.lower_reg_cost = 20;
    });
});

item("BrightsightLenses", |c| {
    c.base_physical_resistance = 9;
    c.base_fire_resistance = 29;
    c.base_cold_resistance = 7;
    c.base_poison_resistance = 8;
    c.base_energy_resistance = 7;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1075039;
    c.on_create(|i| {
        i.hue = 0x501;
        i.attributes.night_sight = 1;
        i.attributes.regen_mana = 3;
        i.armor_attributes.self_repair = 3;
    });
});

item("FleshRipper", |c| {
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1075045;
    c.on_create(|i| {
        i.hue = 0x341;
        i.skill_bonuses = [["Anatomy", 10.0]];
        i.attributes.bonus_str = 5;
        i.attributes.attack_chance = 15;
        i.attributes.weapon_speed = 40;
        i.weapon_attributes.use_best_skill = 1;
    });
});

item("HelmOfSwiftness", |c| {
    c.base_physical_resistance = 6;
    c.base_fire_resistance = 5;
    c.base_cold_resistance = 6;
    c.base_poison_resistance = 6;
    c.base_energy_resistance = 8;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1075037;
    c.on_create(|i| {
        i.hue = 0x592;
        i.attributes.bonus_int = 5;
        i.attributes.cast_speed = 1;
        i.attributes.cast_recovery = 2;
        i.armor_attributes.mage_armor = 1;
    });
});

item("RaedsGlory", |c| {
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1075036;
    c.on_create(|i| {
        i.item_id = 0x2D23;
        i.hue = 0x1E6;
        i.attributes.bonus_mana = 8;
        i.attributes.spell_channeling = 1;
        i.attributes.weapon_speed = 20;
        i.weapon_attributes.hit_leech_hits = 40;
    });
});

item("RighteousAnger", |c| {
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1075049;
    c.on_create(|i| {
        i.hue = 0x284;
        i.attributes.attack_chance = 15;
        i.attributes.defend_chance = 5;
        i.attributes.weapon_speed = 35;
        i.attributes.weapon_damage = 40;
    });
});

item("SoulSeeker", |c| {
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1075046;
    c.damage_types = #{ cold: 100 };
    c.on_create(|i| {
        i.hue = 0x38C;
        i.weapon_attributes.hit_leech_stam = 40;
        i.weapon_attributes.hit_leech_mana = 40;
        i.weapon_attributes.hit_leech_hits = 40;
        i.attributes.weapon_speed = 60;
        i.slayer = "Repond";
    });
});

item("TalonBite", |c| {
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1075029;
    c.on_create(|i| {
        i.item_id = 0x2D34;
        i.hue = 0x47E;
        i.skill_bonuses = [["Tactics", 10.0]];
        i.attributes.bonus_dex = 8;
        i.attributes.weapon_speed = 20;
        i.attributes.weapon_damage = 35;
        i.weapon_attributes.hit_harm = 33;
        i.weapon_attributes.use_best_skill = 1;
    });
});

item("NightsKiss", |c| {
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1063475;
    c.on_create(|i| {
        i.item_id = 0xF51;
        i.hue = 0x455;
        i.weapon_attributes.hit_leech_hits = 40;
        i.slayer = "Repond";
        i.attributes.weapon_speed = 30;
        i.attributes.weapon_damage = 35;
    });
});

item("NoxRangersHeavyCrossbow", |c| {
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1063485;
    c.damage_types = #{ phys: 50, pois: 50 };
    c.on_create(|i| {
        i.hue = 0x58C;
        i.weapon_attributes.hit_leech_stam = 40;
        i.weapon_attributes.resist_poison_bonus = 10;
        i.attributes.spell_channeling = 1;
        i.attributes.weapon_speed = 30;
        i.attributes.weapon_damage = 20;
    });
});

item("OrcishVisage", |c| {
    c.base_physical_resistance = 8;
    c.base_fire_resistance = 5;
    c.base_cold_resistance = 3;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 5;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1070691;
    c.on_create(|i| {
        i.hue = 0x592;
        i.armor_attributes.self_repair = 3;
        i.attributes.bonus_str = 10;
        i.attributes.bonus_stam = 5;
    });
});

item("PixieSwatter", |c| {
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1070854;
    c.damage_types = #{ fire: 100 };
    c.on_create(|i| {
        i.hue = 0x8A;
        i.weapon_attributes.hit_poison_area = 75;
        i.weapon_attributes.use_best_skill = 1;
        i.weapon_attributes.resist_fire_bonus = 12;
        i.weapon_attributes.resist_energy_bonus = 12;
        i.attributes.weapon_speed = 30;
        i.slayer = "Fey";
    });
});

item("StaffOfPower", |c| {
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1070692;
    c.on_create(|i| {
        i.hue = 0x4F2;
        i.weapon_attributes.mage_weapon = 15;
        i.attributes.spell_channeling = 1;
        i.attributes.spell_damage = 5;
        i.attributes.cast_recovery = 2;
        i.attributes.lower_mana_cost = 5;
    });
});

item("VioletCourage", |c| {
    c.base_physical_resistance = 14;
    c.base_fire_resistance = 12;
    c.base_cold_resistance = 12;
    c.base_poison_resistance = 8;
    c.base_energy_resistance = 9;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1063471;
    c.on_create(|i| {
        i.hue = 0x486;
        i.attributes.luck = 95;
        i.attributes.defend_chance = 15;
        i.armor_attributes.lower_stat_req = 100;
        i.armor_attributes.mage_armor = 1;
    });
});

item("CandelabraOfSouls", |c| {
    c.on_create(|i| {
        i.item_id = 0xB26;
    });
});

item("GhostShipAnchor", |c| {
    c.label = 1070816;
    c.on_create(|i| {
        i.item_id = 0x14F7;
        i.hue = 0x47E;
    });
});

item("GoldBricks", |c| {
    c.label = 1063489;
    c.on_create(|i| {
        i.item_id = 0x1BEB;
    });
});

item("ShipModelOfTheHMSCape", |c| {
    c.label = 1063476;
    c.on_create(|i| {
        i.item_id = 0x14F3;
        i.hue = 0x37B;
    });
});

item("PolarBearMask", |c| {
    c.label = 1070637;
    c.base_physical_resistance = 15;
    c.base_cold_resistance = 21;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.on_create(|i| {
        i.hue = 0x481;
        i.armor_attributes.self_repair = 3;
        i.attributes.regen_hits = 2;
        i.attributes.night_sight = 1;
    });
});

item("AlchemistsBauble", |c| {
    c.label = 1070638;
    c.on_create(|i| {
        i.hue = 0x290;
        i.skill_bonuses = [["Magery", 10.0]];
        i.attributes.enhance_potions = 30;
        i.attributes.lower_reg_cost = 20;
        i.element_attributes.poison = 10;
    });
});

item("MelisandesCorrodedHatchet", |c| {
    c.label = 1072115;
    c.on_create(|i| {
        i.hue = 0x494;
        i.skill_bonuses = [["Lumberjacking", 5.0]];
        i.attributes.spell_channeling = 1;
        i.attributes.weapon_speed = 15;
        i.attributes.weapon_damage = -50;
        i.weapon_attributes.self_repair = 4;
    });
});
