shop("SBSpearForkWeapon", |s| {
    s.buys("Spear", 15);
    s.buys("Pitchfork", 9);
    s.buys("ShortSpear", 11);
    s.sells("Pitchfork", 19, 20, 0x0E87, #{ hue: 0 });
    s.sells("ShortSpear", 23, 20, 0x1403, #{ hue: 0 });
    s.sells("Spear", 31, 20, 0x0F62, #{ hue: 0 });
});
