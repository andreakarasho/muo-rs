shop("SBAxeWeapon", |s| {
    s.buys("BattleAxe", 13);
    s.buys("DoubleAxe", 26);
    s.buys("ExecutionersAxe", 15);
    s.buys("LargeBattleAxe", 16);
    s.buys("Pickaxe", 11);
    s.buys("TwoHandedAxe", 16);
    s.buys("WarAxe", 14);
    s.buys("Axe", 20);
    s.sells("ExecutionersAxe", 30, 20, 0x0F45, #{ hue: 0 });
    s.sells("BattleAxe", 26, 20, 0x0F47, #{ hue: 0 });
    s.sells("TwoHandedAxe", 32, 20, 0x1443, #{ hue: 0 });
    s.sells("Axe", 40, 20, 0x0F49, #{ hue: 0 });
    s.sells("DoubleAxe", 52, 20, 0x0F4B, #{ hue: 0 });
    s.sells("Pickaxe", 22, 20, 0x0E86, #{ hue: 0 });
    s.sells("LargeBattleAxe", 33, 20, 0x13FB, #{ hue: 0 });
    s.sells("WarAxe", 29, 20, 0x13B0, #{ hue: 0 });
});
