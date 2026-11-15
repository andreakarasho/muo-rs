mobile("Silvani", |c| {
    c.ai = "Mage";
    c.fight_mode = "Evil";
    c.range_perception = 18;
    c.range_fight = 1;
    c.on_create(|m| {
        m.body = 176;
        m.sound = 0x467;
        m.str = 253..=400;
        m.dex = 157..=850;
        m.int = 503..=800;
        m.hits = 600;
        m.damage = 27..=38;
        m.damage_type("Physical", 75);
        m.damage_type("Cold", 25);
        m.resist("Physical", 45, 55);
        m.resist("Fire", 30, 40);
        m.resist("Cold", 30, 40);
        m.resist("Poison", 40, 50);
        m.resist("Energy", 40, 50);
        m.skill("EvalInt", 100.0);
        m.skill("Magery", 97.6, 107.5);
        m.skill("Meditation", 100.0);
        m.skill("MagicResist", 100.5, 150.0);
        m.skill("Tactics", 97.6, 100.0);
        m.skill("Wrestling", 97.6, 100.0);
        m.fame = 20000;
        m.karma = 20000;
        m.armor = 50;
    });
});

mobile("ServantOfSemidar", |c| {
    c.ai = "Melee";
    c.fight_mode = "None";
    c.range_fight = 1;
    c.on_create(|m| {
        m.body = 38;
    });
});

