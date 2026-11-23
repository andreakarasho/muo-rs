item("ClumsyWand", |c| {
    c.wand_effect = "Clumsiness";
    c.min_charges = 5;
    c.max_charges = 30;
    c.wand_spell = "Server.Spells.First.ClumsySpell";
});

item("FeebleWand", |c| {
    c.wand_effect = "Feeblemindedness";
    c.min_charges = 5;
    c.max_charges = 30;
    c.wand_spell = "Server.Spells.First.FeeblemindSpell";
});

item("FireballWand", |c| {
    c.wand_effect = "Fireball";
    c.min_charges = 5;
    c.max_charges = [15, 109];
    c.wand_spell = "Server.Spells.Third.FireballSpell";
});

item("GreaterHealWand", |c| {
    c.wand_effect = "GreaterHealing";
    c.min_charges = 1;
    c.max_charges = [5, 109];
    c.wand_spell = "Server.Spells.Fourth.GreaterHealSpell";
});

item("HarmWand", |c| {
    c.wand_effect = "Harming";
    c.min_charges = 5;
    c.max_charges = [30, 109];
    c.wand_spell = "Server.Spells.Second.HarmSpell";
});

item("HealWand", |c| {
    c.wand_effect = "Healing";
    c.min_charges = 10;
    c.max_charges = [25, 109];
    c.wand_spell = "Server.Spells.First.HealSpell";
});

item("IDWand", |c| {
    c.wand_effect = "Identification";
    c.min_charges = 25;
    c.max_charges = 175;
});

item("LightningWand", |c| {
    c.wand_effect = "Lightning";
    c.min_charges = 5;
    c.max_charges = [20, 109];
    c.wand_spell = "Server.Spells.Fourth.LightningSpell";
});

item("MagicArrowWand", |c| {
    c.wand_effect = "MagicArrow";
    c.min_charges = 5;
    c.max_charges = [30, 109];
    c.wand_spell = "Server.Spells.First.MagicArrowSpell";
});

item("ManaDrainWand", |c| {
    c.wand_effect = "ManaDraining";
    c.min_charges = 5;
    c.max_charges = 30;
    c.wand_spell = "Server.Spells.Fourth.ManaDrainSpell";
});

item("WeaknessWand", |c| {
    c.wand_effect = "Weakness";
    c.min_charges = 5;
    c.max_charges = 30;
    c.wand_spell = "Server.Spells.First.WeakenSpell";
});

item("Server.Items.BaseWand", |c| {
    c.weight = 1.0;
    c.primary_ability = "Dismount";
    c.secondary_ability = "Disarm";
    c.aos_str_req = 5;
    c.aos_min_damage = 9;
    c.aos_max_damage = 11;
    c.aos_speed = 40;
    c.ml_speed = 2.75;
    c.old_str_req = 0;
    c.old_min_damage = 2;
    c.old_max_damage = 6;
    c.old_speed = 35;
    c.init_min_hits = 31;
    c.init_max_hits = 110;
});
