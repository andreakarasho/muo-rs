mobile("SilverSerpent", |c| {
    c.corpse_name = "a silver serpent corpse";
    c.name = "a silver serpent";
    c.meat = 1;
    c.poison_immune = 4;
    c.loot = ["Average", ["Gems", 2]];
    c.death_adder_charmable = true;
    c.hit_poison = 4;
    c.on_create(|m| {
        m.body = 92;
        m.sound = 0xDB;
        m.str = 161..=360;
        m.dex = 151..=300;
        m.int = 21..=40;
        m.hits = 97..=216;
        m.damage = 5..=21;
        m.damage_type("Physical", 50);
        m.damage_type("Poison", 50);
        m.resist("Physical", 35, 45);
        m.resist("Fire", 5, 10);
        m.resist("Cold", 5, 10);
        m.resist("Poison", 100);
        m.resist("Energy", 5, 10);
        m.skill("Poisoning", 90.1, 100.0);
        m.skill("MagicResist", 95.1, 100.0);
        m.skill("Tactics", 80.1, 95.0);
        m.skill("Wrestling", 85.1, 100.0);
        m.fame = 7000;
        m.karma = -7000;
        m.armor = 40;
    });
});

mobile("Alligator", |c| {
    c.corpse_name = "an alligator corpse";
    c.name = "an alligator";
    c.meat = 1;
    c.hides = 12;
    c.hide_type = "Spined";
    c.favorite_food = ["Meat", "Fish"];
    c.on_create(|m| {
        m.body = 202;
        m.sound = 0x294;
        m.str = 76..=100;
        m.dex = 6..=25;
        m.int = 11..=20;
        m.hits = 46..=60;
        m.stam = 46..=65;
        m.mana = 0;
        m.damage = 5..=15;
        m.damage_type("Physical", 100);
        m.resist("Physical", 25, 35);
        m.resist("Fire", 5, 10);
        m.resist("Poison", 5, 10);
        m.skill("MagicResist", 25.1, 40.0);
        m.skill("Tactics", 40.1, 60.0);
        m.skill("Wrestling", 40.1, 60.0);
        m.fame = 600;
        m.karma = -600;
        m.armor = 30;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = 47.1;
    });
});

mobile("Phoenix", |c| {
    c.ai = "Mage";
    c.fight_mode = "Aggressor";
    c.corpse_name = "a phoenix corpse";
    c.name = "a phoenix";
    c.meat = 1;
    c.feathers = 36;
    c.can_fly = true;
    c.loot = ["FilthyRich", "Rich"];
    c.meat_type = "Bird";
    c.on_create(|m| {
        m.body = 5;
        m.hue = 0x674;
        m.sound = 0x8F;
        m.str = 504..=700;
        m.dex = 202..=300;
        m.int = 504..=700;
        m.hits = 340..=383;
        m.damage = 25..=25;
        m.damage_type("Physical", 50);
        m.damage_type("Fire", 50);
        m.resist("Physical", 45, 55);
        m.resist("Fire", 60, 70);
        m.resist("Poison", 25, 35);
        m.resist("Energy", 40, 50);
        m.skill("EvalInt", 90.2, 100.0);
        m.skill("Magery", 90.2, 100.0);
        m.skill("Meditation", 75.1, 100.0);
        m.skill("MagicResist", 86.0, 135.0);
        m.skill("Tactics", 80.1, 90.0);
        m.skill("Wrestling", 90.1, 100.0);
        m.fame = 15000;
        m.karma = 0;
        m.armor = 60;
    });
});

mobile("BlackBear", |c| {
    c.ai = "Animal";
    c.fight_mode = "Aggressor";
    c.corpse_name = "a bear corpse";
    c.name = "a black bear";
    c.meat = 1;
    c.hides = 12;
    c.favorite_food = ["Fish", "Meat", "FruitsAndVeggies"];
    c.pack_instinct = ["Bear"];
    c.on_create(|m| {
        m.body = 211;
        m.sound = 0xA3;
        m.str = 76..=100;
        m.dex = 56..=75;
        m.int = 11..=14;
        m.hits = 46..=60;
        m.mana = 0;
        m.damage = 4..=10;
        m.damage_type("Physical", 100);
        m.resist("Physical", 20, 25);
        m.resist("Cold", 10, 15);
        m.resist("Poison", 5, 10);
        m.skill("MagicResist", 20.1, 40.0);
        m.skill("Tactics", 40.1, 60.0);
        m.skill("Wrestling", 40.1, 60.0);
        m.fame = 450;
        m.karma = 0;
        m.armor = 24;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = 35.1;
    });
});

mobile("DireWolf", |c| {
    c.corpse_name = "a dire wolf corpse";
    c.name = "a dire wolf";
    c.meat = 1;
    c.hides = 7;
    c.hide_type = "Spined";
    c.favorite_food = ["Meat"];
    c.pack_instinct = ["Canine"];
    c.on_create(|m| {
        m.body = 23;
        m.sound = 0xE5;
        m.str = 96..=120;
        m.dex = 81..=105;
        m.int = 36..=60;
        m.hits = 58..=72;
        m.mana = 0;
        m.damage = 11..=17;
        m.damage_type("Physical", 100);
        m.resist("Physical", 20, 25);
        m.resist("Fire", 10, 20);
        m.resist("Cold", 5, 10);
        m.resist("Poison", 5, 10);
        m.resist("Energy", 10, 15);
        m.skill("MagicResist", 57.6, 75.0);
        m.skill("Tactics", 50.1, 70.0);
        m.skill("Wrestling", 60.1, 80.0);
        m.fame = 2500;
        m.karma = -2500;
        m.armor = 22;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = 83.1;
    });
});

mobile("Eagle", |c| {
    c.ai = "Animal";
    c.fight_mode = "Aggressor";
    c.corpse_name = "a bird corpse";
    c.name = "an eagle";
    c.meat = 1;
    c.feathers = 36;
    c.can_fly = true;
    c.meat_type = "Bird";
    c.favorite_food = ["Meat", "Fish"];
    c.on_create(|m| {
        m.body = 5;
        m.sound = 0x2EE;
        m.str = 31..=47;
        m.dex = 36..=60;
        m.int = 8..=20;
        m.hits = 20..=27;
        m.mana = 0;
        m.damage = 5..=10;
        m.damage_type("Physical", 100);
        m.resist("Physical", 20, 25);
        m.resist("Fire", 10, 15);
        m.resist("Cold", 20, 25);
        m.resist("Poison", 5, 10);
        m.resist("Energy", 5, 10);
        m.skill("MagicResist", 15.3, 30.0);
        m.skill("Tactics", 18.1, 37.0);
        m.skill("Wrestling", 20.1, 30.0);
        m.fame = 300;
        m.karma = 0;
        m.armor = 22;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = 17.1;
    });
});

mobile("BrownBear", |c| {
    c.ai = "Animal";
    c.fight_mode = "Aggressor";
    c.corpse_name = "a bear corpse";
    c.name = "a brown bear";
    c.meat = 1;
    c.hides = 12;
    c.favorite_food = ["Fish", "FruitsAndVeggies", "Meat"];
    c.pack_instinct = ["Bear"];
    c.on_create(|m| {
        m.body = 167;
        m.sound = 0xA3;
        m.str = 76..=100;
        m.dex = 26..=45;
        m.int = 23..=47;
        m.hits = 46..=60;
        m.mana = 0;
        m.damage = 6..=12;
        m.damage_type("Physical", 100);
        m.resist("Physical", 20, 30);
        m.resist("Cold", 15, 20);
        m.resist("Poison", 10, 15);
        m.skill("MagicResist", 25.1, 35.0);
        m.skill("Tactics", 40.1, 60.0);
        m.skill("Wrestling", 40.1, 60.0);
        m.fame = 450;
        m.karma = 0;
        m.armor = 24;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = 41.1;
    });
});

mobile("GrizzlyBear", |c| {
    c.ai = "Animal";
    c.fight_mode = "Aggressor";
    c.corpse_name = "a grizzly bear corpse";
    c.name = "a grizzly bear";
    c.meat = 2;
    c.hides = 16;
    c.favorite_food = ["Fish", "FruitsAndVeggies", "Meat"];
    c.pack_instinct = ["Bear"];
    c.on_create(|m| {
        m.body = 212;
        m.sound = 0xA3;
        m.str = 126..=155;
        m.dex = 81..=105;
        m.int = 16..=40;
        m.hits = 76..=93;
        m.mana = 0;
        m.damage = 8..=13;
        m.damage_type("Physical", 100);
        m.resist("Physical", 25, 35);
        m.resist("Cold", 15, 25);
        m.resist("Poison", 5, 10);
        m.resist("Energy", 5, 10);
        m.skill("MagicResist", 25.1, 40.0);
        m.skill("Tactics", 70.1, 100.0);
        m.skill("Wrestling", 45.1, 70.0);
        m.fame = 1000;
        m.karma = 0;
        m.armor = 24;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = 59.1;
    });
});

mobile("PolarBear", |c| {
    c.ai = "Animal";
    c.fight_mode = "Aggressor";
    c.corpse_name = "a polar bear corpse";
    c.name = "a polar bear";
    c.meat = 2;
    c.hides = 16;
    c.favorite_food = ["Fish", "FruitsAndVeggies", "Meat"];
    c.pack_instinct = ["Bear"];
    c.on_create(|m| {
        m.body = 213;
        m.sound = 0xA3;
        m.str = 116..=140;
        m.dex = 81..=105;
        m.int = 26..=50;
        m.hits = 70..=84;
        m.mana = 0;
        m.damage = 7..=12;
        m.damage_type("Physical", 100);
        m.resist("Physical", 25, 35);
        m.resist("Cold", 60, 80);
        m.resist("Poison", 15, 25);
        m.resist("Energy", 10, 15);
        m.skill("MagicResist", 45.1, 60.0);
        m.skill("Tactics", 60.1, 90.0);
        m.skill("Wrestling", 45.1, 70.0);
        m.fame = 1500;
        m.karma = 0;
        m.armor = 18;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = 35.1;
    });
});

mobile("Chicken", |c| {
    c.ai = "Animal";
    c.fight_mode = "Aggressor";
    c.corpse_name = "a chicken corpse";
    c.name = "a chicken";
    c.meat = 1;
    c.can_fly = true;
    c.feathers = 25;
    c.meat_type = "Bird";
    c.favorite_food = ["GrainsAndHay"];
    c.on_create(|m| {
        m.body = 208;
        m.sound = 0x6E;
        m.str = 5;
        m.dex = 15;
        m.int = 5;
        m.hits = 3;
        m.mana = 0;
        m.damage = 1..=1;
        m.damage_type("Physical", 100);
        m.resist("Physical", 1, 5);
        m.skill("MagicResist", 4.0);
        m.skill("Tactics", 5.0);
        m.skill("Wrestling", 5.0);
        m.fame = 150;
        m.karma = 0;
        m.armor = 2;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = -0.9;
    });
});

