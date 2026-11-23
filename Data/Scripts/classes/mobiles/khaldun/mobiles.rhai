mobile("GrimmochDrummel", |c| {
    c.ai = "Archer";
    c.fight_mode = "Closest";
    c.script = "content/engines/grimmoch_drummel.rhai";
    c.name = "Grimmoch Drummel";
    c.always_murderer = true;
    c.on_create(|m| {
        m.title = "the Cursed";
        m.hue = 0x8596;
        m.body = 400;
        m.mlq_hair_item_id = 8266;
        m.quest_equip = [
            #{ item: "Server.Items.Bow", movable: false },
            #{ item: "Server.Items.Boots", hue: 2212 },
            #{ item: "Server.Items.BodySash", hue: 2212 },
            #{ item: "Server.Items.Backpack", movable: false },
            #{ item: "Server.Items.LeatherGloves", hue: 2415 },
            #{ item: "Server.Items.LeatherChest", hue: 2415 },
        ];
        m.str = 111..=120;
        m.dex = 151..=160;
        m.int = 41..=50;
        m.hits = 180..=207;
        m.mana = 0;
        m.damage = 13..=16;
        m.resist("Physical", 35, 45);
        m.resist("Fire", 25, 30);
        m.resist("Cold", 45, 55);
        m.resist("Poison", 30, 40);
        m.resist("Energy", 20, 25);
        m.skill("Archery", 90.1, 110.0);
        m.skill("Swords", 60.1, 70.0);
        m.skill("Tactics", 90.1, 100.0);
        m.skill("MagicResist", 60.1, 70.0);
        m.skill("Anatomy", 90.1, 100.0);
        m.fame = 5000;
        m.karma = -1000;
    });
});

mobile("LysanderGathenwale", |c| {
    c.ai = "Mage";
    c.fight_mode = "Closest";
    c.script = "content/engines/lysander_gathenwale.rhai";
    c.name = "Lysander Gatherwale";
    c.always_murderer = true;
    c.on_create(|m| {
        m.title = "the Cursed";
        m.hue = 0x8838;
        m.body = 400;
        m.quest_equip = [
            #{ item: "Server.Items.Boots", hue: 1433 },
            #{ item: "Server.Items.Cloak", hue: 2415 },
            #{ item: "Server.Items.Spellbook", hue: 1433 },
            #{ item: "Server.Items.RingmailGloves", hue: 1433 },
            #{ item: "Server.Items.StuddedChest", hue: 2415 },
            #{ item: "Server.Items.PlateArms", hue: 1433 },
        ];
        m.str = 111..=120;
        m.dex = 71..=80;
        m.int = 121..=130;
        m.hits = 180..=207;
        m.mana = 227..=265;
        m.damage = 5..=13;
        m.resist("Physical", 35, 45);
        m.resist("Fire", 25, 30);
        m.resist("Cold", 50, 60);
        m.resist("Poison", 25, 35);
        m.resist("Energy", 25, 35);
        m.skill("Wrestling", 80.1, 90.0);
        m.skill("Tactics", 90.1, 100.0);
        m.skill("MagicResist", 80.1, 90.0);
        m.skill("Magery", 90.1, 100.0);
        m.skill("EvalInt", 95.1, 100.0);
        m.skill("Meditation", 90.1, 100.0);
        m.fame = 5000;
        m.karma = -10000;
    });
});

mobile("MorgBergen", |c| {
    c.ai = "Melee";
    c.fight_mode = "Closest";
    c.name = "Morg Bergen";
    c.always_murderer = true;
    c.on_create(|m| {
        m.title = "the Cursed";
        m.hue = 0x8596;
        m.body = 400;
        m.quest_equip = [
            #{ item: "Server.Items.ShortPants", hue: 1436 },
            #{ item: "Server.Items.Bardiche", hue: 2415, movable: false },
            #{ item: "Server.Items.LeatherGloves", hue: 2415 },
            #{ item: "Server.Items.LeatherArms", hue: 2415 },
        ];
        m.str = 111..=120;
        m.dex = 111..=120;
        m.int = 51..=60;
        m.hits = 180..=207;
        m.mana = 0;
        m.damage = 9..=17;
        m.damage_type("Physical", 40);
        m.damage_type("Cold", 60);
        m.resist("Physical", 35, 45);
        m.resist("Fire", 25, 30);
        m.resist("Cold", 50, 60);
        m.resist("Poison", 25, 35);
        m.resist("Energy", 25, 35);
        m.skill("Swords", 90.1, 100.0);
        m.skill("Tactics", 90.1, 100.0);
        m.skill("MagicResist", 80.1, 90.0);
        m.skill("Anatomy", 90.1, 100.0);
        m.fame = 5000;
        m.karma = -1000;
    });
});

mobile("TavaraSewel", |c| {
    c.ai = "Melee";
    c.fight_mode = "Closest";
    c.name = "Tavara Sewel";
    c.always_murderer = true;
    c.on_create(|m| {
        m.title = "the Cursed";
        m.hue = 0x8838;
        m.female = true;
        m.body = 401;
        m.quest_equip = [
            #{ item: "Server.Items.Kilt", hue: 1436 },
            #{ item: "Server.Items.Sandals", hue: 1433 },
            #{ item: "Server.Items.Kryss", hue: 2415, movable: false },
            #{ item: "Server.Items.Buckler", hue: 2415, movable: false },
            #{ item: "Server.Items.RingmailGloves", hue: 1433 },
            #{ item: "Server.Items.FemalePlateChest", hue: 2415 },
        ];
        m.str = 111..=120;
        m.dex = 111..=120;
        m.int = 111..=120;
        m.hits = 180..=207;
        m.stam = 126..=150;
        m.mana = 0;
        m.damage = 13..=16;
        m.resist("Physical", 25, 30);
        m.resist("Fire", 25, 30);
        m.resist("Cold", 50, 60);
        m.resist("Poison", 25, 35);
        m.resist("Energy", 25, 35);
        m.skill("Fencing", 90.1, 100.0);
        m.skill("Tactics", 90.1, 100.0);
        m.skill("MagicResist", 80.1, 90.0);
        m.skill("Anatomy", 90.1, 100.0);
        m.fame = 5000;
        m.karma = -1000;
    });
});
