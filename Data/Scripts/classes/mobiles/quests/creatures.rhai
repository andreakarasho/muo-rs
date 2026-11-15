mobile("DeadlyImp", |c| {
    c.ai = "Melee";
    c.fight_mode = "Aggressor";
    c.name = "a deadly imp";
    c.on_create(|m| {
        m.body = 74;
        m.sound = 0x1A6;
        m.hue = 0x66A;
        m.str = 91..=115;
        m.dex = 61..=80;
        m.int = 86..=105;
        m.hits = 1000;
        m.damage = 50..=80;
        m.damage_type("Fire", 100);
        m.resist("Physical", 95, 98);
        m.resist("Fire", 95, 98);
        m.resist("Cold", 95, 98);
        m.resist("Poison", 95, 98);
        m.resist("Energy", 95, 98);
        m.skill("Magery", 120.0);
        m.skill("Tactics", 120.0);
        m.skill("Wrestling", 120.0);
        m.fame = 2500;
        m.karma = -2500;
        m.cant_walk = true;
    });
});

mobile("FierceDragon", |c| {
    c.ai = "Melee";
    c.fight_mode = "Aggressor";
    c.name = "a fierce dragon";
    c.on_create(|m| {
        m.body = 103;
        m.sound = 0x16A;
        m.str = 6000..=6020;
        m.dex = 0;
        m.int = 850..=870;
        m.damage = 50..=80;
        m.damage_type("Fire", 100);
        m.resist("Physical", 95, 98);
        m.resist("Fire", 95, 98);
        m.resist("Cold", 95, 98);
        m.resist("Poison", 95, 98);
        m.resist("Energy", 95, 98);
        m.skill("Tactics", 120.0);
        m.skill("Wrestling", 120.0);
        m.skill("Magery", 120.0);
        m.fame = 15000;
        m.karma = 15000;
        m.cant_walk = true;
    });
});

