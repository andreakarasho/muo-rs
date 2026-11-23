mobile("HireBard", |c| {
    c.ai = "Melee";
    c.npc_click_title = false;
    c.on_create(|m| {
        m.npc_speech_hue = "randomDyed";
        m.npc_hue = "humanSkin";
        m.title = "the bard";
        m.npc_roll_hair = true;
        m.str = 16..=16;
        m.dex = 26..=26;
        m.int = 26..=26;
        m.damage = 5..=10;
        m.skill("Tactics", 35.0, 57.0);
        m.skill("Magery", 22.0, 22.0);
        m.skill("Swords", 45.0, 67.0);
        m.skill("Archery", 36.0, 67.0);
        m.skill("Parry", 45.0, 60.0);
        m.skill("Musicianship", 66.0, 97.5);
        m.skill("Peacemaking", 65.0, 87.5);
        m.fame = 100;
        m.karma = 100;
        if coin() {
            m.female = true;
            m.body = 401;
            m.name = random_name("female");
            switch random_below(2) {
                0 => {
                    m.npc_equip = [#{ item: "Server.Items.Skirt", hue: "randomDyed" }];
                },
                1 => {
                    m.npc_equip = [#{ item: "Server.Items.Kilt", hue: "randomNeutral" }];
                }
            }
        } else {
            m.female = false;
            m.body = 400;
            m.name = random_name("male");
            m.npc_equip = [#{ item: "Server.Items.ShortPants", hue: "randomNeutral" }];
        }
        m.npc_equip = [#{ item: "Server.Items.Shoes", hue: "randomNeutral" }];
        switch random_below(2) {
            0 => {
                m.npc_equip = [#{ item: "Server.Items.Doublet", hue: "randomDyed" }];
            },
            1 => {
                m.npc_equip = [#{ item: "Server.Items.Shirt", hue: "randomDyed" }];
            }
        }
    });
});

mobile("HireBardArcher", |c| {
    c.ai = "Archer";
    c.npc_click_title = false;
    c.on_create(|m| {
        m.npc_hold_gold = 8;
        m.npc_speech_hue = "randomDyed";
        m.npc_hue = "humanSkin";
        m.title = "the bard";
        m.npc_roll_hair = true;
        m.str = 16..=16;
        m.dex = 26..=26;
        m.int = 26..=26;
        m.damage = 5..=10;
        m.skill("Tactics", 35.0, 57.0);
        m.skill("Magery", 22.0, 22.0);
        m.skill("Swords", 45.0, 67.0);
        m.skill("Archery", 36.0, 67.0);
        m.skill("Parry", 45.0, 60.0);
        m.skill("Musicianship", 66.0, 97.5);
        m.skill("Peacemaking", 65.0, 87.5);
        m.fame = 100;
        m.karma = 100;
        if coin() {
            m.female = true;
            m.body = 401;
            m.name = random_name("female");
            switch random_below(2) {
                0 => {
                    m.npc_equip = [#{ item: "Server.Items.Skirt", hue: "randomDyed" }];
                },
                1 => {
                    m.npc_equip = [#{ item: "Server.Items.Kilt", hue: "randomNeutral" }];
                }
            }
        } else {
            m.female = false;
            m.body = 400;
            m.name = random_name("male");
            m.npc_equip = [#{ item: "Server.Items.ShortPants", hue: "randomNeutral" }];
        }
        m.npc_equip = [#{ item: "Server.Items.Shoes", hue: "randomNeutral" }];
        switch random_below(2) {
            0 => {
                m.npc_equip = [#{ item: "Server.Items.Doublet", hue: "randomDyed" }];
            },
            1 => {
                m.npc_equip = [#{ item: "Server.Items.Shirt", hue: "randomDyed" }];
            }
        }
    });
});

mobile("HireBeggar", |c| {
    c.ai = "Melee";
    c.npc_click_title = false;
    c.on_create(|m| {
        m.npc_speech_hue = "randomDyed";
        m.npc_hue = "humanSkin";
        m.title = "the beggar";
        m.npc_roll_hair = true;
        m.str = 26..=26;
        m.dex = 21..=21;
        m.int = 36..=36;
        m.damage = 1..=1;
        m.skill("Begging", 66.0, 97.0);
        m.skill("Tactics", 5.0, 27.0);
        m.skill("Wrestling", 5.0, 27.0);
        m.skill("Magery", 2.0, 2.0);
        m.fame = 0;
        m.karma = 0;
        if coin() {
            m.female = true;
            m.body = 401;
            m.name = random_name("female");
            switch random_below(2) {
                0 => {
                    m.npc_equip = [#{ item: "Server.Items.Skirt", hue: "randomNeutral" }];
                },
                1 => {
                    m.npc_equip = [#{ item: "Server.Items.Kilt", hue: "randomNeutral" }];
                }
            }
        } else {
            m.female = false;
            m.body = 400;
            m.name = random_name("male");
            m.npc_equip = [#{ item: "Server.Items.ShortPants", hue: "randomNeutral" }];
        }
        m.npc_equip = [#{ item: "Server.Items.Sandals", hue: "randomNeutral" }];
        switch random_below(2) {
            0 => {
                m.npc_equip = [#{ item: "Server.Items.Doublet", hue: "randomNeutral" }];
            },
            1 => {
                m.npc_equip = [#{ item: "Server.Items.Shirt", hue: "randomNeutral" }];
            }
        }
        m.pack = [#{ gold: [0, 25] }];
    });
});

mobile("HireFighter", |c| {
    c.ai = "Melee";
    c.npc_click_title = false;
    c.on_create(|m| {
        m.npc_speech_hue = "randomDyed";
        m.npc_hue = "humanSkin";
        m.title = "the fighter";
        m.npc_roll_hair = true;
        m.str = 91..=91;
        m.dex = 91..=91;
        m.int = 50..=50;
        m.damage = 7..=14;
        m.skill("Tactics", 36.0, 67.0);
        m.skill("Magery", 22.0, 22.0);
        m.skill("Swords", 64.0, 100.0);
        m.skill("Parry", 60.0, 82.0);
        m.skill("Macing", 36.0, 67.0);
        m.skill("Focus", 36.0, 67.0);
        m.skill("Wrestling", 25.0, 47.0);
        m.fame = 100;
        m.karma = 100;
        if coin() {
            m.female = true;
            m.body = 401;
            m.name = random_name("female");
        } else {
            m.female = false;
            m.body = 400;
            m.name = random_name("male");
        }
        switch random_below(2) {
            0 => {
                m.npc_equip = [#{ item: "Server.Items.Shoes", hue: "randomNeutral" }];
            },
            1 => {
                m.npc_equip = [#{ item: "Server.Items.Boots", hue: "randomNeutral" }];
            }
        }
        m.npc_equip = ["Server.Items.Shirt"];
        m.npc_seq = [
            block([
                #{ oneOf: [
                    #{ npcEquip: ["Server.Items.Longsword"] },
                    #{ npcEquip: ["Server.Items.Broadsword"] },
                    #{ npcEquip: ["Server.Items.VikingSword"] },
                    #{ npcEquip: ["Server.Items.BattleAxe"] },
                    #{ npcEquip: ["Server.Items.TwoHandedAxe"] },
                ] },
                #{ npcIfLayerEmpty: #{
                    layer: "TwoHanded",
                    then: #{
                        oneOf: [
                            #{ npcEquip: ["Server.Items.BronzeShield"] },
                            #{ npcEquip: ["Server.Items.HeaterShield"] },
                            #{ npcEquip: ["Server.Items.MetalKiteShield"] },
                            #{ npcEquip: ["Server.Items.MetalShield"] },
                            #{ npcEquip: ["Server.Items.WoodenKiteShield"] },
                            #{ npcEquip: ["Server.Items.WoodenShield"] },
                        ],
                    },
                } },
            ]),
            #{
                oneOf: [
                    #{},
                    #{ npcEquip: ["Server.Items.Bascinet"] },
                    #{ npcEquip: ["Server.Items.CloseHelm"] },
                    #{ npcEquip: ["Server.Items.NorseHelm"] },
                    #{ npcEquip: ["Server.Items.Helmet"] },
                ],
            },
            #{
                oneOf: [
                    #{
                        npcEquip: [
                            "Server.Items.LeatherChest",
                            "Server.Items.LeatherArms",
                            "Server.Items.LeatherGloves",
                            "Server.Items.LeatherGorget",
                            "Server.Items.LeatherLegs",
                        ],
                    },
                    #{
                        npcEquip: [
                            "Server.Items.StuddedChest",
                            "Server.Items.StuddedArms",
                            "Server.Items.StuddedGloves",
                            "Server.Items.StuddedGorget",
                            "Server.Items.StuddedLegs",
                        ],
                    },
                    #{
                        npcEquip: [
                            "Server.Items.RingmailChest",
                            "Server.Items.RingmailArms",
                            "Server.Items.RingmailGloves",
                            "Server.Items.RingmailLegs",
                        ],
                    },
                    #{ npcEquip: ["Server.Items.ChainChest", "Server.Items.ChainLegs"] },
                ],
                pack: [#{ gold: [25, 100] }],
            },
        ];
    });
});

mobile("HireMage", |c| {
    c.ai = "Mage";
    c.npc_click_title = false;
    c.on_create(|m| {
        m.npc_hold_gold = 8;
        m.npc_speech_hue = "randomDyed";
        m.npc_hue = "humanSkin";
        m.title = "the mage";
        m.npc_roll_hair = true;
        m.str = 61..=75;
        m.dex = 81..=95;
        m.int = 86..=100;
        m.damage = 10..=23;
        m.skill("EvalInt", 100.0, 125.0);
        m.skill("Magery", 100.0, 125.0);
        m.skill("Meditation", 100.0, 125.0);
        m.skill("MagicResist", 100.0, 125.0);
        m.skill("Tactics", 100.0, 125.0);
        m.skill("Macing", 100.0, 125.0);
        m.fame = 100;
        m.karma = 100;
        if coin() {
            m.female = true;
            m.body = 401;
            m.name = random_name("female");
        } else {
            m.female = false;
            m.body = 400;
            m.name = random_name("male");
            m.npc_equip = [#{ item: "Server.Items.ShortPants", hue: "randomNeutral" }];
        }
        m.npc_equip = ["Server.Items.Shirt", #{ item: "Server.Items.Robe", hue: "randomNeutral" }];
        m.npc_seq = [
            #{
                coin: #{
                    then: #{ npcEquip: [#{ item: "Server.Items.Shoes", hue: "randomNeutral" }] },
                    "else": #{ npcEquip: ["Server.Items.ThighBoots"] },
                },
                pack: [#{ gold: [20, 100] }],
            },
        ];
    });
});

mobile("HirePaladin", |c| {
    c.ai = "Melee";
    c.npc_click_title = false;
    c.on_create(|m| {
        m.npc_speech_hue = "randomDyed";
        m.npc_hue = "humanSkin";
        m.title = "the paladin";
        m.npc_roll_hair = true;
        if era("AOS") {
            m.str = 100..=100;
        } else {
            m.str = 86..=100;
        }
        m.dex = 81..=95;
        m.int = 61..=75;
        m.damage = 10..=23;
        m.skill("Swords", 66.0, 97.5);
        m.skill("Anatomy", 65.0, 87.5);
        m.skill("MagicResist", 25.0, 47.5);
        m.skill("Healing", 65.0, 87.5);
        m.skill("Tactics", 65.0, 87.5);
        m.skill("Wrestling", 15.0, 37.5);
        m.skill("Parry", 45.0, 60.5);
        m.skill("Chivalry", 85.0, 100.0);
        m.fame = 100;
        m.karma = 250;
        if coin() {
            m.female = true;
            m.body = 401;
            m.name = random_name("female");
        } else {
            m.female = false;
            m.body = 400;
            m.name = random_name("male");
        }
        switch random_below(5) {
            0 => {},
            1 => {
                m.npc_equip = ["Server.Items.Bascinet"];
            },
            2 => {
                m.npc_equip = ["Server.Items.CloseHelm"];
            },
            3 => {
                m.npc_equip = ["Server.Items.NorseHelm"];
            },
            4 => {
                m.npc_equip = ["Server.Items.Helmet"];
            }
        }
        m.npc_equip = [
            #{ item: "Server.Items.Shoes", hue: "randomNeutral" },
            "Server.Items.Shirt",
            "Server.Items.VikingSword",
            "Server.Items.MetalKiteShield",
            "Server.Items.PlateChest",
            "Server.Items.PlateLegs",
            "Server.Items.PlateArms",
            "Server.Items.LeatherGorget",
        ];
        m.pack = [#{ gold: [20, 100] }];
    });
});

mobile("HirePeasant", |c| {
    c.ai = "Melee";
    c.npc_click_title = false;
    c.on_create(|m| {
        m.npc_speech_hue = "randomDyed";
        m.npc_hue = "humanSkin";
        m.title = "the peasant";
        m.npc_roll_hair = true;
        m.str = 26..=26;
        m.dex = 21..=21;
        m.int = 16..=16;
        m.damage = 10..=23;
        m.skill("Tactics", 5.0, 27.0);
        m.skill("Wrestling", 5.0, 5.0);
        m.skill("Swords", 5.0, 27.0);
        m.fame = 0;
        m.karma = 0;
        if coin() {
            m.female = true;
            m.body = 401;
            m.name = random_name("female");
            switch random_below(2) {
                0 => {
                    m.npc_equip = [#{ item: "Server.Items.Skirt", hue: "randomNeutral" }];
                },
                1 => {
                    m.npc_equip = [#{ item: "Server.Items.Kilt", hue: "randomNeutral" }];
                }
            }
        } else {
            m.female = false;
            m.body = 400;
            m.name = random_name("male");
            m.npc_equip = [#{ item: "Server.Items.ShortPants", hue: "randomNeutral" }];
        }
        m.npc_equip = ["Server.Items.Katana", #{ item: "Server.Items.Sandals", hue: "randomNeutral" }];
        switch random_below(2) {
            0 => {
                m.npc_equip = [#{ item: "Server.Items.Doublet", hue: "randomNeutral" }];
            },
            1 => {
                m.npc_equip = [#{ item: "Server.Items.Shirt", hue: "randomNeutral" }];
            }
        }
        m.pack = [#{ gold: [0, 25] }];
    });
});

mobile("HireRanger", |c| {
    c.ai = "Melee";
    c.on_create(|m| {
        m.npc_speech_hue = "randomDyed";
        m.npc_hue = "humanSkin";
        m.title = "the ranger";
        m.npc_roll_hair = true;
        m.str = 91..=91;
        m.dex = 76..=76;
        m.int = 61..=61;
        m.damage = 13..=24;
        m.skill("Wrestling", 15.0, 37.0);
        m.skill("Parry", 45.0, 60.0);
        m.skill("Archery", 66.0, 97.0);
        m.skill("Magery", 62.0, 62.0);
        m.skill("Swords", 35.0, 57.0);
        m.skill("Fencing", 15.0, 37.0);
        m.skill("Tactics", 65.0, 87.0);
        m.fame = 100;
        m.karma = 125;
        if coin() {
            m.female = true;
            m.body = 401;
            m.name = random_name("female");
        } else {
            m.female = false;
            m.body = 400;
            m.name = random_name("male");
        }
        m.npc_equip = [#{ item: "Server.Items.Shoes", hue: "randomNeutral" }, "Server.Items.Shirt"];
        switch random_below(3) {
            0 => {
                m.npc_equip = ["Server.Items.Longsword"];
            },
            1 => {
                m.npc_equip = ["Server.Items.VikingSword"];
            },
            2 => {
                m.npc_equip = ["Server.Items.Broadsword"];
            }
        }
        m.npc_seq = [
            #{
                npcEquip: [
                    "Server.Items.StuddedChest",
                    "Server.Items.StuddedArms",
                    "Server.Items.StuddedGloves",
                    "Server.Items.StuddedLegs",
                    "Server.Items.StuddedGorget",
                ],
            },
        ];
    });
});

mobile("HireRangerArcher", |c| {
    c.ai = "Archer";
    c.npc_click_title = false;
    c.on_create(|m| {
        m.npc_hold_gold = 8;
        m.npc_speech_hue = "randomDyed";
        m.npc_hue = "humanSkin";
        m.title = "the ranger";
        m.npc_roll_hair = true;
        m.str = 91..=91;
        m.dex = 76..=76;
        m.int = 61..=61;
        m.damage = 13..=24;
        m.skill("Wrestling", 15.0, 37.0);
        m.skill("Parry", 45.0, 60.0);
        m.skill("Archery", 66.0, 97.0);
        m.skill("Magery", 62.0, 62.0);
        m.skill("Swords", 35.0, 57.0);
        m.skill("Fencing", 15.0, 37.0);
        m.skill("Tactics", 65.0, 87.0);
        m.fame = 100;
        m.karma = 125;
        if coin() {
            m.female = true;
            m.body = 401;
            m.name = random_name("female");
        } else {
            m.female = false;
            m.body = 400;
            m.name = random_name("male");
        }
        m.npc_equip = [#{ item: "Server.Items.Shoes", hue: "randomNeutral" }, "Server.Items.Shirt"];
        switch random_below(2) {
            0 => {
                m.npc_equip = ["Server.Items.Bow"];
            },
            1 => {
                m.npc_equip = ["Server.Items.CompositeBow"];
            }
        }
        m.npc_seq = [
            #{
                npcEquip: [
                    "Server.Items.RangerChest",
                    "Server.Items.RangerArms",
                    "Server.Items.RangerGloves",
                    "Server.Items.RangerGorget",
                    "Server.Items.RangerLegs",
                ],
            },
        ];
    });
});

mobile("HireSailor", |c| {
    c.ai = "Melee";
    c.npc_click_title = false;
    c.on_create(|m| {
        m.npc_speech_hue = "randomDyed";
        m.npc_hue = "humanSkin";
        m.title = "the sailor";
        m.npc_roll_hair = true;
        m.str = 86;
        m.dex = 66;
        m.int = 41;
        m.damage = 10..=23;
        m.skill("Stealing", 66.0, 97.5);
        m.skill("Peacemaking", 65.0, 87.5);
        m.skill("MagicResist", 25.0, 47.5);
        m.skill("Healing", 65.0, 87.5);
        m.skill("Tactics", 65.0, 87.5);
        m.skill("Fencing", 65.0, 87.5);
        m.skill("Parry", 45.0, 60.5);
        m.skill("Lockpicking", 65.0, 87.0);
        m.skill("Hiding", 65.0, 87.0);
        m.skill("Snooping", 65.0, 87.0);
        m.fame = 100;
        m.karma = 0;
        if coin() {
            m.female = true;
            m.body = 401;
            m.name = random_name("female");
            m.npc_equip = [#{ item: "Server.Items.ShortPants", hue: "randomNeutral" }];
        } else {
            m.female = false;
            m.body = 400;
            m.name = random_name("male");
            m.npc_equip = [#{ item: "Server.Items.ShortPants", hue: "randomNeutral" }];
        }
        m.npc_equip = [#{ item: "Server.Items.Shoes", hue: "randomNeutral" }, "Server.Items.Cutlass"];
        switch random_below(2) {
            0 => {
                m.npc_equip = [#{ item: "Server.Items.Doublet", hue: "randomDyed" }];
            },
            1 => {
                m.npc_equip = [#{ item: "Server.Items.Shirt", hue: "randomDyed" }];
            }
        }
    });
});

mobile("HireThief", |c| {
    c.ai = "Melee";
    c.npc_click_title = false;
    c.on_create(|m| {
        m.npc_speech_hue = "randomDyed";
        m.npc_hue = "humanSkin";
        m.title = "the thief";
        m.npc_roll_hair = true;
        m.str = 81..=95;
        m.dex = 86..=100;
        m.int = 61..=75;
        m.damage = 10..=23;
        m.skill("Stealing", 66.0, 97.5);
        m.skill("Peacemaking", 65.0, 87.5);
        m.skill("MagicResist", 25.0, 47.5);
        m.skill("Healing", 65.0, 87.5);
        m.skill("Tactics", 65.0, 87.5);
        m.skill("Fencing", 65.0, 87.5);
        m.skill("Parry", 45.0, 60.5);
        m.skill("Lockpicking", 65.0, 87.0);
        m.skill("Hiding", 65.0, 87.0);
        m.skill("Snooping", 65.0, 87.0);
        m.fame = 100;
        m.karma = 0;
        if coin() {
            m.female = true;
            m.body = 401;
            m.name = random_name("female");
            switch random_below(2) {
                0 => {
                    m.npc_equip = [#{ item: "Server.Items.Skirt", hue: "randomNeutral" }];
                },
                1 => {
                    m.npc_equip = [#{ item: "Server.Items.Kilt", hue: "randomNeutral" }];
                }
            }
        } else {
            m.female = false;
            m.body = 400;
            m.name = random_name("male");
            m.npc_equip = [#{ item: "Server.Items.ShortPants", hue: "randomNeutral" }];
        }
        m.npc_equip = [#{ item: "Server.Items.Sandals", hue: "randomNeutral" }, "Server.Items.Dagger"];
        switch random_below(2) {
            0 => {
                m.npc_equip = [#{ item: "Server.Items.Doublet", hue: "randomNeutral" }];
            },
            1 => {
                m.npc_equip = [#{ item: "Server.Items.Shirt", hue: "randomNeutral" }];
            }
        }
        m.pack = [#{ gold: [0, 25] }];
    });
});

mobile("Actor", |c| {
    c.ai = "Animal";
    c.fight_mode = "None";
    c.npc_click_title = false;
    c.on_create(|m| {
        m.npc_init_stats = [31, 41, 51];
        m.npc_speed = [0.2, 0.4];
        m.npc_speech_hue = "randomDyed";
        m.npc_hue = "humanSkin";
        if coin() {
            m.female = true;
            m.body = 401;
            m.name = random_name("female");
            m.npc_add_item = [#{ item: "Server.Items.FancyDress", hue: "randomDyed" }];
            m.title = "the actress";
        } else {
            m.female = false;
            m.body = 400;
            m.name = random_name("male");
            m.npc_add_item = [
                #{ item: "Server.Items.LongPants", hue: "randomNeutral" },
                #{ item: "Server.Items.FancyShirt", hue: "randomDyed" },
            ];
            m.title = "the actor";
        }
        m.npc_add_item = [#{ item: "Server.Items.Boots", hue: "randomNeutral" }];
        m.random_hair = true;
        m.npc_gold_pack = [250, 300];
    });
});

mobile("Artist", |c| {
    c.ai = "Animal";
    c.fight_mode = "None";
    c.npc_click_title = false;
    c.npc_can_teach = true;
    c.on_create(|m| {
        m.npc_init_stats = [31, 41, 51];
        m.skill("Healing", 36.0, 68.0);
        m.npc_speed = [0.2, 0.4];
        m.npc_speech_hue = "randomDyed";
        m.title = "the artist";
        m.npc_hue = "humanSkin";
        if coin() {
            m.female = true;
            m.body = 401;
            m.name = random_name("female");
        } else {
            m.female = false;
            m.body = 400;
            m.name = random_name("male");
        }
        m.npc_add_item = [
            #{ item: "Server.Items.Doublet", hue: "randomDyed" },
            #{ item: "Server.Items.Sandals", hue: "randomNeutral" },
            #{ item: "Server.Items.ShortPants", hue: "randomNeutral" },
            #{ item: "Server.Items.HalfApron", hue: "randomDyed" },
        ];
        m.random_hair = true;
        m.npc_gold_pack = [250, 300];
    });
});

mobile("Gypsy", |c| {
    c.ai = "Animal";
    c.fight_mode = "None";
    c.npc_click_title = false;
    c.npc_can_teach = true;
    c.on_create(|m| {
        m.npc_init_stats = [31, 41, 51];
        m.skill("Cooking", 65.0, 88.0);
        m.skill("Snooping", 65.0, 88.0);
        m.skill("Stealing", 65.0, 88.0);
        m.npc_speed = [0.2, 0.4];
        m.npc_speech_hue = "randomDyed";
        m.npc_hue = "humanSkin";
        if coin() {
            m.female = true;
            m.body = 401;
            m.name = random_name("female");
            m.npc_add_item = [
                #{ item: "Server.Items.Kilt", hue: "randomDyed" },
                #{ item: "Server.Items.Shirt", hue: "randomDyed" },
                "Server.Items.ThighBoots",
            ];
            m.title = "the gypsy";
        } else {
            m.female = false;
            m.body = 400;
            m.name = random_name("male");
            m.npc_add_item = [
                #{ item: "Server.Items.ShortPants", hue: "randomNeutral" },
                #{ item: "Server.Items.Shirt", hue: "randomDyed" },
                "Server.Items.Sandals",
            ];
            m.title = "the gypsy";
        }
        m.npc_add_item = [#{ item: "Server.Items.Bandana", hue: "randomDyed" }, "Server.Items.Dagger"];
        m.random_hair = true;
        m.npc_gold_pack = [250, 300];
    });
});

mobile("HarborMaster", |c| {
    c.ai = "Animal";
    c.fight_mode = "None";
    c.npc_click_title = false;
    c.npc_can_teach = false;
    c.on_create(|m| {
        m.npc_init_stats = [31, 41, 51];
        m.skill("Mining", 36.0, 68.0);
        m.npc_speed = [0.2, 0.4];
        m.npc_speech_hue = "randomDyed";
        m.npc_hue = "humanSkin";
        m.npc_blessed = true;
        if coin() {
            m.female = true;
            m.body = 401;
            m.name = random_name("female");
            m.title = "the Harbor Mistress";
        } else {
            m.female = false;
            m.body = 400;
            m.name = random_name("male");
            m.title = "the Harbor Master";
        }
        m.npc_add_item = [
            #{ item: "Server.Items.Shirt", hue: "randomDyed" },
            "Server.Items.Boots",
            #{ item: "Server.Items.LongPants", hue: "randomNeutral" },
            "Server.Items.QuarterStaff",
        ];
        m.random_hair = true;
        m.npc_gold_pack = [250, 300];
    });
});

mobile("Ninja", |c| {
    c.ai = "Melee";
    c.fight_mode = "Aggressor";
    c.npc_click_title = false;
    c.npc_can_teach = true;
    c.on_create(|m| {
        m.title = "the ninja";
        m.npc_init_stats = [100, 100, 25];
        m.skill("Fencing", 64.0, 80.0);
        m.skill("Macing", 64.0, 80.0);
        m.skill("Ninjitsu", 60.0, 80.0);
        m.skill("Parry", 64.0, 80.0);
        m.skill("Tactics", 64.0, 85.0);
        m.skill("Swords", 64.0, 85.0);
        m.npc_speed = [0.2, 0.4];
        m.npc_speech_hue = "randomDyed";
        m.npc_hue = "humanSkin";
        if coin() {
            m.female = true;
            m.body = 401;
            m.name = random_name("female");
        } else {
            m.female = false;
            m.body = 400;
            m.name = random_name("male");
            m.npc_add_item = ["Server.Items.LeatherNinjaHood"];
        }
        m.npc_add_item = [
            "Server.Items.LeatherNinjaPants",
            "Server.Items.LeatherNinjaBelt",
            "Server.Items.LeatherNinjaJacket",
            "Server.Items.NinjaTabi",
        ];
        m.npc_hair = block([#{ hue: "randomNondyed" }, #{ facialSkipOneIn: 7 }]);
        m.pack = [#{ gold: [250, 300] }];
    });
});

mobile("Samurai", |c| {
    c.ai = "Melee";
    c.fight_mode = "Aggressor";
    c.npc_click_title = false;
    c.npc_can_teach = true;
    c.on_create(|m| {
        m.title = "the samurai";
        m.npc_init_stats = [100, 100, 25];
        m.skill("ArmsLore", 64.0, 80.0);
        m.skill("Bushido", 64.0, 85.0);
        m.skill("Parry", 64.0, 80.0);
        m.skill("Swords", 64.0, 85.0);
        m.npc_speed = [0.2, 0.4];
        m.npc_speech_hue = "randomDyed";
        m.npc_hue = "humanSkin";
        if coin() {
            m.female = true;
            m.body = 401;
            m.name = random_name("female");
        } else {
            m.female = false;
            m.body = 400;
            m.name = random_name("male");
        }
        switch random_below(3) {
            0 => {
                m.npc_add_item = ["Server.Items.Lajatang"];
            },
            1 => {
                m.npc_add_item = ["Server.Items.Wakizashi"];
            },
            2 => {
                m.npc_add_item = ["Server.Items.NoDachi"];
            }
        }
        m.npc_seq = [
            #{
                oneOf: [
                    #{ npcAddItem: ["Server.Items.LeatherSuneate"] },
                    #{ npcAddItem: ["Server.Items.PlateSuneate"] },
                    #{ npcAddItem: ["Server.Items.StuddedHaidate"] },
                ],
            },
            block([
                #{ oneOf: [
                    #{ npcAddItem: ["Server.Items.LeatherJingasa"] },
                    #{ npcAddItem: ["Server.Items.ChainHatsuburi"] },
                    #{ npcAddItem: ["Server.Items.HeavyPlateJingasa"] },
                    #{ npcAddItem: ["Server.Items.DecorativePlateKabuto"] },
                ] },
                #{ npcAddItem: [
                    "Server.Items.LeatherDo",
                    "Server.Items.LeatherHiroSode",
                    #{ item: "Server.Items.SamuraiTabi", hue: "randomNondyed" },
                ] },
                #{ npcHair: #{ hue: "randomNondyed", facialSkipOneIn: 7 } },
                #{ pack: [#{ gold: [250, 300] }] },
            ]),
        ];
    });
});

mobile("Sculptor", |c| {
    c.ai = "Animal";
    c.fight_mode = "None";
    c.npc_click_title = false;
    c.on_create(|m| {
        m.npc_init_stats = [31, 41, 51];
        m.npc_speed = [0.2, 0.4];
        m.npc_speech_hue = "randomDyed";
        m.title = "the sculptor";
        m.npc_hue = "humanSkin";
        if coin() {
            m.female = true;
            m.body = 401;
            m.name = random_name("female");
            m.npc_add_item = [#{ item: "Server.Items.Kilt", hue: "randomNeutral" }];
        } else {
            m.female = false;
            m.body = 400;
            m.name = random_name("male");
            m.npc_add_item = [#{ item: "Server.Items.LongPants", hue: "randomNeutral" }];
        }
        m.npc_add_item = [#{ item: "Server.Items.Doublet", hue: "randomNeutral" }, "Server.Items.HalfApron"];
        m.random_hair = true;
        m.npc_gold_pack = [250, 300];
    });
});
