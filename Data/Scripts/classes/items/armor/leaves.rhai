item("ArmorOfFortune", |c| {
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1061098;
    c.rarity = 11;
    c.on_create(|i| {
        i.hue = 0x501;
        i.attributes.luck = 200;
        i.attributes.defend_chance = 15;
        i.attributes.lower_reg_cost = 40;
        i.armor_attributes.mage_armor = 1;
    });
});

item("BrambleCoat", |c| {
    c.base_physical_resistance = 10;
    c.base_fire_resistance = 8;
    c.base_cold_resistance = 7;
    c.base_poison_resistance = 8;
    c.base_energy_resistance = 7;
    c.label = 1072925;
    c.on_create(|i| {
        i.hue = 1;
        i.armor_attributes.self_repair = 3;
        i.attributes.bonus_hits = 4;
        i.attributes.luck = 150;
        i.attributes.reflect_physical = 25;
        i.attributes.defend_chance = 15;
    });
});

item("IronwoodCrown", |c| {
    c.base_physical_resistance = 10;
    c.base_fire_resistance = 6;
    c.base_cold_resistance = 7;
    c.base_poison_resistance = 7;
    c.base_energy_resistance = 10;
    c.label = 1072924;
    c.on_create(|i| {
        i.hue = 1;
        i.armor_attributes.self_repair = 3;
        i.attributes.bonus_str = 5;
        i.attributes.bonus_dex = 5;
        i.attributes.bonus_int = 5;
    });
});

item("SongWovenMantle", |c| {
    c.base_physical_resistance = 14;
    c.base_cold_resistance = 14;
    c.base_energy_resistance = 16;
    c.label = 1072931;
    c.on_create(|i| {
        i.hue = 0x493;
        i.skill_bonuses = [["Musicianship", 10.0]];
        i.attributes.luck = 100;
        i.attributes.defend_chance = 5;
    });
});

item("SpellWovenBritches", |c| {
    c.base_fire_resistance = 15;
    c.base_poison_resistance = 16;
    c.label = 1072929;
    c.on_create(|i| {
        i.hue = 0x487;
        i.skill_bonuses = [["Meditation", 10.0]];
        i.attributes.bonus_int = 8;
        i.attributes.spell_damage = 10;
        i.attributes.lower_mana_cost = 10;
    });
});

item("StitchersMittens", |c| {
    c.base_physical_resistance = 20;
    c.base_cold_resistance = 20;
    c.label = 1072932;
    c.on_create(|i| {
        i.hue = 0x481;
        i.skill_bonuses = [["Healing", 10.0]];
        i.attributes.bonus_dex = 5;
        i.attributes.lower_reg_cost = 30;
    });
});

item("GauntletsOfNobility", |c| {
    c.base_physical_resistance = 18;
    c.base_poison_resistance = 20;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1061092;
    c.rarity = 11;
    c.on_create(|i| {
        i.hue = 0x4FE;
        i.attributes.bonus_str = 8;
        i.attributes.luck = 100;
        i.attributes.weapon_damage = 20;
    });
});

item("HelmOfInsight", |c| {
    c.base_energy_resistance = 17;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1061096;
    c.rarity = 11;
    c.on_create(|i| {
        i.hue = 0x554;
        i.attributes.bonus_int = 8;
        i.attributes.bonus_mana = 15;
        i.attributes.regen_mana = 2;
        i.attributes.lower_mana_cost = 8;
    });
});

item("HolyKnightsBreastplate", |c| {
    c.base_physical_resistance = 35;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1061097;
    c.rarity = 11;
    c.on_create(|i| {
        i.hue = 0x47E;
        i.attributes.bonus_hits = 10;
        i.attributes.reflect_physical = 15;
    });
});

item("InquisitorsResolution", |c| {
    c.base_cold_resistance = 22;
    c.base_energy_resistance = 17;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1060206;
    c.rarity = 10;
    c.on_create(|i| {
        i.hue = 0x4F2;
        i.attributes.cast_recovery = 3;
        i.attributes.lower_mana_cost = 8;
        i.armor_attributes.mage_armor = 1;
    });
});

item("JackalsCollar", |c| {
    c.base_fire_resistance = 23;
    c.base_cold_resistance = 17;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1061594;
    c.rarity = 11;
    c.on_create(|i| {
        i.hue = 0x6D1;
        i.attributes.bonus_dex = 15;
        i.attributes.regen_hits = 2;
    });
});

item("LeggingsOfBane", |c| {
    c.base_poison_resistance = 36;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1061100;
    c.rarity = 11;
    c.on_create(|i| {
        i.hue = 0x4F5;
        i.armor_attributes.durability_bonus = 100;
        i.hit_points = 255;
        i.attributes.bonus_stam = 8;
        i.attributes.attack_chance = 20;
    });
});

item("MidnightBracers", |c| {
    c.base_physical_resistance = 23;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1061093;
    c.rarity = 11;
    c.on_create(|i| {
        i.hue = 0x455;
        i.skill_bonuses = [["Necromancy", 20.0]];
        i.attributes.spell_damage = 10;
        i.armor_attributes.mage_armor = 1;
    });
});

item("OrnateCrownOfTheHarrower", |c| {
    c.base_poison_resistance = 17;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1061095;
    c.rarity = 11;
    c.on_create(|i| {
        i.hue = 0x4F6;
        i.attributes.regen_hits = 2;
        i.attributes.regen_stam = 3;
        i.attributes.weapon_damage = 25;
    });
});

item("ShadowDancerLeggings", |c| {
    c.base_physical_resistance = 17;
    c.base_poison_resistance = 18;
    c.base_energy_resistance = 18;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1061598;
    c.rarity = 11;
    c.on_create(|i| {
        i.item_id = 0x13D2;
        i.hue = 0x455;
        i.skill_bonuses = [["Stealth", 20.0], ["Stealing", 20.0]];
    });
});

item("TunicOfFire", |c| {
    c.base_physical_resistance = 24;
    c.base_fire_resistance = 34;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1061099;
    c.rarity = 11;
    c.on_create(|i| {
        i.hue = 0x54F;
        i.armor_attributes.self_repair = 5;
        i.attributes.night_sight = 1;
        i.attributes.reflect_physical = 15;
    });
});

item("VoiceOfTheFallenKing", |c| {
    c.base_cold_resistance = 18;
    c.base_energy_resistance = 18;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1061094;
    c.rarity = 11;
    c.on_create(|i| {
        i.hue = 0x76D;
        i.attributes.bonus_str = 8;
        i.attributes.regen_hits = 5;
        i.attributes.regen_stam = 3;
    });
});

item("BoneArms", |c| {
    c.base_physical_resistance = 3;
    c.base_fire_resistance = 3;
    c.base_cold_resistance = 4;
    c.base_poison_resistance = 2;
    c.base_energy_resistance = 4;
    c.init_min_hits = 25;
    c.init_max_hits = 30;
    c.aos_str_req = 55;
    c.old_str_req = 40;
    c.old_dex_bonus = -2;
    c.armor_base = 30;
    c.weight = 2.0;
    c.material_type = "Bone";
    c.default_resource = "RegularLeather";
    c.graphic = 5198;
});

item("BoneChest", |c| {
    c.base_physical_resistance = 3;
    c.base_fire_resistance = 3;
    c.base_cold_resistance = 4;
    c.base_poison_resistance = 2;
    c.base_energy_resistance = 4;
    c.init_min_hits = 25;
    c.init_max_hits = 30;
    c.aos_str_req = 60;
    c.old_str_req = 40;
    c.old_dex_bonus = -6;
    c.armor_base = 30;
    c.weight = 6.0;
    c.material_type = "Bone";
    c.default_resource = "RegularLeather";
    c.graphic = 5199;
});

item("BoneGloves", |c| {
    c.base_physical_resistance = 3;
    c.base_fire_resistance = 3;
    c.base_cold_resistance = 4;
    c.base_poison_resistance = 2;
    c.base_energy_resistance = 4;
    c.init_min_hits = 25;
    c.init_max_hits = 30;
    c.aos_str_req = 55;
    c.old_str_req = 40;
    c.old_dex_bonus = -1;
    c.armor_base = 30;
    c.weight = 2.0;
    c.material_type = "Bone";
    c.default_resource = "RegularLeather";
    c.graphic = 5200;
});

item("BoneLegs", |c| {
    c.base_physical_resistance = 3;
    c.base_fire_resistance = 3;
    c.base_cold_resistance = 4;
    c.base_poison_resistance = 2;
    c.base_energy_resistance = 4;
    c.init_min_hits = 25;
    c.init_max_hits = 30;
    c.aos_str_req = 55;
    c.old_str_req = 40;
    c.old_dex_bonus = -4;
    c.armor_base = 30;
    c.weight = 3.0;
    c.material_type = "Bone";
    c.default_resource = "RegularLeather";
    c.graphic = 5202;
});

item("ChainHatsuburi", |c| {
    c.base_physical_resistance = 5;
    c.base_fire_resistance = 2;
    c.base_cold_resistance = 2;
    c.base_poison_resistance = 2;
    c.base_energy_resistance = 4;
    c.init_min_hits = 55;
    c.init_max_hits = 75;
    c.aos_str_req = 50;
    c.old_str_req = 50;
    c.armor_base = 3;
    c.weight = 7.0;
    c.material_type = "Chainmail";
    c.graphic = 10100;
});

item("ChainLegs", |c| {
    c.base_physical_resistance = 4;
    c.base_fire_resistance = 4;
    c.base_cold_resistance = 4;
    c.base_poison_resistance = 1;
    c.base_energy_resistance = 2;
    c.init_min_hits = 45;
    c.init_max_hits = 60;
    c.aos_str_req = 60;
    c.old_str_req = 20;
    c.old_dex_bonus = -3;
    c.armor_base = 28;
    c.weight = 7.0;
    c.material_type = "Chainmail";
    c.graphic = 5054;
});

item("GargishClothArmsType1", |c| {
    c.base_physical_resistance = 5;
    c.base_fire_resistance = 7;
    c.base_cold_resistance = 6;
    c.base_poison_resistance = 6;
    c.base_energy_resistance = 6;
    c.init_min_hits = 40;
    c.init_max_hits = 50;
    c.aos_str_req = 20;
    c.weight = 2.0;
    c.material_type = "Cloth";
    c.default_resource = "None";
    c.def_med_allowance = "All";
    c.required_races = "AllowGargoylesOnly";
    c.graphic = 1028;
});

item("GargishClothArmsType2", |c| {
    c.base_physical_resistance = 5;
    c.base_fire_resistance = 7;
    c.base_cold_resistance = 6;
    c.base_poison_resistance = 6;
    c.base_energy_resistance = 6;
    c.init_min_hits = 40;
    c.init_max_hits = 50;
    c.aos_str_req = 20;
    c.weight = 2.0;
    c.material_type = "Cloth";
    c.default_resource = "None";
    c.def_med_allowance = "All";
    c.required_races = "AllowGargoylesOnly";
    c.graphic = 1027;
});

