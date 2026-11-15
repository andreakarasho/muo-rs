item("BloodBlade", |c| {
    c.aos_str_req = 10;
    c.aos_min_damage = 10;
    c.aos_max_damage = 12;
    c.ml_speed = 2.0;
    c.def_hit_sound = 572;
    c.def_miss_sound = 568;
    c.init_min_hits = 31;
    c.init_max_hits = 90;
    c.primary_ability = "BleedAttack";
    c.secondary_ability = "ParalyzingBlow";
    c.def_skill = "Fencing";
    c.def_type = "Piercing";
    c.def_animation = "Pierce1H";
    c.required_races = "AllowGargoylesOnly";
    c.graphic = 2302;
});

item("BoneHarvester", |c| {
    c.aos_str_req = 25;
    c.aos_min_damage = 13;
    c.aos_max_damage = 15;
    c.aos_speed = 36;
    c.ml_speed = 3.0;
    c.old_str_req = 25;
    c.old_min_damage = 13;
    c.old_max_damage = 15;
    c.old_speed = 36;
    c.def_hit_sound = 571;
    c.def_miss_sound = 570;
    c.init_min_hits = 31;
    c.init_max_hits = 70;
    c.weight = 3.0;
    c.primary_ability = "ParalyzingBlow";
    c.secondary_ability = "MortalStrike";
    c.graphic = 9915;
});

item("BoneMachete", |c| {
    c.init_min_hits = 5;
    c.init_max_hits = 5;
    c.secondary_ability = ();
    c.primary_ability = ();
    c.physical_resistance = 1;
    c.fire_resistance = 1;
    c.cold_resistance = 1;
    c.poison_resistance = 1;
    c.energy_resistance = 1;
    c.on_create(|i| {
        i.item_id = 0x20E;
    });
});

item("Broadsword", |c| {
    c.aos_str_req = 30;
    c.aos_min_damage = 14;
    c.aos_max_damage = 15;
    c.aos_speed = 33;
    c.ml_speed = 3.25;
    c.old_str_req = 25;
    c.old_min_damage = 5;
    c.old_max_damage = 29;
    c.old_speed = 45;
    c.def_hit_sound = 567;
    c.def_miss_sound = 570;
    c.init_min_hits = 31;
    c.init_max_hits = 100;
    c.weight = 6.0;
    c.primary_ability = "CrushingBlow";
    c.secondary_ability = "ArmorIgnore";
    c.graphic = 3934;
});

item("CrescentBlade", |c| {
    c.aos_str_req = 55;
    c.aos_min_damage = 11;
    c.aos_max_damage = 14;
    c.aos_speed = 47;
    c.ml_speed = 2.5;
    c.old_str_req = 55;
    c.old_min_damage = 11;
    c.old_max_damage = 14;
    c.old_speed = 47;
    c.def_hit_sound = 571;
    c.def_miss_sound = 570;
    c.init_min_hits = 51;
    c.init_max_hits = 80;
    c.weight = 1.0;
    c.primary_ability = "DoubleStrike";
    c.secondary_ability = "MortalStrike";
    c.graphic = 9921;
});

item("Cutlass", |c| {
    c.aos_str_req = 25;
    c.aos_min_damage = 11;
    c.aos_max_damage = 13;
    c.aos_speed = 44;
    c.ml_speed = 2.5;
    c.old_str_req = 10;
    c.old_min_damage = 6;
    c.old_max_damage = 28;
    c.old_speed = 45;
    c.def_hit_sound = 571;
    c.def_miss_sound = 570;
    c.init_min_hits = 31;
    c.init_max_hits = 70;
    c.weight = 8.0;
    c.primary_ability = "BleedAttack";
    c.secondary_ability = "ShadowStrike";
    c.graphic = 5185;
});

item("DreadSword", |c| {
    c.aos_str_req = 35;
    c.aos_min_damage = 14;
    c.aos_max_damage = 18;
    c.ml_speed = 3.5;
    c.def_hit_sound = 567;
    c.def_miss_sound = 570;
    c.init_min_hits = 31;
    c.init_max_hits = 110;
    c.primary_ability = "CrushingBlow";
    c.secondary_ability = "ConcussionBlow";
    c.required_races = "AllowGargoylesOnly";
    c.graphic = 2315;
});

