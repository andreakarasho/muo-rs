mobile("FactionKnight", |c| {
    c.guard_flags = ["Magic", "Melee", "Smart", "Curse", "Bless"];
    c.on_create(|m| {
        m.title = "the knight";
        m.range_home = 6;
        m.faction_skin_hue = true;
        m.female = false;
        m.body = 400;
        m.name = random_name("male");
        m.str = 126..=150;
        m.dex = 61..=85;
        m.int = 81..=95;
        m.damage_type("Physical", 100);
        m.resist("Physical", 30, 50);
        m.resist("Fire", 30, 50);
        m.resist("Cold", 30, 50);
        m.resist("Energy", 30, 50);
        m.resist("Poison", 30, 50);
        m.armor = 24;
        m.skill("Swords", 100.0, 110.0);
        m.skill("Wrestling", 100.0, 110.0);
        m.skill("Tactics", 100.0, 110.0);
        m.skill("MagicResist", 100.0, 110.0);
        m.skill("Healing", 100.0, 110.0);
        m.skill("Anatomy", 100.0, 110.0);
        m.skill("Magery", 100.0, 110.0);
        m.skill("EvalInt", 100.0, 110.0);
        m.skill("Meditation", 100.0, 110.0);
        m.guard_wear = [
            #{ item: "Server.Items.ChainChest", hue: 2125, movable: false },
            #{ item: "Server.Items.ChainLegs", hue: 2125, movable: false },
            #{ item: "Server.Items.ChainCoif", hue: 2125, movable: false },
            #{ item: "Server.Items.PlateArms", hue: 2125, movable: false },
            #{ item: "Server.Items.PlateGloves", hue: 2125, movable: false },
            #{ item: "Server.Items.BodySash", hue: 1254, movable: false },
            #{ item: "Server.Items.Kilt", hue: 1254, movable: false },
            #{ item: "Server.Items.Sandals", hue: 1254, movable: false },
            #{ item: "Server.Items.Bardiche", newbied: true },
        ];
        m.pack = [#{ item: "Server.Items.Bandage", amount: [30, 40] }];
        m.pack_random_items = #{
            count: [6, 12],
            from: [
                "Server.Items.GreaterHealPotion",
                "Server.Items.GreaterHealPotion",
                "Server.Items.GreaterHealPotion",
                "Server.Items.GreaterCurePotion",
                "Server.Items.GreaterCurePotion",
                "Server.Items.GreaterCurePotion",
                "Server.Items.GreaterStrengthPotion",
                "Server.Items.GreaterStrengthPotion",
                "Server.Items.GreaterAgilityPotion",
                "Server.Items.GreaterAgilityPotion",
                "Server.Items.TotalRefreshPotion",
                "Server.Items.TotalRefreshPotion",
                "Server.Items.GreaterExplosionPotion",
            ],
        };
    });
});

mobile("FactionBerserker", |c| {
    c.guard_flags = ["Melee", "Curse", "Bless"];
    c.on_create(|m| {
        m.title = "the berserker";
        m.range_home = 6;
        m.faction_skin_hue = true;
        m.female = false;
        m.body = 400;
        m.name = random_name("male");
        m.str = 126..=150;
        m.dex = 61..=85;
        m.int = 81..=95;
        m.damage_type("Physical", 100);
        m.resist("Physical", 30, 50);
        m.resist("Fire", 30, 50);
        m.resist("Cold", 30, 50);
        m.resist("Energy", 30, 50);
        m.resist("Poison", 30, 50);
        m.armor = 24;
        m.skill("Swords", 100.0, 110.0);
        m.skill("Wrestling", 100.0, 110.0);
        m.skill("Tactics", 100.0, 110.0);
        m.skill("MagicResist", 100.0, 110.0);
        m.skill("Healing", 100.0, 110.0);
        m.skill("Anatomy", 100.0, 110.0);
        m.skill("Magery", 100.0, 110.0);
        m.skill("EvalInt", 100.0, 110.0);
        m.skill("Meditation", 100.0, 110.0);
        m.guard_wear = [
            #{ item: "Server.Items.BodySash", hue: 1645, movable: false },
            #{ item: "Server.Items.Kilt", hue: 1645, movable: false },
            #{ item: "Server.Items.Sandals", hue: 1645, movable: false },
            #{ item: "Server.Items.DoubleAxe", newbied: true },
        ];
        m.mlq_hair_item_id = 8263;
        m.mlq_hair_hue = 41;
        m.mlq_facial_hair_item_id = 8267;
        m.mlq_facial_hair_hue = 41;
        m.pack = [#{ item: "Server.Items.Bandage", amount: [30, 40] }];
        m.pack_random_items = #{
            count: [6, 12],
            from: [
                "Server.Items.GreaterHealPotion",
                "Server.Items.GreaterHealPotion",
                "Server.Items.GreaterHealPotion",
                "Server.Items.GreaterCurePotion",
                "Server.Items.GreaterCurePotion",
                "Server.Items.GreaterCurePotion",
                "Server.Items.GreaterStrengthPotion",
                "Server.Items.GreaterStrengthPotion",
                "Server.Items.GreaterAgilityPotion",
                "Server.Items.GreaterAgilityPotion",
                "Server.Items.TotalRefreshPotion",
                "Server.Items.TotalRefreshPotion",
                "Server.Items.GreaterExplosionPotion",
            ],
        };
    });
});