item("GargishClothChestType1", |c| {
    c.base_physical_resistance = 5;
    c.base_fire_resistance = 7;
    c.base_cold_resistance = 6;
    c.base_poison_resistance = 6;
    c.base_energy_resistance = 6;
    c.init_min_hits = 40;
    c.init_max_hits = 50;
    c.aos_str_req = 25;
    c.weight = 2.0;
    c.material_type = "Cloth";
    c.default_resource = "None";
    c.def_med_allowance = "All";
    c.required_races = "AllowGargoylesOnly";
    c.graphic = 1030;
});

item("GargishClothChestType2", |c| {
    c.base_physical_resistance = 5;
    c.base_fire_resistance = 7;
    c.base_cold_resistance = 6;
    c.base_poison_resistance = 6;
    c.base_energy_resistance = 6;
    c.init_min_hits = 40;
    c.init_max_hits = 50;
    c.aos_str_req = 25;
    c.weight = 2.0;
    c.material_type = "Cloth";
    c.default_resource = "None";
    c.def_med_allowance = "All";
    c.required_races = "AllowGargoylesOnly";
    c.graphic = 1029;
});

item("GargishClothKiltType1", |c| {
    c.base_physical_resistance = 5;
    c.base_fire_resistance = 7;
    c.base_cold_resistance = 6;
    c.base_poison_resistance = 6;
    c.base_energy_resistance = 6;
    c.init_min_hits = 40;
    c.init_max_hits = 50;
    c.aos_str_req = 20;
    c.weight = 2.0;
    c.material_type = "Cloth";
    c.default_resource = "None";
    c.def_med_allowance = "All";
    c.required_races = "AllowGargoylesOnly";
    c.graphic = 1032;
});

item("GargishClothKiltType2", |c| {
    c.base_physical_resistance = 5;
    c.base_fire_resistance = 7;
    c.base_cold_resistance = 6;
    c.base_poison_resistance = 6;
    c.base_energy_resistance = 6;
    c.init_min_hits = 40;
    c.init_max_hits = 50;
    c.aos_str_req = 20;
    c.weight = 2.0;
    c.material_type = "Cloth";
    c.default_resource = "None";
    c.def_med_allowance = "All";
    c.required_races = "AllowGargoylesOnly";
    c.graphic = 1031;
});

item("GargishClothLegsType1", |c| {
    c.base_physical_resistance = 5;
    c.base_fire_resistance = 7;
    c.base_cold_resistance = 6;
    c.base_poison_resistance = 6;
    c.base_energy_resistance = 6;
    c.init_min_hits = 40;
    c.init_max_hits = 50;
    c.aos_str_req = 20;
    c.weight = 2.0;
    c.material_type = "Cloth";
    c.default_resource = "None";
    c.def_med_allowance = "All";
    c.required_races = "AllowGargoylesOnly";
    c.graphic = 1034;
});

item("GargishClothLegsType2", |c| {
    c.base_physical_resistance = 5;
    c.base_fire_resistance = 7;
    c.base_cold_resistance = 6;
    c.base_poison_resistance = 6;
    c.base_energy_resistance = 6;
    c.init_min_hits = 40;
    c.init_max_hits = 50;
    c.aos_str_req = 20;
    c.weight = 2.0;
    c.material_type = "Cloth";
    c.default_resource = "None";
    c.def_med_allowance = "All";
    c.required_races = "AllowGargoylesOnly";
    c.graphic = 1033;
});

item("DaemonArms", |c| {
    c.base_physical_resistance = 6;
    c.base_fire_resistance = 6;
    c.base_cold_resistance = 7;
    c.base_poison_resistance = 5;
    c.base_energy_resistance = 7;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.aos_str_req = 55;
    c.old_str_req = 40;
    c.old_dex_bonus = -2;
    c.armor_base = 46;
    c.label = 1041371;
    c.weight = 2.0;
    c.material_type = "Bone";
    c.default_resource = "RegularLeather";
    c.graphic = 5198;
    c.on_create(|i| {
        i.hue = 0x648;
        i.armor_attributes.self_repair = 1;
    });
});

item("DaemonChest", |c| {
    c.base_physical_resistance = 6;
    c.base_fire_resistance = 6;
    c.base_cold_resistance = 7;
    c.base_poison_resistance = 5;
    c.base_energy_resistance = 7;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.aos_str_req = 60;
    c.old_str_req = 40;
    c.old_dex_bonus = -6;
    c.armor_base = 46;
    c.label = 1041372;
    c.weight = 6.0;
    c.material_type = "Bone";
    c.default_resource = "RegularLeather";
    c.graphic = 5199;
    c.on_create(|i| {
        i.hue = 0x648;
        i.armor_attributes.self_repair = 1;
    });
});

item("DaemonGloves", |c| {
    c.base_physical_resistance = 6;
    c.base_fire_resistance = 6;
    c.base_cold_resistance = 7;
    c.base_poison_resistance = 5;
    c.base_energy_resistance = 7;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.aos_str_req = 55;
    c.old_str_req = 40;
    c.old_dex_bonus = -1;
    c.armor_base = 46;
    c.label = 1041373;
    c.weight = 2.0;
    c.material_type = "Bone";
    c.default_resource = "RegularLeather";
    c.graphic = 5200;
    c.on_create(|i| {
        i.hue = 0x648;
        i.armor_attributes.self_repair = 1;
    });
});

item("DaemonLegs", |c| {
    c.base_physical_resistance = 6;
    c.base_fire_resistance = 6;
    c.base_cold_resistance = 7;
    c.base_poison_resistance = 5;
    c.base_energy_resistance = 7;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.aos_str_req = 55;
    c.old_str_req = 40;
    c.old_dex_bonus = -4;
    c.armor_base = 46;
    c.label = 1041375;
    c.weight = 3.0;
    c.material_type = "Bone";
    c.default_resource = "RegularLeather";
    c.graphic = 5202;
    c.on_create(|i| {
        i.hue = 0x648;
        i.armor_attributes.self_repair = 1;
    });
});

item("DragonArms", |c| {
    c.base_physical_resistance = 3;
    c.base_fire_resistance = 3;
    c.base_cold_resistance = 3;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 3;
    c.init_min_hits = 55;
    c.init_max_hits = 75;
    c.aos_str_req = 75;
    c.old_str_req = 20;
    c.old_dex_bonus = -2;
    c.armor_base = 40;
    c.weight = 5.0;
    c.material_type = "Dragon";
    c.default_resource = "RedScales";
    c.graphic = 9815;
});

item("DragonChest", |c| {
    c.base_physical_resistance = 3;
    c.base_fire_resistance = 3;
    c.base_cold_resistance = 3;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 3;
    c.init_min_hits = 55;
    c.init_max_hits = 75;
    c.aos_str_req = 75;
    c.old_str_req = 60;
    c.old_dex_bonus = -8;
    c.armor_base = 40;
    c.weight = 10.0;
    c.material_type = "Dragon";
    c.default_resource = "RedScales";
    c.graphic = 9793;
});

item("DragonGloves", |c| {
    c.base_physical_resistance = 3;
    c.base_fire_resistance = 3;
    c.base_cold_resistance = 3;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 3;
    c.init_min_hits = 55;
    c.init_max_hits = 75;
    c.aos_str_req = 75;
    c.old_str_req = 30;
    c.old_dex_bonus = -2;
    c.armor_base = 40;
    c.weight = 2.0;
    c.material_type = "Dragon";
    c.default_resource = "RedScales";
    c.graphic = 9795;
});

item("DragonHelm", |c| {
    c.base_physical_resistance = 3;
    c.base_fire_resistance = 3;
    c.base_cold_resistance = 3;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 3;
    c.init_min_hits = 55;
    c.init_max_hits = 75;
    c.aos_str_req = 75;
    c.old_str_req = 40;
    c.old_dex_bonus = -1;
    c.armor_base = 40;
    c.weight = 5.0;
    c.material_type = "Dragon";
    c.default_resource = "RedScales";
    c.graphic = 9797;
});

item("DragonLegs", |c| {
    c.base_physical_resistance = 3;
    c.base_fire_resistance = 3;
    c.base_cold_resistance = 3;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 3;
    c.init_min_hits = 55;
    c.init_max_hits = 75;
    c.aos_str_req = 75;
    c.old_str_req = 60;
    c.old_dex_bonus = -6;
    c.armor_base = 40;
    c.weight = 6.0;
    c.material_type = "Dragon";
    c.default_resource = "RedScales";
    c.graphic = 9799;
});

item("AnthropomorphistGlasses", |c| {
    c.base_physical_resistance = 5;
    c.base_fire_resistance = 5;
    c.base_cold_resistance = 10;
    c.base_poison_resistance = 20;
    c.base_energy_resistance = 20;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1073379;
    c.on_create(|i| {
        i.attributes.bonus_hits = 5;
        i.attributes.regen_mana = 3;
        i.attributes.reflect_physical = 20;
        i.hue = 0x80;
    });
});

item("ArtsGlasses", |c| {
    c.base_physical_resistance = 10;
    c.base_fire_resistance = 8;
    c.base_cold_resistance = 8;
    c.base_poison_resistance = 4;
    c.base_energy_resistance = 10;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1073363;
    c.on_create(|i| {
        i.attributes.bonus_str = 5;
        i.attributes.bonus_int = 5;
        i.attributes.bonus_hits = 15;
        i.hue = 0x73;
    });
});

item("FoldedSteelGlasses", |c| {
    c.base_physical_resistance = 20;
    c.base_fire_resistance = 10;
    c.base_cold_resistance = 10;
    c.base_poison_resistance = 10;
    c.base_energy_resistance = 10;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1073380;
    c.on_create(|i| {
        i.attributes.bonus_str = 8;
        i.attributes.night_sight = 1;
        i.attributes.defend_chance = 15;
        i.hue = 0x47E;
    });
});

item("LightOfWayGlasses", |c| {
    c.base_physical_resistance = 10;
    c.base_fire_resistance = 10;
    c.base_cold_resistance = 10;
    c.base_poison_resistance = 10;
    c.base_energy_resistance = 10;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1073378;
    c.on_create(|i| {
        i.attributes.bonus_str = 7;
        i.attributes.bonus_int = 5;
        i.attributes.weapon_damage = 30;
        i.hue = 0x256;
    });
});

item("LyricalGlasses", |c| {
    c.base_physical_resistance = 10;
    c.base_fire_resistance = 10;
    c.base_cold_resistance = 10;
    c.base_poison_resistance = 10;
    c.base_energy_resistance = 10;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1073382;
    c.on_create(|i| {
        i.weapon_attributes.hit_lower_defend = 20;
        i.attributes.night_sight = 1;
        i.attributes.reflect_physical = 15;
        i.hue = 0x47F;
    });
});