item("GargishTalwar", |c| {
    c.aos_str_req = 40;
    c.aos_min_damage = 16;
    c.aos_max_damage = 19;
    c.ml_speed = 3.5;
    c.def_hit_sound = 567;
    c.def_miss_sound = 568;
    c.init_min_hits = 31;
    c.init_max_hits = 80;
    c.primary_ability = "WhirlwindAttack";
    c.secondary_ability = "Dismount";
    c.required_races = "AllowGargoylesOnly";
    c.graphic = 2312;
});

item("GlassSword", |c| {
    c.aos_str_req = 20;
    c.aos_min_damage = 11;
    c.aos_max_damage = 15;
    c.ml_speed = 2.75;
    c.def_hit_sound = 571;
    c.def_miss_sound = 570;
    c.init_min_hits = 31;
    c.init_max_hits = 90;
    c.weight = 6.0;
    c.primary_ability = "BleedAttack";
    c.secondary_ability = "MortalStrike";
    c.required_races = "AllowGargoylesOnly";
    c.graphic = 2316;
});

item("Kryss", |c| {
    c.aos_str_req = 10;
    c.aos_min_damage = 10;
    c.aos_max_damage = 12;
    c.aos_speed = 53;
    c.ml_speed = 2.0;
    c.old_str_req = 10;
    c.old_min_damage = 3;
    c.old_max_damage = 28;
    c.old_speed = 53;
    c.def_hit_sound = 572;
    c.def_miss_sound = 568;
    c.init_min_hits = 31;
    c.init_max_hits = 90;
    c.weight = 2.0;
    c.primary_ability = "ArmorIgnore";
    c.secondary_ability = "InfectiousStrike";
    c.def_skill = "Fencing";
    c.def_type = "Piercing";
    c.def_animation = "Pierce1H";
    c.graphic = 5121;
});

item("Lance", |c| {
    c.aos_str_req = 95;
    c.aos_min_damage = 17;
    c.aos_max_damage = 18;
    c.aos_speed = 24;
    c.ml_speed = 4.5;
    c.old_str_req = 95;
    c.old_min_damage = 17;
    c.old_max_damage = 18;
    c.old_speed = 24;
    c.def_hit_sound = 572;
    c.def_miss_sound = 568;
    c.init_min_hits = 31;
    c.init_max_hits = 110;
    c.weight = 12.0;
    c.primary_ability = "Dismount";
    c.secondary_ability = "ConcussionBlow";
    c.def_skill = "Fencing";
    c.def_type = "Piercing";
    c.def_animation = "Pierce1H";
    c.graphic = 9920;
});

item("Scimitar", |c| {
    c.aos_str_req = 25;
    c.aos_min_damage = 13;
    c.aos_max_damage = 15;
    c.aos_speed = 37;
    c.ml_speed = 3.0;
    c.old_str_req = 10;
    c.old_min_damage = 4;
    c.old_max_damage = 30;
    c.old_speed = 43;
    c.def_hit_sound = 571;
    c.def_miss_sound = 570;
    c.init_min_hits = 31;
    c.init_max_hits = 90;
    c.weight = 5.0;
    c.primary_ability = "DoubleStrike";
    c.secondary_ability = "ParalyzingBlow";
    c.graphic = 5046;
});

item("ThinLongsword", |c| {
    c.aos_str_req = 35;
    c.aos_min_damage = 15;
    c.aos_max_damage = 16;
    c.aos_speed = 30;
    c.ml_speed = 3.5;
    c.old_str_req = 25;
    c.old_min_damage = 5;
    c.old_max_damage = 33;
    c.old_speed = 35;
    c.def_hit_sound = 567;
    c.def_miss_sound = 570;
    c.init_min_hits = 31;
    c.init_max_hits = 110;
    c.weight = 1.0;
    c.graphic = 5048;
});

item("VikingSword", |c| {
    c.aos_str_req = 40;
    c.aos_min_damage = 15;
    c.aos_max_damage = 17;
    c.aos_speed = 28;
    c.ml_speed = 3.75;
    c.old_str_req = 40;
    c.old_min_damage = 6;
    c.old_max_damage = 34;
    c.old_speed = 30;
    c.def_hit_sound = 567;
    c.def_miss_sound = 570;
    c.init_min_hits = 31;
    c.init_max_hits = 100;
    c.weight = 6.0;
    c.primary_ability = "CrushingBlow";
    c.secondary_ability = "ParalyzingBlow";
    c.graphic = 5049;
});

item("AdventurersMachete", |c| {
    c.label = 1073533;
    c.on_create(|i| {
        i.attributes.luck = 20;
    });
});