mobile("DarkGuardian", |c| {
    c.ai = "Mage";
    c.fight_mode = "Closest";
    c.range_fight = 1;
    c.on_create(|m| {
        m.body = 78;
        m.sound = 0x3E9;
        m.str = 125..=150;
        m.dex = 100..=120;
        m.int = 200..=235;
        m.hits = 150..=180;
        m.damage = 43..=48;
        m.damage_type("Physical", 10);
        m.damage_type("Cold", 40);
        m.damage_type("Energy", 50);
        m.resist("Physical", 40, 50);
        m.resist("Fire", 20, 45);
        m.resist("Cold", 50, 60);
        m.resist("Poison", 20, 45);
        m.resist("Energy", 30, 40);
        m.skill("EvalInt", 40.1, 50.0);
        m.skill("Magery", 50.1, 60.0);
        m.skill("Meditation", 85.1, 95.0);
        m.skill("MagicResist", 50.1, 70.0);
        m.skill("Tactics", 50.1, 70.0);
        m.fame = 5000;
        m.karma = -5000;
        m.armor = 50;
        m.pack = [#{ necroReg: [15, 25] }, #{ item: "Server.Items.DaemonBone", amount: 30 }];
    });
});

mobile("Barracoon", |c| {
    c.ai = "Melee";
    c.fight_mode = "Closest";
    c.range_perception = 18;
    c.range_fight = 1;
    c.on_create(|m| {
        m.title = "the piper";
        m.body = 400;
        m.hue = 0x83EC;
        m.str = 305..=425;
        m.dex = 72..=150;
        m.int = 505..=750;
        m.hits = 4200;
        m.stam = 102..=300;
        m.damage = 25..=35;
        m.damage_type("Physical", 100);
        m.resist("Physical", 60, 70);
        m.resist("Fire", 50, 60);
        m.resist("Cold", 50, 60);
        m.resist("Poison", 40, 50);
        m.resist("Energy", 40, 50);
        m.skill("MagicResist", 100.0);
        m.skill("Tactics", 97.6, 100.0);
        m.skill("Wrestling", 97.6, 100.0);
        m.fame = 22500;
        m.karma = -22500;
        m.armor = 70;
        m.npc_add_item = [
            #{ item: "Server.Items.FancyShirt", hue: "randomGreen" },
            #{ item: "Server.Items.LongPants", hue: "randomYellow" },
            #{ item: "Server.Items.JesterHat", hue: "randomPink" },
            #{ item: "Server.Items.Cloak", hue: "randomPink" },
            "Server.Items.Sandals",
        ];
        m.npc_hair_item_id = 8251;
        m.npc_hair_hue = 148;
    });
});

mobile("Mephitis", |c| {
    c.ai = "Melee";
    c.fight_mode = "Closest";
    c.range_perception = 18;
    c.range_fight = 1;
    c.on_create(|m| {
        m.body = 173;
        m.sound = 0x183;
        m.str = 505..=1000;
        m.dex = 102..=300;
        m.int = 402..=600;
        m.hits = 3000;
        m.stam = 105..=600;
        m.damage = 21..=33;
        m.damage_type("Physical", 50);
        m.damage_type("Poison", 50);
        m.resist("Physical", 75, 80);
        m.resist("Fire", 60, 70);
        m.resist("Cold", 60, 70);
        m.resist("Poison", 100);
        m.resist("Energy", 60, 70);
        m.skill("MagicResist", 70.7, 140.0);
        m.skill("Tactics", 97.6, 100.0);
        m.skill("Wrestling", 97.6, 100.0);
        m.fame = 22500;
        m.karma = -22500;
        m.armor = 80;
    });
});

mobile("Rikktor", |c| {
    c.ai = "Melee";
    c.fight_mode = "Closest";
    c.range_perception = 18;
    c.range_fight = 1;
    c.on_create(|m| {
        m.body = 172;
        m.str = 701..=900;
        m.dex = 201..=350;
        m.int = 51..=100;
        m.hits = 3000;
        m.stam = 203..=650;
        m.damage = 28..=55;
        m.damage_type("Physical", 25);
        m.damage_type("Fire", 50);
        m.damage_type("Energy", 25);
        m.resist("Physical", 80, 90);
        m.resist("Fire", 80, 90);
        m.resist("Cold", 30, 40);
        m.resist("Poison", 80, 90);
        m.resist("Energy", 80, 90);
        m.skill("Anatomy", 100.0);
        m.skill("MagicResist", 140.2, 160.0);
        m.skill("Tactics", 100.0);
        m.fame = 22500;
        m.karma = -22500;
        m.armor = 130;
    });
});

mobile("Semidar", |c| {
    c.ai = "Mage";
    c.fight_mode = "Closest";
    c.range_perception = 18;
    c.range_fight = 1;
    c.on_create(|m| {
        m.body = 174;
        m.sound = 0x4B0;
        m.str = 502..=600;
        m.dex = 102..=200;
        m.int = 601..=750;
        m.hits = 1500;
        m.stam = 103..=250;
        m.damage = 29..=35;
        m.damage_type("Physical", 75);
        m.damage_type("Fire", 25);
        m.resist("Physical", 20, 30);
        m.resist("Fire", 50, 60);
        m.resist("Cold", 20, 30);
        m.resist("Poison", 20, 30);
        m.resist("Energy", 10, 20);
        m.skill("EvalInt", 95.1, 100.0);
        m.skill("Magery", 90.1, 105.0);
        m.skill("Meditation", 95.1, 100.0);
        m.skill("MagicResist", 120.2, 140.0);
        m.skill("Tactics", 90.1, 105.0);
        m.skill("Wrestling", 90.1, 105.0);
        m.fame = 24000;
        m.karma = -24000;
        m.armor = 20;
    });
});

mobile("Serado", |c| {
    c.ai = "Melee";
    c.fight_mode = "Closest";
    c.range_perception = 18;
    c.range_fight = 1;
    c.on_create(|m| {
        m.title = "the awakened";
        m.body = 249;
        m.hue = 0x96C;
        m.str = 1000;
        m.dex = 150;
        m.int = 300;
        m.hits = 9000;
        m.mana = 300;
        m.damage = 29..=35;
        m.damage_type("Physical", 70);
        m.damage_type("Poison", 20);
        m.damage_type("Energy", 10);
        m.resist("Physical", 30);
        m.resist("Fire", 60);
        m.resist("Cold", 60);
        m.resist("Poison", 90);
        m.resist("Energy", 50);
        m.skill("MagicResist", 120.0);
        m.skill("Tactics", 120.0);
        m.skill("Wrestling", 70.0);
        m.skill("Poisoning", 150.0);
        m.fame = 22500;
        m.karma = -22500;
    });
});

mobile("Neira", |c| {
    c.ai = "Mage";
    c.fight_mode = "Closest";
    c.range_perception = 18;
    c.range_fight = 1;
    c.on_create(|m| {
        m.title = "the necromancer";
        m.body = 401;
        m.hue = 0x83EC;
        m.str = 305..=425;
        m.dex = 72..=150;
        m.int = 505..=750;
        m.hits = 4800;
        m.stam = 102..=300;
        m.damage = 25..=35;
        m.damage_type("Physical", 100);
        m.resist("Physical", 25, 30);
        m.resist("Fire", 35, 45);
        m.resist("Cold", 50, 60);
        m.resist("Poison", 30, 40);
        m.resist("Energy", 20, 30);
        m.skill("EvalInt", 120.0);
        m.skill("Magery", 120.0);
        m.skill("Meditation", 120.0);
        m.skill("MagicResist", 150.0);
        m.skill("Tactics", 97.6, 100.0);
        m.skill("Wrestling", 97.6, 100.0);
        m.fame = 22500;
        m.karma = -22500;
        m.armor = 30;
        m.female = true;
    });
});

mobile("LordOaks", |c| {
    c.ai = "Mage";
    c.fight_mode = "Evil";
    c.range_perception = 18;
    c.range_fight = 1;
    c.on_create(|m| {
        m.body = 175;
        m.str = 403..=850;
        m.dex = 101..=150;
        m.int = 503..=800;
        m.hits = 3000;
        m.stam = 202..=400;
        m.damage = 21..=33;
        m.damage_type("Physical", 75);
        m.damage_type("Fire", 25);
        m.resist("Physical", 85, 90);
        m.resist("Fire", 60, 70);
        m.resist("Cold", 60, 70);
        m.resist("Poison", 80, 90);
        m.resist("Energy", 80, 90);
        m.skill("Anatomy", 75.1, 100.0);
        m.skill("EvalInt", 120.1, 130.0);
        m.skill("Magery", 120.0);
        m.skill("Meditation", 120.1, 130.0);
        m.skill("MagicResist", 100.5, 150.0);
        m.skill("Tactics", 100.0);
        m.skill("Wrestling", 100.0);
        m.fame = 22500;
        m.karma = 22500;
        m.armor = 100;
    });
});

mobile("Harrower", |c| {
    c.ai = "Mage";
    c.fight_mode = "Closest";
    c.range_perception = 18;
    c.range_fight = 1;
    c.on_create(|m| {
        m.body = 146;
        m.str = 900..=1000;
        m.dex = 125..=135;
        m.int = 1000..=1200;
        m.fame = 22500;
        m.karma = -22500;
        m.armor = 60;
        m.damage_type("Physical", 50);
        m.damage_type("Energy", 50);
        m.resist("Physical", 55, 65);
        m.resist("Fire", 60, 80);
        m.resist("Cold", 60, 80);
        m.resist("Poison", 60, 80);
        m.resist("Energy", 60, 80);
        m.skill("Wrestling", 90.1, 100.0);
        m.skill("Tactics", 90.2, 110.0);
        m.skill("MagicResist", 120.2, 160.0);
        m.skill("Magery", 120.0);
        m.skill("EvalInt", 120.0);
        m.skill("Meditation", 120.0);
    });
});

mobile("HarrowerTentacles", |c| {
    c.ai = "Melee";
    c.fight_mode = "Closest";
    c.range_fight = 1;
    c.on_create(|m| {
        m.body = 129;
        m.str = 901..=1000;
        m.dex = 126..=140;
        m.int = 1001..=1200;
        m.hits = 541..=600;
        m.damage = 13..=20;
        m.damage_type("Physical", 20);
        m.damage_type("Fire", 20);
        m.damage_type("Cold", 20);
        m.damage_type("Poison", 20);
        m.damage_type("Energy", 20);
        m.resist("Physical", 55, 65);
        m.resist("Fire", 35, 45);
        m.resist("Cold", 35, 45);
        m.resist("Poison", 35, 45);
        m.resist("Energy", 35, 45);
        m.skill("Meditation", 100.0);
        m.skill("MagicResist", 120.1, 140.0);
        m.skill("Swords", 90.1, 100.0);
        m.skill("Tactics", 90.1, 100.0);
        m.skill("Wrestling", 90.1, 100.0);
        m.fame = 15000;
        m.karma = -15000;
        m.armor = 60;
        m.pack = [#{ reg: 50 }, #{ necroReg: [15, 75] }];
    });
});

mobile("ChaosGuard", |c| {
    c.ai = "Melee";
    c.fight_mode = "Aggressor";
    c.range_perception = 14;
    c.range_fight = 1;
    c.on_create(|m| {
        m.npc_init_stats = [1000, 1000, 1000];
        m.title = "the guard";
        m.npc_speed = [0.5, 2.0];
        m.npc_speech_hue = "randomDyed";
        m.npc_hue = "humanSkin";
        if coin() {
            m.female = true;
            m.body = 401;
            m.name = random_name("female");
            m.npc_add_item = ["Server.Items.FemalePlateChest", "Server.Items.PlateArms", "Server.Items.PlateLegs"];
            m.npc_seq = [
                #{
                    oneOf: [
                        #{ npcAddItem: [#{ item: "Server.Items.Doublet", hue: "randomNondyed" }] },
                        #{ npcAddItem: [#{ item: "Server.Items.BodySash", hue: "randomNondyed" }] },
                    ],
                },
                #{
                    oneOf: [
                        #{ npcAddItem: [#{ item: "Server.Items.Skirt", hue: "randomNondyed" }] },
                        #{ npcAddItem: [#{ item: "Server.Items.Kilt", hue: "randomNondyed" }] },
                    ],
                },
            ];
        } else {
            m.female = false;
            m.body = 400;
            m.name = random_name("male");
            m.npc_add_item = ["Server.Items.PlateChest", "Server.Items.PlateArms", "Server.Items.PlateLegs"];
            switch random_below(3) {
                0 => {
                    m.npc_add_item = [#{ item: "Server.Items.Doublet", hue: "randomNondyed" }];
                },
                1 => {
                    m.npc_add_item = [#{ item: "Server.Items.Tunic", hue: "randomNondyed" }];
                },
                2 => {
                    m.npc_add_item = [#{ item: "Server.Items.BodySash", hue: "randomNondyed" }];
                }
            }
        }
        m.npc_seq = [
            block([#{ randomHair: true }, #{ coin: #{ then: #{ npcFacialHair: "hairHue" } } }]),
            #{
                npcAddItem: [
                    #{ item: "Server.Items.VikingSword", movable: false },
                    #{ item: "Server.Items.ChaosShield", movable: false },
                ],
                pack: [#{ gold: [250, 500] }],
                skillBase: #{ Anatomy: 120.0, Tactics: 120.0, Swords: 120.0, MagicResist: 120.0, DetectHidden: 100.0 },
            },
        ];
    });
});

mobile("OrderGuard", |c| {
    c.ai = "Melee";
    c.fight_mode = "Aggressor";
    c.range_perception = 14;
    c.range_fight = 1;
    c.on_create(|m| {
        m.npc_init_stats = [1000, 1000, 1000];
        m.title = "the guard";
        m.npc_speed = [0.5, 2.0];
        m.npc_speech_hue = "randomDyed";
        m.npc_hue = "humanSkin";
        if coin() {
            m.female = true;
            m.body = 401;
            m.name = random_name("female");
            m.npc_add_item = ["Server.Items.FemalePlateChest", "Server.Items.PlateArms", "Server.Items.PlateLegs"];
            m.npc_seq = [
                #{
                    oneOf: [
                        #{ npcAddItem: [#{ item: "Server.Items.Doublet", hue: "randomNondyed" }] },
                        #{ npcAddItem: [#{ item: "Server.Items.BodySash", hue: "randomNondyed" }] },
                    ],
                },
                #{
                    oneOf: [
                        #{ npcAddItem: [#{ item: "Server.Items.Skirt", hue: "randomNondyed" }] },
                        #{ npcAddItem: [#{ item: "Server.Items.Kilt", hue: "randomNondyed" }] },
                    ],
                },
            ];
        } else {
            m.female = false;
            m.body = 400;
            m.name = random_name("male");
            m.npc_add_item = ["Server.Items.PlateChest", "Server.Items.PlateArms", "Server.Items.PlateLegs"];
            switch random_below(3) {
                0 => {
                    m.npc_add_item = [#{ item: "Server.Items.Doublet", hue: "randomNondyed" }];
                },
                1 => {
                    m.npc_add_item = [#{ item: "Server.Items.Tunic", hue: "randomNondyed" }];
                },
                2 => {
                    m.npc_add_item = [#{ item: "Server.Items.BodySash", hue: "randomNondyed" }];
                }
            }
        }
        m.npc_seq = [
            block([#{ randomHair: true }, #{ coin: #{ then: #{ npcFacialHair: "hairHue" } } }]),
            #{
                npcAddItem: [
                    #{ item: "Server.Items.VikingSword", movable: false },
                    #{ item: "Server.Items.OrderShield", movable: false },
                ],
                pack: [#{ gold: [250, 500] }],
                skillBase: #{ Anatomy: 120.0, Tactics: 120.0, Swords: 120.0, MagicResist: 120.0, DetectHidden: 100.0 },
            },
        ];
    });
});