mobile("CursedSoul", |c| {
    c.ai = "Melee";
    c.fight_mode = "Aggressor";
    c.corpse_name = "a cursed soul corpse";
    c.name = "a cursed soul";
    c.on_create(|m| {
        m.body = 3;
        m.sound = 0x1D7;
        m.str = 20..=40;
        m.dex = 40..=60;
        m.int = 15..=25;
        m.hits = 10..=20;
        m.damage = 3..=7;
        m.damage_type("Physical", 100);
        m.resist("Physical", 15, 20);
        m.resist("Fire", 8, 12);
        m.skill("Wrestling", 35.0, 39.0);
        m.skill("Tactics", 5.0, 15.0);
        m.skill("MagicResist", 10.0);
        m.fame = 200;
        m.karma = -200;
        switch random_below(10) {
            0 => {
                m.pack = [#{ item: "Server.Items.LeftArm" }];
            },
            1 => {
                m.pack = [#{ item: "Server.Items.RightArm" }];
            },
            2 => {
                m.pack = [#{ item: "Server.Items.Torso" }];
            },
            3 => {
                m.pack = [#{ item: "Server.Items.Bone" }];
            },
            4 => {
                m.pack = [#{ item: "Server.Items.RibCage" }];
            },
            5 => {
                m.pack = [#{ item: "Server.Items.RibCage" }];
            },
            6 => {
                m.pack = [#{ item: "Server.Items.BonePile" }];
            },
            7 => {
                m.pack = [#{ item: "Server.Items.BonePile" }];
            },
            8 => {
                m.pack = [#{ item: "Server.Items.BonePile" }];
            },
            9 => {
                m.pack = [#{ item: "Server.Items.BonePile" }];
            }
        }
    });
});

mobile("DiseasedCat", |c| {
    c.ai = "Animal";
    c.fight_mode = "Aggressor";
    c.name = "a diseased cat";
    c.always_murderer = true;
    c.on_create(|m| {
        m.body = 201;
        m.mlq_hue = "randomAnimalHue";
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
        m.armor = 8;
    });
});

mobile("InjuredWolf", |c| {
    c.ai = "Animal";
    c.fight_mode = "Aggressor";
    c.corpse_name = "an injured wolf corpse";
    c.name = "an injured wolf";
    c.on_create(|m| {
        m.body = 225;
        m.sound = 0xE5;
        m.mlq_hue = "randomAnimalHue";
        m.str = 10..=20;
        m.dex = 45..=65;
        m.int = 10..=15;
        m.hits = 1;
        m.damage = 1..=3;
        m.damage_type("Physical", 100);
        m.resist("Physical", 15);
        m.resist("Fire", 5, 10);
        m.skill("MagicResist", 10.0);
        m.skill("Tactics", 0.0, 5.0);
        m.skill("Wrestling", 20.0, 30.0);
    });
});

mobile("YoungNinja", |c| {
    c.ai = "Melee";
    c.fight_mode = "Aggressor";
    c.corpse_name = "a young ninja's corpse";
    c.name = "a young ninja";
    c.always_murderer = true;
    c.on_create(|m| {
        m.mlq_init_stats = [45, 30, 5];
        m.hits = 20..=30;
        m.quest_skin_hue = true;
        m.body = 400;
        m.random_hair = true;
        m.quest_random_facial_hair = true;
        m.quest_equip = [
            #{ item: "Server.Items.NinjaTabi" },
            #{ item: "Server.Items.LeatherNinjaPants" },
            #{ item: "Server.Items.LeatherNinjaJacket" },
            #{ item: "Server.Items.LeatherNinjaBelt" },
            #{ item: "Server.Items.Bandana", hue: "randomNondyedHue" },
            #{ oneOf: ["Server.Items.Tessen", "Server.Items.Kama", "Server.Items.Lajatang"] },
        ];
        m.skill("Swords", 50.0);
        m.skill("Tactics", 50.0);
    });
});

mobile("YoungRonin", |c| {
    c.ai = "Melee";
    c.fight_mode = "Aggressor";
    c.corpse_name = "a young ronin's corpse";
    c.name = "a young ronin";
    c.always_murderer = true;
    c.on_create(|m| {
        m.mlq_init_stats = [45, 30, 5];
        m.hits = 10..=20;
        m.quest_skin_hue = true;
        m.body = 400;
        m.random_hair = true;
        m.quest_random_facial_hair = true;
        m.quest_equip = [
            #{ item: "Server.Items.LeatherDo" },
            #{ item: "Server.Items.LeatherHiroSode" },
            #{ item: "Server.Items.SamuraiTabi" },
            #{
                oneOf: ["Server.Items.StuddedHaidate", "Server.Items.PlateSuneate", "Server.Items.LeatherSuneate"],
            },
            #{ item: "Server.Items.Bandana", hue: "randomNondyedHue" },
            #{ oneOf: ["Server.Items.NoDachi", "Server.Items.Lajatang", "Server.Items.Wakizashi"] },
        ];
        m.skill("Swords", 50.0);
        m.skill("Tactics", 50.0);
    });
});

mobile("Henchman", |c| {
    c.ai = "Melee";
    c.fight_mode = "Aggressor";
    c.name = "a henchman";
    c.always_murderer = true;
    c.on_create(|m| {
        m.mlq_init_stats = [45, 30, 5];
        m.quest_skin_hue = true;
        m.body = 400;
        m.random_hair = true;
        m.quest_random_facial_hair = true;
        m.quest_equip = [
            #{ item: "Server.Items.LeatherNinjaJacket" },
            #{ item: "Server.Items.LeatherNinjaPants" },
            #{ item: "Server.Items.NinjaTabi" },
        ];
        if coin() {
            m.quest_equip = [#{ item: "Server.Items.Kama" }];
        } else {
            m.quest_equip = [#{ item: "Server.Items.Tessen" }];
        }
        m.skill("Swords", 50.0);
        m.skill("Tactics", 50.0);
    });
});
