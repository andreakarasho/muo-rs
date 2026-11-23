shop("SBRangedWeapon", |s| {
    s.buys("Bolt", 1);
    s.buys("Arrow", 1);
    s.buys("Shaft", 1);
    s.buys("Feather", 1);
    s.buys("HeavyCrossbow", 27);
    s.buys("Bow", 17);
    s.buys("Crossbow", 25);
    s.era("AOS", |s| {
        s.buys("CompositeBow", 23);
        s.buys("RepeatingCrossbow", 22);
    });
    s.sells("Crossbow", 55, 20, 0x0F50, #{ hue: 0 });
    s.sells("HeavyCrossbow", 55, 20, 0x13FD, #{ hue: 0 });
    s.era("AOS", |s| {
        s.sells("RepeatingCrossbow", 46, 20, 0x26C3, #{ hue: 0 });
        s.sells("CompositeBow", 45, 20, 0x26C2, #{ hue: 0 });
    });
    s.sells("Bolt", 3, #{ from: 30, count: 60 }, 0x1BFB, #{ hue: 0 });
    s.sells("Bow", 40, 20, 0x13B2, #{ hue: 0 });
    s.sells("Arrow", 2, #{ from: 30, count: 60 }, 0x0F3F, #{ hue: 0 });
    s.sells("Feather", 2, #{ from: 30, count: 60 }, 0x1BD1, #{ hue: 0 });
    s.sells("Shaft", 3, #{ from: 30, count: 60 }, 0x1BD4, #{ hue: 0 });
});
