shop("SBSECarpenter", |s| {
    s.buys("Tetsubo", 21);
    s.buys("Fukiya", 10);
    s.buys("BambooFlute", 10);
    s.buys("Bokuto", 10);
    s.sells("Bokuto", 21, 20, 0x27A8, #{ hue: 0 });
    s.sells("Tetsubo", 43, 20, 0x27A6, #{ hue: 0 });
    s.sells("Fukiya", 20, 20, 0x27AA, #{ hue: 0 });
    s.sells("BambooFlute", 21, 20, 0x2805, #{ hue: 0 });
    s.sells("BambooFlute", 21, 20, 0x2805, #{ hue: 0 });
});
