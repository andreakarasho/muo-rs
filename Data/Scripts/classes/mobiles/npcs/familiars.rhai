mobile("Server.Mobiles.BaseFamiliar", |c| {
    c.ai = "Melee";
    c.fight_mode = "Closest";
    c.range_fight = 1;
    c.on_create(|m| {
        m.npc_speed = [0.1, 0.1];
    });
});

mobile("DarkWolfFamiliar", |c| {
    c.inherits = "Server.Mobiles.BaseFamiliar";
    c.on_create(|m| {
        m.body = 99;
        m.hue = 0x901;
        m.sound = 0xE5;
        m.str = 100;
        m.dex = 90;
        m.int = 90;
        m.hits = 60;
        m.stam = 90;
        m.mana = 0;
        m.damage = 5..=10;
        m.damage_type("Physical", 100);
        m.resist("Physical", 40, 50);
        m.resist("Fire", 25, 40);
        m.resist("Cold", 25, 40);
        m.resist("Poison", 25, 40);
        m.resist("Energy", 25, 40);
        m.skill("Wrestling", 85.1, 90.0);
        m.skill("Tactics", 50.0);
        m.control_slots = 1;
    });
});

mobile("DeathAdder", |c| {
    c.inherits = "Server.Mobiles.BaseFamiliar";
    c.on_create(|m| {
        m.body = 21;
        m.hue = 0x455;
        m.sound = 0xDB;
        m.str = 70;
        m.dex = 150;
        m.int = 100;
        m.hits = 50;
        m.stam = 150;
        m.mana = 0;
        m.damage = 1..=4;
        m.damage_type("Physical", 100);
        m.resist("Physical", 10);
        m.resist("Poison", 100);
        m.skill("Wrestling", 90.0);
        m.skill("Tactics", 50.0);
        m.skill("MagicResist", 100.0);
        m.skill("Poisoning", 150.0);
        m.control_slots = 1;
    });
});

mobile("ShadowWispFamiliar", |c| {
    c.inherits = "Server.Mobiles.BaseFamiliar";
    c.on_create(|m| {
        m.body = 165;
        m.hue = 0x901;
        m.sound = 0x1D2;
        m.str = 50;
        m.dex = 60;
        m.int = 100;
        m.hits = 50;
        m.stam = 60;
        m.mana = 0;
        m.damage = 5..=10;
        m.damage_type("Energy", 100);
        m.resist("Physical", 10, 15);
        m.resist("Fire", 10, 15);
        m.resist("Cold", 10, 15);
        m.resist("Poison", 10, 15);
        m.resist("Energy", 99);
        m.skill("Wrestling", 40.0);
        m.skill("Tactics", 40.0);
        m.control_slots = 1;
    });
});

mobile("VampireBatFamiliar", |c| {
    c.inherits = "Server.Mobiles.BaseFamiliar";
    c.on_create(|m| {
        m.body = 317;
        m.sound = 0x270;
        m.str = 120;
        m.dex = 120;
        m.int = 100;
        m.hits = 90;
        m.stam = 120;
        m.mana = 0;
        m.damage = 5..=10;
        m.damage_type("Physical", 100);
        m.resist("Physical", 10, 15);
        m.resist("Fire", 10, 15);
        m.resist("Cold", 10, 15);
        m.resist("Poison", 10, 15);
        m.resist("Energy", 10, 15);
        m.skill("Wrestling", 95.1, 100.0);
        m.skill("Tactics", 50.0);
        m.control_slots = 1;
    });
});

mobile("HordeMinionFamiliar", |c| {
    c.inherits = "Server.Mobiles.BaseFamiliar";
    c.on_create(|m| {
        m.body = 776;
        m.sound = 0x39D;
        m.str = 100;
        m.dex = 110;
        m.int = 100;
        m.hits = 70;
        m.stam = 110;
        m.mana = 0;
        m.damage = 5..=10;
        m.damage_type("Physical", 100);
        m.resist("Physical", 50, 60);
        m.resist("Fire", 50, 55);
        m.resist("Poison", 25, 30);
        m.resist("Energy", 25, 30);
        m.skill("Wrestling", 70.1, 75.0);
        m.skill("Tactics", 50.0);
        m.control_slots = 1;
        m.npc_replace_backpack = #{ weight: 13.0 };
    });
});