item("MaceShieldGlasses", |c| {
    c.base_physical_resistance = 25;
    c.base_fire_resistance = 10;
    c.base_cold_resistance = 10;
    c.base_poison_resistance = 10;
    c.base_energy_resistance = 10;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1073381;
    c.on_create(|i| {
        i.weapon_attributes.hit_lower_defend = 30;
        i.attributes.bonus_str = 10;
        i.attributes.bonus_dex = 5;
        i.hue = 0x1DD;
    });
});

item("MaritimeGlasses", |c| {
    c.base_physical_resistance = 3;
    c.base_fire_resistance = 4;
    c.base_cold_resistance = 30;
    c.base_poison_resistance = 5;
    c.base_energy_resistance = 3;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1073364;
    c.on_create(|i| {
        i.attributes.luck = 150;
        i.attributes.night_sight = 1;
        i.attributes.reflect_physical = 20;
        i.hue = 0x581;
    });
});

item("NecromanticGlasses", |c| {
    c.base_physical_resistance = 0;
    c.base_fire_resistance = 0;
    c.base_cold_resistance = 0;
    c.base_poison_resistance = 0;
    c.base_energy_resistance = 0;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1073377;
    c.on_create(|i| {
        i.attributes.lower_mana_cost = 15;
        i.attributes.lower_reg_cost = 30;
        i.hue = 0x22D;
    });
});

item("PoisonedGlasses", |c| {
    c.base_physical_resistance = 10;
    c.base_fire_resistance = 10;
    c.base_cold_resistance = 10;
    c.base_poison_resistance = 30;
    c.base_energy_resistance = 10;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1073376;
    c.on_create(|i| {
        i.attributes.bonus_stam = 3;
        i.attributes.regen_stam = 4;
        i.hue = 0x113;
    });
});

item("TradeGlasses", |c| {
    c.base_physical_resistance = 10;
    c.base_fire_resistance = 10;
    c.base_cold_resistance = 10;
    c.base_poison_resistance = 10;
    c.base_energy_resistance = 10;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1073362;
    c.on_create(|i| {
        i.attributes.bonus_str = 10;
        i.attributes.bonus_int = 10;
    });
});

item("TreasureTrinketGlasses", |c| {
    c.base_physical_resistance = 10;
    c.base_fire_resistance = 10;
    c.base_cold_resistance = 10;
    c.base_poison_resistance = 10;
    c.base_energy_resistance = 10;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1073373;
    c.on_create(|i| {
        i.attributes.bonus_int = 10;
        i.attributes.bonus_hits = 5;
        i.attributes.spell_damage = 10;
        i.hue = 0x1C2;
    });
});

item("WizardsGlasses", |c| {
    c.base_physical_resistance = 5;
    c.base_fire_resistance = 5;
    c.base_cold_resistance = 5;
    c.base_poison_resistance = 5;
    c.base_energy_resistance = 5;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1073374;
    c.on_create(|i| {
        i.attributes.bonus_mana = 10;
        i.attributes.regen_mana = 3;
        i.attributes.spell_damage = 15;
        i.hue = 0x2B0;
    });
});

item("Bascinet", |c| {
    c.base_physical_resistance = 7;
    c.base_fire_resistance = 2;
    c.base_cold_resistance = 2;
    c.base_poison_resistance = 2;
    c.base_energy_resistance = 2;
    c.init_min_hits = 40;
    c.init_max_hits = 50;
    c.aos_str_req = 40;
    c.old_str_req = 10;
    c.armor_base = 18;
    c.weight = 5.0;
    c.material_type = "Plate";
    c.graphic = 5132;
});

item("BoneHelm", |c| {
    c.base_physical_resistance = 3;
    c.base_fire_resistance = 3;
    c.base_cold_resistance = 4;
    c.base_poison_resistance = 2;
    c.base_energy_resistance = 4;
    c.init_min_hits = 25;
    c.init_max_hits = 30;
    c.aos_str_req = 20;
    c.old_str_req = 40;
    c.armor_base = 30;
    c.weight = 3.0;
    c.material_type = "Bone";
    c.graphic = 5201;
});

item("ChainCoif", |c| {
    c.base_physical_resistance = 4;
    c.base_fire_resistance = 4;
    c.base_cold_resistance = 4;
    c.base_poison_resistance = 1;
    c.base_energy_resistance = 2;
    c.init_min_hits = 35;
    c.init_max_hits = 60;
    c.aos_str_req = 60;
    c.old_str_req = 20;
    c.armor_base = 28;
    c.weight = 1.0;
    c.material_type = "Chainmail";
    c.graphic = 5051;
});

item("Circlet", |c| {
    c.base_physical_resistance = 1;
    c.base_fire_resistance = 5;
    c.base_cold_resistance = 2;
    c.base_poison_resistance = 2;
    c.base_energy_resistance = 5;
    c.init_min_hits = 50;
    c.init_max_hits = 65;
    c.aos_str_req = 10;
    c.old_str_req = 10;
    c.armor_base = 30;
    c.weight = 2.0;
    c.material_type = "Plate";
    c.def_med_allowance = "All";
    c.required_races = "AllowElvesOnly";
    c.graphic = 11118;
});

item("CloseHelm", |c| {
    c.base_physical_resistance = 3;
    c.base_fire_resistance = 3;
    c.base_cold_resistance = 3;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 3;
    c.init_min_hits = 45;
    c.init_max_hits = 60;
    c.aos_str_req = 55;
    c.old_str_req = 40;
    c.armor_base = 30;
    c.weight = 5.0;
    c.material_type = "Plate";
    c.graphic = 5128;
});

item("DaemonHelm", |c| {
    c.base_physical_resistance = 6;
    c.base_fire_resistance = 6;
    c.base_cold_resistance = 7;
    c.base_poison_resistance = 5;
    c.base_energy_resistance = 7;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.aos_str_req = 20;
    c.old_str_req = 40;
    c.armor_base = 46;
    c.label = 1041374;
    c.weight = 3.0;
    c.material_type = "Bone";
    c.default_resource = "RegularLeather";
    c.graphic = 5201;
    c.on_create(|i| {
        i.hue = 0x648;
        i.armor_attributes.self_repair = 1;
    });
});

item("GemmedCirclet", |c| {
    c.base_physical_resistance = 1;
    c.base_fire_resistance = 5;
    c.base_cold_resistance = 2;
    c.base_poison_resistance = 2;
    c.base_energy_resistance = 5;
    c.init_min_hits = 20;
    c.init_max_hits = 35;
    c.aos_str_req = 10;
    c.old_str_req = 10;
    c.armor_base = 30;
    c.weight = 2.0;
    c.required_races = "AllowElvesOnly";
    c.material_type = "Plate";
    c.def_med_allowance = "All";
    c.graphic = 11120;
});

item("Helmet", |c| {
    c.base_physical_resistance = 2;
    c.base_fire_resistance = 4;
    c.base_cold_resistance = 4;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 2;
    c.init_min_hits = 45;
    c.init_max_hits = 60;
    c.aos_str_req = 45;
    c.old_str_req = 40;
    c.armor_base = 30;
    c.weight = 5.0;
    c.material_type = "Plate";
    c.graphic = 5130;
});

item("LeatherCap", |c| {
    c.base_physical_resistance = 2;
    c.base_fire_resistance = 4;
    c.base_cold_resistance = 3;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 3;
    c.init_min_hits = 30;
    c.init_max_hits = 40;
    c.aos_str_req = 20;
    c.old_str_req = 15;
    c.armor_base = 13;
    c.weight = 2.0;
    c.material_type = "Leather";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "All";
    c.graphic = 7609;
});

item("NorseHelm", |c| {
    c.base_physical_resistance = 4;
    c.base_fire_resistance = 1;
    c.base_cold_resistance = 4;
    c.base_poison_resistance = 4;
    c.base_energy_resistance = 2;
    c.init_min_hits = 45;
    c.init_max_hits = 60;
    c.aos_str_req = 55;
    c.old_str_req = 40;
    c.armor_base = 30;
    c.weight = 5.0;
    c.material_type = "Plate";
    c.graphic = 5134;
});

item("OrcHelm", |c| {
    c.base_physical_resistance = 3;
    c.base_fire_resistance = 1;
    c.base_cold_resistance = 3;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 5;
    c.init_min_hits = 30;
    c.init_max_hits = 50;
    c.aos_str_req = 30;
    c.old_str_req = 10;
    c.armor_base = 20;
    c.weight = 5.0;
    c.material_type = "Leather";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "None";
    c.graphic = 7947;
});

item("PlateHelm", |c| {
    c.base_physical_resistance = 5;
    c.base_fire_resistance = 3;
    c.base_cold_resistance = 2;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 2;
    c.init_min_hits = 50;
    c.init_max_hits = 65;
    c.aos_str_req = 80;
    c.old_str_req = 40;
    c.old_dex_bonus = -1;
    c.armor_base = 40;
    c.weight = 5.0;
    c.material_type = "Plate";
    c.graphic = 5138;
});

item("RavenHelm", |c| {
    c.base_physical_resistance = 5;
    c.base_fire_resistance = 1;
    c.base_cold_resistance = 2;
    c.base_poison_resistance = 2;
    c.base_energy_resistance = 5;
    c.init_min_hits = 50;
    c.init_max_hits = 65;
    c.aos_str_req = 25;
    c.old_str_req = 25;
    c.armor_base = 40;
    c.weight = 5.0;
    c.required_races = "AllowElvesOnly";
    c.material_type = "Plate";
    c.graphic = 11121;
});

item("RoyalCirclet", |c| {
    c.base_physical_resistance = 1;
    c.base_fire_resistance = 5;
    c.base_cold_resistance = 2;
    c.base_poison_resistance = 2;
    c.base_energy_resistance = 5;
    c.init_min_hits = 20;
    c.init_max_hits = 35;
    c.aos_str_req = 10;
    c.old_str_req = 10;
    c.armor_base = 30;
    c.weight = 2.0;
    c.required_races = "AllowElvesOnly";
    c.material_type = "Plate";
    c.def_med_allowance = "All";
    c.graphic = 11119;
});

item("VultureHelm", |c| {
    c.base_physical_resistance = 5;
    c.base_fire_resistance = 1;
    c.base_cold_resistance = 2;
    c.base_poison_resistance = 2;
    c.base_energy_resistance = 5;
    c.init_min_hits = 50;
    c.init_max_hits = 65;
    c.aos_str_req = 25;
    c.old_str_req = 25;
    c.armor_base = 40;
    c.weight = 5.0;
    c.required_races = "AllowElvesOnly";
    c.material_type = "Plate";
    c.graphic = 11122;
});

item("WingedHelm", |c| {
    c.base_physical_resistance = 5;
    c.base_fire_resistance = 1;
    c.base_cold_resistance = 2;
    c.base_poison_resistance = 2;
    c.base_energy_resistance = 5;
    c.init_min_hits = 45;
    c.init_max_hits = 55;
    c.aos_str_req = 25;
    c.old_str_req = 25;
    c.armor_base = 40;
    c.weight = 5.0;
    c.required_races = "AllowElvesOnly";
    c.material_type = "Plate";
    c.graphic = 11123;
});