mobile("FactionDeathKnight", |c| {
    c.guard_flags = ["Melee", "Curse", "Bless"];
    c.on_create(|m| {
        m.title = "the death knight";
        m.range_home = 6;
        m.faction_skin_hue = true;
        m.female = false;
        m.body = 400;
        m.name = random_name("male");
        m.hue = 1;
        m.str = 126..=150;
        m.dex = 61..=85;
        m.int = 81..=95;
        m.damage_type("Physical", 100);
        m.resist("Physical", 30, 50);
        m.resist("Fire", 30, 50);
        m.resist("Cold", 30, 50);
        m.resist("Energy", 30, 50);
        m.resist("Poison", 30, 50);
        m.armor = 24;
        m.skill("Swords", 100.0, 110.0);
        m.skill("Wrestling", 100.0, 110.0);
        m.skill("Tactics", 100.0, 110.0);
        m.skill("MagicResist", 100.0, 110.0);
        m.skill("Healing", 100.0, 110.0);
        m.skill("Anatomy", 100.0, 110.0);
        m.skill("Magery", 100.0, 110.0);
        m.skill("EvalInt", 100.0, 110.0);
        m.skill("Meditation", 100.0, 110.0);
        m.guard_wear = [
            #{ plainItem: 8270, layer: "OuterTorso", hue: 1109, movable: false },
            #{ item: "Server.Items.ExecutionersAxe", hue: 2211, newbied: true },
        ];
        m.pack = [#{ item: "Server.Items.Bandage", amount: [30, 40] }];
        m.pack_random_items = #{
            count: [6, 12],
            from: [
                "Server.Items.GreaterHealPotion",
                "Server.Items.GreaterHealPotion",
                "Server.Items.GreaterHealPotion",
                "Server.Items.GreaterCurePotion",
                "Server.Items.GreaterCurePotion",
                "Server.Items.GreaterCurePotion",
                "Server.Items.GreaterStrengthPotion",
                "Server.Items.GreaterStrengthPotion",
                "Server.Items.GreaterAgilityPotion",
                "Server.Items.GreaterAgilityPotion",
                "Server.Items.TotalRefreshPotion",
                "Server.Items.TotalRefreshPotion",
                "Server.Items.GreaterExplosionPotion",
            ],
        };
    });
});

