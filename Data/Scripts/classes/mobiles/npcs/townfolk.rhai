mobile("BaseEscortable", |c| {
    c.ai = "Melee";
    c.fight_mode = "Aggressor";
    c.range_perception = 22;
    c.range_fight = 1;
    c.on_create(|m| {
        m.str = 90..=100;
        m.dex = 90..=100;
        m.int = 15..=25;
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
        m.npc_escort_outfit = true;
        m.fame = 200;
        m.karma = 4000;
        m.npc_speed = [0.2, 1.0];
    });
    c.on_outfit(|m| {
        m.npc_add_item = [
            #{ item: "Server.Items.FancyShirt", hue: "randomNeutral" },
            #{ item: "Server.Items.ShortPants", hue: "randomNeutral" },
            #{ item: "Server.Items.Boots", hue: "randomNeutral" },
        ];
        m.random_hair = true;
        m.pack = [#{ gold: [200, 250] }];
    });
});

mobile("BrideGroom", |c| {
    c.inherits = "BaseEscortable";
    c.npc_click_title = false;
    c.npc_can_teach = true;
    c.on_create(|m| {
        m.npc_title_by_gender = #{ female: "the bride", male: "the groom" };
    });
    c.on_outfit(|m| {
        m.npc_by_gender = #{
            female: #{
                npcSeq: [
                    #{
                        npcAddItem: ["Server.Items.FancyDress"],
                        npcHueVars: #{ low: #{ oneOf: [0, "randomBlue", "randomGreen", "randomRed", "randomYellow", "randomNeutral"] } },
                    },
                    #{
                        npcAddItem: [#{ item: "Server.Items.LongPants", hue: "$low" }, #{ item: "Server.Items.Shoes", hue: "$low" }],
                    },
                ],
            },
            male: #{
                npcSeq: [
                    #{
                        npcAddItem: ["Server.Items.FancyShirt"],
                        npcHueVars: #{ low: #{ oneOf: [0, "randomBlue", "randomGreen", "randomRed", "randomYellow", "randomNeutral"] } },
                    },
                    #{
                        npcAddItem: [#{ item: "Server.Items.LongPants", hue: "$low" }, #{ item: "Server.Items.Boots", hue: "$low" }],
                    },
                ],
            },
        };
        if coin() {
            m.npc_hair_item_id = 8251;
        } else {
            m.npc_hair_item_id = 8252;
        }
        m.npc_hair_hue = "humanHair";
        m.pack = [#{ gold: [200, 250] }];
    });
});

mobile("EscortableMage", |c| {
    c.inherits = "BaseEscortable";
    c.npc_click_title = false;
    c.npc_can_teach = true;
    c.on_create(|m| {
        m.title = "the mage";
        m.skill("EvalInt", 80.0, 100.0);
        m.skill("Inscribe", 80.0, 100.0);
        m.skill("Magery", 80.0, 100.0);
        m.skill("Meditation", 80.0, 100.0);
        m.skill("MagicResist", 80.0, 100.0);
    });
    c.on_outfit(|m| {
        m.npc_add_item = [
            #{
                item: "Server.Items.Robe",
                hue: #{ oneOf: ["randomBlue", "randomGreen", "randomRed", "randomYellow", "randomNeutral"] },
            },
        ];
        m.npc_hue_vars = #{ low: #{ oneOf: ["randomBlue", "randomGreen", "randomRed", "randomYellow", "randomNeutral"] } };
        m.npc_by_gender = #{
            female: #{
                npcAddItem: [
                    #{ item: "Server.Items.ShortPants", hue: "$low" },
                    #{ item: "Server.Items.ThighBoots", hue: "$low" },
                ],
            },
            male: #{
                npcAddItem: [#{ item: "Server.Items.ShortPants", hue: "$low" }, #{ item: "Server.Items.Boots", hue: "$low" }],
            },
        };
        m.random_hair = true;
        m.pack = [#{ gold: [200, 250] }];
    });
});

mobile("Merchant", |c| {
    c.inherits = "BaseEscortable";
    c.npc_click_title = false;
    c.npc_can_teach = true;
    c.on_create(|m| {
        m.title = "the merchant";
        m.skill("ItemID", 55.0, 78.0);
        m.skill("ArmsLore", 55.0, 78.0);
    });
    c.on_outfit(|m| {
        m.npc_hue_vars = #{ low: #{ oneOf: [0, "randomBlue", "randomGreen", "randomRed", "randomYellow", "randomNeutral"] } };
        m.npc_by_gender = #{
            female: #{
                npcAddItem: [
                    "Server.Items.ThighBoots",
                    #{ item: "Server.Items.FancyDress", hue: "$low" },
                    #{ item: "Server.Items.LongPants", hue: "$low" },
                ],
            },
            male: #{
                npcAddItem: [
                    "Server.Items.ThighBoots",
                    #{ item: "Server.Items.FancyShirt", hue: "$low" },
                    #{ item: "Server.Items.LongPants", hue: "$low" },
                    #{ item: "Server.Items.BodySash", hue: "$low" },
                ],
            },
        };
        m.random_hair = true;
        m.pack = [#{ gold: [200, 250] }];
    });
});