item("ChargedAssassinSpike", |c| {
    c.label = 1073518;
    c.on_create(|i| {
        i.weapon_attributes.hit_lightning = 10;
    });
});

item("CorruptedRuneBlade", |c| {
    c.label = 1073540;
    c.on_create(|i| {
        i.weapon_attributes.resist_physical_bonus = -5;
        i.weapon_attributes.resist_poison_bonus = 12;
    });
});

item("DarkglowScimitar", |c| {
    c.label = 1073542;
    c.on_create(|i| {
        i.weapon_attributes.hit_dispel = 10;
    });
});

item("DiseasedMachete", |c| {
    c.label = 1073536;
    c.on_create(|i| {
        i.weapon_attributes.hit_poison_area = 25;
    });
});

item("FierySpellblade", |c| {
    c.label = 1073515;
    c.on_create(|i| {
        i.weapon_attributes.resist_fire_bonus = 5;
    });
});

item("IcyScimitar", |c| {
    c.label = 1073543;
    c.on_create(|i| {
        i.weapon_attributes.hit_harm = 15;
    });
});

item("IcySpellblade", |c| {
    c.label = 1073514;
    c.on_create(|i| {
        i.weapon_attributes.resist_cold_bonus = 5;
    });
});

item("KnightsWarCleaver", |c| {
    c.label = 1073525;
    c.on_create(|i| {
        i.attributes.regen_hits = 3;
    });
});

item("LeafbladeOfEase", |c| {
    c.label = 1073524;
    c.on_create(|i| {
        i.weapon_attributes.use_best_skill = 1;
    });
});

item("Luckblade", |c| {
    c.label = 1073522;
    c.on_create(|i| {
        i.attributes.luck = 20;
    });
});

item("MacheteOfDefense", |c| {
    c.label = 1073535;
    c.on_create(|i| {
        i.attributes.defend_chance = 5;
    });
});

item("MagekillerAssassinSpike", |c| {
    c.label = 1073519;
    c.on_create(|i| {
        i.weapon_attributes.hit_leech_mana = 16;
    });
});

item("MagekillerLeafblade", |c| {
    c.label = 1073523;
    c.on_create(|i| {
        i.weapon_attributes.hit_leech_mana = 16;
    });
});

item("MagesRuneBlade", |c| {
    c.label = 1073538;
    c.on_create(|i| {
        i.attributes.cast_speed = 1;
    });
});

item("OrcishMachete", |c| {
    c.label = 1073534;
    c.on_create(|i| {
        i.attributes.bonus_int = -5;
        i.attributes.weapon_damage = 10;
    });
});

item("RuneBladeOfKnowledge", |c| {
    c.label = 1073539;
    c.on_create(|i| {
        i.attributes.spell_damage = 5;
    });
});

item("Runesabre", |c| {
    c.label = 1073537;
    c.on_create(|i| {
        i.skill_bonuses = [["MagicResist", 5.0]];
        i.weapon_attributes.mage_weapon = -29;
    });
});

item("SerratedWarCleaver", |c| {
    c.label = 1073527;
    c.on_create(|i| {
        i.attributes.weapon_damage = 7;
    });
});

item("SpellbladeOfDefense", |c| {
    c.label = 1073516;
    c.on_create(|i| {
        i.attributes.defend_chance = 5;
    });
});

item("TrueAssassinSpike", |c| {
    c.label = 1073517;
    c.on_create(|i| {
        i.attributes.attack_chance = 4;
        i.attributes.weapon_damage = 4;
    });
});

item("TrueLeafblade", |c| {
    c.label = 1073521;
    c.on_create(|i| {
        i.weapon_attributes.resist_poison_bonus = 5;
    });
});

item("TrueRadiantScimitar", |c| {
    c.label = 1073541;
    c.on_create(|i| {
        i.attributes.night_sight = 1;
    });
});

item("TrueSpellblade", |c| {
    c.label = 1073513;
    c.on_create(|i| {
        i.attributes.spell_channeling = 1;
        i.attributes.cast_speed = -1;
    });
});

item("TrueWarCleaver", |c| {
    c.label = 1073528;
    c.on_create(|i| {
        i.attributes.weapon_damage = 4;
        i.attributes.regen_hits = 2;
    });
});

item("TwinklingScimitar", |c| {
    c.label = 1073544;
    c.on_create(|i| {
        i.attributes.defend_chance = 6;
    });
});

item("WoundingAssassinSpike", |c| {
    c.label = 1073520;
    c.on_create(|i| {
        i.weapon_attributes.hit_harm = 15;
    });
});
