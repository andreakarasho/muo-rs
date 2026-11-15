shop("SBHealer", |s| {
    s.buys("Bandage", 1);
    s.buys("LesserHealPotion", 7);
    s.buys("RefreshPotion", 7);
    s.buys("Garlic", 2);
    s.buys("Ginseng", 2);
    s.sells("Bandage", 5, 20, 0x0E21, #{ hue: 0 });
    s.sells("LesserHealPotion", 15, 20, 0x0F0C, #{ hue: 0 });
    s.sells("Ginseng", 3, 20, 0x0F85, #{ hue: 0 });
    s.sells("Garlic", 3, 20, 0x0F84, #{ hue: 0 });
    s.sells("RefreshPotion", 15, 20, 0x0F0B, #{ hue: 0 });
});
