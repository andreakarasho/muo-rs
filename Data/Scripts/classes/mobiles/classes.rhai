mobile("LadyMarai", |c| {
    c.corpse_name = "a Lady Marai corpse";
    c.name = "Lady Marai";
    c.gives_ml_minor_artifact = true;
    c.weapon_ability = "CrushingBlow";
    c.loot = [["UltraRich", 3]];
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
        m.creature_item = [
            #{
                pick: [
                    "Server.Items.PlateArms",
                    "Server.Items.PlateChest",
                    "Server.Items.PlateGloves",
                    "Server.Items.PlateGorget",
                    "Server.Items.PlateLegs",
                    "Server.Items.PlateHelm",
                ],
            },
        ];
        m.pack = [#{ item: "Server.Items.Scimitar" }, #{ item: "Server.Items.WoodenShield" }];
        m.paragon = true;
        m.hue = 0x21;
        m.str = 221..=304;
        m.dex = 98..=138;
        m.int = 54..=99;
        m.hits = 694..=846;
        m.damage = 15..=25;
        m.damage_type("Physical", 40);
        m.damage_type("Cold", 60);
        m.resist("Physical", 55, 65);
        m.resist("Fire", 40, 50);
        m.resist("Cold", 70, 80);
        m.resist("Poison", 40, 50);
        m.resist("Energy", 50, 60);
        m.skill("Wrestling", 126.6, 137.2);
        m.skill("Tactics", 128.7, 134.5);
        m.skill("MagicResist", 102.1, 119.1);
        m.skill("Anatomy", 126.2, 136.5);
        m.fame = 18000;
        m.karma = -18000;
    });
});