mobile("FactionDragoon", |c| {
    c.guard_flags = ["Magic", "Melee", "Smart", "Bless", "Curse"];
    c.on_create(|m| {
        m.title = "the dragoon";
        m.range_home = 6;
        m.faction_skin_hue = true;
        m.female = false;
        m.body = 400;
        m.name = random_name("male");
        m.str = 151..=175;
        m.dex = 61..=85;
        m.int = 151..=175;
        m.resist("Physical", 40, 60);
        m.resist("Fire", 40, 60);
        m.resist("Cold", 40, 60);
        m.resist("Energy", 40, 60);
        m.resist("Poison", 40, 60);
        m.armor = 32;
        m.skill("Macing", 110.0, 120.0);
        m.skill("Wrestling", 110.0, 120.0);
        m.skill("Tactics", 110.0, 120.0);
        m.skill("MagicResist", 110.0, 120.0);
        m.skill("Healing", 110.0, 120.0);
        m.skill("Anatomy", 110.0, 120.0);
        m.skill("Magery", 110.0, 120.0);
        m.skill("EvalInt", 110.0, 120.0);
        m.skill("Meditation", 110.0, 120.0);
        m.guard_wear = [
            #{ item: "Server.Items.Cloak", hue: 1645, movable: false },
            #{ item: "Server.Items.PlateChest", hue: 1645, movable: false },
            #{ item: "Server.Items.PlateLegs", hue: 1109, movable: false },
            #{ item: "Server.Items.PlateArms", hue: 1109, movable: false },
            #{ item: "Server.Items.PlateGloves", hue: 1109, movable: false },
            #{ item: "Server.Items.PlateGorget", hue: 1109, movable: false },
            #{ item: "Server.Items.PlateHelm", hue: 1109, movable: false },
            #{ item: "Server.Items.WarHammer", newbied: true },
            #{ virtualMount: true, hue: 1109, movable: false },
        ];
        m.pack = [#{ item: "Server.Items.Bandage", amount: [30, 40] }];
        m.pack_random_items = #{
            count: [6, 12],
            from: [
                "Server.Items.GreaterHealPotion",
                "Server.Items.GreaterHealPotion",
                "Server.Items.GreaterHealPotion",
                "Server.Items.GreaterCurePotion",
                "Server.Items.GreaterCurePotion",
                "Server.Items.GreaterCurePotion",
                "Server.Items.GreaterStrengthPotion",
                "Server.Items.GreaterStrengthPotion",
                "Server.Items.GreaterAgilityPotion",
                "Server.Items.GreaterAgilityPotion",
                "Server.Items.TotalRefreshPotion",
                "Server.Items.TotalRefreshPotion",
                "Server.Items.GreaterExplosionPotion",
            ],
        };
    });
});

mobile("FactionHenchman", |c| {
    c.guard_flags = ["Melee"];
    c.on_create(|m| {
        m.title = "the henchman";
        m.range_home = 6;
        m.faction_skin_hue = true;
        m.female = false;
        m.body = 400;
        m.name = random_name("male");
        m.random_hair = true;
        m.quest_facial_hair_of_hair_hue = true;
        m.str = 91..=115;
        m.dex = 61..=85;
        m.int = 81..=95;
        m.damage = 10..=14;
        m.resist("Physical", 10, 30);
        m.resist("Fire", 10, 30);
        m.resist("Cold", 10, 30);
        m.resist("Energy", 10, 30);
        m.resist("Poison", 10, 30);
        m.armor = 8;
        m.skill("Fencing", 80.0, 90.0);
        m.skill("Wrestling", 80.0, 90.0);
        m.skill("Tactics", 80.0, 90.0);
        m.skill("MagicResist", 80.0, 90.0);
        m.skill("Healing", 80.0, 90.0);
        m.skill("Anatomy", 80.0, 90.0);
        m.guard_wear = [
            #{ item: "Server.Items.StuddedChest" },
            #{ item: "Server.Items.StuddedLegs" },
            #{ item: "Server.Items.StuddedArms" },
            #{ item: "Server.Items.StuddedGloves" },
            #{ item: "Server.Items.StuddedGorget" },
            #{ item: "Server.Items.Boots" },
            #{ item: "Server.Items.Spear", newbied: true },
        ];
        m.pack = [#{ item: "Server.Items.Bandage", amount: [10, 20] }];
        m.pack_random_items = #{
            count: [1, 4],
            from: [
                "Server.Items.HealPotion",
                "Server.Items.HealPotion",
                "Server.Items.HealPotion",
                "Server.Items.CurePotion",
                "Server.Items.CurePotion",
                "Server.Items.CurePotion",
                "Server.Items.StrengthPotion",
                "Server.Items.StrengthPotion",
                "Server.Items.AgilityPotion",
                "Server.Items.AgilityPotion",
                "Server.Items.RefreshPotion",
                "Server.Items.RefreshPotion",
                "Server.Items.ExplosionPotion",
            ],
        };
    });
});