item("FemaleLeafChest", |c| {
    c.base_physical_resistance = 2;
    c.base_fire_resistance = 3;
    c.base_cold_resistance = 2;
    c.base_poison_resistance = 4;
    c.base_energy_resistance = 4;
    c.init_min_hits = 30;
    c.init_max_hits = 40;
    c.aos_str_req = 20;
    c.old_str_req = 20;
    c.armor_base = 13;
    c.weight = 2.0;
    c.material_type = "Leather";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "All";
    c.allow_male_wearer = false;
    c.graphic = 12235;
});

item("FemaleLeatherChest", |c| {
    c.base_physical_resistance = 2;
    c.base_fire_resistance = 4;
    c.base_cold_resistance = 3;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 3;
    c.init_min_hits = 30;
    c.init_max_hits = 40;
    c.aos_str_req = 25;
    c.old_str_req = 15;
    c.armor_base = 13;
    c.weight = 1.0;
    c.material_type = "Leather";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "All";
    c.allow_male_wearer = false;
    c.graphic = 7174;
});

item("GargishLeatherArmsType1", |c| {
    c.base_physical_resistance = 5;
    c.base_fire_resistance = 6;
    c.base_cold_resistance = 7;
    c.base_poison_resistance = 6;
    c.base_energy_resistance = 6;
    c.init_min_hits = 30;
    c.init_max_hits = 50;
    c.aos_str_req = 25;
    c.old_str_req = 25;
    c.armor_base = 13;
    c.weight = 4.0;
    c.required_races = "AllowGargoylesOnly";
    c.material_type = "Leather";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "All";
    c.graphic = 769;
});

item("GargishLeatherArmsType2", |c| {
    c.base_physical_resistance = 5;
    c.base_fire_resistance = 6;
    c.base_cold_resistance = 7;
    c.base_poison_resistance = 6;
    c.base_energy_resistance = 6;
    c.init_min_hits = 30;
    c.init_max_hits = 50;
    c.aos_str_req = 25;
    c.old_str_req = 25;
    c.armor_base = 13;
    c.weight = 4.0;
    c.required_races = "AllowGargoylesOnly";
    c.material_type = "Leather";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "All";
    c.graphic = 770;
});

item("GargishLeatherChestType1", |c| {
    c.base_physical_resistance = 5;
    c.base_fire_resistance = 6;
    c.base_cold_resistance = 7;
    c.base_poison_resistance = 6;
    c.base_energy_resistance = 6;
    c.init_min_hits = 30;
    c.init_max_hits = 50;
    c.aos_str_req = 25;
    c.old_str_req = 25;
    c.armor_base = 13;
    c.weight = 4.0;
    c.required_races = "AllowGargoylesOnly";
    c.material_type = "Leather";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "All";
    c.graphic = 772;
});

item("GargishLeatherChestType2", |c| {
    c.base_physical_resistance = 5;
    c.base_fire_resistance = 6;
    c.base_cold_resistance = 7;
    c.base_poison_resistance = 6;
    c.base_energy_resistance = 6;
    c.init_min_hits = 30;
    c.init_max_hits = 50;
    c.aos_str_req = 25;
    c.old_str_req = 25;
    c.armor_base = 13;
    c.weight = 4.0;
    c.required_races = "AllowGargoylesOnly";
    c.material_type = "Leather";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "All";
    c.graphic = 771;
});

item("GargishLeatherKiltType1", |c| {
    c.base_physical_resistance = 5;
    c.base_fire_resistance = 6;
    c.base_cold_resistance = 7;
    c.base_poison_resistance = 6;
    c.base_energy_resistance = 6;
    c.init_min_hits = 30;
    c.init_max_hits = 50;
    c.aos_str_req = 25;
    c.old_str_req = 25;
    c.armor_base = 13;
    c.weight = 4.0;
    c.required_races = "AllowGargoylesOnly";
    c.material_type = "Leather";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "All";
    c.graphic = 785;
});

item("GargishLeatherKiltType2", |c| {
    c.base_physical_resistance = 5;
    c.base_fire_resistance = 6;
    c.base_cold_resistance = 7;
    c.base_poison_resistance = 6;
    c.base_energy_resistance = 6;
    c.init_min_hits = 30;
    c.init_max_hits = 50;
    c.aos_str_req = 25;
    c.old_str_req = 25;
    c.armor_base = 13;
    c.weight = 4.0;
    c.required_races = "AllowGargoylesOnly";
    c.material_type = "Leather";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "All";
    c.graphic = 784;
});

item("GargishLeatherLegsType1", |c| {
    c.base_physical_resistance = 5;
    c.base_fire_resistance = 6;
    c.base_cold_resistance = 7;
    c.base_poison_resistance = 6;
    c.base_energy_resistance = 6;
    c.init_min_hits = 30;
    c.init_max_hits = 50;
    c.aos_str_req = 25;
    c.old_str_req = 25;
    c.armor_base = 13;
    c.weight = 4.0;
    c.required_races = "AllowGargoylesOnly";
    c.material_type = "Leather";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "All";
    c.graphic = 773;
});

item("GargishLeatherLegsType2", |c| {
    c.base_physical_resistance = 5;
    c.base_fire_resistance = 6;
    c.base_cold_resistance = 7;
    c.base_poison_resistance = 6;
    c.base_energy_resistance = 6;
    c.init_min_hits = 30;
    c.init_max_hits = 50;
    c.aos_str_req = 25;
    c.old_str_req = 25;
    c.armor_base = 13;
    c.weight = 4.0;
    c.required_races = "AllowGargoylesOnly";
    c.material_type = "Leather";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "All";
    c.graphic = 774;
});

item("GargishLeatherWingArmor", |c| {
    c.aos_str_req = 10;
    c.old_str_req = 10;
    c.armor_base = 0;
    c.weight = 2.0;
    c.required_races = "AllowGargoylesOnly";
    c.material_type = "Leather";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "All";
    c.graphic = 17790;
    c.physical_resistance = 0;
    c.fire_resistance = 0;
    c.cold_resistance = 0;
    c.poison_resistance = 0;
    c.energy_resistance = 0;
});

item("LeafArms", |c| {
    c.base_physical_resistance = 2;
    c.base_fire_resistance = 3;
    c.base_cold_resistance = 2;
    c.base_poison_resistance = 4;
    c.base_energy_resistance = 4;
    c.init_min_hits = 30;
    c.init_max_hits = 40;
    c.aos_str_req = 15;
    c.old_str_req = 15;
    c.armor_base = 13;
    c.weight = 2.0;
    c.required_races = "AllowElvesOnly";
    c.material_type = "Leather";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "All";
    c.graphic = 12232;
});

item("LeafChest", |c| {
    c.base_physical_resistance = 2;
    c.base_fire_resistance = 3;
    c.base_cold_resistance = 2;
    c.base_poison_resistance = 4;
    c.base_energy_resistance = 4;
    c.init_min_hits = 30;
    c.init_max_hits = 40;
    c.aos_str_req = 20;
    c.old_str_req = 20;
    c.armor_base = 13;
    c.weight = 2.0;
    c.required_races = "AllowElvesOnly";
    c.material_type = "Leather";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "All";
    c.graphic = 12229;
});

item("LeafGorget", |c| {
    c.base_physical_resistance = 2;
    c.base_fire_resistance = 3;
    c.base_cold_resistance = 2;
    c.base_poison_resistance = 4;
    c.base_energy_resistance = 4;
    c.init_min_hits = 30;
    c.init_max_hits = 40;
    c.aos_str_req = 10;
    c.old_str_req = 10;
    c.armor_base = 13;
    c.weight = 2.0;
    c.required_races = "AllowElvesOnly";
    c.material_type = "Leather";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "All";
    c.graphic = 12231;
});

item("LeafLegs", |c| {
    c.base_physical_resistance = 2;
    c.base_fire_resistance = 3;
    c.base_cold_resistance = 2;
    c.base_poison_resistance = 4;
    c.base_energy_resistance = 4;
    c.init_min_hits = 30;
    c.init_max_hits = 40;
    c.aos_str_req = 20;
    c.old_str_req = 20;
    c.armor_base = 13;
    c.weight = 2.0;
    c.required_races = "AllowElvesOnly";
    c.material_type = "Leather";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "All";
    c.graphic = 12233;
});

item("LeafTonlet", |c| {
    c.base_physical_resistance = 2;
    c.base_fire_resistance = 3;
    c.base_cold_resistance = 2;
    c.base_poison_resistance = 4;
    c.base_energy_resistance = 4;
    c.init_min_hits = 30;
    c.init_max_hits = 40;
    c.aos_str_req = 10;
    c.old_str_req = 10;
    c.armor_base = 13;
    c.weight = 2.0;
    c.required_races = "AllowElvesOnly";
    c.material_type = "Leather";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "All";
    c.graphic = 12234;
});

item("LeatherArms", |c| {
    c.base_physical_resistance = 2;
    c.base_fire_resistance = 4;
    c.base_cold_resistance = 3;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 3;
    c.init_min_hits = 30;
    c.init_max_hits = 40;
    c.aos_str_req = 20;
    c.old_str_req = 15;
    c.armor_base = 13;
    c.weight = 2.0;
    c.material_type = "Leather";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "All";
    c.graphic = 5069;
});

item("LeatherBustierArms", |c| {
    c.base_physical_resistance = 2;
    c.base_fire_resistance = 4;
    c.base_cold_resistance = 3;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 3;
    c.init_min_hits = 30;
    c.init_max_hits = 40;
    c.aos_str_req = 20;
    c.old_str_req = 15;
    c.armor_base = 13;
    c.weight = 1.0;
    c.material_type = "Leather";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "All";
    c.allow_male_wearer = false;
    c.graphic = 7178;
});

item("LeatherDo", |c| {
    c.base_physical_resistance = 2;
    c.base_fire_resistance = 4;
    c.base_cold_resistance = 3;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 3;
    c.init_min_hits = 35;
    c.init_max_hits = 45;
    c.aos_str_req = 40;
    c.old_str_req = 40;
    c.armor_base = 3;
    c.weight = 6.0;
    c.material_type = "Leather";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "All";
    c.graphic = 10182;
});

item("LeatherGorget", |c| {
    c.base_physical_resistance = 2;
    c.base_fire_resistance = 4;
    c.base_cold_resistance = 3;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 3;
    c.init_min_hits = 30;
    c.init_max_hits = 40;
    c.aos_str_req = 20;
    c.old_str_req = 10;
    c.armor_base = 13;
    c.weight = 1.0;
    c.material_type = "Leather";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "All";
    c.graphic = 5063;
});

item("LeatherHaidate", |c| {
    c.base_physical_resistance = 2;
    c.base_fire_resistance = 4;
    c.base_cold_resistance = 3;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 3;
    c.init_min_hits = 30;
    c.init_max_hits = 40;
    c.aos_str_req = 20;
    c.old_str_req = 20;
    c.armor_base = 3;
    c.weight = 4.0;
    c.material_type = "Leather";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "All";
    c.graphic = 10122;
});

