shop("SBThief", |s| {
    s.buys("Backpack", 7);
    s.buys("Pouch", 3);
    s.buys("Torch", 3);
    s.buys("Lantern", 1);
    s.buys("Lockpick", 6);
    s.buys("WoodenBox", 7);
    s.buys("HairDye", 19);
    s.sells("Backpack", 15, 20, 0x09B2, #{ hue: 0 });
    s.sells("Pouch", 6, 20, 0x0E79, #{ hue: 0 });
    s.sells("Torch", 8, 20, 0x0F6B, #{ hue: 0 });
    s.sells("Lantern", 2, 20, 0x0A25, #{ hue: 0 });
    s.sells("Lockpick", 12, 20, 0x14FC, #{ hue: 0 });
    s.sells("WoodenBox", 14, 20, 0x09AA, #{ hue: 0 });
    s.sells("Key", 2, 20, 0x100E, #{ hue: 0 });
    s.sells("HairDye", 37, 20, 0x0EFF, #{ hue: 0 });
});
