shop("SBCobbler", |s| {
    s.buys("Shoes", 4);
    s.buys("Boots", 5);
    s.buys("ThighBoots", 7);
    s.buys("Sandals", 2);
    s.sells("ThighBoots", 15, 20, 0x1711, #{ hue: "randomNeutral" });
    s.sells("Shoes", 8, 20, 0x170F, #{ hue: "randomNeutral" });
    s.sells("Boots", 10, 20, 0x170B, #{ hue: "randomNeutral" });
    s.sells("Sandals", 5, 20, 0x170D, #{ hue: "randomNeutral" });
});
