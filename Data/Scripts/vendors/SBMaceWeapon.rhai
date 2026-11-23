shop("SBMaceWeapon", |s| {
    s.buys("Club", 8);
    s.buys("HammerPick", 13);
    s.buys("Mace", 14);
    s.buys("Maul", 10);
    s.buys("WarHammer", 12);
    s.buys("WarMace", 15);
    s.sells("HammerPick", 26, 20, 0x143D, #{ hue: 0 });
    s.sells("Club", 16, 20, 0x13B4, #{ hue: 0 });
    s.sells("Mace", 28, 20, 0x0F5C, #{ hue: 0 });
    s.sells("Maul", 21, 20, 0x143B, #{ hue: 0 });
    s.sells("WarHammer", 25, 20, 0x1439, #{ hue: 0 });
    s.sells("WarMace", 31, 20, 0x1407, #{ hue: 0 });
});
