shop("SBFisherman", |s| {
    s.buys("RawFishSteak", 1);
    s.buys("Fish", 1);
    s.buys("FishingPole", 7);
    s.sells("RawFishSteak", 3, 20, 0x097A, #{ hue: 0 });
    s.sells("Fish", 6, 80, 0x09CC, #{ hue: 0 });
    s.sells("Fish", 6, 80, 0x09CD, #{ hue: 0 });
    s.sells("Fish", 6, 80, 0x09CE, #{ hue: 0 });
    s.sells("Fish", 6, 80, 0x09CF, #{ hue: 0 });
    s.sells("FishingPole", 15, 20, 0x0DC0, #{ hue: 0 });
    s.sells("AquariumFishNet", 250, 20, 0x0DC8, #{ hue: 576 });
    s.sells("AquariumFood", 62, 20, 0x0EFC, #{ hue: 0 });
    s.sells("FishBowl", 6312, 20, 0x241C, #{ hue: 1154 });
    s.sells("VacationWafer", 67, 20, 0x0971, #{ hue: 0 });
    s.sells("AquariumNorthDeed", 250002, 20, 0x14F0, #{ hue: 0 });
    s.sells("AquariumEastDeed", 250002, 20, 0x14F0, #{ hue: 0 });
    s.sells("NewAquariumBook", 15, 20, 0x0FF2, #{ hue: 0 });
});