item("LeatherHiroSode", |c| {
    c.base_physical_resistance = 2;
    c.base_fire_resistance = 4;
    c.base_cold_resistance = 3;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 3;
    c.init_min_hits = 40;
    c.init_max_hits = 50;
    c.aos_str_req = 25;
    c.old_str_req = 25;
    c.armor_base = 3;
    c.weight = 1.0;
    c.material_type = "Leather";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "All";
    c.graphic = 10110;
});

item("LeatherJingasa", |c| {
    c.base_physical_resistance = 4;
    c.base_fire_resistance = 3;
    c.base_cold_resistance = 3;
    c.base_poison_resistance = 2;
    c.base_energy_resistance = 3;
    c.init_min_hits = 20;
    c.init_max_hits = 30;
    c.aos_str_req = 25;
    c.old_str_req = 25;
    c.armor_base = 3;
    c.weight = 3.0;
    c.material_type = "Leather";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "All";
    c.graphic = 10102;
});

item("LeatherLegs", |c| {
    c.base_physical_resistance = 2;
    c.base_fire_resistance = 4;
    c.base_cold_resistance = 3;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 3;
    c.init_min_hits = 30;
    c.init_max_hits = 40;
    c.aos_str_req = 20;
    c.old_str_req = 10;
    c.armor_base = 13;
    c.weight = 4.0;
    c.material_type = "Leather";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "All";
    c.graphic = 5067;
});

item("LeatherMempo", |c| {
    c.base_physical_resistance = 2;
    c.base_fire_resistance = 4;
    c.base_cold_resistance = 3;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 3;
    c.init_min_hits = 35;
    c.init_max_hits = 40;
    c.aos_str_req = 30;
    c.old_str_req = 30;
    c.armor_base = 3;
    c.weight = 2.0;
    c.material_type = "Leather";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "All";
    c.graphic = 10106;
});

item("LeatherNinjaHood", |c| {
    c.base_physical_resistance = 2;
    c.base_fire_resistance = 3;
    c.base_cold_resistance = 3;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 4;
    c.init_min_hits = 25;
    c.init_max_hits = 45;
    c.aos_str_req = 10;
    c.old_str_req = 10;
    c.armor_base = 3;
    c.weight = 2.0;
    c.material_type = "Leather";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "All";
    c.graphic = 10126;
});

item("LeatherNinjaJacket", |c| {
    c.base_physical_resistance = 2;
    c.base_fire_resistance = 4;
    c.base_cold_resistance = 3;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 3;
    c.init_min_hits = 55;
    c.init_max_hits = 65;
    c.aos_str_req = 10;
    c.old_str_req = 10;
    c.armor_base = 3;
    c.weight = 5.0;
    c.material_type = "Leather";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "All";
    c.graphic = 10131;
});

item("LeatherNinjaMitts", |c| {
    c.base_physical_resistance = 2;
    c.base_fire_resistance = 4;
    c.base_cold_resistance = 3;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 3;
    c.init_min_hits = 25;
    c.init_max_hits = 25;
    c.aos_str_req = 10;
    c.old_str_req = 10;
    c.armor_base = 3;
    c.weight = 2.0;
    c.material_type = "Leather";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "All";
    c.graphic = 10130;
});

item("LeatherNinjaPants", |c| {
    c.base_physical_resistance = 2;
    c.base_fire_resistance = 4;
    c.base_cold_resistance = 3;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 3;
    c.init_min_hits = 40;
    c.init_max_hits = 50;
    c.aos_str_req = 10;
    c.old_str_req = 10;
    c.armor_base = 3;
    c.weight = 3.0;
    c.material_type = "Leather";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "All";
    c.graphic = 10129;
});

item("LeatherShorts", |c| {
    c.base_physical_resistance = 2;
    c.base_fire_resistance = 4;
    c.base_cold_resistance = 3;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 3;
    c.init_min_hits = 30;
    c.init_max_hits = 40;
    c.aos_str_req = 20;
    c.old_str_req = 10;
    c.armor_base = 13;
    c.weight = 3.0;
    c.material_type = "Leather";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "All";
    c.allow_male_wearer = false;
    c.graphic = 7168;
});

item("LeatherSkirt", |c| {
    c.base_physical_resistance = 2;
    c.base_fire_resistance = 4;
    c.base_cold_resistance = 3;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 3;
    c.init_min_hits = 30;
    c.init_max_hits = 40;
    c.aos_str_req = 20;
    c.old_str_req = 10;
    c.armor_base = 13;
    c.weight = 1.0;
    c.material_type = "Leather";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "All";
    c.allow_male_wearer = false;
    c.graphic = 7176;
});

item("LeatherSuneate", |c| {
    c.base_physical_resistance = 2;
    c.base_fire_resistance = 4;
    c.base_cold_resistance = 3;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 3;
    c.init_min_hits = 25;
    c.init_max_hits = 40;
    c.aos_str_req = 20;
    c.old_str_req = 20;
    c.armor_base = 3;
    c.weight = 4.0;
    c.material_type = "Leather";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "All";
    c.graphic = 10118;
});

item("DecorativePlateKabuto", |c| {
    c.base_physical_resistance = 6;
    c.base_fire_resistance = 2;
    c.base_cold_resistance = 2;
    c.base_poison_resistance = 2;
    c.base_energy_resistance = 3;
    c.init_min_hits = 55;
    c.init_max_hits = 75;
    c.aos_str_req = 70;
    c.old_str_req = 70;
    c.armor_base = 3;
    c.weight = 6.0;
    c.material_type = "Plate";
    c.graphic = 10104;
});

item("FemalePlateChest", |c| {
    c.base_physical_resistance = 5;
    c.base_fire_resistance = 3;
    c.base_cold_resistance = 2;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 2;
    c.init_min_hits = 50;
    c.init_max_hits = 65;
    c.aos_str_req = 95;
    c.old_str_req = 45;
    c.old_dex_bonus = -5;
    c.armor_base = 30;
    c.weight = 4.0;
    c.allow_male_wearer = false;
    c.material_type = "Plate";
    c.graphic = 7172;
});

item("FemaleElvenPlateChest", |c| {
    c.base_physical_resistance = 5;
    c.base_fire_resistance = 3;
    c.base_cold_resistance = 2;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 2;
    c.init_min_hits = 50;
    c.init_max_hits = 65;
    c.aos_str_req = 95;
    c.old_str_req = 95;
    c.armor_base = 30;
    c.weight = 8.0;
    c.allow_male_wearer = false;
    c.material_type = "Wood";
    c.graphic = 11117;
});

item("HeavyPlateJingasa", |c| {
    c.base_physical_resistance = 7;
    c.base_fire_resistance = 2;
    c.base_cold_resistance = 2;
    c.base_poison_resistance = 2;
    c.base_energy_resistance = 2;
    c.init_min_hits = 50;
    c.init_max_hits = 70;
    c.aos_str_req = 55;
    c.old_str_req = 55;
    c.armor_base = 4;
    c.weight = 5.0;
    c.material_type = "Plate";
    c.graphic = 10103;
});

item("LightPlateJingasa", |c| {
    c.base_physical_resistance = 7;
    c.base_fire_resistance = 2;
    c.base_cold_resistance = 2;
    c.base_poison_resistance = 2;
    c.base_energy_resistance = 2;
    c.init_min_hits = 55;
    c.init_max_hits = 60;
    c.aos_str_req = 55;
    c.old_str_req = 55;
    c.armor_base = 4;
    c.weight = 5.0;
    c.material_type = "Plate";
    c.graphic = 10113;
});

item("PlateArms", |c| {
    c.base_physical_resistance = 5;
    c.base_fire_resistance = 3;
    c.base_cold_resistance = 2;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 2;
    c.init_min_hits = 50;
    c.init_max_hits = 65;
    c.aos_str_req = 80;
    c.old_str_req = 40;
    c.old_dex_bonus = -2;
    c.armor_base = 40;
    c.weight = 5.0;
    c.material_type = "Plate";
    c.graphic = 5136;
});

item("PlateBattleKabuto", |c| {
    c.base_physical_resistance = 6;
    c.base_fire_resistance = 2;
    c.base_cold_resistance = 2;
    c.base_poison_resistance = 2;
    c.base_energy_resistance = 3;
    c.init_min_hits = 60;
    c.init_max_hits = 65;
    c.aos_str_req = 70;
    c.old_str_req = 70;
    c.armor_base = 3;
    c.weight = 6.0;
    c.material_type = "Plate";
    c.graphic = 10117;
});

item("PlateDo", |c| {
    c.base_physical_resistance = 5;
    c.base_fire_resistance = 3;
    c.base_cold_resistance = 2;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 2;
    c.init_min_hits = 60;
    c.init_max_hits = 70;
    c.aos_str_req = 85;
    c.old_str_req = 85;
    c.armor_base = 3;
    c.weight = 10.0;
    c.material_type = "Plate";
    c.graphic = 10109;
});

item("PlateGloves", |c| {
    c.base_physical_resistance = 5;
    c.base_fire_resistance = 3;
    c.base_cold_resistance = 2;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 2;
    c.init_min_hits = 50;
    c.init_max_hits = 65;
    c.aos_str_req = 70;
    c.old_str_req = 30;
    c.old_dex_bonus = -2;
    c.armor_base = 40;
    c.weight = 2.0;
    c.material_type = "Plate";
    c.graphic = 5140;
});

item("PlateGorget", |c| {
    c.base_physical_resistance = 5;
    c.base_fire_resistance = 3;
    c.base_cold_resistance = 2;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 2;
    c.init_min_hits = 50;
    c.init_max_hits = 65;
    c.aos_str_req = 45;
    c.old_str_req = 30;
    c.old_dex_bonus = -1;
    c.armor_base = 40;
    c.weight = 2.0;
    c.material_type = "Plate";
    c.graphic = 5139;
});

item("PlateHaidate", |c| {
    c.base_physical_resistance = 5;
    c.base_fire_resistance = 3;
    c.base_cold_resistance = 2;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 2;
    c.init_min_hits = 55;
    c.init_max_hits = 65;
    c.aos_str_req = 80;
    c.old_str_req = 80;
    c.armor_base = 3;
    c.weight = 7.0;
    c.material_type = "Plate";
    c.graphic = 10125;
});

item("PlateHatsuburi", |c| {
    c.base_physical_resistance = 5;
    c.base_fire_resistance = 3;
    c.base_cold_resistance = 2;
    c.base_poison_resistance = 2;
    c.base_energy_resistance = 3;
    c.init_min_hits = 55;
    c.init_max_hits = 75;
    c.aos_str_req = 65;
    c.old_str_req = 65;
    c.armor_base = 4;
    c.weight = 5.0;
    c.material_type = "Plate";
    c.graphic = 10101;
});

