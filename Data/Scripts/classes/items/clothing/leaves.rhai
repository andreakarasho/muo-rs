item("ClothNinjaJacket", |c| {
    c.weight = 5.0;
    c.graphic = 10132;
    c.on_create(|i| {
        i.layer = "InnerTorso";
    });
});

item("MonkRobe", |c| {
    c.label = 1076584;
    c.weight = 1.0;
    c.graphic = 9863;
    c.on_create(|i| {
        i.str_requirement = 0;
    });
});

item("MagicWizardsHat", |c| {
    c.label = 1041072;
    c.weight = 1.0;
    c.base_physical_resistance = 0;
    c.base_fire_resistance = 5;
    c.base_cold_resistance = 9;
    c.base_poison_resistance = 5;
    c.base_energy_resistance = 5;
    c.init_min_hits = 20;
    c.init_max_hits = 30;
    c.base_str_bonus = -5;
    c.base_dex_bonus = -5;
    c.base_int_bonus = 5;
    c.graphic = 5912;
});

item("SpiritOfTheTotem", |c| {
    c.label = 1061599;
    c.rarity = 11;
    c.base_physical_resistance = 20;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.on_create(|i| {
        i.hue = 0x455;
        i.attributes.bonus_str = 20;
        i.attributes.reflect_physical = 15;
        i.attributes.attack_chance = 15;
    });
});

item("HuntersHeaddress", |c| {
    c.label = 1061595;
    c.rarity = 11;
    c.base_cold_resistance = 23;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.on_create(|i| {
        i.hue = 0x594;
        i.skill_bonuses = [["Archery", 20.0]];
        i.attributes.bonus_dex = 8;
        i.attributes.night_sight = 1;
        i.attributes.attack_chance = 15;
    });
});

item("HatOfTheMagi", |c| {
    c.label = 1061597;
    c.rarity = 11;
    c.base_poison_resistance = 20;
    c.base_energy_resistance = 20;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.on_create(|i| {
        i.hue = 0x481;
        i.attributes.bonus_int = 8;
        i.attributes.regen_mana = 4;
        i.attributes.spell_damage = 10;
    });
});

item("DivineCountenance", |c| {
    c.label = 1061289;
    c.rarity = 11;
    c.base_physical_resistance = 8;
    c.base_fire_resistance = 6;
    c.base_cold_resistance = 9;
    c.base_energy_resistance = 25;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.on_create(|i| {
        i.hue = 0x482;
        i.attributes.bonus_int = 8;
        i.attributes.regen_mana = 2;
        i.attributes.reflect_physical = 15;
        i.attributes.lower_mana_cost = 8;
    });
});

item("CrimsonCincture", |c| {
    c.label = 1075043;
    c.on_create(|i| {
        i.hue = 0x485;
        i.attributes.bonus_dex = 5;
        i.attributes.bonus_hits = 10;
        i.attributes.regen_hits = 2;
    });
});

item("FurCape", |c| {
    c.weight = 4.0;
    c.graphic = 8970;
});

item("Kasa", |c| {
    c.weight = 3.0;
    c.base_physical_resistance = 0;
    c.base_fire_resistance = 5;
    c.base_cold_resistance = 9;
    c.base_poison_resistance = 5;
    c.base_energy_resistance = 5;
    c.init_min_hits = 20;
    c.init_max_hits = 30;
    c.graphic = 10136;
});

item("ClothNinjaHood", |c| {
    c.weight = 2.0;
    c.base_physical_resistance = 3;
    c.base_fire_resistance = 3;
    c.base_cold_resistance = 6;
    c.base_poison_resistance = 9;
    c.base_energy_resistance = 9;
    c.init_min_hits = 20;
    c.init_max_hits = 30;
    c.graphic = 10127;
});

item("FlowerGarland", |c| {
    c.weight = 1.0;
    c.base_physical_resistance = 3;
    c.base_fire_resistance = 3;
    c.base_cold_resistance = 6;
    c.base_poison_resistance = 9;
    c.base_energy_resistance = 9;
    c.init_min_hits = 20;
    c.init_max_hits = 30;
    c.graphic = 8966;
});

item("FloppyHat", |c| {
    c.weight = 1.0;
    c.base_physical_resistance = 0;
    c.base_fire_resistance = 5;
    c.base_cold_resistance = 9;
    c.base_poison_resistance = 5;
    c.base_energy_resistance = 5;
    c.init_min_hits = 20;
    c.init_max_hits = 30;
    c.graphic = 5907;
});