mobile("FactionMercenary", |c| {
    c.guard_flags = ["Melee", "Smart"];
    c.on_create(|m| {
        m.title = "the mercenary";
        m.range_home = 6;
        m.faction_skin_hue = true;
        m.female = false;
        m.body = 400;
        m.name = random_name("male");
        m.random_hair = true;
        m.quest_facial_hair_of_hair_hue = true;
        m.str = 116..=125;
        m.dex = 61..=85;
        m.int = 81..=95;
        m.resist("Physical", 20, 40);
        m.resist("Fire", 20, 40);
        m.resist("Cold", 20, 40);
        m.resist("Energy", 20, 40);
        m.resist("Poison", 20, 40);
        m.armor = 16;
        m.skill("Fencing", 90.0, 100.0);
        m.skill("Wrestling", 90.0, 100.0);
        m.skill("Tactics", 90.0, 100.0);
        m.skill("MagicResist", 90.0, 100.0);
        m.skill("Healing", 90.0, 100.0);
        m.skill("Anatomy", 90.0, 100.0);
        m.guard_wear = [
            #{ item: "Server.Items.ChainChest" },
            #{ item: "Server.Items.ChainLegs" },
            #{ item: "Server.Items.RingmailArms" },
            #{ item: "Server.Items.RingmailGloves" },
            #{ item: "Server.Items.ChainCoif" },
            #{ item: "Server.Items.Boots" },
            #{ item: "Server.Items.ShortSpear", newbied: true },
        ];
        m.pack = [#{ item: "Server.Items.Bandage", amount: [20, 30] }];
        m.pack_random_items = #{
            count: [3, 8],
            from: [
                "Server.Items.GreaterHealPotion",
                "Server.Items.GreaterHealPotion",
                "Server.Items.GreaterHealPotion",
                "Server.Items.GreaterCurePotion",
                "Server.Items.GreaterCurePotion",
                "Server.Items.GreaterCurePotion",
                "Server.Items.GreaterStrengthPotion",
                "Server.Items.GreaterStrengthPotion",
                "Server.Items.GreaterAgilityPotion",
                "Server.Items.GreaterAgilityPotion",
                "Server.Items.TotalRefreshPotion",
                "Server.Items.TotalRefreshPotion",
                "Server.Items.GreaterExplosionPotion",
            ],
        };
    });
});

mobile("FactionNecromancer", |c| {
    c.guard_flags = ["Magic", "Smart", "Bless", "Curse"];
    c.on_create(|m| {
        m.title = "the necromancer";
        m.range_home = 6;
        m.faction_skin_hue = true;
        m.female = false;
        m.body = 400;
        m.name = random_name("male");
        m.hue = 1;
        m.str = 151..=175;
        m.dex = 61..=85;
        m.int = 151..=175;
        m.resist("Physical", 40, 60);
        m.resist("Fire", 40, 60);
        m.resist("Cold", 40, 60);
        m.resist("Energy", 40, 60);
        m.resist("Poison", 40, 60);
        m.armor = 32;
        m.skill("Macing", 110.0, 120.0);
        m.skill("Wrestling", 110.0, 120.0);
        m.skill("Tactics", 110.0, 120.0);
        m.skill("MagicResist", 110.0, 120.0);
        m.skill("Healing", 110.0, 120.0);
        m.skill("Anatomy", 110.0, 120.0);
        m.skill("Magery", 110.0, 120.0);
        m.skill("EvalInt", 110.0, 120.0);
        m.skill("Meditation", 110.0, 120.0);
        m.guard_wear = [
            #{ plainItem: 8270, layer: "OuterTorso", hue: 1109, movable: false },
            #{ item: "Server.Items.GnarledStaff", hue: 2211, newbied: true },
        ];
        m.pack = [#{ item: "Server.Items.Bandage", amount: [30, 40] }];
        m.pack_random_items = #{
            count: [6, 12],
            from: [
                "Server.Items.GreaterHealPotion",
                "Server.Items.GreaterHealPotion",
                "Server.Items.GreaterHealPotion",
                "Server.Items.GreaterCurePotion",
                "Server.Items.GreaterCurePotion",
                "Server.Items.GreaterCurePotion",
                "Server.Items.GreaterStrengthPotion",
                "Server.Items.GreaterStrengthPotion",
                "Server.Items.GreaterAgilityPotion",
                "Server.Items.GreaterAgilityPotion",
                "Server.Items.TotalRefreshPotion",
                "Server.Items.TotalRefreshPotion",
                "Server.Items.GreaterExplosionPotion",
            ],
        };
    });
});

