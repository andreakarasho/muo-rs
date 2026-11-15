item("RingOfTheVile", |c| {
    c.label = 1061102;
    c.rarity = 11;
    c.on_create(|i| {
        i.hue = 0x4F7;
        i.attributes.bonus_dex = 8;
        i.attributes.regen_stam = 6;
        i.attributes.attack_chance = 15;
        i.element_attributes.poison = 20;
    });
});

item("RingOfTheElements", |c| {
    c.label = 1061104;
    c.rarity = 11;
    c.on_create(|i| {
        i.hue = 0x4E9;
        i.attributes.luck = 100;
        i.element_attributes.fire = 16;
        i.element_attributes.cold = 16;
        i.element_attributes.poison = 16;
        i.element_attributes.energy = 16;
    });
});

item("OrnamentOfTheMagician", |c| {
    c.label = 1061105;
    c.rarity = 11;
    c.on_create(|i| {
        i.hue = 0x554;
        i.attributes.cast_recovery = 3;
        i.attributes.cast_speed = 2;
        i.attributes.lower_mana_cost = 10;
        i.attributes.lower_reg_cost = 20;
        i.element_attributes.energy = 15;
    });
});

item("ResilientBracer", |c| {
    c.label = 1072933;
    c.physical_resistance = 20;
    c.on_create(|i| {
        i.hue = 0x488;
        i.skill_bonuses = [["MagicResist", 15.0]];
        i.attributes.bonus_hits = 5;
        i.attributes.regen_hits = 2;
        i.attributes.defend_chance = 10;
    });
});

item("PendantOfTheMagi", |c| {
    c.label = 1072937;
    c.on_create(|i| {
        i.hue = 0x48D;
        i.attributes.bonus_int = 10;
        i.attributes.regen_mana = 3;
        i.attributes.spell_damage = 5;
        i.attributes.lower_mana_cost = 10;
        i.attributes.lower_reg_cost = 30;
    });
});

item("EssenceOfBattle", |c| {
    c.label = 1072935;
    c.on_create(|i| {
        i.hue = 0x550;
        i.attributes.bonus_dex = 7;
        i.attributes.bonus_str = 7;
        i.attributes.weapon_damage = 30;
    });
});

item("BraceletOfHealth", |c| {
    c.label = 1061103;
    c.rarity = 11;
    c.on_create(|i| {
        i.hue = 0x21;
        i.attributes.bonus_hits = 5;
        i.attributes.regen_hits = 10;
    });
});

item("GoldBracelet", |c| {
    c.weight = 0.1;
    c.graphic = 4230;
});

item("SilverBracelet", |c| {
    c.weight = 0.1;
    c.graphic = 7942;
});

item("GoldEarrings", |c| {
    c.weight = 0.1;
    c.graphic = 4231;
});

item("SilverEarrings", |c| {
    c.weight = 0.1;
    c.graphic = 7943;
});

item("GoldNecklace", |c| {
    c.weight = 0.1;
    c.graphic = 4232;
});

item("GoldBeadNecklace", |c| {
    c.weight = 0.1;
    c.graphic = 4233;
});

item("SilverNecklace", |c| {
    c.weight = 0.1;
    c.graphic = 7944;
});

item("SilverBeadNecklace", |c| {
    c.weight = 0.1;
    c.graphic = 7941;
});

item("SilverRing", |c| {
    c.weight = 0.1;
    c.graphic = 7945;
});

item("WeddingRing", |c| {
    c.weight = 0.1;
    c.graphic = 4234;
    c.name = "wedding ring";
});