item("WideBrimHat", |c| {
    c.weight = 1.0;
    c.base_physical_resistance = 0;
    c.base_fire_resistance = 5;
    c.base_cold_resistance = 9;
    c.base_poison_resistance = 5;
    c.base_energy_resistance = 5;
    c.init_min_hits = 20;
    c.init_max_hits = 30;
    c.graphic = 5908;
});

item("SkullCap", |c| {
    c.weight = 1.0;
    c.base_physical_resistance = 0;
    c.base_fire_resistance = 3;
    c.base_cold_resistance = 5;
    c.base_poison_resistance = 8;
    c.base_energy_resistance = 8;
    c.init_min_hits = [7, 14];
    c.init_max_hits = [12, 28];
    c.graphic = 5444;
});

item("Bandana", |c| {
    c.weight = 1.0;
    c.base_physical_resistance = 0;
    c.base_fire_resistance = 3;
    c.base_cold_resistance = 5;
    c.base_poison_resistance = 8;
    c.base_energy_resistance = 8;
    c.init_min_hits = 20;
    c.init_max_hits = 30;
    c.graphic = 5440;
});

item("BearMask", |c| {
    c.weight = 5.0;
    c.base_physical_resistance = 5;
    c.base_fire_resistance = 3;
    c.base_cold_resistance = 8;
    c.base_poison_resistance = 4;
    c.base_energy_resistance = 4;
    c.init_min_hits = 20;
    c.init_max_hits = 30;
    c.graphic = 5445;
});

item("DeerMask", |c| {
    c.weight = 4.0;
    c.base_physical_resistance = 2;
    c.base_fire_resistance = 6;
    c.base_cold_resistance = 8;
    c.base_poison_resistance = 1;
    c.base_energy_resistance = 7;
    c.init_min_hits = 20;
    c.init_max_hits = 30;
    c.graphic = 5447;
});

item("HornedTribalMask", |c| {
    c.weight = 2.0;
    c.base_physical_resistance = 6;
    c.base_fire_resistance = 9;
    c.base_cold_resistance = 0;
    c.base_poison_resistance = 4;
    c.base_energy_resistance = 5;
    c.init_min_hits = 20;
    c.init_max_hits = 30;
    c.graphic = 5449;
});

item("TribalMask", |c| {
    c.weight = 2.0;
    c.base_physical_resistance = 3;
    c.base_fire_resistance = 0;
    c.base_cold_resistance = 6;
    c.base_poison_resistance = 10;
    c.base_energy_resistance = 5;
    c.init_min_hits = 20;
    c.init_max_hits = 30;
    c.graphic = 5451;
});

item("TallStrawHat", |c| {
    c.weight = 1.0;
    c.base_physical_resistance = 0;
    c.base_fire_resistance = 5;
    c.base_cold_resistance = 9;
    c.base_poison_resistance = 5;
    c.base_energy_resistance = 5;
    c.init_min_hits = 20;
    c.init_max_hits = 30;
    c.graphic = 5910;
});

item("StrawHat", |c| {
    c.weight = 1.0;
    c.base_physical_resistance = 0;
    c.base_fire_resistance = 5;
    c.base_cold_resistance = 9;
    c.base_poison_resistance = 5;
    c.base_energy_resistance = 5;
    c.init_min_hits = 20;
    c.init_max_hits = 30;
    c.graphic = 5911;
});

item("WizardsHat", |c| {
    c.weight = 1.0;
    c.base_physical_resistance = 0;
    c.base_fire_resistance = 5;
    c.base_cold_resistance = 9;
    c.base_poison_resistance = 5;
    c.base_energy_resistance = 5;
    c.init_min_hits = 20;
    c.init_max_hits = 30;
    c.graphic = 5912;
});

item("Bonnet", |c| {
    c.weight = 1.0;
    c.base_physical_resistance = 0;
    c.base_fire_resistance = 5;
    c.base_cold_resistance = 9;
    c.base_poison_resistance = 5;
    c.base_energy_resistance = 5;
    c.init_min_hits = 20;
    c.init_max_hits = 30;
    c.graphic = 5913;
});

item("FeatheredHat", |c| {
    c.weight = 1.0;
    c.base_physical_resistance = 0;
    c.base_fire_resistance = 5;
    c.base_cold_resistance = 9;
    c.base_poison_resistance = 5;
    c.base_energy_resistance = 5;
    c.init_min_hits = 20;
    c.init_max_hits = 30;
    c.graphic = 5914;
});

item("TricorneHat", |c| {
    c.weight = 1.0;
    c.base_physical_resistance = 0;
    c.base_fire_resistance = 5;
    c.base_cold_resistance = 9;
    c.base_poison_resistance = 5;
    c.base_energy_resistance = 5;
    c.init_min_hits = 20;
    c.init_max_hits = 30;
    c.graphic = 5915;
});

