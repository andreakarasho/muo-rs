item("DupresShield", |c| {
    c.base_physical_resistance = 1;
    c.base_fire_resistance = 0;
    c.base_cold_resistance = 0;
    c.base_poison_resistance = 0;
    c.base_energy_resistance = 1;
    c.init_min_hits = 255;
    c.init_max_hits = 255;
    c.aos_str_req = 50;
    c.armor_base = 15;
    c.label = 1075196;
    c.weight = 6.0;
    c.graphic = 11009;
    c.on_create(|i| {
        i.loot_type = "Blessed";
        i.attributes.bonus_hits = 5;
        i.attributes.regen_hits = 1;
        i.skill_bonuses = [["Parry", 5.0]];
    });
});

item("QuiverOfInfinity", |c| {
    c.weight = 8.0;
    c.label = 1075201;
    c.graphic = 11010;
    c.on_create(|i| {
        i.loot_type = "Blessed";
        i.quiver_weight_reduction = 30;
        i.quiver_lower_ammo_cost = 20;
        i.attributes.defend_chance = 5;
    });
});

item("BaseFormTalisman", |c| {
    c.weight = 1.0;
    c.graphic = 12121;
    c.on_create(|i| {
        i.loot_type = "Blessed";
        i.layer = "Talisman";
    });
});

item("FerretFormTalisman", |c| {
    c.talisman_form = 1031672;
});

item("SquirrelFormTalisman", |c| {
    c.talisman_form = 1031671;
});

item("CuSidheFormTalisman", |c| {
    c.talisman_form = 1031670;
});

item("ReptalonFormTalisman", |c| {
    c.talisman_form = 1075202;
});