mobile("FactionPaladin", |c| {
    c.guard_flags = ["Magic", "Melee", "Smart", "Curse", "Bless"];
    c.on_create(|m| {
        m.title = "the paladin";
        m.range_home = 6;
        m.faction_skin_hue = true;
        m.female = false;
        m.body = 400;
        m.name = random_name("male");
        m.str = 151..=175;
        m.dex = 61..=85;
        m.int = 81..=95;
        m.resist("Physical", 40, 60);
        m.resist("Fire", 40, 60);
        m.resist("Cold", 40, 60);
        m.resist("Energy", 40, 60);
        m.resist("Poison", 40, 60);
        m.armor = 32;
        m.skill("Swords", 110.0, 120.0);
        m.skill("Wrestling", 110.0, 120.0);
        m.skill("Tactics", 110.0, 120.0);
        m.skill("MagicResist", 110.0, 120.0);
        m.skill("Healing", 110.0, 120.0);
        m.skill("Anatomy", 110.0, 120.0);
        m.skill("Magery", 110.0, 120.0);
        m.skill("EvalInt", 110.0, 120.0);
        m.skill("Meditation", 110.0, 120.0);
        m.guard_wear = [
            #{ item: "Server.Items.PlateChest", hue: 2125, movable: false },
            #{ item: "Server.Items.PlateLegs", hue: 2125, movable: false },
            #{ item: "Server.Items.PlateHelm", hue: 2125, movable: false },
            #{ item: "Server.Items.PlateGorget", hue: 2125, movable: false },
            #{ item: "Server.Items.PlateArms", hue: 2125, movable: false },
            #{ item: "Server.Items.PlateGloves", hue: 2125, movable: false },
            #{ item: "Server.Items.BodySash", hue: 1254, movable: false },
            #{ item: "Server.Items.Cloak", hue: 1254, movable: false },
            #{ item: "Server.Items.Halberd", newbied: true },
            #{ virtualMount: true, hue: 1254, movable: false },
        ];
        m.pack = [#{ item: "Server.Items.Bandage", amount: [30, 40] }];
        m.pack_random_items = #{
            count: [6, 12],
            from: [
                "Server.Items.GreaterHealPotion",
                "Server.Items.GreaterHealPotion",
                "Server.Items.GreaterHealPotion",
                "Server.Items.GreaterCurePotion",
                "Server.Items.GreaterCurePotion",
                "Server.Items.GreaterCurePotion",
                "Server.Items.GreaterStrengthPotion",
                "Server.Items.GreaterStrengthPotion",
                "Server.Items.GreaterAgilityPotion",
                "Server.Items.GreaterAgilityPotion",
                "Server.Items.TotalRefreshPotion",
                "Server.Items.TotalRefreshPotion",
                "Server.Items.GreaterExplosionPotion",
            ],
        };
    });
});