mobile("Crane", |c| {
    c.ai = "Animal";
    c.fight_mode = "Aggressor";
    c.corpse_name = "a bird corpse";
    c.name = "a crane";
    c.meat = 1;
    c.feathers = 25;
    c.anger_sound = 1241;
    c.idle_sound = 1240;
    c.attack_sound = 1239;
    c.hurt_sound = 1242;
    c.death_sound = 1238;
    c.on_create(|m| {
        m.body = 254;
        m.sound = 0x4D7;
        m.str = 26..=35;
        m.dex = 16..=25;
        m.int = 11..=15;
        m.hits = 26..=35;
        m.mana = 0;
        m.damage = 1..=1;
        m.damage_type("Physical", 100);
        m.resist("Physical", 5, 5);
        m.skill("MagicResist", 4.1, 5.0);
        m.skill("Tactics", 10.1, 11.0);
        m.skill("Wrestling", 10.1, 11.0);
        m.fame = 0;
        m.karma = 200;
        m.armor = 5;
    });
});

mobile("GreyWolf", |c| {
    c.ai = "Animal";
    c.fight_mode = "Aggressor";
    c.corpse_name = "a grey wolf corpse";
    c.name = "a grey wolf";
    c.meat = 1;
    c.hides = 6;
    c.favorite_food = ["Meat"];
    c.pack_instinct = ["Canine"];
    c.on_create(|m| {
        m.bodies = [25, 27];
        m.sound = 0xE5;
        m.str = 56..=80;
        m.dex = 56..=75;
        m.int = 31..=55;
        m.hits = 34..=48;
        m.mana = 0;
        m.damage = 3..=7;
        m.damage_type("Physical", 100);
        m.resist("Physical", 15, 20);
        m.resist("Fire", 10, 15);
        m.resist("Cold", 20, 25);
        m.resist("Poison", 10, 15);
        m.resist("Energy", 10, 15);
        m.skill("MagicResist", 20.1, 35.0);
        m.skill("Tactics", 45.1, 60.0);
        m.skill("Wrestling", 45.1, 60.0);
        m.fame = 450;
        m.karma = 0;
        m.armor = 16;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = 53.1;
    });
});

mobile("TimberWolf", |c| {
    c.ai = "Animal";
    c.fight_mode = "Aggressor";
    c.corpse_name = "a timber wolf corpse";
    c.name = "a timber wolf";
    c.meat = 1;
    c.hides = 5;
    c.favorite_food = ["Meat"];
    c.pack_instinct = ["Canine"];
    c.on_create(|m| {
        m.body = 225;
        m.sound = 0xE5;
        m.str = 56..=80;
        m.dex = 56..=75;
        m.int = 11..=25;
        m.hits = 34..=48;
        m.mana = 0;
        m.damage = 5..=9;
        m.damage_type("Physical", 100);
        m.resist("Physical", 15, 20);
        m.resist("Fire", 5, 10);
        m.resist("Cold", 10, 15);
        m.resist("Poison", 5, 10);
        m.resist("Energy", 5, 10);
        m.skill("MagicResist", 27.6, 45.0);
        m.skill("Tactics", 30.1, 50.0);
        m.skill("Wrestling", 40.1, 60.0);
        m.fame = 450;
        m.karma = 0;
        m.armor = 16;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = 23.1;
    });
});

mobile("WhiteWolf", |c| {
    c.ai = "Animal";
    c.fight_mode = "Aggressor";
    c.corpse_name = "a white wolf corpse";
    c.name = "a white wolf";
    c.meat = 1;
    c.hides = 6;
    c.favorite_food = ["Meat"];
    c.pack_instinct = ["Canine"];
    c.on_create(|m| {
        m.bodies = [34, 37];
        m.sound = 0xE5;
        m.str = 56..=80;
        m.dex = 56..=75;
        m.int = 31..=55;
        m.hits = 34..=48;
        m.mana = 0;
        m.damage = 3..=7;
        m.damage_type("Physical", 100);
        m.resist("Physical", 15, 20);
        m.resist("Fire", 10, 15);
        m.resist("Cold", 20, 25);
        m.resist("Poison", 10, 15);
        m.resist("Energy", 10, 15);
        m.skill("MagicResist", 20.1, 35.0);
        m.skill("Tactics", 45.1, 60.0);
        m.skill("Wrestling", 45.1, 60.0);
        m.fame = 450;
        m.karma = 0;
        m.armor = 16;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = 65.1;
    });
});

mobile("Cougar", |c| {
    c.ai = "Animal";
    c.fight_mode = "Aggressor";
    c.corpse_name = "a cougar corpse";
    c.name = "a cougar";
    c.meat = 1;
    c.hides = 10;
    c.favorite_food = ["Fish", "Meat"];
    c.pack_instinct = ["Feline"];
    c.on_create(|m| {
        m.body = 63;
        m.sound = 0x73;
        m.str = 56..=80;
        m.dex = 66..=85;
        m.int = 26..=50;
        m.hits = 34..=48;
        m.mana = 0;
        m.damage = 4..=10;
        m.damage_type("Physical", 100);
        m.resist("Physical", 20, 25);
        m.resist("Fire", 5, 10);
        m.resist("Cold", 10, 15);
        m.resist("Poison", 5, 10);
        m.skill("MagicResist", 15.1, 30.0);
        m.skill("Tactics", 45.1, 60.0);
        m.skill("Wrestling", 45.1, 60.0);
        m.fame = 450;
        m.karma = 0;
        m.armor = 16;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = 41.1;
    });
});

mobile("Panther", |c| {
    c.ai = "Animal";
    c.fight_mode = "Aggressor";
    c.corpse_name = "a panther corpse";
    c.name = "a panther";
    c.meat = 1;
    c.hides = 10;
    c.favorite_food = ["Meat", "Fish"];
    c.pack_instinct = ["Feline"];
    c.on_create(|m| {
        m.body = 214;
        m.hue = 0x901;
        m.sound = 0x462;
        m.str = 61..=85;
        m.dex = 86..=105;
        m.int = 26..=50;
        m.hits = 37..=51;
        m.mana = 0;
        m.damage = 4..=12;
        m.damage_type("Physical", 100);
        m.resist("Physical", 20, 25);
        m.resist("Fire", 5, 10);
        m.resist("Cold", 10, 15);
        m.resist("Poison", 5, 10);
        m.skill("MagicResist", 15.1, 30.0);
        m.skill("Tactics", 50.1, 65.0);
        m.skill("Wrestling", 50.1, 65.0);
        m.fame = 450;
        m.karma = 0;
        m.armor = 16;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = 53.1;
    });
});

mobile("PredatorHellCat", |c| {
    c.corpse_name = "a hell cat corpse";
    c.name = "a hell cat";
    c.hides = 10;
    c.loot = "Average";
    c.hide_type = "Spined";
    c.favorite_food = ["Meat"];
    c.pack_instinct = ["Feline"];
    c.monster_abilities = ["FireBreath"];
    c.on_create(|m| {
        m.body = 127;
        m.sound = 0xBA;
        m.str = 161..=185;
        m.dex = 96..=115;
        m.int = 76..=100;
        m.hits = 97..=131;
        m.damage = 5..=17;
        m.damage_type("Physical", 75);
        m.damage_type("Fire", 25);
        m.resist("Physical", 25, 35);
        m.resist("Fire", 30, 40);
        m.resist("Energy", 5, 15);
        m.skill("MagicResist", 75.1, 90.0);
        m.skill("Tactics", 50.1, 65.0);
        m.skill("Wrestling", 50.1, 65.0);
        m.fame = 2500;
        m.karma = -2500;
        m.armor = 30;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = 89.1;
    });
});

mobile("SnowLeopard", |c| {
    c.ai = "Animal";
    c.fight_mode = "Aggressor";
    c.corpse_name = "a leopard corpse";
    c.name = "a snow leopard";
    c.meat = 1;
    c.hides = 8;
    c.favorite_food = ["Meat", "Fish"];
    c.pack_instinct = ["Feline"];
    c.on_create(|m| {
        m.bodies = [64, 65];
        m.sound = 0x73;
        m.str = 56..=80;
        m.dex = 66..=85;
        m.int = 26..=50;
        m.hits = 34..=48;
        m.mana = 0;
        m.damage = 3..=9;
        m.damage_type("Physical", 100);
        m.resist("Physical", 20, 25);
        m.resist("Fire", 5, 10);
        m.resist("Cold", 30, 40);
        m.resist("Poison", 10, 20);
        m.resist("Energy", 20, 30);
        m.skill("MagicResist", 25.1, 35.0);
        m.skill("Tactics", 45.1, 60.0);
        m.skill("Wrestling", 40.1, 50.0);
        m.fame = 450;
        m.karma = 0;
        m.armor = 24;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = 53.1;
    });
});

mobile("Boar", |c| {
    c.ai = "Animal";
    c.fight_mode = "Aggressor";
    c.corpse_name = "a pig corpse";
    c.name = "a boar";
    c.meat = 2;
    c.favorite_food = ["FruitsAndVeggies", "GrainsAndHay"];
    c.on_create(|m| {
        m.body = 290;
        m.sound = 0xC4;
        m.str = 25;
        m.dex = 15;
        m.int = 5;
        m.hits = 15;
        m.mana = 0;
        m.damage = 3..=6;
        m.damage_type("Physical", 100);
        m.resist("Physical", 10, 15);
        m.resist("Fire", 5, 10);
        m.resist("Poison", 5, 10);
        m.skill("MagicResist", 9.0);
        m.skill("Tactics", 9.0);
        m.skill("Wrestling", 9.0);
        m.fame = 300;
        m.karma = 0;
        m.armor = 10;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = 29.1;
    });
});

mobile("Dolphin", |c| {
    c.ai = "Animal";
    c.fight_mode = "Aggressor";
    c.corpse_name = "a dolphin corpse";
    c.name = "a dolphin";
    c.meat = 1;
    c.on_create(|m| {
        m.body = 151;
        m.sound = 0x8A;
        m.str = 21..=49;
        m.dex = 66..=85;
        m.int = 96..=110;
        m.hits = 15..=27;
        m.damage = 3..=6;
        m.damage_type("Physical", 100);
        m.resist("Physical", 15, 20);
        m.resist("Fire", 70, 80);
        m.resist("Cold", 25, 30);
        m.resist("Poison", 10, 15);
        m.resist("Energy", 10, 15);
        m.skill("MagicResist", 15.1, 20.0);
        m.skill("Tactics", 19.2, 29.0);
        m.skill("Wrestling", 19.2, 29.0);
        m.fame = 500;
        m.karma = 2000;
        m.armor = 16;
        m.can_swim = true;
        m.cant_walk = true;
    });
});

