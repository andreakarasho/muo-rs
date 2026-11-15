shop("SBStavesWeapon", |s| {
    s.buys("BlackStaff", 11);
    s.buys("GnarledStaff", 8);
    s.buys("QuarterStaff", 9);
    s.buys("ShepherdsCrook", 10);
    s.sells("BlackStaff", 22, 20, 0x0DF1, #{ hue: 0 });
    s.sells("GnarledStaff", 16, 20, 0x13F8, #{ hue: 0 });
    s.sells("QuarterStaff", 19, 20, 0x0E89, #{ hue: 0 });
    s.sells("ShepherdsCrook", 20, 20, 0x0E81, #{ hue: 0 });
});