mobile("FactionSorceress", |c| {
    c.guard_flags = ["Magic", "Bless", "Curse"];
    c.on_create(|m| {
        m.title = "the sorceress";
        m.range_home = 6;
        m.faction_skin_hue = true;
        m.female = true;
        m.body = 401;
        m.name = random_name("female");
        m.str = 126..=150;
        m.dex = 61..=85;
        m.int = 126..=150;
        m.damage_type("Physical", 100);
        m.resist("Physical", 30, 50);
        m.resist("Fire", 30, 50);
        m.resist("Cold", 30, 50);
        m.resist("Energy", 30, 50);
        m.resist("Poison", 30, 50);
        m.armor = 24;
        m.skill("Macing", 100.0, 110.0);
        m.skill("Wrestling", 100.0, 110.0);
        m.skill("Tactics", 100.0, 110.0);
        m.skill("MagicResist", 100.0, 110.0);
        m.skill("Healing", 100.0, 110.0);
        m.skill("Anatomy", 100.0, 110.0);
        m.skill("Magery", 100.0, 110.0);
        m.skill("EvalInt", 100.0, 110.0);
        m.skill("Meditation", 100.0, 110.0);
        m.guard_wear = [
            #{ item: "Server.Items.WizardsHat", hue: 1325, movable: false },
            #{ item: "Server.Items.Sandals", hue: 1325, movable: false },
            #{ item: "Server.Items.LeatherGorget", hue: 1325, movable: false },
            #{ item: "Server.Items.LeatherGloves", hue: 1325, movable: false },
            #{ item: "Server.Items.LeatherLegs", hue: 1325, movable: false },
            #{ item: "Server.Items.Skirt", hue: 1325, movable: false },
            #{ item: "Server.Items.FemaleLeatherChest", hue: 1325, movable: false },
            #{ item: "Server.Items.QuarterStaff", hue: 1310, newbied: true },
        ];
        m.pack = [#{ item: "Server.Items.Bandage", amount: [30, 40] }];
        m.pack_random_items = #{
            count: [6, 12],
            from: [
                "Server.Items.GreaterHealPotion",
                "Server.Items.GreaterHealPotion",
                "Server.Items.GreaterHealPotion",
                "Server.Items.GreaterCurePotion",
                "Server.Items.GreaterCurePotion",
                "Server.Items.GreaterCurePotion",
                "Server.Items.GreaterStrengthPotion",
                "Server.Items.GreaterStrengthPotion",
                "Server.Items.GreaterAgilityPotion",
                "Server.Items.GreaterAgilityPotion",
                "Server.Items.TotalRefreshPotion",
                "Server.Items.TotalRefreshPotion",
                "Server.Items.GreaterExplosionPotion",
            ],
        };
    });
});

mobile("FactionWizard", |c| {
    c.guard_flags = ["Magic", "Smart", "Bless", "Curse"];
    c.on_create(|m| {
        m.title = "the wizard";
        m.range_home = 6;
        m.faction_skin_hue = true;
        m.female = false;
        m.body = 400;
        m.name = random_name("male");
        m.str = 151..=175;
        m.dex = 61..=85;
        m.int = 151..=175;
        m.damage_type("Physical", 100);
        m.resist("Physical", 40, 60);
        m.resist("Fire", 40, 60);
        m.resist("Cold", 40, 60);
        m.resist("Energy", 40, 60);
        m.resist("Poison", 40, 60);
        m.armor = 32;
        m.skill("Macing", 110.0, 120.0);
        m.skill("Wrestling", 110.0, 120.0);
        m.skill("Tactics", 110.0, 120.0);
        m.skill("MagicResist", 110.0, 120.0);
        m.skill("Healing", 110.0, 120.0);
        m.skill("Anatomy", 110.0, 120.0);
        m.skill("Magery", 110.0, 120.0);
        m.skill("EvalInt", 110.0, 120.0);
        m.skill("Meditation", 110.0, 120.0);
        m.guard_wear = [
            #{ item: "Server.Items.WizardsHat", hue: 1325, movable: false },
            #{ item: "Server.Items.Sandals", hue: 1325, movable: false },
            #{ item: "Server.Items.Robe", hue: 1310, movable: false },
            #{ item: "Server.Items.LeatherGloves", hue: 1325, movable: false },
            #{ item: "Server.Items.GnarledStaff", hue: 1310, newbied: true },
        ];
        m.pack = [#{ item: "Server.Items.Bandage", amount: [30, 40] }];
        m.pack_random_items = #{
            count: [6, 12],
            from: [
                "Server.Items.GreaterHealPotion",
                "Server.Items.GreaterHealPotion",
                "Server.Items.GreaterHealPotion",
                "Server.Items.GreaterCurePotion",
                "Server.Items.GreaterCurePotion",
                "Server.Items.GreaterCurePotion",
                "Server.Items.GreaterStrengthPotion",
                "Server.Items.GreaterStrengthPotion",
                "Server.Items.GreaterAgilityPotion",
                "Server.Items.GreaterAgilityPotion",
                "Server.Items.TotalRefreshPotion",
                "Server.Items.TotalRefreshPotion",
                "Server.Items.GreaterExplosionPotion",
            ],
        };
    });
});