mobile("Gaman", |c| {
    c.ai = "Animal";
    c.fight_mode = "Aggressor";
    c.corpse_name = "a gaman corpse";
    c.name = "a gaman";
    c.meat = 10;
    c.hides = 15;
    c.favorite_food = ["GrainsAndHay"];
    c.anger_sound = 1272;
    c.idle_sound = 1271;
    c.attack_sound = 1270;
    c.hurt_sound = 1273;
    c.death_sound = 1269;
    c.on_create(|m| {
        m.body = 248;
        m.str = 146..=175;
        m.dex = 111..=150;
        m.int = 46..=60;
        m.hits = 131..=160;
        m.mana = 0;
        m.damage = 6..=11;
        m.damage_type("Physical", 100);
        m.resist("Physical", 50, 70);
        m.resist("Fire", 30, 50);
        m.resist("Cold", 30, 50);
        m.resist("Poison", 40, 60);
        m.resist("Energy", 30, 50);
        m.skill("MagicResist", 37.6, 42.5);
        m.skill("Tactics", 70.6, 83.0);
        m.skill("Wrestling", 50.1, 57.5);
        m.fame = 2000;
        m.karma = -2000;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = 68.7;
    });
});

mobile("GiantToad", |c| {
    c.corpse_name = "a giant toad corpse";
    c.name = "a giant toad";
    c.hides = 12;
    c.loot = "Poor";
    c.hide_type = "Spined";
    c.favorite_food = ["Fish", "Meat"];
    c.on_create(|m| {
        m.body = 80;
        m.sound = 0x26B;
        m.str = 76..=100;
        m.dex = 6..=25;
        m.int = 11..=20;
        m.hits = 46..=60;
        m.mana = 0;
        m.damage = 5..=17;
        m.damage_type("Physical", 100);
        m.resist("Physical", 20, 25);
        m.resist("Fire", 5, 10);
        m.resist("Energy", 5, 10);
        m.skill("MagicResist", 25.1, 40.0);
        m.skill("Tactics", 40.1, 60.0);
        m.skill("Wrestling", 40.1, 60.0);
        m.fame = 750;
        m.karma = -750;
        m.armor = 24;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = 77.1;
    });
});

mobile("Goat", |c| {
    c.ai = "Animal";
    c.fight_mode = "Aggressor";
    c.corpse_name = "a goat corpse";
    c.name = "a goat";
    c.meat = 2;
    c.hides = 8;
    c.favorite_food = ["GrainsAndHay", "FruitsAndVeggies", "Leather"];
    c.on_create(|m| {
        m.body = 209;
        m.sound = 0x99;
        m.str = 19;
        m.dex = 15;
        m.int = 5;
        m.hits = 12;
        m.mana = 0;
        m.damage = 3..=4;
        m.damage_type("Physical", 100);
        m.resist("Physical", 5, 15);
        m.skill("MagicResist", 5.0);
        m.skill("Tactics", 5.0);
        m.skill("Wrestling", 5.0);
        m.fame = 150;
        m.karma = 0;
        m.armor = 10;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = 11.1;
    });
});

mobile("Gorilla", |c| {
    c.ai = "Animal";
    c.fight_mode = "Aggressor";
    c.corpse_name = "a gorilla corpse";
    c.name = "a gorilla";
    c.meat = 1;
    c.hides = 6;
    c.favorite_food = ["FruitsAndVeggies", "GrainsAndHay"];
    c.on_create(|m| {
        m.body = 29;
        m.sound = 0x9E;
        m.str = 53..=95;
        m.dex = 36..=55;
        m.int = 36..=60;
        m.hits = 38..=51;
        m.mana = 0;
        m.damage = 4..=10;
        m.damage_type("Physical", 100);
        m.resist("Physical", 20, 25);
        m.resist("Fire", 5, 10);
        m.resist("Cold", 10, 15);
        m.skill("MagicResist", 45.1, 60.0);
        m.skill("Tactics", 43.3, 58.0);
        m.skill("Wrestling", 43.3, 58.0);
        m.fame = 450;
        m.karma = 0;
        m.armor = 20;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = -18.9;
    });
});

mobile("GreatHart", |c| {
    c.ai = "Animal";
    c.fight_mode = "Aggressor";
    c.corpse_name = "a deer corpse";
    c.name = "a great hart";
    c.meat = 6;
    c.hides = 15;
    c.favorite_food = ["FruitsAndVeggies", "GrainsAndHay"];
    c.attack_sound = 130;
    c.hurt_sound = 131;
    c.death_sound = 132;
    c.on_create(|m| {
        m.body = 234;
        m.str = 41..=71;
        m.dex = 47..=77;
        m.int = 27..=57;
        m.hits = 27..=41;
        m.mana = 0;
        m.damage = 5..=9;
        m.damage_type("Physical", 100);
        m.resist("Physical", 20, 25);
        m.resist("Cold", 5, 10);
        m.skill("MagicResist", 26.8, 44.5);
        m.skill("Tactics", 29.8, 47.5);
        m.skill("Wrestling", 29.8, 47.5);
        m.fame = 300;
        m.karma = 0;
        m.armor = 24;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = 59.1;
    });
});

mobile("Hind", |c| {
    c.ai = "Animal";
    c.fight_mode = "Aggressor";
    c.corpse_name = "a deer corpse";
    c.name = "a hind";
    c.meat = 5;
    c.hides = 8;
    c.favorite_food = ["FruitsAndVeggies", "GrainsAndHay"];
    c.attack_sound = 130;
    c.hurt_sound = 131;
    c.death_sound = 132;
    c.on_create(|m| {
        m.body = 237;
        m.str = 21..=51;
        m.dex = 47..=77;
        m.int = 17..=47;
        m.hits = 15..=29;
        m.mana = 0;
        m.damage = 4..=4;
        m.damage_type("Physical", 100);
        m.resist("Physical", 5, 15);
        m.resist("Cold", 5);
        m.skill("MagicResist", 15.0);
        m.skill("Tactics", 19.0);
        m.skill("Wrestling", 26.0);
        m.fame = 300;
        m.karma = 0;
        m.armor = 8;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = 23.1;
    });
});

mobile("Llama", |c| {
    c.ai = "Animal";
    c.fight_mode = "Aggressor";
    c.corpse_name = "a llama corpse";
    c.name = "a llama";
    c.meat = 1;
    c.hides = 12;
    c.favorite_food = ["FruitsAndVeggies", "GrainsAndHay"];
    c.on_create(|m| {
        m.body = 220;
        m.sound = 0x3F3;
        m.str = 21..=49;
        m.dex = 36..=55;
        m.int = 16..=30;
        m.hits = 15..=27;
        m.mana = 0;
        m.damage = 3..=5;
        m.damage_type("Physical", 100);
        m.resist("Physical", 15, 20);
        m.skill("MagicResist", 15.1, 20.0);
        m.skill("Tactics", 19.2, 29.0);
        m.skill("Wrestling", 19.2, 29.0);
        m.fame = 300;
        m.karma = 0;
        m.armor = 16;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = 35.1;
    });
});

mobile("MountainGoat", |c| {
    c.ai = "Animal";
    c.fight_mode = "Aggressor";
    c.corpse_name = "a mountain goat corpse";
    c.name = "a mountain goat";
    c.meat = 2;
    c.hides = 12;
    c.favorite_food = ["GrainsAndHay", "FruitsAndVeggies"];
    c.on_create(|m| {
        m.body = 88;
        m.sound = 0x99;
        m.str = 22..=64;
        m.dex = 56..=75;
        m.int = 16..=30;
        m.hits = 20..=33;
        m.mana = 0;
        m.damage = 3..=7;
        m.damage_type("Physical", 100);
        m.resist("Physical", 10, 20);
        m.resist("Fire", 5, 10);
        m.resist("Cold", 10, 20);
        m.resist("Poison", 10, 15);
        m.resist("Energy", 10, 15);
        m.skill("MagicResist", 25.1, 30.0);
        m.skill("Tactics", 29.3, 44.0);
        m.skill("Wrestling", 29.3, 44.0);
        m.fame = 300;
        m.karma = 0;
        m.armor = 10;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = -0.9;
    });
});

mobile("Pig", |c| {
    c.ai = "Animal";
    c.fight_mode = "Aggressor";
    c.corpse_name = "a pig corpse";
    c.name = "a pig";
    c.meat = 1;
    c.favorite_food = ["FruitsAndVeggies", "GrainsAndHay"];
    c.on_create(|m| {
        m.body = 203;
        m.sound = 0xC4;
        m.str = 20;
        m.dex = 20;
        m.int = 5;
        m.hits = 12;
        m.mana = 0;
        m.damage = 2..=4;
        m.damage_type("Physical", 100);
        m.resist("Physical", 10, 15);
        m.skill("MagicResist", 5.0);
        m.skill("Tactics", 5.0);
        m.skill("Wrestling", 5.0);
        m.fame = 150;
        m.karma = 0;
        m.armor = 12;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = 11.1;
    });
});

mobile("Walrus", |c| {
    c.ai = "Animal";
    c.fight_mode = "Aggressor";
    c.corpse_name = "a walrus corpse";
    c.name = "a walrus";
    c.meat = 1;
    c.hides = 12;
    c.favorite_food = ["Fish"];
    c.on_create(|m| {
        m.body = 221;
        m.sound = 0xE0;
        m.str = 21..=29;
        m.dex = 46..=55;
        m.int = 16..=20;
        m.hits = 14..=17;
        m.mana = 0;
        m.damage = 4..=10;
        m.damage_type("Physical", 100);
        m.resist("Physical", 20, 25);
        m.resist("Fire", 5, 10);
        m.resist("Cold", 20, 25);
        m.resist("Poison", 5, 10);
        m.resist("Energy", 5, 10);
        m.skill("MagicResist", 15.1, 20.0);
        m.skill("Tactics", 19.2, 29.0);
        m.skill("Wrestling", 19.2, 29.0);
        m.fame = 150;
        m.karma = 0;
        m.armor = 18;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = 35.1;
    });
});

mobile("IceSnake", |c| {
    c.corpse_name = "an ice snake corpse";
    c.name = "an ice snake";
    c.meat = 1;
    c.loot = "Meager";
    c.death_adder_charmable = true;
    c.on_create(|m| {
        m.body = 52;
        m.hue = 0x480;
        m.sound = 0xDB;
        m.str = 42..=54;
        m.dex = 36..=45;
        m.int = 26..=30;
        m.mana = 0;
        m.damage = 4..=12;
        m.damage_type("Physical", 25);
        m.damage_type("Cold", 25);
        m.damage_type("Poison", 50);
        m.resist("Physical", 20, 25);
        m.resist("Cold", 80, 90);
        m.resist("Poison", 60, 70);
        m.resist("Energy", 30, 40);
        m.skill("MagicResist", 15.1, 20.0);
        m.skill("Tactics", 39.3, 54.0);
        m.skill("Wrestling", 39.3, 54.0);
        m.fame = 900;
        m.karma = -900;
        m.armor = 30;
    });
});

