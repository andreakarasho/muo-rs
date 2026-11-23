shop("SBPoleArmWeapon", |s| {
    s.buys("Bardiche", 30);
    s.buys("Halberd", 21);
    s.sells("Bardiche", 60, 20, 0x0F4D, #{ hue: 0 });
    s.sells("Halberd", 42, 20, 0x143E, #{ hue: 0 });
});