mobile("DemonKnight", |c| {
    c.ai = "Mage";
    c.corpse_name = "a demon knight corpse";
    c.poison_immune = 4;
    c.treasure_map_level = 1;
    c.loot = [["SuperBoss", 2], ["HighScrolls", #{ min: 6, max: 60 }]];
    c.ignore_young_protection = #{ atLeast: "ML", then: true, "else": false };
    c.bard_immune = #{ atLeast: "SE", then: false, "else": true };
    c.unprovokable = #{ atLeast: "SE", then: true, "else": false };
    c.area_peace_immune = #{ atLeast: "SE", then: true, "else": false };
    c.weapon_ability = #{ oneOf: ["DoubleStrike", "WhirlwindAttack", "CrushingBlow"] };
    c.on_create(|m| {
        m.name = random_name("demon knight");
        m.title = "the Dark Father";
        m.body = 318;
        m.sound = 0x165;
        m.str = 500;
        m.dex = 100;
        m.int = 1000;
        m.hits = 30000;
        m.mana = 5000;
        m.damage = 17..=21;
        m.damage_type("Physical", 20);
        m.damage_type("Fire", 20);
        m.damage_type("Cold", 20);
        m.damage_type("Poison", 20);
        m.damage_type("Energy", 20);
        m.resist("Physical", 30);
        m.resist("Fire", 30);
        m.resist("Cold", 30);
        m.resist("Poison", 30);
        m.resist("Energy", 30);
        m.skill("Necromancy", 120.0, 120.0);
        m.skill("SpiritSpeak", 120.0, 120.0);
        m.skill("DetectHidden", 80.0);
        m.skill("EvalInt", 100.0);
        m.skill("Magery", 100.0);
        m.skill("Meditation", 120.0);
        m.skill("MagicResist", 150.0);
        m.skill("Tactics", 100.0);
        m.skill("Wrestling", 120.0);
        m.fame = 28000;
        m.karma = -28000;
        m.armor = 64;
    });
});

mobile("Ilhenir", |c| {
    c.ai = "Mage";
    c.range_perception = 18;
    c.corpse_name = "a corpse of Ilhenir";
    c.name = "Ilhenir";
    c.unprovokable = true;
    c.uncalmable = true;
    c.poison_immune = 4;
    c.treasure_map_level = 5;
    c.loot = [["FilthyRich", 8]];
    c.anger_sound = 1409;
    c.idle_sound = 1410;
    c.attack_sound = 1408;
    c.hurt_sound = 1411;
    c.death_sound = 1412;
    c.on_create(|m| {
        m.title = "the Stained";
        m.body = 259;
        m.sound = 0x24D;
        m.str = 1105..=1350;
        m.dex = 82..=160;
        m.int = 505..=750;
        m.hits = 9000;
        m.damage = 21..=28;
        m.damage_type("Physical", 60);
        m.damage_type("Fire", 20);
        m.damage_type("Poison", 20);
        m.resist("Physical", 55, 65);
        m.resist("Fire", 50, 60);
        m.resist("Cold", 55, 65);
        m.resist("Poison", 70, 90);
        m.resist("Energy", 65, 75);
        m.skill("EvalInt", 100.0);
        m.skill("Magery", 100.0);
        m.skill("Meditation", 0.0);
        m.skill("Poisoning", 5.4);
        m.skill("Anatomy", 117.5);
        m.skill("MagicResist", 120.0);
        m.skill("Tactics", 119.9);
        m.skill("Wrestling", 119.9);
        m.fame = 50000;
        m.karma = -50000;
        m.armor = 44;
        if era("ML") {
            m.creature_item = [
                #{
                    pick: [
                        "Server.Items.Blight",
                        "Server.Items.Scourge",
                        "Server.Items.Taint",
                        "Server.Items.Putrefication",
                        "Server.Items.Corruption",
                        "Server.Items.Muculent",
                    ],
                    times: 8,
                },
                #{ item: "Server.Items.RandomTalisman", times: #{ from: 0, count: 5 } },
            ];
        }
    });
});

mobile("Meraktus", |c| {
    c.range_perception = 18;
    c.corpse_name = "the remains of Meraktus";
    c.name = "Meraktus";
    c.meat = 2;
    c.hides = 10;
    c.hide_type = "Regular";
    c.poison_immune = 1;
    c.treasure_map_level = 3;
    c.bard_immune = true;
    c.unprovokable = true;
    c.uncalmable = true;
    c.weapon_ability = "Dismount";
    c.loot = [#{ era: "ML", loot: [["AosSuperBoss", 5]] }];
    c.anger_sound = 1431;
    c.idle_sound = 1430;
    c.attack_sound = 1433;
    c.hurt_sound = 1434;
    c.death_sound = 1436;
    c.on_create(|m| {
        m.creature_setup = "NoFollowers";
        m.title = "the Tormented";
        m.body = 263;
        m.sound = 0x2A8;
        m.hue = 0x835;
        m.str = 1419..=1438;
        m.dex = 309..=413;
        m.int = 129..=131;
        m.hits = 4100..=4200;
        m.damage = 16..=30;
        m.damage_type("Physical", 100);
        m.resist("Physical", 65, 90);
        m.resist("Fire", 65, 70);
        m.resist("Cold", 50, 60);
        m.resist("Poison", 40, 60);
        m.resist("Energy", 50, 55);
        m.skill("Anatomy", 0.0);
        m.skill("MagicResist", 107.0, 111.3);
        m.skill("Tactics", 107.0, 117.0);
        m.skill("Wrestling", 100.0, 105.0);
        m.fame = 70000;
        m.karma = -70000;
        m.armor = 28;
        if era("ML") {
            m.creature_item = [
                #{
                    pick: [
                        "Server.Items.Blight",
                        "Server.Items.Scourge",
                        "Server.Items.Taint",
                        "Server.Items.Putrefication",
                        "Server.Items.Corruption",
                        "Server.Items.Muculent",
                    ],
                    times: 8,
                },
                #{ item: "Server.Items.RandomTalisman", times: #{ from: 0, count: 5 } },
            ];
        }
    });
});

mobile("Twaulo", |c| {
    c.range_perception = 18;
    c.corpse_name = "a corpse of Twaulo";
    c.name = "Twaulo";
    c.opposition = "FeyAndUndead";
    c.unprovokable = true;
    c.poison_immune = 1;
    c.treasure_map_level = 5;
    c.meat = 1;
    c.hides = 8;
    c.hide_type = "Spined";
    c.loot = [["UltraRich", 2], "Average", "Gems"];
    c.monster_abilities = ["SummonPixiesCounter"];
    c.on_create(|m| {
        m.title = "of the Glade";
        m.body = 101;
        m.sound = 0x2A7;
        m.hue = 0x455;
        m.str = 1751..=1950;
        m.dex = 251..=450;
        m.int = 801..=1000;
        m.hits = 7500;
        m.damage = 19..=24;
        m.damage_type("Physical", 100);
        m.resist("Physical", 65, 75);
        m.resist("Fire", 45, 55);
        m.resist("Cold", 50, 60);
        m.resist("Poison", 50, 60);
        m.resist("Energy", 50, 60);
        m.skill("EvalInt", 0.0);
        m.skill("Magery", 0.0);
        m.skill("Meditation", 0.0);
        m.skill("Anatomy", 95.1, 115.0);
        m.skill("Archery", 95.1, 100.0);
        m.skill("MagicResist", 50.3, 80.0);
        m.skill("Tactics", 90.1, 100.0);
        m.skill("Wrestling", 95.1, 100.0);
        m.fame = 50000;
        m.karma = 50000;
        m.armor = 50;
        m.creature_item = #{ item: "Server.Items.Bow", to: "add" };
        m.pack = [#{ item: "Server.Items.Arrow", amount: [500, 700] }];
    });
});

mobile("SwampDragon", |c| {
    c.fight_mode = "Aggressor";
    c.mount_body = 794;
    c.mount_item_id = 16061;
    c.name = "a swamp dragon";
    c.corpse_name = "a swamp dragon corpse";
    c.steps_max = 4480;
    c.reacquire_on_movement = true;
    c.favorite_food = ["Meat"];
    c.meat = 19;
    c.hides = 20;
    c.scales = 5;
    c.scale_type = "Green";
    c.can_anger_on_tame = true;
    c.override_bonding_reqs = true;
    c.idle_sound = 718;
    c.death_sound = 716;
    c.hurt_sound = 721;
    c.attack_sound = 712;
    c.on_create(|m| {
        m.creature_setup = "SwampDragon";
        m.sound = 0x16A;
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
        m.hue = 0x851;
        m.tamable = true;
        m.control_slots = 1;
        m.min_tame_skill = 93.9;
    });
});

mobile("Golem", |c| {
    c.script = "content/creatures/golem.rhai";
    c.corpse_name = "a golem corpse";
    c.is_scared_of_scary_things = false;
    c.is_scary_to_pets = true;
    c.is_bondable = false;
    c.name = "a golem";
    c.delete_on_release = true;
    c.bleed_immune = true;
    c.monster_abilities = ["ColossalBlow"];
    c.anger_sound = 541;
    c.attack_sound = 562;
    c.favorite_food = [];
    c.poison_immune = 4;
    c.on_create(|m| {
        m.body = 752;
        m.control_slots = 3;
    });
});

mobile("KhaldunRevenant", |c| {
    c.name = "a revenant";
    c.delete_corpse_on_death = true;
    c.always_attackable = true;
    c.bard_immune = true;
    c.poison_immune = 4;
    c.idle_sound = 447;
    c.anger_sound = 263;
    c.death_sound = 253;
});

mobile("Leviathan", |c| {
    c.ai = "Mage";
    c.corpse_name = "a leviathan corpse";
    c.name = "a leviathan";
    c.treasure_map_level = 5;
    c.treasure_map_chance = 0.25;
    c.loot = [["FilthyRich", 5]];
    c.monster_abilities = ["LeviathanBreath"];
    c.on_create(|m| {
        m.body = 77;
        m.sound = 0x161;
        m.hue = 0x481;
        m.str = 1000;
        m.dex = 501..=520;
        m.int = 501..=515;
        m.hits = 1500;
        m.damage = 25..=33;
        m.damage_type("Physical", 70);
        m.damage_type("Cold", 30);
        m.resist("Physical", 55, 65);
        m.resist("Fire", 45, 55);
        m.resist("Cold", 45, 55);
        m.resist("Poison", 35, 45);
        m.resist("Energy", 25, 35);
        m.skill("EvalInt", 97.6, 107.5);
        m.skill("Magery", 97.6, 107.5);
        m.skill("MagicResist", 97.6, 107.5);
        m.skill("Meditation", 97.6, 107.5);
        m.skill("Tactics", 97.6, 107.5);
        m.skill("Wrestling", 97.6, 107.5);
        m.fame = 24000;
        m.karma = -24000;
        m.armor = 50;
        m.can_swim = true;
        m.cant_walk = true;
        m.pack = [#{ item: "Server.Items.Rope", itemId: 5368 }, #{ item: "Server.Items.Rope", itemId: 5370 }];
    });
});

mobile("PlagueSpawn", |c| {
    c.corpse_name = "a plague spawn corpse";
    c.always_murderer = true;
    c.name = "a plague spawn";
    c.loot = ["Poor", "Gems"];
    c.on_create(|m| {
        m.creature_hue = block([#{ from: 17 }, #{ count: 15 }]);
        switch random_below(12) {
            0 => {
                m.body = 14;
                m.sound = 0x10C;
            },
            1 => {
                m.body = 31;
                m.sound = 0x39D;
            },
            2 => {
                m.bodies = [400, 401];
            },
            3 => {
                m.body = 29;
                m.sound = 0x9E;
            },
            4 => {
                m.body = 21;
                m.sound = 0xDB;
            },
            5 => {
                m.body = 51;
                m.sound = 0x1C8;
            },
            6 => {
                m.body = 51;
                m.sound = 0x1C8;
            },
            7 => {
                m.body = 51;
                m.sound = 0x1C8;
            },
            8 => {
                m.body = 51;
                m.sound = 0x1C8;
            },
            9 => {
                m.body = 51;
                m.sound = 0x1C8;
            },
            10 => {
                m.body = 51;
                m.sound = 0x1C8;
            },
            11 => {
                m.body = 51;
                m.sound = 0x1C8;
            }
        }
        m.str = 201..=300;
        m.dex = 80;
        m.int = 16..=20;
        m.hits = 121..=180;
        m.damage = 11..=17;
        m.damage_type("Physical", 100);
        m.resist("Physical", 35, 45);
        m.resist("Fire", 30, 40);
        m.resist("Cold", 25, 35);
        m.resist("Poison", 65, 75);
        m.resist("Energy", 25, 35);
        m.skill("MagicResist", 25.0);
        m.skill("Tactics", 25.0);
        m.skill("Wrestling", 50.0);
        m.fame = 1000;
        m.karma = -1000;
        m.armor = 20;
    });
});

mobile("PumpkinHead", |c| {
    c.ai = coin("Melee", "Mage");
    c.corpse_name = "a killer pumpkin corpse";
    c.name = "a killer pumpkin";
    c.auto_dispel = true;
    c.bard_immune = true;
    c.unprovokable = true;
    c.area_peace_immune = true;
    c.loot = [
        #{
            chance: 0.05,
            loot: [
                #{
                    oneOf: [
                        #{ item: "Server.Items.Holiday.PaintedEvilClownMask" },
                        #{ item: "Server.Items.Holiday.PaintedDaemonMask" },
                        #{ item: "Server.Items.Holiday.PaintedPlagueMask" },
                        #{ item: "Server.Items.Holiday.PaintedEvilJesterMask" },
                        #{ item: "Server.Items.Holiday.PaintedPorcelainMask" },
                    ],
                },
            ],
        },
        #{ item: "Server.Items.WrappedCandy" },
        ["UltraRich", 2],
    ];
    c.on_create(|m| {
        m.bodies = [1246, 1247];
        m.sound = 0x10C;
        m.str = 350;
        m.dex = 125;
        m.int = 250;
        m.hits = 500;
        m.mana = 1000;
        m.damage = 10..=15;
        m.damage_type("Physical", 100);
        m.resist("Physical", 55);
        m.resist("Fire", 50);
        m.resist("Cold", 50);
        m.resist("Poison", 65);
        m.resist("Energy", 80);
        m.skill("DetectHidden", 100.0);
        m.skill("Meditation", 120.0);
        m.skill("Necromancy", 100.0);
        m.skill("SpiritSpeak", 120.0);
        m.skill("Magery", 160.0);
        m.skill("EvalInt", 100.0);
        m.skill("MagicResist", 100.0);
        m.skill("Tactics", 100.0);
        m.skill("Wrestling", 80.0);
        m.fame = 5000;
        m.karma = -5000;
        m.armor = 49;
    });
});

mobile("Revenant", |c| {
    c.name = "a revenant";
    c.no_house_restrictions = true;
    c.dispel_difficulty = 80.0;
    c.dispel_focus = 20.0;
    c.always_murderer = true;
    c.bleed_immune = true;
    c.bard_immune = true;
    c.poison_immune = 4;
});

mobile("ZombieSkeleton", |c| {
    c.corpse_name = "a rotting corpse";
    c.bleed_immune = true;
    c.poison_immune = 1;
});

mobile("BaseEnraged", |c| {
    c.script = "content/creatures/enraged.rhai";
    c.on_create(|m| {
        m.str = 50..=200;
        m.dex = 50..=200;
        m.karma = -1000;
        m.tamable = false;
    });
});

mobile("EnragedRabbit", |c| {
    c.inherits = "BaseEnraged";
    c.corpse_name = "a hare corpse";
    c.name = "a rabbit";
    c.attack_sound = 201;
    c.hurt_sound = 202;
    c.death_sound = 203;
    c.on_create(|m| {
        m.body = 205;
    });
});

mobile("EnragedHart", |c| {
    c.inherits = "BaseEnraged";
    c.corpse_name = "a deer corpse";
    c.name = "a great hart";
    c.attack_sound = 130;
    c.hurt_sound = 131;
    c.death_sound = 132;
    c.on_create(|m| {
        m.body = 234;
    });
});

mobile("EnragedHind", |c| {
    c.inherits = "BaseEnraged";
    c.corpse_name = "a deer corpse";
    c.name = "a hind";
    c.attack_sound = 130;
    c.hurt_sound = 131;
    c.death_sound = 132;
    c.on_create(|m| {
        m.body = 237;
    });
});

mobile("EnragedBlackBear", |c| {
    c.inherits = "BaseEnraged";
    c.corpse_name = "a bear corpse";
    c.name = "a black bear";
    c.on_create(|m| {
        m.body = 211;
        m.sound = 0xA3;
    });
});

mobile("EnragedEagle", |c| {
    c.inherits = "BaseEnraged";
    c.corpse_name = "an eagle corpse";
    c.name = "an eagle";
    c.on_create(|m| {
        m.body = 5;
        m.sound = 0x2EE;
    });
});
