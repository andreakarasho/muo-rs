shop("SBSEFood", |s| {
    s.buys("Wasabi", 1);
    s.buys("BentoBox", 3);
    s.buys("GreenTeaBasket", 1);
    s.sells("Wasabi", 2, 20, 0x24E8, #{ hue: 0 });
    s.sells("Wasabi", 2, 20, 0x24E9, #{ hue: 0 });
    s.sells("BentoBox", 6, 20, 0x2836, #{ hue: 0 });
    s.sells("BentoBox", 6, 20, 0x2837, #{ hue: 0 });
    s.sells("GreenTeaBasket", 2, 20, 0x284B, #{ hue: 0 });
});
