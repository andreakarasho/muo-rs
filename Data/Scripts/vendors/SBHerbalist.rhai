shop("SBHerbalist", |s| {
    s.buys("Bloodmoss", 3);
    s.buys("MandrakeRoot", 2);
    s.buys("Garlic", 2);
    s.buys("Ginseng", 2);
    s.buys("Nightshade", 2);
    s.buys("Bottle", 3);
    s.buys("MortarPestle", 4);
    s.sells("Ginseng", 3, 20, 0x0F85, #{ hue: 0 });
    s.sells("Garlic", 3, 20, 0x0F84, #{ hue: 0 });
    s.sells("MandrakeRoot", 3, 20, 0x0F86, #{ hue: 0 });
    s.sells("Nightshade", 3, 20, 0x0F88, #{ hue: 0 });
    s.sells("Bloodmoss", 5, 20, 0x0F7B, #{ hue: 0 });
    s.sells("MortarPestle", 8, 20, 0x0E9B, #{ hue: 0 });
    s.sells("Bottle", 5, 20, 0x0F0E, #{ hue: 0 });
});