mobile("FactionBottleVendor", |c| {
    c.npc_shoe_type = ["Shoes", "Sandals"];
    c.on_create(|m| {
        m.title = "the Bottle Seller";
        m.mlq_frozen = true;
        m.cant_walk = true;
        m.range_home = 0;
        m.female = false;
        m.body = 400;
        m.name = random_name("male");
        m.skill("Alchemy", 85.0, 100.0);
        m.skill("TasteID", 65.0, 88.0);
    });
    c.on_outfit(|m| {
        m.npc_vendor_outfit = true;
        m.npc_add_item = [#{ item: "Server.Items.Robe", hue: "randomPink" }];
    });
});

mobile("FactionBoardVendor", |c| {
    c.on_create(|m| {
        m.title = "the LumberMan";
        m.mlq_frozen = true;
        m.cant_walk = true;
        m.range_home = 0;
        m.female = false;
        m.body = 400;
        m.name = random_name("male");
        m.skill("Carpentry", 85.0, 100.0);
        m.skill("Lumberjacking", 60.0, 83.0);
    });
    c.on_outfit(|m| {
        m.npc_vendor_outfit = true;
        m.npc_add_item = ["Server.Items.HalfApron"];
    });
});

mobile("FactionOreVendor", |c| {
    c.on_create(|m| {
        m.title = "the Ore Man";
        m.mlq_frozen = true;
        m.cant_walk = true;
        m.range_home = 0;
        m.female = false;
        m.body = 400;
        m.name = random_name("male");
        m.skill("Carpentry", 85.0, 100.0);
        m.skill("Lumberjacking", 60.0, 83.0);
    });
    c.on_outfit(|m| {
        m.npc_vendor_outfit = true;
        m.npc_add_item = ["Server.Items.HalfApron"];
    });
});

mobile("FactionReagentVendor", |c| {
    c.npc_shoe_type = ["Shoes", "Sandals"];
    c.on_create(|m| {
        m.title = "the Reagent Man";
        m.mlq_frozen = true;
        m.cant_walk = true;
        m.range_home = 0;
        m.female = false;
        m.body = 400;
        m.name = random_name("male");
        m.skill("EvalInt", 65.0, 88.0);
        m.skill("Inscribe", 60.0, 83.0);
        m.skill("Magery", 64.0, 100.0);
        m.skill("Meditation", 60.0, 83.0);
        m.skill("MagicResist", 65.0, 88.0);
        m.skill("Wrestling", 36.0, 68.0);
    });
    c.on_outfit(|m| {
        m.npc_vendor_outfit = true;
        m.npc_add_item = [#{ item: "Server.Items.Robe", hue: "randomBlue" }, "Server.Items.GnarledStaff"];
    });
});

mobile("FactionHorseVendor", |c| {
    c.npc_shoe_type = #{ female: "ThighBoots", male: "Boots" };
    c.npc_shoe_hue = 0;
    c.on_create(|m| {
        m.title = "the Horse Breeder";
        m.mlq_frozen = true;
        m.cant_walk = true;
        m.range_home = 0;
        m.female = false;
        m.body = 400;
        m.name = random_name("male");
        m.skill("AnimalLore", 64.0, 100.0);
        m.skill("AnimalTaming", 90.0, 100.0);
        m.skill("Veterinary", 65.0, 88.0);
    });
    c.on_outfit(|m| {
        m.npc_vendor_outfit = true;
        if coin() {
            m.npc_add_item = ["Server.Items.QuarterStaff"];
        } else {
            m.npc_add_item = ["Server.Items.ShepherdsCrook"];
        }
    });
});

mobile("FactionWarHorse", |c| {
    c.fight_mode = "Aggressor";
    c.mount_body = 226;
    c.mount_item_id = 16032;
    c.name = "a war horse";
    c.corpse_name = "a war horse corpse";
    c.steps_max = 6400;
    c.favorite_food = ["FruitsAndVeggies", "GrainsAndHay"];
    c.on_create(|m| {
        m.sound = 0xA8;
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
    });
});
