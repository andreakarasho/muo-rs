shop("SBSEHats", |s| {
    s.buys("Kasa", 15);
    s.buys("LeatherJingasa", 5);
    s.buys("ClothNinjaHood", 16);
    s.sells("Kasa", 31, 20, 0x2798, #{ hue: 0 });
    s.sells("LeatherJingasa", 11, 20, 0x2776, #{ hue: 0 });
    s.sells("ClothNinjaHood", 33, 20, 0x278F, #{ hue: 0 });
});
