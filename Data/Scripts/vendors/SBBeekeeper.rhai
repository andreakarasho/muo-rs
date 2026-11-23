shop("SBBeekeeper", |s| {
    s.buys("JarHoney", 1);
    s.buys("Beeswax", 1);
    s.sells("JarHoney", 3, 20, 0x09EC, #{ hue: 0 });
    s.sells("Beeswax", 2, 20, 0x1422, #{ hue: 0 });
});
