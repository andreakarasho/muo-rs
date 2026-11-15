shop("SBBard", |s| {
    s.buys("LapHarp", 10);
    s.buys("Lute", 10);
    s.buys("Drums", 10);
    s.buys("Harp", 10);
    s.buys("Tambourine", 10);
    s.sells("Drums", 21, 10, 0x0E9C, #{ hue: 0 });
    s.sells("Tambourine", 21, 10, 0x0E9E, #{ hue: 0 });
    s.sells("LapHarp", 21, 10, 0x0EB2, #{ hue: 0 });
    s.sells("Lute", 21, 10, 0x0EB3, #{ hue: 0 });
});
