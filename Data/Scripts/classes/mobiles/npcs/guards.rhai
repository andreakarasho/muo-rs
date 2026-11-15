mobile("WarriorGuard", |c| {
    c.on_create(|m| {
        m.title = "the guard";
        m.npc_init_stats = [100, 125, 25];
        m.npc_speech_hue = "randomDyed";
        m.npc_hue = "humanSkin";
        if coin() {
            m.female = true;
            m.body = 401;
            m.name = random_name("female");
            if coin() {
                m.npc_add_item = ["Server.Items.LeatherSkirt"];
            } else {
                m.npc_add_item = ["Server.Items.LeatherShorts"];
            }
            switch random_below(5) {
                0 => {
                    m.npc_add_item = ["Server.Items.FemaleLeatherChest"];
                },
                1 => {
                    m.npc_add_item = ["Server.Items.FemaleStuddedChest"];
                },
                2 => {
                    m.npc_add_item = ["Server.Items.LeatherBustierArms"];
                },
                3 => {
                    m.npc_add_item = ["Server.Items.StuddedBustierArms"];
                },
                4 => {
                    m.npc_add_item = ["Server.Items.FemalePlateChest"];
                }
            }
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
            block([
                #{ npcGuardWeapon: "Server.Items.Halberd" },
                #{ npcUnmovablePack: [#{ gold: [10, 25] }] },
                #{ skillBase: #{ Anatomy: 120.0, Tactics: 120.0, Swords: 120.0, MagicResist: 120.0, DetectHidden: 100.0 } },
                #{ npcCombatDelay: 500 },
            ]),
        ];
    });
});

mobile("ArcherGuard", |c| {
    c.on_create(|m| {
        m.title = "the guard";
        m.npc_init_stats = [100, 125, 25];
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
        m.npc_mount = "Server.Mobiles.Horse";
        m.npc_add_item = [
            "Server.Items.StuddedChest",
            "Server.Items.StuddedArms",
            "Server.Items.StuddedGloves",
            "Server.Items.StuddedGorget",
            "Server.Items.StuddedLegs",
            "Server.Items.Boots",
            "Server.Items.SkullCap",
        ];
        m.npc_guard_weapon = "Server.Items.Bow";
        m.npc_unmovable_pack = [#{ item: "Server.Items.Arrow", amount: 250, newbied: true }, #{ gold: [10, 25] }];
        m.skill_base("Anatomy", 120.0);
        m.skill_base("Tactics", 120.0);
        m.skill_base("Archery", 120.0);
        m.skill_base("MagicResist", 120.0);
        m.skill_base("DetectHidden", 100.0);
        m.npc_combat_delay = 500;
    });
});
