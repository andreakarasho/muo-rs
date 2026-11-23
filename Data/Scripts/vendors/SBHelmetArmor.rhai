shop("SBHelmetArmor", |s| {
    s.buys("Bascinet", 9);
    s.buys("CloseHelm", 9);
    s.buys("Helmet", 9);
    s.buys("NorseHelm", 9);
    s.buys("PlateHelm", 10);
    s.sells("PlateHelm", 21, 20, 0x1412, #{ hue: 0 });
    s.sells("CloseHelm", 18, 20, 0x1408, #{ hue: 0 });
    s.sells("CloseHelm", 18, 20, 0x1409, #{ hue: 0 });
    s.sells("Helmet", 31, 20, 0x140A, #{ hue: 0 });
    s.sells("Helmet", 18, 20, 0x140B, #{ hue: 0 });
    s.sells("NorseHelm", 18, 20, 0x140E, #{ hue: 0 });
    s.sells("NorseHelm", 18, 20, 0x140F, #{ hue: 0 });
    s.sells("Bascinet", 18, 20, 0x140C, #{ hue: 0 });
    s.sells("PlateHelm", 21, 20, 0x1419, #{ hue: 0 });
});
