item("BasePotion", |c| {
    c.weight = 1.0;
});

item("InvisibilityPotion", |c| {
    c.on_create(|i| {
        i.item_id = 0xF0A;
        i.potion_effect = "Invisibility";
        i.hue = 0x48D;
    });
});

item("NightSightPotion", |c| {
    c.on_create(|i| {
        i.item_id = 0xF06;
        i.potion_effect = "Nightsight";
    });
});

item("LesserHealPotion", |c| {
    c.on_create(|i| {
        i.item_id = 0xF0C;
        i.potion_effect = "HealLesser";
    });
});

item("HealPotion", |c| {
    c.on_create(|i| {
        i.item_id = 0xF0C;
        i.potion_effect = "Heal";
    });
});

item("GreaterHealPotion", |c| {
    c.on_create(|i| {
        i.item_id = 0xF0C;
        i.potion_effect = "HealGreater";
    });
});

item("LesserCurePotion", |c| {
    c.on_create(|i| {
        i.item_id = 0xF07;
        i.potion_effect = "CureLesser";
    });
});

item("CurePotion", |c| {
    c.on_create(|i| {
        i.item_id = 0xF07;
        i.potion_effect = "Cure";
    });
});

item("GreaterCurePotion", |c| {
    c.on_create(|i| {
        i.item_id = 0xF07;
        i.potion_effect = "CureGreater";
    });
});

item("RefreshPotion", |c| {
    c.on_create(|i| {
        i.item_id = 0xF0B;
        i.potion_effect = "Refresh";
    });
});

item("TotalRefreshPotion", |c| {
    c.on_create(|i| {
        i.item_id = 0xF0B;
        i.potion_effect = "RefreshTotal";
    });
});

item("AgilityPotion", |c| {
    c.on_create(|i| {
        i.item_id = 0xF08;
        i.potion_effect = "Agility";
    });
});

item("GreaterAgilityPotion", |c| {
    c.on_create(|i| {
        i.item_id = 0xF08;
        i.potion_effect = "AgilityGreater";
    });
});

item("StrengthPotion", |c| {
    c.on_create(|i| {
        i.item_id = 0xF09;
        i.potion_effect = "Strength";
    });
});

item("GreaterStrengthPotion", |c| {
    c.on_create(|i| {
        i.item_id = 0xF09;
        i.potion_effect = "StrengthGreater";
    });
});

item("LesserPoisonPotion", |c| {
    c.on_create(|i| {
        i.item_id = 0xF0A;
        i.potion_effect = "PoisonLesser";
    });
});

item("PoisonPotion", |c| {
    c.on_create(|i| {
        i.item_id = 0xF0A;
        i.potion_effect = "Poison";
    });
});

item("GreaterPoisonPotion", |c| {
    c.on_create(|i| {
        i.item_id = 0xF0A;
        i.potion_effect = "PoisonGreater";
    });
});

item("DeadlyPoisonPotion", |c| {
    c.on_create(|i| {
        i.item_id = 0xF0A;
        i.potion_effect = "PoisonDeadly";
    });
});

item("DarkglowPotion", |c| {
    c.on_create(|i| {
        i.item_id = 0xF0A;
        i.potion_effect = "Darkglow";
        i.hue = 0x96;
    });
});

item("ParasiticPotion", |c| {
    c.on_create(|i| {
        i.item_id = 0xF0A;
        i.potion_effect = "Parasitic";
        i.hue = 0x17C;
    });
});

item("LesserExplosionPotion", |c| {
    c.on_create(|i| {
        i.item_id = 0xF0D;
        i.potion_effect = "ExplosionLesser";
    });
});

item("ExplosionPotion", |c| {
    c.on_create(|i| {
        i.item_id = 0xF0D;
        i.potion_effect = "Explosion";
    });
});

item("GreaterExplosionPotion", |c| {
    c.on_create(|i| {
        i.item_id = 0xF0D;
        i.potion_effect = "ExplosionGreater";
    });
});

item("ConflagrationPotion", |c| {
    c.on_create(|i| {
        i.item_id = 0xF06;
        i.potion_effect = "Conflagration";
        i.hue = 0x489;
    });
});

item("GreaterConflagrationPotion", |c| {
    c.on_create(|i| {
        i.item_id = 0xF06;
        i.potion_effect = "ConflagrationGreater";
        i.hue = 0x489;
    });
});

item("ConfusionBlastPotion", |c| {
    c.on_create(|i| {
        i.item_id = 0xF06;
        i.potion_effect = "ConfusionBlast";
        i.hue = 0x48D;
    });
});

item("GreaterConfusionBlastPotion", |c| {
    c.on_create(|i| {
        i.item_id = 0xF06;
        i.potion_effect = "ConfusionBlastGreater";
        i.hue = 0x48D;
    });
});