mobile("LavaSerpent", |c| {
    c.corpse_name = "a lava serpent corpse";
    c.name = "a lava serpent";
    c.meat = 4;
    c.hides = 15;
    c.loot = "Average";
    c.death_adder_charmable = true;
    c.hide_type = "Spined";
    c.monster_abilities = ["FireBreath"];
    c.on_create(|m| {
        m.body = 90;
        m.sound = 0xDB;
        m.str = 386..=415;
        m.dex = 56..=80;
        m.int = 66..=85;
        m.hits = 232..=249;
        m.mana = 0;
        m.damage = 10..=22;
        m.damage_type("Physical", 20);
        m.damage_type("Fire", 80);
        m.resist("Physical", 35, 45);
        m.resist("Fire", 70, 80);
        m.resist("Poison", 30, 40);
        m.resist("Energy", 10, 20);
        m.skill("MagicResist", 25.3, 70.0);
        m.skill("Tactics", 65.1, 70.0);
        m.skill("Wrestling", 60.1, 80.0);
        m.fame = 4500;
        m.karma = -4500;
        m.armor = 40;
        m.pack = [#{ item: "Server.Items.SulfurousAsh", amount: 3 }, #{ item: "Server.Items.Bone" }];
    });
});

mobile("GiantRat", |c| {
    c.corpse_name = "a giant rat corpse";
    c.name = "a giant rat";
    c.meat = 1;
    c.hides = 6;
    c.loot = "Poor";
    c.favorite_food = ["Fish", "Meat", "FruitsAndVeggies", "Eggs"];
    c.on_create(|m| {
        m.body = 215;
        m.sound = 0x188;
        m.str = 32..=74;
        m.dex = 46..=65;
        m.int = 16..=30;
        m.hits = 26..=39;
        m.mana = 0;
        m.damage = 4..=8;
        m.damage_type("Physical", 100);
        m.resist("Physical", 15, 20);
        m.resist("Fire", 5, 10);
        m.resist("Poison", 25, 35);
        m.skill("MagicResist", 25.1, 30.0);
        m.skill("Tactics", 29.3, 44.0);
        m.skill("Wrestling", 29.3, 44.0);
        m.fame = 300;
        m.karma = -300;
        m.armor = 18;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = 29.1;
    });
});

mobile("JackRabbit", |c| {
    c.ai = "Animal";
    c.fight_mode = "Aggressor";
    c.corpse_name = "a jack rabbit corpse";
    c.name = "a jack rabbit";
    c.meat = 1;
    c.hides = 1;
    c.favorite_food = ["FruitsAndVeggies"];
    c.attack_sound = 201;
    c.hurt_sound = 202;
    c.death_sound = 203;
    c.on_create(|m| {
        m.body = 205;
        m.hue = 0x1BB;
        m.str = 15;
        m.dex = 25;
        m.int = 5;
        m.hits = 9;
        m.mana = 0;
        m.damage = 1..=2;
        m.damage_type("Physical", 100);
        m.resist("Physical", 2, 5);
        m.skill("MagicResist", 5.0);
        m.skill("Tactics", 5.0);
        m.skill("Wrestling", 5.0);
        m.fame = 150;
        m.karma = 0;
        m.armor = 4;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = -18.9;
    });
});

mobile("SewerRat", |c| {
    c.corpse_name = "a rat corpse";
    c.name = "a sewer rat";
    c.meat = 1;
    c.loot = "Poor";
    c.favorite_food = ["Meat", "Eggs", "FruitsAndVeggies"];
    c.on_create(|m| {
        m.body = 238;
        m.sound = 0xCC;
        m.str = 9;
        m.dex = 25;
        m.int = 6..=10;
        m.hits = 6;
        m.mana = 0;
        m.damage = 1..=2;
        m.damage_type("Physical", 100);
        m.resist("Physical", 5, 10);
        m.resist("Poison", 15, 25);
        m.resist("Energy", 5, 10);
        m.skill("MagicResist", 5.0);
        m.skill("Tactics", 5.0);
        m.skill("Wrestling", 5.0);
        m.fame = 300;
        m.karma = -300;
        m.armor = 6;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = -0.9;
    });
});

mobile("Rat", |c| {
    c.ai = "Animal";
    c.fight_mode = "Aggressor";
    c.corpse_name = "a rat corpse";
    c.name = "a rat";
    c.meat = 1;
    c.loot = "Poor";
    c.favorite_food = ["Meat", "Fish", "Eggs", "GrainsAndHay"];
    c.on_create(|m| {
        m.body = 238;
        m.sound = 0xCC;
        m.str = 9;
        m.dex = 35;
        m.int = 5;
        m.hits = 6;
        m.mana = 0;
        m.damage = 1..=2;
        m.damage_type("Physical", 100);
        m.resist("Physical", 5, 10);
        m.resist("Poison", 5, 10);
        m.skill("MagicResist", 4.0);
        m.skill("Tactics", 4.0);
        m.skill("Wrestling", 4.0);
        m.fame = 150;
        m.karma = -150;
        m.armor = 6;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = -0.9;
    });
});

mobile("Server.Mobiles.BaseWarHorse", |c| {
    c.name = "a war horse";
    c.steps_max = 6400;
    c.corpse_name = "a war horse corpse";
    c.favorite_food = ["FruitsAndVeggies", "GrainsAndHay"];
    c.on_create(|m| {
        m.sound = 0xA8;
        m.creature_init_stats = [#{ from: 300, count: 100 }, 125, 60];
        m.str = 400;
        m.dex = 125;
        m.int = 51..=55;
        m.hits = 240;
        m.mana = 0;
        m.damage = 5..=8;
        m.damage_type("Physical", 100);
        m.resist("Physical", 40, 50);
        m.resist("Fire", 30, 40);
        m.resist("Cold", 30, 40);
        m.resist("Poison", 30, 40);
        m.resist("Energy", 30, 40);
        m.skill("MagicResist", 25.1, 30.0);
        m.skill("Tactics", 29.3, 44.0);
        m.skill("Wrestling", 29.3, 44.0);
        m.fame = 300;
        m.karma = 300;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = 29.1;
    });
});

mobile("SkeletalMount", |c| {
    c.ai = "Animal";
    c.fight_mode = "Aggressor";
    c.mount_body = 793;
    c.mount_item_id = 16059;
    c.steps_max = 4480;
    c.corpse_name = "an undead horse corpse";
    c.name = "a skeletal steed";
    c.poison_immune = 4;
    c.bleed_immune = true;
    c.on_create(|m| {
        m.str = 91..=100;
        m.dex = 46..=55;
        m.int = 46..=60;
        m.hits = 41..=50;
        m.damage = 5..=12;
        m.damage_type("Physical", 50);
        m.damage_type("Cold", 50);
        m.resist("Physical", 50, 60);
        m.resist("Cold", 90, 95);
        m.resist("Poison", 100);
        m.resist("Energy", 10, 15);
        m.skill("MagicResist", 95.1, 100.0);
        m.skill("Tactics", 50.0);
        m.skill("Wrestling", 70.1, 80.0);
        m.fame = 0;
        m.karma = 0;
    });
});

mobile("Bull", |c| {
    c.ai = "Animal";
    c.fight_mode = "Aggressor";
    c.corpse_name = "a bull corpse";
    c.name = "a bull";
    c.meat = 10;
    c.hides = 15;
    c.favorite_food = ["GrainsAndHay"];
    c.pack_instinct = ["Bull"];
    c.on_create(|m| {
        m.bodies = [232, 233];
        m.sound = 0x64;
        if coin() {
            m.hue = 0x901;
        } else {}
        m.str = 77..=111;
        m.dex = 56..=75;
        m.int = 47..=75;
        m.hits = 50..=64;
        m.mana = 0;
        m.damage = 4..=9;
        m.damage_type("Physical", 100);
        m.resist("Physical", 25, 30);
        m.resist("Cold", 10, 15);
        m.skill("MagicResist", 17.6, 25.0);
        m.skill("Tactics", 67.6, 85.0);
        m.skill("Wrestling", 40.1, 57.5);
        m.fame = 600;
        m.karma = 0;
        m.armor = 28;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = 71.1;
    });
});

mobile("Cow", |c| {
    c.ai = "Animal";
    c.fight_mode = "Aggressor";
    c.corpse_name = "a cow corpse";
    c.name = "a cow";
    c.meat = 8;
    c.hides = 12;
    c.favorite_food = ["FruitsAndVeggies", "GrainsAndHay"];
    c.on_create(|m| {
        m.creature_setup = "Cow";
        m.bodies = [216, 231];
        m.sound = 0x78;
        m.str = 30;
        m.dex = 15;
        m.int = 5;
        m.hits = 18;
        m.mana = 0;
        m.damage = 1..=4;
        m.damage = 1..=4;
        m.damage_type("Physical", 100);
        m.resist("Physical", 5, 15);
        m.skill("MagicResist", 5.5);
        m.skill("Tactics", 5.5);
        m.skill("Wrestling", 5.5);
        m.fame = 300;
        m.karma = 0;
        m.armor = 10;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = 11.1;
        if era("AOS") {
            m.creature_one_in = #{ n: 1000, then: #{ creatureFightMode: "Closest" } };
        }
    });
});

mobile("HellCat", |c| {
    c.corpse_name = "a hell cat corpse";
    c.name = "a hell cat";
    c.hides = 10;
    c.hide_type = "Spined";
    c.favorite_food = ["Meat"];
    c.pack_instinct = ["Feline"];
    c.loot = "Meager";
    c.monster_abilities = ["FireBreath"];
    c.on_create(|m| {
        m.body = 201;
        m.creature_hue = #{ pick: [1607, 1616, 1625, 1634, 1643, 1652] };
        m.sound = 0x69;
        m.str = 51..=100;
        m.dex = 52..=150;
        m.int = 13..=85;
        m.hits = 48..=67;
        m.damage = 6..=12;
        m.damage_type("Physical", 40);
        m.damage_type("Fire", 60);
        m.resist("Physical", 25, 35);
        m.resist("Fire", 80, 90);
        m.resist("Energy", 15, 20);
        m.skill("MagicResist", 45.1, 60.0);
        m.skill("Tactics", 40.1, 55.0);
        m.skill("Wrestling", 30.1, 40.0);
        m.fame = 1000;
        m.karma = -1000;
        m.armor = 30;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = 71.1;
    });
});

mobile("BullFrog", |c| {
    c.ai = "Animal";
    c.fight_mode = "Aggressor";
    c.corpse_name = "a bull frog corpse";
    c.name = "a bull frog";
    c.meat = 1;
    c.hides = 4;
    c.favorite_food = ["Fish", "Meat"];
    c.loot = "Poor";
    c.on_create(|m| {
        m.body = 81;
        m.creature_hue = #{ pick: [1452, 1443, 1434, 1425, 1416, 1407] };
        m.sound = 0x266;
        m.str = 46..=70;
        m.dex = 6..=25;
        m.int = 11..=20;
        m.hits = 28..=42;
        m.mana = 0;
        m.damage = 1..=2;
        m.damage_type("Physical", 100);
        m.resist("Physical", 5, 10);
        m.skill("MagicResist", 25.1, 40.0);
        m.skill("Tactics", 40.1, 60.0);
        m.skill("Wrestling", 40.1, 60.0);
        m.fame = 350;
        m.karma = 0;
        m.armor = 6;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = 23.1;
    });
});

mobile("PackHorse", |c| {
    c.ai = "Animal";
    c.fight_mode = "Aggressor";
    c.corpse_name = "a horse corpse";
    c.name = "a pack horse";
    c.meat = 3;
    c.hides = 10;
    c.favorite_food = ["FruitsAndVeggies", "GrainsAndHay"];
    c.on_create(|m| {
        m.body = 291;
        m.sound = 0xA8;
        m.str = 44..=120;
        m.dex = 36..=55;
        m.int = 6..=10;
        m.hits = 61..=80;
        m.stam = 81..=100;
        m.mana = 0;
        m.damage = 5..=11;
        m.damage_type("Physical", 100);
        m.resist("Physical", 20, 25);
        m.resist("Fire", 10, 15);
        m.resist("Cold", 20, 25);
        m.resist("Poison", 10, 15);
        m.resist("Energy", 10, 15);
        m.skill("MagicResist", 25.1, 30.0);
        m.skill("Tactics", 29.3, 44.0);
        m.skill("Wrestling", 29.3, 44.0);
        m.fame = 0;
        m.karma = 200;
        m.armor = 16;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = 11.1;
        m.creature_setup = "StrongBackpack";
    });
});

mobile("PackLlama", |c| {
    c.ai = "Animal";
    c.fight_mode = "Aggressor";
    c.corpse_name = "a llama corpse";
    c.name = "a pack llama";
    c.meat = 1;
    c.favorite_food = ["FruitsAndVeggies", "GrainsAndHay"];
    c.on_create(|m| {
        m.body = 292;
        m.sound = 0x3F3;
        m.str = 52..=80;
        m.dex = 36..=55;
        m.int = 16..=30;
        m.hits = 50;
        m.stam = 86..=105;
        m.mana = 0;
        m.damage = 2..=6;
        m.damage_type("Physical", 100);
        m.resist("Physical", 25, 35);
        m.resist("Fire", 10, 15);
        m.resist("Cold", 10, 15);
        m.resist("Poison", 10, 15);
        m.resist("Energy", 10, 15);
        m.skill("MagicResist", 15.1, 20.0);
        m.skill("Tactics", 19.2, 29.0);
        m.skill("Wrestling", 19.2, 29.0);
        m.fame = 0;
        m.karma = 200;
        m.armor = 16;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = 11.1;
        m.creature_setup = "StrongBackpack";
    });
});

mobile("Sheep", |c| {
    c.ai = "Animal";
    c.fight_mode = "Aggressor";
    c.corpse_name = "a sheep corpse";
    c.name = "a sheep";
    c.meat = 3;
    c.meat_type = "LambLeg";
    c.favorite_food = ["FruitsAndVeggies", "GrainsAndHay"];
    c.on_create(|m| {
        m.creature_setup = "Sheep";
        m.body = 207;
        m.sound = 0xD6;
        m.str = 19;
        m.dex = 25;
        m.int = 5;
        m.hits = 12;
        m.mana = 0;
        m.damage = 1..=2;
        m.damage_type("Physical", 100);
        m.resist("Physical", 5, 10);
        m.skill("MagicResist", 5.0);
        m.skill("Tactics", 6.0);
        m.skill("Wrestling", 5.0);
        m.fame = 300;
        m.karma = 0;
        m.armor = 6;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = 11.1;
    });
});

mobile("Beetle", |c| {
    c.mount_body = 791;
    c.mount_item_id = 16060;
    c.name = "a giant beetle";
    c.steps_max = 4480;
    c.corpse_name = "a giant beetle corpse";
    c.subdue_before_tame = true;
    c.reduce_speed_with_damage = false;
    c.favorite_food = ["Meat"];
    c.anger_sound = 541;
    c.idle_sound = 541;
    c.attack_sound = 354;
    c.hurt_sound = 355;
    c.death_sound = 541;
    c.on_create(|m| {
        m.str = 300;
        m.dex = 100;
        m.int = 500;
        m.hits = 200;
        m.damage = 7..=20;
        m.damage_type("Physical", 100);
        m.resist("Physical", 30, 40);
        m.resist("Fire", 20, 30);
        m.resist("Cold", 20, 30);
        m.resist("Poison", 20, 30);
        m.resist("Energy", 20, 30);
        m.skill("MagicResist", 80.0);
        m.skill("Tactics", 100.0);
        m.skill("Wrestling", 100.0);
        m.fame = 4000;
        m.karma = -4000;
        m.tamable = true;
        m.control_slots = 3;
        m.min_tame_skill = 29.1;
        m.creature_setup = "StrongBackpack";
    });
});

mobile("DesertOstard", |c| {
    c.ai = "Animal";
    c.fight_mode = "Aggressor";
    c.mount_body = 210;
    c.mount_item_id = 16035;
    c.name = "a desert ostard";
    c.corpse_name = "an ostard corpse";
    c.meat = 3;
    c.favorite_food = ["FruitsAndVeggies", "GrainsAndHay"];
    c.pack_instinct = ["Ostard"];
    c.on_create(|m| {
        m.sound = 0x270;
        m.str = 94..=170;
        m.dex = 56..=75;
        m.int = 6..=10;
        m.hits = 71..=88;
        m.mana = 0;
        m.damage = 5..=11;
        m.damage_type("Physical", 100);
        m.resist("Physical", 15, 20);
        m.resist("Fire", 5, 15);
        m.skill("MagicResist", 25.1, 30.0);
        m.skill("Tactics", 25.3, 40.0);
        m.skill("Wrestling", 29.3, 44.0);
        m.fame = 450;
        m.karma = 0;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = 29.1;
    });
});

mobile("FireSteed", |c| {
    c.mount_body = 190;
    c.mount_item_id = 16030;
    c.name = "a fire steed";
    c.corpse_name = "a fire steed corpse";
    c.favorite_food = ["Meat"];
    c.pack_instinct = ["Daemon", "Equine"];
    c.monster_abilities = ["FireBreath"];
    c.loot = [];
    c.on_create(|m| {
        m.sound = 0xA8;
        m.str = 376..=400;
        m.dex = 91..=120;
        m.int = 291..=300;
        m.hits = 226..=240;
        m.damage = 11..=30;
        m.damage_type("Physical", 20);
        m.damage_type("Fire", 80);
        m.resist("Physical", 30, 40);
        m.resist("Fire", 70, 80);
        m.resist("Cold", 20, 30);
        m.resist("Poison", 30, 40);
        m.resist("Energy", 30, 40);
        m.skill("MagicResist", 100.0, 120.0);
        m.skill("Tactics", 100.0);
        m.skill("Wrestling", 100.0);
        m.fame = 20000;
        m.karma = -20000;
        m.tamable = true;
        m.control_slots = 2;
        m.min_tame_skill = 106.0;
        m.pack = [
            #{ item: "Server.Items.SulfurousAsh", amount: [151, 300] },
            #{ item: "Server.Items.Ruby", amount: [16, 30] },
        ];
    });
});

mobile("ForestOstard", |c| {
    c.ai = "Animal";
    c.fight_mode = "Aggressor";
    c.mount_body = 219;
    c.mount_item_id = 16037;
    c.name = "a forest ostard";
    c.corpse_name = "an ostard corpse";
    c.meat = 3;
    c.favorite_food = ["FruitsAndVeggies", "GrainsAndHay"];
    c.pack_instinct = ["Ostard"];
    c.on_create(|m| {
        m.creature_hue = block([#{ from: 2201 }, #{ count: 24 }, #{ or: 32768 }]);
        m.sound = 0x270;
        m.str = 94..=170;
        m.dex = 56..=75;
        m.int = 6..=10;
        m.hits = 71..=88;
        m.mana = 0;
        m.damage = 8..=14;
        m.damage_type("Physical", 100);
        m.resist("Physical", 15, 20);
        m.skill("MagicResist", 27.1, 32.0);
        m.skill("Tactics", 29.3, 44.0);
        m.skill("Wrestling", 29.3, 44.0);
        m.fame = 450;
        m.karma = 0;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = 29.1;
    });
});

mobile("FrenziedOstard", |c| {
    c.mount_body = 218;
    c.mount_item_id = 16036;
    c.name = "a frenzied ostard";
    c.steps_max = 5120;
    c.corpse_name = "an ostard corpse";
    c.meat = 3;
    c.favorite_food = ["Meat", "Fish", "Eggs", "FruitsAndVeggies"];
    c.pack_instinct = ["Ostard"];
    c.on_create(|m| {
        m.creature_hue = block([#{ raceHairHue: "Human" }, #{ or: 32768 }]);
        m.sound = 0x275;
        m.str = 94..=170;
        m.dex = 96..=115;
        m.int = 6..=10;
        m.hits = 71..=110;
        m.mana = 0;
        m.damage = 11..=17;
        m.damage_type("Physical", 100);
        m.resist("Physical", 25, 30);
        m.resist("Fire", 10, 15);
        m.resist("Poison", 20, 25);
        m.resist("Energy", 20, 25);
        m.skill("MagicResist", 75.1, 80.0);
        m.skill("Tactics", 79.3, 94.0);
        m.skill("Wrestling", 79.3, 94.0);
        m.fame = 1500;
        m.karma = -1500;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = 77.1;
    });
});

mobile("HellSteed", |c| {
    c.ai = "Animal";
    c.fight_mode = "Aggressor";
    c.mount_body = 793;
    c.mount_item_id = 16059;
    c.name = "a hellsteed";
    c.steps_max = 5120;
    c.corpse_name = "a hellsteed corpse";
    c.poison_immune = 4;
    c.monster_abilities = ["ChaosBreath"];
    c.on_create(|m| {
        m.str = 201..=210;
        m.dex = 101..=110;
        m.int = 101..=115;
        m.hits = 201..=220;
        m.damage = 20..=24;
        m.damage_type("Physical", 25);
        m.damage_type("Fire", 75);
        m.resist("Physical", 60, 70);
        m.resist("Fire", 90);
        m.resist("Poison", 100);
        m.skill("MagicResist", 90.1, 110.0);
        m.skill("Tactics", 50.0);
        m.skill("Wrestling", 90.1, 110.0);
        m.fame = 0;
        m.karma = 0;
    });
});

mobile("GrizzledMare", |c| {
    c.inherits = "HellSteed";
    c.mount_body = 793;
    c.mount_item_id = 16059;
    c.name = "a grizzled mare";
    c.delete_on_release = true;
});

mobile("Hiryu", |c| {
    c.mount_body = 243;
    c.mount_item_id = 16020;
    c.name = "a hiryu";
    c.steps_max = 4480;
    c.corpse_name = "a hiryu corpse";
    c.weapon_ability_chance = 0.07;
    c.stat_loss_after_tame = true;
    c.treasure_map_level = 5;
    c.meat = 16;
    c.hides = 60;
    c.favorite_food = ["Meat"];
    c.can_anger_on_tame = true;
    c.weapon_ability = "Dismount";
    c.loot = [["FilthyRich", 3], ["Gems", 4]];
    c.monster_abilities = ["GraspingClaw"];
    c.anger_sound = 1278;
    c.idle_sound = 1277;
    c.attack_sound = 1276;
    c.hurt_sound = 1279;
    c.death_sound = 1275;
    c.on_create(|m| {
        m.creature_hue = block([
            #{ below: 1075 },
            #{ table: [
                [1074, 2140],
                [1073, 1168],
                [1071, 48],
                [1069, 55],
                [1066, 661],
                [1063, 291],
                [1058, 1154],
                [1050, 1159],
                [1040, #{ craftHue: "Gold" }],
                [1030, #{ craftHue: "Agapite" }],
                [1020, 1173],
                [1010, 1165],
                [1000, 1151],
                [0, 0],
            ] },
            #{ or: 32768 },
        ]);
        m.str = 1201..=1410;
        m.dex = 171..=270;
        m.int = 301..=325;
        m.hits = 901..=1100;
        m.mana = 60;
        m.damage = 20..=30;
        m.damage_type("Physical", 100);
        m.resist("Physical", 55, 70);
        m.resist("Fire", 70, 90);
        m.resist("Cold", 15, 25);
        m.resist("Poison", 40, 50);
        m.resist("Energy", 40, 50);
        m.skill("Anatomy", 75.1, 80.0);
        m.skill("MagicResist", 85.1, 100.0);
        m.skill("Tactics", 100.1, 110.0);
        m.skill("Wrestling", 100.1, 120.0);
        m.fame = 18000;
        m.karma = -18000;
        m.tamable = true;
        m.control_slots = 4;
        m.min_tame_skill = 98.7;
        if chance(0.33) {
            m.creature_item = #{ loot: "BonsaiSeed" };
        }
        if era("ML") {
            if chance(0.33) {
                m.creature_item = block([#{ loot: "PeculiarSeed" }, #{ group: 3 }]);
            }
        }
    });
});

mobile("Horse", |c| {
    c.ai = "Animal";
    c.fight_mode = "Aggressor";
    c.mount_body = 226;
    c.mount_item_id = 16032;
    c.name = "a horse";
    c.corpse_name = "a horse corpse";
    c.meat = 3;
    c.hides = 10;
    c.favorite_food = ["FruitsAndVeggies", "GrainsAndHay"];
    c.on_create(|m| {
        switch random_below(4) {
            0 => {
                m.body = 200;
                m.creature_mount_item_id = 16031;
            },
            1 => {
                m.body = 226;
                m.creature_mount_item_id = 16032;
            },
            2 => {
                m.body = 228;
                m.creature_mount_item_id = 16033;
            },
            3 => {
                m.body = 204;
                m.creature_mount_item_id = 16034;
            }
        }
        m.sound = 0xA8;
        m.str = 22..=98;
        m.dex = 56..=75;
        m.int = 6..=10;
        m.hits = 28..=45;
        m.mana = 0;
        m.damage = 3..=4;
        m.damage_type("Physical", 100);
        m.resist("Physical", 15, 20);
        m.skill("MagicResist", 25.1, 30.0);
        m.skill("Tactics", 29.3, 44.0);
        m.skill("Wrestling", 29.3, 44.0);
        m.fame = 300;
        m.karma = 300;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = 29.1;
    });
});

mobile("Kirin", |c| {
    c.ai = "Mage";
    c.fight_mode = "Evil";
    c.mount_body = 132;
    c.mount_item_id = 16045;
    c.name = "a ki-rin";
    c.steps_max = 4480;
    c.corpse_name = "a ki-rin corpse";
    c.allow_female_rider = false;
    c.allow_female_tamer = false;
    c.initial_innocent = true;
    c.mount_ability_delay = 3600.0;
    c.opposition = "FeyAndUndead";
    c.meat = 3;
    c.hides = 10;
    c.hide_type = "Horned";
    c.favorite_food = ["FruitsAndVeggies", "GrainsAndHay"];
    c.loot = ["Rich", "LowScrolls", "Potions"];
    c.on_create(|m| {
        m.sound = 0x3C5;
        m.str = 296..=325;
        m.dex = 86..=105;
        m.int = 186..=225;
        m.hits = 191..=210;
        m.damage = 16..=22;
        m.damage_type("Physical", 70);
        m.damage_type("Fire", 10);
        m.damage_type("Cold", 10);
        m.damage_type("Energy", 10);
        m.resist("Physical", 55, 65);
        m.resist("Fire", 35, 45);
        m.resist("Cold", 25, 35);
        m.resist("Poison", 25, 35);
        m.resist("Energy", 25, 35);
        m.skill("EvalInt", 80.1, 90.0);
        m.skill("Magery", 60.4, 100.0);
        m.skill("Meditation", 90.1, 100.0);
        m.skill("MagicResist", 85.3, 100.0);
        m.skill("Tactics", 20.1, 22.5);
        m.skill("Wrestling", 80.5, 92.5);
        m.fame = 9000;
        m.karma = 9000;
        m.tamable = true;
        m.control_slots = 2;
        m.min_tame_skill = 95.1;
    });
});

mobile("LesserHiryu", |c| {
    c.mount_body = 243;
    c.mount_item_id = 16020;
    c.name = "a lesser hiryu";
    c.steps_max = 4480;
    c.corpse_name = "a hiryu corpse";
    c.weapon_ability_chance = 0.07;
    c.stat_loss_after_tame = true;
    c.treasure_map_level = 3;
    c.meat = 16;
    c.hides = 60;
    c.favorite_food = ["Meat"];
    c.can_anger_on_tame = true;
    c.weapon_ability = "Dismount";
    c.loot = [["FilthyRich", 2], ["Gems", 4]];
    c.monster_abilities = ["GraspingClaw"];
    c.anger_sound = 1278;
    c.idle_sound = 1277;
    c.attack_sound = 1276;
    c.hurt_sound = 1279;
    c.death_sound = 1275;
    c.on_create(|m| {
        m.creature_hue = block([
            #{ below: 525 },
            #{ table: [[524, 600], [523, #{ craftHue: "Valorite" }], [520, 2004], [510, 355], [500, 661], [0, 0]] },
            #{ or: 32768 },
        ]);
        m.str = 301..=410;
        m.dex = 171..=270;
        m.int = 301..=325;
        m.hits = 401..=600;
        m.mana = 60;
        m.damage = 18..=23;
        m.damage_type("Physical", 100);
        m.resist("Physical", 45, 70);
        m.resist("Fire", 60, 80);
        m.resist("Cold", 5, 15);
        m.resist("Poison", 30, 40);
        m.resist("Energy", 30, 40);
        m.skill("Anatomy", 75.1, 80.0);
        m.skill("MagicResist", 85.1, 100.0);
        m.skill("Tactics", 100.1, 110.0);
        m.skill("Wrestling", 100.1, 120.0);
        m.fame = 10000;
        m.karma = -10000;
        m.tamable = true;
        m.control_slots = 3;
        m.min_tame_skill = 98.7;
        if chance(0.33) {
            m.creature_item = #{ loot: "BonsaiSeed" };
        }
    });
});

mobile("Nightmare", |c| {
    c.ai = "Mage";
    c.mount_body = 116;
    c.mount_item_id = 16039;
    c.name = "a nightmare";
    c.steps_max = 6400;
    c.corpse_name = "a nightmare corpse";
    c.meat = 5;
    c.hides = 10;
    c.hide_type = "Barbed";
    c.favorite_food = ["Meat"];
    c.can_anger_on_tame = true;
    c.loot = ["Rich", "Average", "LowScrolls", "Potions"];
    c.monster_abilities = ["FireBreath"];
    c.on_create(|m| {
        if era("AOS") {
            m.sound = 0xA8;
        } else {
            m.sound = 0x16A;
        }
        switch random_below(1) {
            0 => {
                if era("TOL") {
                    if chance(0.3) {
                        m.str = 296..=315;
                        m.control_slots = 2;
                    } else {
                        m.str = 496..=525;
                        m.control_slots = 3;
                    }
                } else {
                    m.str = 496..=525;
                    m.control_slots = 2;
                }
                m.dex = 86..=105;
                m.int = 86..=125;
                m.hits = 298..=315;
                m.damage = 16..=22;
                m.damage_type("Physical", 40);
                m.damage_type("Fire", 40);
                m.damage_type("Energy", 20);
                m.resist("Physical", 55, 65);
                m.resist("Fire", 30, 40);
                m.resist("Cold", 30, 40);
                m.resist("Poison", 30, 40);
                m.resist("Energy", 20, 30);
                m.skill("EvalInt", 10.4, 50.0);
                m.skill("Magery", 10.4, 50.0);
                m.skill("MagicResist", 85.3, 100.0);
                m.skill("Tactics", 97.6, 100.0);
                m.skill("Wrestling", 80.5, 92.5);
                m.fame = 14000;
                m.karma = -14000;
                m.armor = 60;
                m.tamable = true;
                m.min_tame_skill = 95.1;
                switch random_below(3) {
                    0 => {
                        m.body = 116;
                        m.creature_mount_item_id = 16039;
                    },
                    1 => {
                        m.body = 178;
                        m.creature_mount_item_id = 16041;
                    },
                    2 => {
                        m.body = 179;
                        m.creature_mount_item_id = 16055;
                    }
                }
                m.pack = [#{ item: "Server.Items.SulfurousAsh", amount: [3, 5] }];
            }
        }
    });
});

mobile("RidableLlama", |c| {
    c.ai = "Animal";
    c.fight_mode = "Aggressor";
    c.mount_body = 220;
    c.mount_item_id = 16038;
    c.name = "a ridable llama";
    c.steps_max = 2560;
    c.corpse_name = "a llama corpse";
    c.meat = 1;
    c.hides = 12;
    c.favorite_food = ["FruitsAndVeggies", "GrainsAndHay"];
    c.on_create(|m| {
        m.sound = 0x3F3;
        m.str = 21..=49;
        m.dex = 56..=75;
        m.int = 16..=30;
        m.hits = 15..=27;
        m.mana = 0;
        m.damage = 3..=5;
        m.damage_type("Physical", 100);
        m.resist("Physical", 10, 15);
        m.resist("Fire", 5, 10);
        m.resist("Cold", 5, 10);
        m.resist("Poison", 5, 10);
        m.resist("Energy", 5, 10);
        m.skill("MagicResist", 15.1, 20.0);
        m.skill("Tactics", 19.2, 29.0);
        m.skill("Wrestling", 19.2, 29.0);
        m.fame = 300;
        m.karma = 0;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = 29.1;
    });
});

mobile("Ridgeback", |c| {
    c.ai = "Animal";
    c.fight_mode = "Aggressor";
    c.mount_body = 187;
    c.mount_item_id = 16058;
    c.name = "a ridgeback";
    c.steps_max = 4480;
    c.corpse_name = "a ridgeback corpse";
    c.meat = 1;
    c.hides = 12;
    c.hide_type = "Spined";
    c.favorite_food = ["FruitsAndVeggies", "GrainsAndHay"];
    c.override_bonding_reqs = true;
    c.on_create(|m| {
        m.sound = 0x3F3;
        m.str = 58..=100;
        m.dex = 56..=75;
        m.int = 16..=30;
        m.hits = 41..=54;
        m.mana = 0;
        m.damage = 3..=5;
        m.damage_type("Physical", 100);
        m.resist("Physical", 15, 25);
        m.resist("Fire", 5, 10);
        m.resist("Cold", 5, 10);
        m.resist("Poison", 5, 10);
        m.resist("Energy", 5, 10);
        m.skill("MagicResist", 25.3, 40.0);
        m.skill("Tactics", 29.3, 44.0);
        m.skill("Wrestling", 35.1, 45.0);
        m.fame = 300;
        m.karma = 0;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = 83.1;
    });
});

mobile("SavageRidgeback", |c| {
    c.fight_mode = "Aggressor";
    c.mount_body = 188;
    c.mount_item_id = 16056;
    c.name = "a savage ridgeback";
    c.steps_max = 4480;
    c.corpse_name = "a savage ridgeback corpse";
    c.meat = 1;
    c.hides = 12;
    c.hide_type = "Spined";
    c.favorite_food = ["FruitsAndVeggies", "GrainsAndHay"];
    c.override_bonding_reqs = true;
    c.on_create(|m| {
        m.sound = 0x3F3;
        m.str = 58..=100;
        m.dex = 56..=75;
        m.int = 16..=30;
        m.hits = 41..=54;
        m.mana = 0;
        m.damage = 3..=5;
        m.damage_type("Physical", 100);
        m.resist("Physical", 15, 20);
        m.resist("Fire", 10, 15);
        m.resist("Cold", 15, 20);
        m.resist("Poison", 10, 15);
        m.resist("Energy", 10, 15);
        m.skill("MagicResist", 25.3, 40.0);
        m.skill("Tactics", 29.3, 44.0);
        m.skill("Wrestling", 35.1, 45.0);
        m.fame = 300;
        m.karma = 0;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = 83.1;
    });
});

mobile("ScaledSwampDragon", |c| {
    c.fight_mode = "Aggressor";
    c.mount_body = 799;
    c.mount_item_id = 16062;
    c.name = "a swamp dragon";
    c.steps_max = 4480;
    c.corpse_name = "a swamp dragon corpse";
    c.favorite_food = ["Meat"];
    c.override_bonding_reqs = true;
    c.on_create(|m| {
        m.str = 201..=300;
        m.dex = 66..=85;
        m.int = 61..=100;
        m.hits = 121..=180;
        m.damage = 3..=4;
        m.damage_type("Physical", 75);
        m.damage_type("Poison", 25);
        m.resist("Physical", 35, 40);
        m.resist("Fire", 20, 30);
        m.resist("Cold", 20, 40);
        m.resist("Poison", 20, 30);
        m.resist("Energy", 30, 40);
        m.skill("Anatomy", 45.1, 55.0);
        m.skill("MagicResist", 45.1, 55.0);
        m.skill("Tactics", 45.1, 55.0);
        m.skill("Wrestling", 45.1, 55.0);
        m.fame = 2000;
        m.karma = -2000;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = 93.9;
    });
});

mobile("SeaHorse", |c| {
    c.ai = "Animal";
    c.fight_mode = "Aggressor";
    c.mount_body = 144;
    c.mount_item_id = 16051;
    c.name = "a sea horse";
    c.corpse_name = "a sea horse corpse";
    c.on_create(|m| {
        m.creature_init_stats = [#{ from: 50, count: 30 }, #{ from: 50, count: 30 }, 10];
        m.creature_skill_base = block([
            #{ MagicResist: #{ base: 25.0, random: 5.0 } },
            #{ Wrestling: #{ base: 35.0, random: 10.0 } },
            #{ Tactics: #{ base: 30.0, random: 15.0 } },
        ]);
    });
});

mobile("SilverSteed", |c| {
    c.ai = "Animal";
    c.fight_mode = "Aggressor";
    c.mount_body = 117;
    c.mount_item_id = 16040;
    c.name = "a silver steed";
    c.steps_max = 4480;
    c.corpse_name = "a silver steed corpse";
    c.on_create(|m| {
        m.creature_init_stats = [#{ from: 50, count: 30 }, #{ from: 50, count: 30 }, 10];
        m.creature_skill_base = block([
            #{ MagicResist: #{ base: 25.0, random: 5.0 } },
            #{ Wrestling: #{ base: 35.0, random: 10.0 } },
            #{ Tactics: #{ base: 30.0, random: 15.0 } },
        ]);
        m.control_slots = 1;
        m.tamable = true;
        m.min_tame_skill = 103.1;
    });
});

mobile("Unicorn", |c| {
    c.ai = "Mage";
    c.fight_mode = "Evil";
    c.mount_body = 122;
    c.mount_item_id = 16052;
    c.name = "a unicorn";
    c.steps_max = 4480;
    c.corpse_name = "a unicorn corpse";
    c.allow_male_rider = false;
    c.allow_male_tamer = false;
    c.initial_innocent = true;
    c.mount_ability_delay = 3600.0;
    c.opposition = "FeyAndUndead";
    c.poison_immune = 4;
    c.meat = 3;
    c.hides = 10;
    c.hide_type = "Horned";
    c.favorite_food = ["FruitsAndVeggies", "GrainsAndHay"];
    c.loot = ["Rich", "LowScrolls", "Potions"];
    c.on_create(|m| {
        m.sound = 0x4BC;
        m.str = 296..=325;
        m.dex = 96..=115;
        m.int = 186..=225;
        m.hits = 191..=210;
        m.damage = 16..=22;
        m.damage_type("Physical", 75);
        m.damage_type("Energy", 25);
        m.resist("Physical", 55, 65);
        m.resist("Fire", 25, 40);
        m.resist("Cold", 25, 40);
        m.resist("Poison", 55, 65);
        m.resist("Energy", 25, 40);
        m.skill("EvalInt", 80.1, 90.0);
        m.skill("Magery", 60.2, 80.0);
        m.skill("Meditation", 50.1, 60.0);
        m.skill("MagicResist", 75.3, 90.0);
        m.skill("Tactics", 20.1, 22.5);
        m.skill("Wrestling", 80.5, 92.5);
        m.fame = 9000;
        m.karma = 9000;
        m.tamable = true;
        m.control_slots = 2;
        m.min_tame_skill = 95.1;
    });
});

mobile("CoMWarHorse", |c| {
    c.inherits = "Server.Mobiles.BaseWarHorse";
    c.fight_mode = "Aggressor";
    c.mount_body = 119;
    c.mount_item_id = 16049;
});

mobile("MinaxWarHorse", |c| {
    c.inherits = "Server.Mobiles.BaseWarHorse";
    c.fight_mode = "Aggressor";
    c.mount_body = 120;
    c.mount_item_id = 16047;
});

mobile("SLWarHorse", |c| {
    c.inherits = "Server.Mobiles.BaseWarHorse";
    c.fight_mode = "Aggressor";
    c.mount_body = 121;
    c.mount_item_id = 16048;
});

mobile("TBWarHorse", |c| {
    c.inherits = "Server.Mobiles.BaseWarHorse";
    c.fight_mode = "Aggressor";
    c.mount_body = 118;
    c.mount_item_id = 16050;
});

mobile("GiantSerpent", |c| {
    c.corpse_name = "a giant serpent corpse";
    c.name = "a giant snake";
    c.poison_immune = 2;
    c.death_adder_charmable = true;
    c.meat = 4;
    c.hides = 15;
    c.hide_type = "Spined";
    c.loot = "Average";
    c.hit_poison = #{ chance: 0.8, then: 2, "else": 3 };
    c.on_create(|m| {
        m.body = 21;
        m.creature_hue = block([#{ from: 2001 }, #{ count: 18 }]);
        m.sound = 0xDB;
        m.str = 186..=215;
        m.dex = 56..=80;
        m.int = 66..=85;
        m.hits = 112..=129;
        m.mana = 0;
        m.damage = 7..=17;
        m.damage_type("Physical", 40);
        m.damage_type("Poison", 60);
        m.resist("Physical", 30, 35);
        m.resist("Fire", 5, 10);
        m.resist("Cold", 10, 20);
        m.resist("Poison", 70, 90);
        m.resist("Energy", 10, 20);
        m.skill("Poisoning", 70.1, 100.0);
        m.skill("MagicResist", 25.1, 40.0);
        m.skill("Tactics", 65.1, 70.0);
        m.skill("Wrestling", 60.1, 80.0);
        m.fame = 2500;
        m.karma = -2500;
        m.armor = 32;
        m.pack = [#{ item: "Server.Items.Bone" }];
    });
});

mobile("IceSerpent", |c| {
    c.corpse_name = "an ice serpent corpse";
    c.name = "a giant ice serpent";
    c.death_adder_charmable = true;
    c.meat = 4;
    c.hides = 15;
    c.hide_type = "Spined";
    c.loot = "Meager";
    c.on_create(|m| {
        m.body = 89;
        m.sound = 0xDB;
        m.str = 216..=245;
        m.dex = 26..=50;
        m.int = 66..=85;
        m.hits = 130..=147;
        m.mana = 0;
        m.damage = 7..=17;
        m.damage_type("Physical", 10);
        m.damage_type("Cold", 90);
        m.resist("Physical", 30, 35);
        m.resist("Cold", 80, 90);
        m.resist("Poison", 15, 25);
        m.resist("Energy", 10, 20);
        m.skill("Anatomy", 27.5, 50.0);
        m.skill("MagicResist", 25.1, 40.0);
        m.skill("Tactics", 75.1, 80.0);
        m.skill("Wrestling", 60.1, 80.0);
        m.fame = 3500;
        m.karma = -3500;
        m.armor = 32;
        m.creature_item = [
            #{ loot: "ArmorOrShieldOrWeapon" },
            #{
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
            },
        ];
        m.pack = [#{ chance: 0.025, item: "Server.Items.GlacialStaff" }];
    });
});

mobile("LavaLizard", |c| {
    c.corpse_name = "a lava lizard corpse";
    c.name = "a lava lizard";
    c.hides = 12;
    c.hide_type = "Spined";
    c.favorite_food = ["Metal", "Gold"];
    c.loot = "Meager";
    c.monster_abilities = ["FireBreath"];
    c.on_create(|m| {
        m.body = 206;
        m.creature_hue = #{ pick: [1607, 1616, 1625, 1634, 1643, 1652] };
        m.sound = 0x5A;
        m.str = 126..=150;
        m.dex = 56..=75;
        m.int = 11..=20;
        m.hits = 76..=90;
        m.mana = 0;
        m.damage = 6..=24;
        m.damage_type("Physical", 100);
        m.resist("Physical", 35, 45);
        m.resist("Fire", 30, 45);
        m.resist("Poison", 25, 35);
        m.resist("Energy", 25, 35);
        m.skill("MagicResist", 55.1, 70.0);
        m.skill("Tactics", 60.1, 80.0);
        m.skill("Wrestling", 60.1, 80.0);
        m.fame = 3000;
        m.karma = -3000;
        m.armor = 40;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = 80.7;
        m.creature_item = block([#{ item: "Server.Items.SulfurousAsh" }, #{ amount: #{ from: 4, count: 10 } }]);
    });
});

mobile("LavaSnake", |c| {
    c.corpse_name = "a lava snake corpse";
    c.name = "a lava snake";
    c.death_adder_charmable = true;
    c.meat = 1;
    c.loot = "Poor";
    c.monster_abilities = ["FireBreath"];
    c.on_create(|m| {
        m.body = 52;
        m.creature_hue = #{ pick: [1607, 1616, 1625, 1634, 1643, 1652] };
        m.sound = 0xDB;
        m.str = 43..=55;
        m.dex = 16..=25;
        m.int = 6..=10;
        m.hits = 28..=32;
        m.mana = 0;
        m.damage = 1..=8;
        m.damage_type("Physical", 100);
        m.resist("Physical", 20, 25);
        m.resist("Fire", 30, 40);
        m.resist("Poison", 20, 30);
        m.resist("Energy", 10, 20);
        m.skill("MagicResist", 15.1, 20.0);
        m.skill("Tactics", 19.3, 34.0);
        m.skill("Wrestling", 19.3, 34.0);
        m.fame = 600;
        m.karma = -600;
        m.armor = 24;
        m.pack = [#{ item: "Server.Items.SulfurousAsh" }];
    });
});

mobile("Snake", |c| {
    c.corpse_name = "a snake corpse";
    c.name = "a snake";
    c.poison_immune = 0;
    c.hit_poison = 0;
    c.death_adder_charmable = true;
    c.meat = 1;
    c.favorite_food = ["Eggs"];
    c.on_create(|m| {
        m.body = 52;
        m.creature_hue = block([#{ from: 2001 }, #{ count: 18 }]);
        m.sound = 0xDB;
        m.str = 22..=34;
        m.dex = 16..=25;
        m.int = 6..=10;
        m.hits = 15..=19;
        m.mana = 0;
        m.damage = 1..=4;
        m.damage_type("Physical", 100);
        m.resist("Physical", 15, 20);
        m.resist("Poison", 20, 30);
        m.skill("Poisoning", 50.1, 70.0);
        m.skill("MagicResist", 15.1, 20.0);
        m.skill("Tactics", 19.3, 34.0);
        m.skill("Wrestling", 19.3, 34.0);
        m.fame = 300;
        m.karma = -300;
        m.armor = 16;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = 59.1;
    });
});

mobile("Rabbit", |c| {
    c.ai = "Animal";
    c.fight_mode = "Aggressor";
    c.corpse_name = "a hare corpse";
    c.name = "a rabbit";
    c.meat = 1;
    c.hides = 1;
    c.favorite_food = ["FruitsAndVeggies"];
    c.attack_sound = 201;
    c.hurt_sound = 202;
    c.death_sound = 203;
    c.on_create(|m| {
        m.body = 205;
        if coin() {
            m.creature_hue = block([#{ from: 2301 }, #{ count: 18 }]);
        } else {}
        m.str = 6..=10;
        m.dex = 26..=38;
        m.int = 6..=14;
        m.hits = 4..=6;
        m.mana = 0;
        m.damage = 1..=1;
        m.damage_type("Physical", 100);
        m.resist("Physical", 5, 10);
        m.skill("MagicResist", 5.0);
        m.skill("Tactics", 5.0);
        m.skill("Wrestling", 5.0);
        m.fame = 150;
        m.karma = 0;
        m.armor = 6;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = -18.9;
    });
});

mobile("Jwilson", |c| {
    c.corpse_name = "a jwilson corpse";
    c.name = "a jwilson";
    c.anger_sound = 456;
    c.idle_sound = 457;
    c.attack_sound = 458;
    c.hurt_sound = 459;
    c.death_sound = 460;
    c.on_create(|m| {
        m.creature_hue = #{ pick: [2204, 2210, 2216, 2222] };
        m.body = 51;
        m.armor = 8;
        m.creature_init_stats = [#{ from: 22, count: 13 }, #{ from: 16, count: 6 }, #{ from: 16, count: 5 }];
        m.creature_skill_base = block([
            #{ Wrestling: #{ from: 24, count: 17 } },
            #{ Tactics: #{ from: 18, count: 14 } },
            #{ MagicResist: #{ from: 15, count: 6 } },
            #{ Poisoning: #{ from: 31, count: 20 } },
        ]);
        m.creature_fame = block([#{ from: 0 }, #{ count: 1249 }]);
        m.creature_karma = block([#{ from: 0 }, #{ count: 624 }, #{ neg: true }]);
    });
});

mobile("Parrot", |c| {
    c.ai = "Animal";
    c.fight_mode = "Aggressor";
    c.corpse_name = "a parrot corpse";
    c.name = "a parrot";
    c.anger_sound = 27;
    c.idle_sound = 28;
    c.attack_sound = 29;
    c.hurt_sound = 30;
    c.death_sound = 31;
    c.on_create(|m| {
        m.body = 831;
        m.creature_virtual_armor = block([#{ from: 0 }, #{ count: 6 }]);
        m.creature_init_stats = [10, #{ from: 25, count: 16 }, 10];
        m.skill_base("Wrestling", 6.0);
        m.skill_base("Tactics", 6.0);
        m.skill_base("MagicResist", 5.0);
        m.creature_fame = block([#{ from: 0 }, #{ count: 1249 }]);
        m.creature_karma = block([#{ from: 0 }, #{ count: 624 }, #{ neg: true }]);
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = 0.0;
    });
});

mobile("Bird", |c| {
    c.ai = "Animal";
    c.fight_mode = "Aggressor";
    c.corpse_name = "a bird corpse";
    c.meat_type = "Bird";
    c.meat = 1;
    c.feathers = 25;
    c.favorite_food = ["FruitsAndVeggies", "GrainsAndHay"];
    c.on_create(|m| {
        if coin() {
            m.hue = 0x901;
            switch random_below(3) {
                0 => {
                    m.name = "a crow";
                },
                1 => {
                    m.name = "a magpie";
                },
                2 => {
                    m.name = "a raven";
                }
            }
        } else {
            m.creature_hue = block([#{ from: 2101 }, #{ count: 30 }]);
            m.name = random_name("bird");
        }
        m.body = 6;
        m.sound = 0x1B;
        m.creature_virtual_armor = block([#{ min: 0 }, #{ max: 6 }]);
        m.str = 10;
        m.dex = 25..=35;
        m.int = 10;
        m.damage = 0..=0;
        m.damage_type("Physical", 100);
        m.skill("Wrestling", 4.2, 6.4);
        m.skill("Tactics", 4.0, 6.0);
        m.skill("MagicResist", 4.0, 5.0);
        m.fame = 150;
        m.karma = 0;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = -6.9;
    });
});

mobile("TropicalBird", |c| {
    c.ai = "Animal";
    c.fight_mode = "Aggressor";
    c.corpse_name = "a bird corpse";
    c.name = "a tropical bird";
    c.meat_type = "Bird";
    c.meat = 1;
    c.feathers = 25;
    c.favorite_food = ["FruitsAndVeggies", "GrainsAndHay"];
    c.on_create(|m| {
        m.creature_hue = block([#{ from: 2101 }, #{ count: 30 }]);
        m.body = 6;
        m.sound = 0xBF;
        m.creature_virtual_armor = block([#{ min: 0 }, #{ max: 6 }]);
        m.str = 10;
        m.dex = 25..=35;
        m.int = 10;
        m.damage = 0..=0;
        m.damage_type("Physical", 100);
        m.skill("Wrestling", 4.2, 6.4);
        m.skill("Tactics", 4.0, 6.0);
        m.skill("MagicResist", 4.0, 5.0);
        m.fame = 150;
        m.karma = 0;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = -6.9;
    });
});

mobile("Cat", |c| {
    c.ai = "Animal";
    c.fight_mode = "Aggressor";
    c.corpse_name = "a cat corpse";
    c.name = "a cat";
    c.meat = 1;
    c.favorite_food = ["Meat", "Fish"];
    c.pack_instinct = ["Feline"];
    c.on_create(|m| {
        m.body = 201;
        m.creature_hue = block([#{ from: 2301 }, #{ count: 18 }]);
        m.sound = 0x69;
        m.str = 9;
        m.dex = 35;
        m.int = 5;
        m.hits = 6;
        m.mana = 0;
        m.damage = 1..=1;
        m.damage_type("Physical", 100);
        m.resist("Physical", 5, 10);
        m.skill("MagicResist", 5.0);
        m.skill("Tactics", 4.0);
        m.skill("Wrestling", 5.0);
        m.fame = 0;
        m.karma = 150;
        m.armor = 8;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = -0.9;
    });
});

mobile("Dog", |c| {
    c.ai = "Animal";
    c.fight_mode = "Aggressor";
    c.corpse_name = "a dog corpse";
    c.name = "a dog";
    c.meat = 1;
    c.favorite_food = ["Meat"];
    c.pack_instinct = ["Canine"];
    c.on_create(|m| {
        m.body = 217;
        m.creature_hue = block([#{ from: 2301 }, #{ count: 18 }]);
        m.sound = 0x85;
        m.str = 27..=37;
        m.dex = 28..=43;
        m.int = 29..=37;
        m.hits = 17..=22;
        m.mana = 0;
        m.damage = 4..=7;
        m.damage_type("Physical", 100);
        m.resist("Physical", 10, 15);
        m.skill("MagicResist", 22.1, 47.0);
        m.skill("Tactics", 19.2, 31.0);
        m.skill("Wrestling", 19.2, 31.0);
        m.fame = 0;
        m.karma = 300;
        m.armor = 12;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = -15.3;
    });
});