item("PlateHiroSode", |c| {
    c.base_physical_resistance = 5;
    c.base_fire_resistance = 3;
    c.base_cold_resistance = 2;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 2;
    c.init_min_hits = 55;
    c.init_max_hits = 75;
    c.aos_str_req = 75;
    c.old_str_req = 75;
    c.armor_base = 3;
    c.weight = 3.0;
    c.material_type = "Plate";
    c.graphic = 10112;
});

item("PlateLegs", |c| {
    c.base_physical_resistance = 5;
    c.base_fire_resistance = 3;
    c.base_cold_resistance = 2;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 2;
    c.init_min_hits = 50;
    c.init_max_hits = 65;
    c.aos_str_req = 90;
    c.old_str_req = 60;
    c.old_dex_bonus = -6;
    c.armor_base = 40;
    c.weight = 7.0;
    c.material_type = "Plate";
    c.graphic = 5137;
});

item("PlateMempo", |c| {
    c.base_physical_resistance = 5;
    c.base_fire_resistance = 3;
    c.base_cold_resistance = 2;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 2;
    c.init_min_hits = 60;
    c.init_max_hits = 70;
    c.aos_str_req = 50;
    c.old_str_req = 50;
    c.armor_base = 4;
    c.weight = 3.0;
    c.material_type = "Plate";
    c.graphic = 10105;
});

item("PlateSuneate", |c| {
    c.base_physical_resistance = 5;
    c.base_fire_resistance = 3;
    c.base_cold_resistance = 2;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 2;
    c.init_min_hits = 55;
    c.init_max_hits = 65;
    c.aos_str_req = 80;
    c.old_str_req = 80;
    c.armor_base = 3;
    c.weight = 7.0;
    c.material_type = "Plate";
    c.graphic = 10120;
});

item("SmallPlateJingasa", |c| {
    c.base_physical_resistance = 7;
    c.base_fire_resistance = 2;
    c.base_cold_resistance = 2;
    c.base_poison_resistance = 2;
    c.base_energy_resistance = 2;
    c.init_min_hits = 55;
    c.init_max_hits = 60;
    c.aos_str_req = 55;
    c.old_str_req = 55;
    c.armor_base = 4;
    c.weight = 5.0;
    c.material_type = "Plate";
    c.graphic = 10116;
});

item("StandardPlateKabuto", |c| {
    c.base_physical_resistance = 6;
    c.base_fire_resistance = 2;
    c.base_cold_resistance = 2;
    c.base_poison_resistance = 2;
    c.base_energy_resistance = 3;
    c.init_min_hits = 60;
    c.init_max_hits = 65;
    c.aos_str_req = 70;
    c.old_str_req = 70;
    c.armor_base = 3;
    c.weight = 6.0;
    c.material_type = "Plate";
    c.graphic = 10121;
});

item("WoodlandArms", |c| {
    c.base_physical_resistance = 5;
    c.base_fire_resistance = 3;
    c.base_cold_resistance = 2;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 2;
    c.init_min_hits = 50;
    c.init_max_hits = 65;
    c.aos_str_req = 80;
    c.old_str_req = 80;
    c.armor_base = 40;
    c.weight = 5.0;
    c.material_type = "Wood";
    c.required_races = "AllowElvesOnly";
    c.graphic = 11116;
});

item("WoodlandChest", |c| {
    c.base_physical_resistance = 5;
    c.base_fire_resistance = 3;
    c.base_cold_resistance = 2;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 2;
    c.init_min_hits = 50;
    c.init_max_hits = 65;
    c.aos_str_req = 95;
    c.old_str_req = 95;
    c.armor_base = 40;
    c.weight = 8.0;
    c.material_type = "Wood";
    c.required_races = "AllowElvesOnly";
    c.graphic = 11111;
});

item("WoodlandGloves", |c| {
    c.base_physical_resistance = 5;
    c.base_fire_resistance = 3;
    c.base_cold_resistance = 2;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 2;
    c.init_min_hits = 50;
    c.init_max_hits = 65;
    c.aos_str_req = 70;
    c.old_str_req = 70;
    c.armor_base = 40;
    c.weight = 2.0;
    c.material_type = "Wood";
    c.required_races = "AllowElvesOnly";
    c.graphic = 11114;
});

item("WoodlandGorget", |c| {
    c.base_physical_resistance = 5;
    c.base_fire_resistance = 3;
    c.base_cold_resistance = 2;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 2;
    c.init_min_hits = 50;
    c.init_max_hits = 65;
    c.aos_str_req = 45;
    c.old_str_req = 45;
    c.armor_base = 40;
    c.material_type = "Wood";
    c.required_races = "AllowElvesOnly";
    c.graphic = 11113;
});

item("WoodlandLegs", |c| {
    c.base_physical_resistance = 5;
    c.base_fire_resistance = 3;
    c.base_cold_resistance = 2;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 2;
    c.init_min_hits = 50;
    c.init_max_hits = 65;
    c.aos_str_req = 90;
    c.old_str_req = 90;
    c.armor_base = 40;
    c.weight = 8.0;
    c.material_type = "Wood";
    c.required_races = "AllowElvesOnly";
    c.graphic = 11115;
});

item("RangerArms", |c| {
    c.base_physical_resistance = 2;
    c.base_fire_resistance = 4;
    c.base_cold_resistance = 3;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 4;
    c.init_min_hits = 35;
    c.init_max_hits = 45;
    c.aos_str_req = 25;
    c.old_str_req = 25;
    c.armor_base = 16;
    c.label = 1041493;
    c.weight = 4.0;
    c.material_type = "Studded";
    c.default_resource = "RegularLeather";
    c.graphic = 5084;
    c.on_create(|i| {
        i.hue = 0x59C;
    });
});

item("RangerChest", |c| {
    c.base_physical_resistance = 2;
    c.base_fire_resistance = 4;
    c.base_cold_resistance = 3;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 4;
    c.init_min_hits = 35;
    c.init_max_hits = 45;
    c.aos_str_req = 35;
    c.old_str_req = 35;
    c.armor_base = 16;
    c.label = 1041497;
    c.weight = 8.0;
    c.material_type = "Studded";
    c.default_resource = "RegularLeather";
    c.graphic = 5083;
    c.on_create(|i| {
        i.hue = 0x59C;
    });
});

item("RangerGloves", |c| {
    c.base_physical_resistance = 2;
    c.base_fire_resistance = 4;
    c.base_cold_resistance = 3;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 4;
    c.init_min_hits = 35;
    c.init_max_hits = 45;
    c.aos_str_req = 25;
    c.old_str_req = 25;
    c.armor_base = 16;
    c.label = 1041494;
    c.weight = 1.0;
    c.material_type = "Studded";
    c.default_resource = "RegularLeather";
    c.graphic = 5077;
    c.on_create(|i| {
        i.hue = 0x59C;
    });
});

item("RangerGorget", |c| {
    c.base_physical_resistance = 2;
    c.base_fire_resistance = 4;
    c.base_cold_resistance = 3;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 4;
    c.init_min_hits = 35;
    c.init_max_hits = 45;
    c.aos_str_req = 25;
    c.old_str_req = 25;
    c.armor_base = 16;
    c.label = 1041495;
    c.weight = 1.0;
    c.material_type = "Studded";
    c.default_resource = "RegularLeather";
    c.graphic = 5078;
    c.on_create(|i| {
        i.hue = 0x59C;
    });
});

item("RangerLegs", |c| {
    c.base_physical_resistance = 2;
    c.base_fire_resistance = 4;
    c.base_cold_resistance = 3;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 4;
    c.init_min_hits = 35;
    c.init_max_hits = 45;
    c.aos_str_req = 30;
    c.old_str_req = 35;
    c.armor_base = 16;
    c.label = 1041496;
    c.weight = 3.0;
    c.material_type = "Studded";
    c.default_resource = "RegularLeather";
    c.graphic = 5082;
    c.on_create(|i| {
        i.hue = 0x59C;
    });
});

item("RingmailArms", |c| {
    c.base_physical_resistance = 3;
    c.base_fire_resistance = 3;
    c.base_cold_resistance = 1;
    c.base_poison_resistance = 5;
    c.base_energy_resistance = 3;
    c.init_min_hits = 40;
    c.init_max_hits = 50;
    c.aos_str_req = 40;
    c.old_str_req = 20;
    c.old_dex_bonus = -1;
    c.armor_base = 22;
    c.weight = 15.0;
    c.material_type = "Ringmail";
    c.graphic = 5102;
});

item("RingmailChest", |c| {
    c.base_physical_resistance = 3;
    c.base_fire_resistance = 3;
    c.base_cold_resistance = 1;
    c.base_poison_resistance = 5;
    c.base_energy_resistance = 3;
    c.init_min_hits = 40;
    c.init_max_hits = 50;
    c.aos_str_req = 40;
    c.old_str_req = 20;
    c.old_dex_bonus = -2;
    c.armor_base = 22;
    c.weight = 15.0;
    c.material_type = "Ringmail";
    c.graphic = 5100;
});

item("RingmailGloves", |c| {
    c.base_physical_resistance = 3;
    c.base_fire_resistance = 3;
    c.base_cold_resistance = 1;
    c.base_poison_resistance = 5;
    c.base_energy_resistance = 3;
    c.init_min_hits = 40;
    c.init_max_hits = 50;
    c.aos_str_req = 40;
    c.old_str_req = 20;
    c.old_dex_bonus = -1;
    c.armor_base = 22;
    c.weight = 2.0;
    c.material_type = "Ringmail";
    c.graphic = 5099;
});

item("RingmailLegs", |c| {
    c.base_physical_resistance = 3;
    c.base_fire_resistance = 3;
    c.base_cold_resistance = 1;
    c.base_poison_resistance = 5;
    c.base_energy_resistance = 3;
    c.init_min_hits = 40;
    c.init_max_hits = 50;
    c.aos_str_req = 40;
    c.old_str_req = 20;
    c.old_dex_bonus = -1;
    c.armor_base = 22;
    c.weight = 15.0;
    c.material_type = "Ringmail";
    c.graphic = 5104;
});

item("SpiritualityHelm", |c| {
    c.base_physical_resistance = 13;
    c.base_fire_resistance = 13;
    c.base_cold_resistance = 12;
    c.base_poison_resistance = 14;
    c.base_energy_resistance = 13;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.aos_str_req = 25;
    c.label = 1075237;
    c.material_type = "Plate";
    c.graphic = 11024;
    c.on_create(|i| {
        i.loot_type = "Blessed";
        i.weight = 6.0;
        i.hue = 0x226;
        i.armor_attributes.self_repair = 5;
    });
});

item("ValorGauntlets", |c| {
    c.base_physical_resistance = 11;
    c.base_fire_resistance = 11;
    c.base_cold_resistance = 13;
    c.base_poison_resistance = 14;
    c.base_energy_resistance = 10;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.aos_str_req = 50;
    c.label = 1075238;
    c.material_type = "Plate";
    c.graphic = 11020;
    c.on_create(|i| {
        i.loot_type = "Blessed";
        i.weight = 4.0;
        i.hue = 0x226;
        i.armor_attributes.self_repair = 5;
    });
});