mobile("Messenger", |c| {
    c.inherits = "BaseEscortable";
    c.npc_click_title = false;
    c.npc_can_teach = true;
    c.on_create(|m| {
        m.title = "the messenger";
    });
    c.on_outfit(|m| {
        m.npc_by_gender = #{
            female: #{
                npcSeq: [
                    #{
                        npcAddItem: ["Server.Items.PlainDress"],
                        npcHueVars: #{ low: #{ oneOf: [0, "randomBlue", "randomGreen", "randomRed", "randomYellow", "randomNeutral"] } },
                    },
                    #{
                        npcAddItem: [#{ item: "Server.Items.ShortPants", hue: "$low" }, #{ item: "Server.Items.Boots", hue: "$low" }],
                    },
                ],
            },
            male: #{
                npcSeq: [
                    #{
                        npcAddItem: [
                            #{
                                item: "Server.Items.Shirt",
                                hue: #{ oneOf: [0, "randomBlue", "randomGreen", "randomRed", "randomYellow", "randomNeutral"] },
                            },
                        ],
                        npcHueVars: #{ low: #{ oneOf: [0, "randomBlue", "randomGreen", "randomRed", "randomYellow", "randomNeutral"] } },
                    },
                    #{
                        npcAddItem: [#{ item: "Server.Items.ShortPants", hue: "$low" }, #{ item: "Server.Items.Shoes", hue: "$low" }],
                    },
                ],
            },
        };
        switch random_below(4) {
            0 => {
                m.npc_hair_item_id = 8264;
            },
            1 => {
                m.npc_hair_item_id = 8265;
            },
            2 => {
                m.npc_hair_item_id = 8266;
            },
            3 => {
                m.npc_hair_item_id = 8267;
            }
        }
        m.npc_hair_hue = "humanHair";
        m.pack = [#{ gold: [200, 250] }];
    });
});

mobile("Noble", |c| {
    c.inherits = "BaseEscortable";
    c.npc_click_title = false;
    c.npc_can_teach = true;
    c.on_create(|m| {
        m.title = "the noble";
        m.skill("Parry", 80.0, 100.0);
        m.skill("Swords", 80.0, 100.0);
        m.skill("Tactics", 80.0, 100.0);
    });
    c.on_outfit(|m| {
        m.npc_by_gender = #{
            female: #{
                npcSeq: [
                    #{
                        npcAddItem: ["Server.Items.FancyDress"],
                        npcHueVars: #{ low: #{ oneOf: [0, "randomBlue", "randomGreen", "randomRed", "randomYellow", "randomNeutral"] } },
                    },
                    #{
                        npcAddItem: [
                            #{ item: "Server.Items.ShortPants", hue: "$low" },
                            #{ item: "Server.Items.ThighBoots", hue: "$low" },
                            #{
                                item: "Server.Items.Cloak",
                                hue: #{ oneOf: [0, "randomBlue", "randomGreen", "randomRed", "randomYellow", "randomNeutral"] },
                            },
                        ],
                    },
                ],
            },
            male: #{
                npcSeq: [
                    #{
                        npcAddItem: [
                            #{
                                item: "Server.Items.FancyShirt",
                                hue: #{ oneOf: [0, "randomBlue", "randomGreen", "randomRed", "randomYellow", "randomNeutral"] },
                            },
                        ],
                        npcHueVars: #{ low: #{ oneOf: [0, "randomBlue", "randomGreen", "randomRed", "randomYellow", "randomNeutral"] } },
                    },
                    #{
                        npcAddItem: [
                            #{ item: "Server.Items.ShortPants", hue: "$low" },
                            #{ item: "Server.Items.Boots", hue: "$low" },
                            #{ item: "Server.Items.BodySash", hue: "$low" },
                            #{
                                item: "Server.Items.Cloak",
                                hue: #{ oneOf: [0, "randomBlue", "randomGreen", "randomRed", "randomYellow", "randomNeutral"] },
                            },
                            "Server.Items.Longsword",
                        ],
                    },
                ],
            },
        };
        m.random_hair = true;
        m.pack = [#{ gold: [200, 250] }];
    });
});

mobile("Peasant", |c| {
    c.inherits = "BaseEscortable";
    c.npc_click_title = false;
    c.npc_can_teach = true;
    c.on_create(|m| {
        m.title = "the peasant";
    });
    c.on_outfit(|m| {
        m.npc_by_gender = #{
            female: #{
                npcSeq: [
                    #{
                        npcAddItem: ["Server.Items.PlainDress"],
                        npcHueVars: #{ low: #{ oneOf: [0, "randomBlue", "randomGreen", "randomRed", "randomYellow", "randomNeutral"] } },
                    },
                    #{
                        npcAddItem: [#{ item: "Server.Items.ShortPants", hue: "$low" }, #{ item: "Server.Items.Boots", hue: "$low" }],
                    },
                ],
            },
            male: #{
                npcSeq: [
                    #{
                        npcAddItem: [
                            #{
                                item: "Server.Items.Shirt",
                                hue: #{ oneOf: [0, "randomBlue", "randomGreen", "randomRed", "randomYellow", "randomNeutral"] },
                            },
                        ],
                        npcHueVars: #{ low: #{ oneOf: [0, "randomBlue", "randomGreen", "randomRed", "randomYellow", "randomNeutral"] } },
                    },
                    #{
                        npcAddItem: [#{ item: "Server.Items.ShortPants", hue: "$low" }, #{ item: "Server.Items.Shoes", hue: "$low" }],
                    },
                ],
            },
        };
        m.random_hair = true;
        m.pack = [#{ gold: [200, 250] }];
    });
});

