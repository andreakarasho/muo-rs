shop("SBMiner", |s| {
    s.buys("Pickaxe", 12);
    s.buys("Shovel", 6);
    s.buys("Lantern", 1);
    s.buys("Torch", 3);
    s.buys("Bag", 3);
    s.buys("Candle", 3);
    s.sells("Bag", 6, 20, 0x0E76, #{ hue: 0 });
    s.sells("Candle", 6, 10, 0x0A28, #{ hue: 0 });
    s.sells("Torch", 8, 10, 0x0F6B, #{ hue: 0 });
    s.sells("Lantern", 2, 10, 0x0A25, #{ hue: 0 });
    s.sells("Pickaxe", 25, 10, 0x0E86, #{ hue: 0 });
    s.sells("Shovel", 12, 10, 0x0F39, #{ hue: 0 });
});