item("GargishStoneArmsType1", |c| {
    c.base_physical_resistance = 6;
    c.base_fire_resistance = 6;
    c.base_cold_resistance = 4;
    c.base_poison_resistance = 8;
    c.base_energy_resistance = 6;
    c.init_min_hits = 40;
    c.init_max_hits = 50;
    c.aos_str_req = 40;
    c.old_str_req = 40;
    c.armor_base = 40;
    c.weight = 10.0;
    c.required_races = "AllowGargoylesOnly";
    c.material_type = "Stone";
    c.graphic = 644;
});

item("GargishStoneArmsType2", |c| {
    c.base_physical_resistance = 6;
    c.base_fire_resistance = 6;
    c.base_cold_resistance = 4;
    c.base_poison_resistance = 8;
    c.base_energy_resistance = 6;
    c.init_min_hits = 40;
    c.init_max_hits = 50;
    c.aos_str_req = 40;
    c.old_str_req = 40;
    c.armor_base = 40;
    c.weight = 10.0;
    c.required_races = "AllowGargoylesOnly";
    c.material_type = "Stone";
    c.graphic = 643;
});

item("GargishStoneChestType1", |c| {
    c.base_physical_resistance = 6;
    c.base_fire_resistance = 6;
    c.base_cold_resistance = 4;
    c.base_poison_resistance = 8;
    c.base_energy_resistance = 6;
    c.init_min_hits = 40;
    c.init_max_hits = 50;
    c.aos_str_req = 40;
    c.old_str_req = 40;
    c.armor_base = 40;
    c.weight = 15.0;
    c.required_races = "AllowGargoylesOnly";
    c.material_type = "Stone";
    c.graphic = 646;
});

item("GargishStoneChestType2", |c| {
    c.base_physical_resistance = 6;
    c.base_fire_resistance = 6;
    c.base_cold_resistance = 4;
    c.base_poison_resistance = 8;
    c.base_energy_resistance = 6;
    c.init_min_hits = 40;
    c.init_max_hits = 50;
    c.aos_str_req = 40;
    c.old_str_req = 40;
    c.armor_base = 40;
    c.weight = 15.0;
    c.required_races = "AllowGargoylesOnly";
    c.material_type = "Stone";
    c.graphic = 645;
});

item("GargishStoneKiltType1", |c| {
    c.base_physical_resistance = 6;
    c.base_fire_resistance = 6;
    c.base_cold_resistance = 4;
    c.base_poison_resistance = 8;
    c.base_energy_resistance = 6;
    c.init_min_hits = 40;
    c.init_max_hits = 50;
    c.aos_str_req = 40;
    c.old_str_req = 40;
    c.armor_base = 40;
    c.weight = 10.0;
    c.required_races = "AllowGargoylesOnly";
    c.material_type = "Stone";
    c.graphic = 648;
});

item("GargishStoneKiltType2", |c| {
    c.base_physical_resistance = 6;
    c.base_fire_resistance = 6;
    c.base_cold_resistance = 4;
    c.base_poison_resistance = 8;
    c.base_energy_resistance = 6;
    c.init_min_hits = 40;
    c.init_max_hits = 50;
    c.aos_str_req = 40;
    c.old_str_req = 40;
    c.armor_base = 40;
    c.weight = 10.0;
    c.required_races = "AllowGargoylesOnly";
    c.material_type = "Stone";
    c.graphic = 647;
});

item("GargishStoneLegsType1", |c| {
    c.base_physical_resistance = 6;
    c.base_fire_resistance = 6;
    c.base_cold_resistance = 4;
    c.base_poison_resistance = 8;
    c.base_energy_resistance = 6;
    c.init_min_hits = 40;
    c.init_max_hits = 50;
    c.aos_str_req = 40;
    c.old_str_req = 40;
    c.armor_base = 40;
    c.weight = 15.0;
    c.required_races = "AllowGargoylesOnly";
    c.material_type = "Stone";
    c.graphic = 650;
});

item("GargishStoneLegsType2", |c| {
    c.base_physical_resistance = 6;
    c.base_fire_resistance = 6;
    c.base_cold_resistance = 4;
    c.base_poison_resistance = 8;
    c.base_energy_resistance = 6;
    c.init_min_hits = 40;
    c.init_max_hits = 50;
    c.aos_str_req = 40;
    c.old_str_req = 40;
    c.armor_base = 40;
    c.weight = 15.0;
    c.required_races = "AllowGargoylesOnly";
    c.material_type = "Stone";
    c.graphic = 649;
});

item("FemaleStuddedChest", |c| {
    c.base_physical_resistance = 2;
    c.base_fire_resistance = 4;
    c.base_cold_resistance = 3;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 4;
    c.init_min_hits = 35;
    c.init_max_hits = 45;
    c.aos_str_req = 35;
    c.old_str_req = 35;
    c.armor_base = 16;
    c.weight = 6.0;
    c.material_type = "Studded";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "Half";
    c.allow_male_wearer = false;
    c.graphic = 7170;
});

item("GargishStuddedArmsType1", |c| {
    c.base_physical_resistance = 6;
    c.base_fire_resistance = 6;
    c.base_cold_resistance = 4;
    c.base_poison_resistance = 8;
    c.base_energy_resistance = 6;
    c.init_min_hits = 40;
    c.init_max_hits = 50;
    c.aos_str_req = 40;
    c.old_str_req = 40;
    c.weight = 10.0;
    c.required_races = "AllowGargoylesOnly";
    c.material_type = "Studded";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "Half";
    c.graphic = 644;
});

item("GargishStuddedArmsType2", |c| {
    c.base_physical_resistance = 6;
    c.base_fire_resistance = 6;
    c.base_cold_resistance = 4;
    c.base_poison_resistance = 8;
    c.base_energy_resistance = 6;
    c.init_min_hits = 40;
    c.init_max_hits = 50;
    c.aos_str_req = 40;
    c.old_str_req = 40;
    c.armor_base = 16;
    c.weight = 10.0;
    c.required_races = "AllowGargoylesOnly";
    c.material_type = "Studded";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "Half";
    c.graphic = 643;
});

item("GargishStuddedChestType1", |c| {
    c.base_physical_resistance = 6;
    c.base_fire_resistance = 6;
    c.base_cold_resistance = 4;
    c.base_poison_resistance = 8;
    c.base_energy_resistance = 6;
    c.init_min_hits = 40;
    c.init_max_hits = 50;
    c.aos_str_req = 40;
    c.old_str_req = 40;
    c.armor_base = 16;
    c.weight = 15.0;
    c.required_races = "AllowGargoylesOnly";
    c.material_type = "Studded";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "Half";
    c.graphic = 646;
});

item("GargishStuddedChestType2", |c| {
    c.base_physical_resistance = 6;
    c.base_fire_resistance = 6;
    c.base_cold_resistance = 4;
    c.base_poison_resistance = 8;
    c.base_energy_resistance = 6;
    c.init_min_hits = 40;
    c.init_max_hits = 50;
    c.aos_str_req = 40;
    c.old_str_req = 40;
    c.armor_base = 16;
    c.weight = 15.0;
    c.required_races = "AllowGargoylesOnly";
    c.material_type = "Studded";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "Half";
    c.graphic = 645;
});

item("GargishStuddedKiltType1", |c| {
    c.base_physical_resistance = 6;
    c.base_fire_resistance = 6;
    c.base_cold_resistance = 4;
    c.base_poison_resistance = 8;
    c.base_energy_resistance = 6;
    c.init_min_hits = 40;
    c.init_max_hits = 50;
    c.aos_str_req = 40;
    c.old_str_req = 40;
    c.armor_base = 16;
    c.weight = 10.0;
    c.required_races = "AllowGargoylesOnly";
    c.material_type = "Studded";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "Half";
    c.graphic = 648;
});

item("GargishStuddedKiltType2", |c| {
    c.base_physical_resistance = 6;
    c.base_fire_resistance = 6;
    c.base_cold_resistance = 4;
    c.base_poison_resistance = 8;
    c.base_energy_resistance = 6;
    c.init_min_hits = 40;
    c.init_max_hits = 50;
    c.aos_str_req = 40;
    c.old_str_req = 40;
    c.armor_base = 16;
    c.weight = 10.0;
    c.required_races = "AllowGargoylesOnly";
    c.material_type = "Studded";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "Half";
    c.graphic = 647;
});

item("GargishStuddedLegsType1", |c| {
    c.base_physical_resistance = 6;
    c.base_fire_resistance = 6;
    c.base_cold_resistance = 4;
    c.base_poison_resistance = 8;
    c.base_energy_resistance = 6;
    c.init_min_hits = 40;
    c.init_max_hits = 50;
    c.aos_str_req = 40;
    c.old_str_req = 40;
    c.armor_base = 16;
    c.weight = 15.0;
    c.required_races = "AllowGargoylesOnly";
    c.material_type = "Studded";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "Half";
    c.graphic = 650;
});

item("GargishStuddedLegsType2", |c| {
    c.base_physical_resistance = 6;
    c.base_fire_resistance = 6;
    c.base_cold_resistance = 4;
    c.base_poison_resistance = 8;
    c.base_energy_resistance = 6;
    c.init_min_hits = 40;
    c.init_max_hits = 50;
    c.aos_str_req = 40;
    c.old_str_req = 40;
    c.armor_base = 16;
    c.weight = 15.0;
    c.required_races = "AllowGargoylesOnly";
    c.material_type = "Studded";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "Half";
    c.graphic = 649;
});

item("HideChest", |c| {
    c.base_physical_resistance = 3;
    c.base_fire_resistance = 3;
    c.base_cold_resistance = 4;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 2;
    c.init_min_hits = 35;
    c.init_max_hits = 45;
    c.aos_str_req = 25;
    c.old_str_req = 25;
    c.armor_base = 15;
    c.weight = 6.0;
    c.required_races = "AllowElvesOnly";
    c.material_type = "Studded";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "Half";
    c.graphic = 11124;
});

item("HideFemaleChest", |c| {
    c.base_physical_resistance = 3;
    c.base_fire_resistance = 3;
    c.base_cold_resistance = 4;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 2;
    c.init_min_hits = 35;
    c.init_max_hits = 45;
    c.aos_str_req = 35;
    c.old_str_req = 35;
    c.armor_base = 15;
    c.weight = 6.0;
    c.required_races = "AllowElvesOnly";
    c.material_type = "Studded";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "Half";
    c.allow_male_wearer = false;
    c.graphic = 11129;
});

item("HideGloves", |c| {
    c.base_physical_resistance = 3;
    c.base_fire_resistance = 3;
    c.base_cold_resistance = 4;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 2;
    c.init_min_hits = 35;
    c.init_max_hits = 45;
    c.aos_str_req = 15;
    c.old_str_req = 15;
    c.armor_base = 15;
    c.weight = 2.0;
    c.required_races = "AllowElvesOnly";
    c.material_type = "Studded";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "Half";
    c.graphic = 11125;
});