item("JesterHat", |c| {
    c.weight = 1.0;
    c.base_physical_resistance = 0;
    c.base_fire_resistance = 5;
    c.base_cold_resistance = 9;
    c.base_poison_resistance = 5;
    c.base_energy_resistance = 5;
    c.init_min_hits = 20;
    c.init_max_hits = 30;
    c.graphic = 5916;
});

item("BodySash", |c| {
    c.weight = 1.0;
    c.graphic = 5441;
});

item("FullApron", |c| {
    c.weight = 4.0;
    c.graphic = 5437;
});

item("Doublet", |c| {
    c.weight = 2.0;
    c.graphic = 8059;
});

item("Surcoat", |c| {
    c.weight = 6.0;
    c.graphic = 8189;
});

item("Tunic", |c| {
    c.weight = 5.0;
    c.graphic = 8097;
});

item("FormalShirt", |c| {
    c.weight = 1.0;
    c.graphic = 8976;
});

item("JesterSuit", |c| {
    c.weight = 4.0;
    c.graphic = 8095;
});

item("JinBaori", |c| {
    c.weight = 3.0;
    c.graphic = 10145;
});

item("FurSarong", |c| {
    c.weight = 3.0;
    c.graphic = 8972;
});

item("Skirt", |c| {
    c.weight = 4.0;
    c.graphic = 5398;
});

item("Kilt", |c| {
    c.weight = 2.0;
    c.graphic = 5431;
});

item("Hakama", |c| {
    c.weight = 2.0;
    c.graphic = 10138;
});

item("GildedDress", |c| {
    c.weight = 3.0;
    c.graphic = 8974;
});

item("FancyDress", |c| {
    c.weight = 3.0;
    c.graphic = 7936;
});

item("PlainDress", |c| {
    c.weight = 2.0;
    c.graphic = 7937;
});

item("Kamishimo", |c| {
    c.weight = 3.0;
    c.graphic = 10137;
});

item("HakamaShita", |c| {
    c.weight = 3.0;
    c.graphic = 10140;
});

item("MaleKimono", |c| {
    c.weight = 3.0;
    c.graphic = 10114;
});

item("FemaleKimono", |c| {
    c.weight = 3.0;
    c.graphic = 10115;
});

item("MaleElvenRobe", |c| {
    c.weight = 2.0;
    c.graphic = 12217;
});

item("FemaleElvenRobe", |c| {
    c.weight = 2.0;
    c.required_races = "AllowElvesOnly";
    c.allow_male_wearer = false;
    c.graphic = 12218;
});

item("ShortPants", |c| {
    c.weight = 2.0;
    c.graphic = 5422;
});

item("LongPants", |c| {
    c.weight = 2.0;
    c.graphic = 5433;
});

item("TattsukeHakama", |c| {
    c.weight = 2.0;
    c.graphic = 10139;
});

item("ElvenPants", |c| {
    c.weight = 2.0;
    c.required_races = "AllowElvesOnly";
    c.graphic = 12227;
});

item("Shirt", |c| {
    c.weight = 1.0;
    c.graphic = 5399;
});

item("ElvenShirt", |c| {
    c.weight = 2.0;
    c.required_races = "AllowElvesOnly";
    c.graphic = 12661;
});

item("ElvenDarkShirt", |c| {
    c.weight = 2.0;
    c.graphic = 12662;
});

item("FurBoots", |c| {
    c.weight = 3.0;
    c.graphic = 8967;
});

item("Boots", |c| {
    c.weight = 3.0;
    c.default_resource = "RegularLeather";
    c.graphic = 5899;
});

item("Shoes", |c| {
    c.weight = 2.0;
    c.default_resource = "RegularLeather";
    c.graphic = 5903;
});

item("Sandals", |c| {
    c.weight = 1.0;
    c.default_resource = "RegularLeather";
    c.graphic = 5901;
});

item("NinjaTabi", |c| {
    c.weight = 2.0;
    c.graphic = 10135;
});

item("SamuraiTabi", |c| {
    c.weight = 2.0;
    c.graphic = 10134;
});

item("Waraji", |c| {
    c.weight = 2.0;
    c.graphic = 10134;
});

item("ElvenBoots", |c| {
    c.weight = 2.0;
    c.default_resource = "RegularLeather";
    c.required_races = "AllowElvesOnly";
    c.graphic = 12228;
});

item("HalfApron", |c| {
    c.weight = 2.0;
    c.graphic = 5435;
});

item("Obi", |c| {
    c.weight = 1.0;
    c.graphic = 10144;
});

item("WoodlandBelt", |c| {
    c.weight = 4.0;
    c.required_races = "AllowElvesOnly";
    c.graphic = 11112;
});
