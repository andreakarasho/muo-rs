shop("SBLeatherWorker", |s| {
    s.buys("Hides", 2);
    s.buys("ThighBoots", 7);
    s.sells("Hides", 4, 999, 0x1078, #{ hue: 0 });
    s.sells("ThighBoots", 56, 10, 0x1711, #{ hue: 0 });
});