item("HideGorget", |c| {
    c.base_physical_resistance = 3;
    c.base_fire_resistance = 3;
    c.base_cold_resistance = 4;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 2;
    c.init_min_hits = 35;
    c.init_max_hits = 45;
    c.aos_str_req = 15;
    c.old_str_req = 15;
    c.armor_base = 15;
    c.weight = 3.0;
    c.required_races = "AllowElvesOnly";
    c.material_type = "Studded";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "Half";
    c.graphic = 11126;
});

item("HidePants", |c| {
    c.base_physical_resistance = 3;
    c.base_fire_resistance = 3;
    c.base_cold_resistance = 4;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 2;
    c.init_min_hits = 35;
    c.init_max_hits = 45;
    c.aos_str_req = 25;
    c.old_str_req = 25;
    c.armor_base = 15;
    c.weight = 5.0;
    c.required_races = "AllowElvesOnly";
    c.material_type = "Studded";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "Half";
    c.graphic = 11128;
});

item("HidePauldrons", |c| {
    c.base_physical_resistance = 3;
    c.base_fire_resistance = 3;
    c.base_cold_resistance = 4;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 2;
    c.init_min_hits = 35;
    c.init_max_hits = 45;
    c.aos_str_req = 20;
    c.old_str_req = 20;
    c.armor_base = 15;
    c.weight = 4.0;
    c.required_races = "AllowElvesOnly";
    c.material_type = "Studded";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "Half";
    c.graphic = 11127;
});

item("StuddedArms", |c| {
    c.base_physical_resistance = 2;
    c.base_fire_resistance = 4;
    c.base_cold_resistance = 3;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 4;
    c.init_min_hits = 35;
    c.init_max_hits = 45;
    c.aos_str_req = 25;
    c.old_str_req = 25;
    c.armor_base = 16;
    c.weight = 4.0;
    c.material_type = "Studded";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "Half";
    c.graphic = 5084;
});

item("StuddedBustierArms", |c| {
    c.base_physical_resistance = 2;
    c.base_fire_resistance = 4;
    c.base_cold_resistance = 3;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 4;
    c.init_min_hits = 35;
    c.init_max_hits = 45;
    c.aos_str_req = 35;
    c.old_str_req = 35;
    c.armor_base = 16;
    c.weight = 1.0;
    c.material_type = "Studded";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "Half";
    c.allow_male_wearer = false;
    c.graphic = 7180;
});

item("StuddedChest", |c| {
    c.base_physical_resistance = 2;
    c.base_fire_resistance = 4;
    c.base_cold_resistance = 3;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 4;
    c.init_min_hits = 35;
    c.init_max_hits = 45;
    c.aos_str_req = 35;
    c.old_str_req = 35;
    c.armor_base = 16;
    c.weight = 8.0;
    c.material_type = "Studded";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "Half";
    c.graphic = 5083;
});

item("StuddedDo", |c| {
    c.base_physical_resistance = 2;
    c.base_fire_resistance = 4;
    c.base_cold_resistance = 3;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 4;
    c.init_min_hits = 40;
    c.init_max_hits = 50;
    c.aos_str_req = 55;
    c.old_str_req = 55;
    c.armor_base = 3;
    c.weight = 8.0;
    c.material_type = "Studded";
    c.default_resource = "RegularLeather";
    c.graphic = 10183;
});

item("StuddedGloves", |c| {
    c.base_physical_resistance = 2;
    c.base_fire_resistance = 4;
    c.base_cold_resistance = 3;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 4;
    c.init_min_hits = 35;
    c.init_max_hits = 45;
    c.aos_str_req = 25;
    c.old_str_req = 25;
    c.armor_base = 16;
    c.weight = 1.0;
    c.material_type = "Studded";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "Half";
    c.graphic = 5077;
});

item("StuddedGorget", |c| {
    c.base_physical_resistance = 2;
    c.base_fire_resistance = 4;
    c.base_cold_resistance = 3;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 4;
    c.init_min_hits = 35;
    c.init_max_hits = 45;
    c.aos_str_req = 25;
    c.old_str_req = 25;
    c.armor_base = 16;
    c.weight = 1.0;
    c.material_type = "Studded";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "Half";
    c.graphic = 5078;
});

item("StuddedHaidate", |c| {
    c.base_physical_resistance = 2;
    c.base_fire_resistance = 4;
    c.base_cold_resistance = 3;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 4;
    c.init_min_hits = 35;
    c.init_max_hits = 45;
    c.aos_str_req = 30;
    c.old_str_req = 30;
    c.armor_base = 3;
    c.weight = 5.0;
    c.material_type = "Studded";
    c.default_resource = "RegularLeather";
    c.graphic = 10123;
});

item("StuddedHiroSode", |c| {
    c.base_physical_resistance = 2;
    c.base_fire_resistance = 4;
    c.base_cold_resistance = 3;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 4;
    c.init_min_hits = 45;
    c.init_max_hits = 55;
    c.aos_str_req = 30;
    c.old_str_req = 30;
    c.armor_base = 3;
    c.weight = 1.0;
    c.material_type = "Studded";
    c.default_resource = "RegularLeather";
    c.graphic = 10111;
});

item("StuddedLegs", |c| {
    c.base_physical_resistance = 2;
    c.base_fire_resistance = 4;
    c.base_cold_resistance = 3;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 4;
    c.init_min_hits = 35;
    c.init_max_hits = 45;
    c.aos_str_req = 30;
    c.old_str_req = 35;
    c.armor_base = 16;
    c.weight = 5.0;
    c.material_type = "Studded";
    c.default_resource = "RegularLeather";
    c.def_med_allowance = "Half";
    c.graphic = 5082;
});

item("StuddedMempo", |c| {
    c.base_physical_resistance = 2;
    c.base_fire_resistance = 4;
    c.base_cold_resistance = 3;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 3;
    c.init_min_hits = 30;
    c.init_max_hits = 40;
    c.aos_str_req = 30;
    c.old_str_req = 30;
    c.armor_base = 3;
    c.weight = 2.0;
    c.material_type = "Studded";
    c.default_resource = "RegularLeather";
    c.graphic = 10141;
});

item("StuddedSuneate", |c| {
    c.base_physical_resistance = 2;
    c.base_fire_resistance = 4;
    c.base_cold_resistance = 3;
    c.base_poison_resistance = 3;
    c.base_energy_resistance = 4;
    c.init_min_hits = 35;
    c.init_max_hits = 50;
    c.aos_str_req = 30;
    c.old_str_req = 30;
    c.armor_base = 3;
    c.weight = 5.0;
    c.material_type = "Studded";
    c.default_resource = "RegularLeather";
    c.graphic = 10194;
});

item("Aegis", |c| {
    c.base_physical_resistance = 15;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1061602;
    c.rarity = 11;
    c.on_create(|i| {
        i.hue = 0x47E;
        i.armor_attributes.self_repair = 5;
        i.attributes.reflect_physical = 15;
        i.attributes.defend_chance = 15;
        i.attributes.lower_mana_cost = 8;
    });
});

item("ArcaneShield", |c| {
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.label = 1061101;
    c.rarity = 11;
    c.on_create(|i| {
        i.item_id = 0x1B78;
        i.hue = 0x556;
        i.attributes.night_sight = 1;
        i.attributes.spell_channeling = 1;
        i.attributes.defend_chance = 15;
        i.attributes.cast_speed = 1;
    });
});

item("BronzeShield", |c| {
    c.base_physical_resistance = 0;
    c.base_fire_resistance = 0;
    c.base_cold_resistance = 1;
    c.base_poison_resistance = 0;
    c.base_energy_resistance = 0;
    c.init_min_hits = 25;
    c.init_max_hits = 30;
    c.aos_str_req = 35;
    c.armor_base = 10;
    c.weight = 6.0;
    c.graphic = 7026;
});

item("Server.Items.Buckler", |c| {
    c.base_physical_resistance = 0;
    c.base_fire_resistance = 0;
    c.base_cold_resistance = 0;
    c.base_poison_resistance = 1;
    c.base_energy_resistance = 0;
    c.init_min_hits = 40;
    c.init_max_hits = 50;
    c.aos_str_req = 20;
    c.armor_base = 7;
    c.weight = 5.0;
    c.graphic = 7027;
});

item("GargishWoodenShield", |c| {
    c.base_physical_resistance = 0;
    c.base_fire_resistance = 0;
    c.base_cold_resistance = 0;
    c.base_poison_resistance = 0;
    c.base_energy_resistance = 1;
    c.init_min_hits = 20;
    c.init_max_hits = 25;
    c.aos_str_req = 20;
    c.weight = 5.0;
    c.required_races = "AllowGargoylesOnly";
    c.material_type = "Wood";
    c.graphic = 16896;
});

item("HeaterShield", |c| {
    c.base_physical_resistance = 0;
    c.base_fire_resistance = 1;
    c.base_cold_resistance = 0;
    c.base_poison_resistance = 0;
    c.base_energy_resistance = 0;
    c.init_min_hits = 50;
    c.init_max_hits = 65;
    c.aos_str_req = 90;
    c.armor_base = 23;
    c.weight = 8.0;
    c.graphic = 7030;
});

item("MetalKiteShield", |c| {
    c.base_physical_resistance = 0;
    c.base_fire_resistance = 0;
    c.base_cold_resistance = 0;
    c.base_poison_resistance = 0;
    c.base_energy_resistance = 1;
    c.init_min_hits = 45;
    c.init_max_hits = 60;
    c.aos_str_req = 45;
    c.armor_base = 16;
    c.weight = 7.0;
    c.graphic = 7028;
});

item("MetalShield", |c| {
    c.base_physical_resistance = 0;
    c.base_fire_resistance = 1;
    c.base_cold_resistance = 0;
    c.base_poison_resistance = 0;
    c.base_energy_resistance = 0;
    c.init_min_hits = 50;
    c.init_max_hits = 65;
    c.aos_str_req = 45;
    c.armor_base = 11;
    c.weight = 6.0;
    c.graphic = 7035;
});

item("WoodenKiteShield", |c| {
    c.base_physical_resistance = 0;
    c.base_fire_resistance = 0;
    c.base_cold_resistance = 0;
    c.base_poison_resistance = 0;
    c.base_energy_resistance = 1;
    c.init_min_hits = 50;
    c.init_max_hits = 65;
    c.aos_str_req = 20;
    c.armor_base = 12;
    c.weight = 5.0;
    c.material_type = "Wood";
    c.graphic = 7033;
});

item("WoodenShield", |c| {
    c.base_physical_resistance = 0;
    c.base_fire_resistance = 0;
    c.base_cold_resistance = 0;
    c.base_poison_resistance = 0;
    c.base_energy_resistance = 1;
    c.init_min_hits = 20;
    c.init_max_hits = 25;
    c.aos_str_req = 20;
    c.armor_base = 8;
    c.weight = 5.0;
    c.material_type = "Wood";
    c.graphic = 7034;
});

item("ElvenGlasses", |c| {
    c.label = 1032216;
});
