mobile("Server.Mobiles.BaseHealer", |c| {
    c.npc_shoe_type = "Sandals";
    c.on_create(|m| {
        m.speech_hue = 0;
        m.str = 304..=400;
        m.dex = 102..=150;
        m.int = 204..=300;
        m.damage = 10..=23;
        m.damage_type("Physical", 100);
        m.resist("Physical", 40, 50);
        m.resist("Fire", 40, 50);
        m.resist("Cold", 40, 50);
        m.resist("Poison", 40, 50);
        m.resist("Energy", 40, 50);
        m.skill("Anatomy", 75.0, 97.5);
        m.skill("EvalInt", 82.0, 100.0);
        m.skill("Healing", 75.0, 97.5);
        m.skill("Magery", 82.0, 100.0);
        m.skill("MagicResist", 82.0, 100.0);
        m.skill("Tactics", 82.0, 100.0);
        m.fame = 1000;
        m.karma = 10000;
        m.pack = [
            #{ item: "Server.Items.Bandage", amount: [5, 10] },
            #{ item: "Server.Items.HealPotion" },
            #{ item: "Server.Items.CurePotion" },
        ];
    });
});

mobile("Healer", |c| {
    c.inherits = "Server.Mobiles.BaseHealer";
    c.npc_sb_infos = ["SBHealer"];
    c.on_create(|m| {
        m.title = "the healer";
        if era("AOS") {} else {
            m.npc_name_hue = 53;
        }
        m.skill("Forensics", 80.0, 100.0);
        m.skill("SpiritSpeak", 80.0, 100.0);
        m.skill("Swords", 80.0, 100.0);
    });
});

mobile("EvilHealer", |c| {
    c.inherits = "Server.Mobiles.BaseHealer";
    c.npc_sb_infos = ["SBHealer"];
    c.on_create(|m| {
        m.title = "the healer";
        m.karma = -10000;
        m.skill("Forensics", 80.0, 100.0);
        m.skill("SpiritSpeak", 80.0, 100.0);
        m.skill("Swords", 80.0, 100.0);
    });
});

mobile("EvilWanderingHealer", |c| {
    c.inherits = "Server.Mobiles.BaseHealer";
    c.on_create(|m| {
        if era("AOS") {
            m.title = "the priest Of Mondain";
        } else {
            m.title = "the evil wandering healer";
        }
        m.karma = -10000;
        m.npc_add_item = ["Server.Items.GnarledStaff"];
        m.skill("Camping", 80.0, 100.0);
        m.skill("Forensics", 80.0, 100.0);
        m.skill("SpiritSpeak", 80.0, 100.0);
    });
});

mobile("FortuneTeller", |c| {
    c.inherits = "Server.Mobiles.BaseHealer";
    c.npc_sb_infos = ["SBMage", "SBFortuneTeller"];
    c.on_create(|m| {
        m.title = "the fortune teller";
        m.skill("Anatomy", 85.0, 100.0);
        m.skill("Healing", 90.0, 100.0);
        m.skill("Forensics", 75.0, 98.0);
        m.skill("SpiritSpeak", 65.0, 88.0);
    });
    c.on_outfit(|m| {
        m.npc_vendor_outfit = true;
        m.npc_add_item = [#{ item: "Server.Items.Robe", hue: "randomBright" }];
        switch random_below(3) {
            0 => {
                m.npc_add_item = [#{ item: "Server.Items.SkullCap", hue: "randomBright" }];
            },
            1 => {
                m.npc_add_item = [#{ item: "Server.Items.WizardsHat", hue: "randomBright" }];
            },
            2 => {
                m.npc_add_item = [#{ item: "Server.Items.Bandana", hue: "randomBright" }];
            }
        }
        m.npc_seq = [#{ npcAddItem: ["Server.Items.Spellbook"] }];
    });
});

mobile("WanderingHealer", |c| {
    c.inherits = "Server.Mobiles.BaseHealer";
    c.on_create(|m| {
        m.title = "the wandering healer";
        m.npc_add_item = ["Server.Items.GnarledStaff"];
        m.skill("Camping", 80.0, 100.0);
        m.skill("Forensics", 80.0, 100.0);
        m.skill("SpiritSpeak", 80.0, 100.0);
    });
});

mobile("PricedHealer", |c| {
    c.inherits = "Server.Mobiles.BaseHealer";
    c.on_create(|m| {
        if era("AOS") {} else {
            m.npc_name_hue = 53;
        }
    });
});