mobile("Prisoner", |c| {
    c.inherits = "BaseEscortable";
    c.npc_click_title = false;
    c.npc_can_teach = true;
    c.on_create(|m| {
        m.npc_title_by_gender = #{ female: "the noblewoman", male: "the nobleman" };
        m.cant_walk = true;
        m.npc_prisoner = true;
    });
    c.on_outfit(|m| {
        m.npc_by_gender = #{
            female: #{ npcAddItem: [#{ item: "Server.Items.FancyDress", hue: "randomNondyed" }] },
            male: #{
                npcAddItem: [
                    #{ item: "Server.Items.FancyShirt", hue: "randomNondyed" },
                    #{ item: "Server.Items.LongPants", hue: "randomNondyed" },
                ],
            },
        };
        if coin() {
            m.npc_add_item = ["Server.Items.Boots"];
        } else {
            m.npc_add_item = ["Server.Items.ThighBoots"];
        }
        m.random_hair = true;
        m.npc_facial_hair = "hairHue";
    });
});

mobile("SeekerOfAdventure", |c| {
    c.inherits = "BaseEscortable";
    c.npc_click_title = false;
    c.on_create(|m| {
        m.title = "the seeker of adventure";
    });
    c.on_outfit(|m| {
        m.npc_by_gender = #{
            female: #{
                npcSeq: [
                    #{
                        npcAddItem: [
                            #{
                                item: "Server.Items.FancyDress",
                                hue: #{ oneOf: [0, "randomBlue", "randomGreen", "randomRed", "randomYellow", "randomNeutral"] },
                            },
                        ],
                        npcHueVars: #{ low: #{ oneOf: [0, "randomBlue", "randomGreen", "randomRed", "randomYellow", "randomNeutral"] } },
                    },
                    #{
                        npcAddItem: [
                            #{ item: "Server.Items.ShortPants", hue: "$low" },
                            #{ item: "Server.Items.ThighBoots", hue: "$low" },
                            #{
                                item: "Server.Items.Cloak",
                                hue: #{ oneOf: [0, "randomBlue", "randomGreen", "randomRed", "randomYellow", "randomNeutral"] },
                            },
                            "Server.Items.Longsword",
                        ],
                    },
                ],
            },
            male: #{
                npcSeq: [
                    #{
                        npcAddItem: [
                            #{
                                item: "Server.Items.FancyShirt",
                                hue: #{ oneOf: [0, "randomBlue", "randomGreen", "randomRed", "randomYellow", "randomNeutral"] },
                            },
                        ],
                        npcHueVars: #{ low: #{ oneOf: [0, "randomBlue", "randomGreen", "randomRed", "randomYellow", "randomNeutral"] } },
                    },
                    #{
                        npcAddItem: [
                            #{ item: "Server.Items.ShortPants", hue: "$low" },
                            #{ item: "Server.Items.Boots", hue: "$low" },
                            #{ item: "Server.Items.BodySash", hue: "$low" },
                            #{
                                item: "Server.Items.Cloak",
                                hue: #{ oneOf: [0, "randomBlue", "randomGreen", "randomRed", "randomYellow", "randomNeutral"] },
                            },
                            "Server.Items.Longsword",
                        ],
                    },
                ],
            },
        };
        m.random_hair = true;
        m.pack = [#{ gold: [100, 150] }];
    });
});

mobile("TownCrier", |c| {
    c.on_create(|m| {
        m.npc_init_stats = [100, 100, 25];
        m.title = "the town crier";
        m.npc_hue = "humanSkin";
        if era("AOS") {} else {
            m.npc_name_hue = 53;
        }
        if coin() {
            m.female = true;
            m.body = 401;
            m.name = random_name("female");
        } else {
            m.female = false;
            m.body = 400;
            m.name = random_name("male");
        }
        m.npc_seq = [
            #{ npcAddItem: [#{ item: "Server.Items.FancyShirt", hue: "randomBlue" }] },
            #{
                oneOf: [
                    #{ npcAddItem: [#{ item: "Server.Items.Skirt", hue: "randomGreen" }] },
                    #{ npcAddItem: [#{ item: "Server.Items.Kilt", hue: "randomGreen" }] },
                ],
            },
            #{
                npcAddItem: [#{ item: "Server.Items.FeatheredHat", hue: "randomGreen" }],
                oneOf: [#{ npcAddItem: ["Server.Items.Boots"] }, #{ npcAddItem: ["Server.Items.ThighBoots"] }],
            },
            #{ randomHair: true },
        ];
    });
});
