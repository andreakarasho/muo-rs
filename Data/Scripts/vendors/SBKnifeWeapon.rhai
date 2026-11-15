shop("SBKnifeWeapon", |s| {
    s.buys("ButcherKnife", 7);
    s.buys("Cleaver", 7);
    s.buys("Dagger", 10);
    s.buys("SkinningKnife", 7);
    s.sells("ButcherKnife", 14, 20, 0x13F6, #{ hue: 0 });
    s.sells("Dagger", 21, 20, 0x0F52, #{ hue: 0 });
    s.sells("Cleaver", 15, 20, 0x0EC3, #{ hue: 0 });
    s.sells("SkinningKnife", 14, 20, 0x0EC4, #{ hue: 0 });
});
