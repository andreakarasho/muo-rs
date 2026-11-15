item("PlagueBeastInnard", |c| {
    c.weight = 1.0;
    c.name = "plague beast innards";
});

item("PlagueBeastBackpack", |c| {
    c.max_weight = 0;
    c.max_items = 0;
    c.gump_id = 10851;
    c.drop_sound = 575;
});

item("PlagueBeastMutationCore", |c| {
    c.weight = 1.0;
    c.label = 1153760;
    c.on_create(|i| {
        i.item_id = 0x1CF0;
    });
});

item("PlagueBeastBackupOrgan", |c| {
    c.organ_components = [
        [6939, 66, 16, 39],
        [6939, 66, 39, 49],
        [6939, 66, 39, 48],
        [6939, 66, 44, 42],
        [7410, 66, 20, 34],
        [4959, 66, 47, 58],
        [4960, 66, 70, 68],
    ];
});

item("PlagueBeastMainOrgan", |c| {
    c.organ_components = [
        [6939, 66, -36, -2],
        [8115, 66, -42, 0],
        [2527, 66, -53, -7],
        [6940, 84, 29, 9],
        [7430, 84, 18, -2],
        [2527, 84, 36, -1],
        [7440, 43, -36, 47],
        [6940, 43, -24, 62],
        [2527, 43, -41, 74],
        [6939, 96, 39, 56],
        [8116, 96, 34, 52],
        [2527, 96, 45, 71],
        [4945, 21, 23, 0],
        [4943, 21, -22, 0],
        [4944, 21, 0, 0],
    ];
});
