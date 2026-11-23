item("AncientWildStaff", |c| {
    c.label = 1073550;
    c.on_create(|i| {
        i.weapon_attributes.resist_poison_bonus = 5;
    });
});

item("ArcanistsWildStaff", |c| {
    c.label = 1073549;
    c.on_create(|i| {
        i.attributes.bonus_mana = 3;
        i.attributes.weapon_damage = 3;
    });
});

item("HardenedWildStaff", |c| {
    c.label = 1073552;
    c.on_create(|i| {
        i.attributes.weapon_damage = 5;
    });
});

item("ThornedWildStaff", |c| {
    c.label = 1073551;
    c.on_create(|i| {
        i.attributes.reflect_physical = 12;
    });
});
