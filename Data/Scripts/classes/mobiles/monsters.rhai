mobile("AbysmalHorror", |c| {
    c.ai = "Mage";
    c.corpse_name = "an abysmal horror corpse";
    c.name = "an abysmal horror";
    c.poison_immune = 4;
    c.treasure_map_level = 1;
    c.loot = [["UltraRich", 2]];
    c.ignore_young_protection = #{ atLeast: "ML", then: true, "else": false };
    c.bard_immune = #{ atLeast: "SE", then: false, "else": true };
    c.unprovokable = #{ atLeast: "SE", then: true, "else": false };
    c.area_peace_immune = #{ atLeast: "SE", then: true, "else": false };
    c.weapon_ability = #{ coin: ["MortalStrike", "WhirlwindAttack"] };
    c.on_create(|m| {
        m.body = 312;
        m.sound = 0x451;
        m.str = 401..=420;
        m.dex = 81..=90;
        m.int = 401..=420;
        m.hits = 6000;
        m.damage = 13..=17;
        m.damage_type("Physical", 50);
        m.damage_type("Poison", 50);
        m.resist("Physical", 30, 35);
        m.resist("Fire", 100);
        m.resist("Cold", 50, 55);
        m.resist("Poison", 60, 65);
        m.resist("Energy", 77, 80);
        m.skill("EvalInt", 200.0);
        m.skill("Magery", 112.6, 117.5);
        m.skill("Meditation", 200.0);
        m.skill("MagicResist", 117.6, 120.0);
        m.skill("Tactics", 100.0);
        m.skill("Wrestling", 84.1, 88.0);
        m.fame = 26000;
        m.karma = -26000;
        m.armor = 54;
    });
});

mobile("BoneDemon", |c| {
    c.ai = "Mage";
    c.corpse_name = "a bone demon corpse";
    c.name = "a bone demon";
    c.poison_immune = 4;
    c.treasure_map_level = 1;
    c.loot = [["FilthyRich", 8]];
    c.bard_immune = #{ atLeast: "SE", then: false, "else": true };
    c.unprovokable = #{ atLeast: "SE", then: true, "else": false };
    c.area_peace_immune = #{ atLeast: "SE", then: true, "else": false };
    c.on_create(|m| {
        m.body = 308;
        m.sound = 0x48D;
        m.str = 1000;
        m.dex = 151..=175;
        m.int = 171..=220;
        m.hits = 3600;
        m.damage = 34..=36;
        m.damage_type("Physical", 50);
        m.damage_type("Cold", 50);
        m.resist("Physical", 75);
        m.resist("Fire", 60);
        m.resist("Cold", 90);
        m.resist("Poison", 100);
        m.resist("Energy", 60);
        m.skill("DetectHidden", 80.0);
        m.skill("EvalInt", 77.6, 87.5);
        m.skill("Magery", 77.6, 87.5);
        m.skill("Meditation", 100.0);
        m.skill("MagicResist", 50.1, 75.0);
        m.skill("Tactics", 100.0);
        m.skill("Wrestling", 100.0);
        m.fame = 20000;
        m.karma = -20000;
        m.armor = 44;
    });
});

mobile("CrystalElemental", |c| {
    c.ai = "Mage";
    c.corpse_name = "a crystal elemental corpse";
    c.name = "a crystal elemental";
    c.bleed_immune = true;
    c.poison_immune = 4;
    c.treasure_map_level = 1;
    c.loot = ["Rich", "Average"];
    c.weapon_ability = "BleedAttack";
    c.on_create(|m| {
        m.body = 300;
        m.sound = 0x116;
        m.str = 136..=160;
        m.dex = 51..=65;
        m.int = 86..=110;
        m.hits = 150;
        m.damage = 10..=15;
        m.damage_type("Physical", 80);
        m.damage_type("Energy", 20);
        m.resist("Physical", 50, 60);
        m.resist("Fire", 40, 50);
        m.resist("Cold", 40, 50);
        m.resist("Poison", 100);
        m.resist("Energy", 55, 70);
        m.skill("EvalInt", 70.1, 75.0);
        m.skill("Magery", 70.1, 75.0);
        m.skill("Meditation", 65.1, 75.0);
        m.skill("MagicResist", 80.1, 90.0);
        m.skill("Tactics", 75.1, 85.0);
        m.skill("Wrestling", 65.1, 75.0);
        m.fame = 6500;
        m.karma = -6500;
        m.armor = 54;
    });
});

mobile("DarknightCreeper", |c| {
    c.ai = "Mage";
    c.corpse_name = "a darknight creeper corpse";
    c.bleed_immune = true;
    c.poison_immune = 4;
    c.treasure_map_level = 1;
    c.loot = [["UltraRich", 2]];
    c.hit_poison = 4;
    c.ignore_young_protection = #{ atLeast: "ML", then: true, "else": false };
    c.bard_immune = #{ atLeast: "SE", then: false, "else": true };
    c.unprovokable = #{ atLeast: "SE", then: true, "else": false };
    c.area_peace_immune = #{ atLeast: "SE", then: true, "else": false };
    c.on_create(|m| {
        m.name = random_name("darknight creeper");
        m.body = 313;
        m.sound = 0xE0;
        m.str = 301..=330;
        m.dex = 101..=110;
        m.int = 301..=330;
        m.hits = 4000;
        m.damage = 22..=26;
        m.damage_type("Physical", 85);
        m.damage_type("Poison", 15);
        m.resist("Physical", 60);
        m.resist("Fire", 60);
        m.resist("Cold", 100);
        m.resist("Poison", 90);
        m.resist("Energy", 75);
        m.skill("DetectHidden", 80.0);
        m.skill("EvalInt", 118.1, 120.0);
        m.skill("Magery", 112.6, 120.0);
        m.skill("Meditation", 150.0);
        m.skill("Poisoning", 120.0);
        m.skill("MagicResist", 90.1, 90.9);
        m.skill("Tactics", 100.0);
        m.skill("Wrestling", 90.1, 90.9);
        m.skill("Necromancy", 120.1, 130.0);
        m.skill("SpiritSpeak", 120.1, 130.0);
        m.fame = 22000;
        m.karma = -22000;
        m.armor = 34;
    });
});

mobile("Devourer", |c| {
    c.ai = "Mage";
    c.corpse_name = "a devourer of souls corpse";
    c.name = "a devourer of souls";
    c.poison_immune = 4;
    c.meat = 3;
    c.loot = [["FilthyRich", 2]];
    c.on_create(|m| {
        m.body = 303;
        m.sound = 0x165;
        m.str = 801..=950;
        m.dex = 126..=175;
        m.int = 201..=250;
        m.hits = 650;
        m.damage = 22..=26;
        m.damage_type("Physical", 60);
        m.damage_type("Cold", 20);
        m.damage_type("Energy", 20);
        m.resist("Physical", 45, 55);
        m.resist("Fire", 25, 35);
        m.resist("Cold", 15, 25);
        m.resist("Poison", 60, 70);
        m.resist("Energy", 40, 50);
        m.skill("EvalInt", 90.1, 100.0);
        m.skill("Magery", 90.1, 100.0);
        m.skill("Meditation", 90.1, 100.0);
        m.skill("MagicResist", 90.1, 105.0);
        m.skill("Tactics", 75.1, 85.0);
        m.skill("Wrestling", 80.1, 100.0);
        m.fame = 9500;
        m.karma = -9500;
        m.armor = 44;
        m.pack = [#{ necroReg: [24, 45] }];
    });
});

mobile("FleshGolem", |c| {
    c.corpse_name = "a flesh golem corpse";
    c.name = "a flesh golem";
    c.bleed_immune = true;
    c.treasure_map_level = 1;
    c.loot = "Average";
    c.weapon_ability = "BleedAttack";
    c.on_create(|m| {
        m.body = 304;
        m.sound = 0x2AC;
        m.str = 176..=200;
        m.dex = 51..=75;
        m.int = 46..=70;
        m.hits = 106..=120;
        m.damage = 18..=22;
        m.damage_type("Physical", 100);
        m.resist("Physical", 50, 60);
        m.resist("Fire", 25, 35);
        m.resist("Cold", 15, 25);
        m.resist("Poison", 60, 70);
        m.resist("Energy", 30, 40);
        m.skill("MagicResist", 50.1, 75.0);
        m.skill("Tactics", 55.1, 80.0);
        m.skill("Wrestling", 60.1, 70.0);
        m.fame = 1000;
        m.karma = -1800;
        m.armor = 34;
    });
});

mobile("FleshRenderer", |c| {
    c.corpse_name = "a fleshrenderer corpse";
    c.name = "a fleshrenderer";
    c.poison_immune = 4;
    c.treasure_map_level = 1;
    c.loot = [["UltraRich", 2]];
    c.auto_dispel = true;
    c.ignore_young_protection = #{ atLeast: "ML", then: true, "else": false };
    c.bard_immune = #{ atLeast: "SE", then: false, "else": true };
    c.unprovokable = #{ atLeast: "SE", then: true, "else": false };
    c.area_peace_immune = #{ atLeast: "SE", then: true, "else": false };
    c.weapon_ability = #{ coin: ["Dismount", "ParalyzingBlow"] };
    c.attack_sound = 844;
    c.hurt_sound = 852;
    c.anger_sound = 844;
    c.idle_sound = 844;
    c.death_sound = 852;
    c.on_create(|m| {
        m.body = 315;
        m.str = 401..=460;
        m.dex = 201..=210;
        m.int = 221..=260;
        m.hits = 4500;
        m.damage = 16..=20;
        m.damage_type("Physical", 80);
        m.damage_type("Poison", 20);
        m.resist("Physical", 80, 90);
        m.resist("Fire", 50, 60);
        m.resist("Cold", 50, 60);
        m.resist("Poison", 100);
        m.resist("Energy", 70, 80);
        m.skill("DetectHidden", 80.0);
        m.skill("MagicResist", 155.1, 160.0);
        m.skill("Meditation", 100.0);
        m.skill("Tactics", 100.0);
        m.skill("Wrestling", 90.1, 100.0);
        m.fame = 23000;
        m.karma = -23000;
        m.armor = 24;
    });
});

mobile("Gibberling", |c| {
    c.corpse_name = "a gibberling corpse";
    c.name = "a gibberling";
    c.treasure_map_level = 1;
    c.loot = "Meager";
    c.weapon_ability = "Dismount";
    c.on_create(|m| {
        m.body = 307;
        m.sound = 0x1A6;
        m.str = 141..=165;
        m.dex = 101..=125;
        m.int = 56..=80;
        m.hits = 85..=99;
        m.damage = 12..=17;
        m.damage_type("Physical", 0);
        m.damage_type("Fire", 40);
        m.damage_type("Energy", 60);
        m.resist("Physical", 45, 55);
        m.resist("Fire", 25, 35);
        m.resist("Cold", 25, 35);
        m.resist("Poison", 10, 20);
        m.resist("Energy", 30, 40);
        m.skill("MagicResist", 45.1, 70.0);
        m.skill("Tactics", 67.6, 92.5);
        m.skill("Wrestling", 60.1, 80.0);
        m.fame = 1500;
        m.karma = -1500;
        m.armor = 27;
    });
});

mobile("GoreFiend", |c| {
    c.corpse_name = "a gore fiend corpse";
    c.name = "a gore fiend";
    c.bleed_immune = true;
    c.loot = "Average";
    c.death_sound = 1218;
    c.on_create(|m| {
        m.body = 305;
        m.sound = 0xE0;
        m.str = 161..=185;
        m.dex = 41..=65;
        m.int = 46..=70;
        m.hits = 97..=111;
        m.damage = 15..=21;
        m.damage_type("Physical", 85);
        m.damage_type("Poison", 15);
        m.resist("Physical", 35, 45);
        m.resist("Fire", 25, 35);
        m.resist("Cold", 15, 25);
        m.resist("Poison", 5, 15);
        m.resist("Energy", 30, 40);
        m.skill("MagicResist", 40.1, 55.0);
        m.skill("Tactics", 45.1, 70.0);
        m.skill("Wrestling", 50.1, 70.0);
        m.fame = 1500;
        m.karma = -1500;
        m.armor = 24;
    });
});

mobile("Impaler", |c| {
    c.corpse_name = "an impaler corpse";
    c.poison_immune = 4;
    c.treasure_map_level = 1;
    c.loot = [["UltraRich", 2]];
    c.auto_dispel = true;
    c.ignore_young_protection = #{ atLeast: "ML", then: true, "else": false };
    c.bard_immune = #{ atLeast: "SE", then: false, "else": true };
    c.unprovokable = #{ atLeast: "SE", then: true, "else": false };
    c.area_peace_immune = #{ atLeast: "SE", then: true, "else": false };
    c.hit_poison = #{ chance: 0.8, then: 2, "else": 3 };
    c.weapon_ability = #{ coin: ["MortalStrike", "BleedAttack"] };
    c.on_create(|m| {
        m.name = random_name("impaler");
        m.body = 306;
        m.sound = 0x2A7;
        m.str = 190;
        m.dex = 45;
        m.int = 190;
        m.hits = 5000;
        m.damage = 31..=35;
        m.damage_type("Physical", 100);
        m.resist("Physical", 90);
        m.resist("Fire", 60);
        m.resist("Cold", 75);
        m.resist("Poison", 60);
        m.resist("Energy", 100);
        m.skill("DetectHidden", 80.0);
        m.skill("Meditation", 120.0);
        m.skill("Poisoning", 160.0);
        m.skill("MagicResist", 100.0);
        m.skill("Tactics", 100.0);
        m.skill("Wrestling", 80.0);
        m.fame = 24000;
        m.karma = -24000;
        m.armor = 49;
    });
});

mobile("MoundOfMaggots", |c| {
    c.corpse_name = "a maggoty corpse";
    c.name = "a mound of maggots";
    c.poison_immune = 4;
    c.treasure_map_level = 1;
    c.loot = ["Meager", "Gems"];
    c.on_create(|m| {
        m.body = 319;
        m.sound = 0x382;
        m.str = 61..=70;
        m.dex = 61..=70;
        m.int = 10;
        m.mana = 0;
        m.damage = 3..=9;
        m.damage_type("Physical", 50);
        m.damage_type("Poison", 50);
        m.resist("Physical", 90);
        m.resist("Poison", 100);
        m.skill("Tactics", 50.0);
        m.skill("Wrestling", 50.1, 60.0);
        m.fame = 1000;
        m.karma = -1000;
        m.armor = 24;
    });
});

mobile("PatchworkSkeleton", |c| {
    c.corpse_name = "a patchwork skeletal corpse";
    c.name = "a patchwork skeleton";
    c.bleed_immune = true;
    c.poison_immune = 4;
    c.treasure_map_level = 1;
    c.loot = "Meager";
    c.weapon_ability = "Dismount";
    c.on_create(|m| {
        m.body = 309;
        m.sound = 0x48D;
        m.str = 96..=120;
        m.dex = 71..=95;
        m.int = 16..=40;
        m.hits = 58..=72;
        m.damage = 18..=22;
        m.damage_type("Physical", 85);
        m.damage_type("Cold", 15);
        m.resist("Physical", 55, 65);
        m.resist("Fire", 50, 60);
        m.resist("Cold", 70, 80);
        m.resist("Poison", 100);
        m.resist("Energy", 40, 50);
        m.skill("MagicResist", 70.1, 95.0);
        m.skill("Tactics", 55.1, 80.0);
        m.skill("Wrestling", 50.1, 70.0);
        m.fame = 500;
        m.karma = -500;
        m.armor = 54;
    });
});

mobile("Ravager", |c| {
    c.corpse_name = "a ravager corpse";
    c.name = "a ravager";
    c.loot = "Rich";
    c.weapon_ability = #{ coin: ["Dismount", "CrushingBlow"] };
    c.on_create(|m| {
        m.body = 314;
        m.sound = 0x165;
        m.str = 251..=275;
        m.dex = 101..=125;
        m.int = 66..=90;
        m.hits = 161..=175;
        m.damage = 15..=20;
        m.damage_type("Physical", 100);
        m.resist("Physical", 50, 60);
        m.resist("Fire", 50, 60);
        m.resist("Cold", 60, 70);
        m.resist("Poison", 30, 40);
        m.resist("Energy", 20, 30);
        m.skill("MagicResist", 50.1, 75.0);
        m.skill("Tactics", 75.1, 100.0);
        m.skill("Wrestling", 70.1, 90.0);
        m.fame = 3500;
        m.karma = -3500;
        m.armor = 54;
    });
});

mobile("ShadowKnight", |c| {
    c.ai = "Mage";
    c.corpse_name = "a shadow knight corpse";
    c.poison_immune = 4;
    c.treasure_map_level = 1;
    c.loot = [["UltraRich", 2]];
    c.opposition = "FeyAndUndead";
    c.ignore_young_protection = #{ atLeast: "ML", then: true, "else": false };
    c.bard_immune = #{ atLeast: "SE", then: false, "else": true };
    c.unprovokable = #{ atLeast: "SE", then: true, "else": false };
    c.area_peace_immune = #{ atLeast: "SE", then: true, "else": false };
    c.weapon_ability = #{ coin: ["ConcussionBlow", "CrushingBlow"] };
    c.idle_sound = 718;
    c.death_sound = 705;
    c.hurt_sound = 721;
    c.attack_sound = 712;
    c.on_create(|m| {
        m.name = random_name("shadow knight");
        m.title = "the Shadow Knight";
        m.body = 311;
        m.str = 250;
        m.dex = 100;
        m.int = 100;
        m.hits = 2000;
        m.damage = 20..=30;
        m.damage_type("Physical", 60);
        m.damage_type("Cold", 40);
        m.resist("Physical", 90);
        m.resist("Fire", 65);
        m.resist("Cold", 75);
        m.resist("Poison", 75);
        m.resist("Energy", 55);
        m.skill("Chivalry", 120.0);
        m.skill("DetectHidden", 80.0);
        m.skill("EvalInt", 100.0);
        m.skill("Magery", 100.0);
        m.skill("Meditation", 100.0);
        m.skill("MagicResist", 120.0);
        m.skill("Tactics", 100.0);
        m.skill("Wrestling", 100.0);
        m.fame = 25000;
        m.karma = -25000;
        m.armor = 54;
    });
});

mobile("SkitteringHopper", |c| {
    c.fight_mode = "Aggressor";
    c.corpse_name = "a skittering hopper corpse";
    c.name = "a skittering hopper";
    c.treasure_map_level = 1;
    c.loot = "Meager";
    c.on_create(|m| {
        m.body = 302;
        m.sound = 0x3BF;
        m.str = 41..=65;
        m.dex = 91..=115;
        m.int = 26..=50;
        m.hits = 31..=45;
        m.damage = 3..=5;
        m.damage_type("Physical", 100);
        m.resist("Physical", 5, 10);
        m.resist("Cold", 10, 20);
        m.resist("Energy", 5, 10);
        m.skill("MagicResist", 30.1, 45.0);
        m.skill("Tactics", 45.1, 70.0);
        m.skill("Wrestling", 40.1, 60.0);
        m.fame = 300;
        m.karma = 0;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = -12.9;
        m.armor = 12;
    });
});

mobile("Treefellow", |c| {
    c.fight_mode = "Evil";
    c.corpse_name = "a treefellow corpse";
    c.name = "a treefellow";
    c.bleed_immune = true;
    c.loot = "Average";
    c.opposition = "FeyAndUndead";
    c.weapon_ability = "Dismount";
    c.idle_sound = 443;
    c.death_sound = 31;
    c.attack_sound = 672;
    c.on_create(|m| {
        m.body = 301;
        m.str = 196..=220;
        m.dex = 31..=55;
        m.int = 66..=90;
        m.hits = 118..=132;
        m.damage = 12..=16;
        m.damage_type("Physical", 100);
        m.resist("Physical", 20, 25);
        m.resist("Cold", 50, 60);
        m.resist("Poison", 30, 35);
        m.resist("Energy", 20, 30);
        m.skill("MagicResist", 40.1, 55.0);
        m.skill("Tactics", 65.1, 90.0);
        m.skill("Wrestling", 65.1, 85.0);
        m.fame = 500;
        m.karma = 1500;
        m.armor = 24;
        m.pack = [#{ item: "Server.Items.Log", amount: [23, 34] }];
    });
});

mobile("WailingBanshee", |c| {
    c.corpse_name = "a wailing banshee corpse";
    c.name = "a wailing banshee";
    c.bleed_immune = true;
    c.loot = "Meager";
    c.weapon_ability = "MortalStrike";
    c.on_create(|m| {
        m.body = 310;
        m.sound = 0x482;
        m.str = 126..=150;
        m.dex = 76..=100;
        m.int = 86..=110;
        m.hits = 76..=90;
        m.damage = 10..=14;
        m.damage_type("Physical", 20);
        m.damage_type("Cold", 60);
        m.damage_type("Poison", 20);
        m.resist("Physical", 50, 60);
        m.resist("Fire", 25, 30);
        m.resist("Cold", 70, 80);
        m.resist("Poison", 30, 40);
        m.resist("Energy", 40, 50);
        m.skill("MagicResist", 70.1, 95.0);
        m.skill("Tactics", 45.1, 70.0);
        m.skill("Wrestling", 50.1, 70.0);
        m.fame = 1500;
        m.karma = -1500;
        m.armor = 19;
    });
});

mobile("DreadSpider", |c| {
    c.ai = "Mage";
    c.corpse_name = "a dread spider corpse";
    c.name = "a dread spider";
    c.poison_immune = 4;
    c.treasure_map_level = 3;
    c.loot = "FilthyRich";
    c.hit_poison = 4;
    c.on_create(|m| {
        m.body = 11;
        m.sound = 0x492;
        m.str = 196..=220;
        m.dex = 126..=145;
        m.int = 286..=310;
        m.hits = 118..=132;
        m.damage = 5..=17;
        m.damage_type("Physical", 20);
        m.damage_type("Poison", 80);
        m.resist("Physical", 40, 50);
        m.resist("Fire", 20, 30);
        m.resist("Cold", 20, 30);
        m.resist("Poison", 90, 100);
        m.resist("Energy", 20, 30);
        m.skill("EvalInt", 65.1, 80.0);
        m.skill("Magery", 65.1, 80.0);
        m.skill("Meditation", 65.1, 80.0);
        m.skill("MagicResist", 45.1, 60.0);
        m.skill("Tactics", 55.1, 70.0);
        m.skill("Wrestling", 60.1, 75.0);
        m.fame = 5000;
        m.karma = -5000;
        m.armor = 36;
        m.pack = [#{ item: "Server.Items.SpidersSilk", amount: 8 }];
    });
});

mobile("TerathanAvenger", |c| {
    c.ai = "Mage";
    c.corpse_name = "a terathan avenger corpse";
    c.name = "a terathan avenger";
    c.poison_immune = 3;
    c.treasure_map_level = 3;
    c.meat = 2;
    c.loot = [["Rich", 2]];
    c.hit_poison = 3;
    c.opposition = "TerathansAndOphidians";
    c.on_create(|m| {
        m.body = 152;
        m.sound = 0x24D;
        m.str = 467..=645;
        m.dex = 77..=95;
        m.int = 126..=150;
        m.hits = 296..=372;
        m.mana = 46..=70;
        m.damage = 18..=22;
        m.damage_type("Physical", 50);
        m.damage_type("Poison", 50);
        m.resist("Physical", 40, 50);
        m.resist("Fire", 30, 40);
        m.resist("Cold", 35, 45);
        m.resist("Poison", 90, 100);
        m.resist("Energy", 35, 45);
        m.skill("EvalInt", 70.3, 100.0);
        m.skill("Magery", 70.3, 100.0);
        m.skill("Poisoning", 60.1, 80.0);
        m.skill("MagicResist", 65.1, 80.0);
        m.skill("Tactics", 90.1, 100.0);
        m.skill("Wrestling", 90.1, 100.0);
        m.fame = 15000;
        m.karma = -15000;
        m.armor = 50;
    });
});

mobile("TerathanMatriarch", |c| {
    c.ai = "Mage";
    c.corpse_name = "a terathan matriarch corpse";
    c.name = "a terathan matriarch";
    c.treasure_map_level = 4;
    c.loot = ["Rich", ["Average", 2], ["MedScrolls", 2], "Potions"];
    c.opposition = "TerathansAndOphidians";
    c.on_create(|m| {
        m.body = 72;
        m.sound = 0x257;
        m.str = 316..=405;
        m.dex = 96..=115;
        m.int = 366..=455;
        m.hits = 190..=243;
        m.damage = 11..=14;
        m.damage_type("Physical", 100);
        m.resist("Physical", 45, 55);
        m.resist("Fire", 30, 40);
        m.resist("Cold", 35, 45);
        m.resist("Poison", 40, 50);
        m.resist("Energy", 35, 45);
        m.skill("EvalInt", 90.1, 100.0);
        m.skill("Magery", 90.1, 100.0);
        m.skill("MagicResist", 90.1, 100.0);
        m.skill("Tactics", 50.1, 70.0);
        m.skill("Wrestling", 60.1, 80.0);
        m.fame = 10000;
        m.karma = -10000;
        m.pack = [#{ item: "Server.Items.SpidersSilk", amount: 5 }, #{ necroReg: [4, 10] }];
    });
});

mobile("FrostSpider", |c| {
    c.corpse_name = "a frost spider corpse";
    c.name = "a frost spider";
    c.loot = ["Meager", "Poor"];
    c.favorite_food = ["Meat"];
    c.pack_instinct = ["Arachnid"];
    c.on_create(|m| {
        m.body = 20;
        m.sound = 0x388;
        m.str = 76..=100;
        m.dex = 126..=145;
        m.int = 36..=60;
        m.hits = 46..=60;
        m.mana = 0;
        m.damage = 6..=16;
        m.damage_type("Physical", 20);
        m.damage_type("Cold", 80);
        m.resist("Physical", 25, 30);
        m.resist("Fire", 5, 10);
        m.resist("Cold", 40, 50);
        m.resist("Poison", 20, 30);
        m.resist("Energy", 10, 20);
        m.skill("MagicResist", 25.1, 40.0);
        m.skill("Tactics", 35.1, 50.0);
        m.skill("Wrestling", 50.1, 65.0);
        m.fame = 775;
        m.karma = -775;
        m.armor = 28;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = 74.7;
        m.pack = [#{ item: "Server.Items.SpidersSilk", amount: 7 }];
    });
});

mobile("GiantBlackWidow", |c| {
    c.corpse_name = "a giant black widow spider corpse";
    c.name = "a giant black wide";
    c.poison_immune = 3;
    c.loot = "Average";
    c.favorite_food = ["Meat"];
    c.hit_poison = 3;
    c.on_create(|m| {
        m.body = 157;
        m.sound = 0x388;
        m.str = 76..=100;
        m.dex = 96..=115;
        m.int = 36..=60;
        m.hits = 46..=60;
        m.damage = 5..=17;
        m.damage_type("Physical", 100);
        m.resist("Physical", 20, 30);
        m.resist("Fire", 10, 20);
        m.resist("Cold", 10, 20);
        m.resist("Poison", 50, 60);
        m.resist("Energy", 10, 20);
        m.skill("Anatomy", 30.3, 75.0);
        m.skill("Poisoning", 60.1, 80.0);
        m.skill("MagicResist", 45.1, 60.0);
        m.skill("Tactics", 65.1, 80.0);
        m.skill("Wrestling", 70.1, 85.0);
        m.fame = 3500;
        m.karma = -3500;
        m.armor = 24;
        m.pack = [
            #{ item: "Server.Items.SpidersSilk", amount: 5 },
            #{ item: "Server.Items.LesserPoisonPotion" },
            #{ item: "Server.Items.LesserPoisonPotion" },
        ];
    });
});

mobile("GiantSpider", |c| {
    c.corpse_name = "a giant spider corpse";
    c.name = "a giant spider";
    c.poison_immune = 1;
    c.loot = "Poor";
    c.favorite_food = ["Meat"];
    c.pack_instinct = ["Arachnid"];
    c.hit_poison = 1;
    c.on_create(|m| {
        m.body = 28;
        m.sound = 0x388;
        m.str = 76..=100;
        m.dex = 76..=95;
        m.int = 36..=60;
        m.hits = 46..=60;
        m.mana = 0;
        m.damage = 5..=13;
        m.damage_type("Physical", 100);
        m.resist("Physical", 15, 20);
        m.resist("Poison", 25, 35);
        m.skill("Poisoning", 60.1, 80.0);
        m.skill("MagicResist", 25.1, 40.0);
        m.skill("Tactics", 35.1, 50.0);
        m.skill("Wrestling", 50.1, 65.0);
        m.fame = 600;
        m.karma = -600;
        m.armor = 16;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = 59.1;
        m.pack = [#{ item: "Server.Items.SpidersSilk", amount: 5 }];
    });
});

mobile("TerathanDrone", |c| {
    c.corpse_name = "a terathan drone corpse";
    c.name = "a terathan drone";
    c.meat = 4;
    c.loot = "Meager";
    c.opposition = "TerathansAndOphidians";
    c.on_create(|m| {
        m.body = 71;
        m.sound = 0x252;
        m.str = 36..=65;
        m.dex = 96..=145;
        m.int = 21..=45;
        m.hits = 22..=39;
        m.mana = 0;
        m.damage = 6..=12;
        m.damage_type("Physical", 100);
        m.resist("Physical", 20, 25);
        m.resist("Fire", 10, 20);
        m.resist("Cold", 15, 25);
        m.resist("Poison", 30, 40);
        m.resist("Energy", 15, 25);
        m.skill("Poisoning", 40.1, 60.0);
        m.skill("MagicResist", 30.1, 45.0);
        m.skill("Tactics", 30.1, 50.0);
        m.skill("Wrestling", 40.1, 50.0);
        m.fame = 2000;
        m.karma = -2000;
        m.armor = 24;
        m.pack = [#{ item: "Server.Items.SpidersSilk", amount: 2 }];
    });
});

mobile("AcidElemental", |c| {
    c.ai = "Mage";
    c.corpse_name = "an acid elemental corpse";
    c.name = "an acid elemental";
    c.bleed_immune = true;
    c.loot = ["Rich", "Average"];
    c.hit_poison = 4;
    c.hit_poison_chance = 0.6;
    c.treasure_map_level = #{ atLeast: "AOS", then: 2, "else": 3 };
    c.on_create(|m| {
        m.body = 158;
        m.sound = 0x116;
        m.str = 326..=355;
        m.dex = 66..=85;
        m.int = 271..=295;
        m.hits = 196..=213;
        m.damage = 9..=15;
        m.damage_type("Physical", 25);
        m.damage_type("Fire", 50);
        m.damage_type("Energy", 25);
        m.resist("Physical", 45, 55);
        m.resist("Fire", 40, 50);
        m.resist("Cold", 20, 30);
        m.resist("Poison", 10, 20);
        m.resist("Energy", 30, 40);
        m.skill("Anatomy", 30.3, 60.0);
        m.skill("EvalInt", 70.1, 85.0);
        m.skill("Magery", 70.1, 85.0);
        m.skill("MagicResist", 60.1, 75.0);
        m.skill("Tactics", 80.1, 90.0);
        m.skill("Wrestling", 70.1, 90.0);
        m.fame = 10000;
        m.karma = -10000;
        m.armor = 40;
    });
});

mobile("AirElemental", |c| {
    c.ai = "Mage";
    c.corpse_name = "an air elemental corpse";
    c.name = "an air elemental";
    c.bleed_immune = true;
    c.treasure_map_level = 2;
    c.loot = ["Average", "Meager", "LowScrolls", "MedScrolls"];
    c.on_create(|m| {
        m.body = 13;
        m.hue = 0x4001;
        m.sound = 0x28F;
        m.str = 126..=155;
        m.dex = 166..=185;
        m.int = 101..=125;
        m.hits = 76..=93;
        m.damage = 8..=10;
        m.damage_type("Physical", 20);
        m.damage_type("Cold", 40);
        m.damage_type("Energy", 40);
        m.resist("Physical", 35, 45);
        m.resist("Fire", 15, 25);
        m.resist("Cold", 10, 20);
        m.resist("Poison", 10, 20);
        m.resist("Energy", 25, 35);
        m.skill("EvalInt", 60.1, 75.0);
        m.skill("Magery", 60.1, 75.0);
        m.skill("MagicResist", 60.1, 75.0);
        m.skill("Tactics", 60.1, 80.0);
        m.skill("Wrestling", 60.1, 80.0);
        m.fame = 4500;
        m.karma = -4500;
        m.armor = 40;
    });
});

mobile("BloodElemental", |c| {
    c.ai = "Mage";
    c.corpse_name = "a blood elemental corpse";
    c.name = "a blood elemental";
    c.treasure_map_level = 5;
    c.loot = ["FilthyRich", "Rich"];
    c.on_create(|m| {
        m.body = 159;
        m.sound = 0x116;
        m.str = 526..=615;
        m.dex = 66..=85;
        m.int = 226..=350;
        m.hits = 316..=369;
        m.damage = 17..=27;
        m.damage_type("Physical", 0);
        m.damage_type("Poison", 50);
        m.damage_type("Energy", 50);
        m.resist("Physical", 55, 65);
        m.resist("Fire", 20, 30);
        m.resist("Cold", 40, 50);
        m.resist("Poison", 50, 60);
        m.resist("Energy", 30, 40);
        m.skill("EvalInt", 85.1, 100.0);
        m.skill("Magery", 85.1, 100.0);
        m.skill("Meditation", 10.4, 50.0);
        m.skill("MagicResist", 80.1, 95.0);
        m.skill("Tactics", 80.1, 100.0);
        m.skill("Wrestling", 80.1, 100.0);
        m.fame = 12500;
        m.karma = -12500;
        m.armor = 60;
    });
});

mobile("Efreet", |c| {
    c.ai = "Mage";
    c.corpse_name = "an efreet corpse";
    c.name = "an efreet";
    c.treasure_map_level = #{ atLeast: "AOS", then: 4, "else": 5 };
    c.loot = [
        ["Rich", 1],
        ["Average", 1],
        ["Gems", 1],
        #{
            chance: 0.02,
            loot: [
                #{
                    oneOf: [
                        #{ item: "Server.Items.DaemonArms" },
                        #{ item: "Server.Items.DaemonChest" },
                        #{ item: "Server.Items.DaemonGloves" },
                        #{ item: "Server.Items.DaemonLegs" },
                        #{ item: "Server.Items.DaemonHelm" },
                    ],
                },
            ],
        },
    ];
    c.on_create(|m| {
        m.body = 131;
        m.sound = 0x300;
        m.str = 326..=355;
        m.dex = 266..=285;
        m.int = 171..=195;
        m.hits = 196..=213;
        m.damage = 11..=13;
        m.damage_type("Physical", 0);
        m.damage_type("Fire", 50);
        m.damage_type("Energy", 50);
        m.resist("Physical", 50, 60);
        m.resist("Fire", 60, 70);
        m.resist("Poison", 30, 40);
        m.resist("Energy", 40, 50);
        m.skill("EvalInt", 60.1, 75.0);
        m.skill("Magery", 60.1, 75.0);
        m.skill("MagicResist", 60.1, 75.0);
        m.skill("Tactics", 60.1, 80.0);
        m.skill("Wrestling", 60.1, 80.0);
        m.fame = 10000;
        m.karma = -10000;
        m.armor = 56;
    });
});

mobile("FireElemental", |c| {
    c.ai = "Mage";
    c.corpse_name = "a fire elemental corpse";
    c.name = "a fire elemental";
    c.bleed_immune = true;
    c.treasure_map_level = 2;
    c.loot = ["Average", "Meager", "Gems"];
    c.on_create(|m| {
        m.body = 15;
        m.sound = 0x346;
        m.str = 126..=155;
        m.dex = 166..=185;
        m.int = 101..=125;
        m.hits = 76..=93;
        m.damage = 7..=9;
        m.damage_type("Physical", 25);
        m.damage_type("Fire", 75);
        m.resist("Physical", 35, 45);
        m.resist("Fire", 60, 80);
        m.resist("Cold", 5, 10);
        m.resist("Poison", 30, 40);
        m.resist("Energy", 30, 40);
        m.skill("EvalInt", 60.1, 75.0);
        m.skill("Magery", 60.1, 75.0);
        m.skill("MagicResist", 75.2, 105.0);
        m.skill("Tactics", 80.1, 100.0);
        m.skill("Wrestling", 70.1, 100.0);
        m.fame = 4500;
        m.karma = -4500;
        m.armor = 40;
        m.pack = [
            #{ item: "Server.Items.SulfurousAsh", amount: 3 },
            #{ item: "Server.Items.LightSource", equip: true },
        ];
    });
});

mobile("IceElemental", |c| {
    c.ai = "Mage";
    c.corpse_name = "an ice elemental corpse";
    c.name = "an ice elemental";
    c.bleed_immune = true;
    c.loot = [["Average", 2], ["Gems", 2]];
    c.on_create(|m| {
        m.body = 161;
        m.sound = 0x10C;
        m.str = 156..=185;
        m.dex = 96..=115;
        m.int = 171..=192;
        m.hits = 94..=111;
        m.damage = 10..=21;
        m.damage_type("Physical", 25);
        m.damage_type("Cold", 75);
        m.resist("Physical", 35, 45);
        m.resist("Fire", 5, 10);
        m.resist("Cold", 50, 60);
        m.resist("Poison", 20, 30);
        m.resist("Energy", 20, 30);
        m.skill("EvalInt", 10.5, 60.0);
        m.skill("Magery", 10.5, 60.0);
        m.skill("MagicResist", 30.1, 80.0);
        m.skill("Tactics", 70.1, 100.0);
        m.skill("Wrestling", 60.1, 100.0);
        m.fame = 4000;
        m.karma = -4000;
        m.armor = 40;
        m.pack = [#{ item: "Server.Items.BlackPearl" }, #{ reg: 3 }];
    });
});

mobile("PoisonElemental", |c| {
    c.ai = "Mage";
    c.corpse_name = "a poison elementals corpse";
    c.name = "a poison elemental";
    c.bleed_immune = true;
    c.poison_immune = 4;
    c.treasure_map_level = 5;
    c.loot = ["FilthyRich", "Rich", "MedScrolls"];
    c.hit_poison = 4;
    c.hit_poison_chance = 0.75;
    c.on_create(|m| {
        m.body = 162;
        m.sound = 0x107;
        m.str = 426..=515;
        m.dex = 166..=185;
        m.int = 361..=435;
        m.hits = 256..=309;
        m.damage = 12..=18;
        m.damage_type("Physical", 10);
        m.damage_type("Poison", 90);
        m.resist("Physical", 60, 70);
        m.resist("Fire", 20, 30);
        m.resist("Cold", 20, 30);
        m.resist("Poison", 100);
        m.resist("Energy", 40, 50);
        m.skill("EvalInt", 80.1, 95.0);
        m.skill("Magery", 80.1, 95.0);
        m.skill("Meditation", 80.2, 120.0);
        m.skill("Poisoning", 90.1, 100.0);
        m.skill("MagicResist", 85.2, 115.0);
        m.skill("Tactics", 80.1, 100.0);
        m.skill("Wrestling", 70.1, 90.0);
        m.fame = 12500;
        m.karma = -12500;
        m.armor = 70;
        m.pack = [#{ item: "Server.Items.Nightshade", amount: 4 }, #{ item: "Server.Items.LesserPoisonPotion" }];
    });
});

mobile("EarthElemental", |c| {
    c.corpse_name = "an earth elemental corpse";
    c.name = "an earth elemental";
    c.bleed_immune = true;
    c.treasure_map_level = 1;
    c.loot = ["Average", "Meager", "Gems"];
    c.on_create(|m| {
        m.body = 14;
        m.sound = 0x10C;
        m.str = 126..=155;
        m.dex = 66..=85;
        m.int = 71..=92;
        m.hits = 76..=93;
        m.damage = 9..=16;
        m.damage_type("Physical", 100);
        m.resist("Physical", 30, 35);
        m.resist("Fire", 10, 20);
        m.resist("Cold", 10, 20);
        m.resist("Poison", 15, 25);
        m.resist("Energy", 15, 25);
        m.skill("MagicResist", 50.1, 95.0);
        m.skill("Tactics", 60.1, 100.0);
        m.skill("Wrestling", 60.1, 100.0);
        m.fame = 3500;
        m.karma = -3500;
        m.armor = 34;
        m.pack = [
            #{ item: "Server.Items.FertileDirt", amount: [1, 4] },
            #{ item: "Server.Items.MandrakeRoot" },
            #{ item: "Server.Items.IronOre", amount: 5, itemId: 6583 },
        ];
    });
});

mobile("SnowElemental", |c| {
    c.corpse_name = "a snow elemental corpse";
    c.name = "a snow elemental";
    c.bleed_immune = true;
    c.loot = "Rich";
    c.treasure_map_level = #{ oneOf: [2, 3] };
    c.on_create(|m| {
        m.body = 163;
        m.sound = 0x107;
        m.str = 326..=355;
        m.dex = 166..=185;
        m.int = 71..=95;
        m.hits = 196..=213;
        m.damage = 11..=17;
        m.damage_type("Physical", 20);
        m.damage_type("Cold", 80);
        m.resist("Physical", 45, 55);
        m.resist("Fire", 10, 15);
        m.resist("Cold", 60, 70);
        m.resist("Poison", 25, 35);
        m.resist("Energy", 25, 35);
        m.skill("MagicResist", 50.1, 65.0);
        m.skill("Tactics", 80.1, 100.0);
        m.skill("Wrestling", 80.1, 100.0);
        m.fame = 5000;
        m.karma = -5000;
        m.armor = 50;
        m.pack = [
            #{ item: "Server.Items.BlackPearl", amount: 3 },
            #{ item: "Server.Items.IronOre", amount: 3, itemId: 6584 },
        ];
    });
});

mobile("AncientLich", |c| {
    c.ai = "Mage";
    c.corpse_name = "an ancient lich's corpse";
    c.bleed_immune = true;
    c.poison_immune = 4;
    c.treasure_map_level = 5;
    c.loot = [["FilthyRich", 3], ["MedScrolls", 2]];
    c.opposition = "FeyAndUndead";
    c.unprovokable = true;
    c.monster_abilities = ["SummonLesserUndeadCounter"];
    c.idle_sound = 413;
    c.anger_sound = 373;
    c.death_sound = 264;
    c.attack_sound = 226;
    c.hurt_sound = 651;
    c.on_create(|m| {
        m.name = random_name("ancient lich");
        m.body = 78;
        m.sound = 0x19C;
        m.str = 216..=305;
        m.dex = 96..=115;
        m.int = 966..=1045;
        m.hits = 560..=595;
        m.damage = 15..=27;
        m.damage_type("Physical", 20);
        m.damage_type("Cold", 40);
        m.damage_type("Energy", 40);
        m.resist("Physical", 55, 65);
        m.resist("Fire", 25, 30);
        m.resist("Cold", 50, 60);
        m.resist("Poison", 50, 60);
        m.resist("Energy", 25, 30);
        m.skill("EvalInt", 120.1, 130.0);
        m.skill("Magery", 120.1, 130.0);
        m.skill("Meditation", 100.1, 101.0);
        m.skill("Poisoning", 100.1, 101.0);
        m.skill("MagicResist", 175.2, 200.0);
        m.skill("Tactics", 90.1, 100.0);
        m.skill("Wrestling", 75.1, 100.0);
        m.skill("Necromancy", 120.1, 130.0);
        m.skill("SpiritSpeak", 120.1, 130.0);
        m.fame = 23000;
        m.karma = -23000;
        m.armor = 60;
        m.pack = [#{ necroReg: [30, 275] }];
    });
});

mobile("ArcaneDaemon", |c| {
    c.ai = "Mage";
    c.corpse_name = "an arcane daemon corpse";
    c.name = "an arcane daemon";
    c.poison_immune = 3;
    c.loot = [["Average", 2]];
    c.weapon_ability = "ConcussionBlow";
    c.on_create(|m| {
        m.body = 784;
        m.sound = 0x47D;
        m.str = 131..=150;
        m.dex = 126..=145;
        m.int = 301..=350;
        m.hits = 101..=115;
        m.damage = 12..=16;
        m.damage_type("Physical", 80);
        m.damage_type("Fire", 20);
        m.resist("Physical", 50, 60);
        m.resist("Fire", 70, 80);
        m.resist("Cold", 10, 20);
        m.resist("Poison", 50, 60);
        m.resist("Energy", 30, 40);
        m.skill("MagicResist", 85.1, 95.0);
        m.skill("Tactics", 70.1, 80.0);
        m.skill("Wrestling", 60.1, 80.0);
        m.skill("Magery", 80.1, 90.0);
        m.skill("EvalInt", 70.1, 80.0);
        m.skill("Meditation", 70.1, 80.0);
        m.fame = 7000;
        m.karma = -10000;
        m.armor = 55;
    });
});

mobile("Balron", |c| {
    c.ai = "Mage";
    c.corpse_name = "a balron corpse";
    c.can_rummage_corpses = true;
    c.poison_immune = 3;
    c.treasure_map_level = 5;
    c.meat = 1;
    c.loot = [["FilthyRich", 2], "Rich", ["MedScrolls", 2]];
    c.on_create(|m| {
        m.name = random_name("balron");
        m.body = 40;
        m.sound = 0x165;
        m.str = 986..=1185;
        m.dex = 177..=255;
        m.int = 151..=250;
        m.hits = 592..=711;
        m.damage = 22..=29;
        m.damage_type("Physical", 50);
        m.damage_type("Fire", 25);
        m.damage_type("Energy", 25);
        m.resist("Physical", 65, 80);
        m.resist("Fire", 60, 80);
        m.resist("Cold", 50, 60);
        m.resist("Poison", 100);
        m.resist("Energy", 40, 50);
        m.skill("Anatomy", 25.1, 50.0);
        m.skill("EvalInt", 90.1, 100.0);
        m.skill("Magery", 95.5, 100.0);
        m.skill("Meditation", 25.1, 50.0);
        m.skill("MagicResist", 100.5, 150.0);
        m.skill("Tactics", 90.1, 100.0);
        m.skill("Wrestling", 90.1, 100.0);
        m.fame = 24000;
        m.karma = -24000;
        m.armor = 90;
        m.pack = [#{ item: "Server.Items.Longsword" }];
    });
});

mobile("BoneMagi", |c| {
    c.ai = "Mage";
    c.corpse_name = "a skeletal corpse";
    c.name = "a bone mage";
    c.bleed_immune = true;
    c.poison_immune = 1;
    c.loot = ["Average", "LowScrolls", "Potions"];
    c.opposition = "FeyAndUndead";
    c.on_create(|m| {
        m.body = 148;
        m.sound = 0x1C3;
        m.str = 76..=100;
        m.dex = 56..=75;
        m.int = 186..=210;
        m.hits = 46..=60;
        m.damage = 3..=7;
        m.damage_type("Physical", 100);
        m.resist("Physical", 35, 40);
        m.resist("Fire", 20, 30);
        m.resist("Cold", 50, 60);
        m.resist("Poison", 20, 30);
        m.resist("Energy", 30, 40);
        m.skill("EvalInt", 60.1, 70.0);
        m.skill("Magery", 60.1, 70.0);
        m.skill("MagicResist", 55.1, 70.0);
        m.skill("Tactics", 45.1, 60.0);
        m.skill("Wrestling", 45.1, 55.0);
        m.skill("Necromancy", 89.0, 99.1);
        m.skill("SpiritSpeak", 90.0, 99.0);
        m.fame = 3000;
        m.karma = -3000;
        m.armor = 38;
        m.pack = [#{ reg: 3 }, #{ necroReg: [3, 10] }, #{ item: "Server.Items.Bone" }];
    });
});

mobile("Daemon", |c| {
    c.ai = "Mage";
    c.corpse_name = "a daemon corpse";
    c.can_rummage_corpses = true;
    c.poison_immune = 1;
    c.treasure_map_level = 4;
    c.meat = 1;
    c.can_fly = true;
    c.loot = ["Rich", ["Average", 2], ["MedScrolls", 2]];
    c.on_create(|m| {
        m.name = random_name("daemon");
        m.body = 9;
        m.sound = 0x165;
        m.str = 476..=505;
        m.dex = 76..=95;
        m.int = 301..=325;
        m.hits = 286..=303;
        m.damage = 7..=14;
        m.damage_type("Physical", 100);
        m.resist("Physical", 45, 60);
        m.resist("Fire", 50, 60);
        m.resist("Cold", 30, 40);
        m.resist("Poison", 20, 30);
        m.resist("Energy", 30, 40);
        m.skill("EvalInt", 70.1, 80.0);
        m.skill("Magery", 70.1, 80.0);
        m.skill("MagicResist", 85.1, 95.0);
        m.skill("Tactics", 70.1, 80.0);
        m.skill("Wrestling", 60.1, 80.0);
        m.fame = 15000;
        m.karma = -15000;
        m.armor = 58;
    });
});

mobile("ElderGazer", |c| {
    c.ai = "Mage";
    c.corpse_name = "an elder gazer corpse";
    c.name = "an elder gazer";
    c.loot = "FilthyRich";
    c.treasure_map_level = #{ atLeast: "AOS", then: 4, "else": 0 };
    c.on_create(|m| {
        m.body = 22;
        m.sound = 0x179;
        m.str = 296..=325;
        m.dex = 86..=105;
        m.int = 291..=385;
        m.hits = 178..=195;
        m.damage = 8..=19;
        m.damage_type("Physical", 50);
        m.damage_type("Energy", 50);
        m.resist("Physical", 45, 55);
        m.resist("Fire", 60, 70);
        m.resist("Cold", 40, 50);
        m.resist("Poison", 40, 50);
        m.resist("Energy", 40, 50);
        m.skill("Anatomy", 62.0, 100.0);
        m.skill("EvalInt", 90.1, 100.0);
        m.skill("Magery", 90.1, 100.0);
        m.skill("MagicResist", 115.1, 130.0);
        m.skill("Tactics", 80.1, 100.0);
        m.skill("Wrestling", 80.1, 100.0);
        m.fame = 12500;
        m.karma = -12500;
        m.armor = 50;
    });
});

mobile("FireGargoyle", |c| {
    c.ai = "Mage";
    c.corpse_name = "a charred corpse";
    c.treasure_map_level = 1;
    c.meat = 1;
    c.can_fly = true;
    c.loot = ["Rich", "Gems"];
    c.monster_abilities = ["FireBreath"];
    c.on_create(|m| {
        m.name = random_name("fire gargoyle");
        m.body = 130;
        m.sound = 0x174;
        m.str = 351..=400;
        m.dex = 126..=145;
        m.int = 226..=250;
        m.hits = 211..=240;
        m.damage = 7..=14;
        m.damage_type("Physical", 20);
        m.damage_type("Fire", 80);
        m.resist("Physical", 30, 35);
        m.resist("Fire", 50, 60);
        m.resist("Poison", 20, 30);
        m.resist("Energy", 20, 30);
        m.skill("Anatomy", 75.1, 85.0);
        m.skill("EvalInt", 90.1, 105.0);
        m.skill("Magery", 90.1, 105.0);
        m.skill("Meditation", 90.1, 105.0);
        m.skill("MagicResist", 90.1, 105.0);
        m.skill("Tactics", 80.1, 100.0);
        m.skill("Wrestling", 40.1, 80.0);
        m.fame = 3500;
        m.karma = -3500;
        m.armor = 32;
    });
});

mobile("Gargoyle", |c| {
    c.ai = "Mage";
    c.corpse_name = "a gargoyle corpse";
    c.name = "a gargoyle";
    c.can_fly = true;
    c.treasure_map_level = 1;
    c.meat = 1;
    c.loot = ["Average", "MedScrolls", ["Gems", #{ min: 1, max: 4 }]];
    c.on_create(|m| {
        m.body = 4;
        m.sound = 0x174;
        m.str = 146..=175;
        m.dex = 76..=95;
        m.int = 81..=105;
        m.hits = 88..=105;
        m.damage = 7..=14;
        m.damage_type("Physical", 100);
        m.resist("Physical", 30, 35);
        m.resist("Fire", 25, 35);
        m.resist("Cold", 5, 10);
        m.resist("Poison", 15, 25);
        m.skill("EvalInt", 70.1, 85.0);
        m.skill("Magery", 70.1, 85.0);
        m.skill("MagicResist", 70.1, 85.0);
        m.skill("Tactics", 50.1, 70.0);
        m.skill("Wrestling", 40.1, 80.0);
        m.fame = 3500;
        m.karma = -3500;
        m.armor = 32;
        m.pack = [#{ chance: 0.025, item: "Server.Items.GargoylesPickaxe" }];
    });
});

mobile("GargoyleDestroyer", |c| {
    c.ai = "Mage";
    c.corpse_name = "a gargoyle corpse";
    c.name = "a gargoyle destroyer";
    c.meat = 1;
    c.can_fly = true;
    c.loot = ["FilthyRich", "Rich", "MedScrolls", ["Gems", 2]];
    c.bard_immune = #{ atLeast: "AOS", then: false, "else": true };
    c.monster_abilities = ["ThrowHatchetCounter"];
    c.on_create(|m| {
        m.body = 755;
        m.sound = 0x174;
        m.str = 760..=850;
        m.dex = 102..=150;
        m.int = 152..=200;
        m.hits = 482..=485;
        m.damage = 7..=14;
        m.resist("Physical", 40, 60);
        m.resist("Fire", 60, 70);
        m.resist("Cold", 15, 25);
        m.resist("Poison", 15, 25);
        m.resist("Energy", 15, 25);
        m.skill("Wrestling", 90.1, 100.0);
        m.skill("Tactics", 90.1, 100.0);
        m.skill("MagicResist", 120.4, 160.0);
        m.skill("Anatomy", 50.5, 100.0);
        m.skill("Swords", 90.1, 100.0);
        m.skill("Macing", 90.1, 100.0);
        m.skill("Fencing", 90.1, 100.0);
        m.skill("Magery", 90.1, 100.0);
        m.skill("EvalInt", 90.1, 100.0);
        m.skill("Meditation", 90.1, 100.0);
        m.fame = 10000;
        m.karma = -10000;
        m.armor = 50;
        m.pack = [#{ chance: 0.2, item: "Server.Items.GargoylesPickaxe" }];
    });
});

mobile("GargoyleEnforcer", |c| {
    c.ai = "Mage";
    c.corpse_name = "a gargoyle corpse";
    c.name = "a gargoyle enforcer";
    c.can_fly = true;
    c.meat = 1;
    c.loot = ["Rich", "MedScrolls"];
    c.weapon_ability = "WhirlwindAttack";
    c.on_create(|m| {
        m.body = 754;
        m.sound = 0x174;
        m.str = 760..=850;
        m.dex = 102..=150;
        m.int = 152..=200;
        m.hits = 482..=485;
        m.damage = 7..=14;
        m.resist("Physical", 40, 60);
        m.resist("Fire", 50, 60);
        m.resist("Cold", 20, 30);
        m.resist("Poison", 25, 35);
        m.resist("Energy", 15, 25);
        m.skill("MagicResist", 120.1, 130.0);
        m.skill("Tactics", 70.1, 80.0);
        m.skill("Wrestling", 80.1, 90.0);
        m.skill("Swords", 80.1, 90.0);
        m.skill("Anatomy", 70.1, 80.0);
        m.skill("Magery", 80.1, 90.0);
        m.skill("EvalInt", 70.3, 100.0);
        m.skill("Meditation", 70.3, 100.0);
        m.fame = 5000;
        m.karma = -5000;
        m.armor = 50;
        m.pack = [#{ chance: 0.2, item: "Server.Items.GargoylesPickaxe" }];
    });
});

mobile("Gazer", |c| {
    c.ai = "Mage";
    c.corpse_name = "a gazer corpse";
    c.name = "a gazer";
    c.treasure_map_level = 1;
    c.meat = 1;
    c.loot = ["Average", "Potions"];
    c.on_create(|m| {
        m.body = 22;
        m.sound = 0x179;
        m.str = 96..=125;
        m.dex = 86..=105;
        m.int = 141..=165;
        m.hits = 58..=75;
        m.damage = 5..=10;
        m.damage_type("Physical", 100);
        m.resist("Physical", 35, 40);
        m.resist("Fire", 40, 50);
        m.resist("Cold", 20, 30);
        m.resist("Poison", 10, 20);
        m.resist("Energy", 20, 30);
        m.skill("EvalInt", 50.1, 65.0);
        m.skill("Magery", 50.1, 65.0);
        m.skill("MagicResist", 60.1, 75.0);
        m.skill("Tactics", 50.1, 70.0);
        m.skill("Wrestling", 50.1, 70.0);
        m.fame = 3500;
        m.karma = -3500;
        m.armor = 36;
        m.pack = [#{ item: "Server.Items.Nightshade", amount: 4 }];
    });
});

mobile("IceFiend", |c| {
    c.ai = "Mage";
    c.corpse_name = "an ice fiend corpse";
    c.name = "an ice fiend";
    c.treasure_map_level = 4;
    c.meat = 1;
    c.can_fly = true;
    c.loot = ["FilthyRich", "Average", ["MedScrolls", 2]];
    c.on_create(|m| {
        m.body = 43;
        m.sound = 0x165;
        m.str = 376..=405;
        m.dex = 176..=195;
        m.int = 201..=225;
        m.hits = 226..=243;
        m.damage = 8..=19;
        m.skill("EvalInt", 80.1, 90.0);
        m.skill("Magery", 80.1, 90.0);
        m.skill("MagicResist", 75.1, 85.0);
        m.skill("Tactics", 80.1, 90.0);
        m.skill("Wrestling", 80.1, 100.0);
        m.resist("Physical", 55, 65);
        m.resist("Fire", 10, 20);
        m.resist("Cold", 60, 70);
        m.resist("Poison", 20, 30);
        m.resist("Energy", 30, 40);
        m.fame = 18000;
        m.karma = -18000;
        m.armor = 60;
    });
});

mobile("Imp", |c| {
    c.ai = "Mage";
    c.corpse_name = "an imp corpse";
    c.name = "an imp";
    c.meat = 1;
    c.hides = 6;
    c.can_fly = true;
    c.loot = ["Meager", ["MedScrolls", 2]];
    c.hide_type = "Spined";
    c.favorite_food = ["Meat"];
    c.pack_instinct = ["Daemon"];
    c.on_create(|m| {
        m.body = 74;
        m.sound = 0x1A6;
        m.str = 91..=115;
        m.dex = 61..=80;
        m.int = 86..=105;
        m.hits = 55..=70;
        m.damage = 10..=14;
        m.damage_type("Physical", 0);
        m.damage_type("Fire", 50);
        m.damage_type("Poison", 50);
        m.resist("Physical", 25, 35);
        m.resist("Fire", 40, 50);
        m.resist("Cold", 20, 30);
        m.resist("Poison", 30, 40);
        m.resist("Energy", 30, 40);
        m.skill("EvalInt", 20.1, 30.0);
        m.skill("Magery", 90.1, 100.0);
        m.skill("MagicResist", 30.1, 50.0);
        m.skill("Tactics", 42.1, 50.0);
        m.skill("Wrestling", 40.1, 44.0);
        m.fame = 2500;
        m.karma = -2500;
        m.armor = 30;
        m.tamable = true;
        m.control_slots = 2;
        m.min_tame_skill = 83.1;
    });
});

mobile("Lich", |c| {
    c.ai = "Mage";
    c.corpse_name = "a lich's corpse";
    c.name = "a lich";
    c.can_rummage_corpses = true;
    c.bleed_immune = true;
    c.poison_immune = 4;
    c.treasure_map_level = 3;
    c.loot = ["Rich", ["MedScrolls", 2]];
    c.opposition = "FeyAndUndead";
    c.on_create(|m| {
        m.body = 24;
        m.sound = 0x3E9;
        m.str = 171..=200;
        m.dex = 126..=145;
        m.int = 276..=305;
        m.hits = 103..=120;
        m.damage = 24..=26;
        m.damage_type("Physical", 10);
        m.damage_type("Cold", 40);
        m.damage_type("Energy", 50);
        m.resist("Physical", 40, 60);
        m.resist("Fire", 20, 30);
        m.resist("Cold", 50, 60);
        m.resist("Poison", 55, 65);
        m.resist("Energy", 40, 50);
        m.skill("Necromancy", 89.0, 99.1);
        m.skill("SpiritSpeak", 90.0, 99.0);
        m.skill("EvalInt", 100.0);
        m.skill("Magery", 70.1, 80.0);
        m.skill("Meditation", 85.1, 95.0);
        m.skill("MagicResist", 80.1, 100.0);
        m.skill("Tactics", 70.1, 90.0);
        m.fame = 8000;
        m.karma = -8000;
        m.armor = 50;
        m.pack = [#{ item: "Server.Items.GnarledStaff" }, #{ necroReg: [17, 24] }];
    });
});

mobile("LichLord", |c| {
    c.ai = "Mage";
    c.corpse_name = "a lich's corpse";
    c.name = "a lich lord";
    c.can_rummage_corpses = true;
    c.bleed_immune = true;
    c.poison_immune = 4;
    c.treasure_map_level = 4;
    c.loot = ["FilthyRich", ["MedScrolls", 2]];
    c.opposition = "FeyAndUndead";
    c.monster_abilities = ["SummonSkeletonsCounter"];
    c.on_create(|m| {
        m.body = 79;
        m.sound = 0x19C;
        m.str = 416..=505;
        m.dex = 146..=165;
        m.int = 566..=655;
        m.hits = 250..=303;
        m.damage = 11..=13;
        m.damage_type("Physical", 0);
        m.damage_type("Cold", 60);
        m.damage_type("Energy", 40);
        m.resist("Physical", 40, 50);
        m.resist("Fire", 30, 40);
        m.resist("Cold", 50, 60);
        m.resist("Poison", 50, 60);
        m.resist("Energy", 40, 50);
        m.skill("Necromancy", 90.0, 110.0);
        m.skill("SpiritSpeak", 90.0, 110.0);
        m.skill("EvalInt", 90.1, 100.0);
        m.skill("Magery", 90.1, 100.0);
        m.skill("MagicResist", 150.5, 200.0);
        m.skill("Tactics", 50.1, 70.0);
        m.skill("Wrestling", 60.1, 80.0);
        m.fame = 18000;
        m.karma = -18000;
        m.armor = 50;
        m.pack = [#{ item: "Server.Items.GnarledStaff" }, #{ necroReg: [12, 40] }];
    });
});

mobile("OrcishMage", |c| {
    c.ai = "Mage";
    c.corpse_name = "a glowing orc corpse";
    c.name = "an orcish mage";
    c.can_rummage_corpses = true;
    c.treasure_map_level = 1;
    c.meat = 1;
    c.loot = ["Average", "LowScrolls"];
    c.opposition = "SavagesAndOrcs";
    c.speech = "Orc";
    c.on_create(|m| {
        m.body = 140;
        m.sound = 0x45A;
        m.str = 116..=150;
        m.dex = 91..=115;
        m.int = 161..=185;
        m.hits = 70..=90;
        m.damage = 4..=14;
        m.damage_type("Physical", 100);
        m.resist("Physical", 25, 35);
        m.resist("Fire", 30, 40);
        m.resist("Cold", 20, 30);
        m.resist("Poison", 30, 40);
        m.resist("Energy", 30, 40);
        m.skill("EvalInt", 60.1, 72.5);
        m.skill("Magery", 60.1, 72.5);
        m.skill("MagicResist", 60.1, 75.0);
        m.skill("Tactics", 50.1, 65.0);
        m.skill("Wrestling", 40.1, 50.0);
        m.fame = 3000;
        m.karma = -3000;
        m.armor = 30;
        m.pack = [#{ reg: 6 }, #{ chance: 0.05, item: "Server.Items.OrcishKinMask" }];
    });
});

mobile("RatmanMage", |c| {
    c.ai = "Mage";
    c.corpse_name = "a glowing ratman corpse";
    c.can_rummage_corpses = true;
    c.meat = 1;
    c.hides = 8;
    c.loot = ["Rich", "LowScrolls"];
    c.hide_type = "Spined";
    c.speech = "Ratman";
    c.on_create(|m| {
        m.name = random_name("ratman");
        m.body = 143;
        m.sound = 0x1B5;
        m.str = 146..=180;
        m.dex = 101..=130;
        m.int = 186..=210;
        m.hits = 88..=108;
        m.damage = 7..=14;
        m.damage_type("Physical", 100);
        m.resist("Physical", 40, 45);
        m.resist("Fire", 10, 20);
        m.resist("Cold", 10, 20);
        m.resist("Poison", 10, 20);
        m.resist("Energy", 10, 20);
        m.skill("EvalInt", 70.1, 80.0);
        m.skill("Magery", 70.1, 80.0);
        m.skill("MagicResist", 65.1, 90.0);
        m.skill("Tactics", 50.1, 75.0);
        m.skill("Wrestling", 50.1, 75.0);
        m.fame = 7500;
        m.karma = -7500;
        m.armor = 44;
        m.pack = [#{ reg: 6 }, #{ chance: 0.02, statue: 1 }];
    });
});

mobile("Shade", |c| {
    c.ai = "Mage";
    c.corpse_name = "a ghostly corpse";
    c.name = "a shade";
    c.bleed_immune = true;
    c.poison_immune = 4;
    c.loot = "Meager";
    c.opposition = "FeyAndUndead";
    c.on_create(|m| {
        m.body = 26;
        m.hue = 0x4001;
        m.sound = 0x482;
        m.str = 76..=100;
        m.dex = 76..=95;
        m.int = 36..=60;
        m.hits = 46..=60;
        m.damage = 7..=11;
        m.damage_type("Physical", 50);
        m.damage_type("Cold", 50);
        m.resist("Physical", 25, 30);
        m.resist("Cold", 15, 25);
        m.resist("Poison", 10, 20);
        m.skill("EvalInt", 55.1, 70.0);
        m.skill("Magery", 55.1, 70.0);
        m.skill("MagicResist", 55.1, 70.0);
        m.skill("Tactics", 45.1, 60.0);
        m.skill("Wrestling", 45.1, 55.0);
        m.fame = 4000;
        m.karma = -4000;
        m.armor = 28;
        m.pack = [#{ reg: 10 }];
    });
});

mobile("SkeletalMage", |c| {
    c.ai = "Mage";
    c.corpse_name = "a skeletal corpse";
    c.name = "a skeletal mage";
    c.bleed_immune = true;
    c.poison_immune = 1;
    c.loot = ["Average", "LowScrolls", "Potions"];
    c.opposition = "FeyAndUndead";
    c.on_create(|m| {
        m.body = 148;
        m.sound = 0x1C3;
        m.str = 76..=100;
        m.dex = 56..=75;
        m.int = 186..=210;
        m.hits = 46..=60;
        m.damage = 3..=7;
        m.damage_type("Physical", 100);
        m.resist("Physical", 35, 40);
        m.resist("Fire", 20, 30);
        m.resist("Cold", 50, 60);
        m.resist("Poison", 20, 30);
        m.resist("Energy", 30, 40);
        m.skill("EvalInt", 60.1, 70.0);
        m.skill("Magery", 60.1, 70.0);
        m.skill("MagicResist", 55.1, 70.0);
        m.skill("Tactics", 45.1, 60.0);
        m.skill("Wrestling", 45.1, 55.0);
        m.skill("Necromancy", 89.0, 99.1);
        m.skill("SpiritSpeak", 90.0, 99.0);
        m.fame = 3000;
        m.karma = -3000;
        m.armor = 38;
        m.pack = [#{ reg: 3 }, #{ necroReg: [3, 10] }, #{ item: "Server.Items.Bone" }];
    });
});

mobile("Spectre", |c| {
    c.ai = "Mage";
    c.corpse_name = "a ghostly corpse";
    c.name = "a spectre";
    c.bleed_immune = true;
    c.poison_immune = 4;
    c.loot = "Meager";
    c.opposition = "FeyAndUndead";
    c.on_create(|m| {
        m.body = 26;
        m.hue = 0x4001;
        m.sound = 0x482;
        m.str = 76..=100;
        m.dex = 76..=95;
        m.int = 36..=60;
        m.hits = 46..=60;
        m.damage = 7..=11;
        m.damage_type("Physical", 50);
        m.damage_type("Cold", 50);
        m.resist("Physical", 25, 30);
        m.resist("Cold", 15, 25);
        m.resist("Poison", 10, 20);
        m.skill("EvalInt", 55.1, 70.0);
        m.skill("Magery", 55.1, 70.0);
        m.skill("MagicResist", 55.1, 70.0);
        m.skill("Tactics", 45.1, 60.0);
        m.skill("Wrestling", 45.1, 55.0);
        m.fame = 4000;
        m.karma = -4000;
        m.armor = 28;
        m.pack = [#{ reg: 10 }];
    });
});

mobile("Spellbinder", |c| {
    c.ai = "Mage";
    c.fight_mode = "Aggressor";
    c.corpse_name = "a ghostly corpse";
    c.name = "a spectral spellbinder";
    c.bleed_immune = true;
    c.poison_immune = 1;
    c.opposition = "FeyAndUndead";
    c.loot = [["Meager", 1], #{ pack: "Weapon" }];
    c.on_create(|m| {
        m.bodies = [26, 50, 56];
        m.sound = 0x482;
        m.str = 46..=70;
        m.dex = 47..=65;
        m.int = 187..=210;
        m.hits = 36..=50;
        m.damage = 3..=6;
        m.damage_type("Physical", 50);
        m.damage_type("Cold", 50);
        m.resist("Physical", 25, 30);
        m.resist("Cold", 15, 25);
        m.resist("Poison", 10, 20);
        m.skill("MagicResist", 35.1, 50.0);
        m.skill("Tactics", 35.1, 50.0);
        m.skill("Wrestling", 35.1, 45.0);
        m.fame = 2500;
        m.karma = -2500;
        m.armor = 28;
    });
});

mobile("Succubus", |c| {
    c.ai = "Mage";
    c.corpse_name = "a succubus corpse";
    c.name = "a succubus";
    c.meat = 1;
    c.treasure_map_level = 5;
    c.loot = [["FilthyRich", 2], ["MedScrolls", 2]];
    c.monster_abilities = ["DrainLifeAreaAttack"];
    c.on_create(|m| {
        m.body = 149;
        m.sound = 0x4B0;
        m.str = 488..=620;
        m.dex = 121..=170;
        m.int = 498..=657;
        m.hits = 312..=353;
        m.damage = 18..=28;
        m.damage_type("Physical", 75);
        m.damage_type("Energy", 25);
        m.resist("Physical", 80, 90);
        m.resist("Fire", 70, 80);
        m.resist("Cold", 40, 50);
        m.resist("Poison", 50, 60);
        m.resist("Energy", 50, 60);
        m.skill("EvalInt", 90.1, 100.0);
        m.skill("Magery", 99.1, 100.0);
        m.skill("Meditation", 90.1, 100.0);
        m.skill("MagicResist", 100.5, 150.0);
        m.skill("Tactics", 80.1, 90.0);
        m.skill("Wrestling", 80.1, 90.0);
        m.fame = 24000;
        m.karma = -24000;
        m.armor = 80;
    });
});

mobile("Wraith", |c| {
    c.ai = "Mage";
    c.corpse_name = "a ghostly corpse";
    c.name = "a wraith";
    c.bleed_immune = true;
    c.poison_immune = 4;
    c.loot = "Meager";
    c.opposition = "FeyAndUndead";
    c.on_create(|m| {
        m.body = 26;
        m.hue = 0x4001;
        m.sound = 0x482;
        m.str = 76..=100;
        m.dex = 76..=95;
        m.int = 36..=60;
        m.hits = 46..=60;
        m.damage = 7..=11;
        m.damage_type("Physical", 50);
        m.damage_type("Cold", 50);
        m.resist("Physical", 25, 30);
        m.resist("Cold", 15, 25);
        m.resist("Poison", 10, 20);
        m.skill("EvalInt", 55.1, 70.0);
        m.skill("Magery", 55.1, 70.0);
        m.skill("MagicResist", 55.1, 70.0);
        m.skill("Tactics", 45.1, 60.0);
        m.skill("Wrestling", 45.1, 55.0);
        m.fame = 4000;
        m.karma = -4000;
        m.armor = 28;
        m.pack = [#{ reg: 10 }];
    });
});

mobile("ArcticOgreLord", |c| {
    c.corpse_name = "a frozen ogre lord's corpse";
    c.name = "an arctic ogre lord";
    c.poison_immune = 1;
    c.treasure_map_level = 3;
    c.loot = ["FilthyRich", "Rich"];
    c.on_create(|m| {
        m.body = 135;
        m.sound = 0x1AB;
        m.str = 767..=945;
        m.dex = 66..=75;
        m.int = 46..=70;
        m.hits = 476..=552;
        m.damage = 20..=25;
        m.damage_type("Physical", 30);
        m.damage_type("Cold", 70);
        m.resist("Physical", 45, 55);
        m.resist("Cold", 60, 70);
        m.resist("Poison", 100);
        m.resist("Energy", 40, 50);
        m.skill("MagicResist", 125.1, 140.0);
        m.skill("Tactics", 90.1, 100.0);
        m.skill("Wrestling", 90.1, 100.0);
        m.fame = 15000;
        m.karma = -15000;
        m.armor = 50;
        m.pack = [#{ item: "Server.Items.Club" }];
    });
});

mobile("ChaosDaemon", |c| {
    c.corpse_name = "a chaos daemon corpse";
    c.name = "a chaos daemon";
    c.loot = ["Average", "Meager"];
    c.weapon_ability = "CrushingBlow";
    c.on_create(|m| {
        m.body = 792;
        m.sound = 0x3E9;
        m.str = 106..=130;
        m.dex = 171..=200;
        m.int = 56..=80;
        m.hits = 91..=110;
        m.damage = 12..=17;
        m.damage_type("Physical", 85);
        m.damage_type("Fire", 15);
        m.resist("Physical", 50, 60);
        m.resist("Fire", 60, 70);
        m.resist("Cold", 40, 50);
        m.resist("Poison", 20, 30);
        m.resist("Energy", 20, 30);
        m.skill("MagicResist", 85.1, 95.0);
        m.skill("Tactics", 70.1, 80.0);
        m.skill("Wrestling", 95.1, 100.0);
        m.fame = 3000;
        m.karma = -4000;
        m.armor = 15;
    });
});

mobile("Cyclops", |c| {
    c.corpse_name = "a cyclopean corpse";
    c.name = "a cyclopean warrior";
    c.meat = 4;
    c.treasure_map_level = 3;
    c.loot = ["Rich", "Average"];
    c.on_create(|m| {
        m.body = 75;
        m.sound = 0x25C;
        m.str = 336..=385;
        m.dex = 96..=115;
        m.int = 31..=55;
        m.hits = 202..=231;
        m.mana = 0;
        m.damage = 7..=23;
        m.damage_type("Physical", 100);
        m.resist("Physical", 45, 50);
        m.resist("Fire", 30, 40);
        m.resist("Cold", 25, 35);
        m.resist("Poison", 30, 40);
        m.resist("Energy", 30, 40);
        m.skill("MagicResist", 60.3, 105.0);
        m.skill("Tactics", 80.1, 100.0);
        m.skill("Wrestling", 80.1, 90.0);
        m.fame = 4500;
        m.karma = -4500;
        m.armor = 48;
    });
});

mobile("Doppleganger", |c| {
    c.corpse_name = "a doppleganger corpse";
    c.name = "a doppleganger";
    c.hides = 6;
    c.meat = 1;
    c.loot = "Average";
    c.on_create(|m| {
        m.body = 777;
        m.sound = 0x451;
        m.str = 81..=110;
        m.dex = 56..=75;
        m.int = 81..=105;
        m.hits = 101..=120;
        m.damage = 8..=12;
        m.resist("Physical", 50, 60);
        m.resist("Fire", 10, 20);
        m.resist("Cold", 40, 50);
        m.resist("Poison", 50, 60);
        m.resist("Energy", 30, 40);
        m.skill("MagicResist", 75.1, 85.0);
        m.skill("Tactics", 70.1, 80.0);
        m.skill("Wrestling", 80.1, 90.0);
        m.fame = 1000;
        m.karma = -1000;
        m.armor = 55;
    });
});

mobile("EnslavedGargoyle", |c| {
    c.corpse_name = "an enslaved gargoyle corpse";
    c.name = "an enslaved gargoyle";
    c.meat = 1;
    c.treasure_map_level = 1;
    c.loot = [["Average", 2], "Gems"];
    c.on_create(|m| {
        m.body = 753;
        m.sound = 0x174;
        m.str = 302..=360;
        m.dex = 76..=95;
        m.int = 81..=105;
        m.hits = 186..=212;
        m.damage = 7..=14;
        m.resist("Physical", 30, 40);
        m.resist("Fire", 50, 70);
        m.resist("Cold", 15, 25);
        m.resist("Poison", 25, 30);
        m.resist("Energy", 25, 30);
        m.skill("MagicResist", 70.1, 85.0);
        m.skill("Tactics", 50.1, 70.0);
        m.skill("Wrestling", 40.1, 80.0);
        m.fame = 3500;
        m.karma = 0;
        m.armor = 35;
        m.pack = [#{ chance: 0.2, item: "Server.Items.GargoylesPickaxe" }];
    });
});

mobile("Ettin", |c| {
    c.corpse_name = "an ettins corpse";
    c.name = "an ettin";
    c.can_rummage_corpses = true;
    c.treasure_map_level = 1;
    c.meat = 4;
    c.loot = ["Meager", "Average", "Potions"];
    c.on_create(|m| {
        m.body = 18;
        m.sound = 0x16F;
        m.str = 136..=165;
        m.dex = 56..=75;
        m.int = 31..=55;
        m.hits = 82..=99;
        m.damage = 7..=17;
        m.damage_type("Physical", 100);
        m.resist("Physical", 35, 40);
        m.resist("Fire", 15, 25);
        m.resist("Cold", 40, 50);
        m.resist("Poison", 15, 25);
        m.resist("Energy", 15, 25);
        m.skill("MagicResist", 40.1, 55.0);
        m.skill("Tactics", 50.1, 70.0);
        m.skill("Wrestling", 50.1, 60.0);
        m.fame = 3000;
        m.karma = -3000;
        m.armor = 38;
    });
});

mobile("FrostTroll", |c| {
    c.corpse_name = "a frost troll corpse";
    c.name = "a frost troll";
    c.meat = 2;
    c.treasure_map_level = 1;
    c.loot = ["Average", "Gems"];
    c.on_create(|m| {
        m.body = 55;
        m.sound = 0x1CD;
        m.str = 227..=265;
        m.dex = 66..=85;
        m.int = 46..=70;
        m.hits = 140..=156;
        m.damage = 14..=20;
        m.damage_type("Physical", 75);
        m.damage_type("Cold", 25);
        m.resist("Physical", 45, 55);
        m.resist("Cold", 40, 50);
        m.resist("Poison", 5, 10);
        m.resist("Energy", 5, 10);
        m.skill("MagicResist", 65.1, 80.0);
        m.skill("Tactics", 80.1, 100.0);
        m.skill("Wrestling", 80.1, 100.0);
        m.fame = 4000;
        m.karma = -4000;
        m.armor = 50;
        m.pack = [#{ item: "Server.Items.DoubleAxe" }];
    });
});

mobile("GazerLarva", |c| {
    c.corpse_name = "a gazer larva corpse";
    c.name = "a gazer larva";
    c.meat = 1;
    c.loot = "Poor";
    c.on_create(|m| {
        m.body = 778;
        m.sound = 0x179;
        m.str = 76..=100;
        m.dex = 51..=75;
        m.int = 56..=80;
        m.hits = 36..=47;
        m.damage = 2..=9;
        m.damage_type("Physical", 100);
        m.resist("Physical", 15, 25);
        m.skill("MagicResist", 70.0);
        m.skill("Tactics", 70.0);
        m.skill("Wrestling", 70.0);
        m.fame = 900;
        m.karma = -900;
        m.armor = 25;
        m.pack = [#{ item: "Server.Items.Nightshade", amount: [2, 3] }];
    });
});

mobile("GreaterMongbat", |c| {
    c.corpse_name = "a mongbat corpse";
    c.name = "a greater mongbat";
    c.meat = 1;
    c.hides = 6;
    c.loot = "Poor";
    c.favorite_food = ["Meat"];
    c.on_create(|m| {
        m.body = 39;
        m.sound = 0x1A6;
        m.str = 56..=80;
        m.dex = 61..=80;
        m.int = 26..=50;
        m.hits = 34..=48;
        m.stam = 61..=80;
        m.mana = 26..=50;
        m.damage = 5..=7;
        m.damage_type("Physical", 100);
        m.resist("Physical", 15, 25);
        m.skill("MagicResist", 15.1, 30.0);
        m.skill("Tactics", 35.1, 50.0);
        m.skill("Wrestling", 20.1, 35.0);
        m.fame = 450;
        m.karma = -450;
        m.armor = 10;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = 71.1;
    });
});

mobile("Juggernaut", |c| {
    c.corpse_name = "a juggernaut corpse";
    c.name = "a blackthorn juggernaut";
    c.always_murderer = true;
    c.bleed_immune = true;
    c.poison_immune = 4;
    c.meat = 1;
    c.treasure_map_level = 5;
    c.loot = ["Rich", "Gems"];
    c.bard_immune = #{ atLeast: "AOS", then: false, "else": true };
    c.monster_abilities = ["ColossalBlow"];
    c.death_sound = 1059;
    c.attack_sound = 571;
    c.hurt_sound = 320;
    c.on_create(|m| {
        m.body = 768;
        m.str = 301..=400;
        m.dex = 51..=70;
        m.int = 51..=100;
        m.hits = 181..=240;
        m.damage = 12..=19;
        m.damage_type("Physical", 50);
        m.damage_type("Fire", 25);
        m.damage_type("Energy", 25);
        m.resist("Physical", 65, 75);
        m.resist("Fire", 35, 45);
        m.resist("Cold", 35, 45);
        m.resist("Poison", 15, 25);
        m.resist("Energy", 10, 20);
        m.skill("Anatomy", 90.1, 100.0);
        m.skill("MagicResist", 140.1, 150.0);
        m.skill("Tactics", 90.1, 100.0);
        m.skill("Wrestling", 90.1, 100.0);
        m.fame = 12000;
        m.karma = -12000;
        m.armor = 70;
        m.pack = [
            #{ chance: 0.1, item: "Server.Items.PowerCrystal" },
            #{ chance: 0.4, item: "Server.Items.ClockworkAssembly" },
        ];
    });
});

mobile("Moloch", |c| {
    c.corpse_name = "a moloch corpse";
    c.name = "a moloch";
    c.poison_immune = 1;
    c.loot = "Rich";
    c.weapon_ability = "ConcussionBlow";
    c.on_create(|m| {
        m.body = 785;
        m.sound = 0x300;
        m.str = 331..=360;
        m.dex = 66..=85;
        m.int = 41..=65;
        m.hits = 171..=200;
        m.damage = 15..=23;
        m.resist("Physical", 60, 70);
        m.resist("Fire", 60, 70);
        m.resist("Cold", 40, 50);
        m.resist("Poison", 20, 30);
        m.resist("Energy", 20, 30);
        m.skill("MagicResist", 65.1, 75.0);
        m.skill("Tactics", 75.1, 90.0);
        m.skill("Wrestling", 70.1, 90.0);
        m.fame = 7500;
        m.karma = -7500;
        m.armor = 32;
    });
});

mobile("Mongbat", |c| {
    c.corpse_name = "a mongbat corpse";
    c.name = "a mongbat";
    c.can_fly = true;
    c.meat = 1;
    c.loot = "Poor";
    c.favorite_food = ["Meat"];
    c.on_create(|m| {
        m.body = 39;
        m.sound = 0x1A6;
        m.str = 6..=10;
        m.dex = 26..=38;
        m.int = 6..=14;
        m.hits = 4..=6;
        m.mana = 0;
        m.damage = 1..=2;
        m.damage_type("Physical", 100);
        m.resist("Physical", 5, 10);
        m.skill("MagicResist", 5.1, 14.0);
        m.skill("Tactics", 5.1, 10.0);
        m.skill("Wrestling", 5.1, 10.0);
        m.fame = 150;
        m.karma = -150;
        m.armor = 10;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = -18.9;
    });
});

mobile("Ogre", |c| {
    c.corpse_name = "an ogre corpse";
    c.name = "an ogre";
    c.can_rummage_corpses = true;
    c.treasure_map_level = 1;
    c.meat = 2;
    c.loot = ["Average", "Potions"];
    c.on_create(|m| {
        m.body = 1;
        m.sound = 0x1AB;
        m.str = 166..=195;
        m.dex = 46..=65;
        m.int = 46..=70;
        m.hits = 100..=117;
        m.mana = 0;
        m.damage = 9..=11;
        m.damage_type("Physical", 100);
        m.resist("Physical", 30, 35);
        m.resist("Fire", 15, 25);
        m.resist("Cold", 15, 25);
        m.resist("Poison", 15, 25);
        m.resist("Energy", 25);
        m.skill("MagicResist", 55.1, 70.0);
        m.skill("Tactics", 60.1, 70.0);
        m.skill("Wrestling", 70.1, 80.0);
        m.fame = 3000;
        m.karma = -3000;
        m.armor = 32;
        m.pack = [#{ item: "Server.Items.Club" }];
    });
});

mobile("OgreLord", |c| {
    c.corpse_name = "an ogre lords corpse";
    c.name = "an ogre lord";
    c.can_rummage_corpses = true;
    c.poison_immune = 1;
    c.treasure_map_level = 3;
    c.meat = 2;
    c.loot = [["Rich", 2]];
    c.on_create(|m| {
        m.body = 83;
        m.sound = 0x1AB;
        m.str = 767..=945;
        m.dex = 66..=75;
        m.int = 46..=70;
        m.hits = 476..=552;
        m.damage = 20..=25;
        m.damage_type("Physical", 100);
        m.resist("Physical", 45, 55);
        m.resist("Fire", 30, 40);
        m.resist("Cold", 30, 40);
        m.resist("Poison", 40, 50);
        m.resist("Energy", 40, 50);
        m.skill("MagicResist", 125.1, 140.0);
        m.skill("Tactics", 90.1, 100.0);
        m.skill("Wrestling", 90.1, 100.0);
        m.fame = 15000;
        m.karma = -15000;
        m.armor = 50;
        m.pack = [#{ item: "Server.Items.Club" }];
    });
});

mobile("OrcBomber", |c| {
    c.corpse_name = "an orcish corpse";
    c.name = "an orc bomber";
    c.can_rummage_corpses = true;
    c.loot = ["Average", "Meager"];
    c.opposition = "SavagesAndOrcs";
    c.speech = "Orc";
    c.on_create(|m| {
        m.body = 182;
        m.sound = 0x45A;
        m.str = 147..=215;
        m.dex = 91..=115;
        m.int = 61..=85;
        m.hits = 95..=123;
        m.damage = 1..=8;
        m.damage_type("Physical", 75);
        m.damage_type("Fire", 25);
        m.resist("Physical", 25, 35);
        m.resist("Fire", 30, 40);
        m.resist("Cold", 15, 25);
        m.resist("Poison", 15, 20);
        m.resist("Energy", 25, 30);
        m.skill("MagicResist", 70.1, 85.0);
        m.skill("Swords", 60.1, 85.0);
        m.skill("Tactics", 75.1, 90.0);
        m.skill("Wrestling", 60.1, 85.0);
        m.fame = 2500;
        m.karma = -2500;
        m.armor = 30;
        m.pack = [
            #{ item: "Server.Items.SulfurousAsh", amount: [6, 10] },
            #{ item: "Server.Items.MandrakeRoot", amount: [6, 10] },
            #{ item: "Server.Items.BlackPearl", amount: [6, 10] },
            #{ item: "Server.Items.MortarPestle" },
            #{ item: "Server.Items.LesserExplosionPotion" },
            #{ chance: 0.2, item: "Server.Items.BolaBall" },
        ];
    });
});

mobile("OrcBrute", |c| {
    c.corpse_name = "an orcish corpse";
    c.name = "an orc brute";
    c.poison_immune = 4;
    c.meat = 2;
    c.can_rummage_corpses = true;
    c.loot = ["FilthyRich", "Rich"];
    c.opposition = "SavagesAndOrcs";
    c.auto_dispel = true;
    c.bard_immune = #{ atLeast: "AOS", then: false, "else": true };
    c.on_create(|m| {
        m.body = 189;
        m.sound = 0x45A;
        m.str = 767..=945;
        m.dex = 66..=75;
        m.int = 46..=70;
        m.hits = 476..=552;
        m.damage = 20..=25;
        m.damage_type("Physical", 100);
        m.resist("Physical", 45, 55);
        m.resist("Fire", 40, 50);
        m.resist("Cold", 25, 35);
        m.resist("Poison", 25, 35);
        m.resist("Energy", 25, 35);
        m.skill("Macing", 90.1, 100.0);
        m.skill("MagicResist", 125.1, 140.0);
        m.skill("Tactics", 90.1, 100.0);
        m.skill("Wrestling", 90.1, 100.0);
        m.fame = 15000;
        m.karma = -15000;
        m.armor = 50;
        m.pack = [
            #{ item: "Server.Items.ShadowIronOre", amount: 25, itemId: 6585 },
            #{ item: "Server.Items.IronIngot", amount: 10 },
            #{ chance: 0.05, item: "Server.Items.OrcishKinMask" },
            #{ chance: 0.2, item: "Server.Items.BolaBall" },
        ];
    });
});

mobile("Ratman", |c| {
    c.corpse_name = "a ratman's corpse";
    c.can_rummage_corpses = true;
    c.hides = 8;
    c.loot = "Meager";
    c.hide_type = "Spined";
    c.speech = "Ratman";
    c.on_create(|m| {
        m.name = random_name("ratman");
        m.body = 42;
        m.sound = 0x1B5;
        m.str = 96..=120;
        m.dex = 81..=100;
        m.int = 36..=60;
        m.hits = 58..=72;
        m.damage = 4..=5;
        m.damage_type("Physical", 100);
        m.resist("Physical", 25, 30);
        m.resist("Fire", 10, 20);
        m.resist("Cold", 10, 20);
        m.resist("Poison", 10, 20);
        m.resist("Energy", 10, 20);
        m.skill("MagicResist", 35.1, 60.0);
        m.skill("Tactics", 50.1, 75.0);
        m.skill("Wrestling", 50.1, 75.0);
        m.fame = 1500;
        m.karma = -1500;
        m.armor = 28;
    });
});

mobile("RatmanArcher", |c| {
    c.ai = "Archer";
    c.corpse_name = "a ratman archer corpse";
    c.can_rummage_corpses = true;
    c.hides = 8;
    c.loot = "Rich";
    c.hide_type = "Spined";
    c.speech = "Ratman";
    c.on_create(|m| {
        m.name = random_name("ratman");
        m.body = 142;
        m.sound = 0x1B5;
        m.str = 146..=180;
        m.dex = 101..=130;
        m.int = 116..=140;
        m.hits = 88..=108;
        m.damage = 4..=10;
        m.damage_type("Physical", 100);
        m.resist("Physical", 40, 55);
        m.resist("Fire", 10, 20);
        m.resist("Cold", 10, 20);
        m.resist("Poison", 10, 20);
        m.resist("Energy", 10, 20);
        m.skill("Anatomy", 60.2, 100.0);
        m.skill("Archery", 80.1, 90.0);
        m.skill("MagicResist", 65.1, 90.0);
        m.skill("Tactics", 50.1, 75.0);
        m.skill("Wrestling", 50.1, 75.0);
        m.fame = 6500;
        m.karma = -6500;
        m.armor = 56;
        m.pack = [#{ item: "Server.Items.Bow", equip: true }, #{ item: "Server.Items.Arrow", amount: [50, 70] }];
    });
});

mobile("RestlessSoul", |c| {
    c.corpse_name = "a ghostly corpse";
    c.name = "a restless soul";
    c.bleed_immune = true;
    c.loot = "Poor";
    c.always_attackable = true;
    c.idle_sound = 263;
    c.anger_sound = 447;
    c.death_sound = 253;
    c.on_create(|m| {
        m.body = 970;
        m.hue = 0x453;
        m.str = 26..=40;
        m.dex = 26..=40;
        m.int = 26..=40;
        m.hits = 16..=24;
        m.damage = 1..=10;
        m.damage_type("Physical", 100);
        m.resist("Physical", 15, 25);
        m.resist("Fire", 5, 15);
        m.resist("Cold", 25, 40);
        m.resist("Poison", 5, 10);
        m.resist("Energy", 10, 20);
        m.skill("MagicResist", 20.1, 30.0);
        m.skill("Swords", 20.1, 30.0);
        m.skill("Tactics", 20.1, 30.0);
        m.skill("Wrestling", 20.1, 30.0);
        m.fame = 500;
        m.karma = -500;
        m.armor = 6;
    });
});

mobile("RottingCorpse", |c| {
    c.corpse_name = "a rotting corpse";
    c.name = "a rotting corpse";
    c.bleed_immune = true;
    c.poison_immune = 4;
    c.treasure_map_level = 5;
    c.loot = [["FilthyRich", 2]];
    c.hit_poison = 4;
    c.opposition = "FeyAndUndead";
    c.on_create(|m| {
        m.body = 155;
        m.sound = 0x1D7;
        m.str = 301..=350;
        m.dex = 75;
        m.int = 151..=200;
        m.hits = 1200;
        m.stam = 150;
        m.mana = 0;
        m.damage = 8..=10;
        m.damage_type("Physical", 0);
        m.damage_type("Cold", 50);
        m.damage_type("Poison", 50);
        m.resist("Physical", 35, 45);
        m.resist("Fire", 20, 30);
        m.resist("Cold", 50, 70);
        m.resist("Poison", 40, 50);
        m.resist("Energy", 20, 30);
        m.skill("Poisoning", 120.0);
        m.skill("MagicResist", 250.0);
        m.skill("Tactics", 100.0);
        m.skill("Wrestling", 90.1, 100.0);
        m.fame = 6000;
        m.karma = -6000;
        m.armor = 40;
    });
});

mobile("StoneGargoyle", |c| {
    c.corpse_name = "a gargoyle corpse";
    c.name = "a stone gargoyle";
    c.treasure_map_level = 2;
    c.loot = [["Average", 2], "Gems", "Potions"];
    c.on_create(|m| {
        m.body = 67;
        m.sound = 0x174;
        m.str = 246..=275;
        m.dex = 76..=95;
        m.int = 81..=105;
        m.hits = 148..=165;
        m.damage = 11..=17;
        m.damage_type("Physical", 100);
        m.resist("Physical", 45, 55);
        m.resist("Fire", 20, 30);
        m.resist("Cold", 10, 20);
        m.resist("Poison", 30, 40);
        m.resist("Energy", 30, 40);
        m.skill("MagicResist", 85.1, 100.0);
        m.skill("Tactics", 80.1, 100.0);
        m.skill("Wrestling", 60.1, 100.0);
        m.fame = 4000;
        m.karma = -4000;
        m.armor = 50;
        m.pack = [
            #{ item: "Server.Items.IronIngot", amount: 12 },
            #{ chance: 0.05, item: "Server.Items.GargoylesPickaxe" },
        ];
    });
});

mobile("StrongMongbat", |c| {
    c.corpse_name = "a mongbat corpse";
    c.name = "a mongbat";
    c.meat = 1;
    c.hides = 6;
    c.loot = "Poor";
    c.favorite_food = ["Meat"];
    c.on_create(|m| {
        m.body = 39;
        m.sound = 0x1A6;
        m.str = 6..=10;
        m.dex = 26..=38;
        m.int = 6..=14;
        m.hits = 4..=6;
        m.mana = 0;
        m.damage = 5..=7;
        m.damage_type("Physical", 100);
        m.resist("Physical", 15, 25);
        m.skill("MagicResist", 15.1, 30.0);
        m.skill("Tactics", 35.1, 50.0);
        m.skill("Wrestling", 20.1, 35.0);
        m.fame = 150;
        m.karma = -150;
        m.armor = 10;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = 71.1;
    });
});

mobile("Troll", |c| {
    c.corpse_name = "a troll corpse";
    c.name = "a troll";
    c.can_rummage_corpses = true;
    c.treasure_map_level = 1;
    c.meat = 2;
    c.loot = "Average";
    c.on_create(|m| {
        m.bodies = [53, 54];
        m.sound = 0x1CD;
        m.str = 176..=205;
        m.dex = 46..=65;
        m.int = 46..=70;
        m.hits = 106..=123;
        m.damage = 8..=14;
        m.damage_type("Physical", 100);
        m.resist("Physical", 35, 45);
        m.resist("Fire", 25, 35);
        m.resist("Cold", 15, 25);
        m.resist("Poison", 5, 15);
        m.resist("Energy", 5, 15);
        m.skill("MagicResist", 45.1, 60.0);
        m.skill("Tactics", 50.1, 70.0);
        m.skill("Wrestling", 50.1, 70.0);
        m.fame = 3500;
        m.karma = -3500;
        m.armor = 40;
    });
});

mobile("JukaWarrior", |c| {
    c.corpse_name = "a jukan corpse";
    c.name = "a juka warrior";
    c.always_murderer = true;
    c.can_rummage_corpses = true;
    c.meat = 1;
    c.loot = ["Average", "Meager", "Gems"];
    c.idle_sound = 428;
    c.anger_sound = 461;
    c.hurt_sound = 464;
    c.death_sound = 653;
    c.on_create(|m| {
        m.body = 764;
        m.str = 251..=350;
        m.dex = 61..=80;
        m.int = 101..=150;
        m.hits = 151..=210;
        m.damage = 7..=9;
        m.damage_type("Physical", 100);
        m.resist("Physical", 40, 50);
        m.resist("Fire", 30, 40);
        m.resist("Cold", 25, 35);
        m.resist("Poison", 10, 20);
        m.resist("Energy", 10, 20);
        m.skill("Anatomy", 80.1, 90.0);
        m.skill("Fencing", 80.1, 90.0);
        m.skill("Macing", 80.1, 90.0);
        m.skill("MagicResist", 120.1, 130.0);
        m.skill("Swords", 80.1, 90.0);
        m.skill("Tactics", 80.1, 90.0);
        m.skill("Wrestling", 80.1, 90.0);
        m.fame = 10000;
        m.karma = -10000;
        m.armor = 22;
        m.pack = [#{ chance: 0.1, item: "Server.Items.ArcaneGem" }];
    });
});

mobile("MeerWarrior", |c| {
    c.fight_mode = "Evil";
    c.corpse_name = "a meer corpse";
    c.name = "a meer warrior";
    c.can_rummage_corpses = true;
    c.loot = "Meager";
    c.initial_innocent = true;
    c.bard_immune = #{ atLeast: "AOS", then: false, "else": true };
    c.hurt_sound = 342;
    c.death_sound = 348;
    c.on_create(|m| {
        m.body = 771;
        m.str = 86..=100;
        m.dex = 186..=200;
        m.int = 86..=100;
        m.hits = 52..=60;
        m.damage = 12..=19;
        m.damage_type("Physical", 100);
        m.resist("Physical", 35, 45);
        m.resist("Fire", 5, 15);
        m.resist("Cold", 30, 40);
        m.resist("Poison", 25, 35);
        m.resist("Energy", 25, 35);
        m.skill("MagicResist", 91.0, 100.0);
        m.skill("Tactics", 91.0, 100.0);
        m.skill("Wrestling", 91.0, 100.0);
        m.armor = 22;
        m.fame = 2000;
        m.karma = 5000;
    });
});

mobile("RagingGrizzlyBear", |c| {
    c.ai = "Animal";
    c.fight_mode = "Aggressor";
    c.corpse_name = "a grizzly bear corpse";
    c.name = "a raging grizzly bear";
    c.meat = 4;
    c.hides = 32;
    c.pack_instinct = ["Bear"];
    c.on_create(|m| {
        m.body = 212;
        m.sound = 0xA3;
        m.str = 1251..=1550;
        m.dex = 801..=1050;
        m.int = 151..=400;
        m.hits = 751..=930;
        m.mana = 0;
        m.damage = 18..=23;
        m.damage_type("Physical", 100);
        m.resist("Physical", 50, 70);
        m.resist("Cold", 30, 50);
        m.resist("Poison", 10, 20);
        m.resist("Energy", 10, 20);
        m.skill("Wrestling", 73.4, 88.1);
        m.skill("Tactics", 73.6, 110.5);
        m.skill("MagicResist", 32.8, 54.6);
        m.skill("Anatomy", 0.0, 0.0);
        m.fame = 10000;
        m.karma = 10000;
        m.armor = 24;
        m.tamable = false;
    });
});

mobile("Squirrel", |c| {
    c.ai = "Animal";
    c.fight_mode = "Aggressor";
    c.corpse_name = "a squirrel corpse";
    c.name = "a squirrell";
    c.meat = 1;
    c.favorite_food = ["FruitsAndVeggies"];
    c.on_create(|m| {
        m.body = 278;
        m.str = 44..=50;
        m.dex = 35;
        m.int = 5;
        m.hits = 42..=50;
        m.damage = 1..=2;
        m.damage_type("Physical", 100);
        m.resist("Physical", 30, 34);
        m.resist("Fire", 10, 14);
        m.resist("Cold", 30, 35);
        m.resist("Poison", 20, 25);
        m.resist("Energy", 20, 25);
        m.skill("MagicResist", 4.0);
        m.skill("Tactics", 4.0);
        m.skill("Wrestling", 4.0);
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = -21.3;
    });
});

mobile("Hydra", |c| {
    c.corpse_name = "a hydra corpse";
    c.name = "a hydra";
    c.hides = 40;
    c.meat = 19;
    c.treasure_map_level = 5;
    c.loot = [["UltraRich", 3]];
    c.monster_abilities = ["FireBreath"];
    c.on_create(|m| {
        m.body = 265;
        m.sound = 0x16A;
        m.str = 801..=828;
        m.dex = 102..=118;
        m.int = 102..=120;
        m.hits = 1480..=1500;
        m.damage = 21..=26;
        m.damage_type("Physical", 60);
        m.damage_type("Fire", 10);
        m.damage_type("Cold", 10);
        m.damage_type("Poison", 10);
        m.damage_type("Energy", 10);
        m.resist("Physical", 65, 75);
        m.resist("Fire", 70, 85);
        m.resist("Cold", 25, 35);
        m.resist("Poison", 35, 43);
        m.resist("Energy", 36, 45);
        m.skill("Wrestling", 103.5, 117.4);
        m.skill("Tactics", 100.1, 109.8);
        m.skill("MagicResist", 85.5, 98.5);
        m.skill("Anatomy", 75.4, 79.8);
    });
});

mobile("Abscess", |c| {
    c.inherits = "Hydra";
    c.corpse_name = "an Abscess corpse";
    c.name = "Abscess";
    c.gives_ml_minor_artifact = true;
    c.loot = [["UltraRich", 4]];
    c.on_create(|m| {
        m.paragon = true;
        m.hue = 0x8FD;
        m.str = 845..=871;
        m.dex = 121..=134;
        m.int = 124..=142;
        m.hits = 7470..=7540;
        m.damage = 26..=31;
        m.damage_type("Physical", 60);
        m.damage_type("Fire", 10);
        m.damage_type("Cold", 10);
        m.damage_type("Poison", 10);
        m.damage_type("Energy", 10);
        m.resist("Physical", 65, 75);
        m.resist("Fire", 70, 80);
        m.resist("Cold", 25, 35);
        m.resist("Poison", 35, 45);
        m.resist("Energy", 35, 45);
        m.skill("Wrestling", 132.3, 143.8);
        m.skill("Tactics", 121.0, 130.5);
        m.skill("MagicResist", 102.9, 119.0);
        m.skill("Anatomy", 91.8, 94.3);
    });
});

mobile("Coil", |c| {
    c.inherits = "SilverSerpent";
    c.corpse_name = "a Coil corpse";
    c.name = "Coil";
    c.poison_immune = 4;
    c.gives_ml_minor_artifact = true;
    c.hides = 48;
    c.meat = 1;
    c.loot = [["UltraRich", 3]];
    c.hit_poison = 4;
    c.weapon_ability = "MortalStrike";
    c.on_create(|m| {
        m.paragon = true;
        m.hue = 0x3F;
        m.str = 205..=343;
        m.dex = 202..=283;
        m.int = 88..=142;
        m.hits = 628..=1291;
        m.damage = 19..=28;
        m.damage_type("Physical", 50);
        m.damage_type("Poison", 50);
        m.resist("Physical", 56, 62);
        m.resist("Fire", 25, 30);
        m.resist("Cold", 25, 30);
        m.resist("Poison", 100);
        m.resist("Energy", 25, 30);
        m.skill("Wrestling", 124.5, 141.3);
        m.skill("Tactics", 130.2, 142.0);
        m.skill("MagicResist", 102.3, 113.0);
        m.skill("Anatomy", 120.8, 138.1);
        m.skill("Poisoning", 110.1, 133.4);
        m.pack = [#{ gem: 2 }, #{ item: "Server.Items.Bone" }];
    });
});

mobile("Harpy", |c| {
    c.corpse_name = "a harpy corpse";
    c.name = "a harpy";
    c.can_rummage_corpses = true;
    c.meat = 4;
    c.feathers = 50;
    c.can_fly = true;
    c.loot = [["Meager", 2]];
    c.meat_type = "Bird";
    c.attack_sound = 916;
    c.anger_sound = 916;
    c.death_sound = 917;
    c.hurt_sound = 919;
    c.idle_sound = 918;
    c.on_create(|m| {
        m.body = 30;
        m.sound = 0x192;
        m.str = 96..=120;
        m.dex = 86..=110;
        m.int = 51..=75;
        m.hits = 58..=72;
        m.damage = 5..=7;
        m.damage_type("Physical", 100);
        m.resist("Physical", 25, 30);
        m.resist("Fire", 10, 20);
        m.resist("Cold", 10, 30);
        m.resist("Poison", 20, 30);
        m.resist("Energy", 10, 20);
        m.skill("MagicResist", 50.1, 65.0);
        m.skill("Tactics", 70.1, 100.0);
        m.skill("Wrestling", 60.1, 90.0);
        m.fame = 2500;
        m.karma = -2500;
        m.armor = 28;
    });
});

mobile("Saliva", |c| {
    c.inherits = "Harpy";
    c.corpse_name = "a Saliva corpse";
    c.name = "Saliva";
    c.loot = [["UltraRich", 2]];
    c.on_create(|m| {
        m.hue = 0x11E;
        m.str = 110..=206;
        m.dex = 123..=222;
        m.int = 80..=127;
        m.hits = 409..=842;
        m.damage = 20..=22;
        m.damage_type("Physical", 100);
        m.resist("Physical", 46, 48);
        m.resist("Fire", 32, 40);
        m.resist("Cold", 34, 49);
        m.resist("Poison", 40, 48);
        m.resist("Energy", 35, 39);
        m.skill("Wrestling", 106.4, 128.8);
        m.skill("Tactics", 129.9, 141.0);
        m.skill("MagicResist", 84.3, 105.0);
    });
});

mobile("Thrasher", |c| {
    c.inherits = "Alligator";
    c.corpse_name = "a Thrasher corpse";
    c.name = "Thrasher";
    c.gives_ml_minor_artifact = true;
    c.hides = 48;
    c.meat = 1;
    c.loot = [["FilthyRich", 4]];
    c.weapon_ability = "ArmorIgnore";
    c.on_create(|m| {
        m.paragon = true;
        m.hue = 0x497;
        m.str = 93..=327;
        m.dex = 7..=201;
        m.int = 15..=67;
        m.hits = 260..=984;
        m.stam = 56..=75;
        m.mana = 25..=30;
        m.damage = 20..=30;
        m.damage_type("Physical", 100);
        m.resist("Physical", 50, 55);
        m.resist("Fire", 25, 29);
        m.resist("Poison", 25, 28);
        m.skill("Wrestling", 101.2, 118.3);
        m.skill("Tactics", 96.3, 117.3);
        m.skill("MagicResist", 102.4, 118.6);
    });
});

mobile("FetidEssence", |c| {
    c.ai = "Mage";
    c.corpse_name = "a fetid essence corpse";
    c.name = "a fetid essence";
    c.poison_immune = 3;
    c.loot = "FilthyRich";
    c.hit_poison = 3;
    c.anger_sound = 1389;
    c.idle_sound = 1387;
    c.attack_sound = 1388;
    c.hurt_sound = 1388;
    c.death_sound = 1390;
    c.on_create(|m| {
        m.body = 273;
        m.str = 101..=150;
        m.dex = 210..=250;
        m.int = 451..=550;
        m.hits = 551..=650;
        m.damage = 21..=25;
        m.damage_type("Physical", 30);
        m.damage_type("Poison", 70);
        m.resist("Physical", 40, 50);
        m.resist("Fire", 40, 50);
        m.resist("Cold", 40, 50);
        m.resist("Poison", 70, 90);
        m.resist("Energy", 75, 80);
        m.skill("Meditation", 91.4, 99.4);
        m.skill("EvalInt", 88.5, 92.3);
        m.skill("Magery", 97.9, 101.7);
        m.skill("Poisoning", 100.0);
        m.skill("Anatomy", 0.0, 4.5);
        m.skill("MagicResist", 103.5, 108.8);
        m.skill("Tactics", 81.0, 84.6);
        m.skill("Wrestling", 81.3, 83.9);
        m.fame = 3700;
        m.karma = -3700;
    });
});

mobile("InterredGrizzle", |c| {
    c.ai = "Mage";
    c.corpse_name = "an interred grizzle corpse";
    c.name = "a interred grizzle";
    c.loot = "FilthyRich";
    c.anger_sound = 1409;
    c.idle_sound = 1410;
    c.attack_sound = 1408;
    c.hurt_sound = 1411;
    c.death_sound = 1412;
    c.on_create(|m| {
        m.body = 259;
        m.str = 451..=500;
        m.dex = 201..=250;
        m.int = 801..=850;
        m.hits = 1500;
        m.stam = 150;
        m.damage = 16..=19;
        m.damage_type("Physical", 30);
        m.damage_type("Fire", 70);
        m.resist("Physical", 35, 55);
        m.resist("Fire", 20, 65);
        m.resist("Cold", 55, 80);
        m.resist("Poison", 20, 35);
        m.resist("Energy", 60, 80);
        m.skill("Meditation", 77.7, 84.0);
        m.skill("EvalInt", 72.2, 79.6);
        m.skill("Magery", 83.7, 89.6);
        m.skill("Poisoning", 0.0);
        m.skill("Anatomy", 0.0);
        m.skill("MagicResist", 80.2, 87.3);
        m.skill("Tactics", 104.5, 105.1);
        m.skill("Wrestling", 105.1, 109.4);
        m.fame = 3700;
        m.karma = -3700;
    });
});

mobile("FeralTreefellow", |c| {
    c.fight_mode = "Evil";
    c.corpse_name = "a treefellow corpse";
    c.name = "a feral treefellow";
    c.bleed_immune = true;
    c.loot = "Average";
    c.opposition = "FeyAndUndead";
    c.weapon_ability = "Dismount";
    c.idle_sound = 443;
    c.death_sound = 31;
    c.attack_sound = 672;
    c.on_create(|m| {
        m.body = 301;
        m.str = 1351..=1600;
        m.dex = 301..=550;
        m.int = 651..=900;
        m.hits = 1170..=1320;
        m.damage = 26..=35;
        m.damage_type("Physical", 100);
        m.resist("Physical", 60, 70);
        m.resist("Cold", 70, 80);
        m.resist("Poison", 60, 70);
        m.resist("Energy", 40, 60);
        m.skill("MagicResist", 40.1, 55.0);
        m.skill("Tactics", 65.1, 90.0);
        m.skill("Wrestling", 65.1, 85.0);
        m.fame = 12500;
        m.karma = 12500;
        m.armor = 24;
        m.pack = [#{ item: "Server.Items.Log", amount: [23, 34] }];
    });
});

mobile("Minotaur", |c| {
    c.corpse_name = "a minotaur corpse";
    c.name = "a minotaur";
    c.loot = "Rich";
    c.weapon_ability = "ParalyzingBlow";
    c.anger_sound = 1431;
    c.idle_sound = 1430;
    c.attack_sound = 1433;
    c.hurt_sound = 1434;
    c.death_sound = 1436;
    c.on_create(|m| {
        m.body = 263;
        m.str = 301..=340;
        m.dex = 91..=110;
        m.int = 31..=50;
        m.hits = 301..=340;
        m.damage = 11..=20;
        m.damage_type("Physical", 100);
        m.resist("Physical", 55, 65);
        m.resist("Fire", 25, 35);
        m.resist("Cold", 30, 40);
        m.resist("Poison", 30, 40);
        m.resist("Energy", 30, 40);
        m.skill("Meditation", 0.0);
        m.skill("EvalInt", 0.0);
        m.skill("Magery", 0.0);
        m.skill("Poisoning", 0.0);
        m.skill("Anatomy", 0.0);
        m.skill("MagicResist", 56.1, 64.0);
        m.skill("Tactics", 93.3, 97.8);
        m.skill("Wrestling", 90.4, 92.1);
        m.fame = 5000;
        m.karma = -5000;
        m.armor = 28;
    });
});

mobile("MinotaurCaptain", |c| {
    c.corpse_name = "a minotaur corpse";
    c.name = "a minotaur captain";
    c.loot = "Rich";
    c.weapon_ability = "ParalyzingBlow";
    c.anger_sound = 1431;
    c.idle_sound = 1430;
    c.attack_sound = 1433;
    c.hurt_sound = 1434;
    c.death_sound = 1436;
    c.on_create(|m| {
        m.body = 280;
        m.str = 401..=425;
        m.dex = 91..=110;
        m.int = 31..=50;
        m.hits = 401..=440;
        m.damage = 11..=20;
        m.damage_type("Physical", 100);
        m.resist("Physical", 65, 75);
        m.resist("Fire", 35, 45);
        m.resist("Cold", 40, 50);
        m.resist("Poison", 40, 50);
        m.resist("Energy", 40, 50);
        m.skill("Meditation", 0.0);
        m.skill("EvalInt", 0.0);
        m.skill("Magery", 0.0);
        m.skill("Poisoning", 0.0);
        m.skill("Anatomy", 0.0, 6.3);
        m.skill("MagicResist", 66.1, 73.6);
        m.skill("Tactics", 93.0, 109.9);
        m.skill("Wrestling", 92.6, 107.2);
        m.fame = 7000;
        m.karma = -7000;
        m.armor = 28;
    });
});

mobile("MinotaurScout", |c| {
    c.corpse_name = "a minotaur corpse";
    c.name = "a minotaur scout";
    c.loot = "Rich";
    c.weapon_ability = "ParalyzingBlow";
    c.anger_sound = 1431;
    c.idle_sound = 1430;
    c.attack_sound = 1433;
    c.hurt_sound = 1434;
    c.death_sound = 1436;
    c.on_create(|m| {
        m.body = 281;
        m.str = 353..=375;
        m.dex = 111..=130;
        m.int = 34..=50;
        m.hits = 354..=383;
        m.damage = 11..=20;
        m.damage_type("Physical", 100);
        m.resist("Physical", 55, 65);
        m.resist("Fire", 25, 35);
        m.resist("Cold", 30, 40);
        m.resist("Poison", 30, 40);
        m.resist("Energy", 30, 40);
        m.skill("Anatomy", 0.0);
        m.skill("MagicResist", 60.6, 67.5);
        m.skill("Tactics", 86.9, 103.6);
        m.skill("Wrestling", 85.6, 104.5);
        m.fame = 5000;
        m.karma = -5000;
        m.armor = 28;
    });
});

mobile("PestilentBandage", |c| {
    c.corpse_name = "a pestilent bandage corpse";
    c.name = "a pestilent bandage";
    c.loot = "Rich";
    c.hit_poison = 4;
    c.can_heal = true;
    c.on_create(|m| {
        m.body = 154;
        m.hue = 0x515;
        m.sound = 0x1D7;
        m.str = 691..=740;
        m.dex = 141..=180;
        m.int = 51..=80;
        m.hits = 415..=445;
        m.damage = 13..=23;
        m.damage_type("Physical", 40);
        m.damage_type("Cold", 20);
        m.damage_type("Poison", 40);
        m.resist("Physical", 45, 55);
        m.resist("Fire", 10, 20);
        m.resist("Cold", 50, 60);
        m.resist("Poison", 20, 30);
        m.resist("Energy", 20, 30);
        m.skill("Poisoning", 0.0, 10.0);
        m.skill("Anatomy", 0.0);
        m.skill("MagicResist", 75.0, 80.0);
        m.skill("Tactics", 80.0, 85.0);
        m.skill("Wrestling", 70.0, 75.0);
        m.fame = 20000;
        m.karma = -20000;
        m.pack = [#{ item: "Server.Items.Bandage", amount: 5 }];
    });
});

mobile("TormentedMinotaur", |c| {
    c.corpse_name = "a tormented minotaur corpse";
    c.name = "Tormented Minotaur";
    c.poison_immune = 3;
    c.treasure_map_level = 3;
    c.loot = [["FilthyRich", 10]];
    c.weapon_ability = "Dismount";
    c.death_sound = 1430;
    c.attack_sound = 1431;
    c.idle_sound = 1432;
    c.anger_sound = 1433;
    c.hurt_sound = 1434;
    c.on_create(|m| {
        m.body = 262;
        m.str = 822..=930;
        m.dex = 401..=415;
        m.int = 128..=138;
        m.hits = 4000..=4200;
        m.damage = 16..=30;
        m.damage_type("Physical", 100);
        m.resist("Physical", 62);
        m.resist("Fire", 74);
        m.resist("Cold", 54);
        m.resist("Poison", 56);
        m.resist("Energy", 54);
        m.skill("Wrestling", 110.1, 111.0);
        m.skill("Tactics", 100.7, 102.8);
        m.skill("MagicResist", 104.3, 116.3);
        m.fame = 20000;
        m.karma = -20000;
    });
});

mobile("Troglodyte", |c| {
    c.corpse_name = "a troglodyte corpse";
    c.name = "a troglodyte";
    c.loot = "Rich";
    c.can_heal = true;
    c.on_create(|m| {
        m.body = 267;
        m.sound = 0x59F;
        m.str = 148..=217;
        m.dex = 91..=120;
        m.int = 51..=70;
        m.hits = 302..=340;
        m.damage = 11..=14;
        m.damage_type("Physical", 100);
        m.resist("Physical", 30, 35);
        m.resist("Fire", 20, 30);
        m.resist("Cold", 35, 40);
        m.resist("Poison", 30, 40);
        m.resist("Energy", 30, 40);
        m.skill("Anatomy", 70.5, 94.8);
        m.skill("MagicResist", 51.8, 65.0);
        m.skill("Tactics", 80.4, 94.7);
        m.skill("Wrestling", 70.2, 93.5);
        m.skill("Healing", 70.0, 95.0);
        m.fame = 5000;
        m.karma = -5000;
        m.armor = 28;
        m.pack = [#{ item: "Server.Items.Bandage", amount: 5 }, #{ item: "Server.Items.Ribs" }];
    });
});

mobile("Scorpion", |c| {
    c.corpse_name = "a scorpion corpse";
    c.name = "a scorpion";
    c.meat = 1;
    c.poison_immune = 2;
    c.loot = "Meager";
    c.favorite_food = ["Meat"];
    c.pack_instinct = ["Arachnid"];
    c.hit_poison = #{ chance: 0.8, then: 2, "else": 3 };
    c.on_create(|m| {
        m.body = 48;
        m.sound = 0x18D;
        m.str = 73..=115;
        m.dex = 76..=95;
        m.int = 16..=30;
        m.hits = 50..=63;
        m.mana = 0;
        m.damage = 5..=10;
        m.damage_type("Physical", 60);
        m.damage_type("Poison", 40);
        m.resist("Physical", 20, 25);
        m.resist("Fire", 10, 15);
        m.resist("Cold", 20, 25);
        m.resist("Poison", 40, 50);
        m.resist("Energy", 10, 15);
        m.skill("Poisoning", 80.1, 100.0);
        m.skill("MagicResist", 30.1, 35.0);
        m.skill("Tactics", 60.3, 75.0);
        m.skill("Wrestling", 50.3, 65.0);
        m.fame = 2000;
        m.karma = -2000;
        m.armor = 28;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = 47.1;
        m.pack = [#{ item: "Server.Items.LesserPoisonPotion" }];
    });
});

mobile("Miasma", |c| {
    c.inherits = "Scorpion";
    c.corpse_name = "a Miasma corpse";
    c.name = "Miasma";
    c.gives_ml_minor_artifact = true;
    c.treasure_map_level = 5;
    c.loot = [["UltraRich", 4]];
    c.weapon_ability_chance = 0.75;
    c.hit_poison_chance = 0.35;
    c.hit_poison = 4;
    c.has_mana_override = true;
    c.weapon_ability = "MortalStrike";
    c.on_create(|m| {
        m.paragon = true;
        m.hue = 0x8FD;
        m.str = 255..=847;
        m.dex = 145..=428;
        m.int = 26..=380;
        m.hits = 750..=2000;
        m.mana = 5..=60;
        m.damage = 20..=30;
        m.damage_type("Physical", 60);
        m.damage_type("Poison", 40);
        m.resist("Physical", 50, 54);
        m.resist("Fire", 40, 45);
        m.resist("Cold", 50, 55);
        m.resist("Poison", 70, 80);
        m.resist("Energy", 40, 45);
        m.skill("Wrestling", 84.9, 103.3);
        m.skill("Tactics", 98.4, 110.6);
        m.skill("Anatomy", 0.0);
        m.skill("MagicResist", 74.4, 77.7);
        m.skill("Poisoning", 128.5, 143.6);
        m.fame = 21000;
        m.karma = -21000;
    });
});

mobile("GreaterDragon", |c| {
    c.ai = "Mage";
    c.corpse_name = "a dragon corpse";
    c.name = "a greater dragon";
    c.treasure_map_level = 5;
    c.meat = 19;
    c.hides = 30;
    c.scales = 7;
    c.can_fly = true;
    c.loot = [["FilthyRich", 4], ["Gems", 8]];
    c.stat_loss_after_tame = true;
    c.hide_type = "Barbed";
    c.favorite_food = ["Meat"];
    c.can_anger_on_tame = true;
    c.weapon_ability = "BleedAttack";
    c.monster_abilities = ["FireBreath"];
    c.on_create(|m| {
        m.bodies = [12, 59];
        m.sound = 0x16A;
        m.str = 1025..=1425;
        m.dex = 81..=148;
        m.int = 475..=675;
        m.hits = 1000..=2000;
        m.stam = 120..=135;
        m.damage = 24..=33;
        m.damage_type("Physical", 100);
        m.resist("Physical", 60, 85);
        m.resist("Fire", 65, 90);
        m.resist("Cold", 40, 55);
        m.resist("Poison", 40, 60);
        m.resist("Energy", 50, 75);
        m.skill("Meditation", 0.0);
        m.skill("EvalInt", 110.0, 140.0);
        m.skill("Magery", 110.0, 140.0);
        m.skill("Poisoning", 0.0);
        m.skill("Anatomy", 0.0);
        m.skill("MagicResist", 110.0, 140.0);
        m.skill("Tactics", 110.0, 140.0);
        m.skill("Wrestling", 115.0, 145.0);
        m.fame = 22000;
        m.karma = -15000;
        m.armor = 60;
        m.tamable = true;
        m.control_slots = 5;
        m.min_tame_skill = 104.7;
    });
});

mobile("Lurg", |c| {
    c.inherits = "Troglodyte";
    c.corpse_name = "a Lurg corpse";
    c.name = "Lurg";
    c.gives_ml_minor_artifact = true;
    c.treasure_map_level = 4;
    c.loot = [["UltraRich", 2]];
    c.weapon_ability = "CrushingBlow";
    c.on_create(|m| {
        m.paragon = true;
        m.hue = 0x455;
        m.str = 584..=625;
        m.dex = 163..=176;
        m.int = 90..=106;
        m.hits = 3034..=3189;
        m.stam = 163..=176;
        m.mana = 90..=106;
        m.damage = 16..=19;
        m.damage_type("Physical", 100);
        m.resist("Physical", 50, 53);
        m.resist("Fire", 45, 47);
        m.resist("Cold", 56, 60);
        m.resist("Poison", 50, 60);
        m.resist("Energy", 41, 56);
        m.skill("Wrestling", 122.7, 130.5);
        m.skill("Tactics", 109.3, 118.5);
        m.skill("MagicResist", 72.9, 87.6);
        m.skill("Anatomy", 110.5, 124.0);
        m.skill("Healing", 84.1, 105.0);
        m.fame = 10000;
        m.karma = -10000;
    });
});

mobile("Putrefier", |c| {
    c.inherits = "Balron";
    c.corpse_name = "a Putrefier corpse";
    c.name = "Putrefier";
    c.gives_ml_minor_artifact = true;
    c.treasure_map_level = 5;
    c.loot = [["UltraRich", 3]];
    c.hit_poison = 3;
    c.on_create(|m| {
        m.paragon = true;
        m.hue = 0x3F;
        m.str = 1057..=1400;
        m.dex = 232..=560;
        m.int = 201..=440;
        m.hits = 3010..=4092;
        m.damage = 27..=34;
        m.damage_type("Physical", 50);
        m.damage_type("Fire", 0);
        m.damage_type("Poison", 50);
        m.damage_type("Energy", 0);
        m.resist("Physical", 65, 80);
        m.resist("Fire", 65, 80);
        m.resist("Cold", 50, 60);
        m.resist("Poison", 100);
        m.resist("Energy", 40, 50);
        m.skill("Wrestling", 111.2, 128.0);
        m.skill("Tactics", 115.2, 125.2);
        m.skill("MagicResist", 143.4, 170.0);
        m.skill("Anatomy", 44.6, 67.0);
        m.skill("Magery", 117.6, 118.8);
        m.skill("EvalInt", 113.0, 128.8);
        m.skill("Meditation", 41.4, 85.0);
        m.skill("Poisoning", 45.0, 50.0);
        m.fame = 24000;
        m.karma = -24000;
        m.pack = [#{ scroll: [4, 7] }, #{ scroll: [4, 7] }];
    });
});

mobile("CorporealBrume", |c| {
    c.corpse_name = "a corporeal brume corpse";
    c.name = "a corporeal brume";
    c.loot = ["FilthyRich", "Rich"];
    c.aura_interval = 20.0;
    c.aura_range = 10;
    c.aura_fire_damage = 0;
    c.aura_cold_damage = 100;
    c.aura_base_damage = #{ min: 25, max: 35 };
    c.on_create(|m| {
        m.body = 260;
        m.sound = 0x56B;
        m.str = 400..=450;
        m.dex = 100..=150;
        m.int = 50..=60;
        m.hits = 1150..=1250;
        m.damage = 21..=25;
        m.damage_type("Physical", 100);
        m.resist("Physical", 100);
        m.resist("Fire", 40, 50);
        m.resist("Cold", 40, 50);
        m.resist("Poison", 50, 60);
        m.resist("Energy", 30, 40);
        m.skill("Wrestling", 110.0, 115.0);
        m.skill("Tactics", 110.0, 115.0);
        m.skill("MagicResist", 80.0, 95.0);
        m.skill("Anatomy", 100.0, 110.0);
        m.fame = 12000;
        m.karma = -12000;
    });
});

mobile("CrystalDaemon", |c| {
    c.ai = "Mage";
    c.corpse_name = "a crystal daemon corpse";
    c.name = "a crystal daemon";
    c.loot = [["FilthyRich", 3]];
    c.on_create(|m| {
        m.body = 784;
        m.hue = 0x3E8;
        m.sound = 0x47D;
        m.str = 140..=200;
        m.dex = 120..=150;
        m.int = 800..=850;
        m.hits = 200..=220;
        m.damage = 16..=20;
        m.damage_type("Physical", 0);
        m.damage_type("Cold", 40);
        m.damage_type("Energy", 60);
        m.resist("Physical", 20, 40);
        m.resist("Fire", 0, 20);
        m.resist("Cold", 60, 80);
        m.resist("Poison", 20, 40);
        m.resist("Energy", 65, 75);
        m.skill("Wrestling", 60.0, 80.0);
        m.skill("Tactics", 70.0, 80.0);
        m.skill("MagicResist", 100.0, 110.0);
        m.skill("Magery", 120.0, 130.0);
        m.skill("EvalInt", 100.0, 110.0);
        m.skill("Meditation", 100.0, 110.0);
        m.fame = 15000;
        m.karma = -15000;
        m.pack = [#{ arcaneScroll: [0, 1] }];
    });
});

mobile("CrystalLatticeSeeker", |c| {
    c.ai = "Mage";
    c.corpse_name = "a Crystal Lattice Seeker corpse";
    c.name = "Crystal Lattice Seeker";
    c.feathers = 100;
    c.treasure_map_level = 5;
    c.loot = [["FilthyRich", 4], "Gems"];
    c.attack_sound = 758;
    c.death_sound = 759;
    c.anger_sound = 760;
    c.hurt_sound = 761;
    c.idle_sound = 762;
    c.on_create(|m| {
        m.body = 123;
        m.hue = 0x47E;
        m.str = 550..=850;
        m.dex = 190..=250;
        m.int = 350..=450;
        m.hits = 350..=550;
        m.damage = 13..=19;
        m.damage_type("Physical", 100);
        m.resist("Physical", 80, 90);
        m.resist("Fire", 40, 50);
        m.resist("Cold", 40, 50);
        m.resist("Poison", 40, 50);
        m.resist("Energy", 40, 50);
        m.skill("Anatomy", 50.0, 75.0);
        m.skill("EvalInt", 90.0, 100.0);
        m.skill("Magery", 100.0, 100.0);
        m.skill("Meditation", 90.0, 100.0);
        m.skill("MagicResist", 90.0, 100.0);
        m.skill("Tactics", 90.0, 100.0);
        m.skill("Wrestling", 90.0, 100.0);
        m.fame = 17000;
        m.karma = -17000;
        m.pack = [#{ arcaneScroll: [0, 2] }];
    });
});

mobile("CrystalVortex", |c| {
    c.corpse_name = "a crystal vortex corpse";
    c.name = "a crystal vortex";
    c.loot = [["FilthyRich", 2]];
    c.anger_sound = 21;
    c.attack_sound = 40;
    c.on_create(|m| {
        m.body = 13;
        m.hue = 0x2B2;
        m.sound = 0x107;
        m.str = 800..=900;
        m.dex = 500..=600;
        m.int = 200;
        m.hits = 350..=400;
        m.mana = 0;
        m.damage = 15..=20;
        m.damage_type("Physical", 0);
        m.damage_type("Cold", 50);
        m.damage_type("Energy", 50);
        m.resist("Physical", 60, 80);
        m.resist("Fire", 0, 10);
        m.resist("Cold", 70, 80);
        m.resist("Poison", 40, 50);
        m.resist("Energy", 60, 90);
        m.skill("MagicResist", 120.0);
        m.skill("Tactics", 120.0);
        m.skill("Wrestling", 120.0);
        m.fame = 17000;
        m.karma = -17000;
        m.pack = [#{ arcaneScroll: [0, 2] }];
    });
});

mobile("MantraEffervescence", |c| {
    c.ai = "Mage";
    c.corpse_name = "a mantra effervescence corpse";
    c.name = "a mantra effervescence";
    c.loot = ["FilthyRich", "Rich"];
    c.on_create(|m| {
        m.body = 273;
        m.sound = 0x56E;
        m.str = 130..=150;
        m.dex = 120..=130;
        m.int = 150..=230;
        m.hits = 150..=250;
        m.damage = 21..=25;
        m.damage_type("Physical", 30);
        m.damage_type("Energy", 70);
        m.resist("Physical", 60, 65);
        m.resist("Fire", 40, 50);
        m.resist("Cold", 40, 50);
        m.resist("Poison", 50, 60);
        m.resist("Energy", 100);
        m.skill("Wrestling", 80.0, 85.0);
        m.skill("Tactics", 80.0, 85.0);
        m.skill("MagicResist", 105.0, 115.0);
        m.skill("Magery", 90.0, 110.0);
        m.skill("EvalInt", 80.0, 90.0);
        m.skill("Meditation", 90.0, 100.0);
        m.fame = 6500;
        m.karma = -6500;
    });
});

mobile("UnfrozenMummy", |c| {
    c.ai = "Mage";
    c.corpse_name = "an unfrozen mummy corpse";
    c.name = "an unfrozen mummy";
    c.loot = [["UltraRich", 2]];
    c.on_create(|m| {
        m.body = 155;
        m.hue = 0x480;
        m.sound = 0x1D7;
        m.str = 450..=500;
        m.dex = 200..=250;
        m.int = 800..=850;
        m.hits = 1500;
        m.damage = 16..=20;
        m.damage_type("Physical", 0);
        m.damage_type("Energy", 50);
        m.damage_type("Cold", 50);
        m.resist("Physical", 35, 40);
        m.resist("Fire", 20, 30);
        m.resist("Cold", 60, 80);
        m.resist("Poison", 20, 30);
        m.resist("Energy", 70, 80);
        m.skill("Wrestling", 90.0, 100.0);
        m.skill("Tactics", 100.0);
        m.skill("MagicResist", 250.0);
        m.skill("Magery", 50.0, 60.0);
        m.skill("EvalInt", 50.0, 60.0);
        m.skill("Meditation", 80.0);
        m.fame = 25000;
        m.karma = -25000;
        m.pack = [#{ arcaneScroll: [0, 2] }];
    });
});

mobile("Chiikkaha", |c| {
    c.inherits = "RatmanMage";
    c.corpse_name = "a Chiikkaha the Toothed corpse";
    c.name = "Chiikkaha the Toothed";
    c.on_create(|m| {
        m.str = 450..=476;
        m.dex = 157..=179;
        m.int = 251..=275;
        m.hits = 400..=425;
        m.damage = 10..=17;
        m.damage_type("Physical", 100);
        m.resist("Physical", 40, 45);
        m.resist("Fire", 10, 20);
        m.resist("Cold", 10, 20);
        m.resist("Poison", 10, 20);
        m.resist("Energy", 100);
        m.skill("EvalInt", 70.1, 80.0);
        m.skill("Magery", 70.1, 90.0);
        m.skill("MagicResist", 65.1, 96.0);
        m.skill("Tactics", 50.1, 75.0);
        m.skill("Wrestling", 50.1, 75.0);
        m.fame = 7500;
        m.karma = -7500;
    });
});

mobile("MougGuur", |c| {
    c.inherits = "Ettin";
    c.corpse_name = "a Moug-Guur corpse";
    c.name = "Moug-Guur";
    c.on_create(|m| {
        m.str = 556..=575;
        m.dex = 84..=94;
        m.int = 59..=73;
        m.hits = 400..=415;
        m.damage = 12..=20;
        m.damage_type("Physical", 100);
        m.resist("Physical", 61, 65);
        m.resist("Fire", 16, 19);
        m.resist("Cold", 41, 46);
        m.resist("Poison", 21, 24);
        m.resist("Energy", 19, 25);
        m.skill("MagicResist", 70.2, 75.0);
        m.skill("Tactics", 80.8, 81.7);
        m.skill("Wrestling", 93.9, 99.4);
        m.fame = 3000;
        m.karma = -3000;
    });
});

mobile("Szavetra", |c| {
    c.inherits = "Succubus";
    c.corpse_name = "a Szavetra corpse";
    c.name = "Szavetra";
    c.on_create(|m| {
        m.str = 627..=655;
        m.dex = 164..=193;
        m.int = 566..=595;
        m.hits = 312..=415;
        m.damage = 20..=30;
        m.damage_type("Physical", 75);
        m.damage_type("Energy", 25);
        m.resist("Physical", 83, 90);
        m.resist("Fire", 72, 80);
        m.resist("Cold", 40, 49);
        m.resist("Poison", 51, 60);
        m.resist("Energy", 50, 60);
        m.skill("EvalInt", 90.3, 99.8);
        m.skill("Magery", 100.1, 100.6);
        m.skill("Meditation", 90.1, 110.0);
        m.skill("MagicResist", 112.2, 127.2);
        m.skill("Tactics", 91.2, 92.8);
        m.skill("Wrestling", 80.2, 86.4);
        m.fame = 24000;
        m.karma = -24000;
    });
});

mobile("Gnaw", |c| {
    c.inherits = "DireWolf";
    c.corpse_name = "a Gnaw corpse";
    c.name = "Gnaw";
    c.gives_ml_minor_artifact = true;
    c.hides = 28;
    c.meat = 4;
    c.loot = [["FilthyRich", 2]];
    c.on_create(|m| {
        m.paragon = true;
        m.hue = 0x130;
        m.str = 151..=172;
        m.dex = 124..=145;
        m.int = 60..=86;
        m.hits = 817..=857;
        m.stam = 124..=145;
        m.mana = 52..=86;
        m.damage = 16..=22;
        m.damage_type("Physical", 100);
        m.resist("Physical", 64, 69);
        m.resist("Fire", 53, 56);
        m.resist("Cold", 22, 27);
        m.resist("Poison", 27, 30);
        m.resist("Energy", 21, 34);
        m.skill("Wrestling", 106.4, 116.5);
        m.skill("Tactics", 84.1, 103.2);
        m.skill("MagicResist", 96.8, 110.7);
        m.fame = 17500;
        m.karma = -17500;
    });
});

mobile("LadyLissith", |c| {
    c.inherits = "GiantBlackWidow";
    c.corpse_name = "a Lady Lissith corpse";
    c.name = "Lady Lissith";
    c.gives_ml_minor_artifact = true;
    c.loot = [["UltraRich", 2]];
    c.weapon_ability = "BleedAttack";
    c.on_create(|m| {
        m.paragon = true;
        m.hue = 0x452;
        m.str = 81..=130;
        m.dex = 116..=152;
        m.int = 44..=100;
        m.hits = 245..=375;
        m.stam = 116..=152;
        m.mana = 44..=100;
        m.damage = 15..=22;
        m.damage_type("Physical", 100);
        m.resist("Physical", 40, 50);
        m.resist("Fire", 31, 39);
        m.resist("Cold", 30, 40);
        m.resist("Poison", 71, 80);
        m.resist("Energy", 30, 40);
        m.skill("Wrestling", 108.6, 123.0);
        m.skill("Tactics", 102.7, 119.0);
        m.skill("MagicResist", 78.8, 95.6);
        m.skill("Anatomy", 68.6, 106.8);
        m.skill("Poisoning", 96.6, 112.9);
        m.fame = 18900;
        m.karma = -18900;
    });
});

mobile("LadySabrix", |c| {
    c.inherits = "GiantBlackWidow";
    c.corpse_name = "a Lady Sabrix corpse";
    c.name = "Lady Sabrix";
    c.gives_ml_minor_artifact = true;
    c.loot = [["UltraRich", 2]];
    c.weapon_ability = "ArmorIgnore";
    c.on_create(|m| {
        m.paragon = true;
        m.hue = 0x497;
        m.str = 82..=130;
        m.dex = 117..=146;
        m.int = 50..=98;
        m.hits = 233..=361;
        m.stam = 117..=146;
        m.mana = 50..=98;
        m.damage = 15..=22;
        m.damage_type("Physical", 100);
        m.resist("Physical", 40, 50);
        m.resist("Fire", 30, 40);
        m.resist("Cold", 30, 39);
        m.resist("Poison", 70, 80);
        m.resist("Energy", 35, 44);
        m.skill("Wrestling", 109.8, 122.8);
        m.skill("Tactics", 102.8, 120.0);
        m.skill("MagicResist", 79.4, 95.1);
        m.skill("Anatomy", 68.8, 105.1);
        m.skill("Poisoning", 97.8, 116.7);
        m.fame = 18900;
        m.karma = -18900;
    });
});

mobile("Malefic", |c| {
    c.inherits = "DreadSpider";
    c.corpse_name = "a Malefic corpse";
    c.name = "Malefic";
    c.gives_ml_minor_artifact = true;
    c.loot = [["UltraRich", 3]];
    c.weapon_ability = "Dismount";
    c.on_create(|m| {
        m.paragon = true;
        m.hue = 0x455;
        m.str = 210..=284;
        m.dex = 153..=197;
        m.int = 349..=390;
        m.hits = 600..=747;
        m.stam = 153..=197;
        m.mana = 349..=390;
        m.damage = 15..=22;
        m.damage_type("Physical", 20);
        m.damage_type("Poison", 80);
        m.resist("Physical", 60, 70);
        m.resist("Fire", 40, 50);
        m.resist("Cold", 40, 49);
        m.resist("Poison", 100);
        m.resist("Energy", 41, 48);
        m.skill("Wrestling", 96.9, 112.4);
        m.skill("Tactics", 91.3, 105.4);
        m.skill("MagicResist", 79.8, 95.1);
        m.skill("Magery", 103.0, 118.6);
        m.skill("EvalInt", 105.7, 119.6);
        m.skill("Meditation", 0.0);
        m.fame = 21000;
        m.karma = -21000;
    });
});

mobile("Silk", |c| {
    c.inherits = "GiantBlackWidow";
    c.corpse_name = "a Silk corpse";
    c.name = "Silk";
    c.gives_ml_minor_artifact = true;
    c.loot = [["UltraRich", 2]];
    c.weapon_ability = "ParalyzingBlow";
    c.on_create(|m| {
        m.paragon = true;
        m.hue = 0x47E;
        m.str = 80..=131;
        m.dex = 126..=156;
        m.int = 63..=102;
        m.hits = 279..=378;
        m.stam = 126..=156;
        m.mana = 63..=102;
        m.damage = 15..=22;
        m.damage_type("Physical", 100);
        m.resist("Physical", 40, 50);
        m.resist("Fire", 30, 39);
        m.resist("Cold", 30, 40);
        m.resist("Poison", 70, 76);
        m.resist("Energy", 30, 40);
        m.skill("Wrestling", 114.1, 123.7);
        m.skill("Tactics", 102.6, 118.3);
        m.skill("MagicResist", 78.6, 94.8);
        m.skill("Anatomy", 81.3, 105.7);
        m.skill("Poisoning", 106.0, 119.2);
        m.fame = 18900;
        m.karma = -18900;
    });
});

mobile("Virulent", |c| {
    c.inherits = "DreadSpider";
    c.corpse_name = "a Virulent corpse";
    c.name = "Virulent";
    c.gives_ml_minor_artifact = true;
    c.loot = [["UltraRich", 3]];
    c.weapon_ability = "MortalStrike";
    c.on_create(|m| {
        m.paragon = true;
        m.hue = 0x8FD;
        m.str = 207..=252;
        m.dex = 156..=194;
        m.int = 346..=398;
        m.hits = 616..=740;
        m.stam = 156..=194;
        m.mana = 346..=398;
        m.damage = 15..=22;
        m.damage_type("Physical", 20);
        m.damage_type("Poison", 80);
        m.resist("Physical", 60, 68);
        m.resist("Fire", 40, 49);
        m.resist("Cold", 41, 50);
        m.resist("Poison", 100);
        m.resist("Energy", 40, 49);
        m.skill("Wrestling", 92.8, 111.7);
        m.skill("Tactics", 91.6, 107.4);
        m.skill("MagicResist", 78.1, 93.3);
        m.skill("Poisoning", 120.0);
        m.skill("Magery", 104.2, 119.8);
        m.skill("EvalInt", 102.8, 117.8);
        m.fame = 21000;
        m.karma = -21000;
    });
});

mobile("HellHound", |c| {
    c.corpse_name = "a hell hound corpse";
    c.name = "a hell hound";
    c.meat = 1;
    c.loot = ["Average", "Meager"];
    c.favorite_food = ["Meat"];
    c.pack_instinct = ["Canine"];
    c.monster_abilities = ["FireBreath"];
    c.on_create(|m| {
        m.body = 98;
        m.sound = 0xE5;
        m.str = 102..=150;
        m.dex = 81..=105;
        m.int = 36..=60;
        m.hits = 66..=125;
        m.damage = 11..=17;
        m.damage_type("Physical", 20);
        m.damage_type("Fire", 80);
        m.resist("Physical", 25, 35);
        m.resist("Fire", 30, 40);
        m.resist("Poison", 10, 20);
        m.resist("Energy", 10, 20);
        m.fame = 3400;
        m.karma = -3400;
        m.armor = 30;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = 85.5;
        m.pack = [#{ item: "Server.Items.SulfurousAsh", amount: 5 }];
    });
});

mobile("DarkWisp", |c| {
    c.ai = "Mage";
    c.fight_mode = "Aggressor";
    c.corpse_name = "a wisp corpse";
    c.name = "a wisp";
    c.loot = ["Rich", "Average"];
    c.reacquire_delay = 1.0;
    c.opposition = "FeyAndUndead";
    c.speech = "Wisp";
    c.on_create(|m| {
        m.body = 165;
        m.sound = 0x1D2;
        m.str = 196..=225;
        m.dex = 196..=225;
        m.int = 196..=225;
        m.hits = 118..=135;
        m.damage = 17..=18;
        m.damage_type("Physical", 50);
        m.damage_type("Energy", 50);
        m.resist("Physical", 35, 45);
        m.resist("Fire", 20, 40);
        m.resist("Cold", 10, 30);
        m.resist("Poison", 5, 10);
        m.resist("Energy", 50, 70);
        m.skill("EvalInt", 80.0);
        m.skill("Magery", 80.0);
        m.skill("MagicResist", 80.0);
        m.skill("Tactics", 80.0);
        m.skill("Wrestling", 80.0);
        m.fame = 4000;
        m.karma = -4000;
        m.armor = 40;
        m.pack = [#{ item: "Server.Items.LightSource", equip: true }];
    });
});

mobile("EtherealWarrior", |c| {
    c.ai = "Mage";
    c.fight_mode = "Evil";
    c.corpse_name = "an ethereal warrior corpse";
    c.feathers = 100;
    c.loot = [["Rich", 3], "Gems"];
    c.initial_innocent = true;
    c.opposition = "FeyAndUndead";
    c.treasure_map_level = #{ atLeast: "AOS", then: 5, "else": 0 };
    c.anger_sound = 760;
    c.idle_sound = 760;
    c.hurt_sound = 761;
    c.death_sound = 759;
    c.on_create(|m| {
        m.name = random_name("ethereal warrior");
        m.body = 123;
        m.str = 586..=785;
        m.dex = 177..=255;
        m.int = 351..=450;
        m.hits = 352..=471;
        m.damage = 13..=19;
        m.damage_type("Physical", 100);
        m.resist("Physical", 80, 90);
        m.resist("Fire", 40, 50);
        m.resist("Cold", 40, 50);
        m.resist("Poison", 40, 50);
        m.resist("Energy", 40, 50);
        m.skill("Anatomy", 50.1, 75.0);
        m.skill("EvalInt", 90.1, 100.0);
        m.skill("Magery", 99.1, 100.0);
        m.skill("Meditation", 90.1, 100.0);
        m.skill("MagicResist", 90.1, 100.0);
        m.skill("Tactics", 90.1, 100.0);
        m.skill("Wrestling", 97.6, 100.0);
        m.fame = 7000;
        m.karma = 7000;
        m.armor = 120;
    });
});

mobile("Pixie", |c| {
    c.ai = "Mage";
    c.fight_mode = "Evil";
    c.corpse_name = "a pixie corpse";
    c.hides = 5;
    c.meat = 1;
    c.loot = ["LowScrolls", ["Gems", 2]];
    c.initial_innocent = true;
    c.hide_type = "Spined";
    c.opposition = "FeyAndUndead";
    c.on_create(|m| {
        m.name = random_name("pixie");
        m.body = 128;
        m.sound = 0x467;
        m.str = 21..=30;
        m.dex = 301..=400;
        m.int = 201..=250;
        m.hits = 13..=18;
        m.damage = 9..=15;
        m.damage_type("Physical", 100);
        m.resist("Physical", 80, 90);
        m.resist("Fire", 40, 50);
        m.resist("Cold", 40, 50);
        m.resist("Poison", 40, 50);
        m.resist("Energy", 40, 50);
        m.skill("EvalInt", 90.1, 100.0);
        m.skill("Magery", 90.1, 100.0);
        m.skill("Meditation", 90.1, 100.0);
        m.skill("MagicResist", 100.5, 150.0);
        m.skill("Tactics", 10.1, 20.0);
        m.skill("Wrestling", 10.1, 12.5);
        m.fame = 7000;
        m.karma = 7000;
        m.armor = 100;
        m.pack = [#{ chance: 0.02, statue: 1 }];
    });
});

mobile("Centaur", |c| {
    c.fight_mode = "Aggressor";
    c.corpse_name = "a centaur corpse";
    c.meat = 1;
    c.hides = 8;
    c.loot = ["Rich", "Average", "Gems"];
    c.opposition = "FeyAndUndead";
    c.hide_type = "Spined";
    c.on_create(|m| {
        m.name = random_name("centaur");
        m.body = 101;
        m.sound = 0x2A7;
        m.str = 202..=300;
        m.dex = 104..=260;
        m.int = 91..=100;
        m.hits = 130..=172;
        m.damage = 13..=24;
        m.damage_type("Physical", 100);
        m.resist("Physical", 45, 55);
        m.resist("Fire", 35, 45);
        m.resist("Cold", 25, 35);
        m.resist("Poison", 45, 55);
        m.resist("Energy", 35, 45);
        m.skill("Anatomy", 95.1, 115.0);
        m.skill("Archery", 95.1, 100.0);
        m.skill("MagicResist", 50.3, 80.0);
        m.skill("Tactics", 90.1, 100.0);
        m.skill("Wrestling", 95.1, 100.0);
        m.fame = 6500;
        m.karma = 0;
        m.armor = 50;
        m.pack = [#{ item: "Server.Items.Bow", equip: true }, #{ item: "Server.Items.Arrow", amount: [80, 90] }];
    });
});

mobile("FrostOoze", |c| {
    c.corpse_name = "a frost ooze corpse";
    c.name = "a frost ooze";
    c.loot = [["Gems", #{ min: 1, max: 2 }]];
    c.on_create(|m| {
        m.body = 94;
        m.sound = 0x1C8;
        m.str = 18..=30;
        m.dex = 16..=21;
        m.int = 16..=20;
        m.hits = 13..=17;
        m.damage = 3..=9;
        m.damage_type("Physical", 100);
        m.resist("Physical", 15, 20);
        m.resist("Cold", 40, 50);
        m.resist("Poison", 20, 30);
        m.resist("Energy", 10, 20);
        m.skill("MagicResist", 5.1, 10.0);
        m.skill("Tactics", 19.3, 34.0);
        m.skill("Wrestling", 25.3, 40.0);
        m.fame = 450;
        m.karma = -450;
        m.armor = 38;
    });
});

mobile("PlagueBeastLord", |c| {
    c.corpse_name = "a plague beast lord corpse";
    c.poison_immune = 4;
    c.name = "a plague beast lord";
    c.idle_sound = 447;
    c.attack_sound = 448;
    c.hurt_sound = 449;
    c.death_sound = 450;
    c.on_create(|m| {
        m.body = 775;
        m.sound = 0x2A7;
        m.speech_hue = 946;
        m.str = 500;
        m.dex = 100;
        m.int = 30;
        m.hits = 1800;
        m.damage = 20..=25;
        m.damage_type("Physical", 50);
        m.damage_type("Fire", 25);
        m.damage_type("Poison", 25);
        m.resist("Physical", 45, 55);
        m.resist("Fire", 40, 50);
        m.resist("Cold", 25, 35);
        m.resist("Poison", 75, 85);
        m.resist("Energy", 25, 35);
        m.skill("Tactics", 100.0);
        m.skill("Wrestling", 100.0);
        m.fame = 2000;
        m.karma = -2000;
        m.armor = 50;
    });
});

mobile("SandVortex", |c| {
    c.corpse_name = "a sand vortex corpse";
    c.name = "a sand vortex";
    c.loot = [["Meager", 2]];
    c.on_create(|m| {
        m.body = 790;
        m.sound = 0x107;
        m.str = 96..=120;
        m.dex = 171..=195;
        m.int = 76..=100;
        m.hits = 51..=62;
        m.damage = 3..=16;
        m.damage_type("Physical", 90);
        m.damage_type("Fire", 10);
        m.resist("Physical", 80, 90);
        m.resist("Fire", 60, 70);
        m.resist("Cold", 60, 70);
        m.resist("Poison", 60, 70);
        m.resist("Energy", 60, 70);
        m.skill("MagicResist", 150.0);
        m.skill("Tactics", 70.0);
        m.skill("Wrestling", 80.0);
        m.fame = 4500;
        m.karma = -4500;
        m.armor = 28;
        m.pack = [#{ item: "Server.Items.Bone" }];
    });
});

mobile("AgapiteElemental", |c| {
    c.corpse_name = "an ore elemental corpse";
    c.name = "an agapite elemental";
    c.bleed_immune = true;
    c.treasure_map_level = 1;
    c.loot = ["Average", ["Gems", 2]];
    c.auto_dispel = true;
    c.on_create(|m| {
        m.body = 107;
        m.sound = 0x10C;
        m.str = 226..=255;
        m.dex = 126..=145;
        m.int = 71..=92;
        m.hits = 136..=153;
        m.damage = 28..=28;
        m.damage_type("Physical", 100);
        m.resist("Physical", 30, 40);
        m.resist("Fire", 40, 50);
        m.resist("Cold", 40, 50);
        m.resist("Poison", 30, 40);
        m.resist("Energy", 10, 20);
        m.skill("MagicResist", 50.1, 95.0);
        m.skill("Tactics", 60.1, 100.0);
        m.skill("Wrestling", 60.1, 100.0);
        m.fame = 3500;
        m.karma = -3500;
        m.armor = 32;
        m.pack = [#{ item: "Server.Items.AgapiteOre", amount: 2, itemId: 6585 }];
    });
});

mobile("BronzeElemental", |c| {
    c.corpse_name = "an ore elemental corpse";
    c.name = "a bronze elemental";
    c.bleed_immune = true;
    c.treasure_map_level = 1;
    c.loot = ["Average", ["Gems", 2]];
    c.auto_dispel = true;
    c.monster_abilities = ["PoisonGasCounter"];
    c.on_create(|m| {
        m.body = 108;
        m.sound = 0x10C;
        m.str = 226..=255;
        m.dex = 126..=145;
        m.int = 71..=92;
        m.hits = 136..=153;
        m.damage = 9..=16;
        m.damage_type("Physical", 30);
        m.damage_type("Fire", 70);
        m.resist("Physical", 30, 40);
        m.resist("Fire", 30, 40);
        m.resist("Cold", 10, 20);
        m.resist("Poison", 70, 80);
        m.resist("Energy", 20, 30);
        m.skill("MagicResist", 50.1, 95.0);
        m.skill("Tactics", 60.1, 100.0);
        m.skill("Wrestling", 60.1, 100.0);
        m.fame = 5000;
        m.karma = -5000;
        m.armor = 29;
        m.pack = [#{ item: "Server.Items.BronzeOre", amount: 2, itemId: 6585 }];
    });
});

mobile("CopperElemental", |c| {
    c.corpse_name = "an ore elemental corpse";
    c.name = "a copper elemental";
    c.bleed_immune = true;
    c.treasure_map_level = 1;
    c.loot = ["Average", ["Gems", 2]];
    c.auto_dispel = true;
    c.on_create(|m| {
        m.body = 109;
        m.sound = 0x10C;
        m.str = 226..=255;
        m.dex = 126..=145;
        m.int = 71..=92;
        m.hits = 136..=153;
        m.damage = 9..=16;
        m.damage_type("Physical", 100);
        m.resist("Physical", 30, 40);
        m.resist("Fire", 30, 40);
        m.resist("Cold", 30, 40);
        m.resist("Poison", 20, 30);
        m.resist("Energy", 10, 20);
        m.skill("MagicResist", 50.1, 95.0);
        m.skill("Tactics", 60.1, 100.0);
        m.skill("Wrestling", 60.1, 100.0);
        m.fame = 4800;
        m.karma = -4800;
        m.armor = 26;
        m.pack = [#{ item: "Server.Items.CopperOre", amount: 2, itemId: 6585 }];
    });
});

mobile("DullCopperElemental", |c| {
    c.corpse_name = "an ore elemental corpse";
    c.name = "a dull copper elemental";
    c.bleed_immune = true;
    c.treasure_map_level = 1;
    c.loot = ["Average", ["Gems", 2]];
    c.auto_dispel = true;
    c.monster_abilities = ["DeathExplosion"];
    c.on_create(|m| {
        m.body = 110;
        m.sound = 0x10C;
        m.str = 226..=255;
        m.dex = 126..=145;
        m.int = 71..=92;
        m.hits = 136..=153;
        m.damage = 9..=16;
        m.damage_type("Physical", 100);
        m.resist("Physical", 30, 40);
        m.resist("Fire", 30, 40);
        m.resist("Cold", 10, 20);
        m.resist("Poison", 20, 30);
        m.resist("Energy", 20, 30);
        m.skill("MagicResist", 50.1, 95.0);
        m.skill("Tactics", 60.1, 100.0);
        m.skill("Wrestling", 60.1, 100.0);
        m.fame = 3500;
        m.karma = -3500;
        m.armor = 20;
        m.pack = [#{ item: "Server.Items.DullCopperOre", amount: 2, itemId: 6585 }];
    });
});

mobile("GoldenElemental", |c| {
    c.corpse_name = "an ore elemental corpse";
    c.name = "a golden elemental";
    c.bleed_immune = true;
    c.treasure_map_level = 1;
    c.loot = ["Average", ["Gems", 2]];
    c.auto_dispel = true;
    c.on_create(|m| {
        m.body = 166;
        m.sound = 0x10C;
        m.str = 226..=255;
        m.dex = 126..=145;
        m.int = 71..=92;
        m.hits = 136..=153;
        m.damage = 9..=16;
        m.damage_type("Physical", 100);
        m.resist("Physical", 60, 75);
        m.resist("Fire", 10, 20);
        m.resist("Cold", 30, 40);
        m.resist("Poison", 30, 40);
        m.resist("Energy", 30, 40);
        m.skill("MagicResist", 50.1, 95.0);
        m.skill("Tactics", 60.1, 100.0);
        m.skill("Wrestling", 60.1, 100.0);
        m.fame = 3500;
        m.karma = -3500;
        m.armor = 60;
        m.pack = [#{ item: "Server.Items.GoldOre", amount: 2, itemId: 6585 }];
    });
});

mobile("ShadowIronElemental", |c| {
    c.corpse_name = "an ore elemental corpse";
    c.name = "a shadow iron elemental";
    c.bleed_immune = true;
    c.treasure_map_level = 1;
    c.poison_immune = 3;
    c.loot = ["Average", ["Gems", 2]];
    c.auto_dispel = true;
    c.breath_immune = true;
    c.on_create(|m| {
        m.body = 111;
        m.sound = 0x10C;
        m.str = 226..=255;
        m.dex = 126..=145;
        m.int = 71..=92;
        m.hits = 136..=153;
        m.damage = 9..=16;
        m.damage_type("Physical", 100);
        m.resist("Physical", 30, 40);
        m.resist("Fire", 30, 40);
        m.resist("Cold", 20, 30);
        m.resist("Poison", 10, 20);
        m.resist("Energy", 30, 40);
        m.skill("MagicResist", 50.1, 95.0);
        m.skill("Tactics", 60.1, 100.0);
        m.skill("Wrestling", 60.1, 100.0);
        m.fame = 4500;
        m.karma = -4500;
        m.armor = 23;
        m.pack = [#{ item: "Server.Items.ShadowIronOre", amount: 2, itemId: 6585 }];
    });
});

mobile("ValoriteElemental", |c| {
    c.corpse_name = "an ore elemental corpse";
    c.name = "a valorite elemental";
    c.bleed_immune = true;
    c.treasure_map_level = 1;
    c.loot = ["FilthyRich", ["Gems", 4]];
    c.auto_dispel = true;
    c.monster_abilities = ["PoisonGasCounter", "DestroyEquipment"];
    c.on_create(|m| {
        m.body = 112;
        m.sound = 0x10C;
        m.str = 226..=255;
        m.dex = 126..=145;
        m.int = 71..=92;
        m.hits = 136..=153;
        m.damage = 28..=28;
        m.damage_type("Physical", 25);
        m.damage_type("Fire", 25);
        m.damage_type("Cold", 25);
        m.damage_type("Energy", 25);
        m.resist("Physical", 65, 75);
        m.resist("Fire", 50, 60);
        m.resist("Cold", 50, 60);
        m.resist("Poison", 50, 60);
        m.resist("Energy", 40, 50);
        m.skill("MagicResist", 50.1, 95.0);
        m.skill("Tactics", 60.1, 100.0);
        m.skill("Wrestling", 60.1, 100.0);
        m.fame = 3500;
        m.karma = -3500;
        m.armor = 38;
        m.pack = [#{ item: "Server.Items.ValoriteOre", amount: 2, itemId: 6585 }];
    });
});

mobile("VeriteElemental", |c| {
    c.corpse_name = "an ore elemental corpse";
    c.name = "a verite elemental";
    c.bleed_immune = true;
    c.treasure_map_level = 1;
    c.loot = ["Rich", ["Gems", 2]];
    c.auto_dispel = true;
    c.monster_abilities = ["DestroyEquipment"];
    c.on_create(|m| {
        m.body = 113;
        m.sound = 0x10C;
        m.str = 226..=255;
        m.dex = 126..=145;
        m.int = 71..=92;
        m.hits = 136..=153;
        m.damage = 9..=16;
        m.damage_type("Physical", 50);
        m.damage_type("Energy", 50);
        m.resist("Physical", 30, 40);
        m.resist("Fire", 10, 20);
        m.resist("Cold", 50, 60);
        m.resist("Poison", 50, 60);
        m.resist("Energy", 50, 60);
        m.skill("MagicResist", 50.1, 95.0);
        m.skill("Tactics", 60.1, 100.0);
        m.skill("Wrestling", 60.1, 100.0);
        m.fame = 3500;
        m.karma = -3500;
        m.armor = 35;
        m.pack = [#{ item: "Server.Items.VeriteOre", amount: 2, itemId: 6585 }];
    });
});

mobile("Reaper", |c| {
    c.ai = "Mage";
    c.corpse_name = "a reapers corpse";
    c.name = "a reaper";
    c.poison_immune = 2;
    c.treasure_map_level = 2;
    c.loot = "Average";
    c.disallow_all_moves = true;
    c.on_create(|m| {
        m.body = 47;
        m.sound = 0x1BA;
        m.str = 66..=215;
        m.dex = 66..=75;
        m.int = 101..=250;
        m.hits = 40..=129;
        m.stam = 0;
        m.damage = 9..=11;
        m.damage_type("Physical", 80);
        m.damage_type("Poison", 20);
        m.resist("Physical", 35, 45);
        m.resist("Fire", 15, 25);
        m.resist("Cold", 10, 20);
        m.resist("Poison", 40, 50);
        m.resist("Energy", 30, 40);
        m.skill("EvalInt", 90.1, 100.0);
        m.skill("Magery", 90.1, 100.0);
        m.skill("MagicResist", 100.1, 125.0);
        m.skill("Tactics", 45.1, 60.0);
        m.skill("Wrestling", 50.1, 60.0);
        m.fame = 3500;
        m.karma = -3500;
        m.armor = 40;
        m.pack = [#{ item: "Server.Items.Log", amount: 10 }, #{ item: "Server.Items.MandrakeRoot", amount: 5 }];
    });
});

mobile("Bogling", |c| {
    c.corpse_name = "a plant corpse";
    c.name = "a bogling";
    c.hides = 6;
    c.meat = 1;
    c.loot = "Meager";
    c.on_create(|m| {
        m.body = 779;
        m.sound = 0x1A6;
        m.str = 96..=120;
        m.dex = 91..=115;
        m.int = 21..=45;
        m.hits = 58..=72;
        m.damage = 5..=7;
        m.damage_type("Physical", 100);
        m.resist("Physical", 20, 25);
        m.resist("Fire", 10, 20);
        m.resist("Cold", 15, 25);
        m.resist("Poison", 15, 25);
        m.resist("Energy", 15, 25);
        m.skill("MagicResist", 75.1, 100.0);
        m.skill("Tactics", 55.1, 80.0);
        m.skill("Wrestling", 55.1, 75.0);
        m.fame = 450;
        m.karma = -450;
        m.armor = 28;
        m.pack = [#{ item: "Server.Items.Log", amount: 4 }, #{ item: "Server.Engines.Plants.Seed" }];
    });
});

mobile("Quagmire", |c| {
    c.corpse_name = "a quagmire corpse";
    c.name = "a quagmire";
    c.poison_immune = 4;
    c.loot = "Average";
    c.hit_poison = 4;
    c.hit_poison_chance = 0.1;
    c.anger_sound = 353;
    c.on_create(|m| {
        m.body = 789;
        m.sound = 0x160;
        m.str = 101..=130;
        m.dex = 66..=85;
        m.int = 31..=55;
        m.hits = 91..=105;
        m.damage = 10..=14;
        m.damage_type("Physical", 60);
        m.damage_type("Poison", 40);
        m.resist("Physical", 50, 60);
        m.resist("Fire", 10, 20);
        m.resist("Cold", 10, 20);
        m.resist("Poison", 100);
        m.resist("Energy", 20, 30);
        m.skill("MagicResist", 65.1, 75.0);
        m.skill("Tactics", 50.1, 60.0);
        m.skill("Wrestling", 60.1, 80.0);
        m.fame = 1500;
        m.karma = -1500;
        m.armor = 32;
    });
});

mobile("SwampTentacle", |c| {
    c.corpse_name = "a swamp tentacle corpse";
    c.name = "a swamp tentacle";
    c.poison_immune = 2;
    c.loot = "Average";
    c.on_create(|m| {
        m.body = 66;
        m.sound = 0x160;
        m.str = 96..=120;
        m.dex = 66..=85;
        m.int = 16..=30;
        m.hits = 58..=72;
        m.mana = 0;
        m.damage = 6..=12;
        m.damage_type("Physical", 40);
        m.damage_type("Poison", 60);
        m.resist("Physical", 25, 35);
        m.resist("Fire", 10, 20);
        m.resist("Cold", 10, 20);
        m.resist("Poison", 60, 80);
        m.resist("Energy", 10, 20);
        m.skill("MagicResist", 15.1, 20.0);
        m.skill("Tactics", 65.1, 80.0);
        m.skill("Wrestling", 65.1, 80.0);
        m.fame = 3000;
        m.karma = -3000;
        m.armor = 28;
        m.pack = [#{ reg: 3 }];
    });
});

mobile("WhippingVine", |c| {
    c.corpse_name = "a whipping vine corpse";
    c.name = "a whipping vine";
    c.poison_immune = 4;
    c.bard_immune = #{ atLeast: "AOS", then: false, "else": true };
    c.on_create(|m| {
        m.body = 8;
        m.hue = 0x851;
        m.sound = 0x160;
        m.str = 251..=300;
        m.dex = 76..=100;
        m.int = 26..=40;
        m.mana = 0;
        m.damage = 7..=25;
        m.damage_type("Physical", 70);
        m.damage_type("Poison", 30);
        m.resist("Physical", 75, 85);
        m.resist("Fire", 15, 25);
        m.resist("Cold", 15, 25);
        m.resist("Poison", 75, 85);
        m.resist("Energy", 35, 45);
        m.skill("MagicResist", 70.0);
        m.skill("Tactics", 70.0);
        m.skill("Wrestling", 70.0);
        m.fame = 1000;
        m.karma = -1000;
        m.armor = 45;
        m.pack = [
            #{ reg: 3 },
            #{ item: "Server.Items.FertileDirt", amount: [1, 10] },
            #{ chance: 0.2, item: "Server.Items.ExecutionersCap" },
            #{ item: "Server.Items.Vines" },
        ];
    });
});

mobile("AncientWyrm", |c| {
    c.ai = "Mage";
    c.corpse_name = "a dragon corpse";
    c.name = "an ancient wyrm";
    c.hides = 40;
    c.meat = 19;
    c.scales = 12;
    c.poison_immune = 1;
    c.treasure_map_level = 5;
    c.can_fly = true;
    c.loot = [["FilthyRich", 3], ["Gems", 5]];
    c.reacquire_on_movement = true;
    c.auto_dispel = true;
    c.hide_type = "Barbed";
    c.hit_poison = #{ coin: [0, 1] };
    c.monster_abilities = ["FireBreath"];
    c.idle_sound = 723;
    c.hurt_sound = 721;
    c.on_create(|m| {
        m.body = 46;
        m.sound = 0x16A;
        m.str = 1096..=1185;
        m.dex = 86..=175;
        m.int = 686..=775;
        m.hits = 658..=711;
        m.damage = 29..=35;
        m.damage_type("Physical", 75);
        m.damage_type("Fire", 25);
        m.resist("Physical", 65, 75);
        m.resist("Fire", 80, 90);
        m.resist("Cold", 70, 80);
        m.resist("Poison", 60, 70);
        m.resist("Energy", 60, 70);
        m.skill("EvalInt", 80.1, 100.0);
        m.skill("Magery", 80.1, 100.0);
        m.skill("Meditation", 52.5, 75.0);
        m.skill("MagicResist", 100.5, 150.0);
        m.skill("Tactics", 97.6, 100.0);
        m.skill("Wrestling", 97.6, 100.0);
        m.fame = 22500;
        m.karma = -22500;
        m.armor = 70;
    });
});

mobile("Dragon", |c| {
    c.ai = "Mage";
    c.corpse_name = "a dragon corpse";
    c.name = "a dragon";
    c.treasure_map_level = 4;
    c.meat = 19;
    c.hides = 20;
    c.scales = 7;
    c.can_fly = true;
    c.loot = [["FilthyRich", 2], ["Gems", 8]];
    c.hide_type = "Barbed";
    c.favorite_food = ["Meat"];
    c.can_anger_on_tame = true;
    c.monster_abilities = ["FireBreath"];
    c.on_create(|m| {
        m.bodies = [12, 59];
        m.sound = 0x16A;
        m.str = 796..=825;
        m.dex = 86..=105;
        m.int = 436..=475;
        m.hits = 478..=495;
        m.damage = 16..=22;
        m.damage_type("Physical", 100);
        m.resist("Physical", 55, 65);
        m.resist("Fire", 60, 70);
        m.resist("Cold", 30, 40);
        m.resist("Poison", 25, 35);
        m.resist("Energy", 35, 45);
        m.skill("EvalInt", 30.1, 40.0);
        m.skill("Magery", 30.1, 40.0);
        m.skill("MagicResist", 99.1, 100.0);
        m.skill("Tactics", 97.6, 100.0);
        m.skill("Wrestling", 90.1, 92.5);
        m.fame = 15000;
        m.karma = -15000;
        m.armor = 60;
        m.tamable = true;
        m.control_slots = 3;
        m.min_tame_skill = 93.9;
    });
});

mobile("OphidianMatriarch", |c| {
    c.ai = "Mage";
    c.corpse_name = "an ophidian corpse";
    c.name = "an ophidian matriarch";
    c.poison_immune = 2;
    c.treasure_map_level = 4;
    c.loot = ["Rich", ["Average", 2], ["MedScrolls", 2]];
    c.opposition = "TerathansAndOphidians";
    c.on_create(|m| {
        m.body = 87;
        m.sound = 0x284;
        m.str = 416..=505;
        m.dex = 96..=115;
        m.int = 366..=455;
        m.hits = 250..=303;
        m.damage = 11..=13;
        m.damage_type("Physical", 100);
        m.resist("Physical", 45, 55);
        m.resist("Fire", 30, 40);
        m.resist("Cold", 35, 45);
        m.resist("Poison", 40, 50);
        m.resist("Energy", 35, 45);
        m.skill("EvalInt", 90.1, 100.0);
        m.skill("Magery", 90.1, 100.0);
        m.skill("Meditation", 5.4, 25.0);
        m.skill("MagicResist", 90.1, 100.0);
        m.skill("Tactics", 50.1, 70.0);
        m.skill("Wrestling", 60.1, 80.0);
        m.fame = 16000;
        m.karma = -16000;
        m.armor = 50;
    });
});

mobile("ShadowWyrm", |c| {
    c.ai = "Mage";
    c.corpse_name = "a shadow wyrm corpse";
    c.name = "a shadow wyrm";
    c.poison_immune = 3;
    c.treasure_map_level = 5;
    c.meat = 19;
    c.hides = 20;
    c.scales = 10;
    c.can_fly = true;
    c.loot = [["FilthyRich", 3], ["Gems", 5]];
    c.reacquire_on_movement = true;
    c.auto_dispel = true;
    c.hit_poison = 3;
    c.scale_type = "Black";
    c.hide_type = "Barbed";
    c.monster_abilities = ["FireBreath"];
    c.idle_sound = 725;
    c.hurt_sound = 721;
    c.on_create(|m| {
        m.body = 106;
        m.sound = 0x16A;
        m.str = 898..=1030;
        m.dex = 68..=200;
        m.int = 488..=620;
        m.hits = 558..=599;
        m.damage = 29..=35;
        m.damage_type("Physical", 75);
        m.damage_type("Cold", 25);
        m.resist("Physical", 65, 75);
        m.resist("Fire", 50, 60);
        m.resist("Cold", 45, 55);
        m.resist("Poison", 20, 30);
        m.resist("Energy", 50, 60);
        m.skill("EvalInt", 80.1, 100.0);
        m.skill("Magery", 80.1, 100.0);
        m.skill("Meditation", 52.5, 75.0);
        m.skill("MagicResist", 100.3, 130.0);
        m.skill("Tactics", 97.6, 100.0);
        m.skill("Wrestling", 97.6, 100.0);
        m.fame = 22500;
        m.karma = -22500;
        m.armor = 70;
    });
});

mobile("SkeletalDragon", |c| {
    c.ai = "Mage";
    c.corpse_name = "a skeletal dragon corpse";
    c.name = "a skeletal dragon";
    c.poison_immune = 4;
    c.bleed_immune = true;
    c.meat = 19;
    c.hides = 20;
    c.loot = [["FilthyRich", 4], ["Gems", 5]];
    c.reacquire_on_movement = true;
    c.auto_dispel = true;
    c.hide_type = "Barbed";
    c.bonus_pet_damage_scalar = #{ atLeast: "SE", then: 3.0, "else": 1.0 };
    c.monster_abilities = ["ColdBreath"];
    c.on_create(|m| {
        m.body = 104;
        m.sound = 0x488;
        m.str = 898..=1030;
        m.dex = 68..=200;
        m.int = 488..=620;
        m.hits = 558..=599;
        m.damage = 29..=35;
        m.damage_type("Physical", 75);
        m.damage_type("Fire", 25);
        m.resist("Physical", 75, 80);
        m.resist("Fire", 40, 60);
        m.resist("Cold", 40, 60);
        m.resist("Poison", 70, 80);
        m.resist("Energy", 40, 60);
        m.skill("EvalInt", 80.1, 100.0);
        m.skill("Magery", 80.1, 100.0);
        m.skill("MagicResist", 100.3, 130.0);
        m.skill("Tactics", 97.6, 100.0);
        m.skill("Wrestling", 97.6, 100.0);
        m.skill("Necromancy", 120.1, 130.0);
        m.skill("SpiritSpeak", 120.1, 130.0);
        m.fame = 22500;
        m.karma = -22500;
        m.armor = 80;
    });
});

mobile("Drake", |c| {
    c.corpse_name = "a drake corpse";
    c.name = "a drake";
    c.treasure_map_level = 2;
    c.meat = 10;
    c.hides = 20;
    c.scales = 2;
    c.can_fly = true;
    c.loot = ["Rich", ["MedScrolls", 2]];
    c.reacquire_on_movement = true;
    c.hide_type = "Horned";
    c.favorite_food = ["Meat", "Fish"];
    c.monster_abilities = ["FireBreath"];
    c.on_create(|m| {
        m.bodies = [60, 61];
        m.sound = 0x16A;
        m.str = 401..=430;
        m.dex = 133..=152;
        m.int = 101..=140;
        m.hits = 241..=258;
        m.damage = 11..=17;
        m.damage_type("Physical", 80);
        m.damage_type("Fire", 20);
        m.resist("Physical", 45, 50);
        m.resist("Fire", 50, 60);
        m.resist("Cold", 40, 50);
        m.resist("Poison", 20, 30);
        m.resist("Energy", 30, 40);
        m.skill("MagicResist", 65.1, 80.0);
        m.skill("Tactics", 65.1, 90.0);
        m.skill("Wrestling", 65.1, 80.0);
        m.fame = 5500;
        m.karma = -5500;
        m.armor = 46;
        m.tamable = true;
        m.control_slots = 2;
        m.min_tame_skill = 84.3;
        m.pack = [#{ reg: 3 }];
    });
});

mobile("Lizardman", |c| {
    c.corpse_name = "a lizardman corpse";
    c.can_rummage_corpses = true;
    c.meat = 1;
    c.hides = 12;
    c.loot = "Meager";
    c.hide_type = "Spined";
    c.speech = "Lizardman";
    c.on_create(|m| {
        m.name = random_name("lizardman");
        m.bodies = [35, 36];
        m.sound = 0x1A1;
        m.str = 96..=120;
        m.dex = 86..=105;
        m.int = 36..=60;
        m.hits = 58..=72;
        m.damage = 5..=7;
        m.damage_type("Physical", 100);
        m.resist("Physical", 25, 30);
        m.resist("Fire", 5, 10);
        m.resist("Cold", 5, 10);
        m.resist("Poison", 10, 20);
        m.skill("MagicResist", 35.1, 60.0);
        m.skill("Tactics", 55.1, 80.0);
        m.skill("Wrestling", 50.1, 70.0);
        m.fame = 1500;
        m.karma = -1500;
        m.armor = 28;
    });
});

mobile("StoneHarpy", |c| {
    c.corpse_name = "a stone harpy corpse";
    c.name = "a stone harpy";
    c.meat = 1;
    c.feathers = 50;
    c.can_fly = true;
    c.loot = [["Average", 2], ["Gems", 2]];
    c.attack_sound = 916;
    c.anger_sound = 916;
    c.death_sound = 917;
    c.hurt_sound = 919;
    c.idle_sound = 918;
    c.on_create(|m| {
        m.body = 73;
        m.sound = 0x192;
        m.str = 296..=320;
        m.dex = 86..=110;
        m.int = 51..=75;
        m.hits = 178..=192;
        m.mana = 0;
        m.damage = 8..=16;
        m.damage_type("Physical", 75);
        m.damage_type("Poison", 25);
        m.resist("Physical", 45, 55);
        m.resist("Fire", 20, 30);
        m.resist("Cold", 10, 20);
        m.resist("Poison", 30, 40);
        m.resist("Energy", 30, 40);
        m.skill("MagicResist", 50.1, 65.0);
        m.skill("Tactics", 70.1, 100.0);
        m.skill("Wrestling", 70.1, 100.0);
        m.fame = 4500;
        m.karma = -4500;
        m.armor = 50;
    });
});

mobile("Wyvern", |c| {
    c.corpse_name = "a wyvern corpse";
    c.name = "a wyvern";
    c.poison_immune = 3;
    c.treasure_map_level = 2;
    c.meat = 10;
    c.hides = 20;
    c.can_fly = true;
    c.loot = ["Average", "Meager", "MedScrolls"];
    c.reacquire_on_movement = true;
    c.hit_poison = 3;
    c.hide_type = "Horned";
    c.attack_sound = 713;
    c.anger_sound = 718;
    c.death_sound = 716;
    c.hurt_sound = 721;
    c.idle_sound = 725;
    c.on_create(|m| {
        m.body = 62;
        m.sound = 0x16A;
        m.str = 202..=240;
        m.dex = 153..=172;
        m.int = 51..=90;
        m.hits = 125..=141;
        m.damage = 8..=19;
        m.damage_type("Physical", 50);
        m.damage_type("Poison", 50);
        m.resist("Physical", 35, 45);
        m.resist("Fire", 30, 40);
        m.resist("Cold", 20, 30);
        m.resist("Poison", 90, 100);
        m.resist("Energy", 30, 40);
        m.skill("Poisoning", 60.1, 80.0);
        m.skill("MagicResist", 65.1, 80.0);
        m.skill("Tactics", 65.1, 90.0);
        m.skill("Wrestling", 65.1, 80.0);
        m.fame = 4000;
        m.karma = -4000;
        m.armor = 40;
        m.pack = [#{ item: "Server.Items.LesserPoisonPotion" }];
    });
});

mobile("Raptor", |c| {
    c.corpse_name = "a raptor corpse";
    c.name = "a raptor";
    c.treasure_map_level = 3;
    c.meat = 7;
    c.hides = 11;
    c.loot = [["Rich", 2]];
    c.hide_type = "Horned";
    c.pack_instinct = ["Ostard"];
    c.weapon_ability = "BleedAttack";
    c.idle_sound = 1573;
    c.anger_sound = 1570;
    c.hurt_sound = 1572;
    c.death_sound = 1571;
    c.on_create(|m| {
        m.body = 730;
        m.str = 404..=471;
        m.dex = 132..=155;
        m.int = 105..=145;
        m.hits = 343..=400;
        m.damage = 11..=17;
        m.damage_type("Physical", 100);
        m.resist("Physical", 45, 50);
        m.resist("Fire", 50, 60);
        m.resist("Cold", 40, 50);
        m.resist("Poison", 20, 30);
        m.resist("Energy", 30, 40);
        m.skill("MagicResist", 75.1, 90.0);
        m.skill("Tactics", 75.1, 100.0);
        m.skill("Wrestling", 70.1, 95.1);
        m.fame = 7500;
        m.karma = -7500;
        m.tamable = true;
        m.min_tame_skill = 107.1;
        m.control_slots = 2;
    });
});

mobile("Slith", |c| {
    c.corpse_name = "a slith corpse";
    c.name = "a slith";
    c.treasure_map_level = 2;
    c.meat = 6;
    c.hides = 10;
    c.loot = [["Average", 2]];
    c.monster_abilities = ["FireBreath"];
    c.on_create(|m| {
        m.body = 734;
        m.str = 129..=136;
        m.dex = 72..=75;
        m.int = 12..=13;
        m.hits = 84..=85;
        m.damage = 6..=24;
        m.damage_type("Physical", 100);
        m.resist("Physical", 35, 40);
        m.resist("Fire", 35, 45);
        m.resist("Poison", 25, 35);
        m.resist("Energy", 25, 30);
        m.skill("MagicResist", 59.1, 63.5);
        m.skill("Tactics", 74.6, 76.4);
        m.skill("Wrestling", 62.0, 77.1);
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = 80.7;
    });
});

mobile("StoneSlith", |c| {
    c.corpse_name = "a slith corpse";
    c.name = "a stone slith";
    c.meat = 1;
    c.hides = 12;
    c.loot = [["Average", 2]];
    c.hide_type = "Spined";
    c.weapon_ability = "BleedAttack";
    c.monster_abilities = ["GraspingClaw", "TailSwipe"];
    c.on_create(|m| {
        m.body = 734;
        m.str = 250..=300;
        m.dex = 76..=90;
        m.int = 34..=69;
        m.hits = 154..=166;
        m.stam = 76..=90;
        m.mana = 34..=69;
        m.damage = 6..=24;
        m.damage_type("Physical", 100);
        m.resist("Physical", 50, 55);
        m.resist("Fire", 20, 30);
        m.resist("Cold", 10, 20);
        m.resist("Poison", 30, 40);
        m.resist("Energy", 30, 40);
        m.skill("MagicResist", 86.8, 95.1);
        m.skill("Tactics", 82.6, 88.6);
        m.skill("Wrestling", 75.8, 87.4);
        m.skill("Anatomy", 0.0, 2.9);
        m.tamable = true;
        m.min_tame_skill = 65.1;
        m.control_slots = 2;
    });
});

mobile("ToxicSlith", |c| {
    c.corpse_name = "a slith corpse";
    c.name = "a toxic slith";
    c.meat = 6;
    c.hides = 11;
    c.loot = [["Average", 2]];
    c.hide_type = "Horned";
    c.monster_abilities = ["FireBreath"];
    c.on_create(|m| {
        m.body = 734;
        m.hue = 0x1DC;
        m.str = 223..=306;
        m.dex = 231..=258;
        m.int = 30..=35;
        m.hits = 197..=215;
        m.stam = 231..=258;
        m.damage = 6..=24;
        m.damage_type("Physical", 100);
        m.resist("Physical", 35, 45);
        m.resist("Fire", 0, 9);
        m.resist("Cold", 5, 10);
        m.resist("Poison", 100, 100);
        m.resist("Energy", 5, 7);
        m.skill("MagicResist", 95.4, 98.3);
        m.skill("Tactics", 85.5, 90.9);
        m.skill("Wrestling", 90.4, 95.1);
        m.skill("Poisoning", 90.0, 110.0);
    });
});

mobile("KazeKemono", |c| {
    c.ai = "Mage";
    c.corpse_name = "a kaze kemono corpse";
    c.name = "a kaze kemono";
    c.bleed_immune = true;
    c.loot = [["Rich", 3]];
    c.on_create(|m| {
        m.body = 196;
        m.sound = 0x28F;
        m.str = 201..=275;
        m.dex = 101..=155;
        m.int = 101..=105;
        m.hits = 251..=330;
        m.damage = 15..=20;
        m.damage_type("Physical", 70);
        m.damage_type("Fire", 10);
        m.damage_type("Cold", 10);
        m.damage_type("Poison", 10);
        m.resist("Physical", 50, 70);
        m.resist("Fire", 30, 60);
        m.resist("Cold", 30, 60);
        m.resist("Poison", 50, 70);
        m.resist("Energy", 60, 80);
        m.skill("MagicResist", 110.1, 125.0);
        m.skill("Tactics", 55.1, 65.0);
        m.skill("Wrestling", 85.1, 95.0);
        m.skill("Anatomy", 25.1, 35.0);
        m.skill("Magery", 95.1, 105.0);
        m.fame = 8000;
        m.karma = -8000;
    });
});

mobile("RaiJu", |c| {
    c.corpse_name = "a rai-ju corpse";
    c.name = "a Rai-Ju";
    c.bleed_immune = true;
    c.loot = [["Rich", 2], ["Gems", 2]];
    c.on_create(|m| {
        m.body = 199;
        m.sound = 0x346;
        m.str = 151..=225;
        m.dex = 81..=135;
        m.int = 176..=180;
        m.hits = 201..=280;
        m.damage = 12..=15;
        m.damage_type("Physical", 10);
        m.damage_type("Fire", 10);
        m.damage_type("Cold", 10);
        m.damage_type("Poison", 10);
        m.damage_type("Energy", 60);
        m.resist("Physical", 45, 65);
        m.resist("Fire", 70, 85);
        m.resist("Cold", 30, 60);
        m.resist("Poison", 50, 70);
        m.resist("Energy", 60, 80);
        m.skill("Wrestling", 85.1, 95.0);
        m.skill("Tactics", 55.1, 65.0);
        m.skill("MagicResist", 110.1, 125.0);
        m.skill("Anatomy", 25.1, 35.0);
        m.fame = 8000;
        m.karma = -8000;
    });
});

mobile("VampireBat", |c| {
    c.corpse_name = "a vampire bat corpse";
    c.name = "a vampire bat";
    c.loot = "Poor";
    c.idle_sound = 667;
    c.on_create(|m| {
        m.body = 317;
        m.sound = 0x270;
        m.str = 91..=110;
        m.dex = 91..=115;
        m.int = 26..=50;
        m.hits = 55..=66;
        m.damage = 7..=9;
        m.damage_type("Physical", 80);
        m.damage_type("Poison", 20);
        m.resist("Physical", 35, 45);
        m.resist("Fire", 15, 25);
        m.resist("Cold", 15, 25);
        m.resist("Poison", 60, 70);
        m.resist("Energy", 40, 50);
        m.skill("MagicResist", 70.1, 95.0);
        m.skill("Tactics", 55.1, 80.0);
        m.skill("Wrestling", 30.1, 55.0);
        m.fame = 1000;
        m.karma = -1000;
        m.armor = 14;
    });
});

mobile("WandererOfTheVoid", |c| {
    c.ai = "Mage";
    c.corpse_name = "a wanderer of the void corpse";
    c.name = "a wanderer of the void";
    c.bleed_immune = true;
    c.poison_immune = 4;
    c.loot = "FilthyRich";
    c.treasure_map_level = #{ atLeast: "AOS", then: 4, "else": 1 };
    c.on_create(|m| {
        m.body = 316;
        m.sound = 0x179;
        m.str = 111..=200;
        m.dex = 101..=125;
        m.int = 301..=390;
        m.hits = 351..=400;
        m.damage = 11..=13;
        m.damage_type("Physical", 0);
        m.damage_type("Cold", 15);
        m.damage_type("Energy", 85);
        m.resist("Physical", 40, 50);
        m.resist("Fire", 15, 25);
        m.resist("Cold", 40, 50);
        m.resist("Poison", 50, 75);
        m.resist("Energy", 40, 50);
        m.skill("EvalInt", 60.1, 70.0);
        m.skill("Magery", 60.1, 70.0);
        m.skill("Meditation", 60.1, 70.0);
        m.skill("MagicResist", 50.1, 75.0);
        m.skill("Tactics", 60.1, 70.0);
        m.skill("Wrestling", 60.1, 70.0);
        m.fame = 20000;
        m.karma = -20000;
        m.armor = 44;
        m.creature_item = block([
            #{ item: "Server.Items.TreasureMap" },
            #{ args: [3, #{ map: "Trammel" }] },
            #{ times: #{ min: 2, max: 3 } },
        ]);
    });
});

mobile("AntLion", |c| {
    c.corpse_name = "an ant lion corpse";
    c.name = "an ant lion";
    c.loot = [["Average", 2]];
    c.anger_sound = 90;
    c.idle_sound = 90;
    c.attack_sound = 356;
    c.hurt_sound = 391;
    c.death_sound = 442;
    c.on_create(|m| {
        m.body = 787;
        m.sound = 0x3EE;
        m.str = 296..=320;
        m.dex = 81..=105;
        m.int = 36..=60;
        m.hits = 151..=162;
        m.damage = 7..=21;
        m.damage_type("Physical", 70);
        m.damage_type("Poison", 30);
        m.resist("Physical", 45, 60);
        m.resist("Fire", 25, 35);
        m.resist("Cold", 30, 40);
        m.resist("Poison", 40, 50);
        m.resist("Energy", 30, 35);
        m.skill("MagicResist", 70.0);
        m.skill("Tactics", 90.0);
        m.skill("Wrestling", 90.0);
        m.fame = 4500;
        m.karma = -4500;
        m.armor = 45;
        m.pack = [#{ item: "Server.Items.Bone", amount: 3 }, #{ item: "Server.Items.FertileDirt", amount: [1, 5] }];
        if era("ML") {
            if chance(0.33) {
                m.creature_item = block([#{ loot: "PeculiarSeed" }, #{ group: 2 }]);
            }
        }
        m.creature_item = block([
            #{ pick: [
                "Server.Items.DullCopperOre",
                "Server.Items.ShadowIronOre",
                "Server.Items.CopperOre",
                "Server.Items.BronzeOre",
            ] },
            #{ setAmount: #{ min: 1, max: 10 } },
            #{ itemId: 6585 },
        ]);
    });
});

mobile("BlackSolenInfiltratorQueen", |c| {
    c.corpse_name = "a solen infiltrator corpse";
    c.name = "a black solen infiltrator";
    c.loot = "Rich";
    c.anger_sound = 601;
    c.idle_sound = 601;
    c.attack_sound = 405;
    c.hurt_sound = 592;
    c.death_sound = 603;
    c.on_create(|m| {
        m.body = 807;
        m.sound = 0x3BF;
        m.hue = 0x453;
        m.str = 326..=350;
        m.dex = 141..=165;
        m.int = 96..=120;
        m.hits = 151..=162;
        m.damage = 10..=15;
        m.damage_type("Physical", 70);
        m.damage_type("Poison", 30);
        m.resist("Physical", 30, 40);
        m.resist("Fire", 30, 35);
        m.resist("Cold", 25, 35);
        m.resist("Poison", 35, 40);
        m.resist("Energy", 25, 30);
        m.skill("MagicResist", 90.0);
        m.skill("Tactics", 90.0);
        m.skill("Wrestling", 90.0);
        m.fame = 6500;
        m.karma = -6500;
        m.armor = 50;
        m.creature_one_in = #{
            n: 100,
            then: #{
                creatureItem: #{
                    item: "Server.Items.PicnicBasket",
                    contents: [
                        #{ item: "Server.Items.BeverageBottle", args: [#{ enum: 4 }] },
                        #{ item: "Server.Items.CheeseWedge" },
                    ],
                },
            },
        };
        if chance(0.05) {
            m.pack = [#{ item: "Server.Items.ZoogiFungus", amount: 16 }];
        } else {
            m.pack = [#{ item: "Server.Items.ZoogiFungus", amount: 4 }];
        }
    });
});

mobile("BlackSolenInfiltratorWarrior", |c| {
    c.corpse_name = "a solen infiltrator corpse";
    c.name = "a black solen infiltrator";
    c.loot = [["Average", 2], ["Gems", #{ min: 1, max: 4 }]];
    c.anger_sound = 181;
    c.idle_sound = 181;
    c.attack_sound = 649;
    c.hurt_sound = 188;
    c.death_sound = 228;
    c.on_create(|m| {
        m.body = 806;
        m.sound = 0x3BF;
        m.hue = 0x453;
        m.str = 206..=230;
        m.dex = 121..=145;
        m.int = 66..=90;
        m.hits = 96..=107;
        m.damage = 5..=15;
        m.damage_type("Physical", 80);
        m.damage_type("Poison", 20);
        m.resist("Physical", 20, 35);
        m.resist("Fire", 20, 35);
        m.resist("Cold", 10, 25);
        m.resist("Poison", 20, 35);
        m.resist("Energy", 10, 25);
        m.skill("MagicResist", 80.0);
        m.skill("Tactics", 80.0);
        m.skill("Wrestling", 80.0);
        m.fame = 3000;
        m.karma = -3000;
        m.armor = 40;
        m.creature_one_in = #{
            n: 100,
            then: #{
                creatureItem: #{
                    item: "Server.Items.PicnicBasket",
                    contents: [
                        #{ item: "Server.Items.BeverageBottle", args: [#{ enum: 4 }] },
                        #{ item: "Server.Items.CheeseWedge" },
                    ],
                },
            },
        };
        if chance(0.05) {
            m.pack = [#{ item: "Server.Items.ZoogiFungus", amount: 13 }];
        } else {
            m.pack = [#{ item: "Server.Items.ZoogiFungus", amount: 3 }];
        }
    });
});

mobile("BlackSolenQueen", |c| {
    c.corpse_name = "a solen queen corpse";
    c.name = "a black solen queen";
    c.loot = "Rich";
    c.anger_sound = 601;
    c.idle_sound = 601;
    c.attack_sound = 405;
    c.hurt_sound = 592;
    c.death_sound = 603;
    c.on_create(|m| {
        m.creature_setup = "BurstSac";
        m.body = 807;
        m.sound = 0x3BF;
        m.hue = 0x453;
        m.str = 296..=320;
        m.dex = 121..=145;
        m.int = 76..=100;
        m.hits = 151..=162;
        m.damage = 10..=15;
        m.damage_type("Physical", 70);
        m.damage_type("Poison", 30);
        m.resist("Physical", 30, 40);
        m.resist("Fire", 30, 35);
        m.resist("Cold", 25, 35);
        m.resist("Poison", 35, 40);
        m.resist("Energy", 25, 30);
        m.skill("MagicResist", 70.0);
        m.skill("Tactics", 90.0);
        m.skill("Wrestling", 90.0);
        m.fame = 4500;
        m.karma = -4500;
        m.armor = 45;
        m.creature_one_in = #{
            n: 100,
            then: #{
                creatureItem: #{
                    item: "Server.Items.PicnicBasket",
                    contents: [
                        #{ item: "Server.Items.BeverageBottle", args: [#{ enum: 4 }] },
                        #{ item: "Server.Items.CheeseWedge" },
                    ],
                },
            },
        };
        if chance(0.95) {
            m.pack = [#{ item: "Server.Items.ZoogiFungus", amount: 5 }];
        } else {
            m.pack = [#{ item: "Server.Items.ZoogiFungus", amount: 25 }];
        }
        m.pack = [#{ chance: 0.05, item: "Server.Items.BallOfSummoning" }];
    });
});

mobile("BlackSolenWarrior", |c| {
    c.corpse_name = "a solen warrior corpse";
    c.name = "a black solen warrior";
    c.loot = ["Rich", ["Gems", #{ min: 1, max: 4 }]];
    c.anger_sound = 181;
    c.idle_sound = 181;
    c.attack_sound = 649;
    c.hurt_sound = 188;
    c.death_sound = 228;
    c.on_create(|m| {
        m.creature_setup = "BurstSac";
        m.body = 806;
        m.sound = 0x3BF;
        m.hue = 0x453;
        m.str = 196..=220;
        m.dex = 101..=125;
        m.int = 36..=60;
        m.hits = 96..=107;
        m.damage = 5..=15;
        m.damage_type("Physical", 80);
        m.damage_type("Poison", 20);
        m.resist("Physical", 20, 35);
        m.resist("Fire", 20, 35);
        m.resist("Cold", 10, 25);
        m.resist("Poison", 20, 35);
        m.resist("Energy", 10, 25);
        m.skill("MagicResist", 60.0);
        m.skill("Tactics", 80.0);
        m.skill("Wrestling", 80.0);
        m.fame = 3000;
        m.karma = -3000;
        m.armor = 35;
        m.creature_one_in = #{
            n: 100,
            then: #{
                creatureItem: #{
                    item: "Server.Items.PicnicBasket",
                    contents: [
                        #{ item: "Server.Items.BeverageBottle", args: [#{ enum: 4 }] },
                        #{ item: "Server.Items.CheeseWedge" },
                    ],
                },
            },
        };
        if chance(0.05) {
            m.pack = [#{ item: "Server.Items.ZoogiFungus", amount: 13 }];
        } else {
            m.pack = [#{ item: "Server.Items.ZoogiFungus", amount: 3 }];
        }
        m.pack = [#{ chance: 0.05, item: "Server.Items.BraceletOfBinding" }];
    });
});

mobile("BlackSolenWorker", |c| {
    c.corpse_name = "a solen worker corpse";
    c.name = "a black solen worker";
    c.loot = [["Gems", #{ min: 1, max: 2 }]];
    c.anger_sound = 617;
    c.idle_sound = 617;
    c.attack_sound = 390;
    c.hurt_sound = 446;
    c.death_sound = 142;
    c.on_create(|m| {
        m.body = 805;
        m.sound = 0x3BF;
        m.hue = 0x453;
        m.str = 96..=120;
        m.dex = 81..=105;
        m.int = 36..=60;
        m.hits = 58..=72;
        m.damage = 5..=7;
        m.damage_type("Physical", 100);
        m.resist("Physical", 25, 30);
        m.resist("Fire", 20, 30);
        m.resist("Cold", 10, 20);
        m.resist("Poison", 10, 20);
        m.resist("Energy", 20, 30);
        m.skill("MagicResist", 60.0);
        m.skill("Tactics", 65.0);
        m.skill("Wrestling", 60.0);
        m.fame = 1500;
        m.karma = -1500;
        m.armor = 28;
        m.creature_gold = block([#{ from: 100 }, #{ count: 180 }]);
        m.creature_one_in = #{
            n: 100,
            then: #{
                creatureItem: #{
                    item: "Server.Items.PicnicBasket",
                    contents: [
                        #{ item: "Server.Items.BeverageBottle", args: [#{ enum: 4 }] },
                        #{ item: "Server.Items.CheeseWedge" },
                    ],
                },
            },
        };
        m.pack = [#{ item: "Server.Items.ZoogiFungus" }];
    });
});

mobile("RedSolenInfiltratorQueen", |c| {
    c.corpse_name = "a solen infiltrator corpse";
    c.name = "a red solen infiltrator";
    c.loot = "Rich";
    c.anger_sound = 601;
    c.idle_sound = 601;
    c.attack_sound = 405;
    c.hurt_sound = 592;
    c.death_sound = 603;
    c.on_create(|m| {
        m.body = 783;
        m.sound = 0x3BF;
        m.str = 326..=350;
        m.dex = 141..=165;
        m.int = 96..=120;
        m.hits = 151..=162;
        m.damage = 10..=15;
        m.damage_type("Physical", 70);
        m.damage_type("Poison", 30);
        m.resist("Physical", 30, 40);
        m.resist("Fire", 30, 35);
        m.resist("Cold", 25, 35);
        m.resist("Poison", 35, 40);
        m.resist("Energy", 25, 30);
        m.skill("MagicResist", 90.0);
        m.skill("Tactics", 90.0);
        m.skill("Wrestling", 90.0);
        m.fame = 6500;
        m.karma = -6500;
        m.armor = 50;
        m.creature_one_in = #{
            n: 100,
            then: #{
                creatureItem: #{
                    item: "Server.Items.PicnicBasket",
                    contents: [
                        #{ item: "Server.Items.BeverageBottle", args: [#{ enum: 4 }] },
                        #{ item: "Server.Items.CheeseWedge" },
                    ],
                },
            },
        };
        if chance(0.95) {
            m.pack = [#{ item: "Server.Items.ZoogiFungus", amount: 4 }];
        } else {
            m.pack = [#{ item: "Server.Items.ZoogiFungus", amount: 16 }];
        }
    });
});

mobile("RedSolenInfiltratorWarrior", |c| {
    c.corpse_name = "a solen infiltrator corpse";
    c.name = "a red solen infiltrator";
    c.loot = [["Average", 2], ["Gems", #{ min: 1, max: 4 }]];
    c.anger_sound = 181;
    c.idle_sound = 181;
    c.attack_sound = 649;
    c.hurt_sound = 188;
    c.death_sound = 228;
    c.on_create(|m| {
        m.body = 782;
        m.sound = 0x3BF;
        m.str = 206..=230;
        m.dex = 121..=145;
        m.int = 66..=90;
        m.hits = 96..=107;
        m.damage = 5..=15;
        m.damage_type("Physical", 80);
        m.damage_type("Poison", 20);
        m.resist("Physical", 20, 35);
        m.resist("Fire", 20, 35);
        m.resist("Cold", 10, 25);
        m.resist("Poison", 20, 35);
        m.resist("Energy", 10, 25);
        m.skill("MagicResist", 80.0);
        m.skill("Tactics", 80.0);
        m.skill("Wrestling", 80.0);
        m.fame = 3000;
        m.karma = -3000;
        m.armor = 40;
        m.creature_one_in = #{
            n: 100,
            then: #{
                creatureItem: #{
                    item: "Server.Items.PicnicBasket",
                    contents: [
                        #{ item: "Server.Items.BeverageBottle", args: [#{ enum: 4 }] },
                        #{ item: "Server.Items.CheeseWedge" },
                    ],
                },
            },
        };
        if chance(0.95) {
            m.pack = [#{ item: "Server.Items.ZoogiFungus", amount: 3 }];
        } else {
            m.pack = [#{ item: "Server.Items.ZoogiFungus", amount: 13 }];
        }
    });
});

mobile("RedSolenQueen", |c| {
    c.corpse_name = "a solen queen corpse";
    c.name = "a red solen queen";
    c.loot = "Rich";
    c.anger_sound = 601;
    c.idle_sound = 601;
    c.attack_sound = 405;
    c.hurt_sound = 592;
    c.death_sound = 603;
    c.on_create(|m| {
        m.creature_setup = "BurstSac";
        m.body = 783;
        m.sound = 0x3BF;
        m.str = 296..=320;
        m.dex = 121..=145;
        m.int = 76..=100;
        m.hits = 151..=162;
        m.damage = 10..=15;
        m.damage_type("Physical", 70);
        m.damage_type("Poison", 30);
        m.resist("Physical", 30, 40);
        m.resist("Fire", 30, 35);
        m.resist("Cold", 25, 35);
        m.resist("Poison", 35, 40);
        m.resist("Energy", 25, 30);
        m.skill("MagicResist", 70.0);
        m.skill("Tactics", 90.0);
        m.skill("Wrestling", 90.0);
        m.fame = 4500;
        m.karma = -4500;
        m.armor = 45;
        m.creature_one_in = #{
            n: 100,
            then: #{
                creatureItem: #{
                    item: "Server.Items.PicnicBasket",
                    contents: [
                        #{ item: "Server.Items.BeverageBottle", args: [#{ enum: 4 }] },
                        #{ item: "Server.Items.CheeseWedge" },
                    ],
                },
            },
        };
        if chance(0.95) {
            m.pack = [#{ item: "Server.Items.ZoogiFungus", amount: 5 }];
        } else {
            m.pack = [#{ item: "Server.Items.ZoogiFungus", amount: 25 }];
        }
        m.pack = [#{ chance: 0.05, item: "Server.Items.BallOfSummoning" }];
    });
});

mobile("RedSolenWarrior", |c| {
    c.corpse_name = "a solen warrior corpse";
    c.name = "a red solen warrior";
    c.loot = ["Rich", ["Gems", #{ min: 1, max: 4 }]];
    c.anger_sound = 181;
    c.idle_sound = 181;
    c.attack_sound = 649;
    c.hurt_sound = 188;
    c.death_sound = 228;
    c.on_create(|m| {
        m.creature_setup = "BurstSac";
        m.body = 782;
        m.sound = 0x3BF;
        m.str = 196..=220;
        m.dex = 101..=125;
        m.int = 36..=60;
        m.hits = 96..=107;
        m.damage = 5..=15;
        m.damage_type("Physical", 80);
        m.damage_type("Poison", 20);
        m.resist("Physical", 20, 35);
        m.resist("Fire", 20, 35);
        m.resist("Cold", 10, 25);
        m.resist("Poison", 20, 35);
        m.resist("Energy", 10, 25);
        m.skill("MagicResist", 60.0);
        m.skill("Tactics", 80.0);
        m.skill("Wrestling", 80.0);
        m.fame = 3000;
        m.karma = -3000;
        m.armor = 35;
        m.creature_one_in = #{
            n: 100,
            then: #{
                creatureItem: #{
                    item: "Server.Items.PicnicBasket",
                    contents: [
                        #{ item: "Server.Items.BeverageBottle", args: [#{ enum: 4 }] },
                        #{ item: "Server.Items.CheeseWedge" },
                    ],
                },
            },
        };
        if chance(0.95) {
            m.pack = [#{ item: "Server.Items.ZoogiFungus", amount: 3 }];
        } else {
            m.pack = [#{ item: "Server.Items.ZoogiFungus", amount: 13 }];
        }
        m.pack = [#{ chance: 0.05, item: "Server.Items.BraceletOfBinding" }];
    });
});

mobile("RedSolenWorker", |c| {
    c.corpse_name = "a solen worker corpse";
    c.name = "a red solen worker";
    c.loot = [["Gems", #{ min: 1, max: 2 }]];
    c.anger_sound = 617;
    c.idle_sound = 617;
    c.attack_sound = 390;
    c.hurt_sound = 446;
    c.death_sound = 142;
    c.on_create(|m| {
        m.body = 781;
        m.sound = 0x3BF;
        m.str = 96..=120;
        m.dex = 81..=105;
        m.int = 36..=60;
        m.hits = 58..=72;
        m.damage = 5..=7;
        m.damage_type("Physical", 100);
        m.resist("Physical", 25, 30);
        m.resist("Fire", 20, 30);
        m.resist("Cold", 10, 20);
        m.resist("Poison", 10, 20);
        m.resist("Energy", 20, 30);
        m.skill("MagicResist", 60.0);
        m.skill("Tactics", 65.0);
        m.skill("Wrestling", 60.0);
        m.fame = 1500;
        m.karma = -1500;
        m.armor = 28;
        m.creature_gold = block([#{ from: 100 }, #{ count: 180 }]);
        m.creature_one_in = #{
            n: 100,
            then: #{
                creatureItem: #{
                    item: "Server.Items.PicnicBasket",
                    contents: [
                        #{ item: "Server.Items.BeverageBottle", args: [#{ enum: 4 }] },
                        #{ item: "Server.Items.CheeseWedge" },
                    ],
                },
            },
        };
        m.pack = [#{ item: "Server.Items.ZoogiFungus" }];
    });
});

mobile("TerathanWarrior", |c| {
    c.corpse_name = "a terathan warrior corpse";
    c.name = "a terathan warrior";
    c.treasure_map_level = 1;
    c.meat = 4;
    c.opposition = "TerathansAndOphidians";
    c.loot = "Average";
    c.on_create(|m| {
        m.body = 70;
        m.sound = 0x24D;
        m.str = 166..=215;
        m.dex = 96..=145;
        m.int = 41..=65;
        m.hits = 100..=129;
        m.mana = 0;
        m.damage = 7..=17;
        m.damage_type("Physical", 100);
        m.resist("Physical", 30, 35);
        m.resist("Fire", 20, 30);
        m.resist("Cold", 25, 35);
        m.resist("Poison", 30, 40);
        m.resist("Energy", 25, 35);
        m.skill("Poisoning", 60.1, 80.0);
        m.skill("MagicResist", 60.1, 75.0);
        m.skill("Tactics", 80.1, 100.0);
        m.skill("Wrestling", 80.1, 90.0);
        m.fame = 4000;
        m.karma = -4000;
        m.armor = 30;
        if era("ML") {
            if chance(0.33) {
                m.creature_item = block([#{ loot: "PeculiarSeed" }, #{ group: 3 }]);
            }
        }
    });
});

mobile("WaterElemental", |c| {
    c.ai = "Mage";
    c.corpse_name = "a water elemental corpse";
    c.name = "a water elemental";
    c.bleed_immune = true;
    c.treasure_map_level = 2;
    c.loot = ["Average", "Meager", "Potions"];
    c.on_create(|m| {
        m.creature_setup = "Decanter";
        m.body = 16;
        m.sound = 0x116;
        m.str = 126..=155;
        m.dex = 66..=85;
        m.int = 101..=125;
        m.hits = 76..=93;
        m.damage = 7..=9;
        m.damage_type("Physical", 100);
        m.resist("Physical", 35, 45);
        m.resist("Fire", 10, 25);
        m.resist("Cold", 10, 25);
        m.resist("Poison", 60, 70);
        m.resist("Energy", 5, 10);
        m.skill("EvalInt", 60.1, 75.0);
        m.skill("Magery", 60.1, 75.0);
        m.skill("MagicResist", 100.1, 115.0);
        m.skill("Tactics", 50.1, 70.0);
        m.skill("Wrestling", 50.1, 70.0);
        m.fame = 4500;
        m.karma = -4500;
        m.armor = 40;
        m.can_swim = true;
        m.pack = [#{ item: "Server.Items.BlackPearl", amount: 3 }];
    });
});

mobile("Betrayer", |c| {
    c.ai = "Mage";
    c.corpse_name = "a betrayer corpse";
    c.name = "a betrayer";
    c.always_murderer = true;
    c.poison_immune = 4;
    c.meat = 1;
    c.treasure_map_level = 5;
    c.loot = ["FilthyRich", "Rich", "Gems"];
    c.bard_immune = #{ atLeast: "AOS", then: false, "else": true };
    c.death_sound = 1059;
    c.attack_sound = 571;
    c.hurt_sound = 320;
    c.on_create(|m| {
        m.body = 767;
        m.str = 401..=500;
        m.dex = 81..=100;
        m.int = 151..=200;
        m.hits = 241..=300;
        m.damage = 16..=22;
        m.damage_type("Physical", 100);
        m.resist("Physical", 60, 70);
        m.resist("Fire", 60, 70);
        m.resist("Cold", 60, 70);
        m.resist("Poison", 30, 40);
        m.resist("Energy", 20, 30);
        m.skill("Anatomy", 90.1, 100.0);
        m.skill("EvalInt", 90.1, 100.0);
        m.skill("Magery", 50.1, 100.0);
        m.skill("Meditation", 90.1, 100.0);
        m.skill("MagicResist", 120.1, 130.0);
        m.skill("Tactics", 90.1, 100.0);
        m.skill("Wrestling", 90.1, 100.0);
        m.fame = 15000;
        m.karma = -15000;
        m.armor = 65;
        m.creature_speech_hue = block([#{ from: 2 }, #{ count: 1000 }]);
        m.pack = [
            #{ item: "Server.Items.PowerCrystal" },
            #{ chance: 0.02, item: "Server.Items.BlackthornWelcomeBook" },
        ];
    });
});

mobile("Bogle", |c| {
    c.ai = "Mage";
    c.corpse_name = "a ghostly corpse";
    c.name = "a bogle";
    c.bleed_immune = true;
    c.poison_immune = 4;
    c.opposition = "FeyAndUndead";
    c.loot = "Meager";
    c.on_create(|m| {
        m.body = 153;
        m.sound = 0x482;
        m.str = 76..=100;
        m.dex = 76..=95;
        m.int = 36..=60;
        m.hits = 46..=60;
        m.damage = 7..=11;
        m.skill("EvalInt", 55.1, 70.0);
        m.skill("Magery", 55.1, 70.0);
        m.skill("MagicResist", 55.1, 70.0);
        m.skill("Tactics", 45.1, 60.0);
        m.skill("Wrestling", 45.1, 55.0);
        m.fame = 4000;
        m.karma = -4000;
        m.armor = 28;
        m.creature_item = #{ loot: "Weapon" };
        m.pack = [#{ item: "Server.Items.Bone" }];
    });
});

mobile("EvilMage", |c| {
    c.ai = "Mage";
    c.corpse_name = "an evil mage corpse";
    c.can_rummage_corpses = true;
    c.always_murderer = true;
    c.meat = 1;
    c.loot = ["Average", "MedScrolls"];
    c.treasure_map_level = #{ atLeast: "AOS", then: 1, "else": 0 };
    c.on_create(|m| {
        m.name = random_name("evil mage");
        m.title = "the evil mage";
        if era("UOR") {
            m.body = 124;
        } else {
            m.creature_setup = "HumanLook";
        }
        m.str = 81..=105;
        m.dex = 91..=115;
        m.int = 96..=120;
        m.hits = 49..=63;
        m.damage = 5..=10;
        m.damage_type("Physical", 100);
        m.resist("Physical", 15, 20);
        m.resist("Fire", 5, 10);
        m.resist("Poison", 5, 10);
        m.resist("Energy", 5, 10);
        m.skill("EvalInt", 75.1, 100.0);
        m.skill("Magery", 75.1, 100.0);
        m.skill("MagicResist", 75.0, 97.5);
        m.skill("Tactics", 65.0, 87.5);
        m.skill("Wrestling", 20.2, 60.0);
        m.fame = 2500;
        m.karma = -2500;
        m.armor = 16;
        m.pack = [#{ reg: 6 }];
        m.creature_item = [
            #{ item: "Server.Items.Robe", hue: #{ from: 1801, count: 108 }, to: "equip" },
            #{ item: "Server.Items.Sandals", to: "equip" },
        ];
    });
});

mobile("EvilMageLord", |c| {
    c.ai = "Mage";
    c.corpse_name = "an evil mage lord corpse";
    c.can_rummage_corpses = true;
    c.always_murderer = true;
    c.meat = 1;
    c.loot = ["Average", "Meager", ["MedScrolls", 2]];
    c.treasure_map_level = #{ atLeast: "AOS", then: 2, "else": 0 };
    c.on_create(|m| {
        m.name = random_name("evil mage lord");
        if era("UOR") {
            m.creature_body = block([#{ from: 125 }, #{ count: 2 }]);
        } else {
            m.creature_setup = "HumanLook";
        }
        m.str = 81..=105;
        m.dex = 191..=215;
        m.int = 126..=150;
        m.hits = 49..=63;
        m.damage = 5..=10;
        m.damage_type("Physical", 100);
        m.resist("Physical", 35, 40);
        m.resist("Fire", 30, 40);
        m.resist("Cold", 30, 40);
        m.resist("Poison", 30, 40);
        m.resist("Energy", 30, 40);
        m.skill("EvalInt", 80.2, 100.0);
        m.skill("Magery", 95.1, 100.0);
        m.skill("Meditation", 27.5, 50.0);
        m.skill("MagicResist", 77.5, 100.0);
        m.skill("Tactics", 65.0, 87.5);
        m.skill("Wrestling", 20.3, 80.0);
        m.fame = 10500;
        m.karma = -10500;
        m.armor = 16;
        m.pack = [#{ reg: 23 }];
        m.creature_item = [
            #{ item: "Server.Items.Robe", hue: #{ from: 1801, count: 108 }, to: "equip" },
            #{ item: "Server.Items.WizardsHat", hue: "same", to: "equip" },
            #{ item: "Server.Items.Sandals", to: "equip" },
        ];
    });
});

mobile("GolemController", |c| {
    c.ai = "Mage";
    c.corpse_name = "a golem controller corpse";
    c.always_murderer = true;
    c.loot = "Rich";
    c.click_title = false;
    c.show_fame_title = false;
    c.on_create(|m| {
        m.name = random_name("golem controller");
        m.title = "the controller";
        m.body = 400;
        m.hue = 0x455;
        m.creature_item = [
            #{ item: "Server.Items.Robe", arcaneCharges: 20, hue: 2117, lootType: "Newbied", to: "add" },
            #{ item: "Server.Items.ThighBoots", arcaneCharges: 20, hue: 2117, lootType: "Newbied", to: "add" },
            #{
                item: "Server.Items.LeatherGloves",
                arcaneCharges: 20,
                hue: 2117,
                lootType: "Newbied",
                to: "add",
            },
            #{ item: "Server.Items.Cloak", arcaneCharges: 20, hue: 2117, lootType: "Newbied", to: "add" },
        ];
        m.str = 126..=150;
        m.dex = 96..=120;
        m.int = 151..=175;
        m.hits = 76..=90;
        m.damage = 6..=12;
        m.damage_type("Physical", 100);
        m.resist("Physical", 30, 40);
        m.resist("Fire", 25, 35);
        m.resist("Cold", 35, 45);
        m.resist("Poison", 5, 15);
        m.resist("Energy", 15, 25);
        m.skill("EvalInt", 95.1, 100.0);
        m.skill("Magery", 95.1, 100.0);
        m.skill("Meditation", 95.1, 100.0);
        m.skill("MagicResist", 102.5, 125.0);
        m.skill("Tactics", 65.0, 87.5);
        m.skill("Wrestling", 65.0, 87.5);
        m.fame = 4000;
        m.karma = -4000;
        m.armor = 16;
        m.pack = [#{ chance: 0.7, item: "Server.Items.ArcaneGem" }];
    });
});

mobile("SavageShaman", |c| {
    c.ai = "Mage";
    c.corpse_name = "a savage corpse";
    c.meat = 1;
    c.always_murderer = true;
    c.opposition = "SavagesAndOrcs";
    c.loot = "Average";
    c.show_fame_title = false;
    c.on_create(|m| {
        m.name = random_name("savage shaman");
        if coin() {
            m.body = 184;
        } else {
            m.body = 183;
        }
        m.str = 126..=145;
        m.dex = 91..=110;
        m.int = 161..=185;
        m.damage = 4..=10;
        m.damage_type("Physical", 100);
        m.resist("Physical", 30, 40);
        m.resist("Fire", 20, 30);
        m.resist("Cold", 20, 30);
        m.resist("Poison", 20, 30);
        m.resist("Energy", 40, 50);
        m.skill("EvalInt", 77.5, 100.0);
        m.skill("Fencing", 62.5, 85.0);
        m.skill("Macing", 62.5, 85.0);
        m.skill("Magery", 72.5, 95.0);
        m.skill("Meditation", 77.5, 100.0);
        m.skill("MagicResist", 77.5, 100.0);
        m.skill("Swords", 62.5, 85.0);
        m.skill("Tactics", 62.5, 85.0);
        m.skill("Wrestling", 62.5, 85.0);
        m.fame = 1000;
        m.karma = -1000;
        m.pack = [
            #{ reg: [10, 15] },
            #{ item: "Server.Items.Bandage", amount: [1, 15] },
            #{ chance: 0.1, item: "Server.Items.TribalBerry" },
            #{ item: "Server.Items.BoneArms", equip: true },
            #{ item: "Server.Items.BoneLegs", equip: true },
            #{ item: "Server.Items.DeerMask", equip: true },
        ];
    });
});

mobile("Titan", |c| {
    c.ai = "Mage";
    c.corpse_name = "a titans corpse";
    c.name = "a titan";
    c.meat = 4;
    c.poison_immune = 1;
    c.treasure_map_level = 5;
    c.loot = ["FilthyRich", "Average", "MedScrolls"];
    c.on_create(|m| {
        m.body = 76;
        m.sound = 0x261;
        m.str = 536..=585;
        m.dex = 126..=145;
        m.int = 281..=305;
        m.hits = 322..=351;
        m.damage = 13..=16;
        m.damage_type("Physical", 100);
        m.resist("Physical", 35, 45);
        m.resist("Fire", 30, 40);
        m.resist("Cold", 25, 35);
        m.resist("Poison", 30, 40);
        m.resist("Energy", 30, 40);
        m.skill("EvalInt", 85.1, 100.0);
        m.skill("Magery", 85.1, 100.0);
        m.skill("MagicResist", 80.2, 110.0);
        m.skill("Tactics", 60.1, 80.0);
        m.skill("Wrestling", 40.1, 50.0);
        m.fame = 11500;
        m.karma = -11500;
        m.armor = 40;
        if era("ML") {
            if chance(0.33) {
                m.creature_item = block([#{ loot: "PeculiarSeed" }, #{ group: 1 }]);
            }
        }
    });
});

mobile("BoneKnight", |c| {
    c.corpse_name = "a skeletal corpse";
    c.name = "a bone knight";
    c.bleed_immune = true;
    c.opposition = "FeyAndUndead";
    c.loot = ["Average", "Meager"];
    c.on_create(|m| {
        m.body = 57;
        m.sound = 0x1C3;
        m.str = 196..=250;
        m.dex = 76..=95;
        m.int = 36..=60;
        m.hits = 118..=150;
        m.damage = 8..=18;
        m.damage_type("Physical", 40);
        m.damage_type("Cold", 60);
        m.resist("Physical", 35, 45);
        m.resist("Fire", 20, 30);
        m.resist("Cold", 50, 60);
        m.resist("Poison", 20, 30);
        m.resist("Energy", 30, 40);
        m.skill("MagicResist", 65.1, 80.0);
        m.skill("Tactics", 85.1, 100.0);
        m.skill("Wrestling", 85.1, 95.0);
        m.fame = 3000;
        m.karma = -3000;
        m.armor = 40;
        m.creature_item = #{
            pick: [
                "Server.Items.PlateArms",
                "Server.Items.PlateChest",
                "Server.Items.PlateGloves",
                "Server.Items.PlateGorget",
                "Server.Items.PlateLegs",
                "Server.Items.PlateHelm",
            ],
        };
        m.pack = [#{ slayer: 0.05 }, #{ item: "Server.Items.Scimitar" }, #{ item: "Server.Items.WoodenShield" }];
    });
});

mobile("Brigand", |c| {
    c.always_murderer = true;
    c.loot = "Average";
    c.click_title = false;
    c.on_create(|m| {
        m.creature_speech_hue = block([#{ from: 2 }, #{ count: 1000 }]);
        m.title = "the brigand";
        m.creature_hue = #{ raceSkinHue: "Human" };
        if coin() {
            m.female = true;
            m.body = 401;
            m.name = random_name("female");
            m.creature_item = [#{ item: "Server.Items.Skirt", hue: #{ from: 1801, count: 108 }, to: "add" }];
            m.str = 86..=100;
            m.dex = 81..=95;
            m.int = 61..=75;
            m.damage = 10..=23;
            m.skill("Fencing", 66.0, 97.5);
            m.skill("Macing", 65.0, 87.5);
            m.skill("MagicResist", 25.0, 47.5);
            m.skill("Swords", 65.0, 87.5);
            m.skill("Tactics", 65.0, 87.5);
            m.skill("Wrestling", 15.0, 37.5);
            m.fame = 1000;
            m.karma = -1000;
            m.creature_item = [#{ item: "Server.Items.Boots", hue: #{ from: 1801, count: 108 }, to: "add" }];
            m.pack = [#{ item: "Server.Items.FancyShirt", equip: true }, #{ item: "Server.Items.Bandana", equip: true }];
            m.creature_item = #{
                pick: [
                    "Server.Items.Longsword",
                    "Server.Items.Cutlass",
                    "Server.Items.Broadsword",
                    "Server.Items.Axe",
                    "Server.Items.Club",
                    "Server.Items.Dagger",
                    "Server.Items.Spear",
                ],
                to: "add",
            };
            m.random_hair = true;
        } else {
            m.female = false;
            m.body = 400;
            m.name = random_name("male");
            m.creature_item = [#{ item: "Server.Items.ShortPants", hue: #{ from: 1801, count: 108 }, to: "add" }];
            m.str = 86..=100;
            m.dex = 81..=95;
            m.int = 61..=75;
            m.damage = 10..=23;
            m.skill("Fencing", 66.0, 97.5);
            m.skill("Macing", 65.0, 87.5);
            m.skill("MagicResist", 25.0, 47.5);
            m.skill("Swords", 65.0, 87.5);
            m.skill("Tactics", 65.0, 87.5);
            m.skill("Wrestling", 15.0, 37.5);
            m.fame = 1000;
            m.karma = -1000;
            m.creature_item = [#{ item: "Server.Items.Boots", hue: #{ from: 1801, count: 108 }, to: "add" }];
            m.pack = [#{ item: "Server.Items.FancyShirt", equip: true }, #{ item: "Server.Items.Bandana", equip: true }];
            m.creature_item = #{
                pick: [
                    "Server.Items.Longsword",
                    "Server.Items.Cutlass",
                    "Server.Items.Broadsword",
                    "Server.Items.Axe",
                    "Server.Items.Club",
                    "Server.Items.Dagger",
                    "Server.Items.Spear",
                ],
                to: "add",
            };
            m.random_hair = true;
        }
    });
});

mobile("Cursed", |c| {
    c.corpse_name = "an inhuman corpse";
    c.always_murderer = true;
    c.loot = "Meager";
    c.click_title = false;
    c.show_fame_title = false;
    c.attack_sound = -1;
    c.on_create(|m| {
        m.title = "the Cursed";
        m.creature_hue = block([#{ min: 34198 }, #{ max: 34201 }]);
        m.body = 400;
        m.name = random_name("male");
        m.sound = 0x1D7;
        m.creature_item = [
            #{ item: "Server.Items.ShortPants", hue: #{ from: 1801, count: 108 }, to: "add" },
            #{ item: "Server.Items.Shirt", hue: #{ from: 1801, count: 108 }, to: "add" },
            #{ loot: "Weapon", movable: false, to: "add" },
        ];
        m.str = 91..=100;
        m.dex = 86..=95;
        m.int = 61..=70;
        m.hits = 91..=120;
        m.damage = 5..=13;
        m.resist("Physical", 15, 25);
        m.resist("Fire", 5, 10);
        m.resist("Cold", 25, 35);
        m.resist("Poison", 5, 10);
        m.resist("Energy", 5, 10);
        m.skill("Fencing", 46.0, 77.5);
        m.skill("Macing", 35.0, 57.5);
        m.skill("MagicResist", 53.5, 62.5);
        m.skill("Swords", 55.0, 77.5);
        m.skill("Tactics", 60.0, 82.5);
        m.skill("Poisoning", 60.0, 82.5);
        m.fame = 1000;
        m.karma = -2000;
    });
});

mobile("ElfBrigand", |c| {
    c.always_murderer = true;
    c.loot = "Average";
    c.click_title = false;
    c.on_create(|m| {
        m.creature_speech_hue = block([#{ from: 2 }, #{ count: 1000 }]);
        m.title = "the brigand";
        m.creature_race = "Elf";
        m.creature_hue = #{ raceSkinHue: "Elf" };
        if coin() {
            m.female = true;
            m.body = 606;
            m.name = random_name("female elf brigand");
            switch random_below(2) {
                0 => {
                    m.creature_item = block([#{ item: "Server.Items.Skirt" }, #{ hue: #{ nondyedHue: true } }, #{ to: "add" }]);
                },
                1 => {
                    m.creature_item = block([#{ item: "Server.Items.Kilt" }, #{ hue: #{ nondyedHue: true } }, #{ to: "add" }]);
                }
            }
        } else {
            m.female = false;
            m.body = 605;
            m.name = random_name("male elf brigand");
            m.creature_item = block([#{ item: "Server.Items.ShortPants" }, #{ hue: #{ nondyedHue: true } }, #{ to: "add" }]);
        }
        m.str = 86..=100;
        m.dex = 81..=95;
        m.int = 61..=75;
        m.damage = 10..=23;
        m.skill("Fencing", 66.0, 97.5);
        m.skill("Macing", 65.0, 87.5);
        m.skill("MagicResist", 25.0, 47.5);
        m.skill("Swords", 65.0, 87.5);
        m.skill("Tactics", 65.0, 87.5);
        m.skill("Wrestling", 15.0, 37.5);
        m.fame = 1000;
        m.karma = -1000;
        m.creature_item = [
            #{
                pick: ["Server.Items.Boots", "Server.Items.ThighBoots", "Server.Items.Sandals", "Server.Items.Shoes"],
                to: "add",
            },
            #{ item: "Server.Items.Shirt", hue: #{ nondyedHue: true }, to: "add" },
            #{
                pick: [
                    "Server.Items.Longsword",
                    "Server.Items.Cutlass",
                    "Server.Items.Broadsword",
                    "Server.Items.Axe",
                    "Server.Items.Club",
                    "Server.Items.Dagger",
                    "Server.Items.Spear",
                ],
                to: "add",
            },
        ];
        m.random_hair = true;
    });
});

mobile("Executioner", |c| {
    c.always_murderer = true;
    c.loot = ["FilthyRich", "Meager"];
    c.on_create(|m| {
        m.creature_speech_hue = block([#{ from: 2 }, #{ count: 1000 }]);
        m.title = "the executioner";
        m.creature_hue = #{ raceSkinHue: "Human" };
        if coin() {
            m.female = true;
            m.body = 401;
            m.name = random_name("female");
            m.creature_item = [#{ item: "Server.Items.Skirt", hue: #{ from: 1601, count: 54 }, to: "add" }];
            m.str = 386..=400;
            m.dex = 151..=165;
            m.int = 161..=175;
            m.damage = 8..=10;
            m.damage_type("Physical", 100);
            m.resist("Physical", 35, 45);
            m.resist("Fire", 25, 30);
            m.resist("Cold", 25, 30);
            m.resist("Poison", 10, 20);
            m.resist("Energy", 10, 20);
            m.skill("Anatomy", 125.0);
            m.skill("Fencing", 46.0, 77.5);
            m.skill("Macing", 35.0, 57.5);
            m.skill("Poisoning", 60.0, 82.5);
            m.skill("MagicResist", 83.5, 92.5);
            m.skill("Swords", 125.0);
            m.skill("Tactics", 125.0);
            m.skill("Lumberjacking", 125.0);
            m.fame = 5000;
            m.karma = -5000;
            m.armor = 40;
            m.creature_item = [
                #{ item: "Server.Items.ThighBoots", hue: #{ from: 1601, count: 54 }, to: "add" },
                #{ item: "Server.Items.Surcoat", hue: #{ from: 1601, count: 54 }, to: "add" },
            ];
            m.pack = [#{ item: "Server.Items.ExecutionersAxe", equip: true }];
            m.random_hair = true;
        } else {
            m.female = false;
            m.body = 400;
            m.name = random_name("male");
            m.creature_item = [#{ item: "Server.Items.ShortPants", hue: #{ from: 1601, count: 54 }, to: "add" }];
            m.str = 386..=400;
            m.dex = 151..=165;
            m.int = 161..=175;
            m.damage = 8..=10;
            m.damage_type("Physical", 100);
            m.resist("Physical", 35, 45);
            m.resist("Fire", 25, 30);
            m.resist("Cold", 25, 30);
            m.resist("Poison", 10, 20);
            m.resist("Energy", 10, 20);
            m.skill("Anatomy", 125.0);
            m.skill("Fencing", 46.0, 77.5);
            m.skill("Macing", 35.0, 57.5);
            m.skill("Poisoning", 60.0, 82.5);
            m.skill("MagicResist", 83.5, 92.5);
            m.skill("Swords", 125.0);
            m.skill("Tactics", 125.0);
            m.skill("Lumberjacking", 125.0);
            m.fame = 5000;
            m.karma = -5000;
            m.armor = 40;
            m.creature_item = [
                #{ item: "Server.Items.ThighBoots", hue: #{ from: 1601, count: 54 }, to: "add" },
                #{ item: "Server.Items.Surcoat", hue: #{ from: 1601, count: 54 }, to: "add" },
            ];
            m.pack = [#{ item: "Server.Items.ExecutionersAxe", equip: true }];
            m.random_hair = true;
        }
    });
});

mobile("Ghoul", |c| {
    c.corpse_name = "a ghostly corpse";
    c.name = "a ghoul";
    c.bleed_immune = true;
    c.poison_immune = 1;
    c.opposition = "FeyAndUndead";
    c.loot = "Meager";
    c.on_create(|m| {
        m.body = 153;
        m.sound = 0x482;
        m.str = 76..=100;
        m.dex = 76..=95;
        m.int = 36..=60;
        m.hits = 46..=60;
        m.mana = 0;
        m.damage = 7..=9;
        m.damage_type("Physical", 100);
        m.resist("Physical", 25, 30);
        m.resist("Cold", 20, 30);
        m.resist("Poison", 5, 10);
        m.resist("Energy", 10, 20);
        m.skill("MagicResist", 45.1, 60.0);
        m.skill("Tactics", 45.1, 60.0);
        m.skill("Wrestling", 45.1, 55.0);
        m.fame = 2500;
        m.karma = -2500;
        m.armor = 28;
        m.creature_item = #{ loot: "Weapon" };
    });
});

mobile("Guardian", |c| {
    c.ai = "Archer";
    c.fight_mode = "Aggressor";
    c.on_create(|m| {
        m.creature_init_stats = [100, 125, 25];
        m.title = "the guardian";
        m.creature_speech_hue = block([#{ from: 2 }, #{ count: 1000 }]);
        m.creature_hue = #{ raceSkinHue: "Human" };
        if coin() {
            m.female = true;
            m.body = 401;
            m.name = random_name("female");
            m.creature_rider = "Server.Mobiles.ForestOstard";
            m.creature_item = [
                #{ item: "Server.Items.PlateChest", hue: 2406, to: "add" },
                #{ item: "Server.Items.PlateArms", hue: 2406, to: "add" },
                #{ item: "Server.Items.PlateGloves", hue: 2406, to: "add" },
                #{ item: "Server.Items.PlateGorget", hue: 2406, to: "add" },
                #{ item: "Server.Items.PlateLegs", hue: 2406, to: "add" },
                #{ item: "Server.Items.PlateHelm", hue: 2406, to: "add" },
                #{ item: "Server.Items.Bow", movable: false, crafterSelf: true, quality: "Exceptional", to: "add" },
            ];
            m.pack = [#{ item: "Server.Items.Arrow", amount: 250 }, #{ gold: [250, 500] }];
            m.skill_base("Anatomy", 120.0);
            m.skill_base("Tactics", 120.0);
            m.skill_base("Archery", 120.0);
            m.skill_base("MagicResist", 120.0);
            m.skill_base("DetectHidden", 100.0);
        } else {
            m.female = false;
            m.body = 400;
            m.name = random_name("male");
            m.creature_rider = "Server.Mobiles.ForestOstard";
            m.creature_item = [
                #{ item: "Server.Items.PlateChest", hue: 2406, to: "add" },
                #{ item: "Server.Items.PlateArms", hue: 2406, to: "add" },
                #{ item: "Server.Items.PlateGloves", hue: 2406, to: "add" },
                #{ item: "Server.Items.PlateGorget", hue: 2406, to: "add" },
                #{ item: "Server.Items.PlateLegs", hue: 2406, to: "add" },
                #{ item: "Server.Items.PlateHelm", hue: 2406, to: "add" },
                #{ item: "Server.Items.Bow", movable: false, crafterSelf: true, quality: "Exceptional", to: "add" },
            ];
            m.pack = [#{ item: "Server.Items.Arrow", amount: 250 }, #{ gold: [250, 500] }];
            m.skill_base("Anatomy", 120.0);
            m.skill_base("Tactics", 120.0);
            m.skill_base("Archery", 120.0);
            m.skill_base("MagicResist", 120.0);
            m.skill_base("DetectHidden", 100.0);
        }
    });
});

mobile("HeadlessOne", |c| {
    c.corpse_name = "a headless corpse";
    c.name = "a headless one";
    c.can_rummage_corpses = true;
    c.meat = 1;
    c.loot = "Poor";
    c.on_create(|m| {
        m.body = 31;
        m.creature_hue = block([#{ raceSkinHue: "Human" }, #{ and: 32767 }]);
        m.sound = 0x39D;
        m.str = 26..=50;
        m.dex = 36..=55;
        m.int = 16..=30;
        m.hits = 16..=30;
        m.damage = 5..=10;
        m.damage_type("Physical", 100);
        m.resist("Physical", 15, 20);
        m.skill("MagicResist", 15.1, 20.0);
        m.skill("Tactics", 25.1, 40.0);
        m.skill("Wrestling", 25.1, 40.0);
        m.fame = 450;
        m.karma = -450;
        m.armor = 18;
        m.creature_item = #{
            pick: [
                "Server.Items.Head",
                "Server.Items.Torso",
                "Server.Items.RightArm",
                "Server.Items.LeftArm",
                "Server.Items.RightLeg",
                "Server.Items.LeftLeg",
            ],
            to: "add",
        };
    });
});

mobile("HordeMinion", |c| {
    c.corpse_name = "a horde minion corpse";
    c.name = "a horde minion";
    c.idle_sound = 338;
    c.anger_sound = 338;
    c.death_sound = 338;
    c.attack_sound = 406;
    c.hurt_sound = 194;
    c.on_create(|m| {
        m.body = 776;
        m.sound = 0x165;
        m.str = 16..=40;
        m.dex = 31..=60;
        m.int = 11..=25;
        m.hits = 10..=24;
        m.damage = 5..=10;
        m.damage_type("Physical", 100);
        m.resist("Physical", 15, 20);
        m.resist("Fire", 5, 10);
        m.skill("MagicResist", 10.0);
        m.skill("Tactics", 0.1, 15.0);
        m.skill("Wrestling", 25.1, 40.0);
        m.fame = 500;
        m.karma = -500;
        m.armor = 18;
        m.pack = [#{ item: "Server.Items.LightSource", equip: true }, #{ item: "Server.Items.Bone", amount: 3 }];
    });
});

mobile("KhaldunSummoner", |c| {
    c.ai = "Mage";
    c.name = "Zealot of Khaldun";
    c.always_murderer = true;
    c.unprovokable = true;
    c.click_title = false;
    c.show_fame_title = false;
    c.idle_sound = 388;
    c.anger_sound = 646;
    c.death_sound = 648;
    c.hurt_sound = 415;
    c.on_create(|m| {
        m.body = 400;
        m.title = "the Summoner";
        m.str = 351..=400;
        m.dex = 101..=150;
        m.int = 502..=700;
        m.hits = 421..=480;
        m.damage = 5..=15;
        m.damage_type("Physical", 75);
        m.damage_type("Cold", 25);
        m.resist("Physical", 35, 40);
        m.resist("Fire", 25, 30);
        m.resist("Cold", 50, 60);
        m.resist("Poison", 25, 35);
        m.resist("Energy", 25, 35);
        m.skill("Wrestling", 90.1, 100.0);
        m.skill("Tactics", 90.1, 100.0);
        m.skill("MagicResist", 90.1, 100.0);
        m.skill("Magery", 90.1, 100.0);
        m.skill("EvalInt", 100.0);
        m.skill("Meditation", 120.1, 130.0);
        m.armor = 36;
        m.fame = 10000;
        m.karma = -10000;
        m.creature_item = [
            #{ item: "Server.Items.LeatherGloves", hue: 1645, to: "add" },
            #{ item: "Server.Items.BoneHelm", hue: 2101, to: "add" },
            #{ item: "Server.Items.Necklace", hue: 1645, to: "add" },
            #{ item: "Server.Items.Cloak", hue: 1645, to: "add" },
            #{ item: "Server.Items.Kilt", hue: 1645, to: "add" },
            #{ item: "Server.Items.Sandals", hue: 1645, to: "add" },
        ];
    });
});

mobile("KhaldunZealot", |c| {
    c.name = "Zealot of Khaldun";
    c.always_murderer = true;
    c.unprovokable = true;
    c.poison_immune = 3;
    c.click_title = false;
    c.show_fame_title = false;
    c.idle_sound = 388;
    c.anger_sound = 646;
    c.death_sound = 648;
    c.hurt_sound = 415;
    c.on_create(|m| {
        m.body = 400;
        m.title = "the Knight";
        m.hue = 0;
        m.str = 351..=400;
        m.dex = 151..=165;
        m.int = 76..=100;
        m.hits = 448..=470;
        m.damage = 15..=25;
        m.damage_type("Physical", 75);
        m.damage_type("Cold", 25);
        m.resist("Physical", 35, 45);
        m.resist("Fire", 25, 30);
        m.resist("Cold", 50, 60);
        m.resist("Poison", 25, 35);
        m.resist("Energy", 25, 35);
        m.skill("Wrestling", 70.1, 80.0);
        m.skill("Swords", 120.1, 130.0);
        m.skill("Anatomy", 120.1, 130.0);
        m.skill("MagicResist", 90.1, 100.0);
        m.skill("Tactics", 90.1, 100.0);
        m.fame = 10000;
        m.karma = -10000;
        m.armor = 40;
        m.creature_item = [
            #{ item: "Server.Items.VikingSword", hue: 2101, movable: false, to: "add" },
            #{ item: "Server.Items.MetalShield", hue: 2101, movable: false, to: "add" },
            #{ item: "Server.Items.BoneHelm", hue: 2101, to: "add" },
            #{ item: "Server.Items.BoneArms", hue: 2101, to: "add" },
            #{ item: "Server.Items.BoneGloves", hue: 2101, to: "add" },
            #{ item: "Server.Items.BoneChest", hue: 2101, to: "add" },
            #{ item: "Server.Items.BoneLegs", hue: 2101, to: "add" },
            #{ item: "Server.Items.Boots", to: "add" },
        ];
    });
});

mobile("Mummy", |c| {
    c.corpse_name = "a mummy corpse";
    c.name = "a mummy";
    c.bleed_immune = true;
    c.poison_immune = 0;
    c.opposition = "FeyAndUndead";
    c.loot = ["Rich", "Gems", "Potions"];
    c.on_create(|m| {
        m.body = 154;
        m.sound = 0x1D7;
        m.str = 346..=370;
        m.dex = 71..=90;
        m.int = 26..=40;
        m.hits = 208..=222;
        m.damage = 13..=23;
        m.damage_type("Physical", 40);
        m.damage_type("Cold", 60);
        m.resist("Physical", 45, 55);
        m.resist("Fire", 10, 20);
        m.resist("Cold", 50, 60);
        m.resist("Poison", 20, 30);
        m.resist("Energy", 20, 30);
        m.skill("MagicResist", 15.1, 40.0);
        m.skill("Tactics", 35.1, 50.0);
        m.skill("Wrestling", 35.1, 50.0);
        m.fame = 4000;
        m.karma = -4000;
        m.armor = 50;
        if era("ML") {
            if chance(0.33) {
                m.creature_item = block([#{ loot: "PeculiarSeed" }, #{ group: 2 }]);
            }
        }
        m.pack = [#{ item: "Server.Items.Garlic", amount: 5 }, #{ item: "Server.Items.Bandage", amount: 10 }];
    });
});

mobile("Orc", |c| {
    c.corpse_name = "an orcish corpse";
    c.can_rummage_corpses = true;
    c.treasure_map_level = 1;
    c.meat = 1;
    c.opposition = "SavagesAndOrcs";
    c.loot = "Meager";
    c.speech = "Orc";
    c.on_create(|m| {
        m.name = random_name("orc");
        m.body = 17;
        m.sound = 0x45A;
        m.str = 96..=120;
        m.dex = 81..=105;
        m.int = 36..=60;
        m.hits = 58..=72;
        m.damage = 5..=7;
        m.damage_type("Physical", 100);
        m.resist("Physical", 25, 30);
        m.resist("Fire", 20, 30);
        m.resist("Cold", 10, 20);
        m.resist("Poison", 10, 20);
        m.resist("Energy", 20, 30);
        m.skill("MagicResist", 50.1, 75.0);
        m.skill("Tactics", 55.1, 80.0);
        m.skill("Wrestling", 50.1, 70.0);
        m.fame = 1500;
        m.karma = -1500;
        m.armor = 28;
        m.creature_item = [
            #{
                pick: [
                    "Server.Items.Scimitar",
                    "Server.Items.Katana",
                    "Server.Items.WarMace",
                    "Server.Items.WarHammer",
                    "Server.Items.Kryss",
                    "Server.Items.Pitchfork",
                    (),
                    (),
                    (),
                    (),
                    (),
                    (),
                    (),
                    (),
                    (),
                    (),
                    (),
                    (),
                    (),
                    (),
                ],
            },
        ];
        m.pack = [#{ item: "Server.Items.ThighBoots" }];
        m.creature_item = #{ pick: ["Server.Items.Ribs", "Server.Items.Shaft", "Server.Items.Candle"] };
        m.pack = [#{ chance: 0.2, item: "Server.Items.BolaBall" }];
    });
});

mobile("OrcCaptain", |c| {
    c.corpse_name = "an orcish corpse";
    c.can_rummage_corpses = true;
    c.meat = 1;
    c.opposition = "SavagesAndOrcs";
    c.loot = [["Meager", 2]];
    c.speech = "Orc";
    c.on_create(|m| {
        m.name = random_name("orc");
        m.body = 7;
        m.sound = 0x45A;
        m.str = 111..=145;
        m.dex = 101..=135;
        m.int = 86..=110;
        m.hits = 67..=87;
        m.damage = 5..=15;
        m.damage_type("Physical", 100);
        m.resist("Physical", 30, 35);
        m.resist("Fire", 10, 20);
        m.resist("Cold", 15, 25);
        m.resist("Poison", 5, 10);
        m.resist("Energy", 5, 10);
        m.skill("MagicResist", 70.1, 85.0);
        m.skill("Swords", 70.1, 95.0);
        m.skill("Tactics", 85.1, 100.0);
        m.fame = 2500;
        m.karma = -2500;
        m.armor = 34;
        switch random_below(7) {
            0 => {
                m.pack = [#{ item: "Server.Items.Arrow" }];
            },
            1 => {
                m.pack = [#{ item: "Server.Items.Lockpick" }];
            },
            2 => {
                m.pack = [#{ item: "Server.Items.Shaft" }];
            },
            3 => {
                m.pack = [#{ item: "Server.Items.Ribs" }];
            },
            4 => {
                m.pack = [#{ item: "Server.Items.Bandage" }];
            },
            5 => {
                m.creature_item = block([#{ item: "Server.Items.BeverageBottle" }, #{ args: [#{ enum: 4 }] }]);
            },
            6 => {
                m.creature_item = block([#{ item: "Server.Items.Jug" }, #{ args: [#{ enum: 1 }] }]);
            }
        }
        if era("AOS") {
            m.creature_item = #{ loot: "NecromancyReagent" };
        }
    });
});

mobile("OrcishLord", |c| {
    c.corpse_name = "an orcish corpse";
    c.name = "an orcish lord";
    c.can_rummage_corpses = true;
    c.treasure_map_level = 1;
    c.meat = 1;
    c.opposition = "SavagesAndOrcs";
    c.loot = ["Meager", "Average"];
    c.speech = "Orc";
    c.on_create(|m| {
        m.body = 138;
        m.sound = 0x45A;
        m.str = 147..=215;
        m.dex = 91..=115;
        m.int = 61..=85;
        m.hits = 95..=123;
        m.damage = 4..=14;
        m.damage_type("Physical", 100);
        m.resist("Physical", 25, 35);
        m.resist("Fire", 30, 40);
        m.resist("Cold", 20, 30);
        m.resist("Poison", 30, 40);
        m.resist("Energy", 30, 40);
        m.skill("MagicResist", 70.1, 85.0);
        m.skill("Swords", 60.1, 85.0);
        m.skill("Tactics", 75.1, 90.0);
        m.skill("Wrestling", 60.1, 85.0);
        m.fame = 2500;
        m.karma = -2500;
        m.creature_item = [
            #{
                pick: [
                    "Server.Items.Lockpick",
                    "Server.Items.MortarPestle",
                    "Server.Items.Bottle",
                    "Server.Items.RawRibs",
                    "Server.Items.Shovel",
                ],
            },
        ];
        m.pack = [#{ item: "Server.Items.RingmailChest" }];
        if chance(0.3) {
            m.creature_item = #{ loot: "PossibleReagent" };
        }
        m.pack = [#{ chance: 0.2, item: "Server.Items.BolaBall" }];
    });
});

mobile("Savage", |c| {
    c.corpse_name = "a savage corpse";
    c.meat = 1;
    c.always_murderer = true;
    c.opposition = "SavagesAndOrcs";
    c.loot = "Meager";
    c.show_fame_title = false;
    c.on_create(|m| {
        m.name = random_name("savage");
        if coin() {
            m.female = true;
            m.body = 184;
            m.str = 96..=115;
            m.dex = 86..=105;
            m.int = 51..=65;
            m.damage = 23..=27;
            m.damage_type("Physical", 100);
            m.skill("Fencing", 60.0, 82.5);
            m.skill("Macing", 60.0, 82.5);
            m.skill("Poisoning", 60.0, 82.5);
            m.skill("MagicResist", 57.5, 80.0);
            m.skill("Swords", 60.0, 82.5);
            m.skill("Tactics", 60.0, 82.5);
            m.fame = 1000;
            m.karma = -1000;
            m.pack = [
                #{ item: "Server.Items.Bandage", amount: [1, 15] },
                #{ chance: 0.1, item: "Server.Items.TribalBerry" },
            ];
        } else {
            m.female = false;
            m.body = 183;
            m.str = 96..=115;
            m.dex = 86..=105;
            m.int = 51..=65;
            m.damage = 23..=27;
            m.damage_type("Physical", 100);
            m.skill("Fencing", 60.0, 82.5);
            m.skill("Macing", 60.0, 82.5);
            m.skill("Poisoning", 60.0, 82.5);
            m.skill("MagicResist", 57.5, 80.0);
            m.skill("Swords", 60.0, 82.5);
            m.skill("Tactics", 60.0, 82.5);
            m.fame = 1000;
            m.karma = -1000;
            m.pack = [
                #{ item: "Server.Items.Bandage", amount: [1, 15] },
                #{ chance: 0.1, item: "Server.Items.BolaBall" },
            ];
        }
    });
});

mobile("SavageRider", |c| {
    c.corpse_name = "a savage corpse";
    c.meat = 1;
    c.always_murderer = true;
    c.opposition = "SavagesAndOrcs";
    c.loot = "Average";
    c.show_fame_title = false;
    c.on_create(|m| {
        m.name = random_name("savage rider");
        if coin() {
            m.female = true;
            m.body = 186;
            m.str = 151..=170;
            m.dex = 92..=130;
            m.int = 51..=65;
            m.damage = 29..=34;
            m.damage_type("Physical", 100);
            m.skill("Fencing", 72.5, 95.0);
            m.skill("Healing", 60.3, 90.0);
            m.skill("Macing", 72.5, 95.0);
            m.skill("Poisoning", 60.0, 82.5);
            m.skill("MagicResist", 72.5, 95.0);
            m.skill("Swords", 72.5, 95.0);
            m.skill("Tactics", 72.5, 95.0);
            m.fame = 1000;
            m.karma = -1000;
            m.pack = [
                #{ item: "Server.Items.Bandage", amount: [1, 15] },
                #{ chance: 0.1, item: "Server.Items.BolaBall" },
                #{ item: "Server.Items.TribalSpear", equip: true },
                #{ item: "Server.Items.BoneArms", equip: true },
                #{ item: "Server.Items.BoneLegs", equip: true },
            ];
            m.creature_rider = "Server.Mobiles.SavageRidgeback";
        } else {
            m.female = false;
            m.body = 185;
            m.str = 151..=170;
            m.dex = 92..=130;
            m.int = 51..=65;
            m.damage = 29..=34;
            m.damage_type("Physical", 100);
            m.skill("Fencing", 72.5, 95.0);
            m.skill("Healing", 60.3, 90.0);
            m.skill("Macing", 72.5, 95.0);
            m.skill("Poisoning", 60.0, 82.5);
            m.skill("MagicResist", 72.5, 95.0);
            m.skill("Swords", 72.5, 95.0);
            m.skill("Tactics", 72.5, 95.0);
            m.fame = 1000;
            m.karma = -1000;
            m.pack = [
                #{ item: "Server.Items.Bandage", amount: [1, 15] },
                #{ chance: 0.1, item: "Server.Items.BolaBall" },
                #{ item: "Server.Items.TribalSpear", equip: true },
                #{ item: "Server.Items.BoneArms", equip: true },
                #{ item: "Server.Items.BoneLegs", equip: true },
            ];
            m.creature_rider = "Server.Mobiles.SavageRidgeback";
        }
    });
});

mobile("ShadowFiend", |c| {
    c.delete_corpse_on_death = true;
    c.name = "a shadow fiend";
    c.can_rummage_corpses = true;
    c.idle_sound = 890;
    c.anger_sound = 889;
    c.death_sound = 897;
    c.attack_sound = 895;
    c.hurt_sound = 896;
    c.on_create(|m| {
        m.body = 168;
        m.creature_setup = "ShadowFiendTimer";
    });
});

mobile("SkeletalKnight", |c| {
    c.corpse_name = "a skeletal corpse";
    c.name = "a skeletal knight";
    c.bleed_immune = true;
    c.opposition = "FeyAndUndead";
    c.loot = ["Average", "Meager"];
    c.on_create(|m| {
        m.body = 147;
        m.sound = 0x1C3;
        m.str = 196..=250;
        m.dex = 76..=95;
        m.int = 36..=60;
        m.hits = 118..=150;
        m.damage = 8..=18;
        m.damage_type("Physical", 40);
        m.damage_type("Cold", 60);
        m.resist("Physical", 35, 45);
        m.resist("Fire", 20, 30);
        m.resist("Cold", 50, 60);
        m.resist("Poison", 20, 30);
        m.resist("Energy", 30, 40);
        m.skill("MagicResist", 65.1, 80.0);
        m.skill("Tactics", 85.1, 100.0);
        m.skill("Wrestling", 85.1, 95.0);
        m.fame = 3000;
        m.karma = -3000;
        m.armor = 40;
        m.creature_item = #{
            pick: [
                "Server.Items.PlateArms",
                "Server.Items.PlateChest",
                "Server.Items.PlateGloves",
                "Server.Items.PlateGorget",
                "Server.Items.PlateLegs",
                "Server.Items.PlateHelm",
            ],
        };
        m.pack = [#{ item: "Server.Items.Scimitar" }, #{ item: "Server.Items.WoodenShield" }];
    });
});

mobile("Skeleton", |c| {
    c.corpse_name = "a skeletal corpse";
    c.name = "a skeleton";
    c.bleed_immune = true;
    c.poison_immune = 0;
    c.opposition = "FeyAndUndead";
    c.loot = "Poor";
    c.on_create(|m| {
        m.bodies = [50, 56];
        m.sound = 0x48D;
        m.str = 56..=80;
        m.dex = 56..=75;
        m.int = 16..=40;
        m.hits = 34..=48;
        m.damage = 3..=7;
        m.damage_type("Physical", 100);
        m.resist("Physical", 15, 20);
        m.resist("Fire", 5, 10);
        m.resist("Cold", 25, 40);
        m.resist("Poison", 25, 35);
        m.resist("Energy", 5, 15);
        m.skill("MagicResist", 45.1, 60.0);
        m.skill("Tactics", 45.1, 60.0);
        m.skill("Wrestling", 45.1, 55.0);
        m.fame = 450;
        m.karma = -450;
        m.armor = 16;
        m.creature_item = #{
            pick: [
                "Server.Items.BoneArms",
                "Server.Items.BoneChest",
                "Server.Items.BoneGloves",
                "Server.Items.BoneLegs",
                "Server.Items.BoneHelm",
            ],
        };
    });
});

mobile("SpawnedOrcishLord", |c| {
    c.inherits = "OrcishLord";
    c.corpse_name = "an orcish corpse";
    c.on_create(|m| {
        m.creature_setup = "DeleteBackpack";
        m.creature_setup = "NoKillAwards";
    });
});

mobile("SpectralArmour", |c| {
    c.delete_corpse_on_death = true;
    c.name = "a spectral armour";
    c.poison_immune = 1;
    c.idle_sound = 512;
    c.anger_sound = 86;
    c.on_create(|m| {
        m.body = 637;
        m.hue = 0x8026;
        m.creature_item = [
            #{ item: "Server.Items.Buckler", hue: 2101, movable: false, to: "add" },
            #{ item: "Server.Items.ChainCoif", hue: 2101, to: "add" },
            #{ item: "Server.Items.PlateGloves", hue: 2101, to: "add" },
        ];
        m.str = 101..=110;
        m.dex = 101..=110;
        m.int = 101..=110;
        m.hits = 178..=201;
        m.stam = 191..=200;
        m.damage = 10..=22;
        m.damage_type("Physical", 75);
        m.damage_type("Cold", 25);
        m.resist("Physical", 35, 45);
        m.resist("Fire", 20, 30);
        m.resist("Cold", 30, 40);
        m.resist("Poison", 20, 30);
        m.resist("Energy", 20, 30);
        m.skill("Wrestling", 75.1, 100.0);
        m.skill("Tactics", 90.1, 100.0);
        m.skill("MagicResist", 90.1, 100.0);
        m.armor = 40;
        m.fame = 7000;
        m.karma = -7000;
    });
});

mobile("Zombie", |c| {
    c.corpse_name = "a rotting corpse";
    c.name = "a zombie";
    c.bleed_immune = true;
    c.poison_immune = 1;
    c.opposition = "FeyAndUndead";
    c.loot = "Meager";
    c.on_create(|m| {
        m.body = 3;
        m.sound = 0x1D7;
        m.str = 46..=70;
        m.dex = 31..=50;
        m.int = 26..=40;
        m.hits = 28..=42;
        m.damage = 3..=7;
        m.damage_type("Physical", 100);
        m.resist("Physical", 15, 20);
        m.resist("Cold", 20, 30);
        m.resist("Poison", 5, 10);
        m.skill("MagicResist", 15.1, 40.0);
        m.skill("Tactics", 35.1, 50.0);
        m.skill("Wrestling", 35.1, 50.0);
        m.fame = 600;
        m.karma = -600;
        m.armor = 18;
        m.creature_item = #{
            pick: [
                "Server.Items.LeftArm",
                "Server.Items.RightArm",
                "Server.Items.Torso",
                "Server.Items.Bone",
                "Server.Items.RibCage",
                "Server.Items.RibCage",
                "Server.Items.BonePile",
                "Server.Items.BonePile",
                "Server.Items.BonePile",
                "Server.Items.BonePile",
            ],
        };
    });
});

mobile("ExodusMinion", |c| {
    c.corpse_name = "a minion's corpse";
    c.is_scared_of_scary_things = false;
    c.is_scary_to_pets = true;
    c.name = "an exodus minion";
    c.auto_dispel = true;
    c.poison_immune = 4;
    c.loot = ["Average", "Rich"];
    c.bard_immune = #{ atLeast: "AOS", then: false, "else": true };
    c.idle_sound = 536;
    c.anger_sound = 620;
    c.death_sound = 529;
    c.attack_sound = 562;
    c.hurt_sound = 320;
    c.on_create(|m| {
        m.body = 757;
        m.str = 851..=950;
        m.dex = 71..=80;
        m.int = 61..=90;
        m.hits = 511..=570;
        m.damage = 16..=22;
        m.resist("Physical", 60, 70);
        m.resist("Fire", 40, 50);
        m.resist("Cold", 15, 25);
        m.resist("Poison", 15, 25);
        m.resist("Energy", 15, 25);
        m.skill("MagicResist", 90.1, 100.0);
        m.skill("Tactics", 90.1, 100.0);
        m.skill("Wrestling", 90.1, 100.0);
        m.fame = 18000;
        m.karma = -18000;
        m.armor = 65;
        m.pack = [
            #{ item: "Server.Items.PowerCrystal" },
            #{ item: "Server.Items.ArcaneGem" },
            #{ item: "Server.Items.ClockworkAssembly" },
        ];
        m.creature_item = #{ pick: ["Server.Items.PowerCrystal", "Server.Items.ArcaneGem", "Server.Items.ClockworkAssembly"] };
    });
});

mobile("ExodusOverseer", |c| {
    c.corpse_name = "an overseer's corpse";
    c.is_scared_of_scary_things = false;
    c.is_scary_to_pets = true;
    c.name = "an exodus overseer";
    c.auto_dispel = true;
    c.poison_immune = 4;
    c.loot = "Rich";
    c.bard_immune = #{ atLeast: "AOS", then: false, "else": true };
    c.idle_sound = 253;
    c.anger_sound = 620;
    c.death_sound = 529;
    c.attack_sound = 571;
    c.hurt_sound = 320;
    c.on_create(|m| {
        m.body = 756;
        m.str = 561..=650;
        m.dex = 76..=95;
        m.int = 61..=90;
        m.hits = 331..=390;
        m.damage = 13..=19;
        m.damage_type("Physical", 50);
        m.damage_type("Energy", 50);
        m.resist("Physical", 45, 55);
        m.resist("Fire", 40, 60);
        m.resist("Cold", 25, 35);
        m.resist("Poison", 25, 35);
        m.resist("Energy", 25, 35);
        m.skill("MagicResist", 80.2, 98.0);
        m.skill("Tactics", 80.2, 98.0);
        m.skill("Wrestling", 80.2, 98.0);
        m.fame = 10000;
        m.karma = -10000;
        m.armor = 50;
        switch random_below(2) {
            0 => {
                m.pack = [#{ item: "Server.Items.PowerCrystal" }];
            },
            1 => {
                m.pack = [#{ item: "Server.Items.ArcaneGem" }];
            }
        }
    });
});

mobile("ChaosDragoon", |c| {
    c.corpse_name = "a chaos dragoon corpse";
    c.name = "a chaos dragoon";
    c.auto_dispel = true;
    c.can_rummage_corpses = true;
    c.always_murderer = true;
    c.loot = "Rich";
    c.bard_immune = #{ atLeast: "AOS", then: false, "else": true };
    c.monster_abilities = ["FireBreath"];
    c.show_fame_title = false;
    c.idle_sound = 718;
    c.death_sound = 716;
    c.hurt_sound = 721;
    c.attack_sound = 712;
    c.on_create(|m| {
        m.body = 400;
        m.creature_hue = #{ raceSkinHue: "Human" };
        m.creature_speed = [0.15, 0.4];
        m.str = 176..=225;
        m.dex = 81..=95;
        m.int = 61..=85;
        m.hits = 176..=225;
        m.damage = 24..=26;
        m.damage_type("Physical", 25);
        m.damage_type("Fire", 25);
        m.damage_type("Cold", 25);
        m.damage_type("Energy", 25);
        m.skill("Fencing", 77.6, 92.5);
        m.skill("Healing", 60.3, 90.0);
        m.skill("Macing", 77.6, 92.5);
        m.skill("Anatomy", 77.6, 87.5);
        m.skill("MagicResist", 77.6, 97.5);
        m.skill("Swords", 77.6, 92.5);
        m.skill("Tactics", 77.6, 87.5);
        m.fame = 5000;
        m.karma = -5000;
        switch random_below(6) {
            0 => {
                m.creature_item = [
                    #{
                        pick: ["Server.Items.Kryss", "Server.Items.Broadsword", "Server.Items.Katana"],
                        movable: false,
                        to: "add",
                    },
                    #{ item: "Server.Items.DragonHelm", movable: false, to: "add", resource: "BlackScales" },
                    #{ item: "Server.Items.DragonChest", movable: false, to: "add", resource: "BlackScales" },
                    #{ item: "Server.Items.DragonArms", movable: false, to: "add", resource: "BlackScales" },
                    #{ item: "Server.Items.DragonGloves", movable: false, to: "add", resource: "BlackScales" },
                    #{ item: "Server.Items.DragonLegs", movable: false, to: "add", resource: "BlackScales" },
                    #{ item: "Server.Items.ChaosShield", movable: false, to: "add" },
                    #{ item: "Server.Items.Shirt", to: "add" },
                    #{ item: "Server.Items.Boots", to: "add" },
                    #{ item: "Server.Items.BlackScales", args: [#{ min: 1, max: 3 }], to: "add" },
                ];
                m.creature_rider = "Server.Mobiles.SwampDragon";
            },
            1 => {
                m.creature_item = [
                    #{
                        pick: ["Server.Items.Kryss", "Server.Items.Broadsword", "Server.Items.Katana"],
                        movable: false,
                        to: "add",
                    },
                    #{ item: "Server.Items.DragonHelm", movable: false, to: "add", resource: "RedScales" },
                    #{ item: "Server.Items.DragonChest", movable: false, to: "add", resource: "RedScales" },
                    #{ item: "Server.Items.DragonArms", movable: false, to: "add", resource: "RedScales" },
                    #{ item: "Server.Items.DragonGloves", movable: false, to: "add", resource: "RedScales" },
                    #{ item: "Server.Items.DragonLegs", movable: false, to: "add", resource: "RedScales" },
                    #{ item: "Server.Items.ChaosShield", movable: false, to: "add" },
                    #{ item: "Server.Items.Shirt", to: "add" },
                    #{ item: "Server.Items.Boots", to: "add" },
                    #{ item: "Server.Items.RedScales", args: [#{ min: 1, max: 3 }], to: "add" },
                ];
                m.creature_rider = "Server.Mobiles.SwampDragon";
            },
            2 => {
                m.creature_item = [
                    #{
                        pick: ["Server.Items.Kryss", "Server.Items.Broadsword", "Server.Items.Katana"],
                        movable: false,
                        to: "add",
                    },
                    #{ item: "Server.Items.DragonHelm", movable: false, to: "add", resource: "BlueScales" },
                    #{ item: "Server.Items.DragonChest", movable: false, to: "add", resource: "BlueScales" },
                    #{ item: "Server.Items.DragonArms", movable: false, to: "add", resource: "BlueScales" },
                    #{ item: "Server.Items.DragonGloves", movable: false, to: "add", resource: "BlueScales" },
                    #{ item: "Server.Items.DragonLegs", movable: false, to: "add", resource: "BlueScales" },
                    #{ item: "Server.Items.ChaosShield", movable: false, to: "add" },
                    #{ item: "Server.Items.Shirt", to: "add" },
                    #{ item: "Server.Items.Boots", to: "add" },
                    #{ item: "Server.Items.BlueScales", args: [#{ min: 1, max: 3 }], to: "add" },
                ];
                m.creature_rider = "Server.Mobiles.SwampDragon";
            },
            3 => {
                m.creature_item = [
                    #{
                        pick: ["Server.Items.Kryss", "Server.Items.Broadsword", "Server.Items.Katana"],
                        movable: false,
                        to: "add",
                    },
                    #{ item: "Server.Items.DragonHelm", movable: false, to: "add", resource: "YellowScales" },
                    #{ item: "Server.Items.DragonChest", movable: false, to: "add", resource: "YellowScales" },
                    #{ item: "Server.Items.DragonArms", movable: false, to: "add", resource: "YellowScales" },
                    #{ item: "Server.Items.DragonGloves", movable: false, to: "add", resource: "YellowScales" },
                    #{ item: "Server.Items.DragonLegs", movable: false, to: "add", resource: "YellowScales" },
                    #{ item: "Server.Items.ChaosShield", movable: false, to: "add" },
                    #{ item: "Server.Items.Shirt", to: "add" },
                    #{ item: "Server.Items.Boots", to: "add" },
                    #{ item: "Server.Items.YellowScales", args: [#{ min: 1, max: 3 }], to: "add" },
                ];
                m.creature_rider = "Server.Mobiles.SwampDragon";
            },
            4 => {
                m.creature_item = [
                    #{
                        pick: ["Server.Items.Kryss", "Server.Items.Broadsword", "Server.Items.Katana"],
                        movable: false,
                        to: "add",
                    },
                    #{ item: "Server.Items.DragonHelm", movable: false, to: "add", resource: "GreenScales" },
                    #{ item: "Server.Items.DragonChest", movable: false, to: "add", resource: "GreenScales" },
                    #{ item: "Server.Items.DragonArms", movable: false, to: "add", resource: "GreenScales" },
                    #{ item: "Server.Items.DragonGloves", movable: false, to: "add", resource: "GreenScales" },
                    #{ item: "Server.Items.DragonLegs", movable: false, to: "add", resource: "GreenScales" },
                    #{ item: "Server.Items.ChaosShield", movable: false, to: "add" },
                    #{ item: "Server.Items.Shirt", to: "add" },
                    #{ item: "Server.Items.Boots", to: "add" },
                    #{ item: "Server.Items.GreenScales", args: [#{ min: 1, max: 3 }], to: "add" },
                ];
                m.creature_rider = "Server.Mobiles.SwampDragon";
            },
            5 => {
                m.creature_item = [
                    #{
                        pick: ["Server.Items.Kryss", "Server.Items.Broadsword", "Server.Items.Katana"],
                        movable: false,
                        to: "add",
                    },
                    #{ item: "Server.Items.DragonHelm", movable: false, to: "add", resource: "WhiteScales" },
                    #{ item: "Server.Items.DragonChest", movable: false, to: "add", resource: "WhiteScales" },
                    #{ item: "Server.Items.DragonArms", movable: false, to: "add", resource: "WhiteScales" },
                    #{ item: "Server.Items.DragonGloves", movable: false, to: "add", resource: "WhiteScales" },
                    #{ item: "Server.Items.DragonLegs", movable: false, to: "add", resource: "WhiteScales" },
                    #{ item: "Server.Items.ChaosShield", movable: false, to: "add" },
                    #{ item: "Server.Items.Shirt", to: "add" },
                    #{ item: "Server.Items.Boots", to: "add" },
                    #{ item: "Server.Items.WhiteScales", args: [#{ min: 1, max: 3 }], to: "add" },
                ];
                m.creature_rider = "Server.Mobiles.SwampDragon";
            }
        }
    });
});

mobile("ChaosDragoonElite", |c| {
    c.ai = "Mage";
    c.corpse_name = "a chaos dragoon elite corpse";
    c.name = "a chaos dragoon elite";
    c.auto_dispel = true;
    c.can_rummage_corpses = true;
    c.always_murderer = true;
    c.loot = ["Rich", "Gems"];
    c.bard_immune = #{ atLeast: "AOS", then: false, "else": true };
    c.monster_abilities = ["FireBreath"];
    c.show_fame_title = false;
    c.idle_sound = 718;
    c.death_sound = 716;
    c.hurt_sound = 721;
    c.attack_sound = 712;
    c.on_create(|m| {
        m.body = 400;
        m.creature_hue = #{ raceSkinHue: "Human" };
        m.creature_speed = [0.15, 0.4];
        m.str = 276..=350;
        m.dex = 66..=90;
        m.int = 126..=150;
        m.hits = 276..=350;
        m.damage = 29..=34;
        m.damage_type("Physical", 100);
        m.skill("Tactics", 80.1, 100.0);
        m.skill("MagicResist", 100.1, 110.0);
        m.skill("Anatomy", 80.1, 100.0);
        m.skill("Magery", 85.1, 100.0);
        m.skill("EvalInt", 85.1, 100.0);
        m.skill("Swords", 72.5, 95.0);
        m.skill("Fencing", 85.1, 100.0);
        m.skill("Macing", 85.1, 100.0);
        m.fame = 8000;
        m.karma = -8000;
        switch random_below(6) {
            0 => {
                m.creature_item = [
                    #{
                        pick: ["Server.Items.Kryss", "Server.Items.Broadsword", "Server.Items.Katana"],
                        movable: false,
                        to: "add",
                    },
                    #{ item: "Server.Items.DragonChest", movable: false, to: "add", resource: "BlackScales" },
                    #{ item: "Server.Items.DragonLegs", movable: false, to: "add", resource: "BlackScales" },
                    #{ item: "Server.Items.DragonArms", movable: false, to: "add", resource: "BlackScales" },
                    #{ item: "Server.Items.DragonGloves", movable: false, to: "add", resource: "BlackScales" },
                    #{ item: "Server.Items.DragonHelm", movable: false, to: "add", resource: "BlackScales" },
                    #{ item: "Server.Items.ChaosShield", movable: false, to: "add" },
                    #{ item: "Server.Items.Boots", hue: 1109, to: "add" },
                    #{ item: "Server.Items.Shirt", hue: #{ from: 2401, count: 30 }, to: "add" },
                    #{ item: "Server.Items.BlackScales", args: [#{ min: 1, max: 3 }], to: "add" },
                ];
                switch random_below(9) {
                    0 => {
                        m.creature_rider = block([#{ class: "Server.Mobiles.SwampDragon" }, #{ barding: "DullCopper" }]);
                    },
                    1 => {
                        m.creature_rider = block([#{ class: "Server.Mobiles.SwampDragon" }, #{ barding: "ShadowIron" }]);
                    },
                    2 => {
                        m.creature_rider = block([#{ class: "Server.Mobiles.SwampDragon" }, #{ barding: "Copper" }]);
                    },
                    3 => {
                        m.creature_rider = block([#{ class: "Server.Mobiles.SwampDragon" }, #{ barding: "Bronze" }]);
                    },
                    4 => {
                        m.creature_rider = block([#{ class: "Server.Mobiles.SwampDragon" }, #{ barding: "Gold" }]);
                    },
                    5 => {
                        m.creature_rider = block([#{ class: "Server.Mobiles.SwampDragon" }, #{ barding: "Agapite" }]);
                    },
                    6 => {
                        m.creature_rider = block([#{ class: "Server.Mobiles.SwampDragon" }, #{ barding: "Verite" }]);
                    },
                    7 => {
                        m.creature_rider = block([#{ class: "Server.Mobiles.SwampDragon" }, #{ barding: "Valorite" }]);
                    },
                    8 => {
                        m.creature_rider = block([#{ class: "Server.Mobiles.SwampDragon" }, #{ barding: "Iron" }]);
                    }
                }
            },
            1 => {
                m.creature_item = [
                    #{
                        pick: ["Server.Items.Kryss", "Server.Items.Broadsword", "Server.Items.Katana"],
                        movable: false,
                        to: "add",
                    },
                    #{ item: "Server.Items.DragonChest", movable: false, to: "add", resource: "RedScales" },
                    #{ item: "Server.Items.DragonLegs", movable: false, to: "add", resource: "RedScales" },
                    #{ item: "Server.Items.DragonArms", movable: false, to: "add", resource: "RedScales" },
                    #{ item: "Server.Items.DragonGloves", movable: false, to: "add", resource: "RedScales" },
                    #{ item: "Server.Items.DragonHelm", movable: false, to: "add", resource: "RedScales" },
                    #{ item: "Server.Items.ChaosShield", movable: false, to: "add" },
                    #{ item: "Server.Items.Boots", hue: 1109, to: "add" },
                    #{ item: "Server.Items.Shirt", hue: #{ from: 2401, count: 30 }, to: "add" },
                    #{ item: "Server.Items.RedScales", args: [#{ min: 1, max: 3 }], to: "add" },
                ];
                switch random_below(9) {
                    0 => {
                        m.creature_rider = block([#{ class: "Server.Mobiles.SwampDragon" }, #{ barding: "DullCopper" }]);
                    },
                    1 => {
                        m.creature_rider = block([#{ class: "Server.Mobiles.SwampDragon" }, #{ barding: "ShadowIron" }]);
                    },
                    2 => {
                        m.creature_rider = block([#{ class: "Server.Mobiles.SwampDragon" }, #{ barding: "Copper" }]);
                    },
                    3 => {
                        m.creature_rider = block([#{ class: "Server.Mobiles.SwampDragon" }, #{ barding: "Bronze" }]);
                    },
                    4 => {
                        m.creature_rider = block([#{ class: "Server.Mobiles.SwampDragon" }, #{ barding: "Gold" }]);
                    },
                    5 => {
                        m.creature_rider = block([#{ class: "Server.Mobiles.SwampDragon" }, #{ barding: "Agapite" }]);
                    },
                    6 => {
                        m.creature_rider = block([#{ class: "Server.Mobiles.SwampDragon" }, #{ barding: "Verite" }]);
                    },
                    7 => {
                        m.creature_rider = block([#{ class: "Server.Mobiles.SwampDragon" }, #{ barding: "Valorite" }]);
                    },
                    8 => {
                        m.creature_rider = block([#{ class: "Server.Mobiles.SwampDragon" }, #{ barding: "Iron" }]);
                    }
                }
            },
            2 => {
                m.creature_item = [
                    #{
                        pick: ["Server.Items.Kryss", "Server.Items.Broadsword", "Server.Items.Katana"],
                        movable: false,
                        to: "add",
                    },
                    #{ item: "Server.Items.DragonChest", movable: false, to: "add", resource: "BlueScales" },
                    #{ item: "Server.Items.DragonLegs", movable: false, to: "add", resource: "BlueScales" },
                    #{ item: "Server.Items.DragonArms", movable: false, to: "add", resource: "BlueScales" },
                    #{ item: "Server.Items.DragonGloves", movable: false, to: "add", resource: "BlueScales" },
                    #{ item: "Server.Items.DragonHelm", movable: false, to: "add", resource: "BlueScales" },
                    #{ item: "Server.Items.ChaosShield", movable: false, to: "add" },
                    #{ item: "Server.Items.Boots", hue: 1109, to: "add" },
                    #{ item: "Server.Items.Shirt", hue: #{ from: 2401, count: 30 }, to: "add" },
                    #{ item: "Server.Items.BlueScales", args: [#{ min: 1, max: 3 }], to: "add" },
                ];
                switch random_below(9) {
                    0 => {
                        m.creature_rider = block([#{ class: "Server.Mobiles.SwampDragon" }, #{ barding: "DullCopper" }]);
                    },
                    1 => {
                        m.creature_rider = block([#{ class: "Server.Mobiles.SwampDragon" }, #{ barding: "ShadowIron" }]);
                    },
                    2 => {
                        m.creature_rider = block([#{ class: "Server.Mobiles.SwampDragon" }, #{ barding: "Copper" }]);
                    },
                    3 => {
                        m.creature_rider = block([#{ class: "Server.Mobiles.SwampDragon" }, #{ barding: "Bronze" }]);
                    },
                    4 => {
                        m.creature_rider = block([#{ class: "Server.Mobiles.SwampDragon" }, #{ barding: "Gold" }]);
                    },
                    5 => {
                        m.creature_rider = block([#{ class: "Server.Mobiles.SwampDragon" }, #{ barding: "Agapite" }]);
                    },
                    6 => {
                        m.creature_rider = block([#{ class: "Server.Mobiles.SwampDragon" }, #{ barding: "Verite" }]);
                    },
                    7 => {
                        m.creature_rider = block([#{ class: "Server.Mobiles.SwampDragon" }, #{ barding: "Valorite" }]);
                    },
                    8 => {
                        m.creature_rider = block([#{ class: "Server.Mobiles.SwampDragon" }, #{ barding: "Iron" }]);
                    }
                }
            },
            3 => {
                m.creature_item = [
                    #{
                        pick: ["Server.Items.Kryss", "Server.Items.Broadsword", "Server.Items.Katana"],
                        movable: false,
                        to: "add",
                    },
                    #{ item: "Server.Items.DragonChest", movable: false, to: "add", resource: "YellowScales" },
                    #{ item: "Server.Items.DragonLegs", movable: false, to: "add", resource: "YellowScales" },
                    #{ item: "Server.Items.DragonArms", movable: false, to: "add", resource: "YellowScales" },
                    #{ item: "Server.Items.DragonGloves", movable: false, to: "add", resource: "YellowScales" },
                    #{ item: "Server.Items.DragonHelm", movable: false, to: "add", resource: "YellowScales" },
                    #{ item: "Server.Items.ChaosShield", movable: false, to: "add" },
                    #{ item: "Server.Items.Boots", hue: 1109, to: "add" },
                    #{ item: "Server.Items.Shirt", hue: #{ from: 2401, count: 30 }, to: "add" },
                    #{ item: "Server.Items.YellowScales", args: [#{ min: 1, max: 3 }], to: "add" },
                ];
                switch random_below(9) {
                    0 => {
                        m.creature_rider = block([#{ class: "Server.Mobiles.SwampDragon" }, #{ barding: "DullCopper" }]);
                    },
                    1 => {
                        m.creature_rider = block([#{ class: "Server.Mobiles.SwampDragon" }, #{ barding: "ShadowIron" }]);
                    },
                    2 => {
                        m.creature_rider = block([#{ class: "Server.Mobiles.SwampDragon" }, #{ barding: "Copper" }]);
                    },
                    3 => {
                        m.creature_rider = block([#{ class: "Server.Mobiles.SwampDragon" }, #{ barding: "Bronze" }]);
                    },
                    4 => {
                        m.creature_rider = block([#{ class: "Server.Mobiles.SwampDragon" }, #{ barding: "Gold" }]);
                    },
                    5 => {
                        m.creature_rider = block([#{ class: "Server.Mobiles.SwampDragon" }, #{ barding: "Agapite" }]);
                    },
                    6 => {
                        m.creature_rider = block([#{ class: "Server.Mobiles.SwampDragon" }, #{ barding: "Verite" }]);
                    },
                    7 => {
                        m.creature_rider = block([#{ class: "Server.Mobiles.SwampDragon" }, #{ barding: "Valorite" }]);
                    },
                    8 => {
                        m.creature_rider = block([#{ class: "Server.Mobiles.SwampDragon" }, #{ barding: "Iron" }]);
                    }
                }
            },
            4 => {
                m.creature_item = [
                    #{
                        pick: ["Server.Items.Kryss", "Server.Items.Broadsword", "Server.Items.Katana"],
                        movable: false,
                        to: "add",
                    },
                    #{ item: "Server.Items.DragonChest", movable: false, to: "add", resource: "GreenScales" },
                    #{ item: "Server.Items.DragonLegs", movable: false, to: "add", resource: "GreenScales" },
                    #{ item: "Server.Items.DragonArms", movable: false, to: "add", resource: "GreenScales" },
                    #{ item: "Server.Items.DragonGloves", movable: false, to: "add", resource: "GreenScales" },
                    #{ item: "Server.Items.DragonHelm", movable: false, to: "add", resource: "GreenScales" },
                    #{ item: "Server.Items.ChaosShield", movable: false, to: "add" },
                    #{ item: "Server.Items.Boots", hue: 1109, to: "add" },
                    #{ item: "Server.Items.Shirt", hue: #{ from: 2401, count: 30 }, to: "add" },
                    #{ item: "Server.Items.GreenScales", args: [#{ min: 1, max: 3 }], to: "add" },
                ];
                switch random_below(9) {
                    0 => {
                        m.creature_rider = block([#{ class: "Server.Mobiles.SwampDragon" }, #{ barding: "DullCopper" }]);
                    },
                    1 => {
                        m.creature_rider = block([#{ class: "Server.Mobiles.SwampDragon" }, #{ barding: "ShadowIron" }]);
                    },
                    2 => {
                        m.creature_rider = block([#{ class: "Server.Mobiles.SwampDragon" }, #{ barding: "Copper" }]);
                    },
                    3 => {
                        m.creature_rider = block([#{ class: "Server.Mobiles.SwampDragon" }, #{ barding: "Bronze" }]);
                    },
                    4 => {
                        m.creature_rider = block([#{ class: "Server.Mobiles.SwampDragon" }, #{ barding: "Gold" }]);
                    },
                    5 => {
                        m.creature_rider = block([#{ class: "Server.Mobiles.SwampDragon" }, #{ barding: "Agapite" }]);
                    },
                    6 => {
                        m.creature_rider = block([#{ class: "Server.Mobiles.SwampDragon" }, #{ barding: "Verite" }]);
                    },
                    7 => {
                        m.creature_rider = block([#{ class: "Server.Mobiles.SwampDragon" }, #{ barding: "Valorite" }]);
                    },
                    8 => {
                        m.creature_rider = block([#{ class: "Server.Mobiles.SwampDragon" }, #{ barding: "Iron" }]);
                    }
                }
            },
            5 => {
                m.creature_item = [
                    #{
                        pick: ["Server.Items.Kryss", "Server.Items.Broadsword", "Server.Items.Katana"],
                        movable: false,
                        to: "add",
                    },
                    #{ item: "Server.Items.DragonChest", movable: false, to: "add", resource: "WhiteScales" },
                    #{ item: "Server.Items.DragonLegs", movable: false, to: "add", resource: "WhiteScales" },
                    #{ item: "Server.Items.DragonArms", movable: false, to: "add", resource: "WhiteScales" },
                    #{ item: "Server.Items.DragonGloves", movable: false, to: "add", resource: "WhiteScales" },
                    #{ item: "Server.Items.DragonHelm", movable: false, to: "add", resource: "WhiteScales" },
                    #{ item: "Server.Items.ChaosShield", movable: false, to: "add" },
                    #{ item: "Server.Items.Boots", hue: 1109, to: "add" },
                    #{ item: "Server.Items.Shirt", hue: #{ from: 2401, count: 30 }, to: "add" },
                    #{ item: "Server.Items.WhiteScales", args: [#{ min: 1, max: 3 }], to: "add" },
                ];
                switch random_below(9) {
                    0 => {
                        m.creature_rider = block([#{ class: "Server.Mobiles.SwampDragon" }, #{ barding: "DullCopper" }]);
                    },
                    1 => {
                        m.creature_rider = block([#{ class: "Server.Mobiles.SwampDragon" }, #{ barding: "ShadowIron" }]);
                    },
                    2 => {
                        m.creature_rider = block([#{ class: "Server.Mobiles.SwampDragon" }, #{ barding: "Copper" }]);
                    },
                    3 => {
                        m.creature_rider = block([#{ class: "Server.Mobiles.SwampDragon" }, #{ barding: "Bronze" }]);
                    },
                    4 => {
                        m.creature_rider = block([#{ class: "Server.Mobiles.SwampDragon" }, #{ barding: "Gold" }]);
                    },
                    5 => {
                        m.creature_rider = block([#{ class: "Server.Mobiles.SwampDragon" }, #{ barding: "Agapite" }]);
                    },
                    6 => {
                        m.creature_rider = block([#{ class: "Server.Mobiles.SwampDragon" }, #{ barding: "Verite" }]);
                    },
                    7 => {
                        m.creature_rider = block([#{ class: "Server.Mobiles.SwampDragon" }, #{ barding: "Valorite" }]);
                    },
                    8 => {
                        m.creature_rider = block([#{ class: "Server.Mobiles.SwampDragon" }, #{ barding: "Iron" }]);
                    }
                }
            }
        }
    });
});

mobile("JukaLord", |c| {
    c.ai = "Archer";
    c.range_perception = 10;
    c.range_fight = 3;
    c.corpse_name = "a jukan corpse";
    c.name = "a juka lord";
    c.always_murderer = true;
    c.can_rummage_corpses = true;
    c.meat = 1;
    c.loot = ["Rich", "Average"];
    c.bard_immune = #{ atLeast: "AOS", then: false, "else": true };
    c.idle_sound = 610;
    c.anger_sound = 611;
    c.hurt_sound = 464;
    c.death_sound = 653;
    c.on_create(|m| {
        m.body = 766;
        m.str = 401..=500;
        m.dex = 81..=100;
        m.int = 151..=200;
        m.hits = 241..=300;
        m.damage = 10..=12;
        m.damage_type("Physical", 100);
        m.resist("Physical", 40, 50);
        m.resist("Fire", 45, 50);
        m.resist("Cold", 40, 50);
        m.resist("Poison", 20, 25);
        m.resist("Energy", 40, 50);
        m.skill("Anatomy", 90.1, 100.0);
        m.skill("Archery", 95.1, 100.0);
        m.skill("Healing", 80.1, 100.0);
        m.skill("MagicResist", 120.1, 130.0);
        m.skill("Swords", 90.1, 100.0);
        m.skill("Tactics", 95.1, 100.0);
        m.skill("Wrestling", 90.1, 100.0);
        m.fame = 15000;
        m.karma = -15000;
        m.armor = 28;
        m.creature_item = [
            #{
                item: "Server.Items.Backpack",
                contents: [
                    #{ item: "Server.Items.Arrow", args: [#{ min: 25, max: 35 }] },
                    #{ item: "Server.Items.Arrow", args: [#{ min: 25, max: 35 }] },
                    #{ item: "Server.Items.Bandage", args: [#{ min: 5, max: 15 }] },
                    #{ item: "Server.Items.Bandage", args: [#{ min: 5, max: 15 }] },
                    #{ loot: "Gem" },
                    #{ item: "Server.Items.ArcaneGem" },
                ],
            },
            #{ item: "Server.Items.JukaBow", to: "add" },
        ];
    });
});

mobile("JukaMage", |c| {
    c.ai = "Mage";
    c.corpse_name = "a jukan corpse";
    c.name = "a juka mage";
    c.always_murderer = true;
    c.can_rummage_corpses = true;
    c.meat = 1;
    c.loot = [["Average", 2], ["MedScrolls", 2]];
    c.idle_sound = 428;
    c.anger_sound = 461;
    c.hurt_sound = 464;
    c.death_sound = 653;
    c.on_create(|m| {
        m.body = 765;
        m.str = 201..=300;
        m.dex = 71..=90;
        m.int = 451..=500;
        m.hits = 121..=180;
        m.damage = 4..=10;
        m.damage_type("Physical", 100);
        m.resist("Physical", 20, 30);
        m.resist("Fire", 35, 45);
        m.resist("Cold", 30, 40);
        m.resist("Poison", 10, 20);
        m.resist("Energy", 35, 45);
        m.skill("Anatomy", 80.1, 90.0);
        m.skill("EvalInt", 80.2, 100.0);
        m.skill("Magery", 99.1, 100.0);
        m.skill("Meditation", 80.2, 100.0);
        m.skill("MagicResist", 140.1, 150.0);
        m.skill("Tactics", 80.1, 90.0);
        m.skill("Wrestling", 80.1, 90.0);
        m.fame = 15000;
        m.karma = -15000;
        m.armor = 16;
        m.creature_next_ability = [2, 5];
    });
});

mobile("MeerCaptain", |c| {
    c.ai = "Archer";
    c.fight_mode = "Evil";
    c.corpse_name = "a meer corpse";
    c.name = "a meer captain";
    c.can_rummage_corpses = true;
    c.initial_innocent = true;
    c.loot = "Meager";
    c.bard_immune = #{ atLeast: "AOS", then: false, "else": true };
    c.hurt_sound = 333;
    c.death_sound = 788;
    c.attack_sound = 117;
    c.on_create(|m| {
        m.body = 773;
        m.str = 96..=110;
        m.dex = 186..=200;
        m.int = 96..=110;
        m.hits = 58..=66;
        m.damage = 5..=15;
        m.damage_type("Physical", 100);
        m.resist("Physical", 45, 55);
        m.resist("Fire", 10, 20);
        m.resist("Cold", 40, 50);
        m.resist("Poison", 35, 45);
        m.resist("Energy", 35, 45);
        m.skill("Archery", 90.1, 100.0);
        m.skill("MagicResist", 91.0, 100.0);
        m.skill("Swords", 90.1, 100.0);
        m.skill("Tactics", 91.0, 100.0);
        m.skill("Wrestling", 80.9, 89.9);
        m.fame = 2000;
        m.karma = 5000;
        m.armor = 28;
        m.creature_item = [
            #{
                item: "Server.Items.Backpack",
                contents: [
                    #{ item: "Server.Items.Bolt", args: [#{ min: 10, max: 20 }], times: 2 },
                    #{
                        pick: [
                            "Server.Items.Longsword",
                            "Server.Items.Cutlass",
                            "Server.Items.Broadsword",
                            "Server.Items.Katana",
                            "Server.Items.Scimitar",
                            "Server.Items.VikingSword",
                        ],
                    },
                    #{
                        item: "Server.Items.Bag",
                        contents: [#{ loot: "Reagent", times: #{ min: 10, max: 20 }, to: "tryDrop" }],
                    },
                ],
                keep: "pack",
            },
            #{ item: "Server.Items.Crossbow", to: "add" },
            #{ held: "pack" },
        ];
        m.creature_next_ability = [2, 5];
    });
});

mobile("MeerEternal", |c| {
    c.ai = "Mage";
    c.fight_mode = "Evil";
    c.corpse_name = "a meer's corpse";
    c.name = "a meer eternal";
    c.auto_dispel = true;
    c.can_rummage_corpses = true;
    c.poison_immune = 4;
    c.initial_innocent = true;
    c.loot = [["FilthyRich", 2], ["MedScrolls", 2], ["HighScrolls", 2]];
    c.bard_immune = #{ atLeast: "AOS", then: false, "else": true };
    c.treasure_map_level = #{ atLeast: "AOS", then: 5, "else": 4 };
    c.hurt_sound = 359;
    c.death_sound = 188;
    c.attack_sound = 651;
    c.on_create(|m| {
        m.body = 772;
        m.str = 416..=505;
        m.dex = 146..=165;
        m.int = 566..=655;
        m.hits = 250..=303;
        m.damage = 11..=13;
        m.damage_type("Physical", 100);
        m.resist("Physical", 45, 55);
        m.resist("Fire", 15, 25);
        m.resist("Cold", 45, 55);
        m.resist("Poison", 30, 40);
        m.resist("Energy", 30, 40);
        m.skill("EvalInt", 90.1, 100.0);
        m.skill("Magery", 90.1, 100.0);
        m.skill("Meditation", 90.1, 100.0);
        m.skill("MagicResist", 150.5, 200.0);
        m.skill("Tactics", 50.1, 70.0);
        m.skill("Wrestling", 60.1, 80.0);
        m.fame = 18000;
        m.karma = 18000;
        m.armor = 34;
        m.creature_next_ability = [2, 5];
    });
});

mobile("MeerMage", |c| {
    c.ai = "Mage";
    c.fight_mode = "Evil";
    c.corpse_name = "a meer's corpse";
    c.name = "a meer mage";
    c.auto_dispel = true;
    c.poison_immune = 4;
    c.can_rummage_corpses = true;
    c.treasure_map_level = 3;
    c.initial_innocent = true;
    c.loot = ["FilthyRich", ["MedScrolls", 2]];
    c.hurt_sound = 333;
    c.death_sound = 788;
    c.attack_sound = 117;
    c.on_create(|m| {
        m.body = 770;
        m.str = 171..=200;
        m.dex = 126..=145;
        m.int = 276..=305;
        m.hits = 103..=120;
        m.damage = 24..=26;
        m.damage_type("Physical", 100);
        m.resist("Physical", 45, 55);
        m.resist("Fire", 15, 25);
        m.resist("Cold", 50);
        m.resist("Poison", 25, 35);
        m.resist("Energy", 25, 35);
        m.skill("EvalInt", 100.0);
        m.skill("Magery", 70.1, 80.0);
        m.skill("Meditation", 85.1, 95.0);
        m.skill("MagicResist", 80.1, 100.0);
        m.skill("Tactics", 70.1, 90.0);
        m.skill("Wrestling", 60.1, 80.0);
        m.fame = 8000;
        m.karma = 8000;
        m.armor = 16;
        m.creature_next_ability = [2, 5];
    });
});

mobile("CuSidhe", |c| {
    c.ai = "Animal";
    c.fight_mode = "Aggressor";
    c.mount_body = 277;
    c.mount_item_id = 16017;
    c.corpse_name = "a cu sidhe corpse";
    c.name = "a cu sidhe";
    c.can_heal = true;
    c.can_heal_owner = true;
    c.favorite_food = ["FruitsAndVeggies"];
    c.can_anger_on_tame = true;
    c.stat_loss_after_tame = true;
    c.hides = 10;
    c.meat = 3;
    c.weapon_ability = "BleedAttack";
    c.loot = [["AosFilthyRich", 5]];
    c.idle_sound = 1399;
    c.attack_sound = 1398;
    c.anger_sound = 1400;
    c.hurt_sound = 1398;
    c.death_sound = 1401;
    c.on_create(|m| {
        m.creature_hue = block([
            #{ double: 23301.0 },
            #{ cases: [
                #{ below: 1.0, orEqual: true, value: 1161 },
                #{ below: 50.0, value: #{ pick: [1623, 1301, 1201, 1153, 1154, 1109] } },
                #{ below: 500.0, value: #{ pick: [2426, 2424, 2305, 2220, 1447, 1319] } },
            ] },
        ]);
    });
});

mobile("Ferret", |c| {
    c.ai = "Animal";
    c.fight_mode = "Aggressor";
    c.corpse_name = "a ferret corpse";
    c.name = "a ferret";
    c.meat = 1;
    c.favorite_food = ["Fish"];
    c.on_create(|m| {
        m.body = 279;
        m.str = 41..=48;
        m.dex = 55;
        m.int = 75;
        m.hits = 45..=50;
        m.damage = 7..=9;
        m.damage_type("Physical", 100);
        m.resist("Physical", 45, 50);
        m.resist("Fire", 10, 14);
        m.resist("Cold", 30, 40);
        m.resist("Poison", 21, 25);
        m.resist("Energy", 20, 25);
        m.skill("MagicResist", 4.0);
        m.skill("Tactics", 4.0);
        m.skill("Wrestling", 4.0);
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = -21.3;
        m.creature_setup = "CanTalk";
    });
});

mobile("LadyJennifyr", |c| {
    c.inherits = "SkeletalKnight";
    c.corpse_name = "a Lady Jennifyr corpse";
    c.name = "Lady Jennifyr";
    c.gives_ml_minor_artifact = true;
    c.loot = [["UltraRich", 3]];
    c.on_create(|m| {
        m.paragon = true;
        m.hue = 0x76D;
        m.str = 208..=309;
        m.dex = 91..=118;
        m.int = 44..=101;
        m.hits = 1113..=1285;
        m.damage = 15..=25;
        m.damage_type("Physical", 40);
        m.damage_type("Cold", 60);
        m.resist("Physical", 56, 65);
        m.resist("Fire", 41, 49);
        m.resist("Cold", 71, 80);
        m.resist("Poison", 41, 50);
        m.resist("Energy", 50, 58);
        m.skill("Wrestling", 127.9, 137.1);
        m.skill("Tactics", 128.4, 141.9);
        m.skill("MagicResist", 102.1, 119.5);
        m.skill("Anatomy", 129.0, 137.5);
        m.fame = 18000;
        m.karma = -18000;
    });
});

mobile("MasterJonath", |c| {
    c.inherits = "BoneMagi";
    c.corpse_name = "a Master Jonath corpse";
    c.name = "Master Jonath";
    c.gives_ml_minor_artifact = true;
    c.treasure_map_level = 5;
    c.loot = [["UltraRich", 3]];
    c.on_create(|m| {
        m.paragon = true;
        m.hue = 0x455;
        m.str = 109..=131;
        m.dex = 98..=110;
        m.int = 232..=259;
        m.hits = 766..=920;
        m.damage = 10..=15;
        m.damage_type("Physical", 100);
        m.resist("Physical", 55, 60);
        m.resist("Fire", 43, 49);
        m.resist("Cold", 45, 80);
        m.resist("Poison", 41, 45);
        m.resist("Energy", 54, 55);
        m.skill("Wrestling", 80.5, 88.6);
        m.skill("Tactics", 88.5, 95.1);
        m.skill("MagicResist", 102.7, 102.9);
        m.skill("Magery", 100.0, 106.6);
        m.skill("EvalInt", 99.6, 106.9);
        m.skill("Necromancy", 100.0, 106.6);
        m.skill("SpiritSpeak", 99.6, 106.9);
        m.fame = 18000;
        m.karma = -18000;
        if coin() {
            m.creature_necro_scroll = block([#{ min: 5 }, #{ max: 9 }]);
        } else {
            m.pack = [#{ scroll: [4, 7] }];
        }
        m.pack = [#{ reg: 7 }, #{ reg: 7 }, #{ reg: 8 }];
    });
});

mobile("MasterMikael", |c| {
    c.inherits = "BoneMagi";
    c.corpse_name = "a Master Mikael corpse";
    c.name = "Master Mikael";
    c.gives_ml_minor_artifact = true;
    c.loot = [["UltraRich", 2]];
    c.on_create(|m| {
        m.paragon = true;
        m.hue = 0x8FD;
        m.str = 93..=122;
        m.dex = 91..=100;
        m.int = 252..=271;
        m.hits = 789..=1014;
        m.damage = 11..=19;
        m.damage_type("Physical", 100);
        m.resist("Physical", 55, 59);
        m.resist("Fire", 40, 46);
        m.resist("Cold", 72, 80);
        m.resist("Poison", 44, 49);
        m.resist("Energy", 50, 57);
        m.skill("Wrestling", 80.1, 87.2);
        m.skill("Tactics", 79.0, 90.9);
        m.skill("MagicResist", 90.3, 106.9);
        m.skill("Magery", 103.8, 108.0);
        m.skill("EvalInt", 96.1, 105.3);
        m.skill("Necromancy", 103.8, 108.0);
        m.skill("SpiritSpeak", 96.1, 105.3);
        m.fame = 18000;
        m.karma = -18000;
        if coin() {
            m.creature_necro_scroll = block([#{ min: 5 }, #{ max: 9 }]);
        } else {
            m.pack = [#{ scroll: [4, 7] }];
        }
        m.pack = [#{ reg: 3 }, #{ necroReg: [1, 10] }];
    });
});

mobile("MasterTheophilus", |c| {
    c.inherits = "EvilMageLord";
    c.corpse_name = "a Master Theophilus corpse";
    c.name = "Master Theophilus";
    c.gives_ml_minor_artifact = true;
    c.treasure_map_level = 5;
    c.weapon_ability = "ParalyzingBlow";
    c.loot = [["UltraRich", 3]];
    c.on_create(|m| {
        m.paragon = true;
        m.title = "the necromancer";
        m.hue = 0;
        m.str = 137..=187;
        m.dex = 253..=301;
        m.int = 393..=444;
        m.hits = 663..=876;
        m.damage = 15..=20;
        m.damage_type("Physical", 100);
        m.resist("Physical", 55, 60);
        m.resist("Fire", 50, 58);
        m.resist("Cold", 50, 60);
        m.resist("Poison", 50, 60);
        m.resist("Energy", 50, 60);
        m.skill("Wrestling", 69.9, 105.3);
        m.skill("Tactics", 113.0, 117.9);
        m.skill("MagicResist", 127.0, 132.8);
        m.skill("Magery", 138.1, 143.7);
        m.skill("EvalInt", 125.6, 133.8);
        m.skill("Necromancy", 125.6, 133.8);
        m.skill("SpiritSpeak", 125.6, 133.8);
        m.skill("Meditation", 128.8, 132.9);
        m.fame = 18000;
        m.karma = -18000;
        m.creature_item = [
            #{ item: "Server.Items.Shoes", hue: 1335, to: "replace" },
            #{ item: "Server.Items.Robe", hue: 1106, to: "replace" },
        ];
        if coin() {
            m.creature_necro_scroll = block([#{ min: 5 }, #{ max: 9 }]);
            if coin() {
                m.creature_necro_scroll = block([#{ min: 5 }, #{ max: 9 }]);
            } else {
                m.pack = [#{ scroll: [4, 7] }];
            }
        } else {
            m.pack = [#{ scroll: [4, 7] }];
            if coin() {
                m.creature_necro_scroll = block([#{ min: 5 }, #{ max: 9 }]);
            } else {
                m.pack = [#{ scroll: [4, 7] }];
            }
        }
        m.pack = [#{ reg: 7 }, #{ reg: 7 }, #{ reg: 8 }];
    });
});

mobile("RedDeath", |c| {
    c.inherits = "SkeletalMount";
    c.mount_body = 793;
    c.mount_item_id = 16059;
    c.name = "Red Death";
    c.corpse_name = "a Red Death corpse";
    c.gives_ml_minor_artifact = true;
    c.always_murderer = true;
    c.weapon_ability = "WhirlwindAttack";
    c.loot = [["UltraRich", 3]];
    c.monster_abilities = ["ChaosBreath"];
    c.on_create(|m| {
        m.paragon = true;
        m.hue = 0x21;
        m.sound = 0x1C3;
        m.creature_ai = "AiMelee";
        m.creature_fight_mode = "Closest";
        m.str = 319..=324;
        m.dex = 241..=244;
        m.int = 242..=255;
        m.hits = 1540..=1605;
        m.damage = 25..=29;
        m.damage_type("Physical", 25);
        m.damage_type("Fire", 75);
        m.damage_type("Cold", 0);
        m.resist("Physical", 60, 70);
        m.resist("Fire", 90);
        m.resist("Cold", 0);
        m.resist("Poison", 100);
        m.resist("Energy", 0);
        m.skill("Wrestling", 121.4, 143.7);
        m.skill("Tactics", 120.9, 142.2);
        m.skill("MagicResist", 120.1, 142.3);
        m.skill("Anatomy", 120.2, 144.0);
        m.fame = 28000;
        m.karma = -28000;
        if coin() {
            m.creature_necro_scroll = block([#{ min: 5 }, #{ max: 9 }]);
        } else {
            m.pack = [#{ scroll: [4, 7] }];
        }
    });
});

mobile("SirPatrick", |c| {
    c.inherits = "SkeletalKnight";
    c.corpse_name = "a Sir Patrick corpse";
    c.name = "Sir Patrick";
    c.gives_ml_minor_artifact = true;
    c.loot = [["UltraRich", 2]];
    c.on_create(|m| {
        m.paragon = true;
        m.hue = 0x47E;
        m.str = 208..=319;
        m.dex = 98..=132;
        m.int = 45..=91;
        m.hits = 616..=884;
        m.damage = 15..=25;
        m.damage_type("Physical", 40);
        m.damage_type("Cold", 60);
        m.resist("Physical", 55, 62);
        m.resist("Fire", 40, 48);
        m.resist("Cold", 71, 80);
        m.resist("Poison", 40, 50);
        m.resist("Energy", 50, 60);
        m.skill("Wrestling", 126.3, 136.5);
        m.skill("Tactics", 128.5, 143.8);
        m.skill("MagicResist", 102.8, 117.9);
        m.skill("Anatomy", 127.5, 137.2);
        m.fame = 18000;
        m.karma = -18000;
    });
});

mobile("Satyr", |c| {
    c.ai = "Animal";
    c.fight_mode = "Aggressor";
    c.corpse_name = "a satyr's corpse";
    c.name = "a satyr";
    c.opposition = "FeyAndUndead";
    c.meat = 1;
    c.loot = "MlRich";
    c.on_create(|m| {
        m.body = 271;
        m.sound = 0x586;
        m.str = 177..=195;
        m.dex = 251..=269;
        m.int = 153..=170;
        m.hits = 350..=400;
        m.damage = 13..=24;
        m.damage_type("Physical", 100);
        m.resist("Physical", 55, 60);
        m.resist("Fire", 25, 35);
        m.resist("Cold", 30, 40);
        m.resist("Poison", 30, 40);
        m.resist("Energy", 30, 40);
        m.skill("MagicResist", 55.0, 65.0);
        m.skill("Tactics", 80.0, 100.0);
        m.skill("Wrestling", 80.0, 100.0);
        m.fame = 5000;
        m.karma = 0;
        m.armor = 28;
        if era("ML") {
            if chance(0.05) {
                m.creature_item = #{ loot: "ArcanistScroll" };
            }
        }
    });
});

mobile("EnslavedSatyr", |c| {
    c.inherits = "Satyr";
    c.corpse_name = "an enslaved satyr corpse";
    c.name = "an enslaved satyr";
});

mobile("MLDryad", |c| {
    c.ai = "Mage";
    c.fight_mode = "Evil";
    c.corpse_name = "a dryad's corpse";
    c.initial_innocent = true;
    c.opposition = "FeyAndUndead";
    c.name = "a dryad";
    c.meat = 1;
    c.loot = "MlRich";
    c.on_create(|m| {
        m.body = 266;
        m.sound = 0x57B;
        m.str = 132..=149;
        m.dex = 152..=168;
        m.int = 251..=280;
        m.hits = 304..=321;
        m.damage = 11..=20;
        m.damage_type("Physical", 100);
        m.resist("Physical", 40, 50);
        m.resist("Fire", 15, 25);
        m.resist("Cold", 40, 45);
        m.resist("Poison", 30, 40);
        m.resist("Energy", 25, 35);
        m.skill("Meditation", 80.0, 90.0);
        m.skill("EvalInt", 70.0, 80.0);
        m.skill("Magery", 70.0, 80.0);
        m.skill("Anatomy", 0.0);
        m.skill("MagicResist", 100.0, 120.0);
        m.skill("Tactics", 70.0, 80.0);
        m.skill("Wrestling", 70.0, 80.0);
        m.fame = 5000;
        m.karma = 5000;
        m.armor = 28;
        if era("ML") {
            if chance(0.6) {
                m.creature_item = block([#{ loot: "PeculiarSeed" }, #{ group: 1 }]);
            }
        }
        switch random_below(1) {
            0 => {
                if era("ML") {
                    if chance(0.05) {
                        m.creature_item = #{ loot: "ArcanistScroll" };
                    }
                }
            }
        }
    });
});

mobile("InsaneDryad", |c| {
    c.inherits = "MLDryad";
    c.corpse_name = "an insane dryad corpse";
    c.initial_innocent = false;
    c.name = "an insane dryad";
});

mobile("BogThing", |c| {
    c.corpse_name = "a plant corpse";
    c.name = "a bog thing";
    c.poison_immune = 4;
    c.loot = [["Average", 2]];
    c.bard_immune = #{ atLeast: "AOS", then: false, "else": true };
    c.on_create(|m| {
        m.body = 780;
        m.str = 801..=900;
        m.dex = 46..=65;
        m.int = 36..=50;
        m.hits = 481..=540;
        m.mana = 0;
        m.damage = 10..=23;
        m.damage_type("Physical", 60);
        m.damage_type("Poison", 40);
        m.resist("Physical", 30, 40);
        m.resist("Fire", 20, 25);
        m.resist("Cold", 10, 15);
        m.resist("Poison", 40, 50);
        m.resist("Energy", 20, 25);
        m.skill("MagicResist", 90.1, 95.0);
        m.skill("Tactics", 70.1, 85.0);
        m.skill("Wrestling", 65.1, 80.0);
        m.fame = 8000;
        m.karma = -8000;
        m.armor = 28;
        if chance(0.25) {
            m.pack = [#{ item: "Server.Items.Board", amount: 10 }];
        } else {
            m.pack = [#{ item: "Server.Items.Log", amount: 10 }];
        }
        m.pack = [#{ reg: 3 }, #{ item: "Server.Engines.Plants.Seed" }, #{ item: "Server.Engines.Plants.Seed" }];
    });
});

mobile("Tangle", |c| {
    c.inherits = "BogThing";
    c.corpse_name = "a Tangle corpse";
    c.name = "Tangle";
    c.poison_immune = 4;
    c.loot = [["UltraRich", 3]];
    c.on_create(|m| {
        m.hue = 0x21;
        m.str = 870..=940;
        m.dex = 58..=74;
        m.int = 46..=58;
        m.hits = 2468..=2733;
        m.mana = 8..=12;
        m.damage = 15..=28;
        m.damage_type("Physical", 60);
        m.damage_type("Poison", 40);
        m.resist("Physical", 50, 57);
        m.resist("Fire", 40, 43);
        m.resist("Cold", 30, 35);
        m.resist("Poison", 61, 69);
        m.resist("Energy", 41, 45);
        m.skill("Wrestling", 77.8, 94.6);
        m.skill("Tactics", 90.6, 100.4);
        m.skill("MagicResist", 108.4, 114.0);
    });
});

mobile("CorruptedSoul", |c| {
    c.delete_corpse_on_death = true;
    c.name = "a corrupted soul";
    c.always_attackable = true;
    c.bleed_immune = true;
    c.always_murderer = true;
    c.attack_sound = 563;
    c.on_create(|m| {
        m.body = 970;
        m.hue = 0x453;
        m.str = 102..=115;
        m.dex = 101..=115;
        m.int = 203..=215;
        m.hits = 61..=69;
        m.creature_speed = [0.25, 2.5];
        m.damage = 4..=40;
        m.damage_type("Physical", 100);
        m.resist("Physical", 61, 74);
        m.resist("Fire", 22, 48);
        m.resist("Cold", 73, 100);
        m.resist("Poison", 0);
        m.resist("Energy", 51, 60);
        m.skill("MagicResist", 80.2, 89.4);
        m.skill("Tactics", 81.3, 89.9);
        m.skill("Wrestling", 80.1, 88.7);
        m.fame = 5000;
        m.karma = -5000;
    });
});

mobile("Pyre", |c| {
    c.inherits = "Phoenix";
    c.corpse_name = "a Pyre corpse";
    c.name = "Pyre";
    c.gives_ml_minor_artifact = true;
    c.treasure_map_level = 5;
    c.has_aura = true;
    c.loot = [["UltraRich", 3]];
    c.on_create(|m| {
        m.paragon = true;
        m.hue = 0x489;
        m.creature_fight_mode = "Closest";
        m.str = 605..=611;
        m.dex = 391..=519;
        m.int = 669..=818;
        m.hits = 1783..=1939;
        m.damage = 30..=30;
        m.damage_type("Physical", 50);
        m.damage_type("Fire", 50);
        m.resist("Physical", 65);
        m.resist("Fire", 72, 75);
        m.resist("Poison", 36, 41);
        m.resist("Energy", 50, 51);
        m.skill("Wrestling", 121.9, 130.6);
        m.skill("Tactics", 114.4, 117.4);
        m.skill("MagicResist", 147.7, 153.0);
        m.skill("Poisoning", 122.8, 124.0);
        m.skill("Magery", 121.8, 127.8);
        m.skill("EvalInt", 103.6, 117.0);
        m.skill("Meditation", 100.0, 110.0);
        m.fame = 21000;
        m.karma = -21000;
    });
});

mobile("Reptalon", |c| {
    c.mount_body = 276;
    c.mount_item_id = 16016;
    c.name = "a reptalon";
    c.corpse_name = "a reptalon corpse";
    c.treasure_map_level = 5;
    c.meat = 5;
    c.hides = 10;
    c.can_anger_on_tame = true;
    c.stat_loss_after_tame = true;
    c.favorite_food = ["Meat"];
    c.can_fly = true;
    c.weapon_ability = "ParalyzingBlow";
    c.loot = [["AosUltraRich", 3]];
    c.monster_abilities = ["FireBreath"];
    c.on_create(|m| {
        m.sound = 0x16A;
        m.str = 1001..=1025;
        m.dex = 152..=164;
        m.int = 251..=289;
        m.hits = 833..=931;
        m.damage = 21..=28;
        m.damage_type("Physical", 0);
        m.damage_type("Poison", 25);
        m.damage_type("Energy", 75);
        m.resist("Physical", 53, 64);
        m.resist("Fire", 35, 45);
        m.resist("Cold", 36, 45);
        m.resist("Poison", 52, 63);
        m.resist("Energy", 71, 83);
        m.skill("Wrestling", 101.5, 118.2);
        m.skill("Tactics", 101.7, 108.2);
        m.skill("MagicResist", 76.4, 89.9);
        m.skill("Anatomy", 56.4, 59.7);
        m.tamable = true;
        m.control_slots = 4;
        m.min_tame_skill = 101.1;
    });
});

mobile("Rend", |c| {
    c.inherits = "Reptalon";
    c.mount_body = 276;
    c.mount_item_id = 16016;
    c.corpse_name = "a Rend corpse";
    c.name = "Rend";
    c.gives_ml_minor_artifact = true;
    c.loot = [["UltraRich", 3]];
    c.on_create(|m| {
        m.paragon = true;
        m.hue = 0x455;
        m.str = 1261..=1284;
        m.dex = 363..=384;
        m.int = 601..=642;
        m.hits = 5176..=6100;
        m.damage = 26..=33;
        m.damage_type("Physical", 100);
        m.damage_type("Poison", 0);
        m.damage_type("Energy", 0);
        m.resist("Physical", 75, 85);
        m.resist("Fire", 81, 94);
        m.resist("Cold", 46, 55);
        m.resist("Poison", 35, 44);
        m.resist("Energy", 45, 52);
        m.skill("Wrestling", 136.3, 150.3);
        m.skill("Tactics", 133.4, 141.4);
        m.skill("MagicResist", 90.9, 110.0);
        m.skill("Anatomy", 66.6, 72.0);
        m.fame = 21000;
        m.karma = -21000;
    });
});

mobile("CorrosiveSlime", |c| {
    c.corpse_name = "a slimey corpse";
    c.name = "a corrosive slime";
    c.poison_immune = 1;
    c.hit_poison = 1;
    c.favorite_food = ["Fish"];
    c.loot = ["Poor", "Gems"];
    c.on_create(|m| {
        m.body = 51;
        m.sound = 0x1C8;
        m.creature_hue = block([#{ from: 2201 }, #{ count: 24 }]);
        m.str = 22..=34;
        m.dex = 16..=21;
        m.int = 16..=20;
        m.hits = 15..=19;
        m.damage = 1..=5;
        m.damage_type("Physical", 100);
        m.resist("Physical", 5, 10);
        m.resist("Poison", 15, 20);
        m.skill("Poisoning", 36.0, 49.1);
        m.skill("Anatomy", 0.0);
        m.skill("MagicResist", 15.9, 18.9);
        m.skill("Tactics", 24.6, 26.1);
        m.skill("Wrestling", 24.9, 26.1);
        m.fame = 300;
        m.karma = -300;
        m.armor = 8;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = 23.1;
    });
});

mobile("Grobu", |c| {
    c.inherits = "BlackBear";
    c.corpse_name = "a Grobu corpse";
    c.name = "Grobu";
    c.gives_ml_minor_artifact = true;
    c.loot = [["FilthyRich", 2]];
    c.on_create(|m| {
        m.paragon = true;
        m.hue = 0x455;
        m.creature_ai = "AiMelee";
        m.creature_fight_mode = "Closest";
        m.str = 192..=210;
        m.dex = 132..=150;
        m.int = 50..=52;
        m.hits = 1235..=1299;
        m.stam = 132..=150;
        m.mana = 9;
        m.damage = 15..=18;
        m.damage_type("Physical", 100);
        m.resist("Physical", 40, 45);
        m.resist("Fire", 20, 40);
        m.resist("Cold", 32, 35);
        m.resist("Poison", 25, 30);
        m.resist("Energy", 22, 34);
        m.skill("Wrestling", 96.4, 119.0);
        m.skill("Tactics", 96.2, 116.5);
        m.skill("MagicResist", 66.2, 83.7);
        m.fame = 1000;
        m.karma = 1000;
    });
});

mobile("SeaSerpent", |c| {
    c.ai = "Mage";
    c.corpse_name = "a sea serpents corpse";
    c.name = "a sea serpent";
    c.treasure_map_level = 2;
    c.hides = 10;
    c.hide_type = "Horned";
    c.scales = 8;
    c.scale_type = "Blue";
    c.loot = "Meager";
    c.monster_abilities = ["FireBreath"];
    c.on_create(|m| {
        m.body = 150;
        m.sound = 0x1BF;
        m.creature_hue = block([#{ from: 1328 }, #{ count: 9 }]);
        m.str = 168..=225;
        m.dex = 58..=85;
        m.int = 53..=95;
        m.hits = 110..=127;
        m.damage = 7..=13;
        m.damage_type("Physical", 100);
        m.resist("Physical", 25, 35);
        m.resist("Fire", 50, 60);
        m.resist("Cold", 30, 40);
        m.resist("Poison", 30, 40);
        m.resist("Energy", 15, 20);
        m.skill("MagicResist", 60.1, 75.0);
        m.skill("Tactics", 60.1, 70.0);
        m.skill("Wrestling", 60.1, 70.0);
        m.fame = 6000;
        m.karma = -6000;
        m.armor = 30;
        m.can_swim = true;
        m.cant_walk = true;
        if coin() {
            m.pack = [#{ item: "Server.Items.SulfurousAsh", amount: 4 }];
        } else {
            m.pack = [#{ item: "Server.Items.BlackPearl", amount: 4 }];
        }
        m.pack = [#{ item: "Server.Items.RawFishSteak" }];
    });
});

mobile("CrystalSeaSerpent", |c| {
    c.inherits = "SeaSerpent";
    c.corpse_name = "a crystal sea serpent corpse";
    c.name = "a crystal sea serpent";
    c.on_create(|m| {
        m.hue = 0x47E;
        m.str = 250..=450;
        m.dex = 100..=150;
        m.int = 90..=190;
        m.hits = 230..=330;
        m.damage = 10..=18;
        m.damage_type("Physical", 10);
        m.damage_type("Cold", 45);
        m.damage_type("Energy", 45);
        m.resist("Physical", 50, 70);
        m.resist("Fire", 0);
        m.resist("Cold", 70, 90);
        m.resist("Poison", 20, 30);
        m.resist("Energy", 60, 80);
    });
});

mobile("Wisp", |c| {
    c.ai = "Mage";
    c.fight_mode = "Aggressor";
    c.corpse_name = "a wisp corpse";
    c.reacquire_delay = 1.0;
    c.name = "a wisp";
    c.opposition = "FeyAndUndead";
    c.loot = ["Rich", "Average"];
    c.speech = "Wisp";
    c.on_create(|m| {
        m.body = 58;
        m.sound = 0x1D2;
        m.str = 196..=225;
        m.dex = 196..=225;
        m.int = 196..=225;
        m.hits = 118..=135;
        m.damage = 17..=18;
        m.damage_type("Physical", 50);
        m.damage_type("Energy", 50);
        m.resist("Physical", 35, 45);
        m.resist("Fire", 20, 40);
        m.resist("Cold", 10, 30);
        m.resist("Poison", 5, 10);
        m.resist("Energy", 50, 70);
        m.skill("EvalInt", 80.0);
        m.skill("Magery", 80.0);
        m.skill("MagicResist", 80.0);
        m.skill("Tactics", 80.0);
        m.skill("Wrestling", 80.0);
        m.fame = 4000;
        m.karma = 0;
        m.armor = 40;
        if era("ML") {
            if chance(0.33) {
                m.creature_item = block([#{ loot: "PeculiarSeed" }, #{ group: 3 }]);
            }
        }
        m.pack = [#{ item: "Server.Items.LightSource", equip: true }];
    });
});

mobile("CrystalWisp", |c| {
    c.inherits = "Wisp";
    c.name = "a crystal wisp";
    c.on_create(|m| {
        m.hue = 0x482;
        m.pack = [#{ arcaneScroll: [0, 1] }];
    });
});

mobile("Protector", |c| {
    c.corpse_name = "a human corpse";
    c.name = "a Protector";
    c.always_murderer = true;
    c.loot = "FilthyRich";
    c.property_title = false;
    c.show_fame_title = false;
    c.on_create(|m| {
        m.body = 401;
        m.female = true;
        m.creature_hue = #{ raceSkinHue: "Human" };
        m.creature_setup = "HumanHair";
        m.creature_setup = "HumanHairHue";
        m.title = "the mystic llamaherder";
        m.str = 700..=800;
        m.dex = 100..=150;
        m.int = 50..=75;
        m.hits = 350..=450;
        m.damage = 6..=12;
        m.damage_type("Physical", 100);
        m.resist("Physical", 30, 40);
        m.resist("Fire", 20, 30);
        m.resist("Cold", 35, 40);
        m.resist("Poison", 30, 40);
        m.resist("Energy", 30, 40);
        m.skill("Wrestling", 70.0, 100.0);
        m.skill("Tactics", 80.0, 100.0);
        m.skill("MagicResist", 50.0, 70.0);
        m.skill("Anatomy", 70.0, 100.0);
        m.fame = 10000;
        m.karma = -10000;
        m.creature_item = [
            #{ item: "Server.Items.ThighBoots", movable: false, hue: #{ from: 0, count: 2 }, to: "add" },
            #{ plain: 8270, layer: "OuterTorso", movable: false, hue: #{ from: 0, count: 2 }, to: "add" },
        ];
    });
});

mobile("Changeling", |c| {
    c.ai = "Mage";
    c.corpse_name = "a changeling corpse";
    c.name = "a changeling";
    c.loot = [["AosRich", 3]];
    c.show_fame_title = false;
    c.anger_sound = 1134;
    c.idle_sound = 1136;
    c.attack_sound = 1133;
    c.hurt_sound = 1137;
    c.death_sound = 1135;
    c.on_create(|m| {
        m.body = 264;
        m.creature_setup = "Changeling";
        m.creature_setup = "DefaultHue";
        m.str = 36..=105;
        m.dex = 212..=262;
        m.int = 317..=399;
        m.hits = 201..=211;
        m.stam = 212..=262;
        m.mana = 317..=399;
        m.damage = 9..=15;
        m.damage_type("Physical", 100);
        m.resist("Physical", 81, 90);
        m.resist("Fire", 40, 50);
        m.resist("Cold", 40, 49);
        m.resist("Poison", 40, 50);
        m.resist("Energy", 43, 50);
        m.skill("Wrestling", 10.4, 12.5);
        m.skill("Tactics", 101.1, 108.3);
        m.skill("MagicResist", 121.6, 132.2);
        m.skill("Magery", 91.6, 99.5);
        m.skill("EvalInt", 91.5, 98.8);
        m.skill("Meditation", 91.7, 98.5);
        m.fame = 15000;
        m.karma = -15000;
        m.pack = [
            #{ scroll: [1, 7] },
            #{ item: "Server.Items.Arrow", amount: 35 },
            #{ item: "Server.Items.Bolt", amount: 25 },
            #{ gem: 2 },
            #{ arcaneScroll: [0, 1] },
        ];
    });
});

mobile("Guile", |c| {
    c.inherits = "Changeling";
    c.corpse_name = "a Guile corpse";
    c.name = "Guile";
    c.default_hue = 63;
    c.gives_ml_minor_artifact = true;
    c.loot = [["UltraRich", 2]];
    c.monster_abilities = ["DrainLifeAttack"];
    c.on_create(|m| {
        m.paragon = true;
        m.creature_setup = "DefaultHue";
        m.str = 53..=214;
        m.dex = 243..=367;
        m.int = 369..=586;
        m.hits = 1013..=1058;
        m.stam = 243..=367;
        m.mana = 369..=586;
        m.damage = 14..=20;
        m.damage_type("Physical", 100);
        m.resist("Physical", 80, 90);
        m.resist("Fire", 43, 46);
        m.resist("Cold", 42, 44);
        m.resist("Poison", 42, 50);
        m.resist("Energy", 47, 50);
        m.skill("Wrestling", 12.8, 16.7);
        m.skill("Tactics", 102.6, 131.0);
        m.skill("MagicResist", 141.2, 161.6);
        m.skill("Magery", 108.4, 120.0);
        m.skill("EvalInt", 108.4, 120.0);
        m.skill("Meditation", 109.2, 120.0);
        m.fame = 21000;
        m.karma = -21000;
    });
});

mobile("Irk", |c| {
    c.inherits = "Changeling";
    c.corpse_name = "an Irk corpse";
    c.name = "Irk";
    c.default_hue = 1161;
    c.gives_ml_minor_artifact = true;
    c.loot = [["UltraRich", 2]];
    c.on_create(|m| {
        m.paragon = true;
        m.creature_setup = "DefaultHue";
        m.str = 23..=183;
        m.dex = 259..=360;
        m.int = 374..=600;
        m.hits = 1006..=1064;
        m.stam = 259..=360;
        m.mana = 374..=600;
        m.damage = 14..=20;
        m.resist("Physical", 80, 90);
        m.resist("Fire", 40, 50);
        m.resist("Cold", 40, 50);
        m.resist("Poison", 41, 50);
        m.resist("Energy", 40, 49);
        m.skill("Wrestling", 120.3, 123.0);
        m.skill("Tactics", 120.1, 131.8);
        m.skill("MagicResist", 132.3, 165.8);
        m.skill("Magery", 108.9, 119.7);
        m.skill("EvalInt", 108.4, 120.0);
        m.skill("Meditation", 108.9, 119.1);
        m.fame = 21000;
        m.karma = -21000;
    });
});

mobile("Spite", |c| {
    c.inherits = "Changeling";
    c.corpse_name = "a Spite corpse";
    c.name = "Spite";
    c.default_hue = 33;
    c.gives_ml_minor_artifact = true;
    c.loot = [["UltraRich", 2]];
    c.on_create(|m| {
        m.paragon = true;
        m.creature_setup = "DefaultHue";
        m.str = 53..=214;
        m.dex = 243..=367;
        m.int = 369..=586;
        m.hits = 1013..=1052;
        m.stam = 243..=367;
        m.mana = 369..=586;
        m.damage = 14..=20;
        m.damage_type("Physical", 100);
        m.resist("Physical", 85, 90);
        m.resist("Fire", 41, 46);
        m.resist("Cold", 40, 44);
        m.resist("Poison", 42, 46);
        m.resist("Energy", 45, 47);
        m.skill("Wrestling", 12.8, 16.7);
        m.skill("Tactics", 102.6, 131.0);
        m.skill("MagicResist", 141.2, 161.6);
        m.skill("Magery", 108.4, 119.2);
        m.skill("EvalInt", 108.4, 120.0);
        m.skill("Meditation", 109.2, 120.0);
        m.fame = 21000;
        m.karma = -21000;
    });
});

mobile("Swoop", |c| {
    c.inherits = "Eagle";
    c.corpse_name = "a Swoop corpse";
    c.name = "Swoop";
    c.can_fly = true;
    c.gives_ml_minor_artifact = true;
    c.feathers = 72;
    c.loot = [["UltraRich", 2]];
    c.on_create(|m| {
        m.paragon = true;
        m.hue = 0xE0;
        m.pack = [#{ arcaneScroll: [0, 1] }];
        m.str = 100..=150;
        m.dex = 400..=500;
        m.int = 80..=90;
        m.hits = 1500..=2000;
        m.damage = 20..=30;
        m.damage_type("Physical", 100);
        m.resist("Physical", 75, 90);
        m.resist("Fire", 60, 77);
        m.resist("Cold", 70, 85);
        m.resist("Poison", 55, 85);
        m.resist("Energy", 50, 60);
        m.skill("Wrestling", 120.0, 140.0);
        m.skill("Tactics", 120.0, 140.0);
        m.skill("MagicResist", 95.0, 105.0);
        m.fame = 18000;
        m.karma = 0;
        m.pack = [#{ reg: 4 }, #{ arcaneScroll: [0, 1] }];
    });
});

mobile("VorpalBunny", |c| {
    c.corpse_name = "a vorpal bunny corpse";
    c.name = "a vorpal bunny";
    c.meat = 1;
    c.hides = 1;
    c.loot = ["FilthyRich", ["Rich", 2]];
    c.bard_immune = #{ atLeast: "AOS", then: false, "else": true };
    c.attack_sound = 201;
    c.hurt_sound = 202;
    c.death_sound = 203;
    c.on_create(|m| {
        m.body = 205;
        m.hue = 0x480;
        m.str = 15;
        m.dex = 2000;
        m.int = 1000;
        m.hits = 2000;
        m.stam = 500;
        m.mana = 0;
        m.damage = 1..=1;
        m.damage_type("Physical", 100);
        m.skill("MagicResist", 200.0);
        m.skill("Tactics", 5.0);
        m.skill("Wrestling", 5.0);
        m.fame = 1000;
        m.karma = 0;
        m.armor = 4;
        m.pack = [#{ item: "Server.Items.Carrot", amount: [5, 10] }];
        m.creature_one_in = #{ n: 5, then: #{ pack: [#{ item: "Server.Items.BrightlyColoredEggs" }] } };
        m.pack = [#{ statue: 1 }];
        m.creature_setup = "VorpalBunnyTunnel";
    });
});

mobile("ShadowWisp", |c| {
    c.ai = "Mage";
    c.fight_mode = "Aggressor";
    c.corpse_name = "a wisp corpse";
    c.name = "a shadow wisp";
    c.opposition = "FeyAndUndead";
    c.on_create(|m| {
        m.body = 165;
        m.sound = 0x1D2;
        m.str = 16..=40;
        m.dex = 16..=45;
        m.int = 11..=25;
        m.hits = 10..=24;
        m.damage = 5..=10;
        m.damage_type("Physical", 100);
        m.resist("Physical", 15, 20);
        m.resist("Fire", 5, 10);
        m.resist("Poison", 5, 10);
        m.resist("Energy", 15, 20);
        m.skill("EvalInt", 40.0);
        m.skill("Magery", 50.0);
        m.skill("Meditation", 40.0);
        m.skill("MagicResist", 10.0);
        m.skill("Tactics", 0.1, 15.0);
        m.skill("Wrestling", 25.1, 40.0);
        m.fame = 500;
        m.armor = 18;
        m.pack = [#{ item: "Server.Items.LightSource", equip: true }];
        m.creature_item = #{
            pick: [
                "Server.Items.LeftArm",
                "Server.Items.RightArm",
                "Server.Items.Torso",
                "Server.Items.Bone",
                "Server.Items.RibCage",
                "Server.Items.RibCage",
                "Server.Items.BonePile",
                "Server.Items.BonePile",
                "Server.Items.BonePile",
                "Server.Items.BonePile",
            ],
        };
    });
});

mobile("BladeSpirits", |c| {
    c.corpse_name = "a blade spirit corpse";
    c.name = "a blade spirit";
    c.is_house_summonable = true;
    c.dispel_difficulty = 0.0;
    c.dispel_focus = 20.0;
    c.bleed_immune = true;
    c.poison_immune = 4;
    c.delete_corpse_on_death = #{ atLeast: "AOS", then: true, "else": false };
    c.anger_sound = 570;
    c.attack_sound = 952;
    c.hurt_sound = 570;
    c.on_create(|m| {
        m.body = 574;
        m.str = 150;
        m.dex = 150;
        m.int = 100;
        if era("SE") {
            m.hits = 160;
        } else {
            m.hits = 80;
        }
        m.stam = 250;
        m.mana = 0;
        m.damage = 10..=14;
        m.damage_type("Physical", 60);
        m.damage_type("Poison", 20);
        m.damage_type("Energy", 20);
        m.resist("Physical", 30, 40);
        m.resist("Fire", 40, 50);
        m.resist("Cold", 30, 40);
        m.resist("Poison", 100);
        m.resist("Energy", 20, 30);
        m.skill("MagicResist", 70.0);
        m.skill("Tactics", 90.0);
        m.skill("Wrestling", 90.0);
        m.fame = 0;
        m.karma = 0;
        m.armor = 40;
        switch random_below(1) {
            0 => {
                if era("SE") {
                    m.control_slots = 2;
                } else {
                    m.control_slots = 1;
                }
            }
        }
    });
});

mobile("EnergyVortex", |c| {
    c.corpse_name = "an energy vortex corpse";
    c.always_murderer = true;
    c.dispel_difficulty = 80.0;
    c.dispel_focus = 20.0;
    c.name = "an energy vortex";
    c.bleed_immune = true;
    c.poison_immune = 4;
    c.anger_sound = 21;
    c.attack_sound = 40;
    c.on_create(|m| {
        if era("SE") {
            m.creature_one_in = block([#{ n: 500 }, #{ then: #{ body: 220, hue: 118 } }, #{ "else": #{ body: 164 } }]);
        } else {
            m.body = 164;
        }
        m.str = 200;
        m.dex = 200;
        m.int = 100;
        switch random_below(1) {
            0 => {
                if era("SE") {
                    m.hits = 140;
                } else {
                    m.hits = 70;
                }
                m.stam = 250;
                m.mana = 0;
                m.damage = 14..=17;
                m.damage_type("Physical", 0);
                m.damage_type("Energy", 100);
                m.resist("Physical", 60, 70);
                m.resist("Fire", 40, 50);
                m.resist("Cold", 40, 50);
                m.resist("Poison", 40, 50);
                m.resist("Energy", 90, 100);
                m.skill("MagicResist", 99.9);
                m.skill("Tactics", 100.0);
                m.skill("Wrestling", 120.0);
                m.fame = 0;
                m.karma = 0;
                m.armor = 40;
                switch random_below(1) {
                    0 => {
                        if era("SE") {
                            m.control_slots = 2;
                        } else {
                            m.control_slots = 1;
                        }
                    }
                }
            }
        }
    });
});

mobile("PlagueBeast", |c| {
    c.corpse_name = "a plague beast corpse";
    c.name = "a plague beast";
    c.auto_dispel = true;
    c.poison_immune = 4;
    c.loot = ["FilthyRich", ["Gems", #{ from: 1, count: 3 }]];
    c.idle_sound = 447;
    c.attack_sound = 448;
    c.hurt_sound = 449;
    c.death_sound = 450;
    c.on_create(|m| {
        m.body = 775;
        m.str = 302..=500;
        m.dex = 80;
        m.int = 16..=20;
        m.hits = 318..=404;
        m.damage = 20..=24;
        m.damage_type("Physical", 60);
        m.damage_type("Poison", 40);
        m.resist("Physical", 45, 55);
        m.resist("Fire", 40, 50);
        m.resist("Cold", 25, 35);
        m.resist("Poison", 65, 75);
        m.resist("Energy", 25, 35);
        m.skill("MagicResist", 35.0);
        m.skill("Tactics", 100.0);
        m.skill("Wrestling", 100.0);
        m.fame = 13000;
        m.karma = -13000;
        m.armor = 30;
        m.creature_pack_armor = [1, 5, 1.0];
        m.pack = [#{ chance: 0.8, item: "Server.Items.PlagueBeastGland" }];
        if era("ML") {
            if chance(0.33) {
                m.creature_item = block([#{ loot: "PeculiarSeed" }, #{ group: 4 }]);
            }
        }
        m.creature_setup = "PlagueBeast";
    });
});

mobile("Slime", |c| {
    c.corpse_name = "a slimey corpse";
    c.name = "a slime";
    c.poison_immune = 0;
    c.hit_poison = 0;
    c.favorite_food = ["Meat", "Fish", "FruitsAndVeggies", "GrainsAndHay", "Eggs"];
    c.loot = ["Poor", "Gems"];
    c.on_create(|m| {
        m.body = 51;
        m.sound = 0x1C8;
        m.creature_hue = block([#{ from: 2201 }, #{ count: 24 }]);
        m.str = 22..=34;
        m.dex = 16..=21;
        m.int = 16..=20;
        m.hits = 15..=19;
        m.damage = 1..=5;
        m.damage_type("Physical", 100);
        m.resist("Physical", 5, 10);
        m.resist("Poison", 10, 20);
        m.skill("Poisoning", 30.1, 50.0);
        m.skill("MagicResist", 15.1, 20.0);
        m.skill("Tactics", 19.3, 34.0);
        m.skill("Wrestling", 19.3, 34.0);
        m.fame = 300;
        m.karma = -300;
        m.armor = 8;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = 23.1;
    });
});

mobile("Corpser", |c| {
    c.corpse_name = "a corpser corpse";
    c.name = "a corpser";
    c.poison_immune = 0;
    c.disallow_all_moves = true;
    c.loot = "Meager";
    c.on_create(|m| {
        m.body = 8;
        m.sound = 0x2AC;
        m.str = 156..=180;
        m.dex = 26..=45;
        m.int = 26..=40;
        m.hits = 94..=108;
        m.mana = 0;
        m.damage = 10..=23;
        m.damage_type("Physical", 60);
        m.damage_type("Poison", 40);
        m.resist("Physical", 15, 20);
        m.resist("Fire", 15, 25);
        m.resist("Cold", 10, 20);
        m.resist("Poison", 20, 30);
        m.skill("MagicResist", 15.1, 20.0);
        m.skill("Tactics", 45.1, 60.0);
        m.skill("Wrestling", 45.1, 60.0);
        m.fame = 1000;
        m.karma = -1000;
        m.armor = 18;
        if chance(0.25) {
            m.pack = [#{ item: "Server.Items.Board", amount: 10 }];
        } else {
            m.pack = [#{ item: "Server.Items.Log", amount: 10 }];
        }
        m.pack = [#{ item: "Server.Items.MandrakeRoot", amount: 3 }];
    });
});

mobile("DeepSeaSerpent", |c| {
    c.ai = "Mage";
    c.corpse_name = "a deep sea serpents corpse";
    c.name = "a deep sea serpent";
    c.meat = 1;
    c.scales = 8;
    c.scale_type = "Blue";
    c.loot = "Meager";
    c.monster_abilities = ["FireBreath"];
    c.on_create(|m| {
        m.body = 150;
        m.sound = 0x1BF;
        m.creature_hue = block([#{ from: 2208 }, #{ count: 5 }]);
        m.str = 251..=425;
        m.dex = 87..=135;
        m.int = 87..=155;
        m.hits = 151..=255;
        m.damage = 6..=14;
        m.damage_type("Physical", 100);
        m.resist("Physical", 30, 40);
        m.resist("Fire", 70, 80);
        m.resist("Cold", 40, 50);
        m.resist("Poison", 30, 40);
        m.resist("Energy", 15, 20);
        m.skill("MagicResist", 60.1, 75.0);
        m.skill("Tactics", 60.1, 70.0);
        m.skill("Wrestling", 60.1, 70.0);
        m.fame = 6000;
        m.karma = -6000;
        m.armor = 60;
        m.can_swim = true;
        m.cant_walk = true;
        if coin() {
            m.pack = [#{ item: "Server.Items.SulfurousAsh", amount: 4 }];
        } else {
            m.pack = [#{ item: "Server.Items.BlackPearl", amount: 4 }];
        }
    });
});

mobile("OphidianArchmage", |c| {
    c.ai = "Mage";
    c.corpse_name = "an ophidian corpse";
    c.meat = 1;
    c.opposition = "TerathansAndOphidians";
    c.loot = ["Rich", ["MedScrolls", 2]];
    c.on_create(|m| {
        switch random_below(2) {
            0 => {
                m.name = "an ophidian justicar";
            },
            1 => {
                m.name = "an ophidian zealot";
            }
        }
        m.body = 85;
        m.sound = 0x27F;
        m.str = 281..=305;
        m.dex = 191..=215;
        m.int = 226..=250;
        m.hits = 169..=183;
        m.stam = 36..=45;
        m.damage = 5..=10;
        m.damage_type("Physical", 100);
        m.resist("Physical", 40, 45);
        m.resist("Fire", 20, 30);
        m.resist("Cold", 25, 35);
        m.resist("Poison", 35, 40);
        m.resist("Energy", 25, 35);
        m.skill("EvalInt", 95.1, 100.0);
        m.skill("Magery", 95.1, 100.0);
        m.skill("MagicResist", 75.0, 97.5);
        m.skill("Tactics", 65.0, 87.5);
        m.skill("Wrestling", 20.2, 60.0);
        m.fame = 11500;
        m.karma = -11500;
        m.armor = 44;
        m.pack = [#{ reg: [5, 15] }, #{ necroReg: [5, 15] }];
    });
});

mobile("OphidianMage", |c| {
    c.ai = "Mage";
    c.corpse_name = "an ophidian corpse";
    c.meat = 1;
    c.treasure_map_level = 2;
    c.opposition = "TerathansAndOphidians";
    c.loot = ["Average", "LowScrolls", "MedScrolls", "Potions"];
    c.on_create(|m| {
        switch random_below(2) {
            0 => {
                m.name = "an ophidian apprentice mage";
            },
            1 => {
                m.name = "an ophidian shaman";
            }
        }
        m.body = 85;
        m.sound = 0x27F;
        m.str = 181..=205;
        m.dex = 191..=215;
        m.int = 96..=120;
        m.hits = 109..=123;
        m.damage = 5..=10;
        m.damage_type("Physical", 100);
        m.resist("Physical", 25, 35);
        m.resist("Fire", 30, 40);
        m.resist("Cold", 35, 45);
        m.resist("Poison", 40, 50);
        m.resist("Energy", 35, 45);
        m.skill("EvalInt", 85.1, 100.0);
        m.skill("Magery", 85.1, 100.0);
        m.skill("MagicResist", 75.0, 97.5);
        m.skill("Tactics", 65.0, 87.5);
        m.skill("Wrestling", 20.2, 60.0);
        m.fame = 4000;
        m.karma = -4000;
        m.armor = 30;
        m.pack = [#{ reg: 10 }];
    });
});

mobile("SerpentineDragon", |c| {
    c.ai = "Mage";
    c.fight_mode = "Evil";
    c.corpse_name = "a dragon corpse";
    c.name = "a serpentine dragon";
    c.reacquire_on_movement = true;
    c.auto_dispel = true;
    c.hide_type = "Barbed";
    c.hides = 20;
    c.meat = 19;
    c.scales = 6;
    c.treasure_map_level = 4;
    c.loot = [["FilthyRich", 2], ["Gems", 2]];
    c.bonus_pet_damage_scalar = #{ atLeast: "SE", then: 3.0, "else": 1.0 };
    c.scale_type = #{ coin: ["Black", "White"] };
    c.monster_abilities = ["FireBreath"];
    c.idle_sound = 708;
    c.attack_sound = 704;
    c.death_sound = 705;
    c.anger_sound = 708;
    c.hurt_sound = 707;
    c.on_create(|m| {
        m.body = 103;
        m.sound = 0x16A;
        m.str = 111..=140;
        m.dex = 201..=220;
        m.int = 1001..=1040;
        m.hits = 480;
        m.damage = 5..=12;
        m.damage_type("Physical", 75);
        m.damage_type("Poison", 25);
        m.resist("Physical", 35, 40);
        m.resist("Fire", 25, 35);
        m.resist("Cold", 25, 35);
        m.resist("Poison", 25, 35);
        m.resist("Energy", 25, 35);
        m.skill("EvalInt", 100.1, 110.0);
        m.skill("Magery", 110.1, 120.0);
        m.skill("Meditation", 100.0);
        m.skill("MagicResist", 100.0);
        m.skill("Tactics", 50.1, 60.0);
        m.skill("Wrestling", 30.1, 100.0);
        m.fame = 15000;
        m.karma = 15000;
        m.armor = 36;
        if era("ML") {
            if chance(0.33) {
                m.creature_item = block([#{ loot: "PeculiarSeed" }, #{ group: 2 }]);
            }
        }
    });
});

mobile("WhiteWyrm", |c| {
    c.ai = "Mage";
    c.corpse_name = "a white wyrm corpse";
    c.name = "a white wyrm";
    c.reacquire_on_movement = true;
    c.treasure_map_level = 4;
    c.meat = 19;
    c.hides = 20;
    c.hide_type = "Barbed";
    c.scales = 9;
    c.scale_type = "White";
    c.favorite_food = ["Meat", "Gold"];
    c.can_anger_on_tame = true;
    c.can_fly = true;
    c.loot = [["FilthyRich", 2], "Average", ["Gems", #{ from: 1, count: 5 }]];
    c.on_create(|m| {
        if coin() {
            m.body = 180;
        } else {
            m.body = 49;
        }
        m.sound = 0x16A;
        m.str = 721..=760;
        m.dex = 101..=130;
        m.int = 386..=425;
        m.hits = 433..=456;
        m.damage = 17..=25;
        m.damage_type("Physical", 50);
        m.damage_type("Cold", 50);
        m.resist("Physical", 55, 70);
        m.resist("Fire", 15, 25);
        m.resist("Cold", 80, 90);
        m.resist("Poison", 40, 50);
        m.resist("Energy", 40, 50);
        m.skill("EvalInt", 99.1, 100.0);
        m.skill("Magery", 99.1, 100.0);
        m.skill("MagicResist", 99.1, 100.0);
        m.skill("Tactics", 97.6, 100.0);
        m.skill("Wrestling", 90.1, 100.0);
        m.fame = 18000;
        m.karma = -18000;
        m.armor = 64;
        m.tamable = true;
        m.control_slots = 3;
        m.min_tame_skill = 96.3;
    });
});

mobile("Kraken", |c| {
    c.corpse_name = "a krakens corpse";
    c.name = "a kraken";
    c.treasure_map_level = 4;
    c.loot = "Rich";
    c.on_create(|m| {
        m.body = 77;
        m.sound = 0x161;
        m.str = 756..=780;
        m.dex = 226..=245;
        m.int = 26..=40;
        m.hits = 454..=468;
        m.mana = 0;
        m.damage = 19..=33;
        m.damage_type("Physical", 70);
        m.damage_type("Cold", 30);
        m.resist("Physical", 45, 55);
        m.resist("Fire", 30, 40);
        m.resist("Cold", 30, 40);
        m.resist("Poison", 20, 30);
        m.resist("Energy", 10, 20);
        m.skill("MagicResist", 15.1, 20.0);
        m.skill("Tactics", 45.1, 60.0);
        m.skill("Wrestling", 45.1, 60.0);
        m.fame = 11000;
        m.karma = -11000;
        m.armor = 50;
        m.can_swim = true;
        m.cant_walk = true;
        m.creature_item = #{ item: "Server.Items.Rope", itemId: 5368 };
        if era("UOR") {
            m.pack = [#{ item: "Server.Items.SpecialFishingNet" }];
        }
    });
});

mobile("OphidianKnight", |c| {
    c.corpse_name = "an ophidian corpse";
    c.meat = 2;
    c.poison_immune = 4;
    c.hit_poison = 4;
    c.treasure_map_level = 3;
    c.opposition = "TerathansAndOphidians";
    c.loot = [["Rich", 2]];
    c.on_create(|m| {
        switch random_below(2) {
            0 => {
                m.name = "an ophidian knight-errant";
            },
            1 => {
                m.name = "an ophidian avenger";
            }
        }
        m.body = 86;
        m.sound = 0x27A;
        m.str = 417..=595;
        m.dex = 166..=175;
        m.int = 46..=70;
        m.hits = 266..=342;
        m.mana = 0;
        m.damage = 16..=19;
        m.damage_type("Physical", 100);
        m.resist("Physical", 35, 40);
        m.resist("Fire", 30, 40);
        m.resist("Cold", 35, 45);
        m.resist("Poison", 90, 100);
        m.resist("Energy", 35, 45);
        m.skill("Poisoning", 60.1, 80.0);
        m.skill("MagicResist", 65.1, 80.0);
        m.skill("Tactics", 90.1, 100.0);
        m.skill("Wrestling", 90.1, 100.0);
        m.fame = 10000;
        m.karma = -10000;
        m.armor = 40;
        m.pack = [#{ item: "Server.Items.LesserPoisonPotion" }];
    });
});

mobile("OphidianWarrior", |c| {
    c.corpse_name = "an ophidian corpse";
    c.meat = 1;
    c.treasure_map_level = 1;
    c.opposition = "TerathansAndOphidians";
    c.loot = ["Meager", "Average", "Gems"];
    c.on_create(|m| {
        switch random_below(2) {
            0 => {
                m.name = "an ophidian warrior";
            },
            1 => {
                m.name = "an ophidian enforcer";
            }
        }
        m.body = 86;
        m.sound = 0x27A;
        m.str = 150..=320;
        m.dex = 94..=190;
        m.int = 64..=160;
        m.hits = 128..=155;
        m.mana = 0;
        m.damage = 5..=11;
        m.damage_type("Physical", 100);
        m.resist("Physical", 35, 40);
        m.resist("Fire", 20, 30);
        m.resist("Cold", 25, 35);
        m.resist("Poison", 30, 40);
        m.resist("Energy", 25, 35);
        m.skill("MagicResist", 70.1, 85.0);
        m.skill("Swords", 60.1, 85.0);
        m.skill("Tactics", 75.1, 90.0);
        m.fame = 4500;
        m.karma = -4500;
        m.armor = 36;
    });
});

mobile("BakeKitsune", |c| {
    c.ai = "Mage";
    c.corpse_name = "a bake kitsune corpse";
    c.name = "a bake kitsune";
    c.meat = 5;
    c.hides = 10;
    c.hide_type = "Barbed";
    c.favorite_food = ["Fish"];
    c.loot = ["FilthyRich", "Rich", ["MedScrolls", 2]];
    c.monster_abilities = ["BloodBathAttack"];
    c.show_fame_title = false;
    c.click_title = false;
    c.property_title = false;
    c.anger_sound = 1246;
    c.idle_sound = 1245;
    c.attack_sound = 1244;
    c.hurt_sound = 1247;
    c.death_sound = 1243;
    c.on_create(|m| {
        m.body = 246;
        m.str = 171..=220;
        m.dex = 126..=145;
        m.int = 376..=425;
        m.hits = 301..=350;
        m.damage = 15..=22;
        m.damage_type("Physical", 70);
        m.damage_type("Energy", 30);
        m.resist("Physical", 40, 60);
        m.resist("Fire", 70, 90);
        m.resist("Cold", 40, 60);
        m.resist("Poison", 40, 60);
        m.resist("Energy", 40, 60);
        m.skill("EvalInt", 80.1, 90.0);
        m.skill("Magery", 80.1, 90.0);
        m.skill("MagicResist", 80.1, 100.0);
        m.skill("Tactics", 70.1, 90.0);
        m.skill("Wrestling", 50.1, 55.0);
        m.fame = 8000;
        m.karma = -8000;
        m.tamable = true;
        m.control_slots = 2;
        m.min_tame_skill = 80.7;
        if chance(0.25) {
            m.creature_item = #{ loot: "BonsaiSeed" };
        }
    });
});

mobile("DeathwatchBeetle", |c| {
    c.corpse_name = "a deathwatchbeetle corpse";
    c.name = "a deathwatch beetle";
    c.hides = 8;
    c.weapon_ability = "CrushingBlow";
    c.loot = ["LowScrolls", "Potions"];
    c.anger_sound = 1267;
    c.idle_sound = 1266;
    c.attack_sound = 1265;
    c.hurt_sound = 1268;
    c.death_sound = 1264;
    c.on_create(|m| {
        if era("ML") {
            m.creature_fight_mode = "Aggressor";
        }
        m.body = 242;
        m.str = 136..=160;
        m.dex = 41..=52;
        m.int = 31..=40;
        m.hits = 121..=145;
        m.mana = 20;
        m.damage = 5..=10;
        m.damage_type("Physical", 100);
        m.resist("Physical", 35, 40);
        m.resist("Fire", 15, 30);
        m.resist("Cold", 15, 30);
        m.resist("Poison", 50, 80);
        m.resist("Energy", 20, 35);
        m.skill("MagicResist", 50.1, 58.0);
        m.skill("Tactics", 67.1, 77.0);
        m.skill("Wrestling", 50.1, 60.0);
        m.skill("Anatomy", 30.1, 34.0);
        m.fame = 1400;
        m.karma = -1400;
        m.creature_item = #{
            pick: [
                "Server.Items.LeatherGorget",
                "Server.Items.LeatherGloves",
                "Server.Items.LeatherArms",
                "Server.Items.LeatherLegs",
                "Server.Items.LeatherCap",
                "Server.Items.LeatherChest",
                (),
                (),
                (),
                (),
                (),
                (),
            ],
        };
        if chance(0.5) {
            m.creature_item = #{ loot: "BonsaiSeed" };
        }
        m.tamable = true;
        m.min_tame_skill = 41.1;
        m.control_slots = 1;
    });
});

mobile("DeathwatchBeetleHatchling", |c| {
    c.corpse_name = "a deathwatchbeetle hatchling corpse";
    c.name = "a deathwatch beetle hatchling";
    c.hides = 8;
    c.loot = ["LowScrolls", "Potions"];
    c.anger_sound = 1267;
    c.idle_sound = 1266;
    c.attack_sound = 1265;
    c.hurt_sound = 1268;
    c.death_sound = 1264;
    c.on_create(|m| {
        if era("ML") {
            m.creature_fight_mode = "Aggressor";
        }
        m.body = 242;
        m.str = 26..=50;
        m.dex = 41..=52;
        m.int = 21..=30;
        m.hits = 51..=60;
        m.mana = 20;
        m.damage = 2..=8;
        m.damage_type("Physical", 100);
        m.resist("Physical", 35, 40);
        m.resist("Fire", 15, 30);
        m.resist("Cold", 15, 30);
        m.resist("Poison", 20, 40);
        m.resist("Energy", 20, 35);
        m.skill("Wrestling", 30.1, 40.0);
        m.skill("Tactics", 47.1, 57.0);
        m.skill("MagicResist", 30.1, 38.0);
        m.skill("Anatomy", 20.1, 24.0);
        m.fame = 700;
        m.karma = -700;
        if coin() {
            m.creature_item = block([#{ loot: "Reagent" }, #{ amount: 3 }]);
        } else {}
        m.creature_item = #{
            pick: [
                "Server.Items.LeatherGorget",
                "Server.Items.LeatherGloves",
                "Server.Items.LeatherArms",
                "Server.Items.LeatherLegs",
                "Server.Items.LeatherCap",
                "Server.Items.LeatherChest",
                (),
                (),
                (),
                (),
                (),
                (),
            ],
        };
    });
});

mobile("EliteNinja", |c| {
    c.name = "an elite ninja";
    c.bard_immune = true;
    c.always_murderer = true;
    c.loot = ["FilthyRich", "Rich", ["Gems", 2]];
    c.click_title = false;
    c.on_create(|m| {
        m.creature_speech_hue = block([#{ from: 2 }, #{ count: 1000 }]);
        m.creature_hue = #{ raceSkinHue: "Human" };
        if coin() {
            m.female = true;
            m.body = 401;
            m.hits = 251..=350;
            m.str = 126..=225;
            m.dex = 175..=275;
            m.int = 151..=165;
            m.damage = 12..=20;
            m.damage_type("Physical", 65);
            m.damage_type("Fire", 15);
            m.damage_type("Poison", 15);
            m.damage_type("Energy", 5);
            m.resist("Physical", 35, 65);
            m.resist("Fire", 40, 60);
            m.resist("Cold", 25, 45);
            m.resist("Poison", 40, 60);
            m.resist("Energy", 35, 55);
            m.skill("Anatomy", 105.0, 120.0);
            m.skill("MagicResist", 80.0, 100.0);
            m.skill("Tactics", 115.0, 130.0);
            m.skill("Wrestling", 95.0, 120.0);
            m.skill("Fencing", 95.0, 120.0);
            m.skill("Macing", 95.0, 120.0);
            m.skill("Swords", 95.0, 120.0);
            m.skill("Ninjitsu", 95.0, 120.0);
            m.fame = 8500;
            m.karma = -8500;
            m.pack = [
                #{ item: "Server.Items.NinjaTabi", equip: true },
                #{ item: "Server.Items.LeatherNinjaJacket", equip: true },
                #{ item: "Server.Items.LeatherNinjaHood", equip: true },
                #{ item: "Server.Items.LeatherNinjaPants", equip: true },
                #{ item: "Server.Items.LeatherNinjaMitts", equip: true },
                #{ chance: 0.33, item: "Server.Items.SmokeBomb", equip: true },
            ];
            m.creature_item = #{
                pick: [
                    "Server.Items.Tessen",
                    "Server.Items.Wakizashi",
                    "Server.Items.Nunchaku",
                    "Server.Items.Daisho",
                    "Server.Items.Sai",
                    "Server.Items.Tekagi",
                    "Server.Items.Kama",
                    "Server.Items.Katana",
                ],
                to: "add",
            };
            m.random_hair = true;
        } else {
            m.female = false;
            m.body = 400;
            m.hits = 251..=350;
            m.str = 126..=225;
            m.dex = 175..=275;
            m.int = 151..=165;
            m.damage = 12..=20;
            m.damage_type("Physical", 65);
            m.damage_type("Fire", 15);
            m.damage_type("Poison", 15);
            m.damage_type("Energy", 5);
            m.resist("Physical", 35, 65);
            m.resist("Fire", 40, 60);
            m.resist("Cold", 25, 45);
            m.resist("Poison", 40, 60);
            m.resist("Energy", 35, 55);
            m.skill("Anatomy", 105.0, 120.0);
            m.skill("MagicResist", 80.0, 100.0);
            m.skill("Tactics", 115.0, 130.0);
            m.skill("Wrestling", 95.0, 120.0);
            m.skill("Fencing", 95.0, 120.0);
            m.skill("Macing", 95.0, 120.0);
            m.skill("Swords", 95.0, 120.0);
            m.skill("Ninjitsu", 95.0, 120.0);
            m.fame = 8500;
            m.karma = -8500;
            m.pack = [
                #{ item: "Server.Items.NinjaTabi", equip: true },
                #{ item: "Server.Items.LeatherNinjaJacket", equip: true },
                #{ item: "Server.Items.LeatherNinjaHood", equip: true },
                #{ item: "Server.Items.LeatherNinjaPants", equip: true },
                #{ item: "Server.Items.LeatherNinjaMitts", equip: true },
                #{ chance: 0.33, item: "Server.Items.SmokeBomb", equip: true },
            ];
            m.creature_item = #{
                pick: [
                    "Server.Items.Tessen",
                    "Server.Items.Wakizashi",
                    "Server.Items.Nunchaku",
                    "Server.Items.Daisho",
                    "Server.Items.Sai",
                    "Server.Items.Tekagi",
                    "Server.Items.Kama",
                    "Server.Items.Katana",
                ],
                to: "add",
            };
            m.random_hair = true;
        }
    });
});

mobile("FanDancer", |c| {
    c.corpse_name = "a fan dancer corpse";
    c.name = "a fan dancer";
    c.uncalmable = true;
    c.loot = ["FilthyRich", "Rich", ["Gems", 2]];
    c.monster_abilities = ["ReflectPhysicalDamage", "FanningFire", "FanThrowCounter"];
    c.on_create(|m| {
        m.body = 247;
        m.sound = 0x372;
        m.str = 301..=375;
        m.dex = 201..=255;
        m.int = 21..=25;
        m.hits = 351..=430;
        m.damage = 12..=17;
        m.damage_type("Physical", 70);
        m.damage_type("Fire", 10);
        m.damage_type("Cold", 10);
        m.damage_type("Poison", 10);
        m.resist("Physical", 40, 60);
        m.resist("Fire", 50, 70);
        m.resist("Cold", 50, 70);
        m.resist("Poison", 50, 70);
        m.resist("Energy", 40, 60);
        m.skill("MagicResist", 100.1, 110.0);
        m.skill("Tactics", 85.1, 95.0);
        m.skill("Wrestling", 85.1, 95.0);
        m.skill("Anatomy", 85.1, 95.0);
        m.fame = 9000;
        m.karma = -9000;
        if chance(0.33) {
            m.creature_item = #{ loot: "BonsaiSeed" };
        }
        m.pack = [
            #{ item: "Server.Items.Tessen", equip: true },
            #{ chance: 0.02, item: "Server.Items.OrigamiPaper" },
        ];
    });
});

mobile("FireBeetle", |c| {
    c.mount_body = 169;
    c.mount_item_id = 16021;
    c.name = "a fire beetle";
    c.corpse_name = "a fire beetle corpse";
    c.subdue_before_tame = true;
    c.stat_loss_after_tame = true;
    c.reduce_speed_with_damage = false;
    c.meat = 16;
    c.favorite_food = ["Meat"];
    c.override_bonding_reqs = true;
    c.anger_sound = 541;
    c.idle_sound = 541;
    c.attack_sound = 354;
    c.hurt_sound = 355;
    c.death_sound = 541;
    c.on_create(|m| {
        m.stam = 100;
        m.str = 300;
        m.dex = 65..=100;
        m.int = 500;
        m.hits = 200;
        m.damage = 7..=20;
        m.damage_type("Physical", 0);
        m.damage_type("Fire", 100);
        m.resist("Physical", 40);
        m.resist("Fire", 70, 75);
        m.resist("Cold", 10);
        m.resist("Poison", 30);
        m.resist("Energy", 30);
        m.skill("MagicResist", 90.0);
        m.skill("Tactics", 100.0);
        m.skill("Wrestling", 100.0);
        m.fame = 4000;
        m.karma = -4000;
        m.tamable = true;
        m.control_slots = 3;
        m.min_tame_skill = 93.9;
        m.pack = [
            #{ item: "Server.Items.SulfurousAsh", amount: [16, 25] },
            #{ item: "Server.Items.IronIngot", amount: 2 },
        ];
        m.hue = 0x489;
    });
});

mobile("Kappa", |c| {
    c.corpse_name = "a kappa corpse";
    c.name = "a kappa";
    c.loot = ["Meager", "Average"];
    c.monster_abilities = ["DrainLifeAttack"];
    c.anger_sound = 1291;
    c.idle_sound = 1290;
    c.attack_sound = 1289;
    c.hurt_sound = 1292;
    c.death_sound = 1288;
    c.on_create(|m| {
        m.body = 240;
        m.str = 186..=230;
        m.dex = 51..=75;
        m.int = 41..=55;
        m.mana = 30;
        m.hits = 151..=180;
        m.damage = 6..=12;
        m.damage_type("Physical", 100);
        m.resist("Physical", 35, 50);
        m.resist("Fire", 35, 50);
        m.resist("Cold", 25, 50);
        m.resist("Poison", 35, 50);
        m.resist("Energy", 20, 30);
        m.skill("MagicResist", 60.1, 70.0);
        m.skill("Tactics", 79.1, 89.0);
        m.skill("Wrestling", 60.1, 70.0);
        m.fame = 1700;
        m.karma = -1700;
        m.pack = [#{ item: "Server.Items.RawFishSteak", amount: 3 }];
        m.creature_item = #{ pick: ["Server.Items.Gears", "Server.Items.Hinge", "Server.Items.Axle", (), (), ()], times: 2 };
        if era("ML") {
            switch random_below(3) {
                0 => {
                    m.creature_item = block([#{ loot: "PeculiarSeed" }, #{ group: 4 }]);
                },
                1 => {},
                2 => {}
            }
        }
    });
});

mobile("LadyOfTheSnow", |c| {
    c.ai = "Mage";
    c.corpse_name = "a lady of the snow corpse";
    c.name = "a lady of the snow";
    c.bleed_immune = true;
    c.can_rummage_corpses = true;
    c.treasure_map_level = 4;
    c.loot = ["FilthyRich", "Rich"];
    c.death_sound = 880;
    c.on_create(|m| {
        m.body = 252;
        m.sound = 0x482;
        m.str = 276..=305;
        m.dex = 106..=125;
        m.int = 471..=495;
        m.hits = 596..=625;
        m.damage = 13..=20;
        m.damage_type("Physical", 20);
        m.damage_type("Cold", 80);
        m.resist("Physical", 45, 55);
        m.resist("Fire", 40, 55);
        m.resist("Cold", 70, 90);
        m.resist("Poison", 60, 70);
        m.resist("Energy", 65, 85);
        m.skill("Magery", 95.1, 110.0);
        m.skill("MagicResist", 90.1, 105.0);
        m.skill("Tactics", 80.1, 100.0);
        m.skill("Wrestling", 80.1, 100.0);
        m.skill("Necromancy", 90.0, 110.0);
        m.skill("SpiritSpeak", 90.0, 110.0);
        m.fame = 15200;
        m.karma = -15200;
        m.pack = [#{ reg: 3 }, #{ item: "Server.Items.Necklace" }];
        if chance(0.25) {
            m.creature_item = #{ loot: "BonsaiSeed" };
        }
    });
});

mobile("Oni", |c| {
    c.ai = "Mage";
    c.corpse_name = "an oni corpse";
    c.name = "an oni";
    c.can_rummage_corpses = true;
    c.treasure_map_level = 4;
    c.loot = [["FilthyRich", 3]];
    c.anger_sound = 1251;
    c.idle_sound = 1250;
    c.attack_sound = 1249;
    c.hurt_sound = 1252;
    c.death_sound = 1248;
    c.on_create(|m| {
        m.body = 241;
        m.str = 801..=910;
        m.dex = 151..=300;
        m.int = 171..=195;
        m.hits = 401..=530;
        m.damage = 14..=20;
        m.damage_type("Physical", 70);
        m.damage_type("Fire", 10);
        m.damage_type("Energy", 20);
        m.resist("Physical", 65, 80);
        m.resist("Fire", 50, 70);
        m.resist("Cold", 35, 50);
        m.resist("Poison", 45, 70);
        m.resist("Energy", 45, 65);
        m.skill("EvalInt", 100.1, 125.0);
        m.skill("Magery", 96.1, 106.0);
        m.skill("Anatomy", 85.1, 95.0);
        m.skill("MagicResist", 85.1, 100.0);
        m.skill("Tactics", 86.1, 101.0);
        m.skill("Wrestling", 90.1, 100.0);
        m.fame = 12000;
        m.karma = -12000;
        if chance(0.33) {
            m.creature_item = #{ loot: "BonsaiSeed" };
        }
    });
});

mobile("RevenantLion", |c| {
    c.ai = "Mage";
    c.corpse_name = "a revenant lion corpse";
    c.name = "a Revenant Lion";
    c.bleed_immune = true;
    c.poison_immune = 2;
    c.hit_poison = 2;
    c.weapon_ability = "BleedAttack";
    c.loot = [["Rich", 2], ["MedScrolls", 2]];
    c.anger_sound = 1304;
    c.idle_sound = 1303;
    c.attack_sound = 1302;
    c.hurt_sound = 1305;
    c.death_sound = 1301;
    c.on_create(|m| {
        m.body = 251;
        m.str = 276..=325;
        m.dex = 156..=175;
        m.int = 76..=105;
        m.hits = 251..=280;
        m.damage = 18..=24;
        m.damage_type("Physical", 30);
        m.damage_type("Cold", 30);
        m.damage_type("Poison", 10);
        m.damage_type("Energy", 30);
        m.resist("Physical", 40, 60);
        m.resist("Fire", 20, 30);
        m.resist("Cold", 50, 60);
        m.resist("Poison", 55, 65);
        m.resist("Energy", 40, 50);
        m.skill("EvalInt", 80.1, 90.0);
        m.skill("Magery", 80.1, 90.0);
        m.skill("Poisoning", 120.1, 130.0);
        m.skill("MagicResist", 70.1, 90.0);
        m.skill("Tactics", 60.1, 80.0);
        m.skill("Wrestling", 80.1, 88.0);
        m.fame = 4000;
        m.karma = -4000;
        m.pack = [#{ necroReg: [6, 8] }];
        m.creature_item = #{
            pick: [
                "Server.Items.LeftArm",
                "Server.Items.RightArm",
                "Server.Items.Torso",
                "Server.Items.Bone",
                "Server.Items.RibCage",
                "Server.Items.RibCage",
                "Server.Items.BonePile",
                "Server.Items.BonePile",
                "Server.Items.BonePile",
                "Server.Items.BonePile",
            ],
        };
    });
});

mobile("Ronin", |c| {
    c.corpse_name = "a ronin corpse";
    c.name = "a ronin";
    c.always_murderer = true;
    c.bard_immune = true;
    c.can_rummage_corpses = true;
    c.loot = ["FilthyRich", "Rich", ["Gems", 2]];
    c.click_title = false;
    c.on_create(|m| {
        m.creature_speech_hue = block([#{ from: 2 }, #{ count: 1000 }]);
        m.creature_hue = #{ raceSkinHue: "Human" };
        if coin() {
            m.female = true;
            m.body = 401;
            m.str = 326..=375;
            m.dex = 31..=45;
            m.int = 101..=110;
            m.hits = 301..=400;
            m.mana = 101..=110;
            m.damage = 17..=25;
            m.damage_type("Physical", 90);
            m.damage_type("Poison", 10);
            m.resist("Physical", 55, 75);
            m.resist("Fire", 40, 60);
            m.resist("Cold", 35, 55);
            m.resist("Poison", 50, 70);
            m.resist("Energy", 55, 75);
            m.skill("MagicResist", 42.6, 57.5);
            m.skill("Tactics", 115.1, 130.0);
            m.skill("Wrestling", 92.6, 107.5);
            m.skill("Anatomy", 110.1, 125.0);
            m.skill("Fencing", 92.6, 107.5);
            m.skill("Macing", 92.6, 107.5);
            m.skill("Swords", 92.6, 107.5);
            m.fame = 8500;
            m.karma = -8500;
            m.pack = [
                #{ item: "Server.Items.SamuraiTabi", equip: true },
                #{ item: "Server.Items.LeatherHiroSode", equip: true },
                #{ item: "Server.Items.LeatherDo", equip: true },
            ];
            m.creature_item = [
                #{
                    pick: [
                        "Server.Items.LightPlateJingasa",
                        "Server.Items.ChainHatsuburi",
                        "Server.Items.DecorativePlateKabuto",
                        "Server.Items.LeatherJingasa",
                    ],
                    to: "add",
                },
                #{
                    pick: ["Server.Items.StuddedHaidate", "Server.Items.LeatherSuneate", "Server.Items.PlateSuneate"],
                    to: "add",
                },
            ];
            if chance(0.8) {
                m.pack = [#{ item: "Server.Items.NoDachi", equip: true }];
            } else {
                m.pack = [#{ item: "Server.Items.Halberd", equip: true }];
            }
            m.pack = [#{ item: "Server.Items.Wakizashi" }, #{ item: "Server.Items.Longsword" }];
            m.random_hair = true;
        } else {
            m.female = false;
            m.body = 400;
            m.str = 326..=375;
            m.dex = 31..=45;
            m.int = 101..=110;
            m.hits = 301..=400;
            m.mana = 101..=110;
            m.damage = 17..=25;
            m.damage_type("Physical", 90);
            m.damage_type("Poison", 10);
            m.resist("Physical", 55, 75);
            m.resist("Fire", 40, 60);
            m.resist("Cold", 35, 55);
            m.resist("Poison", 50, 70);
            m.resist("Energy", 55, 75);
            m.skill("MagicResist", 42.6, 57.5);
            m.skill("Tactics", 115.1, 130.0);
            m.skill("Wrestling", 92.6, 107.5);
            m.skill("Anatomy", 110.1, 125.0);
            m.skill("Fencing", 92.6, 107.5);
            m.skill("Macing", 92.6, 107.5);
            m.skill("Swords", 92.6, 107.5);
            m.fame = 8500;
            m.karma = -8500;
            m.pack = [
                #{ item: "Server.Items.SamuraiTabi", equip: true },
                #{ item: "Server.Items.LeatherHiroSode", equip: true },
                #{ item: "Server.Items.LeatherDo", equip: true },
            ];
            m.creature_item = [
                #{
                    pick: [
                        "Server.Items.LightPlateJingasa",
                        "Server.Items.ChainHatsuburi",
                        "Server.Items.DecorativePlateKabuto",
                        "Server.Items.LeatherJingasa",
                    ],
                    to: "add",
                },
                #{
                    pick: ["Server.Items.StuddedHaidate", "Server.Items.LeatherSuneate", "Server.Items.PlateSuneate"],
                    to: "add",
                },
            ];
            if chance(0.8) {
                m.pack = [#{ item: "Server.Items.NoDachi", equip: true }];
            } else {
                m.pack = [#{ item: "Server.Items.Halberd", equip: true }];
            }
            m.pack = [#{ item: "Server.Items.Wakizashi" }, #{ item: "Server.Items.Longsword" }];
            m.random_hair = true;
        }
    });
});

mobile("RuneBeetle", |c| {
    c.ai = "Mage";
    c.corpse_name = "a rune beetle corpse";
    c.name = "a rune beetle";
    c.poison_immune = 2;
    c.hit_poison = 2;
    c.favorite_food = ["FruitsAndVeggies", "GrainsAndHay"];
    c.can_anger_on_tame = true;
    c.weapon_ability = "BleedAttack";
    c.loot = [["FilthyRich", 2], "MedScrolls"];
    c.monster_abilities = ["RuneCorruption"];
    c.anger_sound = 1256;
    c.idle_sound = 1255;
    c.attack_sound = 1254;
    c.hurt_sound = 1257;
    c.death_sound = 1253;
    c.on_create(|m| {
        m.body = 244;
        m.str = 401..=460;
        m.dex = 121..=170;
        m.int = 376..=450;
        m.hits = 301..=360;
        m.damage = 15..=22;
        m.damage_type("Physical", 20);
        m.damage_type("Poison", 10);
        m.damage_type("Energy", 70);
        m.resist("Physical", 40, 65);
        m.resist("Fire", 35, 50);
        m.resist("Cold", 35, 50);
        m.resist("Poison", 75, 95);
        m.resist("Energy", 40, 60);
        m.skill("EvalInt", 100.1, 125.0);
        m.skill("Magery", 100.1, 110.0);
        m.skill("Poisoning", 120.1, 140.0);
        m.skill("MagicResist", 95.1, 110.0);
        m.skill("Tactics", 78.1, 93.0);
        m.skill("Wrestling", 70.1, 77.5);
        m.fame = 15000;
        m.karma = -15000;
        if chance(0.25) {
            m.creature_item = #{ loot: "BonsaiSeed" };
        }
        m.creature_item = #{
            pick: [
                "Server.Items.LeftArm",
                "Server.Items.RightArm",
                "Server.Items.Torso",
                "Server.Items.Bone",
                "Server.Items.RibCage",
                "Server.Items.RibCage",
                "Server.Items.BonePile",
                "Server.Items.BonePile",
                "Server.Items.BonePile",
                "Server.Items.BonePile",
            ],
        };
        m.tamable = true;
        m.control_slots = 3;
        m.min_tame_skill = 93.9;
    });
});

mobile("TsukiWolf", |c| {
    c.corpse_name = "a tsuki wolf corpse";
    c.name = "a tsuki wolf";
    c.meat = 4;
    c.hides = 25;
    c.favorite_food = ["Meat"];
    c.loot = ["Average", "Rich"];
    c.monster_abilities = ["BloodBathAttack"];
    c.anger_sound = 1325;
    c.idle_sound = 1324;
    c.attack_sound = 1323;
    c.hurt_sound = 1326;
    c.death_sound = 1322;
    c.on_create(|m| {
        m.body = 250;
        switch random_below(3) {
            0 => {
                m.creature_hue = block([#{ from: 1801 }, #{ count: 108 }]);
            },
            1 => {
                m.hue = 0;
            },
            2 => {
                m.hue = 0;
            }
        }
        m.str = 401..=450;
        m.dex = 151..=200;
        m.int = 66..=76;
        m.hits = 376..=450;
        m.mana = 40;
        m.damage = 14..=18;
        m.damage_type("Physical", 90);
        m.damage_type("Cold", 5);
        m.damage_type("Energy", 5);
        m.resist("Physical", 40, 60);
        m.resist("Fire", 50, 70);
        m.resist("Cold", 50, 70);
        m.resist("Poison", 50, 70);
        m.resist("Energy", 50, 70);
        m.skill("Anatomy", 65.1, 72.0);
        m.skill("MagicResist", 65.1, 70.0);
        m.skill("Tactics", 95.1, 110.0);
        m.skill("Wrestling", 97.6, 107.5);
        m.fame = 8500;
        m.karma = -8500;
        if era("ML") {
            if chance(0.33) {
                m.creature_item = block([#{ loot: "PeculiarSeed" }, #{ group: 1 }]);
            }
        }
        m.creature_item = #{
            pick: [
                "Server.Items.LeftArm",
                "Server.Items.RightArm",
                "Server.Items.Torso",
                "Server.Items.Bone",
                "Server.Items.RibCage",
                "Server.Items.RibCage",
                "Server.Items.BonePile",
                "Server.Items.BonePile",
                "Server.Items.BonePile",
                "Server.Items.BonePile",
            ],
        };
    });
});

mobile("Yamandon", |c| {
    c.corpse_name = "a yamandon corpse";
    c.name = "a yamandon";
    c.reacquire_on_movement = true;
    c.poison_immune = 4;
    c.treasure_map_level = 5;
    c.hides = 20;
    c.weapon_ability = "DoubleStrike";
    c.loot = ["UltraRich", ["FilthyRich", 2], ["Gems", 6]];
    c.hit_poison = #{ coin: [3, 4] };
    c.attack_sound = 1260;
    c.anger_sound = 1262;
    c.death_sound = 1259;
    c.hurt_sound = 1263;
    c.idle_sound = 1261;
    c.on_create(|m| {
        m.body = 249;
        m.str = 786..=930;
        m.dex = 251..=365;
        m.int = 101..=115;
        m.hits = 1601..=1800;
        m.damage = 19..=35;
        m.damage_type("Physical", 70);
        m.damage_type("Poison", 20);
        m.damage_type("Energy", 10);
        m.resist("Physical", 65, 85);
        m.resist("Fire", 70, 90);
        m.resist("Cold", 50, 70);
        m.resist("Poison", 50, 70);
        m.resist("Energy", 50, 70);
        m.skill("Anatomy", 115.1, 130.0);
        m.skill("MagicResist", 117.6, 132.5);
        m.skill("Poisoning", 120.1, 140.0);
        m.skill("Tactics", 117.1, 132.0);
        m.skill("Wrestling", 112.6, 132.5);
        m.fame = 22000;
        m.karma = -22000;
        if chance(0.5) {
            m.creature_item = #{ loot: "BonsaiSeed" };
        }
        m.pack = [#{ item: "Server.Items.Eggs", amount: 2 }];
    });
});

mobile("YomotsuElder", |c| {
    c.corpse_name = "a wrinkly yomotsu corpse";
    c.name = "a yomotsu elder";
    c.favorite_food = ["Fish"];
    c.meat = 1;
    c.can_rummage_corpses = true;
    c.treasure_map_level = 5;
    c.weapon_ability = "DoubleStrike";
    c.loot = [["FilthyRich", 3], ["Gems", 2]];
    c.idle_sound = 1066;
    c.attack_sound = 1077;
    c.hurt_sound = 1078;
    c.death_sound = 1082;
    c.on_create(|m| {
        m.body = 255;
        m.sound = 0x452;
        m.str = 686..=830;
        m.dex = 251..=365;
        m.int = 17..=31;
        m.hits = 801..=900;
        m.damage = 19..=27;
        m.damage_type("Physical", 100);
        m.resist("Physical", 65, 85);
        m.resist("Fire", 30, 50);
        m.resist("Cold", 45, 65);
        m.resist("Poison", 35, 55);
        m.resist("Energy", 25, 50);
        m.skill("Anatomy", 115.1, 130.0);
        m.skill("MagicResist", 100.1, 120.0);
        m.skill("Tactics", 115.1, 130.0);
        m.skill("Wrestling", 110.1, 130.0);
        m.fame = 12000;
        m.karma = -12000;
        m.pack = [#{ item: "Server.Items.GreenGourd" }, #{ item: "Server.Items.ExecutionersAxe" }];
        m.creature_item = [
            #{ pick: ["Server.Items.LongPants", "Server.Items.ShortPants", ()] },
            #{
                pick: [
                    "Server.Items.Shoes",
                    "Server.Items.Sandals",
                    "Server.Items.Boots",
                    "Server.Items.ThighBoots",
                    (),
                    (),
                ],
            },
        ];
        if chance(0.25) {
            m.creature_item = #{ loot: "BonsaiSeed" };
        }
    });
});

mobile("YomotsuPriest", |c| {
    c.ai = "Mage";
    c.corpse_name = "a glowing yomotsu corpse";
    c.name = "a yomotsu priest";
    c.favorite_food = ["Fish"];
    c.meat = 1;
    c.can_rummage_corpses = true;
    c.weapon_ability = "DoubleStrike";
    c.loot = ["FilthyRich", "Rich", ["Gems", 4]];
    c.idle_sound = 1066;
    c.attack_sound = 1077;
    c.hurt_sound = 1078;
    c.death_sound = 1082;
    c.on_create(|m| {
        m.body = 253;
        m.sound = 0x452;
        m.str = 486..=530;
        m.dex = 101..=115;
        m.int = 601..=670;
        m.hits = 486..=530;
        m.damage = 8..=10;
        m.damage_type("Physical", 100);
        m.resist("Physical", 65, 85);
        m.resist("Fire", 30, 50);
        m.resist("Cold", 45, 65);
        m.resist("Poison", 35, 55);
        m.resist("Energy", 25, 50);
        m.skill("EvalInt", 92.6, 107.5);
        m.skill("Magery", 105.1, 115.0);
        m.skill("Meditation", 100.1, 110.0);
        m.skill("MagicResist", 112.6, 122.5);
        m.skill("Tactics", 55.1, 105.0);
        m.skill("Wrestling", 47.6, 57.5);
        m.fame = 9000;
        m.karma = -9000;
        m.pack = [#{ item: "Server.Items.GreenGourd" }, #{ item: "Server.Items.ExecutionersAxe" }];
        m.creature_item = [
            #{ pick: ["Server.Items.LongPants", "Server.Items.ShortPants", ()] },
            #{
                pick: [
                    "Server.Items.Shoes",
                    "Server.Items.Sandals",
                    "Server.Items.Boots",
                    "Server.Items.ThighBoots",
                    (),
                    (),
                ],
            },
        ];
        if chance(0.25) {
            m.creature_item = #{ loot: "BonsaiSeed" };
        }
    });
});

mobile("YomotsuWarrior", |c| {
    c.corpse_name = "a yomotsu corpse";
    c.name = "a yomotsu warrior";
    c.favorite_food = ["Fish"];
    c.meat = 1;
    c.can_rummage_corpses = true;
    c.treasure_map_level = 3;
    c.weapon_ability = "DoubleStrike";
    c.loot = [["Rich", 2], ["Gems", 2]];
    c.idle_sound = 1066;
    c.attack_sound = 1077;
    c.hurt_sound = 1078;
    c.death_sound = 1082;
    c.on_create(|m| {
        m.body = 245;
        m.sound = 0x452;
        m.str = 486..=530;
        m.dex = 151..=165;
        m.int = 17..=31;
        m.hits = 486..=530;
        m.mana = 17..=31;
        m.damage = 8..=10;
        m.damage_type("Physical", 100);
        m.resist("Physical", 65, 85);
        m.resist("Fire", 30, 50);
        m.resist("Cold", 45, 65);
        m.resist("Poison", 35, 55);
        m.resist("Energy", 25, 50);
        m.skill("Anatomy", 85.1, 95.0);
        m.skill("MagicResist", 82.6, 90.5);
        m.skill("Tactics", 95.1, 105.0);
        m.skill("Wrestling", 97.6, 107.5);
        m.fame = 4200;
        m.karma = -4200;
        m.pack = [#{ item: "Server.Items.GreenGourd" }, #{ item: "Server.Items.ExecutionersAxe" }];
        if coin() {
            m.pack = [#{ item: "Server.Items.LongPants" }];
        } else {
            m.pack = [#{ item: "Server.Items.ShortPants" }];
        }
        m.creature_item = #{
            pick: ["Server.Items.Shoes", "Server.Items.Sandals", "Server.Items.Boots", "Server.Items.ThighBoots"],
        };
        if chance(0.25) {
            m.creature_item = #{ loot: "BonsaiSeed" };
        }
    });
});

mobile("SummonedAirElemental", |c| {
    c.ai = "Mage";
    c.corpse_name = "an air elemental corpse";
    c.name = "an air elemental";
    c.bleed_immune = true;
    c.dispel_difficulty = 117.5;
    c.dispel_focus = 45.0;
    c.on_create(|m| {
        m.body = 13;
        m.hue = 0x4001;
        m.sound = 0x28F;
        if era("AOS") {
            m.str = 200;
            m.dex = 200;
            m.int = 100;
            m.hits = 150;
            m.stam = 50;
            m.damage = 6..=9;
            m.damage_type("Physical", 50);
            m.damage_type("Energy", 50);
            m.resist("Physical", 40, 50);
            m.resist("Fire", 30, 40);
            m.resist("Cold", 35, 45);
            m.resist("Poison", 50, 60);
            m.resist("Energy", 70, 80);
            m.skill("Meditation", 90.0);
            m.skill("EvalInt", 70.0);
            m.skill("Magery", 70.0);
            m.skill("MagicResist", 60.0);
            m.skill("Tactics", 100.0);
            m.skill("Wrestling", 80.0);
        } else {
            m.str = 126..=155;
            m.dex = 166..=185;
            m.int = 101..=125;
            m.hits = 76..=93;
            m.damage = 8..=10;
            m.damage_type("Physical", 20);
            m.damage_type("Cold", 40);
            m.damage_type("Energy", 40);
            m.resist("Physical", 35, 45);
            m.resist("Fire", 15, 25);
            m.resist("Cold", 10, 20);
            m.resist("Poison", 10, 20);
            m.resist("Energy", 25, 35);
            m.skill("EvalInt", 60.1, 75.0);
            m.skill("Magery", 60.1, 75.0);
            m.skill("MagicResist", 60.1, 75.0);
            m.skill("Tactics", 60.1, 80.0);
            m.skill("Wrestling", 60.1, 80.0);
        }
        m.armor = 40;
        m.control_slots = 2;
    });
});

mobile("SummonedDaemon", |c| {
    c.ai = "Mage";
    c.corpse_name = "a daemon corpse";
    c.dispel_difficulty = 125.0;
    c.dispel_focus = 45.0;
    c.poison_immune = 1;
    c.can_fly = true;
    c.on_create(|m| {
        m.name = random_name("daemon");
        if era("AOS") {
            m.body = 10;
        } else {
            m.body = 9;
        }
        m.sound = 0x165;
        switch random_below(1) {
            0 => {
                if era("AOS") {
                    m.str = 200;
                    m.dex = 110;
                    m.int = 150;
                    m.damage = 14..=21;
                    m.damage_type("Physical", 0);
                    m.damage_type("Poison", 100);
                    m.resist("Physical", 45, 55);
                    m.resist("Fire", 50, 60);
                    m.resist("Cold", 20, 30);
                    m.resist("Poison", 70, 80);
                    m.resist("Energy", 40, 50);
                    m.skill("EvalInt", 90.1, 100.0);
                    m.skill("Meditation", 90.1, 100.0);
                    m.skill("Magery", 90.1, 100.0);
                    m.skill("MagicResist", 90.1, 100.0);
                    m.skill("Tactics", 100.0);
                    m.skill("Wrestling", 98.1, 99.0);
                } else {
                    m.str = 476..=505;
                    m.dex = 76..=95;
                    m.int = 301..=325;
                    m.hits = 286..=303;
                    m.damage = 7..=14;
                    m.damage_type("Physical", 100);
                    m.resist("Physical", 45, 60);
                    m.resist("Fire", 50, 60);
                    m.resist("Cold", 30, 40);
                    m.resist("Poison", 20, 30);
                    m.resist("Energy", 30, 40);
                    m.skill("EvalInt", 70.1, 80.0);
                    m.skill("Magery", 70.1, 80.0);
                    m.skill("MagicResist", 85.1, 95.0);
                    m.skill("Tactics", 70.1, 80.0);
                    m.skill("Wrestling", 60.1, 80.0);
                }
                m.armor = 58;
                switch random_below(1) {
                    0 => {
                        if era("SE") {
                            m.control_slots = 4;
                        } else {
                            m.control_slots = 5;
                        }
                    }
                }
            }
        }
    });
});

mobile("SummonedEarthElemental", |c| {
    c.corpse_name = "an earth elemental corpse";
    c.name = "an earth elemental";
    c.bleed_immune = true;
    c.dispel_difficulty = 117.5;
    c.dispel_focus = 45.0;
    c.on_create(|m| {
        m.body = 14;
        m.sound = 0x10C;
        if era("AOS") {
            m.str = 200;
            m.dex = 70;
            m.int = 70;
            m.hits = 180;
            m.damage = 14..=21;
            m.damage_type("Physical", 100);
            m.resist("Physical", 65, 75);
            m.resist("Fire", 40, 50);
            m.resist("Cold", 40, 50);
            m.resist("Poison", 40, 50);
            m.resist("Energy", 40, 50);
            m.skill("MagicResist", 65.0);
            m.skill("Tactics", 100.0);
            m.skill("Wrestling", 90.0);
        } else {
            m.str = 126..=155;
            m.dex = 66..=85;
            m.int = 71..=92;
            m.hits = 76..=93;
            m.damage = 9..=16;
            m.damage_type("Physical", 100);
            m.resist("Physical", 30, 35);
            m.resist("Fire", 10, 20);
            m.resist("Cold", 10, 20);
            m.resist("Poison", 15, 25);
            m.resist("Energy", 15, 25);
            m.skill("MagicResist", 50.1, 95.0);
            m.skill("Tactics", 60.1, 100.0);
            m.skill("Wrestling", 60.1, 100.0);
        }
        m.armor = 34;
        m.control_slots = 2;
    });
});

mobile("SummonedFireElemental", |c| {
    c.ai = "Mage";
    c.corpse_name = "a fire elemental corpse";
    c.name = "a fire elemental";
    c.bleed_immune = true;
    c.dispel_difficulty = 117.5;
    c.dispel_focus = 45.0;
    c.on_create(|m| {
        m.body = 15;
        m.sound = 0x346;
        if era("AOS") {
            m.str = 200;
            m.dex = 200;
            m.int = 100;
            m.damage = 9..=14;
            m.damage_type("Physical", 0);
            m.damage_type("Fire", 100);
            m.resist("Physical", 50, 60);
            m.resist("Fire", 70, 80);
            m.resist("Cold", 0, 10);
            m.resist("Poison", 50, 60);
            m.resist("Energy", 50, 60);
            m.skill("EvalInt", 90.0);
            m.skill("Magery", 90.0);
            m.skill("MagicResist", 85.0);
            m.skill("Tactics", 100.0);
            m.skill("Wrestling", 92.0);
        } else {
            m.str = 126..=155;
            m.dex = 166..=185;
            m.int = 101..=125;
            m.hits = 76..=93;
            m.damage = 7..=9;
            m.damage_type("Physical", 25);
            m.damage_type("Fire", 75);
            m.resist("Physical", 35, 45);
            m.resist("Fire", 60, 80);
            m.resist("Cold", 5, 10);
            m.resist("Poison", 30, 40);
            m.resist("Energy", 30, 40);
            m.skill("EvalInt", 60.1, 75.0);
            m.skill("Magery", 60.1, 75.0);
            m.skill("MagicResist", 75.2, 105.0);
            m.skill("Tactics", 80.1, 100.0);
            m.skill("Wrestling", 70.1, 100.0);
        }
        m.armor = 40;
        m.control_slots = 4;
        m.pack = [#{ item: "Server.Items.LightSource", equip: true }];
    });
});

mobile("SummonedWaterElemental", |c| {
    c.ai = "Mage";
    c.corpse_name = "a water elemental corpse";
    c.name = "a water elemental";
    c.bleed_immune = true;
    c.dispel_difficulty = 117.5;
    c.dispel_focus = 45.0;
    c.on_create(|m| {
        m.body = 16;
        m.sound = 0x116;
        if era("AOS") {
            m.str = 200;
            m.dex = 70;
            m.int = 100;
            m.hits = 165;
            m.damage = 12..=16;
            m.damage_type("Physical", 0);
            m.damage_type("Cold", 100);
            m.resist("Physical", 50, 60);
            m.resist("Fire", 20, 30);
            m.resist("Cold", 70, 80);
            m.resist("Poison", 45, 55);
            m.resist("Energy", 40, 50);
            m.skill("Meditation", 90.0);
            m.skill("EvalInt", 80.0);
            m.skill("Magery", 80.0);
            m.skill("MagicResist", 75.0);
            m.skill("Tactics", 100.0);
            m.skill("Wrestling", 85.0);
        } else {
            m.str = 126..=155;
            m.dex = 66..=85;
            m.int = 101..=125;
            m.hits = 76..=93;
            m.damage = 7..=9;
            m.damage_type("Physical", 100);
            m.resist("Physical", 35, 45);
            m.resist("Fire", 10, 25);
            m.resist("Cold", 10, 25);
            m.resist("Poison", 60, 70);
            m.resist("Energy", 5, 10);
            m.skill("EvalInt", 60.1, 75.0);
            m.skill("Magery", 60.1, 75.0);
            m.skill("MagicResist", 100.1, 115.0);
            m.skill("Tactics", 50.1, 70.0);
            m.skill("Wrestling", 50.1, 70.0);
        }
        m.armor = 40;
        m.control_slots = 3;
        m.can_swim = true;
    });
});
